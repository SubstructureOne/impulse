use anyhow::{Result};
use uuid::Uuid;

use impulse::models::users::*;
use crate::common::ExpectedEquals;

mod common;

impl ExpectedEquals for User {
    fn expected_equals(&self, other: &Self) -> bool {
        // ignore created_at and updated_at
        self.user_id == other.user_id
            && self.pg_name == other.pg_name
            && self.user_status == other.user_status
            && self.balance == other.balance
            && self.status_synced == other.status_synced
    }
}

#[test]
pub fn create_user_test() -> Result<()> {
    let context = common::TestContext::new("create_user")?;
    let mut conn = context.config.pg_connect_db(&context.db_name)?;
    let user_id = Uuid::new_v4();
    let pg_name = "myusertest";
    let balance = 3.35;
    let user_status = UserStatus::Active;
    let new_user = NewUser::create(
        &mut conn,
        user_id.clone(),
        pg_name.to_string(),
        balance,
    )?;
    let expected_user = User {
        user_id: user_id.clone(),
        pg_name: pg_name.to_string(),
        user_status,
        balance,
        status_synced: false,
        created_at: chrono::offset::Utc::now(),
        updated_at: chrono::offset::Utc::now(),
    };
    assert!(new_user.expected_equals(&expected_user));
    let retrieved = User::retrieve(&mut conn, &user_id)?;
    assert_eq!(&retrieved, &new_user);
    Ok(())
}

#[test]
pub fn disable_user_test() -> Result<()> {
    let context = common::TestContext::new("disable_user")?;
    let mut conn = context.config.pg_connect_db(&context.db_name)?;
    let mut user = NewUser::create(
        &mut conn,
        Uuid::new_v4(),
        "my_test_user".to_string(),
        100.,
    )?;
    assert_eq!(user.user_status, UserStatus::Active);
    user.disable(&mut conn)?;
    assert_eq!(user.user_status, UserStatus::Disabled);
    let retrieved = User::retrieve(&mut conn, &user.user_id)?;
    assert_eq!(&retrieved, &user);
    Ok(())
}
