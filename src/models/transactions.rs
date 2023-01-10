use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{transactions, exttransactions};


#[derive(Queryable, Debug, PartialEq)]
pub struct ExtTransaction {
    pub exttransaction_id: i64,
    pub user_id: Uuid,
    pub amount: f64,
    pub exttransaction_time: DateTime<Utc>,
}
impl ExtTransaction {
    pub fn retrieve(conn: &mut PgConnection, exttransaction_id_: i64) -> Result<ExtTransaction> {
        use crate::schema::exttransactions::dsl::*;
        Ok(
            exttransactions
                .find(&exttransaction_id_)
                .first::<ExtTransaction>(conn)?
        )
    }
}
#[derive(Insertable, Debug)]
#[diesel(table_name = exttransactions)]
pub struct NewExtTransaction {
    pub user_id: Uuid,
    pub amount: f64,
    pub exttransaction_time: Option<DateTime<Utc>>,
}
impl NewExtTransaction {
    pub fn create(
        conn: &mut PgConnection,
        user_id: Uuid,
        amount: f64,
        exttransaction_time: Option<DateTime<Utc>>
    ) -> Result<ExtTransaction> {
        let new_txn = NewExtTransaction {
            user_id,
            amount,
            exttransaction_time,
        };
        Ok(
            diesel::insert_into(exttransactions::table)
                .values(&new_txn)
                .get_result::<ExtTransaction>(conn)?
        )
    }
}

#[derive(Queryable, Debug)]
pub struct Transaction_ {
    pub transaction_id: i64,
    pub txn_time: DateTime<Utc>,
    pub from_user: Uuid,
    pub to_user: Uuid,
    pub charge_ids: Option<Vec<Option<i64>>>,
    pub amount: f64,
}
#[derive(PartialEq, Debug)]
pub struct Transaction {
    pub transaction_id: i64,
    pub txn_time: DateTime<Utc>,
    pub from_user: Uuid,
    pub to_user: Uuid,
    pub charge_ids: Option<Vec<i64>>,
    pub amount: f64,
}
impl Transaction {
    pub fn retrieve(conn: &mut PgConnection, txn_id_: i64) -> Result<Transaction> {
        use crate::schema::transactions::dsl::*;
        Ok(
            transactions
                .find(&txn_id_)
                .first::<Transaction_>(conn)?
                .into()
        )
    }
}
impl From<Transaction_> for Transaction {
    fn from(txn_: Transaction_) -> Self {
        let charge_ids = match txn_.charge_ids {
            Some(ids_vec) => Some(
                ids_vec
                    .into_iter()
                    .filter(|&x| !x.is_none())
                    .map(|x| x.unwrap())
                    .collect::<Vec<_>>()
            ),
            None => None
        };
        Transaction {
            transaction_id: txn_.transaction_id,
            txn_time: txn_.txn_time,
            from_user: txn_.from_user,
            to_user: txn_.to_user,
            charge_ids,
            amount: txn_.amount,
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub from_user: Uuid,
    pub to_user: Uuid,
    pub charge_ids: Option<Vec<i64>>,
    pub amount: f64,
    pub txn_time: Option<DateTime<Utc>>,
}
impl NewTransaction {
    pub fn create(
        conn: &mut PgConnection,
        from_user: Uuid,
        to_user: Uuid,
        charge_ids: Option<Vec<i64>>,
        amount: f64,
        txn_time: Option<DateTime<Utc>>,
    ) -> Result<Transaction> {
        let new_txn = NewTransaction {
            from_user,
            to_user,
            charge_ids,
            amount,
            txn_time,
        };
        Ok(
            diesel::insert_into(transactions::table)
                .values(&new_txn)
                .get_result::<Transaction_>(conn)?
                .into()
        )
    }
}
