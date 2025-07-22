use crate::channels::invalidate_channel_cache;
use crate::playlists::fetch::refresh_channel_list_async;
use crate::playlists::types::FetchState;
use crate::state::{ChannelCacheState, ChannelList, DbState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_channel_lists(state: State<DbState>) -> Result<Vec<ChannelList>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT id, name, source, is_default, filepath, last_fetched FROM channel_lists")
        .map_err(|e| e.to_string())?;
    let list_iter = stmt
        .query_map([], |row| {
            Ok(ChannelList {
                id: row.get(0)?,
                name: row.get(1)?,
                source: row.get(2)?,
                is_default: row.get(3)?,
                filepath: row.get(4)?,
                last_fetched: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut lists = Vec::new();
    for list in list_iter {
        lists.push(list.map_err(|e| e.to_string())?);
    }
    Ok(lists)
}

#[tauri::command]
pub fn add_channel_list(state: State<DbState>, name: String, source: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO channel_lists (name, source) VALUES (?1, ?2)",
        &[&name, &source],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_default_channel_list(state: State<DbState>, id: i32) -> Result<(), String> {
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
pub fn delete_channel_list(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: i32,
) -> Result<(), String> {
    let db = db_state.db.lock().unwrap();
    db.execute("DELETE FROM channel_lists WHERE id = ?1", &[&id])
        .map_err(|e| e.to_string())?;
    invalidate_channel_cache(cache_state)?;
    Ok(())
}

#[tauri::command]
pub fn update_channel_list(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: i32,
    name: String,
    source: String,
) -> Result<(), String> {
    let db = db_state.db.lock().unwrap();
    db.execute(
        "UPDATE channel_lists SET name = ?1, source = ?2 WHERE id = ?3",
        &[&name, &source, &id.to_string()],
    )
    .map_err(|e| e.to_string())?;
    invalidate_channel_cache(cache_state)?;
    Ok(())
}

#[tauri::command]
pub fn start_channel_list_selection(cache_state: State<ChannelCacheState>) -> Result<(), String> {
    invalidate_channel_cache(cache_state)?;
    Ok(())
}

#[tauri::command]
pub async fn start_channel_list_selection_async(
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    cache_state: State<'_, ChannelCacheState>,
    fetch_state: State<'_, FetchState>,
    id: i32,
) -> Result<(), String> {
    // Check if the playlist needs to be refreshed based on cache settings
    let needs_refresh = {
        let db = db_state.db.lock().unwrap();
        
        // Get cache duration and current time
        let cache_duration_hours: i64 = db
            .query_row(
                "SELECT cache_duration_hours FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(24);
        
        let now = chrono::Utc::now().timestamp();
        
        // Check if playlist exists and when it was last fetched
        let (last_fetched, source, filepath): (Option<i64>, String, Option<String>) = db
            .query_row(
                "SELECT last_fetched, source, filepath FROM channel_lists WHERE id = ?1",
                [id],
                |row| Ok((row.get(0).ok(), row.get(1)?, row.get(2).ok())),
            )
            .map_err(|e| format!("Failed to get playlist info: {}", e))?;
        
        // Handle both HTTP and file sources
        if !source.starts_with("http") {
            // For file sources, check if the file exists and is valid
            if !std::path::Path::new(&source).exists() {
                return Err(format!("Playlist file '{}' not found", source));
            }
            
            // Read and validate the file
            match std::fs::read_to_string(&source) {
                Ok(content) => {
                    if content.trim().is_empty() || !content.trim_start().starts_with("#EXTM3U") {
                        return Err("Invalid M3U playlist file".to_string());
                    }
                    // File is valid, we can proceed with refresh
                }
                Err(e) => {
                    return Err(format!("Failed to read file '{}': {}", source, e));
                }
            }
        }
        
        // Check if cache is expired or invalid
        match (last_fetched, filepath) {
            (Some(last_fetch_time), Some(cached_file)) => {
                // For file sources, check if the source file is newer than our cached version
                if !source.starts_with("http") {
                    match std::fs::metadata(&source) {
                        Ok(file_metadata) => {
                            if let Ok(modified_time) = file_metadata.modified() {
                                let file_timestamp = modified_time.duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default().as_secs() as i64;
                                // If source file is newer than our last fetch, refresh
                                file_timestamp > last_fetch_time
                            } else {
                                // Can't get modification time, refresh to be safe
                                true
                            }
                        }
                        Err(_) => {
                            // Source file doesn't exist or can't be accessed, but we already validated it above
                            // This shouldn't happen, but refresh to be safe
                            true
                        }
                    }
                } else {
                    // Check if cache is expired for HTTP sources
                    let cache_expired = (now - last_fetch_time) >= (cache_duration_hours * 3600);
                    
                    if cache_expired {
                        true // Cache is expired, need refresh
                    } else {
                        // Cache is not expired, but validate the cached file
                        let data_dir = dirs::data_dir().unwrap().join("tollo");
                        let channel_lists_dir = data_dir.join("channel_lists");
                        let cached_file_path = channel_lists_dir.join(&cached_file);
                    
                        // Check if cached file exists and is not empty
                        match std::fs::metadata(&cached_file_path) {
                            Ok(metadata) => {
                                if metadata.len() == 0 {
                                    // File is empty, need refresh
                                    true
                                } else {
                                    // File exists and has content, check if it's valid M3U
                                    match std::fs::read_to_string(&cached_file_path) {
                                        Ok(content) => {
                                            // Consider it invalid if it's too short or doesn't contain M3U markers
                                            content.trim().is_empty() 
                                                || content.len() < 10 
                                                || (!content.contains("#EXTINF") && !content.contains("#EXTM3U"))
                                        }
                                        Err(_) => true, // Can't read file, need refresh
                                    }
                                }
                            }
                            Err(_) => true, // File doesn't exist, need refresh
                        }
                    }
                }
            }
            _ => true, // Never fetched or no cached file, needs refresh
        }
    };
    
    // Only refresh if cache is expired or never fetched
    if needs_refresh {
        refresh_channel_list_async(app_handle, db_state, cache_state, fetch_state, id).await
    } else {
        // Cache is still valid, no need to refresh
        Ok(())
    }
}
