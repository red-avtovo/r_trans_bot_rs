use frankenstein::*;
use crate::errors::BotError;
use super::models::{
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

pub async fn list_directories(api: &AsyncApi, pool: &Pool, user_id: &i64, chat_id: &ChatId) -> Result<(), BotError> {
    let user = &get_user(pool, user_id).await?.unwrap();
    let dirs: Vec<DownloadDirectory> = get_directories(pool, user).await?;
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::ADD_DIRECTORY, direcoties_commands::ADD_DIRECTORY)]);
    keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::RESET_DIRECTORIES, direcoties_commands::RESET_DIRECTORIES)]);
    match dirs.len() {
        0 => {
            let send_param = SendMessageParams::builder()
                .chat_id(chat_id)
                .text("There are no registered directories yet")
                .reply_markup(keyboard)
                .build();
            api.send_message(&send_param).await?
        },
        _ => {
            let text: String = dirs.iter().map(|dir| {
                format!("<b>{}</b>: {}", dir.alias, dir.path)
            })
            .collect::<Vec<String>>()
            .join("\n");
            let send_param = SendMessageParams::builder()
                .chat_id(chat_id)
                .text(text)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::Html)
                .build();
            api.send_message(&send_param).await?
        }
    };
    Ok(())
}

pub async fn add_directory_prepare(api: &AsyncApi, chat_id: &ChatId) -> Result<(), BotError> {
    let send_param = SendMessageParams::builder()
        .chat_id(chat_id)
        .text("<b>Adding directory</b>\nDirectory format is\n\nfirst line: <i>Directory alias</i>\nsecond line: <i>Directory path</i>")
        .parse_mode(ParseMode::Html)
        .build();
    api.send_message(&send_param).await?;
    Ok(())
}

pub async fn add_directory_perform(api: &AsyncApi, pool: &Pool, user_id: &i64, message: &Message) -> Result<bool, BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    let keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(vec![
            vec![InlineKeyboardButton::builder().text(direcoties_commands::LIST_DIRECTORIES).callback_data(direcoties_commands::LIST_DIRECTORIES).build()],
            vec![InlineKeyboardButton::builder().text(direcoties_commands::ADD_DIRECTORY).callback_data(direcoties_commands::ADD_DIRECTORY).build()]
        ])
        .build();
    let text = message.text().unwrap();
    let lines = text.lines().collect::<Vec<&str>>();
    let lines_count = lines.len();
    match lines_count {
        2 => {
            let alias = lines[0].to_string();
            let path = lines[1].to_owned();
            add_directory(pool, &user, &alias, &path).await?;
            let send_param = SendMessageParams::builder()
                .chat_id(&message.from.unwrap().id)
                .text("Done!")
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .build();
            api.send_message(&send_param).await?;
            Ok(true)
        },
        _ => {
            let send_param = SendMessageParams::builder()
                .chat_id(&message.from.unwrap().id)
                .text(format!("Incorrect format. Found {} lines", lines_count))
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .build();
            api.send_message(&send_param).await?;
            add_directory_prepare(api, &message.from.to_chat_ref()).await?;
            Ok(false)
        }
    }
}

pub async fn reset_directories(api: &AsyncApi, pool: &Pool, user_id: &i64, chat_id: &ChatId) -> Result<(), BotError> {
    let user = get_user(pool, user_id).await?.unwrap();
    delete_directories(pool, user).await?;
    let send_param = SendMessageParams::builder()
        .chat_id(chat_id)
        .text("Done!")
        .build();
    api.send_message(&send_param).await?;
    Ok(())
}