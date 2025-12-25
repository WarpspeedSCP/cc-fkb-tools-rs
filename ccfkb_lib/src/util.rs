
pub fn transmute_to_array<const SIZE: usize>(address: usize, input: &[u8]) -> [u8; SIZE] {
  input[address..address + SIZE].try_into().unwrap()
}

pub fn transmute_to_u32(address: usize, input: &[u8]) -> u32 {
  u32::from_le_bytes(transmute_to_array(address, input))
}

pub fn transmute_to_u16(address: usize, input: &[u8]) -> u16 {
  u16::from_le_bytes(transmute_to_array(address, input))
}

pub fn current_dir() -> camino::Utf8PathBuf {
  camino::Utf8PathBuf::from_path_buf(std::env::current_dir().unwrap()).unwrap()
}

pub fn safe_create_dir(dir: &camino::Utf8Path) -> std::io::Result<()> {
  std::fs::create_dir(dir).or_else(|it| {
    if it.kind() == std::io::ErrorKind::AlreadyExists {
      Ok(())
    } else {
      Err(it)
    }
  })
}

pub fn ends_with_ignore_case(a: &dyn AsRef<str>, b: &dyn AsRef<str>) -> bool {
  a.as_ref().ends_with(b.as_ref()) || a.as_ref().to_ascii_uppercase().ends_with(&b.as_ref().to_ascii_uppercase())
}

pub fn escape_str(input: &str, add_suffix: bool) -> String {
  (input.to_string() + if add_suffix { "%K%P" } else { "" })
      .replace("\\", "<bslash/>")
      .replace("\"", "<dquote/>")
      .trim()
      .to_string()
}

