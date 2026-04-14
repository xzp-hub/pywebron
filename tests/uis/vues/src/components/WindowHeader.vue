<script setup>
import {ref, onMounted} from 'vue'
import {MinusIcon, FullscreenIcon, CloseIcon, BrightnessIcon, MoonIcon} from 'tdesign-icons-vue-next'
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
      <t-button class="window-header-btn window-header-btn-theme" variant="outline" shape="square" :title="currentTheme === 'light' ? '切换到暗色模式' : '切换到亮色模式'" @click="toggleTheme">
        <template #icon>
          <MoonIcon v-if="currentTheme === 'light'"/>
          <BrightnessIcon v-else/>
        </template>
      </t-button>
      <t-button class="window-header-btn window-header-btn-minimize" variant="outline" shape="square" title="最小化"
                @click="windowAction('min')">
        <template #icon>
          <MinusIcon/>
        </template>
      </t-button>
      <t-button class="window-header-btn window-header-btn-maximize" variant="outline" shape="square"
                :title="isMaximized ? '还原' : '最大化'" @click="windowAction('toggle')">
        <template #icon>
          <svg v-if="!isMaximized" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8 3H5a2 2 0 0 0-2 2v3"></path>
            <path d="M21 8V5a2 2 0 0 0-2-2h-3"></path>
            <path d="M3 16v3a2 2 0 0 0 2 2h3"></path>
            <path d="M16 21h3a2 2 0 0 0 2-2v-3"></path>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          </svg>
        </template>
      </t-button>
      <t-button class="window-header-btn window-header-btn-close" variant="outline" shape="square" title="关闭"
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
  height: 36px;
  border-radius: 5px;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-card);
  border: 1px solid var(--border-strong);
  padding-right: 5px;
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
  width: 20px;
  height: 20px;
  margin-right: 5px;
  border-radius: 5px;
  object-fit: contain;
  flex-shrink: 0;
}

.window-header-app-title {
  font-size: 13px;
  color: var(--text-inverse);
  line-height: 16px;
}

[data-theme="dark"] .window-header-app-title {
  color: #ffffff;
}

.window-header-control-buttons {
  display: flex;
  gap: 5px;
  align-items: center;
  height: 100%;
}

.window-header-btn {
  width: 28px !important;
  height: 26px !important;
  min-width: auto !important;
  color: var(--text-tertiary) !important;
  padding: 0 !important;
  margin: 0 !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  line-height: 1 !important;
}

.window-header-btn > .t-button__content,
.window-header-btn > button,
.window-header-btn > div {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  width: 100% !important;
  height: 100% !important;
  padding: 0 !important;
  margin: 0 !important;
}

[data-theme="dark"] .window-header-btn {
  color: #ffffff !important;
}

/* 前三个按钮：hover 无背景色，边框+图标跟随颜色 */
.window-header-btn:hover {
  background: transparent !important;
  color: #165DFF !important;
  border-color: #165DFF !important;
}

.window-header-btn:hover svg,
.window-header-btn:hover .t-icon {
  color: #165DFF !important;
}

[data-theme="dark"] .window-header-btn:hover {
  color: #6aa1ff !important;
  border-color: #6aa1ff !important;
}

[data-theme="dark"] .window-header-btn:hover svg,
[data-theme="dark"] .window-header-btn:hover .t-icon {
  color: #6aa1ff !important;
}

/* 关闭按钮：hover 红色背景 + 白色图标 */
.window-header-btn-close:hover {
  background: #E34D59 !important;
  color: #fff !important;
  border-color: #E34D59 !important;
}

.window-header-btn-close:hover svg,
.window-header-btn-close:hover .t-icon {
  color: #fff !important;
}

.window-header-btn-theme {
  width: 28px;
  height: 26px;
}

.window-header-btn-close {
  border-top-right-radius: 5px !important;
  border-bottom-right-radius: 5px !important;
  height: 26px;
  margin-right: 5px;
}

.window-header-btn-close:hover {
  background: rgba(239, 68, 68, .8) !important;
  color: #fff !important;
}
</style>
