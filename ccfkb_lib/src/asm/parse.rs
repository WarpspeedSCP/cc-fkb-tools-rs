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
//! instruction's string sequence — its string operands, and, through a `choices:` block, its arms'
//! texts, in record order.
//!
//! It also resolves the constants layer: the `include` annotations of the script are followed before
//! its lines are read (see [`Loader`]), so every value token may spell a name from a `.inc` file
//! instead of a literal. Diagnostics carry the source file they belong to, so a problem inside an
//! included file is reported against that file and not against the script.
//!
//! Parsing never fails and never panics: a malformed line becomes diagnostics the caller can note, so
//! an editor can be served on partial input.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use nom::Parser;

use crate::opcodes::{
	lookup_spec, Code, Choice, OpField, Opcode, OpcodeSpecStatic, Script, TLString, OPCODE_SPECS,
};

use super::grammar::{self, AsmError, DecodedFlag, JumpTarget, LineShape};
use super::print::{CONST_MAGIC, MAGIC};
use super::{
	escape_string, is_jump_field, jump_field, operand_labels, AsmDocument, AsmItem, AsmOperand,
	AsmRecord, Comment, CommentAnchor, Constant, ConstantTable, ConstantValue, Diagnostic, Include,
	Severity, Source, SourceKind,
};

/// A jump operand whose value can only be filled once every instruction address is known.
struct PendingJump<'a> {
	opcode_index: usize,
	slot: JumpSlot,
	line: usize,
	column: usize,
	token: &'a str,
}

