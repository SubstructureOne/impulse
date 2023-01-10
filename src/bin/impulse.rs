use std::rc::Rc;
use anyhow::Result;
use clap::Parser;
use log::info;
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;
use impulse::models::reports::Report;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    generate_charges: bool,
    #[arg(short, long)]
    generate_transactions: bool,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    env_logger::init();
    dotenvy::dotenv().ok();
    let args = Args::parse();
    if args.generate_charges {
        info!("Generating charges");
        let config = Rc::new(ManagementConfig::from_env()?);
        let manager = PostgresManager::new(config.clone());
        // let mut conn = manager
        // let uncharged = Report::uncharged()
    }
    Ok(())
}