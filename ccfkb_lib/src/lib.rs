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
macro_rules! main_preamble {
     ($type: expr) => {
          {
            use camino::Utf8PathBuf as PathBuf;
            use ccfkb_lib::walkdir;
            use ccfkb_lib::logging;

            logging::init().unwrap();

            let args = std::env::args().skip(1).collect::<Vec<_>>();
            
            let files = args.into_iter().flat_map(|it| {
                walkdir::WalkDir::new(it)
                    .into_iter()
                    .filter_map(|it| it.ok())
                    .filter(|it| {
                      ccfkb_lib::log::info!("{}", it.path().display());
                      it.file_type().is_file() && (str::is_empty($type) || ccfkb_lib::util::ends_with_ignore_case(&it.file_name().to_string_lossy(), $type))
                    })
                    .map(|it| PathBuf::from_path_buf(it.into_path()).unwrap())
            });
            
            files
         }
     };
 }


