use ccfkb_lib::bin_utils::decode_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::current_dir;


fn main() {
	let files = main_preamble!(&"WSC");

	let target_dir = current_dir().join("wsc_files");
	std::fs::create_dir_all(&target_dir).unwrap();
	for file in files {
		let res = decode_wsc_file_command(&file);
		let output_file = target_dir.join(file.file_name().unwrap()).with_extension("WSC.yaml");
		std::fs::write(output_file, res).unwrap();
	}
}
