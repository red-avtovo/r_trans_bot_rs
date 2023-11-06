use teloxide::{Bot, dptree, RequestError};
use teloxide::dispatching::{dialogue, UpdateFilterExt, UpdateHandler};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use teloxide::types::Update;

use crate::conversation::commands::*;
use crate::conversation::directories::*;
use crate::conversation::friends::{confirm_unfriend_callback, manage_friend_callback, unfriend_callback};
use crate::conversation::messages::*;
use crate::conversation::servers::*;
use crate::conversation::tasks::*;
use crate::db::repository::Pool;

pub type BotDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    AddDirectory,
    RegisterServer,
    AddFriend,
    ServerSharing,
}

pub(crate) fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help_command))
        .branch(case![Command::Start].endpoint(start_command))
        .branch(case![Command::Settings].endpoint(settings_command))
        .branch(case![Command::Cancel].endpoint(cancel_command))
        .branch(case![Command::AddFriend].endpoint(add_friend_command))
        .branch(case![Command::ListFriends].endpoint(list_friends_command))
        ;

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Start].endpoint(process_message))
        .branch(case![State::AddDirectory].endpoint(add_directory_dialogue))
        .branch(case![State::RegisterServer].endpoint(register_server_dialogue))
        .branch(case![State::AddFriend].endpoint(add_friend_dialogue))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query()
        .branch(dptree::endpoint(process_callback));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}


async fn process_callback(
    bot: Bot,
    dialogue: BotDialogue,
    pool: Pool,
    callback_query: CallbackQuery,
) -> HandlerResult {
    let user_id: &u64 = &callback_query.from.id.0;
    let chat_id = &dialogue.chat_id();
    let message = match callback_query.message {
        Some(msg) => msg,
        None => return Ok(())
    };
    let data = match callback_query.data {
        Some(s) => s,
        None => return Ok(())
    };

    match data.as_str() {
        // download:magnet_uuid:directory_ordinal (1-64 bytes)
        value if value.starts_with("download:") => {
            start_download(&bot, &pool, chat_id, user_id, value).await?
        }
        // t_status:task_uuid
        value if value.starts_with("t_status:") => {
            update_task_status(&bot, &pool, user_id, value, &message).await?
        }
        // t_remove:task_uuid
        value if value.starts_with("t_remove:") => {
            remove_task(&bot, &pool, user_id, value, &message).await?
        }
        // manage_friend:user_id
        value if value.starts_with("manage_friend:") => {
            manage_friend_callback(&bot, &pool, value, &message).await?
        }
        // unfriend:user_id
        value if value.starts_with("unfriend:") => {
            unfriend_callback(&bot, &pool, value, &message).await?
        }
        // confirm_unfriend:user_id
        value if value.starts_with("confirm_unfriend:") => {
            confirm_unfriend_callback(&bot, &pool, user_id, value, &message).await?
        }
        // static commands
        directories_commands::LIST_DIRECTORIES => {
            list_directories(&bot, &pool, user_id, chat_id).await?
        }
        directories_commands::ADD_DIRECTORY => {
            add_directory_prepare(&bot, &chat_id).await?;
            dialogue.update(State::AddDirectory).await?;
        }
        directories_commands::RESET_DIRECTORIES => {
            reset_directories(&bot, &pool, user_id, chat_id).await?
        }
        servers_commands::SERVER_STATS => show_stats(&bot, &pool, user_id, chat_id).await?,
        servers_commands::RESET_SERVERS => reset_servers(&bot, &pool, user_id, chat_id).await?,
        servers_commands::REGISTER_SERVER => {
            let result = register_server_prepare(&bot, &pool, user_id, chat_id).await?;
            if result {
                dialogue.update(State::RegisterServer).await?;
            };
        }
        servers_commands::BACK_TO_SETTINGS => {
            back_to_settings_command(&bot,  &message).await?;
        }
        directories_commands::BACK_TO_SETTINGS => {
            back_to_settings_command(&bot,  &message).await?;
        }
        _ => {}
    };
    match data {
        s if s.starts_with("t_status:") => {}
        s if s.starts_with("t_remove:") => {}
        s if s.starts_with(servers_commands::BACK_TO_SETTINGS) => {}
        s if s.starts_with(directories_commands::BACK_TO_SETTINGS) => {}
        _ => delete_or_hide(&bot, &message).await?
    }
    bot.answer_callback_query(callback_query.id).await?;
    Ok(())
}

async fn delete_or_hide(bot: &Bot, message: &Message) -> Result<(), RequestError> {
    match bot.delete_message(message.chat.id, message.id).await {
        Ok(_) => Ok(()),
        // After 24h it is impossible to delete bot message
        Err(_) => bot.edit_message_text(message.chat.id, message.id, "-- Hidden --")
            .await
            .map(|_| ()),
    }
}

