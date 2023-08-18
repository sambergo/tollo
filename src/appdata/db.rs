extern crate rusqlite;
use std::env;

use crate::app::Channel;
use rusqlite::{params, Connection, Result};

#[allow(dead_code)]
pub fn connect_db() -> Result<Connection, rusqlite::Error> {
    let data_home = env::var("XDG_DATA_HOME").unwrap_or_else(|_| String::from(".local/share"));
    let db_path = format!("{}/tollo/favorites.db", data_home);
    let db = Connection::open(db_path)?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS favorites (
            name TEXT,
            id   TEXT,
            logo TEXT,
            favorite   INTEGER,
            agroup TEXT,
            url TEXT PRIMARY KEY
        )",
        (),
    )?;
    Ok(db)
}

pub fn get_favorites(
    db: &Result<Connection, rusqlite::Error>,
) -> Result<Vec<Channel>, rusqlite::Error> {
    if let Ok(db) = db {
        let mut stmt = db.prepare("SELECT * FROM favorites")?;
        let favorites_iter = stmt.query_map([], |row| {
            Ok(Channel {
                name: row.get(0)?,
                id: row.get(1)?,
                logo: row.get(2)?,
                favorite: row.get(3)?,
                group: row.get(4)?,
                url: row.get(5)?,
            })
        })?;
        let mut result = vec![];
        for favorite in favorites_iter {
            result.push(favorite.unwrap())
        }
        Ok(result)
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

pub fn add_favorite(db: &Result<Connection, rusqlite::Error>, channel: &Channel) -> bool {
    if let Ok(db) = db {
        let result = db.execute(
        "INSERT INTO favorites (name, id, logo, favorite, agroup, url) values (?1, ?2, ?3, ?4, ?5, ?6)",
        (&channel.name, &channel.id, &channel.logo, 1, &channel.group, &channel.url ),
    );
        result.is_ok()
    } else {
        false
    }
}

pub fn delete_favorite(db: &Result<Connection, rusqlite::Error>, channel: &Channel) -> bool {
    if let Ok(db) = db {
        let result = db.execute(
            "DELETE FROM favorites WHERE url = ?1",
            params![&channel.url],
        );
        result.is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Result<Connection, rusqlite::Error>, Channel) {
        let db = connect_db();
        let newfav = Channel {
            name: "MOCK channel".to_owned(),
            id: "MOCK id".to_owned(),
            logo: "MOCK logo".to_owned(),
            favorite: true,
            group: "MOCK group".to_owned(),
            url: "MOCK url".to_owned(),
        };
        (db, newfav)
    }

    #[test]
    fn test_db() {
        let (db, _) = setup();
        assert!(db.is_ok());
    }

    #[test]
    fn test_get_favorites() {
        let (db, _) = setup();
        let favs = get_favorites(&db);
        println!("{:#?}", favs);
        assert!(favs.is_ok());
    }

    #[test]
    fn test_add_favorite() {
        let (db, newfav) = setup();
        let add_ok = add_favorite(&db, &newfav);
        assert!(add_ok);
    }

    #[test]
    fn test_delete_favorite() {
        let (db, newfav) = setup();
        let delete_ok = delete_favorite(&db, &newfav);
        assert!(delete_ok);
    }
}
