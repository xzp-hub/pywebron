<script>
export default {
  name: 'title_bar'
}
</script>

<script setup>
import {ref, onMounted, computed} from 'vue'
import {MinusIcon, FullscreenIcon, CloseIcon, BrightnessIcon, MoonIcon} from 'tdesign-icons-vue-next'
import {useTheme} from '@/composables/use_theme'
import ToggleSwitch from './toggle_switch.vue'

const pw = window.pywebron
const isMaximized = ref(false)
const attributes = pw?.attributes || {}
const windows = pw?.interfaces?.windows
const {currentTheme, toggleTheme} = useTheme()

const themeValue = computed({
  get: () => currentTheme.value === 'dark' ? 'dark' : 'light',
  set: () => toggleTheme()
})

const windowAction = async (type) => {
  const action = type === 'toggle' ? (isMaximized.value ? 'rep' : 'max') : type
  try {
    const fn = {min: windows?.minimize, max: windows?.maximize, rep: windows?.reappear, shut: windows?.shutdown}[action]
    if (fn) await fn()
    if (action === 'max') isMaximized.value = true
    else if (action === 'rep') isMaximized.value = false
  } catch (e) { /* noop */
  }
}

const iconSrc = ref('')
const titleText = ref('PyWebron 控制面板')

onMounted(async () => {
  if (attributes?.icon_path) {
    const resolved = pw?.interfaces?.utils?.resolveAssetUrl(attributes.icon_path)
    iconSrc.value = resolved || ''
  }
  if (!iconSrc.value && attributes?.icon_path === undefined) {
    iconSrc.value = ''
  }
  if (attributes?.title) {
    titleText.value = attributes.title
  }
  try {
    await windows?.dragdrop('#window-header')
  } catch (e) {
    console.error(e)
  }
})
</script>

<template>
  <div id="window-header" class="window-header">
    <div class="header-left">
      <img class="window-icon" :src="iconSrc" alt="">
      <span class="window-title">{{ titleText }}</span>
    </div>
    <div class="window-control-buttons">
      <ToggleSwitch v-model="themeValue" inactive-value="light" active-value="dark" :height="22" active-color="#8B5CF6"
                    class="window-theme-switch">
        <template #inactive>
          <BrightnessIcon/>
        </template>
        <template #active>
          <MoonIcon/>
        </template>
      </ToggleSwitch>
      <t-button class="window-control-button" variant="outline" shape="square" title="最小化"
                @click="windowAction('min')">
        <template #icon>
          <MinusIcon/>
        </template>
      </t-button>
      <t-button class="window-control-button" variant="outline" shape="square"
                :title="isMaximized ? '还原' : '最大化'" @click="windowAction('toggle')">
        <template #icon>
          <svg v-if="isMaximized" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
               fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8 3H5a2 2 0 0 0-2 2v3"></path>
            <path d="M21 8V5a2 2 0 0 0-2-2h-3"></path>
            <path d="M3 16v3a2 2 0 0 0 2 2h3"></path>
            <path d="M16 21h3a2 2 0 0 0 2-2v-3"></path>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
               stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          </svg>
        </template>
      </t-button>
      <t-button class="window-control-button" variant="outline" shape="square" title="关闭"
                @click="windowAction('shut')">
        <template #icon>
          <CloseIcon/>
        </template>
      </t-button>
    </div>
  </div>
</template>

<style scoped>
.window-header {
  width: 100%;
  height: 40px;
  border-radius: 5px;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  padding-right: 6px;
}

[data-theme="dark"] .window-header {
  background: #1a1b1d;
}

.header-left {
  display: flex;
  align-items: center;
  padding-left: 5px;
}

.window-icon {
  width: 20px;
  height: 20px;
  margin-right: 5px;
  border-radius: 5px;
  object-fit: contain;
  flex-shrink: 0;
}

.window-title {
  font-size: 13px;
  color: var(--text-inverse);
  line-height: 16px;
}

[data-theme="dark"] .window-title {
  color: #ffffff;
}

.window-control-buttons {
  display: flex;
  gap: 6px;
  align-items: center;
  height: 100%;
}

.window-control-button {
  width: 28px;
  height: 28px;
  box-sizing: border-box;
  border: 1px solid rgba(128, 128, 128, 0.01);
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  background: transparent;
  border-radius: 2px;
}

.window-control-button:hover {
  background: rgba(128, 128, 128, 0.3) !important;
}

.window-control-button:last-child:hover {
  background: #E34D59 !important;
  color: #fff !important;
}

.window-theme-switch :deep(.toggle-switch__core svg) {
  width: 12px;
  height: 12px;
}
</style>
