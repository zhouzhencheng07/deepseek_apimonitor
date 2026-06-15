#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod config;
mod data;
mod endpoints;
mod token;

use std::sync::Mutex;
use std::time::Duration;
use tauri::{Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[cfg(windows)]
fn init_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("启动失败: {}", info);
        let wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
        let title: Vec<u16> = "DeepSeek Monitor\0".encode_utf16().collect();
        unsafe {
            extern "system" {
                fn MessageBoxW(hWnd: isize, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
            }
            MessageBoxW(0, wide.as_ptr(), title.as_ptr(), 0x10);
        }
    }));
}

#[cfg(not(windows))]
fn init_panic_hook() {}

fn log(msg: &str) {
    let log_path = std::env::temp_dir().join("deepseek-monitor.log");
    // 超过 1MB 就清空，避免长期运行无限增长。用 try_ 不让日志本身搞崩程序。
    if log_path.metadata().map(|m| m.len() > 1_048_576).unwrap_or(false) {
        let _ = std::fs::write(&log_path, "");
    }
    if let Ok(f) = std::fs::OpenOptions::new()
        .create(true).append(true).open(&log_path)
    {
        use std::io::Write;
        let _ = writeln!(&f, "[{:?}] {}", std::time::SystemTime::now(), msg);
    }
}

struct AppState {
    token: Mutex<String>,
    report: Mutex<Option<data::ReportData>>,
    config: config::Config,
    endpoints: endpoints::Endpoints,
    client: reqwest::Client,
}

#[tauri::command]
fn ping(win: tauri::Window) -> String {
    log(&format!("ping 被调用, 来自窗口: {}", win.label()));
    "pong".to_string()
}

#[tauri::command]
async fn get_data(state: State<'_, AppState>) -> Result<String, String> {
    log("get_data 被调用");
    let token = state.token.lock().map_err(|e| { log(&format!("锁获取失败: {}", e)); e.to_string() })?.clone();
    if token.is_empty() {
        return Err("NOT_LOGGED_IN".to_string());
    }
    let cfg = &state.config;
    let endpoints = &state.endpoints;
    let client = &state.client;

    let raw = api::fetch_data(client, &token, cfg, endpoints).await.map_err(|e| {
        log(&format!("API请求失败: {}", e));
        if e == "TOKEN_INVALID" { e } else { format!("API 请求失败: {}", e) }
    })?;
    let report = data::make_report_data(&raw, &endpoints.token_types).ok_or_else(|| { log("数据解析失败"); "数据解析失败".to_string() })?;

    *state.report.lock().map_err(|e| e.to_string())? = Some(report.clone());

    serde_json::to_string(&report).map_err(|e| { log(&format!("序列化失败: {}", e)); e.to_string() })
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) -> Result<(), String> {
    log("quit_app 被调用，退出程序");
    app.exit(0);
    Ok(())
}

fn pos_path() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("无法获取用户目录");
    home.join(".deepseek_monitor").join("ball_pos")
}

#[tauri::command]
fn save_ball_pos(x: i32, y: i32) -> Result<(), String> {
    std::fs::write(pos_path(), format!("{} {}", x, y))
        .map_err(|e| format!("保存位置失败: {}", e))
}

#[tauri::command]
fn load_ball_pos() -> Result<Option<(i32, i32)>, String> {
    let path = pos_path();
    if !path.exists() { return Ok(None); }
    let s = std::fs::read_to_string(&path).map_err(|e| format!("读取位置失败: {}", e))?;
    let parts: Vec<&str> = s.trim().split_whitespace().collect();
    if parts.len() < 2 { return Ok(None); }
    let x = parts[0].parse::<i32>().map_err(|_| "解析坐标失败".to_string())?;
    let y = parts[1].parse::<i32>().map_err(|_| "解析坐标失败".to_string())?;
    Ok(Some((x, y)))
}

#[tauri::command]
fn get_refresh_interval(state: State<AppState>) -> u64 {
    state.config.refresh_interval
}

#[tauri::command]
fn start_login(app: tauri::AppHandle) -> Result<(), String> {
    app.opener().open_url("https://platform.deepseek.com", None::<&str>)
        .map_err(|e| format!("打开浏览器失败: {}", e))?;
    Ok(())
}

#[tauri::command]
fn save_token_cmd(app: tauri::AppHandle, token: String) -> Result<(), String> {
    if token.len() < 20 {
        return Err("Token 格式不正确".to_string());
    }
    token::save_token(&token);
    if let Some(state) = app.try_state::<AppState>() {
        *state.token.lock().map_err(|e| format!("更新 Token 失败: {}", e))? = token;
    }
    let _ = app.emit("login-success", "");
    Ok(())
}

fn main() {
    init_panic_hook();
    log("=== 程序启动 ===");

    log("加载配置...");
    let cfg = config::Config::load();
    log("配置已加载");

    log("加载 endpoints...");
    let eps = endpoints::Endpoints::load();
    log("endpoints 已加载");

    // 复用单个 Client（含连接池与 timeout），避免每次请求重建 TLS 栈。
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("构建 HTTP Client 失败");

    let init_token = token::load_token().unwrap_or_default();
    if init_token.is_empty() {
        log("无 Token 文件，稍后提示登录");
    } else {
        log(&format!("Token 已加载（{} 字符），启动后验证", init_token.len()));
    }

    // 启动时不阻塞验证/fetch，交给前端 mount 后的 get_data 处理
    log("启动 Tauri GUI...");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|w, e| {
            if let tauri::WindowEvent::CloseRequested { .. } = e {
                if w.label() == "main" {
                    log("主窗口关闭，退出程序");
                    w.app_handle().exit(0);
                } else {
                    log(&format!("子窗口 {} 关闭，不退出", w.label()));
                }
            }
        })
        .manage(AppState {
            token: Mutex::new(init_token),
            report: Mutex::new(None),
            config: cfg,
            endpoints: eps,
            client,
        })
        .setup(|_app| {
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping, get_data, quit_app, save_ball_pos, load_ball_pos,
            get_refresh_interval, start_login, save_token_cmd
        ])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
