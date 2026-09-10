use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core::AwakeMode;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub properties: ConfigProperties,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigProperties {
    #[serde(default)]
    pub keep_display_on: bool,
    #[serde(default)]
    pub mode: u8,
    #[serde(default)]
    pub interval_hours: u32,
    #[serde(default)]
    pub interval_minutes: u32,
    #[serde(default)]
    pub expiration_datetime: Option<String>,
    #[serde(default)]
    pub custom_tray_times: HashMap<String, u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            properties: ConfigProperties {
                keep_display_on: false,
                mode: 0,
                interval_hours: 0,
                interval_minutes: 0,
                expiration_datetime: None,
                custom_tray_times: HashMap::new(),
            },
            name: "Awake".to_string(),
            version: "1.0".to_string(),
        }
    }
}

impl Config {
    pub fn get_config_path() -> Option<PathBuf> {
        dirs::config_local_dir().map(|path| path.join("aweak").join("settings.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::get_config_path() {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&contents) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(path) = Self::get_config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)?;
            fs::write(path, json)?;
        }
        Ok(())
    }

    pub fn mode_from_u8(value: u8) -> AwakeMode {
        match value {
            0 => AwakeMode::Passive,
            1 => AwakeMode::Indefinite,
            2 => AwakeMode::Timed,
            3 => AwakeMode::Expirable,
            _ => AwakeMode::Passive,
        }
    }

    pub fn mode_to_u8(mode: AwakeMode) -> u8 {
        match mode {
            AwakeMode::Passive => 0,
            AwakeMode::Indefinite => 1,
            AwakeMode::Timed => 2,
            AwakeMode::Expirable => 3,
        }
    }
}