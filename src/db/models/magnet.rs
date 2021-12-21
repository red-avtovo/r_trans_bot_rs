use crate::schema::magnets;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Clone, Debug)]
pub struct Magnet {
    pub id: Uuid,
    pub user_id: i64,
    pub url: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = magnets)]
pub struct NewMagnet {
    id: Uuid,
    user_id: i64,
    url: String,
}

impl NewMagnet {
    pub fn new(user_id: i64, url: String) -> Self {
        NewMagnet {
            id: Uuid::new_v4(),
            user_id,
            url,
        }
    }
}
