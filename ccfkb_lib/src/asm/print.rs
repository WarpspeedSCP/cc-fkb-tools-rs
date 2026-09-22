//! The printer: `Script` (or a parsed [`AsmDocument`]) to annotated assembly. The grammar it emits
//! is normative in `assembly/README.md`.
//!
//! `print → parse → print` is textually identical and `script → asm → script` preserves `opcodes`
//! and `trailer`: the manifest is rebuilt by `manifest_for` on parse, `# addr` is derived from
//! `Opcode::size()`, and a string's `raw` text is what the operand prints (a `translation`, when
//! present, is emitted as its own annotation, so the bytes still come from the model). A choice
//! record prints its gate and its payload, the payload opcode named and valued by its own row of the
//! table — and a token the author wrote (`arg1: SLOT_MAIN`, a payload's `value`) is re-emitted as it
//! stands, exactly as an instruction operand's is.
//!
//! Beside a constants table, the same emitter names values instead of spelling them:
//! [`print_script_with_constants`] is what `ccfkb_disassemble` uses when it is given a `.inc` file,
//! and symbol-named output assembles to the same bytes as canonical output does. The transitive
//! includes of a constants file are *not* re-emitted into the script — a nested path is relative to
//! its own file — only the script's own `#include` lines are.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::bail;

use crate::opcodes::{lookup_spec, Choice, Code, OpField, OpcodeSpecStatic, Script};

use super::grammar::{self, JumpTarget};
use super::{escape_string, is_jump_field, jump_destination, operand_labels, AsmDocument, AsmItem, AsmRecord, Comment, CommentAnchor, ConstantTable, AsmOperand};

/// Line 1 of every script, the only accepted version.
pub const MAGIC: &str = "# cc-fkb asm 1";

/// Line 1 of every constants file, the only accepted version.
pub const CONST_MAGIC: &str = "# cc-fkb inc 1";

/// What a caller brings to [`render`] beside the script itself: the comments to re-attach, whether
/// each instruction carried an `addr` annotation, the operand tokens as the author wrote them, the
/// constants to print names from, and the `include` lines to re-emit.
#[derive(Default, Clone, Copy)]
struct Shape<'a> {
	comments: &'a [Comment],
	annotated: Option<&'a [bool]>,
	tokens: Option<&'a [AsmItem]>,
	constants: Option<&'a ConstantTable>,
	includes: &'a [&'a str],
}

/// Renders a decoded script as annotated assembly, without comments.
pub fn print_script(script: &Script, script_name: &str) -> anyhow::Result<String> {
	render(script, script_name, Shape::default())
}

/// Renders a decoded script with `constants` in scope, naming every value one of them declares and
/// carrying the `include` line that puts the table in scope when this text is read back. The string
/// is emitted as it is given: a path relative to the *output* directory is the caller's business.
pub fn print_script_with_constants(
	script: &Script,
	script_name: &str,
	constants: &ConstantTable,
	include: &str,
) -> anyhow::Result<String> {
	render(
		script,
		script_name,
		Shape { constants: Some(constants), includes: &[include], ..Shape::default() },
	)
}

/// The same emitter with the `;` comments recorded by [`super::parse_document`] re-attached to the
/// instruction they preceded or trailed, every operand printed as its author wrote it, and the
/// script's own `include` lines re-emitted. Used by `ccfkb_asm_fmt`, where instruction indices are
/// stable.
pub fn print_document(doc: &AsmDocument, script_name: &str) -> anyhow::Result<String> {
	// An instruction that carried no `addr` annotation was inserted by hand: keep it unannotated, so
	// the provenance survives formatting.
	let annotated: Vec<bool> = doc.items.iter().map(|it| it.annotated).collect();
	// Only the script's own includes: a path written inside a constants file resolves relative to
	// that file, so re-emitting it here would resolve it against the wrong directory.
	let includes: Vec<&str> = doc
		.includes
		.iter()
		.filter(|it| it.source == 0)
		.map(|it| it.path.as_str())
		.collect();
	render(
		&doc.script,
		script_name,
		Shape {
			comments: &doc.comments,
			annotated: Some(&annotated),
			tokens: Some(&doc.items),
			includes: &includes,
			constants: None,
		},
	)
}

