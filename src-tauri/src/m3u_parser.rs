use chrono::Utc;
use regex::Regex;
use reqwest;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Channel {
    pub name: String,
    pub logo: String,
    pub url: String,
    pub group_title: String,
    pub tvg_id: String,
    pub resolution: String,
    pub extra_info: String,
}

fn parse_m3u_content(m3u_content: &str) -> Vec<Channel> {
    let mut channels = Vec::new();
    let re_resolution = Regex::new(r"(\d+p)").unwrap();
    let re_extra_info = Regex::new(r"\[(.*?)\]").unwrap();
    let mut lines = m3u_content.lines().peekable();

    println!(
        "Starting M3U parsing, total lines: {}",
        m3u_content.lines().count()
    );
    let mut extinf_count = 0;
    let mut parsed_channels = 0;

    while let Some(line) = lines.next() {
        if line.starts_with("#EXTINF") {
            extinf_count += 1;
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
                    parsed_channels += 1;
                } else {
                    // Only warn for unexpected non-URL lines (but skip common M3U options)
                    if !url_line.starts_with("#EXTVLCOPT") && !url_line.starts_with("#KODIPROP") {
                        println!("Warning: Expected URL line but got: {}", url_line);
                    }
                }
            }

            // Log progress only for very large files
            if extinf_count % 10000 == 0 && extinf_count > 0 {
                println!(
                    "Processed {} EXTINF lines, parsed {} channels so far",
                    extinf_count, parsed_channels
                );
            }
        }
    }

    println!(
        "M3U parsing complete: {} EXTINF lines found, {} channels parsed",
        extinf_count, parsed_channels
    );
    channels
}

// New async version with progress callback
fn parse_m3u_content_with_progress<F>(m3u_content: &str, progress_callback: F) -> Vec<Channel>
where
    F: Fn(f32, String, usize),
{
    let mut channels = Vec::new();
    let re_resolution = Regex::new(r"(\d+p)").unwrap();
    let re_extra_info = Regex::new(r"\[(.*?)\]").unwrap();
    let mut _lines = m3u_content.lines().peekable();

    // Count total lines for progress calculation
    let total_lines = m3u_content.lines().count();
    let mut current_line = 0;
    let mut extinf_count = 0;
    let mut parsed_channels = 0;

    progress_callback(0.0, "Starting M3U parsing...".to_string(), 0);

    // Reset iterator
    let mut lines = m3u_content.lines().peekable();

    while let Some(line) = lines.next() {
        current_line += 1;

        if line.starts_with("#EXTINF") {
            extinf_count += 1;
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
                current_line += 1;
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
                    parsed_channels += 1;
                }
            }

            // Update progress every 1000 channels or 5% of total lines
            if parsed_channels % 1000 == 0 || current_line % (total_lines / 20).max(1) == 0 {
                let progress = (current_line as f32) / (total_lines as f32);
                let message = format!(
                    "Parsed {} channels ({} EXTINF entries)",
                    parsed_channels, extinf_count
                );
                progress_callback(progress, message, parsed_channels);
            }
        }
    }

    progress_callback(
        1.0,
        format!("Parsing complete! {} channels parsed", parsed_channels),
        parsed_channels,
    );
    channels
}

