pub mod data;
pub mod opcodes;
pub mod util;

pub use walkdir;
pub use once_cell;
pub use itertools;
pub use log;
pub use rayon;

#[macro_export]
macro_rules! main {
     ($files: ident, $type: literal, $tokens: block) => {
         fn main() {
            use env_logger::Env;
            use ccfkb_lib::walkdir;
            let env = Env::new().default_filter_or("info");
            env_logger::builder()
                .format_timestamp(None)
                .format_level(true)
                .parse_env(env)
                .init();
            
            let args = std::env::args().skip(1).collect::<Vec<_>>();
            
            let $files = args.into_iter().flat_map(|it| {
                walkdir::WalkDir::new(it)
                    .into_iter()
                    .filter_map(|it| it.ok())
                    .filter(|it| it.file_type().is_file() && (str::is_empty($type) || it.file_name().to_string_lossy().ends_with($type)))
                    .map(|it| it.into_path())
            });
            
            $tokens;
         }
     };
    
    ($dirs: ident, $tokens: block) => {
         fn main() {
            use env_logger::Env;
            use ccfkb_lib::walkdir;
            let env = Env::new().default_filter_or("info");
            env_logger::builder()
                .format_timestamp(None)
                .format_level(true)
                .parse_env(env)
                .init();
            
            let args = std::env::args().skip(1).collect::<Vec<_>>();
            
            let $dirs = args.into_iter().flat_map(|it| {
                walkdir::WalkDir::new(it)
                    .into_iter()
                    .filter_map(|it| it.ok())
                    .filter(|it| it.file_type().is_dir())
            });
            
            $tokens;
         }
    }
 }