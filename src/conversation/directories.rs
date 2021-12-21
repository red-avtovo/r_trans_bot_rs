use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode};

use crate::db::models::directories::DownloadDirectory;
use crate::db::repository::{add_directory, delete_directories, get_directories, get_user, Pool};
use crate::errors::BotError;
use crate::router::{HandlerResult, BotDialogue};

pub mod directories_commands {
    pub const LIST_DIRECTORIES: &str = "List Directories 📂";
    pub const ADD_DIRECTORY: &str = "Add Directory 📂+";
    pub const RESET_DIRECTORIES: &str = "Reset Directories 📂❌";

    pub const BACK_TO_SETTINGS: &str = "Back to settings ⬅️";
}

pub async fn list_directories(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    chat_id: &ChatId,
) -> Result<(), BotError> {
    let user = &get_user(pool, &(*user_id as i64)).await?.unwrap();
    let dirs: Vec<DownloadDirectory> = get_directories(pool, user).await?;
    let keyboard = InlineKeyboardMarkup::new(
        vec![
            vec![InlineKeyboardButton::callback(
                directories_commands::ADD_DIRECTORY,
                directories_commands::ADD_DIRECTORY,
            )],
            vec![InlineKeyboardButton::callback(
                directories_commands::RESET_DIRECTORIES,
                directories_commands::RESET_DIRECTORIES,
            )],
        ]
    );
    match dirs.len() {
        0 => {
            bot.send_message(*chat_id, "There are no registered directories yet")
                .reply_markup(keyboard)
                .await?
        }
        _ => {
            let text: String = dirs
                .iter()
                .map(|dir| format!("<b>{}</b>: {}", dir.alias, dir.path))
                .collect::<Vec<String>>()
                .join("\n");
            bot.send_message(*chat_id, text)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::Html)
                .await?
        }
    };
    Ok(())
}

pub async fn add_directory_prepare(bot: &Bot, chat_id: &ChatId) -> Result<(), BotError> {
    bot.send_message(
        *chat_id,
        "<b>Adding directory</b>\nDirectory format is\n\nfirst line: <i>Directory alias</i>\nsecond line: <i>Directory path</i>",
    ).parse_mode(ParseMode::Html).await?;
    Ok(())
}

pub async fn add_directory_dialogue(
    bot: Bot,
    pool: Pool,
    dialogue: BotDialogue,
    message: Message,
) -> HandlerResult {
    let user_id = message.from().unwrap().id.0;
    let user = get_user(&pool, &(user_id as i64)).await.unwrap().unwrap();
    let keyboard = InlineKeyboardMarkup::new(
        vec![
            vec![InlineKeyboardButton::callback(
                directories_commands::LIST_DIRECTORIES,
                directories_commands::LIST_DIRECTORIES,
            )],
            vec![InlineKeyboardButton::callback(
                directories_commands::ADD_DIRECTORY,
                directories_commands::ADD_DIRECTORY,
            )],
            vec![InlineKeyboardButton::callback(
                directories_commands::BACK_TO_SETTINGS,
                directories_commands::BACK_TO_SETTINGS,
            )],
        ]
    );
    let text = message.text().unwrap();
    let lines = text.lines().collect::<Vec<&str>>();
    let lines_count = lines.len();
    match lines_count {
        2 => {
            let alias = lines[0].to_string();
            let path = lines[1].to_owned();
            add_directory(&pool, &user, &alias, &path).await.unwrap();
            bot.send_message(message.chat.id, "Done!")
                .reply_markup(keyboard)
                .await?;
            dialogue.exit().await?;
        }
        _ => {
            bot.send_message(
                message.chat.id,
                format!("Incorrect format. Found {} lines", lines_count),
            ).parse_mode(ParseMode::Html)
                .await?;
            add_directory_prepare(&bot, &message.chat.id).await?;
        }
    };
    Ok(())
}

pub async fn reset_directories(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    chat_id: &ChatId,
) -> Result<(), BotError> {
    let user = get_user(pool, &(*user_id as i64)).await?.unwrap();
    delete_directories(pool, user).await?;
    bot.send_message(*chat_id, "Done!").await?;
    Ok(())
}
