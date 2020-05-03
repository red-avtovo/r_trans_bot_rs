use telegram_bot::*;
use crate::errors::BotError;
use crate::logic::general::*;
use crate::logic::torrents::*;
use crate::logic::db::Pool;

pub async fn route(api: Api, pool: &Pool, update: Update) -> Result<(), BotError> {
    match update.kind {
        UpdateKind::Message(ref message) => process_message(api.clone(), pool.clone(), message.clone()).await?,
        UpdateKind::CallbackQuery(ref callback_query) => process_callback(api.clone(), pool.clone(), callback_query.clone()).await?,
        _ => ()
    }
    Ok(())
}

async fn process_message(api: Api, pool: Pool, message: Message) -> Result<(), BotError> {
    match message.kind {
        MessageKind::Text { ref data, .. } => match data.as_str() {
            "/start" => start_command(api, &pool, message).await?,
            settings_commands::MENU => settings_menu(api, message).await?,
            _ if data.as_str().starts_with("magnet:") => process_magnet(api, message).await?,
            _ => (),
        },
        _ => {
            api.send(message.to_source_chat().text("Message type is not supported!")).await?;
            ()
        }
    };
    Ok(())
}

async fn process_callback(api: Api, _pool: Pool, callback_query: CallbackQuery) -> Result<(), BotError> {
    match callback_query.data {
        // download:url_uuid:directory_id
        Some(ref value) if value.starts_with("download:") => start_download(api, callback_query).await?,
        _ => (),
    };
    Ok(())
}