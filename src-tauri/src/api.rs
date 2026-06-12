use chrono::Local;
use serde_json::Value;

use crate::config::Config;

pub async fn api_get(path: &str, token: &str, config: &Config) -> Result<Value, String> {
    let url = format!("{}{}", config.api_base, path);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if resp.status() == 401 {
        return Err("TOKEN_INVALID".to_string());
    }
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    resp.json::<Value>().await.map_err(|e| format!("JSON 解析失败: {}", e))
}

pub async fn fetch_data(token: &str, config: &Config) -> Result<serde_json::Value, String> {
    let now = Local::now();
    let month = now.format("%m").to_string();
    let year = now.format("%Y").to_string();
    let amount = api_get(&format!("/usage/amount?month={}&year={}", month, year), token, config).await?;
    let cost = api_get(&format!("/usage/cost?month={}&year={}", month, year), token, config).await?;
    let summary = api_get("/users/get_user_summary", token, config).await?;

    Ok(serde_json::json!({
        "amount": amount,
        "cost": cost,
        "summary": summary,
    }))
}
