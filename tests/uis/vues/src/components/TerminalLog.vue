<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { TerminalIcon } from 'tdesign-icons-vue-next'

const isDark = ref(false)
const pw = window.pywebron
const stream = pw?.interfaces?.stream

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
  <div class="terminal-log-panel">
    <div class="terminal-log-panel-header">
      <div class="terminal-log-header-icon-box">
        <TerminalIcon class="terminal-log-header-icon" />
      </div>
      <span class="terminal-log-header-title">终端日志</span>
    </div>
    <div ref="terminalLogsEl" class="terminal-log-content-area"></div>
  </div>
</template>

<style scoped>
.terminal-log-panel {
  border-radius: 5px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: light-dark(#ffffff, #1e1f21);
  box-sizing: border-box;
  box-shadow: inset 0 0 0 1px light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
  flex: 1;
  min-height: 0;
}

.terminal-log-panel-header {
  height: 30px;
  display: flex;
  align-items: center;
  gap: 5px;
  background: light-dark(#ffffff, rgba(184, 183, 183, .15));
  backdrop-filter: blur(6px);
  border-radius: 5px 5px 0 0;
  box-sizing: border-box;
  padding: 0 5px;
  box-shadow: inset 0 -1px 0 0 light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .35));
}

.terminal-log-header-icon-box {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.terminal-log-header-icon {
  width: 16px;
  height: 16px;
  color: #F7BA1E;
}

.terminal-log-header-title {
  font-size: 12px;
  font-weight: 600;
  color: light-dark(#333, #fff);
  letter-spacing: .5px;
  line-height: 1;
}

.terminal-log-content-area {
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

.terminal-log-content-area::-webkit-scrollbar {
  width: 4px;
}

.terminal-log-content-area::-webkit-scrollbar-thumb {
  background: rgba(100, 100, 255, .3);
  border-radius: 5px;
}
</style>
