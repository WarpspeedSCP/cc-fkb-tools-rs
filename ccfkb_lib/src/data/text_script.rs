use anyhow::{bail, Context};
use crate::opcodes::{lookup_spec, Choice, Code, OpField, Opcode, Script, TLString};
use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::combinator::{map_res, opt, value};
use nom::multi::{many0, separated_list0};
use nom::sequence::{preceded, terminated};
use nom::IResult;
use nom::{AsChar, Parser};
use once_cell::sync::Lazy;

use crate::util::{escape_str, unescape_str};
use std::fmt::Formatter;

const TL_CHOICE_END: Lazy<String> = Lazy::new(|| "---~~~---".to_string());
const TL_LINE_END: Lazy<String> = Lazy::new(|| "---===---".to_string());

pub fn tl_reverse_transform_script(script: &mut Script, tl_doc: Vec<DocLine>) -> anyhow::Result<()> {
	let mut addr2opcode: HashMap<usize, &mut Opcode> = HashMap::new();
	for opcode in script.opcodes.iter_mut() {
		if ![0x41, 0x42, 0xE0, 0x02].contains(&opcode.opcode) {
			continue;
		}

		match opcode.opcode {
			0x41 | 0x42 | 0x02 | 0xE0 => {
				addr2opcode.insert(opcode.address, opcode);
			}
			_ => {}
		}
	}

	for line in tl_doc.into_iter() {
		match line {
			DocLine::Line(line) => {
				let Some(opcode) = addr2opcode.get_mut(&(line.address as usize)) else {
					bail!("translation line for 0x{:08X} does not match a text opcode in the script", line.address);
				};

				if opcode.opcode == 0x41 {
					match &mut opcode.fields[3] {
						OpField::String(orig_str) => {
							let _ = std::mem::replace(orig_str, line.translation);
						}
						_ => {}
					};
				}
			}
			DocLine::Scene(line) => {
				let Some(opcode) = addr2opcode.get_mut(&(line.address as usize)) else {
					bail!("scene for 0x{:08X} does not match an opcode 0xE0 in the script", line.address);
				};

				if opcode.opcode == 0xE0 {
					match &mut opcode.fields[0] {
						OpField::String(orig_str) => {
							let _ = std::mem::replace(orig_str, line.translation);
						}
						_ => {}
					};
				}
			}
			DocLine::SpeakerLine(line) => {
				let Some(opcode) = addr2opcode.get_mut(&(line.address as usize)) else {
					bail!("speaker line for 0x{:08X} does not match an opcode 0x42 in the script", line.address);
				};
				if opcode.opcode == 0x42 {
					match &mut opcode.fields[4] {
						OpField::String(orig_str) => {
							let _ = std::mem::replace(orig_str, line.speaker_translation);
						}
						_ => {}
					};

					match &mut opcode.fields[5] {
						OpField::String(orig_str) => {
							let _ = std::mem::replace(orig_str, line.translation);
						}
						_ => {}
					};
				}
			}
			DocLine::Choices(choice) => {
				let Some(opcode) = addr2opcode.get_mut(&(choice.address as usize)) else {
					bail!("choices for 0x{:08X} do not match an opcode 0x02 in the script", choice.address);
				};
				if opcode.opcode == 0x02 {
					match &mut opcode.fields[2] {
						OpField::Choice(orig_choices) => {
							orig_choices
								.into_iter()
								.zip(choice.choices.into_iter())
								.for_each(|(orig, new)| {
									let _ = std::mem::replace(&mut orig.choice_str, new);
								});
						}
						_ => {}
					};
				}
			}
		}
	}

	Ok(())
}

