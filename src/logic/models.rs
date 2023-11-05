use postgres_types::{ToSql, FromSql};
use uuid::Uuid;
use chrono::NaiveDateTime;
use super::trans_url::TransUrl;

use crate::schema::{
    users,
    dirs,
    tasks,
    servers,
    magnets,
};

#[derive(Queryable, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub chat: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub salt: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Clone)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i64, //Can't be changed to u64 because of diesel
    pub chat: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub salt: String,
}

#[derive(Queryable, Clone, Debug)]
pub struct DownloadDirectory {
    pub id: Uuid,
    pub user_id: i64,
    pub alias: String,
    pub path: String,
    pub ordinal: i32,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name = "dirs"]
pub struct NewDownloadDirectory {
    id: Uuid,
    user_id: i64,
    alias: String,
    path: String,
    ordinal: i32,
}

impl NewDownloadDirectory {
    pub fn new( user_id: i64,
            alias: String,
            path: String,
            ordinal: i32,) -> Self {
            NewDownloadDirectory {
                id: Uuid::new_v4(),
                user_id,
                alias,
                path,
                ordinal,
            }
        }
}

#[derive(Queryable, Clone, Debug)]
pub struct DownloadTask {
    pub id: Uuid,
    pub user_id: i64,
    pub server_id: Uuid,
    pub magnet_id: Uuid,
    pub status: String,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl DownloadTask {
    #[allow(dead_code)]
    pub fn status(self: &Self) -> TaskStatus {
        TaskStatus::from(self.status.clone())
    }
}

#[derive(Insertable)]
#[table_name = "tasks"]
pub struct NewDownloadTask {
    id: Uuid,
    user_id: i64,
    server_id: Uuid,
    magnet_id: Uuid,
    status: String,
    description: Option<String>,
}

impl NewDownloadTask {
    pub fn new( user_id: i64,
            server_id: Uuid,
            magnet_id: Uuid,
            status: String,
            description: Option<String>,) -> Self {
        NewDownloadTask{
            id: Uuid::new_v4(),
            user_id,
            server_id,
            magnet_id,
            status,
            description,
        }
    }
}

#[derive(Queryable, Clone, Debug)]
pub struct Server {
    pub id: Uuid,
    pub user_id: i64,
    pub(crate) url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl Server {
    pub fn url(self: &Self) -> TransUrl {
        TransUrl::from(self.url.clone())
    }

    pub fn auth(self: &Self) -> Option<Authentication> {
        self.username.clone().map(|u| {
            self.password.clone().map(|p| Authentication{
                username: u,
                password: p,
            })
        }).flatten()
    }
}

#[derive(Insertable)]
#[table_name = "servers"]
pub struct NewServer {
    id: Uuid,
    user_id: i64,
    url: String,
    username: Option<String>,
    password: Option<String>,
}

impl NewServer {
    pub fn url(self: &Self) -> TransUrl {
        TransUrl::from(self.url.clone())
    }

    pub fn auth(self: &Self) -> Option<Authentication> {
        self.username.clone().map(|u| {
            self.password.clone().map(|p| Authentication{
                username: u,
                password: p,
            })
        }).flatten()
    }
}

impl NewServer {
    pub fn new(
        user_id: i64,
        url: String,
        auth: Option<Authentication>
    ) -> Self {
        let username = auth.clone().map(|a| a.username);
        let password = auth.map(|a| a.password);
        NewServer {
            id: Uuid::new_v4(),
            user_id,
            url,
            username,
            password,
        }
    }
}

#[derive(Queryable, Clone, Debug)]
pub struct Magnet {
    pub id: Uuid,
    pub user_id: i64,
    pub url: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name = "magnets"]
pub struct NewMagnet {
    id: Uuid,
    user_id: i64,
    url: String,
}

impl NewMagnet {
    pub fn new( user_id: i64,
            url: String,) -> Self {
            NewMagnet{
                id: Uuid::new_v4(),
                user_id,
                url,
            }
        }
}

#[derive(Debug, Clone)]
pub struct Authentication {
    pub username: String,
    pub password: String,
}

#[derive(Debug, ToSql, FromSql, Clone)]
pub enum TaskStatus{
    Created,
    Started,
    Finished,
    Error
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
            _  => panic!()
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
        }.to_owned()
    }
}
