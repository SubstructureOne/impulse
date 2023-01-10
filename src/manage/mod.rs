use anyhow::Result;
use diesel::prelude::*;

pub mod postgres;

pub struct ManagementConfig {
    pg_base_uri: String,
    db_name: String,
}

impl ManagementConfig {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        pg_base_uri: S1,
        db_name: S2,
    ) -> ManagementConfig {
        ManagementConfig {
            pg_base_uri: pg_base_uri.into(),
            db_name: db_name.into(),
        }
    }

    pub fn pg_connect(&self) -> Result<PgConnection> {
        Ok(PgConnection::establish(&self.create_uri(&self.db_name))?)
    }

    pub fn pg_connect_db(&self, dbname: &str) -> Result<PgConnection> {
        Ok(PgConnection::establish(&self.create_uri(dbname))?)
    }

    fn create_uri(&self, db_name: &str) -> String {
        format!("{}/{}", &self.pg_base_uri, &db_name)
    }
}