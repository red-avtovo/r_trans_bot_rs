use chrono::prelude::*;
use log::*;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use transmission_rpc::{
    TransClient,
    types::{Id, Torrent, TorrentAddArgs, TorrentAddedOrDuplicate},
};
use uuid::Uuid;

use crate::conversation::directories::directories_commands;
use crate::conversation::servers::servers_commands;
use crate::core::magnet::MagnetLink;
use crate::db::{
    models::{directories::DownloadDirectory, server::Server, user::User},
    repository::{
        add_task, get_directories, get_directory, get_magnet_by_id, get_servers_by_user_id,
        get_task_by_id, get_user, Pool, register_magnet,
    },
};
use crate::errors::BotError;
use crate::errors::BotErrorKind::BotLogic;

pub mod task_commands {
    pub const TASK_STATUS: &str = "Update task status 👀";
    pub const TASK_REMOVE: &str = "Delete torrent ❌";
    pub const TASK_HIDE: &str = "Hide 🙈";
}

async fn get_server(bot: &Bot, pool: &Pool, user: &User, chat_id: &ChatId) -> Option<Server> {
    let servers = get_servers_by_user_id(pool, user).await;
    match servers {
        Ok(ref servers) if servers.len() == 0 => {
            let keyboard = InlineKeyboardMarkup::new(
                vec![vec![InlineKeyboardButton::callback(
                    servers_commands::REGISTER_SERVER,
                    servers_commands::REGISTER_SERVER,
                )]]
            );
            let _res = bot.send_message(
                *chat_id,
                "No Servers found! Please register one first!",
            ).reply_markup(keyboard).await;
            None
        }
        Ok(ref servers) => Some(servers.get(0).unwrap().clone()),
        _ => None,
    }
}

fn update_task_status_button(task_id: &Uuid, torrent: &Torrent) -> InlineKeyboardMarkup {
    let percent = torrent.percent_done;

    match percent {
        Some(ref value) if value.to_owned() == 1.0_f32 => {
            InlineKeyboardMarkup::new(
                vec![vec![InlineKeyboardButton::callback(
                    task_commands::TASK_REMOVE,
                    format!("t_remove:{}", &task_id),
                )],
                     vec![InlineKeyboardButton::callback(
                         task_commands::TASK_HIDE,
                         task_commands::TASK_HIDE,
                     )]],
            )
        }
        _ => {
            InlineKeyboardMarkup::new(
                vec![vec![InlineKeyboardButton::callback(
                    task_commands::TASK_STATUS,
                    format!("t_status:{}", &task_id),
                )]]
            )
        }
    }
}

fn hide_message_button() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        task_commands::TASK_HIDE,
        task_commands::TASK_HIDE,
    )]])
}

pub async fn start_download(
    bot: &Bot,
    pool: &Pool,
    chat_id: &ChatId,
    user_id: &u64,
    data: &str,
) -> Result<(), BotError> {
    let data_parts: Vec<String> = data.split(":").map(|part| String::from(part)).collect();
    if data_parts.len() != 3 {
        error!("Broken download callback received: {}", &data);
        bot.send_message(*chat_id, "We messed up. Can't start downloading :(")
            .await?;
        return Ok(());
    }
    let magnet_id = Uuid::parse_str(data_parts[1].as_ref()).expect("Incorrect uuid received");
    let dir_ordinal = data_parts[2].parse::<i32>().unwrap();
    let user = &get_user(pool, &(*user_id as i64)).await?.unwrap();
    let magnet = get_magnet_by_id(pool, user, magnet_id).await?.unwrap();
    let dir = get_directory(pool, user, dir_ordinal).await?;

    let server = match get_server(bot, pool, user, chat_id).await {
        Some(server) => server,
        None => return Ok(()),
    };

    match dir {
        Some(dir) => {
            let magnet_link = MagnetLink::from(&magnet.url).unwrap();
            let mut client: TransClient = server.to_client();
            match client
                .torrent_add(TorrentAddArgs {
                    filename: Some(magnet_link.clone().short_link()),
                    download_dir: Some(dir.path),
                    ..TorrentAddArgs::default()
                })
                .await
            {
                Ok(response) => {
                    let arguments: TorrentAddedOrDuplicate = response.arguments;
                    match arguments {
                        TorrentAddedOrDuplicate::TorrentAdded(torrent) => {
                            let task = add_task(pool, user, &server.id, &magnet.clone()).await?;
                            let name: String = magnet_link.dn();
                            bot.send_message(*chat_id, format!("Downloading {}\nto {}", &name, &dir.alias))
                                .reply_markup(update_task_status_button(&task.id, &torrent))
                                .await?;
                        }
                        TorrentAddedOrDuplicate::TorrentDuplicate(_) => {
                            bot.send_message(*chat_id, "Such task already exists")
                                .await?;
                        }
                    }
                }
                Err(_) => {
                    bot.send_message(*chat_id, "Unable to add task")
                        .await?;
                }
            }
        }
        None => {
            let keyboard = InlineKeyboardMarkup::new(
                vec![vec![InlineKeyboardButton::callback(
                    directories_commands::ADD_DIRECTORY,
                    directories_commands::ADD_DIRECTORY,
                )]]
            );
            bot.send_message(*chat_id, "No Directories found! Please add one first!")
                .reply_markup(keyboard)
                .await?;
        }
    }
    Ok(())
}

