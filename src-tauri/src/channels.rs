use crate::m3u_parser::{self, Channel};
use crate::m3u_parser_helpers::{get_m3u_content, parse_m3u_with_progress};
use crate::search::clear_advanced_cache;
use crate::state::{ChannelCache, ChannelCacheState, DbState};
use serde::{Deserialize, Serialize};
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::time::{Duration, SystemTime};
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Emitter, State};
use tokio::time;

// Helper function for safe mutex locking with timeout
fn lock_with_timeout<'a, T>(mutex: &'a Mutex<T>, resource_name: &str) -> Result<MutexGuard<'a, T>, String> {
    mutex.lock().map_err(|_| format!("Failed to acquire lock for {}", resource_name))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChannelLoadingStatus {
    pub progress: f32,
    pub message: String,
    pub channel_count: Option<usize>,
    pub is_complete: bool,
}

#[tauri::command]
pub fn get_channels(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: Option<i32>,
) -> std::result::Result<Vec<Channel>, String> {
    get_cached_channels(db_state, cache_state, id)
}

#[tauri::command]
pub fn get_cached_channels(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: Option<i32>,
) -> std::result::Result<Vec<Channel>, String> {
    let mut cache = lock_with_timeout(&cache_state.cache, "channel_cache")?;

    // Check if we have valid cache
    if let Some(ref cached) = *cache {
        if cached.channel_list_id == id {
            // Cache hit - return a clone of cached channels to keep original pristine
            return Ok(cached.channels.clone());
        }
    }

    // Cache miss - load channels and update cache
    println!("Loading channels from M3U parser for list {:?}", id);
    let mut db = lock_with_timeout(&db_state.db, "database_connection")?;
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
pub fn invalidate_channel_cache(cache_state: State<ChannelCacheState>) -> Result<(), String> {
    let mut cache = cache_state.cache.lock().unwrap();
    *cache = None;

    // Also clear search cache since channel data has changed
    clear_advanced_cache();

    Ok(())
}

#[tauri::command]
pub async fn play_channel(state: State<'_, DbState>, channel: Channel) -> Result<(), String> {
    let player_command: String = {
        let db = state.db.lock().unwrap();

        // First, try to add to history
        if let Err(e) = db.execute(
            "INSERT OR REPLACE INTO history (name, logo, url, group_title, tvg_id, resolution, extra_info, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
            &[&channel.name, &channel.logo, &channel.url, &channel.group_title, &channel.tvg_id, &channel.resolution, &channel.extra_info],
        ) {
            eprintln!("Warning: Failed to add channel to history: {}", e);
            // Continue anyway, this shouldn't prevent playback
        }

        db.query_row(
            "SELECT player_command FROM settings WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| crate::settings::detect_default_player())
    }; // Release the database lock here

    let mut command_parts = player_command.split_whitespace();
    let default_player = crate::settings::detect_default_player();
    let command = command_parts.next().unwrap_or(&default_player);
    let args = command_parts.collect::<Vec<&str>>();

    // Try to spawn the external player
    #[cfg(target_os = "windows")]
    let spawn_result = Command::new(command)
        .args(args)
        .arg(&channel.url)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW flag to hide CMD window
        .spawn();
    
    #[cfg(not(target_os = "windows"))]
    let spawn_result = Command::new(command)
        .args(args)
        .arg(&channel.url)
        .spawn();
    
    match spawn_result {
        Ok(mut child) => {
            println!("Successfully launched player for channel: {}", channel.name);

            // Wait a bit to see if the player exits quickly (indicating failure)
            time::sleep(Duration::from_millis(3000)).await;

            // Check if the process is still running
            match child.try_wait() {
                Ok(Some(exit_status)) => {
                    // Process has exited
                    if exit_status.success() {
                        println!("Player exited successfully for channel: {}", channel.name);
                        Ok(())
                    } else {
                        eprintln!(
                            "Player exited with error for channel: {} (exit code: {:?})",
                            channel.name,
                            exit_status.code()
                        );
                        Err("Player failed to play the channel".to_string())
                    }
                }
                Ok(None) => {
                    // Process is still running, assume success
                    println!("Player is still running for channel: {}", channel.name);
                    Ok(())
                }
                Err(e) => {
                    eprintln!(
                        "Failed to check player status for channel {}: {}",
                        channel.name, e
                    );
                    // If we can't check the status, assume it's working
                    Ok(())
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to launch video player '{}': {}", command, e);
            Err(format!("Failed to launch video player: {}", e))
        }
    }
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

    // Clear search cache since channel data has changed
    clear_advanced_cache();

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
