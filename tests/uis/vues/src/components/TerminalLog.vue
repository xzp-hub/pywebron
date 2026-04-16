<script setup>
import { ref } from 'vue'
import { TerminalIcon } from 'tdesign-icons-vue-next'
import BaseCard from './BaseCard.vue'
import { useThemeDetect, useStream } from '@/composables/usePywebron'

const { isDark } = useThemeDetect()
const terminalLogsEl = ref(null)

function getLogColor(text, dark) {
  if (text.includes('[Error]') || text.includes('[error]') || text.includes('Exception') || text.includes('Traceback')) {
    return dark ? '#ff6b6b' : '#c0392b'
  } else if (text.includes('[Performance]')) {
    return dark ? '#ffffff' : 'rgba(0,0,0,0.75)'
  } else if (text.includes('[Window]') || text.includes('[IPC]') || text.includes('[Stream]') || text.includes('[Invoke]') || text.includes('[Timing]')) {
    return dark ? '#ffffff' : 'rgba(0,0,0,0.7)'
  } else if (text.includes('[Warning]') || text.includes('警告')) {
    return dark ? '#ffffff' : 'rgba(0,0,0,0.65)'
  }
  return dark ? '#ffffff' : 'rgba(0,0,0,0.75)'
}

function handleLogData(data) {
  if (!terminalLogsEl.value) return
  const logs = data.data?.logs || data.logs || []
  const frag = document.createDocumentFragment()
  const dark = isDark.value
  
  logs.forEach(log => {
    const line = document.createElement('div')
    const text = typeof log === 'string' ? log : JSON.stringify(log)
    line.style.color = getLogColor(text, dark)
    line.textContent = text
    frag.appendChild(line)
  })
  
  terminalLogsEl.value.appendChild(frag)
  terminalLogsEl.value.scrollTop = terminalLogsEl.value.scrollHeight
}

useStream('terminal_log_stream', handleLogData, { autoRetry: false })
</script>

<template>
  <BaseCard title="终端日志">
    <template #icon>
      <TerminalIcon class="header-icon" />
    </template>
    <div ref="terminalLogsEl" class="content-area"></div>
  </BaseCard>
</template>

<style scoped>
.header-icon {
  width: 16px;
  height: 16px;
  color: #F7BA1E;
}

.content-area {
  height: 100%;
  padding: 5px;
  overflow-y: auto;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-log-normal);
}

.content-area::-webkit-scrollbar {
  width: 4px;
}

.content-area::-webkit-scrollbar-thumb {
  background: var(--log-scrollbar);
  border-radius: 5px;
}
</style>
