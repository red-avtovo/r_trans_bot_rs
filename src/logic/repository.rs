use bb8::{
    self,
    RunError
};
use bb8_postgres;
use tokio_postgres::{
    NoTls,
    row::Row
};
use super::models::*;
use crate::errors::DbError;
use super::crypto::Crypto;
use uuid::Uuid;

pub type Pool = bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>;
pub type PError = tokio_postgres::Error;
pub type RError = RunError<PError>;

pub async fn save_user(pool: &Pool, user: DbUser) -> Result<DbUser, DbError> {
    let connection = pool.get().await?;
    let query = "INSERT INTO users(id, chat, firstName, lastName, username, salt) VALUES($1,$2,$3,$4,$5);";
    connection.execute(query, &[&user.id, &user.chat, &user.first_name, &user.last_name, &user.username, &user.salt]).await?;
    get_user(pool, &user.id).await.map(|it| it.unwrap())
}

pub async fn get_user(pool: &Pool, id: &TelegramId) -> Result<Option<DbUser>, DbError> {
    let connection = pool.get().await?;
    let query = "
    select id, chat, firstName, lastName, username, salt
    from users
    WHERE id=$1;";
    let rows = connection.query(query, &[&id]).await?;
    let row = rows.get(0);
    match row {
        Some(row) => {
            let user = DbUser {
                id: row.get(0),
                chat: row.get(1),
                first_name: row.get(2),
                last_name: row.get(3),
                username: row.get(4),
                salt: row.get(5),
            };
            Ok(Some(user))
        },
        None => Ok(None)
    }
    
}

// DIRECTORIES

pub async fn add_directory(pool: &Pool, user: &DbUser, alias: &String, path: &String) -> Result<DownloadDirectory, DbError> {
    let connection = pool.get().await?;
    let query = "INSERT INTO dirs(id, user_id, alias, path, ordinal) VALUES($1,$2,$3,$4,$5);";
    let dir_id = Uuid::new_v4();
    let next_ordinal = get_directory_next_ordinal(pool, user).await?;
    connection.execute(query, &[&dir_id.to_string(), &user.id, alias, path, &next_ordinal]).await?;
    get_directory_by_id(pool, &dir_id).await
}

async fn get_directory_by_id(pool: &Pool, id: &Uuid) -> Result<DownloadDirectory, DbError> {
    let connection = pool.get().await?;
    let query = "SELECT id, user_id, alias, path, ordinal FROM dirs WHERE id=$1;";
    let rows = connection.query(query, &[&id.to_string()]).await?;
    let row = rows.get(0).unwrap();
    Ok(DownloadDirectory::from_row(&row))
}

impl DownloadDirectory {
    fn from_row(row: &Row) -> Self {
        DownloadDirectory {
            id: uuid_safe(row.get(0)),
            user_id: row.get(1),
            alias: row.get(2),
            path: row.get(3),
            ordinal: row.get(4),
        }
    }
}

pub(crate) async fn get_directory_next_ordinal(pool: &Pool, user: &DbUser) -> Result<i32, DbError> {
    let connection = pool.get().await?;
    let query = "
    select max(ordinal)
    from dirs
    WHERE user_id=$1
    GROUP BY user_id;";
    let rows = connection.query(query, &[&user.id]).await?;
    match rows.get(0) {
        Some(row) => {
            let max: i32 = row.get(0);
            Ok(max + 1)
        },
        None => Ok(1)
    }
}

pub async fn get_directory(pool: &Pool, user: DbUser, ordinal: i32) -> Result<Option<DownloadDirectory>, DbError> {
    let connection = pool.get().await?;
    let query = "SELECT id, user_id, alias, path, ordinal FROM dirs WHERE user_id=$1 AND ordinal=$2;";
    let rows = connection.query(query, &[&user.id, &ordinal]).await?;
    match rows.get(0) {
        Some(row) => Ok(Some(DownloadDirectory::from_row(&row))),
        None => Ok(None)
    }
}

