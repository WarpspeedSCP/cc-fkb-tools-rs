use anyhow::{bail, Context};
use std::collections::BTreeMap;
use crate::opcodes::{make_opcode, Script};
use crate::util::{encode_sjis, get_sjis_bytes, get_sjis_bytes_of_length, safe_create_dir, to_bytes, transmute_to_u32, lz77_decompress, lz77_compress};
use camino::{Utf8Path as Utf8Path, Utf8PathBuf};
use crate::data::text_script::hex_int;

pub mod text_script;

/// On-disk size of a file descriptor: 13-byte null-padded SJIS name + u32 size + u32 offset.
const FILE_DESCRIPTOR_SIZE: usize = 13 + 4 + 4;

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

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone)]
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

pub struct ArcContents<'a> {
	pub extensions: Vec<ExtensionDescriptor>,
	pub files: Vec<FileDescriptor>,
	pub filenames: Vec<String>,
	pub data: Vec<&'a[u8]>,
}

/// Splits an arc into its descriptors and file contents, extracting any WIPF side effects into
/// `out_folder`.
///
/// Fails when the arc declares no file descriptors, in which case there is no first offset to start
/// walking from.
pub fn read_arc<'a>(input: &'a mut [u8], out_folder: &Utf8Path, extract_wipf: bool) -> anyhow::Result<ArcContents<'a>> {
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
	let Some(first_file) = files.first() else {
		bail!("{out_folder}: the arc declares no file descriptors");
	};
	let first_offset = first_file.offset;
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

	Ok(ArcContents {
		extensions: ext_descriptors,
		files,
		filenames,
		data: contents,
	})
}

pub fn write_arc<T: AsRef<Utf8Path>>(input_files: &[T], extensions: Vec<ExtensionDescriptor>, files: Vec<FileDescriptor>) -> anyhow::Result<Vec<u8>> {
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
		let mut contents = if curr_path.is_dir() {
			do_pack_wipf(curr_path).with_context(|| format!("packing the WIPF directory {curr_path}"))?
		} else {
			std::fs::read(curr_path).with_context(|| format!("reading {curr_path}"))?
		};
		if curr_path.file_name().map(|it| it.to_ascii_uppercase().ends_with("WSC")).unwrap_or_default() {
			rotate_wsc_for_pack(&mut contents)
		}

		let actual_content_len = contents.len();

		output.extend((actual_content_len as u32).to_le_bytes());
		output.extend(&(curr_offset as u32).to_le_bytes());
		curr_offset += actual_content_len;

		things_to_append.push(contents);
	}

	things_to_append.iter().for_each(|it| output.extend(it));

	Ok(output)
}

