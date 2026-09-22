use anyhow::{anyhow, bail, Context};
use ccfkb_lib::asm::print_document;
use ccfkb_lib::bin_utils::{apply_doclines, load_asm_file_command, render_diagnostics};
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::entries_with_suffix;

/// Applies every `<arc>.arc.script/<NAME>.WSC.txt` to the assembly beside it, in place: a
/// translation becomes an annotation above its instruction, and a choice block's arms take theirs in
/// record order. An assembly file with an error is refused — the text is not applied and the process
/// exits non-zero — while findings alone are reported and applied, exactly as `ccfkb_assemble`
/// treats them.
fn main() -> anyhow::Result<()> {
	let mut refused = false;
	for script_dir in main_preamble!(dir ".arc.script") {
		let asm_dir = script_dir.with_extension("asm");

		for file in entries_with_suffix(&script_dir, ".WSC.txt")
			.with_context(|| format!("listing {script_dir}"))?
		{
			let file_name = file
				.file_name()
				.ok_or_else(|| anyhow!("{file} has no file name"))?;
			let asm_file = asm_dir.join(file_name).with_extension("asm");
			if !asm_file.is_file() {
				bail!("{asm_file} does not exist; disassemble the arc first");
			}
			let mut doc = load_asm_file_command(&asm_file)?;
			eprint!("{}", render_diagnostics(&asm_file, &doc));
			let summary = doc.summary();
			if doc.has_errors() {
				log::error!(
					"{asm_file}: {} error(s), {} finding(s); refusing to apply {file}",
					summary.errors,
					summary.findings
				);
				refused = true;
				continue;
			}

			apply_doclines(&mut doc, &file)?;
			let printed = print_document(&doc, asm_file.file_stem().unwrap_or_default())?;
			std::fs::write(&asm_file, printed)
				.with_context(|| format!("writing {asm_file}"))?;
		}
	}

	if refused {
		std::process::exit(1);
	}
	Ok(())
}
