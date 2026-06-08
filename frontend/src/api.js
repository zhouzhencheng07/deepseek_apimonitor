import { invoke } from "@tauri-apps/api/core";

function flog(msg) {
  console.log("[DeepSeekMonitor]", msg);
}

export async function ping() {
  const r = await invoke("ping");
  flog("ping 结果: " + r);
  return r;
}

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
  flog("getData 调用");
  const raw = await invoke("get_data");
  const data = JSON.parse(raw);
  flog("getData 成功");
  return data;
}

export async function openModels() {
  flog("openModels 调用");
  await invoke("open_model_window");
  flog("openModels 成功");
}

export async function openDaily() {
  flog("openDaily 调用");
  await invoke("open_daily_window");
  flog("openDaily 成功");
}

export function exitApp() {
  flog("exitApp 调用");
  invoke("quit_app");
}