pub fn gen_descriptors_from_files(files: &[Utf8PathBuf]) -> anyhow::Result<(Vec<ExtensionDescriptor>, Vec<FileDescriptor>, Vec<Utf8PathBuf>, usize, u32)> {
	let mut grouped: BTreeMap<String, Vec<camino::Utf8PathBuf>> = BTreeMap::new();

	for file in files {
		let ext = file.extension().map(|it| it.to_uppercase()).unwrap_or_default();
		grouped.entry(ext).or_default().push(file.clone());
	}

	for group in grouped.values_mut() {
		group.sort();
	}

	let mut extension_descriptors: Vec<ExtensionDescriptor> = grouped
		.iter()
		.map(|(name, group)| ExtensionDescriptor {
			name: name.clone(),
			number: group.len() as u32,
			offset: 0,
		})
		.collect();

	let extension_list_size: usize = extension_descriptors.iter().map(ExtensionDescriptor::size).sum();

	let mut file_descriptors: Vec<FileDescriptor> = vec![];
	let mut pack_files: Vec<Utf8PathBuf> = vec![];
	for (_, group) in grouped.into_iter() {
		for file in group.into_iter() {
			file_descriptors.push(FileDescriptor {
				name: file
					.file_stem()
					.with_context(|| format!("{file} has no file stem"))?
					.to_uppercase(),
				size: std::fs::metadata(&file)
					.with_context(|| format!("reading the metadata of {file}"))?
					.len()
					.next_multiple_of(4) as u32,
				offset: 0,
			});
			pack_files.push(file);
		}
	}

	let file_list_size = FILE_DESCRIPTOR_SIZE * file_descriptors.len();

	// Start offset of files = 4 (n_extensions header) + extension descriptor list + file descriptor list.
	let file_data_start = 4 + extension_list_size + file_list_size;

	// Iterate over the descriptors and fill in the calculated offsets:
	//    - each extension descriptor's offset points at the start of its file descriptor block;
	//    - each file descriptor's offset points at its data (write_arc recomputes the same values).
	let mut descriptor_block_offset = 4 + extension_list_size;
	for descriptor in &mut extension_descriptors {
		descriptor.offset = descriptor_block_offset as u32;
		descriptor_block_offset += FILE_DESCRIPTOR_SIZE * descriptor.number as usize;
	}

	let mut data_offset = file_data_start as u32;
	for descriptor in &mut file_descriptors {
		descriptor.offset = data_offset;
		data_offset += descriptor.size;
	}
	Ok((extension_descriptors, file_descriptors, pack_files, file_data_start, data_offset))
}

