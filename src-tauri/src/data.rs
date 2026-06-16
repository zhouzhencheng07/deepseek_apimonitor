use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::endpoints::Endpoints;
use crate::endpoints::TokenTypes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetail {
    pub name: String,
    pub total_tokens: u64,
    pub cache_hit: u64,
    pub cache_miss: u64,
    pub output_tokens: u64,
    pub cost: f64,
    pub today_tokens: u64,
    pub today_hit: u64,
    pub today_output_tokens: u64,
    pub today_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyDetail {
    pub date: String,
    pub total_tokens: u64,
    pub cache_hit: u64,
    pub output_tokens: u64,
    pub hit_rate: String,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub balance: f64,
    pub month_cost: f64,
    pub month_tokens: u64,
    pub month_hit: String,
    pub month_out_tokens: u64,
    pub today_label: String,
    pub today_cost: f64,
    pub today_tokens: u64,
    pub today_hit: String,
    pub today_out_tokens: u64,
    pub models: Vec<ModelDetail>,
    pub daily: Vec<DailyDetail>,
    pub update_time: String,
}

// ── helpers（提取自 make_report_data 的重复逻辑） ──

/// 从单个 usage 条目解析 amount 字段。
/// DeepSeek 的 amount 统一是字符串数字，这里统一 parse，调用处不用每次 unwrap_or 链。
fn usage_amount<T>(u: &Value) -> T
where T: std::str::FromStr + Default {
    u["amount"].as_str().unwrap_or("0").parse().unwrap_or_default()
}

/// 遍历 usage 条目，按 token type 累加命中/未命中/输出，返回三元组 (hit, miss, resp)。
/// 接收 &Value 的迭代，兼容 &[Value] 与 Vec<&Value>（flatten 结果）。
fn count_tokens<'a, I>(usage: I, tt: &TokenTypes) -> (u64, u64, u64)
where I: IntoIterator<Item = &'a Value> {
    let mut hit = 0u64; let mut miss = 0u64; let mut resp = 0u64;
    for u in usage {
        let typ = u["type"].as_str().unwrap_or("");
        let amt = usage_amount::<u64>(u);
        match typ {
            t if t == tt.cache_hit => hit += amt,
            t if t == tt.cache_miss => miss += amt,
            t if t == tt.response => resp += amt,
            _ => {}
        }
    }
    (hit, miss, resp)
}

/// 遍历 usage 条目，累加 type != request 的 amount（费用），返回总和。
fn sum_cost<'a, I>(usage: I, tt: &TokenTypes) -> f64
where I: IntoIterator<Item = &'a Value> {
    usage.into_iter()
        .filter(|u| u["type"].as_str() != Some(tt.request.as_str()))
        .map(|u| usage_amount::<f64>(u))
        .sum()
}

/// 缓存命中率（hit / (hit+miss)），无 prompt 时返回 "N/A"。
fn hit_rate_str(hit: u64, prompt: u64) -> String {
    if prompt == 0 { "N/A".to_string() } else { format!("{:.1}%", hit as f64 / prompt as f64 * 100.0) }
}

