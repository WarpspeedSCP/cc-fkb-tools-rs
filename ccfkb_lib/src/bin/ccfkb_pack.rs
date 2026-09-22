use anyhow::{anyhow, bail, Context};
use camino::Utf8Path;
use ccfkb_lib::data::{arc_entries, gen_descriptors_from_files, pack_arc};
use ccfkb_lib::util::entries_with_suffix;
use ccfkb_lib::{log, main_preamble};

use ccfkb_lib::bin_utils::{apply_doclines, load_asm_file_command, render_diagnostics};

/// Re-encodes every script of an arc from its assembly, applying the translated text beside it.
///
/// The translation lives in `<arc>.arc.script/` and the structure it is applied to lives in
/// `<arc>.arc.asm/`, so the encoded `*.WSC` overwrites the unpacked copy inside `<arc>/`. A file
/// whose assembly has an error is refused (nothing written); the run still packs the rest and exits
/// non-zero.
fn reencode_scripts(arc_dir: &Utf8Path) -> anyhow::Result<bool> {
	let asm_folder = arc_dir.with_extension("arc.asm");
	let script_folder = arc_dir.with_extension("arc.script");

	if !asm_folder.is_dir() {
		bail!("{asm_folder} is not a directory");
	}

	let mut refused = false;
	for file in entries_with_suffix(&asm_folder, ".WSC.asm")
		.with_context(|| format!("listing {asm_folder}"))?
	{
		let file_name = file
			.file_name()
			.ok_or_else(|| anyhow!("{file} has no file name"))?;
		let mut doc = load_asm_file_command(&file)?;
		eprint!("{}", render_diagnostics(&file, &doc));
		let summary = doc.summary();
		if doc.has_errors() {
			log::error!(
				"{file}: {} error(s), {} finding(s); refusing to assemble",
				summary.errors,
				summary.findings
			);
			refused = true;
			continue;
		}

		// The text beside the script is what a translator edits; without it the assembly alone is
		// the script.
		let docline_file = script_folder.join(file_name).with_extension("txt");
		if docline_file.is_file() {
			apply_doclines(&mut doc, &docline_file)?;
		}

		let out_name = file
			.file_stem()
			.ok_or_else(|| anyhow!("expected a script file name, got {file}"))?;
		let out = doc
			.into_script()?
			.binary_serialise()
			.with_context(|| format!("encoding {file}"))?;
		std::fs::write(arc_dir.join(out_name), out)
			.with_context(|| format!("writing {arc_dir}/{out_name}"))?;
	}

	// A translated file with no assembly beside it has nothing to be applied to, so it is not
	// silently dropped from the arc.
	if script_folder.is_dir() {
		for file in entries_with_suffix(&script_folder, ".WSC.txt")
			.with_context(|| format!("listing {script_folder}"))?
		{
			let file_name = file
				.file_name()
				.ok_or_else(|| anyhow!("{file} has no file name"))?;
			let asm_file = asm_folder.join(file_name).with_extension("asm");
			if !asm_file.is_file() {
				bail!("{file} has no assembly file in {asm_folder}");
			}
		}
	}

	Ok(refused)
}

fn main() -> anyhow::Result<()> {
	let mut refused = false;
	for arc_dir in main_preamble!(dir ".arc") {
		refused |= reencode_scripts(&arc_dir)?;

		// The descriptors come exclusively from the arc dir's immediate children, so the
		// packed layout reflects what is actually on disk (edit `<arc>.arc.asm` to change it).
		let entries = arc_entries(&arc_dir)?;
		let (extension_descriptors, file_descriptors, pack_files, _, _) =
			gen_descriptors_from_files(&entries)?;

		let out_path = arc_dir.with_extension("arc.out");
		pack_arc(&out_path, &pack_files, extension_descriptors, file_descriptors)?;
	}

	if refused {
		std::process::exit(1);
	}
	Ok(())
}
