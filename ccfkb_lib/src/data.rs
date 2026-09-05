use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use crate::opcodes::{make_opcode, Script};
use crate::util::{encode_sjis, get_sjis_bytes, get_sjis_bytes_of_length, safe_create_dir, to_bytes, transmute_to_u32, lz77_decompress, lz77_compress};
use camino::{Utf8Path as Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use regex::Regex;
use crate::data::text_script::hex_int;

pub mod text_script;

#[repr(C, packed)]
pub struct WIPFHeader {
	signature: [u8; 4],
	n_entries: u16,
	depth: u16,
}

impl WIPFHeader {

	fn new(n_entries: u16, depth: u16) -> Self {
		WIPFHeader {
			signature: *b"WIPF",
			n_entries,
			depth,
		}
	}
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

impl BMPHeader {
	pub fn new(filesz: u32, offset: u32) -> Self {
		BMPHeader {
			magic: ['B' as u8, 'M' as u8],
			filesz,
			res1: 0,
			res2: 0,
			offset,
		}
	}
}

impl From<&[u8]> for BMPHeader
{
	fn from(slice: &[u8]) -> Self {
		if slice.len() < size_of::<Self>() {
			panic!("bad input slice for bitmap header!");
		}

		unsafe {
			let data = slice[..14].as_ptr();
			let out: *const BMPHeader = std::mem::transmute(data as *const BMPHeader);
			BMPHeader {
				magic: *b"BM",
				filesz: (*out).filesz,
				res1: 0,
				res2: 0,
				offset: (*out).offset,
			}
		}
	}
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

impl From<&[u8]> for BMPDibV3Header {
	fn from(slice: &[u8]) -> Self {
		if slice.len() < size_of::<Self>() {
			panic!("bad input slice for bitmap DIB header!");
		}

		unsafe {
			let data = slice[..40].as_ptr();
			let out: &BMPDibV3Header = &*(std::mem::transmute::<_, *const BMPDibV3Header>(data as *const BMPDibV3Header));
			let val = &*out;
			BMPDibV3Header {
				header_sz: size_of::<BMPDibV3Header>() as u32,
				width: out.width,
				height: out.height,
				nplanes: out.nplanes,
				depth: out.depth,
				compress_type: out.compress_type,
				bmp_bytesz: out.bmp_bytesz,
				hres: out.hres,
				vres: out.vres,
				ncolors: out.ncolors,
				nimpcolors: out.nimpcolors,
			}
		}
	}
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct WIPFENTRY {
	width: u32,     // unsigned long  width;    // ����
	height: u32,    // unsigned long  height;   // �߶�
	x_offset: u32,  // unsigned long  offset_x; // x������ʾλ��
	y_offset: u32,  // unsigned long  offset_y; // y������ʾλ��
	unk_layer: u32, // unsigned long  unknown1; // layer?
	length: u32,    // unsigned long  length;   // �ļ�����
}

impl WIPFENTRY {

	fn new(width: u32, height: u32, x_offset: u32, y_offset: u32, length: u32) -> Self {
		WIPFENTRY {
			width,
			height,
			x_offset,
			y_offset,
			unk_layer: 0u32,
			length,
		}
	}

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

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ExtensionDescriptor {
	pub name: String,
	pub number: u32,
	pub offset: u32,
}

impl ExtensionDescriptor {
	pub fn size(&self) -> usize {
		(encode_sjis(&self.name).len() + 1) + 4 + 4
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileDescriptor {
	pub name: String,
	pub size: u32,
	pub offset: u32,
}

impl FileDescriptor {
	pub fn size(&self) -> usize {
		(encode_sjis(&self.name[..13]).len() + 1) + 4 + 4
	}
}

pub fn fix_yaml_str(it: String) -> String {
	it.replace("'[", "[")
		.replace("]'", "]")
		.replace(r#"'""#, "")
		.replace(r#""'"#, "")
}

pub struct ArcContents<'a> {
	pub extensions: Vec<ExtensionDescriptor>,
	pub files: Vec<FileDescriptor>,
	pub filenames: Vec<String>,
	pub data: Vec<&'a[u8]>,
}

#[must_use]
pub fn read_arc<'a>(input: &'a mut [u8], out_folder: &Utf8Path, extract_wipf: bool) -> ArcContents<'a> {
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

		log::info!("File type: {unicode} has {n_files} files with descriptors starting at 0x{start_offset:08X}");

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
			filenames.push(format!("{file_name}.{}", ext_descriptor.name));
			files.push(
				FileDescriptor {
					name: file_name,
					size,
					offset,
				}
			);
		}
	}

	let mut contents = vec![];
	let first_offset = files.first().unwrap().offset;
	let mut curr_offset = first_offset;
	let (_, mut input) = input.split_at_mut(first_offset as usize);
	for (filename, desc) in filenames.iter().zip(&files) {
		log::info!("Processing {filename}");

		let output_file_path = out_folder.join(filename.as_str());
		if curr_offset < desc.offset {
			let diff = desc.offset - curr_offset;
			(_, input) = input.split_at_mut(diff as usize);
		}

		let (content, new_input) = input.split_at_mut(desc.size as usize); // [desc.offset..(desc.offset + desc.size)];
		input = new_input;
		curr_offset += desc.size;

		if filename.ends_with("WSC") {
			rotate_wsc_for_unpack(content);
		} else if &content[..4] == "WIPF".as_bytes() && extract_wipf {
			let res = do_extract_wipf(&filename, &output_file_path, content);
			match res {
				Ok(_) => {}
				Err(err) => {
					log::error!("Error while extracting WIPF image {filename}: {}", err);
				}
			}
		}

		// Converts the mut ref back into a normal reference.
		contents.push(&*content);
	}

	ArcContents {
		extensions: ext_descriptors,
		files,
		filenames,
		data: contents,
	}
}

#[must_use]
pub fn write_arc<T: AsRef<Utf8Path>>(input_files: &[T], extensions: Vec<ExtensionDescriptor>, files: Vec<FileDescriptor>) -> Vec<u8> {
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
		log::info!("Packing {}", curr_path);

		let mut sjis_name = encode_sjis(&descriptor.name);
		let sjis_name = if sjis_name.len() < 13 {
			sjis_name.extend(vec![0u8; 13 - sjis_name.len()]);
			sjis_name
		} else {
			sjis_name
		};

		output.extend(sjis_name);
		let mut contents = std::fs::read(&curr_path).unwrap();

		if curr_path.file_name().map(|it| it.to_ascii_uppercase().ends_with("WSC")).unwrap_or_default() {
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
	for i in input.iter_mut() {
		*i = i.rotate_right(2);
	}
}

fn rotate_wsc_for_pack(input: &mut [u8]) {
	for i in input.iter_mut() {
		*i = i.rotate_left(2);
	}
}

fn do_pack_wipf(input_dir: &Utf8Path) -> std::io::Result<Vec<u8>> {
	use nom::branch::alt;
	use nom::bytes::complete::{tag, take_until, take_while};
	use nom::combinator::{map_res, value};
	use nom::multi::{many0, separated_list0};
	use nom::sequence::{preceded, terminated};
	use nom::IResult;
	use nom::{AsChar, Parser};

	let files_to_pack = walkdir::WalkDir::new(input_dir).contents_first(false).into_iter().skip(1).map(|entry| entry.unwrap().into_path()).collect::<Vec<_>>();
	let depth_is_8 = files_to_pack.get(0).map(|it| Utf8Path::from_path(&it).unwrap().file_name().unwrap().contains("d08")).unwrap_or(false);

	let file_name = input_dir.file_name().unwrap();
	let header = WIPFHeader::new(files_to_pack.len() as u16, if depth_is_8 { 8 } else { 24 });

	fn parse_file_name<'a>(file_name: &str, input: &'a str) -> IResult<&'a str, (&'a str, u32, u32, u32)> {
		(terminated(tag(file_name), tag("_")), terminated(hex_int, (tag("-d"), hex_int, tag("+"))), terminated(hex_int, tag("x")), terminated(hex_int, tag("y"))).parse(input)
	}

	let mut wipf_entries = vec![];
	let mut wipf_contents = vec![];

	for file in files_to_pack {
		let path = Utf8Path::from_path(&file).unwrap();
		let (_, (_, index, x, y)) = parse_file_name(file_name, path.file_name().unwrap()).unwrap();

		let bmp = std::fs::read(file)?;

		let dib_header = BMPDibV3Header::from(&bmp[14..(14 + 40)]);

		let entry = WIPFENTRY::new(dib_header.width, dib_header.height, x, y, dib_header.width * dib_header.height * (header.depth / 8) as u32);
		wipf_entries.push(entry);

		let entry_data = &bmp[(14 + 40)..];

		let compression_data = if depth_is_8 { &entry_data[0x400..] } else { entry_data };

		let row_size = ((entry.width * (header.depth / 8) as u32)).next_multiple_of(4);

		let entry_data_flip_iter = compression_data.rchunks_exact(row_size as usize);
		let entry_out_buffer = if !depth_is_8 {
			let colour_stride = entry.width;
			let colour_width = colour_stride * entry.height;
			let mut entry_out_buffer = vec![0u8; entry_data.len()];

			for (row_index, rgb_row) in entry_data_flip_iter.enumerate() {
				let (_, out_row) = entry_out_buffer.split_at_mut(row_index * entry.width as usize);

				let (r_line_out, rest) = out_row.split_at_mut(colour_stride as usize);
				let (g_line_out, rest) = rest.split_at_mut(colour_width as usize);
				let (b_line_out, rest) = rest.split_at_mut(colour_width as usize);

				let (data, row_rest) = rgb_row.as_chunks();
				for (index, &[r, g, b]) in data.iter().enumerate() {
					r_line_out[index] = r;
					g_line_out[index] = g;
					b_line_out[index] = b;
				}
			}
			lz77_compress(&entry_out_buffer)
		} else {
			lz77_compress(compression_data)
		};

		let entry_final_data = if depth_is_8 {
			(0..0xFF).flat_map(|it| [it, it, it, 0]).chain(entry_out_buffer).collect()
		} else {
			entry_out_buffer
		};

		wipf_contents.extend(entry_final_data);
	}


	let mut out_bytes = vec![];
	out_bytes.extend_from_slice(to_bytes(&header));
	for entry in wipf_entries {
		out_bytes.extend_from_slice(to_bytes(&entry));
	}
	out_bytes.extend(wipf_contents);

	Ok(out_bytes)
}

#[cfg(test)]
mod test {
	use camino::Utf8PathBuf;
	use crate::data::do_pack_wipf;

	#[test]
	fn main() {
		let input = Utf8PathBuf::from("/home/wscp/RustroverProjects/cc-fkb-tools-rs/extracted_arcs_old/Chip.arc/BGM_P1G.WIP");
		std::fs::write("/home/wscp/RustroverProjects/cc-fkb-tools-rs/extracted_arcs/Chip.arc/BGM_P1G.WIP.new", do_pack_wipf(&input).unwrap()).unwrap();
	}
}

fn do_extract_wipf(filename: &str, output_file_path: &Utf8Path, content: &mut [u8]) -> std::io::Result<()> {
	let header = WIPFHeader::from_ref(content);
	let entries =
		WIPFENTRY::from_ref_as_slice(&content[size_of_val(header)..], header.n_entries as usize);

	log::debug!(
		"WIPF file {filename} has {} entries with depth {}.",
		entries.len(),
		u32::from(header.depth)
	);

	safe_create_dir(&output_file_path).expect(&format!("Couldn't create output file for {filename} at {output_file_path}"));

	let data = &content[size_of_val(header) + size_of_val(entries)..];
	let mut data_ptr = 0usize;
	for (entry_no, entry) in entries.iter().enumerate() {
		log::debug!(
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

		let raw_depth = header.depth;
		let out_depth = raw_depth as u32 / 8;
		let out_stride = ((entry.width * out_depth + 3) & !3u32) as usize;
		let out_len = entry.height as usize * out_stride;

		let out_buf = lz77_decompress(&data[data_ptr..(data_ptr + entry.length as usize)], out_len);

		if out_buf.len() < out_len {
			log::error!("Could not decompress WIPF entry {entry_no:02} for file {filename}, expected 0x{out_len:08X} bytes but got only 0x{:08X} bytes", out_buf.len());
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "WIPF entry out of bounds"));
		}

		data_ptr += entry.length as usize;

		write_wip_entry(&filename, output_file_path, header, &entry_no, &entry, &palette, raw_depth, out_depth, out_stride, out_len, out_buf)?;
	}

	Ok(())
}

fn write_wip_entry(
	filename: &str,
	output_file_path: &Utf8Path,
	header: &WIPFHeader,
	entry_no: &usize,
	entry: &WIPFENTRY,
	palette: &[u8],
	raw_depth: u16,
	out_depth: u32,
	out_stride: usize,
	out_len: usize,
	out_buf: Vec<u8>
) -> std::io::Result<()> {
	let out_file = output_file_path.join(&format!(
		"{filename}_{entry_no:03}-d{raw_depth:02}+{}x{}y.bmp",
		u32::from(entry.x_offset),
		u32::from(entry.y_offset)
	));

	let out_buf = if header.depth == 24 {
		let mut new_out = vec![0u8; out_len];

		let clr_stride = entry.width as usize;
		let clr_len = entry.height as usize * clr_stride;

		for y in 0..(entry.height as usize) {
			let curr_line_offset = y * clr_stride;

			fn mkrange(start: usize, len: usize) -> std::ops::Range<usize> {
				start..(start + len)
			}

			let out_rgb_line = &mut new_out[mkrange(y * out_stride, out_stride)];

			let r_range = mkrange(curr_line_offset, clr_stride);
			let g_range = mkrange(curr_line_offset + clr_len, clr_stride);
			let b_range = mkrange(curr_line_offset + clr_len * 2, clr_stride);

			let r_line = &out_buf[r_range];
			let g_line = &out_buf[g_range];
			let b_line = &out_buf[b_range];

			for x in (0..out_stride).step_by(3) {
				let x_idx = x / 3;
				out_rgb_line[x] = r_line[x_idx];
				out_rgb_line[x + 1] = g_line[x_idx];
				out_rgb_line[x + 2] = b_line[x_idx];
			}
		}

		new_out
	} else {
		out_buf
	};

	let row_size = entry.width as usize * out_depth as usize;
	let out_buf_iterator = out_buf.rchunks_exact(row_size);

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

	let hdr_bytes = to_bytes(&bmp_header);
	let dib_bytes = to_bytes(&bmp_dib_header);
	let mut res = Vec::with_capacity(hdr_bytes.len() + dib_bytes.len() + palette.len() + out_buf.len());
	
	res.extend(hdr_bytes);
	res.extend(dib_bytes);
	res.extend(palette);
	
	for chunk in out_buf_iterator {
		res.extend(chunk);
	}

	std::fs::write(&out_file, &res)?;
	Ok(())
}

pub fn decode_wsc(input: &[u8]) -> Script {
	let mut ptr = 0;
	let mut opcodes = vec![];
	let mut at_end = false;

	while ptr < input.len() {
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
			log::error!("Unknown opcode at 0x{:08X}", ptr);
			break;
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

