use std::collections::{BTreeMap, HashMap};
use itertools::Itertools;
use ccfkb_lib::data::{write_arc, ExtensionDescriptor, FileDescriptor};
use ccfkb_lib::main_preamble;
use ccfkb_lib::util::current_dir;

fn main() {
	let files = main_preamble!(&"").collect::<Vec<_>>();

	let output_file_name = files.first().unwrap().parent().unwrap().file_name().unwrap();
	let extensions_yaml_file = files.iter().find(|it| it.ends_with("extensions.yaml") || it.ends_with("extensions.yml")).unwrap();
	let files_yaml_file = files.iter().find(|it| it.ends_with("files.yaml") || it.ends_with("files.yml")).unwrap();

	let exts = files.iter().filter_map(|it| {
		if let Some(ext) = it.extension() && (ext != "yml" || ext != "yaml") {
			Some((ext.to_uppercase(), it))
		} else {
			None
		}
	})
		.sorted_by(|a, b| a.0.cmp(&b.0))
		.chunk_by(|it| it.0.clone())
		.into_iter()
		.fold(BTreeMap::new(), |mut map, (k, v)| {
			let vec = v.fold(vec![] , |mut acc, (_, file)| {
				let size = std::fs::metadata(&file).unwrap().len() as u32;
				let entry = FileDescriptor {
					name: file.file_stem().unwrap().to_uppercase(),
					offset: 0,
					size
				};
				acc.push(entry);
				acc
			});

			let key = ExtensionDescriptor {
				name: k.clone(),
				number: files.len() as u32,
				offset: 0
			};

			map.insert(
				key,
				vec
			);

			map
		});

	let n_exts = exts.len();

	let mut final_out_exts = vec![];
	let mut final_out_files = vec![];
	let mut offset = exts.keys().fold(4, |it, ext| it + ext.size());

	for (ext, files) in exts {
		let mut new_ext = ext.clone();

		final_out_files.extend(files);
		new_ext.offset = offset as u32;
		offset += files.iter().fold(offset, |offset, file| offset + file.size());

	}

	// let (ext_descriptors, file_descriptors, end_offset) = exts.iter().fold((vec![], vec![], 4), |(mut ext_acc, mut file_desc_acc, global_offset), (ext, files)| {
	// 	let (actual_file_descriptors, new_global_offset) = files.iter().fold((Vec::new(), global_offset), |(mut acc, offset), file| {
	// 		acc.push(
	// 			FileDescriptor {
	// 				name: file.name.clone(),
	// 				size: file.size,
	// 				offset,
	// 			}
	// 		);
	// 		(acc, offset + file.size)
	// 	});
	//
	// 	let desc = ExtensionDescriptor {
	// 		name: ext.clone(),
	// 		offset: global_offset as u32,
	// 		number: actual_file_descriptors.len() as u32,
	// 	};
	//
	// 	ext_acc.push(desc);
	// 	file_desc_acc.extend(actual_file_descriptors);
	//
	// 	(ext_acc, file_desc_acc, new_global_offset)
	// });

	println!("{n_exts} extensions. {} extension descriptors collected, {} file descriptors collected, {end_offset}", ext_descriptors.len(), file_descriptors.len());
	// for file in &files {
	// 	let ext = file.extension().unwrap_or_default();
	//
	// 	if ext.ends_with("yaml") {
	// 		continue
	// 	}
	// 	let cap_ext = ext.to_uppercase();
	// 	let name = file.file_stem().unwrap();
	//
	// 	let mut offset = 0;
	// 	if extension_descriptors.contains_key(&cap_ext) {
	// 		let data = &mut extension_descriptors[&cap_ext];
	// 		let new_desc = FileDescriptor {
	// 			name: name.to_owned(),
	// 			offset,
	// 			size: std::fs::metadata(&file).unwrap().len() as usize,
	// 		};
	// 		offset += new_desc.size;
	// 		data.push(new_desc);
	// 	}
	//
	// 	if let Some(item) = file_descriptors.iter_mut().find(|it| it.name == name) {
	//
	// 	}
	//
	// }

	// let ext_descriptors: Vec<ExtensionDescriptor> = serde_yml::from_reader(std::fs::File::open(&extensions_yaml_file).unwrap()).unwrap();
	// let file_descriptors: Vec<FileDescriptor> = serde_yml::from_reader(std::fs::File::open(&files_yaml_file).unwrap()).unwrap();

	// let files = files
	// 	.iter()
	// 	.map(|it| it.as_path())
	// 	.filter(|it| {
	// 		ext_descriptors
	// 			.iter()
	// 			.any(|ext| it.file_name().map(|it| it.contains(&ext.name)).unwrap_or_default())
	// 	})
	// 	.collect::<Vec<_>>();
	//
	// let output = write_arc(&files, ext_descriptors, file_descriptors);
	// std::fs::write(current_dir().join(output_file_name), output).unwrap();
}
