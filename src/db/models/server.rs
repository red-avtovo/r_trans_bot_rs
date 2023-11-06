use crate::core::trans_url::TransUrl;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::schema::servers;

#[derive(Insertable)]
#[diesel(table_name = servers)]
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
        self.username
            .clone()
            .map(|u| {
                self.password.clone().map(|p| Authentication {
                    username: u,
                    password: p,
                })
            })
            .flatten()
    }
}

impl NewServer {
    pub fn new(user_id: u64, url: String, auth: Option<Authentication>) -> Self {
        let username = auth.clone().map(|a| a.username);
        let password = auth.map(|a| a.password);
        NewServer {
            id: Uuid::new_v4(),
            user_id: user_id as i64,
            url,
            username,
            password,
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
    pub created_at: NaiveDateTime,
}

impl Server {
    pub fn url(self: &Self) -> TransUrl {
        TransUrl::from(self.url.clone())
    }

    pub fn auth(self: &Self) -> Option<Authentication> {
        self.username
            .clone()
            .map(|u| {
                self.password.clone().map(|p| Authentication {
                    username: u,
                    password: p,
                })
            })
            .flatten()
    }
}

#[derive(Debug, Clone)]
pub struct Authentication {
    pub username: String,
    pub password: String,
}
