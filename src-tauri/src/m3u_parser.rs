use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use regex::Regex;
use rusqlite::Connection;
use std::fs;
use reqwest;
use chrono::{Utc, Duration};
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

    while let Some(line) = lines.next() {
        if line.starts_with("#EXTINF") {
            let name = line.split(',').nth(1).unwrap_or_default().trim().to_string();
            let logo = line.split("tvg-logo=\"").nth(1).unwrap_or_default().split('"').next().unwrap_or_default().to_string();
            let group_title = line.split("group-title=\"").nth(1).unwrap_or_default().split('"').next().unwrap_or_default().to_string();
            let tvg_id = line.split("tvg-id=\"").nth(1).unwrap_or_default().split('"').next().unwrap_or_default().to_string();
            let resolution = re_resolution.captures(&name).and_then(|c| c.get(1)).map_or_else(|| "".to_string(), |m| m.as_str().to_string());
            let extra_info = re_extra_info.captures(&name).and_then(|c| c.get(1)).map_or_else(|| "".to_string(), |m| m.as_str().to_string());

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

pub fn get_channels(conn: &mut Connection, id: Option<i32>) -> Vec<Channel> {
    let query = if let Some(list_id) = id {
        format!("SELECT id, source, filepath, last_fetched FROM channel_lists WHERE id = {}", list_id)
    } else {
        "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE is_default = 1".to_string()
    };

    let mut stmt = conn.prepare(&query).unwrap();
    let mut rows = stmt.query([]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let id: i32 = row.get(0).unwrap();
        let source: String = row.get(1).unwrap();
        let filepath: Option<String> = row.get(2).unwrap();
        let last_fetched: Option<i64> = row.get(3).unwrap();

        let cache_duration_hours: i64 = conn.query_row(
            "SELECT cache_duration_hours FROM settings WHERE id = 1",
            [],
            |row| row.get(0),
        ).unwrap_or(24);

        let now = Utc::now().timestamp();

        if let (Some(fp), Some(lf)) = (filepath, last_fetched) {
            if now - lf < cache_duration_hours * 3600 {
                let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
                if let Ok(content) = fs::read_to_string(data_dir.join(fp)) {
                    return parse_m3u_content(&content);
                }
            }
        }

        if source.starts_with("http") {
            if let Ok(content) = reqwest::blocking::get(&source).and_then(|resp| resp.text()) {
                let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
                let filename = format!("{}.m3u", Uuid::new_v4());
                let new_filepath = data_dir.join(&filename);
                if fs::write(&new_filepath, &content).is_ok() {
                    conn.execute(
                        "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                        &[&filename as &dyn rusqlite::ToSql, &now as &dyn rusqlite::ToSql, &id as &dyn rusqlite::ToSql],
                    ).unwrap();
                    return parse_m3u_content(&content);
                }
            }
        } else {
            let data_dir = dirs::data_dir().unwrap().join("gui-tollo");
            if let Ok(content) = fs::read_to_string(data_dir.join(&source)) {
                return parse_m3u_content(&content);
            }
        }
    }

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

