use anyhow::Result;
use docker_api::{Docker, Container};
use docker_api::opts::{ContainerCreateOpts, PublishPort};
use log::info;

use crate::manage::ManagementConfig;
use crate::manage::postgres::PostgresManager;

pub struct PgContainerConfig {
    pub name: String,
    pub port: u32,
    pub password: String,
}
impl PgContainerConfig {
    pub fn new(
        name: String,
        port: u32,
        password: String
    ) -> PgContainerConfig {
        PgContainerConfig { name, port, password }
    }

    pub fn to_management_config(&self) -> ManagementConfig {
        ManagementConfig::new(
            "localhost",
            self.port,
            "postgres",
            self.password.clone(),
        )
    }
}

pub async fn create_postgres_container(config: &PgContainerConfig) -> Result<Container> {
    let docker = Docker::unix("/var/run/docker.sock");
    let env = vec![
        format!("POSTGRES_PASSWORD={}", &config.password)
    ];
    let opts = ContainerCreateOpts::builder()
        .image("postgres:15")
        .name(config.name.clone())
        .expose(PublishPort::tcp(5432), config.port)
        .env(&env)
        .build();
    let result = docker.containers().create(&opts).await?;
    result.start().await?;
    Ok(result)
}

pub async fn wait_for_connection(manager: &PostgresManager) {
    loop {
        info!("Attempting connection to {} ...", manager);
        let conn = manager.pg_connect();
        match conn {
            Ok(_) => {
                info!("Successfully connected to {}", manager);
                break;
            },
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            },
        }
    }
}