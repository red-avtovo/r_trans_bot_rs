use std::collections::HashMap;
use telegram_bot::*;
use crate::errors::BotError;
use crate::logic::{
    models::TelegramId,
    general::*,
    directories::*,
    tasks::*,
    repository::Pool
};

pub async fn route(api: Api, pool: &Pool, update: Update, mut last_command: &mut HashMap<TelegramId, String>) -> Result<(), BotError> {
    match update.kind {
        UpdateKind::Message(ref message) => process_message(api.clone(), pool.clone(), message.clone(), last_command).await?,
        UpdateKind::CallbackQuery(ref callback_query) => process_callback(api.clone(), &pool.clone(), callback_query.clone(), &mut last_command).await?,
        _ => ()
    }
    Ok(())
}

async fn process_message(api: Api, pool: Pool, message: Message, last_command: &HashMap<TelegramId, String>) -> Result<(), BotError> {
    match message.kind {
        MessageKind::Text { ref data, .. } => match data.as_str() {
            "/start" => start_command(api, &pool, message).await?,
            settings_commands::MENU => settings_menu(api, message).await?,
            _ if data.as_str().contains("magnet:") => process_magnet(api, message).await?,
            _ => (),
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
        Some(ref data_string) => match data_string.as_str() {
            direcoties_commands::LIST_DIRECTORIES => list_directories(&api, pool, &user_id, &chat_ref).await?,
            direcoties_commands::ADD_DIRECTORY => { 
                add_directory_prepare(&api, &chat_ref).await?;
                last_command.insert(user_id, direcoties_commands::ADD_DIRECTORY.to_owned());
            },
            _ => {}
        },
        _ => (),
    };
    api.send(callback_query.acknowledge()).await?;
    Ok(())
}