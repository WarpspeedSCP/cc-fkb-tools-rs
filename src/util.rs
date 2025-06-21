pub fn transmute_to_array<const SIZE: usize>(address: usize, input: &[u8]) -> [u8; SIZE] {
  input[address..address + SIZE].try_into().unwrap()
}

pub fn transmute_to_u32(address: usize, input: &[u8]) -> u32 {
  u32::from_le_bytes(transmute_to_array(address, input))
}

pub fn transmute_to_u16(address: usize, input: &[u8]) -> u16 {
  u16::from_le_bytes(transmute_to_array(address, input))
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

    if in_italics {
      let mut word = vec![];
      for chr in substr.chars() {}
      let output = word;
      collector.push(output);
    } else {
      let output = SHIFT_JIS.encode(&substr).0.to_vec();
      collector.push(output);
    }
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
