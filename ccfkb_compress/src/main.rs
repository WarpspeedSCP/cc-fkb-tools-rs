use std::env::current_dir;
use ccfkb_lib::data::{write_arc, ExtensionDescriptor, FileDescriptor};
use ccfkb_lib::main;

main!(files, "", {
    let files = files.collect::<Vec<_>>();
    let output_file_name = files.first().unwrap().parent().unwrap().file_name().unwrap().to_str().unwrap();
    let extensions_yaml_file = files.iter().find(|it| it.ends_with("extensions.yaml") || it.ends_with("extensions.yml")).unwrap();
    let files_yaml_file = files.iter().find(|it| it.ends_with("files.yaml") || it.ends_with("files.yml")).unwrap();
   
    let ext_descriptors: Vec<ExtensionDescriptor> = serde_yml::from_reader(std::fs::File::open(&extensions_yaml_file).unwrap()).unwrap();
    let file_descriptors: Vec<FileDescriptor> = serde_yml::from_reader(std::fs::File::open(&files_yaml_file).unwrap()).unwrap();

    let files = files
        .iter()
        .map(|it| it.as_path())
        .filter(|it| {
            ext_descriptors
                .iter()
                .any(|ext| it.file_name().unwrap().to_string_lossy().contains(&ext.name))
        })
        .collect::<Vec<_>>();

    let output = write_arc(&files, ext_descriptors, file_descriptors);
    std::fs::write(current_dir().unwrap().join(output_file_name), output).unwrap();
});
