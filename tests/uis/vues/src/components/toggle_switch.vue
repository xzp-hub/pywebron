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
  <div class="thumbs" :class="{right: isRight}" @click="toggle">
    <div class="slider" :class="{active: !isRight}">{{ leftLabel }}</div>
    <div class="slider" :class="{active: isRight}">{{ rightLabel }}</div>
  </div>
</template>

<style scoped>
.thumbs {
  width: auto;
  height: 26px;
  gap: 3px;
  display: flex;
  align-items: center;
  padding: 0 3px;
  cursor: pointer;
  border-radius: 2px;
  border: 1px solid var(--border-default);
  overflow: hidden;
  position: relative;
  left: 1px;
  background: #eeeeee;


  &::before {
    content: '';
    position: absolute;
    top: 3px;
    height: 20px;
    width: calc(50% - 4.5px);
    background: rgb(237 146 28);
    border-radius: 2px;
    transition: transform .25s ease;
  }

  &.right::before {
    transform: translateX(calc(100% + 3px));
  }
}

.slider {
  flex: 1;
  height: 20px;
  padding: 0 5px;
  border-radius: 2px;
  font-size: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1;
  transition: color .25s ease;

  &.active {
    color: #fff;
  }
}

[data-theme="dark"] {
  .thumbs {
    background: rgba(255, 255, 255, .08);
  }

  .slider {
    color: rgb(255 255 255);
  }
}
</style>
