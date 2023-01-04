// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "chargetype"))]
    pub struct Chargetype;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pgpkttype"))]
    pub struct Pgpkttype;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pktdirection"))]
    pub struct Pktdirection;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "timechargetype"))]
    pub struct Timechargetype;
}

diesel::table! {
    balances (user_id) {
        user_id -> Uuid,
        balance -> Float8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Chargetype;

    charges (charge_id) {
        charge_id -> Int8,
        charge_time -> Timestamptz,
        user_id -> Uuid,
        charge_type -> Chargetype,
        quantity -> Float8,
        rate -> Float8,
        amount -> Float8,
        report_ids -> Nullable<Array<Nullable<Int8>>>,
        transacted -> Bool,
    }
}

diesel::table! {
    exttransactions (exttransaction_id) {
        exttransaction_id -> Int8,
        user_id -> Uuid,
        amount -> Float8,
        exttransaction_time -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Pgpkttype;
    use super::sql_types::Pktdirection;

    reports (packet_id) {
        packet_id -> Int8,
        username -> Nullable<Text>,
        packet_type -> Pgpkttype,
        packet_time -> Timestamptz,
        direction -> Nullable<Pktdirection>,
        packet_info -> Nullable<Jsonb>,
        packet_bytes -> Nullable<Bytea>,
        charged -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Timechargetype;

    timecharges (timecharge_id) {
        timecharge_id -> Int8,
        timecharge_time -> Timestamptz,
        user_id -> Uuid,
        timecharge_type -> Timechargetype,
        quantity -> Float8,
    }
}

diesel::table! {
    transactions (txn_id) {
        txn_id -> Int8,
        txn_time -> Timestamptz,
        from_user -> Uuid,
        to_user -> Uuid,
        charge_ids -> Nullable<Array<Nullable<Int8>>>,
        amount -> Float8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    charges,
    exttransactions,
    reports,
    timecharges,
    transactions,
);
