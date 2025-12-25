pub mod data;
pub mod opcodes;
pub mod util;
pub mod bin_utils;

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
            use env_logger::Env;
            use ccfkb_lib::walkdir;
            let env = Env::new().default_filter_or("info");

            let target_buffer: env_logger::Target = {
              #[cfg(not(target_os="windows"))]
              {
                if let Ok(log_file_name) = std::env::var("LOG_FILE") {
                  env_logger::Target::Pipe(Box::from(std::fs::File::create(log_file_name).unwrap()))
                } else {
                  env_logger::Target::Stderr
                }
              }
              #[cfg(target_os="windows")]
              unsafe {
                use windows;
                let console_win = windows::Win32::System::Console::GetConsoleWindow();
                let res = windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(console_win, None);

                if let Ok(log_file_name) = std::env::var("LOG_FILE") {
                  env_logger::Target::Pipe(Box::from(std::fs::File::create(log_file_name).unwrap()))
                } else if windows::Win32::System::Threading::GetCurrentProcessId() != res {
                  env_logger::Target::Pipe(Box::new(std::fs::File::create(env!("CARGO_CRATE_NAME").to_string() + ".log").unwrap()))
                } else {
                  env_logger::Target::Stderr
                }
              }
            };

            env_logger::builder()
                .format_timestamp(None)
                .target(target_buffer)
                .format_level(true)
                .parse_env(env)
                .init();
            
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
    
    ($dirs: ident) => {
         {
            use camino::Utf8PathBuf as PathBuf;
            use env_logger::Env;
            use ccfkb_lib::walkdir;
            let env = Env::new().default_filter_or("info");
            let target_buffer: env_logger::Target = {
              #[cfg(not(target_os="windows"))]
              {
                if let Ok(log_file_name) = std::env::var("LOG_FILE") {
                  env_logger::Target::Pipe(Box::from(std::fs::File::create(log_file_name).unwrap()))
                } else {
                  env_logger::Target::Stderr
                }
              }
              #[cfg(target_os="windows")]
              unsafe {
                use windows;
                let console_win = windows::Win32::System::Console::GetConsoleWindow();
                let res = windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(console_win, None);

                if let Ok(log_file_name) = std::env::var("LOG_FILE") {
                  env_logger::Target::Pipe(Box::from(std::fs::File::create(log_file_name).unwrap()))
                } else if windows::Win32::System::Threading::GetCurrentProcessId() != res {
                  env_logger::Target::Pipe(Box::new(std::fs::File::open(env!("CARGO_CRATE_NAME").to_string() + ".log").unwrap()))
                } else {
                  env_logger::Target::Stderr
                }
              }
            };

            env_logger::builder()
                .format_timestamp(None)
                .target(target_buffer)
                .format_level(true)
                .parse_env(env)
                .init();

            let args = std::env::args().skip(1).collect::<Vec<_>>();
            
            let dirs = args.into_iter().flat_map(|it| {
                walkdir::WalkDir::new(it)
                    .into_iter()
                    .filter_map(|it| it.ok())
                    .filter(|it| it.file_type().is_dir())
                    .map(|it| PathBuf::from_path_buf(it.into_path()).unwrap())
            });
            
            dirs
         }
    }
 }