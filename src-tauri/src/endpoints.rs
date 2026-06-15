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
        }
    }
}

impl Endpoints {
    // 用当前月份/年份填充路径里的 {month}/{year} 占位符。
    // 不支持其它占位符，避免给用户过高的模板预期。
    pub fn fill(&self, path: &str, month: &str, year: &str) -> String {
        path.replace("{month}", month).replace("{year}", year)
    }
}

fn cache_dir() -> PathBuf {
    let home = dirs::home_dir().expect("无法获取用户目录");
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
                    if e.amount_path.is_empty()
                        || e.cost_path.is_empty()
                        || e.summary_path.is_empty()
                    {
                        Endpoints::default()
                    } else {
                        e
                    }
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
