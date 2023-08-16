extern crate rusqlite;
use std::env;

use crate::app::Channel;
use rusqlite::{Connection, Result};

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

pub fn get_favorites(db: Connection) -> Result<Vec<Channel>, rusqlite::Error> {
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
}

pub fn add_favorite(db: Connection, channel: &Channel) -> bool {
    let result = db.execute(
        "INSERT INTO favorites (name, id, logo, favorite, agroup, url) values (?1, ?2, ?3, ?4, ?5, ?6)",
        (&channel.name, &channel.id, &channel.logo, 1, &channel.group, &channel.url ),
    );
    result.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_db() {
        let db = connect_db();
        assert!(db.is_ok());
    }

    #[test]
    fn test_get_favorites() {
        let db = connect_db();
        let favs = get_favorites(db.unwrap());
        println!("{:?}", favs);
        assert!(favs.is_ok());
    }

    #[test]
    fn test_add_favorite() {
        let db = connect_db();
        let newfav = Channel {
            name: "MOCK channel".to_owned(),
            id: "MOCK id".to_owned(),
            logo: "MOCK logo".to_owned(),
            favorite: true,
            group: "MOCK group".to_owned(),
            url: "MOCK url".to_owned(),
        };
        let add_ok = add_favorite(db.unwrap(), &newfav);
        assert!(add_ok);
    }
}
