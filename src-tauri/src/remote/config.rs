use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub port: u16,
    pub token: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8790,
            token: new_token(),
        }
    }
}

pub fn new_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

pub fn load(path: &Path) -> Result<Config, String> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let config: Config = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|_| {
        "Browser Remote configuration is invalid. Disable and enable it to reset access."
            .to_string()
    })?;
    validate(&config)?;
    Ok(config)
}

pub fn validate(config: &Config) -> Result<(), String> {
    if config.port == 0 {
        return Err("Choose a port between 1 and 65535.".into());
    }
    if config.token.len() != 64 || !config.token.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid Browser Remote access key.".into());
    }
    Ok(())
}

pub fn save(path: &Path, config: &Config) -> Result<(), String> {
    validate(config)?;
    let parent = path
        .parent()
        .ok_or("Could not find configuration directory.")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    file.write_all(&serde_json::to_vec(config).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_config_is_disabled_and_opt_in_survives_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("browser-remote.json");
        let mut config = load(&path).unwrap();
        assert!(!config.enabled);
        assert!(!path.exists());
        config.enabled = true;
        save(&path, &config).unwrap();
        let loaded = load(&path).unwrap();
        assert!(loaded.enabled);
        assert_eq!(loaded.token, config.token);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        config.token = new_token();
        save(&path, &config).unwrap();
        assert_ne!(load(&path).unwrap().token, loaded.token);
        config.enabled = false;
        config.token = new_token();
        save(&path, &config).unwrap();
        assert!(!load(&path).unwrap().enabled);
        assert_ne!(load(&path).unwrap().token, loaded.token);
    }
}
