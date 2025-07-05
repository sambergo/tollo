use crate::m3u_parser::Channel;
use crate::state::{ChannelCacheState, DbState};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter, State};

use crate::channels::{get_cached_channels, ChannelLoadingStatus};
use crate::fuzzy_search::FuzzyMatcher;

// Search cancellation system
static SEARCH_COUNTER: AtomicU64 = AtomicU64::new(0);
static ACTIVE_SEARCH_ID: LazyLock<Mutex<Option<u64>>> = LazyLock::new(|| Mutex::new(None));

// Phase 2: Advanced LRU Cache System
#[derive(Clone, Debug)]
struct AdvancedSearchCacheEntry {
    query: String,
    results: Vec<Channel>,
    timestamp: SystemTime,
    channel_list_id: Option<i32>,
    access_count: u32,
    last_accessed: SystemTime,
    result_size: usize,
}

impl AdvancedSearchCacheEntry {
    fn new(query: String, results: Vec<Channel>, channel_list_id: Option<i32>) -> Self {
        let result_size = results.len();
        let now = SystemTime::now();
        Self {
            query,
            results,
            timestamp: now,
            channel_list_id,
            access_count: 1,
            last_accessed: now,
            result_size,
        }
    }

    fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = SystemTime::now();
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed().unwrap_or(Duration::MAX) > ttl
    }
}

// Advanced multi-entry cache with concurrent access
static ADVANCED_CACHE: LazyLock<DashMap<String, AdvancedSearchCacheEntry>> =
    LazyLock::new(|| DashMap::new());

// Cache configuration
const MAX_CACHE_ENTRIES: usize = 100;
const MAX_TOTAL_RESULTS: usize = 50_000;
const CACHE_TTL_SECONDS: u64 = 300; // 5 minutes
const CLEANUP_INTERVAL_SECONDS: u64 = 60; // Cleanup every minute

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchProgress {
    pub progress: f32,
    pub message: String,
    pub current_results: usize,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub total_results: usize,
    pub memory_usage_estimate: usize,
}

// Cache statistics tracking
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

fn make_cache_key(query: &str, channel_list_id: Option<i32>) -> String {
    format!("{}:{}", channel_list_id.unwrap_or(-1), query.to_lowercase())
}

fn cleanup_expired_entries() {
    let ttl = Duration::from_secs(CACHE_TTL_SECONDS);
    ADVANCED_CACHE.retain(|_key, entry| !entry.is_expired(ttl));
}

fn evict_if_needed() {
    cleanup_expired_entries();

    let total_entries = ADVANCED_CACHE.len();
    let total_results: usize = ADVANCED_CACHE.iter().map(|entry| entry.result_size).sum();

    if total_entries <= MAX_CACHE_ENTRIES && total_results <= MAX_TOTAL_RESULTS {
        return;
    }

    // Collect entries for LRU eviction
    let mut entries_by_lru: Vec<(String, SystemTime, usize)> = ADVANCED_CACHE
        .iter()
        .map(|entry| (entry.key().clone(), entry.last_accessed, entry.result_size))
        .collect();

    // Sort by last accessed time (oldest first)
    entries_by_lru.sort_by_key(|(_, last_accessed, _)| *last_accessed);

    // Remove oldest entries until we're under limits
    let mut removed_entries = 0;
    let mut removed_results = 0;

    for (key, _, result_size) in entries_by_lru {
        if total_entries - removed_entries <= MAX_CACHE_ENTRIES
            && total_results - removed_results <= MAX_TOTAL_RESULTS
        {
            break;
        }

        ADVANCED_CACHE.remove(&key);
        removed_entries += 1;
        removed_results += result_size;
    }
}

