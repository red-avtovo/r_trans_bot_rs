use frankenstein::*;
use crate::errors::BotError;
use super::{
    repository::{
        get_user,
        save_user,
        Pool,
    },
    models::{
        NewUser,
    },
    crypto::random_salt,
    directories::direcoties_commands,
    servers::servers_commands,
};

use log::*;

pub mod settings_commands {
    pub const MENU: &str = "Settings ⚙️";
}

pub fn settings_keyboard() -> ReplyKeyboardMarkup {
    let keyboard = ReplyKeyboardMarkup::builder()
        .keyboard(vec![
            vec![
                KeyboardButton::builder().text(settings_commands::MENU).build()
            ]
        ])
        .resize_keyboard(true)
        .build();
    keyboard
}

pub async fn start_command(api: &AsyncApi, pool: &Pool, message: Message) -> Result<(), BotError> {
    debug!("Handle /start command");
    let m_clone = message.clone();
    debug!("Checking if user already exist");
    let from_id = message.from.unwrap().id;
    match get_user(pool, &(from_id as i64)).await {
        Ok(result) => {
            match result {
                Some(_) => {
                    debug!("User was already registered");

                    let send_message_params = SendMessageParams::builder()
                        .chat_id(&from_id)
                        .text(format!("Welcome back, {}", &message.from.map_or("".to_string(), |from| from.first_name)))
                        .reply_markup(settings_keyboard())
                        .build();
                    api.send_message(&send_message_params)?;
                }
                None => {
                    debug!("Registering new user");
                    let user = NewUser {
                        id: from_id.clone().into(),
                        chat: m_clone.chat.id().into(),
                        first_name: m_clone.from.map_or("".to_string(), |from| from.first_name),
                        last_name: m_clone.from.unwrap().last_name,
                        username: m_clone.from.unwrap().username,
                        salt: random_salt(),
                    };
                    save_user(pool, user).await?;
                    let send_params = SendMessageParams::builder()
                        .chat_id(&from_id)
                        .text(format!("Welcome, {}", &message.from.map_or("".to_string(), |from| from.first_name)))
                        .reply_markup(settings_keyboard())
                        .build();
                    api.send_message(&send_params).await?;
                }
            }
        }
        Err(error) => return Err(BotError::from(error))
    }
    Ok(())
}

pub async fn settings_menu(api: &AsyncApi, message: Message) -> Result<(), BotError> {
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::LIST_DIRECTORIES, direcoties_commands::LIST_DIRECTORIES)]);
    keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::SERVER_STATS, servers_commands::SERVER_STATS)]);
    let send_param = SendMessageParams::builder()
        .chat_id(message.from.unwrap().id)
        .text("Settings")
        .reply_markup(keyboard)
        .build();
    api.send_message(&send_param).await?;
    Ok(())
}