use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::debug_query;
use diesel::pg::Pg;
use diesel::prelude::*;
use log::{trace};
use uuid::Uuid;

use crate::schema::reports;


#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Copy, Clone)]
#[DieselTypePath = "crate::schema::sql_types::Pgpkttype"]
#[DbValueStyle = "verbatim"]
pub enum PostgresqlPacketType {
    Authentication,
    Startup,
    Query,
    Other
}
impl From<&prew::postgresql::PostgresqlPacketInfo> for PostgresqlPacketType {
    fn from(prew_packet_type: &prew::postgresql::PostgresqlPacketInfo) -> Self {
        match prew_packet_type {
            prew::postgresql::PostgresqlPacketInfo::Authentication(_) => PostgresqlPacketType::Authentication,
            prew::postgresql::PostgresqlPacketInfo::Startup(_) => PostgresqlPacketType::Other,
            prew::postgresql::PostgresqlPacketInfo::Query(_) => PostgresqlPacketType::Other,
            prew::postgresql::PostgresqlPacketInfo::Other => PostgresqlPacketType::Other,
        }
    }
}

#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Copy, Clone)]
#[DieselTypePath = "crate::schema::sql_types::Pktdirection"]
#[DbValueStyle = "verbatim"]
pub enum PacketDirection {
    Forward,
    Backward
}
impl From<prew::packet::Direction> for PacketDirection {
    fn from(direction: prew::packet::Direction) -> Self {
        match direction {
            prew::packet::Direction::Forward => PacketDirection::Forward,
            prew::packet::Direction::Backward => PacketDirection::Backward,
        }
    }
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
}

mod views {
    use diesel::prelude::*;
    table! {
        use diesel::sql_types::*;
        use crate::schema::sql_types::Pgpkttype;
        use crate::schema::sql_types::Pktdirection;

        reports_to_charge (report_id) {
            report_id -> Int8,
            user_id -> Nullable<Uuid>,
            packet_type -> Pgpkttype,
            direction -> Nullable<Pktdirection>,
            num_bytes -> Nullable<Int4>,
        }
    }
}

#[derive(Queryable, Debug, PartialEq)]
pub struct ReportToCharge {
    pub report_id: i64,
    pub user_id: Option<Uuid>,
    pub packet_type: PostgresqlPacketType,
    pub direction: Option<PacketDirection>,
    pub num_bytes: Option<i32>,
}
impl ReportToCharge {
    pub fn uncharged(conn: &mut PgConnection) -> Result<Vec<ReportToCharge>> {
        use views::reports_to_charge::dsl::*;
        Ok(reports_to_charge.load::<ReportToCharge>(conn)?)
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
        username: Option<String>,
        packet_type: PostgresqlPacketType,
        direction: Option<PacketDirection>,
        packet_info: Option<serde_json::Value>,
        packet_bytes: Option<Vec<u8>>,
        charged: bool
    ) -> NewReport {
        NewReport {
            username,
            packet_type,
            direction,
            packet_info,
            packet_bytes,
            charged
        }
    }

    pub fn commit(&self, conn: &mut PgConnection) -> Result<Report> {
        let query = diesel::insert_into(reports::table)
            .values(self);
        trace!("Creating report: {}", debug_query::<Pg, _>(&query));
        Ok(query.get_result::<Report>(conn)?)
    }
}
