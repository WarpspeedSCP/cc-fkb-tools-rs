use anyhow::{anyhow, Context};
use crate::data::text_script::{parse_doclines, tl_reverse_transform_script, tl_transform_script};
use crate::asm::parse::load_constants;
use crate::asm::{
	parse_document, print_script, print_script_with_constants, AsmDocument, ConstantTable,
	Diagnostic, Source,
};
use crate::data::decode_wsc;
use camino::Utf8Path;

/// Reads an `.asm` file and writes the translator's text form of the script it holds to `out_file`:
/// one `[…]` block per text-bearing opcode, with every raw and translation the model carries, so the
/// sidecar and the assembly describe the same script.
pub fn transform_asm_file_command(asm_path: &Utf8Path, out_file: &Utf8Path) -> anyhow::Result<()> {
	log::info!("Transforming file {}", asm_path.file_name().unwrap_or_default());
	let script = load_asm_file_command(asm_path)?.into_script()?;

	let out = tl_transform_script(&script)
		.with_context(|| format!("transforming {asm_path}"))?;

	std::fs::write(out_file, out)
		.with_context(|| format!("writing {out_file}"))?;

	Ok(())
}

/// Applies a text file to a parsed document: every raw text, `[translation]`, `[choice translation]`
/// and notes line it holds lands in the document's model, so printing the document carries the
/// translations as annotations above the instructions — and the choice translations as the
/// annotations that name the arms.
pub fn apply_doclines(doc: &mut AsmDocument, docline_path: &Utf8Path) -> anyhow::Result<()> {
	log::info!("Applying messages of {}", docline_path.file_name().unwrap_or_default());
	let docline_text = std::fs::read_to_string(docline_path)
		.with_context(|| format!("reading {docline_path}"))?;
	let (_, doclines) = parse_doclines(&docline_text)
		.map_err(|err| anyhow!("parsing {docline_path} as translated script text: {err}"))?;

	let script_path = doc.sources.first().map(|it| it.path.as_str()).unwrap_or_default();
	tl_reverse_transform_script(&mut doc.script, doclines)
		.with_context(|| format!("applying {docline_path} to {script_path}"))?;

	Ok(())
}

/// Reads a decoded `.WSC` and returns its assembly text: canonically, or naming every value the
/// `constants` table declares when one was given on the command line (with the `include` line,
/// spelling that table's path, that puts it back in scope when the text is read again).
pub fn disassemble_wsc_file_command(
	wsc_name_path: &Utf8Path,
	script_name: &str,
	constants: Option<(&ConstantTable, &str)>,
) -> anyhow::Result<String> {
	log::info!("Disassembling file {}", wsc_name_path.file_name().unwrap_or_default());
	let input = std::fs::read(wsc_name_path)
		.with_context(|| format!("reading {wsc_name_path}"))?;

	let script = decode_wsc(&input);

	match constants {
		Some((table, include)) => print_script_with_constants(&script, script_name, table, include),
		None => print_script(&script, script_name),
	}
	.with_context(|| format!("printing {wsc_name_path} as assembly"))
}

/// Loads a constants file and everything it includes. The root file was named on the command line, so
/// it must exist; anything it includes reports a diagnostic instead.
pub fn load_constants_command(
	path: &Utf8Path,
) -> anyhow::Result<(ConstantTable, Vec<Source>, Vec<Diagnostic>)> {
	log::info!("Loading constants {}", path.file_name().unwrap_or_default());
	load_constants(path)
}

/// Reads and parses an `.asm` file. Parsing never fails: problems come back as diagnostics.
pub fn load_asm_file_command(asm_name_path: &Utf8Path) -> anyhow::Result<AsmDocument> {
	log::info!("Reading file {}", asm_name_path.file_name().unwrap_or_default());
	let input = std::fs::read_to_string(asm_name_path)
		.with_context(|| format!("reading {asm_name_path}"))?;

	Ok(parse_document(&input, asm_name_path))
}

/// Renders a document's diagnostics, one per line, as
/// `{path}:{line}:{column}: {error|warning}: {message}` in ascending `(source, line, column)` order —
/// the order `parse_document` produced. A diagnostic a constants file raised names that file, and the
/// script's own name the path this was called with. Empty when the file has none.
pub fn render_diagnostics(path: &Utf8Path, doc: &AsmDocument) -> String {
	render_source_diagnostics(path, &doc.sources, &doc.diagnostics)
}

/// The same rendering for diagnostics that have no document behind them, which is what loading a
/// constants file on its own produces.
pub fn render_source_diagnostics(
	path: &Utf8Path,
	sources: &[Source],
	diagnostics: &[Diagnostic],
) -> String {
	if diagnostics.is_empty() {
		return String::new();
	}
	let lines: Vec<String> = diagnostics
		.iter()
		.map(|it| {
			let source = sources
				.get(it.source)
				.map(|it| it.path.as_str())
				.filter(|it| !it.is_empty())
				.unwrap_or_else(|| path.as_str());
			it.render(source)
		})
		.collect();
	format!("{}\n", lines.join("\n"))
}
