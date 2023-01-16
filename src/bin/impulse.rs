use std::rc::Rc;
use anyhow::Result;
use clap::Parser;
use log::info;
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;
use impulse::models::charges::Charge;
use impulse::models::reports::{ReportToCharge};
use impulse::models::transactions::NewTransaction;

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
    let db_name = std::env::var("DB_NAME")?;
    let config = Rc::new(ManagementConfig::from_env()?);
    let manager = PostgresManager::new(config.clone());
    let mut conn = manager.pg_connect_db(&db_name)?;
    if args.generate_charges {
        info!("Generating charges");
        let uncharged = ReportToCharge::uncharged(&mut conn)?;
        let charges = Charge::create_charges(&mut conn, uncharged)?;
        info!("Generated {} charges", charges.len());
    }
    if args.generate_transactions {
        info!("Generating transactions");
        let charges = Charge::untransacted(&mut conn)?;
        let transactions = NewTransaction::from_charges(
            &mut conn,
            &charges
        )?;
        info!("Generated {} transactions", transactions.len());
   }
    Ok(())
}