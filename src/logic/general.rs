use telegram_bot::*;
use crate::errors::BotError;
use super::{
    repository::{
        get_user,
        save_user,
        Pool
    },
    models::{
        DbUser,
        TelegramId
    },
    crypto::random_salt,
    directories::direcoties_commands,
    servers::servers_commands,
};

use log::*;

pub mod settings_commands {
    pub const MENU: &str = "Settings ⚙️";
}

pub async fn start_command(api: &Api, pool: &Pool, message: Message) -> Result<(), BotError> {
    debug!("Handle /start command");
    let m_clone = message.clone();
    let mut keyboard = ReplyKeyboardMarkup::new();
    keyboard.add_row(vec![
        KeyboardButton::new(settings_commands::MENU)
    ]);
    keyboard.resize_keyboard();
    debug!("Checking if user already exist");
    match get_user(pool, &TelegramId::from(m_clone.from.id)).await {
        Ok(result) => {
            match result {
                Some(_) => {
                    debug!("User was already registered");
                    api.send(&message.to_source_chat().text(format!("Welcome back, {}", &message.from.first_name)).reply_markup(keyboard)).await?;
                },
                None => {
                    debug!("Registering new user");
                    let user=  DbUser {
                        id: m_clone.from.id.into(),
                        chat: m_clone.chat.id().into(),
                        first_name: m_clone.from.first_name,
                        last_name: m_clone.from.last_name,
                        username: m_clone.from.username,
                        salt: random_salt(),
                    };
                    save_user(pool, user).await?;
                    api.send(&message.to_source_chat().text(format!("Welcome, {}", &message.from.first_name)).reply_markup(keyboard)).await?;
                }
            }
        },
        Err(error) => return Err(BotError::from(error))
    }
    Ok(())
}

pub async fn settings_menu(api: &Api, message: Message) -> Result<(), BotError> {
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::LIST_DIRECTORIES, direcoties_commands::LIST_DIRECTORIES)]);
    keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::SERVER_STATS, servers_commands::SERVER_STATS)]);
    api.send(message.to_source_chat().text(settings_commands::MENU).reply_markup(keyboard)).await?;
    Ok(())
}