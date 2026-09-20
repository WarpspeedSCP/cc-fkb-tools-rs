//! Annotated assembly for `.WSC` scripts: the reader-facing text form of a decoded script, the
//! parser that reads it back, and the span-carrying parse result a language server can be built on
//! without re-parsing.
//!
//! The grammar is normative in `assembly/README.md`; this module implements it and nothing else
//! restates it. The binary side is [`crate::opcodes`]: mnemonics and operand names come from
//! `OPCODE_SPECS`, and `Script::binary_serialise` is the inverse of the jump rendering here.

use std::collections::BTreeMap;

use anyhow::anyhow;

use crate::opcodes::{manifest_for, Script};

pub mod parse;
pub mod print;

mod grammar;

pub use parse::{parse_document, parse_line, LineKind, LineParse};
pub use print::{print_document, print_script, print_script_with_constants};

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
/// the only conversion an editor needs (an LSP server converts it to UTF-16 code units). `source`
/// indexes [`AsmDocument::sources`]: 0 is the script itself, any other index a constants file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Diagnostic {
	pub source: usize,
	pub severity: Severity,
	pub line: usize,
	pub column: usize,
	pub message: String,
}

impl Diagnostic {
	pub fn new(
		source: usize,
		severity: Severity,
		line: usize,
		column: usize,
		message: impl Into<String>,
	) -> Self {
		Diagnostic { source, severity, line, column, message: message.into() }
	}

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

/// Which line 1 a file carries: `# cc-fkb asm 1` for a script, `# cc-fkb inc 1` for a constants file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceKind {
	Script,
	Constants,
}

/// What a constant declares, exactly as it was written: an [`ConstantValue::Alias`] is not followed
/// here, [`ConstantTable::resolve`] is what does that.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConstantValue {
	/// A `0x…` literal, with the width its digit count declares (`0x1` one byte, `0x0001` two).
	Number { value: u64, width: u8 },
	/// An `@0x…` literal: an address in the engine image.
	Address(usize),
	/// A `"…"` literal.
	Text(String),
	/// Another constant's name.
	Alias(String),
}

/// A value an alias chain ends at, so following aliases always terminates.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ResolvedValue {
	Number { value: u64, width: u8 },
	Address(usize),
	Text(String),
}

impl ConstantValue {
	/// What this value declares when it is not an alias: `None` for an [`ConstantValue::Alias`],
	/// whose value only the table can name.
	pub fn resolved(&self) -> Option<ResolvedValue> {
		match self {
			ConstantValue::Number { value, width } => {
				Some(ResolvedValue::Number { value: *value, width: *width })
			}
			ConstantValue::Address(value) => Some(ResolvedValue::Address(*value)),
			ConstantValue::Text(text) => Some(ResolvedValue::Text(text.clone())),
			ConstantValue::Alias(_) => None,
		}
	}
}

/// One definition: its name, what it declares, and the file and line it was written on.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Constant {
	pub name: String,
	pub value: ConstantValue,
	pub source: usize,
	pub line: usize,
}

/// Every constant a document was parsed with, in definition order. Lookups are by name; the reverse
/// lookups the printer uses scan that order, which is what a few dozen definitions cost.
#[derive(Clone, Default, Debug)]
pub struct ConstantTable {
	entries: Vec<Constant>,
	index: BTreeMap<String, usize>,
}

impl ConstantTable {
	pub fn get(&self, name: &str) -> Option<&Constant> {
		self.index.get(name).and_then(|it| self.entries.get(*it))
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	pub fn iter(&self) -> impl Iterator<Item = &Constant> + '_ {
		self.entries.iter()
	}

	/// Adds a definition. The loader has already rejected a name that means something else.
	pub(crate) fn insert(&mut self, constant: Constant) {
		self.index.insert(constant.name.clone(), self.entries.len());
		self.entries.push(constant);
	}

	/// The value a name stands for, following aliases to the end. `Err` is the chain that cycles, or
	/// the name that is not defined.
	pub fn resolve(&self, name: &str) -> Result<ResolvedValue, String> {
		let mut chain: Vec<&str> = Vec::new();
		let mut current = name;
		loop {
			let Some(constant) = self.get(current) else {
				return Err(format!("unknown constant \"{current}\""));
			};
			chain.push(current);
			match &constant.value {
				ConstantValue::Alias(target) => {
					if chain.contains(&target.as_str()) {
						chain.push(target);
						return Err(format!("constant alias cycle: {}", chain.join(" -> ")));
					}
					current = target;
				}
				ConstantValue::Number { value, width } => {
					return Ok(ResolvedValue::Number { value: *value, width: *width });
				}
				ConstantValue::Address(value) => return Ok(ResolvedValue::Address(*value)),
				ConstantValue::Text(text) => return Ok(ResolvedValue::Text(text.clone())),
			}
		}
	}

