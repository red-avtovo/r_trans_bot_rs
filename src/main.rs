use std::env;
use futures::StreamExt;
use telegram_bot::*;
use dotenv::dotenv;
use std::collections::HashMap;
use logic::repository::{
    test_db_crypto,
};

mod router;
mod logic;
mod errors;

use errors::BotError;

mod db_config;

use db_config::DbConfig;

#[macro_use]
extern crate diesel;

mod schema;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();
    env_logger::init();
    let pool = DbConfig::get_pool();

    DbConfig::test_connection(pool.clone())?;
    test_db_crypto();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);
    let mut last_command = HashMap::<i64, String>::new();
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        router::route(api.clone(), &pool, update, &mut last_command).await;
    }
    Ok(())
}
