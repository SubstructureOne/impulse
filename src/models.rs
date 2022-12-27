use diesel::prelude::Queryable;
use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::FromStaticSqlRow;
use diesel::pg::Pg;
use diesel::row::Row;
use diesel::sql_types::{BigInt, Binary, Bool, Bytea, Int8, Jsonb, Text, Timestamp};
use postgres_types::{FromSql, ToSql};

use crate::schema::sql_types::{Pgpkttype, Pktdirection};
use crate::schema::reports;


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
