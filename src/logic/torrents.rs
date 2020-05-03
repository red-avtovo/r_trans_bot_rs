use telegram_bot::*;

pub async fn start_download(api: Api, callback_query: CallbackQuery) -> Result<(), Error> {
    let data = callback_query.clone().data.unwrap();
    let rq = callback_query.from.to_chat_ref().text(format!("Callback: {}", data));
    api.send(callback_query.acknowledge()).await?;
    api.send(rq).await?;
    Ok(())
}

pub async fn process_magnet(api: Api, message: Message) -> Result<(), Error> {
    let mut markup = InlineKeyboardMarkup::new();
    markup.add_row(
        vec![
            InlineKeyboardButton::callback(
                "test".to_string(),
                "callback button".to_string()
            )
        ]
    );
    api.send(message.to_source_chat().text("I see magnet").reply_markup(ReplyMarkup::InlineKeyboardMarkup(markup)))
        .await?;
    Ok(())
}