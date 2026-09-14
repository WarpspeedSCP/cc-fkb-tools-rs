use ccfkb_lib::bin_utils::untransform_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::{entries_with_suffix, safe_create_dir};


fn main() {
	for script_dir in main_preamble!(dir ".arc.script") {
		let yaml_dir = script_dir.with_extension("yaml");
		safe_create_dir(&yaml_dir).unwrap();

		for file in entries_with_suffix(&script_dir, ".WSC.txt").unwrap() {
			let yaml_path = yaml_dir.join(file.file_name().unwrap()).with_extension("yaml");
			untransform_wsc_file_command(&yaml_path, &file);
		}
	}
}
