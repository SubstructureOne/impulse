use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::debug_query;
use diesel::prelude::*;
use log::{trace};
use diesel::pg::Pg;

use crate::schema::*;


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
        trace!("Creating report: {}", debug_query::<Pg, _>(&query));
        Ok(query.get_result::<Report>(conn)?)
    }
}