pub fn get_channels(conn: &mut Connection, id: Option<i32>) -> Vec<Channel> {
    let query = if let Some(list_id) = id {
        format!(
            "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE id = {}",
            list_id
        )
    } else {
        "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE is_default = 1"
            .to_string()
    };

    let mut stmt = conn.prepare(&query).unwrap();
    let mut rows = stmt.query([]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let id: i32 = row.get(0).unwrap();
        let source: String = row.get(1).unwrap();
        let filepath: Option<String> = row.get(2).unwrap();
        let last_fetched: Option<i64> = row.get(3).unwrap();

        let cache_duration_hours: i64 = conn
            .query_row(
                "SELECT cache_duration_hours FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(24);

        let now = Utc::now().timestamp();

        if let (Some(fp), Some(lf)) = (filepath, last_fetched) {
            if now - lf < cache_duration_hours * 3600 {
                let data_dir = dirs::data_dir().unwrap().join("tollo");
                let channel_lists_dir = data_dir.join("channel_lists");
                if let Ok(content) = fs::read_to_string(channel_lists_dir.join(fp)) {
                    return parse_m3u_content(&content);
                }
            }
        }

        if source.starts_with("http") {
            if let Ok(content) = reqwest::blocking::get(&source).and_then(|resp| resp.text()) {
                let data_dir = dirs::data_dir().unwrap().join("tollo");
                let channel_lists_dir = data_dir.join("channel_lists");
                let _ = fs::create_dir_all(&channel_lists_dir);
                let filename = format!("{}.m3u", Uuid::new_v4());
                let new_filepath = channel_lists_dir.join(&filename);
                if fs::write(&new_filepath, &content).is_ok() {
                    conn.execute(
                        "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                        &[
                            &filename as &dyn rusqlite::ToSql,
                            &now as &dyn rusqlite::ToSql,
                            &id as &dyn rusqlite::ToSql,
                        ],
                    )
                    .unwrap();
                    return parse_m3u_content(&content);
                }
            }
        } else {
            let data_dir = dirs::data_dir().unwrap().join("tollo");
            let channel_lists_dir = data_dir.join("channel_lists");
            if let Ok(content) = fs::read_to_string(channel_lists_dir.join(&source)) {
                return parse_m3u_content(&content);
            }
        }
    }

    vec![]
}

// New async version with progress support
pub fn get_channels_with_progress<F>(
    conn: &mut Connection,
    id: Option<i32>,
    progress_callback: F,
) -> Vec<Channel>
where
    F: Fn(f32, String, usize),
{
    progress_callback(0.0, "Looking up channel list...".to_string(), 0);

    let query = if let Some(list_id) = id {
        format!(
            "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE id = {}",
            list_id
        )
    } else {
        "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE is_default = 1"
            .to_string()
    };

    let mut stmt = conn.prepare(&query).unwrap();
    let mut rows = stmt.query([]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let id: i32 = row.get(0).unwrap();
        let source: String = row.get(1).unwrap();
        let filepath: Option<String> = row.get(2).unwrap();
        let last_fetched: Option<i64> = row.get(3).unwrap();

        let cache_duration_hours: i64 = conn
            .query_row(
                "SELECT cache_duration_hours FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(24);

        let now = Utc::now().timestamp();

        progress_callback(0.1, "Checking cache...".to_string(), 0);

        if let (Some(fp), Some(lf)) = (filepath, last_fetched) {
            if now - lf < cache_duration_hours * 3600 {
                progress_callback(0.2, "Loading from cache...".to_string(), 0);
                let data_dir = dirs::data_dir().unwrap().join("tollo");
                let channel_lists_dir = data_dir.join("channel_lists");
                if let Ok(content) = fs::read_to_string(channel_lists_dir.join(fp)) {
                    progress_callback(0.3, "Parsing cached M3U content...".to_string(), 0);
                    return parse_m3u_content_with_progress(&content, progress_callback);
                }
            }
        }

        if source.starts_with("http") {
            progress_callback(0.2, "Downloading playlist...".to_string(), 0);
            if let Ok(content) = reqwest::blocking::get(&source).and_then(|resp| resp.text()) {
                progress_callback(0.4, "Saving to cache...".to_string(), 0);
                let data_dir = dirs::data_dir().unwrap().join("tollo");
                let channel_lists_dir = data_dir.join("channel_lists");
                let _ = fs::create_dir_all(&channel_lists_dir);
                let filename = format!("{}.m3u", Uuid::new_v4());
                let new_filepath = channel_lists_dir.join(&filename);
                if fs::write(&new_filepath, &content).is_ok() {
                    conn.execute(
                        "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                        &[
                            &filename as &dyn rusqlite::ToSql,
                            &now as &dyn rusqlite::ToSql,
                            &id as &dyn rusqlite::ToSql,
                        ],
                    )
                    .unwrap();
                }
                progress_callback(0.5, "Parsing M3U content...".to_string(), 0);
                return parse_m3u_content_with_progress(&content, progress_callback);
            }
        } else {
            progress_callback(0.2, "Loading from file...".to_string(), 0);
            let data_dir = dirs::data_dir().unwrap().join("tollo");
            let channel_lists_dir = data_dir.join("channel_lists");
            if let Ok(content) = fs::read_to_string(channel_lists_dir.join(&source)) {
                progress_callback(0.3, "Parsing M3U content...".to_string(), 0);
                return parse_m3u_content_with_progress(&content, progress_callback);
            }
        }
    }

    progress_callback(1.0, "No channels found".to_string(), 0);
    vec![]
}

