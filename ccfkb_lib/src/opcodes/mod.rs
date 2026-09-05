use crate::util::{encode_sjis, get_sjis_bytes, transmute_to_u16};
use itertools::Itertools;
use serde::Serializer;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TLString {
	pub raw: String,
	pub translation: Option<String>,
	pub notes: Option<String>,
}

impl TLString {
	fn bytecode_serialise(&self) -> Vec<u8> {
		let mut output = if let Some(tl) = &self.translation {
			encode_sjis(tl)
		} else {
			encode_sjis(&self.raw)
		};

		// Terminate the string.
		output.push(0);

		output
	}
}

#[derive(Serialize, Deserialize, Clone)]
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
	#[serde(serialize_with = "crate::opcodes::serialize_inline_ints_vec")]
	Padding(Vec<u8>),
}

impl OpField {
	fn as_byte(&self) -> Option<u8> {
		match &self {
			OpField::Byte(b) => Some(*b),
			_ => None,
		}
	}
	fn as_word(&self) -> Option<u16> {
		match &self {
			OpField::Word(w) => Some(*w),
			_ => None,
		}
	}

	fn as_dword(&self) -> Option<u32> {
		match &self {
			OpField::DWord(d) => Some(*d),
			_ => None,
		}
	}

	fn size(&self) -> usize {
		match self {
			OpField::Byte(_) => 1,
			OpField::Word(_) => 2,
			OpField::DWord(_) => 4,
			OpField::String(tlstr) => tlstr.bytecode_serialise().len(),
			OpField::Choice(choices) => {
				let mut acc = 0;
				for choice in choices {
					acc += choice.size();
				}
				acc
			}
			OpField::Padding(contents) => contents.len()
		}
	}

