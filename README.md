# DeepSeek 用量监控

[![Tauri](https://img.shields.io/badge/Tauri-v2-ffc131?logo=tauri)](https://v2.tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.70+-de5842?logo=rust)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3-4fc08d?logo=vue.js)](https://vuejs.org)
[![License](https://img.shields.io/badge/License-MIT-blue)](#license)

DeepSeek 平台 API 用量桌面监控工具。实时查看 API 消耗、缓存命中率、余额等信息。


## 功能

- **仪表盘** — 总览用量、余额、缓存命中率
- **模型统计** — 按模型查看调用次数、输入/输出 Token、缓存命中
- **日统计** — 按日期查看用量趋势
- **悬浮球** — 最小化到桌面悬浮球，实时显示用量

## 快速开始

### 前置要求

- [Rust](https://www.rust-lang.org/tools/install)（1.70 以上）
- [Node.js](https://nodejs.org)（18 以上）
- 一个 DeepSeek 平台账号

### 安装

```bash
# 安装前端依赖
npm install

# 启动开发模式
npx tauri dev
```

### 首次使用

启动后无登录凭证会提示手动输入。按以下步骤获取：

1. 浏览器登录 [DeepSeek 平台](https://platform.deepseek.com)
2. F12 → 网络 → 刷新(F5)
3. 点 get_user_summary → 请求标头
4. 找到 **Authorization: Bearer xxx**，复制 xxx
5. 粘贴到应用输入框，点保存

后续启动自动读取已保存的凭证。

### 构建发布版

```bash
npx tauri build
```

可执行文件在 `src-tauri/target/release/` 目录下。

## 配置

编辑 `~/.deepseek_monitor/config.json`（首次启动自动从默认值创建）：

| 字段 | 默认值 | 说明 |
|------|--------|------|
| `USAGE_URL` | `https://platform.deepseek.com/usage` | 用量页面 URL |
| `API_BASE` | `https://platform.deepseek.com/api/v0` | API 地址 |
| `REFRESH_INTERVAL` | `120` | 自动刷新间隔（秒） |

### 用户数据目录

运行数据存储在 `~/.deepseek_monitor/`：

| 文件 | 说明 |
|------|------|
| `config.json` | 用户配置 |
| `endpoints.json` | 接口路径与 token type 常量（DeepSeek 改接口时可自行编辑，重启生效） |
| `token` | 认证凭证（Authorization 的 Bearer 值） |
| `ball_pos` | 悬浮球位置 |

### endpoints.json 说明

DeepSeek 平台的用量接口属于内部私有接口，路径和字段 type 名偶尔会变。`endpoints.json` 把这些**易变项外置**，无需重新编译即可适配：

```json
{
  "amount_path":  "/usage/amount?month={month}&year={year}",
  "cost_path":    "/usage/cost?month={month}&year={year}",
  "summary_path": "/users/get_user_summary",
  "token_types": {
    "cache_hit":  "PROMPT_CACHE_HIT_TOKEN",
    "cache_miss": "PROMPT_CACHE_MISS_TOKEN",
    "response":   "RESPONSE_TOKEN",
    "request":    "REQUEST"
  },
  "whitelist": []
}
```

- 路径中的 `{month}` / `{year}` 为占位符，启动时自动填充当前年月；不支持其它占位符。
- `token_types` 是 usage 数组里区分 token 类型的字段值（缓存命中、未命中、输出、请求数）。
- `whitelist` 是要统计的**模型白名单**（完整 model 名，精确匹配，非子串）：
  - 留空 `[]` → 显示接口返回的**全部**模型（首次使用建议留空，先看真实名字）。
  - 填了 → 只显示白名单里的；若填的名字本次一个都没匹配上（配错或 DeepSeek 改名），自动回退显示全部，避免静默丢数据。
  - 想知道完整 model 名：先留空跑一次，从界面模型表格里看（例如 `deepseek-v4-flash`），照抄进白名单。
- 编辑后**重启程序生效**；写错某项时该字段自动回退默认值，不会导致崩溃。
- 仓库根的 `endpoints.json` 仅为模板参考，运行时实际读取的是 `~/.deepseek_monitor/endpoints.json`。

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | [Tauri v2](https://v2.tauri.app) |
| 后端 | [Rust](https://www.rust-lang.org) |
| 前端 | [Vue 3](https://vuejs.org) |
| 样式 | [Tailwind CSS](https://tailwindcss.com) |

## 项目结构

```
deepseek-monitor/
├── src-tauri/              Rust 后端
│   └── src/
│       ├── main.rs         Tauri 入口 + 命令
│       ├── config.rs       配置加载
│       ├── endpoints.rs    接口路径/白名单/token type 常量（可外置配置）
│       ├── token.rs        Token 加载/保存
│       ├── api.rs           HTTP API 请求（并发）
│       └── data.rs          数据处理与统计
├── frontend/               Vue 3 前端
│   ├── index.html
│   └── src/
│       ├── main.js          入口（路由 Dashboard / BallView）
│       ├── style.css        Tailwind 全局样式
│       ├── api.js           Tauri 命令封装
│       ├── Dashboard.vue    主界面（仪表盘 + 按模型/按日统计）
│       └── BallView.vue     悬浮球窗口
├── config.json             配置模板（运行时在 ~/.deepseek_monitor/config.json）
├── endpoints.json          接口路径模板（运行时在 ~/.deepseek_monitor/endpoints.json）
├── package.json            Node 依赖
└── vite.config.js          构建配置
```

## License

MIT
