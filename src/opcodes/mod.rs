use serde::Serializer;
use serde_derive::{Deserialize, Serialize};
use crate::util::{encode_sjis, get_sjis_bytes, transmute_to_u16};


enum OpFieldType {
    b, // u8
    w, // u16
    d, // u32
    s, // String
    c, // Choice
    p, // 1 byte padding
}

fn define_opcode(input: &[u8], addr: usize, composition: &[OpFieldType]) -> Opcode {
    let mut ptr = 1usize;
    let mut fields = vec![];

    for i in composition {
        match i {
            OpFieldType::s => {
                let (bytes, string) = crate::util::get_sjis_bytes(ptr, input);
                fields.push(OpField::String(TLString { 
                    raw: string,
                    translation: None,
                    notes: None,
                }));
                ptr += bytes.len();
            }
            OpFieldType::b => {
                fields.push(OpField::Byte(input[ptr]));
                ptr += 1;
            }
            OpFieldType::w => {
                fields.push(OpField::Word(crate::util::transmute_to_u16(ptr, input)));
                ptr += 2;
            }
            OpFieldType::d => {
                fields.push(OpField::DWord(crate::util::transmute_to_u32(ptr, input)));
                ptr += 4;
            }
            OpFieldType::p => {
                fields.push(OpField::Padding(1));
                ptr += 1;
            }
            OpFieldType::c => {
                let n_choices = match &fields[0] {
                    OpField::Byte(n) => *n,
                    _ => panic!("Weird shit!")
                };
                let mut choices = vec![];
                let mut curr_ptr = ptr;
                
                for i in 0..n_choices {
                    let mut new_ptr = curr_ptr;
                    let arg1 = transmute_to_u16(new_ptr, input);
                    new_ptr += 2;
                    let (bytes, choice_str) = get_sjis_bytes(new_ptr, input);
                    new_ptr += bytes.len();
                    let arg3 = input[new_ptr];
                    new_ptr += 1;
                    let arg4 = transmute_to_u16(new_ptr, input);
                    new_ptr += 2;
                    let arg5 = input[new_ptr];
                    new_ptr += 1;
                    let arg6 = input[new_ptr];
                    new_ptr += 1;
                    let arg7 = transmute_to_u16(new_ptr, input);
                    new_ptr += 2;
                    let arg8 = input[new_ptr];
                    new_ptr += 1;
                    let arg9 = transmute_to_u16(new_ptr, input);
                    new_ptr += 2;
                    let choice = Choice {
                        arg1,
                        choice_str: TLString {
                            raw: choice_str,
                            translation: None,
                            notes: None,
                        },
                        arg3,
                        arg4,
                        arg5,
                        arg6,
                        arg7,
                        arg8,
                        arg9,
                    };
                    choices.push(choice);
                    curr_ptr = new_ptr + 1; // account for extra padding 0 byte.
                }
                ptr += choices.iter().map(|it| it.size()).sum::<usize>();
                fields.push(OpField::Choice(choices));
            }
            _ => panic!()
        }
    }

    Opcode { opcode: input[0], address: addr, actual_address: addr, fields }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TLString {
    pub raw: String,
    pub translation: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum OpField {
    Byte(
        #[serde(serialize_with = "crate::opcodes::serialize_hex_u8")]
        u8),
    Word(
        #[serde(serialize_with = "crate::opcodes::serialize_hex_u16")]
        u16),
    DWord(
        #[serde(serialize_with = "crate::opcodes::serialize_hex_u32")]
        u32),
    String(TLString),
    Choice(Vec<Choice>),
    Padding(u8),
}

impl OpField {
    fn size(&self) -> usize {
        match self {
            OpField::Byte(_) => 1,
            OpField::Word(_) => 2,
            OpField::DWord(_) => 4,
            OpField::String(TLString { raw, translation, .. }) => {
                if let Some(tl) = translation {
                    encode_sjis(&(tl.clone() + "%K%P"))
                } else {
                    encode_sjis(raw.as_str())
                }.len() + 1
            },
            OpField::Choice(choices) => {
                let mut acc = 0;
                for choice in choices {
                    acc += choice.size();
                }
                acc
            },
            OpField::Padding(size) => *size as usize
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u16")]
    pub arg1: u16,
    pub choice_str: TLString,
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u8")]
    pub arg3: u8,
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u16")]
    pub arg4: u16,
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u8")]
    pub arg5: u8,
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u8")]
    pub arg6: u8,
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u16")]
    pub arg7: u16,
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u8")]
    pub arg8: u8,
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u16")]
    pub arg9: u16,
}
impl Choice {
    fn size(&self) -> usize {
        let str_len = if let Some(tl) = &self.choice_str.translation {
            encode_sjis(tl).len() + 1
        } else {
            encode_sjis(&self.choice_str.raw).len() + 1
        };
        
        2
        + str_len
        + 1
        + 2
        + 1
        + 1
        + 2
        + 1
        + 2
        + 1
    }
}

#[derive(Serialize, Deserialize)]
pub struct Opcode {
    #[serde(serialize_with = "crate::opcodes::serialize_hex_u8")] 
    pub opcode: u8,
    #[serde(serialize_with = "crate::opcodes::serialize_hex_usize")] 
    pub address: usize,
    #[serde(skip)]
    pub actual_address: usize,
    pub fields: Vec<OpField>,
}

#[derive(Serialize, Deserialize)]
pub struct Script {
    pub opcodes: Vec<Opcode>,
    #[serde(serialize_with = "crate::opcodes::serialize_inline_ints_vec")]
    pub trailer: Vec<u8>,
}

impl Opcode {
    pub(crate) fn size(&self) -> usize {
        let mut acc = 1;
        for i in self.fields.iter() {
            acc += i.size();
        }
        acc
    }
}

pub fn serialize_inline_ints_slice<S>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string = format!(
        "[ {} ]",
        data
            .iter()
            .map(|int| format!("0x{int:02X}"))
            .collect::<Vec<_>>()
            .join(", ")
    );

    serializer.serialize_str(&string)
}

#[allow(dead_code)]
pub fn serialize_hex_usize<S>(data: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!(r#""0x{data:08X}""#))
}

pub fn serialize_hex_u32<S>(data: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!(r#""0x{data:08X}""#))
}

pub fn serialize_hex_u16<S>(data: &u16, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!(r#""0x{data:04X}""#))
}

pub fn serialize_opt_hex_u16<S>(data: &Option<u16>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match data {
        Some(inner) => serializer.serialize_str(&format!(r#""0x{inner:04X}""#)),
        None => serializer.serialize_none(),
    }
}

pub fn serialize_hex_u8<S>(data: &u8, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!(r#""0x{data:02X}""#))
}

pub fn serialize_inline_ints_vec<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string = format!(
        "[ {} ]",
        data
            .iter()
            .map(|int| format!("0x{int:02X}"))
            .collect::<Vec<_>>()
            .join(", ")
    );

    serializer.serialize_str(&string)
}


pub fn make_opcode(input: &[u8], addr: usize) -> Option<Opcode> {
    use OpFieldType::*;

    macro_rules! opcode {
        () => {
            define_opcode(input, addr, &[])
        };
        ($($arg:expr),*) => {
            define_opcode(input, addr, &[$($arg,)*])
        };
    }
    
    
    let res = match &input[0] {
        0x01 => opcode!(b, w, w, d, p), // n_byte_opcode(input, 11), // : 11 bytes (1 + 1 + 2 + 2 + 4 + 1)
        0x02 => opcode!(b, p, c),// 0x02: Variable (minimum 4 bytes + choice data)
        // - Choice variant a: Variable (minimum 15 bytes + string length)
        // - Choice variant b: Variable (minimum 16 bytes + 2 string lengths)
        0x03 => opcode!(b, w, b, w, p), // n_byte_opcode(input, 8), // : 8 bytes (1 + 1 + 2 + 1 + 2 + 1)
        0x04 => opcode!(),// n_byte_opcode(input, 1), // 0x04: 1 byte
        0x05 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x06 => opcode!(d, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 2)
        0x07 => opcode!(w, s),// make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x08 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes
        0x09 => opcode!(s), // make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x0A => opcode!(p), // n_byte_opcode(input, 1), //: 1 byte
        0x0B => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x0C => opcode!(w, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x0D => opcode!(w, w, w, p), // n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 4 + 1)
        0x0E => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x21 => opcode!(b, w, b, w, d, s), // make_string_opcode(12, input), //: Variable (minimum 13 bytes + string length)
        0x22 => opcode!(b, w, p), // n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 2 + 1)
        0x23 | 0x27 => opcode!(b, w, w, w, b, b, s), // make_string_opcode(10, input), // 0x27: Variable (minimum 11 bytes + string length)
        0x24 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x25 => opcode!(b, b, w, p, p, b, w, b, s), // make_string_opcode(12, input), //: Variable (minimum 13 bytes + string length)
        0x26 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x28 => opcode!(b, b, p, p, p), // n_byte_opcode(input, 6), //: 6 bytes (1 + 1 + 1 + 3)
        0x29 => opcode!(b, w, p, p), // n_byte_opcode(input, 6), //: 6 bytes (1 + 1 + 2 + 2)
        0x30 => opcode!(b, p, p, p), // n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 3)
        0x31 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x32 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x33 => opcode!(w, w, w, p), // n_byte_opcode(input, 8), //: 8 bytes (1 + 2 + 2 + 2 + 1)
        0x34 => opcode!(w, b, b, s), // make_string_opcode(6, input), //: Variable (minimum 7 bytes + string length)
        0x41 => opcode!(w, b, b, s), // make_string_opcode(5, input), //: Variable (minimum 6 bytes + string length)
        0x42 => opcode!(w, b, b, b, s, s), // make_n_string_opcode(6, 2, input), //: Variable (minimum 7 bytes + 2 string lengths)
        0x43 => opcode!(b, w, w, b, s), // make_string_opcode(7, input), //: Variable (minimum 8 bytes + string length)
        0x44 => opcode!(b, b, b, p), // n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0x45 => opcode!(b, b, b, p), // n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0x46 => opcode!(w, w, p, p, p, b, b, s), // make_string_opcode(10, input), //: Variable (minimum 11 bytes + string length)
        0x47 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x48 => opcode!(b, w, w, d, b, b, s), // n_byte_opcode(input, 13), //: 13 bytes (1 + 11 + 1)
        0x49 => opcode!(w, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x4A => opcode!(b, w, w, p), // n_byte_opcode(input, 8), //: 8 bytes (1 + 1 + 2 + 2 + 1)
        0x4B => opcode!(b, w, w, d, w, d, d, p), // n_byte_opcode(input, 24), //: 24 bytes (1 + 1 + 2 + 2 + 4 + 2 + 4 + 4 + 1)
        0x4C => opcode!(b, b, d, p, p), // n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 1 + 4 + 1 + 1)
        0x4D => opcode!(b, b, w, w, w, w, w, p), // n_byte_opcode(input, 15), //: 15 bytes (1 + 1 + 1 + 2 + 2 + 2 + 2 + 2 + 1)
        0x4E => opcode!(b, b, b, p), // n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0x4F => opcode!(b, b, b, p), // n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0x50 => opcode!(s), // make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x51 => opcode!(w, w, p), // n_byte_opcode(input, 6), //: 6 bytes (1 + 2 + 2 + 1)
        0x52 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x53 => opcode!(b, w, w, s), // make_string_opcode(6, input), //: Variable (minimum 7 bytes + string length)
        0x54 => opcode!(s), // make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x55 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x56 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x57 => opcode!(w, w, d, p), // n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 4 + 1)
        0x58 => opcode!(b, b, b, w, w, p), // n_byte_opcode(input, 10), //: 10 bytes (1 + 1 + 1 + 1 + 2 + 2 + 1)
        0x59 => opcode!(s), // make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x60 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x61 => opcode!(b, s), // make_string_opcode(2, input), //: Variable (minimum 3 bytes + string length)
        0x62 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x63 => opcode!(b, b, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0x64 => opcode!(b, w, w, w, p), // n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 2 + 2 + 2 + 1)
        0x65 => opcode!(w, w, p), // n_byte_opcode(input, 6), //: 6 bytes (1 + 2 + 2 + 1)
        0x66 => opcode!(b, w, w, b, w, d, w, d, d), // n_byte_opcode(input, 24), //: 24 bytes (1 + 1 + 2 + 2 + 1 + 2 + 4 + 2 + 4 + 4)
        0x67 => opcode!(b, b, p, d, p), // n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 1 + 1 + 4 + 1)
        0x68 => opcode!(w, w, w, w, p), // n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 2 + 2 + 1)
        0x69 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x70 => opcode!(b, b, p, d, p), // n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 1 + 1 + 4 + 1)
        0x71 => opcode!(s), // make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x72 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x73 => opcode!(w, w, d, b, s), // make_string_opcode(9, input), //: Variable (minimum 10 bytes + string length)
        0x74 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x75 => opcode!(w, w, w, w, p), // n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 2 + 2 + 1)
        0x76 => opcode!(w, w, d, b, b, w, d, p), // n_byte_opcode(input, 18), //: 18 bytes (1 + 2 + 2 + 4 + 1 + 1 + 2 + 4 + 1)
        0x77 => opcode!(w, w, d, p), // n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 4 + 1)
        0x78 => opcode!(b, b, b, d, p), // n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 1 + 1 + 4 + 1)
        0x79 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x81 => opcode!(p, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x82 => opcode!(w, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x83 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x84 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x85 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x86 => opcode!(p, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x87 => opcode!(w, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x88 => opcode!(p, p, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0x89 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x8A => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x8B => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x8C => opcode!(w, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x8D => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x8E => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xA0 => opcode!(w, w, b, p), // n_byte_opcode(input, 7), //: 7 bytes (1 + 2 + 2 + 1 + 1)
        0xA1 => opcode!(b, w, w, b, p), // n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 2 + 2 + 1 + 1)
        0xA2 => opcode!(b, w, w, p), // n_byte_opcode(input, 8), //: 8 bytes (1 + 1 + 2 + 2 + 1)
        0xA3 => opcode!(p, w, w, p), // n_byte_opcode(input, 7), //: 7 bytes (1 + 1 + 2 + 2 + 1)
        0xA4 => opcode!(p, w, w, p), // n_byte_opcode(input, 7), //: 7 bytes (1 + 1 + 2 + 2 + 1)
        0xA5 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xA6 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xA7 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xA8 => opcode!(b, b, b, p, p, p, p, w, w, w, w, p), // n_byte_opcode(input, 19), //: 19 bytes (1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 2 + 2 + 2 + 1)
        0xA9 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xAA => opcode!(b, b, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0xAB => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xAC => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xAD => opcode!(b, d, d, p), // n_byte_opcode(input, 11), //: 11 bytes (1 + 1 + 4 + 4 + 1)
        0xAE => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xB1 => opcode!(w, w, p), // n_byte_opcode(input, 6), //: 6 bytes (1 + 2 + 2 + 1)
        0xB2 => opcode!(b, p, s), // make_string_opcode(3, input), //: Variable (minimum 4 bytes + string length)
        0xB3 => opcode!(p, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xB4 => opcode!(p, p, w, w, d, b, p), // n_byte_opcode(input, 14), //: 14 bytes (1 + 1 + 1 + 2 + 2 + 4 + 1 + 1)
        0xB5 => opcode!(b, b, p, p, p, p, p), // n_byte_opcode(input, 8), //: 8 bytes (1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)
        0xB6 => opcode!(w, s), // make_string_opcode(3, input), //: Variable (minimum 4 bytes + string length)
        0xB7 => opcode!(b, w, w, s), // make_string_opcode(6, input), //: Variable (minimum 7 bytes + string length)
        0xB8 => opcode!(b, b, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0xB9 => opcode!(b, b, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0xBA => opcode!(w, w, b, b, b, b, b, w, s), // make_string_opcode(12, input), //: Variable (minimum 13 bytes + string length)
        0xBB => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xBC => opcode!(b, b, b, p), // n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0xBD => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xBE => opcode!(b, b, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0xBF => opcode!(b, b, b, w, p), // n_byte_opcode(input, 7), //: 7 bytes (1 + 1 + 1 + 1 + 2 + 1)
        0xE0 => opcode!(s), // make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0xE2 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xE3 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xE4 => opcode!(b, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xE5 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xE6 => opcode!(p, p), // n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xE7 => opcode!(w, p), // n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0xE8 => opcode!(s), // make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0xE9 => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xEA => opcode!(b, s), // make_string_opcode(2, input), //: Variable (minimum 3 bytes + string length)
        0xEB => opcode!(p), // n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xFF => opcode!(), // n_byte_opcode(input, 1), //: 1 byte
        _ => {
            log::error!("Unknown opcode 0x{:02X}", &input[0]);
            return None;
        }
    };
    
    Some(res)
}