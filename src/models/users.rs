use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::users;


#[derive(diesel_derive_enum::DbEnum, Debug, PartialEq, Copy, Clone)]
#[DieselTypePath = "crate::schema::sql_types::Userstatus"]
#[DbValueStyle = "verbatim"]
pub enum UserStatus {
    Active,
    Disabled,
    Deleted
}

#[derive(Queryable, Debug, PartialEq)]
pub struct User {
    pub user_id: Uuid,
    pub pg_name: String,
    pub user_status: UserStatus,
    pub balance: f64,
    pub status_synced: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl User {
    pub fn retrieve(conn: &mut PgConnection, user_id_: &Uuid) -> Result<User>
    {
        use crate::schema::users::dsl::*;
        Ok(
            users
                .find(user_id_)
                .first::<User>(conn)?
                .into()
        )
    }

    pub fn disable(&mut self, conn: &mut PgConnection) -> Result<()> {
        use crate::schema::users::dsl::*;
        let result = diesel::update(users.filter(user_id.eq(&self.user_id)))
            .set(user_status.eq(UserStatus::Disabled))
            .get_result::<User>(conn)?;
        *self = result;
        Ok(())
    }
}
#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub user_id: Uuid,
    pub pg_name: String,
    pub balance: f64,
}
impl NewUser {
    pub fn create(
        conn: &mut PgConnection,
        user_id: Uuid,
        pg_name: String,
        balance: f64,
    ) -> Result<User> {
        let new_user = NewUser {
            user_id,
            pg_name,
            balance,
        };
        Ok(
            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result::<User>(conn)?
        )
    }
}
