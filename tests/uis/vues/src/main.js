import { createApp } from 'vue'
import TDesign from 'tdesign-vue-next'
import 'tdesign-vue-next/es/style/index.css'
import App from './App.vue'
// 引入主题系统（包含 CSS 变量 + 全局基础样式）
import './assets/main.css'
// 引入主题切换功能
import { useTheme } from './composables/useTheme'

import { use } from 'echarts/core'
import { GaugeChart, LineChart } from 'echarts/charts'
import { GridComponent, TooltipComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

use([GaugeChart, LineChart, GridComponent, TooltipComponent, CanvasRenderer])

// 初始化主题
const { initTheme } = useTheme()
initTheme()

const app = createApp(App)
app.use(TDesign)
app.mount('#app')
