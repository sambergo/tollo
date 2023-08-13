use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::app::Settings;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    settings: Settings,
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct Settings {
//     player: String,
//     m3u_url: String,
//     args: Vec<String>,
// }

pub fn init_settings() -> Settings {
    let mut config_path: PathBuf = dirs::home_dir().expect("Failed to get home directory.");
    config_path.push(".config/tollo/tollo.toml");

    if !config_path.exists() {
        return create_default_settings(config_path);
    }
    // Read the TOML file into a string
    let toml_str = fs::read_to_string(config_path).expect("Failed to read file");

    // Deserialize the TOML string into the Config struct
    let config: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    config.settings
}

fn create_default_settings(config_path: PathBuf) -> Settings {
    let config = Config {
        settings: Settings {
            player: "mpv".to_string(),
            m3u_url: "https://iptv-org.github.io/iptv/index.m3u".to_string(),
            args: vec![],
        },
    };
    let toml_str = toml::to_string(&config).expect("Failed to serialize");
    let mut file = File::create(config_path).expect("Failed to create file");
    file.write_all(toml_str.as_bytes())
        .expect("Failed to write data");
    config.settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_init() {
        let settings = init_settings();
        println!("{:#?}", settings);
    }

    #[test]
    fn test_create_default_config() {
        let mut config_path: PathBuf = dirs::home_dir().expect("Failed to get home directory.");
        config_path.push(".config/tollo/testing.toml");
        let settings = create_default_settings(config_path);
        println!("{:#?}", settings);
    }
}
