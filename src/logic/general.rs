use telegram_bot::*;
use crate::errors::BotError;
use crate::logic::db::{save_user, DbUser, Pool};

pub mod settings_commands {
    pub const MENU: &str = "Settings ⚙️";
    pub const LIST_DIRECTORIES: &str = "List Directories 📂";
    pub const RESET_DIRECTORIES: &str = "Reset Directories 📂❌";
}

pub async fn start_command(api: Api, pool: &Pool, message: Message) -> Result<(), BotError> {
    let m_clone = message.clone();
    let user=  DbUser {
        id: m_clone.from.id.into(),
        chat: m_clone.chat.id().into(),
        first_name: m_clone.from.first_name,
        last_name: m_clone.from.last_name,
        username: m_clone.from.username
    };
    
    save_user(pool, user).await?;

    let mut keyboard = ReplyKeyboardMarkup::new();
    keyboard.add_row(vec![
        KeyboardButton::new(settings_commands::MENU)
    ]);
    keyboard.resize_keyboard();
    api.send(&message.to_source_chat().text(format!("Welcome: {}", &message.from.first_name)).reply_markup(keyboard)).await?;
    Ok(())
}

pub async fn settings_menu(api: Api, message: Message) -> Result<(), Error> {
    let mut keyboard = ReplyKeyboardMarkup::new();
    keyboard.add_row(vec![KeyboardButton::new(settings_commands::LIST_DIRECTORIES)]);
    keyboard.add_row(vec![KeyboardButton::new(settings_commands::RESET_DIRECTORIES)]);
    // list directories
        // -> add directory
        // -> reset directories
    // show stats
        // -> reset server settings
    api.send(message.to_source_chat().text(settings_commands::MENU).reply_markup(keyboard)).await?;
    Ok(())
}
