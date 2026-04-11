<script setup>
import { ref } from 'vue'
import { usePywebron } from '@/composables/usePywebron'
import { DownloadIcon, ThunderboltIcon, WindowIcon } from 'tdesign-icons-vue-next'

const { invoke } = usePywebron()

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
  <div class="panel invoke-panel">
    <div class="panel-body invoke-panel-button-row">
      <t-button class="invoke-panel-action invoke-panel-download" :disabled="downloadDisabled" @click="downloadFile" theme="primary">
        <template #icon><DownloadIcon /></template>
        下载文件
      </t-button>
      <t-button class="invoke-panel-action invoke-panel-cpu" :disabled="cpuTaskDisabled" @click="runCpuTask" theme="success">
        <template #icon><ThunderboltIcon /></template>
        执行 CPU 密集任务
      </t-button>
      <t-button class="invoke-panel-action invoke-panel-window" :disabled="createWindowDisabled" @click="createNewWindow" theme="warning">
        <template #icon><WindowIcon /></template>
        创建新窗口
      </t-button>
    </div>
  </div>
</template>

<style scoped>
.panel {
  border-radius: 5px;
  border: 1px solid light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: light-dark(#ffffff, #1e1f21);
  box-sizing: border-box;
}

.panel-body {
  flex: 1;
  padding: 5px;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.invoke-panel {
  flex: 0 0 auto;
}

.invoke-panel-button-row {
  padding: 0;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}

.invoke-panel-action {
  flex: 1;
  height: 36px;
  border-radius: 0;
  font-weight: 600;
  font-size: 13px;
  gap: 5px;
  border: none;
  border-right: 1px solid light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .25));
}

.invoke-panel-action:last-child {
  border-right: none;
}

.invoke-panel-download {
  background: #165DFF;
  color: #fff;
}

.invoke-panel-cpu {
  background: #7BE188;
  color: rgba(0, 0, 0, .75);
}

.invoke-panel-window {
  background: #722ED1;
  color: #fff;
}
</style>
