use crate::types::{AccountsConfig, WidgetConfig};
use std::fs;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    let new_dir = directories::ProjectDirs::from("com", "clampist", "ai-usage")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| {
            dirs_fallback().join(".config").join("ai-usage")
        });

    if new_dir.exists() {
        return new_dir;
    }

    // Migrate from pre-release bundle identifier (com.clampist.toolbox).
    if let Some(old) = directories::ProjectDirs::from("com", "clampist", "toolbox") {
        let old_dir = old.config_dir();
        if old_dir.exists() {
            return old_dir.to_path_buf();
        }
    }

    new_dir
}

fn dirs_fallback() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn accounts_config_path() -> PathBuf {
    config_dir().join("accounts.json")
}

pub fn widget_config_path() -> PathBuf {
    config_dir().join("widget.json")
}

pub fn load_accounts_config() -> AccountsConfig {
    let path = accounts_config_path();
    if !path.exists() {
        return AccountsConfig::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn load_widget_config() -> WidgetConfig {
    let path = widget_config_path();
    if !path.exists() {
        return WidgetConfig::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_widget_config(config: &WidgetConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(widget_config_path(), json).map_err(|e| e.to_string())
}

pub fn save_accounts_config(config: &AccountsConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(accounts_config_path(), json).map_err(|e| e.to_string())
}
