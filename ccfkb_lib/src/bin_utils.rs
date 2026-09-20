use anyhow::{anyhow, Context};
use crate::data::text_script::{parse_doclines, tl_reverse_transform_script, tl_transform_script};
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
