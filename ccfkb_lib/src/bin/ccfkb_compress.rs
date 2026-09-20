use anyhow::anyhow;
use ccfkb_lib::data::{arc_entries, gen_descriptors_from_files, pack_arc};
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::current_dir;


fn main() -> anyhow::Result<()> {
	for arc_dir in main_preamble!(dir ".arc") {
		let entries = arc_entries(&arc_dir)?;

		let (extension_descriptors, file_descriptors, pack_files, file_data_start, data_offset) = gen_descriptors_from_files(&entries)?;

		let n_extensions = extension_descriptors.len();
		let n_file_descriptors = file_descriptors.len();

		let arc_name = arc_dir
			.file_name()
			.ok_or_else(|| anyhow!("{arc_dir} has no directory name"))?;
		let out_path = current_dir()?.join(arc_name);
		pack_arc(&out_path, &pack_files, extension_descriptors, file_descriptors)?;

		println!(
			"Packed {n_extensions} extensions ({n_file_descriptors} file descriptors): file data starts at 0x{file_data_start:08X} and ends at 0x{data_offset:08X}."
		);
	}

	Ok(())
}
