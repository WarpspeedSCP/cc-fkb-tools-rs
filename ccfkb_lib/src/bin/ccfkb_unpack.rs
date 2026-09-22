use anyhow::{anyhow, Context};
use camino::Utf8PathBuf;
use rayon::prelude::*;

use ccfkb_lib::bin_utils::{disassemble_wsc_file_command, transform_asm_file_command};
use ccfkb_lib::data::{read_arc, ArcContents};
use ccfkb_lib::util::current_dir;
use ccfkb_lib::util::safe_create_dir;
use ccfkb_lib::{log, main_preamble};

/// Unpacks every `.arc` into `<arc>/` (its files), `<arc>.arc.asm/` (each `.WSC` disassembled) and
/// `<arc>.arc.script/` (the translator's text form of each script), so the three trees describe the
/// same scripts side by side.
fn main() -> anyhow::Result<()> {
	let top_out_path = current_dir()?.join("extracted_arcs");
	safe_create_dir(&top_out_path)
		.with_context(|| format!("creating {top_out_path}"))?;
	let files: Vec<_> = main_preamble!(file ".arc").collect();

	for dirent in files {
		let file_name = dirent
			.file_name()
			.ok_or_else(|| anyhow!("{dirent} has no file name"))?;
		let out_folder_base = top_out_path.join(file_name);
		let out_asm_folder = out_folder_base.with_extension("arc.asm");
		let out_script_folder = out_folder_base.with_extension("arc.script");

		safe_create_dir(&out_folder_base)
			.with_context(|| format!("creating {out_folder_base}"))?;
		safe_create_dir(&out_asm_folder)
			.with_context(|| format!("creating {out_asm_folder}"))?;
		safe_create_dir(&out_script_folder)
			.with_context(|| format!("creating {out_script_folder}"))?;

		let mut file_contents = std::fs::read(&dirent)
			.with_context(|| format!("reading {dirent}"))?;

		// Only the decoded contents matter here; the descriptors come from what is on disk.
		let ArcContents { 
			filenames,
			data,
			.. 
		} = read_arc(&mut file_contents[..], &out_folder_base, true)?;

		// Rayon workers cannot unwind into an error, so each item carries its own `Result` and the
		// first failure is reported once every item has finished.
		let output_file_paths: Vec<_> = filenames
			.iter()
			.zip(&data)
			.par_bridge()
			.map(|(filename, content)| {
				let out_path = out_folder_base.join(filename);
				if !out_path.is_dir() {
					std::fs::write(&out_path, content)
						.with_context(|| format!("writing {out_path}"))?;
				}
				Ok(out_path)
			})
			.collect::<Vec<anyhow::Result<Utf8PathBuf>>>()
			.into_iter()
			.collect::<anyhow::Result<Vec<_>>>()?;
		log::info!("==============================================");
		log::info!("              Decoding WSC files              ");
		log::info!("==============================================");
		let output_file_paths: Vec<_> = output_file_paths
			.par_iter()
			.map(|file| {
				if !file.extension().map(|it| it.ends_with("WSC")).unwrap_or_default() {
					return Ok(None);
				}
				let file_name = file
					.file_name()
					.ok_or_else(|| anyhow!("{file} has no file name"))?;
				let out_path = out_asm_folder.join(file_name).with_extension("WSC.asm");
				let res = disassemble_wsc_file_command(file, file_name, None)?;
				std::fs::write(&out_path, res)
					.with_context(|| format!("writing {out_path}"))?;

				Ok(Some(out_path))
			})
			.collect::<Vec<anyhow::Result<Option<Utf8PathBuf>>>>()
			.into_iter()
			.collect::<anyhow::Result<Vec<_>>>()?
			.into_iter()
			.flatten()
			.collect();

		log::info!("==============================================");
		log::info!("         Transforming assembly files          ");
		log::info!("==============================================");

		output_file_paths
			.par_iter()
			.map(|file| {
				let file_name = file
					.file_name()
					.ok_or_else(|| anyhow!("{file} has no file name"))?;
				let out_path = out_script_folder.join(file_name).with_extension("txt");
				transform_asm_file_command(file, &out_path)
			})
			.collect::<Vec<anyhow::Result<()>>>()
			.into_iter()
			.collect::<anyhow::Result<()>>()?;
	}

	Ok(())
}
