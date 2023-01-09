use anyhow::Result;
use log::info;
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
    let pg_base_uri = common::BASE_URL.to_string();
    let config = ManagementConfig::new(
        context.base_url.clone(),
        context.db_name.clone(),
    );
    let manager = PostgresManager::new(&config);
    let username = "testuser";
    let user_manager = PgUserManager::new(&manager, username);
    user_manager.with(|| {
        let info = manager.create_pg_user(username)?;
        info!("User {} created with password {}", &info.username, &info.password);
        Ok(())
    })?;
    info!("User {} dropped", username);
    Ok(())
}
