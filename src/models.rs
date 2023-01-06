use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::debug_query;
use diesel::pg::Pg;
use log::{info, trace};

use crate::schema::*;

pub mod reports;
pub mod charges;



#[derive(Queryable, Debug)]
pub struct Balance {
    pub user_id: uuid::Uuid,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Debug)]
pub struct ExtTransactions {
    pub exttransaction_id: i64,
    pub user_id: uuid::Uuid,
    pub amount: f64,
    pub exttransaction_time: DateTime<Utc>,
}

#[derive(Queryable, Debug)]
pub struct Transaction {
    pub transaction_id: i64,
    pub txn_time: DateTime<Utc>,
    pub from_user: uuid::Uuid,
    pub to_user: uuid::Uuid,
    pub charge_ids: Option<Vec<i64>>,
    pub amount: f64,
}