pub async fn get_directories(pool: &Pool, user: DbUser) -> Result<Vec<DownloadDirectory>, DbError> {
    let connection = pool.get().await?;
    let query = "SELECT id, user_id, alias, path, ordinal FROM dirs WHERE user_id=$1;";
    let rows:Vec<Row> = connection.query(query, &[&user.id]).await?;
    Ok(rows.iter().map(DownloadDirectory::from_row).collect())
}

pub async fn delete_directories(pool: &Pool, user: DbUser) -> Result<(), DbError> {
    let connection = pool.get().await?;
    let query = "DELETE dirs WHERE user_id=$1;";
    connection.execute(query, &[&user.id]).await?;
    Ok(())
}

pub async fn delete_directory(pool: &Pool, user: DbUser, ordinal: i32) -> Result<(), DbError> {
    let connection = pool.get().await?;
    let query = "DELETE dirs WHERE user_id=$1 AND ordinal=$2;";
    connection.execute(query, &[&user.id, &ordinal]).await?;
    Ok(())
}

// TASKS

pub async fn add_task(pool: &Pool, user: DbUser, server_id: &Uuid, magnet: ShortMagnet, directory: &Uuid) -> Result<DownloadTask, DbError> {
    let connection = pool.get().await?;
    let query = "INSERT INTO tasks(id, user_id, server_id, magnet_id, directory, status) VALUES($1,$2,$3,$4,$5,$6);";
    let task_id = Uuid::new_v4();
    connection.query(query, &[&task_id.to_string(), &user.id, &server_id.to_string(), &String::from(magnet), &directory.to_string(), &TaskStatus::Created]).await?;
    get_task_by_id(pool, &task_id).await.map(|it| it.unwrap())
}

pub(crate) async fn get_task_by_id(pool: &Pool, id: &Uuid) -> Result<Option<DownloadTask>, DbError> {
    let connection = pool.get().await?;
    let query = "SELECT id, user_id, server_id, magnet, directory, status,description FROM tasks WHERE id=$1;";
    let rows = connection.query(query, &[&id.to_string()]).await?;
    match rows.get(0) {
        Some(row) => Ok(Some(DownloadTask::from_row(&row))),
        None => Ok(None)
    }
}

pub(crate) async fn get_task_by_server_id(pool: &Pool, id: &Uuid) -> Result<Vec<DownloadTask>, DbError> {
    let connection = pool.get().await?;
    let query = "SELECT id, user_id, server_id, magnet, directory, status,description FROM tasks WHERE server_id=$1;";
    let rows:Vec<Row> = connection.query(query, &[&id.to_string()]).await?;
    Ok(rows.iter().map(DownloadTask::from_row).collect())

}

impl DownloadTask {
    fn from_row(row: &Row) -> Self {
        DownloadTask {
            id: uuid_safe(row.get(0)),
            user_id: row.get(1),
            server_id: uuid_safe(row.get(2)),
            magnet_id: uuid_safe(row.get(3)),
            status: row.get(4),
            description: row.get(5),
        }
    }
}

pub async fn update_task_status(pool: &Pool, user: DbUser, id: &Uuid, status: TaskStatus) -> Result<Option<DownloadTask>, DbError> {
    let connection = pool.get().await?;
    let query = "UPDATE tasks SET status=$1 WHERE id=$2 AND user_id=$3;";
    connection.execute(query, &[&status, &id.to_string(), &user.id]).await?;
    get_task_by_id(pool, id).await
}

pub async fn update_task_status_description(pool: &Pool, user: DbUser, id: &Uuid, status: TaskStatus, description: &String) -> Result<Option<DownloadTask>, DbError> {
    let connection = pool.get().await?;
    let query = "UPDATE tasks SET status=$1, description=$2 WHERE id=$3 AND user_id=$4;";
    connection.execute(query, &[&status, &description, &id.to_string(), &user.id]).await?;
    get_task_by_id(pool, id).await
}

// SERVERS

fn init_crypto(user: &DbUser) -> Crypto{
    Crypto::new(
        std::env::var("SECRET").expect("SECRET is not set"), 
        user.salt.clone()
    )
    .expect("All keys should be valid since the system sets them up")
}

