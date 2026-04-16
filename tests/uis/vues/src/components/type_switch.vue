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
  <div class="thumbs" @click="toggle">
    <div class="slider" :class="{ active: !isRight }">{{ leftLabel }}</div>
    <div class="slider" :class="{ active: isRight }">{{ rightLabel }}</div>
    <div class="slider-thumb" :class="{ right: isRight }"></div>
  </div>
</template>

<style scoped>
.thumbs {
  position: relative;
  width: 100px;
  height: 26px;
  display: flex;
  cursor: pointer;
  border: 1px solid var(--border-default);
  border-radius: 3px;
  overflow: hidden;
}

.slider-thumb {
  position: absolute;
  top: 3px;
  left: 4px;
  width: calc(50% - 6px);
  height: calc(100% - 6px);
  background: rgb(94 43 2);
  border-radius: 3px;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.slider-thumb.right {
  transform: translateX(calc(100% + 4px));
}

.slider {
  flex: 1;
  font-size: 11px;
  color: var(--text-secondary);
  transition: color 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1;
}

.slider:first-child {
  left: 1px;
}

.slider.active {
  color: #fff;
  font-weight: 500;
}

[data-theme="dark"] .thumbs {
  background: rgba(255, 255, 255, 0.05);
}

[data-theme="dark"] .slider {
  color: rgba(255, 255, 255, 0.6);
}

[data-theme="dark"] .slider.active {
  color: #fff;
}
</style>
