use std::path::{Path, PathBuf};
use ccfkb_lib::data::{decode_wsc, fix_yaml_str, read_arc};
use ccfkb_lib::{log, main};
use ccfkb_lib::data::text_script::tl_transform_script;

main!(files, ".WSC.txt", {
    let files: Vec<_> = files.collect();
    let yaml_folder = files.first().unwrap().parent().unwrap().with_extension("ARC.yaml");



    // for i in files {
    //     let dirent = i;
    //     let mut file_contents = std::fs::read(&dirent).unwrap();
    //
    //     let path = PathBuf::from("extracted_arcs").join(dirent.file_name().unwrap());
    //     std::fs::create_dir_all(&path).unwrap();
    //     let (exts, files, filenames, data) = read_arc(&mut file_contents[..], &path, false);
    //
    //     let exts_yml_path = path.join("extensions.yaml");
    //     let exts_yml = serde_yml::to_string(&exts).unwrap();
    //     std::fs::write(&exts_yml_path, &exts_yml).unwrap();
    //
    //     let files_yml_path = path.join("files.yaml");
    //     let files_yml = serde_yml::to_string(&files).unwrap();
    //     std::fs::write(&files_yml_path, &files_yml).unwrap();
    //
    //     let output_file_paths: Vec<_> = filenames
    //         .iter()
    //         .zip(&data)
    //         .map(|(filename, content)| {
    //             let out_path = path.join(filename);
    //             std::fs::write(&out_path, content).unwrap();
    //             out_path
    //         })
    //         .collect();
    //
    //     let path = path.with_extension("ARC.yaml");
    //     let output_file_paths: Vec<_> = output_file_paths.iter().map(|file| {
    //         let out_path = path.join(file.file_name().unwrap()).with_extension("WSC.yaml");
    //         let res = decode_wsc_file_command(&file);
    //         std::fs::write(&out_path, res).unwrap();
    //
    //         out_path
    //     }).collect();
    //
    //     let path = path.with_extension("ARC.script");
    //     output_file_paths.iter().for_each(|file| {
    //         let out_path = path.join(file.file_name().unwrap()).with_extension("WSC.txt");
    //         transform_wsc_file_command(&dirent, &out_path);
    //     });
    // }
});