fn render(script: &Script, script_name: &str, shape: Shape<'_>) -> anyhow::Result<String> {
	let addresses = derive_addresses(script);
	let destinations = jump_destinations(script, &addresses);
	// Where each instruction sat in the text against where its sizes put it now. The two differ only
	// when a caller changed the script's sizes (`ccfkb_untransform`, whose translations do), and a
	// jump destination the text spells as an address has to move with them.
	let moved: BTreeMap<usize, usize> = script
		.opcodes
		.iter()
		.map(|it| it.address)
		.zip(addresses.iter().copied())
		.collect();

	let mut blocks: Vec<Vec<String>> = Vec::with_capacity(script.opcodes.len());
	for (index, opcode) in script.opcodes.iter().enumerate() {
		let address = addresses[index];
		let spec = lookup_spec(opcode.opcode).ok_or_else(|| {
			anyhow::anyhow!(
				"instruction at 0x{address:08X} has opcode 0x{:02X}, which the opcode table does not define",
				opcode.opcode
			)
		})?;
		let keep_addr = shape
			.annotated
			.and_then(|flags| flags.get(index))
			.copied()
			.unwrap_or(true);
		// The author's own tokens, when they are this instruction's: a hand-built `Script` has no
		// tokens beside it and one whose operand count does not match the row falls back to canonical.
		let item = shape
			.tokens
			.and_then(|items| items.get(index))
			.filter(|it| it.operands.len() == spec.layout.len());
		blocks.push(print_instruction(
			spec,
			opcode,
			address,
			&moved,
			destinations.contains(&address),
			keep_addr,
			item,
			shape.constants,
		)?);
	}

	// A comment anchored to an instruction whose line did not parse has no block to sit on: those are
	// emitted at the end rather than dropped.
	let comments = shape.comments;
	let orphaned: Vec<&Comment> = comments
		.iter()
		.filter(|it| matches!(it.anchor, CommentAnchor::Trailing(index) if index >= blocks.len()))
		.collect();

	let mut out = String::new();
	out.push_str(MAGIC);
	out.push('\n');
	out.push_str(&format!("# script {script_name}\n"));
	for include in shape.includes {
		out.push_str(&format!("#include \"{}\"\n", escape_string(include)));
	}
	if !script.opcodes.is_empty() {
		out.push('\n');
	}
	for (index, block) in blocks.iter().enumerate() {
		for comment in comments {
			if comment.anchor == CommentAnchor::Before(index) {
				out.push_str(&comment_line(&comment.text));
				out.push('\n');
			}
		}
		let trailing: Vec<&str> = comments
			.iter()
			.filter(|it| it.anchor == CommentAnchor::Trailing(index))
			.map(|it| it.text.as_str())
			.collect();
		let trailing_at = instruction_line(block);
		for (line_index, line) in block.iter().enumerate() {
			out.push_str(line);
			if line_index == trailing_at {
				for text in &trailing {
					if text.is_empty() {
						out.push_str("  ;");
					} else {
						out.push_str(&format!("  ; {text}"));
					}
				}
			}
			out.push('\n');
		}
		if index + 1 < blocks.len() {
			out.push('\n');
		}
	}
	for comment in comments {
		if comment.anchor == CommentAnchor::Before(blocks.len()) {
			out.push('\n');
			out.push_str(&comment_line(&comment.text));
			out.push('\n');
		}
	}
	for comment in &orphaned {
		out.push('\n');
		out.push_str(&comment_line(&comment.text));
		out.push('\n');
	}
	if !blocks.is_empty() {
		out.push('\n');
	}
	out.push_str(&format!("#trailer {}\n", byte_list(&script.trailer)));
	Ok(out)
}

