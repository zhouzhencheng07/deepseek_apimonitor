#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod config;
mod data;
mod token;

use std::sync::Mutex;
use tauri::{Manager, State};

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
}

#[tauri::command]
fn ping() -> String {
    log("ping -> pong");
    "pong".to_string()
}

#[tauri::command]
fn get_data(state: State<AppState>) -> Result<String, String> {
    log("get_data 被调用");
    let token = state.token.lock().map_err(|e| { log(&format!("锁获取失败: {}", e)); e.to_string() })?;
    let cfg = &state.config;

    let raw = api::fetch_data(&token, cfg).map_err(|e| { log(&format!("API请求失败: {}", e)); format!("API 请求失败: {}", e) })?;
    let report = data::make_report_data(&raw).ok_or_else(|| { log("数据解析失败"); "数据解析失败".to_string() })?;

    *state.report.lock().map_err(|e| e.to_string())? = Some(report.clone());

    serde_json::to_string(&report).map_err(|e| { log(&format!("序列化失败: {}", e)); e.to_string() })
}

fn open_or_focus_window(app: &tauri::AppHandle, label: &str, url: &str, title: &str, w: f64, h: f64) -> Result<(), String> {
    log(&format!("尝试打开窗口: {} ({})", label, title));
    if let Some(win) = app.get_webview_window(label) {
        log(&format!("窗口 {} 已存在，聚焦", label));
        let _ = win.set_focus();
        return Ok(());
    }
    log(&format!("创建新窗口: {}", label));
    tauri::WebviewWindowBuilder::new(app, label, tauri::WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(w, h)
        .center()
        .closable(true)
        .build()
        .map_err(|e| { log(&format!("创建窗口失败: {}", e)); e.to_string() })?;
    log(&format!("窗口 {} 创建成功", label));
    Ok(())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) -> Result<(), String> {
    log("quit_app 被调用，退出程序");
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn open_model_window(app: tauri::AppHandle) -> Result<(), String> {
    log("open_model_window 被调用");
    open_or_focus_window(&app, "models", "models.html", "按模型统计", 760.0, 300.0)
}

#[tauri::command]
fn open_daily_window(app: tauri::AppHandle) -> Result<(), String> {
    log("open_daily_window 被调用");
    open_or_focus_window(&app, "daily", "daily.html", "按日统计", 640.0, 340.0)
}

fn main() {
    init_panic_hook();
    log("=== 程序启动 ===");

    log("加载配置...");
    let cfg = config::Config::load();
    log("配置已加载");

    let token = match token::load_token() {
        Some(t) if token::validate_token(&t, &cfg) => {
            log("Token 有效，直连 API");
            t
        }
        _ => {
            log("无有效 Token，尝试提取");
            match token::extract_token(&cfg, true) {
                Some(t) => {
                    token::save_token(&t);
                    log("Token 提取成功");
                    t
                }
                None => {
                    log("Token 提取失败");
                    panic!("获取 Token 失败");
                }
            }
        }
    };

    log("获取数据...");
    let report = api::fetch_data(&token, &cfg)
        .ok()
        .and_then(|raw| data::make_report_data(&raw));
    log("数据获取完成");

    log("启动 Tauri GUI...");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|w, e| {
            if let tauri::WindowEvent::CloseRequested { .. } = e {
                log("窗口关闭请求，退出程序");
                w.app_handle().exit(0);
            }
        })
        .manage(AppState {
            token: Mutex::new(token),
            report: Mutex::new(report),
            config: cfg,
        })
        .invoke_handler(tauri::generate_handler![
            ping, get_data, open_model_window, open_daily_window, quit_app
        ])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
