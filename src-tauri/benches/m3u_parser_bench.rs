use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tollo_lib::m3u_parser::Channel;
use regex::Regex;

// Simple M3U parsing function for benchmarking
fn parse_m3u_content(m3u_content: &str) -> Vec<Channel> {
    let mut channels = Vec::new();
    let re_resolution = Regex::new(r"(\d+p)").unwrap();
    let re_extra_info = Regex::new(r"\[(.*?)\]").unwrap();
    let mut lines = m3u_content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("#EXTINF") {
            let name = line
                .split(',')
                .nth(1)
                .unwrap_or_default()
                .trim()
                .to_string();
            let logo = line
                .split("tvg-logo=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let group_title = line
                .split("group-title=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let tvg_id = line
                .split("tvg-id=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let resolution = re_resolution
                .captures(&name)
                .and_then(|c| c.get(1))
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string());
            let extra_info = re_extra_info
                .captures(&name)
                .and_then(|c| c.get(1))
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string());

            if let Some(url_line) = lines.next() {
                if !url_line.starts_with('#') {
                    channels.push(Channel {
                        name,
                        logo,
                        url: url_line.to_string(),
                        group_title,
                        tvg_id,
                        resolution,
                        extra_info,
                    });
                }
            }
        }
    }

    channels
}

fn parse_m3u_content_with_progress<F>(m3u_content: &str, progress_callback: F) -> Vec<Channel>
where
    F: Fn(usize, usize),
{
    let lines: Vec<&str> = m3u_content.lines().collect();
    let total_lines = lines.len();
    let mut channels = Vec::new();
    let re_resolution = Regex::new(r"(\d+p)").unwrap();
    let re_extra_info = Regex::new(r"\[(.*?)\]").unwrap();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("#EXTINF") {
            let name = line
                .split(',')
                .nth(1)
                .unwrap_or_default()
                .trim()
                .to_string();
            let logo = line
                .split("tvg-logo=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let group_title = line
                .split("group-title=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let tvg_id = line
                .split("tvg-id=\"")
                .nth(1)
                .unwrap_or_default()
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
            let resolution = re_resolution
                .captures(&name)
                .and_then(|c| c.get(1))
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string());
            let extra_info = re_extra_info
                .captures(&name)
                .and_then(|c| c.get(1))
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string());

            if i + 1 < lines.len() {
                let url_line = lines[i + 1];
                if !url_line.starts_with('#') {
                    channels.push(Channel {
                        name,
                        logo,
                        url: url_line.to_string(),
                        group_title,
                        tvg_id,
                        resolution,
                        extra_info,
                    });
                    i += 1; // Skip the URL line
                }
            }
            
            // Report progress every 100 processed lines
            if channels.len() % 100 == 0 {
                progress_callback(i, total_lines);
            }
        }
        i += 1;
    }

    channels
}

fn generate_m3u_content(channel_count: usize) -> String {
    let mut content = String::from("#EXTM3U\n");
    
    for i in 0..channel_count {
        content.push_str(&format!(
            "#EXTINF:-1 tvg-id=\"channel{}\" tvg-logo=\"http://example.com/logo{}.png\" group-title=\"Group {}\",Test Channel {} [HD] 1080p\n",
            i, i, i % 10, i
        ));
        content.push_str(&format!("http://example.com/stream{}.m3u8\n", i));
        
        // Add some VLC options occasionally to simulate real-world M3U files
        if i % 50 == 0 {
            content.push_str("#EXTVLCOPT:http-referrer=http://example.com\n");
            content.push_str("#EXTVLCOPT:http-user-agent=TestAgent\n");
        }
    }
    
    content
}

