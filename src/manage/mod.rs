use anyhow::Result;
use diesel::prelude::*;

pub mod postgres;

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

    pub fn pg_connect(&self) -> Result<PgConnection> {
        Ok(self.pg_connect_db(&self.pg_user)?)
    }

    pub fn pg_connect_db(&self, db_name: &str) -> Result<PgConnection> {
        Ok(PgConnection::establish(&self.create_uri(db_name))?)
    }

    pub fn with_user(&self, username: &str, password: &str) -> ManagementConfig {
        ManagementConfig::new(
            self.pg_host.clone(),
            self.pg_port.clone(),
            username,
            password,
        )
    }

    pub fn base_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.pg_user,
            self.pg_pw,
            self.pg_host,
            self.pg_port,
        )
    }

    fn create_uri(&self, db_name: &str) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.pg_user,
            self.pg_pw,
            self.pg_host,
            self.pg_port,
            db_name,
        )
    }
}