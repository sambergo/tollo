use std::fs;
use tauri::{State, Manager};
use reqwest;
use rusqlite;
use uuid::Uuid;
use chrono::Utc;
use dirs;
use crate::state::{DbState, ChannelCacheState, ChannelList};
use crate::channels::invalidate_channel_cache;

#[tauri::command]
pub fn get_channel_lists(state: State<DbState>) -> Result<Vec<ChannelList>, String> {
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
pub fn add_channel_list(state: State<DbState>, name: String, source: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO channel_lists (name, source) VALUES (?1, ?2)",
        &[&name, &source],
    ).map_err(|e| e.to_string())?;
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
pub fn refresh_channel_list(
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
            let filename = format!("{}.m3u", Uuid::new_v4());
            let new_filepath = channel_lists_dir.join(&filename);
            
            fs::write(&new_filepath, &content)
                .map_err(|e| format!("Failed to save playlist file: {}", e))?;
            
            let now = Utc::now().timestamp();
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
pub fn validate_and_add_channel_list(
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
            let filename = format!("{}.m3u", Uuid::new_v4());
            let new_filepath = channel_lists_dir.join(&filename);
            
            fs::write(&new_filepath, &content)
                .map_err(|e| format!("Failed to save playlist file: {}", e))?;
            
            let now = Utc::now().timestamp();
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
pub fn delete_channel_list(
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
pub fn update_channel_list(
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
pub fn start_channel_list_selection(
    cache_state: State<ChannelCacheState>,
) -> Result<(), String> {
    // Simply invalidate the cache to trigger a reload
    invalidate_channel_cache(cache_state)?;
    Ok(())
} 