use clap::Parser;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use nonzero_ext::nonzero;
use std::{io, num::NonZeroU32, path::Path};
use tokio::{fs::File, io::AsyncReadExt};

/// 命令行参数
#[derive(Parser)]
pub struct CliArgs {
    /// 输入文件路径
    pub file: String,
    /// 输出文件路径 (可选)
    #[arg(short, long)]
    pub output: Option<String>,
    /// 速率限制 (KB/s)
    #[arg(short, long, default_value = "512")]
    pub rate: u32,
}

/// 限速读取器 (异步版本)
pub struct RateLimitedReader {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    chunk_size: usize,
}

impl RateLimitedReader {
    /// 创建新读取器
    /// - `rate_kb`: 限速值 (单位 KB/s)
    /// - `chunk_size`: 每次读取块大小 (字节)
    pub fn new(rate_kb: u32, chunk_size: usize) -> Self {
        let bytes_per_second = NonZeroU32::new(rate_kb * 1024).expect("Invalid rate value");
        let quota = Quota::per_second(bytes_per_second);

        Self {
            limiter: RateLimiter::direct(quota),
            chunk_size,
        }
    }

    /// 异步读取文件并应用限速
    pub async fn read_file<P: AsRef<Path>>(&self, file_path: P) -> io::Result<Vec<u8>> {
        let mut file = File::open(file_path).await?;
        let mut buffer = Vec::new();
        let mut chunk = vec![0u8; self.chunk_size];

        loop {
            // 申请令牌
            self.limiter.until_n_ready(nonzero!(1u32)).await.unwrap();

            let bytes_read = file.read(&mut chunk).await?;
            if bytes_read == 0 {
                break;
            }

            buffer.extend_from_slice(&chunk[..bytes_read]);
        }

        Ok(buffer)
    }

    /// 动态调整速率 (单位 KB/s)
    pub fn adjust_rate(&mut self, new_rate_kb: u32) {
        let new_quota =
            Quota::per_second(NonZeroU32::new(new_rate_kb * 1024).expect("Invalid rate value"));
        let new_limiter = RateLimiter::direct(new_quota);
        self.limiter = new_limiter;
    }

    pub async fn read_file_with_progress<P, F>(
        &self,
        file_path: P,
        progress_cb: F,
    ) -> io::Result<Vec<u8>>
    where
        P: AsRef<Path>,
        F: Fn(usize, usize), // (已读字节, 总字节)
    {
        let mut file = File::open(file_path).await?;
        let total_size = file.metadata().await?.len() as usize;
        let mut buffer = Vec::new();
        let mut chunk = vec![0u8; self.chunk_size];
        let mut bytes_read = 0;

        loop {
            self.limiter.until_n_ready(nonzero!(1u32)).await.unwrap();
            let n = file.read(&mut chunk).await?;
            if n == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..n]);
            bytes_read += n;
            progress_cb(bytes_read, total_size);
        }

        Ok(buffer)
    }
}
