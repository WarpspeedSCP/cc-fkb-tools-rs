use anyhow::{anyhow, Context};
use ccfkb_lib::bin_utils::transform_asm_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::{entries_with_suffix, safe_create_dir};

/// Writes the translator's text form of every script beside the assembly it is read from:
/// `<arc>.arc.asm/<NAME>.WSC.asm` → `<arc>.arc.script/<NAME>.WSC.txt`. The assembly is only read.
fn main() -> anyhow::Result<()> {
	for asm_dir in main_preamble!(dir ".arc.asm") {
		let script_dir = asm_dir.with_extension("script");
		safe_create_dir(&script_dir)
			.with_context(|| format!("creating {script_dir}"))?;

		for file in entries_with_suffix(&asm_dir, ".WSC.asm")
			.with_context(|| format!("listing {asm_dir}"))?
		{
			let file_name = file
				.file_name()
				.ok_or_else(|| anyhow!("{file} has no file name"))?;
			let out_file = script_dir.join(file_name).with_extension("txt");
			transform_asm_file_command(&file, &out_file)?;
		}
	}

	Ok(())
}
