use anyhow::Result;
use diesel::PgConnection;
use diesel::prelude::*;
use log::{debug, info};
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;

mod common;

/// Context manager for making sure temporary test users get dropped at the
/// end of the scope.
struct PgUserManager<'a> {
    username: &'a str,
    pg_manager: &'a PostgresManager<'a>,
}
impl<'a> PgUserManager<'a> {
    fn new(pg_manager: &'a PostgresManager, username: &'a str) -> PgUserManager<'a> {
        PgUserManager { username, pg_manager }
    }

    fn with<F>(&self, f: F) -> Result<()> where F: Fn() -> Result<()> {
        f()?;
        self.pg_manager.drop_pg_user(self.username)?;
        Ok(())
    }
}

#[test]
pub fn user_creation_test() -> Result<()> {
    let context = common::TestContext::new("postgres_test")?;
    let config = ManagementConfig::new(
        context.base_url.clone(),
        context.db_name.clone(),
    );
    let manager = PostgresManager::new(&config);
    let username = "testuser";
    let user_manager = PgUserManager::new(&manager, username);
    user_manager.with(|| {
        let info = manager.create_pg_user_and_database(username)?;
        let port = std::env::var("TESTING_DB_PORT")
            .unwrap()
            .parse::<u32>()
            .unwrap();
        info!("User {} created with password {}", &info.username, &info.password);
        let user_pguri = format!(
            "postgres://{}:{}@localhost:{}/{}",
            &info.username,
            &info.password,
            port,
            &info.username
        );
        debug!("Testing that user can connect to their database ({})", user_pguri);
        let mut user_conn = PgConnection::establish(&user_pguri)?;
        debug!("Testing that user can create tables");
        diesel::sql_query("CREATE TABLE test_table (col1 INT, col2 INT)")
            .execute(&mut user_conn)?;
        debug!("Testing that user can insert data");
        diesel::sql_query("INSERT INTO test_table (col1, col2) VALUES (1, 3)")
            .execute(&mut user_conn)?;
        debug!("Testing that user cannot connect to other databases");
        let failed_pguri = format!(
            "postgres://{}:{}@localhost:{}/postgres",
            &info.username,
            &info.password,
            port
        );
        let failed_conn = PgConnection::establish(&failed_pguri);
        if let Ok(_) = failed_conn {
            assert!(
                false,
                "Should not have been able to connect to postgres database as user {}",
                &info.username
            );
        }
        // need to drop the created database before going out of the user
        // manager scope, because the newly created database has the new
        // user defined as its owner, so the database must be dropped
        // before the user is dropped
        manager.drop_pg_user(username)?;
        Ok(())
    })?;
    info!("User {} dropped", username);
    Ok(())
}
