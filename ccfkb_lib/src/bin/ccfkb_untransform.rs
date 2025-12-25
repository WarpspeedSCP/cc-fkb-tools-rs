use ccfkb_lib::bin_utils::untransform_wsc_file_command;
use ccfkb_lib::util::current_dir;
use ccfkb_lib::{log, main_preamble};

fn main() {
	let files: Vec<_> = main_preamble!(&"WSC.txt").collect();
	let out_parent_path = files.first().unwrap().parent().unwrap().file_name().unwrap();
	let out_path = current_dir().join(out_parent_path).with_extension("yaml");

	if !std::fs::exists(&out_parent_path).unwrap() {
		log::error!("Path {} does not exist", out_parent_path);
		std::process::exit(1);
	}

	for file in files {
		let yaml_path = out_path.join(file.with_extension("yaml").file_name().unwrap());
		untransform_wsc_file_command(&yaml_path, &file);
	}
}

