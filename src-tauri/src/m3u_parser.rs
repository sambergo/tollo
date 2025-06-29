use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub logo: String,
    pub url: String,
    pub group_title: String,
}

pub fn get_channels() -> Vec<Channel> {
    let m3u_content = include_str!("../data/fin.m3u");
    let mut channels = Vec::new();

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

            if let Some(url_line) = lines.next() {
                if !url_line.starts_with('#') {
                    channels.push(Channel {
                        name,
                        logo,
                        url: url_line.to_string(),
                        group_title,
                    });
                }
            }
        }
    }

    channels
}