pub fn make_report_data(raw: &Value, endpoints: &Endpoints) -> Option<ReportData> {
    let tt = &endpoints.token_types;
    let biz = raw["summary"]["data"]["biz_data"].as_object()?;
    let balance = biz["normal_wallets"][0]["balance"]
        .as_str().unwrap_or("0").parse::<f64>().ok()?;
    let month_cost = biz["monthly_costs"][0]["amount"]
        .as_str().unwrap_or("0").parse::<f64>().ok()?;
    let month_tokens = biz["monthly_token_usage"]
        .as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);

    let amt_biz = raw["amount"]["data"]["biz_data"].as_object()?;
    let total_list = amt_biz["total"].as_array()?;

    // 先收集本次接口实际出现的所有 model 名，再用 whitelist 过滤出展示集。
    // whitelist 空 / 全没命中时回退到全部，避免静默丢数据。
    let recognized: Vec<String> = total_list
        .iter()
        .filter_map(|m| m["model"].as_str().map(|s| s.to_string()))
        .collect();
    let show: std::collections::HashSet<String> = endpoints.filter_models(&recognized).into_iter().collect();

    let mut all_hit = 0u64; let mut all_miss = 0u64; let mut all_resp = 0u64;
    for m in total_list {
        if let Some(usage) = m["usage"].as_array() {
            let (h, mi, r) = count_tokens(usage, tt);
            all_hit += h; all_miss += mi; all_resp += r;
        }
    }
    let month_hit = hit_rate_str(all_hit, all_hit + all_miss);

    // 最近有数据日期
    let empty_days: Vec<Value> = vec![];
    let amount_days = amt_biz["days"].as_array().unwrap_or(&empty_days);
    let mut today_label = Local::now().format("%Y-%m-%d").to_string();
    let mut cur_hit = 0u64; let mut cur_miss = 0u64; let mut cur_resp = 0u64;
    for d in amount_days.iter().rev() {
        if let Some(data) = d["data"].as_array() {
            let usage_iter = data.iter().filter_map(|m| m["usage"].as_array()).flatten();
            let (hit, miss, resp) = count_tokens(usage_iter, tt);
            if hit + miss + resp > 0 {
                today_label = d["date"].as_str().unwrap_or("").to_string();
                cur_hit = hit; cur_miss = miss; cur_resp = resp;
                break;
            }
        }
    }
    let t_total = cur_hit + cur_miss + cur_resp;
    let today_hit = hit_rate_str(cur_hit, cur_hit + cur_miss);

    // 费用
    let empty_days2: Vec<Value> = vec![];
    let cost_days = raw["cost"]["data"]["biz_data"][0]["days"].as_array().unwrap_or(&empty_days2);
    let mut t_cost = 0.0;
    for d in cost_days {
        if d["date"].as_str() == Some(&today_label) {
            if let Some(data) = d["data"].as_array() {
                for m in data {
                    if let Some(usage) = m["usage"].as_array() {
                        t_cost += sum_cost(usage, tt);
                    }
                }
            }
            break;
        }
    }

    // 各模型当日
    let mut today_model: std::collections::HashMap<String, (u64, u64, u64, f64)> = std::collections::HashMap::new();
    for d in cost_days {
        if d["date"].as_str() != Some(&today_label) { continue; }
        if let Some(data) = d["data"].as_array() {
            for m in data {
                let name = m["model"].as_str().unwrap_or("").to_string();
                if !show.contains(&name) { continue; }
                let cost = m["usage"].as_array().map_or(0.0, |u| sum_cost(u, tt));
                today_model.insert(name, (0, 0, 0, cost));
            }
        }
        break;
    }
    for d in amount_days {
        if d["date"].as_str() != Some(&today_label) { continue; }
        if let Some(data) = d["data"].as_array() {
            for m in data {
                let name = m["model"].as_str().unwrap_or("").to_string();
                if !show.contains(&name) { continue; }
                let (hit, miss, resp) = m["usage"].as_array()
                    .map_or((0, 0, 0), |u| count_tokens(u, tt));
                let e = today_model.entry(name).or_insert((0, 0, 0, 0.0));
                e.0 = hit + miss + resp; e.1 = hit; e.2 = resp;
            }
        }
        break;
    }

    // 模型明细
    let empty_total: Vec<Value> = vec![];
    let cost_total = raw["cost"]["data"]["biz_data"][0]["total"].as_array().unwrap_or(&empty_total);
    let mut cost_map = std::collections::HashMap::new();
    for m in cost_total {
        let name = m["model"].as_str().unwrap_or("").to_string();
        if !show.contains(&name) { continue; }
        let cost = m["usage"].as_array().map_or(0.0, |u| sum_cost(u, tt));
        cost_map.insert(name, cost);
    }

    let mut models = Vec::new();
    for m in total_list {
        let name = m["model"].as_str().unwrap_or("").to_string();
        if !show.contains(&name) { continue; }
        let (hit, miss, resp) = m["usage"].as_array()
            .map_or((0, 0, 0), |u| count_tokens(u, tt));
        let cost = *cost_map.get(&name).unwrap_or(&0.0);
        let td = today_model.get(&name).unwrap_or(&(0, 0, 0, 0.0));
        models.push(ModelDetail {
            name, total_tokens: hit + miss + resp, cache_hit: hit,
            cache_miss: miss, output_tokens: resp, cost,
            today_tokens: td.0, today_hit: td.1, today_output_tokens: td.2, today_cost: td.3,
        });
    }

    // 每日明细
    let mut cost_day_map = std::collections::HashMap::new();
    for d in cost_days {
        let date = d["date"].as_str().unwrap_or("").to_string();
        let cost: f64 = d["data"].as_array().map_or(0.0, |data| {
            data.iter().map(|m| m["usage"].as_array().map_or(0.0, |u| sum_cost(u, tt))).sum()
        });
        cost_day_map.insert(date, cost);
    }

    let mut daily = Vec::new();
    for d in amount_days {
        let date = d["date"].as_str().unwrap_or("").to_string();
        let data = d["data"].as_array();
        let mut hit = 0u64; let mut miss = 0u64; let mut resp = 0u64;
        if let Some(data) = data {
            let usage_iter = data.iter().filter_map(|m| m["usage"].as_array()).flatten();
            let (h, mi, r) = count_tokens(usage_iter, tt);
            hit = h; miss = mi; resp = r;
        }
        let hr = hit_rate_str(hit, hit + miss);
        let day_cost = *cost_day_map.get(&date).unwrap_or(&0.0);
        daily.push(DailyDetail {
            date, total_tokens: hit + miss + resp, cache_hit: hit, output_tokens: resp, hit_rate: hr,
            cost: day_cost,
        });
    }

    Some(ReportData {
        balance, month_cost, month_tokens,
        month_hit, month_out_tokens: all_resp,
        today_label, today_cost: t_cost, today_tokens: t_total,
        today_hit, today_out_tokens: cur_resp,
        models, daily,
        update_time: Local::now().format("%m-%d %H:%M").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::endpoints::Endpoints;

    /// 构造一个通用的 mock API 响应，两个模型、一天数据。
    fn mock_api_data() -> Value {
        let amount_days = vec![json!({
            "date": "2026-06-15",
            "data": [{
                "model": "deepseek-v4-flash",
                "usage": [
                    {"type": "PROMPT_CACHE_HIT_TOKEN", "amount": "30"},
                    {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": "60"},
                    {"type": "RESPONSE_TOKEN", "amount": "20"}
                ]
            }]
        })];
        let cost_days = vec![json!({
            "date": "2026-06-15",
            "data": [{
                "model": "deepseek-v4-flash",
                "usage": [
                    {"type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0.0005"},
                    {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0.0010"},
                    {"type": "RESPONSE_TOKEN", "amount": "0.0003"},
                    {"type": "REQUEST", "amount": "1"}
                ]
            }]
        })];
        json!({
            "summary": {
                "data": {
                    "biz_data": {
                        "normal_wallets": [{"balance": "100.50"}],
                        "monthly_costs": [{"amount": "12.34"}],
                        "monthly_token_usage": "500000"
                    }
                }
            },
            "amount": {
                "data": {
                    "biz_data": {
                        "total": [
                            {
                                "model": "deepseek-v4-flash",
                                "usage": [
                                    {"type": "PROMPT_CACHE_HIT_TOKEN", "amount": "100"},
                                    {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": "200"},
                                    {"type": "RESPONSE_TOKEN", "amount": "50"}
                                ]
                            },
                            {
                                "model": "deepseek-v4-pro",
                                "usage": [
                                    {"type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0"},
                                    {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0"},
                                    {"type": "RESPONSE_TOKEN", "amount": "0"}
                                ]
                            }
                        ],
                        "days": amount_days
                    }
                }
            },
            "cost": {
                "data": {
                    "biz_data": [{
                        "total": [{
                            "model": "deepseek-v4-flash",
                            "usage": [
                                {"type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0.0015"},
                                {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0.0030"},
                                {"type": "RESPONSE_TOKEN", "amount": "0.0010"},
                                {"type": "REQUEST", "amount": "1"}
                            ]
                        }],
                        "days": cost_days
                    }]
                }
            }
        })
    }

    #[test]
    fn test_parse_basic() {
        let raw = mock_api_data();
        let eps = Endpoints::default();
        let r = make_report_data(&raw, &eps).unwrap();

        assert_eq!(r.balance, 100.50);
        assert_eq!(r.month_cost, 12.34);
        assert_eq!(r.month_tokens, 500000);
        assert_eq!(r.month_out_tokens, 50);
        assert_eq!(r.month_hit, "33.3%");
        assert_eq!(r.today_label, "2026-06-15");
        assert!((r.today_cost - 0.0018).abs() < 1e-10, "today_cost {} != 0.0018", r.today_cost);
        assert_eq!(r.today_tokens, 110);
        assert_eq!(r.today_hit, "33.3%");
        assert_eq!(r.today_out_tokens, 20);

        // 两个模型
        assert_eq!(r.models.len(), 2);
        let flash = r.models.iter().find(|m| m.name == "deepseek-v4-flash").unwrap();
        assert_eq!(flash.total_tokens, 350);
        assert_eq!(flash.cache_hit, 100);
        assert_eq!(flash.cache_miss, 200);
        assert_eq!(flash.output_tokens, 50);
        assert!((flash.cost - 0.0055).abs() < 1e-10, "cost {} != 0.0055", flash.cost);
        assert_eq!(flash.today_tokens, 110);
        assert_eq!(flash.today_hit, 30);
        assert_eq!(flash.today_output_tokens, 20);
        assert!((flash.today_cost - 0.0018).abs() < 1e-10, "today_cost {} != 0.0018", flash.today_cost);

        // pro 没有数据
        let pro = r.models.iter().find(|m| m.name == "deepseek-v4-pro").unwrap();
        assert_eq!(pro.total_tokens, 0);
        assert_eq!(pro.cost, 0.0);

        // 每日明细
        assert_eq!(r.daily.len(), 1);
        assert_eq!(r.daily[0].date, "2026-06-15");
        assert_eq!(r.daily[0].total_tokens, 110);
        assert_eq!(r.daily[0].hit_rate, "33.3%");
        assert!((r.daily[0].cost - 0.0018).abs() < 1e-10, "cost {} != 0.0018", r.daily[0].cost);
    }

    #[test]
    fn test_whitelist_empty() {
        // whitelist 空 → 显示全部
        let raw = mock_api_data();
        let eps = Endpoints::default(); // whitelist = vec![]
        let r = make_report_data(&raw, &eps).unwrap();
        assert_eq!(r.models.len(), 2);
    }

    #[test]
    fn test_whitelist_matching() {
        // whitelist 命中 → 只显示匹配的
        let raw = mock_api_data();
        let mut eps = Endpoints::default();
        eps.whitelist = vec!["deepseek-v4-flash".to_string()];
        let r = make_report_data(&raw, &eps).unwrap();
        assert_eq!(r.models.len(), 1);
        assert_eq!(r.models[0].name, "deepseek-v4-flash");
    }

    #[test]
    fn test_whitelist_none_match() {
        // whitelist 全没命中 → 回退到全部
        let raw = mock_api_data();
        let mut eps = Endpoints::default();
        eps.whitelist = vec!["nonexistent-model".to_string()];
        let r = make_report_data(&raw, &eps).unwrap();
        assert_eq!(r.models.len(), 2);
    }

    #[test]
    fn test_no_data() {
        // 空数据 → None
        let raw = json!({});
        let eps = Endpoints::default();
        assert!(make_report_data(&raw, &eps).is_none());
    }

    #[test]
    fn test_hit_rate_na() {
        let raw = json!({
            "summary": {
                "data": {
                    "biz_data": {
                        "normal_wallets": [{"balance": "0"}],
                        "monthly_costs": [{"amount": "0"}],
                        "monthly_token_usage": "0"
                    }
                }
            },
            "amount": {
                "data": {
                    "biz_data": {
                        "total": [{
                            "model": "test-model",
                            "usage": [
                                {"type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0"},
                                {"type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0"},
                                {"type": "RESPONSE_TOKEN", "amount": "0"}
                            ]
                        }],
                        "days": []
                    }
                }
            },
            "cost": {
                "data": {
                    "biz_data": [[{
                        "total": [],
                        "days": []
                    }]]
                }
            }
        });
        let eps = Endpoints::default();
        let r = make_report_data(&raw, &eps).unwrap();
        assert_eq!(r.month_hit, "N/A");
        assert_eq!(r.today_hit, "N/A");
    }
}
