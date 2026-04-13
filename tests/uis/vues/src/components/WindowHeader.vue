<script setup>
import {ref, onMounted} from 'vue'
import {MinusIcon, FullscreenIcon, FullscreenExitIcon, CloseIcon} from 'tdesign-icons-vue-next'

const pw = window.pywebron
const isMaximized = ref(false)
const invoke = pw?.interfaces?.invoke
const attributes = pw?.attributes || {}

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
  if (attributes?.icon_path) {
    iconSrc.value = pw?.interfaces?.resolveAssetUrl(attributes.icon_path) || ''
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

<style scoped>
.window-header {
  width: 100%;
  height: 30px;
  border-radius: 5px;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: light-dark(#ffffff, #1a1b1d);
  border: 1px solid light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .2));
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
  color: light-dark(#000000, #ffffff);
  line-height: 16px;
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
  color: light-dark(rgba(0, 0, 0, .7), rgba(255, 255, 255, .8)) !important;
  background: transparent !important;
  transition: .2s;
}

.window-header-btn:hover {
  background: light-dark(rgba(0, 0, 0, .1), rgba(255, 255, 255, .15)) !important;
  color: light-dark(#000000, #ffffff) !important;
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
