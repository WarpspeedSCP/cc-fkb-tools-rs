mod util;

use core::ptr::addr_of;
use std::{collections::HashMap, io::Read, path::PathBuf};

use util::{get_sjis_bytes, get_sjis_bytes_of_length, transmute_to_u16, transmute_to_u32};

fn main() {
  env_logger::builder().format_timestamp(None).init();
  std::fs::create_dir_all("unpacked_arcs").unwrap();
  let files = walkdir::WalkDir::new("/home/wscp/cc-fkb-tools/cross channel fkb/").into_iter().filter_map(|it| it.ok()).collect::<Vec<_>>();
  for i in files.iter() {
    let dirent = i;
    let pth = dirent.path();
    if !pth.extension().map(|it| it.to_string_lossy().to_owned()).unwrap_or_default().ends_with("arc") {
      continue;
    }
    let mut file = std::fs::read(dirent.path()).unwrap();
    
    let path = PathBuf::from("unpacked_arcs").join(dirent.file_name());
    std::fs::create_dir_all(&path).unwrap();
    let out = read_arc(&mut file[..], path);

  }


}

#[repr(C, packed)]
struct WIPFHeader {
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
struct BMP_Header {
  magic: [u8; 2],
  filesz: u32,
  res1: u16,
  res2: u16,
  offset: u32,
}

#[repr(C, packed)]
struct BMP_Dib_V3_Header {
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
struct WIPFENTRY {
  width: u32,       // unsigned long  width;    // ����
  height: u32,      // unsigned long  height;   // �߶�
  x_offset: u32,    // unsigned long  offset_x; // x������ʾλ��
  y_offset: u32,    // unsigned long  offset_y; // y������ʾλ��
  unk_layer: u32,   // unsigned long  unknown1; // layer?
  length: u32,      // unsigned long  length;   // �ļ�����
}                     // WIPF�ں��ļ�ϸ��

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

struct ExtensionDescriptor {
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

struct FileDescriptor {
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

fn read_arc(input: &mut [u8], out_folder: PathBuf) -> () {
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


  log::info!("There are {} files to process.", ext_descriptors.iter().map(|it| it.number).sum::<u32>());

  let mut files = HashMap::new();

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
      log::info!("File {unicode}.{} of size 0x{size:08X} starts at 0x{offset:08X}", descriptor.name.as_str());
      files.insert(
        format!("{}.{}", unicode, descriptor.name),
        FileDescriptor::new(unicode, size, offset)
      );
    }
  }

