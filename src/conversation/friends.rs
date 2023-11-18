use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::conversation::commands::settings_commands::HIDE_MESSAGE;

use crate::db::repository::{delete_friend, get_friends, get_user, Pool};
use crate::errors::BotError;

pub async fn list_friends(
    bot: &Bot,
    pool: &Pool,
    user_id: &u64,
    chat_id: &ChatId,
) -> Result<(), BotError> {
    let friends = get_friends(pool, user_id).await.unwrap();
    if friends.is_empty() {
        bot.send_message(*chat_id, "You don't have any friends now 😢. Try adding one with /add_friend command").await?;
    } else {
        let buttons = friends.chunks(2).into_iter()
            .map(|chunk|
                chunk.into_iter().map(|u|
                    {
                        let username = u.username.clone().unwrap();
                        InlineKeyboardButton::callback(&username, format!("manage_friend:{}", u.id))
                    }
                ).collect::<Vec<InlineKeyboardButton>>()
            ).collect::<Vec<Vec<InlineKeyboardButton>>>();
        let kb = InlineKeyboardMarkup::new(buttons);
        bot.send_message(*chat_id, "This is the list of your friends:")
            .reply_markup(kb).await?;
    }
    Ok(())
}

pub async fn manage_friend_callback(bot: &Bot, pool: &Pool, data: &str, message: &Message) -> Result<(), BotError> {
    let friend_id = data.split(":").last().unwrap().parse::<i64>().unwrap();
    let user = get_user(pool, &friend_id).await?;
    match user {
        Some(u) => {
            let kb = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("Unfriend", format!("unfriend:{}", u.id))],
                vec![InlineKeyboardButton::callback(HIDE_MESSAGE, HIDE_MESSAGE)]
            ]);
            bot.send_message(message.chat.id, format!("This is your friend {}", u.username.unwrap()))
                .reply_markup(kb)
                .await?;
        }
        None => {
            bot.send_message(message.chat.id, "I don't know this person anymore").await?;
        }
    };
    Ok(())
}

pub async fn unfriend_callback(bot: &Bot, pool: &Pool, data: &str, message: &Message) -> Result<(), BotError> {
    let friend_id = data.split(":").last().unwrap().parse::<i64>().unwrap();
    let friend =  get_user(pool, &friend_id).await?;
    if friend.is_none() {
        bot.send_message(message.chat.id, "I don't know this person anymore").await?;
    }
    let confirm_callback = format!("confirm_{}", data);
    let mut rng = StdRng::from_entropy();
    let mut buttons = vec![
            vec![InlineKeyboardButton::callback("Yes", confirm_callback)],
            vec![InlineKeyboardButton::callback("No", "-")],
            vec![InlineKeyboardButton::callback("Also No", "-")]
    ];
    buttons.shuffle(&mut rng);
    let kb = InlineKeyboardMarkup::new(buttons);
    bot.send_message(message.chat.id, format!("Are you sure you want to unfriend {}", friend.unwrap().username.unwrap()))
        .reply_markup(kb)
        .await?;
    Ok(())
}

pub async fn confirm_unfriend_callback(bot: &Bot, pool: &Pool, user_id: &u64, data: &str, message: &Message) -> Result<(), BotError> {
    let friend_id = data.split(":").last().unwrap().parse::<i64>().unwrap();
    let i_user_id = *user_id as i64;
    let user = match get_user(pool, &i_user_id).await? {
        Some(it) => it,
        None => return Ok(()),
    };
    let friend = match get_user(pool, &friend_id).await? {
        Some(it) => it,
        None => {
            bot.send_message(message.chat.id, "I don't know this person anymore").await?;
            return Ok(());
        }
    };
    delete_friend(pool, &i_user_id, &friend_id).await?;
    delete_friend(pool, &friend_id, &i_user_id).await?;
    bot.send_message(message.chat.id, format!("You and {} are no longer friends", friend.username.clone().unwrap())).await?;
    bot.send_message(friend, format!("You and {} are no longer friends", user.username.unwrap())).await?;
    Ok(())
}
