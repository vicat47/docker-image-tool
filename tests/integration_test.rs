use docker_image_tool::tool::trim;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_trim_image_with_real_files() {
    // 跳过测试，因为需要实际文件，但我们可以使用测试目录中的文件
    // 此测试仅在有测试文件时运行
    let tar_path = "test/example.tar";
    let json_path = "test/info.json";
    if !std::path::Path::new(tar_path).exists() || !std::path::Path::new(json_path).exists() {
        println!("测试文件不存在，跳过集成测试");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("trimmed.tar");
    let output_str = output_path.to_str().unwrap();

    // 运行裁剪
    let result = trim::trim_image(tar_path, json_path, Some(output_str));
    assert!(result.is_ok());

    // 验证输出文件存在且非空
    assert!(output_path.exists());
    let metadata = fs::metadata(&output_path).unwrap();
    assert!(metadata.len() > 0);

    // 原始文件大小应大于裁剪后文件大小（或相等）
    let original_size = fs::metadata(tar_path).unwrap().len();
    let trimmed_size = metadata.len();
    assert!(trimmed_size <= original_size);
}

#[test]
fn test_trim_image_default_output() {
    let tar_path = "test/example.tar";
    let json_path = "test/info.json";
    if !std::path::Path::new(tar_path).exists() || !std::path::Path::new(json_path).exists() {
        println!("测试文件不存在，跳过集成测试");
        return;
    }

    // 在临时目录中复制文件以避免污染原始文件
    let temp_dir = TempDir::new().unwrap();
    let temp_tar = temp_dir.path().join("image.tar");
    let temp_json = temp_dir.path().join("layers.json");
    fs::copy(tar_path, &temp_tar).unwrap();
    fs::copy(json_path, &temp_json).unwrap();

    let tar_str = temp_tar.to_str().unwrap();
    let json_str = temp_json.to_str().unwrap();

    // 不指定输出路径，让函数生成默认输出
    let result = trim::trim_image(tar_str, json_str, None);
    assert!(result.is_ok());

    // 检查默认输出文件是否存在（同级目录中的 image_trimmed.tar）
    let expected_output = temp_dir.path().join("image_trimmed.tar");
    assert!(expected_output.exists());
}