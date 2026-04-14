<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { TerminalIcon } from 'tdesign-icons-vue-next'

const isDark = ref(false)
const pw = window.pywebron
const stream = pw?.interfaces?.stream

// 主题切换
function applyTheme() {
  isDark.value = document.documentElement.getAttribute('data-theme') === 'dark'
    || window.matchMedia?.('(prefers-color-scheme: dark)').matches
  
  // 更新已存在的日志颜色
  if (terminalLogsEl.value) {
    const dark = isDark.value
    const lines = terminalLogsEl.value.children
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]
      const text = line.textContent || ''
      if (text.includes('[Error]') || text.includes('[error]') || text.includes('Exception') || text.includes('Traceback')) {
        line.style.color = dark ? '#ff6b6b' : '#c0392b'
      } else if (text.includes('[Performance]')) {
        line.style.color = dark ? '#ffffff' : 'rgba(0,0,0,0.75)'
      } else if (text.includes('[Window]') || text.includes('[IPC]') || text.includes('[Stream]') || text.includes('[Invoke]') || text.includes('[Timing]')) {
        line.style.color = dark ? '#ffffff' : 'rgba(0,0,0,0.7)'
      } else if (text.includes('[Warning]') || text.includes('警告')) {
        line.style.color = dark ? '#ffffff' : 'rgba(0,0,0,0.65)'
      } else {
        line.style.color = dark ? '#ffffff' : 'rgba(0,0,0,0.75)'
      }
    }
  }
}

onMounted(() => {
  applyTheme()
  const observer = new MutationObserver(applyTheme)
  observer.observe(document.documentElement, {attributes: true, attributeFilter: ['data-theme']})
})

const terminalLogsEl = ref(null)
let retryTimer = null

async function startTerminalLog() {
  try {
    const terminalStream = await stream('terminal_log_stream')
    terminalStream.recv((data) => {
      if (!terminalLogsEl.value) return
      const logs = data.data?.logs || data.logs || []
      const frag = document.createDocumentFragment()
      // 每次接收日志时都检查当前主题
      const dark = document.documentElement.getAttribute('data-theme') === 'dark'
      logs.forEach(log => {
        const line = document.createElement('div')
        let text = typeof log === 'string' ? log : JSON.stringify(log)
        if (text.includes('[Error]') || text.includes('[error]') || text.includes('Exception') || text.includes('Traceback')) {
          line.style.color = dark ? '#ff6b6b' : '#c0392b'
        } else if (text.includes('[Performance]')) {
          line.style.color = dark ? '#ffffff' : 'rgba(0,0,0,0.75)'
        } else if (text.includes('[Window]') || text.includes('[IPC]') || text.includes('[Stream]') || text.includes('[Invoke]') || text.includes('[Timing]')) {
          line.style.color = dark ? '#ffffff' : 'rgba(0,0,0,0.7)'
        } else if (text.includes('[Warning]') || text.includes('警告')) {
          line.style.color = dark ? '#ffffff' : 'rgba(0,0,0,0.65)'
        } else {
          line.style.color = dark ? '#ffffff' : 'rgba(0,0,0,0.75)'
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

// 辅助函数：读取 CSS 变量值，带 fallback
function varToColor(varName, fallback) {
  if (typeof getComputedStyle !== 'undefined') {
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim() || fallback
  }
  return fallback
}
</script>

<template>
  <div class="card">
    <div class="header">
      <div class="header-icon-box">
        <TerminalIcon class="header-icon" />
      </div>
      <span class="header-title">终端日志</span>
    </div>
    <div ref="terminalLogsEl" class="content-area"></div>
  </div>
</template>

<style lang="scss" scoped>
@use 'assets/themes/mixins' as *;

.card {
  @include card-base;
  height: auto;
  flex: 1;
}

.header {
  @include card-header-base;
  display: flex;
  padding-left: 6px;
  gap: 5px;
}

.header-icon-box {
  @include icon-box;
  width: auto;
}

.header-icon {
  @include icon-base;
  color: #F7BA1E;
}

.header-title {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1;
}

[data-theme="dark"] .header-title {
  color: #ffffff;
}

.content-area {
  flex: 1;
  min-height: 0;
  padding: 5px;
  overflow-y: auto;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-log-normal);
  background: var(--bg-card);
}

[data-theme="dark"] .content-area {
  background: #1a1b1d;
  color: #ffffff;
}

.content-area::-webkit-scrollbar {
  width: 4px;
}

.content-area::-webkit-scrollbar-thumb {
  background: var(--log-scrollbar);
  border-radius: 5px;
}
</style>
