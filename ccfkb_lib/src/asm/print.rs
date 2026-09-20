//! The printer: `Script` (or a parsed [`AsmDocument`]) to annotated assembly. The grammar it emits
//! is normative in `assembly/README.md`.
//!
//! `print → parse → print` is textually identical and `script → asm → script` preserves `opcodes`
//! and `trailer`: the manifest is rebuilt by `manifest_for` on parse, `# addr` is derived from
//! `Opcode::size()`, and a string's `raw` text is what the operand prints (a `translation`, when
//! present, is emitted as its own annotation, so the bytes still come from the model).

use std::collections::BTreeSet;

use anyhow::bail;

use crate::opcodes::{lookup_spec, Code, OpField, OpcodeSpecStatic, Script};

use super::{
	escape_string, is_jump_field, jump_destination, operand_labels, Comment, CommentAnchor,
	AsmDocument,
};

/// Line 1 of every file, the only accepted version.
pub const MAGIC: &str = "# cc-fkb asm 1";

/// Renders a decoded script as annotated assembly, without comments.
pub fn print_script(script: &Script, script_name: &str) -> anyhow::Result<String> {
	render(script, script_name, &[], None)
}

/// The same emitter with the `;` comments recorded by [`super::parse_document`] re-attached to the
/// instruction they preceded or trailed. Used by `ccfkb_asm_fmt`, where instruction indices are
/// stable.
pub fn print_document(doc: &AsmDocument, script_name: &str) -> anyhow::Result<String> {
	// An instruction that carried no `addr` annotation was inserted by hand: keep it unannotated, so
	// the provenance survives formatting.
	let annotated: Vec<bool> = doc.items.iter().map(|it| it.annotated).collect();
	render(&doc.script, script_name, &doc.comments, Some(&annotated))
}

fn render(
	script: &Script,
	script_name: &str,
	comments: &[Comment],
	annotated: Option<&[bool]>,
) -> anyhow::Result<String> {
	let addresses = derive_addresses(script);
	let destinations = jump_destinations(script, &addresses);

	let mut blocks: Vec<Vec<String>> = Vec::with_capacity(script.opcodes.len());
	for (index, opcode) in script.opcodes.iter().enumerate() {
		let address = addresses[index];
		let spec = lookup_spec(opcode.opcode).ok_or_else(|| {
			anyhow::anyhow!(
				"instruction at 0x{address:08X} has opcode 0x{:02X}, which the opcode table does not define",
				opcode.opcode
			)
		})?;
		let keep_addr = annotated
			.and_then(|flags| flags.get(index))
			.copied()
			.unwrap_or(true);
		blocks.push(print_instruction(
			spec,
			opcode,
			address,
			destinations.contains(&address),
			keep_addr,
		)?);
	}

	// A comment anchored to an instruction whose line did not parse has no block to sit on: those are
	// emitted at the end rather than dropped.
	let orphaned: Vec<&Comment> = comments
		.iter()
		.filter(|it| matches!(it.anchor, CommentAnchor::Trailing(index) if index >= blocks.len()))
		.collect();

	let mut out = String::new();
	out.push_str(MAGIC);
	out.push('\n');
	out.push_str(&format!("# script {script_name}\n"));
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
	out.push_str(&format!(".trailer {}\n", byte_list(&script.trailer)));
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
	}
	out
}

fn print_instruction(
	spec: &OpcodeSpecStatic,
	opcode: &crate::opcodes::Opcode,
	address: usize,
	is_destination: bool,
	with_addr: bool,
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
			for choice in choices {
				if choice.trailer.len() != 11 {
					log::warn!(
						"choice trailer at 0x{address:08X} is {} bytes; the decoder always reads 11",
						choice.trailer.len()
					);
				}
				choice_records.push(format!(
					"  arg1: 0x{:04X}, text: \"{}\", trailer: {}",
					choice.arg1,
					escape_string(&choice.choice_str.raw),
					byte_list(&choice.trailer)
				));
			}
			continue;
		}
		operands.push(format!("{label}: {}", print_value(spec, index, code, field, address)?));
	}
	if operands.is_empty() {
		lines.push(spec.name.to_owned());
	} else {
		lines.push(format!("{} {}", spec.name, operands.join(", ")));
	}
	lines.extend(choice_records);
	Ok(lines)
}

fn print_value(
	spec: &OpcodeSpecStatic,
	index: usize,
	code: &Code,
	field: &OpField,
	address: usize,
) -> anyhow::Result<String> {
	let mismatched = |what: &str| {
		anyhow::anyhow!(
			"instruction at 0x{address:08X} ({}) declares {what} for operand {index} but holds another kind",
			spec.name
		)
	};
	Ok(match (code, field) {
		(Code::Byte, OpField::Byte(value)) => format!("0x{value:02X}"),
		(Code::Word, OpField::Word(value)) => format!("0x{value:04X}"),
		(Code::DWord, OpField::DWord(value)) => {
			if is_jump_field(spec.opcode, index) {
				let destination = jump_destination(spec.opcode, address, *value)
					.ok_or_else(|| anyhow::anyhow!("0x{:02X} is not a jump opcode", spec.opcode))?;
				format!("L_{destination:08X}")
			} else {
				format!("0x{value:08X}")
			}
		}
		(Code::Str, OpField::String(value)) => format!("\"{}\"", escape_string(&value.raw)),
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

/// `# translation "…"` for every string operand that carries a translation. A preceding string
/// operand without one gets an empty annotation when a later one has text, so the *k*-th annotation
/// keeps naming the *k*-th string operand when the file is parsed again.
fn translation_annotations(fields: &[OpField]) -> Vec<String> {
	let strings: Vec<Option<&str>> = fields
		.iter()
		.filter_map(|it| match it {
			OpField::String(value) => Some(value.translation.as_deref()),
			_ => None,
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
