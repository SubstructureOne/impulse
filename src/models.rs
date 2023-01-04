use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::debug_query;
use diesel::pg::Pg;
use log::{info, trace};

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
        trace!("Creating report: {}", debug_query::<Pg, _>(&query));
        Ok(query.get_result::<Report>(conn)?)
    }
}

// diesel translates "report_ids bigint[]" as "Nullable<Array<Nullable<Int8>>>"
// since arrays in Postgres can contain NULL values. We prevent that by adding
// a constraint, but Diesel is not aware of it. Therefore we translate directly
// using the `Charge_` type with Option<Vec<Option<i64>>> for report_ids and
// then convert into `Charge` with Option<Vec<i64>> for report_ids.
#[derive(Queryable, Debug)]
struct Charge_ {
    pub charge_id: i64,
    pub charge_time: DateTime<Utc>,
    pub user_id: uuid::Uuid,
    pub charge_type: ChargeType,
    pub quantity: f64,
    pub rate: f64,
    pub amount: f64,
    pub report_ids: Option<Vec<Option<i64>>>,
    pub transacted: bool,
}
#[derive(Debug)]
pub struct Charge {
    pub charge_id: i64,
    pub charge_time: DateTime<Utc>,
    pub user_id: uuid::Uuid,
    pub charge_type: ChargeType,
    pub quantity: f64,
    pub rate: f64,
    pub amount: f64,
    pub report_ids: Option<Vec<i64>>,
    pub transacted: bool,
}
impl Charge {
    fn untransacted(conn: &mut PgConnection) -> Result<Vec<Charge>> {
        use crate::schema::charges::dsl::*;
        Ok(charges
            .filter(transacted.eq(false))
            .load::<Charge_>(conn)?
            .into_iter()
            .map(|charge| charge.into())
            .collect::<Vec<_>>()
        )
    }
}
impl From<Charge_> for Charge {
    fn from(charge_: Charge_) -> Self {
        let report_ids = match charge_.report_ids {
            Some(reports) => Some(
                reports
                    .iter()
                    .map(|rid| rid.unwrap())
                    .collect::<Vec<_>>()
            ),
            None => None
        };
        Charge {
            charge_id: charge_.charge_id,
            charge_time: charge_.charge_time,
            user_id: charge_.user_id,
            charge_type: charge_.charge_type,
            quantity: charge_.quantity,
            rate: charge_.rate,
            amount: charge_.amount,
            report_ids,
            transacted: charge_.transacted,
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = charges)]
pub struct NewCharge {
    pub user_id: uuid::Uuid,
    pub charge_type: ChargeType,
    pub quantity: f64,
    pub rate: f64,
    pub report_ids: Vec<i64>,
}

impl NewCharge {
    pub fn create(
        conn: &mut PgConnection,
        user_id: uuid::Uuid,
        charge_type: ChargeType,
        quantity: f64,
        rate: f64,
        report_ids: Vec<i64>
    ) -> Result<Charge> {
        let new_charge = NewCharge {
            user_id,
            charge_type,
            quantity,
            rate,
            report_ids
        };
        let query = diesel::insert_into(charges::table)
            .values(&new_charge);
        trace!("Creating charge: {}", debug_query::<Pg, _>(&query));
        Ok(query.get_result::<Charge_>(conn)?.into())
    }
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
