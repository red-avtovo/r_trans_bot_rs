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
        DownloadDirectory,
        Server,
        User
    },
    servers::servers_commands,
    directories::direcoties_commands,
    magnet::MagnetLink
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
use chrono::prelude::*;

pub mod task_commands {
    pub const TASK_STATUS: &str = "Update task status 👀";
    pub const TASK_REMOVE: &str = "Delete torrent ❌";
    pub const TASK_HIDE: &str = "Hide 🙈";
}

async fn get_server(api: &Api, pool: &Pool, user: &User, chat_ref: &ChatRef) -> Option<Server> {
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

fn update_task_status_button(task_id: &Uuid, torrent: &Torrent) -> InlineKeyboardMarkup {
    let percent = torrent.percent_done;
    let mut keyboard = InlineKeyboardMarkup::new();
    match percent {
        Some(ref value) if value.to_owned() == 1.0_f32 => {
            keyboard.add_row(vec![InlineKeyboardButton::callback(task_commands::TASK_REMOVE, format!("t_remove:{}", &task_id))]);
            keyboard.add_row(vec![InlineKeyboardButton::callback(task_commands::TASK_HIDE, task_commands::TASK_HIDE)]);
        },
        _ => {
            keyboard.add_row(vec![InlineKeyboardButton::callback(task_commands::TASK_STATUS, format!("t_status:{}", &task_id))]);
        }
    }
    keyboard
}

fn hide_message_button() -> InlineKeyboardMarkup {
    let mut keyboard = InlineKeyboardMarkup::new();
    keyboard.add_row(vec![InlineKeyboardButton::callback(task_commands::TASK_HIDE, task_commands::TASK_HIDE)]);
    keyboard
}

pub async fn start_download(api: &Api, pool: &Pool, user_id: &i64, data: &str, chat_ref: &ChatRef) -> Result<(), BotError> {
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
            let magnet_link = MagnetLink::from(&magnet.url).unwrap();
            let task = add_task(pool, user, &server.id, &magnet.clone()).await?;
            let client: TransClient = server.to_client();
            match client.torrent_add(TorrentAddArgs{
                filename: Some(magnet_link.clone().short_link()),
                download_dir: Some(dir.path),
                ..TorrentAddArgs::default()
            }).await {
                Ok(response) => {
                    let arguments: TorrentAdded = response.arguments;
                    let torrent = arguments.torrent_added.unwrap();
                    let name: String = magnet_link.dn();
                    api.send(chat_ref.text(format!("Downloading {}\nto {}", &name, &dir.alias))
                    .reply_markup(update_task_status_button(&task.id, &torrent))).await?;
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

pub async fn update_task_status(api: &Api, pool: &Pool, user_id: &i64, data: &str, message: &Message) -> Result<(), BotError> {
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

    let link = MagnetLink::from(&magnet.url).unwrap();
    let hash = link.clone().hash();
    let client: TransClient = server.to_client();
    match client.torrent_get(None, Some(vec![Id::Hash(hash.clone())])).await {
        Ok(response) => {
            match response.arguments.torrents.iter().next() {
                Some(torrent) => {
                    let name = torrent.name.as_ref().unwrap_or(&hash);
                    api.send(message.edit_text(format!("Downloading {}\n{}", &name, torrent_status(torrent)))
                        .reply_markup(update_task_status_button(&task.id, torrent))).await?;
                },
                None => {
                    api.send(message.edit_text(format!("Torrent\n{}\nwas removed", &link.dn()))).await?;
                }
            }
        },
        _ => {
            api.send(message.edit_text(format!("{}\nTorrent was not found on the server!", &hash))).await?;
        }
    }
    Ok(())
}

pub async fn remove_task(api: &Api, pool: &Pool, user_id: &i64, data: &str, message: &Message) -> Result<(), BotError> {
    let data_parts: Vec<String> = data.split(":")
        .map(|part| String::from(part))
        .collect();
    if data_parts.len()!=2 {
        error!("Broken task removal callback received: {}", &data);
        api.send(message.from.to_chat_ref().text("We messed up. Can't remove the task :(")).await?;
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

    let link = MagnetLink::from(&magnet.url).unwrap();
    let hash = link.clone().hash();
    let client: TransClient = server.to_client();
    match client.torrent_remove(vec![Id::Hash(hash.clone())], true).await {
        Ok(_) => {
            api.send(message.edit_text(format!("Torrent\n{}\nwas removed!", &link.dn()))
                .reply_markup(hide_message_button())
            ).await?;
        },
        _ => {
            return Err(BotError::logic(format!("Failed to remove the torrent: {}", link.dn())))
        }
    };
    Ok(())
}

fn torrent_status(torrent: &Torrent) -> String {
    let percent = match torrent.percent_done {
        Some(percent) => (percent * 100.0) as i32,
        _ => return String::default()
    };
    let filled: String = (0..percent/10).map(|_|"❇️").collect();
    let empty: String = (percent/10..10).map(|_|"◻️").collect();
    format!("{}{}[{}%]\nUpdated at: {}", filled, empty, percent, Utc::now().format("%d.%m.%Y %H:%M:%S"))
}

pub async fn process_magnet(api: &Api, pool: &Pool, message: &Message) -> Result<(), BotError> {
    let text = message.text().unwrap_or_default();
    let magnet = MagnetLink::find(&text);
    match magnet {
        Some(link) => {
            let user = &get_user(pool, &message.from.id.into()).await?.unwrap();
            let server_count = get_servers_by_user_id(pool, user).await?.len();
            if server_count == 0 {
                let mut keyboard = InlineKeyboardMarkup::new();
                keyboard.add_row(vec![InlineKeyboardButton::callback(servers_commands::REGISTER_SERVER, servers_commands::REGISTER_SERVER)]);
                let err_message = "No Servers found! Please register one first!".to_owned();
                api.send(message.to_source_chat().text(&err_message).reply_markup(keyboard)).await?;
                return Err(BotError::logic(err_message));
            }

            let dirs: Vec<DownloadDirectory> = get_directories(pool, user).await?;
            if dirs.len() == 0 {
                let mut keyboard = InlineKeyboardMarkup::new();
                keyboard.add_row(vec![InlineKeyboardButton::callback(direcoties_commands::ADD_DIRECTORY, direcoties_commands::ADD_DIRECTORY)]);
                let err_message = "No Directories found! Please add one first!".to_owned();
                api.send(message.to_source_chat().text(&err_message).reply_markup(keyboard)).await?;
                return Err(BotError::logic(err_message));
            }

            let magnet_id = register_magnet(pool, user, &link.clone().full_link()).await?;
            let mut keyboard = InlineKeyboardMarkup::new();
            for dir in dirs {
                keyboard.add_row(vec![InlineKeyboardButton::callback(dir.alias,format!("download:{}:{}", &magnet_id, &dir.ordinal))]);
            }
            keyboard.add_row(vec![InlineKeyboardButton::callback("-- Cancel --","cancel")]);

            api.send(message.to_source_chat().text(format!("{}\nChoose directory to download", link.dn()))
                .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))).await?;
        },
        None => {
            let err_message = format!("Couldn't parse magnet from text: {}", &text);
            error!("{}", &err_message);
            api.send(message.text_reply("Sorry. Couldn't handle this magnet. Try later :(")).await?;
            return Err(BotError::logic(err_message));
        }
    };
    Ok(())
}
