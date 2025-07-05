use crate::m3u_parser::Channel;
use crate::state::{ChannelCacheState, DbState};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::SystemTime;
use tauri::{AppHandle, Emitter, State};

use crate::channels::{get_cached_channels, ChannelLoadingStatus};
use crate::fuzzy_search::FuzzyMatcher;

// Search cancellation system
static SEARCH_COUNTER: AtomicU64 = AtomicU64::new(0);
static ACTIVE_SEARCH_ID: LazyLock<Mutex<Option<u64>>> = LazyLock::new(|| Mutex::new(None));

// Phase 1: Incremental search cache
#[derive(Clone, Debug)]
struct SearchCacheEntry {
    query: String,
    results: Vec<Channel>,
    timestamp: SystemTime,
    channel_list_id: Option<i32>,
}

static INCREMENTAL_CACHE: LazyLock<Mutex<Option<SearchCacheEntry>>> =
    LazyLock::new(|| Mutex::new(None));

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
    // Generate unique search ID
    let search_id = SEARCH_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Set this as the active search (cancels previous searches)
    {
        let mut active_id = ACTIVE_SEARCH_ID.lock().unwrap();
        *active_id = Some(search_id);
    }

    // If query is empty, clear cache and return all channels
    if query.is_empty() {
        {
            let mut cache = INCREMENTAL_CACHE.lock().unwrap();
            *cache = None;
        }
        let original_channels = get_cached_channels(db_state, cache_state, id)?;
        return Ok(original_channels);
    }

    // Phase 1: Check incremental cache
    let search_space = {
        let cache = INCREMENTAL_CACHE.lock().unwrap();
        if let Some(ref cached) = *cache {
            // Check if we can use incremental search
            if cached.channel_list_id == id
                && query.len() > cached.query.len()
                && query.starts_with(&cached.query)
                && !cached.results.is_empty()
                && cached
                    .timestamp
                    .elapsed()
                    .unwrap_or(std::time::Duration::from_secs(60))
                    < std::time::Duration::from_secs(30)
            {
                // Use cached results as search space (much smaller dataset)
                Some(cached.results.clone())
            } else {
                None
            }
        } else {
            None
        }
    };

    // Get search space (either from cache or full dataset)
    let channels_to_search = if let Some(cached_results) = search_space {
        cached_results
    } else {
        get_cached_channels(db_state, cache_state, id)?
    };

    // Check if we're still the active search
    {
        let active_id = ACTIVE_SEARCH_ID.lock().unwrap();
        if *active_id != Some(search_id) {
            return Err("Search cancelled".to_string());
        }
    }

    // Use fuzzy matcher for intelligent search
    let matcher = FuzzyMatcher::new();
    let filtered_channels = matcher.search_channels(&channels_to_search, &query);

    // Final check if we're still the active search
    {
        let active_id = ACTIVE_SEARCH_ID.lock().unwrap();
        if *active_id != Some(search_id) {
            return Err("Search cancelled".to_string());
        }
    }

    // Update incremental cache with new results
    {
        let mut cache = INCREMENTAL_CACHE.lock().unwrap();
        *cache = Some(SearchCacheEntry {
            query: query.clone(),
            results: filtered_channels.clone(),
            timestamp: SystemTime::now(),
            channel_list_id: id,
        });
    }

    Ok(filtered_channels)
}

// Helper function to clear incremental cache when channel data changes
pub fn clear_incremental_cache() {
    let mut cache = INCREMENTAL_CACHE.lock().unwrap();
    *cache = None;
}

#[tauri::command]
pub fn invalidate_search_cache() -> Result<(), String> {
    clear_incremental_cache();
    Ok(())
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

    // Use the main search function (now with cancellation and incremental cache)
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
