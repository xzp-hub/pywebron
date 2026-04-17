<script>
export default {
  name: 'toggle_switch'
}
</script>

<script setup>
import {computed} from 'vue'

const props = defineProps({
  modelValue: {type: String, required: true},
  leftLabel: {type: String, default: '左'},
  rightLabel: {type: String, default: '右'},
  leftValue: {type: String, default: 'left'},
  rightValue: {type: String, default: 'right'},
  activeColor: {type: String, default: 'rgb(94 43 2)'}
})

const emit = defineEmits(['update:modelValue', 'change'])
const isRight = computed(() => props.modelValue === props.rightValue)

function toggle() {
  const val = isRight.value ? props.leftValue : props.rightValue
  emit('update:modelValue', val)
  emit('change', val)
}
</script>

<template>
  <div class="thumbs" :class="{right: isRight}" :style="{'--active-color': activeColor}" @click="toggle">
    <span class="slider" :class="{active: !isRight}">{{ leftLabel }}</span>
    <span class="slider" :class="{active: isRight}">{{ rightLabel }}</span>
  </div>
</template>

<style scoped>
.thumbs {
  width: 100px;
  height: 26px;
  display: flex;
  cursor: pointer;
  border: 1px solid var(--border-default);
  border-radius: 3px;
  position: relative;

  &::before {
    content: '';
    position: absolute;
    top: 3.5px;
    bottom: 3.5px;
    left: 4px;
    width: calc(50% - 8px);
    background: var(--active-color, rgb(94 43 2));
    border-radius: 2px;
    transition: transform .25s ease;
  }

  &.right::before {
    transform: translateX(calc(100% + 8px));
  }
}

.slider {
  flex: 1;
  font-size: 11px;
  color: var(--text-secondary);
  transition: color .25s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1;
  line-height: 1;

  &.active {
    color: #fff;
  }
}

[data-theme="dark"] {
  .thumbs {
    background: rgba(255, 255, 255, .05);
  }

  .slider {
    color: rgba(255, 255, 255, .6);
  }
}
</style>
