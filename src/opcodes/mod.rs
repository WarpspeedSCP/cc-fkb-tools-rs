
fn make_string_opcode(header_size: usize, input: &[u8]) -> Vec<u8> {
    make_n_string_opcode(header_size, 1, input)
    // let mut tmp = input[0..header_size].to_vec();
    // for i in &input[header_size..] {
    //     tmp.push(*i);
    //     if *i == 0 {
    //         break;
    //     }
    // }
    // tmp
}

fn make_n_string_opcode(header_size: usize, n_strings: usize, input: &[u8]) -> Vec<u8> {
    let mut tmp = input[0..header_size].to_vec();
    let mut header_size = header_size;
    for _ in 0..n_strings {
        for i in &input[header_size..] {
            tmp.push(*i);
            if *i == 0 {
                break;
            }
        }
        header_size = tmp.len();
    }
    
    tmp
}

fn n_byte_opcode(input: &[u8], size: usize) -> Vec<u8> {
    input[..size].to_vec()
}

enum Opcode {
    Normal(Vec<u8>),
    Jump,
    Choice(),
    SingleString,
    
}

pub fn make_opcode(input: &[u8]) -> Option<Vec<u8>> {

    let res = match &input[0] {
        0x01 => n_byte_opcode(input, 11), // : 11 bytes (1 + 1 + 2 + 2 + 4 + 1)
        // 0x02: Variable (minimum 4 bytes + choice data)
        // - Choice variant a: Variable (minimum 15 bytes + string length)
        // - Choice variant b: Variable (minimum 16 bytes + 2 string lengths)
        0x03 => n_byte_opcode(input, 8), // : 8 bytes (1 + 1 + 2 + 1 + 2 + 1)
        0x04 => n_byte_opcode(input, 1), // 0x04: 1 byte
        0x05 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x06 => n_byte_opcode(input, 3), //: 3 bytes (1 + 2)
        0x07 => make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x08 => n_byte_opcode(input, 2), //: 2 bytes
        0x09 => make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x0A => n_byte_opcode(input, 1), //: 1 byte
        0x0B => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x0C => n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x0D => n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 4 + 1)
        0x0E => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x21 => make_string_opcode(12, input), //: Variable (minimum 13 bytes + string length)
        0x22 => n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 2 + 1)
        0x23 | 0x27 => make_string_opcode(10, input), // 0x27: Variable (minimum 11 bytes + string length)
        0x24 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x25 => make_string_opcode(12, input), //: Variable (minimum 13 bytes + string length)
        0x26 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x28 => n_byte_opcode(input, 6), //: 6 bytes (1 + 1 + 1 + 3)
        0x29 => n_byte_opcode(input, 6), //: 6 bytes (1 + 1 + 2 + 2)
        0x30 => n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 3)
        0x31 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x32 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x33 => n_byte_opcode(input, 8), //: 8 bytes (1 + 2 + 2 + 2 + 1)
        0x34 => make_string_opcode(6, input), //: Variable (minimum 7 bytes + string length)
        0x41 => make_string_opcode(5, input), //: Variable (minimum 6 bytes + string length)
        0x42 => make_n_string_opcode(6, 2, input), //: Variable (minimum 7 bytes + 2 string lengths)
        0x43 => make_string_opcode(7, input), //: Variable (minimum 8 bytes + string length)
        0x44 => n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0x45 => n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0x46 => make_string_opcode(10, input), //: Variable (minimum 11 bytes + string length)
        0x47 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x48 => n_byte_opcode(input, 13), //: 13 bytes (1 + 11 + 1)
        0x49 => n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x4A => n_byte_opcode(input, 8), //: 8 bytes (1 + 1 + 2 + 2 + 1)
        0x4B => n_byte_opcode(input, 24), //: 24 bytes (1 + 1 + 2 + 2 + 4 + 2 + 4 + 4 + 1)
        0x4C => n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 1 + 4 + 1 + 1)
        0x4D => n_byte_opcode(input, 15), //: 15 bytes (1 + 1 + 1 + 2 + 2 + 2 + 2 + 2 + 1)
        0x4E => n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0x4F => n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0x50 => make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x51 => n_byte_opcode(input, 6), //: 6 bytes (1 + 2 + 2 + 1)
        0x52 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x53 => make_string_opcode(6, input), //: Variable (minimum 7 bytes + string length)
        0x54 => make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x55 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x56 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x57 => n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 4 + 1)
        0x58 => n_byte_opcode(input, 10), //: 10 bytes (1 + 1 + 1 + 1 + 2 + 2 + 1)
        0x59 => make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x60 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x61 => make_string_opcode(2, input), //: Variable (minimum 3 bytes + string length)
        0x62 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x63 => n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0x64 => n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 2 + 2 + 2 + 1)
        0x65 => n_byte_opcode(input, 6), //: 6 bytes (1 + 2 + 2 + 1)
        0x66 => n_byte_opcode(input, 24), //: 24 bytes (1 + 1 + 2 + 2 + 1 + 2 + 4 + 2 + 4 + 4)
        0x67 => n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 1 + 1 + 4 + 1)
        0x68 => n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 2 + 2 + 1)
        0x69 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x70 => n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 1 + 1 + 4 + 1)
        0x71 => make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0x72 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x73 => make_string_opcode(9, input), //: Variable (minimum 10 bytes + string length)
        0x74 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x75 => n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 2 + 2 + 1)
        0x76 => n_byte_opcode(input, 18), //: 18 bytes (1 + 2 + 2 + 4 + 1 + 1 + 2 + 4 + 1)
        0x77 => n_byte_opcode(input, 10), //: 10 bytes (1 + 2 + 2 + 4 + 1)
        0x78 => n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 1 + 1 + 4 + 1)
        0x79 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x81 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x82 => n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x83 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x84 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x85 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x86 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0x87 => n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x88 => n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0x89 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x8A => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x8B => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x8C => n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0x8D => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0x8E => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xA0 => n_byte_opcode(input, 7), //: 7 bytes (1 + 2 + 2 + 1 + 1)
        0xA1 => n_byte_opcode(input, 9), //: 9 bytes (1 + 1 + 2 + 2 + 1 + 1)
        0xA2 => n_byte_opcode(input, 8), //: 8 bytes (1 + 1 + 2 + 2 + 1)
        0xA3 => n_byte_opcode(input, 7), //: 7 bytes (1 + 1 + 2 + 2 + 1)
        0xA4 => n_byte_opcode(input, 7), //: 7 bytes (1 + 1 + 2 + 2 + 1)
        0xA5 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xA6 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xA7 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xA8 => n_byte_opcode(input, 19), //: 19 bytes (1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 2 + 2 + 2 + 1)
        0xA9 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xAA => n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0xAB => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xAC => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xAD => n_byte_opcode(input, 11), //: 11 bytes (1 + 1 + 4 + 4 + 1)
        0xAE => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xB1 => n_byte_opcode(input, 6), //: 6 bytes (1 + 2 + 2 + 1)
        0xB2 => make_string_opcode(3, input), //: Variable (minimum 4 bytes + string length)
        0xB3 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xB4 => n_byte_opcode(input, 14), //: 14 bytes (1 + 1 + 1 + 2 + 2 + 4 + 1 + 1)
        0xB5 => n_byte_opcode(input, 8), //: 8 bytes (1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)
        0xB6 => make_string_opcode(3, input), //: Variable (minimum 4 bytes + string length)
        0xB7 => make_string_opcode(6, input), //: Variable (minimum 7 bytes + string length)
        0xB8 => n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0xB9 => n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0xBA => make_string_opcode(12, input), //: Variable (minimum 13 bytes + string length)
        0xBB => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xBC => n_byte_opcode(input, 5), //: 5 bytes (1 + 1 + 1 + 1 + 1)
        0xBD => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xBE => n_byte_opcode(input, 4), //: 4 bytes (1 + 1 + 1 + 1)
        0xBF => n_byte_opcode(input, 7), //: 7 bytes (1 + 1 + 1 + 1 + 2 + 1)
        0xE0 => make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0xE2 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xE3 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xE4 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xE5 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xE6 => n_byte_opcode(input, 3), //: 3 bytes (1 + 1 + 1)
        0xE7 => n_byte_opcode(input, 4), //: 4 bytes (1 + 2 + 1)
        0xE8 => make_string_opcode(1, input), //: Variable (minimum 2 bytes + string length)
        0xE9 => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xEA => make_string_opcode(2, input), //: Variable (minimum 3 bytes + string length)
        0xEB => n_byte_opcode(input, 2), //: 2 bytes (1 + 1)
        0xFF => n_byte_opcode(input, 1), //: 1 byte
        _ => {
            log::error!("Unknown opcode 0x{:02X}", &input[8]);
            return None;
        }
    };
    
    Some(res)
}