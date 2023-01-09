use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::sql_types::Text;
use log::trace;
use uuid::Uuid;

use crate::manage::ManagementConfig;

sql_function!(
    fn create_pg_user(p_username: Text, p_password: Text);
);
sql_function!(
    fn drop_pg_user(p_username: Text);
);

pub struct PostgresManager<'a> {
    config: &'a ManagementConfig
}

pub struct PgUserInfo {
    pub username: String,
    pub password: String,
}

impl<'a> PostgresManager<'a> {
    pub fn new(config: &ManagementConfig) -> PostgresManager {
        PostgresManager { config }
    }

    pub fn create_pg_user(&self, username: &str) -> Result<PgUserInfo> {
        let mut conn = self.config.pg_connect()?;
        let password_gen = passwords::PasswordGenerator::new()
            .length(16)
            .numbers(true)
            .lowercase_letters(true)
            .uppercase_letters(true)
            .symbols(true)
            .spaces(false)
            .exclude_similar_characters(false)
            .strict(true);
        match password_gen.generate_one()
        {
            Err(err_msg) => Err(anyhow!("Couldn't generate password: {}", err_msg)),
            Ok(password) => {
                let row_count = diesel::select(
                    create_pg_user(username, &password)
                ).execute(&mut conn)?;
                trace!("{} rows affected", row_count);
                Ok(PgUserInfo {
                    username: username.to_string(),
                    password
                })
            }
        }
    }

    pub fn drop_pg_user(&self, username: &str) -> Result<()> {
        let mut conn = self.config.pg_connect()?;
        let row_count = diesel::select(
            drop_pg_user(username)
        ).execute(&mut conn)?;
        trace!("{} rows affected", row_count);
        Ok(())
    }

    pub fn disable_pg_user(&self, user_id: &Uuid) -> Result<()> {
        Ok(())
    }
}