pub async fn update_task_status(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    data: &str,
    message: &Message,
) -> Result<(), BotError> {
    let data_parts: Vec<String> = data.split(":").map(|part| String::from(part)).collect();
    if data_parts.len() != 2 {
        error!("Broken task status callback received: {}", &data);
        bot.send_message(message.chat.id, "We messed up. Can't check the status :(")
            .await?;
        return Ok(());
    }
    let task_id = Uuid::parse_str(data_parts[1].as_ref()).expect("Incorrect uuid received");
    let user = &get_user(pool, &(*user_id as i64)).await?.unwrap();
    let task = match get_task_by_id(pool, &task_id).await? {
        Some(task) => task,
        None => return Err(BotError::logic("No task found!".to_string())),
    };
    let server = match get_server(bot, pool, user, &message.chat.id).await {
        Some(server) => server,
        None => return Ok(()),
    };
    let magnet = match get_magnet_by_id(pool, user, task.magnet_id).await {
        Ok(ref link) if link.is_some() => link.clone().unwrap(),
        _ => return Ok(()),
    };

    let link = MagnetLink::from(&magnet.url).unwrap();
    let hash = link.clone().hash();
    let mut client: TransClient = server.to_client();
    match client
        .torrent_get(None, Some(vec![Id::Hash(hash.clone())]))
        .await
    {
        Ok(response) => match response.arguments.torrents.iter().next() {
            Some(torrent) => {
                let name = torrent.name.as_ref().unwrap_or(&hash);
                bot.edit_message_text(
                    message.chat.id,
                    message.id,
                    format!(
                        "Downloading {}\n{}",
                        &name,
                        torrent_status(torrent)
                    ),
                ).reply_markup(update_task_status_button(&task.id, torrent))
                    .await?;
            }
            None => {
                bot.edit_message_text(
                    message.chat.id,
                    message.id,
                    format!("Torrent\n{}\nwas removed", &link.dn()),
                ).reply_markup(hide_message_button()).await?;
            }
        },
        _ => {
            bot.edit_message_text(
                message.chat.id,
                message.id,
                format!("{}\nTorrent was not found on the server!", &hash),
            ).await?;
        }
    }
    Ok(())
}

