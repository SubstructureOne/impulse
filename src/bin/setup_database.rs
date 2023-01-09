use anyhow::Result;
use diesel::prelude::*;
use docker_api::{Docker};
use docker_api::opts::{ContainerCreateOpts, PublishPort};

#[tokio::main]
async fn main() -> Result<()> {
    let docker = Docker::unix("/var/run/docker.sock");
    let env = vec!["POSTGRES_PASSWORD=pw"];
    dotenvy::dotenv().ok();
    let opts = ContainerCreateOpts::builder()
        .image("postgres:15")
        .name("postgres")
        .expose(PublishPort::tcp(5432), 9432)
        .env(&env)
        .build();
    let result = docker.containers().create(&opts).await?;
    result.start().await?;
    // wait for the container to accept connections
    let postgres_url = format!(
        "{}/postgres",
        std::env::var("TESTING_BASE_URL").unwrap()
    );
    loop {
        let conn = PgConnection::establish(&postgres_url);
        match conn {
            Ok(_) => { break; }
            Err(_) => {}
        }
    }
    println!("{}", result.id());
    Ok(())
}

