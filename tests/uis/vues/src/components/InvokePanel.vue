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

<style scoped>
.card {
  height: auto;
  flex: none;
  display: flex;
  border-radius: 6px;
  flex-direction: column;
  overflow: hidden;
  background: light-dark(#ffffff, #1e1f21);
  box-sizing: border-box;
  border: 1px solid light-dark(rgba(0, 0, 0, .2), rgba(255, 255, 255, .2));
}

.header {
  height: 30px;
  display: flex;
  align-items: center;
  background: light-dark(#ffffff, rgba(184, 183, 183, .15));
  box-sizing: border-box;
  border-bottom: 1px solid light-dark(rgba(0, 0, 0, .2), rgba(255, 255, 255, .2));
}

.header-icon-box {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.header-icon {
  width: 16px;
  height: 16px;
  color: #722ED1;
}

.header-title {
  font-size: 14px;
  color: light-dark(#5e5e5e, #fff);
  line-height: 1;
}

.body {
  display: flex;
  gap: 6px;
  padding: 8px;
  box-sizing: border-box;
  flex-shrink: 0;
}

.action-btn {
  flex: 1;
  height: 36px;
  border-radius: 5px !important;
  font-weight: 600;
  font-size: 13px;
  gap: 4px;
  border: none !important;
}
</style>
