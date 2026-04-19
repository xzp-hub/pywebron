<script>
export default {
  name: 'toggle_switch'
}
</script>

<script setup>
import {computed, useSlots} from 'vue'

const props = defineProps({
  modelValue: {type: String, required: true},
  leftLabel: {type: String, default: ''},
  rightLabel: {type: String, default: ''},
  leftValue: {type: String, default: 'left'},
  rightValue: {type: String, default: 'right'},
  activeColor: {type: String, default: 'rgb(94 43 2)'},
  height: {type: Number, default: 30}
})

const emit = defineEmits(['update:modelValue', 'change'])
const slots = useSlots()
const isRight = computed(() => props.modelValue === props.rightValue)
const hasLeftSlot = computed(() => !!slots.left)
const hasRightSlot = computed(() => !!slots.right)

const sliderH = computed(() => props.height - 8)
const pad = computed(() => (props.height - sliderH.value) / 2)

function toggle() {
  const val = isRight.value ? props.leftValue : props.rightValue
  emit('update:modelValue', val)
  emit('change', val)
}
</script>

<template>
  <div class="thumbs"
       :style="{height: height + 'px'}"
       @click="toggle">
    <!-- 滑动块 -->
    <div class="slider"
         :class="{right: isRight}"
         :style="{
           width: sliderH + 'px',
           height: sliderH + 'px',
           top: pad + 'px',
           background: activeColor,
         }">
      <!-- 左侧：优先插槽 -->
      <slot v-if="hasLeftSlot" name="left"/>
      <span v-else-if="!isRight">{{ leftLabel }}</span>
      <!-- 右侧：优先插槽 -->
      <slot v-if="hasRightSlot" name="right"/>
      <span v-else-if="isRight">{{ rightLabel }}</span>
    </div>

    <!-- 未激活侧的文字（静态背景文字） -->
    <span v-if="(hasLeftSlot || leftLabel) && isRight" class="label label-left">
      <slot v-if="hasLeftSlot" name="left"/>
      <template v-else>{{ leftLabel }}</template>
    </span>
    <span v-if="(hasRightSlot || rightLabel) && !isRight" class="label label-right">
      <slot v-if="hasRightSlot" name="right"/>
      <template v-else>{{ rightLabel }}</template>
    </span>
  </div>
</template>

<style scoped>
.thumbs {
  display: flex;
  align-items: center;
  position: relative;
  cursor: pointer;
  border-radius: 2px;
  border: 1px solid var(--border-default);
  overflow: hidden;
  background: #eeeeee;
  box-sizing: border-box;
}

.slider {
  position: absolute;
  left: 4px;
  border-radius: 2px;
  transition: transform .25s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2;

  & > * {
    color: #fff;
  }
}

.slider.right {
  transform: translateX(calc(100% + 2px));
}

.label {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  font-size: 10px;
  z-index: 1;
}

.label-left {
  left: calc(50% - 5px);
}

.label-right {
  right: calc(50% - 5px);
}

[data-theme="dark"] .thumbs {
  background: rgba(255, 255, 255, .08);
}
</style>
