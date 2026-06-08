<script setup>
import { ref, onMounted } from "vue";
import { getData, openModels, openDaily, exitApp, fmt, balance } from "./api.js";

const d = ref(null);

async function load() {
  try { d.value = await getData(); } catch (e) { d.value = null; }
}

function refresh() { load(); }

onMounted(() => {
  load();
  setInterval(load, 120000);
});
</script>

<template>
  <div class="p-1 select-none font-sans text-xs min-w-[280px]">
    <div v-if="d" class="space-y-0.5">
      <!-- 余额 + 时间 -->
      <div class="flex justify-between items-center">
        <span class="font-bold text-sm">[余额] {{ balance(d.balance) }}</span>
        <span class="text-gray-400 text-[10px]">{{ d.update_time }} 每120s</span>
      </div>

      <!-- 数据表 -->
      <table class="w-full">
        <thead>
          <tr class="text-left">
            <th class="w-12"></th>
            <th class="font-bold">月度</th>
            <th class="font-bold text-right">{{ d.today_label.slice(5) }}</th>
          </tr>
        </thead>
        <tbody>
          <tr><td>消费</td><td>{{ balance(d.month_cost) }}</td><td class="text-right">{{ balance(d.today_cost) }}</td></tr>
          <tr><td>Token</td><td>{{ d.month_tokens }}</td><td class="text-right">{{ fmt(d.today_tokens) }}</td></tr>
          <tr><td>请求</td><td>{{ fmt(d.month_req) }}</td><td class="text-right">{{ fmt(d.today_req) }}</td></tr>
          <tr><td>命中</td><td>{{ d.month_hit }}</td><td class="text-right">{{ d.today_hit }}</td></tr>
          <tr><td>输出</td><td>{{ fmt(d.month_out_tokens) }}</td><td class="text-right">{{ fmt(d.today_out_tokens) }}</td></tr>
        </tbody>
      </table>

      <!-- 按钮 -->
      <div class="flex gap-1 pt-0.5">
        <button @click="openModels" class="px-2 py-0.5 border border-gray-300 rounded text-xs cursor-pointer hover:bg-gray-100">按模型统计</button>
        <button @click="openDaily" class="px-2 py-0.5 border border-gray-300 rounded text-xs cursor-pointer hover:bg-gray-100">按日统计</button>
        <button @click="refresh" class="px-2 py-0.5 border border-gray-300 rounded text-xs cursor-pointer hover:bg-gray-100">刷新</button>
        <button @click="exitApp" class="px-2 py-0.5 border border-gray-300 rounded text-xs cursor-pointer hover:bg-gray-100">退出</button>
      </div>
    </div>
    <div v-else class="text-red-500">加载失败</div>
  </div>
</template>
