use std::env;
use log::{debug, info, warn};
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{ForwardedFrom, Me, True};
use crate::conversation::tasks::process_magnet;
use crate::core::{
    flaresolver::Flaresolver,
    rutracker::find_magnet
};
use crate::db::repository::{add_friend, find_friend, get_user, Pool};
use crate::router::HandlerResult;
use futures::stream::StreamExt; // for .next()

pub async fn process_message(
    bot: Bot,
    pool: Pool,
    message: Message,
) -> HandlerResult {
    let document = message.document().map(ToOwned::to_owned);
    if let Some(document) = document {
        match document.file_name {
            Some(s) if s.ends_with(".torrent") => {
                bot.send_message(message.chat.id, format!("You've sent {} file and I will support it soon", s)).await?;
                let file = bot.get_file(document.file.id).await?;
                let data = bot.download_file_stream(&file.path).next().await.unwrap()?;
                // let content = String::from_utf8(data.to_vec())?;
                bot.send_message(message.chat.id, format!("File  of {} bytes received", data.len())).await?;
            }
            Some(s) => {
                bot.send_message(message.chat.id, format!("You've sent {} file, but I don't support it", s)).await?;
            }
            None => {}
        }
        return Ok(());
    };

    match message.text().map(ToOwned::to_owned) {
        Some(s) if s.contains("magnet:") => try_to_process_magnet(&bot, &pool, &message, &s).await?,
        Some(s) if s.starts_with("https://rutracker.org/forum/viewtopic.php?t=") => {
            try_to_process_rutracker_link(&bot, &pool, &message, &s).await?
        }
        _ => {
            bot.send_message(message.chat.id, "I don't know what you mean").await?;
        }
    };

    Ok(())
}

async fn try_to_process_rutracker_link(
    bot: &Bot,
    pool: &Pool,
    message: &Message,
    data: &String,
) -> HandlerResult {
    let url = data.to_lowercase();
    let solver_url = match env::var("FLARESOLVER_URL") {
        Ok(v) => v,
        Err(_) => {
            bot.send_message(
                message.chat.id,
                "FLARESOLVER_URL is not set, can't parse rutracker links",
            ).await?;
            return Ok(());
        }
    };
    let solver = Flaresolver::new(solver_url.to_string());
    info!("Fetching '{}'", &url);
    match solver.get_page_html(url)
        .await
        .map(|html|
            html.and_then(|text| find_magnet(text))
        ) {
        Ok(optional_magnet) => {
            info!("Fetched successfully");
            match optional_magnet {
                Some(magnet_link) => {
                    try_to_process_magnet(&bot, pool, message, &magnet_link).await
                }
                _ => {
                    bot.send_message(
                        message.chat.id,
                        "Couldn't find a magnet on the page. Try to send the magnet manually",
                    ).await?;
                    Ok(())
                }
            }
        }
        _ => {
            info!("Failed to fetch");
            bot.send_message(
                message.chat.id,
                "Couldn't fetch the link. Try to send the magnet manually",
            ).await?;
            Ok(())
        }
    }
}

async fn try_to_process_magnet(
    bot: &Bot,
    pool: &Pool,
    message: &Message,
    link: &String,
) -> HandlerResult {
    debug!("Processing a magnet link: {}", link);
    match process_magnet(bot, pool, message, link).await {
        Ok(_) => {
            debug!("Processing of a magnet link passed. Deleting the original message");
            bot.delete_message(message.chat.id, message.id).await?
        }
        Err(_) => {
            warn!(
                "Processing of a magnet link failed! The link was: {}",
                &link
            );
            True
        }
    };
    Ok(())
}

pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "I don't know what you mean")
        .await?;
    Ok(())
}

pub async fn add_friend_dialogue(bot: Bot, pool: Pool, msg: Message, me: Me) -> HandlerResult {
    let forwarded_from = match msg.forward_from() {
        None => {
            bot.send_message(msg.chat.id, "Please forward me a friends message to you from any other chat with your friend").await?;
            return Ok(());
        }
        Some(it) => {
            it
        }
    };
    let forwarded_user = match forwarded_from {
        ForwardedFrom::User(it) => { it }
        _ => {
            bot.send_message(msg.chat.id, "Please forward me a friends message to you from any other PERSONAL chat with your friend").await?;
            return Ok(());
        }
    };

    let from = msg.from().unwrap();
    let from_user_username = from.username.clone().unwrap();
    if forwarded_user.id.eq(&me.id) {
        bot.send_message(msg.chat.id, format!("Aww! We are already friends, {} ❤️", from_user_username)).await?;
        return Ok(());
    }


    if forwarded_user.id.eq(&from.id) {
        bot.send_message(msg.chat.id, "Funny enough, you can't befriend yourself.").await?;
        return Ok(());
    }

    let friend_user_id = forwarded_user.id.0 as i64;
    let db_user = get_user(&pool, &friend_user_id).await?;
    if db_user.is_none() {
        bot.send_message(msg.chat.id, "I don't know this user. Please ask your friend to start a conversation with me first").await?;
        return Ok(());
    }

    let forwarded_user_username = forwarded_user.username.clone().unwrap();
    let requester_user_id = from.id.0 as i64;
    let friend_user = find_friend(&pool, &requester_user_id, &friend_user_id).await?;
    if friend_user.is_some() {
        bot.send_message(msg.chat.id, format!("You are already friends with {}", forwarded_user_username)).await?;
        return Ok(());
    }

    add_friend(&pool, &requester_user_id, &friend_user_id).await?;
    add_friend(&pool, &friend_user_id, &requester_user_id).await?;
    bot.send_message(msg.chat.id, format!("You are now friends with {} 🎉", forwarded_user_username)).await?;
    let befriended_user = db_user.unwrap();
    bot.send_message(befriended_user, format!("{} added you as a friend! 🎉\n\nNow potentially you can use their shared servers.\nYou can find those with the command /listservers", from_user_username)).await?;

    Ok(())
}