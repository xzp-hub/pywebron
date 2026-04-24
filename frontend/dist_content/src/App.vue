<script setup>
import WindowHeader from '@/components/window_header.vue'
import SystemMonitor from '@/components/performance_chart.vue'
import IoMonitor from '@/components/network_chart.vue'
import ChatRoom from '@/components/message_panel.vue'
import InvokePanel from '@/components/action_panel.vue'
import TerminalLog from '@/components/console_viewer.vue'
</script>

<template>
  <div class="window">
    <WindowHeader/>
    <div class="window-content">
      <div class="window-top-content">
        <div class="window-left-content">
          <SystemMonitor/>
          <IoMonitor/>
        </div>
        <div class="window-right-content">
          <InvokePanel/>
          <ChatRoom/>
        </div>
      </div>
      <div class="terminal-log-wrapper">
        <TerminalLog/>
      </div>
    </div>
  </div>
</template>

<style scoped>
.window {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  flex: 1;
  gap: 5px;
  padding: 5px;
  border-radius: 6px;
  border: 1px solid var(--border-default);
  min-height: 0;
  height: 100%;
  background: var(--bg-base);
}

[data-theme="dark"] .window {
  background: #393b40;
}

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
  height: 560px;
  max-height: 560px;
  display: flex;
  flex-direction: row;
  gap: 5px;
  min-height: 0;
  flex-shrink: 0;
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

.window-right-content > :last-child {
  flex: 1;
  min-height: 0;
}

.terminal-log-wrapper {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
