<script setup>
import { onMounted } from 'vue'
import { usePywebron } from '@/composables/usePywebron'
import WindowHeader from '@/components/WindowHeader.vue'
import SystemMonitor from '@/components/SystemMonitor.vue'
import IoMonitor from '@/components/IoMonitor.vue'
import ChatRoom from '@/components/ChatRoom.vue'
import InvokePanel from '@/components/InvokePanel.vue'
import TerminalLog from '@/components/TerminalLog.vue'
import ResizeArea from '@/components/ResizeArea.vue'

const { attributes } = usePywebron()

onMounted(() => {
  if (attributes?.show_title_bar === true) {
    const appRoot = document.getElementById('app')
    if (appRoot) appRoot.classList.add('has-system-title-bar')
  }
})
</script>

<template>
  <div class="app-root">
    <WindowHeader />
    <div class="app-content">
      <div class="app-content-left">
        <SystemMonitor />
        <IoMonitor />
        <ChatRoom />
      </div>
      <div class="app-content-right">
        <InvokePanel />
        <TerminalLog />
      </div>
    </div>
  </div>
  <ResizeArea />
</template>

<style scoped>
.app-root {
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  padding: 5px;
  gap: 5px;
  display: flex;
  flex-direction: column;
  background: light-dark(#eeeeee, #3a3a3d);
}

.app-content {
  border-radius: 5px;
  flex: 1;
  display: flex;
  gap: 5px;
  box-sizing: border-box;
  min-height: 0;
  overflow: hidden;
}

.app-content-left {
  width: 50%;
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-height: 0;
}

.app-content-right {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-height: 0;
}
</style>
