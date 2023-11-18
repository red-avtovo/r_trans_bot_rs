use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

use crate::errors::DbError;
use crate::core::crypto::{random_salt, Crypto};
use crate::schema::{dirs, magnets, servers, tasks, users, friends};
use diesel::prelude::*;
use uuid::Uuid;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

use super::models::{
    directories::{DownloadDirectory, NewDownloadDirectory},
    download_task::{DownloadTask, NewDownloadTask, TaskStatus},
    magnet::{Magnet, NewMagnet},
    server::{Authentication, NewServer, Server},
    user::{NewUser, User},
    friends::NewFriend,
};
use log::*;

pub async fn save_user(pool: &Pool, user: NewUser) -> Result<User, DbError> {
    let mut connection = pool.get()?;
    diesel::insert_into(users::table)
        .values(&user)
        .execute(&mut connection)?;
    get_user(pool, &user.id).await.map(|u| u.unwrap())
}

pub async fn get_user(pool: &Pool, id: &i64) -> Result<Option<User>, DbError> {
    let mut connection = pool.get()?;
    users::table
        .filter(users::id.eq(id))
        .first::<User>(&mut connection)
        .optional()
        .map_err(|e| e.into())
}

// DIRECTORIES

pub async fn add_directory(
    pool: &Pool,
    user: &User,
    alias: &String,
    path: &String,
) -> Result<DownloadDirectory, DbError> {
    let mut connection = pool.get()?;
    let next_ordinal = get_directory_next_ordinal(pool, user).await?;
    let new_dir =
        NewDownloadDirectory::new(user.id as i64, alias.to_owned(), path.to_owned(), next_ordinal);
    let dir_id = diesel::insert_into(dirs::table)
        .values(&new_dir)
        .returning(dirs::id)
        .get_result(&mut connection)?;

    get_directory_by_id(pool, &dir_id).await
}

async fn get_directory_by_id(pool: &Pool, id: &Uuid) -> Result<DownloadDirectory, DbError> {
    let mut connection = pool.get()?;
    dirs::table
        .filter(dirs::id.eq(id))
        .first::<DownloadDirectory>(&mut connection)
        .optional()?
        .ok_or("Directory not found!".to_owned().into())
}

pub(crate) async fn get_directory_next_ordinal(pool: &Pool, user: &User) -> Result<i32, DbError> {
    let mut connection = pool.get()?;
    let last_ordinal: i32 = dirs::table
        .filter(dirs::user_id.eq(&(user.id as i64)))
        .select(diesel::dsl::max(dirs::ordinal))
        .first::<Option<i32>>(&mut connection)?
        .unwrap_or(0);
    Ok(last_ordinal + 1)
}

pub async fn get_directory(
    pool: &Pool,
    user: &User,
    ordinal: i32,
) -> Result<Option<DownloadDirectory>, DbError> {
    let mut connection = pool.get()?;
    Ok(dirs::table
        .filter(dirs::user_id.eq(&(user.id as i64)).and(dirs::ordinal.eq(ordinal)))
        .first::<DownloadDirectory>(&mut connection)
        .optional()?)
}

pub async fn get_directories(pool: &Pool, user: &User) -> Result<Vec<DownloadDirectory>, DbError> {
    let mut connection = pool.get()?;
    dirs::table
        .filter(dirs::user_id.eq(&(user.id as i64)))
        .load::<DownloadDirectory>(&mut connection)
        .map_err(|e| e.into())
}

pub async fn delete_directories(pool: &Pool, user: User) -> Result<(), DbError> {
    let mut connection = pool.get()?;
    diesel::delete(dirs::table.filter(dirs::user_id.eq(&(user.id as i64)))).execute(&mut connection)?;
    Ok(())
}

#[allow(dead_code)]
pub async fn delete_directory(pool: &Pool, user: User, ordinal: i32) -> Result<(), DbError> {
    let mut connection = pool.get()?;
    diesel::delete(dirs::table.filter(dirs::user_id.eq(&(user.id as i64)).and(dirs::ordinal.eq(ordinal))))
        .execute(&mut connection)?;
    Ok(())
}