pub fn tl_transform_script(input: &Script) -> anyhow::Result<String> {
	let mut lines = vec![];

	for opcode in input.opcodes.iter() {
		if ![0x41, 0x42, 0x02, 0xE0].contains(&opcode.opcode) {
			continue;
		}

		match opcode.opcode {
			0x42 => {
				let address = opcode.address;
				let mut thing = opcode.fields.iter().filter_map(|it| match it {
					OpField::String(it) => Some(it),
					_ => None,
				});

				let speaker_tl_string = thing.next().with_context(|| {
					format!("opcode 0x42 at 0x{address:08X} has no speaker string")
				})?;
				let tl_string = thing.next().with_context(|| {
					format!("opcode 0x42 at 0x{address:08X} has no text string")
				})?;

				let docline = DocLine::SpeakerLine(SpeakerLine {
					speaker_translation: speaker_tl_string.clone(),
					address: address as u32,
					translation: tl_string.clone(),
					speaker_address: address as u32,
				});

				lines.push(docline.to_string());
			}
			// Scene title.
			0xE0 => {
				let address = opcode.address;
				let tl_string = match &opcode.fields[0] {
					OpField::String(orig_str) => orig_str.clone(),
					_ => {
						log::error!("Weird stuff happening to scene!");
						continue;
					}
				};

				let docline = DocLine::Scene(Line {
					translation: tl_string,
					address: address as u32,
				});
				lines.push(docline.to_string());
			}

			// Textbox with no speaker.
			0x41 => {
				let address = opcode.address;
				let tl_string = opcode
					.fields
					.iter()
					.find_map(|it| match it {
						OpField::String(it) => Some(it),
						_ => None,
					})
					.with_context(|| format!("opcode 0x41 at 0x{address:08X} has no string field"))?;

				let docline = DocLine::Line(Line {
					translation: tl_string.clone(),
					address: address as u32,
				});
				lines.push(docline.to_string());
			}
			0x02 => {
				let address = opcode.address;
				let choices = opcode
					.fields
					.iter()
					.find_map(|it| match it {
						OpField::Choice(it) => Some(it),
						_ => None,
					})
					.with_context(|| format!("opcode 0x02 at 0x{address:08X} has no choice list"))?;

				let docline = DocLine::Choices(ChoiceLine {
					address: address as u32,
					choices: choices
						.iter()
						.map(|Choice { choice_str, .. }| choice_str.clone())
						.collect(),
				});
				lines.push(docline.to_string());
			}
			_ => continue,
		}
		lines.push(TL_LINE_END.clone());
		lines.push("\n\n\n".to_string());
	}

	Ok(lines.join(""))
}

pub fn is_digit_a(c: char) -> bool {
	c.is_digit(10)
}

pub fn hex_int(input: &str) -> IResult<&str, u32> {
	map_res(take_while(is_digit_a), |it: &str| {
		u32::from_str_radix(it, 10)
	})
		.parse(input)
}

/// An address as the sidecar writes it: `0x` and hex digits, the spelling `tl_transform_script`
/// emits (`[original text @ 0x000003C3]`). `hex_int` is the decimal, prefix-less form the BMP file
/// names use, so the two are separate parsers.
pub fn sidecar_address(input: &str) -> IResult<&str, u32> {
	map_res(
		preceded(tag("0x"), take_while(|c: char| c.is_ascii_hexdigit())),
		|it: &str| u32::from_str_radix(it, 16),
	)
		.parse(input)
}

pub enum TLTag {
	Scene { address: u32 },
	Speaker { address: u32 },
	Text { address: u32 },
	Choice { address: u32 },
}

pub fn tltag(input: &str) -> IResult<&str, TLTag> {
	map_res(
		alt((
			terminated(
				(value("text", tag("[original text @ ")), sidecar_address),
				tag("]:"),
			),
			terminated((value("speaker", tag("[speaker @ ")), sidecar_address), tag("]:")),
			terminated((value("choice", tag("[choices @ ")), sidecar_address), tag("]")),
			terminated((value("scene", tag("[scene title @ ")), sidecar_address), tag("]:")),
		)),
		|(enum_thing, address)| match enum_thing {
			"text" => Ok(TLTag::Text { address }),
			"speaker" => Ok(TLTag::Speaker { address }),
			"choice" => Ok(TLTag::Choice { address }),
			"scene" => Ok(TLTag::Scene { address }),
			_ => Err("Bad input."),
		},
	)
		.parse(input)
}

#[derive(Default, Debug)]
pub struct Line {
	address: u32,
	translation: TLString,
}

#[derive(Default, Debug)]
pub struct SpeakerLine {
	address: u32,
	speaker_address: u32,
	speaker_translation: TLString,
	translation: TLString,
}

#[derive(Default, Debug)]
pub struct ChoiceLine {
	address: u32,
	choices: Vec<TLString>,
}

/// The opcodes a sidecar entry can describe. A `[scene title @ …]` entry describes `0xE0`, a
/// `[original text @ …]` one `0x41`, a `[speaker @ …]` one `0x42`, and a `[choices @ …]` one `0x02`.
const TEXT_OPCODES: [u8; 4] = [0x41, 0x42, 0x02, 0xE0];

