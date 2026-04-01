// IO 监控数据
let ioType = 'disk';
let lastIoData = null;
let ioDataPoints = [];  // 存储数据点位置用于 tooltip
let ioHoveredIndex = -1;  // 当前悬停的数据点索引
const ioHistory = {disk: {read: [], write: [], times: []}, net: {upload: [], download: [], times: []}};
const IO_MAX_POINTS = 5;  // 只保留 5 个数据点
const IO_COLORS = {
    disk: {read: '#00D4FF', write: '#FF6B6B'},
    net: {upload: '#00FF88', download: '#FFB347'}
};

async function windowAction(type) {
    const map = {min: 'minimize_window', max: 'maximize_window', rep: 'reappear_window', shut: 'shutdown_window'};
    const action = type === 'toggle' ? (isMaximized ? 'rep' : 'max') : type;
    try {
        const res = await window.pywebron.invoke('window_controls_invoke', {control_type: map[action]}, 5000);
        if (res?.code === 200) {
            isMaximized = action === 'max' || (type === 'toggle' && !isMaximized);
            console.log('[Window] isMaximized after=', isMaximized);
        }
        if (['max', 'rep', 'toggle'].includes(type)) {
            $('maximizeRestoreBtn').title = isMaximized ? '还原' : '最大化';
            $('maximizeRestoreBtn').querySelector('svg').innerHTML = isMaximized
                ? '<rect x="4" y="8" width="12" height="12" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><rect x="8" y="4" width="12" height="12" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>'
                : '<rect x="4" y="4" width="16" height="16" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>';
        }
    } catch (e) {
        console.error('[Window]', e.message);
    }
}

