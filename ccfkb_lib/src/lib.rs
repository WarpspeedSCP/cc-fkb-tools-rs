pub mod data;
pub mod opcodes;
pub mod util;
pub mod bin_utils;
pub mod logging;

pub use itertools;
pub use log;
pub use once_cell;
pub use rayon;
pub use walkdir;

#[macro_export]
macro_rules! main_preamble_inner {
     ($type: expr, $dirness: expr) => {
          {
            use camino::Utf8PathBuf as PathBuf;
            use ccfkb_lib::walkdir;
            use ccfkb_lib::logging;
            use ccfkb_lib::util::Dirness;

            logging::init().unwrap();

            let args = std::env::args().skip(1).collect::<Vec<_>>();

            let files = args.into_iter().flat_map(|it| {
                // if the arg is a file, we can't skip anything.
                // if its a dir, we probably want to skip the dir itself.
                let path = PathBuf::from(it);
                if path.is_file() {
                    walkdir::WalkDir::new(path)
                        .max_depth(2)
                        .contents_first(false).into_iter()
                    .collect::<Vec<_>>()
                } else {
                    walkdir::WalkDir::new(path)
                        .max_depth(2)
                        .contents_first(false)
                        .into_iter()
                        .skip(1).collect::<Vec<_>>()
                }
                .into_iter()
                .filter_map(|it| it.ok())
                .filter(|it| {
                  ccfkb_lib::log::info!("{}", it.path().display());
                  let dirness_ok = $dirness.matches(it.file_type());
                  dirness_ok
                    && (str::is_empty($type) || ccfkb_lib::util::ends_with_ignore_case(&it.file_name().to_string_lossy(), &$type))
                })
                .map(|it| PathBuf::from_path_buf(it.into_path()).unwrap())
            });

            files
         }
     };
 }

#[macro_export]
macro_rules! main_preamble {
    (dir $type: expr) => {
        $crate::main_preamble_inner!($type, Dirness::Dir)
    };
    (file $type: expr) => {
        $crate::main_preamble_inner!($type, Dirness::File)
    };
    ($type: expr) => {
        $crate::main_preamble_inner!($type, Dirness::Any)
    };
}
