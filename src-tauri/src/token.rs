use std::path::PathBuf;
use crate::config::Config;

fn cache_dir() -> PathBuf {
    let home = dirs::home_dir().expect("无法获取用户目录");
    home.join(".deepseek_monitor")
}

fn token_path() -> PathBuf {
    cache_dir().join("token")
}

pub fn load_token() -> Option<String> {
    let path = token_path();
    if !path.exists() { return None; }
    let t = std::fs::read_to_string(path).ok()?.trim().to_string();
    if t.len() >= 20 { Some(t) } else { None }
}

pub fn save_token(token: &str) {
    let dir = cache_dir();
    std::fs::create_dir_all(dir).ok();
    std::fs::write(token_path(), token).ok();
}

pub fn validate_token(token: &str, config: &Config) -> bool {
    let url = format!("{}/users/get_user_summary", config.api_base);
    let client = reqwest::blocking::Client::new();
    match client.get(&url).bearer_auth(token).send() {
        Ok(resp) => {
            if let Ok(body) = resp.json::<serde_json::Value>() {
                return body["data"]["biz_data"]["normal_wallets"]
                    .as_array().map(|a| !a.is_empty()).unwrap_or(false);
            }
            false
        }
        Err(_) => false,
    }
}

pub fn extract_token(_config: &Config, _headless: bool) -> Option<String> {
    None
}
