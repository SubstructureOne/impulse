use std::env;
use std::rc::Rc;

use anyhow::{Context, Result};
use clap::Parser;

use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;


#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct SetupDatabaseArgs {
    #[arg(short, long)]
    port: Option<u32>,
    #[arg(short='P', long)]
    password: Option<String>,
    #[arg(short='n', long)]
    host: Option<String>,
    #[arg(short, long)]
    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    let args = SetupDatabaseArgs::parse();
    let config = Rc::new(ManagementConfig::new(
        args.host.or(env::var("MANAGED_DB_HOST").ok())
            .context("Host not provided")?,
        args.port.or(env::var("MANAGED_DB_PORT").ok()
            .and_then(|port_str| port_str.parse::<u32>().ok())
        ).context("Port not provided")?,
        args.username.or(env::var("MANAGED_DB_USER").ok())
            .context("Username not provided")?,
        args.password.or(env::var("MANAGED_DB_PASSWORD").ok())
            .context("Password not provided")?,
    ));
    let manager = PostgresManager::new(config.clone());
    manager.setup_database()?;
    Ok(())
}
