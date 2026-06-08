<script setup>
import { ref, onMounted } from "vue";
import { getData, fmt } from "./api.js";

const daily = ref([]);
const err = ref("");

onMounted(async () => {
  console.log("[DeepSeekMonitor] DailyView 挂载");
  try {
    const d = await getData();
    console.log("[DeepSeekMonitor] DailyView 数据获取成功，", d.daily.length + " 天");
    daily.value = d.daily;
  } catch (e) {
    console.error("[DeepSeekMonitor] DailyView 错误:", e);
    err.value = "加载失败: " + e;
  }
});
</script>

<template>
  <div class="p-2 font-sans">
    <h3 class="font-bold text-sm mb-2">按日统计</h3>
    <div v-if="err" class="text-red-500 text-xs mb-2">{{ err }}</div>
    <table v-if="daily.length" class="w-full text-xs border-collapse">
      <thead><tr class="text-center bg-gray-100 font-bold">
        <th class="px-1">日期</th><th class="px-1">请求数</th><th class="px-1">总Token</th>
        <th class="px-1">缓存命中率</th><th class="px-1">输出Token</th><th class="px-1">费用(￥)</th>
      </tr></thead>
      <tbody>
        <tr v-for="d in daily" :key="d.date" class="text-center border-b border-gray-100 even:bg-gray-50">
          <td class="px-1">{{ d.date }}</td><td class="px-1">{{ fmt(d.requests) }}</td>
          <td class="px-1">{{ fmt(d.total_tokens) }}</td><td class="px-1">{{ d.hit_rate }}</td>
          <td class="px-1">{{ fmt(d.output_tokens) }}</td><td class="px-1">{{ d.cost.toFixed(4) }}</td>
        </tr>
      </tbody>
    </table>
    <div v-else-if="!err" class="text-gray-400 text-xs">加载中...</div>
  </div>
</template>
