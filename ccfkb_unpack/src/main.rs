use std::env::current_dir;
use std::path::{Path, PathBuf};
use ccfkb_lib::data::{decode_wsc, fix_yaml_str, read_arc};
use ccfkb_lib::{log, main};
use ccfkb_lib::data::text_script::tl_transform_script;

main!(files, "ARC", {
    let top_out_path = current_dir().unwrap().join("extracted_arcs");
    std::fs::create_dir(&top_out_path).unwrap();
    let files: Vec<_> = files.collect();

    for i in files {
        let dirent = i;
        let out_folder_base_name = &top_out_path.join(dirent.file_name().unwrap());
        let out_yaml_folder = &out_folder_base_name.with_extension("ARC.yaml");
        let out_script_folder = &out_folder_base_name.with_extension("ARC.script");

        std::fs::create_dir(&out_folder_base_name).unwrap();
        std::fs::create_dir(&out_yaml_folder).unwrap();
        std::fs::create_dir(&out_script_folder).unwrap();

        let mut file_contents = std::fs::read(&dirent).unwrap();

        let (exts, files, filenames, data) = read_arc(&mut file_contents[..], &dirent, false);

        let exts_yml_path = out_folder_base_name.join("extensions.yaml");
        let exts_yml = serde_yml::to_string(&exts).unwrap();
        std::fs::write(&exts_yml_path, &exts_yml).unwrap();

        let files_yml_path = out_folder_base_name.join("files.yaml");
        let files_yml = serde_yml::to_string(&files).unwrap();
        std::fs::write(&files_yml_path, &files_yml).unwrap();

        let output_file_paths: Vec<_> = filenames
            .iter()
            .zip(&data)
            .map(|(filename, content)| {
                let out_path = out_folder_base_name.join(filename);
                std::fs::write(&out_path, content).unwrap();
                out_path
            })
            .collect();

        let output_file_paths: Vec<_> = output_file_paths.iter().filter_map(|file| {
            if !file.extension().unwrap().to_string_lossy().contains("WSC") {
                return None;
            }
            let out_path = out_yaml_folder.join(file.file_name().unwrap()).with_extension("WSC.yaml");
            let res = decode_wsc_file_command(&file);
            std::fs::write(&out_path, res).unwrap();

            Some(out_path)
        }).collect();

        output_file_paths.iter().for_each(|file| {
            let out_path = out_script_folder.join(file.file_name().unwrap()).with_extension("WSC.txt");
            transform_wsc_file_command(&dirent, &out_path);
        });
    }
});

fn decode_wsc_file_command(wsc_name_path: &Path) -> String {
    log::info!("Decoding file {}", wsc_name_path.file_name().unwrap_or_default().to_string_lossy());
    let input = std::fs::read(wsc_name_path).unwrap();

    let out = decode_wsc(&input);

    let res = fix_yaml_str(serde_yml::to_string(&out).unwrap());

    res
}

fn transform_wsc_file_command(wsc_name_path: &Path, out_file: &Path) {

    log::info!("Transforming file {}", wsc_name_path.file_name().unwrap_or_default().to_string_lossy());
    let input = std::fs::read_to_string(wsc_name_path).unwrap();

    let script = serde_yml::from_str(&input).unwrap();

    let out = tl_transform_script(&script);

    std::fs::write(out_file, out).unwrap();
}
