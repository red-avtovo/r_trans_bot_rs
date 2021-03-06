use telegram_bot::*;
use crate::errors::BotError;
use super::{
    models::{
        NewServer,
        Server,
        Authentication,
        User,
    },
    repository::{
        get_user,
        get_servers_by_user_id,
        tasks_count_by_server_id,
        delete_servers,
        add_server,
        add_server_auth,
        Pool,
    },
    trans_url::TransUrl,
};
use transmission_rpc::{
    TransClient,
    types::BasicAuth
};

pub mod servers_commands {
    pub const SERVER_STATS: &str = "Server stats 👀";
    pub const REGISTER_SERVER: &str = "Register server 🧰+";
    pub const RESET_SERVERS: &str = "Reset Servers 🧰❌";
}

pub async fn show_stats(api: &Api, pool: &Pool, user_id: &i64, chat_ref: &ChatRef) -> Result<(), BotError> {
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::REGISTER_SERVER, servers_commands::REGISTER_SERVER)]);
    keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::RESET_SERVERS, servers_commands::RESET_SERVERS)]);
    let user = get_user(pool, user_id).await?.unwrap();
    let servers: Vec<Server> = get_servers_by_user_id(pool, &user).await?;
    let mut stat_lines = vec!["Downloads for server:".to_string()];
    // for now it is just one
    match servers.get(0) {
        Some(server) => {
            let tasks = tasks_count_by_server_id(&pool, &server.id).await?;
            stat_lines.push(format!("<b>{}</b>: <i>{}</i>", server.url().get_base_url(), tasks));
            let client = &server.to_client();
            let status = match client.session_get().await {
                Ok(_) => "👍",
                Err(_) => "👎"
            };
            stat_lines.push(format!("Server status: {}", status))
        },
        _ => {
            stat_lines.push("Nothing yet :(".to_string())
        }
    }
    let text = stat_lines.join("\n");
    api.send(&chat_ref.text(text).reply_markup(keyboard).parse_mode(ParseMode::Html)).await?;
    Ok(())
}

impl Server {
    pub fn to_client(&self) -> TransClient {
        match &self.auth() {
            Some(auth) => TransClient::with_auth(&self.url().to_rpc_url(), BasicAuth{
                user: auth.username.clone(),
                password: auth.password.clone()
            }),
            None => TransClient::new(&self.url().to_rpc_url())
        }
    }
}

impl NewServer {
    pub fn to_client(&self) -> TransClient {
        match &self.auth() {
            Some(auth) => TransClient::with_auth(&self.url().to_rpc_url(), BasicAuth{
                user: auth.username.clone(),
                password: auth.password.clone()
            }),
            None => TransClient::new(&self.url().to_rpc_url())
        }
    }
}

pub async fn reset_servers(api: &Api, pool: &Pool, user_id: &i64, chat_ref: &ChatRef) -> Result<(), BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    delete_servers(pool, &user).await?;
    api.send(&chat_ref.text("Done!")).await?;
    Ok(())
}

pub async fn register_server_prepare(api: &Api, pool: &Pool, user_id: &i64, chat_ref: &ChatRef) -> Result<bool, BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    let servers: Vec<Server> = get_servers_by_user_id(pool, &user).await?;
    // for now is only 1 allowed
    if servers.len()!=0 {
        api.send(&chat_ref.text("There is already a server registered!")).await?;
        Ok(false)
    } else {
        api.send(&chat_ref.text("Enter server details in the format:\n<i>A link to you webui: E.g. http://localhost:9091/transmission/web</i>\n<i>Optional: user</i>\n<i>Optional: password</i>").parse_mode(ParseMode::Html)).await?;
        Ok(true)
    }
}

pub async fn register_server_perform(api: &Api, pool: &Pool, user_id: &i64, message: &Message) -> Result<bool, BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    let text = message.text().unwrap();
    let lines = text.lines().collect::<Vec<&str>>();
    let lines_count = lines.len();
    match lines_count {
        1 => {
            let url = TransUrl::from_web_url(&lines.get(0).unwrap().to_string());
            if url.is_none() { return Ok(false); }
            let server = NewServer::new(
                user_id.clone(),
                url.unwrap().get_base_url(),
                None,
            );
            try_to_add_server(api, pool, &user, &server, message).await
        },
        3 => {
            let url = TransUrl::from_web_url(&lines.get(0).unwrap().to_string());
            if url.is_none() { return Ok(false); }
            let server = NewServer::new(
                user_id.clone(),
                url.unwrap().get_base_url(),
                Some(Authentication {
                    username: lines.get(1).unwrap().to_string(),
                    password: lines.get(2).unwrap().to_string()
                })
            );
            try_to_add_server(api, pool, &user, &server, message).await
        },
        _ => {
            api.send(&message.to_source_chat().text(format!("Incorrect format. Found {} lines", lines_count)).parse_mode(ParseMode::Html)).await?;
            register_server_prepare(api, pool, user_id, &message.from.to_chat_ref()).await?;
            Ok(false)
        }
    }
}

async fn try_to_add_server(api: &Api, pool: &Pool, user: &User, server: &NewServer, message: &Message) -> Result<bool, BotError> {
    let client = server.to_client();
    match client.session_get().await {
        Ok(_) => {
            add_a_server(pool, user, &server).await?;
            api.send(&message.to_source_chat().text("Done!")).await?;
            Ok(true)
        },
        Err(_) => {
            api.send(&message.to_source_chat().text(format!("Unable to connect to server! Check details"))).await?;
            register_server_prepare(api, pool, &user.id, &message.from.to_chat_ref()).await?;
            Ok(false)
        }
    }
}

async fn add_a_server(pool: &Pool, user: &User, server: &NewServer) -> Result<Server, BotError> {
    match &server.auth() {
        Some(auth) => add_server_auth(pool, user, &server.url().get_base_url(), &auth.username, &auth.password).await,
        None => add_server(pool, &user, &server.url().get_base_url()).await
    }.map_err(BotError::from)
}