	/// The name a number prints as: a constant that declares this value at this width *and* whose name
	/// spells the operand it stands in — `label`, so a `branch_type` operand prints `BRANCH_TYPE_NE`
	/// and a `kind` operand `HEAP_KIND_ASSIGN`.
	///
	/// The label is what makes the reverse lookup usable: the engine reuses 0x00/0x01 in six different
	/// operands of the corpus, and a lookup by value alone would print `screen: HEAP_KIND_ASSIGN`.
	/// A value no name spells for this operand is left as the literal, so naming never guesses.
	pub fn name_for_number(&self, value: u64, width: u8, label: &str) -> Option<&str> {
		if label.is_empty() {
			return None;
		}
		for constant in &self.entries {
			let ConstantValue::Number { value: found, width: declared } = &constant.value else {
				continue;
			};
			if *found == value && *declared == width && names_operand(&constant.name, label) {
				return Some(constant.name.as_str());
			}
		}
		None
	}

	pub fn name_for_address(&self, address: usize) -> Option<&str> {
		self.entries.iter().find_map(|it| match it.value {
			ConstantValue::Address(value) if value == address => Some(it.name.as_str()),
			_ => None,
		})
	}

	pub fn name_for_text(&self, text: &str) -> Option<&str> {
		self.entries.iter().find_map(|it| match &it.value {
			ConstantValue::Text(value) if value == text => Some(it.name.as_str()),
			_ => None,
		})
	}
}

/// A file a document was read from: the script first, then every constants file in the order it was
/// first included. [`Diagnostic::source`] indexes this list.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Source {
	pub path: String,
	pub kind: SourceKind,
}

/// An `include` annotation as written. `source` is the file it sits in — `ccfkb_asm_fmt` re-emits
/// only the includes of the script itself, because a path written inside a constants file resolves
/// relative to *that* file, not the script.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Include {
	pub source: usize,
	pub path: String,
	pub line: usize,
}

/// One operand of an instruction line: its label, the value token *as written* — which
/// [`print::print_document`] re-emits, so an author's `0x0114` or `PRESET_MAIN` survives formatting —
/// and the 1-based line and character column of the operand.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AsmOperand {
	pub label: String,
	pub text: String,
	pub line: usize,
	pub column: usize,
}

/// One choice record's operand tokens as the author wrote them: the record's own `arg1`, `text`,
/// `gate_indirect` and `gate_value`, then its payload's operands. `ccfkb_asm_fmt` re-emits these, so
/// a name or a literal survives formatting on a choice record exactly as it does on an instruction
/// line.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct AsmRecord {
	pub operands: Vec<AsmOperand>,
	pub payload: Vec<AsmOperand>,
}

/// One instruction as it appears in the text, for tooling: its line, derived `address`, the opcode
/// byte it resolved to, whether it carried an `addr` annotation (`annotated == false` means the line
/// was inserted by hand), its label if any, the [`AsmOperand`] of every operand token, and — for a
/// `choice_jump` — one [`AsmRecord`] per record line that followed it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AsmItem {
	pub line: usize,
	pub address: usize,
	pub opcode: u8,
	pub annotated: bool,
	pub label: Option<String>,
	pub operands: Vec<AsmOperand>,
	pub records: Vec<AsmRecord>,
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
	pub constants: usize,
}

