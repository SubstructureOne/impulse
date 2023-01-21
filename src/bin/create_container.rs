use std::rc::Rc;

use anyhow::Result;

use impulse::manage::container;
use impulse::manage::postgres::PostgresManager;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct CreateContainerArgs {
    /// Name to assign to the container [default: postgres]
    #[arg(short, long)]
    name: Option<String>,
    /// Localhost port to connect to the Postgres instance
    #[arg(short, long)]
    port: u32,
    /// Password for the admin "postgres" user [default: pw]
    #[arg(short='P', long)]
    password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CreateContainerArgs::parse();
    let container_config = container::PgContainerConfig::new(
        args.name.unwrap_or(String::from("postgres")),
        args.port,
        args.password.unwrap_or(String::from("pw")).clone(),
    );
    let pg_container = container::create_postgres_container(&container_config).await?;
    let config = Rc::new(container_config.to_management_config());
    // wait for the container to accept connections
    let manager = PostgresManager::new(config.clone());
    container::wait_for_connection(&manager).await;
    println!("{}", pg_container.id());
    Ok(())
}
