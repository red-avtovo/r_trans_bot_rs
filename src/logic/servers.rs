use telegram_bot::*;
use uuid::Uuid;
use crate::errors::BotError;
use super::{
    models::{
        TelegramId,
        Server,
        Authentication,
        DbUser,
    },
    repository::{
        get_user,
        get_servers_by_user_id,
        get_task_by_server_id,
        delete_servers,
        add_server,
        add_server_auth,
        Pool,
    },
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

pub async fn show_stats(api: &Api, pool: &Pool, user_id: &TelegramId, chat_ref: &ChatRef) -> Result<(), BotError> {
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::RESET_SERVERS, servers_commands::RESET_SERVERS)]);
    let user = get_user(pool, user_id).await?.unwrap();
    let servers: Vec<Server> = get_servers_by_user_id(pool, &user).await?;
    let mut stat_lines = vec!["Stats".to_string()];
    // for now it is just one
    match servers.get(0) {
        Some(server) => {
            let tasks = get_task_by_server_id(&pool, &server.id).await?.len();
            stat_lines.push(format!("<b>{}</b>: <i>{}</i>", server.url, tasks));
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
    let text = stat_lines.join("<br/>");
    api.send(&chat_ref.text(text).reply_markup(keyboard).parse_mode(ParseMode::Html)).await?;
    Ok(())
}

impl Server {
    pub fn to_client(&self) -> TransClient {
        match &self.auth {
            Some(auth) => TransClient::with_auth(&self.url, BasicAuth{
                user: auth.username.clone(),
                password: auth.password.clone()
            }),
            None => TransClient::new(&self.url)
        }
    }
}

pub async fn reset_servers(api: &Api, pool: &Pool, user_id: &TelegramId, chat_ref: &ChatRef) -> Result<(), BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    delete_servers(pool, &user).await?;
    api.send(&chat_ref.text("Done!")).await?;
    Ok(())
}

pub async fn register_server_prepare(api: &Api, pool: &Pool, user_id: &TelegramId, chat_ref: &ChatRef) -> Result<bool, BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    let servers: Vec<Server> = get_servers_by_user_id(pool, &user).await?;
    // for now is only 1 allowed
    if servers.len()!=0 {
        api.send(&chat_ref.text("There is already a server registered!")).await?;
        Ok(false)
    } else {
        api.send(&chat_ref.text("Enter server details in the format:<br/><i>Host:port</i><br/><i>Optional: user</i><br/><i>Optional: password</i><br/>").parse_mode(ParseMode::Html)).await?;
        Ok(true)
    }
}

pub async fn register_server_perform(api: &Api, pool: &Pool, user_id: &TelegramId, message: &Message) -> Result<bool, BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    let text = message.text().unwrap();
    let lines = text.lines().collect::<Vec<&str>>();
    let lines_count = lines.len();
    match lines_count {
        1 => {
            let server = Server {
                id: Uuid::new_v4(),
                user_id: user_id.clone(),
                url: lines.get(0).unwrap().to_string(),
                auth: None
            };
            try_to_add_server(api, pool, &user, &server, message).await
        },
        3 => {
            let server = Server {
                id: Uuid::new_v4(),
                user_id: user_id.clone(),
                url: lines.get(0).unwrap().to_string(),
                auth: Some(Authentication {
                    username: lines.get(1).unwrap().to_string(),
                    password: lines.get(2).unwrap().to_string()
                })
            };
            try_to_add_server(api, pool, &user, &server, message).await
        },
        _ => {
            api.send(&message.to_source_chat().text(format!("Incorrect format. Found {} lines", lines_count)).parse_mode(ParseMode::Html)).await?;
            register_server_prepare(api, pool, user_id, &message.from.to_chat_ref()).await?;
            Ok(false)
        }
    }
}

async fn try_to_add_server(api: &Api, pool: &Pool, user: &DbUser, server: &Server, message: &Message) -> Result<bool, BotError> {
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

async fn add_a_server(pool: &Pool, user: &DbUser, server: &Server) -> Result<Server, BotError> {
    match &server.auth {
        Some(auth) => add_server_auth(pool, user, &server.url, &auth.username, &auth.password).await,
        None => add_server(pool, &user, &server.url).await
    }.map_err(BotError::from)
    
}