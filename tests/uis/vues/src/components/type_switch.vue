<script setup>
import {computed} from 'vue'

const props = defineProps({
  modelValue: {
    type: String,
    required: true
  },
  leftLabel: {
    type: String,
    default: '左'
  },
  rightLabel: {
    type: String,
    default: '右'
  },
  leftValue: {
    type: String,
    default: 'left'
  },
  rightValue: {
    type: String,
    default: 'right'
  }
})

const emit = defineEmits(['update:modelValue', 'change'])

const isRight = computed(() => props.modelValue === props.rightValue)

function toggle() {
  const newValue = isRight.value ? props.leftValue : props.rightValue
  emit('update:modelValue', newValue)
  emit('change', newValue)
}
</script>

<template>
  <div class="io-type-switch" @click="toggle">
    <div class="switch-track">
      <div class="switch-thumb" :class="{ 'switch-thumb-right': isRight }"></div>
    </div>
    <div class="switch-label switch-label-left" :class="{ active: !isRight }">{{ leftLabel }}</div>
    <div class="switch-label switch-label-right" :class="{ active: isRight }">{{ rightLabel }}</div>
  </div>
</template>

<style scoped>
.io-type-switch {
  position: relative;
  width: 100px;
  height: 26px;
  display: flex;
  align-items: center;
  cursor: pointer;
  user-select: none;
}

.switch-track {
  position: absolute;
  width: 100%;
  height: 100%;
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: 3px;
  overflow: hidden;
}

[data-theme="dark"] .switch-track {
  background: rgba(255, 255, 255, 0.05);
}

.switch-thumb {
  position: absolute;
  left: 4px;
  top: 4px;
  width: calc(50% - 5px);
  height: calc(100% - 7px);
  background: rgb(94 43 2);
  border-radius: 2px;
  transition: left 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.switch-thumb-right {
  left: calc(50% + 2px);
}

.switch-label {
  position: relative;
  z-index: 1;
  font-size: 11px;
  color: var(--text-secondary);
  transition: color 0.3s;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 0;
  line-height: 1;
}

.switch-label.active {
  color: #fff;
  font-weight: 500;
}

[data-theme="dark"] .switch-label {
  color: rgba(255, 255, 255, 0.6);
}

[data-theme="dark"] .switch-label.active {
  color: #fff;
}
</style>
