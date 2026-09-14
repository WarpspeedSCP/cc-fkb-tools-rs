use camino::Utf8Path;
use ccfkb_lib::data::{arc_entries, gen_descriptors_from_files, pack_arc};
use ccfkb_lib::util::entries_with_suffix;
use ccfkb_lib::{log, main_preamble};

use ccfkb_lib::data::text_script::{parse_doclines, tl_reverse_transform_script};
use ccfkb_lib::opcodes::Script;

fn encode_wsc_file_command(yaml_name_path: &Utf8Path, script: Script) -> Vec<u8> {
	log::info!("Encoding file {}", yaml_name_path.file_name().unwrap_or_default());
	script.binary_serialise()
}

fn untransform_wsc_file_command(yaml_file: &Utf8Path, yaml_text: &str, text: &str) -> Script {
	log::info!("Untransforming file {}", &yaml_file.file_name().unwrap_or_default());
	let mut script: Script = serde_yml::from_str(yaml_text).unwrap();
	let (_, doclines) = parse_doclines(text).unwrap();

	tl_reverse_transform_script(&mut script, doclines);

	script
}

/// Re-encodes every edited `*.WSC.txt` back into its arc directory.
///
/// The translation lives in `<arc>.script/` and the structure it is applied to lives in
/// `<arc>.yaml/`, so the encoded `*.WSC` overwrites the unpacked copy inside `<arc>/`.
fn reencode_scripts(arc_dir: &Utf8Path) {
	let script_folder = arc_dir.with_extension("arc.script");
	let yaml_folder = arc_dir.with_extension("arc.yaml");

	if !script_folder.is_dir() {
		return;
	}

	for script_file in entries_with_suffix(&script_folder, ".WSC.txt").unwrap() {
		let text = std::fs::read_to_string(&script_file).unwrap();
		let yaml_file = yaml_folder.join(script_file.file_name().unwrap()).with_extension("yaml");
		let yaml_text = std::fs::read_to_string(&yaml_file).unwrap();

		let yaml_script = untransform_wsc_file_command(&yaml_file, &yaml_text, &text);
		let encoded_script = encode_wsc_file_command(&yaml_file, yaml_script);

		let out_name = script_file.file_stem().expect("expected a script file name");
		std::fs::write(arc_dir.join(out_name), encoded_script).unwrap();
	}
}

fn main() {
	for arc_dir in main_preamble!(dir ".arc") {
		reencode_scripts(&arc_dir);

		// The descriptors come exclusively from the arc dir's immediate children, so the
		// packed layout reflects what is actually on disk (edit `<arc>.script` to change it).
		let entries = arc_entries(&arc_dir).unwrap();
		let (extension_descriptors, file_descriptors, pack_files, _, _) = gen_descriptors_from_files(&entries);

		let out_path = arc_dir.with_extension("arc.out");
		pack_arc(&out_path, &pack_files, extension_descriptors, file_descriptors).unwrap();
	}
}
