use crate::db::{
    models::user::NewUser,
    repository::{get_user, save_user, Pool},
};
use crate::errors::BotError;
use crate::core::crypto::random_salt;

use crate::conversation::{
    friends::list_friends,
    directories::directories_commands,
    servers::servers_commands,
};
use log::*;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::requests::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardMarkup, KeyboardRemove, True};
use teloxide::utils::command::BotCommands;

use crate::router::{BotDialogue, HandlerResult, State};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start working with bot")]
    Start,
    #[command(description = "show settings")]
    Settings,
    #[command(description = "cancel the command")]
    Cancel,
    #[command(description = "add friend")]
    AddFriend,
    #[command(description = "list friends")]
    ListFriends,
    #[command(description = "list servers")]
    ListServers,
    #[command(description = "manage server sharing")]
    ServerSharing,
}

pub mod settings_commands {
    pub const MENU: &str = "Settings ⚙️";
}

fn settings_buttons() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        vec![
            vec![InlineKeyboardButton::callback(
                directories_commands::LIST_DIRECTORIES,
                directories_commands::LIST_DIRECTORIES,
            )],
            vec![InlineKeyboardButton::callback(
                servers_commands::SERVER_STATS,
                servers_commands::SERVER_STATS,
            )],
        ]
    )
}

pub async fn settings_command(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, settings_commands::MENU)
        .reply_markup(settings_buttons())
        .await?;
    Ok(())
}

pub async fn back_to_settings_command(bot: &Bot, message: &Message) -> HandlerResult {
    bot.edit_message_text(message.chat.id, message.id, settings_commands::MENU)
        .reply_markup(settings_buttons())
        .await?;
    Ok(())
}

pub async fn help_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

pub async fn start_command(bot: Bot, pool: Pool, message: Message) -> HandlerResult {
    debug!("Handle /start command");
    let m_clone = message.clone();
    debug!("Checking if user already exist");
    match get_user(&pool, &(m_clone.from().unwrap().id.0 as i64)).await {
        Ok(result) => match result {
            Some( user ) => {
                debug!("User was already registered");
                bot.send_message(message.chat.id, format!("Welcome back, {}", user.first_name))
                    .reply_markup(KeyboardRemove { remove_keyboard: True, selective: None })
                    .await?;
            }
            None => {
                debug!("Registering new user");
                match m_clone.from() {
                    Some(chat_user) => {
                        let user = NewUser {
                            id: chat_user.id.0 as i64,
                            chat: m_clone.chat.id.0,
                            first_name: chat_user.first_name.clone(),
                            last_name: chat_user.last_name.clone(),
                            username: chat_user.username.clone(),
                            salt: random_salt(),
                        };
                        match save_user(&pool, user).await {
                            Ok(_) => {
                                bot.send_message(message.chat.id, format!("Welcome, {}", chat_user.first_name))
                                    .await?;
                            }
                            Err(err) => {
                                bot.send_message(message.chat.id, format!("Unable to save user: {}", &err))
                                    .await?;
                            }
                        };

                    },
                    None => {
                        bot.send_message(message.chat.id, "Unable to register user")
                            .await?;
                    }
                }
            }
        },
        Err(error) => return Err(BotError::from(error).into()),
    }
    Ok(())
}

pub async fn cancel_command(bot: Bot, dialogue: BotDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Forgetting what we talked about").await?;
    dialogue.exit().await?;
    Ok(())
}

pub async fn add_friend_command(bot: Bot, dialogue: BotDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Share any message with your friend and I add him/her").await?;
    dialogue.update(State::AddFriend).await?;
    Ok(())
}

pub async fn list_friends_command(bot: Bot, pool: Pool, msg: Message) -> HandlerResult {
    let user = msg.from().unwrap();
    list_friends(&bot, &pool, &user.id.0, &msg.chat.id).await?;
    Ok(())
}