/// The result of parsing an `.asm` file: the script it describes, the per-instruction spans, the
/// constants it was parsed with, and every problem found. Parsing never fails, so an editor can be
/// served on partial input.
#[derive(Clone, Debug)]
pub struct AsmDocument {
	pub script: Script,
	pub items: Vec<AsmItem>,
	pub labels: BTreeMap<String, usize>,
	pub comments: Vec<Comment>,
	pub diagnostics: Vec<Diagnostic>,
	/// The constants every value token in this document resolved against.
	pub constants: ConstantTable,
	/// The files this document was read from, the script first; [`Diagnostic::source`] indexes it.
	pub sources: Vec<Source>,
	/// The `include` annotations of the document's own files, in the order they were written, for
	/// [`print::print_document`] to re-emit.
	pub includes: Vec<Include>,
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
			constants: self.constants.len(),
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
	///
	/// A diagnostic from a constants file names that file; the script's own keep the bare
	/// `{line}:{column}:` position the bins render with the path they were given.
	pub fn into_script(self) -> anyhow::Result<Script> {
		if let Some(first) = self.diagnostics.iter().find(|it| it.severity == Severity::Error) {
			let path = self
				.sources
				.get(first.source)
				.filter(|_| first.source != 0)
				.map(|it| it.path.as_str())
				.unwrap_or_default();
			return Err(anyhow!(
				"{path}{}:{}: {}",
				first.line,
				first.column,
				first.message
			));
		}
		Ok(self.script)
	}
}

