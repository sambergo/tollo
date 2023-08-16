use sqlx::{SqliteConnection, Sqlite, migrate::MigrateDatabase, Connection, pool::PoolConnection, sqlite::SqliteQueryResult};
use std::env;

use crate::app::Channel;

#[tokio::main]
pub async fn connect_database() -> Result<SqliteConnection, sqlx::Error> {
    let data_home = env::var("XDG_DATA_HOME").unwrap_or_else(|_| String::from(".local/share"));
    let db_path = format!("{}/tollo/favorites.db", data_home);
    if !Sqlite::database_exists(&db_path).await.unwrap_or(false) {
        match Sqlite::create_database(&db_path).await {
            Ok(_) => {
                let pool = SqliteConnection::connect(&db_path).await?;
                Ok(pool)
            },
            Err(error) => panic!("Error creating database: {}", error),
        }
    } else {
        let pool = SqliteConnection::connect(&db_path).await?;
        Ok(pool)
    }
}


async fn query_and_create(db: &mut SqliteConnection) {
    let create_table = sqlx::query("
        CREATE TABLE IF NOT EXISTS favorites (
            id   String PRIMARY KEY,
            name String NOT NULL,
            logo String,
            group String,
            url String,
            favorites Bool
        );
INSERT INTO favorites VALUES ('1', 'lempikananva', 'loogourli', 'ryhma', 'urrrrrrrrrrrrrrrrrrrrrrrrrrliiiiiiiiiiiiii', true)
    ").execute(db)
    .await
    .expect("Could not create table if it does not exist"); 
    // let favorites_list: Vec<Favorite> = sqlx::query_as("SELECT * FROM favorites")
    // .fetch_all(db)
    // .await
    // .expect("Could not fetch all rows from the table");
    // println!("{:?}", favorites_list);
}

async fn query_favorites(db: &mut SqliteConnection)  {
    let favorites = sqlx::query_as("SELECT * FROM favorites")
        .fetch_all(db)
        .await.expect("ei");
    println!("{:?}", favorites);
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_connect_db() {
        let db = connect_database();
        assert!(db.is_ok());
    }
    #[test]
    fn test_table_db() {
        let db = connect_database();
        let _ = query_and_create(& mut db.unwrap());
    }
}