  for (filename, desc) in files {
    log::info!("Processing {filename}.");

    let output_file_path = out_folder.join(filename.as_str());

    let content = &mut input[desc.offset..(desc.offset + desc.size)];

    if filename.ends_with("WSC") {
      content.iter_mut().for_each(|chr| *chr = chr.rotate_right(2));
    } else if &content[..4] == "WIPF".as_bytes() {
      let header = WIPFHeader::from_ref(content);
      let entries = WIPFENTRY::from_ref_as_slice(
        &content[size_of_val(header)..],
        header.n_entries as usize
      );

      log::warn!("WIPF file {filename} has {} entries with depth {}.", entries.len(), u32::from(header.depth));
      
      /*
      let data = &content[size_of_val(header) + size_of_val(entries)..];
      let mut data_ptr = 0usize;
      for (entry_no, entry) in entries.iter().enumerate() {
        log::warn!("    entry is {}x{}", u32::from(entry.width), u32::from(entry.height));
        
        let palette = if header.depth == 8 {
          let palette = &data[data_ptr..data_ptr + 1024];
          data_ptr += 1024;
          palette
        } else { &[] };

        let out_depth = header.depth as u32 / 8;
        let out_stride = (entry.width * out_depth + 3) & !3u32;
        let out_len = (entry.height * out_stride) as usize;

        let out_buf = lz77_decompress(&data[data_ptr..(data_ptr + entry.length as usize)], out_len);

        data_ptr += entry.length as usize;

        
        let out_file = output_file_path.join(&format!("{filename}_{entry_no:03}+{}x{}y.bmp", u32::from(entry.x_offset), u32::from(entry.y_offset)));

        /*
        					if( wipfhdr -> depth == 24 ) { // 24位图
						unsigned char *temp_buff = new unsigned char[ out_len ];
						
						unsigned long color_stride = wipfentries[ k ].width;
							// RGB色间距 = 宽
						unsigned long color_len    = wipfentries[ k ].height * color_stride;
							// 高 x 宽 （因为是RGB三色的，所以一行图是一色）

						for( unsigned long y = 0; y < wipfentries[ k ].height; y ++ ) {
							// 行间循环
							struct RGB { unsigned char r, g, b; };
							RGB *rgb_line = (RGB *)( temp_buff + y * out_stride );
								// temp_buff[ y * out_stride ] 定义的是指针
							unsigned char *r_line = out_buff + y * color_stride;
							unsigned char *g_line = r_line + color_len;
							unsigned char *b_line = g_line + color_len;

							for( unsigned long x = 0; x < wipfentries[ k ].width; x ++ ) {
								// 列循环（行内循环）rgb_line[ ]是一行内的内容
								rgb_line[ x ].r = r_line[ x ];
								rgb_line[ x ].g = g_line[ x ];
								rgb_line[ x ].b = b_line[ x ];
							}
						}

						delete [ ] out_buff;
						out_buff = temp_buff;
					}
         */

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

        /*
        	for( unsigned int rr = 0; rr < wipfentries[ k ].height / 2; rr ++ ) {
						for( unsigned int ee = 0; ee < wipfentries[ k ].width * out_depth; ee ++ ) {
							swap( out_buff[ ( wipfentries[ k ].height - rr - 1 ) * wipfentries[ k ].width * out_depth + ee ],
								  out_buff[ rr * wipfentries[ k ].width * out_depth + ee ] );
						}
					}
         */

        for i in 0..(entry.height / 2) as usize {
          for j in 0..(entry.width * out_depth) as usize {
            let a = out_buf[(entry.height as usize - i - 1) * entry.width as usize * out_depth as usize + j];
            let b = out_buf[i * entry.width as usize * out_depth as usize + j];

            out_buf[(entry.height as usize - i - 1) * entry.width as usize * out_depth as usize + j] = b;
            out_buf[i * entry.width as usize * out_depth as usize + j] = a;
          }
        }

        let (file_size, bmp_offset, imgdata_size) = if header.depth == 8 {
          (
            0x436 + out_buf.len(),
            0x436,
            0x400 + out_buf.len()
          )
        } else {
          (
            0x36 + out_buf.len(),
            0x36,
            out_buf.len()
          )
        };

        let bmp_header = BMP_Header {
            magic: ['B' as u8, 'M' as u8],
            filesz: file_size as u32,
            res1: 0,
            res2: 0,
            offset: bmp_offset,
        };

        let mut bmp_dib_header: BMP_Dib_V3_Header = unsafe { std::mem::transmute([0u8; size_of::<BMP_Dib_V3_Header>()]) };

        bmp_dib_header.header_sz = 0x28;
        bmp_dib_header.width = entry.width;
        bmp_dib_header.height = entry.height;
        bmp_dib_header.depth = header.depth;
        bmp_dib_header.nplanes = 1;
        bmp_dib_header.bmp_bytesz = imgdata_size as u32;

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
            .collect::<Vec<u8>>()
          ).unwrap();
      }
      continue;
      */
    }

    std::fs::write(output_file_path, content).unwrap();
  }
}

pub fn to_bytes<T: Sized> (value: &T) -> &[u8] {
  unsafe {
    let data = std::mem::transmute(value as *const T);
    &*core::ptr::slice_from_raw_parts(data, size_of_val(value))
  }
}

