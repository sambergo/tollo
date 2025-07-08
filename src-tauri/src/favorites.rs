use crate::m3u_parser::Channel;
use crate::state::DbState;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn add_favorite(state: State<DbState>, channel: Channel) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO favorites (name, logo, url, group_title, tvg_id, resolution, extra_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        &[&channel.name, &channel.logo, &channel.url, &channel.group_title, &channel.tvg_id, &channel.resolution, &channel.extra_info],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_favorite(state: State<DbState>, name: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute("DELETE FROM favorites WHERE name = ?1", &[&name])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_favorites(state: State<DbState>) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT name, logo, url, group_title, tvg_id, resolution, extra_info FROM favorites",
        )
        .map_err(|e| e.to_string())?;
    let channel_iter = stmt
        .query_map([], |row| {
            Ok(Channel {
                name: row.get(0)?,
                logo: row.get(1)?,
                url: row.get(2)?,
                group_title: row.get(3)?,
                tvg_id: row.get(4)?,
                resolution: row.get(5)?,
                extra_info: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut channels = Vec::new();
    for channel in channel_iter {
        channels.push(channel.map_err(|e| e.to_string())?);
    }
    Ok(channels)
}

#[tauri::command]
pub async fn add_favorite_async(
    app_handle: AppHandle,
    state: State<'_, DbState>,
    channel: Channel,
) -> Result<(), String> {
    // Emit start
    let _ = app_handle.emit("favorite_operation", "Adding to favorites...");

    // Use blocking version for now
    let result = add_favorite(state, channel);

    // Emit completion
    let _ = app_handle.emit("favorite_operation", "Added to favorites!");

    result
}

#[tauri::command]
pub async fn remove_favorite_async(
    app_handle: AppHandle,
    state: State<'_, DbState>,
    name: String,
) -> Result<(), String> {
    // Emit start
    let _ = app_handle.emit("favorite_operation", "Removing from favorites...");

    // Use blocking version for now
    let result = remove_favorite(state, name);

    // Emit completion
    let _ = app_handle.emit("favorite_operation", "Removed from favorites!");

    result
}

#[tauri::command]
pub async fn get_favorites_async(
    app_handle: AppHandle,
    state: State<'_, DbState>,
) -> Result<Vec<Channel>, String> {
    // Emit start
    let _ = app_handle.emit("favorites_loading", "Loading favorites...");

    // Use blocking version for now
    let result = get_favorites(state);

    // Emit completion
    let _ = app_handle.emit("favorites_loading", "Favorites loaded!");

    result
}