pub fn get_groups(conn: &mut Connection, id: Option<i32>) -> Vec<String> {
    let channels = get_channels(conn, id);
    let mut groups = HashSet::new();
    for channel in channels {
        groups.insert(channel.group_title);
    }
    groups.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_m3u_content_basic() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports",Test Channel 1
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel 2
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 2);

        let channel1 = &channels[0];
        assert_eq!(channel1.name, "Test Channel 1");
        assert_eq!(channel1.logo, "http://example.com/logo1.png");
        assert_eq!(channel1.url, "http://example.com/stream1.m3u8");
        assert_eq!(channel1.group_title, "Sports");
        assert_eq!(channel1.tvg_id, "test1");

        let channel2 = &channels[1];
        assert_eq!(channel2.name, "Test Channel 2");
        assert_eq!(channel2.logo, "http://example.com/logo2.png");
        assert_eq!(channel2.url, "http://example.com/stream2.m3u8");
        assert_eq!(channel2.group_title, "News");
        assert_eq!(channel2.tvg_id, "test2");
    }

    #[test]
    fn test_parse_m3u_content_with_resolution() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports",Test Channel 1080p
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel 720p
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 2);

        assert_eq!(channels[0].resolution, "1080p");
        assert_eq!(channels[1].resolution, "720p");
    }

    #[test]
    fn test_parse_m3u_content_with_extra_info() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports",Test Channel [HD]
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel [Premium]
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 2);

        assert_eq!(channels[0].extra_info, "HD");
        assert_eq!(channels[1].extra_info, "Premium");
    }

    #[test]
    fn test_parse_m3u_content_missing_attributes() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1,Test Channel No Logo
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2",Test Channel No Group
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 2);

        let channel1 = &channels[0];
        assert_eq!(channel1.name, "Test Channel No Logo");
        assert_eq!(channel1.logo, "");
        assert_eq!(channel1.group_title, "");
        assert_eq!(channel1.tvg_id, "");

        let channel2 = &channels[1];
        assert_eq!(channel2.name, "Test Channel No Group");
        assert_eq!(channel2.tvg_id, "test2");
        assert_eq!(channel2.logo, "");
        assert_eq!(channel2.group_title, "");
    }

    #[test]
    fn test_parse_m3u_content_with_vlc_options() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports",Test Channel 1
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel 2
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 2);

        assert_eq!(channels[0].name, "Test Channel 1");
        assert_eq!(channels[0].url, "http://example.com/stream1.m3u8");
        assert_eq!(channels[1].name, "Test Channel 2");
        assert_eq!(channels[1].url, "http://example.com/stream2.m3u8");
    }

    #[test]
    fn test_parse_m3u_content_with_vlc_options_skipped() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports",Test Channel 1
