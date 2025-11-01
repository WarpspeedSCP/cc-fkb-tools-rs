use std::env::current_dir;
use std::path::{Path, PathBuf};
use ccfkb_lib::{log, main};
use ccfkb_lib::data::{decode_wsc, fix_yaml_str};

fn decode_wsc_file_command(wsc_name_path: &Path) -> String {
    log::info!("Decoding file {}", wsc_name_path.file_name().unwrap_or_default().to_string_lossy());
    let input = std::fs::read(wsc_name_path).unwrap();

    let out = decode_wsc(&input);

    let res = fix_yaml_str(serde_yml::to_string(&out).unwrap());

    res
}

main!(files, "WSC", {
    let target_dir = current_dir().unwrap().join("wsc_files");
    std::fs::create_dir_all(&target_dir).unwrap();
    for file in files {
        let res = decode_wsc_file_command(&file);
        let output_file = target_dir.join(file.file_name().unwrap()).with_extension("WSC.yaml");
        std::fs::write(output_file, res).unwrap();
    }
});
