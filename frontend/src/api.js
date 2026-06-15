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
  // 后端 get_data 直接返回 ReportData（Tauri 自动序列化），前端无需再 JSON.parse。
  return invoke("get_data");
}

export function exitApp() {
  invoke("quit_app");
}
