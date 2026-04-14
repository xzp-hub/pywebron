<script setup>
import { ref } from 'vue'
import { DownloadIcon, ThunderIcon, WindowIcon } from 'tdesign-icons-vue-next'

const pw = window.pywebron
const invoke = pw?.interfaces?.invoke

const downloadDisabled = ref(false)
const cpuTaskDisabled = ref(false)
const createWindowDisabled = ref(false)

async function downloadFile() {
  downloadDisabled.value = true
  try {
    await invoke('file_download_invoke')
  } catch (e) { /* noop */ }
  finally { downloadDisabled.value = false }
}

async function runCpuTask() {
  cpuTaskDisabled.value = true
  try {
    await invoke('cpu_intensive_task_invoke_command')
  } catch (e) { /* noop */ }
  finally { cpuTaskDisabled.value = false }
}

async function createNewWindow() {
  createWindowDisabled.value = true
  try {
    await invoke('running_create_window_invoke_handle')
  } catch (e) { /* noop */ }
  finally { createWindowDisabled.value = false }
}
</script>

<template>
  <div class="card">
    <div class="header">
      <div class="header-icon-box">
        <WindowIcon class="header-icon"/>
      </div>
      <span class="header-title">快捷操作</span>
    </div>
    <div class="body">
      <t-button class="action-btn action-btn-primary" :disabled="downloadDisabled" @click="downloadFile" theme="primary">
        <template #icon><DownloadIcon /></template>
        下载文件
      </t-button>
      <t-button class="action-btn action-btn-success" :disabled="cpuTaskDisabled" @click="runCpuTask" theme="success">
        <template #icon><ThunderIcon /></template>
        执行 CPU 密集任务
      </t-button>
      <t-button class="action-btn action-btn-warning" :disabled="createWindowDisabled" @click="createNewWindow" theme="warning">
        <template #icon><WindowIcon /></template>
        创建新窗口
      </t-button>
    </div>
  </div>
</template>

<style lang="scss" scoped>
@use 'assets/themes/mixins' as *;

.card {
  @include card-base;
  height: auto;
  flex: none;
}

.header {
  @include card-header-base;
  display: flex;
  padding-left: 6px;
  gap: 5px;
}

.header-icon-box {
  @include icon-box;
  width: auto;
}

.header-icon {
  @include icon-base;
  color: #722ED1;
}

.header-title {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1;
}

[data-theme="dark"] .header-title {
  color: #ffffff;
}

.body {
  display: flex;
  gap: 6px;
  padding: 8px;
  box-sizing: border-box;
  flex-shrink: 0;
  background: var(--bg-card);
}

[data-theme="dark"] .body {
  background: #1a1b1d;
}

.action-btn {
  flex: 1;
  height: 30px;
  margin-top: 3px;
  border-radius: 5px !important;
  font-weight: 600;
  font-size: 13px;
  gap: 4px;
  border: none !important;
}
</style>
