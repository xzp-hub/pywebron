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
  <div class="io-type-switch" :class="{ 'switch-right': isRight }" @click="toggle">
    <div class="switch-label" :class="{ active: !isRight }">{{ leftLabel }}</div>
    <div class="switch-label" :class="{ active: isRight }">{{ rightLabel }}</div>
  </div>
</template>

<style scoped>
.io-type-switch {
  position: relative;
  width: 100px;
  height: 26px;
  display: flex;
  cursor: pointer;
  user-select: none;
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: 3px;
  overflow: hidden;
}

[data-theme="dark"] .io-type-switch {
  background: rgba(255, 255, 255, 0.05);
}

.io-type-switch::before {
  content: '';
  position: absolute;
  top: 3px;
  left: 4px;
  width: calc(50% - 5px);
  height: calc(100% - 6px);
  background: rgb(94 43 2);
  border-radius: 2px;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.io-type-switch.switch-right::before {
  transform: translateX(calc(100% + 2px));
}

.switch-label {
  position: relative;
  z-index: 1;
  flex: 1;
  font-size: 11px;
  color: var(--text-secondary);
  transition: color 0.3s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.switch-label:first-child {
  left: 2px;
}

.switch-label:last-child {
  right: 1px;
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
