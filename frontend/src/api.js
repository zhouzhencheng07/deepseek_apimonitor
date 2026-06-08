import { invoke } from "@tauri-apps/api/core";

export function fmt(n) {
  const num = Number(n);
  if (num >= 100_000_000) return (num / 100_000_000).toFixed(2) + "亿";
  if (num >= 10_000) return (num / 10_000).toFixed(2) + "万";
  return num.toLocaleString();
}

export function balance(n) {
  return "¥" + Number(n).toFixed(2);
}

export async function getData() {
  const raw = await invoke("get_data");
  return JSON.parse(raw);
}

export async function openModels() {
  try { await invoke("open_model_window"); } catch (_) {}
}

export async function openDaily() {
  try { await invoke("open_daily_window"); } catch (_) {}
}

export function exitApp() {
  import("@tauri-apps/api/webviewWindow").then(m =>
    m.getCurrentWebviewWindow().close()
  );
}