pub(crate) fn lz77_decompress(input: &[u8], size: usize) -> Vec<u8> {
  let mut output = vec![0; size as usize];

  let mut input_ptr = 4usize;

  let mut offset = 0usize;

  let mut ring = [0u8; 4096];
  let mut ring_idx = 1usize;

  while offset < size {
    if input_ptr >= input.len() {
      break;
    }
    let mut flags = input[input_ptr];
    input_ptr += 1;

    for _ in 0..8 {
      if input_ptr >= input.len() {
        break;
      }

      if flags & 1 == 1 {
        // *out_buff = ring[ring_index % ring_len] = *buff++;
        output[offset as usize] = input[input_ptr];
        ring[ring_idx % 4096] = input[input_ptr];
        input_ptr += 1;
        offset += 1;
        ring_idx += 1;
      } else {
        if (input_ptr + 2) > input.len() {
          break;
        }

        let p = input[input_ptr];
        let n = input[input_ptr + 1];
        input_ptr += 2;

        let mut ring_pos = ((p << 4) | (n >> 4)) as usize;   // ( RHL p, 4 ) OR ( RHR n, 4 ) 即 p的八位 与 n的前四位 构成
				let n_bytes = ((n & 0x0F) + 2) as usize;        // AND 00001111b; ADD 2 这里用到的是n的后四位

        while (offset as usize) < output.len() && ring_pos < n_bytes {
          ring[ring_idx % 4096] = ring[ring_pos % 4096];
          output[offset as usize] = ring[ring_pos % 4096];
          ring_idx += 1;
          ring_pos += 1;
          offset += 1;
        }
      }
      flags >>= 1;
      if offset >= size {
        break;
      }
    }
  }

  output
}

/*
unsigned long CrossChannelCrack::unwipf( unsigned char* buff,      // 输入文件正文的array
										 unsigned long  len,       // 输入文件长度
										 unsigned char* out_buff,  // 输出文件正文的array
										 unsigned long  out_len )  // 输出文件长度
{
	unsigned long  ring_len   = 4096;                        // 寻址范围就是0-4095
	unsigned char* ring       = new unsigned char[ring_len]; // 剪贴板
//	unsigned long ring_index  = 0xFEE;                       // 剪贴板的索引
	unsigned long  ring_index = 1;                           // 剪贴板的索引
	unsigned char* end        = buff + len;                  // 指向输入array的末尾
	unsigned char* out_end    = out_buff + out_len;          // 指向输出array的末尾

	memset(ring, 0, ring_len); // 剪贴板 置零

	while (buff < end && out_buff < out_end) { // 判断条件：不超过array的末尾
		unsigned char flags = *buff++; //<1> 取当前字符存到flags里面
	
		// 这个循环是判断8位的char型的每一位是否为1
		for (int i = 0; i < 8 && buff < end && out_buff < out_end; i++) { // 判断条件：char型长度8循环&&不超过array的末尾
			if (flags & 0x01) { // 判断：最低位是否为1
			/*
			*out_buff = ring[ring_index % ring_len] = *buff++; // 从ring[ 1 ]开始赋值到[ 0 ]，以4096循环
			ring_index++;
			*out_buff++;
			*/
		    *out_buff++ = ring[ring_index++ % ring_len] = *buff++; //<2>
			} else {
				if (end - buff < 2) break;
		
        // p, n.
        // pn = 0xFFFF
        // buffer_offset = (pn & 0x0F00 >> 4) | (pn & 0x00F0 >> 4);  
        // (p & 0x0F) << 4 | (n & 0xF0)
        // n_bytes = 
				unsigned long p = *buff++; //<3>
				unsigned long n = *buff++; //<4>
		
				//p = p | ( ( n >> 4 ) << 8 );
				p = (p << 4) | (n >> 4);   // ( RHL p, 4 ) OR ( RHR n, 4 ) 即 p的八位 与 n的前四位 构成
				n = (n & 0x0F) + 2;        // AND 00001111b; ADD 2 这里用到的是n的后四位
		
				for (unsigned long j = 0; j < n && out_buff < out_end; j++) {
				//for (unsigned long j = 0; j <= n && out_buff < out_end; j++) {
					/*
					*out_buff = ring[ring_index % ring_len] = ring[p % ring_len];
					p++;
					ring_index++;
					*out_buff++;
					*/
					*out_buff++ = ring[ring_index++ % ring_len] = ring[p++ % ring_len];
				}
			}

			flags >>= 1; // 右移1位，进入下一轮判断
		}
	}
	
	delete [] ring;

	return out_len - (out_end - out_buff); // 忽略掉的字符数量
}
*/