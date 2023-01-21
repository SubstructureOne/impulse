#![recursion_limit = "1024"]

use std::env;

use anyhow::Result;
use diesel::prelude::*;

pub mod schema;
pub mod models;
pub mod manage;

pub fn connect_impulse_db() -> Result<PgConnection> {
    let db_url = env::var("DATABASE_URL")?;
    Ok(PgConnection::establish(&db_url)?)
}
