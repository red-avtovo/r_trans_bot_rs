use crate::schema::tasks;
use chrono::NaiveDateTime;
use postgres_types::{FromSql, ToSql};
use uuid::Uuid;

#[derive(Queryable, Clone, Debug)]
pub struct DownloadTask {
    pub id: Uuid,
    pub user_id: i64,
    pub server_id: Uuid,
    pub magnet_id: Uuid,
    pub status: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

impl DownloadTask {
    #[allow(dead_code)]
    pub fn status(self: &Self) -> TaskStatus {
        TaskStatus::from(self.status.clone())
    }
}

#[derive(Insertable)]
#[diesel(table_name = tasks)]
pub struct NewDownloadTask {
    id: Uuid,
    user_id: i64,
    server_id: Uuid,
    magnet_id: Uuid,
    status: String,
    description: Option<String>,
}

impl NewDownloadTask {
    pub fn new(
        user_id: i64,
        server_id: Uuid,
        magnet_id: Uuid,
        status: String,
        description: Option<String>,
    ) -> Self {
        NewDownloadTask {
            id: Uuid::new_v4(),
            user_id,
            server_id,
            magnet_id,
            status,
            description,
        }
    }
}

#[derive(Debug, ToSql, FromSql, Clone)]
pub enum TaskStatus {
    Created,
    Started,
    Finished,
    Error,
}

impl From<diesel::sql_types::Text> for TaskStatus {
    fn from(text: diesel::sql_types::Text) -> Self {
        let s = text.into();
        s
    }
}

impl From<String> for TaskStatus {
    fn from(str: String) -> Self {
        match str.as_ref() {
            "created" => TaskStatus::Created,
            "started" => TaskStatus::Started,
            "finished" => TaskStatus::Finished,
            "error" => TaskStatus::Error,
            _ => panic!(),
        }
    }
}

impl ToString for TaskStatus {
    fn to_string(&self) -> String {
        match self {
            TaskStatus::Created => "created",
            TaskStatus::Started => "started",
            TaskStatus::Finished => "finished",
            TaskStatus::Error => "error",
        }
        .to_owned()
    }
}
