<script setup>
import {SaveIcon, CpuIcon, WindowIcon, AddIcon} from 'tdesign-icons-vue-next'
import {ref} from 'vue'

const invoke = window.pywebron?.interfaces?.invoke

const saveFilesViaDialogRef = ref(false)
const executeCpuIntensiveTasksRef = ref(false)
const createNewWindowsAtRuntimeRef = ref(false)

async function saveFilesViaDialog() {
  saveFilesViaDialogRef.value = true
  const res = await invoke('save_files_via_dialog_invoke')
  if (res['data']) {
    console.log(res['mssg'], res['data'])
  }
  saveFilesViaDialogRef.value = false
}

async function executeCpuIntensiveTasks() {
  executeCpuIntensiveTasksRef.value = true
  const res = await invoke('execute_cpu_intensive_tasks_invoke')
  if (res['data']) {
    console.log(res['mssg'], res['data'])
  }
  executeCpuIntensiveTasksRef.value = false
}

async function createNewWindowsAtRuntime() {
  createNewWindowsAtRuntimeRef.value = true
  const res = await invoke('create_new_windows_at_runtime_invoke')
  if (res['data']) {
    console.log(res['mssg'], res['data'])
  }
  createNewWindowsAtRuntimeRef.value = false
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
      <t-button class="t-btn" :disabled="saveFilesViaDialogRef" @click="saveFilesViaDialog" variant="outline">
        <template #icon>
          <SaveIcon/>
        </template>
        通过对话框保存文件
      </t-button>
      <t-button class="t-btn" :disabled="executeCpuIntensiveTasksRef" @click="executeCpuIntensiveTasks"
                variant="outline">
        <template #icon>
          <CpuIcon/>
        </template>
        执行 CPU 密集任务
      </t-button>
      <t-button class="t-btn" :disabled="createNewWindowsAtRuntimeRef" @click="createNewWindowsAtRuntime"
                variant="outline">
        <template #icon>
          <AddIcon/>
        </template>
        运行时创建新窗口
      </t-button>
    </div>
  </div>
</template>

<style scoped>
.card {
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-card);
  box-sizing: border-box;
  border: 1px solid var(--border-default);
  color: var(--text-secondary);
}

.header {
  height: 36px;
  display: flex;
  align-items: center;
  background: var(--bg-card-header);
  box-sizing: border-box;
  border-bottom: 1px solid var(--border-default);
  gap: 5px;
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
  color: #065e51;
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
  align-items: center;
  justify-content: space-between;
  padding: 5px;
  box-sizing: border-box;
  background: var(--bg-card);
}

[data-theme="dark"] .body {
  background: #1a1b1d;
}

.t-button {
  height: 26px;
  padding: 0 6px !important;
  border-radius: 5px;
  font-size: 12px;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
}
</style>
