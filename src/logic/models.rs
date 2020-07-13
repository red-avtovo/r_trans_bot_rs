use telegram_bot_raw::types::refs::UserId;
use postgres_types::{ToSql, FromSql};
use uuid::Uuid;

#[derive(Debug, ToSql, FromSql, Clone, PartialEq, Eq, Hash)]
pub struct TelegramId(i64);

impl From<i64> for TelegramId {
    fn from(id: i64) -> Self {
        TelegramId(id)
    }
}

impl From<TelegramId> for i64 {
    fn from(id: TelegramId) -> Self {
        id.0
    }
}

impl From<UserId> for TelegramId {
    fn from(id: UserId) -> Self {
        let a: i64 = id.into();
        TelegramId(a)
    }
}

#[derive(Clone, Debug)]
pub struct DbUser {
    pub id: TelegramId,
    pub chat: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub salt: String,
}

#[derive(Clone, Debug)]
pub struct DownloadDirectory {
    pub id: Uuid,
    pub user_id: TelegramId,
    pub ordinal: i32,
    pub alias: String,
    pub path: String
}

#[derive(Clone, Debug)]
pub struct DownloadTask {
    pub id: Uuid,
    pub user_id: TelegramId,
    pub server_id: Uuid,
    pub magnet_id: Uuid,
    pub status: TaskStatus,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Authentication {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub id: Uuid,
    pub user_id: TelegramId,
    pub(crate) url: TransUrl,
    pub auth: Option<Authentication>,
}

#[derive(Debug, ToSql, FromSql, Clone)]
#[postgres(name = "task_status")]
pub enum TaskStatus{
    #[postgres(name = "created")]
    Created,
    #[postgres(name = "started")]
    Started,
    #[postgres(name = "finished")]
    Finished,
    #[postgres(name = "error")]
    Error
}

#[derive(Debug, Clone)]
pub struct Magnet {
    pub id: Uuid,
    pub user_id: TelegramId,
    pub url: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TransUrl(String);

impl TransUrl {
    pub fn from_web_url(url: &String) -> Option<Self> {
        let lowercased_url = url.clone().to_lowercase();
        let base_url = lowercased_url.split("/transmission/web").into_iter().next();
        base_url.map(|url| TransUrl(url.to_owned()))
    }
    pub fn to_rpc_url(&self) -> String {
        self.0.clone()+"/transmission/rpc"
    }

    pub(crate) fn get_base_url(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for TransUrl {
    fn from(url: String) -> Self {
        TransUrl(url)    
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_trans_url_rpc_generation() {
        let url = TransUrl("http://localhost".to_owned());
        assert_eq!("http://localhost/transmission/rpc", url.to_rpc_url())
    }

    #[test]
    fn test_trans_url_web_parsing() {
        let full_url = "http://localhost:9091/transmission/web/#confirm".to_owned();
        let t = TransUrl::from_web_url(&full_url).unwrap();
        assert_eq!("http://localhost:9091", t.get_base_url());
    }
}
