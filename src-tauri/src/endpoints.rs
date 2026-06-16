use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// DeepSeek 平台请求路径与 token type 常量。
// 运行时从 ~/.deepseek_monitor/endpoints.json 读取，缺失/字段不全时用默认值兜底，
// 这样 DeepSeek 改了路径或 type 名时，用户编辑本地文件、重启即可，无需重新发版。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Endpoints {
    #[serde(default)]
    pub amount_path: String,
    #[serde(default)]
    pub cost_path: String,
    #[serde(default)]
    pub summary_path: String,
    #[serde(default)]
    pub token_types: TokenTypes,
    // 模型白名单：只统计完整名称等于其中某一项的模型（精确匹配，不做子串/模糊匹配）。
    // 留空，或本次一个都没匹配上时，回退到"本次识别到的全部模型"——
    // 这样配错（或 DeepSeek 改了模型名）时用户仍能在界面看到真实 model 名，据此把白名单补全。
    // 与黑名单相比：新增模型默认不显示，垃圾字段（如 deepseek-chat/reasoner 网页字段）天然被排除。
    #[serde(default)]
    pub whitelist: Vec<String>,
}

// usage 数组里按 type 区分 token 类型，DeepSeek 偶尔会改这些常量名。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTypes {
    #[serde(default)]
    pub cache_hit: String,
    #[serde(default)]
    pub cache_miss: String,
    #[serde(default)]
    pub response: String,
    #[serde(default)]
    pub request: String,
}

impl Default for TokenTypes {
    fn default() -> Self {
        TokenTypes {
            cache_hit: "PROMPT_CACHE_HIT_TOKEN".to_string(),
            cache_miss: "PROMPT_CACHE_MISS_TOKEN".to_string(),
            response: "RESPONSE_TOKEN".to_string(),
            request: "REQUEST".to_string(),
        }
    }
}

impl Default for Endpoints {
    fn default() -> Self {
        Endpoints {
            amount_path: "/usage/amount?month={month}&year={year}".to_string(),
            cost_path: "/usage/cost?month={month}&year={year}".to_string(),
            summary_path: "/users/get_user_summary".to_string(),
            token_types: TokenTypes::default(),
            whitelist: vec![],
        }
    }
}

impl Endpoints {
    // 用当前月份/年份填充路径里的 {month}/{year} 占位符。
    // 不支持其它占位符，避免给用户过高的模板预期。
    pub fn fill(&self, path: &str, month: &str, year: &str) -> String {
        path.replace("{month}", month).replace("{year}", year)
    }

    // 按 whitelist 过滤本次识别到的模型。
    // 规则（精确匹配，非子串）：
    //   - whitelist 为空            → 返回 recognized 全部（不启用白名单）
    //   - whitelist 命中了至少一个  → 只返回命中的子集
    //   - whitelist 配了但全没命中  → 返回 recognized 全部（配错/DeepSeek 改名，回退避免丢数据）
    // recognized 需保序去重，保证界面展示稳定。
    pub fn filter_models(&self, recognized: &[String]) -> Vec<String> {
        if self.whitelist.is_empty() {
            return dedup_preserve_order(recognized);
        }
        let hit: Vec<String> = dedup_preserve_order(recognized)
            .into_iter()
            .filter(|name| self.whitelist.iter().any(|w| w == name))
            .collect();
        if hit.is_empty() {
            dedup_preserve_order(recognized)
        } else {
            hit
        }
    }
}

// 保序去重。
fn dedup_preserve_order(names: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(names.len());
    for n in names {
        if seen.insert(n.clone()) {
            out.push(n.clone());
        }
    }
    out
}

fn cache_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
    home.join(".deepseek_monitor")
}

fn endpoints_path() -> PathBuf {
    cache_dir().join("endpoints.json")
}

impl Endpoints {
    pub fn load() -> Self {
        let path = endpoints_path();
        if path.exists() {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => return Endpoints::default(),
            };
            // 解析失败或字段缺失一律回退默认值，避免配置写错导致程序崩溃。
            match serde_json::from_str::<Endpoints>(&content) {
                Ok(e) => {
                    // 任一关键字段为空（用户写成了空串）则整体用默认值，简化后续判空逻辑。
                    let e = if e.amount_path.is_empty()
                        || e.cost_path.is_empty()
                        || e.summary_path.is_empty()
                    {
                        Endpoints::default()
                    } else {
                        e
                    };
                    // 回写：把缺失字段（如老用户升级后没有的 whitelist）补齐落盘。
                    // 仅当序列化后的内容与磁盘读入的不一致时才写，避免每次启动都做无谓 IO。
                    persist_if_changed(&path, &content, &e);
                    e
                }
                Err(_) => Endpoints::default(),
            }
        } else {
            // 首次运行，用默认值写出，方便用户后续编辑。
            let cfg = Endpoints::default();
            if let Ok(content) = serde_json::to_string_pretty(&cfg) {
                let _ = std::fs::create_dir_all(cache_dir());
                let _ = std::fs::write(&path, &content);
            }
            cfg
        }
    }
}

// 仅当序列化结果与磁盘读入的不一致（字段缺失被 default 补齐、或用户手改过格式）
// 才回写，避免每次启动都做无谓 IO。
fn persist_if_changed(path: &std::path::Path, old_content: &str, eps: &Endpoints) {
    let Ok(new_content) = serde_json::to_string_pretty(eps) else { return };
    if new_content == old_content { return; }
    let _ = std::fs::create_dir_all(cache_dir());
    let _ = std::fs::write(path, new_content);
}
