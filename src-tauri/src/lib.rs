mod m3u_parser;
mod database;

use std::process::Command;
use m3u_parser::Channel;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;
use serde::{Serialize, Deserialize};
use std::fs;
use reqwest;

struct DbState {
    db: Mutex<Connection>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChannelList {
    id: i32,
    name: String,
    source: String,
    is_default: bool,
    filepath: Option<String>,
    last_fetched: Option<i64>,
}

#[tauri::command]
fn get_channel_lists(state: State<DbState>) -> Result<Vec<ChannelList>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, name, source, is_default, filepath, last_fetched FROM channel_lists").map_err(|e| e.to_string())?;
    let list_iter = stmt.query_map([], |row| {
        Ok(ChannelList {
            id: row.get(0)?,
            name: row.get(1)?,
            source: row.get(2)?,
            is_default: row.get(3)?,
            filepath: row.get(4)?,
            last_fetched: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut lists = Vec::new();
    for list in list_iter {
        lists.push(list.map_err(|e| e.to_string())?);
    }
    Ok(lists)
}

#[tauri::command]
fn add_channel_list(state: State<DbState>, name: String, source: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO channel_lists (name, source) VALUES (?1, ?2)",
        &[&name, &source],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_default_channel_list(state: State<DbState>, id: i32) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let tx = db.transaction().map_err(|e| e.to_string())?;
    tx.execute("UPDATE channel_lists SET is_default = 0", [])
        .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE channel_lists SET is_default = 1 WHERE id = ?1",
        &[&id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_channels(state: State<DbState>, id: Option<i32>) -> Result<Vec<Channel>, String> {
    let mut db = state.db.lock().unwrap();
    Ok(m3u_parser::get_channels(&mut db, id))
}

#[tauri::command]
fn get_groups(state: State<DbState>, id: Option<i32>) -> Result<Vec<String>, String> {
    let mut db = state.db.lock().unwrap();
    Ok(m3u_parser::get_groups(&mut db, id))
}

#[tauri::command]
fn search_channels(state: State<DbState>, query: String) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT c.name, c.logo, c.url, c.group_title, c.tvg_id, c.resolution, c.extra_info FROM channels c JOIN channels_fts fts ON c.id = fts.rowid WHERE fts.name MATCH ?").map_err(|e| e.to_string())?;
    let channel_iter = stmt.query_map([query], |row| {
        Ok(Channel {
            name: row.get(0)?,
            logo: row.get(1)?,
            url: row.get(2)?,
            group_title: row.get(3)?,
            tvg_id: row.get(4)?,
            resolution: row.get(5)?,
            extra_info: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut channels = Vec::new();
    for channel in channel_iter {
        channels.push(channel.map_err(|e| e.to_string())?);
    }
    Ok(channels)
}

#[tauri::command]
fn play_channel(state: State<DbState>, channel: Channel) {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT OR REPLACE INTO history (name, logo, url, group_title, tvg_id, resolution, extra_info, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
        &[&channel.name, &channel.logo, &channel.url, &channel.group_title, &channel.tvg_id, &channel.resolution, &channel.extra_info],
    ).unwrap();

    let player_command: String = db.query_row(
        "SELECT player_command FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| "mpv".to_string());

    let mut command_parts = player_command.split_whitespace();
    let command = command_parts.next().unwrap_or("mpv");
    let args = command_parts.collect::<Vec<&str>>();

    Command::new(command)
        .args(args)
        .arg(channel.url)
        .spawn()
        .expect("Failed to launch video player");
}

#[tauri::command]
fn add_favorite(state: State<DbState>, channel: Channel) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO favorites (name, logo, url, group_title, tvg_id, resolution, extra_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        &[&channel.name, &channel.logo, &channel.url, &channel.group_title, &channel.tvg_id, &channel.resolution, &channel.extra_info],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn remove_favorite(state: State<DbState>, name: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute("DELETE FROM favorites WHERE name = ?1", &[&name])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_favorites(state: State<DbState>) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT name, logo, url, group_title, tvg_id, resolution, extra_info FROM favorites").map_err(|e| e.to_string())?;
    let channel_iter = stmt.query_map([], |row| {
        Ok(Channel {
            name: row.get(0)?,
            logo: row.get(1)?,
            url: row.get(2)?,
            group_title: row.get(3)?,
            tvg_id: row.get(4)?,
            resolution: row.get(5)?,
            extra_info: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut channels = Vec::new();
    for channel in channel_iter {
        channels.push(channel.map_err(|e| e.to_string())?);
    }
    Ok(channels)
}

#[tauri::command]
fn get_history(state: State<DbState>) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT name, logo, url, group_title, tvg_id, resolution, extra_info FROM history ORDER BY timestamp DESC LIMIT 20").map_err(|e| e.to_string())?;
    let channel_iter = stmt.query_map([], |row| {
        Ok(Channel {
            name: row.get(0)?,
            logo: row.get(1)?,
            url: row.get(2)?,
            group_title: row.get(3)?,
            tvg_id: row.get(4)?,
            resolution: row.get(5)?,
            extra_info: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut channels = Vec::new();
    for channel in channel_iter {
        channels.push(channel.map_err(|e| e.to_string())?);
    }
    Ok(channels)
}

#[tauri::command]
fn get_player_command(state: State<DbState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    db.query_row(
        "SELECT player_command FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_player_command(state: State<DbState>, command: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT OR REPLACE INTO settings (id, player_command) VALUES (1, ?1)",
        &[&command],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_cache_duration(state: State<DbState>) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    db.query_row(
        "SELECT cache_duration_hours FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_cache_duration(state: State<DbState>, hours: i64) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "UPDATE settings SET cache_duration_hours = ?1 WHERE id = 1",
        &[&hours],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn refresh_channel_list(state: State<DbState>, id: i32) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT source FROM channel_lists WHERE id = ?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(&[&id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let source: String = row.get(0).map_err(|e| e.to_string())?;
        if source.starts_with("http") {
            if let Ok(content) = reqwest::blocking::get(&source).and_then(|resp| resp.text()) {
                let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
                let filename = format!("{}.m3u", uuid::Uuid::new_v4());
                let new_filepath = data_dir.join(&filename);
                if fs::write(&new_filepath, &content).is_ok() {
                    let now = chrono::Utc::now().timestamp();
                    db.execute(
                        "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                        &[&filename as &dyn rusqlite::ToSql, &now as &dyn rusqlite::ToSql, &id as &dyn rusqlite::ToSql],
                    ).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn delete_channel_list(state: State<DbState>, id: i32) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute("DELETE FROM channel_lists WHERE id = ?1", &[&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_channel_list(state: State<DbState>, id: i32, name: String, source: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "UPDATE channel_lists SET name = ?1, source = ?2 WHERE id = ?3",
        &[&name, &source, &id.to_string()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut db_connection = database::initialize_database().expect("Failed to initialize database");
    let channels = m3u_parser::get_channels(&mut db_connection, None);
    database::populate_channels(&mut db_connection, &channels).expect("Failed to populate channels");

    tauri::Builder::default()
        .manage(DbState {
            db: Mutex::new(db_connection),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_channels,
            get_groups,
            search_channels,
            play_channel,
            add_favorite,
            remove_favorite,
            get_favorites,
            get_history,
            get_player_command,
            set_player_command,
            get_channel_lists,
            add_channel_list,
            set_default_channel_list,
            get_cache_duration,
            set_cache_duration,
            refresh_channel_list,
            delete_channel_list,
            update_channel_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
