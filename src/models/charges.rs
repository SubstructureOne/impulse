use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{debug_query, sql_query};
use diesel::pg::Pg;
use enum_iterator::Sequence;
use itertools::Itertools;
use log::{trace};
use uuid::Uuid;

use crate::models::reports::{PacketDirection, ReportToCharge};
use crate::schema;
use crate::schema::charges;


#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Eq, Hash, Copy, Clone, Sequence)]
#[DieselTypePath = "crate::schema::sql_types::Chargetype"]
#[DbValueStyle = "verbatim"]
pub enum ChargeType {
    DataTransferInBytes,
    DataTransferOutBytes,
    DataStorageByteHours,
}
impl From<TimeChargeType> for ChargeType {
    fn from(timecharge_type: TimeChargeType) -> Self {
        match timecharge_type {
            TimeChargeType::DataStorageBytes => ChargeType::DataStorageByteHours
        }
    }
}

#[derive(diesel_derive_enum::DbEnum, Debug, Copy, Clone, Sequence)]
#[DieselTypePath = "crate::schema::sql_types::Timechargetype"]
#[DbValueStyle = "verbatim"]
pub enum TimeChargeType {
    DataStorageBytes,
}
impl From<ChargeType> for Option<TimeChargeType> {
    fn from(charge_type: ChargeType) -> Self {
        match charge_type {
            ChargeType::DataStorageByteHours => Some(TimeChargeType::DataStorageBytes),
            _ => None,
        }
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
#[derive(QueryableByName, Debug)]
pub struct LastTimeCharge {
    #[diesel(sql_type = crate::schema::sql_types::Chargetype)]
    pub charge_type: ChargeType,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub charge_time: DateTime<Utc>,
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

    pub fn from_reports(conn: &mut PgConnection, reports: Vec<ReportToCharge>) -> Result<Vec<Charge>> {
        let mut user2type2charge: HashMap<Option<Uuid>, HashMap<ChargeType, NewCharge>> = HashMap::new();
        for report in reports {
            let type2charge = user2type2charge
                .entry(report.user_id.clone())
                .or_insert(HashMap::new());
            Self::append_report(type2charge, &report);
        }
        user2type2charge
            .values()
            .flat_map(|hashmap| hashmap.values())
            .map(|new_charge| new_charge.commit(conn))
            .collect::<Result<Vec<Charge>>>()
    }

    pub fn from_timecharges_for_user(conn: &mut PgConnection, user_id: &Uuid) -> Result<Vec<Charge>> {
        let mut created_charges = vec![];
        let last_time_charges = sql_query(r#"
            SELECT charge_type, max(charge_time) as charge_time
            FROM charges
            WHERE user_id = $1
            GROUP BY charge_type
        "#)
            .bind::<diesel::sql_types::Uuid,_>(user_id)
            .load::<LastTimeCharge>(conn)?;
        let mut last_time_map = last_time_charges
            .iter()
            .into_grouping_map_by(|timecharge| timecharge.charge_type)
            .aggregate(|_acc, _key, val| Some(Some(val.charge_time)));
        for charge_type in enum_iterator::all::<ChargeType>() {
            last_time_map.entry(charge_type).or_insert(None);
        }
        for (match_charge_type, last_charge_time) in last_time_map {
            use schema::timecharges::dsl::*;
            let option_timecharge_type: Option<TimeChargeType> = match_charge_type.into();
            if let Some(match_timecharge_type) = option_timecharge_type {
                let tc_query = timecharges
                    .filter(timecharge_type.eq(match_timecharge_type))
                    .order(timecharge_time.asc());
                let tcs;
                if let Some(prev_charge) = last_charge_time {
                    tcs = tc_query
                        .filter(timecharge_time.gt(prev_charge))
                        .load::<TimeCharge>(conn)?;
                } else {
                    tcs = tc_query
                        .load::<TimeCharge>(conn)?;
                }
                let mut opt_prev_charge_time = last_charge_time;
                for tc in tcs {
                    if let Some(prev_charge_time) = opt_prev_charge_time {
                        let new_charge = tc.to_new_charge(prev_charge_time).commit(conn)?;
                        created_charges.push(new_charge);
                    }
                    opt_prev_charge_time = Some(tc.timecharge_time);
                }
                // FIXME: incomplete; need to charge final span from last timecharge to current time
            }

        }
        Ok(created_charges)
    }

    fn append_report(existing_charges: &mut HashMap<ChargeType, NewCharge>, new_report: &ReportToCharge) {
        if new_report.num_bytes == None {
            return;
        }
        if let Some(charge_type) = Self::report_charge_type(new_report) {
            let existing = existing_charges.get_mut(&charge_type);
            match existing {
                Some(charge) => {
                    charge.quantity += new_report.num_bytes.unwrap() as f64;
                    charge.report_ids.as_mut().unwrap().push(new_report.report_id);
                },
                None => {
                    let charge = NewCharge {
                        user_id: new_report.user_id.unwrap(), // FIXME
                        charge_type,
                        quantity: new_report.num_bytes.unwrap() as f64,
                        rate: 0.1, // FIXME
                        report_ids: Some(vec![new_report.report_id])
                    };
                    existing_charges.insert(charge_type, charge);
                }
            }
        }
    }

    fn report_charge_type(report: &ReportToCharge) -> Option<ChargeType> {
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
    pub quantity: f64,
}
impl TimeCharge {
    pub fn to_new_charge(&self, prev_charge_time: DateTime<Utc>) -> NewCharge {
        // calculate in seconds, store a s horus
        let charge_quantity =
            self.quantity
                * (self.timecharge_time - prev_charge_time).num_seconds() as f64
                / 3600.0;
        NewCharge::new(
            self.user_id.clone(),
            self.timecharge_type.into(),
            charge_quantity,
            1.0, // FIXME
            None
        )
    }
}