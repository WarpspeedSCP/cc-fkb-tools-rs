use camino::Utf8Path;
use ccfkb_lib::data::{descriptor_paths, pack_arc, ExtensionDescriptor, FileDescriptor};
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

	for entry in script_folder.read_dir_utf8().unwrap() {
		let script_file = entry.unwrap().path().to_owned();
		if !script_file.file_name().map(|it| it.to_ascii_uppercase().ends_with(".WSC.TXT")).unwrap_or(false) {
			continue;
		}

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
	for arc_dir in main_preamble!(file ".arc") {
		reencode_scripts(&arc_dir);

		let ext_desc_yaml = arc_dir.join("extensions.yaml");
		let file_desc_yaml = arc_dir.join("files.yaml");
		let ext_descriptors: Vec<ExtensionDescriptor> = serde_yml::from_reader(std::fs::File::open(&ext_desc_yaml).unwrap()).unwrap();
		let file_descriptors: Vec<FileDescriptor> = serde_yml::from_reader(std::fs::File::open(&file_desc_yaml).unwrap()).unwrap();

		let out_files = descriptor_paths(&arc_dir, &ext_descriptors, &file_descriptors);
		let out_path = arc_dir.with_extension("arc.out");
		pack_arc(&out_path, &out_files, ext_descriptors, file_descriptors).unwrap();
	}
}
