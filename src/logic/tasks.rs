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
        get_task_by_id,
    },
    models:: {
        TelegramId,
        ShortMagnet,
        DownloadDirectory,
        Server,
        DbUser
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
        TorrentAdded,
        Id,
        Torrent,
    }
};

pub mod task_commands {
    pub const TASK_STATUS: &str = "Update task status 👀";
}

async fn get_server(api: &Api, pool: &Pool, user: &DbUser, chat_ref: &ChatRef) -> Option<Server> {
    let servers = get_servers_by_user_id(pool, user).await;
    match servers {
        Ok(ref servers) if servers.len() == 0 => {
            let mut keyboard = InlineKeyboardMarkup::new();
            keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::REGISTER_SERVER, servers_commands::REGISTER_SERVER)]);
            let _res = api.send(chat_ref.text("No Servers found! Please register one first!").reply_markup(keyboard)).await;
            None
        }
        Ok(ref servers) => Some(servers.get(0).unwrap().clone()),
        _ => None
    }
}

fn update_task_status_button(task_id: &Uuid) -> InlineKeyboardMarkup {
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(task_commands::TASK_STATUS, format!("t_status:{}", &task_id))]);
    keyboard
}

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
    
    let server = match get_server(api, pool, user, chat_ref).await {
        Some(server) => server,
        None => return Ok(())
    };

    match dir {
        Some(dir) => {
            let short = ShortMagnet::from(&magnet.url).unwrap();
            let task = add_task(pool, user, &server.id, &magnet.clone()).await?;
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
                    
                    api.send(chat_ref.text(format!("Downloading {}", &name))
                    .reply_markup(update_task_status_button(&task.id))).await?;
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

pub async fn update_task_status(api: &Api, pool: &Pool, user_id: &TelegramId, data: &str, message: &Message) -> Result<(), BotError> {
    let data_parts: Vec<String> = data.split(":")
        .map(|part| String::from(part))
        .collect();
    if data_parts.len()!=2 {
        error!("Broken task status callback received: {}", &data);
        api.send(message.from.to_chat_ref().text("We messed up. Can't check the status :(")).await?;
        return Ok(())    
    }
    let task_id = Uuid::parse_str(data_parts[1].as_ref()).expect("Incorrect uuid received");
    let user = &get_user(pool, user_id).await?.unwrap();
    let task = get_task_by_id(pool, &task_id).await?.unwrap();
    let server = match get_server(api, pool, user, &message.from.to_chat_ref()).await {
        Some(server) => server,
        None => return Ok(())
    };
    let magnet = match get_magnet_by_id(pool, user, task.magnet_id).await {
        Ok(ref link) if link.is_some() => link.clone().unwrap(),
        _ => return Ok(())
    };
    
    let short = ShortMagnet::from(&magnet.url).unwrap();
    let hash = short.hash();
    let client: TransClient = server.to_client();
    match client.torrent_get(None, Some(vec![Id::Hash(hash.clone())])).await {
        Ok(response) => {
            let torrent = response.arguments.torrents.iter().next().unwrap();
            let name = torrent.name.as_ref().unwrap_or(&hash);
            api.send(message.edit_text(format!("{}\n{}", &name, torrent_status(torrent)))
                .reply_markup(update_task_status_button(&task.id))).await?;
        },
        _ => {
            api.send(message.edit_text(format!("{}\nTorrent was not found on the server!", &hash))).await?;
        }
    }
    Ok(())
}

fn torrent_status(torrent: &Torrent) -> String {
    let percent = match torrent.percent_done {
        Some(percent) => percent as i32,
        _ => return String::default()
    };
    let filled: String = (0..percent/10).map(|_|"◾️").collect();
    let empty: String = (percent/10..10).map(|_|"◻️").collect();
    format!("{}{}[{}%]", filled, empty, percent)
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