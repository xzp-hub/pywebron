<script setup>
import { ref, nextTick, onMounted } from 'vue'
import { SendIcon, ChatIcon } from 'tdesign-icons-vue-next'
import BaseCard from './BaseCard.vue'
import { useMessageDedup, escapeHtml, avatarCache, attributes } from '@/composables/usePywebron'

const chatMessages = ref([])
const chatInput = ref('')
const chatMessagesEl = ref(null)
const { isDuplicate } = useMessageDedup()
const sentMsgs = new Set()

let chatStream = null

function displayMsg(data, isLocal = false) {
  const type = data.type || data.data?.type || 'message'
  const msg = data.mssg || data.message || (data.mssg?.message) || ''
  if (!msg.trim()) return

  const id = type === 'system' ? `sys-${msg}` : `${isLocal ? 'local' : 'remote'}-${data.window_id || 'u'}-${msg}`
  if (isDuplicate(id)) return

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
  
  displayMsg({ type: 'message', message: msg, window_id: attributes?.window_id }, true)
  chatStream.send(msg)
  chatInput.value = ''
}

function onKeydown(e) {
  if (e.key === 'Enter') sendMsg()
}

async function startChat() {
  try {
    if (!window.pywebron?.interfaces?.stream) {
      setTimeout(startChat, 100)
      return
    }
    chatStream = await window.pywebron.interfaces.stream('chat_room_stream')
    chatStream.recv(displayMsg)
  } catch (e) {
    console.error('Chat stream error:', e)
  }
}

onMounted(() => {
  startChat()
})
</script>

<template>
  <BaseCard title="聊天室">
    <template #icon>
      <ChatIcon class="header-icon" />
    </template>
    
    <div class="chat-body">
      <div ref="chatMessagesEl" class="message-list">
        <div
          v-for="m in chatMessages"
          :key="m.id"
          class="message-item"
          :class="{ 
            'message-system': m.type === 'system', 
            'message-self': m.isLocal && m.type !== 'system', 
            'message-other': !m.isLocal && m.type !== 'system' 
          }"
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
    
    <template #footer>
      <div class="chat-footer">
        <t-input
          v-model="chatInput"
          placeholder="输入消息按回车发送..."
          :maxlength="200"
          @keydown="onKeydown"
        />
        <t-button class="send-button" variant="outline" @click="sendMsg" size="small">
          <template #icon>
            <SendIcon />
          </template>
        </t-button>
      </div>
    </template>
  </BaseCard>
</template>

<style scoped>
.header-icon {
  width: 16px;
  height: 16px;
  color: #00B42A;
}

.chat-body {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 0 6px;
  overflow: hidden;
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

.chat-footer {
  width: 100%;
  display: flex;
  align-items: center;
  padding: 5px;
  gap: 5px;
  box-sizing: border-box;
}

:deep(.t-input) {
  height: 26px;
}

:deep(.t-input__wrap) {
  border: none !important;
  box-shadow: none !important;
  background: var(--bg-card) !important;
}

:deep(.t-input__inner) {
  border: none !important;
  box-shadow: none !important;
}

:deep(.t-input:hover .t-input__wrap),
:deep(.t-input__wrap:hover),
:deep(.t-input.t-is-focused .t-input__wrap) {
  border: none !important;
  box-shadow: none !important;
}

.send-button {
  height: 26px;
  width: 52px;
  flex-shrink: 0;
}

.send-button:hover {
  background: #06B6D4 !important;
  color: #fff !important;
  border-color: #06B6D4 !important;
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
  padding: 0;
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
