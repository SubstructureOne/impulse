use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::debug_query;
use diesel::prelude::*;
use log::{trace};
use diesel::pg::Pg;

use crate::schema::users;


#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Copy, Clone)]
#[DieselTypePath = "crate::schema::sql_types::Userstatus"]
#[DbValueStyle = "verbatim"]
pub enum UserStatus {
    Active,
    Disabled,
    Deleted
}

#[derive(Queryable, Debug, PartialEq)]
pub struct Users {
    pub user_id: i64,
    pub pg_name: String,
    pub user_status: UserStatus,
    pub balance: f64,
    pub status_synced: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
