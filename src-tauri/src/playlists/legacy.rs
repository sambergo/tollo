use crate::channels::invalidate_channel_cache;
use crate::state::{ChannelCacheState, DbState};
use chrono::Utc;
use dirs;
use reqwest;
use rusqlite;
use std::fs;
use tauri::State;
use uuid::Uuid;

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

        let data_dir = dirs::data_dir().unwrap().join("tollo/channel_lists");
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

        let data_dir = dirs::data_dir().unwrap().join("tollo/channel_lists");
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
