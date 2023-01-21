use anyhow::Result;

pub mod postgres;
pub mod cli;
pub mod container;

#[derive(Debug)]
pub struct ManagementConfig {
    pg_host: String,
    pg_port: u32,
    pg_user: String,
    pg_pw: String,
}

impl ManagementConfig {
    pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        pg_host: S1,
        pg_port: u32,
        pg_user: S2,
        pg_pw: S3,
    ) -> ManagementConfig {
        ManagementConfig {
            pg_host: pg_host.into(),
            pg_port,
            pg_user: pg_user.into(),
            pg_pw: pg_pw.into(),
        }
    }

    pub fn from_env() -> Result<ManagementConfig> {
        Ok(
            ManagementConfig {
                pg_host: std::env::var("DB_HOST")?,
                pg_port: std::env::var("DB_PORT")?.parse::<u32>()?,
                pg_user: std::env::var("DB_USER")?,
                pg_pw: std::env::var("DB_PASSWORD")?,
            }
        )
    }
}