fn generate_complex_m3u_content(channel_count: usize) -> String {
    let mut content = String::from("#EXTM3U\n");
    
    let groups = [
        "Sports", "News", "Entertainment", "Movies", "Documentary", 
        "Kids", "Music", "Religious", "International", "Local"
    ];
    
    let resolutions = ["720p", "1080p", "4K", "SD"];
    let countries = ["US", "UK", "DE", "FR", "ES", "IT", "RU", "CN", "JP", "BR"];
    
    for i in 0..channel_count {
        let group = groups[i % groups.len()];
        let resolution = resolutions[i % resolutions.len()];
        let country = countries[i % countries.len()];
        
        // Create more realistic channel names with various patterns
        let channel_name = match i % 4 {
            0 => format!("{} Channel {} [{}] {}", group, i, country, resolution),
            1 => format!("{}-{} HD", group, i),
            2 => format!("Channel {} | {} | {}", i, group, resolution),
            _ => format!("{} Network {}", group, i),
        };
        
        content.push_str(&format!(
            "#EXTINF:-1 tvg-id=\"{}{}\" tvg-logo=\"http://cdn{}.example.com/logos/{}.png\" group-title=\"{}\",{}\n",
            country.to_lowercase(), i, i % 5, i, group, channel_name
        ));
        
        // Vary URL patterns
        let url = match i % 3 {
            0 => format!("http://stream{}.example.com/live/channel{}/playlist.m3u8", i % 10, i),
            1 => format!("https://live.example.com/hls/ch{}/index.m3u8?token=abc123", i),
            _ => format!("http://example.com:8080/live/{}/stream.m3u8", i),
        };
        content.push_str(&format!("{}\n", url));
        
        // Add VLC options for some channels
        if i % 25 == 0 {
            content.push_str("#EXTVLCOPT:http-referrer=http://example.com\n");
            content.push_str("#EXTVLCOPT:http-user-agent=Mozilla/5.0\n");
        }
        
        if i % 100 == 0 {
            content.push_str("#KODIPROP:inputstream.adaptive.license_type=clearkey\n");
        }
    }
    
    content
}

fn bench_m3u_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("m3u_parsing");
    
    // Benchmark different sizes of simple M3U content
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let content = generate_m3u_content(*size);
        group.bench_with_input(
            BenchmarkId::new("simple_m3u", size),
            &content,
            |b, content| {
                b.iter(|| {
                    let channels = parse_m3u_content(black_box(content));
                    black_box(channels)
                });
            },
        );
    }
    
    // Benchmark complex M3U content with realistic patterns
    for size in [100, 500, 1000, 2500].iter() {
        let content = generate_complex_m3u_content(*size);
        group.bench_with_input(
            BenchmarkId::new("complex_m3u", size),
            &content,
            |b, content| {
                b.iter(|| {
                    let channels = parse_m3u_content(black_box(content));
                    black_box(channels)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_m3u_parsing_with_progress(c: &mut Criterion) {
    let mut group = c.benchmark_group("m3u_parsing_with_progress");
    
    let content = generate_complex_m3u_content(5000);
    
    group.bench_function("with_progress_callback", |b| {
        b.iter(|| {
            let progress_callback = |processed: usize, total: usize| {
                // Simulate progress reporting overhead
                black_box((processed, total));
            };
            
            let channels = parse_m3u_content_with_progress(
                black_box(&content),
                black_box(progress_callback)
            );
            black_box(channels)
        });
    });
    
    group.bench_function("without_progress_callback", |b| {
        b.iter(|| {
            let channels = parse_m3u_content(black_box(&content));
            black_box(channels)
        });
    });
    
    group.finish();
}

fn bench_channel_deduplication(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_deduplication");
    
    // Create content with many duplicate channels
    let mut content = String::from("#EXTM3U\n");
    for i in 0..1000 {
        // Every 10th channel is a duplicate
        let channel_id = i / 10;
        content.push_str(&format!(
            "#EXTINF:-1 tvg-id=\"channel{}\" tvg-logo=\"http://example.com/logo{}.png\" group-title=\"Group\",Channel {}\n",
            channel_id, channel_id, channel_id
        ));
        content.push_str(&format!("http://example.com/stream{}.m3u8\n", channel_id));
    }
    
    group.bench_function("parse_with_duplicates", |b| {
        b.iter(|| {
            let channels = parse_m3u_content(black_box(&content));
            black_box(channels)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_m3u_parsing,
    bench_m3u_parsing_with_progress,
    bench_channel_deduplication
);
criterion_main!(benches);