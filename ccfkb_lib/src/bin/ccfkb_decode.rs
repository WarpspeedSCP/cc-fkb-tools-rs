use anyhow::{anyhow, Context};
use ccfkb_lib::bin_utils::decode_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::safe_create_dir;


fn main() -> anyhow::Result<()> {
	for file in main_preamble!(file ".WSC") {
		// The WSC lives inside the extracted arc dir; its decoded structure goes into the
		// sibling `<arc>.yaml` folder (the same layout ccfkb_unpack produces).
		let yaml_dir = file
			.parent()
			.ok_or_else(|| anyhow!("expected {file} to live inside an arc directory"))?
			.with_extension("arc.yaml");
		safe_create_dir(&yaml_dir)
			.with_context(|| format!("creating {yaml_dir}"))?;

		let res = decode_wsc_file_command(&file)?;
		let file_name = file
			.file_name()
			.ok_or_else(|| anyhow!("{file} has no file name"))?;
		let output_file = yaml_dir.join(file_name).with_extension("WSC.yaml");
		std::fs::write(&output_file, res)
			.with_context(|| format!("writing {output_file}"))?;
	}

	Ok(())
}
