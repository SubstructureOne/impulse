use std::collections::HashMap;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{debug_query, sql_query};
use diesel::pg::Pg;
use enum_iterator::Sequence;
use itertools::Itertools;
use log::{trace};
use uuid::{Uuid};

use crate::models::reports::{PacketDirection, ReportToCharge};
use crate::schema;
use crate::schema::charges;
use crate::schema::timecharges;
use crate::models::reports::Report;


#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Eq, Hash, Copy, Clone, Sequence)]
#[ExistingTypePath = "crate::schema::sql_types::Chargetype"]
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
impl ChargeType {
    pub fn rate(&self) -> f64 {
        match self {
            ChargeType::DataTransferInBytes => 0.0,
            ChargeType::DataTransferOutBytes => 1.5e-15,  // $.15/Gb
            ChargeType::DataStorageByteHours => 2.0534e-13,  // $.15/Gb*mo
        }
    }
}

#[derive(diesel_derive_enum::DbEnum, Debug, Copy, Clone, Sequence)]
#[ExistingTypePath = "crate::schema::sql_types::Timechargetype"]
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
pub struct LastChargeTime {
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

    pub fn from_timecharges_for_user(
        conn: &mut PgConnection,
        match_user_id: &Uuid,
        final_charge_time: Option<DateTime<Utc>>
    ) -> Result<Vec<Charge>> {
        let mut created_charges = vec![];
        // Data we need for this function:
        //
        //   1. the most recent charge (if it exists) corresponding to each
        //      timecharge type
        //   2. the most recent timecharge from *before* the most
        //      recent charge, to charge the first period
        //   3. all the timecharges (if any) created since the most recent
        //      charge (if it exists), or else all timecharges (if it doesn't)
        //   4. the most recent timecharge, to charge the last period
        //
        // 2 will either be identical to #4 (if there are no
        // timecharges since the most recent charge), or it will be an
        // element of #3.
        //
        // Obviously if there are no timecharges of a given type then there is
        // no charge to be created for the corresponding ChargeType of that
        // TimeChargeType.
        let last_charges = sql_query(r#"
            SELECT charge_type, max(charge_time) as charge_time
            FROM charges
            WHERE user_id = $1
            GROUP BY charge_type
        "#)
            .bind::<diesel::sql_types::Uuid,_>(match_user_id)
            .load::<LastChargeTime>(conn)?;
        let last_charge_time_map = last_charges
            .iter()
            .into_grouping_map_by(|chargetime| chargetime.charge_type)
            .aggregate(|_acc, _key, val| Some(val.charge_time));
        for match_timecharge_type in enum_iterator::all::<TimeChargeType>() {
            use schema::timecharges::dsl::*;
            let mut opt_prev_charge_time = last_charge_time_map.get(&match_timecharge_type.into());
            let tc_query = timecharges
                .filter(user_id.eq(match_user_id))
                .filter(timecharge_type.eq(match_timecharge_type))
                .order(timecharge_time.asc());
            let tcs: Vec<TimeCharge> = match opt_prev_charge_time {
                Some(prev_charge) => tc_query
                    .filter(timecharge_time.gt(prev_charge))
                    .load::<TimeCharge>(conn)?,
                None => tc_query.load::<TimeCharge>(conn)?,
            };
            let mut opt_prev_timecharge: Option<TimeCharge> = match opt_prev_charge_time {
                Some(last_charge_time) => timecharges
                    .filter(user_id.eq(match_user_id))
                    .filter(timecharge_time.le(last_charge_time))
                    .order(timecharge_time.desc())
                    .first::<TimeCharge>(conn)
                    .optional()?,
                None => None
            };
            for tc in tcs.iter() {
                if let (Some(prev_charge_time), Some(prev_timecharge))
                    = (opt_prev_charge_time, opt_prev_timecharge)
                {
                    let new_charge = prev_timecharge
                        .to_new_charge(
                            prev_charge_time,
                            &tc.timecharge_time,
                            final_charge_time)?
                        .commit(conn)?;
                    created_charges.push(new_charge);
                }
                opt_prev_charge_time = Some(&tc.timecharge_time);
                opt_prev_timecharge = Some(tc.clone());
            }
            // calculate final charge representing most recent timecharge to
            // current time
            if let (Some(last_charge_time), Some(last_timecharge))
                = (opt_prev_charge_time, opt_prev_timecharge)
            {
                created_charges.push(
                    last_timecharge
                            .to_new_charge(
                                last_charge_time,
                                &final_charge_time.unwrap_or(Utc::now()),
                                final_charge_time,
                            )?
                            .commit(conn)?
                );
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
                    // some "charges" will have the user unassigned; e.g.,
                    // all Postgres bytes transferred before user auth. We
                    // still want to "charge" these to an account, so we
                    // charge them to the "postgres" user, representing the
                    // system administrator, who is the defined to be the nil
                    // UUID.
                    let charge = NewCharge::new(
                        new_report.user_id.unwrap_or_else(|| Uuid::nil()),
                        charge_type,
                        new_report.num_bytes.unwrap() as f64,
                        charge_type.rate(),
                        Some(vec![new_report.report_id]),
                        None,
                    );
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
    pub charge_time: Option<DateTime<Utc>>,
}

impl NewCharge {
    pub fn new(
        user_id: Uuid,
        charge_type: ChargeType,
        quantity: f64,
        rate: f64,
        report_ids: Option<Vec<i64>>,
        charge_time: Option<DateTime<Utc>>,
    ) -> NewCharge {
        NewCharge {
            user_id,
            charge_type,
            quantity,
            rate,
            report_ids,
            charge_time,
        }
    }

    pub fn commit(&self, conn: &mut PgConnection) -> Result<Charge> {
        let query = diesel::insert_into(charges::table)
            .values(self);
        trace!("Creating charge: {}", debug_query::<Pg, _>(&query));
        let result: Charge = query.get_result::<Charge_>(conn)?.into();
        if let Some(reports) = &result.report_ids {
            reports
                .iter()
                .map(|report_id| Report::mark_charged(*report_id, conn))
                .collect::<Result<Vec<_>, _>>()?;
        }
        Ok(result)
    }
}

#[derive(Queryable, Debug, Clone)]
pub struct TimeCharge {
    pub timecharge_id: i64,
    pub timecharge_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub timecharge_type: TimeChargeType,
    pub quantity: f64,
}
impl TimeCharge {
    pub fn to_new_charge(
        &self,
        charge_starttime: &DateTime<Utc>,
        charge_endtime: &DateTime<Utc>,
        charge_time: Option<DateTime<Utc>>,
    ) -> Result<NewCharge> {
        if charge_starttime < &self.timecharge_time {
            return Err(anyhow!(
                "Charge start time ({}) cannot be before timecharge time ({})",
                &charge_starttime,
                &self.timecharge_time
            ));
        }
        if charge_endtime < charge_starttime {
            return Err(anyhow!(
                "Charge endtime ({}) cannot be before charge starttime ({})",
                &charge_endtime,
                &charge_starttime,
            ))
        }
        // calculate in seconds, store as horus
        let charge_quantity =
            self.quantity
                * (*charge_endtime - *charge_starttime).num_seconds() as f64
                / 3600.0;
        let charge_type: ChargeType = self.timecharge_type.into();
        Ok(NewCharge::new(
            self.user_id.clone(),
            self.timecharge_type.into(),
            charge_quantity,
            charge_type.rate(),
            None,
            charge_time,
        ))
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = timecharges)]
pub struct NewTimeCharge {
    pub user_id: Uuid,
    pub timecharge_time: Option<DateTime<Utc>>,
    pub timecharge_type: TimeChargeType,
    pub quantity: f64,
}
impl NewTimeCharge {
    pub fn create(
        user_id: Uuid,
        timecharge_time: Option<DateTime<Utc>>,
        timecharge_type: TimeChargeType,
        quantity: f64
    ) -> NewTimeCharge {
        NewTimeCharge {
            user_id,
            timecharge_time,
            timecharge_type,
            quantity
        }
    }

    pub fn commit(&self, conn: &mut PgConnection) -> Result<TimeCharge> {
        Ok(
            diesel::insert_into(timecharges::table)
                .values(self)
                .get_result::<TimeCharge>(conn)?
        )
    }
}