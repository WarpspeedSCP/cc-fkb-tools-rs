use camino::Utf8PathBuf;
use ccfkb_lib::data::{read_arc, ArcContents};
use ccfkb_lib::main_preamble;

fn main() {
	let files = main_preamble!(file ".arc");

	std::fs::create_dir_all("extracted_arcs").unwrap();

	for i in files {
		let dirent = i;
		let mut file_contents = std::fs::read(&dirent).unwrap();

		let path = Utf8PathBuf::from("extracted_arcs").join(dirent.file_name().unwrap());
		std::fs::create_dir_all(&path).unwrap();

		let ArcContents {
			extensions: exts,
			files,
			filenames,
			data
		} = read_arc(&mut file_contents[..], &path, true);

		for (filename, content) in filenames.iter().zip(&data) {
			let out_path = path.join(filename);
			if out_path.exists() {
				continue;
			}
			std::fs::write(out_path, content).unwrap();
		}
	}
}

