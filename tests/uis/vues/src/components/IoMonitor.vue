<script setup>
import { ref, onMounted, onUnmounted, reactive, computed } from 'vue'
import { useTheme, formatSpeed, formatTotal } from '@/composables/usePywebron'
import VChart from 'vue-echarts'

const { isDark } = useTheme()

const ioType = ref('disk')

const lastIoData = ref(null)

const IO_MAX_POINTS = 5
const IO_COLORS = {
  disk: { read: '#00D4FF', write: '#FF6B6B' },
  net: { upload: '#00FF88', download: '#FFB347' }
}

const ioHistory = reactive({
  disk: { read: [], write: [], times: [] },
  net: { upload: [], download: [], times: [] }
})

const ioPanel = reactive({
  label1: '总读取', label2: '总写入',
  val1: '0 MB', val2: '0 MB',
  color1: '#00D4FF', color2: '#FF6B6B',
  legText1: '读取', legText2: '写入'
})

const chartOption = computed(() => {
  const type = ioType.value
  const history = ioHistory[type]
  const colors = IO_COLORS[type]
  const keys = type === 'disk' ? ['read', 'write'] : ['upload', 'download']
  const labels = type === 'disk' ? ['读取', '写入'] : ['上行', '下行']
  const colorKeys = type === 'disk' ? ['read', 'write'] : ['upload', 'download']
  const dark = isDark.value

  const times = history.times.slice(-IO_MAX_POINTS)
  const seriesData = keys.map((key, idx) => ({
    name: labels[idx],
    type: 'line',
    smooth: false,
    symbol: 'circle',
    symbolSize: 6,
    lineStyle: { color: colors[colorKeys[idx]], width: 2 },
    itemStyle: { color: colors[colorKeys[idx]] },
    areaStyle: {
      color: {
        type: 'linear',
        x: 0, y: 0, x2: 0, y2: 1,
        colorStops: [
          { offset: 0, color: colors[colorKeys[idx]] + '40' },
          { offset: 1, color: colors[colorKeys[idx]] + '05' }
        ]
      }
    },
    data: history[key].slice(-IO_MAX_POINTS)
  }))

  let yMax = 100
  if (lastIoData.value?.ios?.y_ticks) {
    const ticks = type === 'disk' ? lastIoData.value.ios.y_ticks.disk : lastIoData.value.ios.y_ticks.net
    if (ticks && ticks.length > 0) yMax = ticks[0]
  } else {
    let maxVal = 0
    keys.forEach(key => { history[key].forEach(v => { if (v > maxVal) maxVal = v }) })
    yMax = maxVal > 0 ? Math.ceil(maxVal * 1.2 / 100) * 100 : 100
  }

  return {
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(30, 30, 50, 0.95)',
      borderColor: 'rgba(255, 255, 255, 0.3)',
      borderRadius: 5,
      padding: [5, 8],
      textStyle: { color: '#fff', fontSize: 12 },
      formatter(params) {
        if (!params || !params.length) return ''
        let html = `<div style="color:rgba(255,255,255,.5);font-size:11px;margin-bottom:4px;">${params[0].axisValue}</div>`
        params.forEach(p => {
          html += `<div style="display:flex;align-items:center;gap:5px;">
            <span style="display:inline-block;width:10px;height:10px;border-radius:2px;background:${p.color};"></span>
            <span style="color:rgba(255,255,255,.7)">${p.seriesName}:</span>
            <span style="font-weight:600;margin-left:auto;">${formatSpeed(p.value)}</span>
          </div>`
        })
        return html
      }
    },
    grid: { top: 30, right: 15, bottom: 30, left: 55 },
    xAxis: {
      type: 'category',
      data: times,
      axisLine: { lineStyle: { color: dark ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.2)' } },
      axisTick: { show: false },
      axisLabel: { color: dark ? 'rgba(255,255,255,0.6)' : 'rgba(0,0,0,0.5)', fontSize: 12 }
    },
    yAxis: {
      type: 'value',
      max: yMax,
      splitLine: { lineStyle: { color: dark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.06)' } },
      axisLine: { lineStyle: { color: dark ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.2)' } },
      axisLabel: { color: dark ? 'rgba(255,255,255,0.6)' : 'rgba(0,0,0,0.5)', fontSize: 12 }
    },
    series: seriesData,
    animationDurationUpdate: 300
  }
})

function switchIoType(type) {
  ioType.value = type
  if (lastIoData.value) {
    updateIoPanel(lastIoData.value.ios)
  }
}

function updateIoPanel(ios) {
  if (ioType.value === 'disk') {
    ioPanel.label1 = '总读取'; ioPanel.label2 = '总写入'
    ioPanel.val1 = formatTotal(ios.disk_io.read_total)
    ioPanel.val2 = formatTotal(ios.disk_io.write_total)
    ioPanel.color1 = '#00D4FF'; ioPanel.color2 = '#FF6B6B'
    ioPanel.legText1 = '读取'; ioPanel.legText2 = '写入'
  } else {
    ioPanel.label1 = '总发送'; ioPanel.label2 = '总接收'
    ioPanel.val1 = formatTotal(ios.net_io.write_total)
    ioPanel.val2 = formatTotal(ios.net_io.read_total)
    ioPanel.color1 = '#00FF88'; ioPanel.color2 = '#FFB347'
    ioPanel.legText1 = '上行'; ioPanel.legText2 = '下行'
  }
}

function onIoUpdate(e) {
  const { time, ios } = e.detail
  const diskReadKB = ios.disk_io.read_speed
  const diskWriteKB = ios.disk_io.write_speed
  const netReadKB = ios.net_io.read_speed
  const netWriteKB = ios.net_io.write_speed

  ioHistory.disk.read.push(diskReadKB)
  ioHistory.disk.write.push(diskWriteKB)
  ioHistory.net.upload.push(netWriteKB)
  ioHistory.net.download.push(netReadKB)
  ioHistory.disk.times.push(time)
  ioHistory.net.times.push(time)

  Object.values(ioHistory).forEach(h => {
    Object.keys(h).forEach(k => {
      if (k !== 'times') { while (h[k].length > IO_MAX_POINTS) h[k].shift() }
    })
    while (h.times.length > IO_MAX_POINTS) h.times.shift()
  })

  lastIoData.value = { time, ios }
  updateIoPanel(ios)
}

onMounted(() => {
  window.addEventListener('pywebron-io-update', onIoUpdate)
})

onUnmounted(() => {
  window.removeEventListener('pywebron-io-update', onIoUpdate)
})
</script>

<template>
  <div class="panel io-monitor-panel">
    <div class="panel-header io-monitor-header">
      <div class="panel-header-icon-wrapper">
        <svg class="panel-header-icon io-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
        </svg>
      </div>
      <span class="panel-header-text">IO 监控</span>
      <div style="flex:1"></div>
      <div class="io-monitor-type-switch">
        <div class="io-monitor-type-switch-item" :class="{ active: ioType === 'disk' }" @click="switchIoType('disk')">磁盘IO</div>
        <div class="io-monitor-type-switch-item" :class="{ active: ioType === 'net' }" @click="switchIoType('net')">网络IO</div>
      </div>
    </div>
    <div class="panel-body io-monitor-chart-area">
      <v-chart class="io-monitor-chart" :option="chartOption" autoresize />
      <div class="io-monitor-legend">
        <span class="io-monitor-legend-item">
          <span class="io-monitor-legend-color" :style="{ background: ioPanel.color1 }"></span>
          <span class="io-monitor-legend-label">{{ ioPanel.legText1 }}</span>
        </span>
        <span class="io-monitor-legend-item">
          <span class="io-monitor-legend-color" :style="{ background: ioPanel.color2 }"></span>
          <span class="io-monitor-legend-label">{{ ioPanel.legText2 }}</span>
        </span>
        <span class="io-monitor-stat">
          <span class="io-monitor-stat-label">{{ ioPanel.label1 }}</span>
          <span class="io-monitor-stat-value">{{ ioPanel.val1 }}</span>
        </span>
        <span class="io-monitor-stat">
          <span class="io-monitor-stat-label">{{ ioPanel.label2 }}</span>
          <span class="io-monitor-stat-value">{{ ioPanel.val2 }}</span>
        </span>
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

.io-icon {
  color: #722ED1;
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

.io-monitor-panel {
  height: 350px;
  min-height: 350px;
}

.io-monitor-header {
  padding-right: 5px;
}

.io-monitor-type-switch {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}

.io-monitor-type-switch-item {
  padding: 2px 8px;
  font-size: 11px;
  color: light-dark(rgba(0, 0, 0, .55), rgba(255, 255, 255, .65));
  cursor: pointer;
  border-radius: 5px;
  transition: all .2s;
  white-space: nowrap;
}

.io-monitor-type-switch-item:hover {
  color: light-dark(#000, #fff);
  background: light-dark(rgba(0, 0, 0, .06), rgba(255, 255, 255, .12));
}

.io-monitor-type-switch-item.active {
  background: #722ED1;
  color: #fff;
}

.io-monitor-chart-area {
  flex: 1;
  min-height: 0;
  position: relative;
  padding: 5px;
}

.io-monitor-chart {
  width: 100%;
  height: calc(100% - 28px);
  min-height: 0;
}

.io-monitor-legend {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 5px 5px 0;
  white-space: nowrap;
}

.io-monitor-legend-item {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
}

.io-monitor-legend-color {
  width: 10px;
  height: 10px;
  border-radius: 2px;
  flex-shrink: 0;
}

.io-monitor-legend-label {
  font-size: 14px;
  color: light-dark(rgba(0, 0, 0, .65), rgba(255, 255, 255, .7));
}

.io-monitor-stat {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  border-left: 1px solid light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .25));
  padding-left: 18px;
}

.io-monitor-stat-label {
  color: light-dark(rgba(0, 0, 0, .55), rgba(255, 255, 255, .65));
}

.io-monitor-stat-label::after {
  content: ':';
}

.io-monitor-stat-value {
  font-weight: 600;
  color: light-dark(#222, #fff);
}
</style>
