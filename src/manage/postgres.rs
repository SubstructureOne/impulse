use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{error, info, trace};
use uuid::Uuid;

use crate::manage::ManagementConfig;
use crate::models::users::User;

sql_function!(
    fn create_pg_user(p_username: Text, p_password: Text);
);
sql_function!(
    fn drop_pg_user(p_username: Text);
);
const DB_SEPARATOR: &str = "_";

pub struct PostgresManager {
    pub config: Rc<ManagementConfig>,
}
impl<'a> Display for PostgresManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base_url())
    }
}
impl PostgresManager {
    pub fn pg_connect(&self) -> Result<PgConnection> {
        Ok(self.pg_connect_db(&self.config.pg_user)?)
    }

    pub fn pg_connect_db(&self, db_name: &str) -> Result<PgConnection> {
        Ok(PgConnection::establish(&self.create_uri(db_name))?)
    }

    pub fn with_user(&self, username: &str, password: &str) -> ManagementConfig {
        ManagementConfig::new(
            self.config.pg_host.clone(),
            self.config.pg_port.clone(),
            username,
            password,
        )
    }

    pub fn base_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.config.pg_user,
            self.config.pg_pw,
            self.config.pg_host,
            self.config.pg_port,
        )
    }

    fn create_uri(&self, db_name: &str) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.config.pg_user,
            self.config.pg_pw,
            self.config.pg_host,
            self.config.pg_port,
            db_name,
        )
    }
}

pub struct PgUserInfo {
    pub username: String,
    pub password: String,
}

#[derive(QueryableByName, Debug)]
pub struct PgDatabaseSize {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub db_name: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub db_bytes: i64,
}

impl PostgresManager {
    pub fn new(config: Rc<ManagementConfig>) -> PostgresManager {
        PostgresManager { config }
    }

    /// Initialize permissions on a newly created Postgres instance.
    ///
    /// Only needs to be run once to initialize a Postgres instance, but
    /// idempotent in actions.
    pub fn setup_database(&self) -> Result<()> {
        let mut conn = self.pg_connect()?;
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
        let mut conn = self.pg_connect()?;
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
                let row_count = sql_query(
                    format!(
                        r#"CREATE ROLE "{}" WITH LOGIN CREATEDB NOSUPERUSER NOINHERIT NOCREATEROLE PASSWORD '{}'"#,
                        username,
                        password,
                    )
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
                // FIXME: this shouldn't be necessary given changes to template1
                trace!("Revoking public permissions on new database: {}", username);
                let row_count = sql_query(
                    format!("REVOKE ALL ON DATABASE \"{}\" FROM public", username)
                ).execute(&mut conn)?;
                trace!("{} rows affected", row_count);
                trace!("Connecting to new database");
                let mut user_conn = self.pg_connect_db(username)?;
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
        let mut conn = self.pg_connect()?;
        trace!("Dropping user database '{}'", username);
        self.drop_database(username)?;
        trace!("Dropping user '{}'", username);
        // let row_count = diesel::select(
        //     drop_pg_user(username)
        // ).execute(&mut conn)?;
        let row_count = sql_query(
            format!(
                r#"DROP ROLE IF EXISTS "{}""#,
                username,
            )
        ).execute(&mut conn)?;
        trace!("{} rows affected", row_count);
        Ok(())
    }

    pub fn drop_database(&self, database_name: &str) -> Result<()> {
        // TODO: just use DROP DATABASE WITH FORCE
        let mut conn = self.pg_connect()?;
        info!("Force disconnecting any users connected to {}", &database_name);
        let count = sql_query(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = $1"
        )
            .bind::<Text,_>(database_name)
            .execute(&mut conn)?;
        info!("{} users disconnected", count);

        info!("Dropping database {}", &database_name);
        let query = sql_query(
            format!(r#"DROP DATABASE IF EXISTS "{}""#, &database_name)
        );
        query.execute(&mut conn)?;
        Ok(())
    }

    pub fn compute_storage(&self) -> Result<HashMap<Uuid, i64>> {
        let mut conn = self.pg_connect()?;
        let mut result: HashMap<Uuid, i64> = HashMap::new();
        let db_sizes = sql_query(
            "SELECT datname, pg_database_size(datname) FROM pg_database"
        ).load::<PgDatabaseSize>(&mut conn)?;
        let name2uuid = User::all(&mut conn)?
            .iter()
            .map(|user| (user.pg_name.clone(), user.user_id))
            .into_iter()
            .collect::<HashMap<_,_>>();
        for db_size in db_sizes {
            if let Some(pg_name_index) = db_size.db_name.rfind(DB_SEPARATOR) {
                let pg_name = &db_size.db_name[pg_name_index+1..];
                if let Some(user_id) = name2uuid.get(pg_name) {
                    *result.entry(*user_id).or_insert(0) += db_size.db_bytes;
                }
            }
        }
        Ok(result)
    }

    // pub fn disable_pg_user(&self, user_id: &Uuid) -> Result<()> {
    //     Ok(())
    // }
}