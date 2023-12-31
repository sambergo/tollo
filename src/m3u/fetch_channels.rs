use crate::appdata::local::get_local_path;
use crate::m3u::parse::parse_channels;
use std::error::Error;
use std::path::Path;
use std::result::Result;

use std::fs::{self, File};
use std::io::{prelude::*, Write};
use std::time::{Duration, SystemTime};

use reqwest::blocking::get;

use crate::app::Channel;

fn get_file_path() -> String {
    let mut local_path = get_local_path();
    local_path.push("playlist.m3u");
    local_path.to_str().unwrap().to_string()
}

fn save_to_file(file_content: &str) -> std::io::Result<()> {
    let file_path = get_file_path();
    let mut file = File::create(file_path)?;
    file.write_all(file_content.as_bytes())?;
    Ok(())
}

fn read_from_file(file_path: &str) -> std::io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Checks if the file at "~/.local/share/tollo/playlist.m3u" exists and has been modified within the last 24 hours.
pub fn check_local_playlist_status() -> bool {
    let file_path = get_file_path();
    if let Ok(metadata) = fs::metadata(file_path) {
        // Get the last modified time of the file
        if let Ok(modified_time) = metadata.modified() {
            // Get the current system time
            if let Ok(current_time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                // Calculate the time difference between the current time and the last modified time
                let time_difference = current_time
                    - modified_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or(Duration::from_secs(0));
                // Check if the time difference is less than 24 hours
                if time_difference.as_secs() < (60 * 60 * 24) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn check_is_m3u_filepath(argument: &str) -> bool {
    let path = Path::new(argument);
    if path.exists() && path.is_file() {
        if let Some(file_name) = path.file_name() {
            if let Some(extension) = file_name.to_str().and_then(|s| s.split('.').last()) {
                if extension == "m3u" || extension == "m3u8" {
                    return true;
                }
            }
        }
    }
    false
}

/// Read from file if its fresh enough, otherwise fetch from url
pub fn fetch_channels(
    m3u_url: &str,
    always_reload: bool,
    never_reload: bool,
    favorites: &[Channel],
) -> Result<Vec<Channel>, Box<dyn Error>> {
    if check_is_m3u_filepath(m3u_url) {
        let m3u_content = read_from_file(m3u_url)?;
        let channels = parse_channels(&m3u_content, favorites);
        Ok(channels)
    } else if never_reload || (check_local_playlist_status() && !always_reload) {
        let file_path = get_file_path();
        let m3u_content = read_from_file(&file_path)?;
        let channels = parse_channels(&m3u_content, favorites);
        Ok(channels)
    } else {
        let response = get(m3u_url)?;
        let m3u_content = response.text()?;
        let channels = parse_channels(&m3u_content, favorites);
        let _ = save_to_file(&m3u_content);
        Ok(channels)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        check_is_m3u_filepath, check_local_playlist_status, fetch_channels, get_file_path,
    };

    #[test]
    fn test_fetch_channels() {
        let m3u_url = "https://iptv-org.github.io/iptv/index.m3u";
        let result = fetch_channels(m3u_url, false, false, vec![].as_ref());
        if let Ok(channels) = &result {
            println!("channels len: {}", channels.len());
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_local_playlist_status() {
        let is_ok = check_local_playlist_status();
        println!("file is ok: {}", is_ok);
    }

    #[test]
    fn test_file_path() {
        let file_path = get_file_path();
        assert!(check_is_m3u_filepath(&file_path));
    }

    #[test]
    fn test_check_is_m3u_filepath_with_directory() {
        let directory = "path/to/directory/";
        assert!(!check_is_m3u_filepath(directory));
    }

    #[test]
    fn test_check_is_m3u_filepath_with_invalid_extension() {
        let filepath = "path/to/file.txt";
        assert!(!check_is_m3u_filepath(filepath));
    }

    #[test]
    fn test_check_is_m3u_filepath_with_url() {
        let url = "https://iptv-org.github.io/iptv/index.m3u";
        assert!(!check_is_m3u_filepath(url));
    }
}
