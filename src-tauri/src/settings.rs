use tauri::State;
use crate::state::DbState;
use std::process::Command;

pub fn detect_default_player() -> String {
    let players = if cfg!(target_os = "windows") {
        vec!["vlc", "mpv", "mpc-hc"]
    } else if cfg!(target_os = "macos") {
        vec!["mpv", "vlc", "iina"]
    } else {
        vec!["mpv", "vlc", "totem", "smplayer"]
    };

    for player in players {
        if Command::new("which").arg(player).output().map(|o| o.status.success()).unwrap_or(false) ||
           Command::new("where").arg(player).output().map(|o| o.status.success()).unwrap_or(false) {
            return player.to_string();
        }
    }
    
    // Fallback to mpv on Linux/macOS, vlc on Windows
    if cfg!(target_os = "windows") {
        "vlc".to_string()
    } else {
        "mpv".to_string()
    }
}

#[tauri::command]
pub fn get_player_command(state: State<DbState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    match db.query_row(
        "SELECT player_command FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ) {
        Ok(command) => Ok(command),
        Err(_) => {
            let default_player = detect_default_player();
            Ok(default_player)
        }
    }
}

#[tauri::command]
pub fn set_player_command(state: State<DbState>, command: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "UPDATE settings SET player_command = ?1 WHERE id = 1",
        &[&command],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_cache_duration(state: State<DbState>) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    db.query_row(
        "SELECT cache_duration_hours FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_cache_duration(state: State<DbState>, hours: i64) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "UPDATE settings SET cache_duration_hours = ?1 WHERE id = 1",
        &[&hours],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_enable_preview(state: State<DbState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let enable_preview: bool = db.query_row(
        "SELECT enable_preview FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(true); // Default to true if not found
    Ok(enable_preview)
}

#[tauri::command]
pub fn set_enable_preview(state: State<DbState>, enabled: bool) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    
    // First try to update existing row
    let rows_affected = db.execute(
        "UPDATE settings SET enable_preview = ?1 WHERE id = 1",
        &[&enabled],
    ).map_err(|e| e.to_string())?;
    
    // If no rows were affected, insert a new settings row with default values
    if rows_affected == 0 {
        let default_player = detect_default_player();
        db.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview) VALUES (1, ?1, 24, ?2)",
            rusqlite::params![default_player, enabled],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// --- Video Player Settings: Mute on Start ---
#[tauri::command]
pub fn get_mute_on_start(state: State<DbState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let mute_on_start: bool = db.query_row(
        "SELECT mute_on_start FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(false); // Default to false if not found
    Ok(mute_on_start)
}

#[tauri::command]
pub fn set_mute_on_start(state: State<DbState>, enabled: bool) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let rows_affected = db.execute(
        "UPDATE settings SET mute_on_start = ?1 WHERE id = 1",
        &[&enabled],
    ).map_err(|e| e.to_string())?;
    if rows_affected == 0 {
        let default_player = detect_default_player();
        db.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, ?1, 24, 1, ?2, 1, 0)",
            rusqlite::params![default_player, enabled],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// --- Video Player Settings: Show Controls ---
#[tauri::command]
pub fn get_show_controls(state: State<DbState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let show_controls: bool = db.query_row(
        "SELECT show_controls FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(true); // Default to true if not found
    Ok(show_controls)
}

#[tauri::command]
pub fn set_show_controls(state: State<DbState>, enabled: bool) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let rows_affected = db.execute(
        "UPDATE settings SET show_controls = ?1 WHERE id = 1",
        &[&enabled],
    ).map_err(|e| e.to_string())?;
    if rows_affected == 0 {
        let default_player = detect_default_player();
        db.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, ?1, 24, 1, 0, ?2, 0)",
            rusqlite::params![default_player, enabled],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// --- Video Player Settings: Autoplay ---
#[tauri::command]
pub fn get_autoplay(state: State<DbState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let autoplay: bool = db.query_row(
        "SELECT autoplay FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(false); // Default to false if not found
    Ok(autoplay)
}

#[tauri::command]
pub fn set_autoplay(state: State<DbState>, enabled: bool) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let rows_affected = db.execute(
        "UPDATE settings SET autoplay = ?1 WHERE id = 1",
        &[&enabled],
    ).map_err(|e| e.to_string())?;
    if rows_affected == 0 {
        let default_player = detect_default_player();
        db.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, ?1, 24, 1, 0, 1, ?2)",
            rusqlite::params![default_player, enabled],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
} 