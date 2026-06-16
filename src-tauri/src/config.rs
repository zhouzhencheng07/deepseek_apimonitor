use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "USAGE_URL")]
    pub usage_url: String,
    #[serde(rename = "API_BASE")]
    pub api_base: String,
    #[serde(rename = "REFRESH_INTERVAL")]
    pub refresh_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            usage_url: "https://platform.deepseek.com/usage".to_string(),
            api_base: "https://platform.deepseek.com/api/v0".to_string(),
            refresh_interval: 120,
        }
    }
}

fn cache_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
    home.join(".deepseek_monitor")
}

fn config_path() -> PathBuf {
    cache_dir().join("config.json")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                // 读不了（权限/编码等）回退默认值，不让配置问题搞崩启动。
                Err(_) => return Config::default(),
            };
            // 解析失败（用户手改坏 JSON）也回退默认值，与 endpoints 行为一致。
            let cfg = serde_json::from_str::<Config>(&content).unwrap_or_default();
            // 补齐缺失字段并回写，和老用户升级体验对齐。
            persist_if_changed(&path, &content, &cfg);
            return cfg;
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

// 仅当序列化结果与磁盘读入的不一致（字段缺失被 default 补齐、或用户手改过格式）
// 才回写，避免每次启动都做无谓 IO。
fn persist_if_changed(path: &std::path::Path, old_content: &str, cfg: &Config) {
    let Ok(new_content) = serde_json::to_string_pretty(cfg) else { return };
    if new_content == old_content { return; }
    let _ = std::fs::create_dir_all(cache_dir());
    let _ = std::fs::write(path, new_content);
}
