//! 文件 I/O 操作模块。
//!
//! 提供读取 JSON 层列表、读取 tar 镜像层列表以及写入裁剪后 tar 的功能。

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use anyhow::Result;
use serde_json::Value;
use tar::{Archive, Builder};

use crate::tool::layers::LayerSet;

/// 从 JSON 文件读取层列表。
///
/// # 参数
/// - `path`: JSON 文件路径。
///
/// # 返回
/// - `Ok(LayerSet)`：成功解析的层集合。
/// - `Err(e)`：文件读取或解析错误。
pub fn read_json_layers(path: &str) -> Result<LayerSet> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let content = content.trim();
    let json: Value = serde_json::from_str(content)?;
    let layers_array = &json[0]["RootFS"]["Layers"];
    let layers: HashSet<String> = layers_array
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("JSON 中 Layers 字段不是数组"))?
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| anyhow::anyhow!("层项不是字符串"))
                .map(|s| s.replace("sha256:", ""))
        })
        .collect::<Result<HashSet<_>, _>>()?;
    Ok(layers)
}

/// 从 tar 镜像文件中读取层列表。
///
/// # 参数
/// - `path`: tar 文件路径。
///
/// # 返回
/// - `Ok(LayerSet)`：镜像中包含的所有层（blob）的集合。
/// - `Err(e)`：文件打开或 tar 解析错误。
pub fn read_tar_layers(path: &str) -> Result<LayerSet> {
    let file = File::open(path)?;
    let mut archive = Archive::new(file);
    let mut layers = HashSet::new();

    for entry_result in archive.entries_with_seek()? {
        let entry = entry_result?;
        let blob_path = entry.path()?.display().to_string();
        if blob_path.starts_with("blobs/sha256/") {
            let blob_name = blob_path.replace("blobs/sha256/", "").trim().to_string();
            if !blob_name.is_empty() {
                layers.insert(blob_name);
            }
        }
    }

    Ok(layers)
}

/// 将裁剪后的镜像写入新的 tar 文件。
///
/// # 参数
/// - `input_path`: 原始 tar 文件路径。
/// - `output_path`: 输出 tar 文件路径。
/// - `keep_layers`: 需要保留的层集合。
///
/// # 返回
/// - `Ok(())`：写入成功。
/// - `Err(e)`：任何 I/O 错误。
pub fn write_trimmed_tar(
    input_path: &str,
    output_path: &str,
    keep_layers: &LayerSet,
) -> Result<()> {
    let mut input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    let mut tar_builder = Builder::new(output_file);

    // 重置文件指针
    input_file.seek(SeekFrom::Start(0))?;
    let mut archive = Archive::new(input_file);

    for entry_result in archive.entries()? {
        let entry = entry_result?;
        let file_path = entry.path()?.display().to_string();
        if file_path.starts_with("blobs/sha256/") && !file_path.trim().is_empty() {
            let layer_sha256 = file_path.replace("blobs/sha256/", "").trim().to_string();
            if !keep_layers.contains(&layer_sha256) {
                continue;
            }
        }
        tar_builder.append(&entry.header().clone(), entry)?;
    }

    tar_builder.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_json_layers_valid() {
        let json_content = r#"
        [
            {
                "RootFS": {
                    "Type": "layers",
                    "Layers": [
                        "sha256:abc123",
                        "sha256:def456"
                    ]
                }
            }
        ]
        "#;
        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), json_content).unwrap();
        let layers = read_json_layers(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(layers.len(), 2);
        assert!(layers.contains("abc123"));
        assert!(layers.contains("def456"));
    }

    #[test]
    fn test_read_json_layers_missing_layers() {
        let json_content = r#"[{}]"#;
        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), json_content).unwrap();
        let result = read_json_layers(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_read_json_layers_invalid_json() {
        let json_content = r#"not json"#;
        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), json_content).unwrap();
        let result = read_json_layers(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    // 注意：read_tar_layers 和 write_trimmed_tar 的测试需要实际的 tar 文件，
    // 这里暂时省略，将在集成测试中覆盖。
}