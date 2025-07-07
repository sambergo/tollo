use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tollo_lib::m3u_parser::Channel;
use std::sync::Arc;
use std::collections::HashMap;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

// Mock cache entry for benchmarking
#[derive(Clone)]
struct MockCacheEntry {
    query: String,
    results: Vec<Channel>,
    id: Option<i32>,
    access_count: u32,
    created_at: std::time::SystemTime,
}

impl MockCacheEntry {
    fn new(query: String, results: Vec<Channel>, id: Option<i32>) -> Self {
        Self {
            query,
            results,
            id,
            access_count: 1,
            created_at: std::time::SystemTime::now(),
        }
    }
}

// Mock image cache for benchmarking
struct MockImageCache {
    cache_dir: PathBuf,
}

impl MockImageCache {
    fn url_to_filename(&self, url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = hasher.finalize();

        // Get file extension from URL if possible
        let extension = url
            .split('?')
            .next()
            .and_then(|clean_url| clean_url.split('.').last())
            .filter(|ext| {
                matches!(
                    ext.to_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg"
                )
            })
            .unwrap_or("jpg");

        format!("{:x}.{}", hash, extension)
    }
}

#[derive(Debug, Clone)]
enum MockDownloadStatus {
    NotCached,
    Downloading,
    Cached,
    Failed(String),
}

fn create_test_channels(count: usize) -> Vec<Channel> {
    let channel_names = [
        "CNN International", "BBC World News", "ESPN Sports Center", "Discovery Channel",
        "National Geographic", "MTV Music", "Fox News", "Sky Sports", "HBO Movies",
        "Cartoon Network", "Disney Channel", "Animal Planet", "History Channel",
        "Comedy Central", "Nickelodeon", "Food Network", "Travel Channel",
        "Science Channel", "Discovery Science", "Eurosport"
    ];
    
    let groups = [
        "News", "Sports", "Entertainment", "Movies", "Documentary", "Kids", "Music"
    ];
    
    (0..count).map(|i| {
        let base_name = channel_names[i % channel_names.len()];
        let group = groups[i % groups.len()];
        let name = if i >= channel_names.len() {
            format!("{} {}", base_name, (i / channel_names.len()) + 1)
        } else {
            base_name.to_string()
        };
        
        Channel {
            name,
            logo: format!("http://example.com/logo{}.png", i),
            url: format!("http://example.com/stream{}.m3u8", i),
            group_title: group.to_string(),
            tvg_id: format!("ch{}", i),
            resolution: if i % 3 == 0 { "1080p".to_string() } else { "720p".to_string() },
            extra_info: if i % 5 == 0 { format!("[HD] Extra info {}", i) } else { "".to_string() },
        }
    }).collect()
}

fn bench_search_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_cache");
    
    let channels = create_test_channels(5000);
    
    // Benchmark cache entry creation
    group.bench_function("cache_entry_creation", |b| {
        b.iter(|| {
            let query = "news".to_string();
            let entry = MockCacheEntry::new(
                black_box(query),
                black_box(channels.clone()),
                black_box(Some(1)),
            );
            black_box(entry)
        });
    });
    
    // Benchmark cache hit performance
    group.bench_function("cache_hit_simulation", |b| {
        b.iter(|| {
            // Simulate cache hit by checking if query matches
            let query = black_box("news");
            let cached_query = "news";
            let matches = query == cached_query;
            
            if matches {
                // Simulate access count increment
                let access_count = 1;
                black_box(access_count + 1)
            } else {
                black_box(0)
            }
        });
    });
    
    // Benchmark cache expiration check
    group.bench_function("cache_expiration_check", |b| {
        b.iter(|| {
            let now = std::time::SystemTime::now();
            let created_at = now - std::time::Duration::from_secs(black_box(3600));
            let ttl = std::time::Duration::from_secs(7200);
            
            let is_expired = now.duration_since(created_at).unwrap() > ttl;
            black_box(is_expired)
        });
    });
    
    group.finish();
}

