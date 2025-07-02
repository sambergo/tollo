use tauri::State;
use crate::state::ImageCacheState;

#[tauri::command]
pub fn get_cached_image(state: State<ImageCacheState>, url: String) -> Result<String, String> {
    let cache = state.cache.lock().unwrap();
    cache.get_cached_image_path(&url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_image_cache(state: State<ImageCacheState>) -> Result<(), String> {
    let cache = state.cache.lock().unwrap();
    cache.clear_cache().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_image_cache_size(state: State<ImageCacheState>) -> Result<u64, String> {
    let cache = state.cache.lock().unwrap();
    cache.get_cache_size().map_err(|e| e.to_string())
} 