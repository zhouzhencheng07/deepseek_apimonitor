use serde_json::Value;

use crate::config::Config;

pub fn api_get(path: &str, token: &str, config: &Config) -> Result<Value, String> {
    let url = format!("{}{}", config.api_base, path);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    resp.json::<Value>().map_err(|e| format!("JSON 解析失败: {}", e))
}

pub fn fetch_data(token: &str, config: &Config) -> Result<serde_json::Value, String> {
    let amount = api_get("/usage/amount?month=6&year=2026", token, config)?;
    let cost = api_get("/usage/cost?month=6&year=2026", token, config)?;
    let summary = api_get("/users/get_user_summary", token, config)?;

    Ok(serde_json::json!({
        "amount": amount,
        "cost": cost,
        "summary": summary,
    }))
}
