use tauri::State;
use crate::state::DbState;

#[tauri::command]
pub fn get_player_command(state: State<DbState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    db.query_row(
        "SELECT player_command FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())
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
        db.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview) VALUES (1, 'mpv', 24, ?1)",
            &[&enabled],
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
        db.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, 'mpv', 24, 1, ?1, 1, 0)",
            &[&enabled],
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
        db.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, 'mpv', 24, 1, 0, ?1, 0)",
            &[&enabled],
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
        db.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, 'mpv', 24, 1, 0, 1, ?1)",
            &[&enabled],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
} 