mod common;

use anyhow::{Result};
use uuid::Uuid;
use impulse::models::charges::{ChargeType, NewCharge};

use impulse::models::transactions::*;
use impulse::models::transactions::NewTransaction;

use crate::common::ExpectedEquals;


impl ExpectedEquals for Transaction {
    fn expected_equals(&self, other: &Self) -> bool {
        // ignore transaction_id, allow tolerance for txn time
        self.amount == other.amount
            && self.charge_ids == other.charge_ids
            && self.to_user == other.to_user
            && self.from_user == other.from_user
            && self.txn_time.expected_equals(&other.txn_time)
    }
}

impl ExpectedEquals for ExtTransaction {
    fn expected_equals(&self, other: &Self) -> bool {
        // ignore exttransaction_id, allow tolerance for txn time
        self.user_id == other.user_id
            && self.amount == other.amount
            && self.exttransaction_time.expected_equals(&other.exttransaction_time)
    }
}

#[test]
fn create_transaction_test() -> Result<()> {
    let context = common::TestContext::new("create_txn")?;
    let mut conn = context.manager.pg_connect_db(&context.db_name)?;
    let from_user = Uuid::new_v4();
    let to_user = Uuid::new_v4();
    let txn_charge_ids = vec![1,5,7];
    let amount = 300.;
    let expected_time = chrono::offset::Utc::now();

    let expected_txn = Transaction {
        transaction_id: 0,
        txn_time: expected_time,
        from_user: from_user.clone(),
        to_user: to_user.clone(),
        charge_ids: Some(txn_charge_ids.clone()),
        amount,
    };

    let new_txn = NewTransaction::create(
        &mut conn,
        from_user.clone(),
        to_user.clone(),
        Some(txn_charge_ids.clone()),
        amount,
        None,
    )?;
    assert!(&expected_txn.expected_equals(&new_txn));

    let retrieved = Transaction::retrieve(&mut conn, new_txn.transaction_id)?;
    assert_eq!(&retrieved, &new_txn);
    Ok(())
}

#[test]
fn create_txn_from_charges_test() -> Result<()> {
    let context = common::TestContext::new("create_txn_from_charges")?;
    let mut conn = context.manager.pg_connect_db(&context.db_name)?;
    let from_user_id = Uuid::new_v4();
    let charge_type = ChargeType::DataTransferInBytes;
    let quantity1 = 2.5;
    let rate1 = 3.4;
    let report_ids = None;
    let charge1 = NewCharge::new(
        from_user_id,
        charge_type,
        quantity1,
        rate1,
        report_ids.clone()
    ).commit(&mut conn)?;
    let quantity2 = 5.3;
    let rate2 = 1.8;
    let charge2 = NewCharge::new(
        from_user_id,
        charge_type,
        quantity2,
        rate2,
        report_ids.clone()
    ).commit(&mut conn)?;
    let expected_charge_ids = vec![charge1.charge_id, charge2.charge_id];
    let charges = vec![charge1, charge2];
    let to_user_id = Uuid::nil();
    let transactions = NewTransaction::from_charges(
        &mut conn,
        &charges,
    )?;
    let expected_amount = rate1*quantity1 + rate2*quantity2;
    let now = chrono::offset::Utc::now();
    assert_eq!(transactions.len(), 1);
    let transaction = &transactions[0];
    assert!(transaction.txn_time.expected_equals(&now));
    assert_eq!(transaction.charge_ids, Some(expected_charge_ids));
    assert_eq!(&transaction.from_user, &from_user_id);
    assert_eq!(transaction.amount, expected_amount);
    assert_eq!(&transaction.to_user, &to_user_id);
    Ok(())
}

#[test]
fn create_exttransaction_test() -> Result<()> {
    let context = common::TestContext::new("create_exttransaction")?;
    let mut conn = context.manager.pg_connect_db(&context.db_name)?;
    let user_id = Uuid::new_v4();
    let amount = 154.3;
    let expected_txn = ExtTransaction {
        exttransaction_id: 0,
        user_id: user_id.clone(),
        amount,
        exttransaction_time: chrono::offset::Utc::now(),
    };
    let new_txn = NewExtTransaction::create(
        &mut conn,
        user_id.clone(),
        amount,
        None
    )?;
    assert!(&expected_txn.expected_equals(&new_txn));
    let retrieved = ExtTransaction::retrieve(
        &mut conn,
        new_txn.exttransaction_id
    )?;
    assert_eq!(&retrieved, &new_txn);
    Ok(())
}
