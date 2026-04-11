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
  <div class="invoke-panel">
    <div class="invoke-panel-action-bar">
      <t-button class="invoke-panel-action-btn invoke-panel-download-btn" :disabled="downloadDisabled" @click="downloadFile" theme="primary">
        <template #icon><DownloadIcon /></template>
        下载文件
      </t-button>
      <t-button class="invoke-panel-action-btn invoke-panel-cpu-btn" :disabled="cpuTaskDisabled" @click="runCpuTask" theme="success">
        <template #icon><ThunderIcon /></template>
        执行 CPU 密集任务
      </t-button>
      <t-button class="invoke-panel-action-btn invoke-panel-window-btn" :disabled="createWindowDisabled" @click="createNewWindow" theme="warning">
        <template #icon><WindowIcon /></template>
        创建新窗口
      </t-button>
    </div>
  </div>
</template>

<style scoped>
.invoke-panel {
  border-radius: 5px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: light-dark(#ffffff, #1e1f21);
  box-sizing: border-box;
  box-shadow: inset 0 0 0 1px light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
  flex: 0 0 auto;
}

.invoke-panel-action-bar {
  padding: 0;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}

.invoke-panel-action-btn {
  flex: 1;
  height: 36px;
  border-radius: 0 !important;
  font-weight: 600;
  font-size: 13px;
  gap: 5px;
  border: none !important;
  box-shadow: inset -1px 0 0 0 light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .25)) !important;
}

.invoke-panel-action-btn:last-child {
  box-shadow: none !important;
}

.invoke-panel-download-btn {
  background: #165DFF !important;
  color: #fff !important;
}

.invoke-panel-cpu-btn {
  background: #7BE188 !important;
  color: rgba(0, 0, 0, .75) !important;
}

.invoke-panel-window-btn {
  background: #722ED1 !important;
  color: #fff !important;
}
</style>
