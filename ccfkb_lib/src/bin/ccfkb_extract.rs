use camino::Utf8PathBuf;
use ccfkb_lib::data::read_arc;
use ccfkb_lib::main_preamble;

fn main() {
	let files = main_preamble!(&"ARC");

	std::fs::create_dir_all("extracted_arcs").unwrap();

	for i in files {
		let dirent = i;
		let mut file_contents = std::fs::read(&dirent).unwrap();

		let path = Utf8PathBuf::from("extracted_arcs").join(dirent.file_name().unwrap());
		std::fs::create_dir_all(&path).unwrap();
		let (exts, files, filenames, data) = read_arc(&mut file_contents[..], &path, false);

		let exts_yml_path = path.join("extensions.yml");
		let exts_yml = serde_yml::to_string(&exts).unwrap();
		std::fs::write(&exts_yml_path, &exts_yml).unwrap();

		let files_yml_path = path.join("files.yml");
		let files_yml = serde_yml::to_string(&files).unwrap();
		std::fs::write(&files_yml_path, &files_yml).unwrap();

		for (filename, content) in filenames.iter().zip(&data) {
			let out_path = path.join(filename);
			std::fs::write(out_path, content).unwrap();
		}
	}
}

