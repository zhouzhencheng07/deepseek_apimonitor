<script setup>
import { ref, onMounted } from "vue";
import { getData, fmt, balance } from "./api.js";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { listen } from '@tauri-apps/api/event';

const appWindow = getCurrentWindow();

function hitRate(hit, total, output) {
  const d = total - output;
  if (d <= 0) return 'N/A';
  return (hit / d * 100).toFixed(1) + '%';
}

const d = ref(null);
const err = ref("");
const showDaily = ref(false);

async function load() {
  err.value = "";
  try { d.value = await getData(); }
  catch (e) { d.value = null; err.value = "数据加载失败: " + e; }
}
function refresh() { load(); }

async function openBall() {
  const existing = await WebviewWindow.getByLabel('ball');
  if (existing) { appWindow.hide(); return; }
  new WebviewWindow('ball', {
    url: '/?ball',
    width: 105, height: 55,
    decorations: false,
    alwaysOnTop: true, skipTaskbar: true,
  });
  appWindow.hide();
}

onMounted(() => {
  listen('focus-main', () => { appWindow.show(); appWindow.setFocus(); });
  load();
  setInterval(load, 120000);
});
</script>

<template>
  <div class="py-0.5 px-1 select-none font-sans text-xs">
    <div v-if="err" class="text-red-500 mb-1">{{ err }}</div>
    <div v-if="d">
      <template v-if="!showDaily">
        <div class="flex gap-1">
          <span class="font-bold text-sm">[余额] {{ balance(d.balance) }}</span>
          <span class="text-gray-400 text-[10px] leading-[18px]">{{ d.update_time }} 每120s</span>
        </div>
        <table>
          <thead><tr class="text-left">
            <th class="pr-2"></th><th class="pr-2">月度</th><th>{{ d.today_label.slice(5) }}</th>
          </tr></thead>
          <tbody>
            <tr><td class="pr-2">消费</td><td class="pr-2">{{ balance(d.month_cost) }}</td><td>{{ balance(d.today_cost) }}</td></tr>
            <tr><td class="pr-2">Token</td><td class="pr-2">{{ d.month_tokens }}</td><td>{{ fmt(d.today_tokens) }}</td></tr>
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
          <button @click="refresh" class="px-2 py-0.5 border border-gray-300 rounded cursor-pointer hover:bg-gray-100">刷新</button>
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