// TASKS

pub async fn add_task(
    pool: &Pool,
    user: &User,
    server_id: &Uuid,
    magnet: &Magnet,
) -> Result<DownloadTask, DbError> {
    let mut connection = pool.get()?;
    let new_task = NewDownloadTask::new(
        user.id as i64,
        server_id.clone(),
        magnet.id,
        TaskStatus::Created.to_string(),
        None,
    );
    let new_id = diesel::insert_into(tasks::table)
        .values(new_task)
        .returning(tasks::id)
        .get_result(&mut connection)?;
    get_task_by_id(pool, &new_id).await.map(|it| it.unwrap())
}

pub(crate) async fn get_task_by_id(
    pool: &Pool,
    id: &Uuid,
) -> Result<Option<DownloadTask>, DbError> {
    let mut connection = pool.get()?;
    Ok(tasks::table
        .filter(tasks::id.eq(id))
        .first::<DownloadTask>(&mut connection)
        .optional()?)
}

#[allow(dead_code)]
pub(crate) async fn get_tasks_by_server_id(
    pool: &Pool,
    id: &Uuid,
) -> Result<Vec<DownloadTask>, DbError> {
    let mut connection = pool.get()?;
    Ok(tasks::table
        .filter(tasks::server_id.eq(id))
        .load::<DownloadTask>(&mut connection)?)
}

pub(crate) async fn tasks_count_by_server_id(pool: &Pool, id: &Uuid) -> Result<i64, DbError> {
    let mut connection = pool.get()?;
    Ok(tasks::table
        .select(diesel::dsl::count(tasks::id))
        .filter(tasks::server_id.eq(id))
        .count()
        .first::<i64>(&mut connection)?)
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
    Crypto::new(std::env::var("SECRET").expect("SECRET is not set"), salt)
        .expect("All keys should be valid since the system sets them up")
}

pub async fn add_server(pool: &Pool, user: &User, url: &String) -> Result<Server, DbError> {
    let mut connection = pool.get()?;
    let new_server = NewServer::new(user.id as u64, url.clone(), None);

    let new_id = diesel::insert_into(servers::table)
        .values(new_server)
        .returning(servers::id)
        .get_result(&mut connection)?;

    get_server_by_id(pool, user, new_id)
        .await
        .map(|s| s.unwrap())
}

pub async fn add_server_auth(
    pool: &Pool,
    user: &User,
    url: &String,
    username: &String,
    password: &String,
) -> Result<Server, DbError> {
    let mut connection = pool.get()?;

    let crypto = init_crypto(&user);
    let auth = Authentication {
        username: username.clone(),
        password: crypto.encrypt(password),
    };
    let new_server = NewServer::new(user.id as u64, url.clone(), Some(auth));

    let new_id = diesel::insert_into(servers::table)
        .values(new_server)
        .returning(servers::id)
        .get_result(&mut connection)?;

    let server = get_server_by_id(pool, user, new_id).await?.unwrap();
    Ok(server)
}

#[allow(dead_code)]
pub async fn delete_server(pool: &Pool, user: &User, id: &Uuid) -> Result<(), DbError> {
    let mut connection = pool.get()?;
    diesel::delete(servers::table.filter(servers::user_id.eq(user.id as i64).and(servers::id.eq(id))))
        .execute(&mut connection)?;
    Ok(())
}

#[allow(dead_code)]
pub async fn delete_servers(pool: &Pool, user: &User) -> Result<(), DbError> {
    let mut connection = pool.get()?;
    diesel::delete(servers::table.filter(servers::user_id.eq(user.id as i64))).execute(&mut connection)?;
    Ok(())
}

impl Server {
    pub(crate) fn decrypt(self: &Self, crypto: &Crypto) -> Server {
        let clone = self.clone();
        Server {
            password: clone.password.map(|val| crypto.decrypt(&val)),
            ..clone
        }
    }
}

