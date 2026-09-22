use anyhow::{anyhow, Context};
use camino::Utf8Path;
use ccfkb_lib::asm::Severity;
use ccfkb_lib::bin_utils::{
	disassemble_wsc_file_command, load_constants_command, render_source_diagnostics,
};
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::{ends_with_ignore_case, safe_create_dir};

fn main() -> anyhow::Result<()> {
	// At most one constants file may be given: it names the engine values the output spells instead of
	// writing hex, and its path is written into the output as it was given, so it has to resolve from
	// wherever the `.asm` is read back.
	let includes: Vec<String> = std::env::args()
		.skip(1)
		.filter(|it| ends_with_ignore_case(it, &".inc"))
		.collect();
	if includes.len() > 1 {
		return Err(anyhow!(
			"at most one constants file can be given; got {}",
			includes.len()
		));
	}
	let mut constants = None;
	if let Some(path) = includes.first() {
		let path = Utf8Path::new(path);
		let (table, sources, diagnostics) = load_constants_command(path)?;
		eprint!("{}", render_source_diagnostics(path, &sources, &diagnostics));
		if diagnostics.iter().any(|it| it.severity == Severity::Error) {
			return Err(anyhow!("{path}: refusing to disassemble with a constants file that has errors"));
		}
		constants = Some((table, path.to_owned()));
	}

	for file in main_preamble!(file ".WSC") {
		// The WSC lives inside the extracted arc dir; its assembly goes into the sibling
		// `<arc>.arc.asm` folder, beside the translator's `<arc>.arc.script`.
		let asm_dir = file
			.parent()
			.ok_or_else(|| anyhow!("expected {file} to live inside an arc directory"))?
			.with_extension("arc.asm");
		safe_create_dir(&asm_dir)
			.with_context(|| format!("creating {asm_dir}"))?;

		let file_name = file
			.file_name()
			.ok_or_else(|| anyhow!("{file} has no file name"))?;
		let symbols = constants.as_ref().map(|(table, include)| (table, include.as_str()));
		let res = disassemble_wsc_file_command(&file, file_name, symbols)?;
		let output_file = asm_dir.join(file_name).with_extension("WSC.asm");
		std::fs::write(&output_file, res)
			.with_context(|| format!("writing {output_file}"))?;
	}

	Ok(())
}
