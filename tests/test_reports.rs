use anyhow::{Result};

use impulse::models::reports::*;

mod common;

#[test]
fn create_report_test() -> Result<()> {
    let context = common::TestContext::new("create_report")?;
    let mut conn = context.config.pg_connect_db(&context.db_name)?;
    let username = Some("MyUser".to_string());
    let packet_type = PostgresqlPacketType::Other;
    let direction = Some(PacketDirection::Backward);
    let packet_info_json = r#"{
        "protocol_version": 196608,
        "parameters": [
            ["user", "postgres"],
            ["database", "testdb"]
        ]
    }"#;
    let packet_info: Option<serde_json::Value> = Some(
        serde_json::from_str(packet_info_json)?
    );
    let packet_bytes = None;
    let charged = false;
    let report = NewReport::create(
        &mut conn,
        username.clone(),
        packet_type,
        direction,
        packet_info.clone(),
        packet_bytes.clone(),
        charged
    )?;
    assert_eq!(&report.username, &username);
    assert_eq!(report.packet_type, packet_type);
    assert_eq!(report.direction, direction);
    assert_eq!(&report.packet_info, &packet_info);
    assert_eq!(&report.packet_bytes, &packet_bytes);
    assert_eq!(report.charged, charged);
    let uncharged = Report::uncharged(&mut conn)?;
    assert_eq!(uncharged.len(), 1);
    assert_eq!(&uncharged[0], &report);
    Ok(())
}
