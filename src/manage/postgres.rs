use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{error, info, trace};

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

    /// Initialize permissions on a newly created Postgres instance.
    ///
    /// Only needs to be run once to initialize a Postgres instance, but
    /// idempotent in actions.
    pub fn setup_database(&self) -> Result<()> {
        let mut conn = self.config.pg_connect()?;
        // revoke all rights from public on public schema
        sql_query("REVOKE ALL ON DATABASE template1 FROM public;").execute(&mut conn)?;
        sql_query("REVOKE ALL ON DATABASE postgres FROM public;").execute(&mut conn)?;
        sql_query("REVOKE ALL ON SCHEMA public FROM public;").execute(&mut conn)?;
        sql_query("GRANT ALL ON SCHEMA public TO postgres;").execute(&mut conn)?;
        // further revoke rights from PUBLIC to system catalogs
        sql_query("REVOKE ALL ON pg_user FROM public;").execute(&mut conn)?;
        sql_query("REVOKE ALL ON pg_roles FROM public;").execute(&mut conn)?;
        sql_query("REVOKE ALL ON pg_group FROM public;").execute(&mut conn)?;
        sql_query("REVOKE ALL ON pg_authid FROM public;").execute(&mut conn)?;
        sql_query("REVOKE ALL ON pg_auth_members FROM public;").execute(&mut conn)?;

        sql_query("REVOKE ALL ON pg_database FROM public;").execute(&mut conn)?;
        sql_query("REVOKE ALL ON pg_tablespace FROM public;").execute(&mut conn)?;
        sql_query("REVOKE ALL ON pg_settings FROM public;").execute(&mut conn)?;
        Ok(())
    }

    pub fn create_pg_user_and_database(&self, username: &str) -> Result<PgUserInfo> {
        // enforce strict naming conventions to prevent SQL injection
        let name_regex = regex::Regex::new(r"^[0-9a-zA-Z\.]+$")?;
        if !name_regex.is_match(username) {
            return Err(anyhow!("Illegal username: {}", username));
        }
        let mut conn = self.config.pg_connect()?;
        let password_gen = passwords::PasswordGenerator::new()
            .length(16)
            .numbers(true)
            .lowercase_letters(true)
            .uppercase_letters(true)
            // exclude symbols and spaces to make connection strings simpler
            .symbols(false)
            .spaces(false)
            .exclude_similar_characters(false)
            .strict(true);
        match password_gen.generate_one()
        {
            Err(err_msg) => Err(anyhow!("Couldn't generate password: {}", err_msg)),
            Ok(password) => {
                trace!("Creating PG user account: {}", username);
                let row_count = diesel::select(
                    create_pg_user(username, &password)
                ).execute(&mut conn)?;
                trace!("{} rows affected", row_count);
                trace!("Creating user database: {}", username);
                let row_count = sql_query(
                    format!("CREATE DATABASE \"{}\" WITH OWNER=\"{}\"", username, username)
                ).execute(&mut conn);
                if let Err(err) = row_count {
                    error!("CREATE DATABASE call failed: {}", err);
                    return Err(anyhow!(err))
                }
                trace!("{} rows affected", row_count.unwrap());
                trace!("Revoking public permissions on new database: {}", username);
                let row_count = sql_query(
                    format!("REVOKE ALL ON DATABASE \"{}\" FROM public", username)
                ).execute(&mut conn)?;
                trace!("{} rows affected", row_count);
                trace!("Connecting to new database");
                let mut user_conn = self.config.pg_connect_db(username)?;
                trace!("Granting all to user '{}' on database '{}'", username, username);
                let row_count = sql_query(
                    format!("GRANT ALL ON SCHEMA public TO {} WITH GRANT OPTION", username)
                ).execute(&mut user_conn)?;
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
        trace!("Dropping user database '{}'", username);
        self.drop_database(username)?;
        trace!("Dropping user '{}'", username);
        let row_count = diesel::select(
            drop_pg_user(username)
        ).execute(&mut conn)?;
        trace!("{} rows affected", row_count);
        Ok(())
    }

    pub fn drop_database(&self, database_name: &str) -> Result<()> {
        let mut conn = self.config.pg_connect()?;
        info!("Force disconnecting any users connected to {}", &database_name);
        let disconnect_users = sql_query(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = $1"
        ).bind::<Text, _>(database_name.to_string());
        let count = disconnect_users.execute(&mut conn)?;
        info!("{} users disconnected", count);

        info!("Dropping database {}", &database_name);
        let query = sql_query(
            format!(r#"DROP DATABASE IF EXISTS "{}""#, &database_name)
        );
        query.execute(&mut conn)?;
        Ok(())
    }

    // pub fn disable_pg_user(&self, user_id: &Uuid) -> Result<()> {
    //     Ok(())
    // }
}