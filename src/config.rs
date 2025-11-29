use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "default_repo_url")]
    pub repo_url: String,
}

fn default_repo_url() -> String {
    "https://github.com/AiYo-Studio/emod-cli.git".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repo_url: default_repo_url(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        
        Self::default()
    }

    fn config_path() -> PathBuf {
        let home = dirs::home_dir().expect("无法获取用户主目录");
        home.join(".emod-cli.json")
    }
}
