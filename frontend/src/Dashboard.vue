<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { getData, fmt, balance } from "./api.js";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { listen } from '@tauri-apps/api/event';

const appWindow = getCurrentWindow();

let timer = null;
let unlisten = null;
const intervalSec = ref(120);

async function startTimer() {
  if (timer) clearInterval(timer);
  try {
    const interval = await invoke('get_refresh_interval');
    intervalSec.value = interval;
    timer = setInterval(load, interval * 1000);
  } catch {
    intervalSec.value = 120;
    timer = setInterval(load, 120000);
  }
}

function stopTimer() {
  if (timer) { clearInterval(timer); timer = null; }
}

function hitRate(hit, total, output) {
  const d = total - output;
  if (d <= 0) return 'N/A';
  return (hit / d * 100).toFixed(1) + '%';
}

const d = ref(null);
const err = ref("");
const showDaily = ref(false);
const showLogin = ref(false);
const loginErr = ref("");
const tokenInput = ref("");

async function load() {
  err.value = "";
  showLogin.value = false;
  try { d.value = await getData(); }
  catch (e) {
    d.value = null;
    if (e === "NOT_LOGGED_IN" || e === "TOKEN_INVALID") {
      showLogin.value = true;
    } else {
      err.value = "数据加载失败: " + e;
    }
  }
}
function doLogin() {
  loginErr.value = "";
  invoke("start_login");
}

async function submitToken() {
  const t = tokenInput.value.trim();
  if (!t) { loginErr.value = "请输入 Token"; return; }
  loginErr.value = "";
  try {
    await invoke("save_token_cmd", { token: t });
    load();
  } catch (e) {
    loginErr.value = "保存失败: " + e;
  }
}

async function openBall() {
  const existing = await WebviewWindow.getByLabel('ball');
  if (existing) { appWindow.hide(); stopTimer(); return; }
  new WebviewWindow('ball', {
    url: '/?ball',
    width: 105, height: 55,
    decorations: false,
    alwaysOnTop: true, skipTaskbar: true,
  });
  appWindow.hide();
  // 主窗口隐藏后由悬浮球窗口负责轮询，这里停掉自己的 timer，避免重复请求翻倍。
  stopTimer();
}

onMounted(() => {
  unlisten = listen('focus-main', () => {
    appWindow.show();
    appWindow.setFocus();
    load();
    startTimer();
  });
  listen('login-success', () => { load(); });
  load();
  startTimer();
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
  if (unlisten) unlisten.then(fn => fn());
});
</script>

