use crate::image_cache::DownloadStatus;
use crate::state::ImageCacheState;
use tauri::State;

#[tauri::command]
pub fn get_cached_image(state: State<ImageCacheState>, url: String) -> Result<String, String> {
    state
        .cache
        .get_cached_image_path(&url)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_image_cache(state: State<ImageCacheState>) -> Result<(), String> {
    state.cache.clear_cache().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_image_cache_size(state: State<ImageCacheState>) -> Result<u64, String> {
    state.cache.get_cache_size().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cached_image_async(
    state: State<'_, ImageCacheState>,
    url: String,
) -> Result<String, String> {
    state
        .cache
        .get_cached_image_path_async(&url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_image_cache_async(state: State<'_, ImageCacheState>) -> Result<(), String> {
    state
        .cache
        .clear_cache_async()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_image_cache_size_async(state: State<'_, ImageCacheState>) -> Result<u64, String> {
    state
        .cache
        .get_cache_size_async()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_image_download_status(
    state: State<'_, ImageCacheState>,
    url: String,
) -> Result<String, String> {
    let status = state.cache.get_download_status(&url).await;

    let status_str = match status {
        DownloadStatus::NotCached => "not_cached",
        DownloadStatus::Downloading => "downloading",
        DownloadStatus::Cached => "cached",
        DownloadStatus::Failed(_) => "failed",
    };

    Ok(status_str.to_string())
}

#[tauri::command]
pub async fn preload_images(
    state: State<'_, ImageCacheState>,
    urls: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut results = Vec::new();

    for url in urls {
        match state.cache.get_cached_image_path_async(&url).await {
            Ok(path) => results.push(path),
            Err(e) => {
                eprintln!("Failed to preload image {}: {}", url, e);
                results.push(url);
            }
        }
    }

    Ok(results)
}
