use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::schema::friends;

#[derive(Queryable, PartialEq, Clone, Debug)]
pub struct Friend {
    pub id: Uuid,
    pub user_id: i64,
    pub friend_user_id: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = friends)]
pub struct NewFriend {
    pub user_id: i64,
    pub friend_user_id: i64,
}