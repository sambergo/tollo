use crate::m3u_parser::Channel;
use crate::state::{ChannelCacheState, DbState};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::{AppHandle, Emitter, State};

use crate::channels::{get_cached_channels, ChannelLoadingStatus};
use crate::fuzzy_search::FuzzyMatcher;

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchProgress {
    pub progress: f32,
    pub message: String,
    pub current_results: usize,
    pub is_complete: bool,
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

    // If query is empty, return all channels
    if query.is_empty() {
        return Ok(original_channels);
    }

    // Use fuzzy matcher for intelligent search
    let matcher = FuzzyMatcher::new();
    let filtered_channels = matcher.search_channels(&original_channels, &query);

    Ok(filtered_channels)
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
    let mut groups = HashSet::new();
    for channel in &original_channels {
        // Use reference to avoid consuming
        groups.insert(channel.group_title.clone()); // Clone the group title
    }
    Ok(groups.into_iter().collect())
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

    // Use the updated blocking version with fuzzy search
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