function drawGauge(canvas, val, label, color, anim = true) {
    const ctx = canvas.getContext('2d'), dpr = devicePixelRatio || 1, rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    const cx = rect.width / 2, cy = rect.height / 2, r = (Math.min(cx, cy) - 10) * 1.08, lw = 8;
    const id = canvas.id;
    let cur = gaugeAnims[id]?.cur || 0;
    gaugeAnims[id] = {val, cur};

    const render = v => {
        ctx.clearRect(0, 0, rect.width, rect.height);
        ctx.beginPath();
        ctx.arc(cx, cy, r, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(255,255,255,0.2)';
        ctx.lineWidth = lw;
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(cx, cy, r, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2 * v / 100);
        ctx.strokeStyle = color;
        ctx.lineCap = 'butt';
        ctx.stroke();
        ctx.fillStyle = '#fff';
        ctx.font = `bold ${FONTS.percent}px Arial`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(v.toFixed(2) + '%', cx, cy - 3);
        ctx.font = `${FONTS.label}px Arial`;
        ctx.fillStyle = 'rgba(255,255,255,0.8)';
        ctx.fillText(label, cx, cy + 15);
    };

    if (!anim) {
        render(val);
        gaugeAnims[id].cur = val;
        return;
    }
    const frame = () => {
        const diff = gaugeAnims[id].val - gaugeAnims[id].cur;
        if (Math.abs(diff) < 0.1) {
            render(gaugeAnims[id].val);
            return;
        }
        gaugeAnims[id].cur += diff * 0.15;
        render(gaugeAnims[id].cur);
        requestAnimationFrame(frame);
    };
    frame();
}

function drawIoChart(yTicks) {
    const canvas = $('ioCanvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d'), dpr = devicePixelRatio || 1, rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    const w = rect.width, h = rect.height;
    const pad = {t: 20, r: 15, b: 35, l: 55};
    const cw = w - pad.l - pad.r, ch = h - pad.t - pad.b;

    ctx.clearRect(0, 0, w, h);

    const history = ioHistory[ioType];
    const colors = IO_COLORS[ioType];
    const keys = ioType === 'disk' ? ['read', 'write'] : ['upload', 'download'];
    const labels = ioType === 'disk' ? ['读取', '写入'] : ['上行', '下行'];

    // 使用后端返回的 Y 轴刻度，或者动态计算
    let yMax = 100;
    if (yTicks) {
        const ticks = ioType === 'disk' ? yTicks.disk : yTicks.net;
        if (ticks && ticks.length > 0) {
            yMax = ticks[0];
        }
    } else {
        let maxVal = 0;
        keys.forEach(key => {
            history[key].forEach(v => {
                if (v > maxVal) maxVal = v;
            });
        });
        yMax = maxVal > 0 ? Math.ceil(maxVal * 1.2 / 100) * 100 : 100;
    }

    const dataLen = history.times.length;
    if (dataLen < 1) return;

    // 清空数据点
    ioDataPoints = [];
    const maxPoints = 5;
    const effectiveLen = Math.min(dataLen, maxPoints);
    const getX = (i) => pad.l + cw * i / (maxPoints - 1);

    // 存储每个数据点的信息
    for (let i = 0; i < effectiveLen; i++) {
        const x = getX(i);
        const pointData = {x, time: history.times[i], values: {}};
        keys.forEach((key, ki) => {
            const val = history[key][i];
            const y = pad.t + ch * (1 - Math.min(val, yMax) / yMax);
            pointData.values[key] = {val, y, label: labels[ki], color: colors[key]};
        });
        ioDataPoints.push(pointData);
    }

    // 绘制 Y 轴垂直线
    ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    ctx.beginPath();
    ctx.moveTo(pad.l, pad.t);
    ctx.lineTo(pad.l, h - pad.b);
    ctx.stroke();

    // 绘制网格线和 Y 轴刻度
    ctx.fillStyle = 'rgba(255,255,255,0.6)';
    ctx.font = '13px Arial';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';

    const ticks = (yTicks && (ioType === 'disk' ? yTicks.disk : yTicks.net)) || [yMax, yMax * 0.8, yMax * 0.6, yMax * 0.4, yMax * 0.2, 0];
    const tickCount = ticks.length;

    for (let i = 0; i < tickCount; i++) {
        const y = pad.t + ch * i / (tickCount - 1);
        ctx.strokeStyle = 'rgba(255,255,255,0.1)';
        ctx.beginPath();
        ctx.moveTo(pad.l, y);
        ctx.lineTo(w - pad.r, y);
        ctx.stroke();
        ctx.fillText(ticks[i].toFixed(0), pad.l - 8, y);
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.beginPath();
        ctx.moveTo(pad.l - 5, y);
        ctx.lineTo(pad.l, y);
        ctx.stroke();
    }

    // 绘制 X 轴基线
    ctx.strokeStyle = 'rgba(255,255,255,0.3)';
    ctx.beginPath();
    ctx.moveTo(pad.l, h - pad.b);
    ctx.lineTo(w - pad.r, h - pad.b);
    ctx.stroke();

    // 绘制面积图、线条和数据点
    keys.forEach((key) => {
        const arr = history[key];
        if (arr.length < 1) return;

        // 面积图
        ctx.beginPath();
        arr.forEach((v, i) => {
            if (i >= maxPoints) return;
            const x = getX(i);
            const y = pad.t + ch * (1 - Math.min(v, yMax) / yMax);
            i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
        });
        const lastIdx = Math.min(arr.length, maxPoints) - 1;
        ctx.lineTo(getX(lastIdx), pad.t + ch);
        ctx.lineTo(getX(0), pad.t + ch);
        ctx.closePath();
        const grad = ctx.createLinearGradient(0, pad.t, 0, pad.t + ch);
        grad.addColorStep(0, colors[key] + '40');
        grad.addColorStep(1, colors[key] + '05');
        ctx.fillStyle = grad;
        ctx.fill();

        // 线条
        ctx.beginPath();
        arr.forEach((v, i) => {
            if (i >= maxPoints) return;
            const x = getX(i);
            const y = pad.t + ch * (1 - Math.min(v, yMax) / yMax);
            i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
        });
        ctx.strokeStyle = colors[key];
        ctx.lineWidth = 2;
        ctx.stroke();

        // 数据点（方块）
        arr.forEach((v, i) => {
            if (i >= maxPoints) return;
            const x = getX(i);
            const y = pad.t + ch * (1 - Math.min(v, yMax) / yMax);
            ctx.fillStyle = colors[key];
            ctx.fillRect(x - 3, y - 3, 6, 6);
        });
    });

    // X 轴时间标签 - 只显示最多 5 个
    if (dataLen > 0) {
        ctx.fillStyle = 'rgba(255,255,255,0.7)';
        ctx.font = '13px Arial';
        const labelY = h - pad.b + 18;
        ctx.strokeStyle = 'rgba(255,255,255,0.3)';
        ctx.textBaseline = 'top';

        const displayCount = Math.min(dataLen, maxPoints);
        for (let i = 0; i < displayCount; i++) {
            const x = getX(i);

            if (i === 0) ctx.textAlign = 'left';
            else if (i === displayCount - 1) ctx.textAlign = 'right';
            else ctx.textAlign = 'center';

            ctx.fillText(history.times[i], x, labelY);
            ctx.beginPath();
            ctx.moveTo(x, h - pad.b);
            ctx.lineTo(x, h - pad.b + 5);
            ctx.stroke();
        }
    }

    // 如果 tooltip 正在显示，更新其内容
    if (ioHoveredIndex >= 0 && ioHoveredIndex < ioDataPoints.length) {
        updateTooltipContent();
    }
}

// IO 图表鼠标悬停处理
function setupIoChartHover() {
    const canvas = $('ioCanvas');
    const tooltip = $('ioTooltip');
    if (!canvas || !tooltip) return;

    canvas.addEventListener('mousemove', (e) => {
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        // 找到最近的数据点
        let nearestIndex = -1;
        let minDist = 30; // 检测半径

        ioDataPoints.forEach((point, idx) => {
            const dist = Math.abs(x - point.x);
            if (dist < minDist) {
                minDist = dist;
                nearestIndex = idx;
            }
        });

        if (nearestIndex !== -1 && nearestIndex !== ioHoveredIndex) {
            ioHoveredIndex = nearestIndex;
            updateTooltipContent();
            tooltip.classList.add('show');
        }

        if (nearestIndex !== -1) {
            const nearestPoint = ioDataPoints[nearestIndex];
            // 定位 tooltip
            let tx = nearestPoint.x + 10;
            let ty = y - 20;
            if (tx + 120 > rect.width) tx = nearestPoint.x - 130;
            if (ty < 10) ty = 10;
            tooltip.style.left = tx + 'px';
            tooltip.style.top = ty + 'px';
        } else {
            tooltip.classList.remove('show');
            ioHoveredIndex = -1;
        }
    });

    canvas.addEventListener('mouseleave', () => {
        tooltip.classList.remove('show');
        ioHoveredIndex = -1;
    });
}

// 更新 tooltip 内容
function updateTooltipContent() {
    const tooltip = $('ioTooltip');
    if (!tooltip || ioHoveredIndex < 0 || ioHoveredIndex >= ioDataPoints.length) return;

    const point = ioDataPoints[ioHoveredIndex];
    if (!point) return;

    const keys = ioType === 'disk' ? ['read', 'write'] : ['upload', 'download'];
    let html = `<div style="color:rgba(255,255,255,.5);font-size:11px;margin-bottom:4px;">${point.time}</div>`;
    keys.forEach(key => {
        const v = point.values[key];
        if (v) {
            html += `<div class="io-tooltip-row">
                <span class="io-tooltip-color" style="background:${v.color}"></span>
                <span class="io-tooltip-label">${v.label}:</span>
                <span class="io-tooltip-val">${formatSpeed(v.val)}</span>
            </div>`;
        }
    });
    tooltip.innerHTML = html;
}

function formatKB(kb) {
    if (kb >= 1024 * 1024) return (kb / 1024 / 1024).toFixed(0) + ' GB/s';
    if (kb >= 1024) return (kb / 1024).toFixed(0) + ' MB/s';
    return kb.toFixed(0) + ' KB/s';
}

// 格式化速度值
function formatSpeed(val) {
    return val.toFixed(2) + ' KB/S';
}

// 格式化总量值
function formatTotal(val) {
    return val.toFixed(2) + ' MB';
}

function updateIoData(time, ios) {
    lastIoData = {time, ios};

    // 获取速度值
    const diskReadKB = ios.disk_io.read_speed;
    const diskWriteKB = ios.disk_io.write_speed;
    const netReadKB = ios.net_io.read_speed;
    const netWriteKB = ios.net_io.write_speed;

    ioHistory.disk.read.push(diskReadKB);
    ioHistory.disk.write.push(diskWriteKB);
    ioHistory.net.upload.push(netWriteKB);  // 上行 = 发送
    ioHistory.net.download.push(netReadKB); // 下行 = 接收
    ioHistory.disk.times.push(time);
    ioHistory.net.times.push(time);

    Object.values(ioHistory).forEach(h => {
        Object.keys(h).forEach(k => {
            if (k !== 'times') {
                while (h[k].length > IO_MAX_POINTS) h[k].shift();
            }
        });
        while (h.times.length > IO_MAX_POINTS) h.times.shift();
    });

    drawIoChart(ios.y_ticks);
    updateIoPanel(ios);
}

function updateIoPanel(ios) {
    if (ioType === 'disk') {
        $('ioLabel1').textContent = '总读取';
        $('ioLabel2').textContent = '总写入';
        $('ioVal1').textContent = formatTotal(ios.disk_io.read_total);
        $('ioVal2').textContent = formatTotal(ios.disk_io.write_total);
        $('ioColor1').style.background = '#00D4FF';
        $('ioColor2').style.background = '#FF6B6B';
        $('ioLegText1').textContent = '读取';
        $('ioLegText2').textContent = '写入';
    } else {
        $('ioLabel1').textContent = '总发送';
        $('ioLabel2').textContent = '总接收';
        $('ioVal1').textContent = formatTotal(ios.net_io.write_total);
        $('ioVal2').textContent = formatTotal(ios.net_io.read_total);
        $('ioColor1').style.background = '#00FF88';
        $('ioColor2').style.background = '#FFB347';
        $('ioLegText1').textContent = '上行';
        $('ioLegText2').textContent = '下行';
    }
}

function switchIoType(type) {
    ioType = type;
    $('diskTab').classList.toggle('active', type === 'disk');
    $('netTab').classList.toggle('active', type === 'net');
    if (lastIoData) {
        drawIoChart(lastIoData.ios.y_ticks);
        updateIoPanel(lastIoData.ios);
    }
}

async function startMonitoring() {
    if (!window.pywebron?.stream) return setTimeout(startMonitoring, 100);
    const t0 = performance.now();
    try {
        const t1 = performance.now();
        const stream = await window.pywebron.stream('system_monitoring_stream');
        const t2 = performance.now();
        console.log(`[Timing][Frontend] stream() 连接建立耗时：${(t2 - t1).toFixed(1)}ms`);
        let firstData = true;
        stream.recv(data => {
            if (firstData) {
                firstData = false;
                console.log(`[Timing][Frontend] 首次数据耗时：${(performance.now() - t0).toFixed(1)}ms (从窗口加载开始计)`);
                console.log(`[Timing][Frontend] stream 连接建立后到首次数据：${(performance.now() - t2).toFixed(1)}ms (从连接建立完成开始计)`);
            }
            // 数据在 payload.data 中
            const payload = data.data || data;
            const time = payload.time;
            const info = payload.info;
            MONITORS.forEach(m => {
                const d = info?.[m.key];
                if (d?.usage !== undefined) {
                    drawGauge($(m.canvas), d.usage, m.label, m.color);
                    $(m.header).textContent = d.stats || m.label;
                }
            });
            if (info?.ios) updateIoData(time, info.ios);
        }).end(() => setTimeout(startMonitoring, 1000));
    } catch (e) {
        console.error('[Monitor]', e);
        setTimeout(startMonitoring, 1000);
    }
}

async function startChat() {
    if (!window.pywebron?.stream) return setTimeout(startChat, 100);
    try {
        chatStream = await window.pywebron.stream('chat_room_stream');
        chatStream.recv(displayMsg);
    } catch (e) {
        console.error('[Chat]', e);
    }
}

async function downloadFile() {
    const btn = $('downloadBtn'), status = $('downloadStatus');
    btn.disabled = true;
    status.textContent = '正在打开保存对话框...';
    try {
        const res = await window.pywebron.invoke('file_download_invoke');
        if (res?.code !== 200) status.textContent = `保存失败：${res?.mssg || '未知错误'}`;
        else if (res.data?.cancelled) status.textContent = '已取消保存';
        else if (res.data?.saved_path) status.innerHTML = `✅ 文件已保存:<br><span style="font-size:11px;color:#00D4FF">${res.data.saved_path}</span>`;
        else status.textContent = '保存失败：未返回文件路径';
    } catch (e) {
        status.textContent = `下载失败：${e.message}`;
    } finally {
        btn.disabled = false;
    }
}

async function runCpuTask() {
    const btn = $('cpuTaskBtn'), status = $('cpuTaskStatus');
    btn.disabled = true;
    status.textContent = '正在执行 CPU 密集任务...';
    status.style.color = '#00D4FF';
    const startTime = Date.now();
    try {
        const res = await window.pywebron.invoke('cpu_intensive_task_invoke_command');
        const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);
        if (res?.code === 200) {
            const data = res.data || {};
            status.innerHTML = `✅ 任务完成<br><span style="font-size:11px">结果：${data.res || 'N/A'}<br>耗时：${data.time?.toFixed(3) || elapsed}s</span>`;
            status.style.color = '#00FF88';
        } else {
            status.textContent = `任务失败：${res?.mssg || '未知错误'}`;
            status.style.color = '#FF6B6B';
        }
    } catch (e) {
        status.textContent = `执行失败：${e.message}`;
        status.style.color = '#FF6B6B';
    } finally {
        btn.disabled = false;
    }
}

