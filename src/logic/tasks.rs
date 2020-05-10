use telegram_bot::*;

pub async fn start_download(api: &Api, data: &str, chat_ref: &ChatRef) -> Result<(), Error> {
    let rq = chat_ref.text(format!("Callback: {}", data));
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
    // extract magnet link
    // parse and clean the link
    // create a task
    // offer directories
    Ok(())
}