use anyhow::{anyhow, Context};
use ccfkb_lib::bin_utils::disassemble_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::safe_create_dir;

fn main() -> anyhow::Result<()> {
	for file in main_preamble!(file ".WSC") {
		// The WSC lives inside the extracted arc dir; its assembly goes into the sibling
		// `<arc>.arc.asm` folder, mirroring ccfkb_decode's `<arc>.arc.yaml`.
		let asm_dir = file
			.parent()
			.ok_or_else(|| anyhow!("expected {file} to live inside an arc directory"))?
			.with_extension("arc.asm");
		safe_create_dir(&asm_dir)
			.with_context(|| format!("creating {asm_dir}"))?;

		let file_name = file
			.file_name()
			.ok_or_else(|| anyhow!("{file} has no file name"))?;
		let res = disassemble_wsc_file_command(&file, file_name)?;
		let output_file = asm_dir.join(file_name).with_extension("WSC.asm");
		std::fs::write(&output_file, res)
			.with_context(|| format!("writing {output_file}"))?;
	}

	Ok(())
}
