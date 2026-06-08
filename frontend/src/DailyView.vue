<script setup>
import { ref, onMounted } from "vue";
import { getData, fmt } from "./api.js";

const daily = ref([]);

onMounted(async () => {
  try { const d = await getData(); daily.value = d.daily; }
  catch (e) { daily.value = []; }
});
</script>

<template>
  <div class="p-2 font-sans">
    <h3 class="font-bold text-sm mb-2">按日统计</h3>
    <table class="w-full text-xs border-collapse">
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
  </div>
</template>
