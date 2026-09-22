use anyhow::{anyhow, Context};
use ccfkb_lib::asm::Severity;
use ccfkb_lib::bin_utils::load_asm_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::{entries_with_suffix, safe_create_dir};

/// Assembles every `.WSC.asm` under an `<arc>.arc.asm` directory back into `<arc>/<NAME>.WSC`.
///
/// Every diagnostic is printed with its line and column before anything is decided, so one run
/// reports every problem in the file — not just the first. A file with an error diagnostic is
/// refused (nothing written) and the process exits non-zero; findings alone are reported and the
/// file is assembled.
fn main() -> anyhow::Result<()> {
	let mut refused = false;
	for asm_dir in main_preamble!(dir ".arc.asm") {
		let arc_dir = asm_dir.with_extension("");
		safe_create_dir(&arc_dir)
			.with_context(|| format!("creating {arc_dir}"))?;

		for file in entries_with_suffix(&asm_dir, ".WSC.asm")
			.with_context(|| format!("listing {asm_dir}"))?
		{
			let doc = load_asm_file_command(&file)?;
			eprint!("{}", ccfkb_lib::bin_utils::render_diagnostics(&file, &doc));
			let summary = doc.summary();
			if doc.diagnostics.iter().any(|it| it.severity == Severity::Error) {
				log::error!(
					"{file}: {} error(s), {} finding(s); refusing to assemble",
					summary.errors, summary.findings
				);
				refused = true;
				continue;
			}

			let script = doc.into_script()?;
			let out = script
				.binary_serialise()
				.with_context(|| format!("assembling {file}"))?;

			let out_name = file
				.with_extension("")
				.file_name()
				.ok_or_else(|| anyhow!("{file} has no file name"))?
				.to_owned();
			std::fs::write(arc_dir.join(&out_name), out)
				.with_context(|| format!("writing {arc_dir}/{out_name}"))?;
		}
	}

	if refused {
		std::process::exit(1);
	}
	Ok(())
}
