
use diesel::pg::PgConnection;
use diesel::r2d2::{
    ConnectionManager,
    Pool
};
use std::env;
use super::errors::{
    DbError
};

#[derive(Clone, Debug)]
pub struct DbConfig;

impl DbConfig {
    pub fn get_pool() -> Pool<ConnectionManager<PgConnection>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().max_size(15)
            .build(manager)
            .expect("Failed to create pool.")
    }

    pub fn test_connection(pool: Pool<ConnectionManager<PgConnection>>) -> Result<(), DbError> {
        // https://dev.to/werner/practical-rust-web-development-connection-pool-46f4
        pool.get()?;
        Ok(())
    }
}
