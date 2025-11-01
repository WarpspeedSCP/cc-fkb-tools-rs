pub mod util;
pub mod data;
pub mod opcodes;

use std::{
  env::current_dir,
  fs::create_dir_all,
  path::{Path, PathBuf},
  process::exit,
};
use util::{get_sjis_bytes, get_sjis_bytes_of_length, transmute_to_u32};
use walkdir::WalkDir;
use ccfkb_lib::data::text_script::tl_transform_script;

use crate::opcodes::Script;

fn maain() {


  let args = std::env::args().collect::<Vec<_>>();

  if args.contains(&"extract_all".to_string()) {
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

  } else if args.contains(&"encode".to_string()) {
    if args.len() < 5 {
      log::error!(
        "argument order is: ccfkb encode <yaml file> <-o or --output> <output directory name>"
      );
      exit(1);
    }

    let yaml_name = &args[2];

    let wsc_name_path = current_dir()
      .unwrap()
      .join(yaml_name)
      .canonicalize()
      .map_err(|err| log::error!("The input file {yaml_name} does not exist! error: {err}"))
      .unwrap();

    if args[3] != "--output" && args[3] != "-o" {
      log::error!("argument order is: ccfkb encode <yaml file> <-o or --output> <output directory name>");
      exit(1);
    }

    let out_dir = &args[4];

    let out_dir_path = current_dir().unwrap().join(out_dir);
    create_dir_all(&out_dir_path).unwrap();

  } else if args.contains(&"encode-all".to_string()) {
    if args.len() < 5 {
      log::error!(
        "argument order is: ccfkb encode-all <yaml directory name> <-o or --output> <output directory name>"
      );
      exit(1);
    }

    let yaml_name = &args[2];

    let yaml_name_path = current_dir()
      .unwrap()
      .join(yaml_name)
      .canonicalize()
      .map_err(|err| log::error!("The input directory {yaml_name} does not exist! error: {err}"))
      .unwrap();

    if args[3] != "--output" && args[3] != "-o" {
      log::error!("argument order is: ccfkb encode-all <yaml directory name> <-o or --output> <output directory name>");
      exit(1);
    }

    let out_dir = &args[4];

    let out_dir_path = current_dir().unwrap().join(out_dir);
    create_dir_all(&out_dir_path).unwrap();

    let files = walkdir::WalkDir::new(&yaml_name_path)
      .into_iter()
      .filter_map(|it| it.ok())
      .map(|it| it.into_path())
      .collect::<Vec<_>>();

    for file in files {
      if !file.extension().unwrap().to_string_lossy().ends_with("yaml") {
        continue;
      }
    }

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


  } else if args.contains(&"untransform".to_string()){
    if args.len() < 4 {
      log::error!(
        "argument order is: ccfkb untransform <yaml file> <script txt file>"
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

    let script_name = &args[3];

    let script_name_path = current_dir().unwrap().join(script_name).canonicalize().unwrap();


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


    }

  }
}
