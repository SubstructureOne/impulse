use diesel::prelude::*;

use impulse::models::*;

mod common;

#[test]
fn create_report_test() {
    let context = common::TestContext::new("create_report")
        .expect("Couldn't establish test context");
    let mut conn = context.connect().expect("Couldn't connect to db");
    let report = create_report(
        &mut conn,
        Some("MyUser".to_string()),
        PostgresqlPacketType::Other,
        Some(PacketDirection::Backward),
        None,
        None,
        false
    ).expect("Couldn't create report");
}
