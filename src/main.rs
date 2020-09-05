use std::env;
use futures::StreamExt;
use telegram_bot::*;
use dotenv::dotenv;
use std::collections::HashMap;
use logic::models::TelegramId;
use logic::repository::{
    test_connection,
    test_db_crypto
};

mod router;
mod logic;
mod errors;
use errors::BotError;

mod db_config;
use db_config::*;

use log::{info, error};

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();
    env_logger::init();
    let pool = DbConfig::getPool();

    match test_connection(&pool.clone()).await {
        Ok(_) => info!("Db Connection established!"),
        Err(error) => error!("Db test failed: {:#?}", error)
    };

    test_db_crypto();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);
    let mut last_command = HashMap::<TelegramId, String>::new();
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        router::route(api.clone(), &pool, update, &mut last_command).await;
    }
    Ok(())
}
