import { ref, onMounted, onUnmounted } from 'vue'

const isDark = ref(false)

function initThemeDetection() {
  if (typeof window === 'undefined') return
  isDark.value = window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false
  const handler = (e) => { isDark.value = e.matches }
  window.matchMedia?.('(prefers-color-scheme: dark)').addEventListener('change', handler)
}

initThemeDetection()

export function useTheme() {
  return { isDark }
}

export function usePywebron() {
  const interfaces = ref(null)

  const waitForInterfaces = () => {
    return new Promise((resolve) => {
      const check = () => {
        if (window.pywebron?.interfaces) {
          interfaces.value = window.pywebron.interfaces
          resolve(interfaces.value)
        } else {
          setTimeout(check, 100)
        }
      }
      check()
    })
  }

  async function invoke(command, params = {}, timeout = 5000) {
    if (!window.pywebron?.interfaces?.invoke) return null
    return await window.pywebron.interfaces.invoke(command, params, timeout)
  }

  async function stream(name) {
    await waitForInterfaces()
    return await window.pywebron.interfaces.stream(name)
  }

  const attributes = typeof window !== 'undefined' ? window.pywebron?.attributes : null

  return { interfaces, invoke, stream, attributes, waitForInterfaces }
}

export function formatSpeed(val) {
  return val.toFixed(2) + ' KB/S'
}

export function formatTotal(val) {
  return val.toFixed(2) + ' MB'
}

const _escapeMap = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }

export function escapeHtml(t) {
  return t.replace(/[&<>"']/g, c => _escapeMap[c])
}
