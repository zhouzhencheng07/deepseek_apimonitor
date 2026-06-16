use std::path::PathBuf;

fn cache_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
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

