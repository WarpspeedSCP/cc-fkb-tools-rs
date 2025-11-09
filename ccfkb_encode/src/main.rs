use std::env::current_dir;
use std::fs::create_dir_all;
use std::path::Path;
use ccfkb_lib::{log, main_preamble};
use ccfkb_lib::opcodes::Script;

fn main() {
    let files = main_preamble!(&"WSC.yaml").collect::<Vec<_>>();

    let output_folder = files.first().unwrap().parent().unwrap().file_name().unwrap();
    
    let out_dir_path = current_dir().unwrap().join(output_folder);
    create_dir_all(&out_dir_path).unwrap();

    for file in files {
      if !file.extension().unwrap().to_string_lossy().ends_with("yaml") {
        continue;
      }
      encode_wsc_file_command(&file, &out_dir_path)
    }
}

fn encode_wsc_file_command(yaml_name_path: &Path, out_dir_path: &Path) {
    log::info!("Encoding file {}", yaml_name_path.file_name().unwrap_or_default().to_string_lossy());
    let input = std::fs::read_to_string(yaml_name_path).unwrap();

    let script: Script = serde_yml::from_str(&input).unwrap();

    let out = script.binary_serialise();

    std::fs::write(out_dir_path.join(yaml_name_path.with_extension("").file_name().unwrap()), out).unwrap();
}
