<script setup>
import { ref, onMounted } from "vue";
import { getData, fmt, balance } from "./api.js";

const d = ref(null);
const err = ref("");
const showPanel = ref(""); // "models" | "daily" | ""

async function load() {
  err.value = "";
  try { d.value = await getData(); }
  catch (e) { d.value = null; err.value = "数据加载失败: " + e; }
}
function refresh() { load(); }
function togglePanel(name) {
  showPanel.value = showPanel.value === name ? "" : name;
}

onMounted(() => { load(); setInterval(load, 120000); });
</script>

<template>
  <div class="p-1 select-none font-sans text-xs min-w-[280px] max-w-[500px]">
    <div v-if="err" class="text-red-500 text-[10px] mb-1">{{ err }}</div>
    <div v-if="d">
      <!-- 主界面 -->
      <template v-if="!showPanel">
        <div class="flex justify-between items-center">
          <span class="font-bold text-sm">[余额] {{ balance(d.balance) }}</span>
          <span class="text-gray-400 text-[10px]">{{ d.update_time }} 每120s</span>
        </div>
        <table class="w-full">
          <thead><tr class="text-left">
            <th class="w-12"></th><th class="font-bold">月度</th>
            <th class="font-bold text-right">{{ d.today_label.slice(5) }}</th>
          </tr></thead>
          <tbody>
            <tr><td>消费</td><td>{{ balance(d.month_cost) }}</td><td class="text-right">{{ balance(d.today_cost) }}</td></tr>
            <tr><td>Token</td><td>{{ d.month_tokens }}</td><td class="text-right">{{ fmt(d.today_tokens) }}</td></tr>
            <tr><td>请求</td><td>{{ fmt(d.month_req) }}</td><td class="text-right">{{ fmt(d.today_req) }}</td></tr>
            <tr><td>命中</td><td>{{ d.month_hit }}</td><td class="text-right">{{ d.today_hit }}</td></tr>
            <tr><td>输出</td><td>{{ fmt(d.month_out_tokens) }}</td><td class="text-right">{{ fmt(d.today_out_tokens) }}</td></tr>
          </tbody>
        </table>
        <div class="flex gap-1 pt-0.5">
          <button @click="togglePanel('models')" class="px-2 py-0.5 border border-gray-300 rounded text-xs cursor-pointer hover:bg-gray-100">按模型统计</button>
          <button @click="togglePanel('daily')" class="px-2 py-0.5 border border-gray-300 rounded text-xs cursor-pointer hover:bg-gray-100">按日统计</button>
          <button @click="refresh" class="px-2 py-0.5 border border-gray-300 rounded text-xs cursor-pointer hover:bg-gray-100">刷新</button>
          <button @click="exitApp" class="px-2 py-0.5 border border-gray-300 rounded text-xs cursor-pointer hover:bg-gray-100">退出</button>
        </div>
      </template>

      <!-- 按模型统计面板 -->
      <template v-if="showPanel === 'models'">
        <div class="flex items-center mb-1">
          <button @click="togglePanel('models')" class="text-blue-500 hover:text-blue-700 text-xs mr-2">← 返回</button>
          <span class="font-bold text-sm">按模型统计</span>
        </div>
        <table class="w-full text-xs border-collapse">
          <thead><tr class="text-center bg-gray-100 font-bold">
            <th class="text-left px-1">模型</th><th class="px-1">月请求</th><th class="px-1">月Token</th><th class="px-1">月费用</th>
            <th class="px-1">日请求</th><th class="px-1">日Token</th><th class="px-1">日费用</th>
          </tr></thead>
          <tbody>
            <tr v-for="m in d.models" :key="m.name" class="text-center border-b border-gray-100 even:bg-gray-50">
              <td class="text-left px-1">{{ m.name }}</td>
              <td class="px-1">{{ fmt(m.requests) }}</td><td class="px-1">{{ fmt(m.total_tokens) }}</td>
              <td class="px-1">{{ m.cost.toFixed(4) }}</td>
              <td class="px-1">{{ fmt(m.today_req) }}</td><td class="px-1">{{ fmt(m.today_tokens) }}</td>
              <td class="px-1">{{ m.today_cost.toFixed(4) }}</td>
            </tr>
          </tbody>
        </table>
      </template>

      <!-- 按日统计面板 -->
      <template v-if="showPanel === 'daily'">
        <div class="flex items-center mb-1">
          <button @click="togglePanel('daily')" class="text-blue-500 hover:text-blue-700 text-xs mr-2">← 返回</button>
          <span class="font-bold text-sm">按日统计</span>
        </div>
        <div class="overflow-x-auto max-h-[400px] overflow-y-auto">
          <table class="w-full text-xs border-collapse">
            <thead><tr class="text-center bg-gray-100 font-bold">
              <th class="px-1">日期</th><th class="px-1">请求数</th><th class="px-1">总Token</th>
              <th class="px-1">缓存命中率</th><th class="px-1">输出Token</th><th class="px-1">费用(￥)</th>
            </tr></thead>
            <tbody>
              <tr v-for="day in d.daily" :key="day.date" class="text-center border-b border-gray-100 even:bg-gray-50">
                <td class="px-1">{{ day.date }}</td><td class="px-1">{{ fmt(day.requests) }}</td>
                <td class="px-1">{{ fmt(day.total_tokens) }}</td><td class="px-1">{{ day.hit_rate }}</td>
                <td class="px-1">{{ fmt(day.output_tokens) }}</td><td class="px-1">{{ day.cost.toFixed(4) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </div>
    <div v-else-if="!err" class="text-gray-400">加载中...</div>
  </div>
</template>
