mod common;

use std::rc::Rc;
use anyhow::Result;
use diesel::prelude::*;
use log::{debug, info};

use impulse::manage::postgres::PostgresManager;


/// Context manager for making sure temporary test users get dropped at the
/// end of the scope.
struct PgUserManager<'a> {
    username: &'a str,
    pg_manager: &'a PostgresManager,
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
    let username = "testuser";
    let managed_db_manager = &context.managed_db_manager;
    let user_manager = PgUserManager::new(
        &managed_db_manager,
        username
    );
    user_manager.with(|| {
        let info = managed_db_manager.create_pg_user_and_database(username)?;
        info!("User {} created with password {}", &info.username, &info.password);
        debug!("Testing that user can connect to their database ({})", &managed_db_manager);
        let user_config = Rc::new(managed_db_manager.with_user(username, &info.password));
        let user_conn_mgr = PostgresManager::new(user_config.clone());
        let mut user_conn = user_conn_mgr.pg_connect_db(username)?;
        debug!("Testing that user can create tables");
        diesel::sql_query("CREATE TABLE test_table (col1 INT, col2 INT)")
            .execute(&mut user_conn)?;
        debug!("Testing that user can insert data");
        diesel::sql_query("INSERT INTO test_table (col1, col2) VALUES (1, 3)")
            .execute(&mut user_conn)?;
        debug!("Testing that user cannot connect to other databases");
        let failed_conn = user_conn_mgr.pg_connect_db("postgres");
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
        managed_db_manager.drop_pg_user(username)?;
        Ok(())
    })?;
    info!("User {} dropped", username);
    Ok(())
}