/// Direct content entries of an extracted arc directory, sorted.
///
/// The YAML trees are the retired decoded-script form, generated beside an arc rather than arc
/// content, so they are excluded and a packed arc carries none.
pub fn arc_entries(arc_dir: &Utf8Path) -> std::io::Result<Vec<Utf8PathBuf>> {
	let mut entries = vec![];
	for entry in arc_dir.read_dir_utf8()? {
		let entry = entry?;
		let path = entry.path();
		let is_yaml = path
			.extension()
			.map(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
			.unwrap_or(false);
		if !is_yaml {
			entries.push(path.to_owned());
		}
	}
	entries.sort();
	Ok(entries)
}

/// Serializes an arc from `input_files` + descriptors and writes it to `output_path`.
pub fn pack_arc(output_path: &Utf8Path, input_files: &[Utf8PathBuf], extensions: Vec<ExtensionDescriptor>, files: Vec<FileDescriptor>) -> anyhow::Result<()> {
	let output = write_arc(input_files, extensions, files)
		.with_context(|| format!("packing the arc at {output_path}"))?;
	std::fs::write(output_path, output)
		.with_context(|| format!("writing {output_path}"))
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
	use nom::bytes::complete::tag;
	use nom::sequence::terminated;
	use nom::IResult;
	use nom::Parser;

	let mut files_to_pack = walkdir::WalkDir::new(input_dir)
		.contents_first(false)
		.into_iter()
		.skip(1)
		.map(|entry| entry.map(|it| it.into_path()).map_err(|err| std::io::Error::other(err.to_string())))
		.collect::<std::io::Result<Vec<_>>>()?;
	files_to_pack.sort();
	let depth = files_to_pack
		.first()
		.map(|it| {
			std::fs::read(it).and_then(|bmp| {
				if bmp.len() < 14 + 40 {
					return Err(std::io::Error::new(
						std::io::ErrorKind::InvalidData,
						format!("BMP {it:?} is too small"),
					));
				}
				let dib = BMPDibV3Header::from(&bmp[14..(14 + 40)]);
				Ok(dib.depth)
			})
		})
		.transpose()?
		.unwrap_or(24);
	let depth_is_8 = depth == 8;

	let file_name = input_dir.file_name().ok_or_else(|| {
		std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{input_dir} has no directory name"))
	})?;
	let header = WIPFHeader::new(files_to_pack.len() as u16, depth);

	fn parse_file_name<'a>(file_name: &str, input: &'a str) -> IResult<&'a str, (&'a str, u32, u32, u32)> {
		(terminated(tag(file_name), tag("_")), terminated(hex_int, (tag("-d"), hex_int, tag("+"))), terminated(hex_int, tag("x")), terminated(hex_int, tag("y"))).parse(input)
	}

	let mut wipf_entries = vec![];
	let mut wipf_contents = vec![];

	for file in files_to_pack {
		let path = Utf8Path::from_path(&file).ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{} is not valid UTF-8", file.display()))
		})?;
		let name = path.file_name().ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{path} has no file name"))
		})?;
		let (_, (_, _, x, y)) = parse_file_name(file_name, name).map_err(|err| {
			std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!("{path} does not match the expected \"{file_name}_H-dH+HxH+H.BMP\" name: {err}"),
			)
		})?;

		let bmp = std::fs::read(path)?;

		if bmp.len() < 14 + 40 {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!("BMP {path} is too small"),
			));
		}

		let bmp_header = BMPHeader::from(&bmp[..14]);
		let dib_header = BMPDibV3Header::from(&bmp[14..(14 + 40)]);
		let bmp_width = dib_header.width;
		let bmp_height = dib_header.height;
		let bmp_depth = dib_header.depth;
		let wipf_depth = header.depth;

		if bmp_depth != 8 && bmp_depth != 24 {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!("BMP {path} has unsupported depth {bmp_depth}"),
			));
		}

		if bmp_depth != wipf_depth {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!(
					"BMP {path} has depth {} but WIPF depth is {}",
					bmp_depth, wipf_depth
				),
			));
		}

		// BMP files written by modern image editors may use a BITMAPV5HEADER
		// (or another extended DIB header), so the palette/pixel data do not
		// necessarily start at 14 + 40. Honour the DIB header size and the
		// file header's bfOffBits field instead of hardcoding the V3 layout.
		let dib_size = u32::from_le_bytes(bmp[14..18].try_into().unwrap()) as usize;
		let palette_start = 14usize.checked_add(dib_size).ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::InvalidData, "BMP DIB size overflow")
		})?;
		let pixel_offset = bmp_header.offset as usize;

		if pixel_offset > bmp.len() || palette_start > pixel_offset {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!("BMP {path} has invalid palette/pixel offsets"),
			));
		}

		let bytes_per_pixel = (bmp_depth / 8) as usize;
		let row_size = (bmp_width as usize)
			.checked_mul(bytes_per_pixel)
			.and_then(|it| it.checked_add(3))
			.map(|it| it & !3)
			.ok_or_else(|| {
				std::io::Error::new(std::io::ErrorKind::InvalidData, "BMP row size overflow")
			})?;
		let pixel_len = row_size
			.checked_mul(bmp_height as usize)
			.ok_or_else(|| {
				std::io::Error::new(std::io::ErrorKind::InvalidData, "BMP pixel data size overflow")
			})?;
		let pixel_end = pixel_offset.checked_add(pixel_len).ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::InvalidData, "BMP pixel data end overflow")
		})?;

		if pixel_end > bmp.len() {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				format!(
					"BMP {path} is truncated: pixel data ends at 0x{pixel_end:X}, file is 0x{:X} bytes",
					bmp.len()
				),
			));
		}

		let pixel_data = &bmp[pixel_offset..pixel_end];
		let palette = if depth_is_8 {
			let mut palette = vec![0u8; 0x400];
			let palette_src = &bmp[palette_start..pixel_offset];
			let copy_len = palette.len().min(palette_src.len());
			palette[..copy_len].copy_from_slice(&palette_src[..copy_len]);
			palette
		} else {
			vec![]
		};

		let mut entry = WIPFENTRY::new(
			bmp_width,
			bmp_height,
			x,
			y,
			bmp_width * bmp_height * (bmp_depth / 8) as u32,
		);

		let entry_out_buffer = if !depth_is_8 {
			let clr_len = entry.width as usize * entry.height as usize;
			let mut entry_out_buffer = vec![0u8; clr_len * 3];
			let (r_plane, rest) = entry_out_buffer.split_at_mut(clr_len);
			let (g_plane, b_plane) = rest.split_at_mut(clr_len);

			for (row_index, rgb_row) in pixel_data.rchunks_exact(row_size).enumerate() {
				let base = row_index * entry.width as usize;
				let (data, _) = rgb_row.as_chunks();
				for (index, &[r, g, b]) in data.iter().enumerate() {
					r_plane[base + index] = r;
					g_plane[base + index] = g;
					b_plane[base + index] = b;
				}
			}
			lz77_compress(&entry_out_buffer)
		} else {
			let raw_pixels = pixel_data
				.rchunks_exact(row_size)
				.flatten()
				.copied()
				.collect::<Vec<u8>>();
			lz77_compress(&raw_pixels)
		};

		entry.length = entry_out_buffer.len() as u32;

		let entry_final_data = if depth_is_8 {
			palette.iter().copied().chain(entry_out_buffer).collect()
		} else {
			entry_out_buffer
		};

		wipf_entries.push(entry);
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

