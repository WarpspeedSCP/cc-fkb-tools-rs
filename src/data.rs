use std::path::{Path, PathBuf};
use serde_derive::{Deserialize, Serialize};
use crate::util::{to_bytes, unwipf, encode_sjis, get_sjis_bytes, get_sjis_bytes_of_length, transmute_to_u32};

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
pub struct BMP_Header {
  magic: [u8; 2],
  filesz: u32,
  res1: u16,
  res2: u16,
  offset: u32,
}

#[repr(C, packed)]
pub struct BMP_Dib_V3_Header {
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

  let extensions = std::fs::read_to_string(input_folder.join("extensions.json"))
      .iter()
      .flat_map(|str| serde_json::from_str::<Vec<ExtensionDescriptor>>(&str))
      .next()
      .unwrap_or_default();
  let files = std::fs::read_to_string(input_folder.join("files.json"))
      .iter()
      .flat_map(|str| serde_json::from_str::<Vec<(String, FileDescriptor)>>(&str))
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
      sjis_name.extend(vec![ 0u8; 13 - sjis_name.len() ]);
      sjis_name
    } else {
      sjis_name
    };
    output.extend(sjis_name);
    let curr_path = input_folder.join(&filename);
    let mut contents = std::fs::read(&curr_path).unwrap();

    if filename.ends_with("WSC") {
      contents
          .iter_mut()
          .for_each(|chr| *chr = chr.rotate_left(2));
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

  let ext_descriptors_out_file = out_folder.join("extensions.json");
  std::fs::write(
    &ext_descriptors_out_file,
    serde_json::to_string_pretty(&ext_descriptors).unwrap(),
  )
      .unwrap();

  log::info!(
    "There are {} files to process.",
    ext_descriptors.iter().map(|it| it.number).sum::<u32>()
  );

  let mut files = vec![];

  let ext_descriptors_out_file = out_folder.join("extensions.json");
  std::fs::write(
    &ext_descriptors_out_file,
    serde_json::to_string_pretty(&ext_descriptors).unwrap(),
  ).unwrap();

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

  let file_descriptors_out_file = out_folder.join("files.json");
  std::fs::write(
    &file_descriptors_out_file,
    serde_json::to_string_pretty(&files).unwrap(),
  )
      .unwrap();

  for (filename, desc) in files {
    log::info!("Processing {filename}.");

    let output_file_path = out_folder.join(filename.as_str());

    let content = &mut input[desc.offset..(desc.offset + desc.size)];

    if filename.ends_with("WSC") {
      content
          .iter_mut()
          .for_each(|chr| *chr = chr.rotate_right(2));
    } else if &content[..4] == "WIPF".as_bytes() && extract_wipf {
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

        let bmp_header = BMP_Header {
          magic: ['B' as u8, 'M' as u8],
          filesz: file_size as u32,
          res1: 0,
          res2: 0,
          offset: bmp_offset,
        };

        let bmp_dib_header = BMP_Dib_V3_Header {
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
      continue;
    }

    std::fs::write(output_file_path, content).unwrap();
  }
}