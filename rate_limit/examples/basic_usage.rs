use rate_limit::RateLimitedReader;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建限速器 (512 KB/s)
    let mut reader = RateLimitedReader::new(512, 4096);

    // 读取文件
    let data = reader.read_file("input.bin").await?;

    // 动态调整速率至 256 KB/s
    reader.adjust_rate(256);

    // 处理数据...
    println!("Read {} bytes", data.len());

    Ok(())
}