fn do_extract_wipf(filename: &str, output_file_path: &Utf8Path, content: &mut [u8]) -> std::io::Result<()> {
	let header = WIPFHeader::from_ref(content);
	let entries =
		WIPFENTRY::from_ref_as_slice(&content[size_of_val(header)..], header.n_entries as usize);

	log::debug!(
		"WIPF file {filename} has {} entries with depth {}.",
		entries.len(),
		u32::from(header.depth)
	);

	safe_create_dir(output_file_path).map_err(|err| {
		std::io::Error::new(err.kind(), format!("could not create {output_file_path} for {filename}: {err}"))
	})?;

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

	let row_size = (entry.width as usize * out_depth as usize).next_multiple_of(4);
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
	let mut at_end;

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

	let out = Script { opcodes, trailer: rest };

	out
}

#[cfg(test)]
mod test {
	use camino::{Utf8Path, Utf8PathBuf};
	use crate::data::{do_extract_wipf, do_pack_wipf, WIPFENTRY, WIPFHeader};

	fn compare_dirs(orig_dir: &Utf8Path, new_dir: &Utf8Path) {
		let mut total = 0usize;
		let mut mismatches = vec![];
		let mut missing = vec![];
		for entry in std::fs::read_dir(orig_dir).unwrap() {
			let entry = entry.unwrap();
			if !entry.file_type().unwrap().is_file() { continue; }
			let name = entry.file_name();
			let orig = std::fs::read(entry.path()).unwrap();
			match std::fs::read(new_dir.join(name.to_string_lossy().as_ref())) {
				Ok(new) => {
					total += 1;
					if orig != new {
						let first_diff = orig.iter().zip(new.iter()).position(|(a, b)| a != b);
						mismatches.push(format!("{}: orig={} new={} first_diff={:?}", name.to_string_lossy(), orig.len(), new.len(), first_diff));
					}
				}
				Err(_) => missing.push(name.to_string_lossy().to_string()),
			}
		}
		println!("  compared {} files, {} mismatches, {} missing", total, mismatches.len(), missing.len());
		for m in mismatches.iter().take(10) { println!("    MISMATCH {m}"); }
		for m in missing.iter().take(10) { println!("    MISSING {m}"); }
	}

	/// WIPF fixtures live in the repository root while `cargo test` runs with the CWD set to the
	/// crate directory, so fixture paths must be anchored to `CARGO_MANIFEST_DIR`.
	fn repo_fixture(rel: &str) -> Utf8PathBuf {
		Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(rel)
	}

