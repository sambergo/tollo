use crate::m3u_parser::Channel;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

// Cache configuration
const MAX_CACHE_ENTRIES: usize = 50;
const CACHE_TTL_SECONDS: u64 = 300; // 5 minutes

#[derive(Debug, Clone)]
struct CacheEntry {
    channels: Vec<Channel>,
    timestamp: u64,
    access_count: usize,
}

// Smart search cache with LRU and TTL eviction
static SEARCH_CACHE: LazyLock<DashMap<String, CacheEntry>> = LazyLock::new(|| DashMap::new());
static CACHE_SIZE: AtomicUsize = AtomicUsize::new(0);

pub struct SearchCache;

impl SearchCache {
    pub fn get(key: &str) -> Option<Vec<Channel>> {
        let now = Self::current_timestamp();
        
        if let Some(mut entry) = SEARCH_CACHE.get_mut(key) {
            // Check if entry is still valid
            if now - entry.timestamp <= CACHE_TTL_SECONDS {
                entry.access_count += 1;
                return Some(entry.channels.clone());
            } else {
                // Entry expired, remove it
                drop(entry);
                SEARCH_CACHE.remove(key);
                CACHE_SIZE.fetch_sub(1, Ordering::Relaxed);
            }
        }
        
        None
    }

    pub fn put(key: String, channels: Vec<Channel>) {
        let now = Self::current_timestamp();
        
        // Check if we need to evict entries
        if CACHE_SIZE.load(Ordering::Relaxed) >= MAX_CACHE_ENTRIES {
            Self::evict_old_entries();
        }

        let entry = CacheEntry {
            channels,
            timestamp: now,
            access_count: 1,
        };

        // Insert or update
        let was_new = SEARCH_CACHE.insert(key, entry).is_none();
        if was_new {
            CACHE_SIZE.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn can_use_incremental_search(new_query: &str, previous_query: &str) -> bool {
        // Check if new query is an extension of the previous query
        new_query.len() > previous_query.len() 
            && new_query.starts_with(previous_query)
            && !previous_query.is_empty()
    }

    pub fn get_incremental_search_space(
        new_query: &str, 
        previous_query: &str, 
        previous_results: &[Channel]
    ) -> Option<Vec<Channel>> {
        if Self::can_use_incremental_search(new_query, previous_query) {
            Some(previous_results.to_vec())
        } else {
            None
        }
    }

    pub fn clear() {
        SEARCH_CACHE.clear();
        CACHE_SIZE.store(0, Ordering::Relaxed);
    }

    // Private helper methods
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn evict_old_entries() {
        let now = Self::current_timestamp();
        let mut entries_to_remove = Vec::new();

        // Collect expired entries and least recently used entries
        for entry in SEARCH_CACHE.iter() {
            let key = entry.key();
            let value = entry.value();
            
            // Remove expired entries
            if now - value.timestamp > CACHE_TTL_SECONDS {
                entries_to_remove.push(key.clone());
            }
        }

        // Remove expired entries
        for key in &entries_to_remove {
            SEARCH_CACHE.remove(key);
            CACHE_SIZE.fetch_sub(1, Ordering::Relaxed);
        }

        // If still too many entries, remove least recently used
        let current_size = CACHE_SIZE.load(Ordering::Relaxed);
        if current_size >= MAX_CACHE_ENTRIES {
            let excess = current_size - MAX_CACHE_ENTRIES + 10; // Remove a few extra
            
            // Collect entries with their access counts
            let mut entries: Vec<(String, usize)> = SEARCH_CACHE
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().access_count))
                .collect();
            
            // Sort by access count (ascending - least used first)
            entries.sort_by_key(|(_, count)| *count);
            
            // Remove least used entries
            for (key, _) in entries.into_iter().take(excess) {
                SEARCH_CACHE.remove(&key);
                CACHE_SIZE.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }
}

// Helper struct for tracking previous search state
#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<Channel>,
}

static LAST_SEARCH_STATE: LazyLock<DashMap<i32, SearchState>> = LazyLock::new(|| DashMap::new());

pub struct IncrementalSearch;

impl IncrementalSearch {
    pub fn get_last_state(channel_list_id: i32) -> Option<SearchState> {
        LAST_SEARCH_STATE.get(&channel_list_id).map(|entry| entry.clone())
    }

    pub fn update_state(channel_list_id: i32, query: String, results: Vec<Channel>) {
        let state = SearchState { query, results };
        LAST_SEARCH_STATE.insert(channel_list_id, state);
    }

    pub fn clear_state(channel_list_id: i32) {
        LAST_SEARCH_STATE.remove(&channel_list_id);
    }

    pub fn clear_all_states() {
        LAST_SEARCH_STATE.clear();
    }
}
