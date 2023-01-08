use anyhow::Result;
use log::info;
use impulse::manage::ManagementConfig;
use impulse::manage::postgres::PostgresManager;

mod common;

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
    let info = manager.create_pg_user(username)?;
    info!("User {} created with password {}", &info.username, &info.password);
    manager.drop_pg_user(username)?;
    info!("User {} dropped", username);
    Ok(())
}
