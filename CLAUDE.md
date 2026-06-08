# DeepSeek 用量监控

Tauri v2 桌面应用（Rust + Vue 3 + Tailwind CSS），监控 DeepSeek 平台使用数据。

## 运行方式

```cmd
cd D:\zzc\PythonProject\deepseek
npx tauri dev
```

或打包：

```cmd
npx tauri build
```

exe 在 `src-tauri\target\release\deepseek-monitor.exe`

## 工作原理

```
启动 → 读取缓存 Token → HTTP 验证
  ├─ 有效 → 直连 API → GUI 显示
  └─ 无效 → 提取失败 → 弹错误框
```

- Token 缓存在 `~/.deepseek_monitor/token`
- 首次需要从 Python 版获取 Token，或后续实现浏览器提取

## 项目结构

| 路径 | 说明 |
|------|------|
| `src-tauri/` | Rust 后端 |
| `src-tauri/src/main.rs` | Tauri 入口 + 命令 |
| `src-tauri/src/config.rs` | 配置加载 |
| `src-tauri/src/token.rs` | Token 缓存/验证 |
| `src-tauri/src/api.rs` | HTTP API 请求 |
| `src-tauri/src/data.rs` | 数据处理 |
| `frontend/src/` | Vue 3 前端 |
| `frontend/src/Dashboard.vue` | 主界面组件 |
| `frontend/src/ModelsView.vue` | 按模型统计组件 |
| `frontend/src/DailyView.vue` | 按日统计组件 |
| `config.json` | 配置（URL、间隔等） |

## 构建说明

```cmd
npm install        # 装前端依赖
npx tauri build    # 编译发布版
```
