use chrono::{Duration, Utc};
use regex::Regex;
use reqwest;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    let mut lines = m3u_content.lines().peekable();

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
