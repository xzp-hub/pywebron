<script setup>
import { ref, onMounted, onUnmounted, reactive, computed, watch } from 'vue'
import { usePywebron, useTheme } from '@/composables/usePywebron'
import VChart from 'vue-echarts'

const { isDark } = useTheme()
const { stream } = usePywebron()

const monitors = [
  { key: 'cpu', label: 'CPU 使用率', color: '#00D4FF' },
  { key: 'ram', label: '内存使用率', color: '#00FF88' },
  { key: 'vrm', label: '交换区使用率', color: '#FF6B6B' }
]

const gaugeData = reactive({
  cpu: { usage: 0, stats: 'CPU使用率' },
  ram: { usage: 0, stats: '内存使用率' },
  vrm: { usage: 0, stats: '交换区使用率' }
})

function buildGaugeOption(val, label, color) {
  return {
    series: [{
      type: 'gauge',
      startAngle: 225,
      endAngle: -45,
      min: 0,
      max: 100,
      pointer: { show: false },
      progress: {
        show: true,
        overlap: false,
        roundCap: false,
        clip: false,
        itemStyle: { color: color }
      },
      axisLine: {
        lineStyle: {
          width: 8,
          color: [[1, isDark.value ? 'rgba(255,255,255,0.15)' : 'rgba(0,0,0,0.08)']]
        }
      },
      axisTick: { show: false },
      splitLine: { show: false },
      axisLabel: { show: false },
      detail: {
        fontSize: 16,
        fontWeight: 'bold',
        color: isDark.value ? '#fff' : '#222',
        offsetCenter: [0, '-5%'],
        formatter: '{value}%',
        valueAnimation: true
      },
      title: {
        fontSize: 11,
        color: isDark.value ? 'rgba(255,255,255,0.7)' : 'rgba(0,0,0,0.5)',
        offsetCenter: [0, '30%']
      },
      data: [{ value: parseFloat(val.toFixed(2)), name: label }],
      animationDurationUpdate: 600
    }]
  }
}

const gaugeOptions = computed(() => ({
  cpu: buildGaugeOption(gaugeData.cpu.usage, 'CPU', monitors[0].color),
  ram: buildGaugeOption(gaugeData.ram.usage, '内存', monitors[1].color),
  vrm: buildGaugeOption(gaugeData.vrm.usage, '交换区', monitors[2].color)
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
        window.dispatchEvent(new CustomEvent('pywebron-io-update', { detail: { time: payload.time, ios: info.ios } }))
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
  <div class="panel system-monitor-panel">
    <div class="panel-header">
      <div class="panel-header-icon-wrapper">
        <svg class="panel-header-icon monitor-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="2" y="3" width="20" height="14" rx="2"/>
          <line x1="8" y1="21" x2="16" y2="21"/>
          <line x1="12" y1="17" x2="12" y2="21"/>
        </svg>
      </div>
      <span class="panel-header-text">系统监控</span>
    </div>
    <div class="panel-body system-monitor-body">
      <div class="system-monitor-gauges">
        <div v-for="m in monitors" :key="m.key" class="system-monitor-gauge-item">
          <v-chart class="system-monitor-gauge-chart" :option="gaugeOptions[m.key]" autoresize />
          <div class="system-monitor-gauge-label">{{ gaugeData[m.key].stats }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel {
  border-radius: 5px;
  border: 1px solid light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: light-dark(#ffffff, #1e1f21);
  box-sizing: border-box;
}

.panel-header {
  height: 30px;
  display: flex;
  align-items: center;
  gap: 5px;
  background: light-dark(#ffffff, rgba(184, 183, 183, .15));
  backdrop-filter: blur(6px);
  border-bottom: 1px solid light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .35));
  border-radius: 5px 5px 0 0;
  box-sizing: border-box;
  padding: 0 5px;
}

.panel-header-icon-wrapper {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.panel-header-icon {
  width: 16px;
  height: 16px;
}

.monitor-icon {
  color: #00D4FF;
}

.panel-header-text {
  font-size: 12px;
  font-weight: 600;
  color: light-dark(#333, #fff);
  letter-spacing: .5px;
  line-height: 1;
}

.panel-body {
  flex: 1;
  padding: 5px;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.system-monitor-panel {
  flex: 0 0 auto;
}

.system-monitor-body {
  border: 1px solid light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
}

.system-monitor-gauges {
  height: 160px;
  flex-shrink: 0;
  display: flex;
  gap: 5px;
  padding: 5px;
  box-sizing: border-box;
}

.system-monitor-gauge-item {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  border: 1px solid light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
  border-radius: 5px;
}

.system-monitor-gauge-chart {
  flex: 1;
  min-height: 0;
  width: 100%;
}

.system-monitor-gauge-label {
  height: 18px;
  font-weight: 600;
  color: light-dark(#555, rgba(255, 255, 255, .8));
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
}
</style>
