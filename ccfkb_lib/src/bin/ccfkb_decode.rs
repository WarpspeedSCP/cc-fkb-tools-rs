use ccfkb_lib::bin_utils::decode_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::safe_create_dir;


fn main() {
	for file in main_preamble!(file ".WSC") {
		// The WSC lives inside the extracted arc dir; its decoded structure goes into the
		// sibling `<arc>.yaml` folder (the same layout ccfkb_unpack produces).
		let yaml_dir = file
			.parent()
			.expect("expected the WSC to live inside an arc directory")
			.with_extension("arc.yaml");
		safe_create_dir(&yaml_dir).unwrap();

		let res = decode_wsc_file_command(&file);
		let output_file = yaml_dir.join(file.file_name().unwrap()).with_extension("WSC.yaml");
		std::fs::write(output_file, res).unwrap();
	}
}
