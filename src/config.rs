use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CONFIG_DIR_NAME: &str = "zed-config-sync";
const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub github_token: String,
    pub gist_id: String,
}

impl Config {
    pub fn dir() -> PathBuf {
        let exe = std::env::current_exe().expect("Failed to get executable path");
        exe.parent()
            .expect("Executable has no parent directory")
            .join(CONFIG_DIR_NAME)
    }

    pub fn file_path() -> PathBuf {
        Self::dir().join(CONFIG_FILE_NAME)
    }

    pub fn load() -> Option<Self> {
        let path = Self::file_path();
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(Self::file_path(), content)?;
        Ok(())
    }
}