fn bench_search_cache_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_cache_scaling");
    
    // Test cache performance with different result set sizes
    for size in [100, 500, 1000, 2500, 5000].iter() {
        let channels = create_test_channels(*size);
        
        group.bench_with_input(
            BenchmarkId::new("cache_entry_clone", size),
            &channels,
            |b, channels| {
                b.iter(|| {
                    let entry = MockCacheEntry::new(
                        "test_query".to_string(),
                        channels.clone(),
                        Some(1),
                    );
                    // Simulate accessing cached results
                    black_box(&entry.results)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("cache_prefix_match", size),
            &channels,
            |b, channels| {
                b.iter(|| {
                    // Simulate prefix matching logic
                    let query = black_box("new");
                    let cached_query = "news";
                    let matches = cached_query.starts_with(query);
                    
                    if matches {
                        // Simulate filtering cached results
                        let filtered: Vec<_> = channels.iter()
                            .filter(|ch| ch.name.to_lowercase().contains(query))
                            .take(50)
                            .collect();
                        black_box(filtered)
                    } else {
                        black_box(Vec::new())
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_image_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_cache");
    
    let temp_dir = std::env::temp_dir();
    let cache_dir = temp_dir.join("bench_image_cache");
    let _ = std::fs::create_dir_all(&cache_dir);
    
    let cache = MockImageCache {
        cache_dir: cache_dir.clone(),
    };
    
    // Benchmark URL to filename conversion
    group.bench_function("url_to_filename", |b| {
        b.iter(|| {
            let url = black_box("http://example.com/logo123.png?v=1.0");
            cache.url_to_filename(url)
        });
    });
    
    // Benchmark hash computation with different URL lengths
    for url_length in [50, 100, 200, 500, 1000].iter() {
        let long_url = format!("http://example.com/{}.png", "a".repeat(*url_length));
        
        group.bench_with_input(
            BenchmarkId::new("hash_computation", url_length),
            &long_url,
            |b, url| {
                b.iter(|| {
                    cache.url_to_filename(black_box(url))
                });
            },
        );
    }
    
    // Benchmark path operations
    group.bench_function("path_construction", |b| {
        b.iter(|| {
            let filename = black_box("abc123.png");
            let path = cache_dir.join(filename);
            black_box(path)
        });
    });
    
    group.bench_function("file_existence_check", |b| {
        b.iter(|| {
            let filename = black_box("nonexistent.png");
            let path = cache_dir.join(filename);
            path.exists()
        });
    });
    
    // Clean up
    let _ = std::fs::remove_dir_all(&cache_dir);
    
    group.finish();
}

fn bench_download_status_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("download_status");
    
    // Benchmark status creation and cloning
    group.bench_function("status_creation", |b| {
        b.iter(|| {
            let statuses = vec![
                MockDownloadStatus::NotCached,
                MockDownloadStatus::Downloading,
                MockDownloadStatus::Cached,
                MockDownloadStatus::Failed(black_box("Network error".to_string())),
            ];
            black_box(statuses)
        });
    });
    
    group.bench_function("status_cloning", |b| {
        let status = MockDownloadStatus::Failed("Error message".to_string());
        b.iter(|| {
            black_box(status.clone())
        });
    });
    
    group.bench_function("status_pattern_matching", |b| {
        let statuses = vec![
            MockDownloadStatus::NotCached,
            MockDownloadStatus::Downloading,
            MockDownloadStatus::Cached,
            MockDownloadStatus::Failed("Error".to_string()),
        ];
        
        b.iter(|| {
            for status in &statuses {
                let result = match status {
                    MockDownloadStatus::NotCached => 0,
                    MockDownloadStatus::Downloading => 1,
                    MockDownloadStatus::Cached => 2,
                    MockDownloadStatus::Failed(_) => 3,
                };
                black_box(result);
            }
        });
    });
    
    group.finish();
}

fn bench_cache_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_memory");
    
    // Benchmark memory allocation patterns for different cache sizes
    for entry_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("cache_entry_allocation", entry_count),
            entry_count,
            |b, &count| {
                b.iter(|| {
                    let mut entries = Vec::new();
                    for i in 0..count {
                        let channels = create_test_channels(100);
                        let entry = MockCacheEntry::new(
                            format!("query_{}", i),
                            channels,
                            Some(i),
                        );
                        entries.push(entry);
                    }
                    black_box(entries)
                });
            },
        );
    }
    
    // Benchmark cache eviction simulation
    group.bench_function("lru_eviction_simulation", |b| {
        b.iter(|| {
            let mut access_counts = HashMap::new();
            
            // Simulate access pattern
            for i in 0..100 {
                let key = format!("key_{}", i % 20); // 20 unique keys, causing some eviction
                let count = access_counts.entry(key).or_insert(0);
                *count += 1;
            }
            
            // Find least recently used (simulate eviction)
            if access_counts.len() > 10 {
                let min_key = access_counts.iter()
                    .min_by_key(|(_, &count)| count)
                    .map(|(key, _)| key.clone());
                
                if let Some(key) = min_key {
                    access_counts.remove(&key);
                }
            }
            
            black_box(access_counts)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_search_cache_operations,
    bench_search_cache_scaling,
    bench_image_cache_operations,
    bench_download_status_operations,
    bench_cache_memory_usage
);
criterion_main!(benches);