use std::env;
use bb8_postgres::PostgresConnectionManager;
use crate::logic::db::{PError, Pool};
#[derive(Clone, Debug)]
pub struct DbConfig {
    config: String
}

impl DbConfig {
    pub fn create() -> DbConfig {
        let host = &env::var("DB_HOST").expect("DB_HOST not set");
        let user_name = &env::var("DB_USER_NAME").expect("DB_USER_NAME not set");
        let password = &env::var("DB_PASSWORD").expect("DB_PASSWORD not set");
        let port = &env::var("DB_PORT").expect("DB_PORT not set");
        let db_name = &env::var("DB_NAME").expect("DB_NAME not set");
        let config = format!(
            "host={} port={} user={} password={} dbname={}",
            host,
            port,
            user_name,
            password,
            db_name
        );
        DbConfig {
            config
        }
    }

    pub async fn pool(&self) -> Result<Pool, PError> {
        let manager = PostgresConnectionManager::new(
            self.config.parse().unwrap(),
            tokio_postgres::NoTls,
        );
        let pool = bb8::Pool::builder()
            .max_size(15)
            .build(manager).await?;
        Ok(pool)
    }
}
