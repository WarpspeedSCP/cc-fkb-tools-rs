use anyhow::{anyhow, Context};
use log::{Level, Metadata, Record};
use std::env;
use std::io::Write;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

enum LogOutput {
	None,
	Stdout,
	Stderr,
	File(Arc<Mutex<std::io::BufWriter<std::fs::File>>>),
}

struct SimpleLogger {
	level: Level,
	log_filenames: bool,
	output_buffers: [LogOutput; 6],
}

impl log::Log for SimpleLogger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= self.level
	}

	fn log(&self, record: &Record) {
		if self.enabled(record.metadata()) {
			let output = &self.output_buffers[record.level() as usize];
			let file_str = if self.log_filenames {
				format!("({}:{})", record.file().unwrap_or("???"), record.line().unwrap_or(0))
			} else {
				"".to_owned()
			};
			match output {
				LogOutput::None => {}
				LogOutput::Stderr => eprintln!("[{}] {} {file_str}", record.level(), record.args()),
				LogOutput::Stdout => println!("[{}] {} {file_str}", record.level(), record.args()),
				LogOutput::File(mutex) => {
					let res = mutex.lock().map_err(|_err| "Could not lock log file!").and_then(|ref mut w| {
						writeln!(w, "[{}] {} {file_str}", record.level(), record.args()).map_err(|_err| "Could not write to log file!")
					});

					if let Err(e) = res {
						eprintln!("[{}] {file_str} {e}", Level::Error);
						eprintln!("Original log message: {}", record.args());
					}
				}

			}
		}
	}

	fn flush(&self) {
		// `log::Log::flush` cannot report a failure, and panicking here would abort a program that
		// may already be reporting a failure of its own, so a broken sink goes to stderr instead.
		for output in self.output_buffers.iter() {
			let res = match output {
				LogOutput::None => Ok(()),
				LogOutput::Stdout => std::io::stdout().flush(),
				LogOutput::Stderr => std::io::stderr().flush(),
				LogOutput::File(output) => output
					.lock()
					.map_err(|_err| std::io::Error::other("could not lock the log file"))
					.and_then(|mut it| it.flush()),
			};

			if let Err(err) = res {
				eprintln!("[{}] could not flush the log output: {err}", Level::Error);
			}
		}
	}
}

impl SimpleLogger {
	pub fn from_env() -> anyhow::Result<Box<Self>> {
		let matching_level = match env::var("RUST_LOG") {
			Ok(value) => log::Level::from_str(&value)
				.map_err(|_err| anyhow!("RUST_LOG has unknown log level {value:?}"))?,
			Err(_) => Level::Info,
		};

		let log_output_str = env::var("LOG_OUTPUT")
			.map(|it| it.to_lowercase())
			.unwrap_or_default();

		let mut output_buffers = [
			LogOutput::None,                     // Log levels are one indexed.
			LogOutput::Stderr,
			LogOutput::Stderr,
			LogOutput::Stdout,
			LogOutput::None,
			LogOutput::None,
		];

		if log_output_str.is_empty() {
			for (idx, output) in output_buffers.iter_mut().enumerate() {
				if idx <= matching_level as usize {
					let _ = std::mem::replace(output, LogOutput::Stderr);
				}
			}
		} else {
			for (k, v) in log_output_str.split(';').map(|it| {
				let mut data = it.split('=');
				let key = data.next().unwrap_or_default();
				let value = data.next().unwrap_or_default();
				(key, value)
			}) {
				if k.is_empty() || v.is_empty() {
					// TODO: Log an error here.
					continue;
				}
				let log_level = log::Level::from_str(k)
					.map_err(|_err| anyhow!("LOG_OUTPUT has unknown log level {k:?}"))? as usize;

				let configured_log_level = match v {
					"off" => LogOutput::None,
					"stderr" => LogOutput::Stderr,
					"stdout" => LogOutput::Stdout,
					_ => {
						let path = std::path::PathBuf::from(v);
						let file = std::fs::File::create(&path)
							.with_context(|| format!("could not open the log file {}", path.display()))?;
						LogOutput::File(Arc::new(Mutex::new(std::io::BufWriter::new(file))))
					}
				};

				output_buffers[log_level] = configured_log_level;
			}
		}

		Ok(Box::new(SimpleLogger {
			level: matching_level,
			log_filenames: false,
			output_buffers
		}))
	}
}

pub fn init() -> anyhow::Result<()> {
	let logger = SimpleLogger::from_env()?;
	let level_filter = logger.level.to_level_filter();
	log::set_boxed_logger(logger)
		.map(|()| log::set_max_level(level_filter))
		.map_err(|_| anyhow!("logger already initialised"))
}
