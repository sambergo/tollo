mod m3u_parser;
mod database;

use std::process::Command;
use m3u_parser::Channel;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

struct DbState {
    db: Mutex<Connection>,
}

#[tauri::command]
fn get_channels() -> Vec<Channel> {
    m3u_parser::get_channels()
}

#[tauri::command]
fn play_channel(url: String) {
    Command::new("mpv")
        .arg(url)
        .spawn()
        .expect("Failed to launch mpv");
}

#[tauri::command]
fn add_favorite(state: State<DbState>, channel: Channel) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO favorites (name, logo, url, group_title) VALUES (?1, ?2, ?3, ?4)",
        &[&channel.name, &channel.logo, &channel.url, &channel.group_title],
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
    let mut stmt = db.prepare("SELECT name, logo, url, group_title FROM favorites").map_err(|e| e.to_string())?;
    let channel_iter = stmt.query_map([], |row| {
        Ok(Channel {
            name: row.get(0)?,
            logo: row.get(1)?,
            url: row.get(2)?,
            group_title: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut channels = Vec::new();
    for channel in channel_iter {
        channels.push(channel.map_err(|e| e.to_string())?);
    }
    Ok(channels)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_connection = database::initialize_database().expect("Failed to initialize database");

    tauri::Builder::default()
        .manage(DbState {
            db: Mutex::new(db_connection),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_channels,
            play_channel,
            add_favorite,
            remove_favorite,
            get_favorites
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}