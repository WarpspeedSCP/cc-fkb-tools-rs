//! The driver: annotated assembly to a [`Script`], with every problem reported as a positioned
//! [`Diagnostic`] instead of a failed parse.
//!
//! The grammar is [`super::grammar`] — one nom parser per construct, and the normative text is
//! `assembly/README.md` — and this module is the line-at-a-time API on top of it. [`parse_line`]
//! parses exactly one line, with no reference to any other line, and is the one place a parser
//! failure becomes a [`Diagnostic`]; a line that fails contributes its diagnostics and nothing else,
//! so the caller moves on to the next line. [`parse_document`] runs it over every line and adds the
//! document semantics: addresses derived from the parsed operand sizes (`# addr` is only ever
//! compared against them), labels — which resolve in a second pass, so forward references work —,
//! choice blocks, the trailer, and `# translation` annotations, which apply positionally to an
//! instruction's string operands.
//!
//! Parsing never fails and never panics: a malformed line becomes diagnostics the caller can note, so
//! an editor can be served on partial input.

use std::collections::BTreeMap;

use nom::Parser;

use crate::opcodes::{Code, Choice, OpField, Opcode, OpcodeSpecStatic, Script, TLString, OPCODE_SPECS};

use super::grammar::{self, AsmError, JumpTarget, LineShape};
use super::print::MAGIC;
use super::{
	finish_manifest, is_jump_field, jump_field, AsmDocument, AsmItem, Comment, CommentAnchor,
	Diagnostic, Severity,
};

/// A jump operand whose value can only be filled once every instruction address is known.
struct PendingJump<'a> {
	opcode_index: usize,
	line: usize,
	column: usize,
	token: &'a str,
}

/// The annotations of one instruction's block, applied when the instruction line arrives.
#[derive(Default)]
struct PendingAnnotations {
	addr: Option<(u32, usize)>,
	yields: Option<usize>,
	labels: Vec<(String, usize, usize)>,
	translations: Vec<(usize, usize, Option<String>)>,
}

impl PendingAnnotations {
	fn clear(&mut self) {
		*self = PendingAnnotations::default();
	}
}

/// What one line is, after its `;` comment has been split off. Every token borrows the line it was
/// read from, so parsing a line allocates nothing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LineKind<'a> {
	/// A line with nothing on it.
	Blank,
	/// An annotation: its key and value as written, and the 1-based column of its `#`.
	Annotation { key: &'a str, value: &'a str, column: usize },
	/// The `.trailer` directive, and the text after the directive's name.
	Directive { rest: &'a str },
	/// An instruction line: its mnemonic, and its operand region (everything after the mnemonic).
	Instruction { mnemonic: &'a str, operands: &'a str },
	/// A choice record line: its text, and its comma-separated operands. The driver splits each
	/// operand into `label: value` in order, because a record's problem is reported in the order the
	/// record's own parser finds it.
	Record { text: &'a str, segments: Vec<&'a str> },
	/// An instruction line the grammar rejects — a `#` outside a string literal, or a mnemonic that
	/// is not an identifier. Its diagnostic is already recorded in [`LineParse::diagnostics`], and
	/// the pending annotation block above the line is discarded with it. `ends_choice_block` says
	/// whether the rejection also ends a choice block: a `#` is rejected before the line is read as
	/// an instruction, so it does not, while a rejected mnemonic does.
	RejectedInstruction { ends_choice_block: bool },
	/// A choice record line with a `#` outside a string literal: its diagnostic is already recorded,
	/// and a record line leaves the pending annotations and the choice block for the lines that
	/// follow.
	RejectedRecord,
	/// A line no rule accepts: [`parse_line`] has already recorded its diagnostic.
	Bad,
}

/// One line, parsed on its own: what it is, its trailing comment (trimmed, with the 1-based column of
/// its `;`), and every problem it produced.
pub struct LineParse<'a> {
	pub kind: LineKind<'a>,
	pub comment: Option<(&'a str, usize)>,
	pub diagnostics: Vec<Diagnostic>,
}