/// One instruction a sidecar entry can describe: its opcode, its address, and the raw texts the entry
/// has to carry — the instruction's whole string sequence, which is what `print_document` reads its
/// `translation` annotations from.
struct TextInstruction {
	opcode: u8,
	address: usize,
	raws: Vec<String>,
}

impl TextInstruction {
	fn mnemonic(&self) -> &'static str {
		lookup_spec(self.opcode).map(|it| it.name).unwrap_or("?")
	}
}

/// Every instruction a sidecar entry can describe, in file order.
fn text_instructions(script: &Script) -> Vec<TextInstruction> {
	let mut out = Vec::new();
	for opcode in &script.opcodes {
		if !TEXT_OPCODES.contains(&opcode.opcode) {
			continue;
		}
		let Some(spec) = lookup_spec(opcode.opcode) else {
			continue;
		};
		let mut raws = Vec::new();
		for (index, code) in spec.layout.iter().enumerate() {
			match code {
				Code::Str => {
					if let Some(OpField::String(text)) = opcode.fields.get(index) {
						raws.push(text.raw.clone());
					}
				}
				// A choice list continues the sequence, exactly as it continues the annotations.
				Code::Choice => {
					if let Some(OpField::Choice(choices)) = opcode.fields.get(index) {
						raws.extend(choices.iter().map(|it| it.choice_str.raw.clone()));
					}
				}
				_ => {}
			}
		}
		out.push(TextInstruction { opcode: opcode.opcode, address: opcode.address, raws });
	}
	out
}

/// The opcode an entry describes: the tag it was written under decides it.
fn entry_opcode(entry: &DocLine) -> u8 {
	match entry {
		DocLine::Line(_) => 0x41,
		DocLine::SpeakerLine(_) => 0x42,
		DocLine::Choices(_) => 0x02,
		DocLine::Scene(_) => 0xE0,
	}
}

/// What an entry calls itself in a diagnostic.
fn entry_kind(entry: &DocLine) -> &'static str {
	match entry {
		DocLine::Line(_) => "text",
		DocLine::SpeakerLine(_) => "speaker",
		DocLine::Choices(_) => "choice list",
		DocLine::Scene(_) => "scene title",
	}
}

/// The raw texts an entry carries, in the order the writer emits them.
fn entry_raws(entry: &DocLine) -> Vec<&str> {
	match entry {
		DocLine::Line(line) | DocLine::Scene(line) => vec![line.translation.raw.as_str()],
		DocLine::SpeakerLine(line) => {
			vec![line.speaker_translation.raw.as_str(), line.translation.raw.as_str()]
		}
		DocLine::Choices(line) => line.choices.iter().map(|it| it.raw.as_str()).collect(),
	}
}

fn entry_address(entry: &DocLine) -> usize {
	match entry {
		DocLine::Line(line) | DocLine::Scene(line) => line.address as usize,
		DocLine::SpeakerLine(line) => line.address as usize,
		DocLine::Choices(line) => line.address as usize,
	}
}

fn set_entry_address(entry: &mut DocLine, address: usize) {
	match entry {
		DocLine::Line(line) | DocLine::Scene(line) => line.address = address as u32,
		DocLine::SpeakerLine(line) => {
			line.address = address as u32;
			line.speaker_address = address as u32;
		}
		DocLine::Choices(line) => line.address = address as u32,
	}
}

