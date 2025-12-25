use crate::data::text_script::{parse_doclines, tl_reverse_transform_script, tl_transform_script};
use crate::data::{decode_wsc, fix_yaml_str};
use crate::opcodes::Script;
use camino::Utf8Path;

pub fn transform_wsc_file_command(wsc_name_path: &Utf8Path, out_file: &Utf8Path) {
	log::info!("Transforming file {}", wsc_name_path.file_name().unwrap_or_default());
	let input = std::fs::read_to_string(wsc_name_path).unwrap();

	let script = serde_yml::from_str(&input).unwrap();

	let out = tl_transform_script(&script);

	std::fs::write(out_file, out).unwrap();
}

pub fn decode_wsc_file_command(wsc_name_path: &Utf8Path) -> String {
	log::info!("Decoding file {}", wsc_name_path.file_name().unwrap_or_default());
	let input = std::fs::read(wsc_name_path).unwrap();

	let out = decode_wsc(&input);

	let res = fix_yaml_str(serde_yml::to_string(&out).unwrap());

	res
}

pub fn untransform_wsc_file_command(wsc_name_path: &Utf8Path, docline_path: &Utf8Path) {
	log::info!("Untransforming file {}", wsc_name_path.file_name().unwrap_or_default());
	let script_text = std::fs::read_to_string(wsc_name_path).unwrap();
	let mut script: Script = serde_yml::from_str(&script_text).unwrap();

	let docline_text = std::fs::read_to_string(docline_path).unwrap();
	let (_, doclines) = parse_doclines(&docline_text).unwrap();

	tl_reverse_transform_script(&mut script, doclines);

	let res = fix_yaml_str(serde_yml::to_string(&script).unwrap());
	std::fs::write(wsc_name_path, res).unwrap();
}
pub fn encode_wsc_file_command(yaml_name_path: &Utf8Path, out_dir_path: &Utf8Path) {
	log::info!("Encoding file {}", yaml_name_path.file_name().unwrap_or_default());
	let input = std::fs::read_to_string(yaml_name_path).unwrap();

	let script: Script = serde_yml::from_str(&input).unwrap();

	let out = script.binary_serialise();

	std::fs::write(out_dir_path.join(yaml_name_path.with_extension("").file_name().unwrap()), out).unwrap();
}
