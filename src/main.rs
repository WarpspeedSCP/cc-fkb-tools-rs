mod util;
mod data;
mod opcodes;

use serde_derive::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  env::current_dir,
  fs::create_dir_all,
  path::{Path, PathBuf},
  process::exit,
};
use serde_json_fmt::JsonFormat;
use util::{get_sjis_bytes, get_sjis_bytes_of_length, transmute_to_u32};
use walkdir::WalkDir;
use data::{BMP_Dib_V3_Header, BMP_Header, ExtensionDescriptor, FileDescriptor, WIPFHeader, WIPFENTRY};
use crate::data::{extract_all_arcs, read_arc, write_arc};
use crate::opcodes::{make_opcode, OpField, Opcode, Script};
use crate::util::encode_sjis;

fn main() {
  env_logger::builder()
    .format_timestamp(None)
    .format_level(true)
    .filter_level(log::LevelFilter::Info)
    .init();
  let files = walkdir::WalkDir::new(current_dir().unwrap().join("output/Rio.arc"))
      .into_iter()
      .filter_map(|it| it.ok())
      .map(|it| it.into_path())
      .collect::<Vec<_>>();
  for file in files {
    if !file.extension().unwrap().to_string_lossy().ends_with("WSC") {
      continue;
    }
    log::info!("Decoding file {}", file.file_name().unwrap().to_string_lossy());
    let input = std::fs::read(file.clone()).unwrap();

    let mut ptr = 0;
    let mut ptr_old = 1;
    let mut opcodes = vec![];
    let mut at_end = false;

    while ptr < input.len() {
      if ptr_old == ptr {
        break;
      }

      let op = make_opcode(&input[ptr..], ptr);
      if let Some(op) = op {
        log::info!("Got 0x{:02X} of length 0x{:02X} at 0x{:08X}", op.opcode, op.size(), ptr);
        at_end = op.opcode == 0xFF;
        ptr += op.size();
        opcodes.push(op);
      } else {
        ptr_old = ptr;
        log::error!("Unknown opcode at 0x{:08X}", ptr);
      }
    }
    
    let rest = input[ptr..].to_vec();
    
    let out = Script {
      opcodes,
      trailer: rest
    };

    let res = serde_yml::to_string(&out).unwrap()    .replace("'[", "[")
        .replace("]'", "]")
        .replace(r#"'""#, "")
        .replace(r#""'"#, "");

    let new = serde_yml::from_str::<Vec<Opcode>>(&res);

    std::fs::write(file.with_extension("WSC.yaml"), res).unwrap();

  }

  return;

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
    
  }
}

fn get_final_file_list<T>(input_files: Vec<T>, recurse_dirs: bool, action: &str) -> Vec<PathBuf>
where
  T: AsRef<Path> + Sync + Send,
{
  let filter_func = |it: &PathBuf| {
    let value = it.extension().unwrap_or_default();
    if it.file_stem().unwrap_or_default() == "directory" {
      return false;
    }
    if action == "decode" {
      value == "opcodescript" || value == "yaml"
    } else if action == "transform" {
      value == "yaml"
    } else {
      true
    }
  };

  use rayon::prelude::*;
  let output = if !recurse_dirs {
    input_files
      .par_iter()
      .map(|it| it.as_ref().to_path_buf())
      .filter(filter_func)
      .collect()
  } else {
    input_files
      .par_iter()
      .flat_map(|file| {
        WalkDir::new(file.as_ref())
          .contents_first(true)
          .into_iter()
          .filter_map(Result::ok)
          .filter(|it| it.path().is_file())
          .map(|it| it.into_path())
          .filter(filter_func)
          .collect::<Vec<_>>()
      })
      .collect()
  };
  
  output
}
