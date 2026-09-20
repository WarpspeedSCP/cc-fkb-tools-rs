use anyhow::Context;
use ccfkb_lib::asm::{print_document, Severity};
use ccfkb_lib::bin_utils::{load_asm_file_command, render_diagnostics};
use ccfkb_lib::main_preamble;

/// Rewrites every `.asm` file in place from its parsed model: `# addr` is recomputed from the
/// derived addresses, inserted lines stay unannotated, `;` comments and `# translation` annotations
/// are re-emitted, and `# script` follows the file name.
///
/// Diagnostics are printed with their line and column first; a file with an error diagnostic is
/// refused and left untouched, and the process exits non-zero.
fn main() -> anyhow::Result<()> {
	let mut refused = false;
	for file in main_preamble!(file ".asm") {
		let doc = load_asm_file_command(&file)?;
		eprint!("{}", render_diagnostics(&file, &doc));
		let summary = doc.summary();
		if doc.diagnostics.iter().any(|it| it.severity == Severity::Error) {
			log::error!(
				"{file}: {} error(s), {} finding(s); refusing to assemble",
				summary.errors, summary.findings
			);
			refused = true;
			continue;
		}

		let script_name = file.file_stem().unwrap_or_default();
		let text = print_document(&doc, script_name)?;
		std::fs::write(&file, text).with_context(|| format!("writing {file}"))?;
	}

	if refused {
		std::process::exit(1);
	}
	Ok(())
}
