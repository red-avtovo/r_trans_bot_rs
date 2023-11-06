#[macro_use]
extern crate diesel;

use std::env;
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use dotenvy::dotenv;
use teloxide::{Bot, dptree};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dispatcher;

use db::db_config::DbConfig;
use db::repository::test_db_crypto;

use crate::router::{schema, State};


mod db;
mod errors;
mod core;
mod router;

mod conversation;
mod schema;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let pool = DbConfig::get_pool();

    DbConfig::test_connection(pool.clone()).unwrap();
    run_migration(&mut DbConfig::get_single_connection());

    test_db_crypto();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let bot = Bot::new(token);

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn run_migration(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

fn error_handler_fn(bot: Bot, error: Box<dyn std::error::Error>) {

}