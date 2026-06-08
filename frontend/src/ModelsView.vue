<script setup>
import { ref, onMounted } from "vue";
import { getData, fmt } from "./api.js";

const models = ref([]);
const err = ref("");

onMounted(async () => {
  console.log("[DeepSeekMonitor] ModelsView 挂载");
  try {
    const d = await getData();
    console.log("[DeepSeekMonitor] ModelsView 数据获取成功", d ? d.models.length + " 个模型" : "无数据");
    models.value = d.models;
    console.log("[DeepSeekMonitor] ModelsView 渲染", models.value.length + " 行");
  } catch (e) {
    console.error("[DeepSeekMonitor] ModelsView 错误:", e);
    err.value = "加载失败: " + e;
  }
});
</script>

<template>
  <div class="p-2 font-sans">
    <h3 class="font-bold text-sm mb-2">按模型统计</h3>
    <div v-if="err" class="text-red-500 text-xs mb-2">{{ err }}</div>
    <table v-if="models.length" class="w-full text-xs border-collapse">
      <thead><tr class="text-center bg-gray-100 font-bold">
        <th class="text-left px-1">模型</th><th class="px-1">月请求</th><th class="px-1">月Token</th><th class="px-1">月费用</th>
        <th class="px-1">日请求</th><th class="px-1">日Token</th><th class="px-1">日费用</th>
      </tr></thead>
      <tbody>
        <tr v-for="m in models" :key="m.name" class="text-center border-b border-gray-100 even:bg-gray-50">
          <td class="text-left px-1">{{ m.name }}</td>
          <td class="px-1">{{ fmt(m.requests) }}</td><td class="px-1">{{ fmt(m.total_tokens) }}</td>
          <td class="px-1">{{ m.cost.toFixed(4) }}</td>
          <td class="px-1">{{ fmt(m.today_req) }}</td><td class="px-1">{{ fmt(m.today_tokens) }}</td>
          <td class="px-1">{{ m.today_cost.toFixed(4) }}</td>
        </tr>
      </tbody>
    </table>
    <div v-else-if="!err" class="text-gray-400 text-xs">加载中...</div>
  </div>
</template>
