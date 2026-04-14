<script setup>
import {ref, onMounted} from 'vue'
import {MinusIcon, FullscreenIcon, FullscreenExitIcon, CloseIcon} from 'tdesign-icons-vue-next'
import {useTheme} from '@/composables/useTheme'

const pw = window.pywebron
const isMaximized = ref(false)
const invoke = pw?.interfaces?.invoke
const attributes = pw?.attributes || {}
const {currentTheme, toggleTheme} = useTheme()

const windowAction = async (type) => {
  const map = {
    min: 'minimize_window',
    max: 'maximize_window',
    rep: 'reappear_window',
    shut: 'shutdown_window'
  }
  const action = type === 'toggle' ? (isMaximized.value ? 'rep' : 'max') : type
  try {
    const res = await invoke('window_controls_invoke', {control_type: map[action]}, 5000)
    if (res?.code === 200) {
      isMaximized.value = action === 'max' || (type === 'toggle' && !isMaximized.value)
    }
  } catch (e) { /* noop */
  }
}

const iconSrc = ref('')
const titleText = ref('PyWebron 控制面板')

onMounted(async () => {
  console.log('[WindowHeader] attributes:', JSON.stringify(attributes))
  console.log('[WindowHeader] icon_path:', attributes?.icon_path)
  if (attributes?.icon_path) {
    const resolved = pw?.interfaces?.resolveAssetUrl(attributes.icon_path)
    console.log('[WindowHeader] resolveAssetUrl result:', resolved, '| input:', attributes.icon_path)
    iconSrc.value = resolved || ''
    console.log('[WindowHeader] iconSrc set to:', iconSrc.value)
  }
  // 如果没有图标，使用内置的 app 协议图标作为 fallback
  if (!iconSrc.value && attributes?.icon_path === undefined) {
    iconSrc.value = ''
  }
  if (attributes?.title) {
    titleText.value = attributes.title
  }
  try {
    await invoke('setup_drag_region_invoke', {selector: '#window-header'})
  } catch (e) {
    console.error(e)
  }
})
</script>

<template>
  <div id="window-header" class="window-header">
    <div class="window-header-info">
      <img class="window-header-app-icon" :src="iconSrc || 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiIgdmlld0JveD0iMCAwIDE2IDE2Ij48cmVjdCB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHJ4PSIzIiBmaWxsPSIjODIwMWY4Ii8+PHRleHQgeD0iOCIgeT0iMTIuNSIgZm9udC1zaXplPSIxMCIgZmlsbD0id2hpdGUiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtd2VpZ2h0PSJib2xkIj5QPC90ZXh0Pjwvc3ZnPg=='" alt="">
      <span class="window-header-app-title">{{ titleText }}</span>
    </div>
    <div class="window-header-control-buttons">
      <button class="window-header-btn window-header-btn-theme" :title="currentTheme === 'light' ? '切换到暗色模式' : '切换到亮色模式'" @click="toggleTheme">
        <svg v-if="currentTheme === 'dark'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="5"></circle>
          <line x1="12" y1="1" x2="12" y2="3"></line>
          <line x1="12" y1="21" x2="12" y2="23"></line>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
          <line x1="1" y1="12" x2="3" y2="12"></line>
          <line x1="21" y1="12" x2="23" y2="12"></line>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
        </svg>
      </button>
      <t-button class="window-header-btn window-header-btn-minimize" variant="text" shape="square" title="最小化"
                @click="windowAction('min')">
        <template #icon>
          <MinusIcon/>
        </template>
      </t-button>
      <t-button class="window-header-btn window-header-btn-maximize" variant="text" shape="square"
                :title="isMaximized ? '还原' : '最大化'" @click="windowAction('toggle')">
        <template #icon>
          <FullscreenExitIcon v-if="isMaximized"/>
          <FullscreenIcon v-else/>
        </template>
      </t-button>
      <t-button class="window-header-btn window-header-btn-close" variant="text" shape="square" title="关闭"
                @click="windowAction('shut')">
        <template #icon>
          <CloseIcon/>
        </template>
      </t-button>
    </div>
  </div>
</template>

<style lang="scss" scoped>
@use 'assets/themes/mixins' as *;

.window-header {
  width: 100%;
  height: 30px;
  border-radius: 5px;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-card);
  border: 1px solid var(--border-strong);
}

[data-theme="dark"] .window-header {
  background: #1a1b1d;
}

.window-header-info {
  display: flex;
  align-items: center;
  padding-left: 5px;
}

.window-header-app-icon {
  width: 16px;
  height: 16px;
  margin-right: 5px;
  border-radius: 5px;
  object-fit: contain;
  flex-shrink: 0;
}

.window-header-app-title {
  font-size: 12px;
  color: var(--text-inverse);
  line-height: 16px;
}

[data-theme="dark"] .window-header-app-title {
  color: #ffffff;
}

.window-header-control-buttons {
  display: flex;
  gap: 0;
  align-items: stretch;
  height: 100%;
}

.window-header-btn {
  width: 30px;
  height: 30px;
  min-width: auto;
  border-radius: 0 !important;
  color: var(--text-tertiary) !important;
  background: transparent !important;
  transition: .2s;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.window-header-btn:hover {
  background: var(--hover-bg-strong) !important;
  color: var(--text-inverse) !important;
}

.window-header-btn-theme {
  svg {
    width: 16px;
    height: 16px;
  }
}

.window-header-btn-close {
  border-top-right-radius: 5px !important;
  border-bottom-right-radius: 5px !important;
  margin: 0;
  height: 100%;
}

.window-header-btn-close:hover {
  background: rgba(239, 68, 68, .8) !important;
  color: #fff !important;
}
</style>
