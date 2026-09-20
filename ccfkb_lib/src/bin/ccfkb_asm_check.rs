use ccfkb_lib::bin_utils::{load_asm_file_command, render_diagnostics};
use ccfkb_lib::main_preamble;

/// Reports every problem in every `.asm` file with its line and column, then a per-file summary.
/// Annotations and inserted instructions are counted but never fail the run; a file with an error
/// diagnostic makes the process exit 1.
fn main() -> anyhow::Result<()> {
	let mut failed = false;
	for file in main_preamble!(file ".asm") {
		let doc = load_asm_file_command(&file)?;
		print!("{}", render_diagnostics(&file, &doc));
		let summary = doc.summary();
		println!(
			"{file}: {} instructions ({} annotated, {} inserted, {} errors, {} findings), {} bytes",
			summary.instructions,
			summary.annotated,
			summary.inserted,
			summary.errors,
			summary.findings,
			doc.byte_len()
		);
		if summary.errors > 0 {
			failed = true;
		}
	}

	if failed {
		std::process::exit(1);
	}
	Ok(())
}