	fn roundtrip_dir(input_dir: &Utf8Path, filename: &str, out_dir: &Utf8Path) {
		let _ = std::fs::remove_dir_all(out_dir);
		std::fs::create_dir_all(out_dir.parent().unwrap()).unwrap();
		println!("roundtripping {filename} from {input_dir}");
		let packed = do_pack_wipf(input_dir).unwrap();
		println!("  repacked size={}", packed.len());

		let header = WIPFHeader::from_ref(&packed);
		let entries = WIPFENTRY::from_ref_as_slice(&packed[std::mem::size_of_val(header)..], header.n_entries as usize);
		let data_start = std::mem::size_of_val(header) + std::mem::size_of_val(entries);
		let n_entries = header.n_entries;
		let depth = header.depth;
		let sum_len: usize = entries.iter().map(|e| e.length as usize).sum();
		println!("  n_entries={n_entries} depth={depth} data_start=0x{data_start:X} total={} sum(entry.length)={sum_len} available={}", packed.len(), packed.len() - data_start);

		let mut content = packed.clone();
		do_extract_wipf(filename, out_dir, &mut content).unwrap();
		compare_dirs(input_dir, out_dir);
	}

	#[test]
	fn lz77_pair_roundtrip() {
		let data: Vec<u8> = (0..=255u8).collect();
		let comp = crate::util::lz77_compress(&data);
		let dec = crate::util::lz77_decompress(&comp, data.len());
		println!("lz77: input={} compressed={} decompressed={} identical={}", data.len(), comp.len(), dec.len(), dec == data);
		println!("  comp head: {:02X?}", &comp[..comp.len().min(16)]);
		println!("  dec  head: {:02X?}", &dec[..dec.len().min(16)]);
		println!("  in   head: {:02X?}", &data[..16]);
	}

	#[test]
	fn lz77_matches_original_entries() {
		let file = "BGM_P1G.WIP";
		let content = std::fs::read(repo_fixture(file)).unwrap();
		let header = WIPFHeader::from_ref(&content);
		let entries = WIPFENTRY::from_ref_as_slice(&content[std::mem::size_of_val(header)..], header.n_entries as usize);
		let data_start = std::mem::size_of_val(header) + std::mem::size_of_val(entries);
		let data = &content[data_start..];
		let mut data_ptr = 0usize;
		let mut mismatches = 0usize;
		for (i, entry) in entries.iter().enumerate() {
			let out_depth = 3usize;
			let out_stride = (entry.width as usize * out_depth + 3) & !3;
			let out_len = entry.height as usize * out_stride;
			let original = &data[data_ptr..data_ptr + entry.length as usize];
			let decompressed = crate::util::lz77_decompress(original, out_len);
			let recompressed = crate::util::lz77_compress(&decompressed);
			if recompressed != original {
				mismatches += 1;
				println!("  entry {i}: original_len={} recompressed_len={}", original.len(), recompressed.len());
				if mismatches <= 3 {
					let first_diff = original.iter().zip(recompressed.iter()).position(|(a, b)| a != b);
					println!("    first_diff={:?}", first_diff);
				}
			}
			data_ptr += entry.length as usize;
		}
		println!("{file}: {} entries, {} mismatches", entries.len(), mismatches);
		assert_eq!(mismatches, 0);
	}

	#[test]
	fn wipf_roundtrip_8bit() {
		let input = repo_fixture("extracted_arcs/Chip.arc/EVCC0020A.MOS");
		roundtrip_dir(&input, "EVCC0020A.MOS", &Utf8PathBuf::from("/tmp/wipf_rt_8"));
	}

	#[test]
	fn wipf_roundtrip_24bit() {
		let orig = std::fs::read(repo_fixture("BGM_P1G.WIP")).unwrap();
		let extract_dir = Utf8PathBuf::from("/tmp/wipf_rt_24/BGM_P1G.WIP");
		let _ = std::fs::remove_dir_all(&extract_dir);
		std::fs::create_dir_all(extract_dir.parent().unwrap()).unwrap();
		let mut content = orig.clone();
		do_extract_wipf("BGM_P1G.WIP", &extract_dir, &mut content).unwrap();
		roundtrip_dir(&extract_dir, "BGM_P1G.WIP", &Utf8PathBuf::from("/tmp/wipf_rt_24_out/BGM_P1G.WIP"));
	}

