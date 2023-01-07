use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::debug_query;
use diesel::pg::Pg;
use log::{trace};
use uuid::Uuid;

use crate::schema::charges;


#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Eq)]
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


// diesel translates "report_ids bigint[]" as "Nullable<Array<Nullable<Int8>>>"
// since arrays in Postgres can contain NULL values. We prevent that by adding
// a constraint, but Diesel is not aware of it. Therefore we translate directly
// using the `Charge_` type with Option<Vec<Option<i64>>> for report_ids and
// then convert into `Charge` with Option<Vec<i64>> for report_ids.
#[derive(Queryable, Debug)]
struct Charge_ {
    pub charge_id: i64,
    pub charge_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub charge_type: ChargeType,
    pub quantity: f64,
    pub rate: f64,
    pub amount: f64,
    pub report_ids: Option<Vec<Option<i64>>>,
    pub transacted: bool,
}
#[derive(Debug, PartialEq)]
pub struct Charge {
    pub charge_id: i64,
    pub charge_time: DateTime<Utc>,
    pub user_id: Uuid,
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
    
    pub fn retrieve(conn: &mut PgConnection, charge_id_: i64) -> Result<Charge> {
        use crate::schema::charges::dsl::*;
        Ok(
            charges
                .find(&charge_id_)
                .first::<Charge_>(conn)?
                .into()
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
    pub user_id: Uuid,
    pub charge_type: ChargeType,
    pub quantity: f64,
    pub rate: f64,
    pub report_ids: Option<Vec<i64>>,
}

impl NewCharge {
    pub fn create(
        conn: &mut PgConnection,
        user_id: Uuid,
        charge_type: ChargeType,
        quantity: f64,
        rate: f64,
        report_ids: Option<Vec<i64>>,
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
    pub user_id: Uuid,
    pub timecharge_type: TimeChargeType,
    pub amount: f64,
}
