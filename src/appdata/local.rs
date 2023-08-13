use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

use crate::app::Settings;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    settings: Settings,
}

pub fn get_local_path() -> PathBuf {
    let mut local_path: PathBuf;
    match env::var_os("XDG_DATA_HOME") {
        Some(val) => local_path = PathBuf::from(val),
        None => {
            local_path = PathBuf::from(env::var_os("HOME").expect("Failed to get home directory."));
            local_path.push(".local/share");
        }
    }
    local_path.push("tollo");
    local_path
}

fn write_prev_url(local_path: PathBuf, m3u_url: &str) {
    if let Some(parent) = local_path.parent() {
        if !parent.exists() {
            create_dir_all(parent).expect("Failed to create directory");
        }
    }
    let mut file = File::create(local_path).expect("Failed to create file");
    file.write_all(m3u_url.to_string().as_bytes())
        .expect("Failed to write data");
}

pub fn is_same_as_prev(m3u_url: &str) -> bool {
    let mut local_path = get_local_path();
    local_path.push("prev_url");
    if !local_path.exists() {
        write_prev_url(local_path, m3u_url);
        return false;
    }
    let content = fs::read_to_string(&local_path).expect("Failed to read file");
    write_prev_url(local_path, m3u_url);
    content.eq_ignore_ascii_case(m3u_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_same_as_prev() {
        is_same_as_prev("testing");
        let is_same1 = is_same_as_prev("testing");
        let is_same2 = is_same_as_prev("testing2");
        assert!(is_same1);
        assert!(!is_same2);
    }
}
