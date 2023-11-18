use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::conversation::commands::settings_commands::HIDE_MESSAGE;
use crate::conversation::shared_server::shared_servers_commands::{RESET, SHARE, UN_SHARE};

use crate::db::{
    repository::Pool,
};
use crate::errors::BotError;

pub mod shared_servers_commands {
    pub const SHARE: &str = "Share a server";
    pub const UN_SHARE: &str = "Un-share a server";
    pub const RESET: &str = "Reset Servers Sharing❌";
}

pub async fn share_server_management(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    chat_id: &ChatId,
) -> Result<(), BotError> {
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(SHARE, SHARE)],
        vec![InlineKeyboardButton::callback(UN_SHARE, UN_SHARE)],
        vec![InlineKeyboardButton::callback(RESET, RESET)],
        vec![InlineKeyboardButton::callback(HIDE_MESSAGE, HIDE_MESSAGE)],
    ]);
    bot.send_message(*chat_id, "Manage Server sharing:")
        .reply_markup(kb)
        .await?;
    Ok(())
}