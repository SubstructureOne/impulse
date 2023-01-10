use anyhow::Result;
use docker_api::{Docker};
use docker_api::opts::{ContainerCreateOpts, PublishPort};
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;

#[tokio::main]
async fn main() -> Result<()> {
    let docker = Docker::unix("/var/run/docker.sock");
    dotenvy::dotenv().ok();
    let port = std::env::var("DOCKER_DB_PORT")
        .expect("Must specify DOCKER_DB_PORT")
        .parse::<u32>()?;
    let password = std::env::var("DOCKER_DB_PASSWORD")
        .expect("Must specify DOCKER_DB_PASSWORD");
    let config = ManagementConfig::new(
        std::env::var("DOCKER_DB_HOST").expect("Must specify DOCKER_DB_HOST"),
        port,
        std::env::var("DOCKER_DB_USER").expect("Must specify DOCKER_DB_USER"),
        password.clone(),
    );

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
    loop {
        eprintln!("Attempting connection ...");
        let conn = config.pg_connect();
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

    let manager = PostgresManager::new(&config);
    manager.setup_database()?;
    Ok(())
}