/// The line that spells the instruction itself — the first line that is not an annotation — which is
/// where a trailing comment belongs (appending it to an annotation line would re-anchor it on the
/// next parse, and the file would not be stable under `fmt`).
fn instruction_line(block: &[String]) -> usize {
	block.iter().position(|it| !it.starts_with('#')).unwrap_or(0)
}

fn comment_line(text: &str) -> String {
	if text.is_empty() {
		";".to_owned()
	} else {
		format!("; {text}")
	}
}

/// The address of every instruction, accumulated from the first one by `Opcode::size()` — the same
/// numbering the YAML `address:` field and the `.txt` sidecar tags use.
fn derive_addresses(script: &Script) -> Vec<usize> {
	let mut addresses = Vec::with_capacity(script.opcodes.len());
	let mut address = script.opcodes.first().map(|it| it.address).unwrap_or_default();
	for opcode in &script.opcodes {
		addresses.push(address);
		address += opcode.size();
	}
	addresses
}

/// Every address a jump instruction points at, for `# label` emission.
fn jump_destinations(script: &Script, addresses: &[usize]) -> BTreeSet<usize> {
	let mut out = BTreeSet::new();
	for (opcode, address) in script.opcodes.iter().zip(addresses.iter()) {
		let field = match opcode.opcode {
			0x06 => opcode.fields.first().and_then(dword_of),
			0x01 => opcode.fields.get(3).and_then(dword_of),
			_ => None,
		};
		if let Some(field) = field {
			if let Some(destination) = jump_destination(opcode.opcode, *address, field) {
				out.insert(destination);
			}
		}
		// A choice record's payload is an opcode of its own, so an `absolute_jump` written there names
		// the same kind of destination an instruction's does — and gets the same `# label` line.
		for field in &opcode.fields {
			let OpField::Choice(choices) = field else {
				continue;
			};
			for choice in choices {
				if !is_jump_field(choice.payload_kind, 0) {
					continue;
				}
				if let Some(field) = choice.payload.first().and_then(dword_of) {
					if let Some(destination) = jump_destination(choice.payload_kind, *address, field)
					{
						out.insert(destination);
					}
				}
			}
		}
	}
	out
}

fn print_instruction(
	spec: &OpcodeSpecStatic,
	opcode: &crate::opcodes::Opcode,
	address: usize,
	moved: &BTreeMap<usize, usize>,
	is_destination: bool,
	with_addr: bool,
	item: Option<&AsmItem>,
	constants: Option<&ConstantTable>,
) -> anyhow::Result<Vec<String>> {
	if opcode.fields.len() != spec.layout.len() {
		bail!(
			"instruction at 0x{address:08X} has {} fields but {} declares {}",
			opcode.fields.len(),
			spec.name,
			spec.layout.len()
		);
	}
	let labels = operand_labels(spec.operands);
	let mut lines = Vec::new();
	if with_addr {
		lines.push(format!("# addr 0x{address:08X}"));
	}
	if spec.yields {
		lines.push("# yields".to_owned());
	}
	if is_destination {
		lines.push(format!("# label L_{address:08X}"));
	}
	for line in translation_annotations(&opcode.fields) {
		lines.push(line);
	}

	let mut operands: Vec<String> = Vec::with_capacity(spec.layout.len());
	let mut choice_records: Vec<String> = Vec::new();
	for (index, (code, field)) in spec.layout.iter().zip(opcode.fields.iter()).enumerate() {
		let label = &labels[index];
		if let Code::Choice = code {
			operands.push("choices:".to_owned());
			let OpField::Choice(choices) = field else {
				bail!(
					"instruction at 0x{address:08X} ({}) declares a choice list but the field is not one",
					spec.name
				);
			};
			for (index, choice) in choices.iter().enumerate() {
				choice_records.push(print_choice(
					choice,
					address,
					moved,
					item.and_then(|it| it.records.get(index)),
					constants,
				)?);
			}
			continue;
		}
		// A token the author wrote is re-emitted as it stands — a name or a literal, either case, both
		// spell the same bytes — so `fmt` never rewrites what it was given. A jump destination spelled
		// as an address cannot always stand: see `keep_token` and `jump_destination_in_text`.
		let destination = jump_destination_in_text(spec.opcode, index, opcode.address, field, moved);
		let token = item
			.and_then(|it| it.operands.get(index))
			.filter(|it| keep_token(it.text.as_str(), destination, index, spec.opcode));
		let printed = print_token(
			address,
			constants,
			spec,
			index,
			code,
			field,
			destination,
			token
		)?;
		operands.push(format!("{label}: {printed}"));
	}
	if operands.is_empty() {
		lines.push(spec.name.to_owned());
	} else {
		lines.push(format!("{} {}", spec.name, operands.join(", ")));
	}
	lines.extend(choice_records);
	Ok(lines)
}

