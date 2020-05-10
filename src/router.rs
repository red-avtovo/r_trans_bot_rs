use std::collections::HashMap;
use telegram_bot::*;
use crate::errors::BotError;
use crate::logic::{
    models::TelegramId,
    general::*,
    directories::*,
    servers::*,
    tasks::*,
    repository::Pool
};
use async_trait::async_trait;
use log::*;

pub async fn route(api: Api, pool: &Pool, update: Update, mut last_command: &mut HashMap<TelegramId, String>) {
    match update.kind {
        UpdateKind::Message(ref message) => { process_message(api.clone(), pool.clone(), message.clone(), &mut last_command)
            .await
            .handle_error(&api, &message.from.to_chat_ref())
            .await;
        },
        UpdateKind::CallbackQuery(ref callback_query) => { process_callback(api.clone(), &pool.clone(), callback_query.clone(), &mut last_command)
            .await
            .handle_error(&api, &callback_query.from.to_chat_ref())
            .await;
        },
        _ => {}
    };
}


#[async_trait]
trait ErrorHandler{
    async fn handle_error(&self, api: &Api, chat_ref: &ChatRef);
}

#[async_trait]
impl ErrorHandler for Result<(), BotError> {
    async fn handle_error(&self, api: &Api, chat_ref: &ChatRef) {
        match &self {
            Ok(_) => {},
            Err(error) => { 
                error!("Error while handling the message: {}", error);
                api.send(&chat_ref.text("Something went wrong :(")).await; 
            }
        }
    }
}

async fn process_message(api: Api, pool: Pool, message: Message, last_command: &mut HashMap<TelegramId, String>) -> Result<(), BotError> {
    let user_id = &TelegramId::from(message.from.id);
    match message.kind {
        MessageKind::Text { ref data, .. } => match data.as_str() {
            "/start" => start_command(&api, &pool, message).await?,
            settings_commands::MENU => settings_menu(&api, message).await?,
            _ if data.as_str().contains("magnet:") => process_magnet(api, &pool, message).await?,
            // step 2 messages
            _ => if last_command.contains_key(user_id) {
                let result = match last_command.get(user_id).unwrap().as_str() {
                    direcoties_commands::ADD_DIRECTORY => add_directory_perform(&api, &pool, user_id, &message).await?,
                    servers_commands::REGISTER_SERVER => register_server_perform(&api, &pool, user_id, &message).await?,
                    _ => true
                };
                if result { last_command.remove(user_id); }
            },
        },
        _ => {
            api.send(message.to_source_chat().text("Message type is not supported!")).await?;
            ()
        }
    };
    Ok(())
}

async fn process_callback(api: Api, pool: &Pool, callback_query: CallbackQuery, last_command: &mut HashMap<TelegramId, String>) -> Result<(), BotError> {
    let user_id = TelegramId::from(callback_query.from.id);
    let chat_ref = callback_query.from.to_chat_ref();
    let data = callback_query.data.clone(); 
    match data {
        // download:url_uuid:directory_id
        Some(ref value) if value.starts_with("download:") => start_download(&api, value, &chat_ref).await?,
        // static commands
        Some(ref data_string) => match data_string.as_str() {
            direcoties_commands::LIST_DIRECTORIES => list_directories(&api, pool, &user_id, &chat_ref).await?,
            direcoties_commands::ADD_DIRECTORY => { 
                add_directory_prepare(&api, &chat_ref).await?;
                last_command.insert(user_id, direcoties_commands::ADD_DIRECTORY.to_owned());
            },
            direcoties_commands::RESET_DIRECTORIES => reset_directories(&api, pool, &user_id, &chat_ref).await?,
            servers_commands::SERVER_STATS => show_stats(&api, pool, &user_id, &chat_ref).await?,
            servers_commands::RESET_SERVERS => reset_servers(&api, pool, &user_id, &chat_ref).await?,
            servers_commands::REGISTER_SERVER => {
                let result = register_server_prepare(&api, pool, &user_id, &chat_ref).await?;
                if result { last_command.insert(user_id, servers_commands::REGISTER_SERVER.to_owned()); }
            }
            _ => {}
        },
        _ => (),
    };
    api.send(callback_query.acknowledge()).await?;
    match callback_query.message {
        Some(message) => api.send(message.delete()).await?,
        _ => {}
    }
    Ok(())
}