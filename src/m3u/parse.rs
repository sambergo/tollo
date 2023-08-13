use regex::Regex;

use crate::app::Channel;
pub fn parse_channels(m3u_content: &str) -> Vec<Channel> {
    let re = Regex::new(r#"EXTINF:-?1?[^\n]+\n(https?://\S+)"#).expect("Invalid regex pattern");
    let channel_text_captures: Vec<String> = re
        .captures_iter(m3u_content)
        .map(|capture| capture[0].to_string())
        .collect();

    let re_id = Regex::new(r#"tvg-id="(.*?)""#).expect("Invalid regex pattern");
    let re_tvg_name = Regex::new(r#"tvg-name="(.*?)""#).expect("Invalid regex pattern");
    let re_logo = Regex::new(r#"tvg-logo="(.*?)""#).expect("Invalid regex pattern");
    let re_group = Regex::new(r#"group-title="(.*?)""#).expect("Invalid regex pattern");
    let re_url = Regex::new(r#"EXTINF[^\n]+\n(https?://\S+)"#).expect("Invalid regex pattern");
    let re_tail_name = Regex::new(r#",([^\n]+)"#).expect("Invalid regex pattern");
    let mut channels: Vec<Channel> = vec![];

    for channel_text in channel_text_captures {
        let id: String = re_id
            .captures(&channel_text)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
            .unwrap_or_default();
        let tail_name: String = re_tail_name
            .captures(&channel_text)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
            .unwrap_or_default();
        let name: String = re_tvg_name
            .captures(&channel_text)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
            .unwrap_or(tail_name);
        let logo: String = re_logo
            .captures(&channel_text)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
            .unwrap_or_default();
        let group: String = re_group
            .captures(&channel_text)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
            .unwrap_or_default();
        let url: String = re_url
            .captures(&channel_text)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
            .unwrap_or_default();
        channels.push(Channel {
            name,
            id,
            logo,
            favorite: false,
            group,
            url,
        });
    }
    channels
}
