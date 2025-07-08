use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::fs as async_fs;
use tokio::sync::{Mutex, Semaphore};

// Simple download status
#[derive(Debug, Clone)]
pub enum DownloadStatus {
    NotCached,
    Downloading,
    Cached,
    Failed(String),
}

pub struct ImageCache {
    cache_dir: PathBuf,
    // Async download management
    download_queue: Arc<Mutex<HashMap<String, DownloadStatus>>>,
    download_semaphore: Arc<Semaphore>, // Limit concurrent downloads
    client: reqwest::Client,
}

impl ImageCache {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        let cache_dir = app_data_dir.join("image_cache");

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)?;

        // Create HTTP client with reasonable timeouts
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("tollo/1.0")
            .build()?;

        Ok(ImageCache {
            cache_dir,
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            download_semaphore: Arc::new(Semaphore::new(5)), // Max 5 concurrent downloads
            client,
        })
    }

    fn url_to_filename(&self, url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = hasher.finalize();

        // Get file extension from URL if possible
        let extension = url
            .split('?')
            .next() // Remove query params
            .and_then(|clean_url| clean_url.split('.').last())
            .filter(|ext| {
                matches!(
                    ext.to_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg"
                )
            })
            .unwrap_or("jpg"); // Default to jpg if no valid extension found

        format!("{:x}.{}", hash, extension)
    }

    pub fn get_cached_image_path(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let filename = self.url_to_filename(url);
        let file_path = self.cache_dir.join(&filename);

        // Check if file already exists
        if file_path.exists() {
            // Return the absolute file path
            return Ok(file_path.to_string_lossy().to_string());
        }

        // Download and cache the image
        self.download_and_cache(url, &file_path)?;

        // Return the absolute file path
        Ok(file_path.to_string_lossy().to_string())
    }

    fn download_and_cache(
        &self,
        url: &str,
        file_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Download the image
        let response = reqwest::blocking::get(url)?;

        // Check if the response is successful
        if !response.status().is_success() {
            return Err(format!("Failed to download image: HTTP {}", response.status()).into());
        }

        // Get the image bytes
        let image_bytes = response.bytes()?;

        // Write to cache file
        fs::write(file_path, image_bytes)?;

        Ok(())
    }

    pub fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    pub fn get_cache_size(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let mut total_size = 0u64;

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    total_size += entry.metadata()?.len();
                }
            }
        }

        Ok(total_size)
    }

    pub async fn get_cached_image_path_async(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let filename = self.url_to_filename(url);
        let file_path = self.cache_dir.join(&filename);

        // Check if file already exists
        if file_path.exists() {
            return Ok(file_path.to_string_lossy().to_string());
        }

        // Check if download is already in progress
        {
            let mut queue = self.download_queue.lock().await;
            match queue.get(url) {
                Some(DownloadStatus::Downloading) => {
                    // Wait a bit and check again
                    drop(queue);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                    // Check if file exists now
                    if file_path.exists() {
                        return Ok(file_path.to_string_lossy().to_string());
                    }

                    // Still downloading, return error to let caller retry
                    return Err("Download in progress".into());
                }
                Some(DownloadStatus::Failed(err)) => {
                    return Err(err.clone().into());
                }
                Some(DownloadStatus::Cached) => {
                    if file_path.exists() {
                        return Ok(file_path.to_string_lossy().to_string());
                    }
                    // File was deleted, remove from queue and continue
                    queue.remove(url);
                }
                _ => {}
            }

            // Mark as downloading
            queue.insert(url.to_string(), DownloadStatus::Downloading);
        }

        // Download and cache the image asynchronously
        let result = self.download_and_cache_async(url, &file_path).await;

        // Update queue status
        {
            let mut queue = self.download_queue.lock().await;
            match &result {
                Ok(_) => {
                    queue.insert(url.to_string(), DownloadStatus::Cached);
                }
                Err(e) => {
                    queue.insert(url.to_string(), DownloadStatus::Failed(e.to_string()));
                }
            }
        }

        // Clean up old entries after a delay
        let queue = self.download_queue.clone();
        let url_clone = url.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let mut queue = queue.lock().await;
            queue.remove(&url_clone);
        });

        result?;
        Ok(file_path.to_string_lossy().to_string())
    }

    async fn download_and_cache_async(
        &self,
        url: &str,
        file_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Acquire semaphore to limit concurrent downloads
        let _permit = self.download_semaphore.acquire().await?;

        // Download the image
        let response = self.client.get(url).send().await?;

        // Check if the response is successful
        if !response.status().is_success() {
            return Err(format!("Failed to download image: HTTP {}", response.status()).into());
        }

        // Get the image bytes
        let image_bytes = response.bytes().await?;

        // Write to cache file asynchronously
        async_fs::write(file_path, image_bytes).await?;

        Ok(())
    }

    pub async fn get_download_status(&self, url: &str) -> DownloadStatus {
        let queue = self.download_queue.lock().await;
        queue.get(url).cloned().unwrap_or(DownloadStatus::NotCached)
    }

    pub async fn clear_cache_async(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.cache_dir.exists() {
            async_fs::remove_dir_all(&self.cache_dir).await?;
            async_fs::create_dir_all(&self.cache_dir).await?;
        }

        // Clear download queue
        let mut queue = self.download_queue.lock().await;
        queue.clear();

        Ok(())
    }

    pub async fn get_cache_size_async(
        &self,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let mut total_size = 0u64;

        if self.cache_dir.exists() {
            let mut dir = async_fs::read_dir(&self.cache_dir).await?;
            while let Some(entry) = dir.next_entry().await? {
                if entry.file_type().await?.is_file() {
                    total_size += entry.metadata().await?.len();
                }
            }
        }

        Ok(total_size)
    }
}
