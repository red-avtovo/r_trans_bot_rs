use bb8;
use bb8::RunError;
use bb8_postgres;
use tokio_postgres::NoTls;

pub type Pool = bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>;
pub type PError = tokio_postgres::Error;
pub type RError = RunError<PError>;

pub async fn save_user(pool: &Pool, user: DbUser) -> Result<(), RError> {
    let connection = pool.get().await?;
    let query = "INSERT INTO users(id, chat, firstName, lastName, username) VALUES($1,$2,$3,$4,$5);";
    connection.query(query, &[&user.id, &user.chat, &user.first_name, &user.last_name, &user.username]).await?;
    Ok(())
}

pub async fn get_user(pool: &Pool, id: i64) -> Result<DbUser, RError> {
    let connection = pool.get().await?;
    let query = "
    select id, chat, firstName, lastName, username
    from users
    WHERE id=$1;";
    let rows = connection.query(query, &[&id]).await?;
    let row = rows.get(0).unwrap();
    let user = DbUser {
        id: row.get(0),
        chat: row.get(1),
        first_name: row.get(2),
        last_name: row.get(3),
        username: row.get(4)
    };
    Ok(user)
}

#[derive(Clone, Debug)]
pub struct DbUser {
    pub id: i64,
    pub chat: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>
}
