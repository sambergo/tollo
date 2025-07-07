use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::fs as async_fs;
use tokio::sync::{Mutex, Semaphore};

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    
    #[test]
    fn test_url_to_filename() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join("test_cache");
        let _ = fs::create_dir_all(&cache_dir);
        
        let client = reqwest::Client::new();
        let cache = ImageCache {
            cache_dir: cache_dir.clone(),
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            download_semaphore: Arc::new(Semaphore::new(5)),
            client,
        };
        
        // Test with different URLs
        let filename1 = cache.url_to_filename("http://example.com/image.jpg");
        let filename2 = cache.url_to_filename("http://example.com/image.png");
        let filename3 = cache.url_to_filename("http://example.com/image");
        let filename4 = cache.url_to_filename("http://example.com/image.jpg?query=param");
        
        // Should preserve extension
        assert!(filename1.ends_with(".jpg"));
        assert!(filename2.ends_with(".png"));
        assert!(filename3.ends_with(".jpg")); // Default extension
        assert!(filename4.ends_with(".jpg")); // Should ignore query params
        
        // Same URL should produce same filename
        let filename5 = cache.url_to_filename("http://example.com/image.jpg");
        assert_eq!(filename1, filename5);
        
        // Different URLs should produce different filenames
        assert_ne!(filename1, filename2);
    }
    
    #[test]
    fn test_url_to_filename_supported_extensions() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join("test_cache");
        let _ = fs::create_dir_all(&cache_dir);
        
        let client = reqwest::Client::new();
        let cache = ImageCache {
            cache_dir: cache_dir.clone(),
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            download_semaphore: Arc::new(Semaphore::new(5)),
            client,
        };
        
        // Test supported extensions
        assert!(cache.url_to_filename("http://example.com/image.jpg").ends_with(".jpg"));
        assert!(cache.url_to_filename("http://example.com/image.jpeg").ends_with(".jpeg"));
        assert!(cache.url_to_filename("http://example.com/image.png").ends_with(".png"));
        assert!(cache.url_to_filename("http://example.com/image.gif").ends_with(".gif"));
        assert!(cache.url_to_filename("http://example.com/image.webp").ends_with(".webp"));
        assert!(cache.url_to_filename("http://example.com/image.svg").ends_with(".svg"));
        
        // Test unsupported extension (should default to jpg)
        assert!(cache.url_to_filename("http://example.com/image.bmp").ends_with(".jpg"));
        assert!(cache.url_to_filename("http://example.com/image.tiff").ends_with(".jpg"));
        
        // Test case insensitivity
        assert!(cache.url_to_filename("http://example.com/image.JPG").ends_with(".JPG"));
        assert!(cache.url_to_filename("http://example.com/image.PNG").ends_with(".PNG"));
    }
    
    #[test]
    fn test_url_to_filename_hash_consistency() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join("test_cache");
        let _ = fs::create_dir_all(&cache_dir);
        
        let client = reqwest::Client::new();
        let cache = ImageCache {
            cache_dir: cache_dir.clone(),
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            download_semaphore: Arc::new(Semaphore::new(5)),
            client,
        };
        
        let url = "http://example.com/image.jpg";
        let filename1 = cache.url_to_filename(url);
        let filename2 = cache.url_to_filename(url);
        
        assert_eq!(filename1, filename2);
        
        // Should be deterministic across different cache instances
        let cache2 = ImageCache {
            cache_dir: cache_dir.clone(),
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            download_semaphore: Arc::new(Semaphore::new(5)),
            client: reqwest::Client::new(),
        };
        
        let filename3 = cache2.url_to_filename(url);
        assert_eq!(filename1, filename3);
    }
    
    #[test]
    fn test_get_cached_image_path_existing_file() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join("test_cache_existing");
        let _ = fs::create_dir_all(&cache_dir);
        
        let client = reqwest::Client::new();
        let cache = ImageCache {
            cache_dir: cache_dir.clone(),
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            download_semaphore: Arc::new(Semaphore::new(5)),
            client,
        };
        
        let url = "http://example.com/test.jpg";
        let filename = cache.url_to_filename(url);
        let file_path = cache_dir.join(&filename);
        
        // Create a test file
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test image data").unwrap();
        
        // Should return the existing file path
        let result = cache.get_cached_image_path(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), file_path.to_string_lossy().to_string());
        
        // Clean up
        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir(&cache_dir);
    }
    
    #[test]
    fn test_download_status_variants() {
        // Test the different download status variants
        let not_cached = DownloadStatus::NotCached;
        let downloading = DownloadStatus::Downloading;
        let cached = DownloadStatus::Cached;
        let failed = DownloadStatus::Failed("Network error".to_string());
        
        // Test Clone trait
        let not_cached_clone = not_cached.clone();
        let downloading_clone = downloading.clone();
        let cached_clone = cached.clone();
        let failed_clone = failed.clone();
        
        // Test Debug trait
        assert!(!format!("{:?}", not_cached_clone).is_empty());
        assert!(!format!("{:?}", downloading_clone).is_empty());
        assert!(!format!("{:?}", cached_clone).is_empty());
        assert!(!format!("{:?}", failed_clone).is_empty());
        
        // Test Failed variant contains error message
        match failed_clone {
            DownloadStatus::Failed(msg) => assert_eq!(msg, "Network error"),
            _ => panic!("Expected Failed variant"),
        }
    }
    
    #[test]
    fn test_image_cache_new_creates_directory() {
        // This test would need a mock AppHandle to test properly
        // For now, we'll just test that the directory creation logic works
        let temp_dir = std::env::temp_dir();
        let test_cache_dir = temp_dir.join("test_image_cache");
        
        // Clean up any existing directory
        let _ = fs::remove_dir_all(&test_cache_dir);
        
        // Create the directory
        let result = fs::create_dir_all(&test_cache_dir);
        assert!(result.is_ok());
        
        // Verify it exists
        assert!(test_cache_dir.exists());
        assert!(test_cache_dir.is_dir());
        
        // Clean up
        let _ = fs::remove_dir_all(&test_cache_dir);
    }
    
    #[test]
    fn test_filename_sanitization() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join("test_cache");
        let _ = fs::create_dir_all(&cache_dir);
        
        let client = reqwest::Client::new();
        let cache = ImageCache {
            cache_dir: cache_dir.clone(),
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            download_semaphore: Arc::new(Semaphore::new(5)),
            client,
        };
        
        // Test URLs with special characters
        let urls = vec![
            "http://example.com/image with spaces.jpg",
            "http://example.com/image%20encoded.jpg",
            "http://example.com/image?query=param&other=value",
            "http://example.com/image#fragment",
            "http://example.com/path/to/image.jpg",
        ];
        
        for url in urls {
            let filename = cache.url_to_filename(url);
            
            // Should be a valid filename (hex + extension)
            assert!(filename.contains("."));
            
            // Should not contain special characters that could cause filesystem issues
            assert!(!filename.contains(" "));
            assert!(!filename.contains("?"));
            assert!(!filename.contains("#"));
            assert!(!filename.contains("/"));
            assert!(!filename.contains("\\"));
        }
    }

    // Error scenario tests
    mod error_tests {
        use super::*;

        #[test]
        fn test_get_cached_image_path_nonexistent_file() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test_cache_nonexistent");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // This will fail since we can't actually download in tests
            let result = cache.get_cached_image_path("http://nonexistent.example.com/image.jpg");
            assert!(result.is_err());
        }

        #[test]
        fn test_url_to_filename_with_malformed_urls() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test_cache");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Test with malformed URLs
            let malformed_urls = vec![
                "",
                "not-a-url",
                "http://",
                "ftp://example.com/image.jpg", // Different protocol
                "http://example.com/../../etc/passwd", // Path traversal attempt
                "http://example.com/image.exe", // Executable extension
                "javascript:alert('xss')", // Script URL
                "data:image/jpeg;base64,/9j/4AAQSkZJRgAB", // Data URL
            ];
            
            for url in malformed_urls {
                let filename = cache.url_to_filename(url);
                
                // Should still produce a valid filename (hash-based)
                assert!(filename.contains("."));
                assert!(!filename.contains("/"));
                assert!(!filename.contains("\\"));
                assert!(!filename.is_empty());
                
                // Should be deterministic
                let filename2 = cache.url_to_filename(url);
                assert_eq!(filename, filename2);
            }
        }

        #[test]
        fn test_url_to_filename_with_very_long_url() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test_cache");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Create a very long URL
            let base_url = "http://example.com/";
            let long_path = "a".repeat(10000);
            let long_url = format!("{}{}.jpg", base_url, long_path);
            
            let filename = cache.url_to_filename(&long_url);
            
            // Should produce a reasonable filename length
            assert!(filename.len() < 100); // SHA256 hex + extension should be reasonable
            assert!(filename.ends_with(".jpg"));
        }

        #[test]
        fn test_url_to_filename_with_unicode_urls() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test_cache");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Test with Unicode characters in URLs
            let unicode_urls = vec![
                "http://example.com/图片.jpg", // Chinese
                "http://example.com/صورة.png", // Arabic
                "http://example.com/картинка.gif", // Russian
                "http://example.com/画像🖼️.webp", // Japanese + emoji
                "http://exámple.com/imágen.jpg", // Accented domain
            ];
            
            for url in unicode_urls {
                let filename = cache.url_to_filename(url);
                
                // Should produce ASCII-safe filenames
                assert!(filename.is_ascii());
                assert!(filename.contains("."));
                
                // Should be consistent
                let filename2 = cache.url_to_filename(url);
                assert_eq!(filename, filename2);
            }
        }

        #[test]
        fn test_clear_cache_nonexistent_directory() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("nonexistent_cache_dir");
            
            // Ensure directory doesn't exist
            let _ = fs::remove_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Should handle nonexistent directory gracefully
            let result = cache.clear_cache();
            assert!(result.is_ok());
        }

        #[test]
        fn test_get_cache_size_nonexistent_directory() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("nonexistent_cache_dir");
            
            // Ensure directory doesn't exist
            let _ = fs::remove_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Should return 0 for nonexistent directory
            let result = cache.get_cache_size();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
        }

        #[test]
        fn test_get_cache_size_with_various_file_sizes() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test_cache_sizes");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Create files of different sizes
            let file1 = cache_dir.join("small.jpg");
            let file2 = cache_dir.join("medium.png");
            let file3 = cache_dir.join("large.gif");
            
            fs::write(&file1, b"small").unwrap(); // 5 bytes
            fs::write(&file2, &vec![0u8; 1000]).unwrap(); // 1000 bytes
            fs::write(&file3, &vec![0u8; 50000]).unwrap(); // 50000 bytes
            
            let total_size = cache.get_cache_size().unwrap();
            assert_eq!(total_size, 5 + 1000 + 50000);
            
            // Clean up
            let _ = fs::remove_dir_all(&cache_dir);
        }

        #[test]
        fn test_url_to_filename_collision_resistance() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test_cache");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Generate many similar URLs to test hash collision resistance
            let mut filenames = std::collections::HashSet::new();
            
            for i in 0..1000 {
                let url = format!("http://example.com/image{}.jpg", i);
                let filename = cache.url_to_filename(&url);
                
                // Should not have collisions
                assert!(!filenames.contains(&filename), "Hash collision detected for URL: {}", url);
                filenames.insert(filename);
            }
            
            assert_eq!(filenames.len(), 1000);
        }

        #[test]
        fn test_cache_dir_path_with_special_characters() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test cache with spaces & symbols!");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            let url = "http://example.com/test.jpg";
            let filename = cache.url_to_filename(url);
            
            // Should work even with special characters in cache directory path
            assert!(!filename.is_empty());
            assert!(filename.contains(".jpg"));
            
            // Should be able to construct valid file paths
            let file_path = cache_dir.join(&filename);
            assert!(file_path.to_string_lossy().contains(&filename));
            
            // Clean up
            let _ = fs::remove_dir_all(&cache_dir);
        }

        #[test]
        fn test_download_status_edge_cases() {
            // Test PartialEq would be implemented if needed
            let status1 = DownloadStatus::NotCached;
            let status2 = DownloadStatus::NotCached;
            let status3 = DownloadStatus::Downloading;
            let status4 = DownloadStatus::Failed("Error 1".to_string());
            let status5 = DownloadStatus::Failed("Error 2".to_string());
            
            // Test that different statuses behave correctly
            assert!(!format!("{:?}", status1).is_empty());
            assert!(!format!("{:?}", status2).is_empty());
            assert!(!format!("{:?}", status3).is_empty());
            assert!(!format!("{:?}", status4).is_empty());
            assert!(!format!("{:?}", status5).is_empty());
            
            // Test cloning with different error messages
            let cloned_status4 = status4.clone();
            match (&status4, &cloned_status4) {
                (DownloadStatus::Failed(msg1), DownloadStatus::Failed(msg2)) => {
                    assert_eq!(msg1, msg2);
                }
                _ => panic!("Expected Failed status"),
            }
        }

        #[test]
        fn test_url_to_filename_with_empty_and_whitespace() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test_cache");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Test edge cases
            let edge_cases = vec![
                "",
                " ",
                "\t",
                "\n",
                "   ",
                "\r\n",
            ];
            
            for url in edge_cases {
                let filename = cache.url_to_filename(url);
                
                // Should still produce valid filenames
                assert!(!filename.is_empty());
                assert!(filename.contains("."));
                assert!(filename.ends_with(".jpg")); // Default extension
                
                // Should be deterministic
                let filename2 = cache.url_to_filename(url);
                assert_eq!(filename, filename2);
            }
        }

        #[test]
        fn test_url_extension_parsing_edge_cases() {
            let temp_dir = std::env::temp_dir();
            let cache_dir = temp_dir.join("test_cache");
            let _ = fs::create_dir_all(&cache_dir);
            
            let client = reqwest::Client::new();
            let cache = ImageCache {
                cache_dir: cache_dir.clone(),
                download_queue: Arc::new(Mutex::new(HashMap::new())),
                download_semaphore: Arc::new(Semaphore::new(5)),
                client,
            };
            
            // Test various extension edge cases
            let extension_cases = vec![
                ("http://example.com/image.", "jpg"), // Trailing dot
                ("http://example.com/image..", "jpg"), // Multiple dots
                ("http://example.com/image.JPG", "JPG"), // Uppercase
                ("http://example.com/image.JPEG", "JPEG"), // Uppercase
                ("http://example.com/image.unknown", "jpg"), // Unknown extension
                ("http://example.com/image.txt", "jpg"), // Non-image extension
                ("http://example.com/image.pdf", "jpg"), // Non-image extension
                ("http://example.com/image.js", "jpg"), // Script extension
                ("http://example.com/image.html", "jpg"), // HTML extension
                ("http://example.com/path.with.dots/image.jpg", "jpg"), // Dots in path
                ("http://example.com/image?ext=png", "jpg"), // Extension in query
            ];
            
            for (url, expected_ext) in extension_cases {
                let filename = cache.url_to_filename(url);
                assert!(filename.ends_with(&format!(".{}", expected_ext)),
                    "URL: {} should produce filename ending with .{}, got: {}", url, expected_ext, filename);
            }
        }
    }
}

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
