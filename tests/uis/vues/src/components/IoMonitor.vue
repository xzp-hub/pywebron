<script setup>
import {ref, onMounted, onUnmounted, reactive, computed, watch} from 'vue'
import VChart from 'vue-echarts'
import {ChartLineDataIcon} from 'tdesign-icons-vue-next'

const isDark = ref(false)

function formatSpeed(bytes) {
  if (!bytes || bytes === 0) return '0 B/s'
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  let i = 0
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024;
    i++
  }
  return bytes.toFixed(i > 0 ? 1 : 0) + ' ' + units[i]
}

function formatTotal(bytes) {
  if (!bytes || bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024;
    i++
  }
  return bytes.toFixed(i > 0 ? 1 : 0) + ' ' + units[i]
}

const ioType = ref('disk')

const lastIoData = ref(null)

const IO_MAX_POINTS = 5
const IO_COLORS = {
  disk: {read: '#00D4FF', write: '#FF6B6B'},
  net: {upload: '#00FF88', download: '#FFB347'}
}

const ioHistory = reactive({
  disk: {read: [], write: [], times: []},
  net: {upload: [], download: [], times: []}
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
  const timeMap = Object.fromEntries(times.map((t, i) => [i, t]))
  const seriesData = keys.map((key, idx) => ({
    name: labels[idx],
    type: 'line',
    smooth: false,
    symbol: 'circle',
    symbolSize: 6,
    lineStyle: {color: colors[colorKeys[idx]], width: 2},
    itemStyle: {color: colors[colorKeys[idx]]},
    areaStyle: {
      color: {
        type: 'linear',
        x: 0, y: 0, x2: 0, y2: 1,
        colorStops: [
          {offset: 0, color: colors[colorKeys[idx]] + '40'},
          {offset: 1, color: colors[colorKeys[idx]] + '05'}
        ]
      }
    },
    data: history[key].slice(-IO_MAX_POINTS).map((v, i) => [i, v])
  }))

  let yMax = 100
  if (lastIoData.value?.ios?.y_ticks) {
    const ticks = type === 'disk' ? lastIoData.value.ios.y_ticks.disk : lastIoData.value.ios.y_ticks.net
    if (ticks && ticks.length > 0) yMax = ticks[0]
  } else {
    let maxVal = 0
    keys.forEach(key => {
      history[key].forEach(v => {
        if (v > maxVal) maxVal = v
      })
    })
    yMax = maxVal > 0 ? Math.ceil(maxVal * 1.2 / 100) * 100 : 100
  }

  return {
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(30, 30, 50, 0.95)',
      borderColor: 'rgba(255, 255, 255, 0.3)',
      borderRadius: 5,
      padding: [5, 8],
      textStyle: {color: '#fff', fontSize: 12},
      formatter(params) {
        if (!params || !params.length) return ''
        let html = `<div style="color:rgba(255,255,255,.5);font-size:11px;margin-bottom:4px;">${timeMap[params[0].axisValue] || params[0].axisValue}</div>`
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
    grid: {top: 8, right: 8, bottom: 22, left: 45},
    xAxis: {
      type: 'value',
      min: 0,
      axisLine: {lineStyle: {color: dark ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.2)'}},
      axisTick: {show: false},
      axisLabel: {
        color: dark ? 'rgba(255,255,255,0.6)' : 'rgba(0,0,0,0.5)', fontSize: 12,
        formatter: (v) => timeMap[v] || ''
      }
    },
    yAxis: {
      type: 'value',
      max: yMax,
      splitLine: {lineStyle: {color: dark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.06)'}},
      axisLine: {show: true, lineStyle: {color: dark ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.2)'}},
      axisLabel: {color: dark ? 'rgba(255,255,255,0.6)' : 'rgba(0,0,0,0.5)', fontSize: 12}
    },
    series: seriesData,
    animationDurationUpdate: 300
  }
})

function switchIoType(type) {
  if (lastIoData.value) {
    updateIoPanel(lastIoData.value.ios)
  }
}

watch(ioType, (val) => {
  switchIoType(val)
})

function updateIoPanel(ios) {
  if (ioType.value === 'disk') {
    ioPanel.label1 = '总读取';
    ioPanel.label2 = '总写入'
    ioPanel.val1 = formatTotal(ios.disk_io.read_total)
    ioPanel.val2 = formatTotal(ios.disk_io.write_total)
    ioPanel.color1 = '#00D4FF';
    ioPanel.color2 = '#FF6B6B'
    ioPanel.legText1 = '读取';
    ioPanel.legText2 = '写入'
  } else {
    ioPanel.label1 = '总发送';
    ioPanel.label2 = '总接收'
    ioPanel.val1 = formatTotal(ios.net_io.write_total)
    ioPanel.val2 = formatTotal(ios.net_io.read_total)
    ioPanel.color1 = '#00FF88';
    ioPanel.color2 = '#FFB347'
    ioPanel.legText1 = '上行';
    ioPanel.legText2 = '下行'
  }
}

function onIoUpdate(e) {
  const {time, ios} = e.detail
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
      if (k !== 'times') {
        while (h[k].length > IO_MAX_POINTS) h[k].shift()
      }
    })
    while (h.times.length > IO_MAX_POINTS) h.times.shift()
  })

  lastIoData.value = {time, ios}
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
  <div class="card">
    <div class="header">
      <div class="header-item">
        <div class="header-icon-box">
          <ChartLineDataIcon class="header-icon"/>
        </div>
        <span class="header-title">IO 监控</span>
      </div>
      <div class="header-item">
        <span class="legend-mark" :style="{ background: ioPanel.color1 }"></span>
        <span class="legend-text">{{ ioPanel.legText1 }}</span>
        <span class="legend-divider"></span>
        <span class="legend-mark" :style="{ background: ioPanel.color2 }"></span>
        <span class="legend-text">{{ ioPanel.legText2 }}</span>
      </div>
      <div class="header-item">
        <div
          class="io-type-btn"
          :class="{ active: ioType === 'disk' }"
          @click="ioType = 'disk'"
        >磁盘IO</div>
        <div
          class="io-type-btn"
          :class="{ active: ioType === 'net' }"
          @click="ioType = 'net'"
        >网络IO</div>
      </div>
    </div>
    <div class="body">
      <v-chart class="io-monitor-line-chart" :option="chartOption" autoresize/>
    </div>
    <div class="footer">
      <div class="footer-item">
        <span class="footer-label">{{ ioPanel.label1 }}:</span>
        <span class="footer-value">{{ ioPanel.val1 }}</span>
      </div>
      <div class="footer-item">
        <span class="footer-label">{{ ioPanel.label2 }}:</span>
        <span class="footer-value">{{ ioPanel.val2 }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card {
  height: auto;
  flex: 1;
  display: flex;
  border-radius: 6px;
  flex-direction: column;
  overflow: hidden;
  background: light-dark(#ffffff, #1e1f21);
  box-sizing: border-box;
  border: 1px solid light-dark(rgba(0, 0, 0, .2), rgba(255, 255, 255, .2));
}

.header {
  height: 30px;
  display: flex;
  padding-left: 6px;
  justify-content: space-between;
  align-items: center;
  background: light-dark(#ffffff, rgba(184, 183, 183, .15));
  box-sizing: border-box;
  border-bottom: 1px solid light-dark(rgba(0, 0, 0, .2), rgba(255, 255, 255, .2));
}

.header-icon-box {
  width: auto;
  height: 30px;
  display: flex;
  align-items: center;
}

.header-icon {
  width: 16px;
  height: 16px;
  color: #8201f8;
}

.header-title {
  font-size: 14px;
  color: light-dark(#5e5e5e, #fff);
}

.header-item {
  width: auto;
  height: 30px;
  display: flex;
  align-items: center;
}

.header-item:nth-child(1), .header-item:nth-child(2) {
  gap: 5px;
}

.io-type-btn {
  flex: 1;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  cursor: pointer;
  user-select: none;
  padding: 0 16px;
  border: 1px solid light-dark(rgba(0, 0, 0, .15), rgba(255, 255, 255, .25));
  background: light-dark(#f5f5f5, #2a2a2a);
  color: light-dark(#858a99, #ccc);
  transition: all 0.2s ease;
}

.io-type-btn.active {
  background: #8201f8;
  color: #fff;
  border-color: #8201f8;
}

.header-item:nth-child(2) {
  justify-content: center;
}



.legend-mark {
  width: 10px;
  height: 10px;
  border-radius: 2px;
  flex-shrink: 0;
}

.legend-text {
  font-size: 14px;
  color: light-dark(#5e5e5e, #fff);
}

.legend-divider {
  width: 1px;
  height: 10px;
  background: light-dark(rgba(0, 0, 0, .2), rgba(255, 255, 255, .3));
  display: inline-block;
  flex-shrink: 0;
}

.body {
  flex: 1;
  min-height: 0;
  position: relative;
  padding: 2px;
  display: flex;
  flex-direction: column;
}

.io-monitor-line-chart {
  width: 100%;
  height: 100%;
  min-height: 0;
}

.footer {
  height: 30px;
  display: flex;
  justify-content: space-between;
  background: light-dark(#ffffff, rgba(184, 183, 183, .15));
  box-sizing: border-box;
  border-top: 1px solid light-dark(rgba(0, 0, 0, .2), rgba(255, 255, 255, .2));
}

.footer-item {
  flex: 1;
  font-size: 14px;
  color: light-dark(#5e5e5e, #fff);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
}

.footer-value {
  font-weight: 600;
}

</style>
