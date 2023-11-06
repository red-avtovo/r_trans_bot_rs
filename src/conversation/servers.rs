use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode};
use transmission_rpc::{TransClient, types::BasicAuth};

use crate::core::trans_url::TransUrl;
use crate::db::{
    models::{
        server::{Authentication, NewServer, Server},
        user::User,
    },
    repository::{
        add_server, add_server_auth, delete_servers, get_servers_by_user_id, get_user,
        Pool, tasks_count_by_server_id,
    },
};
use crate::errors::BotError;
use crate::router::{BotDialogue, HandlerResult};

pub mod servers_commands {
    pub const SERVER_STATS: &str = "Server stats 👀";
    pub const REGISTER_SERVER: &str = "Register server 🧰+";
    pub const RESET_SERVERS: &str = "Reset Servers 🧰❌";
    pub const BACK_TO_SETTINGS: &str = "Back to settings ⬅️";
}

pub async fn show_stats(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    chat_id: &ChatId,
) -> Result<(), BotError> {
    let keyboard = InlineKeyboardMarkup::new(
        vec![
            vec![InlineKeyboardButton::callback(
                servers_commands::REGISTER_SERVER,
                servers_commands::REGISTER_SERVER,
            )],
            vec![InlineKeyboardButton::callback(
                servers_commands::RESET_SERVERS,
                servers_commands::RESET_SERVERS,
            )],
            vec![InlineKeyboardButton::callback(
                servers_commands::BACK_TO_SETTINGS,
                servers_commands::BACK_TO_SETTINGS,
            )],
        ]
    );
    let user = get_user(pool, &(*user_id as i64)).await?.unwrap();
    let servers: Vec<Server> = get_servers_by_user_id(pool, &user).await?;
    let mut stat_lines = vec!["Downloads for server:".to_string()];
    // for now it is just one
    match servers.get(0) {
        Some(server) => {
            let tasks = tasks_count_by_server_id(&pool, &server.id).await?;
            stat_lines.push(format!(
                "<b>{}</b>: <i>{}</i>",
                server.url().get_base_url(),
                tasks
            ));
            let client = &mut server.to_client();
            let status = match client.session_get().await {
                Ok(_) => "👍",
                Err(_) => "👎",
            };
            stat_lines.push(format!("Server status: {}", status))
        }
        _ => stat_lines.push("Nothing yet :(".to_string()),
    }
    let text = stat_lines.join("\n");
    bot.send_message(*chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

impl Server {
    pub fn to_client(&self) -> TransClient {
        match &self.auth() {
            Some(auth) => TransClient::with_auth(
                self.url().to_rpc_url(),
                BasicAuth {
                    user: auth.username.clone(),
                    password: auth.password.clone(),
                },
            ),
            None => TransClient::new(self.url().to_rpc_url()),
        }
    }
}

impl NewServer {
    pub fn to_client(&self) -> TransClient {
        match &self.auth() {
            Some(auth) => TransClient::with_auth(
                self.url().to_rpc_url(),
                BasicAuth {
                    user: auth.username.clone(),
                    password: auth.password.clone(),
                },
            ),
            None => TransClient::new(self.url().to_rpc_url()),
        }
    }
}

pub async fn reset_servers(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    chat_id: &ChatId,
) -> Result<(), BotError> {
    let user = get_user(pool, &(*user_id as i64)).await?.unwrap();
    delete_servers(pool, &user).await?;
    bot.send_message(*chat_id, "Done!").await?;
    Ok(())
}

pub async fn register_server_prepare(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    chat_id: &ChatId,
) -> Result<bool, BotError> {
    let user = get_user(pool, &(*user_id as i64)).await?.unwrap();
    let servers: Vec<Server> = get_servers_by_user_id(pool, &user).await?;
    // for now is only 1 allowed
    if servers.len() != 0 {
        bot.send_message(*chat_id, "There is already a server registered!")
            .await?;
        Ok(false)
    } else {
        bot.send_message(
            *chat_id,
            "Enter server details in the format:\n<i>A link to you webui: E.g. http://localhost:9091/transmission/web</i>\n<i>Optional: user</i>\n<i>Optional: password</i>",
        ).parse_mode(ParseMode::Html).await?;
        Ok(true)
    }
}

pub async fn register_server_dialogue(
    bot: Bot,
    pool: Pool,
    dialogue: BotDialogue,
    message: Message,
) -> HandlerResult {
    let user_id = message.from().unwrap().id.0;
    let user = get_user(&pool, &(user_id as i64)).await.unwrap().unwrap();
    let text = message.text().unwrap();
    let lines = text.lines().collect::<Vec<&str>>();
    let lines_count = lines.len();
    match lines_count {
        1 => {
            let url = TransUrl::from_web_url(&lines.get(0).unwrap().to_string());
            if url.is_none() {
                return Ok(());
            }
            let server = NewServer::new(user_id.clone(), url.unwrap().get_base_url(), None);
            if try_to_add_server(&bot, &pool, &user, &server, &message).await? {
                dialogue.exit().await?
            }
        }
        3 => {
            let url = TransUrl::from_web_url(&lines.get(0).unwrap().to_string());
            if url.is_none() {
                return Ok(());
            }
            let server = NewServer::new(
                user_id.clone(),
                url.unwrap().get_base_url(),
                Some(Authentication {
                    username: lines.get(1).unwrap().to_string(),
                    password: lines.get(2).unwrap().to_string(),
                }),
            );
            if try_to_add_server(&bot, &pool, &user, &server, &message).await? {
                dialogue.exit().await?
            }
        }
        _ => {
            bot.send_message(message.chat.id, format!("Incorrect format. Found {} lines", lines_count))
                .parse_mode(ParseMode::Html)
                .await?;
            if register_server_prepare(&bot, &pool, &user_id, &message.chat.id).await? {
                dialogue.exit().await?
            }
        }
    };
    Ok(())
}

async fn try_to_add_server(
    bot: &Bot,
    pool: &Pool,
    user: &User,
    server: &NewServer,
    message: &Message,
) -> Result<bool, BotError> {
    let mut client = server.to_client();
    match client.session_get().await {
        Ok(_) => {
            add_a_server(pool, user, &server).await?;
            bot.send_message(message.chat.id, "Done!").await?;

            Ok(true)
        }
        Err(_) => {
            bot.send_message(
                message.chat.id,
                "Unable to connect to server! Check details",
            ).await?;
            register_server_prepare(bot, pool, &(user.id as u64), &message.chat.id).await?;
            Ok(false)
        }
    }
}

async fn add_a_server(pool: &Pool, user: &User, server: &NewServer) -> Result<Server, BotError> {
    match &server.auth() {
        Some(auth) => {
            add_server_auth(
                pool,
                user,
                &server.url().get_base_url(),
                &auth.username,
                &auth.password,
            )
                .await
        }
        None => add_server(pool, &user, &server.url().get_base_url()).await,
    }
        .map_err(BotError::from)
}
