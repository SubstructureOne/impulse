use std::fmt::{Display, Formatter};
use std::str::FromStr;
use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::{debug_query};
use diesel::pg::{Pg};
use diesel::prelude::*;
use log::{trace};
use uuid::Uuid;

use crate::schema::reports;


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PostgresqlPacketType {
    Authentication,
    Startup,
    Query,
    SslRequest,
    DataRow,
    Other
}
impl From<&prew::postgresql::PostgresqlPacketInfo> for PostgresqlPacketType {
    fn from(prew_packet_type: &prew::postgresql::PostgresqlPacketInfo) -> Self {
        match prew_packet_type {
            prew::postgresql::PostgresqlPacketInfo::Authentication(_) => PostgresqlPacketType::Authentication,
            prew::postgresql::PostgresqlPacketInfo::Startup(_) => PostgresqlPacketType::Startup,
            prew::postgresql::PostgresqlPacketInfo::Query(_) => PostgresqlPacketType::Query,
            prew::postgresql::PostgresqlPacketInfo::SslRequest => PostgresqlPacketType::SslRequest,
            prew::postgresql::PostgresqlPacketInfo::DataRow(_) => PostgresqlPacketType::DataRow,
            prew::postgresql::PostgresqlPacketInfo::Other => PostgresqlPacketType::Other,
        }
    }
}
impl Display for PostgresqlPacketType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl FromStr for PostgresqlPacketType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Authentication" => Ok(PostgresqlPacketType::Authentication),
            "Startup" => Ok(PostgresqlPacketType::Startup),
            "Query" => Ok(PostgresqlPacketType::Query),
            "Other" => Ok(PostgresqlPacketType::Other),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
impl Display for PacketDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl FromStr for PacketDirection {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Forward" => Ok(PacketDirection::Forward),
            "Backward" => Ok(PacketDirection::Backward),
            _ => Err(()),
        }
    }
}

#[derive(Queryable, Debug, PartialEq)]
pub struct Report_ {
    pub report_id: i64,
    pub username: Option<String>,
    pub packet_type: String,
    pub packet_time: DateTime<Utc>,
    pub direction: Option<String>,
    pub packet_info: Option<serde_json::Value>,
    pub packet_bytes: Option<Vec<u8>>,
    pub charged: bool,
}
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
            .load::<Report_>(conn)?
            .into_iter()
            .map(Report::from)
            .collect()
        )
    }

    pub fn mark_charged(report_id: i64, conn: &mut PgConnection) -> Result<()> {
        use crate::schema::reports::dsl::*;
        diesel::update(reports.find(report_id))
            .set(charged.eq(true))
            .execute(conn)?;
        Ok(())
    }
}
impl From<Report_> for Report {
    fn from(value: Report_) -> Self {
        let direction = match value.direction {
            Some(dir) => Some(PacketDirection::from_str(&dir).unwrap()),
            None => None
        };
        return Report {
            report_id: value.report_id,
            username: value.username,
            packet_type: PostgresqlPacketType::from_str(&value.packet_type).unwrap(),
            packet_time: value.packet_time,
            direction,
            packet_info: value.packet_info,
            packet_bytes: value.packet_bytes,
            charged: value.charged,
        }
    }
}

mod views {
    use diesel::prelude::*;
    table! {
        use diesel::sql_types::*;

        reports_to_charge (report_id) {
            report_id -> Int8,
            user_id -> Nullable<Uuid>,
            packet_type -> Text,
            direction -> Nullable<Text>,
            num_bytes -> Nullable<Int4>,
        }
    }
}

#[derive(Queryable, Debug, PartialEq)]
pub struct ReportToCharge_ {
    pub report_id: i64,
    pub user_id: Option<Uuid>,
    pub packet_type: String,
    pub direction: Option<String>,
    pub num_bytes: Option<i32>,
}
#[derive(Debug, PartialEq)]
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
        Ok(
            reports_to_charge
                .load::<ReportToCharge_>(conn)?
                .into_iter()
                .map(ReportToCharge::from)
                .collect()
        )
    }

    pub fn with_userid(report: Report, user_id: Uuid) -> ReportToCharge {
        let num_bytes: Option<i32> = match report.packet_bytes {
            Some(byte_arr) => i32::try_from(byte_arr.len()).ok(),
            None => None,
        };
        ReportToCharge {
            report_id: report.report_id,
            user_id: Some(user_id),
            packet_type: report.packet_type,
            direction: report.direction,
            num_bytes
        }
    }
}
impl From<ReportToCharge_> for ReportToCharge {
    fn from(value: ReportToCharge_) -> Self {
        let direction = match value.direction {
            Some(dir) => Some(PacketDirection::from_str(&dir).unwrap()),
            None => None
        };
        return ReportToCharge {
            report_id: value.report_id,
            user_id: value.user_id,
            packet_type: PostgresqlPacketType::from_str(&value.packet_type).unwrap(),
            direction,
            num_bytes: value.num_bytes,
        }
    }
}
impl From<Report> for ReportToCharge {
    fn from(value: Report) -> Self {
        let num_bytes: Option<i32> = match value.packet_bytes {
            Some(byte_arr) => i32::try_from(byte_arr.len()).ok(),
            None => None,
        };
        return ReportToCharge {
            report_id: value.report_id,
            user_id: None,
            packet_type: value.packet_type,
            direction: value.direction,
            num_bytes
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = reports)]
pub struct NewReport {
    pub username: Option<String>,
    pub packet_type: String,
    pub direction: Option<String>,
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
        let direction = match direction {
            Some(dir) => Some(dir.to_string()),
            None => None
        };
        NewReport {
            username,
            packet_type: packet_type.to_string(),
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
        Ok(query.get_result::<Report_>(conn)?.into())
    }
}