/// Pairs every entry of a text file with the instruction it describes, and moves the entry there.
///
/// The pairing is by kind and position — the *k*-th text entry belongs to the *k*-th text
/// instruction — and the raw texts are the witness: an entry whose texts are not that instruction's is
/// refused, never applied to a neighbour. That is what lets a text written against an older layout
/// still be applied, e.g. after an instruction was inserted by hand, and it is why the addresses in
/// the text cannot be their own authority: they are what the pairing corrects. Returns how many
/// entries moved.
pub fn repoint_entries(doclines: &mut [DocLine], script: &Script) -> anyhow::Result<usize> {
	let instructions = text_instructions(script);
	let mut cursors: HashMap<u8, usize> = HashMap::new();
	let mut moved = 0;
	for entry in doclines.iter_mut() {
		let opcode = entry_opcode(entry);
		let (kind, found) = (entry_kind(entry), entry_raws(entry));
		let sequence: Vec<&TextInstruction> =
			instructions.iter().filter(|it| it.opcode == opcode).collect();
		let index = cursors.entry(opcode).or_default();
		let Some(instruction) = sequence.get(*index) else {
			bail!(
				"the {kind} at 0x{:08X} has no {} (0x{opcode:02X}) left to be applied to",
				entry_address(entry),
				lookup_spec(opcode).map(|it| it.name).unwrap_or("instruction")
			);
		};
		let want: Vec<&str> = instruction.raws.iter().map(String::as_str).collect();
		if found != want {
			bail!(
				"the {kind} at 0x{:08X} holds {found:?} but the {} (0x{opcode:02X}) at 0x{:08X} holds {want:?}; the text and the script disagree about it",
				entry_address(entry),
				instruction.mnemonic(),
				instruction.address
			);
		}
		if instruction.address != entry_address(entry) {
			moved += 1;
		}
		set_entry_address(entry, instruction.address);
		*index += 1;
	}
	// Entries after a gap would otherwise pair with the instruction after the one they describe.
	for (opcode, count) in &cursors {
		let total = instructions.iter().filter(|it| it.opcode == *opcode).count();
		if *count != total {
			bail!(
				"the text describes {count} {} (0x{opcode:02X}) instruction(s) but the script has {total}",
				lookup_spec(*opcode).map(|it| it.name).unwrap_or("?")
			);
		}
	}
	for instruction in &instructions {
		if !cursors.contains_key(&instruction.opcode) {
			bail!(
				"{} (0x{:02X}) at 0x{:08X} has no entry in the text",
				instruction.mnemonic(),
				instruction.opcode,
				instruction.address
			);
		}
	}
	Ok(moved)
}

/// The `0x` prefix of a tag's address and the text after its digits: the tags are `[key @ 0x{8
/// digits}]` and `[key @ 0x{8 digits}]:`, and only the digits are ever rewritten.
fn split_tag(body: &str) -> (&str, Option<&str>, &str) {
	let Some(at) = body.find("0x") else {
		return (body, None, "");
	};
	let digits_at = at + 2;
	let count = body[digits_at..].chars().take_while(|it| it.is_ascii_hexdigit()).count();
	(&body[..digits_at], Some(&body[digits_at..digits_at + count]), &body[digits_at + count..])
}

/// Rewrites the `@ 0x…` address of every entry tag in a text file, in the order the tags appear, and
/// leaves every other byte of the file as it was. `addresses` is one address per entry, in file order;
/// a speaker's two tags name the same entry, so both take its address. Returns the text and how many
/// tags moved.
pub fn relabel_tags(text: &str, addresses: &[usize]) -> (String, usize) {
	let mut out = String::with_capacity(text.len());
	let mut next = addresses.iter();
	let mut speaker: Option<usize> = None;
	let mut moved = 0usize;
	for chunk in text.split_inclusive('\n') {
		let body = chunk.strip_suffix('\n').unwrap_or(chunk);
		let tail = &chunk[body.len()..];
		let address = if body.starts_with("[speaker @ ") {
			let address = next.next().copied();
			speaker = address;
			address
		} else if body.starts_with("[original text @ ") {
			speaker.take().or_else(|| next.next().copied())
		} else if body.starts_with("[scene title @ ") || body.starts_with("[choices @ ") {
			next.next().copied()
		} else {
			None
		};
		if let Some(address) = address {
			let (head, digits, rest) = split_tag(body);
			if let Some(digits) = digits.filter(|it| it.len() == 8) {
				let renumbered = format!("{address:08X}");
				if renumbered != digits {
					moved += 1;
					out.push_str(head);
					out.push_str(&renumbered);
					out.push_str(rest);
					out.push_str(tail);
					continue;
				}
			}
		}
		out.push_str(chunk);
	}
	(out, moved)
}

/// The text file's own text with every entry renumbered to the instructions `script` holds, and how
/// many tags moved. `0` means the text already named them.
pub fn renumber_entries(text: &str, script: &Script) -> anyhow::Result<(String, usize)> {
	let (_, mut doclines) = parse_doclines(text)
		.map_err(|err| anyhow::anyhow!("parsing the translated script text: {err}"))?;
	repoint_entries(&mut doclines, script)?;
	let addresses: Vec<usize> = doclines.iter().map(entry_address).collect();
	Ok(relabel_tags(text, &addresses))
}

#[derive(Debug)]
pub enum DocLine {
	Line(Line),
	SpeakerLine(SpeakerLine),
	Choices(ChoiceLine),
	Scene(Line),
}

