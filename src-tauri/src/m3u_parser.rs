use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use regex::Regex;

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

pub fn get_channels() -> Vec<Channel> {
    let m3u_content = include_str!("../data/fin.m3u");
    let mut channels = Vec::new();

    let re_resolution = Regex::new(r"(\\d+p)").unwrap();
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

pub fn get_groups() -> Vec<String> {
    let channels = get_channels();
    let mut groups = HashSet::new();
    for channel in channels {
        groups.insert(channel.group_title);
    }
    groups.into_iter().collect()
}