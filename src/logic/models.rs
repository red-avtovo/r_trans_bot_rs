use telegram_bot_raw::types::refs::UserId;
use postgres_types::{ToSql, FromSql};
use uuid::Uuid;
use serde::Serialize;
use url::Url;
use url::form_urlencoded;
use crate::errors::MagnetMappingError;
use log::*;

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

#[derive(Debug, Clone, Serialize)]
pub struct ShortMagnet {
    xt: String,
    tr: Vec<String>,
}

impl ShortMagnet {
    pub fn from(string: &String) -> Result<Self, MagnetMappingError> {
        debug!("Parsing magnet: {}", string);
        let url: Url = Url::parse(string.as_ref()).expect("Invalid magnet");
        let parameters = url.query_pairs();

        let xts: Vec<String> = parameters.filter(|pair| pair.0 == "xt")
        .map(|pair| pair.1.to_string())
        .collect();
        let xt = match xts.len() {
            0 => Err(MagnetMappingError::new("No XT parameter found")),
            1 => Ok(xts.first().unwrap().to_owned()),
            _ => Err(MagnetMappingError::new("There were found more then 1 xt parameter")),
        }?;

        let tr: Vec<String> = parameters.filter(|pair| pair.0 == "tr")
        .map(|pair| pair.1.to_string())
        .collect();

        Ok(ShortMagnet{ xt, tr})
    }

    pub fn find(string: &String) -> Option<Self> {
        string.split("\n")
            .map(|line| line.split(" "))
            .flatten()
            .find(|part| part.starts_with("magnet:?"))
            .map(|it| match ShortMagnet::from(&it.to_string()){
                Ok(res) =>  Some(res),
                Err(_) => None
            })
            .flatten()
    }
}

impl From<ShortMagnet> for String {
    fn from(short: ShortMagnet) -> Self {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        short.tr.iter().for_each(|tr| {
            &serializer.append_pair("tr", tr);
        });
        let tracker_params = serializer.finish();
        format!("magnet:?xt={}&{}", &short.xt, tracker_params)
    }
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
    pub fn test_short_magnet_generation_from_magnet_string() {
        let magnet = String::from("magnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&dn=ubuntu-mate-16.10-desktop-amd64.iso&tr=http%3A%2F%2Fsometracker.com%2Fannounce2");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let short = ShortMagnet::from(&magnet).unwrap();
        assert_eq!(short.clone().xt, urn);
        assert_eq!(short.clone().tr, trackers);
    }

    #[test]
    pub fn test_short_magnet_generation_from_magnet_string_in_message() {
        let magnet = &String::from("some info magnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&dn=ubuntu-mate-16.10-desktop-amd64.iso&tr=http%3A%2F%2Fsometracker.com%2Fannounce2 and some comment after");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let short = ShortMagnet::find(magnet);
        assert!(short.clone().is_some());
        assert_eq!(short.clone().unwrap().xt, urn);
        assert_eq!(short.clone().unwrap().tr, trackers);
    }

    #[test]
    pub fn test_short_magnet_generation_from_magnet_string_in_message_on_new_line() {
        let magnet = &String::from("some info\nmagnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&dn=ubuntu-mate-16.10-desktop-amd64.iso&tr=http%3A%2F%2Fsometracker.com%2Fannounce2\nand some comment after");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let short = ShortMagnet::find(magnet);
        assert!(short.clone().is_some());
        assert_eq!(short.clone().unwrap().xt, urn);
        assert_eq!(short.clone().unwrap().tr, trackers);
    }

    #[test]
    pub fn test_short_magnet_string_generation() {
        let magnet = String::from("magnet:?xt=urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb&tr=http%3A%2F%2Fsometracker.com%2Fannounce&tr=http%3A%2F%2Fsometracker.com%2Fannounce2");
        let urn = String::from("urn:btih:e249fe4dc957be4b4ce3ecaac280fdf1c71bc5bb");
        let trackers = vec![ "http://sometracker.com/announce".to_owned(), "http://sometracker.com/announce2".to_owned()];
        let short = ShortMagnet { xt: urn, tr: trackers};
        let actual:String = short.into();
        assert_eq!(actual, magnet);
    }

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