/// Parses exactly one line of assembly, with no reference to any other line — the unit an editor
/// serves a buffer in, and the unit the document loop runs on.
///
/// `line` is the 1-based line number, which is stamped on every diagnostic and used for nothing else.
/// The function never looks at another line, never fails and never panics: trailing blanks are
/// trimmed, the `;` comment is split off (a `;` inside a `"…"` literal is data), and what remains is
/// classified and read. A line that no rule accepts comes back as [`LineKind::Bad`], or as a rejected
/// instruction or record when the line was one of those but is malformed, with the diagnostics that
/// say why — so the caller only has to note the problem and move on to the next line.
pub fn parse_line(input: &str, line: usize) -> LineParse<'_> {
	let (code, comment) = grammar::code_and_comment(input);
	let trimmed = code.trim_end();
	let body = trimmed.trim_start();
	let mut diagnostics = Vec::new();
	let kind = match grammar::classify(trimmed) {
		LineShape::Blank => LineKind::Blank,
		LineShape::Annotation => match grammar::annotation(trimmed) {
			// The classifier has seen the `#`, so this cannot fail.
			Ok((_, (key, value))) => LineKind::Annotation {
				key,
				value,
				column: grammar::column_of(input, &body[..1]),
			},
			Err(_) => {
				diagnostics.push(unrecognized(line, trimmed));
				LineKind::Bad
			}
		},
		LineShape::Directive => match grammar::directive(trimmed) {
			Ok((_, (name, rest))) if name == "trailer" => LineKind::Directive { rest },
			_ => {
				diagnostics.push(unrecognized(line, trimmed));
				LineKind::Bad
			}
		},
		LineShape::Record => {
			if grammar::has_top_level_hash(trimmed) {
				diagnostics.push(trailing_hash(line));
				LineKind::RejectedRecord
			} else {
				LineKind::Record { text: body, segments: grammar::segments(body) }
			}
		}
		LineShape::Instruction => {
			if grammar::has_top_level_hash(trimmed) {
				diagnostics.push(trailing_hash(line));
				LineKind::RejectedInstruction { ends_choice_block: false }
			} else {
				match grammar::instruction(trimmed) {
					Ok((_, (mnemonic, operands))) => LineKind::Instruction { mnemonic, operands },
					Err(_) => {
						diagnostics.push(unrecognized(line, trimmed));
						LineKind::RejectedInstruction { ends_choice_block: true }
					}
				}
			}
		}
		// The one message that quotes the line as written, comment and all: no rule accepted it, so
		// there is no code half to name.
		LineShape::Bad => {
			diagnostics.push(error(
				line,
				1,
				format!("unrecognized line: \"{}\"", input.trim_end()),
			));
			LineKind::Bad
		}
	};
	LineParse { kind, comment, diagnostics }
}

