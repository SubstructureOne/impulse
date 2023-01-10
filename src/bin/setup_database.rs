use anyhow::Result;
use diesel::prelude::*;
use docker_api::{Docker};
use docker_api::opts::{ContainerCreateOpts, PublishPort};
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;

#[tokio::main]
async fn main() -> Result<()> {
    let docker = Docker::unix("/var/run/docker.sock");
    dotenvy::dotenv().ok();
    let password = std::env::var("DOCKER_POSTGRES_PASSWORD").unwrap();
    let port = std::env::var("DOCKER_POSTGRES_PORT")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let env = vec![
        format!("POSTGRES_PASSWORD={}", password)
    ];
    let opts = ContainerCreateOpts::builder()
        .image("postgres:15")
        .name("postgres")
        .expose(PublishPort::tcp(5432), port)
        .env(&env)
        .build();
    let result = docker.containers().create(&opts).await?;
    result.start().await?;
    // wait for the container to accept connections
    let base_url = format!(
        "postgres://postgres:{}@localhost:{}",
        password, port
    );
    let postgres_url = format!("{}/postgres", base_url);
    loop {
        eprintln!("Attempting connection {}...", &postgres_url);
        let conn = PgConnection::establish(&postgres_url);
        match conn {
            Ok(_) => {
                eprintln!("Successfully connected to postgres database");
                break;
            },
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_secs(1))
            },
        }
    }
    println!("{}", result.id());
    let config = ManagementConfig::new(&base_url, "postgres");
    let manager = PostgresManager::new(&config);
    manager.setup_database()?;
    Ok(())
}
