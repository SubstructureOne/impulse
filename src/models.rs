use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::debug_query;
use log::info;

use crate::schema::reports;


#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Copy, Clone)]
#[DieselTypePath = "crate::schema::sql_types::Pgpkttype"]
#[DbValueStyle = "verbatim"]
pub enum PostgresqlPacketType {
    Startup,
    Query,
    Other
}

#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Copy, Clone)]
#[DieselTypePath = "crate::schema::sql_types::Pktdirection"]
#[DbValueStyle = "verbatim"]
pub enum PacketDirection {
    Forward,
    Backward
}

#[derive(diesel_derive_enum::DbEnum, Debug)]
#[DieselTypePath = "crate::schema::sql_types::Chargetype"]
#[DbValueStyle = "verbatim"]
pub enum ChargeType {
    DataTransferInBytes,
    DataTransferOutBytes,
    DataStorageByteHours,
}

#[derive(diesel_derive_enum::DbEnum, Debug)]
#[DieselTypePath = "crate::schema::sql_types::Timechargetype"]
#[DbValueStyle = "verbatim"]
pub enum TimeChargeType {
    DataStorageBytes,
}

#[derive(Queryable, Debug, PartialEq)]
pub struct Report {
    pub report_id: i64,
    pub username: Option<String>,
    pub packet_type: PostgresqlPacketType,
    pub packet_time: DateTime<Utc>,
    pub direction: Option<PacketDirection>,
    pub packet_info: Option<serde_json::Value>,
    pub packet_bytes: Option<Vec<u8>>,
    pub charged: bool,
}

impl Report {
    pub fn for_user<S: Into<String>>(conn: &mut PgConnection, username_: S) -> Result<Vec<Report>>{
        use crate::schema::reports::dsl::*;
        Ok(reports
            .filter(username.eq(&username_.into()))
            .load::<Report>(conn)?)
    }

    pub fn uncharged(conn: &mut PgConnection) -> Result<Vec<Report>> {
        use crate::schema::reports::dsl::*;
        Ok(reports
            .filter(charged.eq(false))
            .load::<Report>(conn)?
        )
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = reports)]
pub struct NewReport {
    pub username: Option<String>,
    pub packet_type: PostgresqlPacketType,
    pub direction: Option<PacketDirection>,
    pub packet_info: Option<serde_json::Value>,
    pub packet_bytes: Option<Vec<u8>>,
    pub charged: bool,
}

impl NewReport {
    pub fn create(
        conn: &mut PgConnection,
        username: Option<String>,
        packet_type: PostgresqlPacketType,
        direction: Option<PacketDirection>,
        packet_info: Option<serde_json::Value>,
        packet_bytes: Option<Vec<u8>>,
        charged: bool
    ) -> Result<Report> {
        let new_report = NewReport {
            username,
            packet_type,
            direction,
            packet_info,
            packet_bytes,
            charged
        };
        let query = diesel::insert_into(reports::table)
            .values(&new_report);
        info!("Creating report: {}", debug_query::<diesel::pg::Pg, _>(&query));
        Ok(query.get_result::<Report>(conn)?)
    }
}

#[derive(Queryable, Debug)]
pub struct Charge {
    pub charge_id: i64,
    pub charge_time: DateTime<Utc>,
    pub user_id: uuid::Uuid,
    pub charge_type: ChargeType,
    pub quantity: f64,
    pub rate: f64,
    pub amount: f64,
}

#[derive(Queryable, Debug)]
pub struct TimeCharge {
    pub timecharge_id: i64,
    pub timecharge_time: DateTime<Utc>,
    pub user_id: uuid::Uuid,
    pub timecharge_type: TimeChargeType,
    pub amount: f64,
}

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
