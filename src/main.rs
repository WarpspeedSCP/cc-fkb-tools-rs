mod util;
mod data;
mod opcodes;

use std::{
  env::current_dir,
  fs::create_dir_all,
  path::{Path, PathBuf},
  process::exit,
};
use util::{get_sjis_bytes, get_sjis_bytes_of_length, transmute_to_u32};
use walkdir::WalkDir;
use crate::data::{decode_wsc, extract_all_arcs, read_arc, tl_transform_script, write_arc};

fn main() {
  env_logger::builder()
    .format_timestamp(None)
    .format_level(true)
    .filter_level(log::LevelFilter::Info)
    .init();

  let args = std::env::args().collect::<Vec<_>>();

  if args.contains(&"extract_all".to_string()) {
    extract_all_arcs();
  } else if args.contains(&"extract".to_string()) {
    if args.len() < 5 {
      log::error!(
        "argument order is: ccfkb extract <arc file> <-o or --output> <output folder name>"
      );
      exit(1);
    }

    let arc_name = &args[2];

    let arc_name_path = current_dir()
      .unwrap()
      .join(arc_name)
      .canonicalize()
      .unwrap();

    if args[3] != "--output" && args[3] != "-o" {
      log::error!("argument order is: ccfkb extract <arc file> -o/--output <output folder name>");
      exit(1);
    }

    let out_dir = &args[4];

    let out_dir_path = current_dir().unwrap().join(out_dir).canonicalize().unwrap();

    create_dir_all(&out_dir_path).unwrap();

    let mut file = std::fs::read(&arc_name_path).unwrap();

    let path = out_dir_path.join(&arc_name_path.file_name().unwrap());
    std::fs::create_dir_all(&path).unwrap();
    read_arc(&mut file[..], path, false);
  } else if args.contains(&"repack".to_string()) {
    if args.len() < 5 {
      log::error!("argument order is: ccfkb repack <arc folder name> -o <output file name>");
      exit(1);
    }

    let arc_name = &args[2];

    let arc_name_path = current_dir()
      .unwrap()
      .join(arc_name)
      .canonicalize()
      .unwrap();

    if args[3] != "--output" && args[3] != "-o" {
      log::error!("argument order is: ccfkb repack <arc folder name> -o <output file name>");
      exit(1);
    }

    let out_file = &args[4];

    let output = current_dir().unwrap().join(out_file);

    let data = write_arc(&arc_name_path);
    std::fs::write(output, data).unwrap();
  } else if args.contains(&"decode".to_string()) {
    if args.len() < 5 {
      log::error!(
        "argument order is: ccfkb decode <wsc file> <-o or --output> <output directory name>"
      );
      exit(1);
    }

    let wsc_name = &args[2];

    let wsc_name_path = current_dir()
        .unwrap()
        .join(wsc_name)
        .canonicalize()
        .map_err(|err| log::error!("The input file {wsc_name} does not exist! error: {err}"))
        .unwrap();

    if args[3] != "--output" && args[3] != "-o" {
      log::error!("argument order is: ccfkb decode <wsc file> <-o or --output> <output directory name>");
      exit(1);
    }

    let out_dir = &args[4];

    let out_dir_path = current_dir().unwrap().join(out_dir);
    create_dir_all(&out_dir_path).unwrap();

    decode_wsc_file_command(&wsc_name_path, &out_dir_path);
  } else if args.contains(&"decode-all".to_string()) {
    if args.len() < 5 {
      log::error!(
        "argument order is: ccfkb decode-all <wsc directory name> <-o or --output> <output directory name>"
      );
      exit(1);
    }

    let wsc_name = &args[2];

    let wsc_name_path = current_dir()
        .unwrap()
        .join(wsc_name)
        .canonicalize()
        .map_err(|err| log::error!("The input directory {wsc_name} does not exist! error: {err}"))
        .unwrap();

    if args[3] != "--output" && args[3] != "-o" {
      log::error!("argument order is: ccfkb decode-all <wsc directory name> <-o or --output> <output directory name>");
      exit(1);
    }

    let out_dir = &args[4];
    
    let out_dir_path = current_dir().unwrap().join(out_dir);
    create_dir_all(&out_dir_path).unwrap();

    let files = walkdir::WalkDir::new(&wsc_name_path)
        .into_iter()
        .filter_map(|it| it.ok())
        .map(|it| it.into_path())
        .collect::<Vec<_>>();
    for file in files {
      if !file.extension().unwrap().to_string_lossy().ends_with("WSC") {
        continue;
      }
      
      decode_wsc_file_command(&file, &out_dir_path)

    }
  } else if args.contains(&"transform".to_string()){
    if args.len() < 5 {
      log::error!(
        "argument order is: ccfkb transform <yaml file> <-o or --output> <output directory name>"
      );
      exit(1);
    }

    let wsc_name = &args[2];

    let wsc_name_path = current_dir()
        .unwrap()
        .join(wsc_name)
        .canonicalize()
        .map_err(|err| log::error!("The input file {wsc_name} does not exist! error: {err}"))
        .unwrap();

    if args[3] != "--output" && args[3] != "-o" {
      log::error!("argument order is: ccfkb transform <yaml file> <-o or --output> <output directory name>");
      exit(1);
    }

    let out_dir = &args[4];

    let out_dir_path = current_dir().unwrap().join(out_dir);
    create_dir_all(&out_dir_path).unwrap();

    transform_wsc_file_command(&wsc_name_path, &out_dir_path.canonicalize().unwrap());
    
  } else if args.contains(&"transform-all".to_string()){
    if args.len() < 5 {
      log::error!(
        "argument order is: ccfkb transform-all <yaml file> <-o or --output> <output directory name>"
      );
      exit(1);
    }

    let wsc_name = &args[2];

    let wsc_name_path = current_dir()
        .unwrap()
        .join(wsc_name)
        .canonicalize()
        .map_err(|err| log::error!("The input directory {wsc_name} does not exist! error: {err}"))
        .unwrap();

    if args[3] != "--output" && args[3] != "-o" {
      log::error!("argument order is: ccfkb transform-all <yaml directory> <-o or --output> <output directory name>");
      exit(1);
    }

    let out_dir = &args[4];

    let out_dir_path = current_dir().unwrap().join(out_dir);
    create_dir_all(&out_dir_path).unwrap();

    let files = walkdir::WalkDir::new(&wsc_name_path)
        .into_iter()
        .filter_map(|it| it.ok())
        .map(|it| it.into_path())
        .collect::<Vec<_>>();
    for file in files {
      if !file.extension().unwrap().to_string_lossy().ends_with("yaml") {
        continue;
      }

      transform_wsc_file_command(&file, &out_dir_path);

    }

  }
}

