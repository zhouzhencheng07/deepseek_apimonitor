use serde::Deserialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
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

impl Config {
    pub fn load() -> Self {
        // 从 exe 路径往上找 config.json（适配 dev/release 不同目录深度）
        let exe_dir = std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();

        let mut candidates = vec![
            PathBuf::from("config.json"),
            PathBuf::from("../config.json"),
        ];

        // 从 exe 目录往上找 5 层
        let mut d = exe_dir.clone();
        for _ in 0..5 {
            candidates.push(d.join("config.json"));
            if let Some(parent) = d.parent() {
                d = parent.to_path_buf();
            }
        }

        for p in &candidates {
            if p.exists() {
                let content = std::fs::read_to_string(p)
                    .unwrap_or_else(|_| panic!("无法读取 config.json: {:?}", p));
                return serde_json::from_str(&content).expect("config.json 格式错误");
            }
        }
        panic!("找不到 config.json，请确保 config.json 在项目根目录\n搜索路径: {:?}", candidates);
    }
}
