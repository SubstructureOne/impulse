use anyhow::Result;
use clap::Parser;

use impulse::manage::cli;


#[tokio::main]
pub async fn main() -> Result<()> {
    env_logger::init();
    dotenvy::dotenv().ok();
    let args = cli::ImpulseArgs::parse();
    cli::impulse(&args).await?;
    Ok(())
}