<template>
  <div class="py-0.5 px-1 select-none font-sans text-xs">
    <div v-if="err && !showLogin" class="text-red-500 mb-1">{{ err }}</div>

    <!-- 登录界面 -->
    <div v-if="showLogin" class="flex flex-col px-2 py-1 select-none text-xs">
      <div class="font-bold text-sm mb-1">首次使用：获取 DeepSeek 开发平台认证凭证</div>
      <button @click="doLogin"
        class="self-start px-3 py-1 bg-blue-500 text-white rounded cursor-pointer hover:bg-blue-600 mb-1 text-[11px]">
        打开 DeepSeek 平台
      </button>
      <div class="text-gray-400 text-[10px] mb-1 leading-relaxed">
        ① 浏览器登录后，F12 → 网络 → 刷新(F5)<br>
        ② 点 get_user_summary → 请求标头<br>
        ③ 找 Authorization: Bearer xxx → 复制 xxxx（获取完即可关浏览器）
      </div>
      <div class="flex gap-1">
        <input v-model="tokenInput" type="text" placeholder="粘贴 xxx 到此处..."
          class="flex-1 px-2 py-1 border border-gray-300 rounded outline-none focus:border-blue-400 text-[11px]" />
        <button @click="submitToken"
          class="px-3 py-1 bg-green-500 text-white rounded cursor-pointer hover:bg-green-600 text-[11px]">
          保存
        </button>
      </div>
      <div v-if="loginErr" class="text-red-500 mt-1">{{ loginErr }}</div>
    </div>

    <div v-if="d">
      <template v-if="!showDaily">
        <div class="flex gap-1">
          <span class="font-bold text-sm">[余额] {{ balance(d.balance) }}</span>
          <span class="text-gray-400 text-[10px] leading-[18px]">{{ d.update_time }} 每{{ intervalSec }}s</span>
        </div>
        <table>
          <thead><tr class="text-left">
            <th class="pr-2"></th><th class="pr-2">月度</th><th>{{ d.today_label.slice(5) }}</th>
          </tr></thead>
          <tbody>
            <tr><td class="pr-2">消费</td><td class="pr-2">{{ balance(d.month_cost) }}</td><td>{{ balance(d.today_cost) }}</td></tr>
            <tr><td class="pr-2">Token</td><td class="pr-2">{{ fmt(d.month_tokens) }}</td><td>{{ fmt(d.today_tokens) }}</td></tr>
            <tr><td class="pr-2">命中</td><td class="pr-2">{{ d.month_hit }}</td><td>{{ d.today_hit }}</td></tr>
            <tr><td class="pr-2">输出</td><td class="pr-2">{{ fmt(d.month_out_tokens) }}</td><td>{{ fmt(d.today_out_tokens) }}</td></tr>
          </tbody>
        </table>
        <table class="mt-1">
          <thead><tr class="text-center bg-gray-100"><th class="text-left pr-2">模型</th><th class="pr-2">月Token</th><th class="pr-2">月费用</th><th class="pr-2">月命中</th><th class="pr-2">月输出</th><th class="pr-2">日Token</th><th class="pr-2">日费用</th><th class="pr-2">日命中</th><th>日输出</th></tr></thead>
          <tbody>
            <tr v-for="m in d.models" :key="m.name" class="text-center"><td class="text-left pr-2">{{ m.name }}</td>
              <td class="pr-2">{{ fmt(m.total_tokens) }}</td><td class="pr-2">{{ m.cost.toFixed(4) }}</td>
              <td class="pr-2">{{ hitRate(m.cache_hit, m.total_tokens, m.output_tokens) }}</td><td class="pr-2">{{ fmt(m.output_tokens) }}</td>
              <td class="pr-2">{{ fmt(m.today_tokens) }}</td><td class="pr-2">{{ m.today_cost.toFixed(4) }}</td>
              <td class="pr-2">{{ hitRate(m.today_hit, m.today_tokens, m.today_output_tokens) }}</td><td>{{ fmt(m.today_output_tokens) }}</td>
            </tr>
          </tbody>
        </table>
        <div class="flex gap-1 mt-1">
          <button @click="showDaily = true" class="px-2 py-0.5 border border-gray-300 rounded cursor-pointer hover:bg-gray-100">按日统计</button>
          <button @click="load" class="px-2 py-0.5 border border-gray-300 rounded cursor-pointer hover:bg-gray-100">刷新</button>
          <button @click="openBall" class="ml-auto px-2 py-0.5 border border-gray-300 rounded cursor-pointer hover:bg-gray-100">悬浮球</button>
        </div>
      </template>

      <template v-if="showDaily">
        <div class="flex items-center mb-1">
          <button @click="showDaily = false" class="text-blue-500 mr-2">← 返回</button>
          <span class="font-bold">按日统计</span>
        </div>
        <table>
          <thead><tr class="text-center bg-gray-100"><th class="pr-2">日期</th><th class="pr-2">总Token</th><th class="pr-2">缓存命中率</th><th class="pr-2">输出Token</th><th>费用(￥)</th></tr></thead>
          <tbody><tr v-for="day in d.daily" :key="day.date" class="text-center">
            <td class="pr-2">{{ day.date }}</td><td class="pr-2">{{ fmt(day.total_tokens) }}</td>
            <td class="pr-2">{{ day.hit_rate }}</td><td class="pr-2">{{ fmt(day.output_tokens) }}</td><td>{{ day.cost.toFixed(4) }}</td>
          </tr></tbody>
        </table>
      </template>
    </div>
    <div v-else-if="!err">加载中...</div>
  </div>
</template>
