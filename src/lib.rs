use std::env;
use std::rc::Rc;

use diesel::{PgConnection};
use dotenvy::dotenv;
use crate::manage::ManagementConfig;
use crate::manage::postgres::PostgresManager;

pub mod schema;
pub mod models;
pub mod manage;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let config = Rc::new(ManagementConfig::from_env().unwrap());
    let manager = PostgresManager::new(config);
    manager.pg_connect_db(&env::var("DB_NAME").unwrap()).unwrap()
}