fn find_best_cached_prefix(
    query: &str,
    channel_list_id: Option<i32>,
) -> Option<(String, Vec<Channel>)> {
    let query_lower = query.to_lowercase();
    let mut best_match: Option<(String, Vec<Channel>)> = None;
    let mut best_length = 0;

    for entry in ADVANCED_CACHE.iter() {
        if entry.channel_list_id == channel_list_id {
            let cached_query = entry.query.to_lowercase();

            // Check if current query starts with cached query
            if query_lower.starts_with(&cached_query) && cached_query.len() > best_length {
                best_length = cached_query.len();
                best_match = Some((entry.query.clone(), entry.results.clone()));
            }
        }
    }

    best_match
}

fn get_search_space(
    query: &str,
    channel_list_id: Option<i32>,
    db_state: &State<DbState>,
    cache_state: &State<ChannelCacheState>,
) -> Result<Vec<Channel>, String> {
    let cache_key = make_cache_key(query, channel_list_id);

    // 1. Try exact match
    if let Some(mut cached_entry) = ADVANCED_CACHE.get_mut(&cache_key) {
        if !cached_entry.is_expired(Duration::from_secs(CACHE_TTL_SECONDS)) {
            cached_entry.access();
            CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            return Ok(cached_entry.results.clone());
        }
    }

    // 2. Try longest prefix match
    if let Some((prefix_query, cached_results)) = find_best_cached_prefix(query, channel_list_id) {
        if query.len() > prefix_query.len() {
            CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            return Ok(cached_results);
        }
    }

    // 3. Cache miss - use full dataset
    CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
    get_cached_channels(db_state.clone(), cache_state.clone(), channel_list_id)
}

#[tauri::command]
pub fn search_channels(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    query: String,
    id: Option<i32>,
) -> Result<Vec<Channel>, String> {
    // Generate unique search ID for cancellation
    let search_id = SEARCH_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Set this as the active search (cancels previous searches)
    {
        let mut active_id = ACTIVE_SEARCH_ID.lock().unwrap();
        *active_id = Some(search_id);
    }

    // If query is empty, clear cache and return all channels
    if query.is_empty() {
        let original_channels = get_cached_channels(db_state, cache_state, id)?;
        return Ok(original_channels);
    }

    // Periodic cache maintenance
    if SEARCH_COUNTER.load(Ordering::Relaxed) % 10 == 0 {
        evict_if_needed();
    }

    // Get search space using advanced cache strategy
    let channels_to_search = get_search_space(&query, id, &db_state, &cache_state)?;

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

    // Update advanced cache with new results
    let cache_key = make_cache_key(&query, id);
    let cache_entry = AdvancedSearchCacheEntry::new(query.clone(), filtered_channels.clone(), id);
    ADVANCED_CACHE.insert(cache_key, cache_entry);

    Ok(filtered_channels)
}

// Helper function to clear advanced cache when channel data changes
pub fn clear_advanced_cache() {
    ADVANCED_CACHE.clear();
}

#[tauri::command]
pub fn invalidate_search_cache() -> Result<(), String> {
    clear_advanced_cache();
    Ok(())
}

#[tauri::command]
pub fn get_cache_stats() -> Result<CacheStats, String> {
    let entries = ADVANCED_CACHE.len();
    let total_results: usize = ADVANCED_CACHE.iter().map(|entry| entry.result_size).sum();
    let memory_estimate = total_results * std::mem::size_of::<Channel>()
        + entries * std::mem::size_of::<AdvancedSearchCacheEntry>();

    Ok(CacheStats {
        hits: CACHE_HITS.load(Ordering::Relaxed),
        misses: CACHE_MISSES.load(Ordering::Relaxed),
        entries,
        total_results,
        memory_usage_estimate: memory_estimate,
    })
}

#[tauri::command]
pub fn warm_cache_with_common_searches(
    db_state: State<DbState>,
    cache_state: State<ChannelCacheState>,
    id: Option<i32>,
) -> Result<(), String> {
    let common_searches = vec!["news", "sport", "hd", "music", "movie", "tv", "live"];

    for search_term in common_searches {
        let _ = search_channels(
            db_state.clone(),
            cache_state.clone(),
            search_term.to_string(),
            id,
        );
    }

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

    // Use the main search function (now with advanced caching and cancellation)
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
