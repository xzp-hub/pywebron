<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { usePywebron, useTheme } from '@/composables/usePywebron'

const { isDark } = useTheme()
const { stream } = usePywebron()

const terminalLogsEl = ref(null)
let retryTimer = null

async function startTerminalLog() {
  try {
    const terminalStream = await stream('terminal_log_stream')
    terminalStream.recv((data) => {
      if (!terminalLogsEl.value) return
      const logs = data.data?.logs || data.logs || []
      const frag = document.createDocumentFragment()
      const dark = isDark.value
      logs.forEach(log => {
        const line = document.createElement('div')
        let text = typeof log === 'string' ? log : JSON.stringify(log)
        if (text.includes('[Error]') || text.includes('[error]') || text.includes('Exception') || text.includes('Traceback')) {
          line.style.color = dark ? '#ff6b6b' : '#c0392b'
        } else if (text.includes('[Performance]')) {
          line.style.color = dark ? 'rgba(255,255,255,0.85)' : 'rgba(0,0,0,0.75)'
        } else if (text.includes('[Window]') || text.includes('[IPC]') || text.includes('[Stream]') || text.includes('[Invoke]') || text.includes('[Timing]')) {
          line.style.color = dark ? 'rgba(255,255,255,0.8)' : 'rgba(0,0,0,0.7)'
        } else if (text.includes('[Warning]') || text.includes('警告')) {
          line.style.color = dark ? 'rgba(255,255,255,0.75)' : 'rgba(0,0,0,0.65)'
        } else {
          line.style.color = dark ? 'rgba(255,255,255,0.8)' : 'rgba(0,0,0,0.75)'
        }
        line.textContent = text
        frag.appendChild(line)
      })
      terminalLogsEl.value.appendChild(frag)
      terminalLogsEl.value.scrollTop = terminalLogsEl.value.scrollHeight
    })
  } catch (e) { /* noop */ }
}

onMounted(() => {
  startTerminalLog()
})

onUnmounted(() => {
  if (retryTimer) clearTimeout(retryTimer)
})
</script>

<template>
  <div class="panel terminal-log-panel">
    <div class="panel-header">
      <div class="panel-header-icon-wrapper">
        <svg class="panel-header-icon terminal-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="4 17 10 11 4 5"/>
          <line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
      </div>
      <span class="panel-header-text">终端日志</span>
    </div>
    <div ref="terminalLogsEl" class="panel-body terminal-log-content"></div>
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

.panel-header {
  height: 30px;
  display: flex;
  align-items: center;
  gap: 5px;
  background: light-dark(#ffffff, rgba(184, 183, 183, .15));
  backdrop-filter: blur(6px);
  border-bottom: 1px solid light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .35));
  border-radius: 5px 5px 0 0;
  box-sizing: border-box;
  padding: 0 5px;
}

.panel-header-icon-wrapper {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.panel-header-icon {
  width: 16px;
  height: 16px;
}

.terminal-icon {
  color: #F7BA1E;
}

.panel-header-text {
  font-size: 12px;
  font-weight: 600;
  color: light-dark(#333, #fff);
  letter-spacing: .5px;
  line-height: 1;
}

.panel-body {
  flex: 1;
  padding: 5px;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.terminal-log-panel {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.terminal-log-content {
  flex: 1;
  min-height: 0;
  padding: 5px;
  overflow-y: auto;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: light-dark(rgba(0, 0, 0, .7), rgba(255, 255, 255, 0.8));
  background: light-dark(#ffffff, rgba(30, 31, 33, 0.6));
}

.terminal-log-content::-webkit-scrollbar {
  width: 4px;
}

.terminal-log-content::-webkit-scrollbar-thumb {
  background: rgba(100, 100, 255, .3);
  border-radius: 5px;
}
</style>
