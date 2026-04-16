<script setup>
defineProps({
  title: String,
  iconColor: {
    type: String,
    default: '#8201f8'
  }
})
</script>

<template>
  <div class="base-card">
    <div class="base-card-header">
      <div class="base-card-header-icon-box">
        <slot name="icon"></slot>
      </div>
      <span class="base-card-header-title">{{ title }}</span>
      <slot name="header-extra"></slot>
    </div>
    <div class="base-card-body">
      <slot></slot>
    </div>
    <div v-if="$slots.footer" class="base-card-footer">
      <slot name="footer"></slot>
    </div>
  </div>
</template>

<style scoped>
.base-card {
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-card);
  box-sizing: border-box;
  border: 1px solid var(--border-default);
  height: 100%;
  flex: 1;
}

.base-card-header {
  height: 36px;
  display: flex;
  align-items: center;
  background: var(--bg-card-header);
  box-sizing: border-box;
  border-bottom: 1px solid var(--border-default);
  padding-left: 6px;
  gap: 5px;
}

.base-card-header-icon-box {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.base-card-header-title {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1;
}

[data-theme="dark"] .base-card-header-title {
  color: #ffffff;
}

.base-card-body {
  flex: 1;
  min-height: 0;
  background: var(--bg-card);
  overflow: hidden;
}

[data-theme="dark"] .base-card-body {
  background: #1a1b1d;
}

.base-card-footer {
  min-height: 36px;
  height: auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-card-footer);
  box-sizing: border-box;
  border-top: 1px solid var(--border-default);
}
</style>