pub fn unescape_str(input: &str) -> String {
  input
      .trim_end_matches("%K%P")
      .replace("<bslash/>", "\\")
      .replace("<dquote/>", "\"")
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SJISChar {
  SingleByte(u8),
  DoubleByte(u16),
}

impl SJISChar {
  fn new(value: char) -> Self {
    if value.is_ascii() {
      Self::SingleByte(value as u8)
    } else {
      use encoding_rs::SHIFT_JIS;
      let sjis = SHIFT_JIS.encode(&format!("{value}")).0.to_vec();
      let sjis_u16 = transmute_to_u16(0, &sjis);
      Self::DoubleByte(sjis_u16)
    }
  }
  fn from_number(value: u16) -> Self {
    if value <= u8::MAX as u16 {
      Self::SingleByte(value as u8)
    } else {
      Self::DoubleByte(value as u16)
    }
  }

  fn to_vec(&self) -> Vec<u8> {
    match self {
      Self::DoubleByte(value) => value.to_be_bytes().to_vec(),
      Self::SingleByte(value) => value.to_be_bytes().to_vec(),
    }
  }
}

// Tokenises a string into its constituents.
fn tokens(unicode: &str) -> Vec<String> {
  let mut curr_str = String::new();
  let mut result = vec![];
  let chrs = unicode
    .replace("<dquote/>", "\"")
    .replace("<bslash/>", "\\")
    .chars()
    .collect::<Vec<_>>();
  let mut char_idx = 0usize;
  loop {
    let chr = chrs.get(char_idx);
    if chr.is_none() {
      result.push(std::mem::take(&mut curr_str));
      break;
    }

    let chr = *chr.unwrap();
    char_idx += 1;

    if chr.is_whitespace() {
      if !curr_str.is_empty() {
        result.push(std::mem::take(&mut curr_str));
      }
      curr_str.push(chr);
      let mut cont = false;
      while let Some(&chr) = chrs.get(char_idx) {
        if !chr.is_whitespace() {
          if !curr_str.is_empty() {
            result.push(std::mem::take(&mut curr_str));
          }
          cont = true;
          break;
        }
        char_idx += 1;
        curr_str.push(chr);
      }

      if cont {
        continue;
      }
    } else if chr == '\\' {
      if !curr_str.is_empty() {
        result.push(std::mem::take(&mut curr_str));
      }
      if chrs.get(char_idx) == Some(&'*') {
        char_idx += 1;
        curr_str = "\\*".to_owned();
      } else {
        curr_str.push('\\');
        if let Some(c) = chrs.get(char_idx) {
          char_idx += 1;
          curr_str.push(*c);
        }
      }
      result.push(std::mem::take(&mut curr_str));
    } else if chr == '*' {
      if !curr_str.is_empty() {
        result.push(std::mem::take(&mut curr_str));
      }
      curr_str.push(chr);
      result.push(std::mem::take(&mut curr_str));
    } else {
      curr_str.push(chr);
    }
  }
  result
}

/// Does not null terminate the resultant string.
pub fn encode_sjis(unicode: &str) -> Vec<u8> {
  use encoding_rs::SHIFT_JIS;
  let mut in_italics = false;
  let mut collector = vec![];
  let tokens = tokens(unicode);
  let mut bslash_active = false;

  for substr in tokens {
    let substr = substr;

    if substr == "\\" && !bslash_active {
      bslash_active = true;
      continue;
    }

    if substr == "*" {
      if in_italics {
        in_italics = false;
      } else {
        in_italics = true;
      }
      continue;
    }

    if bslash_active {
      bslash_active = false;
    }

    if substr == "\\*" {
      collector.push(['*' as u8].to_vec());
      continue;
    }

    let output = SHIFT_JIS.encode(&substr).0.to_vec();
    collector.push(output);
  }

  let output: Vec<u8> = collector.into_iter().flatten().collect();

  output
}

pub fn get_sjis_bytes(address: usize, input: &[u8]) -> (Vec<u8>, String) {
  let mut size = 0usize;
  let mut output = vec![];
  while input[address + size] != 0 && size < 1024 {
    output.push(input[address + size]);
    size += 1;
  }
  output.push(0);
  use encoding_rs::SHIFT_JIS;
  let encoded = SHIFT_JIS.decode(&output[..size]).0.to_string();
  (output, encoded)
}

pub fn get_sjis_bytes_of_length(address: usize, size: usize, input: &[u8]) -> (Vec<u8>, String) {
  let mut output = vec![];
  let mut ptr = 0usize;
  while ptr < size && (address + ptr) < input.len() {
    output.push(input[address + ptr]);
    ptr += 1;
  }
  output.push(0);
  use encoding_rs::SHIFT_JIS;
  let first_null_idx = output.iter().position(|it| *it == 0).unwrap_or(0);
  let encoded = SHIFT_JIS.decode(&output[..first_null_idx]).0.to_string();
  (output, encoded)
}

pub fn to_bytes<T: Sized>(value: &T) -> &[u8] {
  unsafe {
    let data = std::mem::transmute(value as *const T);
    &*core::ptr::slice_from_raw_parts(data, size_of_val(value))
  }
}

// unsigned long CrossChannelCrack::unwipf( unsigned char* buff,      // 输入文件正文的array
// 										 unsigned long  len,       // 输入文件长度
// 										 unsigned char* out_buff,  // 输出文件正文的array
// 										 unsigned long  out_len )  // 输出文件长度
// {
pub fn unwipf(input: &[u8], out_len: usize) -> Vec<u8> {
  let mut buff = 0usize;

  // 	unsigned long  ring_len   = 4096;                        // 寻址范围就是0-4095
  // 	unsigned char* ring       = new unsigned char[ring_len]; // 剪贴板
  let mut ring = [0u8; 4096];
  // //	unsigned long ring_index  = 0xFEE;                       // 剪贴板的索引
  // 	unsigned long  ring_index = 1;                           // 剪贴板的索引
  let mut ring_index = 0xFEEusize;
  // 	unsigned char* end        = buff + len;                  // 指向输入array的末尾
  let end = input.len();

  let mut out = vec![0u8; out_len];
  let mut out_buff = 0usize;
  // 	unsigned char* out_end    = out_buff + out_len;          // 指向输出array的末尾
  let out_end = out.len();

  // 	memset(ring, 0, ring_len); // 剪贴板 置零

  // 	while (buff < end && out_buff < out_end) { // 判断条件：不超过array的末尾
  while buff < end && out_buff < out_end {
    // 		unsigned char flags = *buff++; //<1> 取当前字符存到flags里面
    let mut flags = input[buff];
    buff += 1;
    // 		// 这个循环是判断8位的char型的每一位是否为1
    // 		for (int i = 0; i < 8 && buff < end && out_buff < out_end; i++) { // 判断条件：char型长度8循环&&不超过array的末尾
    for _ in 0..8 {
      if buff >= end || out_buff >= out_end {
        break;
      }
      // 			if (flags & 0x01) { // 判断：最低位是否为1
      if flags & 1 == 1 {
        // 			/*
        // 			*out_buff = ring[ring_index % ring_len] = *buff++; // 从ring[ 1 ]开始赋值到[ 0 ]，以4096循环
        out[out_buff] = ring[ring_index % 4096];
        // 			ring_index++;
        ring_index += 1;
        // 			*out_buff++;
        out_buff += 1;
      // 			*/
      // 		    *out_buff++ = ring[ring_index++ % ring_len] = *buff++; //<2>
      // 			} else {
      } else {
        // 				if (end - buff < 2) break;
        if (end - buff) < 2 {
          break;
        }
        // 				unsigned long p = *buff++; //<3>
        let p = input[buff] as u64;
        buff += 1;
        // 				unsigned long n = *buff++; //<4>
        let n = input[buff] as u64;
        buff += 1;
        // 				//p = p | ( ( n >> 4 ) << 8 );
        // 				p = (p << 4) | (n >> 4);   // ( RHL p, 4 ) OR ( RHR n, 4 ) 即 p的八位 与 n的前四位 构成
        let mut p1 = (p << 4) | (n >> 4);
        // 				n = (n & 0x0F) + 2;        // AND 00001111b; ADD 2 这里用到的是n的后四位
        let n1 = (n & 0x0F) + 2;
        // 				for (unsigned long j = 0; j < n && out_buff < out_end; j++) {
        for _ in 0..n1 {
          if out_buff >= out_end {
            break;
          }
          // 				//for (unsigned long j = 0; j <= n && out_buff < out_end; j++) {
          // 					/*
          // 					*out_buff = ring[ring_index % ring_len] = ring[p % ring_len];
          ring[ring_index % 4096] = ring[(p1 as usize) % 4096];
          out[out_buff] = ring[ring_index % 4096];
          // 					p++;
          p1 += 1;
          // 					ring_index++;
          ring_index += 1;
          // 					*out_buff++;
          out_buff += 1;
          // 					*/
          // 					*out_buff++ = ring[ring_index++ % ring_len] = ring[p++ % ring_len];
          // 				}
        }
        // 			}
      }
      // 			flags >>= 1; // 右移1位，进入下一轮判断
      flags >>= 1;
      // 		}
    }
    // 	}
  }

  out
  // 	delete [] ring;

  // 	return out_len - (out_end - out_buff); // 忽略掉的字符数量
  // }
}