/// Whether a constant's name spells an operand's label as whole `_`-separated words: a `branch_type`
/// operand is named by `BRANCH_TYPE_NE`, a `kind` operand by `HEAP_KIND_ASSIGN`. This is what ties a
/// name to the operand it belongs to, so a value the engine reuses never borrows another operand's
/// name.
fn names_operand(name: &str, label: &str) -> bool {
	let (name, label) = (name.as_bytes(), label.as_bytes());
	if label.is_empty() || label.len() > name.len() {
		return false;
	}
	for start in 0..=name.len() - label.len() {
		let start_ok = start == 0 || name[start - 1] == b'_';
		let end = start + label.len();
		let end_ok = end == name.len() || name[end] == b'_';
		if start_ok && end_ok && name[start..end].eq_ignore_ascii_case(label) {
			return true;
		}
	}
	false
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
	use camino::{Utf8Path, Utf8PathBuf};
	use tempfile::TempDir;

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

	/// A choice record shaped like the corpus: the option is listed when heap word `gate_value` is
	/// set, and taking it runs the `variable_heap_op` payload that stores `value` in heap word 2.
	fn choice(arg1: u16, text: &str, gate_value: u16, value: u16) -> Choice {
		Choice {
			arg1,
			choice_str: TLString { raw: text.to_owned(), translation: None, notes: None },
			gate_indirect: 0x01,
			gate_value,
			payload_kind: 0x03,
			payload: vec![
				OpField::Byte(0x01),
				OpField::Word(0x0002),
				OpField::Byte(0x00),
				OpField::Word(value),
				OpField::Padding(vec![0x00]),
			],
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
						OpField::Choice(vec![
							choice(0x0102, "はい", 0x0352, 0x0001),
							choice(0x0103, "いいえ", 0x0353, 0x0002),
						]),
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
		let doc = parse_document(&text, Utf8Path::new(NAME));
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
#trailer [ 0x43 ]\n";
		let doc = parse_document(fixture, Utf8Path::new(NAME));
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
		let again = parse_document(&text, Utf8Path::new(NAME));
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
#trailer [ ]\n";
		let doc = parse_document(two, Utf8Path::new(NAME));
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		let script = doc.clone().into_script().unwrap();
		let OpField::String(first) = &script.opcodes[0].fields[4] else { panic!("speaker_text") };
		let OpField::String(second) = &script.opcodes[0].fields[5] else { panic!("text") };
		assert_eq!(first.translation, None);
		assert_eq!(second.translation.as_deref(), Some("Second line"));
		let text = print_document(&doc, "T.WSC").unwrap();
		assert!(text.contains("# translation \"\"\n# translation \"Second line\""), "{text}");
		assert_eq!(
			print_document(&parse_document(&text, Utf8Path::new(NAME)), "T.WSC").unwrap(),
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
		let doc = parse_document(fixture, Utf8Path::new(NAME));
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
textbox_state_preset preset: 0x0114, pad: 0x00\n\n{bad_label}\n\n{bad_width}\n\n#trailer [ ]\n"
		);
		let doc = parse_document(&fixture, Utf8Path::new("/tmp/T.asm"));
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
textbox_state_preset preset: 0x0114, pad: 0x00\n\nwait_rerun\n\nnop_yield pad: 0xFF\n\n#trailer [ ]\n";
		let doc = parse_document(fixture, Utf8Path::new(NAME));
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		assert_eq!(doc.items.len(), 3);
		let first = &doc.items[0];
		assert_eq!((first.line, first.address, first.opcode), (6, 0x00, 0x8C));
		assert!(first.annotated);
		assert_eq!(first.label.as_deref(), Some("main"));
		assert_eq!(first.operands.len(), 2);
		assert_eq!(first.operands[0].label, "preset");
		assert_eq!(first.operands[0].text, "0x0114", "the token as written is kept");
		assert_eq!(first.operands[0].line, 6);
		assert_eq!(first.operands[0].column, char_column("textbox_state_preset preset: 0x0114, pad: 0x00", "preset:"));
		assert_eq!(
			first.operands[1].column,
			char_column("textbox_state_preset preset: 0x0114, pad: 0x00", "pad:")
		);
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
		let doc = parse_document(truncated, Utf8Path::new(NAME));
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
		let doc = parse_document(fixture, Utf8Path::new(NAME));
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
			Summary { instructions: 4, annotated: 0, inserted: 4, errors: 11, findings: 1, constants: 0 },
			"one line's problem never stops the next line: the file is read to its end"
		);
	}

	fn render_diagnostics(path: &Utf8Path, doc: &AsmDocument) -> String {
		crate::bin_utils::render_diagnostics(path, doc)
	}

	/// Writes `files` into a fresh directory: the script's own `#include` paths resolve against it, so
	/// a test can exercise the loader without a fixture in the repository.
	fn scratch(files: &[(&str, &str)]) -> (TempDir, Utf8PathBuf) {
		let dir = TempDir::new().expect("a temporary directory");
		for (name, text) in files {
			std::fs::write(dir.path().join(name), text).expect("writing the fixture");
		}
		let script = Utf8PathBuf::from_path_buf(dir.path().join(files[0].0))
			.expect("the temporary path is UTF-8");
		(dir, script)
	}

	/// A script's text and the table it was parsed with, including the constants beside it.
	fn parse_file(path: &Utf8Path) -> AsmDocument {
		let text = std::fs::read_to_string(path).expect("reading the fixture");
		parse_document(&text, path)
	}

	const ENGINE_INC: &str = "# cc-fkb inc 1\n\
; every name here spells the operand label it stands in, which is what the printer matches on\n\
PRESET_MAIN = 0x0005\n\
PAD_BYTE = 0x04\n\
PRESET_ALIAS = PRESET_MAIN\n\
TITLE_TEXT = \"CROSS†CHANNEL\"\n";

	#[test]
	fn includes_define_constants_used_by_operands() {
		let (_dir, script) = scratch(&[
			(
				"T.WSC.asm",
				"# cc-fkb asm 1\n# script T.WSC\n\n# include \"engine.inc\"\n\n\
textbox_state_preset preset: PRESET_MAIN, pad: PAD_BYTE\n\
scene_text text: TITLE_TEXT\n\
end_of_script\n\n#trailer [ ]\n",
			),
			("engine.inc", ENGINE_INC),
		]);
		let doc = parse_file(&script);
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		assert_eq!(doc.constants.len(), 4, "every definition is in the table");
		assert_eq!(doc.sources.len(), 2, "the script and the file it includes");

		let script = doc.clone().into_script().expect("the file is well formed");
		assert_eq!(script.opcodes.len(), 3);
		let fields = &script.opcodes[0].fields;
		assert!(matches!(fields[0], OpField::Word(0x0005)), "{fields:?}");
		assert!(matches!(&fields[1], OpField::Padding(bytes) if bytes == &[0x04]), "{fields:?}");
		let OpField::String(text) = &script.opcodes[1].fields[0] else { panic!("a string operand") };
		assert_eq!(text.raw, "CROSS†CHANNEL", "a text constant is the operand's text");

		// Every definition remembers where it was written, so a diagnostic can name that file.
		let include = Utf8Path::new(&doc.sources[1].path);
		assert_eq!(include.file_name(), Some("engine.inc"));
		for (name, line) in [("PRESET_MAIN", 3), ("PAD_BYTE", 4), ("PRESET_ALIAS", 5), ("TITLE_TEXT", 6)] {
			let constant = doc.constants.get(name).expect("the definition");
			assert_eq!(constant.line, line, "{name}");
			assert_eq!(constant.source, 1, "{name} was written in the include");
		}
	}

	#[test]
	fn constant_tokens_and_includes_survive_fmt() {
		let (_dir, path) = scratch(&[
			(
				"T.WSC.asm",
				"# cc-fkb asm 1\n# script T.WSC\n\n# include \"engine.inc\"\n\n\
textbox_state_preset preset: PRESET_MAIN, pad: PRESET_ALIAS\n\
end_of_script\n\n#trailer [ ]\n",
			),
			("engine.inc", ENGINE_INC),
		]);
		let text = std::fs::read_to_string(&path).expect("reading the fixture");
		let first = print_document(&parse_document(&text, &path), "T.WSC").expect("printing");
		// The author's own spelling comes back: the include line and both constant tokens.
		assert!(first.contains("#include \"engine.inc\""), "{first}");
		assert!(first.contains("preset: PRESET_MAIN"), "{first}");
		assert!(first.contains("pad: PRESET_ALIAS"), "{first}");
		let second = print_document(&parse_document(&first, &path), "T.WSC").expect("printing");
		assert_eq!(first, second, "formatting twice changes nothing");
		assert_eq!(
			parse_document(&first, &path).into_script().unwrap().binary_serialise().unwrap(),
			parse_document(&text, &path).into_script().unwrap().binary_serialise().unwrap(),
			"a formatted file assembles to the same bytes"
		);
	}

	#[test]
	fn include_diagnostics_carry_their_own_file() {
		let (_dir, path) = scratch(&[
			(
				"T.WSC.asm",
				"# cc-fkb asm 1\n# script T.WSC\n\n# include \"missing.inc\"\n# include \"bad.inc\"\n\
# include \"cyc_a.inc\"\n# include \"wide.inc\"\n\n\
textbox_state_preset preset: NOPE, pad: 0x00\n\
textbox_state_preset preset: 0x0001, pad: WIDE\n\
end_of_script\n\n#trailer [ ]\n",
			),
			("bad.inc", "# cc-fkb inc 1\nSCREEN_X = 0x0001\nSCREEN_X = 0x0002\n"),
			("cyc_a.inc", "# cc-fkb inc 1\n# include \"cyc_b.inc\"\n"),
			("cyc_b.inc", "# cc-fkb inc 1\n# include \"cyc_a.inc\"\n"),
			("wide.inc", "# cc-fkb inc 1\nWIDE = 0x1234\n"),
		]);
		// Every include above is either missing or wrong on purpose: the script's own lines and the
		// two files it pulls in each contribute their own diagnostics.
		let doc = parse_file(&path);
		assert!(doc.has_errors());
		let messages: Vec<(usize, usize, &str)> = doc
			.diagnostics
			.iter()
			.map(|it| (it.source, it.line, it.message.as_str()))
			.collect();
		assert!(
			messages.iter().any(|it| it.0 == 0 && it.2 == "include file not found: \"missing.inc\""),
			"{messages:#?}"
		);
		assert!(
			messages.iter().any(|it| it.2.starts_with("constant \"SCREEN_X\" is already defined")),
			"{messages:#?}"
		);
		assert!(
			messages.iter().any(|it| it.2.starts_with("cyclic include: ")),
			"{messages:#?}"
		);
		assert!(
			messages.iter().any(|it| it.2 == "unknown constant \"NOPE\""),
			"{messages:#?}"
		);
		assert!(
			messages
				.iter()
				.any(|it| it.2 == "constant \"WIDE\" is 0x1234, which does not fit a 1-byte hex literal"),
			"{messages:#?}"
		);

		// A diagnostic a constants file raised names that file, and its own line inside it.
		let bad = doc.sources.iter().position(|it| it.path.ends_with("bad.inc")).expect("bad.inc");
		let rendered = render_diagnostics(&path, &doc);
		let line = messages.iter().find(|it| it.0 == bad).expect("the redefinition");
		assert!(
			rendered.contains(&format!(
				"{}:{}:1: error: constant \"SCREEN_X\" is already defined as 0x1 ({}:2)",
				doc.sources[bad].path,
				line.1,
				doc.sources[bad].path
			)),
			"{rendered}"
		);

		// A clean include leaves no diagnostic at all.
		let (_clean_dir, clean) = scratch(&[
			(
				"T.WSC.asm",
				"# cc-fkb asm 1\n# script T.WSC\n\n# include \"wide.inc\"\n\n\
textbox_state_preset preset: WIDE, pad: 0x00\n\
end_of_script\n\n#trailer [ ]\n",
			),
			("wide.inc", "# cc-fkb inc 1\nWIDE = 0x0007\n"),
		]);
		let doc = parse_file(&clean);
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		assert_eq!(doc.constants.len(), 1);
	}

	#[test]
	fn symbol_named_disassembly_round_trips() {
		let (_dir, path) = scratch(&[("engine.inc", ENGINE_INC)]);
		let table = crate::asm::parse::load_constants(Utf8Path::new(&path)).expect("loading").0;
		let script = Script {
			opcode_table: vec![],
			opcodes: vec![
				Opcode {
					opcode: 0x8C,
					address: 0,
					actual_address: 0,
					fields: vec![OpField::Word(0x0005), OpField::Padding(vec![0x04])],
				},
				Opcode { opcode: 0xFF, address: 4, actual_address: 0, fields: vec![] },
			],
			trailer: vec![0x00],
		};
		let text = print_script_with_constants(&script, "T.WSC", &table, "engine.inc")
			.expect("printing with constants");
		assert!(text.contains("#include \"engine.inc\""), "{text}");
		assert!(text.contains("preset: PRESET_MAIN"), "a named value prints as its name:\n{text}");
		// Padding bytes are structural, so they stay literals even when a constant declares the value.
		assert!(text.contains("pad: 0x04"), "{text}");
		// A value no constant declares stays a literal.
		assert!(text.contains("#trailer [ 0x00 ]"), "{text}");

		// The same instructions with a value nothing names: no name, no include line.
		let mut bare = script.clone();
		bare.opcodes[0].fields[0] = OpField::Word(0x0009);
		let plain = print_script(&bare, "T.WSC").expect("printing");
		assert!(!plain.contains("#include"), "{plain}");
		assert!(plain.contains("preset: 0x0009"), "{plain}");

		// What the printer wrote parses back to the same bytes, read as a sibling of the include it
		// names (the path is what the include resolves against).
		let beside = path.with_file_name("T.WSC.asm");
		let back = parse_document(&text, &beside);
		assert!(back.diagnostics.is_empty(), "{:#?}", back.diagnostics);
		assert_eq!(
			back.into_script().unwrap().binary_serialise().unwrap(),
			script.binary_serialise().unwrap()
		);
	}

	#[test]
	fn constant_kinds_are_enforced() {
		let (_dir, path) = scratch(&[
			(
				"T.WSC.asm",
				"# cc-fkb asm 1\n# script T.WSC\n\n# include \"engine.inc\"\n\n\
scene_text text: PRESET_MAIN\n\
textbox_state_preset preset: TITLE_TEXT, pad: 0x00\n",
			),
			("engine.inc", ENGINE_INC),
		]);
		let doc = parse_file(&path);
		let messages: Vec<&str> = doc.diagnostics.iter().map(|it| it.message.as_str()).collect();
		assert_eq!(messages.len(), 2, "{messages:#?}");
		assert_eq!(
			messages[0],
			"operand \"text\" needs a string; \"PRESET_MAIN\" is a number constant"
		);
		assert_eq!(
			messages[1],
			"operand \"preset\" needs a 2-byte hex literal; \"TITLE_TEXT\" is a text constant"
		);
		assert!(doc.diagnostics.iter().all(|it| it.severity == Severity::Error));

		// The kinds that do match encode: a text constant as a string, a small number constant in a
		// 4-byte operand.
		let (_good_dir, good) = scratch(&[
			(
				"T.WSC.asm",
				"# cc-fkb asm 1\n# script T.WSC\n\n# include \"engine.inc\"\n\n\
scene_text text: TITLE_TEXT\n\
load_static_sprite slot: 0x00, x: 0x0000, y: 0x0000, id: ID_ENTRY, flags: 0x00, use_default: 0x01, filename: \"a.wip\"\n\
end_of_script\n\n#trailer [ ]\n",
			),
			("engine.inc", "# cc-fkb inc 1\nTITLE_TEXT = \"CROSS†CHANNEL\"\nID_ENTRY = 0x0001\n"),
		]);
		let doc = parse_file(&good);
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		let script = doc.into_script().expect("both kinds fit");
		let OpField::DWord(id) = script.opcodes[1].fields[3] else { panic!("the id operand") };
		assert_eq!(id, 0x0001);
	}

	#[test]
	fn a_choice_payload_is_an_opcode_of_its_own() {
		let fixture = "# cc-fkb asm 1\n# script T.WSC\n\n\
choice_jump count: 0x02, separator: 0x00, choices:\n  arg1: 0x0001, text: \"はい\", gate_indirect: 0x00, gate_value: 0x0001, payload: absolute_jump target: L_00000000, pad: 0x00\n  arg1: 0x0002, text: \"いいえ\", gate_indirect: 0x01, gate_value: 0x0352, payload: 0x04\n\
\n\
wait_rerun\n\n#trailer [ ]\n";
		let doc = parse_document(fixture, Utf8Path::new(NAME));
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		let OpField::Choice(choices) = &doc.script.opcodes[0].fields[2] else {
			panic!("the instruction declares a choice list")
		};
		// The payload is an opcode of its own, so its jump operand is resolved by the same second
		// pass an instruction's is.
		assert_eq!(choices[0].payload_kind, 0x06);
		assert_eq!(choices[0].payload.len(), 2, "a dword target and its padding byte");
		assert!(matches!(choices[0].payload[0], OpField::DWord(0)));
		assert!(matches!(&choices[0].payload[1], OpField::Padding(pad) if pad == &vec![0x00]));
		// A kind the interpreter does not dispatch keeps its byte and carries no operands.
		assert_eq!(choices[1].payload_kind, 0x04);
		assert!(choices[1].payload.is_empty());

		let text = print_document(&doc, NAME).unwrap();
		assert!(text.contains("# label L_00000000"), "{text}");
		assert!(
			text.contains("payload: absolute_jump target: L_00000000, pad: 0x00"),
			"{text}"
		);
		assert!(text.contains("payload: 0x04"), "{text}");
		let again = print_document(&parse_document(&text, Utf8Path::new(NAME)), NAME).unwrap();
		assert_eq!(again, text, "print → parse → print");
	}

	#[test]
	fn a_payload_that_is_not_a_payload_opcode_is_reported() {
		let cases = [
			(
				"wait_rerun",
				"payload must be variable_heap_op (0x03), absolute_jump (0x06), resource_string (0x07) or a kind byte, found \"wait_rerun\"",
			),
			("0x03", "payload 0x03 names an opcode; write \"variable_heap_op\""),
			(
				"variable_heap_op kind: 0x01",
				"variable_heap_op (0x03) takes 5 operands, found 1",
			),
		];
		for (payload, expected) in cases {
			let fixture = format!(
				"# cc-fkb asm 1\n# script T.WSC\n\n\
choice_jump count: 0x01, separator: 0x00, choices:\n  arg1: 0x0001, text: \"x\", gate_indirect: 0x00, gate_value: 0x0001, payload: {payload}\n\n#trailer [ ]\n"
			);
			let doc = parse_document(&fixture, Utf8Path::new(NAME));
			let messages: Vec<&str> = doc.diagnostics.iter().map(|it| it.message.as_str()).collect();
			assert!(messages.contains(&expected), "{payload}: {messages:#?}");
		}
	}

	#[test]
	fn fmt_keeps_a_constant_named_on_a_record() {
		let (_dir, path) = scratch(&[
			(
				"T.WSC.asm",
				"# cc-fkb asm 1\n# script T.WSC\n\n# include \"engine.inc\"\n\n\
choice_jump count: 0x01, separator: 0x00, choices:\n  arg1: SLOT_MAIN, text: \"はい\", gate_indirect: 0x01, gate_value: ROUTE_GATE, payload: variable_heap_op kind: HEAP_SET, var_index: 0x0002, indirect: 0x00, value: 0x0001, pad: 0x00\n\n#trailer [ ]\n",
			),
			(
				"engine.inc",
				"# cc-fkb inc 1\nSLOT_MAIN = 0x0001\nROUTE_GATE = 0x0352\nHEAP_SET = 0x01\n",
			),
		]);
		let doc = parse_file(&path);
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		// A record's fields and its payload's operands are re-emitted as written, like an
		// instruction's, so a name survives formatting.
		let printed = print_document(&doc, NAME).unwrap();
		assert!(printed.contains("arg1: SLOT_MAIN"), "{printed}");
		assert!(printed.contains("gate_value: ROUTE_GATE"), "{printed}");
		assert!(printed.contains("kind: HEAP_SET"), "{printed}");
		assert!(printed.contains("#include \"engine.inc\""), "{printed}");
		let script = doc.into_script().expect("the names resolve");
		let OpField::Choice(choices) = &script.opcodes[0].fields[2] else { panic!("choice list") };
		assert_eq!(choices[0].arg1, 0x0001);
		assert_eq!(choices[0].gate_value, 0x0352);
		assert!(matches!(choices[0].payload[0], OpField::Byte(0x01)));
	}
}