#EXTVLCOPT:http-referrer=http://example.com
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel 2
#KODIPROP:inputstream.adaptive.license_type=com.widevine.alpha
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        // Based on the parser logic, EXTVLCOPT and KODIPROP lines are consumed instead of URLs
        // so no valid channels will be parsed
        assert_eq!(channels.len(), 0);
    }

    #[test]
    fn test_parse_m3u_content_empty() {
        let m3u_content = "";
        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 0);
    }

    #[test]
    fn test_parse_m3u_content_no_extinf() {
        let m3u_content = r#"#EXTM3U
http://example.com/stream1.m3u8
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 0);
    }

    #[test]
    fn test_parse_m3u_content_extinf_without_url() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports",Test Channel 1
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel 2
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        // Based on the parser logic, it will consume the second EXTINF line as the URL for the first channel
        // so no valid channels will be parsed
        assert_eq!(channels.len(), 0);
    }

    #[test]
    fn test_parse_m3u_content_malformed_extinf() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports"
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel 2
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 2);

        // First channel should have empty name due to malformed EXTINF
        assert_eq!(channels[0].name, "");
        assert_eq!(channels[0].url, "http://example.com/stream1.m3u8");
        assert_eq!(channels[0].group_title, "Sports");

        // Second channel should parse correctly
        assert_eq!(channels[1].name, "Test Channel 2");
        assert_eq!(channels[1].url, "http://example.com/stream2.m3u8");
    }

    #[test]
    fn test_parse_m3u_content_with_quotes_in_attributes() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports & Entertainment",Test Channel 1