/// One choice record: the option's `arg1` and text, the availability gate the interpreter evaluates
/// before listing it, and the payload — the byte pattern of another opcode, which the interpreter
/// runs on this record when the option is taken. A payload opcode's own operand names and canonical
/// values come from its row of the table, so a record reads exactly like an instruction.
///
/// A token the author wrote is re-emitted as it stands; a record of a hand-built script carries no
/// tokens and is rendered canonically.
fn print_choice(
	choice: &Choice,
	address: usize,
	moved: &BTreeMap<usize, usize>,
	record: Option<&AsmRecord>,
	constants: Option<&ConstantTable>,
) -> anyhow::Result<String> {
	let token = |index: usize| {
		record
			.and_then(|it| it.operands.get(index))
			.map(|it| it.text.clone())
	};
	let arg1 = token(0).unwrap_or_else(|| {
		named_number(constants, choice.arg1 as u64, 2, "arg1")
			.unwrap_or_else(|| format!("0x{:04X}", choice.arg1))
	});
	let text = token(1).unwrap_or_else(|| {
		constants
			.and_then(|it| it.name_for_text(&choice.choice_str.raw))
			.map(|it| it.to_owned())
			.unwrap_or_else(|| format!("\"{}\"", escape_string(&choice.choice_str.raw)))
	});
	let gate_indirect = token(2).unwrap_or_else(|| {
		named_number(constants, choice.gate_indirect as u64, 1, "gate_indirect")
			.unwrap_or_else(|| format!("0x{:02X}", choice.gate_indirect))
	});
	let gate_value = token(3).unwrap_or_else(|| {
		named_number(constants, choice.gate_value as u64, 2, "gate_value")
			.unwrap_or_else(|| format!("0x{:04X}", choice.gate_value))
	});
	let mut line = format!(
		"  arg1: {arg1}, text: {text}, gate_indirect: {gate_indirect}, gate_value: {gate_value}, payload: "
	);

	let spec = lookup_spec(choice.payload_kind)
		.filter(|it| Choice::PAYLOAD_KINDS.contains(&it.opcode));
	match spec {
		Some(spec) => {
			if choice.payload.len() != spec.layout.len() {
				bail!(
					"choice payload at 0x{address:08X} has {} fields but {} declares {}",
					choice.payload.len(),
					spec.name,
					spec.layout.len()
				);
			}
			let labels = operand_labels(spec.operands);
			let mut printed = Vec::with_capacity(spec.layout.len());
			for (index, (code, field)) in spec.layout.iter().zip(choice.payload.iter()).enumerate() {
				// A payload's own jump destination is an absolute address like an instruction's, and
				// the record has no address of its own to measure a relative one from.
				let destination =
					jump_destination_in_text(spec.opcode, index, address, field, moved);
				let token = record
					.and_then(|it| it.payload.get(index))
					.filter(|it| keep_token(it.text.as_str(), destination, index, spec.opcode));
				let value = print_token(
					address,
					constants,
					spec,
					index,
					code,
					field,
					destination,
					token
				)?;
				printed.push(format!("{}: {value}", labels[index]));
			}
			line.push_str(&format!("{} {}", spec.name, printed.join(", ")));
		}
		None => {
			// A kind the interpreter does not dispatch carries no operand bytes: it is written as the
			// byte itself, and no opcode names it.
			if !choice.payload.is_empty() {
				bail!(
					"choice payload at 0x{address:08X} has kind 0x{:02X}, which carries no operands, but holds {} fields",
					choice.payload_kind,
					choice.payload.len()
				);
			}
			line.push_str(&format!("0x{:02X}", choice.payload_kind));
		}
	}
	Ok(line)
}

