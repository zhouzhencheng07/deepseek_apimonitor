use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub month_tokens: String,
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

pub fn make_report_data(raw: &Value, tt: &TokenTypes) -> Option<ReportData> {
    let biz = raw["summary"]["data"]["biz_data"].as_object()?;
    let balance = biz["normal_wallets"][0]["balance"]
        .as_str().unwrap_or("0").parse::<f64>().ok()?;
    let month_cost = biz["monthly_costs"][0]["amount"]
        .as_str().unwrap_or("0").parse::<f64>().ok()?;
    let month_tokens_str = biz["monthly_token_usage"].as_str().unwrap_or("0").to_string();

    let amt_biz = raw["amount"]["data"]["biz_data"].as_object()?;
    let total_list = amt_biz["total"].as_array()?;

    let mut all_hit = 0u64; let mut all_miss = 0u64; let mut all_resp = 0u64;
    for m in total_list {
        if let Some(usage) = m["usage"].as_array() {
            for u in usage {
                let typ = u["type"].as_str().unwrap_or("");
                let amt = u["amount"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);
                match typ {
                    t if t == tt.cache_hit => all_hit += amt,
                    t if t == tt.cache_miss => all_miss += amt,
                    t if t == tt.response => all_resp += amt,
                    _ => {}
                }
            }
        }
    }
    let all_prompt = all_hit + all_miss;
    let month_hit = if all_prompt > 0 {
        format!("{:.1}%", all_hit as f64 / all_prompt as f64 * 100.0)
    } else { "N/A".to_string() };

    // 最近有数据日期
    let empty_days: Vec<Value> = vec![];
    let amount_days = amt_biz["days"].as_array().unwrap_or(&empty_days);
    let mut t_tokens: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut today_label = Local::now().format("%Y-%m-%d").to_string();
    for d in amount_days.iter().rev() {
        if let Some(data) = d["data"].as_array() {
            let has_activity = data.iter().any(|m| {
                m["usage"].as_array().map_or(false, |usage| {
                    usage.iter().any(|u| {
                        let t = u["type"].as_str().unwrap_or("");
                        (t == tt.cache_hit || t == tt.cache_miss || t == tt.response)
                            && u["amount"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0) > 0
                    })
                })
            });
            if has_activity {
                today_label = d["date"].as_str().unwrap_or("").to_string();
                for m in data {
                    if let Some(usage) = m["usage"].as_array() {
                        for u in usage {
                            let typ = u["type"].as_str().unwrap_or("").to_string();
                            let amt = u["amount"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);
                            *t_tokens.entry(typ).or_insert(0) += amt;
                        }
                    }
                }
                break;
            }
        }
    }

    let t_hit = t_tokens.get(&tt.cache_hit).copied().unwrap_or(0);
    let t_miss = t_tokens.get(&tt.cache_miss).copied().unwrap_or(0);
    let t_resp = t_tokens.get(&tt.response).copied().unwrap_or(0);
    let t_total = t_hit + t_miss + t_resp;
    let t_prompt = t_hit + t_miss;
    let today_hit = if t_prompt > 0 {
        format!("{:.1}%", t_hit as f64 / t_prompt as f64 * 100.0)
    } else { "N/A".to_string() };

    // 费用
    let empty_days2: Vec<Value> = vec![];
    let cost_days = raw["cost"]["data"]["biz_data"][0]["days"].as_array().unwrap_or(&empty_days2);
    let mut t_cost = 0.0;
    for d in cost_days {
        if d["date"].as_str() == Some(&today_label) {
            if let Some(data) = d["data"].as_array() {
                for m in data {
                    if let Some(usage) = m["usage"].as_array() {
                        for u in usage {
                            if u["type"].as_str() != Some(tt.request.as_str()) {
                                t_cost += u["amount"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                            }
                        }
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
                if name.to_lowercase().contains("chat") || name.to_lowercase().contains("reasoner") { continue; }
                let mut cost = 0.0;
                if let Some(usage) = m["usage"].as_array() {
                    for u in usage {
                        if u["type"].as_str() != Some(tt.request.as_str()) {
                            cost += u["amount"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                        }
                    }
                }
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
                if name.to_lowercase().contains("chat") || name.to_lowercase().contains("reasoner") { continue; }
                let mut toks = 0u64; let mut hit = 0u64; let mut resp = 0u64;
                if let Some(usage) = m["usage"].as_array() {
                    for u in usage {
                        let typ = u["type"].as_str().unwrap_or("");
                        let amt = u["amount"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);
                        match typ {
                            t if t == tt.cache_hit => { hit += amt; toks += amt; }
                            t if t == tt.cache_miss => toks += amt,
                            t if t == tt.response => { resp += amt; toks += amt; }
                            _ => {}
                        }
                    }
                }
                let e = today_model.entry(name).or_insert((0, 0, 0, 0.0));
                e.0 = toks; e.1 = hit; e.2 = resp;
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
        if name.to_lowercase().contains("chat") || name.to_lowercase().contains("reasoner") { continue; }
        let mut cost = 0.0;
        if let Some(usage) = m["usage"].as_array() {
            for u in usage {
                if u["type"].as_str() != Some(tt.request.as_str()) {
                    cost += u["amount"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                }
            }
        }
        cost_map.insert(name, cost);
    }

    let mut models = Vec::new();
    for m in total_list {
        let name = m["model"].as_str().unwrap_or("").to_string();
        if name.to_lowercase().contains("chat") || name.to_lowercase().contains("reasoner") { continue; }
        let mut hit = 0u64; let mut miss = 0u64; let mut resp = 0u64;
        if let Some(usage) = m["usage"].as_array() {
            for u in usage {
                let typ = u["type"].as_str().unwrap_or("");
                let amt = u["amount"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);
                match typ {
                    t if t == tt.cache_hit => hit += amt,
                    t if t == tt.cache_miss => miss += amt,
                    t if t == tt.response => resp += amt,
                    _ => {}
                }
            }
        }
        let tt_tokens = hit + miss + resp;
        let cost = *cost_map.get(&name).unwrap_or(&0.0);
        let td = today_model.get(&name).unwrap_or(&(0, 0, 0, 0.0));
        models.push(ModelDetail {
            name, total_tokens: tt_tokens, cache_hit: hit,
            cache_miss: miss, output_tokens: resp, cost,
            today_tokens: td.0, today_hit: td.1, today_output_tokens: td.2, today_cost: td.3,
        });
    }

    // 每日明细
    let mut cost_day_map = std::collections::HashMap::new();
    for d in cost_days {
        let date = d["date"].as_str().unwrap_or("").to_string();
        let mut cost = 0.0;
        if let Some(data) = d["data"].as_array() {
            for m in data {
                if let Some(usage) = m["usage"].as_array() {
                    for u in usage {
                        if u["type"].as_str() != Some(tt.request.as_str()) {
                            cost += u["amount"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                        }
                    }
                }
            }
        }
        cost_day_map.insert(date, cost);
    }

    let mut daily = Vec::new();
    for d in amount_days {
        let date = d["date"].as_str().unwrap_or("").to_string();
        let mut u: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        if let Some(data) = d["data"].as_array() {
            for m in data {
                if let Some(usage) = m["usage"].as_array() {
                    for x in usage {
                        let typ = x["type"].as_str().unwrap_or("").to_string();
                        let amt = x["amount"].as_str().unwrap_or("0").parse::<u64>().unwrap_or(0);
                        *u.entry(typ).or_insert(0) += amt;
                    }
                }
            }
        }
        let hit = u.get(&tt.cache_hit).copied().unwrap_or(0);
        let miss = u.get(&tt.cache_miss).copied().unwrap_or(0);
        let resp = u.get(&tt.response).copied().unwrap_or(0);
        let d_total = hit + miss + resp;
        let tp = hit + miss;
        let hr = if tp > 0 { format!("{:.1}%", hit as f64 / tp as f64 * 100.0) } else { "N/A".to_string() };
        let day_cost = *cost_day_map.get(&date).unwrap_or(&0.0);
        daily.push(DailyDetail {
            date, total_tokens: d_total, cache_hit: hit, output_tokens: resp, hit_rate: hr,
            cost: day_cost,
        });
    }

    Some(ReportData {
        balance, month_cost, month_tokens: month_tokens_str,
        month_hit, month_out_tokens: all_resp,
        today_label, today_cost: t_cost, today_tokens: t_total,
        today_hit, today_out_tokens: t_resp,
        models, daily,
        update_time: Local::now().format("%m-%d %H:%M").to_string(),
    })
}