impl Server {
    fn from_row(user: &DbUser, row: &Row) -> Self {
        let crypto = init_crypto(user);
        let username: String = row.get(3);
        let enc_password: String = row.get(4);
        let auth = if username.is_empty() || enc_password.is_empty() { 
            None 
        } else { 
            let password = crypto.decrypt(&enc_password);
            Some(Authentication{ username, password})
        };
        Server {
            id: uuid_safe(row.get(0)),
            user_id: row.get(1),
            url: row.get(2),
            auth
        }
    }
}

pub async fn add_server(pool: &Pool, user: &DbUser, url: &String) -> Result<Server, DbError> {
    let connection = pool.get().await?;
    let query = "INSERT INTO servers(id, user_id, url) VALUES($1,$2,$3);";
    let server_id = Uuid::new_v4();
    connection.execute(query, &[&server_id.to_string(), &user.id, url]).await?;
    get_server_by_id(pool, user, server_id).await
}

pub async fn add_server_auth(pool: &Pool, user: &DbUser, url: &String, username: &String, password: &String) -> Result<Server, DbError> {
    let connection = pool.get().await?;
    let query = "INSERT INTO servers(id, user_id, url, username, password) VALUES($1,$2,$3,$4,$5);";
    let server_id = Uuid::new_v4();
    let crypto = init_crypto(&user);
    let enc_password = crypto.encrypt(password);
    connection.execute(query, &[&server_id.to_string(), &user.id, url, username, &enc_password]).await?;
    get_server_by_id(pool, user, server_id).await
}

pub async fn delete_server(pool: &Pool, user: &DbUser, id: &Uuid) -> Result<(), DbError>{
    let connection = pool.get().await?;
    let query = "DELETE servers WHERE user_id=$1 AND id=$2;";
    connection.execute(query, &[&user.id, &id.to_string()]).await?;
    Ok(())
}

pub async fn delete_servers(pool: &Pool, user: &DbUser) -> Result<(), DbError>{
    let connection = pool.get().await?;
    let query = "DELETE servers WHERE user_id=$1;";
    connection.execute(query, &[&user.id]).await?;
    Ok(())
}

pub(crate) async fn get_servers_by_user_id(pool: &Pool, user: &DbUser) -> Result<Vec<Server>, DbError> {
    let connection = pool.get().await?;
    let query = "SELECT id, user_id, url, username, password FROM servers WHERE user_id=$1;";
    let rows: Vec<Row> = connection.query(query, &[&user.id]).await?;
    Ok(rows.iter().map(|row| Server::from_row(user, &row)).collect())
}

pub(crate) async fn get_server_by_id(pool: &Pool, user: &DbUser, id: Uuid) -> Result<Server, DbError> {
    let connection = pool.get().await?;
    let query = "SELECT id, user_id, url, username, password FROM servers WHERE id=$1;";
    let rows = connection.query(query, &[&id.to_string()]).await?;
    let row = rows.get(0).unwrap();
    Ok(Server::from_row(user, &row))
}

// MAGNETS

pub(crate) async fn register_magnet(pool: &Pool, user: &DbUser, url: &String) -> Result<Uuid, DbError> {
    let connection = pool.get().await?;
    let query = "INSERT INTO magnets(id, user_id, url) VALUES($1,$2,$3);";
    let id = Uuid::new_v4();
    connection.execute(query, &[&id.to_string(), &user.id, url]).await?;
    Ok(id)
}

pub(crate) async fn get_magnet_by_id(pool: &Pool, user: &DbUser, id: Uuid) -> Result<Option<Magnet>, DbError> {
    let connection = pool.get().await?;
    let query = "SELECT id, user_id, url FROM magnets WHERE id=$1;";
    let rows = connection.query(query, &[&id.to_string()]).await?;
    match rows.get(0) {
        Some(row) => Ok(Some(Magnet{
            id: uuid_safe(row.get(0)),
            user_id: row.get(1),
            url: row.get(2),
        })),
        None => Ok(None)
    }
}

fn uuid_safe(str: &str) -> Uuid {
    Uuid::parse_str(str).expect("Uuid was generated by the system but failed to parse")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test() {
           
    }
}