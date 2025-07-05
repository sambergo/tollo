use crate::channels::invalidate_channel_cache;
use crate::state::{ChannelCacheState, ChannelList, DbState};
use chrono::Utc;
use dirs;
use reqwest;
use rusqlite;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistFetchStatus {
    pub id: i32,
    pub status: String, // "starting", "fetching", "processing", "saving", "completed", "error"
    pub progress: f32,  // 0.0 to 1.0
    pub message: String,
    pub channel_count: Option<usize>,
    pub error: Option<String>,
}

pub struct FetchState {
    pub operations: Arc<AsyncMutex<HashMap<i32, PlaylistFetchStatus>>>,
}

impl FetchState {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }
}

// Helper function to emit progress events
async fn emit_progress(
    app_handle: &AppHandle,
    fetch_state: &State<'_, FetchState>,
    status: PlaylistFetchStatus,
) {
    // Update the state
    let mut operations = fetch_state.operations.lock().await;
    operations.insert(status.id, status.clone());
    drop(operations);

    // Emit event to frontend
    if let Err(e) = app_handle.emit("playlist_fetch_status", &status) {
        eprintln!("Failed to emit playlist_fetch_status event: {}", e);
    }
}

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

// Async commands with proper implementation
#[tauri::command]
pub async fn refresh_channel_list_async(
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    cache_state: State<'_, ChannelCacheState>,
    fetch_state: State<'_, FetchState>,
    id: i32,
) -> Result<(), String> {
    // Get the source URL from database
    let source = {
        let db = db_state.db.lock().unwrap();
        db.query_row(
            "SELECT source FROM channel_lists WHERE id = ?1",
            &[&id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "Channel list not found".to_string())?
    };

    // Only process HTTP sources
    if !source.starts_with("http") {
        return Err("Only HTTP sources can be refreshed".to_string());
    }

    // Emit starting status
    emit_progress(
        &app_handle,
        &fetch_state,
        PlaylistFetchStatus {
            id,
            status: "starting".to_string(),
            progress: 0.0,
            message: "Initializing refresh...".to_string(),
            channel_count: None,
            error: None,
        },
    )
    .await;

    // Emit fetching status
    emit_progress(
        &app_handle,
        &fetch_state,
        PlaylistFetchStatus {
            id,
            status: "fetching".to_string(),
            progress: 0.2,
            message: "Downloading playlist...".to_string(),
            channel_count: None,
            error: None,
        },
    )
    .await;

    // Fetch the playlist
    let client = reqwest::Client::new();
    let response = client
        .get(&source)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch: {}", e))?;

    // Emit processing status
    emit_progress(
        &app_handle,
        &fetch_state,
        PlaylistFetchStatus {
            id,
            status: "processing".to_string(),
            progress: 0.6,
            message: "Processing playlist content...".to_string(),
            channel_count: None,
            error: None,
        },
    )
    .await;

    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read: {}", e))?;

    if content.trim().is_empty() || !content.trim_start().starts_with("#EXTM3U") {
        let error_msg = "Invalid M3U playlist".to_string();
        emit_progress(
            &app_handle,
            &fetch_state,
            PlaylistFetchStatus {
                id,
                status: "error".to_string(),
                progress: 0.0,
                message: "Failed to process playlist".to_string(),
                channel_count: None,
                error: Some(error_msg.clone()),
            },
        )
        .await;
        return Err(error_msg);
    }

    // Count channels
    let channel_count = content
        .lines()
        .filter(|line| line.starts_with("#EXTINF:"))
        .count();

    // Emit saving status
    emit_progress(
        &app_handle,
        &fetch_state,
        PlaylistFetchStatus {
            id,
            status: "saving".to_string(),
            progress: 0.8,
            message: "Saving playlist...".to_string(),
            channel_count: Some(channel_count),
            error: None,
        },
    )
    .await;

    // Save to file
    let data_dir = dirs::data_dir().unwrap().join("gui-tollo/channel_lists");
    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    let filename = format!("{}.m3u", Uuid::new_v4());
    let filepath = data_dir.join(&filename);

    fs::write(&filepath, &content).map_err(|e| format!("Failed to save: {}", e))?;

    // Update database
    let now = Utc::now().timestamp();
    {
        let db = db_state.db.lock().unwrap();
        db.execute(
            "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
            &[
                &filename as &dyn rusqlite::ToSql,
                &now as &dyn rusqlite::ToSql,
                &id as &dyn rusqlite::ToSql,
            ],
        )
        .map_err(|e| format!("Failed to update: {}", e))?;
    }

    // Invalidate cache
    invalidate_channel_cache(cache_state)?;

    // Emit completed status
    emit_progress(
        &app_handle,
        &fetch_state,
        PlaylistFetchStatus {
            id,
            status: "completed".to_string(),
            progress: 1.0,
            message: "Playlist refreshed successfully".to_string(),
            channel_count: Some(channel_count),
            error: None,
        },
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn validate_and_add_channel_list_async(
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    cache_state: State<'_, ChannelCacheState>,
    fetch_state: State<'_, FetchState>,
    name: String,
    source: String,
) -> Result<i32, String> {
    let clean_name = name.trim();
    let clean_source = source.trim();

    if clean_name.is_empty() || clean_source.is_empty() {
        return Err("Name and source cannot be empty".to_string());
    }

    // First, add the list to get an ID
    let list_id = {
        let db = db_state.db.lock().unwrap();

        // Check if already exists
        let existing: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM channel_lists WHERE name = ?1",
                [clean_name],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if existing > 0 {
            return Err(format!("Channel list '{}' already exists", clean_name));
        }

        // Insert the new list
        db.execute(
            "INSERT INTO channel_lists (name, source) VALUES (?1, ?2)",
            &[&clean_name, &clean_source],
        )
        .map_err(|e| e.to_string())?;

        // Get the ID
        db.query_row(
            "SELECT id FROM channel_lists WHERE name = ?1 AND source = ?2",
            [clean_name, clean_source],
            |row| row.get::<_, i32>(0),
        )
        .map_err(|e| e.to_string())?
    };

    // If it's an HTTP source, fetch it asynchronously
    if clean_source.starts_with("http") {
        if !clean_source.starts_with("http://") && !clean_source.starts_with("https://") {
            return Err("Invalid URL format".to_string());
        }

        // Emit starting status
        emit_progress(
            &app_handle,
            &fetch_state,
            PlaylistFetchStatus {
                id: list_id,
                status: "starting".to_string(),
                progress: 0.0,
                message: "Validating playlist...".to_string(),
                channel_count: None,
                error: None,
            },
        )
        .await;

        // Emit fetching status
        emit_progress(
            &app_handle,
            &fetch_state,
            PlaylistFetchStatus {
                id: list_id,
                status: "fetching".to_string(),
                progress: 0.2,
                message: "Downloading playlist...".to_string(),
                channel_count: None,
                error: None,
            },
        )
        .await;

        // Fetch the playlist
        let client = reqwest::Client::new();
        let response = client
            .get(clean_source)
            .header("User-Agent", "Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        // Emit processing status
        emit_progress(
            &app_handle,
            &fetch_state,
            PlaylistFetchStatus {
                id: list_id,
                status: "processing".to_string(),
                progress: 0.6,
                message: "Processing playlist content...".to_string(),
                channel_count: None,
                error: None,
            },
        )
        .await;

        let content = response
            .text()
            .await
            .map_err(|e| format!("Failed to read: {}", e))?;

        if content.trim().is_empty() || !content.trim_start().starts_with("#EXTM3U") {
            let error_msg = "Invalid M3U playlist".to_string();
            emit_progress(
                &app_handle,
                &fetch_state,
                PlaylistFetchStatus {
                    id: list_id,
                    status: "error".to_string(),
                    progress: 0.0,
                    message: "Failed to validate playlist".to_string(),
                    channel_count: None,
                    error: Some(error_msg.clone()),
                },
            )
            .await;
            return Err(error_msg);
        }

        let channel_count = content
            .lines()
            .filter(|line| line.starts_with("#EXTINF:"))
            .count();

        if channel_count == 0 {
            let error_msg = "No channels found".to_string();
            emit_progress(
                &app_handle,
                &fetch_state,
                PlaylistFetchStatus {
                    id: list_id,
                    status: "error".to_string(),
                    progress: 0.0,
                    message: "No channels found in playlist".to_string(),
                    channel_count: None,
                    error: Some(error_msg.clone()),
                },
            )
            .await;
            return Err(error_msg);
        }

        // Emit saving status
        emit_progress(
            &app_handle,
            &fetch_state,
            PlaylistFetchStatus {
                id: list_id,
                status: "saving".to_string(),
                progress: 0.8,
                message: "Saving playlist...".to_string(),
                channel_count: Some(channel_count),
                error: None,
            },
        )
        .await;

        // Save the playlist
        let data_dir = dirs::data_dir().unwrap().join("gui-tollo/channel_lists");
        fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
        let filename = format!("{}.m3u", Uuid::new_v4());
        let filepath = data_dir.join(&filename);

        fs::write(&filepath, &content).map_err(|e| format!("Failed to save: {}", e))?;

        // Update database with file info
        let now = Utc::now().timestamp();
        {
            let db = db_state.db.lock().unwrap();
            db.execute(
                "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                &[
                    &filename as &dyn rusqlite::ToSql,
                    &now as &dyn rusqlite::ToSql,
                    &list_id as &dyn rusqlite::ToSql,
                ],
            )
            .map_err(|e| format!("Failed to update: {}", e))?;
        }

        // Invalidate cache
        invalidate_channel_cache(cache_state)?;

        // Emit completed status
        emit_progress(
            &app_handle,
            &fetch_state,
            PlaylistFetchStatus {
                id: list_id,
                status: "completed".to_string(),
                progress: 1.0,
                message: "Playlist added successfully".to_string(),
                channel_count: Some(channel_count),
                error: None,
            },
        )
        .await;
    }

    Ok(list_id)
}

#[tauri::command]
pub async fn get_playlist_fetch_status(
    fetch_state: State<'_, FetchState>,
    id: i32,
) -> Result<Option<PlaylistFetchStatus>, String> {
    let operations = fetch_state.operations.lock().await;
    Ok(operations.get(&id).cloned())
}

#[tauri::command]
pub async fn get_all_playlist_fetch_status(
    fetch_state: State<'_, FetchState>,
) -> Result<Vec<PlaylistFetchStatus>, String> {
    let operations = fetch_state.operations.lock().await;
    Ok(operations.values().cloned().collect())
}

// Legacy sync functions for backward compatibility
#[tauri::command]
pub fn refresh_channel_list(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: i32,
) -> Result<(), String> {
    let db = db_state.db.lock().unwrap();
    let source: String = db
        .query_row(
            "SELECT source FROM channel_lists WHERE id = ?1",
            &[&id],
            |row| row.get(0),
        )
        .map_err(|_| "Channel list not found")?;

    if source.starts_with("http") {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&source)
            .header("User-Agent", "Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .map_err(|e| format!("Failed to fetch: {}", e))?;
        let content = response
            .text()
            .map_err(|e| format!("Failed to read: {}", e))?;

        if content.trim().is_empty() || !content.trim_start().starts_with("#EXTM3U") {
            return Err("Invalid M3U playlist".to_string());
        }

        let data_dir = dirs::data_dir().unwrap().join("gui-tollo/channel_lists");
        fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
        let filename = format!("{}.m3u", Uuid::new_v4());
        let filepath = data_dir.join(&filename);

        fs::write(&filepath, &content).map_err(|e| format!("Failed to save: {}", e))?;

        let now = Utc::now().timestamp();
        db.execute(
            "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
            &[
                &filename as &dyn rusqlite::ToSql,
                &now as &dyn rusqlite::ToSql,
                &id as &dyn rusqlite::ToSql,
            ],
        )
        .map_err(|e| format!("Failed to update: {}", e))?;
    }

    invalidate_channel_cache(cache_state)?;
    Ok(())
}

#[tauri::command]
pub fn validate_and_add_channel_list(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    name: String,
    source: String,
) -> Result<i32, String> {
    let clean_name = name.trim();
    let clean_source = source.trim();

    if clean_name.is_empty() || clean_source.is_empty() {
        return Err("Name and source cannot be empty".to_string());
    }

    if clean_source.starts_with("http") {
        if !clean_source.starts_with("http://") && !clean_source.starts_with("https://") {
            return Err("Invalid URL format".to_string());
        }

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(clean_source)
            .header("User-Agent", "Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .map_err(|e| format!("Failed to connect: {}", e))?;
        let content = response
            .text()
            .map_err(|e| format!("Failed to read: {}", e))?;

        if content.trim().is_empty() || !content.trim_start().starts_with("#EXTM3U") {
            return Err("Invalid M3U playlist".to_string());
        }

        let channel_count = content
            .lines()
            .filter(|line| line.starts_with("#EXTINF:"))
            .count();
        if channel_count == 0 {
            return Err("No channels found".to_string());
        }
    }

    let db = db_state.db.lock().unwrap();
    let existing: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM channel_lists WHERE name = ?1",
            [clean_name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if existing > 0 {
        return Err(format!("Channel list '{}' already exists", clean_name));
    }

    db.execute(
        "INSERT INTO channel_lists (name, source) VALUES (?1, ?2)",
        &[&clean_name, &clean_source],
    )
    .map_err(|e| e.to_string())?;
    let list_id: i32 = db
        .query_row(
            "SELECT id FROM channel_lists WHERE name = ?1 AND source = ?2",
            [clean_name, clean_source],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if clean_source.starts_with("http") {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(clean_source)
            .header("User-Agent", "Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .map_err(|e| format!("Failed to re-fetch: {}", e))?;
        let content = response
            .text()
            .map_err(|e| format!("Failed to read: {}", e))?;

        let data_dir = dirs::data_dir().unwrap().join("gui-tollo/channel_lists");
        fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
        let filename = format!("{}.m3u", Uuid::new_v4());
        let filepath = data_dir.join(&filename);

        fs::write(&filepath, &content).map_err(|e| format!("Failed to save: {}", e))?;

        let now = Utc::now().timestamp();
        db.execute(
            "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
            &[
                &filename as &dyn rusqlite::ToSql,
                &now as &dyn rusqlite::ToSql,
                &list_id as &dyn rusqlite::ToSql,
            ],
        )
        .map_err(|e| format!("Failed to update: {}", e))?;

        invalidate_channel_cache(cache_state)?;
    }

    Ok(list_id)
}
