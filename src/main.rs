use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use clap::Parser;
use serde_json::Value;
use tar::{Archive, Builder};
use crate::cli::command::ToolArgs;

mod cli;

fn main() -> anyhow::Result<()>  {
    println!("Hello, world!");
    let args = ToolArgs::parse();

    println!("args: {:?}", args);
    struct FileStruct {
        image_path: String,
        json_path: String,
    }

    let mut file_struct = FileStruct { image_path: "".to_string(), json_path: "".to_string() };
    for file in args.file_path {
        if file.ends_with(".tar") {
            file_struct.image_path = file
        } else if file.ends_with(".json") {
            file_struct.json_path = file
        }
    }
    let file_struct = file_struct;

    println!("开始处理镜像...");

    let mut result = File::open(file_struct.json_path)?;
    let mut json_content = String::new();
    result.read_to_string(&mut json_content)?;
    let json_content = json_content.trim();
    let json: Value = serde_json::from_str(json_content)?;
    let layers_already_exist: &Value = &json[0]["RootFS"]["Layers"];
    let layers_already_exist: HashSet<String> = layers_already_exist.as_array().unwrap().iter()
        .map(|item| item.to_string().trim().replace("\"", "").replace("sha256:", ""))
        .collect();

    let mut tar_data = File::open(&file_struct.image_path)?;

    let mut archive= Archive::new(&tar_data);
    let mut layers_in_image: HashSet<String> = HashSet::new();
    for entry in archive.entries_with_seek()? {
        let blob_path = entry?.path()?.display().to_string();
        if blob_path.starts_with("blobs/sha256/") {
            let blob_name = blob_path.replace("blobs/sha256/", "").trim().to_string();
            if !blob_name.is_empty() {
                layers_in_image.insert(blob_name);
            }
        }
    }

    println!("已成功读取镜像信息");

    let layers_in_image = layers_in_image;

    let difference = layers_in_image.difference(&layers_already_exist).map(|i| i.clone()).collect::<HashSet<_>>();

    println!("正在裁剪镜像");
    let out_file = File::create(&file_struct.image_path.replace(".tar", "_trimmed.tar"))?;
    let mut tar_builder = Builder::new(out_file);
    tar_data.seek(SeekFrom::Start(0))?;
    let mut archive = Archive::new(tar_data);
    for entry_result in archive.entries()? {
        let entry = entry_result?;
        let file_path = entry.path()?.display().to_string();
        if file_path.starts_with("blobs/sha256/") && !file_path.trim().is_empty() {
            let layer_sha256 = file_path.replace("blobs/sha256/", "").trim().to_string();
            if !difference.contains(&layer_sha256) {
                continue;
            }
        }
        tar_builder.append(&entry.header().clone(), entry)?;
    }

    // println!("tar包镜像：{:?}", layers_in_image);
    // println!("现场镜像: {:?}", layers_already_exist);
    // println!("difference: {:?}", difference);
    tar_builder.finish()?;
    println!("镜像已完成裁剪，默认生成在与原镜像相同目录...");
    println!("请按回车继续");
    std::io::stdin().read_line(&mut String::new())?;

    Ok(())
}
