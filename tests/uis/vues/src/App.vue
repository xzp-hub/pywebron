<script setup>
import WindowHeader from '@/components/WindowHeader.vue'
import SystemMonitor from '@/components/SystemMonitor.vue'
import IoMonitor from '@/components/IoMonitor.vue'
import ChatRoom from '@/components/ChatRoom.vue'
import InvokePanel from '@/components/InvokePanel.vue'
import TerminalLog from '@/components/TerminalLog.vue'
console.log(window.pywebron)
</script>

<template>
  <WindowHeader/>
  <div class="app-main-content">
    <div class="app-left-column">
      <SystemMonitor/>
      <IoMonitor/>
      <ChatRoom/>
    </div>
    <div class="app-right-column">
      <InvokePanel/>
      <TerminalLog/>
    </div>
  </div>
</template>

<style scoped>


.app-main-content {
  border-radius: 5px;
  flex: 1;
  display: flex;
  gap: 5px;
  box-sizing: border-box;
  min-height: 0;
  overflow: hidden;
}

.app-left-column {
  width: 50%;
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-height: 0;
}

.app-right-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-height: 0;
}
</style>
