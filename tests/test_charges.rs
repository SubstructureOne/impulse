use anyhow::{Result};
use log::trace;
use uuid::Uuid;

use impulse::models::charges::*;
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
    let mut conn = context.manager.pg_connect_db(&context.db_name)?;
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

    let charge = NewCharge::create(
        &mut conn,
        user_id,
        ChargeType::DataTransferInBytes,
        quantity,
        rate,
        report_ids.clone(),
    )?;
    trace!("New charge: {:?}", &charge);
    assert!(charge.expected_equals(&expected_charge));

    let retrieved = Charge::retrieve(&mut conn, charge.charge_id)?;
    trace!("Retrieved charge: {:?}", &retrieved);
    assert_eq!(&charge, &retrieved);
    Ok(())
}
