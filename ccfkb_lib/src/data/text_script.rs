use std::collections::HashMap;
use crate::opcodes::{Choice, OpField, Opcode, Script, TLString};

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::combinator::{map_res, value};
use nom::multi::{many0, separated_list0};
use nom::sequence::{preceded, terminated};
use nom::{AsChar, Parser};
use once_cell::sync::Lazy;

use std::fmt::Formatter;
use crate::util::{escape_str, unescape_str};

const TL_CHOICE_END: Lazy<String> = Lazy::new(|| "---~~~---".to_string());
const TL_LINE_END: Lazy<String> = Lazy::new(|| "---===---".to_string());

pub fn tl_reverse_transform_script(script: &mut Script, tl_doc: Vec<DocLine>) {
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
        let opcode = addr2opcode.get_mut(&(line.address as usize)).unwrap();

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
        let opcode = addr2opcode.get_mut(&(line.address as usize)).unwrap();

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
        let opcode = addr2opcode.get_mut(&(line.address as usize)).unwrap();
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
        let opcode = addr2opcode.get_mut(&(choice.address as usize)).unwrap();
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
}

pub fn tl_transform_script(input: &Script) -> String {
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

        let speaker_tl_string = thing.next().unwrap();
        let tl_string = thing.next().unwrap();

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
            continue
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
            .unwrap();

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
            .unwrap();

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

  lines.join("")
}

fn is_hex_digit_a(c: char) -> bool {
  c.is_digit(16) || c == 'x'
}

pub fn hex_int(input: &str) -> IResult<&str, u32> {
  map_res(take_while(is_hex_digit_a), |it: &str| {
    u32::from_str_radix(it.trim_start_matches("0x"), 16)
  })
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
        (value("text", tag("[original text @ ")), hex_int),
        tag("]:"),
      ),
      terminated((value("speaker", tag("[speaker @ ")), hex_int), tag("]:")),
      terminated((value("choice", tag("[choices @ ")), hex_int), tag("]")),
      terminated((value("scene", tag("[scene title @ ")), hex_int), tag("]:")),
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

impl DocLine {
  fn line_type_string(&self) -> &str {
    match self {
      DocLine::Scene(_) => "Scene",
      DocLine::Line(_) => "Line",
      DocLine::SpeakerLine(_) => "SpeakerLine",
      DocLine::Choices(_) => "Choices",
    }
  }

  fn address(&self) -> u32 {
    match self {
      DocLine::Scene(Line { address, .. }) => *address,
      DocLine::Line(Line { address, .. }) => *address,
      DocLine::SpeakerLine(SpeakerLine { address, .. }) => *address,
      DocLine::Choices(ChoiceLine { address, .. }) => *address,
    }
  }
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
        raw: raw.trim().to_string(),
      };

      let (rest, _) = take_until("[").parse(rest)?;

      if let (rest, (TLTag::Text { address: text_addr }, raw)) =
          (tltag, take_until("\n[")).parse(rest)?
      {
        this_line.address = text_addr;

        if !is_blank(raw) {
          this_line.translation.raw = raw.trim().to_string();
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
        textline.translation.raw = header_contents.trim().to_string();
      }

      (rest, DocLine::Line(textline))
    }
    TLTag::Scene { address } => {
      let mut textline = Line::default();
      textline.address = address;

      let (rest, _) = take_until("\n[").parse(rest)?;

      if !is_blank(header_contents) {
        textline.translation.raw = header_contents.trim().to_string();
      }

      (rest, DocLine::Scene(textline))
    }
    TLTag::Choice { address } => {
      let mut choiceline = ChoiceLine::default();
      choiceline.address = address;

      let (rest, stuff) = many0(terminated(
        (
          preceded(tag("\n[choice original text]:"), take_until("\n[")),
          preceded(tag("\n[choice translation]:"), take_until("\n[")),
          preceded(tag("\n[choice notes]:"), take_until(TL_CHOICE_END.as_str())),
        ),
        (tag(TL_CHOICE_END.as_str()), alt((take_until("\n["), take_until(TL_LINE_END.as_str())))),
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
          raw: raw.trim().to_string(),
          translation,
          notes,
        });
      }

      let (rest, _) = tag(TL_LINE_END.as_str()).parse(rest)?;
      return Ok((rest, DocLine::Choices(choiceline)))
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
