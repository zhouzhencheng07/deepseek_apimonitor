use chrono::Local;
use serde_json::Value;

use crate::config::Config;
use crate::endpoints::Endpoints;

pub async fn api_get(client: &reqwest::Client, path: &str, token: &str, config: &Config) -> Result<Value, String> {
    let url = format!("{}{}", config.api_base, path);
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

pub async fn fetch_data(client: &reqwest::Client, token: &str, config: &Config, endpoints: &Endpoints) -> Result<serde_json::Value, String> {
    let now = Local::now();
    let month = now.format("%m").to_string();
    let year = now.format("%Y").to_string();
    let amount = api_get(client, &endpoints.fill(&endpoints.amount_path, &month, &year), token, config).await?;
    let cost = api_get(client, &endpoints.fill(&endpoints.cost_path, &month, &year), token, config).await?;
    let summary = api_get(client, &endpoints.summary_path, token, config).await?;

    Ok(serde_json::json!({
        "amount": amount,
        "cost": cost,
        "summary": summary,
    }))
}
