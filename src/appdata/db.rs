use sqlx::{Sqlite, migrate::MigrateDatabase};
use std::env;

#[tokio::main]
pub async fn create_database() {
    let data_home = env::var("XDG_DATA_HOME").unwrap_or_else(|_| String::from(".local/share"));
    let db_path = format!("{}/tollo/favorites.db", data_home);

    if !Sqlite::database_exists(&db_path).await.unwrap_or(false) {
        println!("Creating database {}", &db_path);
        match Sqlite::create_database(&db_path).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_init_db() {
        create_database();
    }
}