/// Parses an `.asm` file. The result always holds the script that could be read plus every problem
/// found; [`AsmDocument::has_errors`] decides whether that script may be assembled.
pub fn parse_document(text: &str) -> AsmDocument {
	let mut out = AsmDocument {
		script: Script { opcode_table: vec![], opcodes: vec![], trailer: vec![] },
		items: vec![],
		labels: BTreeMap::new(),
		comments: vec![],
		diagnostics: vec![],
	};
	let mut pending = PendingAnnotations::default();
	let mut label_lines: BTreeMap<String, usize> = BTreeMap::new();
	let mut jumps: Vec<PendingJump> = vec![];
	let mut address = 0usize;
	let mut choice_slot: Option<(usize, usize)> = None;
	let mut trailer_line: Option<usize> = None;

	let has_content = text.lines().any(|it| !it.trim().is_empty());
	if has_content {
		let first = text.lines().next().unwrap_or_default().trim_end();
		if first != MAGIC {
			out.diagnostics.push(error(
				1,
				1,
				format!("line 1 must be exactly \"{MAGIC}\", found \"{first}\""),
			));
		}
	}

	for (index, raw) in text.lines().enumerate() {
		let number = index + 1;
		if has_content && index == 0 {
			continue;
		}
		let parsed = parse_line(raw, number);
		if let Some((text, _)) = parsed.comment {
			// A trailing comment belongs to the instruction on its line: the one this line is about to
			// add (an index that is valid even for the first instruction), or the one already parsed
			// for a choice record. Anything else belongs to the instruction that follows.
			let anchor = match parsed.kind {
				LineKind::Instruction { .. } | LineKind::RejectedInstruction { .. } => {
					CommentAnchor::Trailing(out.items.len())
				}
				LineKind::Record { .. } | LineKind::RejectedRecord if !out.items.is_empty() => {
					CommentAnchor::Trailing(out.items.len() - 1)
				}
				_ => CommentAnchor::Before(out.items.len()),
			};
			out.comments.push(Comment { line: number, text: text.to_owned(), anchor });
		}
		out.diagnostics.extend(parsed.diagnostics);

		match parsed.kind {
			LineKind::Blank | LineKind::Bad | LineKind::RejectedRecord => {}
			LineKind::Annotation { key, value, column } => {
				parse_annotation(&mut out, &mut pending, number, column, key, value);
			}
			LineKind::Directive { rest } => {
				if let Some(first) = trailer_line {
					out.diagnostics.push(error(
						number,
						1,
						format!("duplicate \".trailer\" directive (first on line {first})"),
					));
				} else {
					trailer_line = Some(number);
					match grammar::value_of(grammar::byte_list_value(rest)) {
						Ok(bytes) => out.script.trailer = bytes,
						Err(message) => out.diagnostics.push(error(number, 1, message)),
					}
				}
			}
			LineKind::RejectedInstruction { ends_choice_block } => {
				if ends_choice_block {
					choice_slot = None;
				}
				pending.clear();
			}
			LineKind::Record { text, segments } => {
				match choice_slot {
					None => out.diagnostics.push(error(
						number,
						1,
						"choice record with no preceding \"choices:\" block".to_owned(),
					)),
					Some((opcode_index, field_index)) => match choice_record(text, &segments) {
						Ok(choice) => {
							if choice.trailer.len() != 11 {
								out.diagnostics.push(flag(
									number,
									1,
									format!(
										"choice trailer is {} bytes; the decoder always reads 11",
										choice.trailer.len()
									),
								));
							}
							if let OpField::Choice(choices) =
								&mut out.script.opcodes[opcode_index].fields[field_index]
							{
								// The instruction line sized the choice list as empty; the records
								// arrive on the lines that follow, so the running address has to grow
								// with them.
								address += choice.size();
								choices.push(choice);
							}
						}
						Err(message) => out.diagnostics.push(error(number, 1, message)),
					},
				}
			}
			LineKind::Instruction { mnemonic, operands } => {
				choice_slot = None;
				let candidates: Vec<&'static OpcodeSpecStatic> =
					OPCODE_SPECS.iter().filter(|it| it.name == mnemonic).collect();
				if candidates.is_empty() {
					out.diagnostics.push(error(
						number,
						1,
						format!("unknown mnemonic \"{mnemonic}\""),
					));
					pending.clear();
					continue;
				}
				// A mnemonic several rows share is resolved by the operand list: the row this line
				// spells is the one that decodes, and two that decode are a table problem rather than
				// a line problem.
				let mut attempts: Vec<(&OpcodeSpecStatic, Result<grammar::Decoded, AsmError>)> =
					candidates
						.iter()
						.map(|spec| (*spec, grammar::finished(grammar::row_values(spec).parse(operands))))
						.collect();
				let decoded_rows = attempts.iter().filter(|(_, it)| it.is_ok()).count();
				let index = match (attempts.len(), decoded_rows) {
					(1, _) | (_, 0) => 0,
					(_, 1) => attempts
						.iter()
						.position(|(_, it)| it.is_ok())
						.unwrap_or_default(),
					// Two rows decode, so only the table can say which one this line is. Both are
					// successes, so both are in `attempts`.
					_ => {
						let shared: Vec<u8> = attempts
							.iter()
							.filter(|(_, it)| it.is_ok())
							.map(|(spec, _)| spec.opcode)
							.take(2)
							.collect();
						out.diagnostics.push(error(
							number,
							1,
							format!(
								"\"{mnemonic}\" matches 0x{:02X} and 0x{:02X} with this operand list; the opcode table must resolve it uniquely",
								shared[0], shared[1]
							),
						));
						pending.clear();
						continue;
					}
				};
				let (spec, result) = attempts.swap_remove(index);
				let decoded = match result {
					Ok(decoded) => decoded,
					Err(it) => {
						out.diagnostics.push(positioned(raw, number, it));
						pending.clear();
						continue;
					}
				};
				let mut fields = decoded.fields;
				let opcode_index = out.script.opcodes.len();

				// Annotation block checks, now that the row is known.
				if let Some((annotated, line)) = pending.addr {
					if annotated as usize != address {
						out.diagnostics.push(flag(
							line,
							1,
							format!(
								"stale address annotation: says 0x{annotated:08X}, computed 0x{address:08X}"
							),
						));
					}
				}
				if let Some(line) = pending.yields {
					if !spec.yields {
						out.diagnostics.push(flag(
							line,
							1,
							format!(
								"\"yields\" annotation on {mnemonic} (0x{:02X}), whose opcode_table row does not yield",
								spec.opcode
							),
						));
					}
				}
				let mut item = AsmItem {
					line: number,
					address,
					opcode: spec.opcode,
					annotated: pending.addr.is_some(),
					label: None,
					operands: decoded
						.spans
						.iter()
						.map(|(label, slice)| {
							(label.clone(), number, grammar::column_of(raw, slice))
						})
						.collect(),
				};
				for (name, line, column) in &pending.labels {
					if let Some(first) = label_lines.get(name) {
						out.diagnostics.push(error(
							*line,
							*column,
							format!(
								"duplicate label \"{name}\" (first defined on line {first})"
							),
						));
						continue;
					}
					label_lines.insert(name.clone(), *line);
					out.labels.insert(name.clone(), address);
					if item.label.is_none() {
						item.label = Some(name.clone());
					}
				}
				let strings: Vec<usize> = fields
					.iter()
					.enumerate()
					.filter(|(_, it)| matches!(it, OpField::String(_)))
					.map(|(i, _)| i)
					.collect();
				for (k, (line, column, text)) in pending.translations.iter().enumerate() {
					let Some(field_index) = strings.get(k).copied() else {
						out.diagnostics.push(flag(
							*line,
							*column,
							format!(
								"\"translation\" annotation on {mnemonic} (0x{:02X}), which has no string operand",
								spec.opcode
							),
						));
						continue;
					};
					if let OpField::String(value) = &mut fields[field_index] {
						// An empty annotation clears the translation: `Some("")` would encode an
						// empty string and silently destroy the operand's text.
						value.translation = text.clone();
					}
				}
				for &(field_index, token) in &decoded.jump_tokens {
					let column = decoded
						.spans
						.get(field_index)
						.map(|(_, slice)| grammar::column_of(raw, slice))
						.unwrap_or_else(|| grammar::column_of(raw, operands));
					jumps.push(PendingJump { opcode_index, line: number, column, token });
				}
				let mut size = 1usize;
				for field in &fields {
					size += field.size();
				}
				if spec.layout.iter().any(|it| matches!(it, Code::Choice)) {
					let field_index = spec
						.layout
						.iter()
						.position(|it| matches!(it, Code::Choice))
						.unwrap_or_default();
					choice_slot = Some((opcode_index, field_index));
				}
				out.script.opcodes.push(Opcode {
					opcode: spec.opcode,
					address,
					actual_address: 0,
					fields,
				});
				out.items.push(item);
				address += size;
				pending.clear();
			}
		}
	}

	resolve_jumps(&mut out, &jumps);
	check_choice_counts(&mut out);
	out.diagnostics.sort_by_key(|it| (it.line, it.column));
	finish_manifest(&mut out.script);
	out
}