	#[test]
	fn wipf_pack_honours_v5_bmp_header() {
		fn put_u16(buf: &mut [u8], off: usize, value: u16) {
			buf[off..off + 2].copy_from_slice(&value.to_le_bytes());
		}

		fn put_u32(buf: &mut [u8], off: usize, value: u32) {
			buf[off..off + 4].copy_from_slice(&value.to_le_bytes());
		}

		// Classic 24-bit BMP with a 40-byte BITMAPINFOHEADER and bfOffBits = 54.
		fn make_v3_bmp(width: usize, height: usize) -> Vec<u8> {
			let row_size = (width * 3).next_multiple_of(4);
			let pixel_len = row_size * height;
			let mut bmp = vec![0u8; 14 + 40 + pixel_len];

			bmp[0] = b'B';
			bmp[1] = b'M';
			let bmp_len = bmp.len() as u32;
			put_u32(&mut bmp, 2, bmp_len);
			put_u32(&mut bmp, 10, 54);
			put_u32(&mut bmp, 14, 40);
			put_u32(&mut bmp, 18, width as u32);
			put_u32(&mut bmp, 22, height as u32);
			put_u16(&mut bmp, 26, 1);
			put_u16(&mut bmp, 28, 24);
			put_u32(&mut bmp, 34, pixel_len as u32);

			for (index, byte) in bmp[54..].iter_mut().enumerate() {
				*byte = (index as u8).wrapping_mul(17).wrapping_add(3);
			}
			bmp
		}

		// Simulate GIMP's BITMAPV5HEADER: 84 extra DIB bytes and bfOffBits = 138.
		fn make_v5_bmp(v3: &[u8]) -> Vec<u8> {
			let mut v5 = v3.to_vec();
			let extra = 124 - 40;
			v5.splice(54..54, std::iter::repeat(0u8).take(extra));
			let v5_len = v5.len() as u32;
			put_u32(&mut v5, 2, v5_len);
			put_u32(&mut v5, 10, 138);
			put_u32(&mut v5, 14, 124);
			v5
		}

		let root = Utf8PathBuf::from("/tmp/wipf_v5_test");
		let _ = std::fs::remove_dir_all(&root);

		let v3_dir = root.join("v3").join("CFGALPHA.WIP");
		let v5_dir = root.join("v5").join("CFGALPHA.WIP");
		std::fs::create_dir_all(&v3_dir).unwrap();
		std::fs::create_dir_all(&v5_dir).unwrap();

		let filename = "CFGALPHA.WIP_000-d24+0x0y.bmp";
		let v3 = make_v3_bmp(4, 4);
		let v5 = make_v5_bmp(&v3);
		std::fs::write(v3_dir.join(filename), &v3).unwrap();
		std::fs::write(v5_dir.join(filename), &v5).unwrap();

		let packed_v3 = do_pack_wipf(&v3_dir).unwrap();
		let packed_v5 = do_pack_wipf(&v5_dir).unwrap();
		assert_eq!(packed_v3, packed_v5, "V5 and V3 BMPs must produce identical WIPF data");

		let header = WIPFHeader::from_ref(&packed_v5);
		let depth = header.depth;
		let n_entries = header.n_entries as usize;
		assert_eq!(depth, 24);
		let entries = WIPFENTRY::from_ref_as_slice(
			&packed_v5[std::mem::size_of_val(header)..],
			n_entries,
		);
		assert_eq!(entries.len(), 1);
		let data_start = std::mem::size_of_val(header) + std::mem::size_of_val(entries);
		let payload = &packed_v5[data_start..];
		let out_len = 4 * 4 * 3;
		let decoded = crate::util::lz77_decompress(payload, out_len);
		assert_eq!(decoded.len(), out_len);
	}
}