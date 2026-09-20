//! Annotated assembly for `.WSC` scripts: the reader-facing text form of a decoded script, the
//! parser that reads it back, and the span-carrying parse result a language server can be built on
//! without re-parsing.
//!
//! The grammar is normative in `assembly/README.md`; this module implements it and nothing else
//! restates it. The binary side is [`crate::opcodes`]: mnemonics and operand names come from
//! `OPCODE_SPECS`, and `Script::binary_serialise` is the inverse of the jump rendering here.

use anyhow::anyhow;

use crate::opcodes::{manifest_for, Script};

pub mod parse;
pub mod print;

mod grammar;

pub use parse::{parse_document, parse_line, LineKind, LineParse};
pub use print::{print_document, print_script};

/// How a [`Diagnostic`] is rendered: `error` refuses assembly, `warning` is a finding.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
	Error,
	Flag,
}

impl Severity {
	pub fn as_str(&self) -> &'static str {
		match self {
			Severity::Error => "error",
			Severity::Flag => "warning",
		}
	}
}

/// A positioned message. `line` is 1-based; `column` is 1-based in Unicode scalar values, which is
/// the only conversion an editor needs (an LSP server converts it to UTF-16 code units).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Diagnostic {
	pub severity: Severity,
	pub line: usize,
	pub column: usize,
	pub message: String,
}

impl Diagnostic {
	/// Renders one diagnostic as `{path}:{line}:{column}: {error|warning}: {message}`.
	pub fn render(&self, path: &str) -> String {
		format!(
			"{path}:{}:{}: {}: {}",
			self.line,
			self.column,
			self.severity.as_str(),
			self.message
		)
	}
}

/// One instruction as it appears in the text, for tooling: its line, derived `address`, the opcode
/// byte it resolved to, whether it carried an `addr` annotation (`annotated == false` means the line
/// was inserted by hand), its label if any, and the `(label, line, column)` of every operand token.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AsmItem {
	pub line: usize,
	pub address: usize,
	pub opcode: u8,
	pub annotated: bool,
	pub label: Option<String>,
	pub operands: Vec<(String, usize, usize)>,
}

/// A `;` comment: its text without the `;`, the line it sits on, and the instruction it is attached
/// to — the next instruction line for a whole-line comment, the same line's instruction for a
/// trailing one. `Before(items.len())` is a comment after the last instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Comment {
	pub line: usize,
	pub text: String,
	pub anchor: CommentAnchor,
}

/// Where a [`Comment`] sits relative to the instruction list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommentAnchor {
	Before(usize),
	Trailing(usize),
}

/// Parse and assembly counts, for the summary line of `ccfkb_asm_check`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Summary {
	pub instructions: usize,
	pub annotated: usize,
	pub inserted: usize,
	pub errors: usize,
	pub findings: usize,
}

/// The result of parsing an `.asm` file: the script it describes, the per-instruction spans, and
/// every problem found. Parsing never fails, so an editor can be served on partial input.
#[derive(Clone, Debug)]
pub struct AsmDocument {
	pub script: Script,
	pub items: Vec<AsmItem>,
	pub labels: std::collections::BTreeMap<String, usize>,
	pub comments: Vec<Comment>,
	pub diagnostics: Vec<Diagnostic>,
}

impl AsmDocument {
	pub fn has_errors(&self) -> bool {
		self.diagnostics.iter().any(|it| it.severity == Severity::Error)
	}

	pub fn summary(&self) -> Summary {
		Summary {
			instructions: self.items.len(),
			annotated: self.items.iter().filter(|it| it.annotated).count(),
			inserted: self.items.iter().filter(|it| !it.annotated).count(),
			errors: self
				.diagnostics
				.iter()
				.filter(|it| it.severity == Severity::Error)
				.count(),
			findings: self
				.diagnostics
				.iter()
				.filter(|it| it.severity == Severity::Flag)
				.count(),
		}
	}

	/// The size of the script this document describes, in bytes: every instruction plus the trailer.
	pub fn byte_len(&self) -> usize {
		let instructions: usize = self
			.script
			.opcodes
			.iter()
			.map(|it| 1 + it.fields.iter().map(|field| field.size()).sum::<usize>())
			.sum();
		instructions + self.script.trailer.len()
	}

	/// The script to encode: `Err` (the first error, rendered with its position) when the file has an
	/// error, else the parsed script. The manifest, every operand and every `translation` annotation
	/// are already in place — see `parse::parse_document`. Comments are not part of `Script` and are
	/// dropped here; `print_document` is what preserves them.
	pub fn into_script(self) -> anyhow::Result<Script> {
		if let Some(first) = self.diagnostics.iter().find(|it| it.severity == Severity::Error) {
			return Err(anyhow!(
				"{}:{}: {}",
				first.line,
				first.column,
				first.message
			));
		}
		Ok(self.script)
	}
}