pub async fn remove_task(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    data: &str,
    message: &Message,
) -> Result<(), BotError> {
    let data_parts: Vec<String> = data.split(":").map(|part| String::from(part)).collect();
    if data_parts.len() != 2 {
        error!("Broken task removal callback received: {}", &data);
        bot.send_message(message.chat.id, "We messed up. Can't remove the task :(")
            .await?;
        return Ok(());
    }
    let task_id = Uuid::parse_str(data_parts[1].as_ref()).expect("Incorrect uuid received");
    let user = &get_user(pool, &(*user_id as i64)).await?.unwrap();
    let task = get_task_by_id(pool, &task_id).await?.unwrap();
    let server = match get_server(bot, pool, user, &message.chat.id).await {
        Some(server) => server,
        None => return Ok(()),
    };
    let magnet = match get_magnet_by_id(pool, user, task.magnet_id).await {
        Ok(ref link) if link.is_some() => link.clone().unwrap(),
        _ => return Ok(()),
    };

    let link = MagnetLink::from(&magnet.url).unwrap();
    let hash = link.clone().hash();
    let mut client: TransClient = server.to_client();
    match client
        .torrent_remove(vec![Id::Hash(hash.clone())], true)
        .await
    {
        Ok(_) => {
            bot.edit_message_text(
                message.chat.id,
                message.id,
                format!("Torrent\n{}\nwas removed", &link.dn()),
            ).reply_markup(hide_message_button()).await?;
        }
        _ => {
            return Err(BotError::logic(format!(
                "Failed to remove the torrent: {}",
                link.dn()
            )));
        }
    };
    Ok(())
}

fn torrent_status(torrent: &Torrent) -> String {
    let percent = match torrent.percent_done {
        Some(percent) => (percent * 100.0) as i32,
        _ => return String::default(),
    };
    let filled: String = (0..percent / 10).map(|_| "❇️").collect();
    let empty: String = (percent / 10..10).map(|_| "◻️").collect();
    format!(
        "{}{}[{}%]\nUpdated at: {}",
        filled,
        empty,
        percent,
        Utc::now().format("%d.%m.%Y %H:%M:%S")
    )
}

pub async fn process_magnet(
    bot: &Bot,
    pool: &Pool,
    message: &Message,
    link: &String,
) -> Result<(), BotError> {
    let magnet = MagnetLink::find(link);
    match magnet {
        Some(link) => {
            let user = &get_user(pool, &(message.from().unwrap().id.0 as i64)).await?.unwrap();
            let server_count = get_servers_by_user_id(pool, user).await?.len();
            if server_count == 0 {
                let keyboard = InlineKeyboardMarkup::new(
                    vec![vec![InlineKeyboardButton::callback(
                        servers_commands::REGISTER_SERVER,
                        servers_commands::REGISTER_SERVER,
                    )]]
                );
                let err_message = "No Servers found! Please register one first!".to_owned();
                bot.send_message(message.chat.id, &err_message).reply_markup(keyboard)
                    .await?;
                return Err(BotError::logic(err_message));
            }

            let dirs: Vec<DownloadDirectory> = get_directories(pool, user).await?;
            if dirs.len() == 0 {
                let keyboard = InlineKeyboardMarkup::new(
                    vec![vec![InlineKeyboardButton::callback(
                        directories_commands::ADD_DIRECTORY,
                        directories_commands::ADD_DIRECTORY,
                    )]]
                );
                let err_message = "No Directories found! Please add one first!".to_owned();
                bot.send_message(message.chat.id, &err_message).reply_markup(keyboard)
                    .await?;
                return Err(BotError::logic(err_message));
            }

            let magnet_id = register_magnet(pool, user, &link.clone().full_link()).await?;
            let mut keys = dirs.iter().map(|dir|
                vec![InlineKeyboardButton::callback(
                    &dir.alias,
                    format!("download:{}:{}", &magnet_id, &dir.ordinal),
                )]
            ).collect::<Vec<Vec<InlineKeyboardButton>>>();
            keys.append(&mut vec![vec![InlineKeyboardButton::callback(
                "-- Cancel --",
                "cancel",
            )]]);
            let keyboard = InlineKeyboardMarkup::new(keys);

            bot.send_message(message.chat.id, format!("{}\nChoose directory to download", link.dn()))
                .reply_markup(keyboard)
                .await?;
        }
        None => {
            let err_message = format!("Couldn't parse magnet from text: {}", link);
            error!("{}", &err_message);
            bot.send_message(message.chat.id, "Sorry. Couldn't handle this magnet. Try later :(")
                .await?;
            return Err(BotError::logic(err_message));
        }
    };
    Ok(())
}
