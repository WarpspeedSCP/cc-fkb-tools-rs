use anyhow::{anyhow, Context};
use ccfkb_lib::bin_utils::untransform_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::{entries_with_suffix, safe_create_dir};


fn main() -> anyhow::Result<()> {
	for script_dir in main_preamble!(dir ".arc.script") {
		let yaml_dir = script_dir.with_extension("yaml");
		safe_create_dir(&yaml_dir)
			.with_context(|| format!("creating {yaml_dir}"))?;

		for file in entries_with_suffix(&script_dir, ".WSC.txt")
			.with_context(|| format!("listing {script_dir}"))?
		{
			let file_name = file
				.file_name()
				.ok_or_else(|| anyhow!("{file} has no file name"))?;
			let yaml_path = yaml_dir.join(file_name).with_extension("yaml");
			untransform_wsc_file_command(&yaml_path, &file)?;
		}
	}

	Ok(())
}
