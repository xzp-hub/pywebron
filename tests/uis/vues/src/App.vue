<script setup>
import WindowHeader from '@/components/WindowHeader.vue'
import SystemMonitor from '@/components/SystemMonitor.vue'
import IoMonitor from '@/components/IoMonitor.vue'
import ChatRoom from '@/components/ChatRoom.vue'
import InvokePanel from '@/components/InvokePanel.vue'
import TerminalLog from '@/components/TerminalLog.vue'
</script>

<template>
  <WindowHeader/>
  <div class="window-content">
    <div class="window-top-content">
      <div class="window-left-content">
        <SystemMonitor/>
        <IoMonitor/>
      </div>
      <div class="window-right-content">
        <ChatRoom/>
        <InvokePanel/>
      </div>
    </div>
    <TerminalLog/>
  </div>
</template>

<style scoped>
.window-content {
  border-radius: 5px;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
  box-sizing: border-box;
  min-height: 0;
  overflow: hidden;
}

.window-top-content {
  flex: 1;
  display: flex;
  flex-direction: row;
  gap: 5px;
  min-height: 0;
}

.window-left-content {
  width: 50%;
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-height: 0;
}



.window-right-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-height: 0;
}

.window-right-content > * {
  flex: 1;
  min-height: 0;
}
</style>
