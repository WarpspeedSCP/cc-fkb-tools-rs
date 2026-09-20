use anyhow::{anyhow, Context};
use ccfkb_lib::bin_utils::transform_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::{entries_with_suffix, safe_create_dir};


fn main() -> anyhow::Result<()> {
	for yaml_dir in main_preamble!(dir ".arc.yaml") {
		let script_dir = yaml_dir.with_extension("script");
		safe_create_dir(&script_dir)
			.with_context(|| format!("creating {script_dir}"))?;

		for file in entries_with_suffix(&yaml_dir, ".WSC.yaml")
			.with_context(|| format!("listing {yaml_dir}"))?
		{
			let file_name = file
				.file_name()
				.ok_or_else(|| anyhow!("{file} has no file name"))?;
			let out_file = script_dir.join(file_name).with_extension("txt");
			transform_wsc_file_command(&file, &out_file)?;
		}
	}

	Ok(())
}
