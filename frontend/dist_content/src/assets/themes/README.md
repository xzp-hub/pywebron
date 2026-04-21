# PyWebron 主题系统

## 概述

本项目使用纯手写的主题切换系统，基于 CSS 变量（CSS Custom Properties）实现亮色/暗色主题切换。

## 架构

### 文件结构

```
src/
├── assets/
│   ├── themes/
│   │   ├── _variables.scss    # CSS 变量定义（亮色/暗色主题）
│   │   ├── _mixins.scss        # SCSS mixins 工具
│   │   ├── index.scss          # 主题系统入口
│   │   └── README.md           # 本文档
│   ├── base.scss               # 全局基础样式
│   └── main.scss               # 样式入口文件
├── composables/
│   └── useTheme.js             # 主题切换逻辑
└── components/
    └── WindowHeader.vue        # 包含主题切换按钮
```

### 工作原理

1. **CSS 变量定义**：在 `_variables.scss` 中定义两套主题变量
   - `:root` 和 `[data-theme="light"]` 定义亮色主题
   - `[data-theme="dark"]` 定义暗色主题

2. **主题切换**：通过 `useTheme` composable 管理主题状态
   - 切换 HTML 元素的 `data-theme` 属性
   - 自动保存到 localStorage
   - 支持系统主题检测

3. **样式应用**：组件中使用 CSS 变量
   ```scss
   .my-component {
     background: var(--bg-card);
     color: var(--text-primary);
   }
   ```

## 使用方法

### 在组件中使用主题

```vue
<script setup>
import { useTheme } from '@/composables/useTheme'

const { currentTheme, toggleTheme, setTheme } = useTheme()
</script>

<template>
  <button @click="toggleTheme">
    当前主题: {{ currentTheme }}
  </button>
</template>

<style lang="scss" scoped>
.my-element {
  background: var(--bg-card);
  color: var(--text-primary);
  border: 1px solid var(--border-default);
}
</style>
```

### 可用的 CSS 变量

#### 背景色
- `--bg-base`: 应用根背景
- `--bg-card`: 卡片背景
- `--bg-elevated`: 浮层/输入框背景
- `--bg-content-area`: 内容区域背景

#### 文本色
- `--text-primary`: 主要文本
- `--text-secondary`: 次要文本
- `--text-tertiary`: 三级文本
- `--text-placeholder`: 占位符文本

#### 边框色
- `--border-default`: 默认边框
- `--border-strong`: 强调边框
- `--border-subtle`: 弱化边框

#### 交互状态
- `--hover-bg`: 悬停背景
- `--hover-bg-strong`: 强调悬停背景
- `--active-bg`: 激活背景

更多变量请查看 `_variables.scss` 文件。

### 使用 Mixins

```scss
@use 'assets/themes/mixins' as *;

.my-card {
  @include card-base;
}

.my-card-header {
  @include card-header-base;
}
```

## 特性

- ✅ 纯手写实现，无第三方主题框架依赖
- ✅ 基于 CSS 变量，性能优秀
- ✅ 支持亮色/暗色主题
- ✅ 自动保存用户偏好
- ✅ 支持系统主题检测
- ✅ 所有样式文件使用 SCSS
- ✅ 完整的类型提示和注释

## 添加新主题变量

1. 在 `_variables.scss` 中添加变量：
```scss
:root,
[data-theme="light"] {
  --my-new-color: #ffffff;
}

[data-theme="dark"] {
  --my-new-color: #000000;
}
```

2. 在组件中使用：
```scss
.my-element {
  color: var(--my-new-color);
}
```

## 注意事项

- 所有颜色值应该在 `_variables.scss` 中定义，避免硬编码
- 使用语义化的变量名，如 `--text-primary` 而不是 `--color-black`
- 新增组件样式时，优先使用现有的 CSS 变量
- 确保亮色和暗色主题都有对应的变量定义
