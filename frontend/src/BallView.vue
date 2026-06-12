<script setup>
import { ref, onMounted } from "vue";
import { getData, fmt, balance } from "./api.js";
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { PhysicalPosition } from '@tauri-apps/api/dpi';
import { emit } from '@tauri-apps/api/event';

const appWindow = getCurrentWindow();
const d = ref(null);

let dragStart = null;
let dragMoved = false;

async function load() {
  try { d.value = await getData(); }
  catch { /* ignore */ }
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
  const pos = await invoke('load_ball_pos');
  if (pos) {
    const [x, y] = pos;
    await appWindow.setPosition(new PhysicalPosition(x, y));
  }
  load();
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
