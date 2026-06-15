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

    // 三个请求互不依赖，并发发出，整体延迟由最慢的一个决定（原来串行是三者之和）。
    // fill() 返回的 String 要先绑定，否则临时值在语句结束就释放，Future 还持有引用。
    let amount_path = endpoints.fill(&endpoints.amount_path, &month, &year);
    let cost_path = endpoints.fill(&endpoints.cost_path, &month, &year);
    let amount = api_get(client, &amount_path, token, config);
    let cost = api_get(client, &cost_path, token, config);
    let summary = api_get(client, &endpoints.summary_path, token, config);

    let (amount, cost, summary) = tokio::try_join!(amount, cost, summary)?;

    Ok(serde_json::json!({
        "amount": amount,
        "cost": cost,
        "summary": summary,
    }))
}