/// A parser failure as the positioned error a line reports: the column is the character column of the
/// token the parser stopped at.
fn positioned(line: &str, number: usize, failure: AsmError<'_>) -> Diagnostic {
	Diagnostic {
		severity: Severity::Error,
		line: number,
		column: grammar::column_of(line, failure.input),
		message: failure.message,
	}
}

fn unrecognized(line: usize, text: &str) -> Diagnostic {
	error(line, 1, format!("unrecognized line: \"{text}\""))
}

/// The diagnostic for a `#` in an operand region: the format's annotations are whole lines.
fn trailing_hash(line: usize) -> Diagnostic {
	error(
		line,
		1,
		"trailing \"#\" annotation; put it on its own line or use a \"; \" comment".to_owned(),
	)
}

/// One choice record line's `arg1`, `text` and `trailer`, in the order they are written. The record's
/// own parser reports the first problem it meets, so the parts are read left to right and a value
/// the format rejects keeps its own message; everything else (a part with no `:`, a label the record
/// does not define, a missing label) is an unrecognized line.
fn choice_record(text: &str, segments: &[&str]) -> Result<Choice, String> {
	let unrecognized = || format!("unrecognized line: \"{text}\"");
	let mut arg1 = None;
	let mut choice_str = None;
	let mut trailer = None;
	for segment in segments {
		let Ok((_, (label, value))) = grammar::label_value(segment) else {
			return Err(unrecognized());
		};
		match label {
			"arg1" => {
				arg1 = Some(
					grammar::value_of(
						grammar::hex_value("arg1", 4, "a 2-byte hex literal").parse(value),
					)? as u16,
				);
			}
			"text" => choice_str = Some(grammar::value_of(grammar::string_value(value))?),
			"trailer" => trailer = Some(grammar::value_of(grammar::byte_list_value(value))?),
			_ => return Err(unrecognized()),
		}
	}
	let (Some(arg1), Some(choice_str), Some(trailer)) = (arg1, choice_str, trailer) else {
		return Err(unrecognized());
	};
	Ok(Choice {
		arg1,
		choice_str: TLString { raw: choice_str, translation: None, notes: None },
		trailer,
	})
}

