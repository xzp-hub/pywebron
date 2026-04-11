<script setup>
import { ref, onMounted } from 'vue'
import { usePywebron } from '@/composables/usePywebron'

const isMaximized = ref(false)
const { invoke, attributes } = usePywebron()

const windowAction = async (type) => {
  const map = {
    min: 'minimize_window',
    max: 'maximize_window',
    rep: 'reappear_window',
    shut: 'shutdown_window'
  }
  const action = type === 'toggle' ? (isMaximized.value ? 'rep' : 'max') : type
  try {
    const res = await invoke('window_controls_invoke', { control_type: map[action] }, 5000)
    if (res?.code === 200) {
      isMaximized.value = action === 'max' || (type === 'toggle' && !isMaximized.value)
    }
  } catch (e) { /* noop */ }
}

const showSystemTitleBar = attributes?.show_title_bar === true
const iconSrc = ref('')
const titleText = ref('PyWebron 控制面板')

onMounted(async () => {
  if (attributes?.icon_path) {
    const iconPath = attributes.icon_path.replace(/\\/g, '/')
    const fileName = iconPath.split('/').pop()
    if (fileName) iconSrc.value = 'http://app.' + fileName
  }
  if (attributes?.title) {
    titleText.value = attributes.title
  }
  try {
    await invoke('setup_drag_region_invoke', { selector: '#windowHeader' })
  } catch (e) {
    console.error(e)
  }
})
</script>

<template>
  <div id="windowHeader" class="window-header">
    <div class="window-header-left">
      <img v-if="iconSrc" class="window-header-icon" :src="iconSrc" alt="">
      <span class="window-header-title">{{ titleText }}</span>
    </div>
    <div class="window-header-controls">
      <div class="window-header-button" title="最小化" @click="windowAction('min')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="4" y1="12" x2="20" y2="12"/>
        </svg>
      </div>
      <div class="window-header-button" :title="isMaximized ? '还原' : '最大化'" @click="windowAction('toggle')">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <template v-if="isMaximized">
            <rect x="4" y="8" width="12" height="12" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
            <rect x="8" y="4" width="12" height="12" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </template>
          <template v-else>
            <rect x="4" y="4" width="16" height="16" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </template>
        </svg>
      </div>
      <div class="window-header-button window-header-close" title="关闭" @click="windowAction('shut')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.window-header {
  width: 100%;
  height: 30px;
  border-radius: 5px;
  border: 1px solid light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: light-dark(#ffffff, #1a1b1d);
}

.window-header-left {
  display: flex;
  align-items: center;
  padding-left: 5px;
}

.window-header-icon {
  width: 16px;
  height: 16px;
  margin-right: 5px;
  border-radius: 5px;
  object-fit: contain;
  flex-shrink: 0;
}

.window-header-title {
  font-size: 12px;
  color: light-dark(#000000, #ffffff);
  line-height: 16px;
}

.window-header-controls {
  display: flex;
  gap: 0;
  align-items: stretch;
  height: 100%;
}

.window-header-button {
  width: 30px;
  background: transparent;
  color: light-dark(rgba(0, 0, 0, .7), rgba(255, 255, 255, .8));
  border-radius: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: .2s;
}

.window-header-button:hover {
  background: light-dark(rgba(0, 0, 0, .1), rgba(255, 255, 255, .15));
  color: light-dark(#000000, #ffffff);
}

.window-header-close {
  border-top-right-radius: 5px;
  border-bottom-right-radius: 5px;
  margin: 0;
  height: 100%;
}

.window-header-close:hover {
  background: rgba(239, 68, 68, .8);
}
</style>
