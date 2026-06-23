<script setup>
import { ref, onMounted } from "vue";
import { getData, getCachedData, fmt, balance } from "./api.js";
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';

const appWindow = getCurrentWindow();
const d = ref(null);

let dragStart = null;
let dragMoved = false;

async function load() {
  try {
    d.value = await getData();
  } catch (e) {
    // token 失效：唤起主窗口弹登录，悬浮球自身不静默卡在旧数据。
    if (e === "TOKEN_INVALID" || e === "NOT_LOGGED_IN") {
      emit('focus-main');
    }
  }
}

function onDown(e) {
  dragStart = { x: e.clientX, y: e.clientY };
  dragMoved = false;
}

function onMove(e) {
  if (!dragStart || dragMoved) return;
  const dx = e.clientX - dragStart.x;
  const dy = e.clientY - dragStart.y;
  if (Math.abs(dx) > 5 || Math.abs(dy) > 5) {
    dragMoved = true;
    appWindow.startDragging();
  }
}

async function onUp() {
  if (!dragStart) return;
  const was = dragMoved;
  const p = await appWindow.outerPosition();
  dragStart = null;
  dragMoved = false;
  if (was) return;
  invoke('save_ball_pos', { x: p.x, y: p.y });
  emit('focus-main');
  appWindow.close();
}

onMounted(async () => {
  // 优先展示缓存数据，瞬时渲染，避免白屏
  const cached = await getCachedData();
  if (cached) d.value = cached;
  // 异步拉取最新数据
  load();
  // 启动轮询
  try {
    const interval = await invoke('get_refresh_interval');
    setInterval(load, interval * 1000);
  } catch {
    setInterval(load, 120000);
  }
});
</script>

<template>
  <div @pointerdown="onDown" @pointermove="onMove" @pointerup="onUp"
    class="h-screen overflow-hidden select-none flex items-center justify-center cursor-pointer"
    style="background: rgba(25, 30, 45, 0.88); border-radius: 10px;">
    <div class="text-white/95 text-[11px] leading-[15px] font-medium whitespace-nowrap px-3 py-2 text-center">
      <div>{{ d ? balance(d.balance) : '--' }}</div>
      <div class="mt-[2px]">{{ d ? balance(d.today_cost) + ' | ' + fmt(d.today_tokens) : '--' }}</div>
      <div class="mt-[2px]">{{ d ? d.today_hit + ' | ' + fmt(d.today_out_tokens) : '--' }}</div>
    </div>
  </div>
</template>