async function createNewWindow() {
    const btn = $('createWindowBtn'), status = $('createWindowStatus');
    btn.disabled = true;
    status.textContent = '正在创建新窗口...';
    status.style.color = '#00D4FF';
    try {
        const res = await window.pywebron.invoke('running_create_window_invoke_handle');
        if (res?.code === 200) {
            status.innerHTML = `✅ ${res.mssg || '窗口创建成功'}<br><span style="font-size:11px;color:#00D4FF">结果：${res.data}</span>`;
            status.style.color = '#00FF88';
        } else {
            status.textContent = `创建失败：${res?.mssg || '未知错误'}`;
            status.style.color = '#FF6B6B';
        }
    } catch (e) {
        status.textContent = `创建失败：${e.message}`;
        status.style.color = '#FF6B6B';
    } finally {
        btn.disabled = false;
    }
}

function escapeHtml(t) {
    const d = document.createElement('div');
    d.textContent = t;
    return d.innerHTML;
}

function displayMsg(data, isLocal = false) {
    const container = $('chatMessages');
    if (!container) return;
    const type = data.type || data.data?.type || 'message';
    const msg = data.mssg || data.message || (data.mssg?.message) || '';
    if (!msg.trim()) return;

    if (type === 'system' && msg === '欢迎加入聊天室') $('welcomeMessage').style.display = 'none';

    const id = type === 'system' ? `sys-${msg}` : `${isLocal ? 'local' : 'remote'}-${data.window_id || 'u'}-${Date.now()}-${msg}`;
    if (msgIds.has(id)) return;
    msgIds.add(id);

    const div = document.createElement('div');
    div.className = `chat_msg ${type === 'system' ? 'system' : isLocal ? 'self' : 'other'}`;
    if (type === 'system') {
        div.innerHTML = `<div class="chat_msg_content">${escapeHtml(msg)}</div>`;
    } else {
        const av = isLocal ? 'https://api.dicebear.com/7.x/avataaars/svg?seed=user' : 'https://api.dicebear.com/7.x/bottts/svg?seed=backend';
        const content = `<div class="chat_msg_content">${escapeHtml(msg)}</div>`;
        const avatar = `<img class="chat_avatar" src="${av}">`;
        div.innerHTML = `<div class="chat_msg_row ${isLocal ? 'self' : 'other'}">${isLocal ? content + avatar : avatar + content}</div>`;
    }
    container.appendChild(div);
    container.scrollTop = container.scrollHeight;
}