fn print_token(address: usize, constants: Option<&ConstantTable>, spec: &OpcodeSpecStatic, index: usize, code: &Code, field: &OpField, destination: Option<usize>, token: Option<&AsmOperand>) -> anyhow::Result<String> {
	let value = match token {
		Some(operand) => operand.text.clone(),
		None => match destination {
			Some(destination) => print_destination(destination, constants),
			None => print_value(spec, index, code, field, address, constants)?,
		},
	};
	Ok(value)
}

/// One operand, canonically: a literal for every number. With a `constants` table in scope, a value
/// one of its definitions declares prints as that name instead — a 1-byte operand prefers a constant
/// of that width. Padding bytes and choice trailers are structural and are never named.
///
/// A jump destination never reaches this: its caller resolved it through
/// [`jump_destination_in_text`] and printed it with [`print_destination`], because the place it names
/// moves when a caller changes the script's sizes.
fn print_value(
	spec: &OpcodeSpecStatic,
	index: usize,
	code: &Code,
	field: &OpField,
	address: usize,
	constants: Option<&ConstantTable>,
) -> anyhow::Result<String> {
	let label = spec.operands.get(index).copied().unwrap_or_default();
	let mismatched = |what: &str| {
		anyhow::anyhow!(
			"instruction at 0x{address:08X} ({}) declares {what} for operand {index} but holds another kind",
			spec.name
		)
	};
	Ok(match (code, field) {
		(Code::Byte, OpField::Byte(value)) => {
			named_number(constants, *value as u64, 1, label).unwrap_or_else(|| format!("0x{value:02X}"))
		}
		(Code::Word, OpField::Word(value)) => {
			named_number(constants, *value as u64, 2, label).unwrap_or_else(|| format!("0x{value:04X}"))
		}
		(Code::DWord, OpField::DWord(value)) => named_number(constants, *value as u64, 4, label)
			.unwrap_or_else(|| format!("0x{value:08X}")),
		(Code::Str, OpField::String(value)) => constants
			.and_then(|it| it.name_for_text(&value.raw))
			.map(|it| it.to_owned())
			.unwrap_or_else(|| format!("\"{}\"", escape_string(&value.raw))),
		(Code::Padding(size), OpField::Padding(bytes)) => {
			if bytes.len() != *size as usize {
				bail!(
					"instruction at 0x{address:08X} ({}) declares {size} padding bytes but holds {}",
					spec.name,
					bytes.len()
				);
			}
			if *size == 1 {
				format!("0x{:02X}", bytes.first().copied().unwrap_or_default())
			} else {
				byte_list(bytes)
			}
		}
		(Code::Byte, _) => return Err(mismatched("a byte")),
		(Code::Word, _) => return Err(mismatched("a word")),
		(Code::DWord, _) => return Err(mismatched("a dword")),
		(Code::Str, _) => return Err(mismatched("a string")),
		(Code::Padding(_), _) => return Err(mismatched("padding")),
		(Code::Choice, _) => return Err(mismatched("a choice list")),
	})
}

