use ccfkb_lib::bin_utils::transform_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::{entries_with_suffix, safe_create_dir};


fn main() {
	for yaml_dir in main_preamble!(dir ".arc.yaml") {
		let script_dir = yaml_dir.with_extension("script");
		safe_create_dir(&script_dir).unwrap();

		for file in entries_with_suffix(&yaml_dir, ".WSC.yaml").unwrap() {
			let out_file = script_dir.join(file.file_name().unwrap()).with_extension("txt");
			transform_wsc_file_command(&file, &out_file);
		}
	}
}
