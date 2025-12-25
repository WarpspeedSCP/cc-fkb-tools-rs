use ccfkb_lib::bin_utils::encode_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::current_dir;
use std::fs::create_dir_all;

fn main() {
    let files = main_preamble!(&"WSC.yaml").collect::<Vec<_>>();

    let output_folder = files.first().unwrap().parent().unwrap().file_name().unwrap();
    
    let out_dir_path = current_dir().join(output_folder);
    create_dir_all(&out_dir_path).unwrap();

    for file in files {
      if !file.extension().unwrap().ends_with("yaml") {
        continue;
      }
      encode_wsc_file_command(&file, &out_dir_path)
    }
}

