use camino::Utf8PathBuf;
use ccfkb_lib::data::{arc_dirs, arc_entries, gen_descriptors_from_files, pack_arc};
use ccfkb_lib::logging;
use ccfkb_lib::util::current_dir;


fn main() {
	logging::init().unwrap();

	for arg in std::env::args().skip(1) {
		let parent = Utf8PathBuf::from(arg);

		for arc_dir in arc_dirs(&parent).unwrap() {
			let entries = arc_entries(&arc_dir).unwrap();

			let (extension_descriptors, file_descriptors, pack_files, file_data_start, data_offset) = gen_descriptors_from_files(&entries);

			let n_extensions = extension_descriptors.len();
			let n_file_descriptors = file_descriptors.len();

			let out_path = current_dir().join(arc_dir.file_name().expect("expected an arc directory name"));
			pack_arc(&out_path, &pack_files, extension_descriptors, file_descriptors).unwrap();

			println!(
				"Packed {n_extensions} extensions ({n_file_descriptors} file descriptors): file data starts at 0x{file_data_start:08X} and ends at 0x{data_offset:08X}."
			);
		}
	}
}
