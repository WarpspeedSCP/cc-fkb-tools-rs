use anyhow::anyhow;
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

/// One component of an opcode's binary layout, in decode order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Code {
	Byte,
	Word,
	DWord,
	Str,
	Choice,
	Padding(u8),
}

/// Static opcode specification: one row per implemented opcode, the single source of truth for its
/// mnemonic, layout, and operand names. `layout[i]` and `operands[i]` describe the same emitted
/// field `i`, so `layout.len() == operands.len()` holds for every row.
#[derive(Clone, Copy, Debug)]
pub struct OpcodeSpecStatic {
	pub opcode: u8,
	pub name: &'static str,
	pub layout: &'static [Code],
	pub operands: &'static [&'static str],
}

/// Renders a layout with the same spelling the docs use: `b w b w p 1`.
pub fn render_layout(layout: &[Code]) -> String {
	layout
		.iter()
		.map(|code| match code {
			Code::Byte => "b".to_owned(),
			Code::Word => "w".to_owned(),
			Code::DWord => "d".to_owned(),
			Code::Str => "s".to_owned(),
			Code::Choice => "c".to_owned(),
			Code::Padding(n) => format!("p {n}"),
		})
		.collect::<Vec<_>>()
		.join(" ")
}

/// Per-file opcode manifest entry. `decode_wsc` embeds one per opcode a file uses as the top-level
/// `opcode_table:` block, and `encode` validates every entry against [`OPCODE_SPECS`].
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct OpcodeSpec {
	#[serde(serialize_with = "serialize_hex_u8")]
	pub opcode: u8,
	pub name: String,
	pub layout: String,
	#[serde(serialize_with = "serialize_flow_strings")]
	pub operands: Vec<String>,
}

impl OpcodeSpecStatic {
	pub fn to_manifest(&self) -> OpcodeSpec {
		OpcodeSpec {
			opcode: self.opcode,
			name: self.name.to_owned(),
			layout: render_layout(self.layout),
			operands: self.operands.iter().map(|it| (*it).to_owned()).collect(),
		}
	}
}

/// One decoded instruction. The mnemonic is deliberately absent: it is defined once in
/// [`OPCODE_SPECS`] and, per file, in the `opcode_table:` manifest, so `opcode` is the only key an
/// instruction needs.
#[derive(Serialize, Deserialize, Clone)]
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
	/// Global opcode manifest: mnemonic, layout and operand names for every opcode this file uses.
	/// Emitted once per file (sorted by opcode byte) so instruction records stay positional.
	pub opcode_table: Vec<OpcodeSpec>,
	pub opcodes: Vec<Opcode>,
	#[serde(serialize_with = "crate::opcodes::serialize_inline_ints_vec")]
	pub trailer: Vec<u8>,
}

