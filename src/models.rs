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


// #[derive(Debug, ToSql, FromSql)]
// #[postgres(name="pgpkttype")]
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

// impl Queryable<reports::SqlType, diesel::pg::Pg> for Report {
//     // type Row = (i64, Option<String>, PostgresqlPacketType,); // Timestamp, Nullable<Pktdirection>, Nullable<Jsonb>, Nullable<Binary>, Bool);
//     type Row = (PostgresqlPacketType,); // Timestamp, Nullable<Pktdirection>, Nullable<Jsonb>, Nullable<Binary>, Bool);
//
//     fn build(row: Self::Row) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
//         // let smth = row.7;
//         // smth.
//         Ok(Report {
//             // id: row.0.into(),
//             // username: row.1.into(),
//             packet_type: row.0.into(),
//             // packet_time: row.3.into(),
//             // direction: row.4.into(),
//             // packet_info: row.5.into(),
//             // packet_bytes: row.6.into(),
//             // charged: row.7.from_sql(),
//         })
//     }
// }
//
// impl FromStaticSqlRow<reports::SqlType, diesel::pg::Pg> for (PostgresqlPacketType,) {
//     fn build_from_row<'a>(row: &impl Row<'a, Pg>) -> diesel::deserialize::Result<Self> {
//         todo!()
//     }
// }