/// The name a number prints as, when a constants table is in scope and a constant that spells this
/// operand's `label` declares the value at this width.
fn named_number(
	constants: Option<&ConstantTable>,
	value: u64,
	width: u8,
	label: &str,
) -> Option<String> {
	constants
		.and_then(|it| it.name_for_number(value, width, label))
		.map(|it| it.to_owned())
}

/// `# translation "…"` for every string operand that carries a translation, and for every choice
/// arm's `text`, which continues the instruction's string sequence in record order. A preceding
/// element without one gets an empty annotation when a later one has text, so the *k*-th annotation
/// keeps naming the *k*-th element of the sequence when the file is parsed again.
fn translation_annotations(fields: &[OpField]) -> Vec<String> {
	let strings: Vec<Option<&str>> = fields
		.iter()
		.flat_map(|it| match it {
			OpField::String(value) => vec![value.translation.as_deref()],
			OpField::Choice(choices) => {
				choices.iter().map(|it| it.choice_str.translation.as_deref()).collect()
			}
			_ => Vec::new(),
		})
		.collect();
	let last = strings.iter().rposition(|it| it.is_some());
	match last {
		None => Vec::new(),
		Some(last) => strings[..=last]
			.iter()
			.map(|it| match it {
				Some(text) => format!("# translation \"{}\"", escape_string(text)),
				None => "# translation \"\"".to_owned(),
			})
			.collect(),
	}
}

/// `[ 0xNN, … ]`, or `[ ]` when empty — the spelling the existing `!Padding [ 0x00 ]` uses too.
pub(crate) fn byte_list(bytes: &[u8]) -> String {
	if bytes.is_empty() {
		return "[ ]".to_owned();
	}
	let items: Vec<String> = bytes.iter().map(|it| format!("0x{it:02X}")).collect();
	format!("[ {} ]", items.join(", "))
}

fn dword_of(field: &OpField) -> Option<u32> {
	match field {
		OpField::DWord(value) => Some(*value),
		_ => None,
	}
}

/// Whether an author's operand token can be re-emitted as it stands.
///
/// Every token can, except a jump destination spelled as an address: `L_<hex>` and `0x…` name a
/// *place*, and the printer may have moved it — `ccfkb_untransform` changes an instruction's size
/// when it carries a translation over, and every later address moves with it. A *name* is resolved
/// from its `# label` line, which moves with the instruction, so it always stands.
fn keep_token(token: &str, destination: Option<usize>, index: usize, opcode: u8) -> bool {
	if !is_jump_field(opcode, index) {
		return true;
	}
	match grammar::jump_target(token) {
		Some(JumpTarget::Name(_)) => true,
		Some(JumpTarget::Address(spelled)) | Some(JumpTarget::LabelAddress(spelled)) => {
			destination == Some(spelled as usize)
		}
		None => false,
	}
}

/// The destination a jump field names, in the numbering the printer is about to write: the field
/// names a place in the numbering the text carries — the address itself for `0x06`, an offset from
/// the instruction for `0x01` — and `moved` says where that place is now. `None` when the operand is
/// no jump destination at all.
fn jump_destination_in_text(
	opcode: u8,
	index: usize,
	text_address: usize,
	field: &OpField,
	moved: &BTreeMap<usize, usize>,
) -> Option<usize> {
	if !is_jump_field(opcode, index) {
		return None;
	}
	let destination =
		dword_of(field).and_then(|it| jump_destination(opcode, text_address, it))?;
	Some(moved.get(&destination).copied().unwrap_or(destination))
}

/// A destination as the text spells one: the name of an address constant that declares it, or the
/// format's own `L_<hex>` label.
fn print_destination(destination: usize, constants: Option<&ConstantTable>) -> String {
	constants
		.and_then(|it| it.name_for_address(destination))
		.map(|it| it.to_owned())
		.unwrap_or_else(|| format!("L_{destination:08X}"))
}
