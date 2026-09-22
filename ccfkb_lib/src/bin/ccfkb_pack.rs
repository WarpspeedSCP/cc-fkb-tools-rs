use anyhow::{anyhow, bail, Context};
use camino::Utf8Path;
use ccfkb_lib::data::{arc_entries, gen_descriptors_from_files, pack_arc};
use ccfkb_lib::util::entries_with_suffix;
use ccfkb_lib::{log, main_preamble};

use ccfkb_lib::bin_utils::{apply_doclines, load_asm_file_command, render_diagnostics};

/// Re-encodes every script of an arc from its assembly, applying the translated text beside it.
///
/// The translation lives in `<arc>.arc.script/` and the structure it is applied to lives in
/// `<arc>.arc.asm/`, so the encoded `*.WSC` overwrites the unpacked copy inside `<arc>/`. An arc with
/// no assembly tree is packed from the `*.WSC` files it holds — nothing to re-encode is not an error.
/// A file whose assembly has an error is refused (nothing written); the run still packs the rest and
/// exits non-zero.
fn reencode_scripts(arc_dir: &Utf8Path) -> anyhow::Result<bool> {
	let asm_folder = arc_dir.with_extension("arc.asm");
	let script_folder = arc_dir.with_extension("arc.script");

	if !asm_folder.is_dir() {
		// Without the structure there is nothing to re-encode, so the arc's own scripts are what gets
		// packed — unless translations are meant to ship: text beside the arc and the scripts to apply
		// it to, with no structure to apply it to. That is refused rather than packed with the text
		// silently dropped.
		let texts = if script_folder.is_dir() {
			entries_with_suffix(&script_folder, ".WSC.txt")
				.with_context(|| format!("listing {script_folder}"))?
		} else {
			Vec::new()
		};
		let scripts = entries_with_suffix(arc_dir, ".WSC")
			.with_context(|| format!("listing {arc_dir}"))?;
		if !texts.is_empty() && !scripts.is_empty() {
			bail!(
				"{asm_folder} is missing: {} translated script(s) in {script_folder} cannot be applied to the {} script(s) in {arc_dir}; disassemble the arc first",
				texts.len(),
				scripts.len()
			);
		}
		log::info!("{arc_dir}: no {asm_folder}; packing the scripts as they are");
		return Ok(false);
	}

	let mut refused = false;
	reencode_from_assembly(arc_dir, &asm_folder, &script_folder, &mut refused)?;

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

/// Re-encodes every `<NAME>.WSC.asm` into `<arc>/<NAME>.WSC`, applying `<NAME>.WSC.txt` when the
/// translator's text sits beside it. A file whose assembly has an error is refused (nothing written)
/// and `refused` is set; the caller still packs the rest and exits non-zero.
fn reencode_from_assembly(
	arc_dir: &Utf8Path,
	asm_folder: &Utf8Path,
	script_folder: &Utf8Path,
	refused: &mut bool,
) -> anyhow::Result<()> {
	for file in entries_with_suffix(asm_folder, ".WSC.asm")
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
			*refused = true;
			continue;
		}

		// The text beside the script is what a translator edits; without it the assembly alone is
		// the script.
		let docline_file = script_folder.join(file_name).with_extension("txt");
		if docline_file.is_file() {
			let stale = apply_doclines(&mut doc, &docline_file)?;
			if stale > 0 {
				log::info!(
					"{docline_file}: {stale} address tag(s) name an older layout; paired by kind and raw text"
				);
			}
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

	Ok(())
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
