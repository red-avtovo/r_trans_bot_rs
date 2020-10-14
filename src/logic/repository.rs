use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

use super::models::*;
use crate::schema::{
    users,
    dirs,
    tasks,
    servers,
    magnets,
};
use diesel::prelude::*;
use crate::errors::DbError;
use super::crypto::{
    Crypto,
    random_salt,
};
use uuid::Uuid;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

use log::*;

pub async fn save_user(pool: &Pool, user: NewUser) -> Result<User, DbError> {
    let connection = pool.get()?;
    diesel::insert_into(users::table)
    .values(&user)
    .execute(&connection)?;
    get_user(pool, &user.id).await
    .map(|u| u.unwrap())
}

pub async fn get_user(pool: &Pool, id: &i64) -> Result<Option<User>, DbError> {
    let connection = pool.get()?;
    users::table
    .filter(users::id.eq(id))
    .first::<User>(&connection)
    .optional()
    .map_err(|e| e.into())
}

// DIRECTORIES

pub async fn add_directory(pool: &Pool, user: &User, alias: &String, path: &String) -> Result<DownloadDirectory, DbError> {
    let connection = pool.get()?;
    let next_ordinal = get_directory_next_ordinal(pool, user).await?;
    let new_dir = NewDownloadDirectory::new(
        user.id,
        alias.to_owned(),
        path.to_owned(),
        next_ordinal
    );
    let dir_id = diesel::insert_into(dirs::table)
    .values(&new_dir)
    .returning(dirs::id)
    .get_result(&connection)?;

    get_directory_by_id(pool, &dir_id).await
}

async fn get_directory_by_id(pool: &Pool, id: &Uuid) -> Result<DownloadDirectory, DbError> {
    let connection = pool.get()?;
    dirs::table
    .filter(dirs::id.eq(id))
    .first::<DownloadDirectory>(&connection)
    .optional()?
    .ok_or("Directory not found!".to_owned().into())
}

pub(crate) async fn get_directory_next_ordinal(pool: &Pool, user: &User) -> Result<i32, DbError> {
    let connection = pool.get()?;
    let v: Option<i32> = dirs::table.filter(dirs::user_id.eq(&user.id))
    .select(diesel::dsl::max(dirs::ordinal))
    .first(&connection)?;
    Ok(v.unwrap_or(0))
}

pub async fn get_directory(pool: &Pool, user: &User, ordinal: i32) -> Result<Option<DownloadDirectory>, DbError> {
    let connection = pool.get()?;
    Ok(dirs::table.filter(dirs::user_id.eq(&user.id).and(dirs::ordinal.eq(ordinal)))
        .first::<DownloadDirectory>(&connection)
        .optional()?)
}

pub async fn get_directories(pool: &Pool, user: &User) -> Result<Vec<DownloadDirectory>, DbError> {
    let connection = pool.get()?;
    dirs::table.filter(dirs::user_id.eq(&user.id))
        .load::<DownloadDirectory>(&connection)
        .map_err(|e| e.into())
}

pub async fn delete_directories(pool: &Pool, user: User) -> Result<(), DbError> {
    let connection = pool.get()?;
    diesel::delete(dirs::table.filter(dirs::user_id.eq(&user.id)))
        .execute(&connection)?;
    Ok(())
}

#[allow(dead_code)]
pub async fn delete_directory(pool: &Pool, user: User, ordinal: i32) -> Result<(), DbError> {
    let connection = pool.get()?;
    diesel::delete(dirs::table.filter(dirs::user_id.eq(&user.id).and(dirs::ordinal.eq(ordinal))))
        .execute(&connection)?;
    Ok(())
}

// TASKS

pub async fn add_task(pool: &Pool, user: &User, server_id: &Uuid, magnet: &Magnet) -> Result<DownloadTask, DbError> {
    let connection = pool.get()?;
    let new_task = NewDownloadTask::new(
        user.id,
        server_id.clone(),
        magnet.id,
        TaskStatus::Created.to_string(),
        None
    );
    let new_id = diesel::insert_into(tasks::table)
    .values(new_task)
    .returning(tasks::id)
    .get_result(&connection)?;
    get_task_by_id(pool, &new_id).await.map(|it| it.unwrap())
}

pub(crate) async fn get_task_by_id(pool: &Pool, id: &Uuid) -> Result<Option<DownloadTask>, DbError> {
    let connection = pool.get()?;
    Ok(tasks::table.filter(tasks::id.eq(id))
        .first::<DownloadTask>(&connection)
        .optional()?)
}

pub(crate) async fn get_tasks_by_server_id(pool: &Pool, id: &Uuid) -> Result<Vec<DownloadTask>, DbError> {
    let connection = pool.get()?;
    Ok(tasks::table.filter(tasks::id.eq(id))
        .load::<DownloadTask>(&connection)?)
}

pub(crate) async fn tasks_count_by_server_id(pool: &Pool, id: &Uuid) -> Result<i64, DbError> {
    let connection = pool.get()?;
    Ok(tasks::table
        .select(diesel::dsl::count(tasks::id))
        .filter(tasks::id.eq(id))
        .count()
        .first::<i64>(&connection)?)
}

// SERVERS

pub(crate) fn test_db_crypto() {
    debug!("Testing db crypto");
    let salt = random_salt();
    init_salty_crypto(salt);
    debug!("Test passed!");
}

