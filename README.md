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
| `token` | 认证凭证（Authorization 的 Bearer 值） |
| `ball_pos` | 悬浮球位置 |

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
│       ├── token.rs        Token 缓存/验证
│       ├── api.rs          HTTP API 请求
│       └── data.rs         数据处理
├── frontend/               Vue 3 前端
│   └── src/
│       ├── Dashboard.vue   主界面
│       ├── ModelsView.vue  按模型统计
│       ├── DailyView.vue   按日统计
│       └── BallView.vue    悬浮球窗口
├── config.json             配置模板（运行时在 ~/.deepseek_monitor/config.json）
├── package.json            Node 依赖
└── vite.config.js          构建配置
```

## License

MIT
