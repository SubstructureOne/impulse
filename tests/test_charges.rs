use anyhow::{Result};
use chrono::{Duration, TimeZone, Utc};
use log::{debug, trace};
use uuid::Uuid;

use impulse::models::charges::*;
use impulse::models::reports;
use impulse::models::reports::{PacketDirection, PostgresqlPacketType};
use crate::common::ExpectedEquals;

mod common;

impl ExpectedEquals for Charge {
    fn expected_equals(&self, other: &Self) -> bool {
        // ignore charge_id
        self.charge_time.expected_equals(&other.charge_time)
            && self.user_id == other.user_id
            && self.charge_type == other.charge_type
            && self.quantity == other.quantity
            && self.rate == other.rate
            && self.amount == other.amount
            && self.report_ids == other.report_ids
            && self.transacted == other.transacted
    }
}


#[test]
fn create_charge_test() -> Result<()> {
    log::info!("Starting test");
    let context = common::TestContext::new("create_charge")?;
    let mut conn = context.impulse_manager.pg_connect_db(&context.db_name)?;
    let user_id = Uuid::new_v4();
    let quantity = 1024.;
    let rate = 0.00015;
    let charge_type = ChargeType::DataTransferInBytes;
    let report_ids = Some(vec![1, 2, 3]);

    let expected_charge = Charge {
        charge_id: 0,
        charge_time: chrono::offset::Utc::now(),
        user_id: user_id.clone(),
        charge_type,
        quantity,
        rate,
        amount: rate * quantity,
        report_ids: report_ids.clone(),
        transacted: false,
    };
    trace!("Expected charge: {:?}", &expected_charge);

    let charge = NewCharge::new(
        user_id,
        ChargeType::DataTransferInBytes,
        quantity,
        rate,
        report_ids.clone(),
        None,
    ).commit(&mut conn)?;
    trace!("New charge: {:?}", &charge);
    assert!(charge.expected_equals(&expected_charge));

    let retrieved = Charge::retrieve(&mut conn, charge.charge_id)?;
    trace!("Retrieved charge: {:?}", &retrieved);
    assert_eq!(&charge, &retrieved);
    Ok(())
}

#[test]
fn create_from_timecharges_test() -> Result<()> {
    let context = common::TestContext::new("create_from_timecharges")?;
    let mut conn = context.impulse_manager.pg_connect_db(&context.db_name)?;
    let user_id = Uuid::new_v4();

    // situation 1: a single timecharge and no existing charges
    let timecharge1_time = Utc.with_ymd_and_hms(2022, 1, 1, 12, 0, 0).unwrap();
    let timecharge1_type = TimeChargeType::DataStorageBytes;
    let quantity1 = 10.0;
    let timecharge1 = NewTimeCharge::create(
        user_id,
        Some(timecharge1_time),
        timecharge1_type,
        quantity1
    ).commit(&mut conn)?;
    let charge1_time = timecharge1_time + Duration::minutes(10);
    let created_charges1 = Charge::from_timecharges_for_user(
        &mut conn,
        &user_id,
        Some(charge1_time),
    )?;
    assert_eq!(created_charges1.len(), 1);
    let expected_quantity = quantity1 * (charge1_time - timecharge1_time).num_seconds() as f64 / 3600.0;
    let created_charge1 = &created_charges1[0];
    let expected_chargetype = ChargeType::DataStorageByteHours;
    let expected_rate = expected_chargetype.rate();
    let expected_charge1 = Charge {
        charge_id: 0,
        charge_time: charge1_time,
        user_id,
        charge_type: expected_chargetype,
        quantity: expected_quantity,
        rate: expected_rate,
        amount: expected_quantity * expected_rate,
        report_ids: None,
        transacted: false
    };
    debug!("Expecting {:?} to roughly equal {:?}", &created_charge1, &expected_charge1);
    assert!(created_charge1.expected_equals(&expected_charge1));

    // situation 2: one existing timecharge and an existing charge
    let charge2_time = charge1_time + Duration::minutes(10);
    let created_charges2 = Charge::from_timecharges_for_user(
        &mut conn,
        &user_id,
        Some(charge2_time)
    )?;
    assert_eq!(created_charges2.len(), 1);
    let created_charge2 = &created_charges2[0];
    // note we use the same "quantity" here because we still have only 1
    // TimeCharge (i.e., we had 10 bytes before and we still have 10 bytes
    // now).
    let expected_quantity2 = quantity1 * (charge2_time - charge1_time).num_seconds() as f64 / 3600.0;
    let expected_charge2 = Charge {
        charge_id: 0,
        charge_time: charge2_time,
        user_id,
        charge_type: ChargeType::DataStorageByteHours,
        quantity: expected_quantity2,
        rate: expected_rate,
        amount: expected_quantity2 * expected_rate,
        report_ids: None,
        transacted: false,
    };
    debug!("Expecting {:?} to roughly equal {:?}", &created_charge2, &expected_charge2);
    assert!(created_charge2.expected_equals(&expected_charge2));
    Ok(())
}

#[test]
fn rate_test() -> Result<()> {
    let context = common::TestContext::new("rate_test")?;
    let mut conn = context.impulse_manager.pg_connect_db(&context.db_name)?;
    let username = Some(String::from("username"));
    let packet_type = PostgresqlPacketType::Other;
    let direction = Some(PacketDirection::Forward);
    let packet_info = None;
    let packet_bytes = Some(vec![1, 2, 3, 4]);
    let charged = false;
    let report = reports::NewReport::create(
        username,
        packet_type,
        direction,
        packet_info,
        packet_bytes,
        charged
    ).commit(&mut conn)?;
    let report_id = report.report_id;
    let userid = Uuid::new_v4();
    let reports: Vec<reports::ReportToCharge> = vec![reports::ReportToCharge::with_userid(report, userid)];
    let charges = Charge::from_reports(&mut conn, reports)?;
    assert_eq!(charges.len(), 1);
    let charge = &charges[0];
    let expected_charge = Charge {
        charge_id: 0,
        charge_time: Utc::now(),
        user_id: userid,
        charge_type: ChargeType::DataTransferInBytes,
        quantity: 4.0,
        rate: ChargeType::DataTransferInBytes.rate(),
        amount: 4.0 * ChargeType::DataTransferInBytes.rate(),
        report_ids: Some(vec![report_id]),
        transacted: false,
    };
    assert!(charge.expected_equals(&expected_charge));
    Ok(())
}
