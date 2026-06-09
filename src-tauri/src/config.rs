use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "USAGE_URL")]
    pub usage_url: String,
    #[serde(rename = "API_BASE")]
    pub api_base: String,
    #[serde(rename = "CHANNEL")]
    pub channel: String,
    #[serde(rename = "DATA_DIR")]
    pub data_dir: String,
    #[serde(rename = "REFRESH_INTERVAL")]
    pub refresh_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            usage_url: "https://platform.deepseek.com/usage".to_string(),
            api_base: "https://platform.deepseek.com/api/v0".to_string(),
            channel: "msedge".to_string(),
            data_dir: ".browser_data".to_string(),
            refresh_interval: 120,
        }
    }
}

fn cache_dir() -> PathBuf {
    let home = dirs::home_dir().expect("无法获取用户目录");
    home.join(".deepseek_monitor")
}

fn config_path() -> PathBuf {
    cache_dir().join("config.json")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("无法读取 {}: {}", path.display(), e));
            return serde_json::from_str(&content).expect("config.json 格式错误");
        }
        // 首次运行，用默认值创建
        let cfg = Config::default();
        if let Ok(content) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::create_dir_all(cache_dir());
            let _ = std::fs::write(&path, &content);
        }
        cfg
    }
}
