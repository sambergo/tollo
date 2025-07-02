mod m3u_parser;
mod database;
mod image_cache;

use std::process::Command;
use std::time::SystemTime;
use m3u_parser::Channel;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{State, Manager};
use serde::{Serialize, Deserialize};
use std::fs;
use reqwest;
use image_cache::ImageCache;

struct DbState {
    db: Mutex<Connection>,
}

struct ImageCacheState {
    cache: Mutex<ImageCache>,
}

#[derive(Debug, Clone)]
struct ChannelCache {
    channel_list_id: Option<i32>,
    channels: Vec<Channel>,
    last_updated: SystemTime,
}

struct ChannelCacheState {
    cache: Mutex<Option<ChannelCache>>,
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
fn get_channels(
    db_state: State<DbState>, 
    cache_state: State<ChannelCacheState>, 
    id: Option<i32>
) -> Result<Vec<Channel>, String> {
    get_cached_channels(db_state, cache_state, id)
}

#[tauri::command]
fn get_groups(
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
fn get_cached_channels(
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
fn search_channels(
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
fn invalidate_channel_cache(cache_state: State<ChannelCacheState>) -> Result<(), String> {
    let mut cache = cache_state.cache.lock().unwrap();
    *cache = None;
    Ok(())
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
        "UPDATE settings SET player_command = ?1 WHERE id = 1",
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
fn get_enable_preview(state: State<DbState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let enable_preview: bool = db.query_row(
        "SELECT enable_preview FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(true); // Default to true if not found
    Ok(enable_preview)
}

#[tauri::command]
fn set_enable_preview(state: State<DbState>, enabled: bool) -> Result<(), String> {
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
fn get_mute_on_start(state: State<DbState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let mute_on_start: bool = db.query_row(
        "SELECT mute_on_start FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(false); // Default to false if not found
    Ok(mute_on_start)
}

#[tauri::command]
fn set_mute_on_start(state: State<DbState>, enabled: bool) -> Result<(), String> {
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
fn get_show_controls(state: State<DbState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let show_controls: bool = db.query_row(
        "SELECT show_controls FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(true); // Default to true if not found
    Ok(show_controls)
}

#[tauri::command]
fn set_show_controls(state: State<DbState>, enabled: bool) -> Result<(), String> {
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
fn get_autoplay(state: State<DbState>) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let autoplay: bool = db.query_row(
        "SELECT autoplay FROM settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(false); // Default to false if not found
    Ok(autoplay)
}

#[tauri::command]
fn set_autoplay(state: State<DbState>, enabled: bool) -> Result<(), String> {
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

#[tauri::command]
fn refresh_channel_list(
    db_state: State<DbState>, 
    cache_state: State<ChannelCacheState>, 
    id: i32
) -> Result<(), String> {
    let db = db_state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT source FROM channel_lists WHERE id = ?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(&[&id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let source: String = row.get(0).map_err(|e| e.to_string())?;
        if source.starts_with("http") {
            // Attempt to fetch the content with proper error handling and headers
            let client = reqwest::blocking::Client::new();
            let response = client
                .get(&source)
                .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .header("Accept", "*/*")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Connection", "keep-alive")
                .header("Upgrade-Insecure-Requests", "1")
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .map_err(|e| format!("Failed to connect to URL: {}", e))?;
            
            println!("HTTP Response Status: {} for URL: {}", response.status(), source);
            
            let content = response
                .text()
                .map_err(|e| format!("Failed to read response: {}", e))?;
            
            println!("Content length: {} bytes", content.len());
            
            // Validate that we got some content
            if content.trim().is_empty() {
                return Err("The downloaded file is empty".to_string());
            }
            
            // Basic M3U validation
            if !content.trim_start().starts_with("#EXTM3U") {
                return Err("The downloaded file is not a valid M3U playlist (missing #EXTM3U header)".to_string());
            }
            
            let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
            let channel_lists_dir = data_dir.join("channel_lists");
            fs::create_dir_all(&channel_lists_dir).map_err(|e| format!("Failed to create channel_lists directory: {}", e))?;
            let filename = format!("{}.m3u", uuid::Uuid::new_v4());
            let new_filepath = channel_lists_dir.join(&filename);
            
            fs::write(&new_filepath, &content)
                .map_err(|e| format!("Failed to save playlist file: {}", e))?;
            
            let now = chrono::Utc::now().timestamp();
            db.execute(
                "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                &[&filename as &dyn rusqlite::ToSql, &now as &dyn rusqlite::ToSql, &id as &dyn rusqlite::ToSql],
            ).map_err(|e| format!("Failed to update database: {}", e))?;
        }
    } else {
        return Err("Channel list not found".to_string());
    }

    // Invalidate cache since channel list was refreshed
    invalidate_channel_cache(cache_state)?;

    Ok(())
}

#[tauri::command]
fn validate_and_add_channel_list(
    db_state: State<DbState>, 
    cache_state: State<ChannelCacheState>, 
    name: String, 
    source: String
) -> Result<i32, String> {
    // Clean and validate the inputs
    let clean_name = name.trim().to_string();
    let clean_source = source.trim().to_string();
    
    if clean_name.is_empty() {
        return Err("Channel list name cannot be empty".to_string());
    }
    
    if clean_source.is_empty() {
        return Err("Channel list source cannot be empty".to_string());
    }
    
    // Validate URL format if it's an HTTP URL
    if clean_source.starts_with("http") {
        // Basic URL validation
        if !clean_source.starts_with("http://") && !clean_source.starts_with("https://") {
            return Err("Invalid URL format. Must start with http:// or https://".to_string());
        }
        
        // Test the URL by attempting to fetch it
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&clean_source)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "*/*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .map_err(|e| format!("Failed to connect to URL: {}", e))?;
        
        println!("HTTP Response Status: {} for URL: {}", response.status(), clean_source);
        
        let content = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;
        
        println!("Content length: {} bytes", content.len());
        
        // Validate that we got some content
        if content.trim().is_empty() {
            return Err("The URL returned an empty response".to_string());
        }
        
        // Basic M3U validation
        if !content.trim_start().starts_with("#EXTM3U") {
            return Err("The URL does not contain a valid M3U playlist (missing #EXTM3U header)".to_string());
        }
        
        // Count channels to give user feedback
        let channel_count = content.lines()
            .filter(|line| line.starts_with("#EXTINF:"))
            .count();
        
        if channel_count == 0 {
            return Err("The playlist appears to be valid but contains no channels".to_string());
        }
        
        println!("Validated M3U playlist with {} channels", channel_count);
    }
    
    let list_id = {
        // Add to database in a scoped block
        let db = db_state.db.lock().unwrap();
        
        // Check for duplicate names
        let existing_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM channel_lists WHERE name = ?1",
            [&clean_name],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        
        if existing_count > 0 {
            return Err(format!("A channel list with the name '{}' already exists", clean_name));
        }
        
        db.execute(
            "INSERT INTO channel_lists (name, source) VALUES (?1, ?2)",
            &[&clean_name, &clean_source],
        ).map_err(|e| format!("Failed to add channel list to database: {}", e))?;
        
        // Get the ID of the newly inserted list
        let list_id: i32 = db.query_row(
            "SELECT id FROM channel_lists WHERE name = ?1 AND source = ?2",
            [&clean_name, &clean_source],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        
        list_id
    }; // db lock is released here
    
    // If it's an HTTP URL, immediately fetch and store the content
    if clean_source.starts_with("http") {
        // Fetch and store in a separate scope
        {
            let db = db_state.db.lock().unwrap();
            
            // We already validated it above, so we know this should work
            let client = reqwest::blocking::Client::new();
            let response = client
                .get(&clean_source)
                .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .header("Accept", "*/*")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("Connection", "keep-alive")
                .header("Upgrade-Insecure-Requests", "1")
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .map_err(|e| format!("Failed to re-fetch URL: {}", e))?;
            
            println!("HTTP Response Status: {} for URL: {}", response.status(), clean_source);
            
            let content = response
                .text()
                .map_err(|e| format!("Failed to read response: {}", e))?;
            
            println!("Content length: {} bytes", content.len());
            
            let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
            let channel_lists_dir = data_dir.join("channel_lists");
            fs::create_dir_all(&channel_lists_dir).map_err(|e| format!("Failed to create channel_lists directory: {}", e))?;
            let filename = format!("{}.m3u", uuid::Uuid::new_v4());
            let new_filepath = channel_lists_dir.join(&filename);
            
            fs::write(&new_filepath, &content)
                .map_err(|e| format!("Failed to save playlist file: {}", e))?;
            
            let now = chrono::Utc::now().timestamp();
            db.execute(
                "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                &[&filename as &dyn rusqlite::ToSql, &now as &dyn rusqlite::ToSql, &list_id as &dyn rusqlite::ToSql],
            ).map_err(|e| format!("Failed to update database: {}", e))?;
        } // db lock is released here
        
        // Now invalidate cache
        invalidate_channel_cache(cache_state)?;
    }
    
    Ok(list_id)
}

#[tauri::command]
fn delete_channel_list(
    db_state: State<DbState>, 
    cache_state: State<ChannelCacheState>, 
    id: i32
) -> Result<(), String> {
    let db = db_state.db.lock().unwrap();
    db.execute("DELETE FROM channel_lists WHERE id = ?1", &[&id])
        .map_err(|e| e.to_string())?;
    
    // Invalidate cache since channel list was deleted
    invalidate_channel_cache(cache_state)?;
    
    Ok(())
}

#[tauri::command]
fn update_channel_list(
    db_state: State<DbState>, 
    cache_state: State<ChannelCacheState>, 
    id: i32, 
    name: String, 
    source: String
) -> Result<(), String> {
    let db = db_state.db.lock().unwrap();
    db.execute(
        "UPDATE channel_lists SET name = ?1, source = ?2 WHERE id = ?3",
        &[&name, &source, &id.to_string()],
    )
    .map_err(|e| e.to_string())?;
    
    // Invalidate cache since channel list was updated
    invalidate_channel_cache(cache_state)?;
    
    Ok(())
}

#[tauri::command]
fn get_cached_image(state: State<ImageCacheState>, url: String) -> Result<String, String> {
    let cache = state.cache.lock().unwrap();
    cache.get_cached_image_path(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_image_cache(state: State<ImageCacheState>) -> Result<(), String> {
    let cache = state.cache.lock().unwrap();
    cache.clear_cache().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_image_cache_size(state: State<ImageCacheState>) -> Result<u64, String> {
    let cache = state.cache.lock().unwrap();
    cache.get_cache_size().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_enabled_groups(state: State<DbState>, channel_list_id: i64) -> Result<Vec<String>, String> {
    let db = state.db.lock().unwrap();
    database::get_enabled_groups(&db, channel_list_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_group_selection(state: State<DbState>, channel_list_id: i64, group_name: String, enabled: bool) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    database::set_group_enabled(&db, channel_list_id, group_name, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
fn sync_channel_list_groups(state: State<DbState>, channel_list_id: i64, groups: Vec<String>) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    database::sync_channel_list_groups(&mut db, channel_list_id, groups).map_err(|e| e.to_string())
}

#[tauri::command]
fn enable_all_groups(state: State<DbState>, channel_list_id: i64, groups: Vec<String>) -> Result<(), String> {
    let mut db = state.db.lock().unwrap();
    database::enable_all_groups(&mut db, channel_list_id, groups).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_filter(state: State<DbState>, channel_list_id: i64, slot_number: i32, search_query: String, selected_group: Option<String>, name: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    database::save_filter(&db, channel_list_id, slot_number, search_query, selected_group, name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_saved_filters(state: State<DbState>, channel_list_id: i64) -> Result<Vec<database::SavedFilter>, String> {
    let db = state.db.lock().unwrap();
    database::get_saved_filters(&db, channel_list_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_saved_filter(state: State<DbState>, channel_list_id: i64, slot_number: i32) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    database::delete_saved_filter(&db, channel_list_id, slot_number).map_err(|e| e.to_string())
}

// Add cleanup function near the top with other utility functions
fn cleanup_orphaned_channel_files(db_connection: &Connection) -> Result<(), String> {
    let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
    let channel_lists_dir = data_dir.join("channel_lists");
    
    // Create channel_lists directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&channel_lists_dir) {
        println!("Warning: Failed to create channel_lists directory: {}", e);
        return Ok(()); // Don't fail startup if we can't create the directory
    }
    
    // Get all .m3u files in the channel_lists directory
    let disk_files = match fs::read_dir(&channel_lists_dir) {
        Ok(entries) => {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.path().is_file() && 
                    entry.path().extension().map_or(false, |ext| ext == "m3u")
                })
                .filter_map(|entry| {
                    entry.file_name().to_str().map(|s| s.to_string())
                })
                .collect::<Vec<String>>()
        },
        Err(_) => {
            println!("Channel lists directory not found or inaccessible, skipping cleanup");
            return Ok(());
        }
    };
    
    // Get all filepaths from database
    let mut stmt = match db_connection.prepare("SELECT filepath FROM channel_lists WHERE filepath IS NOT NULL") {
        Ok(stmt) => stmt,
        Err(e) => {
            println!("Warning: Failed to prepare database query for cleanup: {}", e);
            return Ok(());
        }
    };
    
    let db_files: Result<Vec<String>, _> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).and_then(|iter| iter.collect());
    
    let db_files = match db_files {
        Ok(files) => files,
        Err(e) => {
            println!("Warning: Failed to query database for cleanup: {}", e);
            return Ok(());
        }
    };
    
    // Find orphaned files (on disk but not in database)
    let orphaned_files: Vec<String> = disk_files
        .into_iter()
        .filter(|disk_file| !db_files.contains(disk_file))
        .collect();
    
    let mut deleted_count = 0;
    let mut failed_deletions = 0;
    
    // Delete orphaned files
    for filename in &orphaned_files {
        let file_path = channel_lists_dir.join(filename);
        match fs::remove_file(&file_path) {
            Ok(_) => {
                deleted_count += 1;
                println!("Deleted orphaned channel list file: {}", filename);
            },
            Err(e) => {
                failed_deletions += 1;
                println!("Failed to delete orphaned file {}: {}", filename, e);
            }
        }
    }
    
    // Log cleanup statistics
    if deleted_count > 0 || failed_deletions > 0 {
        println!("Channel list cache cleanup completed: {} files deleted, {} failures", 
                deleted_count, failed_deletions);
    } else if !orphaned_files.is_empty() {
        println!("Channel list cache cleanup: no orphaned files found");
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut db_connection = database::initialize_database().expect("Failed to initialize database");
    
    // Run cleanup on startup to remove orphaned channel list files
    if let Err(e) = cleanup_orphaned_channel_files(&db_connection) {
        println!("Warning: Channel list cleanup failed: {}", e);
    }
    
    let channels = m3u_parser::get_channels(&mut db_connection, None);
    database::populate_channels(&mut db_connection, &channels).expect("Failed to populate channels");

    tauri::Builder::default()
        .manage(DbState {
            db: Mutex::new(db_connection),
        })
        .manage(ChannelCacheState {
            cache: Mutex::new(None),
        })
        .setup(|app| {
            let image_cache = ImageCache::new(app.handle()).expect("Failed to initialize image cache");
            app.manage(ImageCacheState {
                cache: Mutex::new(image_cache),
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_channel_lists,
            add_channel_list,
            set_default_channel_list,
            get_channels,
            get_groups,
            play_channel,
            add_favorite,
            remove_favorite,
            get_favorites,
            get_history,
            search_channels,
            get_player_command,
            set_player_command,
            get_cache_duration,
            set_cache_duration,
            get_enable_preview,
            set_enable_preview,
            get_mute_on_start,
            set_mute_on_start,
            get_show_controls,
            set_show_controls,
            get_autoplay,
            set_autoplay,
            refresh_channel_list,
            validate_and_add_channel_list,
            delete_channel_list,
            update_channel_list,
            invalidate_channel_cache,
            get_cached_image,
            clear_image_cache,
            get_image_cache_size,
            get_enabled_groups,
            update_group_selection,
            sync_channel_list_groups,
            enable_all_groups,
            save_filter,
            get_saved_filters,
            delete_saved_filter,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