#[allow(dead_code)]
pub(crate) async fn get_servers_by_user_id(
    pool: &Pool,
    user: &User,
) -> Result<Vec<Server>, DbError> {
    let mut connection = pool.get()?;
    let crypto = init_crypto(&user);

    let encrypted_servers = servers::table
        .filter(servers::user_id.eq(user.id as i64))
        .load::<Server>(&mut connection)?;
    let decrypted_servers = encrypted_servers
        .iter()
        .map(|server| server.decrypt(&crypto))
        .collect();
    Ok(decrypted_servers)
}

pub(crate) async fn get_server_by_id(
    pool: &Pool,
    user: &User,
    id: Uuid,
) -> Result<Option<Server>, DbError> {
    let mut connection = pool.get()?;
    let crypto = init_crypto(&user);

    Ok(servers::table
        .filter(servers::user_id.eq(user.id as i64).and(servers::id.eq(id)))
        .first::<Server>(&mut connection)
        .optional()?
        .map(|v| v.decrypt(&crypto)))
}

// MAGNETS

pub(crate) async fn register_magnet(
    pool: &Pool,
    user: &User,
    url: &String,
) -> Result<Uuid, DbError> {
    let mut connection = pool.get()?;
    let new_magnet = NewMagnet::new(user.id as i64, url.clone());

    let new_id = diesel::insert_into(magnets::table)
        .values(new_magnet)
        .returning(magnets::id)
        .get_result(&mut connection)?;

    Ok(new_id)
}

pub(crate) async fn get_magnet_by_id(
    pool: &Pool,
    user: &User,
    id: Uuid,
) -> Result<Option<Magnet>, DbError> {
    let mut connection = pool.get()?;
    let magnet = magnets::table
        .filter(magnets::user_id.eq(user.id as i64).and(magnets::id.eq(id)))
        .first::<Magnet>(&mut connection)
        .optional();

    magnet.map_err(|e| e.into())
}

// FRIENDS

pub(crate) async fn get_friends(
    pool: &Pool,
    user_id: &u64
) -> Result<Vec<User>, DbError> {
    let mut connection = pool.get()?;
    let user = *user_id as i64;
    let friends = friends::table
        .filter(friends::user_id.eq(&user))
        .inner_join(users::table.on(users::id.eq(friends::friend_user_id)))
        .select(users::all_columns)
        .load::<User>(&mut connection)?;
    Ok(friends)
}

pub(crate) async fn find_friend(
    pool: &Pool,
    user_id: &i64,
    friend_id: &i64,
) -> Result<Option<User>, DbError> {
    let mut connection = pool.get()?;
    let friends = friends::table
        .filter(friends::user_id.eq(&user_id)
            .and(friends::friend_user_id.eq(friend_id)))
        .inner_join(users::table.on(users::id.eq(&user_id)))
        .select(users::all_columns)
        .first::<User>(&mut connection)
        .optional()?;
    Ok(friends)
}

pub(crate) async fn add_friend(
    pool: &Pool,
    user_id: &i64,
    friend_id: &i64,
) -> Result<Uuid, DbError> {
    let mut connection = pool.get()?;
    let new_friend = NewFriend{user_id: user_id.clone(), friend_user_id: friend_id.clone()};
    let new_registered_friend = diesel::
        insert_into(friends::table)
        .values(new_friend)
        .returning(friends::id)
        .get_result(&mut connection)?;
    Ok(new_registered_friend)
}

pub(crate) async fn delete_friend(
    pool: &Pool,
    user_id: &i64,
    friend_id: &i64,
) -> Result<(), DbError> {
    let mut connection = pool.get()?;
    diesel::delete(friends::table.filter(friends::user_id.eq(&user_id).and(friends::friend_user_id.eq(&friend_id))))
        .execute(&mut connection)?;
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::DbConfig;
    use rand::Rng;

    fn pool() -> Pool {
        dotenvy::from_filename("test.env").ok();
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
            salt: random_salt(),
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

        assert_eq!(&server.user_id, &(user.id));
        assert_eq!(&server.url, &"Some url".to_owned());

        Ok(())
    }
}
