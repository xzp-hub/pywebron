<script setup>
import {ref, onMounted, onUnmounted, reactive, computed} from 'vue'
import VChart from 'vue-echarts'
import {DesktopIcon} from 'tdesign-icons-vue-next'

const isDark = ref(false)
const pw = window.pywebron
const stream = pw?.interfaces?.stream

// 主题切换：监听系统偏好 + 手动切换
function applyTheme() {
  isDark.value = document.documentElement.getAttribute('data-theme') === 'dark'
    || window.matchMedia?.('(prefers-color-scheme: dark)').matches
}

onMounted(() => {
  applyTheme()
  const observer = new MutationObserver(applyTheme)
  observer.observe(document.documentElement, {attributes: true, attributeFilter: ['data-theme']})
  window.matchMedia?.('(prefers-color-scheme: dark)')?.addEventListener('change', applyTheme)
})

const monitors = [
  {key: 'cpu', label: 'CPU 使用率', color: '#00D4FF'},
  {key: 'ram', label: '内存使用率', color: '#00FF88'},
  {key: 'vrm', label: '交换区使用率', color: '#FF6B6B'}
]

const gaugeData = reactive({
  cpu: {usage: 0, stats: 'CPU使用率'},
  ram: {usage: 0, stats: '内存使用率'},
  vrm: {usage: 0, stats: '交换区使用率'}
})

function buildGaugeOption(val, label, color) {
  return {
    series: [{
      type: 'gauge',
      startAngle: 270,
      endAngle: -269.999,
      min: 0,
      max: 100,
      radius: '70px',
      pointer: {show: false},
      progress: {
        show: true,
        overlap: false,
        roundCap: false,
        clip: false,
        itemStyle: {color: color}
      },
      axisLine: {
        lineStyle: {
          width: 10,
          color: [[1, isDark.value ? 'rgba(255,255,255,0.15)' : 'rgba(0,0,0,0.08)']]
        }
      },
      axisTick: {show: false},
      splitLine: {show: false},
      axisLabel: {show: false},
      detail: {
        fontSize: 18,
        fontWeight: 'bold',
        color: isDark.value ? '#fff' : '#222',
        offsetCenter: [0, '-2%'],
        formatter: '{value}%',
        valueAnimation: true
      },
      title: {
        fontSize: 11,
        color: isDark.value ? '#ffffff' : 'rgba(0,0,0,0.5)',
        offsetCenter: [0, '22%']
      },
      data: [{value: parseFloat(val.toFixed(2)), name: label}],
      animationDurationUpdate: 600
    }]
  }
}

const gaugeOptions = computed(() => ({
  cpu: buildGaugeOption(gaugeData.cpu.usage, 'CPU使用率', monitors[0].color),
  ram: buildGaugeOption(gaugeData.ram.usage, '内存使用率', monitors[1].color),
  vrm: buildGaugeOption(gaugeData.vrm.usage, '交换区使用率', monitors[2].color)
}))

let monitorStream = null
let retryTimer = null

async function startMonitoring() {
  try {
    monitorStream = await stream('system_monitoring_stream')
    monitorStream.recv((data) => {
      const payload = data.data || data
      const info = payload.info
      monitors.forEach(m => {
        const d = info?.[m.key]
        if (d?.usage !== undefined) {
          gaugeData[m.key].usage = d.usage
          gaugeData[m.key].stats = d.stats || m.label
        }
      })
      if (info?.ios && typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('pywebron-io-update', {detail: {time: payload.time, ios: info.ios}}))
      }
    }).end(() => {
      retryTimer = setTimeout(startMonitoring, 1000)
    })
  } catch (e) {
    retryTimer = setTimeout(startMonitoring, 1000)
  }
}

onMounted(() => {
  startMonitoring()
})

onUnmounted(() => {
  if (retryTimer) clearTimeout(retryTimer)
})
</script>

<template>
  <div class="card">
    <div class="header">
      <div class="header-icon-box">
        <DesktopIcon class="header-icon"/>
      </div>
      <span class="header-title">系统监控</span>
    </div>
    <div class="body">
      <div v-for="m in monitors" :key="m.key" class="body-item">
        <v-chart :key="m.key" :option="gaugeOptions[m.key]"/>
      </div>
    </div>
    <div class="footer">
      <div class="footer-item" v-for="m in monitors" :key="m.key">{{ gaugeData[m.key].stats }}</div>
    </div>
  </div>
</template>

<style scoped>

.card {
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-card);
  box-sizing: border-box;
  border: 1px solid var(--border-default);
  height: auto;
  flex: none;
}

.header {
  height: 36px;
  display: flex;
  align-items: center;
  background: var(--bg-card-header);
  box-sizing: border-box;
  border-bottom: 1px solid var(--border-default);
  display: flex;
  padding-left: 6px;
  gap: 5px;
}

.header-icon-box {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: auto;
}

.header-icon {
  width: 16px;
  height: 16px;
  color: #9a8600;
}

.header-title {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1;
}

[data-theme="dark"] .header-title {
  color: #ffffff;
}

.body {
  display: flex;
  justify-content: space-between;
  gap: 6px;
  box-sizing: border-box;
  flex-shrink: 0;
  background: var(--bg-card);
}

[data-theme="dark"] .body {
  background: #1a1b1d;
}

.body-item {
  width: 150px;
  height: 150px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: 5px;
  box-sizing: border-box;
  background: transparent;
}

.footer {
  height: 36px;
  display: flex;
  justify-content: space-between;
  background: var(--bg-card-footer);
  box-sizing: border-box;
  border-top: 1px solid var(--border-default);
}


.footer-item {
  width: 150px;
  font-size: 14px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
}

[data-theme="dark"] .footer-item {
  color: #ffffff;
}
</style>