fn decode_wsc_file_command(wsc_name_path: &Path, out_dir_path: &Path) {
  log::info!("Decoding file {}", wsc_name_path.file_name().unwrap_or_default().to_string_lossy());
  let input = std::fs::read(wsc_name_path).unwrap();

  let out = decode_wsc(&input);

  let res = serde_yml::to_string(&out)
      .unwrap()
      .replace("'[", "[")
      .replace("]'", "]")
      .replace(r#"'""#, "")
      .replace(r#""'"#, "");

  let output_file = out_dir_path.join(wsc_name_path.file_name().unwrap()).with_extension("WSC.yaml");
  std::fs::write(output_file, res).unwrap();
}


fn transform_wsc_file_command(wsc_name_path: &Path, out_dir_path: &Path) {
  log::info!("Transforming file {}", wsc_name_path.file_name().unwrap_or_default().to_string_lossy());
  let input = std::fs::read_to_string(wsc_name_path).unwrap();
  
  let script = serde_yml::from_str(&input).unwrap();
  
  let out = tl_transform_script(&script);
  
  let out_path = out_dir_path.join(wsc_name_path
      .file_name()
      .unwrap()
      .to_string_lossy()
      .replace("yaml", "txt")
  );
  std::fs::write(out_path, out).unwrap();
}