/// Applies one annotation to the block of annotations the instruction below it will use. A value the
/// format rejects, and a key it does not define, are diagnostics — the block itself still reaches the
/// instruction that follows.
fn parse_annotation(
	out: &mut AsmDocument,
	pending: &mut PendingAnnotations,
	line: usize,
	column: usize,
	key: &str,
	value: &str,
) {
	let malformed = |out: &mut AsmDocument| {
		out.diagnostics.push(error(
			line,
			column,
			format!("malformed \"{key}\" annotation: \"{value}\""),
		));
	};
	match key {
		"script" => {
			if value.is_empty() {
				malformed(out);
			}
		}
		"addr" => {
			let digits = value
				.strip_prefix("0x")
				.or_else(|| value.strip_prefix("0X"))
				.unwrap_or("");
			match grammar::hex_digits(digits) {
				Ok((_, int)) if value.len() == 10 => pending.addr = Some((int, line)),
				_ => malformed(out),
			}
		}
		"label" => match grammar::label_name(value) {
			Ok((_, name)) => pending.labels.push((name.to_owned(), line, column)),
			Err(_) => malformed(out),
		},
		"yields" => {
			if value.is_empty() {
				pending.yields = Some(line);
			} else {
				malformed(out);
			}
		}
		"translation" => match grammar::value_of(grammar::string_value(value)) {
			Ok(text) => pending
				.translations
				.push((line, column, if text.is_empty() { None } else { Some(text) })),
			Err(message) => out.diagnostics.push(error(line, column, message)),
		},
		_ => out
			.diagnostics
			.push(flag(line, column, format!("unknown annotation key \"{key}\""))),
	}
}

