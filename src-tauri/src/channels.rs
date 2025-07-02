use std::process::Command;
use std::time::SystemTime;
use tauri::State;
use crate::m3u_parser::{self, Channel};
use crate::state::{DbState, ChannelCacheState, ChannelCache};

#[tauri::command]
pub fn get_channels(
    db_state: State<DbState>, 
    cache_state: State<ChannelCacheState>, 
    id: Option<i32>
) -> Result<Vec<Channel>, String> {
    get_cached_channels(db_state, cache_state, id)
}

#[tauri::command]
pub fn get_groups(
    db_state: State<DbState>, 
    cache_state: State<ChannelCacheState>, 
    id: Option<i32>
) -> Result<Vec<String>, String> {
    // Get original channels from cache (this already returns a clone)
    let original_channels = get_cached_channels(db_state, cache_state, id)?;
    
    // Extract unique groups from cached channels without consuming the original
    let mut groups = std::collections::HashSet::new();
    for channel in &original_channels {  // Use reference to avoid consuming
        groups.insert(channel.group_title.clone());  // Clone the group title
    }
    Ok(groups.into_iter().collect())
}

#[tauri::command]
pub fn get_cached_channels(
    db_state: State<DbState>, 
    cache_state: State<ChannelCacheState>, 
    id: Option<i32>
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
        channels: channels.clone(),  // Store a copy in cache
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
    id: Option<i32>
) -> Result<Vec<Channel>, String> {
    // Get original channels from cache (this already returns a clone)
    let original_channels = get_cached_channels(db_state, cache_state, id)?;
    
    // Clone the original list and perform case-insensitive search
    // This ensures the cached original list remains untouched
    let query_lower = query.to_lowercase();
    let filtered_channels: Vec<Channel> = original_channels
        .iter()  // Use iter() instead of into_iter() to avoid consuming
        .filter(|channel| {
            channel.name.to_lowercase().contains(&query_lower) ||
            channel.group_title.to_lowercase().contains(&query_lower)
        })
        .cloned()  // Clone each matching channel
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
pub fn get_history(state: State<DbState>) -> Result<Vec<Channel>, String> {
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