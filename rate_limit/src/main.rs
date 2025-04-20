use clap::Parser;
use rate_limit::{CliArgs, RateLimitedReader};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    let reader = RateLimitedReader::new(args.rate, 4096);
    let data = reader.read_file(&args.file).await?;

    if let Some(output_path) = args.output {
        tokio::fs::write(output_path, data).await?;
        println!("File saved with rate limiting ({} KB/s)", args.rate);
    } else {
        println!("Read {} bytes with rate limiting", data.len());
    }

    Ok(())
}
