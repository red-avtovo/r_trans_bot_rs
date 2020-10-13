use std::collections::HashMap;
use telegram_bot::*;
use crate::errors::BotError;
use crate::logic::{
    general::*,
    directories::*,
    servers::*,
    tasks::*,
    repository::Pool
};
use async_trait::async_trait;
use log::*;

pub async fn route(api: Api, pool: &Pool, update: Update, mut last_command: &mut HashMap<i64, String>) {
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
                match api.send(&chat_ref.text("Something went wrong :(")).await {
                    Ok(_) => {}
                    Err(_) => {
                        error!("Unable to send generic error message");
                    },
                };
            }
        };
    }
}

async fn process_message(api: Api, pool: Pool, message: Message, last_command: &mut HashMap<i64, String>) -> Result<(), BotError> {
    let user_id: &i64 = &message.from.id.into();
    match message.kind {
        MessageKind::Text { ref data, .. } => match data.as_str() {
            "/start" => start_command(&api, &pool, message).await?,
            settings_commands::MENU => settings_menu(&api, message).await?,
            _ if data.as_str().contains("magnet:") => {
                log::debug!("Processing a magnet link");
                match process_magnet(&api, &pool, &message).await {
                    Ok(_) => {
                        log::debug!("Processing of a magnet link passed. Deleting the original message");
                        match api.send(message.delete()).await {
                            Ok(_) => log::debug!("Message was successfully deleted!"),
                            _ => log::warn!("Unable to delete the original message!")
                        }
                    },
                    _ => {
                        log::warn!("Processing of a magnet link failed! The link was: {}", &data);
                    }
                }
            },
            // step 2 messages
            _ if last_command.contains_key(user_id) => {
                let result = match last_command.get(user_id).unwrap().as_str() {
                    direcoties_commands::ADD_DIRECTORY => add_directory_perform(&api, &pool, user_id, &message).await?,
                    servers_commands::REGISTER_SERVER => register_server_perform(&api, &pool, user_id, &message).await?,
                    _ => true
                };
                if result { last_command.remove(user_id); }
            },
            _ => { api.send(message.to_source_chat().text("I don't know what you mean")).await?; }
        },
        _ => {
            api.send(message.to_source_chat().text("Message type is not supported!")).await?;
            ()
        }
    };
    Ok(())
}

async fn process_callback(api: Api, pool: &Pool, callback_query: CallbackQuery, last_command: &mut HashMap<i64, String>) -> Result<(), BotError> {
    api.send(callback_query.acknowledge()).await?;
    let user_id: &i64 = &callback_query.from.id.into();
    let chat_ref = &callback_query.from.to_chat_ref();
    let data = callback_query.data.clone();
    match data {
        // download:magnet_uuid:directory_ordinal (1-64 bytes)
        Some(ref value) if value.starts_with("download:") => start_download(&api, pool, user_id,value, chat_ref).await?,
        // t_status:task_uuid
        Some(ref value) if value.starts_with("t_status:") => update_task_status(&api, pool, user_id,value, &callback_query.clone().message.unwrap()).await?,
        // t_remove:task_uuid
        Some(ref value) if value.starts_with("t_remove:") => remove_task(&api, pool, user_id,value, &callback_query.clone().message.unwrap()).await?,
        // static commands
        Some(ref data_string) => match data_string.as_str() {
            direcoties_commands::LIST_DIRECTORIES => list_directories(&api, pool, user_id, chat_ref).await?,
            direcoties_commands::ADD_DIRECTORY => {
                add_directory_prepare(&api, &chat_ref).await?;
                last_command.insert(user_id.to_owned(), direcoties_commands::ADD_DIRECTORY.to_owned());
            },
            direcoties_commands::RESET_DIRECTORIES => reset_directories(&api, pool, user_id, chat_ref).await?,
            servers_commands::SERVER_STATS => show_stats(&api, pool, user_id, chat_ref).await?,
            servers_commands::RESET_SERVERS => reset_servers(&api, pool, user_id, chat_ref).await?,
            servers_commands::REGISTER_SERVER => {
                let result = register_server_prepare(&api, pool, user_id, chat_ref).await?;
                if result { last_command.insert(user_id.to_owned(), servers_commands::REGISTER_SERVER.to_owned()); }
            }
            _ => {}
        },
        _ => (),
    };
    match callback_query.message {
        Some(message) if data.is_some() && !data.unwrap().starts_with("t_status:") => api.send(message.delete()).await?,
        _ => {}
    }
    Ok(())
}