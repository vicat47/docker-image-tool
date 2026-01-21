use clap::Parser;
use tool::trim;
use crate::cli::command::ToolArgs;

mod cli;
mod tool;

fn main() -> anyhow::Result<()> {
    println!("Docker 镜像裁剪工具启动...");
    let args = ToolArgs::parse();

    // 从文件路径列表中识别镜像文件和 JSON 文件
    let (image_path, json_path) = identify_files(&args.file_path)?;

    // 确定输出路径
    let output_path = args.output.as_ref().map(|p| p.to_string_lossy().into_owned());

    // 执行裁剪
    trim::trim_image(&image_path, &json_path, output_path.as_deref())?;

    // 等待用户按回车（可选）
    println!("裁剪完成，按回车退出...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}

pub(crate) fn identify_files(files: &[String]) -> anyhow::Result<(String, String)> {
    let mut image_path = None;
    let mut json_path = None;

    for file in files {
        if file.ends_with(".tar") {
            if image_path.is_some() {
                anyhow::bail!("发现多个 .tar 文件，请只提供一个镜像文件");
            }
            image_path = Some(file.clone());
        } else if file.ends_with(".json") {
            if json_path.is_some() {
                anyhow::bail!("发现多个 .json 文件，请只提供一个层列表文件");
            }
            json_path = Some(file.clone());
        }
    }

    match (image_path, json_path) {
        (Some(img), Some(json)) => Ok((img, json)),
        (None, None) => anyhow::bail!("未提供 .tar 或 .json 文件"),
        (None, _) => anyhow::bail!("未提供 .tar 镜像文件"),
        (_, None) => anyhow::bail!("未提供 .json 层列表文件"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_files_success() {
        let files = vec!["image.tar".to_string(), "layers.json".to_string()];
        let (img, json) = identify_files(&files).unwrap();
        assert_eq!(img, "image.tar");
        assert_eq!(json, "layers.json");
    }

    #[test]
    fn test_identify_files_extra_files() {
        let files = vec![
            "image.tar".to_string(),
            "layers.json".to_string(),
            "readme.txt".to_string(),
        ];
        let (img, json) = identify_files(&files).unwrap();
        assert_eq!(img, "image.tar");
        assert_eq!(json, "layers.json");
    }

    #[test]
    fn test_identify_files_multiple_tar() {
        let files = vec!["image1.tar".to_string(), "image2.tar".to_string(), "layers.json".to_string()];
        let result = identify_files(&files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("多个 .tar 文件"));
    }

    #[test]
    fn test_identify_files_multiple_json() {
        let files = vec!["image.tar".to_string(), "layers1.json".to_string(), "layers2.json".to_string()];
        let result = identify_files(&files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("多个 .json 文件"));
    }

    #[test]
    fn test_identify_files_no_tar() {
        let files = vec!["layers.json".to_string()];
        let result = identify_files(&files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未提供 .tar 镜像文件"));
    }

    #[test]
    fn test_identify_files_no_json() {
        let files = vec!["image.tar".to_string()];
        let result = identify_files(&files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未提供 .json 层列表文件"));
    }

    #[test]
    fn test_identify_files_empty() {
        let files: Vec<String> = vec![];
        let result = identify_files(&files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未提供 .tar 或 .json 文件"));
    }
}
