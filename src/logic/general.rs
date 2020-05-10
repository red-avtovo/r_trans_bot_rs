use telegram_bot::*;
use crate::errors::{
    BotError,
    DbErrorKind,
    DbError
};
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
    crypto::{AesOfb, EncSize}
};
use rand::{
    thread_rng, 
    Rng,
    distributions::Alphanumeric
};

pub mod settings_commands {
    pub const MENU: &str = "Settings ⚙️";
    pub const LIST_DIRECTORIES: &str = "List Directories 📂";
    pub const SERVER_STATS: &str = "Server stats 👀";
    pub const RESET_DIRECTORIES: &str = "Reset Directories 📂❌";
}

fn random_salt() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(AesOfb::nonce_size())
        .collect()
}

pub async fn start_command(api: Api, pool: &Pool, message: Message) -> Result<(), BotError> {
    let m_clone = message.clone();
    let mut keyboard = ReplyKeyboardMarkup::new();
    keyboard.add_row(vec![
        KeyboardButton::new(settings_commands::MENU)
    ]);
    keyboard.resize_keyboard();
    match get_user(pool, &TelegramId::from(m_clone.from.id)).await {
        Ok(result) => {
            match result {
                Some(_) => {
                    api.send(&message.to_source_chat().text(format!("Welcome back: {}", &message.from.first_name)).reply_markup(keyboard)).await?;
                },
                None => {
                    let user=  DbUser {
                        id: m_clone.from.id.into(),
                        chat: m_clone.chat.id().into(),
                        first_name: m_clone.from.first_name,
                        last_name: m_clone.from.last_name,
                        username: m_clone.from.username,
                        salt: random_salt(),
                    };
                    save_user(pool, user).await?;
                    api.send(&message.to_source_chat().text(format!("Welcome: {}", &message.from.first_name)).reply_markup(keyboard)).await?;
                }
            }
        },
        Err(_) => {
            api.send(&message.to_source_chat().text("Something went wrong :-(").reply_markup(keyboard)).await?; 
        }
    }
    Ok(())
}

pub async fn settings_menu(api: Api, message: Message) -> Result<(), Error> {
    let mut keyboard = ReplyKeyboardMarkup::new();
    keyboard.add_row(vec![KeyboardButton::new(settings_commands::LIST_DIRECTORIES)]);
    keyboard.add_row(vec![KeyboardButton::new(settings_commands::SERVER_STATS)]);
    // list directories
        // -> add directory
        // -> reset directories
    // show stats
        // -> reset server settings
    api.send(message.to_source_chat().text(settings_commands::MENU).reply_markup(keyboard)).await?;
    Ok(())
}