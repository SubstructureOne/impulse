use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::debug_query;
use diesel::pg::Pg;
use log::{trace};
use uuid::Uuid;
use crate::models::reports::{PacketDirection, Report};

use crate::schema::charges;


#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
    pub fn untransacted(conn: &mut PgConnection) -> Result<Vec<Charge>> {
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

    pub fn create_charges(conn: &mut PgConnection, reports: Vec<Report>) -> Result<Vec<Charge>> {
        let mut user2type2charge: HashMap<Option<String>, HashMap<ChargeType, NewCharge>> = HashMap::new();
        for report in reports {
            let type2charge = user2type2charge
                .entry(report.username.clone())
                .or_insert(HashMap::new());
            Self::append_report(type2charge, &report);
        }
        user2type2charge
            .values()
            .flat_map(|hashmap| hashmap.values())
            .map(|new_charge| new_charge.commit(conn))
            .collect::<Result<Vec<Charge>>>()
    }

    fn append_report(existing_charges: &mut HashMap<ChargeType, NewCharge>, new_report: &Report) {
        if let Some(charge_type) = Self::report_charge_type(new_report) {
            let existing = existing_charges.get_mut(&charge_type);
            match existing {
                Some(charge) => {
                    charge.quantity += new_report.packet_bytes.as_ref().unwrap().len() as f64;
                    charge.report_ids.as_mut().unwrap().push(new_report.report_id);
                },
                None => {
                    let charge = NewCharge {
                        user_id: Uuid::new_v4(), // FIXME
                        charge_type,
                        quantity: new_report.packet_bytes.as_ref().unwrap().len() as f64,
                        rate: 0.1, // FIXME
                        report_ids: Some(vec![new_report.report_id])
                    };
                    existing_charges.insert(charge_type, charge);
                }
            }
        }
    }

    fn report_charge_type(report: &Report) -> Option<ChargeType> {
        match report.direction {
            None => None,
            Some(PacketDirection::Forward) => Some(ChargeType::DataTransferInBytes),
            Some(PacketDirection::Backward) => Some(ChargeType::DataTransferOutBytes),
        }
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
    pub fn new(
        user_id: Uuid,
        charge_type: ChargeType,
        quantity: f64,
        rate: f64,
        report_ids: Option<Vec<i64>>,
    ) -> NewCharge {
        NewCharge {
            user_id,
            charge_type,
            quantity,
            rate,
            report_ids
        }
    }

    pub fn commit(&self, conn: &mut PgConnection) -> Result<Charge> {
        let query = diesel::insert_into(charges::table)
            .values(self);
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