	fn binary_serialise(&self) -> Vec<u8> {
		let mut buf = vec![];

		match self {
			OpField::Byte(value) => buf.push(*value),
			OpField::Word(value) => buf.extend(value.to_le_bytes()),
			OpField::DWord(value) => buf.extend(value.to_le_bytes()),
			OpField::String(value) => buf.extend(value.bytecode_serialise()),
			OpField::Choice(choices) => {
				for choice in choices {
					buf.extend(choice.binary_serialise());
				}
			}
			OpField::Padding(contents) => buf.extend_from_slice(contents),
		};

		buf
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Choice {
	#[serde(serialize_with = "crate::opcodes::serialize_hex_u16")]
	pub arg1: u16,
	pub choice_str: TLString,
	#[serde(serialize_with = "crate::opcodes::serialize_inline_ints_vec")]
	pub trailer: Vec<u8>,
}

impl Choice {
	fn size(&self) -> usize {
		let str_len = if let Some(tl) = &self.choice_str.translation {
			encode_sjis(tl).len() + 1
		} else {
			encode_sjis(&self.choice_str.raw).len() + 1
		};

		2 + str_len + self.trailer.len()
	}
	fn binary_serialise(&self) -> Vec<u8> {
		let mut buf = vec![];
		buf.extend(self.arg1.to_le_bytes());
		buf.extend(self.choice_str.bytecode_serialise());
		buf.extend(&self.trailer);
		buf
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Opcode {
	pub name: String,
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

impl Script {
	pub fn binary_serialise(mut self) -> Vec<u8> {
		let mut buf = vec![];

		let mut jump_map: HashMap<u32, usize> = HashMap::new();
		let mut actual_address = self
			.opcodes
			.first()
			.map(|it| it.address)
			.unwrap_or_default();

		let orig_opcodes = self.opcodes.clone();

		log::debug!("Actual address start is 0x{actual_address:08X}");
		for opcode in self.opcodes.iter_mut() {
			match opcode.opcode {
				0x06 => {
					let (idx, orig_op) = orig_opcodes
						.iter().find_position(|it| it.address == (opcode.fields[0].as_dword().unwrap() as usize))
						.unwrap();
					log::debug!(
            "Direct jump opcode at 0x{:08X} (actual 0x{:08X}) jumps to: 0x{:04X}",
            opcode.address,
            actual_address,
            orig_op.address,
          );
					jump_map.insert(opcode.address as u32, idx);
				}
				0x01 => {
					let (idx, orig_op) = orig_opcodes.iter().find_position(|it| it.address == (opcode.address + 11 + opcode.fields[3].as_dword().unwrap() as usize))
						.unwrap();

					jump_map.insert(opcode.address as u32, idx);
					log::debug!(
            "Conditional jump Opcode at 0x{:08X} (actual {:08X}) jumps to: {:08X}",
            opcode.address,
            actual_address,
            orig_op.address
          );
				}
				_ => {}
			}
			opcode.actual_address = actual_address;
			actual_address += opcode.size();
		}


		for op in &self.opcodes {
			let op = adjust_single_opcode(op.clone(), &jump_map, &self.opcodes);
			let serialised = op.binary_serialise();
			buf.extend(serialised);
		}

		buf.extend(&self.trailer);

		buf
	}
}

fn adjust_single_opcode(
	opcode: Opcode,
	jump_table: &HashMap<u32, usize>,
	opcodes: &[Opcode],
) -> Opcode {
	let mut opcode = opcode;
	match opcode.opcode {
		0x06 => {
			let tbl_entry = jump_table[&(opcode.address as u32)];
			opcode.fields[0] = OpField::DWord(opcodes[tbl_entry].actual_address as u32);
			log::debug!(
        "Adjusting direct jump Opcode at 0x{:08X} (actual {:08X}) to jump to: {:08X}",
        opcode.address,
        opcode.actual_address,
        opcode.fields[0].as_word().unwrap(),
      );
			opcode
		}
		// conditional jump
		0x01 => {
			let tbl_entry = jump_table[&(opcode.address as u32)];
			let curr_actual_address = opcode.actual_address;
			let target_address = opcodes[tbl_entry].actual_address;
			let offset = target_address - (curr_actual_address + 11);
			opcode.fields[3] = OpField::DWord(offset as u32);
			log::debug!(
        "Adjusting conditional jump Opcode ({:02X}) at 0x{:08X} (actual 0x{:08X}) originally targetting {:08X} to jump to offset: 0x{:04X} (0x{:08X})",
        opcode.opcode,
        opcode.address,
        opcode.actual_address,
        opcodes[tbl_entry].address,
        offset,
        target_address
      );
			opcode
		}
		_ => opcode,
	}
}


impl Opcode {
	pub(crate) fn size(&self) -> usize {
		let mut acc = 1;
		for i in self.fields.iter() {
			acc += i.size();
		}
		acc
	}

	pub(crate) fn binary_serialise(&self) -> Vec<u8> {
		let mut buf = vec![self.opcode];

		for field in &self.fields {
			buf.extend(field.binary_serialise());
		}

		buf
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

fn make_choice(input: &[u8]) -> Choice {
	let mut ptr = 0;

	let arg1 = transmute_to_u16(ptr, input);
	ptr += 2;

	let (bytes, choice_str) = get_sjis_bytes(ptr, input);
	ptr += bytes.len();

	let trailer = &input[ptr..(ptr + 11)];

	Choice {
		arg1,
		choice_str: TLString {
			raw: choice_str,
			translation: None,
			notes: None,
		},
		trailer: trailer.to_vec(),
	}
}

pub fn make_opcode(input: &[u8], addr: usize) -> Option<Opcode> {
	let mut ptr = 1usize;
	let mut fields = vec![];
	let mut name: &'static str;

	macro_rules! expand_opcode_component {
        (c) => {
            {
                let n_choices = match &fields[0] {
                    OpField::Byte(n) => *n,
                    _ => panic!("Weird shit!")
                };
                let mut choices = vec![];
                let mut curr_ptr = ptr;
                for _ in 0..n_choices {
                  let choice = make_choice(&input[curr_ptr..]);
                  curr_ptr += choice.size();
                  choices.push(choice);
                }
                ptr += choices.iter().map(|it| it.size()).sum::<usize>();
                fields.push(OpField::Choice(choices));
            }
        };
        (s) => {
            {
                let (bytes, string) = crate::util::get_sjis_bytes(ptr, input);
                fields.push(OpField::String(TLString {
                    raw: string,
                    translation: None,
                    notes: None,
                }));
                ptr += bytes.len();
            }
        };
        (b) => {
            {
                fields.push(OpField::Byte(input[ptr]));
                ptr += 1;
            }
        };
        (w) => {
            {
                fields.push(OpField::Word(crate::util::transmute_to_u16(ptr, input)));
                ptr += 2;
            }
        };
        (d) => {
            {
                fields.push(OpField::DWord(crate::util::transmute_to_u32(ptr, input)));
                ptr += 4;
            }
        };
        (p) => {
            {
				if let Some(OpField::Padding(contents)) = fields.last_mut() {
					contents.push(input[ptr]);
				} else {
					fields.push(OpField::Padding(vec![input[ptr]]));
				}
                // fields.push(OpField::Padding(1));
                ptr += 1;
            }
        };
    }

	macro_rules! expand_opcode_inner {
        () => {
            {

            }
        };
        (c, $($tail:tt)*) => {
                expand_opcode_component!(c);

                expand_opcode_inner!($($tail)*)
        };
        (s, $($tail:tt)*) => {
                expand_opcode_component!(s);

                expand_opcode_inner!($($tail)*)
        };
        (b, $($tail:tt)*) => {
                expand_opcode_component!(b);

                expand_opcode_inner!($($tail)*)
        };
        (w, $($tail:tt)*) => {
                expand_opcode_component!(w);

                expand_opcode_inner!($($tail)*)
        };
        (d, $($tail:tt)*) => {
                expand_opcode_component!(d);

                expand_opcode_inner!($($tail)*)
        };
        (p, $($tail:tt)*) => {
                expand_opcode_component!(p);

                expand_opcode_inner!($($tail)*)
        };
    }
	
	macro_rules! expand_opcode {
        ($name: expr) => {
			{
				name = $name;
			}
        };
        ($name: expr, c, $($tail:tt)*) => {
            name = $name;

			expand_opcode_component!(c);

            expand_opcode_inner!($($tail)*)
        };
        ($name: expr, s, $($tail:tt)*) => {
			name = $name;

            expand_opcode_component!(s);

            expand_opcode_inner!($($tail)*)
        };
        ($name: expr, b, $($tail:tt)*) => {
        	name = $name;

		    expand_opcode_component!(b);

            expand_opcode_inner!($($tail)*)
        };
        ($name: expr, w, $($tail:tt)*) => {
    		name = $name;

	        expand_opcode_component!(w);

            expand_opcode_inner!($($tail)*)
        };
        ($name: expr, d, $($tail:tt)*) => {
            name = $name;

			expand_opcode_component!(d);

            expand_opcode_inner!($($tail)*)
        };
        ($name: expr, p, $($tail:tt)*) => {
    		name = $name;

	        expand_opcode_component!(p);

            expand_opcode_inner!($($tail)*)
        };
    }

	match &input[0] {
		0x01 => { expand_opcode!("conditional_branch", b, w, w, d, p,); } 
		0x02 => { expand_opcode!("choice_jump", b, p, c,); } 
		0x03 => { expand_opcode!("variable_heap_op", b, w, b, w, p,); } 
		0x04 => { expand_opcode!("wait_rerun"); }, 
		0x05 => { expand_opcode!("movie_overlay_flag", b, p,); } 
		0x06 => { expand_opcode!("absolute_jump", d, p,); } 
		0x07 => { expand_opcode!("resource_string", s,); } 
		0x08 => { expand_opcode!("nop", p,); } 
		0x09 => { expand_opcode!("call_script", s,); } 
		0x0A => { expand_opcode!("return", p,); } 
		0x0B => { expand_opcode!("start_timer", b, p,); } 
		0x0C => { expand_opcode!("read_timer", w, p,); } 
		0x0D => { expand_opcode!("fill_variable_range", w, w, w, p,); } 
		0x0E => { expand_opcode!("stop_movie_flag", b, p,); } 
		0x21 => { expand_opcode!("play_ogg_voice_pair", b, w, b, w, d, s,); } 
		0x22 => { expand_opcode!("voice_pair_stop_volume", b, w, p,); } 
 0x23 | 0x27 => { expand_opcode!("sprite_linked_voice", b, w, w, w, b, b, s,); } 
		0x24 => { expand_opcode!("audio_mixer_reset", p,); } 
		0x25 => { expand_opcode!("sfx_play", b, b, w, p, p, b, w, b, b, s,); } 
		0x26 => { expand_opcode!("sfx_stop", b, p,); } 
		0x28 => { expand_opcode!("sfx_seek", b, b, w, p,); } 
		0x29 => { expand_opcode!("sfx_stop_fade", b, w, p,); } 
		0x30 => { expand_opcode!("voice_pair_pan", b, w, p,); } 
		0x31 => { expand_opcode!("sfx_slot_rearm_persist", b, p,); } 
		0x32 => { expand_opcode!("sfx_slot_rearm", b, p,); } 
		0x33 => { expand_opcode!("read_voice_position", w, w, w, p,); } 
		0x41 => { expand_opcode!("textbox_no_speaker", w, b, b, s,); } 
		0x42 => { expand_opcode!("textbox_with_speaker", w, b, b, b, s, s,); } 
		0x43 => { expand_opcode!("load_anm_animation", b, w, w, b, s,); } 
		0x44 => { expand_opcode!("enable_sprite_frame", b, b, b, p,); } 
		0x45 => { expand_opcode!("flip_sprite_frame", b, b, b, p,); } 
		0x46 => { expand_opcode!("load_background", w, w, d, b, s,); } 
		0x47 => { expand_opcode!("background_show_hide", b, p,); } 
		0x48 => { expand_opcode!("load_static_sprite", b, w, w, d, b, b, s,); } 
		0x49 => { expand_opcode!("static_sprite_active_flag", w, p,); } 
		0x4A => { expand_opcode!("scene_wipe_transition", b, w, w, p,); } 
		0x4B => { expand_opcode!("transition_entry", b, w, w, d, w, d, d, p,); } 
		0x4C => { expand_opcode!("scene_transition_1", b, b, b, d, p,); } 
		0x4D => { expand_opcode!("transition_effect_params", b, b, w, w, w, w, w, p,); } 
		0x4E => { expand_opcode!("wipe_particle_effect", b, b, b, p,); } 
		0x4F => { expand_opcode!("clear_sprite_frame_markers", b, b, b, p,); } 
		0x50 => { expand_opcode!("load_tbl", s,); } 
		0x51 => { expand_opcode!("read_words_to_variables", w, w, p,); } 
		0x52 => { expand_opcode!("unload_tbl", b, p,); }
		0x53 => { expand_opcode!("load_wip_and_mask", b, w, w, s,); }
		0x54 => { expand_opcode!("load_msk", s,); } 
		0x55 => { expand_opcode!("free_msk", p,); } 
		0x56 => { expand_opcode!("message_subsystem_state", p,); } 
		0x57 => { expand_opcode!("movement_block_setup", w, w, d, p,); } 
		0x58 => { expand_opcode!("per_slot_value_pair", b, b, b, w, w, p,); } 
		0x59 => { expand_opcode!("preload_wip", s,); } 
		0x60 => { expand_opcode!("release_wipe_resources", p,); } 
		0x61 => { expand_opcode!("load_start_movie", b, s,); } 
		0x62 => { expand_opcode!("cancel_transition", p,); } 
		0x63 => { expand_opcode!("static_sprite_flag", b, b, p,); } 
		0x64 => { expand_opcode!("sprite_transform", b, w, w, w, p,); } 
		0x65 => { expand_opcode!("transform_origin_reapply", w, w, p,); } 
		0x66 => { expand_opcode!("inlay_entry", b, w, w, b, w, d, w, d, d,); } 
		0x67 => { expand_opcode!("scene_transition_2", b, b, b, d, p,); } 
		0x68 => { expand_opcode!("background_zoom", w, w, w, w, p,); } 
		0x69 => { expand_opcode!("movie_state_byte", b, p,); } 
		0x70 => { expand_opcode!("scene_transition_3", b, b, p, d, p,); } 
		0x71 => { expand_opcode!("filename_resource_op", s,); } 
		0x72 => { expand_opcode!("filename_resource_op2", p,); } 
		0x73 => { expand_opcode!("load_inlay", w, w, d, b, s,); } 
		0x74 => { expand_opcode!("inlay_stop", b, p,); } 
		0x75 => { expand_opcode!("inlay_move_resize", w, w, w, w, p,); } 
		0x76 => { expand_opcode!("inlay_fade_setup", w, w, d, b, b, w, d, p,); } 
		0x77 => { expand_opcode!("inlay_move_animation", w, w, d, p,); } 
		0x78 => { expand_opcode!("textbox_fade_start", b, b, b, d, p,); } 
		0x79 => { expand_opcode!("textbox_fade_cancel", p,); } 
		0x81 => { expand_opcode!("nop", p, p,); } 
		0x82 => { expand_opcode!("start_timer", w, p,); } 
		0x83 => { expand_opcode!("resume", p,); } 
		0x84 => { expand_opcode!("pause", p,); } 
		0x85 => { expand_opcode!("flag_setter", b, p,); } 
		0x86 => { expand_opcode!("state_snapshot", p, p,); } 
		0x87 => { expand_opcode!("movie_flag_to_variable", w, p,); } 
		0x88 => { expand_opcode!("transition_flag_snapshot", p, p, p,); } 
		0x89 => { expand_opcode!("full_state_reset", p,); } 
		0x8A => { expand_opcode!("single_call", p,); } 
		0x8B => { expand_opcode!("single_call", p,); } 
		0x8C => { expand_opcode!("textbox_state_preset", w, p,); } 
		0x8D => { expand_opcode!("box_state_op", p,); } 
		0x8E => { expand_opcode!("flag_setter", p,); } 
		0xA0 => { expand_opcode!("background_position", w, w, b, p,); } 
		0xA1 => { expand_opcode!("character_slot_position", b, w, w, b, p,); } 
		0xA2 => { expand_opcode!("positional_sfx_position", b, w, w, p,); } 
		0xA3 => { expand_opcode!("positional_sfx_play", p, w, w, p,); } 
		0xA4 => { expand_opcode!("positional_sfx_play_mode2", p, w, w, p,); } 
		0xA5 => { expand_opcode!("positional_sfx_stop", b, p,); } 
		0xA6 => { expand_opcode!("stop_movie", p,); } 
		0xA7 => { expand_opcode!("crosshair_cursor", p,); } 
		0xA8 => { expand_opcode!("movie_parameters", b, b, b, p, p, p, p, w, w, w, w, p,); } 
		0xA9 => { expand_opcode!("stop_video", p,); } 
		0xAA => { expand_opcode!("numbered_cursor", b, b, p,); } 
		0xAB => { expand_opcode!("pointer_position_snapshot", p,); } 
		0xAC => { expand_opcode!("cursor_show_hide", p,); } 
		0xAD => { expand_opcode!("pointer_animation_state", b, d, d, p,); } 
		0xAE => { expand_opcode!("single_call", p,); } 
		0xB1 => { expand_opcode!("background_center", w, w, p,); } 
		0xB2 => { expand_opcode!("load_effect_file", b, p, s,); } 
		0xB3 => { expand_opcode!("stop_effect", p, p,); } 
		0xB4 => { expand_opcode!("effect_parameters", p, p, w, w, d, b, p,); } 
		0xB5 => { expand_opcode!("effect_frame_step", b, b, p, p, p, p, p,); } 
		0xB6 => { expand_opcode!("append_textbox_text", w, s,); } 
		0xB7 => { expand_opcode!("load_slot_image", b, w, w, s,); } 
		0xB8 => { expand_opcode!("slot_image_show_hide", b, b, p,); } 
		0xB9 => { expand_opcode!("per_slot_default", b, b, p,); } 
		0xBA => { expand_opcode!("colour_effect_parameters", w, w, b, b, b, b, b, w, s,); } 
		0xBB => { expand_opcode!("colour_effect_reset", p,); } 
		0xBC => { expand_opcode!("advance_animation_frame", b, b, b, p,); } 
		0xBD => { expand_opcode!("flag_setter", b, p,); } 
		0xBE => { expand_opcode!("swap_character_slots", b, b, p,); } 
		0xBF => { expand_opcode!("textbox_fade_update", b, b, b, d, p,); } 
		0xE0 => { expand_opcode!("scene_text", s,); } 
		0xE2 => { expand_opcode!("implicit_resource_op", p,); } 
		0xE3 => { expand_opcode!("implicit_resource_op_gated", p,); } 
		0xE4 => { expand_opcode!("textbox_mode", b, p,); } 
		0xE5 => { expand_opcode!("end_textbox_sequence", p,); } 
		0xE6 => { expand_opcode!("nop", p, p,); } 
		0xE7 => { expand_opcode!("mark_table_entry", w, p,); } 
		0xE8 => { expand_opcode!("filename_op", s,); } 
		0xE9 => { expand_opcode!("filename_op_no_string", p,); } 
		0xEA => { expand_opcode!("ogg_file_op", b, s,); } 
		0xEB => { expand_opcode!("ogg_file_op2", p,); } 
		0xFF => { expand_opcode!("end_of_script"); } 
		_ => {
			log::error!("Unknown opcode 0x{:02X}", &input[0]);
			return None;
		}
	};

	log::debug!("final pointer value: {ptr}");
	Some(Opcode {
		name: name.to_owned(),
		opcode: input[0],
		address: addr,
		actual_address: addr,
		fields
	})
}