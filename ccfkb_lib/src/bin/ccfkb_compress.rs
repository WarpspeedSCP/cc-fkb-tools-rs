use std::collections::BTreeMap;
use camino::{Utf8Path, Utf8PathBuf};
use ccfkb_lib::data::{write_arc, ExtensionDescriptor, FileDescriptor, gen_descriptors_from_files};
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::current_dir;


fn main() {
	let files = main_preamble!(&"").collect::<Vec<_>>();

	let arc_dir = files.first().expect("expected a path to the extracted arc directory");
	let output_file_name = arc_dir
		.file_name()
		.expect("expected a directory name")
		.to_owned();

	let files: Vec<_> = files
		.into_iter()
		.filter(|it| !matches!(it.extension(), Some("yaml" | "yml" | "YAML" | "YML")))
		.collect();

	let (extension_descriptors, file_descriptors, pack_files, file_data_start, data_offset) = gen_descriptors_from_files(&files);

	let n_extensions = extension_descriptors.len();
	let n_file_descriptors = file_descriptors.len();

	let output = write_arc(&pack_files, extension_descriptors, file_descriptors);
	std::fs::write(current_dir().join(output_file_name), &output).unwrap();

	println!(
		"Packed {n_extensions} extensions ({n_file_descriptors} file descriptors): file data starts at 0x{file_data_start:08X} and ends at 0x{data_offset:08X}."
	);
}