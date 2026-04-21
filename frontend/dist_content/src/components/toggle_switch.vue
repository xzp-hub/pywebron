<script>
export default {
  name: 'ToggleSwitch'
}
</script>

<script setup>
import {computed, useSlots} from 'vue'

const props = defineProps({
  modelValue: {type: [String, Number, Boolean], default: false},
  activeValue: {type: [String, Number, Boolean], default: true},
  inactiveValue: {type: [String, Number, Boolean], default: false},
  activeText: {type: String, default: ''},
  inactiveText: {type: String, default: ''},
  activeColor: {type: String, default: '#409EFF'},
  inactiveColor: {type: String, default: '#C0CCDA'},
  height: {type: Number, default: 22},
  disabled: {type: Boolean, default: false}
})

const emit = defineEmits(['update:modelValue', 'change'])
const slots = useSlots()

const isActive = computed(() => props.modelValue === props.activeValue)
const hasSlot = computed(() => !!slots.active || !!slots.inactive)
const hasText = computed(() => !!props.activeText || !!props.inactiveText)

const thumbSize = computed(() => props.height - 4)
const trackWidth = computed(() => {
  if (hasSlot.value) return thumbSize.value * 2 + 6
  if (hasText.value) return thumbSize.value * 2 + 12
  return thumbSize.value * 2 + 4
})

const thumbTranslateX = computed(() => {
  return isActive.value ? trackWidth.value - thumbSize.value - 2 : 2
})

function toggle() {
  if (props.disabled) return
  const val = isActive.value ? props.inactiveValue : props.activeValue
  emit('update:modelValue', val)
  emit('change', val)
}
</script>

<template>
  <div class="toggle-switch"
       :class="{'is-active': isActive, 'is-disabled': disabled}"
       :style="{
         height: height + 'px',
         width: trackWidth + 'px',
         borderRadius: '2px'
       }"
       @click="toggle">
    <!-- 滑块（彩色） -->
    <span class="toggle-switch__core"
          :style="{
            width: thumbSize + 'px',
            height: thumbSize + 'px',
            transform: `translateX(${thumbTranslateX}px)`,
            borderRadius: '2px',
            backgroundColor: activeColor
          }">
      <!-- 滑块内显示当前状态内容 -->
      <span class="toggle-switch__inner">
        <slot v-if="isActive" name="active">{{ activeText }}</slot>
        <slot v-else name="inactive">{{ inactiveText }}</slot>
      </span>
    </span>
  </div>
</template>

<style scoped>
.toggle-switch {
  display: inline-flex;
  align-items: center;
  position: relative;
  cursor: pointer;
  transition: background-color .3s;
  vertical-align: middle;
  user-select: none;
  box-sizing: border-box;
  padding: 2px;
  background: #e0e0e0;
}

.toggle-switch.is-disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.toggle-switch__core {
  position: relative;
  flex-shrink: 0;
  transition: transform .25s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
}

.toggle-switch__inner {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 9px;
  line-height: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding: 0 2px;
}

.toggle-switch__inner :deep(svg) {
  width: 12px;
  height: 12px;
}

[data-theme="dark"] .toggle-switch {
  background: rgba(255, 255, 255, .12);
}
</style>
