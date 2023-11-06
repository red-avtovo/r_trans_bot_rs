use chrono::NaiveDateTime;
use teloxide::prelude::ChatId;
use teloxide::types::Recipient;

use crate::schema::users;

#[derive(Queryable, PartialEq, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub chat: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub salt: String,
    pub created_at: NaiveDateTime,
}

impl Into<Recipient> for User {
    fn into(self) -> Recipient {
        Recipient::Id(ChatId(self.chat))
    }
}

#[derive(Insertable, Clone)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: i64,
    pub chat: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub salt: String,
}
