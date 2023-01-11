use std::collections::HashMap;
use std::rc::Rc;
use anyhow::Result;
use clap::Parser;
use log::info;
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;
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
        let mut grouped: HashMap<String, Vec<Report>> = HashMap::new();
        let mut no_user: Vec<Report> = vec![];
        for report in uncharged.into_iter() {
            match report.username.clone() {
                Some(username) => {
                    let user_reports = grouped.get_mut(&username);
                    match user_reports {
                        Some(report_vec) => report_vec.push(report),
                        None => {
                            let mut report_vec = vec![];
                            report_vec.push(report);
                            grouped.insert(username.clone(), report_vec);
                        }
                    }
                }
                None => no_user.push(report)
            }
        }
    }
    Ok(())
}