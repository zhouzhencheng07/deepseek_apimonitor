#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod config;
mod data;
mod token;

use std::sync::Mutex;
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
fn ping(win: tauri::Window) -> String {
    log(&format!("ping 被调用, 来自窗口: {}", win.label()));
    "pong".to_string()
}

#[tauri::command]
fn get_data(state: State<AppState>) -> Result<String, String> {
    log("get_data 被调用");
    let token = state.token.lock().map_err(|e| { log(&format!("锁获取失败: {}", e)); e.to_string() })?;
    if token.is_empty() {
        return Err("NOT_LOGGED_IN".to_string());
    }
    let cfg = &state.config;

    let raw = api::fetch_data(&token, cfg).map_err(|e| { log(&format!("API请求失败: {}", e)); format!("API 请求失败: {}", e) })?;
    let report = data::make_report_data(&raw).ok_or_else(|| { log("数据解析失败"); "数据解析失败".to_string() })?;

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
        *state.token.lock().unwrap() = token;
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

    let token = token::load_token();
    let has_valid = token.as_ref().map_or(false, |t| token::validate_token(t, &cfg));
    let init_token = if has_valid {
        log("Token 有效，直连 API");
        token.unwrap()
    } else {
        log("无有效 Token");
        String::new()
    };

    let report = if has_valid {
        api::fetch_data(&init_token, &cfg)
            .ok()
            .and_then(|raw| data::make_report_data(&raw))
    } else {
        None
    };

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
            report: Mutex::new(report),
            config: cfg,
        })
        .setup(|_app| {
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping, get_data, quit_app, save_ball_pos, load_ball_pos,
            start_login, save_token_cmd
        ])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
