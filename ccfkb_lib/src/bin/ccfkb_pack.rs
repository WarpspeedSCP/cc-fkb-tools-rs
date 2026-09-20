use anyhow::{anyhow, Context};
use camino::Utf8Path;
use ccfkb_lib::data::{arc_entries, gen_descriptors_from_files, pack_arc};
use ccfkb_lib::util::entries_with_suffix;
use ccfkb_lib::{log, main_preamble};

use ccfkb_lib::data::text_script::{parse_doclines, tl_reverse_transform_script};
use ccfkb_lib::opcodes::{validate_opcode_table, Script};

fn encode_wsc_file_command(yaml_name_path: &Utf8Path, script: Script) -> anyhow::Result<Vec<u8>> {
	log::info!("Encoding file {}", yaml_name_path.file_name().unwrap_or_default());
	validate_opcode_table(&script)
		.map_err(|err| anyhow!("Refusing to encode {yaml_name_path}: {err}"))?;

	script.binary_serialise()
		.with_context(|| format!("encoding {yaml_name_path}"))
}

fn untransform_wsc_file_command(yaml_file: &Utf8Path, yaml_text: &str, text: &str) -> anyhow::Result<Script> {
	log::info!("Untransforming file {}", &yaml_file.file_name().unwrap_or_default());
	let mut script: Script = serde_yml::from_str(yaml_text)
		.with_context(|| format!("parsing {yaml_file} as a decoded script"))?;

	let (_, doclines) = parse_doclines(text)
		.map_err(|err| anyhow!("Failed to parse doclines for {yaml_file}: {err}"))?;
	tl_reverse_transform_script(&mut script, doclines)
		.with_context(|| format!("applying the script text of {yaml_file}"))?;

	Ok(script)
}

/// Re-encodes every edited `*.WSC.txt` back into its arc directory.
///
/// The translation lives in `<arc>.script/` and the structure it is applied to lives in
/// `<arc>.yaml/`, so the encoded `*.WSC` overwrites the unpacked copy inside `<arc>/`.
fn reencode_scripts(arc_dir: &Utf8Path) -> anyhow::Result<()> {
	let script_folder = arc_dir.with_extension("arc.script");
	let yaml_folder = arc_dir.with_extension("arc.yaml");

	if !script_folder.is_dir() {
		return Err(anyhow!("{script_folder} is not a directory"));
	}

	for script_file in entries_with_suffix(&script_folder, ".WSC.txt")? {
		let text = std::fs::read_to_string(&script_file)?;
		let script_name = script_file
			.file_name()
			.ok_or_else(|| anyhow!("{script_file} has no file name"))?;
		let yaml_file = yaml_folder.join(script_name).with_extension("yaml");
		let yaml_text = std::fs::read_to_string(&yaml_file)?;

		let yaml_script = untransform_wsc_file_command(&yaml_file, &yaml_text, &text)?;
		let encoded_script = encode_wsc_file_command(&yaml_file, yaml_script)?;

		let out_name = script_file.file_stem().ok_or(anyhow!("expected a script file name, got {script_file}"))?;
		std::fs::write(arc_dir.join(out_name), encoded_script)?;
	}

	Ok(())
}

fn main() -> anyhow::Result<()> {
	for arc_dir in main_preamble!(dir ".arc") {
		reencode_scripts(&arc_dir)?;

		// The descriptors come exclusively from the arc dir's immediate children, so the
		// packed layout reflects what is actually on disk (edit `<arc>.script` to change it).
		let entries = arc_entries(&arc_dir)?;
		let (extension_descriptors, file_descriptors, pack_files, _, _) = gen_descriptors_from_files(&entries)?;

		let out_path = arc_dir.with_extension("arc.out");
		pack_arc(&out_path, &pack_files, extension_descriptors, file_descriptors)?;
	}

	Ok(())
}
