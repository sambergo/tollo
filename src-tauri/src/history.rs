use crate::m3u_parser::Channel;
use crate::state::DbState;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_history(state: State<DbState>) -> Result<Vec<Channel>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT name, logo, url, group_title, tvg_id, resolution, extra_info FROM history ORDER BY timestamp DESC LIMIT 20").map_err(|e| e.to_string())?;
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
pub async fn get_history_async(
    app_handle: AppHandle,
    state: State<'_, DbState>,
) -> Result<Vec<Channel>, String> {
    // Emit start
    let _ = app_handle.emit("history_loading", "Loading history...");

    // Use blocking version for now
    let result = get_history(state);

    // Emit completion
    let _ = app_handle.emit("history_loading", "History loaded!");

    result
}
