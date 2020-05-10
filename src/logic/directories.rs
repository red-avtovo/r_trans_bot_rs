use telegram_bot::*;
use crate::errors::BotError;
use super::models::{
    TelegramId,
    DownloadDirectory
};
use super::repository::{
    get_user,
    get_directories,
    add_directory,
    delete_directories,
    Pool
};

pub mod direcoties_commands {
    pub const LIST_DIRECTORIES: &str = "List Directories 📂";
    pub const ADD_DIRECTORY: &str = "Add Directory 📂+";
    pub const RESET_DIRECTORIES: &str = "Reset Directories 📂❌";
}

pub async fn list_directories(api: &Api, pool: &Pool, user_id: &TelegramId, chat_ref: &ChatRef) -> Result<(), BotError> {
    let user = &get_user(pool, user_id).await?.unwrap();
    let dirs: Vec<DownloadDirectory> = get_directories(pool, user).await?;
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::ADD_DIRECTORY, direcoties_commands::ADD_DIRECTORY)]);
    keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::RESET_DIRECTORIES, direcoties_commands::RESET_DIRECTORIES)]);
    match dirs.len() {
        0 => api.send(&chat_ref.text("There are no registere directories yet").reply_markup(keyboard)).await?,
        _ => {
            let text: String = dirs.iter().map(|dir| {
                format!("<b>{}</b>: {}<br/>", dir.alias, dir.path)
            })
            .collect();
            api.send(&chat_ref.text(text).reply_markup(keyboard).parse_mode(ParseMode::Html)).await?
        }
    };
    Ok(())
}

pub async fn add_directory_prepare(api: &Api, chat_ref: &ChatRef) -> Result<(), BotError> {
    api.send(&chat_ref.text("<b>Adding directory<br/>Directory format is</b><br/>first line: Directory alias<br/>second line: Directory path").parse_mode(ParseMode::Html)).await?;
    Ok(())
}

pub async fn add_directory_perform(api: &Api, pool: &Pool, user_id: &TelegramId, message: &Message) -> Result<bool, BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::LIST_DIRECTORIES, direcoties_commands::LIST_DIRECTORIES)]);
    keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::ADD_DIRECTORY, direcoties_commands::ADD_DIRECTORY)]);
    let text = message.text().unwrap();
    let lines = text.lines().collect::<Vec<&str>>();
    let lines_count = lines.len();
    match lines_count {
        2 => {
            let alias = lines[0].to_string();
            let path = lines[1].to_owned();
            add_directory(pool, &user, &alias, &path).await?;
            api.send(&message.to_source_chat().text("Done!").reply_markup(keyboard)).await?;
            Ok(true)
        },
        _ => {
            api.send(&message.to_source_chat().text(format!("Incorrect format. Found {} lines", lines_count)).parse_mode(ParseMode::Html)).await?;
            add_directory_prepare(api, &message.from.to_chat_ref()).await?;
            Ok(false)
        }
    }
}

pub async fn reset_directories(api: &Api, pool: &Pool, user_id: &TelegramId, chat_ref: &ChatRef) -> Result<(), BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    delete_directories(pool, user).await?;
    api.send(chat_ref.text("Done!")).await?;
    Ok(())
}