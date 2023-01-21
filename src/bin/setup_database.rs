use std::rc::Rc;

use anyhow::Result;
use clap::Parser;

use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;


#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct SetupDatabaseArgs {
    #[arg(short, long)]
    port: u32,
    #[arg(short='P', long)]
    password: String,
    #[arg(short='n', long)]
    host: String,
    #[arg(short, long)]
    username: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    let args = SetupDatabaseArgs::parse();
    let config = Rc::new(ManagementConfig::new(
        args.host,
        args.port,
        args.username,
        args.password,
    ));
    let manager = PostgresManager::new(config.clone());
    manager.setup_database()?;
    Ok(())
}