/// Second pass: every jump operand, once the address of every instruction is known.
fn resolve_jumps(out: &mut AsmDocument, jumps: &[PendingJump]) {
	let addresses: Vec<usize> = out.items.iter().map(|it| it.address).collect();
	for pending in jumps {
		if pending.opcode_index >= out.script.opcodes.len() {
			continue;
		}
		let instruction_address = out.script.opcodes[pending.opcode_index].address;
		let opcode = out.script.opcodes[pending.opcode_index].opcode;
		let token = pending.token;
		let undefined = |out: &mut AsmDocument| {
			out.diagnostics.push(error(
				pending.line,
				pending.column,
				format!("undefined label \"{token}\""),
			));
		};
		let (destination, literal) = match grammar::jump_target(token) {
			Some(JumpTarget::LabelAddress(address)) => (address as usize, false),
			Some(JumpTarget::Address(address)) => (address as usize, true),
			Some(JumpTarget::Name(name)) => match out.labels.get(name) {
				Some(target) => (*target, false),
				None => {
					undefined(out);
					continue;
				}
			},
			None => {
				undefined(out);
				continue;
			}
		};
		if !addresses.contains(&destination) {
			let (severity, message) = if literal {
				(
					Severity::Flag,
					format!(
						"jump destination 0x{destination:08X} is not the address of an instruction in this file"
					),
				)
			} else {
				(
					Severity::Error,
					format!(
						"label \"{token}\" resolves to 0x{destination:08X}, which is not the address of an instruction in this file"
					),
				)
			};
			out.diagnostics.push(Diagnostic {
				severity,
				line: pending.line,
				column: pending.column,
				message,
			});
		}
		let field_index = if is_jump_field(opcode, 0) { 0 } else { 3 };
		if let Some(field) = out.script.opcodes[pending.opcode_index].fields.get_mut(field_index) {
			*field = OpField::DWord(jump_field(opcode, instruction_address, destination));
		}
	}
}

/// The count byte an opcode declares must match the number of records that follow it.
fn check_choice_counts(out: &mut AsmDocument) {
	let mut found = Vec::new();
	for opcode in &out.script.opcodes {
		let Some(spec) = crate::opcodes::lookup_spec(opcode.opcode) else { continue };
		let Some(choice_index) = spec.layout.iter().position(|it| matches!(it, Code::Choice)) else {
			continue;
		};
		let Some(count_index) = spec.operands.iter().position(|it| *it == "count") else {
			continue;
		};
		let declared = match opcode.fields.get(count_index) {
			Some(OpField::Byte(value)) => *value as usize,
			_ => continue,
		};
		let records = match opcode.fields.get(choice_index) {
			Some(OpField::Choice(choices)) => choices.len(),
			_ => continue,
		};
		if declared != records {
			let line = out
				.items
				.iter()
				.find(|it| it.address == opcode.address)
				.map(|it| it.line)
				.unwrap_or_default();
			found.push(error(
				line,
				1,
				format!("{} declares count {declared} but {records} choice records follow", spec.name),
			));
		}
	}
	out.diagnostics.extend(found);
}

fn error(line: usize, column: usize, message: String) -> Diagnostic {
	Diagnostic { severity: Severity::Error, line, column, message }
}

