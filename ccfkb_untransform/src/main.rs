use std::env::current_dir;
use std::path::Path;
use ccfkb_lib::data::text_script::{parse_doclines, tl_reverse_transform_script};
use ccfkb_lib::{log, main};
use ccfkb_lib::data::fix_yaml_str;
use ccfkb_lib::opcodes::Script;

main!(files, ".WSC.txt", {
    let files: Vec<_> = files.collect();
    let out_parent_path = files.first().unwrap().parent().unwrap().file_name().unwrap();
    let out_path = current_dir().unwrap().join(out_parent_path).with_extension("ARC.yaml");

    if !std::fs::exists(&out_parent_path).unwrap() {
        log::error!("Path {} does not exist", out_parent_path.display());
        std::process::exit(1);
    }

    for file in files {
        let yaml_path = out_path.join(file.with_extension("WSC.yaml").file_name().unwrap());
        untransform_wsc_file_command(&yaml_path, &file);
    }
});

fn untransform_wsc_file_command(wsc_name_path: &Path, docline_path: &Path) {
    log::info!("Untransforming file {}", wsc_name_path.file_name().unwrap_or_default().to_string_lossy());
    let script_text = std::fs::read_to_string(wsc_name_path).unwrap();
    let mut script: Script = serde_yml::from_str(&script_text).unwrap();

    let docline_text = std::fs::read_to_string(docline_path).unwrap();
    let (_, doclines) = parse_doclines(&docline_text).unwrap();

    tl_reverse_transform_script(&mut script, doclines);

    let res = fix_yaml_str(serde_yml::to_string(&script).unwrap());
    std::fs::write(wsc_name_path, res).unwrap();
}
