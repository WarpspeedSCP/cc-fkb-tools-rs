use anyhow::{anyhow, Context};
use crate::data::text_script::{parse_doclines, tl_reverse_transform_script, tl_transform_script};
use crate::asm::parse::load_constants;
use crate::asm::{
	parse_document, print_script, print_script_with_constants, AsmDocument, ConstantTable,
	Diagnostic, Source,
};
use crate::data::{decode_wsc, fix_yaml_str};
use crate::opcodes::{validate_opcode_table, Script};
use camino::Utf8Path;

pub fn transform_wsc_file_command(wsc_name_path: &Utf8Path, out_file: &Utf8Path) -> anyhow::Result<()> {
	log::info!("Transforming file {}", wsc_name_path.file_name().unwrap_or_default());
	let input = std::fs::read_to_string(wsc_name_path)
		.with_context(|| format!("reading {wsc_name_path}"))?;

	let script: Script = serde_yml::from_str(&input)
		.with_context(|| format!("parsing {wsc_name_path} as a decoded script"))?;

	let out = tl_transform_script(&script)
		.with_context(|| format!("transforming {wsc_name_path}"))?;

	std::fs::write(out_file, out)
		.with_context(|| format!("writing {out_file}"))?;

	Ok(())
}

pub fn decode_wsc_file_command(wsc_name_path: &Utf8Path) -> anyhow::Result<String> {
	log::info!("Decoding file {}", wsc_name_path.file_name().unwrap_or_default());
	let input = std::fs::read(wsc_name_path)
		.with_context(|| format!("reading {wsc_name_path}"))?;

	let out = decode_wsc(&input);

	let res = serde_yml::to_string(&out)
		.map(|it| fix_yaml_str(it))
		.with_context(|| format!("serialising {wsc_name_path} as YAML"))?;

	Ok(res)
}

pub fn untransform_wsc_file_command(wsc_name_path: &Utf8Path, docline_path: &Utf8Path) -> anyhow::Result<()> {
	log::info!("Untransforming file {}", wsc_name_path.file_name().unwrap_or_default());
	let script_text = std::fs::read_to_string(wsc_name_path)
		.with_context(|| format!("reading {wsc_name_path}"))?;
	let mut script: Script = serde_yml::from_str(&script_text)
		.with_context(|| format!("parsing {wsc_name_path} as a decoded script"))?;

	let docline_text = std::fs::read_to_string(docline_path)
		.with_context(|| format!("reading {docline_path}"))?;
	let (_, doclines) = parse_doclines(&docline_text)
		.map_err(|err| anyhow!("parsing {docline_path} as translated script text: {err}"))?;

	tl_reverse_transform_script(&mut script, doclines)
		.with_context(|| format!("applying {docline_path} to {wsc_name_path}"))?;

	let res = fix_yaml_str(serde_yml::to_string(&script)
		.with_context(|| format!("serialising {wsc_name_path} as YAML"))?);
	std::fs::write(wsc_name_path, res)
		.with_context(|| format!("writing {wsc_name_path}"))?;

	Ok(())
}

pub fn encode_wsc_file_command(yaml_name_path: &Utf8Path, out_dir_path: &Utf8Path) -> anyhow::Result<()> {
	log::info!("Encoding file {}", yaml_name_path.file_name().unwrap_or_default());
	let input = std::fs::read_to_string(yaml_name_path)
		.with_context(|| format!("reading {yaml_name_path}"))?;

	let script: Script = serde_yml::from_str(&input)
		.with_context(|| format!("parsing {yaml_name_path} as a decoded script"))?;

	validate_opcode_table(&script)
		.map_err(|err| anyhow!("Refusing to encode {yaml_name_path}: {err}"))?;

	let out = script.binary_serialise()
		.with_context(|| format!("encoding {yaml_name_path}"))?;

	let out_name = yaml_name_path.with_extension("").file_name()
		.ok_or_else(|| anyhow!("{yaml_name_path} has no file name"))?
		.to_owned();

	std::fs::write(out_dir_path.join(&out_name), out)
		.with_context(|| format!("writing {out_dir_path}/{out_name}"))?;

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