fn init_crypto(user: &User) -> Crypto {
    init_salty_crypto(user.salt.clone())
}

fn init_salty_crypto(salt: String) -> Crypto {
    Crypto::new(
        std::env::var("SECRET").expect("SECRET is not set"),
        salt
    )
    .expect("All keys should be valid since the system sets them up")
}

pub async fn add_server(pool: &Pool, user: &User, url: &String) -> Result<Server, DbError> {
    let connection = pool.get()?;
    let new_server = NewServer::new(user.id, url.clone(), None);

    let new_id = diesel::insert_into(servers::table)
    .values(new_server)
    .returning(servers::id)
    .get_result(&connection)?;

    get_server_by_id(pool, user, new_id).await.map(|s| s.unwrap())
}

pub async fn add_server_auth(pool: &Pool, user: &User, url: &String, username: &String, password: &String) -> Result<Server, DbError> {
    let connection = pool.get()?;

    let crypto = init_crypto(&user);
    let auth = Authentication {
        username: username.clone(),
        password: crypto.encrypt(password),
    };
    let new_server = NewServer::new(user.id, url.clone(), Some(auth));

    let new_id = diesel::insert_into(servers::table)
    .values(new_server)
    .returning(servers::id)
    .get_result(&connection)?;

    let server = get_server_by_id(pool, user, new_id).await?.unwrap();
    Ok(server)
}

#[allow(dead_code)]
pub async fn delete_server(pool: &Pool, user: &User, id: &Uuid) -> Result<(), DbError>{
    let connection = pool.get()?;
    diesel::delete(servers::table.filter(servers::user_id.eq(user.id).and(servers::id.eq(id))))
        .execute(&connection)?;
    Ok(())
}

#[allow(dead_code)]
pub async fn delete_servers(pool: &Pool, user: &User) -> Result<(), DbError>{
    let connection = pool.get()?;
    diesel::delete(servers::table.filter(servers::user_id.eq(user.id)))
        .execute(&connection)?;
    Ok(())
}

impl Server {
    pub(crate) fn decrypt(self: &Self,  crypto: &Crypto) -> Server {
        let clone = self.clone();
        Server {
            password: clone.password.map(|val| crypto.decrypt(&val)),
            ..clone
        }
    }
}

#[allow(dead_code)]
pub(crate) async fn get_servers_by_user_id(pool: &Pool, user: &User) -> Result<Vec<Server>, DbError> {
    let connection = pool.get()?;
    let crypto = init_crypto(&user);

    let encrypted_servers = servers::table.filter(servers::user_id.eq(user.id))
    .load::<Server>(&connection)?;
    let decrypted_servers = encrypted_servers.iter()
        .map(|server|  server.decrypt(&crypto))
        .collect();
    Ok(decrypted_servers)
}

pub(crate) async fn get_server_by_id(pool: &Pool, user: &User, id: Uuid) -> Result<Option<Server>, DbError> {
    let connection = pool.get()?;
    let crypto = init_crypto(&user);

    Ok(servers::table.filter(servers::user_id.eq(user.id).and(servers::id.eq(id)))
        .first::<Server>(&connection).optional()?
        .map(|v| v.decrypt(&crypto)))
}

// MAGNETS

pub(crate) async fn register_magnet(pool: &Pool, user: &User, url: &String) -> Result<Uuid, DbError> {
    let connection = pool.get()?;
    let new_magnet = NewMagnet::new(user.id, url.clone());

    let new_id = diesel::insert_into(magnets::table)
    .values(new_magnet)
    .returning(magnets::id)
    .get_result(&connection)?;

    Ok(new_id)
}

pub(crate) async fn get_magnet_by_id(pool: &Pool, user: &User, id: Uuid) -> Result<Option<Magnet>, DbError> {
    let connection = pool.get()?;
    let magnet = magnets::table.filter(magnets::user_id.eq(user.id).and(magnets::id.eq(id)))
    .first::<Magnet>(&connection).optional();

    magnet.map_err(|e| e.into())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::DbConfig;
    use rand::Rng;

    fn pool() -> Pool {
        dotenv::from_filename("test.env").ok();
        DbConfig::get_pool()
    }

    fn new_user() -> NewUser {
        let mut rng = rand::thread_rng();
        NewUser {
            id: rng.gen(),
            chat: rng.gen(),
            first_name: String::from("A"),
            last_name: None,
            username: None,
            salt: random_salt()
        }
    }

    impl NewUser {
        async fn save(self: Self, pool: &Pool) -> Result<User, DbError> {
            save_user(pool, self).await
        }
    }

    #[tokio::test]
    pub async fn test_user_get() -> Result<(), DbError> {
        let pool = pool();
        let user = new_user();
        let res = save_user(&pool, user.clone()).await?;
        assert_eq!(&user.first_name, &res.first_name);
        assert_eq!(&user.chat, &res.chat);
        assert_eq!(&user.username, &res.username);
        Ok(())
    }

    #[tokio::test]
    pub async fn test_server_get() -> Result<(), DbError> {
        let pool = pool();
        let user = new_user().save(&pool).await?;
        let server = add_server(&pool, &user, &"Some url".to_owned()).await?;

        assert_eq!(&server.user_id, &user.id);
        assert_eq!(&server.url, &"Some url".to_owned());

        Ok(())
    }
}