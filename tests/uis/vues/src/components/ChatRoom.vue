<script setup>
import {ref, onMounted, onUnmounted, nextTick} from 'vue'
import {SendIcon, ChatIcon} from 'tdesign-icons-vue-next'
import {Button} from 'tdesign-vue-next'

const isDark = ref(false)
const pw = window.pywebron
const attributes = pw?.attributes || {}
const stream = pw?.interfaces?.stream

// 主题切换
function applyTheme() {
  isDark.value = document.documentElement.getAttribute('data-theme') === 'dark'
      || window.matchMedia?.('(prefers-color-scheme: dark)').matches
}

onMounted(() => {
  applyTheme()
  const observer = new MutationObserver(applyTheme)
  observer.observe(document.documentElement, {attributes: true, attributeFilter: ['data-theme']})
})

function escapeHtml(str) {
  if (!str) return ''
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

const chatMessages = ref([])
const chatInput = ref('')
const chatMessagesEl = ref(null)
const msgIds = new Set()
const sentMsgs = new Set()

const avatarCache = {
  user: 'https://api.dicebear.com/7.x/avataaars/svg?seed=user',
  bot: 'https://api.dicebear.com/7.x/bottts/svg?seed=backend'
}

new Image().src = avatarCache.user
new Image().src = avatarCache.bot

let chatStream = null
let retryTimer = null

function displayMsg(data, isLocal = false) {
  const type = data.type || data.data?.type || 'message'
  const msg = data.mssg || data.message || (data.mssg?.message) || ''
  if (!msg.trim()) return

  const id = type === 'system' ? `sys-${msg}` : `${isLocal ? 'local' : 'remote'}-${data.window_id || 'u'}-${msg}`
  if (id && msgIds.has(id)) return
  if (id) {
    msgIds.add(id)
    if (msgIds.size > 500) {
      const iter = msgIds.values()
      msgIds.delete(iter.next().value)
    }
  }

  chatMessages.value.push({
    id: Date.now() + Math.random(),
    type,
    isLocal,
    msg: escapeHtml(msg),
    avatar: isLocal ? avatarCache.user : avatarCache.bot
  })

  nextTick(() => {
    if (chatMessagesEl.value) {
      chatMessagesEl.value.scrollTop = chatMessagesEl.value.scrollHeight
    }
  })
}

function sendMsg() {
  const msg = chatInput.value.trim()
  if (!msg || !chatStream?.send) return
  const sendId = `sent-${Date.now()}-${msg}`
  if (sentMsgs.has(sendId)) return
  sentMsgs.add(sendId)
  displayMsg({type: 'message', message: msg, window_id: attributes?.window_id}, true)
  chatStream.send(msg)
  chatInput.value = ''
}

function onKeydown(e) {
  if (e.key === 'Enter') sendMsg()
}

async function startChat() {
  try {
    chatStream = await stream('chat_room_stream')
    chatStream.recv(displayMsg)
  } catch (e) { /* noop */
  }
}

onMounted(() => {
  startChat()
})

onUnmounted(() => {
  if (retryTimer) clearTimeout(retryTimer)
})
</script>

<template>
  <div class="card">
    <div class="header">
      <div class="header-icon-box">
        <ChatIcon class="header-icon"/>
      </div>
      <span class="header-title">聊天室</span>
    </div>
    <div class="body">
      <div ref="chatMessagesEl" class="message-list">
        <div
            v-for="m in chatMessages"
            :key="m.id"
            class="message-item"
            :class="{ 'message-system': m.type === 'system', 'message-self': m.isLocal && m.type !== 'system', 'message-other': !m.isLocal && m.type !== 'system' }"
        >
          <template v-if="m.type === 'system'">
            <div class="message-bubble" v-html="m.msg"></div>
          </template>
          <template v-else>
            <div class="message-row" :class="{ 'message-row-self': m.isLocal, 'message-row-other': !m.isLocal }">
              <template v-if="!m.isLocal">
                <img class="user-avatar" :src="m.avatar">
                <div class="message-bubble" v-html="m.msg"></div>
              </template>
              <template v-else>
                <div class="message-bubble" v-html="m.msg"></div>
                <img class="user-avatar" :src="m.avatar">
              </template>
            </div>
          </template>
        </div>
      </div>
    </div>
    <div class="footer">
      <t-input
          v-model="chatInput"
          placeholder="输入消息按回车发送..."
          :maxlength="200"
          @keydown="onKeydown"
      />
      <t-button class="send-button" variant="outline" @click="sendMsg" size="small">
        <template #icon>
          <SendIcon/>
        </template>
      </t-button>
    </div>
  </div>
</template>

<style scoped>

.card {
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-card);
  box-sizing: border-box;
  border: 1px solid var(--border-default);
  height: auto;
  flex: 1;
  overflow: hidden;
}

.header {
  height: 36px;
  display: flex;
  align-items: center;
  background: var(--bg-card-header);
  box-sizing: border-box;
  border-bottom: 1px solid var(--border-default);
  display: flex;
  padding-left: 6px;
  gap: 5px;
}

.header-icon-box {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: auto;
}

.header-icon {
  width: 16px;
  height: 16px;
  color: #00B42A;
}

.header-title {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1;
}

[data-theme="dark"] .header-title {
  color: #ffffff;
}

.body {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-card);
  min-height: 0;
  overflow: hidden;
  padding: 5px;
}

[data-theme="dark"] .body {
  background: #1a1b1d;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.message-list::-webkit-scrollbar {
  width: 4px;
}

.message-list::-webkit-scrollbar-thumb {
  background: var(--log-scrollbar);
  border-radius: 5px;
}

.footer {
  height: 36px;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  box-sizing: border-box;
  border-top: 1px solid var(--border-default);
  padding: 5px;
  gap: 5px;
}


:deep(.t-input) {
  height: 26px;
}

.send-button {
  height: 26px;
  width: 52px;
  flex-shrink: 0;
  position: relative;
  z-index: 2;
  display: inline-flex !important;
  align-items: center !important;
  justify-content: center !important;
}

.message-item {
  display: flex;
  flex-direction: column;
  max-width: calc(100% - 5px);
  animation: fade-up .3s;
}

@keyframes fade-up {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.message-system {
  align-self: center;
  max-width: 100%;
}

.message-other {
  align-self: flex-start;
}

.message-self {
  align-self: flex-end;
}

.message-row {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 0 5px;
}

.message-row-self {
  justify-content: flex-end;
}

.user-avatar {
  width: 30px;
  height: 30px;
  border-radius: 5px;
  background: var(--avatar-bg);
  flex-shrink: 0;
  box-shadow: inset 0 0 0 1px var(--border-input);
}

.message-bubble {
  padding: 5px;
  border-radius: 5px;
  font-size: 13px;
  line-height: 20px;
  word-break: break-word;
}

.message-system .message-bubble {
  color: var(--text-system);
  font-size: 12px;
  font-style: italic;
  text-align: center;
}

.message-row-other .message-bubble {
  background: var(--bubble-other-bg);
  border: none;
  color: var(--bubble-text);
}

.message-row-self .message-bubble {
  background: var(--bubble-self-bg);
  border: none;
  color: var(--bubble-text);
}
</style>
