use ccfkb_lib::bin_utils::transform_wsc_file_command;
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::current_dir;

fn main() {
    let files: Vec<_> = main_preamble!(&"WSC.yaml").collect();
    let current_dir = current_dir();
    if files.len() == 1 {
        let in_file = files.first().unwrap();
        let out_file = current_dir.join(in_file.with_extension("txt"));
        transform_wsc_file_command(&in_file, &out_file);
    }
    else {
        let out_parent_path = files.first().unwrap().parent().unwrap().file_name().unwrap();
        let out_path = current_dir.join(out_parent_path).with_extension("script");
        for i in files {
            let dirent = i;
            let out_file = out_path.join(dirent.with_extension("txt"));
            transform_wsc_file_command(&dirent, &out_file);
        }
    }
}
