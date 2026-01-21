//! 镜像裁剪核心逻辑模块。
//!
//! 提供裁剪 Docker 镜像的主要函数。

use anyhow::Result;
use crate::tool::io::{read_json_layers, read_tar_layers, write_trimmed_tar};
use crate::tool::layers::difference;

/// 裁剪 Docker 镜像。
///
/// # 参数
/// - `image_path`: Docker 镜像 tar 文件路径。
/// - `json_path`: JSON 层列表文件路径。
/// - `output_path`: 输出裁剪后 tar 文件路径（可选）。
///   如果为 `None`，则默认在输入镜像同级目录生成 `{image_name}_trimmed.tar`。
///
/// # 返回
/// - `Ok(())`：裁剪成功。
/// - `Err(e)`：任何步骤中的错误。
pub fn trim_image(
    image_path: &str,
    json_path: &str,
    output_path: Option<&str>,
) -> Result<()> {
    println!("开始处理镜像...");

    // 读取层列表
    let existing_layers = read_json_layers(json_path)?;
    let image_layers = read_tar_layers(image_path)?;

    println!("已成功读取镜像信息");

    // 计算需要保留的层
    let keep_layers = difference(&image_layers, &existing_layers);

    println!("正在裁剪镜像");

    // 确定输出路径
    let output_storage;
    let output = match output_path {
        Some(path) => path,
        None => {
            output_storage = if image_path.ends_with(".tar") {
                image_path.replace(".tar", "_trimmed.tar")
            } else {
                "trimmed.tar".to_string()
            };
            &output_storage
        }
    };

    // 写入裁剪后的 tar
    write_trimmed_tar(image_path, output, &keep_layers)?;

    println!("镜像已完成裁剪，保存至: {}", output);
    Ok(())
}