<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { usePywebron, useTheme, escapeHtml } from '@/composables/usePywebron'
import { SendIcon } from 'tdesign-icons-vue-next'

const { isDark } = useTheme()
const { stream, attributes } = usePywebron()

const chatMessages = ref([])
const showWelcome = ref(true)
const chatInput = ref('')
const chatMessagesEl = ref(null)
const msgIds = new Set()

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

  if (type === 'system' && msg === '欢迎加入聊天室') showWelcome.value = false

  const id = type === 'system' ? `sys-${msg}` : `${isLocal ? 'local' : 'remote'}-${data.window_id || 'u'}-${Date.now()}-${msg}`
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
  displayMsg({ type: 'message', message: msg, window_id: attributes?.window_id }, true)
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
  } catch (e) { /* noop */ }
}

onMounted(() => {
  startChat()
})

onUnmounted(() => {
  if (retryTimer) clearTimeout(retryTimer)
})
</script>

<template>
  <div class="panel chat-room-panel">
    <div class="panel-header">
      <div class="panel-header-icon-wrapper">
        <svg class="panel-header-icon chat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
        </svg>
      </div>
      <span class="panel-header-text">聊天室</span>
    </div>
    <div class="panel-body chat-room-message-area">
      <div v-if="showWelcome" class="chat-room-welcome">欢迎加入聊天室</div>
      <div ref="chatMessagesEl" class="chat-room-message-list">
        <div
          v-for="m in chatMessages"
          :key="m.id"
          class="chat-room-message-item"
          :class="{ 'chat-room-message-system': m.type === 'system', 'chat-room-message-self': m.isLocal && m.type !== 'system', 'chat-room-message-other': !m.isLocal && m.type !== 'system' }"
        >
          <template v-if="m.type === 'system'">
            <div class="chat-room-message-bubble" v-html="m.msg"></div>
          </template>
          <template v-else>
            <div class="chat-room-message-row" :class="{ 'chat-room-message-row-self': m.isLocal, 'chat-room-message-row-other': !m.isLocal }">
              <template v-if="!m.isLocal">
                <img class="chat-room-avatar" :src="m.avatar">
                <div class="chat-room-message-bubble" v-html="m.msg"></div>
              </template>
              <template v-else>
                <div class="chat-room-message-bubble" v-html="m.msg"></div>
                <img class="chat-room-avatar" :src="m.avatar">
              </template>
            </div>
          </template>
        </div>
      </div>
    </div>
    <div class="chat-room-input-bar">
      <input
        type="text"
        v-model="chatInput"
        class="chat-room-input-field"
        placeholder="输入消息按回车发送..."
        maxlength="200"
        @keydown="onKeydown"
      >
      <t-button class="chat-room-send-button" theme="primary" @click="sendMsg">
        <template #icon><SendIcon /></template>
      </t-button>
    </div>
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

.chat-icon {
  color: #00B42A;
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

.chat-room-panel {
  flex: 1;
}

.chat-room-message-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: light-dark(#ffffff, rgba(30, 31, 33, 0.6));
  min-height: 0;
  overflow: hidden;
}

.chat-room-welcome {
  padding: 5px;
  font-size: 13px;
  color: light-dark(rgba(0, 0, 0, .45), rgba(255, 255, 255, .7));
  text-align: center;
  flex-shrink: 0;
}

.chat-room-message-list {
  flex: 1;
  overflow-y: auto;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.chat-room-message-list::-webkit-scrollbar {
  width: 4px;
}

.chat-room-message-list::-webkit-scrollbar-thumb {
  background: rgba(100, 100, 255, .3);
  border-radius: 5px;
}

.chat-room-input-bar {
  height: 30px;
  border-top: 1px solid light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .35));
  display: flex;
  flex-shrink: 0;
}

.chat-room-input-field {
  flex: 1;
  height: 30px;
  padding: 0 5px;
  border: none;
  border-radius: 0;
  background: transparent;
  color: light-dark(rgba(0, 0, 0, .8), rgba(255, 255, 255, 0.8));
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
}

.chat-room-input-field:focus {
  outline: none;
}

.chat-room-send-button {
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 0;
  flex-shrink: 0;
}

.chat-room-message-item {
  display: flex;
  flex-direction: column;
  max-width: calc(100% - 5px);
  animation: chat-room-fade-up .3s;
}

@keyframes chat-room-fade-up {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.chat-room-message-system {
  align-self: center;
  max-width: 100%;
}

.chat-room-message-other {
  align-self: flex-start;
}

.chat-room-message-self {
  align-self: flex-end;
}

.chat-room-message-row {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 0 5px;
}

.chat-room-message-row-self {
  justify-content: flex-end;
}

.chat-room-avatar {
  width: 30px;
  height: 30px;
  border-radius: 5px;
  background: light-dark(#f0f0f0, rgba(255, 255, 255, .1));
  border: 1px solid light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
  flex-shrink: 0;
}

.chat-room-message-bubble {
  padding: 5px;
  border-radius: 5px;
  font-size: 13px;
  line-height: 20px;
  word-break: break-word;
}

.chat-room-message-system .chat-room-message-bubble {
  color: light-dark(rgba(0, 0, 0, .5), rgba(255, 255, 255, .7));
  font-size: 12px;
  font-style: italic;
  text-align: center;
}

.chat-room-message-row-other .chat-room-message-bubble {
  background: #e5e6eb;
  border: none;
  color: rgba(0, 0, 0, .75);
}

.chat-room-message-row-self .chat-room-message-bubble {
  background: #7BE188;
  border: none;
  color: rgba(0, 0, 0, .75);
}
</style>
