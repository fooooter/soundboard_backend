use std::env;
use std::sync::LazyLock;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, MySqlPool};
use sqlx::pool::PoolConnection;

pub static CONN: LazyLock<Result<MySqlPool, String>> = LazyLock::new(|| {
    let Ok(conn_string) = env::var("MYSQL_CONN") else {
        return Err(String::from("\"MYSQL_CONN\" environment variable not found."));
    };

    match MySqlPoolOptions::new()
        .max_connections(50)
        .connect_lazy(&*conn_string) {
        Ok(pool) => Ok(pool),
        Err(error) => Err(error.to_string())
    }
});

pub async fn get_connection() -> Result<PoolConnection<MySql>, String> {
    match &*CONN {
        Ok(c) => {
            match c.acquire().await {
                Ok(c) => Ok(c),
                Err(e) => Err(e.to_string())
            }
        },
        Err(e) => Err(e.clone())
    }
}