impl std::fmt::Display for DocLine {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DocLine::Scene(Line {
											 address,
											 translation:
											 TLString {
												 translation: tl_text,
												 raw,
												 notes: note_text,
											 },
										 }) => {
				let translation = tl_text.as_ref().map(|it| it.as_str().trim()).unwrap_or_default();
				let notes = note_text.as_ref().map(|it| it.as_str().trim()).unwrap_or_default();

				write!(f, "[scene title @ 0x{address:08X}]: {raw}\n")?;
				write!(f, "[translation]: {translation}\n")?;
				write!(f, "[notes]: {notes}\n")
			}
			DocLine::Line(Line {
											address,
											translation:
											TLString {
												translation: tl_text,
												raw,
												notes: note_text,
											},
										}) => {
				let translation = tl_text.as_ref().map(|it| it.as_str().trim()).unwrap_or_default();
				let notes = note_text.as_ref().map(|it| it.as_str().trim()).unwrap_or_default();

				write!(f, "[original text @ 0x{address:08X}]: {raw}\n")?;
				write!(f, "[translation]: {translation}\n")?;
				write!(f, "[notes]: {notes}\n")
			}
			DocLine::SpeakerLine(SpeakerLine {
														 speaker_translation:
														 TLString {
															 raw: speaker_raw,
															 translation: speaker_translation,
															 ..
														 },
														 address,
														 speaker_address,
														 translation:
														 TLString {
															 translation: tl_text,
															 raw,
															 notes: note_text,
														 },
													 }) => {
				let speaker_tl_text = speaker_translation
					.as_ref()
					.map(|it| it.as_str())
					.unwrap_or_default();
				let translation = tl_text.as_ref().map(|it| it.as_str().trim()).unwrap_or_default();
				let notes = note_text.as_ref().map(|it| it.as_str().trim()).unwrap_or_default();

				write!(f, "[speaker @ 0x{address:08X}]: {speaker_tl_text} ({speaker_raw})\n")?;
				write!(f, "[original text @ 0x{speaker_address:08X}]: {raw}\n")?;
				write!(f, "[translation]: {translation}\n")?;
				write!(f, "[notes]: {notes}\n")
			}
			DocLine::Choices(ChoiceLine { address, choices }) => {
				write!(f, "[choices @ 0x{address:08X}]\n")?;
				for TLString {
					raw,
					notes,
					translation,
				} in choices.iter()
				{
					let raw = unescape_str(raw);
					let tl_text = translation
						.as_ref()
						.map(|it| unescape_str(it.as_str().trim()))
						.unwrap_or_default();
					let note_text = notes
						.as_ref()
						.map(|it| unescape_str(it.as_str().trim()))
						.unwrap_or_default();
					write!(f, "[choice original text]: {raw}\n")?;
					write!(f, "[choice translation]: {tl_text}\n")?;
					write!(f, "[choice notes]: {note_text}\n")?;
					write!(f, "{}\n\n", TL_CHOICE_END.clone())?;
				}
				Ok(())
			}
		}
	}
}

// impl DocLine {
//   fn line_type_string(&self) -> &str {
//     match self {
//       DocLine::Scene(_) => "Scene",
//       DocLine::Line(_) => "Line",
//       DocLine::SpeakerLine(_) => "SpeakerLine",
//       DocLine::Choices(_) => "Choices",
//     }
//   }
//
//   fn address(&self) -> u32 {
//     match self {
//       DocLine::Scene(Line { address, .. }) => *address,
//       DocLine::Line(Line { address, .. }) => *address,
//       DocLine::SpeakerLine(SpeakerLine { address, .. }) => *address,
//       DocLine::Choices(ChoiceLine { address, .. }) => *address,
//     }
//   }
// }

/// Splits the tag from the value of a `[tag]: value` line: the writer separates them with exactly
/// one space, so that space is the separator and everything after it is data. A raw text may open
/// with a full-width space (U+3000, which `trim` would destroy), so the value is kept verbatim.
fn value_after_separator(text: &str) -> &str {
	text.strip_prefix(' ').unwrap_or(text)
}

fn is_blank(input: &str) -> bool {
	input.is_empty() || input.chars().all(|it| it.is_space() || it.is_newline())
}

