use crate::m3u_parser::Channel;
use rusqlite::{Connection, Result};
use std::fs;

pub fn initialize_database() -> Result<Connection> {
    let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
    fs::create_dir_all(&data_dir).unwrap();
    let db_path = data_dir.join("database.sqlite");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS favorites (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS channels (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS channels_fts USING fts5(name, content='channels', content_rowid='id')",
        [],
    )?;

    Ok(conn)
}

pub fn populate_channels(conn: &mut Connection, channels: &[Channel]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT OR IGNORE INTO channels (name, logo, url, group_title) VALUES (?1, ?2, ?3, ?4)")?;
        for channel in channels {
            stmt.execute(&[&channel.name, &channel.logo, &channel.url, &channel.group_title])?;
        }
    }
    tx.commit()?;

    conn.execute(
        "INSERT INTO channels_fts(rowid, name) SELECT id, name FROM channels",
        [],
    )?;

    Ok(())
}
