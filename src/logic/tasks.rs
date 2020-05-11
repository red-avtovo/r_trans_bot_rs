use telegram_bot::*;
use crate::errors::BotError;
use super::{
    repository::{
        Pool,
        get_user,
        register_magnet,
        get_servers_by_user_id,
        get_directories,
        get_magnet_by_id,
        get_directory,
        add_task,
    },
    models:: {
        TelegramId,
        ShortMagnet,
        DownloadDirectory
    },
    servers::servers_commands,
    directories::direcoties_commands,
};
use log::*;
use uuid::Uuid;
use transmission_rpc::{
    TransClient,
    types::{
        TorrentAddArgs,
        TorrentAdded
    }
};

pub async fn start_download(api: &Api, pool: &Pool, user_id: &TelegramId, data: &str, chat_ref: &ChatRef) -> Result<(), BotError> {
    let data_parts: Vec<String> = data.split(":")
        .map(|part| String::from(part))
        .collect();
    if data_parts.len()!=3 {
        error!("Broken download callback received: {}", &data);
        api.send(chat_ref.text("We messed up. Can't start donwloading :(")).await?;
        return Ok(())    
    }
    let magnet_id = Uuid::parse_str(data_parts[1].as_ref()).expect("Incorrect uuid received");
    let dir_ordinal = data_parts[2].parse::<i32>().unwrap();
    let user = &get_user(pool, user_id).await?.unwrap();
    let magnet = get_magnet_by_id(pool, user, magnet_id).await?.unwrap();
    let dir = get_directory(pool, user, dir_ordinal).await?;
    
    let servers = get_servers_by_user_id(pool, user).await?;
    if servers.len() == 0 {
        let mut keyboard = InlineKeyboardMarkup::new();
        keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::REGISTER_SERVER, servers_commands::REGISTER_SERVER)]);
        api.send(chat_ref.text("No Servers found! Please register one first!").reply_markup(keyboard)).await?;
        return Ok(());
    }

    match dir {
        Some(dir) => {
            let server = servers.get(0).unwrap();
            let short = ShortMagnet::from(&magnet.url).unwrap();
            add_task(pool, user, &servers[0].id, &magnet.clone()).await?;
            let client: TransClient = server.to_client();
            match client.torrent_add(TorrentAddArgs{
                filename: Some(short.into()),
                download_dir: Some(dir.path),
                ..TorrentAddArgs::default()
            }).await {
                Ok(response) => {
                    let arguments: TorrentAdded = response.arguments;
                    let name: String = arguments.torrent_added.name
                        .unwrap_or(arguments.torrent_added.hash_string.unwrap());
                    
                    api.send(chat_ref.text(format!("Downloading {}", &name))).await?;
                }
                Err(_) => {
                    api.send(chat_ref.text("Unable to add task")).await?;
                }
            }
        },
        None => {
            let mut keyboard = InlineKeyboardMarkup::new();
            keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::ADD_DIRECTORY, direcoties_commands::ADD_DIRECTORY)]);
            api.send(chat_ref.text("No Directories found! Please add one first!").reply_markup(keyboard)).await?;
        }
    }
    Ok(())
}

pub async fn process_magnet(api: Api, pool: &Pool, message: Message) -> Result<(), BotError> {
    let text = message.text().unwrap_or_default();
    let magnet = ShortMagnet::find(&text);
    match magnet {
        Some(short) => {
            let user = &get_user(pool, &message.from.id.into()).await?.unwrap();
            let server_count = get_servers_by_user_id(pool, user).await?.len();
            if server_count == 0 {
                let mut keyboard = InlineKeyboardMarkup::new();
                keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::REGISTER_SERVER, servers_commands::REGISTER_SERVER)]);
                api.send(message.to_source_chat().text("No Servers found! Please register one first!").reply_markup(keyboard)).await?;
                return Ok(());
            }
            
            let dirs: Vec<DownloadDirectory> = get_directories(pool, user).await?;
            if dirs.len() == 0 {
                let mut keyboard = InlineKeyboardMarkup::new();
                keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::ADD_DIRECTORY, direcoties_commands::ADD_DIRECTORY)]);
                api.send(message.to_source_chat().text("No Directories found! Please add one first!").reply_markup(keyboard)).await?;
                return Ok(());
            }
            
            let magnet_id = register_magnet(pool, user, &short.into()).await?;
            let mut keyboard = InlineKeyboardMarkup::new();
            for dir in dirs {
                keyboard.add_row(vec![InlineKeyboardButton::callback(dir.alias,format!("download:{}:{}", &magnet_id, &dir.ordinal))]);
            }
            api.send(message.text_reply("Choose directory to download").reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard)))
                .await?;    
        },
        None => {
            error!("Couldn't parse magnet from text: {}", &text);
            api.send(message.text_reply("Sorry. Couldn't handle this magnet. Try later :(")).await?;
        }
    };
    Ok(())
}