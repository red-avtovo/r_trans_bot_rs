use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use std::env;

#[derive(Clone, Debug)]
pub struct DbConfig;

impl DbConfig {
    pub fn getPool() -> Pool<ConnectionManager<PgConnection>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().max_size(15)
            .build(manager)
            .expect("Failed to create pool.")
    }
}
