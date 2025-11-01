use std::env::current_dir;
use std::path::{Path, PathBuf};
use ccfkb_lib::data::text_script::tl_transform_script;
use ccfkb_lib::{log, main};
use ccfkb_lib::opcodes::Script;

main!(files, "WSC.yaml", {
    let files: Vec<_> = files.collect();
    
    if files.len() == 1 {
        let in_file = files.first().unwrap();
        let out_file = current_dir().unwrap().join(in_file.with_extension("WSC.txt"));
        transform_wsc_file_command(&in_file, &out_file);
    }
    else {
        let out_parent_path = files.first().unwrap().parent().unwrap().file_name().unwrap();
        let out_path = current_dir().unwrap().join(out_parent_path).with_extension("ARC.script");
        for i in files {
            let dirent = i;
            let out_file = out_path.join(dirent.with_extension("WSC.txt"));
            transform_wsc_file_command(&dirent, &out_file);
        }
    }
});

fn transform_wsc_file_command(wsc_name_path: &Path, out_file: &Path) {
    
    log::info!("Transforming file {}", wsc_name_path.file_name().unwrap_or_default().to_string_lossy());
    let input = std::fs::read_to_string(wsc_name_path).unwrap();

    let script = serde_yml::from_str(&input).unwrap();

    let out = tl_transform_script(&script);
    
    std::fs::write(out_file, out).unwrap();
}
