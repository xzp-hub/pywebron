/**
 * PyWebron 全局工具 Composable
 * 统一管理 pywebron API 访问和常用工具函数
 */
import { ref, onMounted, onUnmounted } from 'vue'

// 全局 pywebron 实例
export const pw = window.pywebron
export const invoke = pw?.interfaces?.invoke
export const stream = pw?.interfaces?.stream
export const attributes = pw?.attributes || {}

/**
 * 主题管理 Hook
 */
export function useThemeDetect() {
    const isDark = ref(false)
    let observer = null

    function applyTheme() {
        isDark.value = document.documentElement.getAttribute('data-theme') === 'dark'
            || window.matchMedia?.('(prefers-color-scheme: dark)').matches
    }

    onMounted(() => {
        applyTheme()
        observer = new MutationObserver(applyTheme)
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['data-theme']
        })
        window.matchMedia?.('(prefers-color-scheme: dark)')?.addEventListener('change', applyTheme)
    })

    onUnmounted(() => {
        observer?.disconnect()
        window.matchMedia?.('(prefers-color-scheme: dark)')?.removeEventListener('change', applyTheme)
    })

    return { isDark }
}

/**
 * HTML 转义工具
 */
export function escapeHtml(str) {
    if (!str) return ''
    const escapeMap = {
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        '"': '&quot;',
        "'": '&#39;'
    }
    return String(str).replace(/[&<>"']/g, c => escapeMap[c])
}

/**
 * 格式化速度（字节/秒）
 */
export function formatSpeed(bytes) {
    if (!bytes || bytes === 0) return '0 B/s'
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
    let i = 0
    while (bytes >= 1024 && i < units.length - 1) {
        bytes /= 1024
        i++
    }
    return bytes.toFixed(i > 0 ? 1 : 0) + ' ' + units[i]
}

/**
 * 格式化总量（字节）
 */
export function formatTotal(bytes) {
    if (!bytes || bytes === 0) return '0 B'
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    let i = 0
    while (bytes >= 1024 && i < units.length - 1) {
        bytes /= 1024
        i++
    }
    return bytes.toFixed(i > 0 ? 1 : 0) + ' ' + units[i]
}

/**
 * 头像缓存
 */
export const avatarCache = {
    user: 'https://api.dicebear.com/7.x/avataaars/svg?seed=user',
    bot: 'https://api.dicebear.com/7.x/bottts/svg?seed=backend'
}

// 预加载头像
if (typeof Image !== 'undefined') {
    new Image().src = avatarCache.user
    new Image().src = avatarCache.bot
}

/**
 * 消息去重管理
 */
export function useMessageDedup(maxSize = 500) {
    const msgIds = new Set()

    function isDuplicate(id) {
        if (!id || msgIds.has(id)) return true
        msgIds.add(id)
        if (msgIds.size > maxSize) {
            const iter = msgIds.values()
            msgIds.delete(iter.next().value)
        }
        return false
    }

    return { isDuplicate }
}

/**
 * Stream 连接管理
 */
export function useStream(handleId, onData, options = {}) {
    let streamInstance = null
    let retryTimer = null
    const { autoRetry = true, retryDelay = 1000 } = options

    async function connect() {
        try {
            streamInstance = await stream(handleId)
            streamInstance.recv(onData)
        } catch (e) {
            if (autoRetry) {
                retryTimer = setTimeout(connect, retryDelay)
            }
        }
    }

    onMounted(() => {
        connect()
    })

    onUnmounted(() => {
        if (retryTimer) clearTimeout(retryTimer)
        streamInstance?.close?.()
    })

    return { streamInstance, reconnect: connect }
}
