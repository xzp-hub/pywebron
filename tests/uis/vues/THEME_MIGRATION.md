# 主题系统迁移指南

## 变更概述

项目已从框架依赖的主题系统迁移到纯手写的主题切换实现。

## 主要变更

### 1. 移除的文件
- ❌ `src/assets/base.css`
- ❌ `src/assets/main.css`

### 2. 新增的文件
- ✅ `src/composables/useTheme.js` - 主题切换逻辑
- ✅ `src/assets/themes/README.md` - 主题系统文档

### 3. 更新的文件
- 📝 `src/main.js` - 添加主题初始化
- 📝 `src/assets/base.scss` - 添加字体导入
- 📝 `src/assets/themes/_variables.scss` - 优化注释和组织
- 📝 `src/components/WindowHeader.vue` - 添加主题切换按钮

## 新功能

### 主题切换按钮

在窗口标题栏添加了主题切换按钮：
- 🌙 月亮图标 = 切换到暗色模式
- ☀️ 太阳图标 = 切换到亮色模式

### 自动主题检测

- 首次访问时自动检测系统主题偏好
- 用户手动切换后会保存到 localStorage
- 下次访问时自动恢复用户选择的主题

### 系统主题跟随

- 如果用户未手动设置主题，会自动跟随系统主题变化
- 用户手动设置后，不再跟随系统主题

## 使用示例

### 在新组件中使用主题

```vue
<script setup>
import { useTheme } from '@/composables/useTheme'

const { currentTheme, toggleTheme } = useTheme()
</script>

<template>
  <div class="my-component">
    <p>当前主题: {{ currentTheme }}</p>
    <button @click="toggleTheme">切换主题</button>
  </div>
</template>

<style lang="scss" scoped>
.my-component {
  background: var(--bg-card);
  color: var(--text-primary);
  padding: 16px;
  border-radius: 8px;
  border: 1px solid var(--border-default);
  
  &:hover {
    background: var(--hover-bg);
  }
}
</style>
```

### 常用 CSS 变量速查

```scss
// 背景
background: var(--bg-card);
background: var(--bg-base);
background: var(--bg-elevated);

// 文本
color: var(--text-primary);
color: var(--text-secondary);
color: var(--text-tertiary);

// 边框
border: 1px solid var(--border-default);
border-color: var(--border-strong);

// 交互
&:hover {
  background: var(--hover-bg);
}
```

## 迁移检查清单

如果你要添加新组件或修改现有组件：

- [ ] 使用 `<style lang="scss" scoped>` 而不是 `<style scoped>`
- [ ] 使用 CSS 变量而不是硬编码颜色值
- [ ] 确保在亮色和暗色主题下都测试过
- [ ] 如果需要主题切换功能，使用 `useTheme` composable
- [ ] 新增的颜色变量应该在 `_variables.scss` 中定义

## 技术细节

### 主题切换原理

1. 通过修改 `<html>` 元素的 `data-theme` 属性来切换主题
2. CSS 变量根据 `data-theme` 属性自动应用对应的值
3. 主题偏好保存在 localStorage 中

### 性能优化

- CSS 变量切换是浏览器原生支持，性能优秀
- 无需重新加载样式表
- 切换主题时无闪烁

## 常见问题

### Q: 如何添加新的主题颜色？
A: 在 `src/assets/themes/_variables.scss` 中为亮色和暗色主题都添加对应的变量。

### Q: 可以添加更多主题吗（如蓝色主题）？
A: 可以，在 `_variables.scss` 中添加 `[data-theme="blue"]` 选择器，并在 `useTheme.js` 中添加对应的切换逻辑。

### Q: 为什么要用 SCSS 而不是 CSS？
A: SCSS 提供了变量、嵌套、mixins 等功能，使样式代码更易维护和复用。

### Q: 主题切换会影响性能吗？
A: 不会，CSS 变量的切换是浏览器原生支持的，性能开销极小。

## 相关文档

- [主题系统详细文档](./src/assets/themes/README.md)
- [CSS 变量完整列表](./src/assets/themes/_variables.scss)
- [Mixins 工具](./src/assets/themes/_mixins.scss)
