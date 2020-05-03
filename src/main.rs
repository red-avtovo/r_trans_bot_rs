use std::env;
use futures::StreamExt;
use telegram_bot::*;
use dotenv::dotenv;

mod router;
mod logic;
mod errors;
use errors::BotError;

mod db_config;
use db_config::*;


#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();
    env_logger::init();
    let db_pool = match DbConfig::create().pool().await {
        Ok(pool) => pool,
        Err(e) => panic!("builder error: {:?}", e)
    };

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        router::route(api.clone(), &db_pool, update).await?;
    }
    Ok(())
}
