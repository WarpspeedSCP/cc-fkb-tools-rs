use std::collections::HashMap;
use crate::opcodes::{Choice, OpField, Script, TLString, make_opcode, Opcode};
use crate::util::{
  encode_sjis, get_sjis_bytes, get_sjis_bytes_of_length, to_bytes, transmute_to_u32, unescape_str,
  unwipf,
};
use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::combinator::{map_res, value};
use nom::multi::{many0, separated_list0};
use nom::sequence::{preceded, terminated};
use nom::{AsChar, Parser};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::path::{Path, PathBuf};

#[repr(C, packed)]
pub struct WIPFHeader {
  signature: [u8; 4],
  n_entries: u16,
  depth: u16,
}

impl WIPFHeader {
  fn from_ref(slice: &[u8]) -> &Self {
    if slice.len() < size_of::<Self>() {
      panic!("bad input slice for wipfheader!");
    } else {
      unsafe {
        let data = slice.as_ptr();
        &*(data as *const Self)
      }
    }
  }
}

#[repr(C, packed)]
pub struct BMPHeader {
  magic: [u8; 2],
  filesz: u32,
  res1: u16,
  res2: u16,
  offset: u32,
}

#[repr(C, packed)]
pub struct BMPDibV3Header {
  header_sz: u32,
  width: u32,
  height: u32,
  nplanes: u16,
  depth: u16,
  compress_type: u32,
  bmp_bytesz: u32,
  hres: u32,
  vres: u32,
  ncolors: u32,
  nimpcolors: u32,
}

#[repr(C, packed)]
pub struct WIPFENTRY {
  width: u32,     // unsigned long  width;    // ����
  height: u32,    // unsigned long  height;   // �߶�
  x_offset: u32,  // unsigned long  offset_x; // x������ʾλ��
  y_offset: u32,  // unsigned long  offset_y; // y������ʾλ��
  unk_layer: u32, // unsigned long  unknown1; // layer?
  length: u32,    // unsigned long  length;   // �ļ�����
}

impl WIPFENTRY {
  fn from_ref(slice: &[u8]) -> &Self {
    if slice.len() < size_of::<Self>() {
      panic!("bad input slice for wipfentry!");
    } else {
      unsafe {
        let data = slice.as_ptr();
        &*(data as *const Self)
      }
    }
  }

