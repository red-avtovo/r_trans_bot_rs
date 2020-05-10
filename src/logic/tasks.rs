use telegram_bot::*;
use crate::errors::BotError;
use super::{
    repository::{
        Pool,
        get_user,
        register_magnet,
        get_servers_by_user_id,
        get_directories,
    },
    models:: {
        TelegramId,
        ShortMagnet
    },
    servers::servers_commands,
};
use log::*;

pub async fn start_download(api: &Api, data: &str, chat_ref: &ChatRef) -> Result<(), Error> {
    let rq = chat_ref.text(format!("Callback: {}", data));
    api.send(rq).await?;
    Ok(())
}

pub async fn process_magnet(api: Api, pool: &Pool, message: Message) -> Result<(), BotError> {
    let mut markup = InlineKeyboardMarkup::new();
    markup.add_row(
        vec![
            InlineKeyboardButton::callback(
                "test".to_string(),
                "callback button".to_string()
            )
        ]
    );
    let text = message.text().unwrap_or_default();
    let magnet = ShortMagnet::find(text);
    match magnet {
        Some(short) => {
            let user = &get_user(pool, &message.from.id.into()).await?.unwrap();
            let server_count = get_servers_by_user_id(pool, user).await?.len();
            if server_count==0 {
                let mut keyboard = InlineKeyboardMarkup::new();
                keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::REGISTER_SERVER, servers_commands::REGISTER_SERVER)]);
                api.send(message.to_source_chat().text("No Servers found! Please register one first!").reply_markup(keyboard)).await?;
                return Ok(());
            }
            let magnet_id = register_magnet(pool, user, &short.into()).await?;
            api.send(message.text_reply("I see magnet").reply_markup(ReplyMarkup::InlineKeyboardMarkup(markup)))
                .await?;       
        },
        None => {
            
        }
    }
    
    // extract magnet link
    // parse and clean the link
    // create a task
    // offer directories
    Ok(())
}