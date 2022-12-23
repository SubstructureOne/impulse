// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pgpkttype"))]
    pub struct Pgpkttype;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pktdirection"))]
    pub struct Pktdirection;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Pgpkttype;
    use super::sql_types::Pktdirection;

    reports (packet_id) {
        packet_id -> Int8,
        username -> Nullable<Text>,
        packet_type -> Pgpkttype,
        packet_time -> Timestamp,
        direction -> Nullable<Pktdirection>,
        packet_info -> Nullable<Jsonb>,
        packet_bytes -> Nullable<Bytea>,
        charged -> Bool,
    }
}
