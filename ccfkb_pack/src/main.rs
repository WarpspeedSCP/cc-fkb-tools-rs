use std::path::{Path, PathBuf};
use ccfkb_lib::data::{write_arc, ExtensionDescriptor, FileDescriptor};
use ccfkb_lib::{log, main_preamble};

use ccfkb_lib::data::text_script::{parse_doclines, tl_reverse_transform_script};
use ccfkb_lib::opcodes::Script;

fn encode_wsc_file_command(yaml_name_path: &Path, script: Script) -> Vec<u8> {
    log::info!("Encoding file {}", yaml_name_path.file_name().unwrap_or_default().to_string_lossy());
    let out = script.binary_serialise();
    out
}

fn untransform_wsc_file_command(yaml_file: &Path, yaml_text: &str, text: &str) -> Script {
    log::info!("Untransforming file {}", &yaml_file.file_name().unwrap_or_default().to_string_lossy());
    let mut script: Script = serde_yml::from_str(&yaml_text).unwrap();
    let (_, doclines) = parse_doclines(&text).unwrap();

    tl_reverse_transform_script(&mut script, doclines);

    script
}

fn main() {
    let files: Vec<_> = main_preamble!(&"WSC.txt").collect();
    let files = if files.is_empty() {
        let args = std::env::args().collect::<Vec<_>>();
        args.iter().skip(1).map(|it| PathBuf::from(it).join("nonexistant")).collect::<Vec<_>>()
    } else {
        files
    };
    let yaml_folder = files.first().unwrap().parent().unwrap().with_extension("yaml");
    let output_folder = yaml_folder.with_extension("");
    let file_desc_yaml = output_folder.join("files.yaml");
    let ext_desc_yaml = output_folder.join("extensions.yaml");

    for script_file in files {
        if script_file.file_name().unwrap().to_string_lossy() == "nonexistant" {
            break;
        }

        let text = std::fs::read_to_string(&script_file).unwrap();
        let yaml_file = yaml_folder.join(script_file.file_name().unwrap()).with_extension("yaml");
        let yaml_text = std::fs::read_to_string(&yaml_file).unwrap();

        let yaml_script = untransform_wsc_file_command(&yaml_file, &yaml_text, &text);

        let encoded_script = encode_wsc_file_command(&yaml_file, yaml_script);
        std::fs::write(output_folder.join(yaml_file.with_extension("").file_name().unwrap()), encoded_script).unwrap();
    }

    {
        let ext_descriptors: Vec<ExtensionDescriptor> = serde_yml::from_reader(std::fs::File::open(&ext_desc_yaml).unwrap()).unwrap();
        let file_descriptors: Vec<FileDescriptor> = serde_yml::from_reader(std::fs::File::open(&file_desc_yaml).unwrap()).unwrap();

        let mut file_desc_iter = file_descriptors.iter().peekable();

        let mut out_files = vec![];
        for ExtensionDescriptor { name: ext, number, .. } in ext_descriptors.iter() {
            for _ in 0..*number {
                if let Some(file_desc) = file_desc_iter.next() {
                    let name = format!("{}.{}", file_desc.name, ext);
                    out_files.push(output_folder.join(name));
                } else {
                    log::warn!("No more files left!");
                }
            }
        }

        let output = write_arc(&out_files, ext_descriptors, file_descriptors);
        std::fs::write(output_folder.with_extension("arc.out"), output).unwrap();
    }
}
