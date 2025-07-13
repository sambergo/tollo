use crate::m3u_parser::Channel;
use chrono;
use dirs;
use regex;
use reqwest;
use rusqlite;
use uuid;

// Helper function to get M3U content without parsing
pub fn get_m3u_content(conn: &mut rusqlite::Connection, id: Option<i32>) -> Result<String, String> {
    let query = if let Some(list_id) = id {
        format!(
            "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE id = {}",
            list_id
        )
    } else {
        "SELECT id, source, filepath, last_fetched FROM channel_lists WHERE is_default = 1"
            .to_string()
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let id: i32 = row.get(0).map_err(|e| e.to_string())?;
        let source: String = row.get(1).map_err(|e| e.to_string())?;
        let filepath: Option<String> = row.get(2).map_err(|e| e.to_string())?;
        let last_fetched: Option<i64> = row.get(3).map_err(|e| e.to_string())?;

        let cache_duration_hours: i64 = conn
            .query_row(
                "SELECT cache_duration_hours FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(24);

        let now = chrono::Utc::now().timestamp();

        // Check if we have cached content
        if let (Some(fp), Some(lf)) = (filepath, last_fetched) {
            if now - lf < cache_duration_hours * 3600 {
                let data_dir = dirs::data_dir().unwrap().join("tollo");
                let channel_lists_dir = data_dir.join("channel_lists");
                if let Ok(content) = std::fs::read_to_string(channel_lists_dir.join(fp)) {
                    return Ok(content);
                }
            }
        }

        // Fetch from source
        if source.starts_with("http") {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
            
            let content = client
                .get(&source)
                .header("User-Agent", "Mozilla/5.0")
                .send()
                .map_err(|e| format!("Failed to fetch playlist: {}", e))?
                .text()
                .map_err(|e| format!("Failed to read response: {}", e))?;

            // Save to cache
            let data_dir = dirs::data_dir().unwrap().join("tollo");
            let channel_lists_dir = data_dir.join("channel_lists");
            let _ = std::fs::create_dir_all(&channel_lists_dir);
            let filename = format!("{}.m3u", uuid::Uuid::new_v4());
            let new_filepath = channel_lists_dir.join(&filename);
            if std::fs::write(&new_filepath, &content).is_ok() {
                let _ = conn.execute(
                    "UPDATE channel_lists SET filepath = ?1, last_fetched = ?2 WHERE id = ?3",
                    &[
                        &filename as &dyn rusqlite::ToSql,
                        &now as &dyn rusqlite::ToSql,
                        &id as &dyn rusqlite::ToSql,
                    ],
                );
            }

            return Ok(content);
        } else {
            let data_dir = dirs::data_dir().unwrap().join("tollo");
            let channel_lists_dir = data_dir.join("channel_lists");
            if let Ok(content) = std::fs::read_to_string(channel_lists_dir.join(&source)) {
                return Ok(content);
            }
        }
    }

    Err("No channel list found".to_string())
}

// Helper function to parse M3U content with progress
pub fn parse_m3u_with_progress<F>(m3u_content: &str, progress_callback: F) -> Vec<Channel>
where
    F: Fn(f32, String, usize),
{
    let mut channels = Vec::new();
    let re_resolution = regex::Regex::new(r"(\d+p)").unwrap();
    let re_extra_info = regex::Regex::new(r"\[(.*?)\]").unwrap();

    // Count total lines for progress calculation
    let total_lines = m3u_content.lines().count();
    let mut current_line = 0;
    let mut extinf_count = 0;
    let mut parsed_channels = 0;

    progress_callback(0.0, "Starting M3U parsing...".to_string(), 0);

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
