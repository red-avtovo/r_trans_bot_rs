use crate::schema::dirs;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Clone, Debug)]
pub struct DownloadDirectory {
    pub id: Uuid,
    pub user_id: i64,
    pub alias: String,
    pub path: String,
    pub ordinal: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = dirs)]
pub struct NewDownloadDirectory {
    id: Uuid,
    user_id: i64,
    alias: String,
    path: String,
    ordinal: i32,
}

impl NewDownloadDirectory {
    pub fn new(user_id: i64, alias: String, path: String, ordinal: i32) -> Self {
        NewDownloadDirectory {
            id: Uuid::new_v4(),
            user_id,
            alias,
            path,
            ordinal,
        }
    }
}
