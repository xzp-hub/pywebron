<script setup>
import { useTheme } from '@/composables/useTheme'

const { currentTheme, toggleTheme, setTheme } = useTheme()
</script>

<template>
  <div class="theme-demo">
    <div class="demo-header">
      <h2>主题系统演示</h2>
      <p>当前主题: <strong>{{ currentTheme === 'light' ? '亮色' : '暗色' }}</strong></p>
    </div>

    <div class="demo-controls">
      <button @click="toggleTheme" class="btn-toggle">
        切换主题
      </button>
      <button @click="setTheme('light')" class="btn-light">
        亮色模式
      </button>
      <button @click="setTheme('dark')" class="btn-dark">
        暗色模式
      </button>
    </div>

    <div class="demo-colors">
      <div class="color-group">
        <h3>背景色</h3>
        <div class="color-item" style="background: var(--bg-base)">
          <span>--bg-base</span>
        </div>
        <div class="color-item" style="background: var(--bg-card)">
          <span>--bg-card</span>
        </div>
        <div class="color-item" style="background: var(--bg-elevated)">
          <span>--bg-elevated</span>
        </div>
      </div>

      <div class="color-group">
        <h3>文本色</h3>
        <div class="color-item">
          <span style="color: var(--text-primary)">--text-primary</span>
        </div>
        <div class="color-item">
          <span style="color: var(--text-secondary)">--text-secondary</span>
        </div>
        <div class="color-item">
          <span style="color: var(--text-tertiary)">--text-tertiary</span>
        </div>
      </div>

      <div class="color-group">
        <h3>边框色</h3>
        <div class="color-item" style="border: 2px solid var(--border-default)">
          <span>--border-default</span>
        </div>
        <div class="color-item" style="border: 2px solid var(--border-strong)">
          <span>--border-strong</span>
        </div>
        <div class="color-item" style="border: 2px solid var(--border-subtle)">
          <span>--border-subtle</span>
        </div>
      </div>

      <div class="color-group">
        <h3>交互状态</h3>
        <div class="color-item interactive">
          <span>悬停效果</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.theme-demo {
  padding: 20px;
  background: var(--bg-card);
  border-radius: 8px;
  border: 1px solid var(--border-default);
}

.demo-header {
  margin-bottom: 20px;
  
  h2 {
    color: var(--text-primary);
    margin: 0 0 8px 0;
  }
  
  p {
    color: var(--text-secondary);
    margin: 0;
  }
  
  strong {
    color: var(--text-primary);
  }
}

.demo-controls {
  display: flex;
  gap: 10px;
  margin-bottom: 30px;
  
  button {
    padding: 8px 16px;
    border-radius: 6px;
    border: 1px solid var(--border-default);
    background: var(--bg-elevated);
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.2s;
    
    &:hover {
      background: var(--hover-bg-strong);
      border-color: var(--border-strong);
    }
    
    &:active {
      transform: scale(0.98);
    }
  }
  
  .btn-toggle {
    background: var(--active-bg);
    color: white;
    border-color: var(--active-bg);
    
    &:hover {
      opacity: 0.9;
    }
  }
}

.demo-colors {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
}

.color-group {
  h3 {
    color: var(--text-primary);
    font-size: 14px;
    margin: 0 0 12px 0;
  }
}

.color-item {
  padding: 12px;
  margin-bottom: 8px;
  border-radius: 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  
  span {
    color: var(--text-primary);
    font-size: 12px;
    font-family: monospace;
  }
  
  &.interactive {
    cursor: pointer;
    transition: background 0.2s;
    
    &:hover {
      background: var(--hover-bg-strong);
    }
  }
}
</style>