function sendMsg() {
    const input = $('chatInput'), msg = input.value.trim();
    if (!msg || !chatStream?.send) return;
    displayMsg({type: 'message', message: msg, window_id: window.pywebron.window_id}, true);
    chatStream.send(msg);
    input.value = '';
}

async function init() {
    MONITORS.forEach(m => {
        $(m.header).style.fontSize = FONTS.stats + 'px';
        drawGauge($(m.canvas), 0, '', m.color, false);
    });
    setupIoChartHover();

    // Linux 专用：设置窗口拖动
    if (window.pywebron?.isLinux) {
        const header = $('windowHeader');
        if (header) {
            header.addEventListener('mousedown', (e) => {
                // 排除按钮区域
                if (e.target.closest('.h-ctrls') || e.target.closest('.win-btn')) return;
                window.pywebron.startDrag(1);
            });
            // 双击最大化/还原
            header.addEventListener('dblclick', (e) => {
                if (e.target.closest('.h-ctrls') || e.target.closest('.win-btn')) return;
                windowAction('toggle');
            });
        }
    }

    await Promise.all([startMonitoring(), startChat()]);
    $('chatSendBtn')?.addEventListener('click', sendMsg);
    $('chatInput')?.addEventListener('keypress', e => e.key === 'Enter' && sendMsg());
    $('downloadBtn')?.addEventListener('click', downloadFile);
    $('cpuTaskBtn')?.addEventListener('click', runCpuTask);
    $('createWindowBtn')?.addEventListener('click', createNewWindow);
    $('diskTab')?.addEventListener('click', () => switchIoType('disk'));
    $('netTab')?.addEventListener('click', () => switchIoType('net'));
}

