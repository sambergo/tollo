use std::path::PathBuf;
use std::fs;
use sha2::{Sha256, Digest};
use tauri::Manager;

pub struct ImageCache {
    cache_dir: PathBuf,
}

impl ImageCache {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        let cache_dir = app_data_dir.join("image_cache");
        
        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)?;
        
        Ok(ImageCache { cache_dir })
    }
    
    fn url_to_filename(&self, url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = hasher.finalize();
        
        // Get file extension from URL if possible
        let extension = url.split('?').next() // Remove query params
            .and_then(|clean_url| clean_url.split('.').last())
            .filter(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg"))
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
    
    fn download_and_cache(&self, url: &str, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
} 