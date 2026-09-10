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
    pub fn get_default_config_path() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.join("settings.json")))
    }

    pub fn load(custom_path: Option<&str>) -> Result<Self, String> {
        let path = if let Some(p) = custom_path {
            Some(PathBuf::from(p))
        } else {
            Self::get_default_config_path()
        };

        if let Some(path) = path {
            log::info!("加载配置文件: {}", path.display());
            if !path.exists() {
                return Err(format!("配置文件不存在: {}", path.display()));
            }
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&contents) {
                    log::info!("配置文件加载成功");
                    return Ok(config);
                } else {
                    return Err(format!("配置文件解析失败: {}", path.display()));
                }
            } else {
                return Err(format!("无法读取配置文件: {}", path.display()));
            }
        }
        Err("无法获取配置文件路径".to_string())
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::get_default_config_path()
            .ok_or_else(|| std::io::Error::other("无法获取配置文件路径"))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        log::info!("配置已保存: {}", path.display());
        fs::write(&path, json)?;
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