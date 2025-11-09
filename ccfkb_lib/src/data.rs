use crate::opcodes::{Script, make_opcode};
use crate::util::{
  encode_sjis, get_sjis_bytes, get_sjis_bytes_of_length, to_bytes, transmute_to_u32,
  unwipf,
};
use serde_derive::{Deserialize, Serialize};
use std::path::Path;

pub mod text_script;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtensionDescriptor {
  pub name: String,
  pub number: u32,
  pub offset: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileDescriptor {
  pub name: String,
  pub size: usize,
  pub offset: usize,
}

pub fn fix_yaml_str(it: String) -> String {
  it.replace("'[", "[")
      .replace("]'", "]")
      .replace(r#"'""#, "")
      .replace(r#""'"#, "")
}

pub fn read_arc<'a>(input: &'a mut [u8], out_folder: &Path, extract_wipf: bool) -> (Vec<ExtensionDescriptor>, Vec<FileDescriptor>, Vec<String>, Vec<&'a [u8]>) {
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

  let mut filenames = vec![];
  let mut files = vec![];

  for ext_descriptor in ext_descriptors.iter() {
    let start_addr = ext_descriptor.offset as usize;
    let mut descriptor_ptr = start_addr;
    for _ in 0..ext_descriptor.number {
      let (name, file_name) = get_sjis_bytes_of_length(descriptor_ptr, 13, input);
      descriptor_ptr += name.len() - 1;
      let size = transmute_to_u32(descriptor_ptr, input);
      descriptor_ptr += 4;
      let offset = transmute_to_u32(descriptor_ptr, input);
      descriptor_ptr += 4;
      log::debug!(
        "File {file_name}.{} of size 0x{size:08X} starts at 0x{offset:08X}",
        ext_descriptor.name.as_str()
      );
      filenames.push(format!("{}.{}", file_name, ext_descriptor.name), );
      files.push(
        FileDescriptor {
          name: file_name,
          size: size as usize,
          offset: offset as usize,
        }
      );
    }
  }

  let mut contents = vec![];
  let first_offset = files.first().unwrap().offset;
  let mut curr_offset = first_offset;
  let (_, mut input) = input.split_at_mut(first_offset);
  for (filename, desc) in filenames.iter().zip(&files) {
    log::info!("Processing {filename}");

    let output_file_path = out_folder.join(filename.as_str());
    if curr_offset < desc.offset {
      let diff = desc.offset - curr_offset;
      (_, input) = input.split_at_mut(diff);
    }

    let (content, new_input) = input.split_at_mut(desc.size); // [desc.offset..(desc.offset + desc.size)];
    input = new_input;
    curr_offset += desc.size;

    if filename.ends_with("WSC") {
      rotate_wsc_for_unpack(content);
    } else if &content[..4] == "WIPF".as_bytes() && extract_wipf {
      do_extract_wipf(&filename, &output_file_path, content);
    }

    // Converts the mut ref back into a normal reference.
    contents.push(&*content);

  }

  (ext_descriptors, files, filenames, contents)
}

pub fn write_arc<T: AsRef<Path>>(input_files: &[T], extensions: Vec<ExtensionDescriptor>, files: Vec<FileDescriptor>) -> Vec<u8> {
  let mut output = vec![];

  output.extend((extensions.len() as u32).to_le_bytes());

  for descriptor in extensions {
    output.extend(encode_sjis(&descriptor.name));
    output.push(0);
    output.extend(&descriptor.number.to_le_bytes());
    output.extend(&descriptor.offset.to_le_bytes());
  }

  let mut things_to_append = vec![];
  let mut curr_offset = output.len() + (13 + 4 + 4) * files.len(); // the size of a file descriptor.

  for (descriptor, curr_path) in files.iter().zip(input_files.iter().map(AsRef::as_ref)) {
    log::info!("Packing {}", curr_path.display());

    let mut sjis_name = encode_sjis(&descriptor.name);
    let sjis_name = if sjis_name.len() < 13 {
      sjis_name.extend(vec![0u8; 13 - sjis_name.len()]);
      sjis_name
    } else {
      sjis_name
    };

    output.extend(sjis_name);
    let mut contents = std::fs::read(&curr_path).unwrap();

    if curr_path.file_name().unwrap().to_string_lossy().ends_with("WSC") {
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


fn rotate_wsc_for_unpack(input: &mut [u8]) {
  input.iter_mut().for_each(|chr| *chr = chr.rotate_right(2));
}

fn rotate_wsc_for_pack(input: &mut [u8]) {
  input.iter_mut().for_each(|chr| *chr = chr.rotate_left(2));
}

fn do_extract_wipf(filename: &str, output_file_path: &Path, content: &mut [u8]) -> () {
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
        let curr_line_offset = y * clr_stride;

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

  let rest = if ptr >= input.len() {
    vec![]
  } else {
    input[ptr..].to_vec()
  };

  let out = Script {
    opcodes,
    trailer: rest,
  };

  out
}

