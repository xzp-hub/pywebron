<script setup>
import { ref, onMounted, onUnmounted, reactive, computed } from 'vue'
import VChart from 'vue-echarts'
import { DesktopIcon } from 'tdesign-icons-vue-next'

const isDark = ref(false)
const pw = window.pywebron
const stream = pw?.interfaces?.stream

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
  <div class="monitor-panel">
    <div class="monitor-panel-header">
      <div class="monitor-panel-header-icon-box">
        <DesktopIcon class="monitor-panel-header-icon" />
      </div>
      <span class="monitor-panel-header-title">系统监控</span>
    </div>
    <div class="monitor-panel-body">
      <div class="monitor-gauges-container">
        <div v-for="m in monitors" :key="m.key" class="monitor-gauge-card">
          <v-chart class="monitor-gauge-chart" :option="gaugeOptions[m.key]" autoresize />
          <div class="monitor-gauge-stat-label">{{ gaugeData[m.key].stats }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.monitor-panel {
  border-radius: 5px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: light-dark(#ffffff, #1e1f21);
  box-sizing: border-box;
  box-shadow: inset 0 0 0 1px light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
  flex: 0 0 auto;
}

.monitor-panel-header {
  height: 30px;
  display: flex;
  align-items: center;
  gap: 5px;
  background: light-dark(#ffffff, rgba(184, 183, 183, .15));
  backdrop-filter: blur(6px);
  border-radius: 5px 5px 0 0;
  box-sizing: border-box;
  padding: 0 5px;
  box-shadow: inset 0 -1px 0 0 light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .35));
}

.monitor-panel-header-icon-box {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.monitor-panel-header-icon {
  width: 16px;
  height: 16px;
  color: #00D4FF;
}

.monitor-panel-header-title {
  font-size: 12px;
  font-weight: 600;
  color: light-dark(#333, #fff);
  letter-spacing: .5px;
  line-height: 1;
}

.monitor-panel-body {
  flex: 1;
  padding: 5px;
  min-height: 0;
  display: flex;
  flex-direction: column;
  box-shadow: inset 0 0 0 1px light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
}

.monitor-gauges-container {
  height: 160px;
  flex-shrink: 0;
  display: flex;
  gap: 5px;
  padding: 5px;
  box-sizing: border-box;
}

.monitor-gauge-card {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  border-radius: 5px;
  box-shadow: inset 0 0 0 1px light-dark(rgba(0, 0, 0, .3), rgba(255, 255, 255, .3));
}

.monitor-gauge-chart {
  flex: 1;
  min-height: 0;
  width: 100%;
}

.monitor-gauge-stat-label {
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
