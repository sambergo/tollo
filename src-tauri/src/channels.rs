use crate::m3u_parser::{self, Channel};
use crate::state::{ChannelCache, ChannelCacheState, DbState};
use chrono;
use dirs;
use regex;
use reqwest;
use rusqlite;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter, State};
use uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct ChannelLoadingStatus {
    pub progress: f32,
    pub message: String,
    pub channel_count: Option<usize>,
    pub is_complete: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchProgress {
    pub progress: f32,
    pub message: String,
    pub current_results: usize,
    pub is_complete: bool,
}

#[tauri::command]
pub fn get_channels(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: Option<i32>,
) -> Result<Vec<Channel>, String> {
    get_cached_channels(db_state, cache_state, id)
}

#[tauri::command]
pub fn get_groups(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: Option<i32>,
) -> Result<Vec<String>, String> {
    // Get original channels from cache (this already returns a clone)
    let original_channels = get_cached_channels(db_state, cache_state, id)?;

    // Extract unique groups from cached channels without consuming the original
    let mut groups = std::collections::HashSet::new();
    for channel in &original_channels {
        // Use reference to avoid consuming
        groups.insert(channel.group_title.clone()); // Clone the group title
    }
    Ok(groups.into_iter().collect())
}

#[tauri::command]
pub fn get_cached_channels(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: Option<i32>,
) -> Result<Vec<Channel>, String> {
    let mut cache = cache_state.cache.lock().unwrap();

    // Check if we have valid cache
    if let Some(ref cached) = *cache {
        if cached.channel_list_id == id {
            // Cache hit - return a clone of cached channels to keep original pristine
            return Ok(cached.channels.clone());
        }
    }

    // Cache miss - load channels and update cache
    println!("Loading channels from M3U parser for list {:?}", id);
    let mut db = db_state.db.lock().unwrap();
    let channels = m3u_parser::get_channels(&mut db, id);
    println!("Loaded {} channels for list {:?}", channels.len(), id);

    // Store original channels in cache for future use
    *cache = Some(ChannelCache {
        channel_list_id: id,
        channels: channels.clone(), // Store a copy in cache
        last_updated: SystemTime::now(),
    });

    // Return a clone to keep the cached original untouched
    Ok(channels)
}

#[tauri::command]
pub fn search_channels(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    query: String,
    id: Option<i32>,
) -> Result<Vec<Channel>, String> {
    // Get original channels from cache (this already returns a clone)
    let original_channels = get_cached_channels(db_state, cache_state, id)?;

    // Clone the original list and perform case-insensitive search
    // This ensures the cached original list remains untouched
    let query_lower = query.to_lowercase();
    let filtered_channels: Vec<Channel> = original_channels
        .iter() // Use iter() instead of into_iter() to avoid consuming
        .filter(|channel| {
            channel.name.to_lowercase().contains(&query_lower)
                || channel.group_title.to_lowercase().contains(&query_lower)
        })
        .cloned() // Clone each matching channel
        .collect();

    Ok(filtered_channels)
}

#[tauri::command]
pub fn invalidate_channel_cache(cache_state: State<ChannelCacheState>) -> Result<(), String> {
    let mut cache = cache_state.cache.lock().unwrap();
    *cache = None;
    Ok(())
}

#[tauri::command]
pub fn play_channel(state: State<DbState>, channel: Channel) {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT OR REPLACE INTO history (name, logo, url, group_title, tvg_id, resolution, extra_info, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
        &[&channel.name, &channel.logo, &channel.url, &channel.group_title, &channel.tvg_id, &channel.resolution, &channel.extra_info],
    ).unwrap();

    let player_command: String = db
        .query_row(
            "SELECT player_command FROM settings WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "mpv".to_string());

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

// NEW ASYNC COMMANDS
#[tauri::command]
pub async fn get_channels_async(
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    cache_state: State<'_, ChannelCacheState>,
    id: Option<i32>,
) -> Result<Vec<Channel>, String> {
    // Emit loading start
    let _ = app_handle.emit(
        "channel_loading",
        ChannelLoadingStatus {
            progress: 0.0,
            message: "Starting to load channels...".to_string(),
            channel_count: None,
            is_complete: false,
        },
    );

    // Check cache first (fast operation)
    {
        let cache = cache_state.cache.lock().unwrap();
        if let Some(ref cached) = *cache {
            if cached.channel_list_id == id {
                let _ = app_handle.emit(
                    "channel_loading",
                    ChannelLoadingStatus {
                        progress: 1.0,
                        message: "Loaded from cache instantly!".to_string(),
                        channel_count: Some(cached.channels.len()),
                        is_complete: true,
                    },
                );
                return Ok(cached.channels.clone());
            }
        }
    }

    // Get the file content on the main thread (database operations are fast)
    let m3u_content = {
        let mut db = db_state.db.lock().unwrap();
        get_m3u_content(&mut db, id)?
    };

    // Clone app handle for background parsing
    let app_handle_clone = app_handle.clone();

    // Move only the heavy parsing to background thread
    let channels = tokio::task::spawn_blocking(move || {
        parse_m3u_with_progress(&m3u_content, |progress, message, count| {
            let _ = app_handle_clone.emit(
                "channel_loading",
                ChannelLoadingStatus {
                    progress,
                    message,
                    channel_count: if count > 0 { Some(count) } else { None },
                    is_complete: false,
                },
            );
        })
    })
    .await
    .map_err(|e| format!("Background parsing failed: {}", e))?;

    // Update cache with new channels
    {
        let mut cache = cache_state.cache.lock().unwrap();
        *cache = Some(ChannelCache {
            channel_list_id: id,
            channels: channels.clone(),
            last_updated: SystemTime::now(),
        });
    }

    // Emit completion
    let _ = app_handle.emit(
        "channel_loading",
        ChannelLoadingStatus {
            progress: 1.0,
            message: "Channels loaded successfully!".to_string(),
            channel_count: Some(channels.len()),
            is_complete: true,
        },
    );

    Ok(channels)
}

#[tauri::command]
pub async fn search_channels_async(
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    cache_state: State<'_, ChannelCacheState>,
    query: String,
    id: Option<i32>,
) -> Result<Vec<Channel>, String> {
    let query_clone = query.clone();

    // Emit search start
    let _ = app_handle.emit(
        "search_progress",
        SearchProgress {
            progress: 0.0,
            message: format!("Searching for '{}'...", query),
            current_results: 0,
            is_complete: false,
        },
    );

    // For now, use the blocking version directly to avoid lifetime issues
    let channels = search_channels(db_state, cache_state, query_clone, id)?;

    // Emit completion
    let _ = app_handle.emit(
        "search_progress",
        SearchProgress {
            progress: 1.0,
            message: format!("Search complete! Found {} channels.", channels.len()),
            current_results: channels.len(),
            is_complete: true,
        },
    );

    Ok(channels)
}

#[tauri::command]
pub async fn get_groups_async(
    app_handle: AppHandle,
    db_state: State<'_, DbState>,
    cache_state: State<'_, ChannelCacheState>,
    id: Option<i32>,
) -> Result<Vec<String>, String> {
    // Emit loading start
    let _ = app_handle.emit(
        "groups_loading",
        ChannelLoadingStatus {
            progress: 0.0,
            message: "Loading channel groups...".to_string(),
            channel_count: None,
            is_complete: false,
        },
    );

    // For now, use the blocking version directly to avoid lifetime issues
    let groups = get_groups(db_state, cache_state, id)?;

    // Emit completion
    let _ = app_handle.emit(
        "groups_loading",
        ChannelLoadingStatus {
            progress: 1.0,
            message: format!("Loaded {} groups successfully!", groups.len()),
            channel_count: Some(groups.len()),
            is_complete: true,
        },
    );

    Ok(groups)
}

// Async database operations
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

// Helper function to get M3U content without parsing
fn get_m3u_content(conn: &mut rusqlite::Connection, id: Option<i32>) -> Result<String, String> {
    let query = if let Some(list_id) = id {
        format!(
            "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE id = {}",
            list_id
        )
    } else {
        "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE is_default = 1"
            .to_string()
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let id: i32 = row.get(0).map_err(|e| e.to_string())?;
        let source: String = row.get(1).map_err(|e| e.to_string())?;
        let filepath: Option<String> = row.get(2).map_err(|e| e.to_string())?;
        let last_fetched: Option<i64> = row.get(3).map_err(|e| e.to_string())?;

        let cache_duration_hours: i64 = conn
            .query_row(
                "SELECT cache_duration_hours FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(24);

        let now = chrono::Utc::now().timestamp();

        // Check if we have cached content
        if let (Some(fp), Some(lf)) = (filepath, last_fetched) {
            if now - lf < cache_duration_hours * 3600 {
                let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
                let channel_lists_dir = data_dir.join("channel_lists");
                if let Ok(content) = std::fs::read_to_string(channel_lists_dir.join(fp)) {
                    return Ok(content);
                }
            }
        }

        // Fetch from source
        if source.starts_with("http") {
            let content = reqwest::blocking::get(&source)
                .map_err(|e| format!("Failed to fetch playlist: {}", e))?
                .text()
                .map_err(|e| format!("Failed to read response: {}", e))?;

            // Save to cache
            let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
            let channel_lists_dir = data_dir.join("channel_lists");
            let _ = std::fs::create_dir_all(&channel_lists_dir);
            let filename = format!("{}.m3u", uuid::Uuid::new_v4());
            let new_filepath = channel_lists_dir.join(&filename);
            if std::fs::write(&new_filepath, &content).is_ok() {
                let _ = conn.execute(
                    "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                    &[
                        &filename as &dyn rusqlite::ToSql,
                        &now as &dyn rusqlite::ToSql,
                        &id as &dyn rusqlite::ToSql,
                    ],
                );
            }

            return Ok(content);
        } else {
            let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
            let channel_lists_dir = data_dir.join("channel_lists");
            if let Ok(content) = std::fs::read_to_string(channel_lists_dir.join(&source)) {
                return Ok(content);
            }
        }
    }

    Err("No channel list found".to_string())
}

// Helper function to parse M3U content with progress
fn parse_m3u_with_progress<F>(m3u_content: &str, progress_callback: F) -> Vec<Channel>
where
    F: Fn(f32, String, usize),
{
    let mut channels = Vec::new();
    let re_resolution = regex::Regex::new(r"(\d+p)").unwrap();
    let re_extra_info = regex::Regex::new(r"\[(.*?)\]").unwrap();

    // Count total lines for progress calculation
    let total_lines = m3u_content.lines().count();
    let mut current_line = 0;
    let mut extinf_count = 0;
    let mut parsed_channels = 0;

    progress_callback(0.0, "Starting M3U parsing...".to_string(), 0);

    let mut lines = m3u_content.lines().peekable();

    while let Some(line) = lines.next() {
        current_line += 1;

        if line.starts_with("#EXTINF") {
            extinf_count += 1;
            let name = line
                .split(',')
                .nth(1)
                .unwrap_or_default()
                .trim()
                .to_string();
            let logo = line
                .split("tvg-logo=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let group_title = line
                .split("group-title=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let tvg_id = line
                .split("tvg-id=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let resolution = re_resolution
                .captures(&name)
                .and_then(|c| c.get(1))
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string());
            let extra_info = re_extra_info
                .captures(&name)
                .and_then(|c| c.get(1))
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string());

            if let Some(url_line) = lines.next() {
                current_line += 1;
                if !url_line.starts_with('#') {
                    channels.push(Channel {
                        name,
                        logo,
                        url: url_line.to_string(),
                        group_title,
                        tvg_id,
                        resolution,
                        extra_info,
                    });
                    parsed_channels += 1;
                }
            }

            // Update progress every 1000 channels or 5% of total lines
            if parsed_channels % 1000 == 0 || current_line % (total_lines / 20).max(1) == 0 {
                let progress = (current_line as f32) / (total_lines as f32);
                let message = format!(
                    "Parsed {} channels ({} EXTINF entries)",
                    parsed_channels, extinf_count
                );
                progress_callback(progress, message, parsed_channels);
            }
        }
    }

    progress_callback(
        1.0,
        format!("Parsing complete! {} channels parsed", parsed_channels),
        parsed_channels,
    );
    channels
}