/// Where a jump operand's dword lives: on the instruction itself, or in the payload of one of its
/// choice records. A payload is an opcode of its own spelled inside a record, and its jump operand is
/// resolved by the same second pass, so it prints as the same `L_<hex>` label.
#[derive(Clone, Copy)]
enum JumpSlot {
	Instruction,
	Payload { field_index: usize, choice_index: usize, operand_index: usize },
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

/// The `choices:` block an instruction opened: where its records land, and the `# translation`
/// annotations of the instruction's block, which name the arms in record order the way they name a
/// string operand. One annotation is bound per record as that record is read, and `next` is the one
/// the next record takes — so the leftovers are the annotations that named no arm.
struct ChoiceSlot {
	opcode_index: usize,
	field_index: usize,
	item_index: usize,
	spec: &'static OpcodeSpecStatic,
	annotations: Vec<(usize, usize, Option<String>)>,
	next: usize,
}

/// What one line is, after its `;` comment has been split off. Every token borrows the line it was
/// read from, so parsing a line allocates nothing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LineKind<'a> {
	/// A line with nothing on it.
	Blank,
	/// An annotation: its key and value as written, and the 1-based column of its `#`.
	Annotation { key: &'a str, value: &'a str, column: usize },
	/// A `.`-led directive. `.trailer` is the only directive the format ever had and it is retired in
	/// favour of the `#trailer` annotation, so this carries nothing: the driver reports the line as
	/// renamed.
	Directive,
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
			Ok((_, "trailer")) => LineKind::Directive,
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
///
/// `path` is the file's location: it is what `include` paths resolve from and what becomes
/// `sources[0].path`. It is never read from disk — an editor buffer is enough.
pub fn parse_document(text: &str, path: &Utf8Path) -> AsmDocument {
	let mut loader = Loader::new(path);
	loader.load_script(text, path);
	let constants = std::mem::take(&mut loader.constants);
	let mut out = AsmDocument {
		script: Script { opcodes: vec![], trailer: vec![] },
		items: vec![],
		labels: BTreeMap::new(),
		comments: vec![],
		diagnostics: std::mem::take(&mut loader.errors),
		// Filled at the end: the loop needs the table on its own while it mutates the document.
		constants: ConstantTable::default(),
		sources: std::mem::take(&mut loader.sources),
		includes: std::mem::take(&mut loader.includes),
	};
	let mut pending = PendingAnnotations::default();
	let mut label_lines: BTreeMap<String, usize> = BTreeMap::new();
	let mut jumps: Vec<PendingJump> = vec![];
	let mut address = 0usize;
	let mut choice_slot: Option<ChoiceSlot> = None;
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
				// annotations don't apply inside choice slots.
				if choice_slot.is_some() && column > 1 {
					out.diagnostics.push(flag(
						number,
						column,
						format!("\"{key}\" annotation inside a \"choices:\" block"),
					));
					continue;
				}
				parse_annotation(
					&mut out,
					&mut pending,
					&constants,
					&mut trailer_line,
					number,
					column,
					key,
					value,
				);
			}
			LineKind::Directive => out.diagnostics.push(error(
				number,
				1,
				"\".trailer\" is now \"#trailer\"".to_owned(),
			)),
			LineKind::RejectedInstruction { ends_choice_block } => {
				if ends_choice_block {
					close_choice_block(&mut out, choice_slot.take());
				}
				pending.clear();
			}
			LineKind::Record { text, segments } => {
				let Some(slot) = choice_slot.as_mut() else {
					out.diagnostics.push(error(
						number,
						1,
						"choice record with no preceding \"choices:\" block".to_owned(),
					));
					continue;
				};
				let (opcode_index, field_index, item_index) =
					(slot.opcode_index, slot.field_index, slot.item_index);
				let mut findings = Vec::new();
				let choice_index = match &out.script.opcodes[opcode_index].fields[field_index] {
					OpField::Choice(choices) => choices.len(),
					_ => 0,
				};
				match choice_record(raw, number, text, &segments, &constants, &mut findings) {
					Ok((mut choice, record, payload_jumps)) => {
						// The annotation block above the opcode line names this arm: the record
						// takes the *k*-th annotation of that block, and an empty one clears the
						// translation. Bound before the address grows, because a translated arm is
						// longer than its `raw`.
						if let Some((_, _, text)) = slot.annotations.get(slot.next) {
							choice.choice_str.translation = text.clone();
						}
						slot.next += 1;
						// A payload jump is resolved by the same pass as an instruction's, so
						// its token is queued the same way.
						for (operand_index, token) in payload_jumps {
							jumps.push(PendingJump {
								opcode_index,
								slot: JumpSlot::Payload {
									field_index,
									choice_index,
									operand_index,
								},
								line: number,
								column: grammar::column_of(raw, token),
								token,
							});
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
						if let Some(item) = out.items.get_mut(item_index) {
							item.records.push(record);
						}
					}
					Err(message) => out.diagnostics.push(error(number, 1, message)),
				}
				for finding in findings {
					out.diagnostics.push(finding_at(raw, number, finding));
				}
			}
			LineKind::Instruction { mnemonic, operands } => {
				close_choice_block(&mut out, choice_slot.take());
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
						.map(|spec| {
							(
								*spec,
								grammar::finished(
									grammar::row_values(spec, &constants).parse(operands),
								),
							)
						})
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
				for finding in decoded.flags {
					out.diagnostics.push(finding_at(raw, number, finding));
				}
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
								"\"yields\" annotation on {mnemonic} (0x{:02X}), whose table row does not yield",
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
						.map(|(label, slice, value)| AsmOperand {
							label: label.clone(),
							text: (*value).to_owned(),
							line: number,
							column: grammar::column_of(raw, slice),
						})
						.collect(),
					records: Vec::new(),
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
				// The arms of a `choices:` block continue this instruction's string sequence, and
				// they arrive on the lines below: the block takes the annotations and binds each one
				// as its arm is read. Every other opcode applies them here.
				let choice_field = spec.layout.iter().position(|it| matches!(it, Code::Choice));
				if choice_field.is_none() {
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
				}
				for &(field_index, token) in &decoded.jump_tokens {
					let column = decoded
						.spans
						.get(field_index)
						.map(|(_, slice, _)| grammar::column_of(raw, slice))
						.unwrap_or_else(|| grammar::column_of(raw, operands));
					jumps.push(PendingJump {
						opcode_index,
						slot: JumpSlot::Instruction,
						line: number,
						column,
						token,
					});
				}
				let mut size = 1usize;
				for field in &fields {
					size += field.size();
				}
				// The records of this instruction arrive on the lines below it, so the slot keeps
				// where they land — the opcode, the field, the item the tokens are recorded on — and
				// the annotations that will name its arms, one per record in order.
				choice_slot = choice_field.map(|field_index| ChoiceSlot {
					opcode_index,
					field_index,
					item_index: out.items.len(),
					spec,
					annotations: std::mem::take(&mut pending.translations),
					next: 0,
				});
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

	// A block that runs to the end of the file is closed here, like one closed by the instruction
	// that follows it.
	close_choice_block(&mut out, choice_slot.take());
	resolve_jumps(&mut out, &jumps);
	check_choice_counts(&mut out);
	// The loader's table is the document's: every operand above already resolved through it.
	out.constants = constants;
	// A constants file's diagnostics stay together and follow the script's, which is the order
	// `render_diagnostics` prints them in.
	out.diagnostics.sort_by_key(|it| (it.source, it.line, it.column));
	out
}

// -- Constants files ------------------------------------------------------------------------------

/// The deepest chain of constants files a document may have open at once.
const INCLUDE_DEPTH: usize = 32;

/// Resolves the `include` annotations of every file a document is made of, and reads each constants
/// file's definitions into one table.
///
/// A file's includes are resolved before its own definitions are read, a path already read is not
/// read again, and a name may be used before it is defined (a name is resolved when an operand is
/// parsed, not when the file is read), so `include` is order-independent and a symbol is visible
/// file-wide. Nothing here knows about an engine: a `.inc` file is data, which is what makes a port
/// to another engine an `.inc` set rather than a code change.
#[derive(Default)]
struct Loader {
	constants: ConstantTable,
	sources: Vec<Source>,
	includes: Vec<Include>,
	loaded: BTreeSet<PathBuf>,
	stack: Vec<PathBuf>,
	errors: Vec<Diagnostic>,
}

impl Loader {
	/// A loader whose first source is the script itself.
	fn new(script: &Utf8Path) -> Self {
		Loader {
			sources: vec![Source { path: script.to_string(), kind: SourceKind::Script }],
			..Loader::default()
		}
	}

	/// Resolves every `include` annotation of the script. The script's own lines are parsed by
	/// [`parse_document`], which is what reports a malformed one; this is what reads the files it
	/// names, each recorded for [`super::print::print_document`] to re-emit.
	fn load_script(&mut self, text: &str, path: &Utf8Path) {
		self.stack.push(key_of(path));
		for (line, target) in include_paths(text) {
			self.includes.push(Include { source: 0, path: target.clone(), line });
			self.include(path, 0, line, &target);
		}
		self.stack.pop();
	}

	/// Resolves one `include` annotation: the file it names becomes a source, and that file's own
	/// includes are read before the caller's next line is considered.
	fn include(&mut self, from: &Utf8Path, source: usize, line: usize, target: &str) {
		if self.stack.len() > INCLUDE_DEPTH {
			self.error(source, line, "include depth exceeds 32".to_owned());
			return;
		}
		let path = resolve_path(from, target);
		let key = key_of(&path);
		if self.stack.contains(&key) {
			let mut chain: Vec<String> =
				self.stack.iter().map(|it| it.display().to_string()).collect();
			chain.push(key.display().to_string());
			self.error(source, line, format!("cyclic include: {}", chain.join(" -> ")));
			return;
		}
		if self.loaded.contains(&key) {
			return;
		}
		let Ok(text) = std::fs::read_to_string(&path) else {
			self.error(source, line, format!("include file not found: \"{target}\""));
			return;
		};
		self.loaded.insert(key.clone());
		let index = self.sources.len();
		self.sources.push(Source { path: path.to_string(), kind: SourceKind::Constants });
		self.stack.push(key);
		self.read_definitions(&text, index, &path);
		self.stack.pop();
	}

	/// A constants file: its magic, then its includes, then its definitions. The shapes are the ones
	/// the scripts are read with, so a constants file and a script agree about what a blank line, an
	/// annotation, a record and an unrecognizable line are.
	fn read_definitions(&mut self, text: &str, source: usize, path: &Utf8Path) {
		let has_content = text.lines().any(|it| !it.trim().is_empty());
		if has_content {
			let first = text.lines().next().unwrap_or_default().trim_end();
			if first != CONST_MAGIC {
				self.error(
					source,
					1,
					format!("line 1 must be exactly \"{CONST_MAGIC}\", found \"{first}\""),
				);
			}
		}
		for (line, target) in include_paths(text) {
			self.includes.push(Include { source, path: target.clone(), line });
			self.include(path, source, line, &target);
		}
		for (index, raw) in text.lines().enumerate() {
			let number = index + 1;
			if has_content && index == 0 {
				continue;
			}
			// A `;` comment is ignored: nothing re-emits a constants file, so it needs no anchor.
			let (code, _) = grammar::code_and_comment(raw);
			let trimmed = code.trim_end();
			match grammar::classify(trimmed) {
				LineShape::Blank => {}
				LineShape::Annotation => {
					// The classifier has seen the `#`, so this cannot fail.
					let Ok((_, (key, value))) = grammar::annotation(trimmed) else {
						continue;
					};
					if key != "include" {
						self.error(
							source,
							number,
							"annotations are not allowed in a constants file".to_owned(),
						);
						continue;
					}
					// The file was resolved above; only a path that is no literal is left to report.
					if grammar::string_value(value).is_err() {
						self.error(
							source,
							number,
							"malformed \"#include\" directive: expected a quoted path".to_owned(),
						);
					}
				}
				LineShape::Directive => self.error(
					source,
					number,
					"directives are not allowed in a constants file".to_owned(),
				),
				LineShape::Instruction => self.define(trimmed, source, number),
				LineShape::Record => self.error(
					source,
					number,
					format!("malformed constant definition: \"{trimmed}\""),
				),
				LineShape::Bad => self.error(
					source,
					number,
					format!("unrecognized line: \"{trimmed}\""),
				),
			}
		}
	}

	/// One `NAME = <literal>` definition. A name that means something else is the only definition a
	/// later one may not replace; a name that already means the same thing is accepted silently, so
	/// two files may share a constant (and two paths to one file may defeat the dedup).
	fn define(&mut self, text: &str, source: usize, line: usize) {
		let malformed = || format!("malformed constant definition: \"{text}\"");
		let Some((name, value)) = text.split_once('=') else {
			self.error(source, line, malformed());
			return;
		};
		let (name, value) = (name.trim(), value.trim());
		// A definition is `NAME = <literal>`: a second `=` means nothing, and neither does anything
		// after the literal.
		if value.contains('=') {
			self.error(source, line, malformed());
			return;
		}
		// `L_<hex>` is the format's address label, so it can never be a constant's name; the rest of
		// the rule is the one identifier every value token is read with.
		if !grammar::is_identifier(name)
			|| matches!(grammar::jump_target(name), Some(JumpTarget::LabelAddress(_)))
		{
			self.error(source, line, format!("invalid constant name \"{name}\""));
			return;
		}
		let declared = match self.literal(text, name, value) {
			Ok(it) => it,
			Err(message) => {
				self.error(source, line, message);
				return;
			}
		};
		if let Some(first) = self.constants.get(name) {
			if self.same_value(first, &declared) {
				return;
			}
			let path = self
				.sources
				.get(first.source)
				.map(|it| it.path.as_str())
				.unwrap_or_default();
			let message = format!(
				"constant \"{name}\" is already defined as {} ({path}:{})",
				render_value(&first.value),
				first.line
			);
			self.error(source, line, message);
			return;
		}
		self.constants.insert(Constant { name: name.to_owned(), value: declared, source, line });
	}

	/// A definition's value: a `0x…` number with the width its digit count declares, an `@0x…`
	/// address, a `"…"` string, or another constant's name. The two messages are the difference
	/// between a broken line and a value that is no literal: anything after a literal makes the line
	/// malformed, while a token that is no literal at all says what was found.
	fn literal(&self, text: &str, name: &str, value: &str) -> Result<ConstantValue, String> {
		let malformed = || format!("malformed constant definition: \"{text}\"");
		let needs = || {
			format!(
				"constant \"{name}\" needs a hex literal, an address, a string or another constant, found \"{value}\""
			)
		};
		if let Some((address, digits)) = split_literal(value) {
			let run: &str = {
				let rest = digits.trim_start_matches(|c: char| c.is_ascii_hexdigit());
				&digits[..digits.len() - rest.len()]
			};
			if run.is_empty() {
				return Err(needs());
			}
			if run.len() != digits.len() {
				return Err(malformed());
			}
			if run.len() > 8 {
				let digits = run.trim_start_matches('0');
				let digits = if digits.is_empty() { "0" } else { digits };
				return Err(format!(
					"constant \"{name}\" value {}{} is wider than 4 bytes",
					if address { "@0x" } else { "0x" },
					digits.to_ascii_uppercase()
				));
			}
			let value = u64::from_str_radix(run, 16).unwrap_or_default();
			return Ok(if address {
				ConstantValue::Address(value as usize)
			} else {
				ConstantValue::Number { value, width: (run.len() as u8).div_ceil(2) }
			});
		}
		if grammar::is_identifier(value) {
			return Ok(ConstantValue::Alias(value.to_owned()));
		}
		// A leading identifier followed by anything else is a definition with junk after its name.
		if value.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') {
			return Err(malformed());
		}
		if value.starts_with('"') {
			if let Ok((_, text)) = grammar::string_value(value) {
				return Ok(ConstantValue::Text(text));
			}
			// A literal that closed and then kept going broke the line; one that never closed is no
			// literal at all.
			return Err(if value[1..].contains('"') { malformed() } else { needs() });
		}
		Err(needs())
	}

	/// Whether a redefinition declares what the name already means, compared after alias resolution:
	/// a shared constant may be written as a literal in one file and as another name in the next.
	fn same_value(&self, first: &Constant, value: &ConstantValue) -> bool {
		let declared = match value {
			ConstantValue::Alias(target) => self.constants.resolve(target).ok(),
			other => other.resolved(),
		};
		match (declared, self.constants.resolve(&first.name)) {
			(Some(declared), Ok(existing)) => declared == existing,
			_ => false,
		}
	}

	fn error(&mut self, source: usize, line: usize, message: String) {
		self.errors.push(Diagnostic::new(source, Severity::Error, line, 1, message));
	}
}

/// Loads a constants file and everything it includes, with no script beside it — how
/// `ccfkb_disassemble` enters the loader. The root file was named on the command line, so it must
/// exist; a file it includes reports a diagnostic instead, which is what makes every problem a
/// positioned message rather than an error out of the middle of a walk.
pub(crate) fn load_constants(
	path: &Utf8Path,
) -> anyhow::Result<(ConstantTable, Vec<Source>, Vec<Diagnostic>)> {
	let input =
		std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
	let mut loader = Loader {
		sources: vec![Source { path: path.to_string(), kind: SourceKind::Constants }],
		..Loader::default()
	};
	loader.stack.push(key_of(path));
	loader.read_definitions(&input, 0, path);
	Ok((loader.constants, loader.sources, loader.errors))
}

/// The `include` annotations of a file, in the order they are written: the line each sits on and the
/// path it names. A path that is no quoted literal is skipped — only the caller knows which file's
/// line the report belongs to, so reporting it is the caller's.
fn include_paths(text: &str) -> Vec<(usize, String)> {
	let mut out = Vec::new();
	for (index, raw) in text.lines().enumerate() {
		let (code, _) = grammar::code_and_comment(raw);
		let trimmed = code.trim_end();
		if !matches!(grammar::classify(trimmed), LineShape::Annotation) {
			continue;
		}
		// The classifier has seen the `#`, so this cannot fail.
		let Ok((_, (key, value))) = grammar::annotation(trimmed) else {
			continue;
		};
		if key != "include" {
			continue;
		}
		if let Ok(target) = grammar::value_of(grammar::string_value(value)) {
			out.push((index + 1, target));
		}
	}
	out
}

/// The path an `include` annotation names: an absolute path is used as it is, a relative one resolves
/// against the directory of the file that wrote it (no search path, no environment variable).
fn resolve_path(from: &Utf8Path, target: &str) -> Utf8PathBuf {
	let target = Utf8PathBuf::from(target);
	match (target.is_absolute(), from.parent()) {
		(true, _) | (false, None) => target,
		(false, Some(dir)) => dir.join(target),
	}
}

/// The key two paths are compared by: the canonical path when the file exists, the path as written
/// when it does not, so a missing file is reported rather than merged with a later one.
fn key_of(path: &Utf8Path) -> PathBuf {
	std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path.as_str()))
}