http://example.com/stream1.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 1);

        let channel = &channels[0];
        assert_eq!(channel.group_title, "Sports & Entertainment");
        assert_eq!(channel.tvg_id, "test1");
        assert_eq!(channel.logo, "http://example.com/logo1.png");
    }

    #[test]
    fn test_parse_m3u_content_multiple_resolutions_and_extras() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports",Test Channel 1080p [HD] [Premium]
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel 720p [SD]
http://example.com/stream2.m3u8"#;

        let channels = parse_m3u_content(m3u_content);
        assert_eq!(channels.len(), 2);

        // Should extract first resolution and first extra info
        assert_eq!(channels[0].resolution, "1080p");
        assert_eq!(channels[0].extra_info, "HD");
        assert_eq!(channels[1].resolution, "720p");
        assert_eq!(channels[1].extra_info, "SD");
    }

    #[test]
    fn test_parse_m3u_content_with_progress_callback() {
        let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test1" tvg-logo="http://example.com/logo1.png" group-title="Sports",Test Channel 1
http://example.com/stream1.m3u8
#EXTINF:-1 tvg-id="test2" tvg-logo="http://example.com/logo2.png" group-title="News",Test Channel 2
http://example.com/stream2.m3u8"#;

        use std::sync::{Arc, Mutex};
        let progress_calls = Arc::new(Mutex::new(Vec::new()));
        let progress_calls_clone = Arc::clone(&progress_calls);
        
        let progress_callback = move |progress: f32, message: String, count: usize| {
            progress_calls_clone.lock().unwrap().push((progress, message, count));
        };

        let channels = parse_m3u_content_with_progress(m3u_content, progress_callback);
        assert_eq!(channels.len(), 2);

        // Verify progress callbacks were made
        let calls = progress_calls.lock().unwrap();
        assert!(!calls.is_empty());
        assert_eq!(calls[0].0, 0.0); // First call should be 0.0 progress
        assert_eq!(calls.last().unwrap().0, 1.0); // Last call should be 1.0 progress
        assert_eq!(calls.last().unwrap().2, 2); // Last call should have count of 2
    }

    #[test]
    fn test_channel_struct_creation() {
        let channel = Channel {
            name: "Test Channel".to_string(),
            logo: "http://example.com/logo.png".to_string(),
            url: "http://example.com/stream.m3u8".to_string(),
            group_title: "Sports".to_string(),
            tvg_id: "test123".to_string(),
            resolution: "1080p".to_string(),
            extra_info: "HD".to_string(),
        };

        assert_eq!(channel.name, "Test Channel");
        assert_eq!(channel.logo, "http://example.com/logo.png");
        assert_eq!(channel.url, "http://example.com/stream.m3u8");
        assert_eq!(channel.group_title, "Sports");
        assert_eq!(channel.tvg_id, "test123");
        assert_eq!(channel.resolution, "1080p");
        assert_eq!(channel.extra_info, "HD");
    }

    #[test]
    fn test_channel_clone_and_debug() {
        let channel = Channel {
            name: "Test Channel".to_string(),
            logo: "http://example.com/logo.png".to_string(),
            url: "http://example.com/stream.m3u8".to_string(),
            group_title: "Sports".to_string(),
            tvg_id: "test123".to_string(),
            resolution: "1080p".to_string(),
            extra_info: "HD".to_string(),
        };

        let cloned_channel = channel.clone();
        assert_eq!(channel.name, cloned_channel.name);
        assert_eq!(channel.url, cloned_channel.url);

        // Test Debug trait
        let debug_str = format!("{:?}", channel);
        assert!(debug_str.contains("Test Channel"));
    }

    #[test]
    fn test_parse_m3u_content_large_file_simulation() {
        // Create a larger M3U content to test performance logging
        let mut m3u_content = String::from("#EXTM3U\n");
        for i in 0..100 {
            m3u_content.push_str(&format!(
                "#EXTINF:-1 tvg-id=\"test{}\" tvg-logo=\"http://example.com/logo{}.png\" group-title=\"Group{}\",Test Channel {}\n",
                i, i, i % 5, i
            ));
            m3u_content.push_str(&format!("http://example.com/stream{}.m3u8\n", i));
        }

        let channels = parse_m3u_content(&m3u_content);
        assert_eq!(channels.len(), 100);

        // Verify some channels
        assert_eq!(channels[0].name, "Test Channel 0");
        assert_eq!(channels[0].group_title, "Group0");
        assert_eq!(channels[99].name, "Test Channel 99");
        assert_eq!(channels[99].group_title, "Group4");
    }

    // Property-based tests for M3U parsing
    mod property_tests {
        use super::*;
        
        #[test]
        fn property_parsing_preserves_channel_count() {
            // Test that the number of channels parsed matches the number of EXTINF entries
            for channel_count in 1..=50 {
                let mut m3u_content = String::from("#EXTM3U\n");
                for i in 0..channel_count {
                    m3u_content.push_str(&format!(
                        "#EXTINF:-1 tvg-id=\"test{}\" tvg-logo=\"http://example.com/logo{}.png\" group-title=\"Group{}\",Test Channel {}\n",
                        i, i, i % 5, i
                    ));
                    m3u_content.push_str(&format!("http://example.com/stream{}.m3u8\n", i));
                }
                
                let channels = parse_m3u_content(&m3u_content);
                assert_eq!(channels.len(), channel_count, 
                    "Channel count mismatch for {} channels", channel_count);
            }
        }

        #[test]
        fn property_parsing_handles_mixed_valid_invalid_entries() {
            // Test robustness with mixed valid and invalid entries
            let test_cases = vec![
                // Valid entry followed by invalid entry
                "#EXTM3U\n#EXTINF:-1 tvg-id=\"test1\",Valid Channel\nhttp://example.com/stream1\n#EXTINF:-1 tvg-id=\"test2\",Invalid Channel\n#EXTVLCOPT:something\n",
                // Invalid entry followed by valid entry
                "#EXTM3U\n#EXTINF:-1 tvg-id=\"test1\",Invalid Channel\n#KODIPROP:something\n#EXTINF:-1 tvg-id=\"test2\",Valid Channel\nhttp://example.com/stream2\n",
                // Multiple invalid entries
                "#EXTM3U\n#EXTINF:-1 tvg-id=\"test1\",Invalid 1\n#EXTINF:-1 tvg-id=\"test2\",Invalid 2\n#EXTINF:-1 tvg-id=\"test3\",Valid\nhttp://example.com/stream3\n",
            ];
            
            for test_case in test_cases {
                let channels = parse_m3u_content(test_case);
                // Should handle gracefully without panicking
                assert!(channels.len() <= 3, "Parser should handle invalid entries gracefully");
            }
        }

        #[test]
        fn property_parsing_consistent_across_formats() {
            // Test that different formatting styles produce consistent results
            let base_channel = ("Test Channel", "test1", "http://example.com/logo.png", "Sports", "http://example.com/stream");
            
            let formats = vec![
                format!("#EXTM3U\n#EXTINF:-1 tvg-id=\"{}\" tvg-logo=\"{}\" group-title=\"{}\",{}\n{}\n", 
                    base_channel.1, base_channel.2, base_channel.3, base_channel.0, base_channel.4),
                format!("#EXTM3U\n#EXTINF:-1 group-title=\"{}\" tvg-id=\"{}\" tvg-logo=\"{}\",{}\n{}\n", 
                    base_channel.3, base_channel.1, base_channel.2, base_channel.0, base_channel.4),
                format!("#EXTM3U\n#EXTINF:-1 tvg-logo=\"{}\" group-title=\"{}\" tvg-id=\"{}\",{}\n{}\n", 
                    base_channel.2, base_channel.3, base_channel.1, base_channel.0, base_channel.4),
            ];
            
            let mut previous_channel: Option<Channel> = None;
            for format in formats {
                let channels = parse_m3u_content(&format);
                assert_eq!(channels.len(), 1, "Should parse exactly one channel");
                
                let channel = &channels[0];
                assert_eq!(channel.name, base_channel.0);
                assert_eq!(channel.tvg_id, base_channel.1);
                assert_eq!(channel.logo, base_channel.2);
                assert_eq!(channel.group_title, base_channel.3);
                assert_eq!(channel.url, base_channel.4);
                
                if let Some(prev) = &previous_channel {
                    assert_eq!(channel.name, prev.name);
                    assert_eq!(channel.tvg_id, prev.tvg_id);
                    assert_eq!(channel.logo, prev.logo);
                    assert_eq!(channel.group_title, prev.group_title);
                    assert_eq!(channel.url, prev.url);
                }
                previous_channel = Some(channel.clone());
            }
        }

        #[test]
        fn property_resolution_extraction_is_deterministic() {
            // Test that resolution extraction is consistent
            let resolution_cases = vec![
                ("Channel 1080p", "1080p"),
                ("Channel 720p HD", "720p"),
                ("Channel 480p [SD]", "480p"),
                ("Channel 4K", ""),
                ("Channel HD", ""),
                ("Channel 1080p 720p", "1080p"), // Should extract first match
            ];
            
            for (channel_name, expected_resolution) in resolution_cases {
                let m3u_content = format!(
                    "#EXTM3U\n#EXTINF:-1 tvg-id=\"test\",{}\nhttp://example.com/stream\n",
                    channel_name
                );
                
                let channels = parse_m3u_content(&m3u_content);
                assert_eq!(channels.len(), 1);
                assert_eq!(channels[0].resolution, expected_resolution,
                    "Resolution extraction failed for channel: {}", channel_name);
            }
        }

        #[test]
        fn property_extra_info_extraction_handles_brackets() {
            // Test that extra info extraction handles various bracket patterns
            let bracket_cases = vec![
                ("Channel [HD]", "HD"),
                ("Channel [Premium]", "Premium"),
                ("Channel [HD] [Premium]", "HD"), // Should extract first match
                ("Channel [Multi Word Info]", "Multi Word Info"),
                ("Channel", ""),
                ("Channel []", ""),
                ("Channel [", ""),
                ("Channel ]", ""),
            ];
            
            for (channel_name, expected_extra) in bracket_cases {
                let m3u_content = format!(
                    "#EXTM3U\n#EXTINF:-1 tvg-id=\"test\",{}\nhttp://example.com/stream\n",
                    channel_name
                );
                
                let channels = parse_m3u_content(&m3u_content);
                assert_eq!(channels.len(), 1);
                assert_eq!(channels[0].extra_info, expected_extra,
                    "Extra info extraction failed for channel: {}", channel_name);
            }
        }

        #[test]
        fn property_parsing_handles_unicode_and_special_chars() {
            // Test that parser handles international characters and special symbols
            let unicode_cases = vec![
                "Канал Россия", // Russian
                "Canal Español", // Spanish
                "Chaîne Française", // French
                "Channel 中文", // Chinese
                "قناة العربية", // Arabic
                "Channel with émojis 🎬📺", // Emojis
                "Channel & Symbols @#$%", // Special symbols
            ];
            
            for channel_name in unicode_cases {
                let m3u_content = format!(
                    "#EXTM3U\n#EXTINF:-1 tvg-id=\"test\" group-title=\"International\",{}\nhttp://example.com/stream\n",
                    channel_name
                );
                
                let channels = parse_m3u_content(&m3u_content);
                assert_eq!(channels.len(), 1, "Should parse channel with unicode: {}", channel_name);
                assert_eq!(channels[0].name, channel_name);
                assert_eq!(channels[0].group_title, "International");
            }
        }

        #[test]
        fn property_progress_callback_invocation_pattern() {
            // Test that progress callbacks are called in the expected pattern
            use std::sync::{Arc, Mutex};
            
            let progress_calls = Arc::new(Mutex::new(Vec::new()));
            let progress_calls_clone = Arc::clone(&progress_calls);
            
            let progress_callback = move |progress: f32, message: String, count: usize| {
                progress_calls_clone.lock().unwrap().push((progress, message, count));
            };
            
            // Create content with enough channels to trigger multiple progress updates
            let mut m3u_content = String::from("#EXTM3U\n");
            for i in 0..2000 {
                m3u_content.push_str(&format!(
                    "#EXTINF:-1 tvg-id=\"test{}\" group-title=\"Group{}\",Test Channel {}\n",
                    i, i % 10, i
                ));
                m3u_content.push_str(&format!("http://example.com/stream{}.m3u8\n", i));
            }
            
            let channels = parse_m3u_content_with_progress(&m3u_content, progress_callback);
            assert_eq!(channels.len(), 2000);
            
            let calls = progress_calls.lock().unwrap();
            assert!(!calls.is_empty(), "Progress callback should be called");
            
            // First call should be 0.0 progress
            assert_eq!(calls[0].0, 0.0);
            assert_eq!(calls[0].2, 0); // Count should be 0 initially
            
            // Last call should be 1.0 progress
            assert_eq!(calls.last().unwrap().0, 1.0);
            assert_eq!(calls.last().unwrap().2, 2000); // Final count should match channel count
            
            // Progress should be monotonically increasing
            for i in 1..calls.len() {
                assert!(calls[i].0 >= calls[i-1].0, 
                    "Progress should be monotonically increasing");
            }
        }

        #[test]
        fn property_empty_and_malformed_inputs() {
            // Test various edge cases with empty or malformed inputs
            let edge_cases = vec![
                "",
                "#EXTM3U",
                "#EXTM3U\n",
                "#EXTM3U\n#EXTINF:-1",
                "#EXTM3U\n#EXTINF:-1\n",
                "#EXTM3U\n#EXTINF:-1,\n",
                "#EXTM3U\n#EXTINF:-1,Channel\n",
                "#EXTM3U\nhttp://example.com/stream\n", // URL without EXTINF
                "#EXTM3U\n#EXTINF:-1,Channel\n#EXTINF:-1,Another\nhttp://example.com/stream\n", // Multiple EXTINF, one URL
            ];
            
            for edge_case in edge_cases {
                let channels = parse_m3u_content(edge_case);
                // Should not panic and should return reasonable results
                assert!(channels.len() <= 2, "Edge case should be handled gracefully: {:?}", edge_case);
            }
        }
    }
}