/// The operand labels one asm line spells for a table row: the row's names in order, with the *k*-th
/// occurrence of a repeated name suffixed `_<k>` (`pad`, `pad_2`, …). Five rows repeat a name.
pub fn operand_labels(operands: &[&'static str]) -> Vec<String> {
	let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
	let mut out = Vec::with_capacity(operands.len());
	for name in operands {
		let count = seen.entry(name).or_insert(0);
		*count += 1;
		if operands.iter().filter(|it| **it == *name).count() > 1 {
			out.push(format!("{}_{}", name, count));
		} else {
			out.push((*name).to_owned());
		}
	}
	out
}

/// Escapes a model string for the `"…"` form: `\\`, `\"`, `\n`, `\r`, `\t`, and `\xNN` for every
/// other code point below 0x20 or equal to 0x7F. Everything else is verbatim UTF-8.
pub(crate) fn escape_string(input: &str) -> String {
	let mut out = String::with_capacity(input.len());
	for c in input.chars() {
		match c {
			'\\' => out.push_str("\\\\"),
			'"' => out.push_str("\\\""),
			'\n' => out.push_str("\\n"),
			'\r' => out.push_str("\\r"),
			'\t' => out.push_str("\\t"),
			c if (c as u32) < 0x20 || c == '\u{7F}' => out.push_str(&format!("\\x{:02X}", c as u32)),
			c => out.push(c),
		}
	}
	out
}

/// Fills `script.opcode_table` from the opcodes it holds. Kept here so the parser and the tests
/// agree on the one way a manifest is produced.
pub(crate) fn finish_manifest(script: &mut Script) {
	script.opcode_table = manifest_for(&script.opcodes);
}

/// True when the field is the jump destination of its opcode (`0x06` `target`, `0x01` `offset`).
pub(crate) fn is_jump_field(opcode: u8, index: usize) -> bool {
	matches!((opcode, index), (0x06, 0) | (0x01, 3))
}

/// The destination address a jump instruction points at, given its declared `address`. This is the
/// inverse of what `Script::binary_serialise` writes (`adjust_single_opcode`).
pub(crate) fn jump_destination(opcode: u8, address: usize, field: u32) -> Option<usize> {
	match opcode {
		0x06 => Some(field as usize),
		0x01 => Some(address.wrapping_add(11).wrapping_add(field as usize)),
		_ => None,
	}
}

/// The field value that makes a jump instruction land on `destination`, given its `address`.
pub(crate) fn jump_field(opcode: u8, address: usize, destination: usize) -> u32 {
	match opcode {
		0x06 => destination as u32,
		_ => destination.wrapping_sub(address.wrapping_add(11)) as u32,
	}
}


#[cfg(test)]
mod test {
	use super::*;
	use crate::opcodes::{Choice, OpField, Opcode, Script, TLString};
	use camino::Utf8Path;

	const NAME: &str = "FIXTURE.WSC";

	fn string(raw: &str) -> OpField {
		OpField::String(TLString { raw: raw.to_owned(), translation: None, notes: None })
	}

	/// Addresses are derived the way the printer derives them, and the conditional jump's offset is
	/// the one the engine expects, so the fixture is self-consistent.
	fn script_with(mut opcodes: Vec<Opcode>, trailer: Vec<u8>, jump_to_end: bool) -> Script {
		let mut address = 0usize;
		for opcode in opcodes.iter_mut() {
			opcode.address = address;
			address += 1 + opcode.fields.iter().map(|it| it.size()).sum::<usize>();
		}
		if jump_to_end {
			let target = opcodes.last().map(|it| it.address).unwrap_or_default();
			let at = opcodes
				.iter()
				.position(|it| it.opcode == 0x01)
				.expect("fixture has a conditional jump");
			opcodes[at].fields[3] = OpField::DWord(jump_target(&opcodes[at], target));
		}
		Script { opcode_table: vec![], opcodes, trailer }
	}

	fn jump_target(opcode: &Opcode, destination: usize) -> u32 {
		jump_field(opcode.opcode, opcode.address, destination)
	}

	fn choice(arg1: u16, text: &str, marker: u8) -> Choice {
		Choice {
			arg1,
			choice_str: TLString { raw: text.to_owned(), translation: None, notes: None },
			trailer: vec![marker; 11],
		}
	}

	#[test]
	fn print_parse_roundtrip_is_idempotent() {
		let script = script_with(
			vec![
				Opcode {
					opcode: 0x41,
					address: 0,
					actual_address: 0,
					fields: vec![
						OpField::Word(0x0114),
						OpField::Byte(0x01),
						OpField::Byte(0x00),
						string("  a\\b  "),
					],
				},
				Opcode {
					opcode: 0xB4,
					address: 0,
					actual_address: 0,
					fields: vec![
						OpField::Padding(vec![0xAA, 0xBB]),
						OpField::Word(0x0001),
						OpField::Word(0x0002),
						OpField::DWord(0x00000003),
						OpField::Byte(0x04),
						OpField::Padding(vec![0x00]),
					],
				},
				Opcode {
					opcode: 0x02,
					address: 0,
					actual_address: 0,
					fields: vec![
						OpField::Byte(0x02),
						OpField::Padding(vec![0x00]),
						OpField::Choice(vec![choice(0x0102, "はい", 0x01), choice(0x0103, "いいえ", 0x02)]),
					],
				},
				Opcode {
					opcode: 0x01,
					address: 0,
					actual_address: 0,
					fields: vec![
						OpField::Byte(0x03),
						OpField::Word(0x03E3),
						OpField::Word(0x0001),
						OpField::DWord(0),
						OpField::Padding(vec![0x00]),
					],
				},
				Opcode { opcode: 0xFF, address: 0, actual_address: 0, fields: vec![] },
			],
			vec![0x43, 0x00, 0x00],
			true,
		);

		let text = print_script(&script, NAME).unwrap();
		assert!(text.contains("# yields"), "the conditional-branch row yields:\n{text}");
		assert!(text.contains("# label L_"), "the jump target is labelled:\n{text}");
		let doc = parse_document(&text);
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		assert_eq!(print_document(&doc, NAME).unwrap(), text, "print → parse → print");
		let back = doc.clone().into_script().unwrap();
		assert_eq!(
			back.clone().binary_serialise().unwrap(),
			script.clone().binary_serialise().unwrap(),
			"script → asm → script must preserve the bytes"
		);
		assert_eq!(back.trailer, script.trailer);
	}

	#[test]
	fn comments_and_translations_survive_the_round_trip() {
		let fixture = "# cc-fkb asm 1\n# script T.WSC\n\n\
; what the original script sets before the title card\n\
# addr 0x00000000\n\
# translation \"Oldest memory.\"\n\
textbox_no_speaker layout_id: 0x0114, mode: 0x01, timer_param: 0x00, text: \"a; \\\"b\\\"\"   ; the wide box\n\
\n\
# addr 0x00000014\n\
end_of_script\n\
\n\
.trailer [ 0x43 ]\n";
		let doc = parse_document(fixture);
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		let script = doc.clone().into_script().unwrap();
		let OpField::String(first) = &script.opcodes[0].fields[3] else { panic!("string operand") };
		assert_eq!(first.raw, "a; \"b\"", "a `;` inside a string literal is data");
		assert_eq!(first.translation.as_deref(), Some("Oldest memory."));

		// The translation is what gets encoded, so it is what the address derivation counts: the
		// 14-byte "Oldest memory." (plus its NUL) puts the next instruction at 0x14.
		assert_eq!(doc.items[1].address, 0x14);
		let text = print_document(&doc, "T.WSC").unwrap();
		assert!(text.contains("; what the original script sets before the title card"), "{text}");
		assert!(text.contains("; the wide box"), "a trailing comment keeps its anchor:\n{text}");
		assert!(text.contains("# translation \"Oldest memory.\""), "{text}");
		let again = parse_document(&text);
		assert!(again.diagnostics.is_empty(), "{:#?}", again.diagnostics);
		assert_eq!(print_document(&again, "T.WSC").unwrap(), text, "comments survive fmt");

		// `translation` is positional: an empty annotation reaches the second string operand of a
		// two-string opcode without translating the first, and the position survives printing.
		let two = "# cc-fkb asm 1\n# script T.WSC\n\n\
# addr 0x00000000\n\
# translation \"\"\n\
# translation \"Second line\"\n\
textbox_with_speaker layout_id: 0x0114, mode: 0x01, speaker_arg: 0x02, timer_param: 0x00, speaker_text: \"first\", text: \"second\"\n\
\n\
.trailer [ ]\n";
		let doc = parse_document(two);
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		let script = doc.clone().into_script().unwrap();
		let OpField::String(first) = &script.opcodes[0].fields[4] else { panic!("speaker_text") };
		let OpField::String(second) = &script.opcodes[0].fields[5] else { panic!("text") };
		assert_eq!(first.translation, None);
		assert_eq!(second.translation.as_deref(), Some("Second line"));
		let text = print_document(&doc, "T.WSC").unwrap();
		assert!(text.contains("# translation \"\"\n# translation \"Second line\""), "{text}");
		assert_eq!(
			print_document(&parse_document(&text), "T.WSC").unwrap(),
			text,
			"positional translations round-trip"
		);
	}

	#[test]
	fn annotations_are_reported_not_fatal() {
		let fixture = "# cc-fkb asm 1\n# script T.WSC\n\n\
# addr 0x00000009\n\
textbox_state_preset preset: 0x0114, pad: 0x00\n\
\n\
# yields\n\
# translation \"nothing to translate\"\n\
nop pad: [ 0x00, 0x00 ]\n\
\n\
# colour blue\n\
end_of_script\n";
		let doc = parse_document(fixture);
		assert!(!doc.has_errors(), "{:#?}", doc.diagnostics);
		let lines: Vec<(usize, Severity, String)> = doc
			.diagnostics
			.iter()
			.map(|it| (it.line, it.severity, it.message.clone()))
			.collect();
		assert_eq!(lines.len(), 4, "{lines:#?}");
		assert_eq!(lines[0].0, 4);
		assert!(lines[0].2.starts_with("stale address annotation: says 0x00000009"), "{:?}", lines[0]);
		assert_eq!(lines[1].0, 7);
		assert!(lines[1].2.contains("\"yields\" annotation on nop (0x81)"), "{:?}", lines[1]);
		assert_eq!(lines[2].0, 8);
		assert!(lines[2].2.contains("\"translation\" annotation on nop (0x81)"), "{:?}", lines[2]);
		assert_eq!(lines[3].0, 11);
		assert_eq!(lines[3].2, "unknown annotation key \"colour\"");
		assert!(lines.iter().all(|it| it.1 == Severity::Flag), "{lines:#?}");
		let script = doc.into_script().expect("findings alone never refuse a file");
		assert_eq!(script.opcodes.len(), 3);
	}

	#[test]
	fn errors_carry_line_and_column() {
		let bad_label =
			"variable_heap_op kind: 0x01, var_index: 0x0429, wrong: 0x00, value: 0x0001, pad: 0x00";
		let bad_width =
			"variable_heap_op kind: 0x01, var_index: 0x777, indirect: 0x00, value: 0x0001, pad: 0x00";
		let fixture = format!(
			"# cc-fkb asm 1\n# script T.WSC\n\n# addr 0x00000000\n\
textbox_state_preset preset: 0x0114, pad: 0x00\n\n{bad_label}\n\n{bad_width}\n\n.trailer [ ]\n"
		);
		let doc = parse_document(&fixture);
		assert!(doc.has_errors());
		assert_eq!(doc.diagnostics.len(), 2, "{:#?}", doc.diagnostics);
		let first = &doc.diagnostics[0];
		assert_eq!(first.severity, Severity::Error);
		assert_eq!(first.line, 7);
		assert_eq!(
			first.column,
			char_column(bad_label, "wrong"),
			"the column points at the offending operand"
		);
		assert!(
			first.message.starts_with("operand 2 is \"wrong\"; variable_heap_op (0x03) expects \"indirect\""),
			"{}",
			first.message
		);
		let second = &doc.diagnostics[1];
		assert_eq!(second.line, 9);
		assert_eq!(second.column, char_column(bad_width, "0x777"));
		assert_eq!(
			second.message,
			"operand \"var_index\" takes a 2-byte hex literal, found \"0x777\""
		);
		assert_eq!(
			render_diagnostics(Utf8Path::new("/tmp/T.asm"), &doc),
			format!(
				"/tmp/T.asm:7:{}: error: {}\n/tmp/T.asm:9:{}: error: {}\n",
				first.column, first.message, second.column, second.message
			)
		);
		assert!(doc.into_script().is_err());
	}

	#[test]
	fn parse_spans_and_inserted_provenance() {
		let fixture = "# cc-fkb asm 1\n# script T.WSC\n\n# addr 0x00000000\n# label main\n\
textbox_state_preset preset: 0x0114, pad: 0x00\n\nwait_rerun\n\nnop_yield pad: 0xFF\n\n.trailer [ ]\n";
		let doc = parse_document(fixture);
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		assert_eq!(doc.items.len(), 3);
		let first = &doc.items[0];
		assert_eq!((first.line, first.address, first.opcode), (6, 0x00, 0x8C));
		assert!(first.annotated);
		assert_eq!(first.label.as_deref(), Some("main"));
		assert_eq!(first.operands.len(), 2);
		assert_eq!(first.operands[0].0, "preset");
		assert_eq!(first.operands[0].1, 6);
		assert_eq!(first.operands[0].2, char_column("textbox_state_preset preset: 0x0114, pad: 0x00", "preset:"));
		assert_eq!(first.operands[1].2, char_column("textbox_state_preset preset: 0x0114, pad: 0x00", "pad:"));
		// A line with no `# addr` is an inserted instruction: no annotation, still addressed.
		let second = &doc.items[1];
		assert_eq!((second.line, second.address, second.opcode), (8, 0x04, 0x04));
		assert!(!second.annotated);
		assert!(second.operands.is_empty());
		assert_eq!(doc.items[2].address, 0x05);
		assert!(!doc.items[2].annotated);
		assert_eq!(doc.labels.get("main"), Some(&0x00));

		// A truncated line is a positioned error, never a panic.
		let truncated = "# cc-fkb asm 1\n# script T.WSC\n\nvariable_heap_op kind: 0x01,\n";
		let doc = parse_document(truncated);
		assert!(doc.has_errors());
		assert_eq!(doc.diagnostics[0].line, 4);
		assert_eq!(
			doc.diagnostics[0].message,
			"variable_heap_op (0x03) takes 5 operands, found 2"
		);
	}

	/// The 1-based character column of `needle` in `line` — the expectation the parser derives
	/// independently, so a span and its assertion cannot drift together.
	fn char_column(line: &str, needle: &str) -> usize {
		line.find(needle).expect("needle in fixture line") + 1
	}

	/// Every line kind's failure at once: the wording, severity, line *and column* of each row of the
	/// format's diagnostic table, and that one line's problem never stops the next line from being
	/// read. This is the net under a grammar change — a reworded message or a shifted column shows up
	/// here as a diff.
	#[test]
	fn diagnostics_match_the_spec_on_a_broken_file() {
		let fixture = "# cc-fkb asm 1\n# script T.WSC\n\n\
nop_yield pad: [ 0x00, 0x00 ]\n\n\
wait_rerun extra: 0x01\nnope\n\
textbox_state_preset preset: 0x777, pad: 0x00\n\
textbox_state_preset nothing: 0x01, pad: 0x00\n\
# ylds\n# addr\n\
scene_text text: \"unterminated\n\
nop_yield pad: [ 0x00, 0x01 ]\n\
conditional_branch branch_type: 0x19, arg1: 0x03E8, arg2: 0x0000, offset: NOPE, pad: 0x00\n\
choice_jump count: 0x01, separator: 0x00, choices:\n\
.destroy\n\
textbox_state_preset preset: 0x0001, pad: 0x00 # trailing\n\
context\n";
		let doc = parse_document(fixture);
		let reported: Vec<(usize, usize, Severity, &str)> = doc
			.diagnostics
			.iter()
			.map(|it| (it.line, it.column, it.severity, it.message.as_str()))
			.collect();
		let expected: Vec<(usize, usize, Severity, &str)> = vec![
			(6, 12, Severity::Error, "wait_rerun (0x04) takes 0 operands, found 1"),
			(7, 1, Severity::Error, "unknown mnemonic \"nope\""),
			(8, 30, Severity::Error, "operand \"preset\" takes a 2-byte hex literal, found \"0x777\""),
			(
				9,
				22,
				Severity::Error,
				"operand 0 is \"nothing\"; textbox_state_preset (0x8C) expects \"preset\" (order: \"preset\", \"pad\")",
			),
			(10, 1, Severity::Flag, "unknown annotation key \"ylds\""),
			(11, 1, Severity::Error, "malformed \"addr\" annotation: \"\""),
			(12, 18, Severity::Error, "unterminated string literal"),
			(14, 67, Severity::Error, "undefined label \"NOPE\""),
			(
				15,
				1,
				Severity::Error,
				"choice_jump declares count 1 but 0 choice records follow",
			),
			(16, 1, Severity::Error, "unrecognized line: \".destroy\""),
			(
				17,
				1,
				Severity::Error,
				"trailing \"#\" annotation; put it on its own line or use a \"; \" comment",
			),
			(18, 1, Severity::Error, "unknown mnemonic \"context\""),
		];
		assert_eq!(reported, expected, "{:#?}", doc.diagnostics);
		assert_eq!(
			doc.summary(),
			Summary { instructions: 4, annotated: 0, inserted: 4, errors: 11, findings: 1 },
			"one line's problem never stops the next line: the file is read to its end"
		);
	}

	fn render_diagnostics(path: &Utf8Path, doc: &AsmDocument) -> String {
		crate::bin_utils::render_diagnostics(path, doc)
	}
}