impl Script {
	/// Serialises the script, resolving every jump target to a byte offset.
	///
	/// Fails when a jump operand is not a dword or points at an address that holds no instruction:
	/// either means the file was edited into a state the game could not run.
	pub fn binary_serialise(mut self) -> anyhow::Result<Vec<u8>> {
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
					let target = opcode.fields[0].as_dword().ok_or_else(|| anyhow!(
						"direct jump at 0x{:08X} does not hold a dword target",
						opcode.address
					))? as usize;
					let (idx, orig_op) = orig_opcodes
						.iter()
						.find_position(|it| it.address == target)
						.ok_or_else(|| anyhow!(
							"direct jump at 0x{:08X} targets 0x{target:08X}, which is not an instruction in this file",
							opcode.address
						))?;
					log::debug!(
            "Direct jump opcode at 0x{:08X} (actual 0x{:08X}) jumps to: 0x{:04X}",
            opcode.address,
            actual_address,
            orig_op.address,
          );
					jump_map.insert(opcode.address as u32, idx);
				}
				0x01 => {
					let target = opcode.address + 11 + opcode.fields[3].as_dword().ok_or_else(|| anyhow!(
						"conditional jump at 0x{:08X} does not hold a dword offset",
						opcode.address
					))? as usize;
					let (idx, orig_op) = orig_opcodes
						.iter()
						.find_position(|it| it.address == target)
						.ok_or_else(|| anyhow!(
							"conditional jump at 0x{:08X} targets 0x{target:08X}, which is not an instruction in this file",
							opcode.address
						))?;

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

		Ok(buf)
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
			let target_address = opcodes[tbl_entry].actual_address as u32;
			opcode.fields[0] = OpField::DWord(target_address);
			log::debug!(
        "Adjusting direct jump Opcode at 0x{:08X} (actual {:08X}) to jump to: {:08X}",
        opcode.address,
        opcode.actual_address,
        target_address,
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

/// Emits a `Vec<String>` as one inline flow sequence; `data::fix_yaml_str` unquotes it, which is how
/// `!Padding [ … ]` is emitted too.
pub fn serialize_flow_strings<S>(data: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("[ {} ]", data.join(", ")))
}

/// Looks up the compiled-in spec for an opcode byte.
pub fn lookup_spec(opcode: u8) -> Option<&'static OpcodeSpecStatic> {
	OPCODE_SPECS.iter().find(|it| it.opcode == opcode)
}

/// Validates a parsed script against the compiled-in spec: every `opcode_table:` entry must match
/// its spec row exactly, and every instruction's opcode byte must be a known, listed opcode.
pub fn validate_opcode_table(script: &Script) -> Result<(), String> {
	for entry in &script.opcode_table {
		let spec = lookup_spec(entry.opcode)
			.ok_or_else(|| format!("opcode_table lists unknown opcode 0x{:02X}", entry.opcode))?;
		let expected = spec.to_manifest();
		if *entry != expected {
			return Err(format!(
				"opcode_table entry 0x{:02X} disagrees with the compiled opcode table: {entry:?} (expected {expected:?})",
				entry.opcode
			));
		}
	}

	for op in &script.opcodes {
		if lookup_spec(op.opcode).is_none() {
			return Err(format!(
				"instruction at 0x{:08X} uses unknown opcode 0x{:02X}",
				op.address, op.opcode
			));
		}
		if !script
			.opcode_table
			.iter()
			.any(|it| it.opcode == op.opcode)
		{
			return Err(format!(
				"instruction at 0x{:08X} uses opcode 0x{:02X}, which is missing from opcode_table",
				op.address, op.opcode
			));
		}
	}

	Ok(())
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

/// Maps a layout code token to a [`Code`] in const context.
macro_rules! layout_code {
	(b) => { Code::Byte };
	(w) => { Code::Word };
	(d) => { Code::DWord };
	(s) => { Code::Str };
	(c) => { Code::Choice };
	(p $num:literal) => { Code::Padding($num) };
}

/// Decodes one opcode row's components in order. The decode state is passed in by identifier so the
/// generated statements operate on the caller's locals (`ptr`, `fields`, `input`).
macro_rules! decode_components {
	($ptr:ident, $fields:ident, $input:ident;) => {};
	($ptr:ident, $fields:ident, $input:ident; $name:ident : b, $($rest:tt)*) => {
		$fields.push(OpField::Byte($input[$ptr]));
		$ptr += 1;
		decode_components!($ptr, $fields, $input; $($rest)*);
	};
	($ptr:ident, $fields:ident, $input:ident; $name:ident : w, $($rest:tt)*) => {
		$fields.push(OpField::Word(crate::util::transmute_to_u16($ptr, $input)));
		$ptr += 2;
		decode_components!($ptr, $fields, $input; $($rest)*);
	};
	($ptr:ident, $fields:ident, $input:ident; $name:ident : d, $($rest:tt)*) => {
		$fields.push(OpField::DWord(crate::util::transmute_to_u32($ptr, $input)));
		$ptr += 4;
		decode_components!($ptr, $fields, $input; $($rest)*);
	};
	($ptr:ident, $fields:ident, $input:ident; $name:ident : s, $($rest:tt)*) => {
		{
			let (bytes, string) = crate::util::get_sjis_bytes($ptr, $input);
			$fields.push(OpField::String(TLString {
				raw: string,
				translation: None,
				notes: None,
			}));
			$ptr += bytes.len();
		}
		decode_components!($ptr, $fields, $input; $($rest)*);
	};
	($ptr:ident, $fields:ident, $input:ident; $name:ident : c, $($rest:tt)*) => {
		{
			let n_choices = match &$fields[0] {
				OpField::Byte(n) => *n,
				_ => panic!("Choice count must be the first byte of a choice opcode"),
			};
			let mut choices = vec![];
			let mut curr_ptr = $ptr;
			for _ in 0..n_choices {
				let choice = make_choice(&$input[curr_ptr..]);
				curr_ptr += choice.size();
				choices.push(choice);
			}
			$ptr += choices.iter().map(|it| it.size()).sum::<usize>();
			$fields.push(OpField::Choice(choices));
		}
		decode_components!($ptr, $fields, $input; $($rest)*);
	};
	($ptr:ident, $fields:ident, $input:ident; $name:ident : p $num:literal, $($rest:tt)*) => {
		$fields.push(OpField::Padding($input[$ptr..($ptr + $num)].to_vec()));
		$ptr += $num;
		decode_components!($ptr, $fields, $input; $($rest)*);
	};
}

/// The single source of truth for opcode mnemonics, layouts and operand names: one row per
/// implemented opcode, in ascending opcode order. Expands to [`OPCODE_SPECS`] and to the decoder
/// used by [`make_opcode`], so the table, the decoder and the YAML manifest cannot drift apart.
///
/// Row grammar: `0xNN => mnemonic [ operand_name: code, … ]`, one entry per emitted field
/// (padding included, spelled `pad: p <bytes>`). Derivation of names:
/// `validation/opcodes/reference.md` + `layouts.md` + `heap-access.md`; agreement pinned by the
/// test module below.
macro_rules! opcode_table {
	( $( $op:literal => $mnemonic:ident [ $( $name:ident : $code:ident $( $num:literal )? ),* $(,)? ] ),* $(,)? ) => {
		/// Every implemented opcode, ascending by opcode byte.
		pub static OPCODE_SPECS: &[OpcodeSpecStatic] = &[
			$(
				OpcodeSpecStatic {
					opcode: $op,
					name: stringify!($mnemonic),
					layout: &[ $( layout_code!($code $( $num )?) ),* ],
					operands: &[ $( stringify!($name) ),* ],
				}
			),*
		];

		/// Decodes an opcode's payload into fields, in table order. The mnemonic is not carried on the
		/// decoded instruction — it is defined once in this table and, per file, in the
		/// `opcode_table:` manifest — so only the opcode byte keys an instruction.
		pub(crate) fn decode_opcode_fields(opcode: u8, input: &[u8]) -> Option<Vec<OpField>> {
			let mut ptr = 1usize;
			let mut fields: Vec<OpField> = vec![];
			match opcode {
				$(
					$op => {
						decode_components!(ptr, fields, input; $( $name : $code $( $num )?, )*);
					}
				),*
				_ => {
					log::error!("Unknown opcode 0x{:02X}", opcode);
					return None;
				}
			};

			log::debug!("final pointer value: {ptr}");
			Some(fields)
		}
	};
}

opcode_table! {
	0x01 => conditional_branch [ branch_type: b, arg1: w, arg2: w, offset: d, pad: p 1,],
	0x02 => choice_jump [ count: b, separator: p 1, choices: c,],  // ❌ choice-list trailer byte the engine reads is padding in the Rust arm
	0x03 => variable_heap_op [ kind: b, var_index: w, indirect: b, value: w, pad: p 1,],
	0x04 => wait_rerun [ ],
	0x05 => movie_overlay_flag [ arg1: b, pad: p 1,],
	0x06 => absolute_jump [ target: d, pad: p 1,],
	0x07 => resource_string [ filename: s,],
	0x08 => nop [ pad: p 1,],
	0x09 => call_script [ filename: s,],
	0x0A => return [ pad: p 1,],
	0x0B => start_timer [ seconds: b, pad: p 1,],
	0x0C => read_timer [ var_index: w, pad: p 1,],
	0x0D => fill_variable_range [ var_index: w, count: w, value: w, pad: p 1,],
	0x0E => stop_movie_flag [ arg1: b, pad: p 1,],
	0x21 => play_ogg_voice_pair [ arg1: b, arg2: w, pan: b, arg4: w, arg5: d, filename: s,],
	0x22 => voice_pair_stop_volume [ arg1: b, volume: w, pad: p 1,],
	0x23 => sprite_linked_voice [ flags: b, slot: w, frame: w, arg4: w, mode: b, arg6: b, filename: s,],
	0x24 => audio_mixer_reset [ pad: p 1,],
	0x25 => sfx_play [ slot: b, repeat: b, persist: w, pad: p 2, position: b, arg6: w, volume: b, flags: b, filename: s,],  // ❌ engine: w@+3..4 merged, live u16 @+5..6 hidden in p 2
	0x26 => sfx_stop [ slot: b, pad: p 1,],
	0x27 => sprite_linked_voice [ flags: b, slot: w, frame: w, arg4: w, mode: b, arg6: b, filename: s,],
	0x28 => sfx_seek [ slot: b, position: b, arg3: w, pad: p 1,],
	0x29 => sfx_stop_fade [ slot: b, arg2: w, pad: p 1,],
	0x30 => voice_pair_pan [ pan: b, arg2: w, pad: p 1,],
	0x31 => sfx_slot_rearm_persist [ slot: b, pad: p 1,],
	0x32 => sfx_slot_rearm [ slot: b, pad: p 1,],
	0x33 => read_voice_position [ minutes: w, seconds: w, milliseconds: w, pad: p 1,],
	0x41 => textbox_no_speaker [ layout_id: w, mode: b, timer_param: b, text: s,],
	0x42 => textbox_with_speaker [ layout_id: w, mode: b, speaker_arg: b, timer_param: b, speaker_text: s, text: s,],
	0x43 => load_anm_animation [ slot: b, x: w, y: w, flags: b, filename: s,],
	0x44 => enable_sprite_frame [ slot: b, frame: b, active: b, pad: p 1,],
	0x45 => flip_sprite_frame [ slot: b, frame: b, arg3: b, pad: p 1,],
	0x46 => load_background [ x: w, y: w, format_param: d, flags: b, filename: s,],
	0x47 => background_show_hide [ mode: b, pad: p 1,],
	0x48 => load_static_sprite [ slot: b, x: w, y: w, id: d, flags: b, use_default: b, filename: s,],
	0x49 => static_sprite_active_flag [ slot: w, pad: p 1,],  // ⚠ w@+1..2 spans two engine operands (lossless)
	0x4A => scene_wipe_transition [ mode: b, param_a: w, param_b: w, pad: p 1,],
	0x4B => transition_entry [ layer_id: b, arg2: w, arg3: w, handle: d, flag_5c: w, field_84: d, field_88: d, pad: p 1,],
	0x4C => scene_transition_1 [ flag_a: b, param_a: b, arg3: b, param_c: d, pad: p 1,],
	0x4D => transition_effect_params [ mode: b, gate_arg: b, count: w, param_a: w, param_b: w, param_c: w, param_d: w, pad: p 1,],
	0x4E => wipe_particle_effect [ kind: b, direction: b, regen: b, pad: p 1,],
	0x4F => clear_sprite_frame_markers [ slot: b, frame: b, arg3: b, pad: p 1,],
	0x50 => load_tbl [ filename: s,],
	0x51 => read_words_to_variables [ mouse_x: w, mouse_y: w, pad: p 1,],
	0x52 => unload_tbl [ unused: b, pad: p 1,],  // ⚠ operand is never read by the engine
	0x53 => load_wip_and_mask [ flags: b, arg1: w, arg2: w, filename: s,],
	0x54 => load_msk [ filename: s,],
	0x55 => free_msk [ pad: p 1,],
	0x56 => message_subsystem_state [ pad: p 1,],
	0x57 => movement_block_setup [ arg1: w, arg2: w, arg3: d, pad: p 1,],
	0x58 => per_slot_value_pair [ slot: b, arg2: b, arg3: b, arg4: w, arg5: w, pad: p 1,],
	0x59 => preload_wip [ filename: s,],
	0x60 => release_wipe_resources [ pad: p 1,],
	0x61 => load_start_movie [ mode: b, filename: s,],
	0x62 => cancel_transition [ pad: p 1,],
	0x63 => static_sprite_flag [ slot: b, arg2: b, pad: p 1,],
	0x64 => sprite_transform [ slot: b, scale_x: w, scale_y: w, rotation: w, pad: p 1,],
	0x65 => transform_origin_reapply [ arg1: w, arg2: w, pad: p 1,],
	0x66 => inlay_entry [ slot: b, scale_x: w, scale_y: w, flags: b, rotation: w, active: d, arg7: w, arg8: d, arg9: d,],
	0x67 => scene_transition_2 [ arg1: b, arg2: b, arg3: b, arg4: d, pad: p 1,],
	0x68 => background_zoom [ scale_x: w, scale_y: w, center_x: w, center_y: w, pad: p 1,],
	0x69 => movie_state_byte [ state: b, pad: p 1,],
	0x70 => scene_transition_3 [ arg1: b, arg2: b, pad: p 1, arg4: d, pad: p 1,],  // ❌ engine reads byte @+3, which is padding here
	0x71 => filename_resource_op [ filename: s,],
	0x72 => filename_resource_op2 [ pad: p 1,],  // ⚠ operand is only preserved by the engine
	0x73 => load_inlay [ position_x: w, position_y: w, id: d, flags: b, filename: s,],
	0x74 => inlay_stop [ arg1: b, pad: p 1,],
	0x75 => inlay_move_resize [ x: w, y: w, width: w, height: w, pad: p 1,],
	0x76 => inlay_fade_setup [ arg1: w, arg2: w, duration: d, arg4: b, arg5: b, arg6: w, arg7: d, pad: p 1,],
	0x77 => inlay_move_animation [ x: w, y: w, duration: d, pad: p 1,],
	0x78 => textbox_fade_start [ percent: b, arg2: b, arg3: b, arg4: d, pad: p 1,],
	0x79 => textbox_fade_cancel [ pad: p 1,],
	0x81 => nop [ pad: p 2,],
	0x82 => start_timer [ duration: w, pad: p 1,],
	0x83 => resume [ pad: p 1,],
	0x84 => pause [ pad: p 1,],
	0x85 => flag_setter [ flags: b, pad: p 1,],
	0x86 => state_snapshot [ pad: p 2,],
	0x87 => movie_flag_to_variable [ var_index: w, pad: p 1,],
	0x88 => transition_flag_snapshot [ pad: p 3,],
	0x89 => full_state_reset [ pad: p 1,],
	0x8A => single_call [ pad: p 1,],
	0x8B => single_call [ pad: p 1,],
	0x8C => textbox_state_preset [ preset: w, pad: p 1,],
	0x8D => box_state_op [ pad: p 1,],
	0x8E => flag_setter [ pad: p 1,],
	0xA0 => background_position [ x: w, y: w, flags: b, pad: p 1,],
	0xA1 => character_slot_position [ slot: b, x: w, y: w, flags: b, pad: p 1,],
	0xA2 => positional_sfx_position [ slot: b, x: w, y: w, pad: p 1,],
	0xA3 => positional_sfx_play [ pad: p 1, x: w, y: w, pad: p 1,],  // ❌ engine reads byte @+1, which is padding here
	0xA4 => positional_sfx_play_mode2 [ pad: p 1, x: w, y: w, pad: p 1,],  // ❌ engine reads byte @+1, which is padding here
	0xA5 => positional_sfx_stop [ slot: b, pad: p 1,],
	0xA6 => stop_movie [ pad: p 1,],
	0xA7 => crosshair_cursor [ pad: p 1,],
	0xA8 => movie_parameters [ arg1: b, arg2: b, arg3: b, pad: p 4, arg4: w, arg5: w, arg6: w, arg7: w, pad: p 1,],  // ❌ engine: bytes @+4..5 and word @+6..7 hidden in p 4
	0xA9 => stop_video [ pad: p 1,],
	0xAA => numbered_cursor [ index: b, number: b, pad: p 1,],
	0xAB => pointer_position_snapshot [ pad: p 1,],
	0xAC => cursor_show_hide [ pad: p 1,],
	0xAD => pointer_animation_state [ arg1: b, arg2: d, arg3: d, pad: p 1,],
	0xAE => single_call [ pad: p 1,],
	0xB1 => background_center [ x: w, y: w, pad: p 1,],
	0xB2 => load_effect_file [ arg1: b, pad: p 1, filename: s,],
	0xB3 => stop_effect [ pad: p 2,],  // ❌ engine reads byte @+1, which is padding here
	0xB4 => effect_parameters [ pad: p 2, arg3: w, arg4: w, arg5: d, arg6: b, pad: p 1,],  // ❌ engine: bytes @+1..2 hidden in p 2
	0xB5 => effect_frame_step [ arg1: b, arg2: b, pad: p 5,],  // ❌ engine: dword @+3..6 hidden in p 5
	0xB6 => append_textbox_text [ mode: w, text: s,],
	0xB7 => load_slot_image [ slot: b, x: w, y: w, filename: s,],
	0xB8 => slot_image_show_hide [ slot: b, visible: b, pad: p 1,],
	0xB9 => per_slot_default [ slot: b, value: b, pad: p 1,],
	0xBA => colour_effect_parameters [ arg1: w, arg2: w, arg3: b, arg4: b, arg5: b, arg6: b, arg7: b, arg8: w, text: s,],
	0xBB => colour_effect_reset [ pad: p 1,],
	0xBC => advance_animation_frame [ slot: b, frame: b, state: b, pad: p 1,],
	0xBD => flag_setter [ flags: b, pad: p 1,],
	0xBE => swap_character_slots [ slot_a: b, slot_b: b, pad: p 1,],
	0xBF => textbox_fade_update [ arg1: b, arg2: b, arg3: b, arg4: d, pad: p 1,],
	0xE0 => scene_text [ text: s,],
	0xE2 => implicit_resource_op [ pad: p 1,],
	0xE3 => implicit_resource_op_gated [ pad: p 1,],
	0xE4 => textbox_mode [ mode: b, pad: p 1,],
	0xE5 => end_textbox_sequence [ pad: p 1,],
	0xE6 => nop [ pad: p 2,],
	0xE7 => mark_table_entry [ table_id: w, pad: p 1,],
	0xE8 => filename_op [ filename: s,],
	0xE9 => filename_op_no_string [ pad: p 1,],
	0xEA => ogg_file_op [ arg1: b, filename: s,],
	0xEB => ogg_file_op2 [ pad: p 1,],
	0xFF => end_of_script [ ],
}

pub fn make_opcode(input: &[u8], addr: usize) -> Option<Opcode> {
	let fields = decode_opcode_fields(input[0], input)?;
	Some(Opcode {
		opcode: input[0],
		address: addr,
		actual_address: addr,
		fields,
	})
}

#[cfg(test)]
mod spec_test {
	use super::*;
	use crate::data::{decode_wsc, fix_yaml_str};

	/// Operand names as reviewed against `validation/opcodes/reference.md` (with `layouts.md` for
	/// offsets/widths and `heap-access.md` for variable-index operands) on 2026-09-20, one row per
	/// implemented opcode in ascending order.
	///
	/// This constant is the reviewed record the compiled table is pinned to: renaming an operand,
	/// reordering a row, or getting the arity wrong in `opcode_table!` fails
	/// [`every_opcode_declares_its_operand_names`] until this table changes too. Whether a name is
	/// *semantically* right is a human judgement made once against the docs; what the test enforces
	/// is that the table and this record cannot drift apart.
	const EXPECTED: &[(u8, &[&str])] = &[
		(0x01, &["branch_type", "arg1", "arg2", "offset", "pad"]),
		(0x02, &["count", "separator", "choices"]),
		(0x03, &["kind", "var_index", "indirect", "value", "pad"]),
		(0x04, &[]),
		(0x05, &["arg1", "pad"]),
		(0x06, &["target", "pad"]),
		(0x07, &["filename"]),
		(0x08, &["pad"]),
		(0x09, &["filename"]),
		(0x0A, &["pad"]),
		(0x0B, &["seconds", "pad"]),
		(0x0C, &["var_index", "pad"]),
		(0x0D, &["var_index", "count", "value", "pad"]),
		(0x0E, &["arg1", "pad"]),
		(0x21, &["arg1", "arg2", "pan", "arg4", "arg5", "filename"]),
		(0x22, &["arg1", "volume", "pad"]),
		(0x23, &["flags", "slot", "frame", "arg4", "mode", "arg6", "filename"]),
		(0x24, &["pad"]),
		(0x25, &["slot", "repeat", "persist", "pad", "position", "arg6", "volume", "flags", "filename"]),
		(0x26, &["slot", "pad"]),
		(0x27, &["flags", "slot", "frame", "arg4", "mode", "arg6", "filename"]),
		(0x28, &["slot", "position", "arg3", "pad"]),
		(0x29, &["slot", "arg2", "pad"]),
		(0x30, &["pan", "arg2", "pad"]),
		(0x31, &["slot", "pad"]),
		(0x32, &["slot", "pad"]),
		(0x33, &["minutes", "seconds", "milliseconds", "pad"]),
		(0x41, &["layout_id", "mode", "timer_param", "text"]),
		(0x42, &["layout_id", "mode", "speaker_arg", "timer_param", "speaker_text", "text"]),
		(0x43, &["slot", "x", "y", "flags", "filename"]),
		(0x44, &["slot", "frame", "active", "pad"]),
		(0x45, &["slot", "frame", "arg3", "pad"]),
		(0x46, &["x", "y", "format_param", "flags", "filename"]),
		(0x47, &["mode", "pad"]),
		(0x48, &["slot", "x", "y", "id", "flags", "use_default", "filename"]),
		(0x49, &["slot", "pad"]),
		(0x4A, &["mode", "param_a", "param_b", "pad"]),
		(0x4B, &["layer_id", "arg2", "arg3", "handle", "flag_5c", "field_84", "field_88", "pad"]),
		(0x4C, &["flag_a", "param_a", "arg3", "param_c", "pad"]),
		(0x4D, &["mode", "gate_arg", "count", "param_a", "param_b", "param_c", "param_d", "pad"]),
		(0x4E, &["kind", "direction", "regen", "pad"]),
		(0x4F, &["slot", "frame", "arg3", "pad"]),
		(0x50, &["filename"]),
		(0x51, &["mouse_x", "mouse_y", "pad"]),
		(0x52, &["unused", "pad"]),
		(0x53, &["flags", "arg1", "arg2", "filename"]),
		(0x54, &["filename"]),
		(0x55, &["pad"]),
		(0x56, &["pad"]),
		(0x57, &["arg1", "arg2", "arg3", "pad"]),
		(0x58, &["slot", "arg2", "arg3", "arg4", "arg5", "pad"]),
		(0x59, &["filename"]),
		(0x60, &["pad"]),
		(0x61, &["mode", "filename"]),
		(0x62, &["pad"]),
		(0x63, &["slot", "arg2", "pad"]),
		(0x64, &["slot", "scale_x", "scale_y", "rotation", "pad"]),
		(0x65, &["arg1", "arg2", "pad"]),
		(0x66, &["slot", "scale_x", "scale_y", "flags", "rotation", "active", "arg7", "arg8", "arg9"]),
		(0x67, &["arg1", "arg2", "arg3", "arg4", "pad"]),
		(0x68, &["scale_x", "scale_y", "center_x", "center_y", "pad"]),
		(0x69, &["state", "pad"]),
		(0x70, &["arg1", "arg2", "pad", "arg4", "pad"]),
		(0x71, &["filename"]),
		(0x72, &["pad"]),
		(0x73, &["position_x", "position_y", "id", "flags", "filename"]),
		(0x74, &["arg1", "pad"]),
		(0x75, &["x", "y", "width", "height", "pad"]),
		(0x76, &["arg1", "arg2", "duration", "arg4", "arg5", "arg6", "arg7", "pad"]),
		(0x77, &["x", "y", "duration", "pad"]),
		(0x78, &["percent", "arg2", "arg3", "arg4", "pad"]),
		(0x79, &["pad"]),
		(0x81, &["pad"]),
		(0x82, &["duration", "pad"]),
		(0x83, &["pad"]),
		(0x84, &["pad"]),
		(0x85, &["flags", "pad"]),
		(0x86, &["pad"]),
		(0x87, &["var_index", "pad"]),
		(0x88, &["pad"]),
		(0x89, &["pad"]),
		(0x8A, &["pad"]),
		(0x8B, &["pad"]),
		(0x8C, &["preset", "pad"]),
		(0x8D, &["pad"]),
		(0x8E, &["pad"]),
		(0xA0, &["x", "y", "flags", "pad"]),
		(0xA1, &["slot", "x", "y", "flags", "pad"]),
		(0xA2, &["slot", "x", "y", "pad"]),
		(0xA3, &["pad", "x", "y", "pad"]),
		(0xA4, &["pad", "x", "y", "pad"]),
		(0xA5, &["slot", "pad"]),
		(0xA6, &["pad"]),
		(0xA7, &["pad"]),
		(0xA8, &["arg1", "arg2", "arg3", "pad", "arg4", "arg5", "arg6", "arg7", "pad"]),
		(0xA9, &["pad"]),
		(0xAA, &["index", "number", "pad"]),
		(0xAB, &["pad"]),
		(0xAC, &["pad"]),
		(0xAD, &["arg1", "arg2", "arg3", "pad"]),
		(0xAE, &["pad"]),
		(0xB1, &["x", "y", "pad"]),
		(0xB2, &["arg1", "pad", "filename"]),
		(0xB3, &["pad"]),
		(0xB4, &["pad", "arg3", "arg4", "arg5", "arg6", "pad"]),
		(0xB5, &["arg1", "arg2", "pad"]),
		(0xB6, &["mode", "text"]),
		(0xB7, &["slot", "x", "y", "filename"]),
		(0xB8, &["slot", "visible", "pad"]),
		(0xB9, &["slot", "value", "pad"]),
		(0xBA, &["arg1", "arg2", "arg3", "arg4", "arg5", "arg6", "arg7", "arg8", "text"]),
		(0xBB, &["pad"]),
		(0xBC, &["slot", "frame", "state", "pad"]),
		(0xBD, &["flags", "pad"]),
		(0xBE, &["slot_a", "slot_b", "pad"]),
		(0xBF, &["arg1", "arg2", "arg3", "arg4", "pad"]),
		(0xE0, &["text"]),
		(0xE2, &["pad"]),
		(0xE3, &["pad"]),
		(0xE4, &["mode", "pad"]),
		(0xE5, &["pad"]),
		(0xE6, &["pad"]),
		(0xE7, &["table_id", "pad"]),
		(0xE8, &["filename"]),
		(0xE9, &["pad"]),
		(0xEA, &["arg1", "filename"]),
		(0xEB, &["pad"]),
		(0xFF, &[]),
	];

	/// A minimal, self-consistent encoding of `spec`: opcode byte plus zeroed operands, one NUL for a
	/// string, and `count = 0` for a choice list.
	fn synthetic(spec: &OpcodeSpecStatic) -> Vec<u8> {
		let mut buf = vec![spec.opcode];
		for code in spec.layout {
			match code {
				Code::Byte => buf.push(0x00),
				Code::Word => buf.extend([0x00, 0x00]),
				Code::DWord => buf.extend([0x00, 0x00, 0x00, 0x00]),
				Code::Str => buf.push(0x00),
				Code::Choice => {}
				Code::Padding(n) => buf.extend(std::iter::repeat_n(0x00, *n as usize)),
			}
		}
		buf
	}

	#[test]
	fn every_opcode_declares_its_operand_names() {
		assert_eq!(
			OPCODE_SPECS.len(),
			EXPECTED.len(),
			"opcode count changed: update EXPECTED with the table"
		);
		for (spec, (byte, names)) in OPCODE_SPECS.iter().zip(EXPECTED.iter()) {
			assert_eq!(spec.opcode, *byte, "opcode order/coverage mismatch");
			assert_eq!(
				spec.operands,
				*names,
				"operand names for 0x{:02X} ({}) disagree with the reviewed table",
				spec.opcode,
				spec.name
			);
			assert_eq!(
				spec.layout.len(),
				spec.operands.len(),
				"0x{:02X} ({}) declares {} layout codes but {} operand names",
				spec.opcode,
				spec.name,
				spec.layout.len(),
				spec.operands.len()
			);
		}
	}

	#[test]
	fn table_decodes_exactly_the_fields_it_declares() {
		for spec in OPCODE_SPECS {
			let input = synthetic(spec);
			let fields = decode_opcode_fields(spec.opcode, &input)
				.unwrap_or_else(|| panic!("0x{:02X} ({}) failed to decode", spec.opcode, spec.name));
			assert_eq!(
				fields.len(),
				spec.operands.len(),
				"0x{:02X} ({}) decoded {} fields for {} declared operands",
				spec.opcode,
				spec.name,
				fields.len(),
				spec.operands.len()
			);

			let opcode = Opcode {
				opcode: spec.opcode,
				address: 0,
				actual_address: 0,
				fields,
			};
			assert_eq!(
				opcode.size(),
				input.len(),
				"0x{:02X} ({}) decoded {} bytes from a {}-byte instruction",
				spec.opcode,
				spec.name,
				opcode.size(),
				input.len()
			);
			assert_eq!(
				opcode.binary_serialise(),
				input,
				"0x{:02X} ({}) does not round-trip",
				spec.opcode,
				spec.name
			);
		}
	}

	fn sample_script() -> Script {
		// 0x03 variable_heap_op (8 bytes) then 0xFF end_of_script.
		decode_wsc(&[0x03, 0x01, 0xC7, 0x02, 0x00, 0x00, 0x00, 0x00, 0xFF])
	}

	#[test]
	fn manifest_is_emitted_once_per_file_for_the_opcodes_in_use() {
		let script = sample_script();
		assert_eq!(
			script
				.opcode_table
				.iter()
				.map(|it| it.opcode)
				.collect::<Vec<_>>(),
			vec![0x03, 0xFF]
		);

		let yaml = fix_yaml_str(serde_yml::to_string(&script).unwrap());
		assert!(yaml.starts_with("opcode_table:"), "manifest must lead the file:\n{yaml}");
		assert!(
			yaml.contains("operands: [ kind, var_index, indirect, value, pad ]"),
			"operand names must be emitted inline:\n{yaml}"
		);
		assert!(
			yaml.contains("layout: b w b w p 1"),
			"layout must use the code spelling:\n{yaml}"
		);
		assert_eq!(script.opcode_table[0].name, "variable_heap_op");
		// The mnemonic is defined once, in the manifest: instruction records carry no name.
		let body_at = yaml.find("\nopcodes:").expect("emitted file has no opcode list");
		assert!(
			!yaml[body_at..].contains("name:"),
			"instructions must not repeat the mnemonic:\n{yaml}"
		);

		let back: Script = serde_yml::from_str(&yaml).unwrap();
		assert_eq!(back.opcode_table, script.opcode_table);
		assert_eq!(back.binary_serialise().unwrap(), script.binary_serialise().unwrap());
	}

	#[test]
	fn instructions_carry_no_mnemonic_but_stale_name_keys_still_parse() {
		// Files written before the mnemonic was removed repeated `name:` on every instruction.
		let stale = "\
opcode_table:
- opcode: 0x03
  name: variable_heap_op
  layout: b w b w p 1
  operands: [ kind, var_index, indirect, value, pad ]
opcodes:
- name: variable_heap_op
  opcode: 0x03
  address: 0x00000000
  fields:
  - !Byte 0x01
  - !Word 0x02C7
  - !Byte 0x00
  - !Word 0x0000
  - !Padding [ 0x00 ]
trailer: [  ]
";
		let script: Script =
			serde_yml::from_str(stale).expect("a stale instruction `name:` key must not break parsing");
		assert_eq!(script.opcodes.len(), 1);
		assert_eq!(script.opcode_table.len(), 1);
		assert!(validate_opcode_table(&script).is_ok());
		// Re-emitting drops the stale key: the mnemonic lives in the manifest only.
		let yaml = fix_yaml_str(serde_yml::to_string(&script).unwrap());
		let body_at = yaml.find("\nopcodes:").expect("emitted file has no opcode list");
		assert!(!yaml[body_at..].contains("name:"), "{yaml}");
	}

	#[test]
	fn encode_rejects_a_manifest_that_disagrees_with_the_compiled_table() {
		let script = sample_script();
		assert!(validate_opcode_table(&script).is_ok());

		let mut renamed = script.opcode_table.clone();
		renamed[0].operands[0] = "renamed_by_hand".to_owned();
		let tampered = Script {
			opcode_table: renamed,
			opcodes: script.opcodes.clone(),
			trailer: script.trailer.clone(),
		};
		let err = validate_opcode_table(&tampered).expect_err("renamed operand must be rejected");
		assert!(err.contains("disagrees with the compiled opcode table"), "{err}");

		let missing = Script {
			opcode_table: script.opcode_table[..1].to_vec(),
			opcodes: script.opcodes.clone(),
			trailer: script.trailer.clone(),
		};
		let err = validate_opcode_table(&missing).expect_err("unlisted opcode must be rejected");
		assert!(err.contains("missing from opcode_table"), "{err}");
	}
}