/// Splits a number or address literal's prefix from its digits: `(is_address, digits)`.
fn split_literal(value: &str) -> Option<(bool, &str)> {
	if let Some(rest) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) {
		return Some((false, rest));
	}
	if let Some(rest) = value.strip_prefix("@0x").or_else(|| value.strip_prefix("@0X")) {
		return Some((true, rest));
	}
	None
}

/// Renders a definition the way the format writes one: `0x` + uppercase hex without padding, `@0x` +
/// eight digits, a quoted string, or an alias's name.
fn render_value(value: &ConstantValue) -> String {
	match value {
		ConstantValue::Number { value, .. } => format!("0x{value:X}"),
		ConstantValue::Address(value) => format!("@0x{value:08X}"),
		ConstantValue::Text(text) => format!("\"{}\"", escape_string(text)),
		ConstantValue::Alias(name) => name.clone(),
	}
}

/// A parser failure as the positioned error a line reports: the column is the character column of the
/// token the parser stopped at.
fn positioned(line: &str, number: usize, failure: AsmError<'_>) -> Diagnostic {
	Diagnostic::new(
		0,
		Severity::Error,
		number,
		grammar::column_of(line, failure.input),
		failure.message,
	)
}

/// A finding a value parser produced while still succeeding, positioned at the token it belongs to
/// (the token is a slice of the line, exactly like an error's).
fn finding_at(line: &str, number: usize, finding: DecodedFlag<'_>) -> Diagnostic {
	Diagnostic::new(
		0,
		Severity::Flag,
		number,
		grammar::column_of(line, finding.input),
		finding.message,
	)
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

/// The payload opcodes as the format spells them, built from the table so a rename cannot leave a
/// message behind.
fn payload_kinds_spelled() -> String {
	Choice::PAYLOAD_KINDS
		.iter()
		.filter_map(|kind| lookup_spec(*kind))
		.map(|spec| format!("{} (0x{:02X})", spec.name, spec.opcode))
		.collect::<Vec<String>>()
		.join(", ")
}

/// One choice record line: the option's `arg1` and text, the availability gate the interpreter
/// evaluates before listing it, and the payload — the byte pattern of another opcode, which the
/// interpreter runs on this record when the option is taken.
///
/// Returns the record, its operand tokens as written (which `ccfkb_asm_fmt` re-emits, so a name or a
/// literal survives formatting exactly as it does on an instruction line) and its payload's jump
/// operands, which the driver resolves once every address is known.
///
/// The record's four fields are written in the order of the bytes and the payload is last, so its
/// operand list runs to the end of the line. The parser reports the first problem it meets, so a
/// value the format rejects keeps its own message; a part with no top-level `:`, a label the record
/// does not define, a field out of order or a missing field are all an unrecognized line.
fn choice_record<'a>(
	raw: &'a str,
	line: usize,
	text: &'a str,
	segments: &[&'a str],
	constants: &'a ConstantTable,
	findings: &mut Vec<DecodedFlag<'a>>,
) -> Result<(Choice, AsmRecord, Vec<(usize, &'a str)>), String> {
	/// The record's own fields, in the order the bytes are: the two the option needs, then the gate.
	const FIELDS: [&str; 4] = ["arg1", "text", "gate_indirect", "gate_value"];
	let unrecognized = || format!("unrecognized line: \"{text}\"");
	// Each part is its label, the operand as written (a payload's operand is passed on whole) and the
	// value token alone.
	let mut parts: Vec<(&'a str, &'a str, &'a str)> = Vec::new();
	for segment in segments.iter().copied() {
		let Ok((_, (label, value))) = grammar::label_value(segment) else {
			return Err(unrecognized());
		};
		parts.push((label, segment.trim(), value));
	}
	let Some(payload_at) = parts.iter().position(|(label, _, _)| *label == "payload") else {
		return Err(unrecognized());
	};
	let head = &parts[..payload_at];
	if head.len() != FIELDS.len()
		|| head.iter().zip(FIELDS).any(|((label, _, _), want)| *label != want)
	{
		return Err(unrecognized());
	}
	let field = |index: usize| head[index].2;
	let arg1 = grammar::value_of(
		grammar::hex_value("arg1", 4, "a 2-byte hex literal", constants, findings).parse(field(0)),
	)? as u16;
	let choice_str =
		grammar::value_of(grammar::string_operand("text", constants).parse(field(1)))?;
	let gate_indirect = grammar::value_of(
		grammar::hex_value("gate_indirect", 2, "a 1-byte hex literal", constants, findings)
			.parse(field(2)),
	)? as u8;
	let gate_value = grammar::value_of(
		grammar::hex_value("gate_value", 4, "a 2-byte hex literal", constants, findings)
			.parse(field(3)),
	)? as u16;

	// The payload is an opcode of its own, so its operand list is that opcode's: the text after its
	// mnemonic, then every field the record spells after `payload:`.
	let payload_text = parts[payload_at].2;
	let (mnemonic, region) =
		grammar::value_of(grammar::instruction(payload_text)).map_err(|_| unrecognized())?;
	let spec = OPCODE_SPECS
		.iter()
		.find(|it| it.name == mnemonic && Choice::PAYLOAD_KINDS.contains(&it.opcode));
	let (payload_kind, payload, payload_tokens, payload_jumps) = match spec {
		Some(spec) => {
			let operands: Vec<&'a str> = std::iter::once(region)
				.chain(parts[payload_at + 1..].iter().map(|(_, operand, _)| *operand))
				.collect();
			if operands.len() != spec.operands.len() {
				return Err(grammar::operand_count_message(spec, operands.len()));
			}
			let labels = operand_labels(spec.operands);
			let mut fields = Vec::with_capacity(spec.layout.len());
			let mut tokens = Vec::with_capacity(spec.layout.len());
			let mut jumps = Vec::new();
			for (index, operand) in operands.iter().enumerate() {
				let (field, jump) = grammar::value_of(grammar::operand_field(
					spec,
					&labels,
					index,
					operand,
					constants,
					findings,
				))?;
				let value = match grammar::label_value(operand) {
					Ok((_, (_, value))) => value,
					Err(_) => operand,
				};
				tokens.push(AsmOperand {
					label: labels[index].clone(),
					text: value.to_owned(),
					line,
					column: grammar::column_of(raw, value),
				});
				if let Some(jump) = jump {
					jumps.push((index, jump));
				}
				fields.push(field);
			}
			(spec.opcode, fields, tokens, jumps)
		}
		None => {
			// A kind the interpreter does not dispatch carries no operand bytes, so it is written as
			// the byte itself and nothing follows it.
			let kind = (region.trim().is_empty() && payload_at + 1 == parts.len())
				.then(|| {
					grammar::value_of(
						grammar::hex_value("payload", 2, "a 1-byte hex literal", constants, findings)
							.parse(mnemonic),
					)
					.ok()
				})
				.flatten();
			match kind {
				Some(kind) if Choice::PAYLOAD_KINDS.contains(&(kind as u8)) => {
					return Err(format!(
						"payload {kind:#04X} names an opcode; write \"{}\"",
						lookup_spec(kind as u8).map(|it| it.name).unwrap_or_default()
					));
				}
				Some(kind) => (kind as u8, Vec::new(), Vec::new(), Vec::new()),
				None => {
					return Err(format!(
						"payload must be {} or a kind byte, found \"{payload_text}\"",
						payload_kinds_spelled()
					));
				}
			}
		}
	};

	let operands: Vec<AsmOperand> = head
		.iter()
		.map(|(label, _, value)| AsmOperand {
			label: (*label).to_owned(),
			text: (*value).to_owned(),
			line,
			column: grammar::column_of(raw, value),
		})
		.collect();
	Ok((
		Choice {
			arg1,
			choice_str: TLString { raw: choice_str, translation: None, notes: None },
			gate_indirect,
			gate_value,
			payload_kind,
			payload,
		},
		AsmRecord { operands, payload: payload_tokens },
		payload_jumps,
	))
}

/// Applies one annotation to the block of annotations the instruction below it will use. A value the
/// format rejects, and a key it does not define, are diagnostics — the block itself still reaches the
/// instruction that follows.
fn parse_annotation(
	out: &mut AsmDocument,
	pending: &mut PendingAnnotations,
	constants: &ConstantTable,
	trailer_line: &mut Option<usize>,
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
		// The file the annotation names was already resolved by the loader, before this line was
		// reached: what is left for a line-at-a-time reader is the one thing the loader skips, a path
		// that is not a literal. A well-formed include reports nothing at all.
		"include" => {
			if grammar::string_value(value).is_err() {
				out.diagnostics.push(error(
					line,
					column,
					"malformed \"#include\" directive: expected a quoted path".to_owned(),
				));
			}
		}
		// File-scope like `include`: the trailer is not part of the block an instruction consumes.
		"trailer" => {
			if let Some(first) = *trailer_line {
				out.diagnostics.push(error(
					line,
					1,
					format!("duplicate \"#trailer\" annotation (first on line {first})"),
				));
				return;
			}
			*trailer_line = Some(line);
			match grammar::value_of(grammar::byte_list_value(value, constants)) {
				Ok(bytes) => out.script.trailer = bytes,
				Err(message) => out.diagnostics.push(error(line, column, message)),
			}
		}
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
			out.diagnostics.push(Diagnostic::new(
				0,
				severity,
				pending.line,
				pending.column,
				message,
			));
		}
		match pending.slot {
			JumpSlot::Instruction => {
				let field_index = if is_jump_field(opcode, 0) { 0 } else { 3 };
				let value = jump_field(opcode, instruction_address, destination);
				if let Some(field) =
					out.script.opcodes[pending.opcode_index].fields.get_mut(field_index)
				{
					*field = OpField::DWord(value);
				}
			}
			JumpSlot::Payload { field_index, choice_index, operand_index } => {
				// A payload jump is an `absolute_jump` written inside a record, so its value is the
				// destination itself: the record has no address of its own to measure from.
				let kind = match &out.script.opcodes[pending.opcode_index].fields[field_index] {
					OpField::Choice(choices) => choices.get(choice_index).map(|it| it.payload_kind),
					_ => None,
				};
				let Some(kind) = kind else { continue };
				let value = jump_field(kind, instruction_address, destination);
				if let Some(OpField::Choice(choices)) =
					out.script.opcodes[pending.opcode_index].fields.get_mut(field_index)
				{
					if let Some(field) = choices
						.get_mut(choice_index)
						.and_then(|choice| choice.payload.get_mut(operand_index))
					{
						*field = OpField::DWord(value);
					}
				}
			}
		}
	}
}

/// Closes a `choices:` block. any annotations that are unmatched to an arm are reported.
fn close_choice_block(out: &mut AsmDocument, slot: Option<ChoiceSlot>) {
	let Some(slot) = slot else {
		return;
	};
	let records = match out
		.script
		.opcodes
		.get(slot.opcode_index)
		.and_then(|it| it.fields.get(slot.field_index))
	{
		Some(OpField::Choice(choices)) => choices.len(),
		_ => 0,
	};
	for (line, column, _) in slot.annotations.iter().skip(slot.next) {
		out.diagnostics.push(flag(
			*line,
			*column,
			format!(
				"\"translation\" annotation on {} (0x{:02X}), which has only {records} choice records",
				slot.spec.name, slot.spec.opcode
			),
		));
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
	Diagnostic::new(0, Severity::Error, line, column, message)
}

fn flag(line: usize, column: usize, message: String) -> Diagnostic {
	Diagnostic::new(0, Severity::Flag, line, column, message)
}

#[cfg(test)]
mod test {
	use super::*;
	use camino::Utf8Path;

	/// Every test parses as if the file were `T.WSC` beside the working directory: no include it
	/// names exists, which is exactly what a fixture without one needs.
	fn parse(text: &str) -> AsmDocument {
		parse_document(text, Utf8Path::new("T.WSC"))
	}

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
		let doc = parse("# cc-fkb asm 1\n\n   nope\nnop_yield pad: 0x00 ; why\n");
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
		let doc = parse(&format!("# cc-fkb asm 1\n{bad}\n"));
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
		let doc = parse(&format!("# cc-fkb asm 1\n{line}\n"));
		assert!(doc.diagnostics.is_empty(), "{:#?}", doc.diagnostics);
		let operands = &doc.items[0].operands;
		assert_eq!(operands.len(), 6, "{operands:?}");
		let operand = &operands[5];
		assert_eq!(operand.label, "text");
		assert_eq!(operand.line, 2);
		assert_eq!(operand.text, "\"うえ\"", "the value token as written is kept");
		let byte_offset = line.find("text: \"うえ\"").expect("the last string operand");
		assert_ne!(operand.column, byte_offset + 1, "a byte offset is not a column");
		assert_eq!(operand.column, line[..byte_offset].chars().count() + 1);

		// A line is parsed on its own, so a bad line after it reports its own number.
		let doc = parse(&format!("# cc-fkb asm 1\n{line}\nnope\n"));
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
		let doc = parse(fixture);
		assert!(doc.has_errors(), "garbage is reported:\n{:#?}", doc.diagnostics);
		// Every one of those lines is noted and the file is still read to the end.
		assert_eq!(doc.items.last().map(|it| it.opcode), Some(0xE6));
	}
}