pub fn parse_docline_group(input: &str) -> IResult<&str, DocLine> {
	let (rest, (tl_tag, header_contents)) = (tltag, take_until("\n[")).parse(input)?;

	let (rest, mut docline) = match tl_tag {
		TLTag::Speaker { address } => {
			let (_, (tl, raw)) = map_res(
				(take_until("("), tag("("), take_until(")"), tag(")")),
				|it| Ok::<(&str, &str), &str>((it.0, it.2)),
			)
				.parse(header_contents)?;

			let mut this_line = SpeakerLine::default();
			this_line.speaker_address = address;

			this_line.speaker_translation = TLString {
				translation: if is_blank(tl) {
					None
				} else {
					Some(tl.trim().to_string())
				},
				notes: None,
				raw: raw.to_string(),
			};

			let (rest, _) = take_until("[").parse(rest)?;

			if let (rest, (TLTag::Text { address: text_addr }, raw)) =
				(tltag, take_until("\n[")).parse(rest)?
			{
				this_line.address = text_addr;

				if !is_blank(raw) {
					this_line.translation.raw = value_after_separator(raw).to_string();
				}

				(rest, DocLine::SpeakerLine(this_line))
			} else {
				(rest, DocLine::SpeakerLine(this_line))
			}
		}
		TLTag::Text { address } => {
			let mut textline = Line::default();
			textline.address = address;

			let (rest, _) = take_until("\n[").parse(rest)?;

			if !is_blank(header_contents) {
				textline.translation.raw = value_after_separator(header_contents).to_string();
			}

			(rest, DocLine::Line(textline))
		}
		TLTag::Scene { address } => {
			let mut textline = Line::default();
			textline.address = address;

			let (rest, _) = take_until("\n[").parse(rest)?;

			if !is_blank(header_contents) {
				textline.translation.raw = value_after_separator(header_contents).to_string();
			}

			(rest, DocLine::Scene(textline))
		}
		TLTag::Choice { address } => {
			let mut choiceline = ChoiceLine::default();
			choiceline.address = address;

			let (rest, stuff) = many0(terminated(
				(
					// The writer puts a blank line between two arms' blocks, so every arm but the
					// first is introduced by `\n\n[choice original text]:`.
					preceded(
						(tag("\n"), opt(tag("\n")), tag("[choice original text]:")),
						take_until("\n["),
					),
					preceded(tag("\n[choice translation]:"), take_until("\n[")),
					preceded(tag("\n[choice notes]:"), take_until(TL_CHOICE_END.as_str())),
				),
				tag(TL_CHOICE_END.as_str()),
			)).parse(rest)?;

			for (raw, choice_tl, choice_notes) in stuff {
				let translation = if is_blank(choice_tl) {
					None
				} else {
					Some(escape_str(choice_tl.trim(), false))
				};

				let notes = if is_blank(choice_notes) {
					None
				} else {
					Some(choice_notes.trim().to_string())
				};

				choiceline.choices.push(TLString {
					raw: value_after_separator(raw).to_string(),
					translation,
					notes,
				});
			}

			// The last arm's block is followed by a blank line and then the line end the writer
			// appends to the entry.
			let (rest, _) =
				preceded(take_while(|c: char| c == '\n'), tag(TL_LINE_END.as_str())).parse(rest)?;
			return Ok((rest, DocLine::Choices(choiceline)));
		}
	};

	let (rest, (tl, notes)) = terminated(
		alt((
			(
				preceded(tag("\n[translation]:"), take_until("\n[")),
				preceded(tag("\n[notes]:"), take_until(TL_LINE_END.as_str())),
			),
			value(("", ""), tag("\n")),
		)),
		tag(TL_LINE_END.as_str()),
	)
		.parse(rest)?;

	if !is_blank(tl) {
		match docline {
			DocLine::Line(ref mut line) => {
				line.translation.translation = Some(escape_str(tl.trim(), true));
			}
			DocLine::SpeakerLine(ref mut line) => {
				line.translation.translation = Some(escape_str(tl.trim(), true));
			}
			DocLine::Scene(ref mut line) => {
				line.translation.translation = Some(escape_str(tl.trim(), false));
			}
			_ => {}
		}
	}

	if !is_blank(notes) {
		match docline {
			DocLine::Line(ref mut line) => {
				line.translation.notes = Some(notes.trim().to_string());
			}
			DocLine::SpeakerLine(ref mut line) => {
				line.translation.notes = Some(notes.trim().to_string());
			}
			DocLine::Scene(ref mut line) => {
				line.translation.notes = Some(notes.trim().to_string());
			}
			_ => {}
		}
	}

	Ok((rest, docline))
}

pub fn parse_doclines(input: &str) -> IResult<&str, Vec<DocLine>> {
	separated_list0(take_until("["), parse_docline_group).parse(input)
}
