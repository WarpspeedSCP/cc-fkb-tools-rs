use ccfkb_lib::bin_utils::encode_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::{entries_with_suffix, safe_create_dir};


fn main() {
	for yaml_dir in main_preamble!(dir ".arc.yaml") {
		let arc_dir = yaml_dir.with_extension("");
		safe_create_dir(&arc_dir).unwrap();

		for file in entries_with_suffix(&yaml_dir, ".WSC.yaml").unwrap() {
			encode_wsc_file_command(&file, &arc_dir);
		}
	}
}
