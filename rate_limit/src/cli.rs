use clap::Parser;
use std::path::PathBuf;

/// 命令行参数解析
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Your Name")]
pub struct CliArgs {
    /// 目标文件路径
    #[clap(short, long)]
    pub file: PathBuf,

    /// 速率限制 (单位 KB/s)
    #[clap(short, long, default_value = "1024")]
    pub rate: u32,

    /// 输出文件路径 (可选)
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}