document.readyState === 'interactive' ? init() : document.addEventListener('DOMContentLoaded', init, {once: true});

// 窗口调整大小功能
(function () {
    const HT = {
        'top': 12, 'bottom': 15, 'left': 10, 'right': 11,
        'topleft': 13, 'topright': 14, 'bottomleft': 16, 'bottomright': 17
    };

    document.querySelectorAll('.resize-edge, .resize-corner').forEach(el => {
        el.addEventListener('mousedown', (e) => {
            e.preventDefault();
            e.stopPropagation();

            let edge = '';
            if (el.classList.contains('top')) edge = 'top';
            else if (el.classList.contains('bottom')) edge = 'bottom';
            else if (el.classList.contains('left')) edge = 'left';
            else if (el.classList.contains('right')) edge = 'right';
            else if (el.classList.contains('top-left')) edge = 'topleft';
            else if (el.classList.contains('top-right')) edge = 'topright';
            else if (el.classList.contains('bottom-left')) edge = 'bottomleft';
            else if (el.classList.contains('bottom-right')) edge = 'bottomright';

            if (!edge || !window.pywebron?.window_id) return;

            // 调用 Rust 开始调整大小
            window.pywebron.invoke('__rust_start_resize', {
                window_id: window.pywebron.window_id,
                hit_test: HT[edge]
            });
        });
    });
})();
