use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::debug_query;
use diesel::pg::Pg;
use log::{info, trace};

use crate::schema::*;

pub mod reports;
pub mod charges;
pub mod transactions;


#[derive(Queryable, Debug)]
pub struct Balance {
    pub user_id: uuid::Uuid,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

