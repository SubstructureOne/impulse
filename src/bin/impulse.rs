use std::collections::HashMap;
use std::rc::Rc;
use anyhow::Result;
use clap::Parser;
use log::info;
use uuid::Uuid;
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;
use impulse::models::charges::Charge;
use impulse::models::reports::{Report, ReportToCharge};

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
        let db_name = std::env::var("DB_NAME")?;
        let config = Rc::new(ManagementConfig::from_env()?);
        let manager = PostgresManager::new(config.clone());
        let mut conn = manager.pg_connect_db(&db_name)?;
        let uncharged = ReportToCharge::uncharged(&mut conn)?;
        let charges = Charge::create_charges(&mut conn, uncharged)?;
   }
    Ok(())
}