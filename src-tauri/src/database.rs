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
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL
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
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL,
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
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS channels_fts USING fts5(name, content='channels', content_rowid='id')",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            player_command TEXT NOT NULL,
            cache_duration_hours INTEGER NOT NULL DEFAULT 24
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS channel_lists (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            source TEXT NOT NULL,
            filepath TEXT,
            last_fetched INTEGER,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            CONSTRAINT is_default_check CHECK (is_default IN (0, 1))
        )",
        [],
    )?;

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_one_default_list ON channel_lists (is_default) WHERE is_default = 1",
        [],
    )?;

    let list_count: i64 = conn.query_row("SELECT COUNT(*) FROM channel_lists", [], |row| row.get(0))?;
    if list_count == 0 {
        conn.execute(
            "INSERT INTO channel_lists (name, source, is_default) VALUES (?1, ?2, ?3)",
            &["Default", "https://iptv-org.github.io/iptv/countries/fi.m3u", "1"],
        )?;
    }

    Ok(conn)
}

pub fn populate_channels(conn: &mut Connection, channels: &[Channel]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT OR IGNORE INTO channels (name, logo, url, group_title, tvg_id, resolution, extra_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")?;
        for channel in channels {
            stmt.execute(&[
                &channel.name,
                &channel.logo,
                &channel.url,
                &channel.group_title,
                &channel.tvg_id,
                &channel.resolution,
                &channel.extra_info,
            ])?;
        }
    }
    tx.commit()?;

    conn.execute(
        "INSERT INTO channels_fts(rowid, name) SELECT id, name FROM channels",
        [],
    )?;

    Ok(())
}
