<script setup>
import { ref, onMounted } from "vue";
import { getData, fmt } from "./api.js";

const models = ref([]);

onMounted(async () => {
  try { const d = await getData(); models.value = d.models; }
  catch (e) { models.value = []; }
});
</script>

<template>
  <div class="p-2 font-sans">
    <h3 class="font-bold text-sm mb-2">按模型统计</h3>
    <table class="w-full text-xs border-collapse">
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
  </div>
</template>