  fn from_ref_as_slice(slice: &[u8], count: usize) -> &[Self] {
    if slice.len() < (size_of::<Self>() * count) {
      panic!("Bad input slice for wipf entry array!");
    } else {
      unsafe {
        let data = slice.as_ptr() as *const Self;
        &*core::ptr::slice_from_raw_parts(data, count)
      }
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct ExtensionDescriptor {
  name: String,
  number: u32,
  offset: u32,
}

impl ExtensionDescriptor {
  fn new(name: String, number: u32, offset: u32) -> Self {
    Self {
      name,
      number,
      offset,
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct FileDescriptor {
  name: String,
  size: usize,
  offset: usize,
}

impl FileDescriptor {
  fn new(name: String, size: u32, offset: u32) -> Self {
    Self {
      name,
      size: size as usize,
      offset: offset as usize,
    }
  }
}

pub fn extract_all_arcs() {
  std::fs::create_dir_all("unpacked_arcs").unwrap();
  let files = walkdir::WalkDir::new("C:/Users/warps/Desktop/cross-channel-fkb")
    .into_iter()
    .filter_map(|it| it.ok())
    .collect::<Vec<_>>();
  for i in files.iter() {
    let dirent = i;
    let pth = dirent.path();
    if !pth
      .extension()
      .map(|it| it.to_string_lossy().to_owned())
      .unwrap_or_default()
      .ends_with("arc")
    {
      continue;
    }
    let mut file = std::fs::read(dirent.path()).unwrap();

    let path = PathBuf::from("unpacked_arcs").join(dirent.file_name());
    std::fs::create_dir_all(&path).unwrap();
    read_arc(&mut file[..], path, false);
  }
}

// WIPF�ں��ļ�ϸ��

pub fn write_arc(input_folder: &Path) -> Vec<u8> {
  let mut output = vec![];

  let extensions = std::fs::read_to_string(input_folder.join("extensions.yaml"))
    .iter()
    .flat_map(|str| serde_yml::from_str::<Vec<ExtensionDescriptor>>(&str))
    .next()
    .unwrap_or_default();
  let files = std::fs::read_to_string(input_folder.join("files.yaml"))
    .iter()
    .flat_map(|str| serde_yml::from_str::<Vec<(String, FileDescriptor)>>(&str))
    .next()
    .unwrap_or_default();

  output.extend((extensions.len() as u32).to_le_bytes());

  for descriptor in extensions {
    output.extend(encode_sjis(&descriptor.name));
    output.push(0);
    output.extend(&descriptor.number.to_le_bytes());
    output.extend(&descriptor.offset.to_le_bytes());
  }

  let mut things_to_append = vec![];
  let mut curr_offset = output.len() + (13 + 4 + 4) * files.len(); // the size of a file descriptor.

  for (filename, descriptor) in files {
    let mut sjis_name = encode_sjis(&descriptor.name);
    let sjis_name = if sjis_name.len() < 13 {
      sjis_name.extend(vec![0u8; 13 - sjis_name.len()]);
      sjis_name
    } else {
      sjis_name
    };
    output.extend(sjis_name);
    let curr_path = input_folder.join(&filename);
    let mut contents = std::fs::read(&curr_path).unwrap();

    if filename.ends_with("WSC") {
      rotate_wsc_for_pack(&mut contents)
    }

    output.extend(&(contents.len() as u32).to_le_bytes());
    output.extend(&(curr_offset as u32).to_le_bytes());
    curr_offset += contents.len();

    things_to_append.push(contents);
  }

  things_to_append.iter().for_each(|it| output.extend(it));

  output
}

pub fn read_arc(input: &mut [u8], out_folder: PathBuf, extract_wipf: bool) -> () {
  let n_ext_descriptors = transmute_to_u32(0, input);

  let mut ext_descriptors = vec![];
  let mut curr_idx = 4usize;

  for _ in 0..n_ext_descriptors {
    let (sjis_bytes, unicode) = get_sjis_bytes(curr_idx, input);
    curr_idx += sjis_bytes.len();
    let n_files = transmute_to_u32(curr_idx, input);
    curr_idx += 4;
    let start_offset = transmute_to_u32(curr_idx, input);
    curr_idx += 4;

    log::info!(
      "File type: {unicode} has {n_files} files with descriptors starting at 0x{start_offset:08X}"
    );

    ext_descriptors.push(ExtensionDescriptor {
      name: unicode,
      number: n_files,
      offset: start_offset,
    });
  }

  log::info!(
    "There are {} files to process.",
    ext_descriptors.iter().map(|it| it.number).sum::<u32>()
  );

  let mut files = vec![];

  let ext_descriptors_out_file = out_folder.join("extensions.yaml");
  std::fs::write(
    &ext_descriptors_out_file,
    serde_yml::to_string(&ext_descriptors).unwrap(),
  )
  .unwrap();

  for descriptor in ext_descriptors.iter() {
    let start_addr = descriptor.offset as usize;
    let mut descriptor_ptr = start_addr;
    for _ in 0..descriptor.number {
      let (name, unicode) = get_sjis_bytes_of_length(descriptor_ptr, 13, input);
      descriptor_ptr += name.len() - 1;
      let size = transmute_to_u32(descriptor_ptr, input);
      descriptor_ptr += 4;
      let offset = transmute_to_u32(descriptor_ptr, input);
      descriptor_ptr += 4;
      log::debug!(
        "File {unicode}.{} of size 0x{size:08X} starts at 0x{offset:08X}",
        descriptor.name.as_str()
      );
      files.push((
        format!("{}.{}", unicode, descriptor.name),
        FileDescriptor::new(unicode, size, offset),
      ));
    }
  }

  let file_descriptors_out_file = out_folder.join("files.yaml");
  std::fs::write(
    &file_descriptors_out_file,
    serde_yml::to_string(&files).unwrap(),
  )
  .unwrap();

  for (filename, desc) in files {
    log::info!("Processing {filename}.");

    let output_file_path = out_folder.join(filename.as_str());

    let content = &mut input[desc.offset..(desc.offset + desc.size)];

    if filename.ends_with("WSC") {
      rotate_wsc_for_unpack(content);
    } else if &content[..4] == "WIPF".as_bytes() && extract_wipf {
      do_extract_wipf(&filename, &output_file_path, content);
      continue;
    }

    std::fs::write(output_file_path, content).unwrap();
  }
}

fn rotate_wsc_for_unpack(input: &mut [u8]) {
  input.iter_mut().for_each(|chr| *chr = chr.rotate_right(2));
}

fn rotate_wsc_for_pack(input: &mut [u8]) {
  input.iter_mut().for_each(|chr| *chr = chr.rotate_left(2));
}

fn do_extract_wipf(filename: &str, output_file_path: &Path, content: &mut [u8]) {
  let header = WIPFHeader::from_ref(content);
  let entries =
    WIPFENTRY::from_ref_as_slice(&content[size_of_val(header)..], header.n_entries as usize);

  log::warn!(
        "WIPF file {filename} has {} entries with depth {}.",
        entries.len(),
        u32::from(header.depth)
      );

  let data = &content[size_of_val(header) + size_of_val(entries)..];
  let mut data_ptr = 0usize;
  for (entry_no, entry) in entries.iter().enumerate() {
    log::warn!(
          "    entry is {}x{}",
          u32::from(entry.width),
          u32::from(entry.height)
        );

    let palette = if header.depth == 8 {
      let palette = &data[data_ptr..data_ptr + 1024];
      data_ptr += 1024;
      palette
    } else {
      &[]
    };

    let out_depth = header.depth as u32 / 8;
    let out_stride = (entry.width * out_depth + 3) & !3u32;
    let out_len = (entry.height * out_stride) as usize;

    let out_buf = unwipf(&data[data_ptr..(data_ptr + entry.length as usize)], out_len);

    data_ptr += entry.length as usize;

    let out_file = output_file_path.join(&format!(
      "{filename}_{entry_no:03}+{}x{}y.bmp",
      u32::from(entry.x_offset),
      u32::from(entry.y_offset)
    ));

    let mut out_buf = if header.depth == 24 {
      let mut new_out = vec![0u8; out_buf.len()];

      let clr_stride = entry.width as usize;
      let clr_len = entry.height as usize * clr_stride;

      for y in 0..(entry.height as usize) {
        let curr_line_offset = (y * clr_stride);

        fn mkrange(start: usize, len: usize) -> std::ops::Range<usize> {
          start..(start + len)
        }

        let out_rgb_line = &mut new_out[curr_line_offset..(curr_line_offset + clr_stride)];

        let r_range = mkrange(curr_line_offset, clr_stride);
        let g_range = mkrange(curr_line_offset + clr_len, clr_stride);
        let b_range = mkrange(curr_line_offset + clr_len * 2, clr_stride);

        let r_line = &out_buf[r_range];
        let g_line = &out_buf[g_range];
        let b_line = &out_buf[b_range];

        for x in 0..entry.width as usize {
          out_rgb_line[x] = r_line[x];
          out_rgb_line[x] = g_line[x];
          out_rgb_line[x] = b_line[x];
        }
      }

      new_out
    } else {
      out_buf
    };

    for i in 0..(entry.height / 2) as usize {
      for j in 0..(entry.width * out_depth) as usize {
        let a = out_buf
          [(entry.height as usize - i - 1) * entry.width as usize * out_depth as usize + j];
        let b = out_buf[i * entry.width as usize * out_depth as usize + j];

        out_buf
          [(entry.height as usize - i - 1) * entry.width as usize * out_depth as usize + j] = b;
        out_buf[i * entry.width as usize * out_depth as usize + j] = a;
      }
    }

    let (file_size, bmp_offset, imgdata_size) = if header.depth == 8 {
      (0x436 + out_buf.len(), 0x436, 0x400 + out_buf.len())
    } else {
      (0x36 + out_buf.len(), 0x36, out_buf.len())
    };

    let bmp_header = BMPHeader {
      magic: ['B' as u8, 'M' as u8],
      filesz: file_size as u32,
      res1: 0,
      res2: 0,
      offset: bmp_offset,
    };

    let bmp_dib_header = BMPDibV3Header {
      header_sz: 0x28,
      width: entry.width,
      height: entry.height,
      nplanes: 1,
      bmp_bytesz: imgdata_size as u32,
      depth: header.depth,
      compress_type: 0,
      hres: 0,
      vres: 0,
      ncolors: 0,
      nimpcolors: 0,
    };

    std::fs::create_dir_all(&output_file_path).unwrap();
    let hdr_bytes = to_bytes(&bmp_header);
    let dib_bytes = to_bytes(&bmp_dib_header);
    std::fs::write(
      out_file,
      hdr_bytes
        .iter()
        .chain(dib_bytes)
        .chain(if header.depth == 8 { palette } else { &[] })
        .chain(out_buf.iter())
        .copied()
        .collect::<Vec<u8>>(),
    )
      .unwrap();
  }
}

pub fn decode_wsc(input: &[u8]) -> Script {
  let mut ptr = 0;
  let mut ptr_old = 1;
  let mut opcodes = vec![];
  let mut at_end = false;

  while ptr < input.len() {
    if ptr_old == ptr {
      break;
    }

    let op = make_opcode(&input[ptr..], ptr);
    if let Some(op) = op {
      log::debug!(
        "Got 0x{:02X} of length 0x{:02X} at 0x{:08X}",
        op.opcode,
        op.size(),
        ptr
      );
      at_end = op.opcode == 0xFF;
      ptr += op.size();
      opcodes.push(op);
    } else {
      ptr_old = ptr;
      log::error!("Unknown opcode at 0x{:08X}", ptr);
    }
    if at_end {
      break;
    }
  }

  let rest = input[ptr..].to_vec();

  let out = Script {
    opcodes,
    trailer: rest,
  };

  out
}

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
          match opcode.fields.get_mut(3) {
            Some(OpField::String(orig_str)) => {
              let _ = std::mem::replace(orig_str, line.translation);
            }
            _ => {}
          };
        }
      }
      DocLine::Scene(line) => {
        let opcode = addr2opcode.get_mut(&(line.address as usize)).unwrap();

        if opcode.opcode == 0xE0 {
          match opcode.fields.get_mut(0) {
            Some(OpField::String(orig_str)) => {
              let _ = std::mem::replace(orig_str, line.translation);
            }
            _ => {}
          };
        }

      }
      DocLine::SpeakerLine(line) => {
        let opcode = addr2opcode.get_mut(&(line.address as usize)).unwrap();
        if opcode.opcode == 0x42 {
          match opcode.fields.get_mut(4) {
            Some(OpField::String(orig_str)) => {
              let _ = std::mem::replace(orig_str, line.speaker_translation);
            }
            _ => {}
          };

          match opcode.fields.get_mut(5) {
            Some(OpField::String(orig_str)) => {
              let _ = std::mem::replace(orig_str, line.translation);
            }
            _ => {}
          };
        }
      }
      DocLine::Choices(choice) => {
        let opcode = addr2opcode.get_mut(&(choice.address as usize)).unwrap();
        if opcode.opcode == 0x02 {
          match opcode.fields.get_mut(2) {
            Some(OpField::Choice(orig_choices)) => {
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
      // Textbox with no speaker.
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
      terminated((value("scene", tag("[scene title @ ")), hex_int), tag("]")),
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
struct Line {
  address: u32,
  translation: TLString,
}

#[derive(Default, Debug)]
struct SpeakerLine {
  address: u32,
  speaker_address: u32,
  speaker_translation: TLString,
  translation: TLString,
}

#[derive(Default, Debug)]
struct ChoiceLine {
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
          (
            preceded(tag("\n[choice translation]:"), take_until("\n[")),
            preceded(
              tag("\n[choice notes]:"),
              terminated(take_until("\n---"), take_until(TL_CHOICE_END.as_str())),
            ),
          ),
        ),
        tag(TL_CHOICE_END.as_str()),
      ))
      .parse(rest)?;

      for (raw, (choice_tl, choice_notes)) in stuff {
        let translation = if is_blank(choice_tl) {
          None
        } else {
          Some(choice_tl.trim().to_string())
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

      (rest, DocLine::Choices(choiceline))
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
        line.translation.translation = Some(tl.trim().to_string());
      }
      DocLine::SpeakerLine(ref mut line) => {
        line.translation.translation = Some(tl.trim().to_string());
      }
      DocLine::Scene(ref mut line) => {
        line.translation.translation = Some(tl.trim().to_string());
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
