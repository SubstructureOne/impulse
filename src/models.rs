use diesel::prelude::Queryable;
use chrono::{DateTime, NaiveDateTime, Utc};


#[derive(diesel_derive_enum::DbEnum, Debug)]
#[DieselTypePath = "crate::schema::sql_types::Pgpkttype"]
pub enum PostgresqlPacketType {
    Startup,
    Query,
    Other
}

#[derive(diesel_derive_enum::DbEnum, Debug)]
#[DieselTypePath = "crate::schema::sql_types::Pktdirection"]
pub enum PacketDirection {
    Forward,
    Backward
}

#[derive(diesel_derive_enum::DbEnum, Debug)]
#[DieselTypePath = "crate::schema::sql_types::Chargetype"]
pub enum ChargeType {
    DataTransferInBytes,
    DataTransferOutBytes,
    DataStorageByteHours,
}

#[derive(diesel_derive_enum::DbEnum, Debug)]
#[DieselTypePath = "crate::schema::sql_types::Timechargetype"]
pub enum TimeChargeType {
    DataStorageBytes,
}

#[derive(Queryable, Debug)]
pub struct Report {
    pub id: i64,
    pub username: Option<String>,
    pub packet_type: PostgresqlPacketType,
    pub packet_time: NaiveDateTime,
    pub direction: Option<PacketDirection>,
    pub packet_info: Option<serde_json::Value>,
    pub packet_bytes: Option<Vec<u8>>,
    pub charged: bool,
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
pub struct ExtTransacations {
    pub exttransaction_id: i64,
    pub user_id: uuid::Uuid,
    pub amount: f64,
    pub exttransaction_time: DateTime<Utc>,
}
