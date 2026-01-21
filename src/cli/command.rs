//! 命令行参数解析模块。
//!
//! 定义 `ToolArgs` 结构体，用于解析用户输入的文件路径和输出目录。

use std::path::PathBuf;
use clap::Parser;
use clap::ArgAction;

/// 命令行参数结构体。
///
/// 该结构体通过 `clap` 派生宏自动实现参数解析。
/// 程序期望用户提供至少两个文件：一个 Docker 镜像 tar 包和一个 JSON 层列表文件。
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct ToolArgs {
    /// 输入文件路径列表。
    ///
    /// 程序会自动识别扩展名为 `.tar` 的镜像文件和 `.json` 的层列表文件。
    /// 支持拖拽操作，顺序无关。
    #[arg(required = true, value_name = "FILE", action = ArgAction::Append)]
    pub(crate) file_path: Vec<String>,

    /// 输出目录（可选）。
    ///
    /// 如果指定，裁剪后的镜像将保存到该目录；
    /// 否则，输出文件将生成在输入镜像的同级目录。
    #[arg(short, long, value_name = "DIR")]
    output: Option<PathBuf>,
}