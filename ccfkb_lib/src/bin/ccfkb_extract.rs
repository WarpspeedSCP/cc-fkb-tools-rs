use anyhow::{anyhow, Context};
use camino::Utf8PathBuf;
use ccfkb_lib::data::{read_arc, ArcContents};
use ccfkb_lib::main_preamble;

fn main() -> anyhow::Result<()> {
	let files = main_preamble!(file ".arc");

	let out_dir = Utf8PathBuf::from("extracted_arcs");
	std::fs::create_dir_all(&out_dir)
		.with_context(|| format!("creating {out_dir}"))?;

	for dirent in files {
		let mut file_contents = std::fs::read(&dirent)
			.with_context(|| format!("reading {dirent}"))?;

		let file_name = dirent
			.file_name()
			.ok_or_else(|| anyhow!("{dirent} has no file name"))?;
		let path = out_dir.join(file_name);
		std::fs::create_dir_all(&path)
			.with_context(|| format!("creating {path}"))?;

		// Only the decoded contents matter here; the descriptors come from what is on disk.
		let ArcContents {
			filenames,
			data,
			..
		} = read_arc(&mut file_contents[..], &path, true)?;

		for (filename, content) in filenames.iter().zip(&data) {
			let out_path = path.join(filename);
			if out_path.exists() {
				continue;
			}
			std::fs::write(&out_path, content)
				.with_context(|| format!("writing {out_path}"))?;
		}
	}

	Ok(())
}