fn flag(line: usize, column: usize, message: String) -> Diagnostic {
	Diagnostic { severity: Severity::Flag, line, column, message }
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn parse_line_is_line_atomic() {
		// The same text is the same tokens wherever it sits in a file: the line number is the
		// caller's, and nothing else about the line's parsing depends on its position.
		let first = parse_line("nop_yield pad: 0x00", 1);
		let later = parse_line("nop_yield pad: 0x00", 40);
		assert_eq!(first.kind, later.kind);
		assert_eq!(first.diagnostics, later.diagnostics);
		assert_eq!(
			later.kind,
			LineKind::Instruction { mnemonic: "nop_yield", operands: "pad: 0x00" }
		);
		assert_eq!(later.comment, None);

		// A rejected line is the whole story of that line, and its diagnostic carries its number.
		let bad = parse_line("no_pe! x", 7);
		assert_eq!(bad.kind, LineKind::RejectedInstruction { ends_choice_block: true });
		assert_eq!(bad.diagnostics.len(), 1, "{:#?}", bad.diagnostics);
		assert_eq!(bad.diagnostics[0].line, 7);
		assert_eq!(bad.diagnostics[0].column, 1);
		assert_eq!(bad.diagnostics[0].message, "unrecognized line: \"no_pe! x\"");

		// The document loop is this parser and nothing else: the diagnostics a document reports for
		// a line are exactly the line's own, comment included.
		let doc = parse_document("# cc-fkb asm 1\n\n   nope\nnop_yield pad: 0x00 ; why\n");
		assert_eq!(doc.diagnostics, parse_line("   nope", 3).diagnostics);
		assert_eq!(doc.diagnostics[0].message, "unrecognized line: \"   nope\"");
		assert_eq!(parse_line("nop_yield pad: 0x00 ; why", 4).comment, Some(("why", 21)));
		assert_eq!(doc.comments.len(), 1);
		assert_eq!(doc.comments[0].text, "why");
	}

	#[test]
	fn columns_are_character_based_on_the_raw_line() {
		// A wrong label after a multi-byte operand: the column is the *character* column of the
		// operand, not its byte offset.
		let bad = "textbox_with_speaker layout_id: 0x0114, mode: 0x01, speaker_arg: 0x02, \
timer_param: 0x00, speaker_text: \"あい\", wrng: \"うえ\"";
		let doc = parse_document(&format!("# cc-fkb asm 1\n{bad}\n"));
		assert_eq!(doc.diagnostics.len(), 1, "{:#?}", doc.diagnostics);
		let reported = &doc.diagnostics[0];
		assert_eq!(reported.line, 2);
		assert!(reported.message.starts_with("operand 5 is \"wrng\""), "{}", reported.message);
		let byte_offset = bad.find("wrng").expect("the wrong label");
		assert_ne!(reported.column, byte_offset + 1, "a byte offset is not a column");
		assert_eq!(reported.column, bad[..byte_offset].chars().count() + 1);

		// The operand spans are character columns too.
		let line = "textbox_with_speaker layout_id: 0x0114, mode: 0x01, speaker_arg: 0x02, \
timer_param: 0x00, speaker_text: \"あい\", text: \"うえ\"";
		let doc = parse_document(&format!("# cc-fkb asm 1\n{line}\n"));
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		let operands = &doc.items[0].operands;
		assert_eq!(operands.len(), 6, "{operands:?}");
		let (label, number, column) = &operands[5];
		assert_eq!(label, "text");
		assert_eq!(*number, 2);
		let byte_offset = line.find("text: \"うえ\"").expect("the last string operand");
		assert_ne!(*column, byte_offset + 1, "a byte offset is not a column");
		assert_eq!(*column, line[..byte_offset].chars().count() + 1);

		// A line is parsed on its own, so a bad line after it reports its own number.
		let doc = parse_document(&format!("# cc-fkb asm 1\n{line}\nnope\n"));
		assert_eq!(doc.diagnostics.len(), 1, "{:#?}", doc.diagnostics);
		assert_eq!(doc.diagnostics[0].line, 3);
		assert_eq!(doc.diagnostics[0].message, "unknown mnemonic \"nope\"");
	}

	#[test]
	fn garbage_lines_never_panic() {
		let fixture = "# cc-fkb asm 1\n\
\"\n\
\"abc\n\
[ 0x00\n\
foo bar:\n\
# addr 0x\n\
\t\n\
  \n\
mnemonic a: \"\n\
:\n\
,,\n\
[ ] ]\n\
nop pad: [ 0x00,\n\
# label\n\
#\n\
# translation \"\\q\"\n\
  trailer: [ 0x00 ]\n\
nop_yield pad: [ 0x00, 0x00 ]\n";
		let doc = parse_document(fixture);
		assert!(doc.has_errors(), "garbage is reported:\n{:#?}", doc.diagnostics);
		// Every one of those lines is noted and the file is still read to the end.
		assert_eq!(doc.items.last().map(|it| it.opcode), Some(0xE6));
	}
}
