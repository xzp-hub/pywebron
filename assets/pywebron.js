(function () {
    if (window.__pywebron_initialized) {
        return;
    }
    window.__pywebron_initialized = true;

    const pending = new Map(), streams = new Map();

    const interceptors = {
        response: [],
        error: []
    };

    const streamMessages = new Map();

    const {
        window_id,
        title,
        width,
        height,
        show_title_bar,
        window_radius,
        enable_resizable,
        enable_devtools
    } = window.pywebron || {};

    // 调试信息
    console.log('[PyWebron] 配置信息:', {
        show_title_bar,
        window_radius,
        shouldInject: !show_title_bar && window_radius && window_radius > 0
    });

    // 注入圆角 CSS（当 show_title_bar 为 false 且 window_radius > 0 时）
    if (!show_title_bar && window_radius && window_radius > 0) {
        console.log('[PyWebron] 条件满足，准备注入圆角样式');

        const injectRadiusStyle = () => {
            console.log('[PyWebron] injectRadiusStyle 被调用');

            if (document.getElementById('__pywebron_radius_style__')) {
                console.log('[PyWebron] 样式已存在，跳过');
                return true; // 已存在
            }

            if (!document.head) {
                console.log('[PyWebron] document.head 不存在，稍后重试');
                return false; // head 还不存在
            }

            console.log('[PyWebron] 开始创建样式元素');
            const style = document.createElement('style');
            style.id = '__pywebron_radius_style__';
            style.textContent = `
                html, body {
                    border-radius: ${window_radius}px !important;
                    color-scheme: light dark !important;
                    overflow: hidden !important;
                    background: transparent !important;
                    margin: 0 !important;
                    padding: 0 !important;
                    width: 100% !important;
                    height: 100% !important;
                    box-sizing: border-box !important;
                }
                #app {
                    border-radius: ${window_radius}px !important;
                }
            `;

            // 插入到最前面，确保优先级
            document.head.insertBefore(style, document.head.firstChild);
            console.log('[PyWebron] 圆角样式注入成功:', window_radius + 'px');
            return true;
        };

        console.log('[PyWebron] 立即尝试注入');
        // 立即尝试注入
        if (!injectRadiusStyle()) {
            console.log('[PyWebron] 首次注入失败，设置重试机制');
            // 如果失败，等待 DOM 加载
            if (document.readyState === 'loading') {
                console.log('[PyWebron] DOM 正在加载，监听 DOMContentLoaded');
                document.addEventListener('DOMContentLoaded', injectRadiusStyle, { once: true });
            } else {
                console.log('[PyWebron] DOM 已加载，使用定时器重试');
                // DOM 已加载，但 head 不存在？重试几次
                let retries = 0;
                const retry = setInterval(() => {
                    if (injectRadiusStyle() || retries++ > 10) {
                        clearInterval(retry);
                    }
                }, 50);
            }
        }

        // 监听 head 的变化，防止样式被移除
        if (typeof MutationObserver !== 'undefined') {
            const observer = new MutationObserver(() => {
                injectRadiusStyle();
            });

            // 等待 head 存在后开始监听
            const startObserving = () => {
                if (document.head) {
                    observer.observe(document.head, { childList: true });
                } else {
                    setTimeout(startObserving, 50);
                }
            };
            startObserving();
        }
    } else {
        console.log('[PyWebron] 条件不满足，不注入圆角样式', {
            show_title_bar,
            window_radius,
            condition1: !show_title_bar,
            condition2: window_radius && window_radius > 0
        });
    }

    function generateRequestId(handleId) {
        const timestamp = Date.now();
        const random = Math.random().toString(36).slice(2, 8);
        return `${handleId}_${timestamp}_${random}`;
    }

    function ipcSend(message) {
        if (window.ipc && window.ipc.postMessage) {
            window.ipc.postMessage(JSON.stringify(message));
        }
    }

    function handleResponse(response) {
        interceptors.response.forEach(fn => fn(response));
        return response;
    }

    function handleError(error) {
        interceptors.error.forEach(fn => fn(error));
        return error;
    }

    function showMessage(mssg, type = 200, streamId = null) {
        if (streamId) {
            const lastMssg = streamMessages.get(streamId);
            if (lastMssg === mssg) return;
            streamMessages.set(streamId, mssg);
        }

        if (!document.body) {
            setTimeout(() => showMessage(mssg, type, streamId), 100);
            return;
        }

        let container = document.getElementById('pywebron-message-container');
        if (!container) {
            container = document.createElement('div');
            container.id = 'pywebron-message-container';
            container.style.cssText = `
                position: fixed;
                top: 20px;
                left: 50%;
                transform: translateX(-50%);
                z-index: 9999;
                display: flex;
                flex-direction: column;
                gap: 10px;
                max-width: 80%;
            `;
            document.body.appendChild(container);
        }

        const messageEl = document.createElement('div');

        let color;
        if (type >= 100 && type < 200) {
            color = '#165DFF';
        } else if (type >= 200 && type < 300) {
            color = '#00B42A';
        } else if (type >= 300 && type < 400) {
            color = '#165DFF';
        } else if (type >= 400 && type < 500) {
            color = '#F7BA1E';
        } else if (type >= 500) {
            color = '#F53F3F';
        } else {
            color = '#00B42A';
        }

        messageEl.style.cssText = `
            padding: 12px 24px;
            background: ${color};
            color: white;
            border-radius: 4px;
            box-shadow: 0 2px 12px rgba(0,0,0,0.15);
            animation: slideIn 0.3s ease;
            cursor: pointer;
            transition: opacity 0.3s;
        `;
        messageEl.textContent = mssg;

        messageEl.onclick = () => removeMessage(messageEl);

        container.appendChild(messageEl);

        setTimeout(() => removeMessage(messageEl), 3000);
    }

    function removeMessage(el) {
        el.style.opacity = '0';
        setTimeout(() => el.remove(), 300);
    }

    setTimeout(() => {
        if (document.head) {
            const style = document.createElement('style');
            style.textContent = `
                @keyframes slideIn {
                    from {
                        transform: translateY(-100%);
                        opacity: 0;
                    }
                    to {
                        transform: translateY(0);
                        opacity: 1;
                    }
                }
            `;
            document.head.appendChild(style);
        }
    }, 0);

    interceptors.response.push((response) => {
        if (response && response.code !== undefined && response.mssg) {
            if (response.code >= 400) {
                showMessage(response.mssg, response.code);
            }
        }
    });

    function handleStreamMessage(streamId, data) {
        if (data && data.code !== undefined && data.mssg) {
            showMessage(data.mssg, data.code, streamId);
        }
    }

    window.__pywebron_dispatch = function (msg) {
        const { handle_id, handle_type, request_id, payload } = msg;

        if (handle_type === 'invoke') {
            const h = pending.get(request_id);
            if (h) {
                if (payload && payload.error) {
                    handleError(new Error(payload.error));
                    h.reject(new Error(payload.error));
                } else {
                    handleResponse(payload);
                    h.resolve(payload);
                }
                pending.delete(request_id);
            }
        } else if (handle_type === 'stream') {
            const s = streams.get(handle_id);
            if (s) {
                if (s.onData) {
                    handleStreamMessage(handle_id, payload);
                    s.onData(payload);
                }
            }
        }
    };

    window.pywebron = {
        attributes: {
            window_id,
            title,
            width,
            height,
            show_title_bar,
            enable_resizable,
            enable_devtools
        },

        interfaces: {
            async invoke(handle, payload = {}, timeout = 6e4) {
                const pywebron = window.pywebron;
                const request_id = generateRequestId(handle);

                return new Promise((resolve, reject) => {
                    pending.set(request_id, { resolve, reject });

                    const message = {
                        window_id: pywebron.attributes.window_id,
                        handle_id: handle,
                        handle_type: 'invoke',
                        request_id: request_id,
                        payload: payload
                    };

                    ipcSend(message);

                    setTimeout(() => {
                        if (pending.delete(request_id)) {
                            reject(new Error('Timeout'));
                        }
                    }, timeout);
                });
            },

            async stream(handle, payload = {}) {
                const pywebron = window.pywebron;
                const hid = String(handle);
                const request_id = generateRequestId(hid);

                const obj = {
                    request_id,
                    handle: hid,
                    onData: null,
                    onEnd: null,

                    recv(cb) {
                        this.onData = cb;
                        return this;
                    },
                    end(cb) {
                        this.onEnd = cb;
                        return this;
                    },
                    close() {
                        streams.delete(this.handle);
                    },
                    send(data) {
                        const message = {
                            window_id: pywebron.attributes.window_id,
                            handle_id: hid,
                            handle_type: 'stream',
                            request_id: generateRequestId(hid),
                            payload: data
                        };

                        ipcSend(message);
                        return this;
                    }
                };

                streams.set(hid, obj);

                const startMessage = {
                    window_id: pywebron.attributes.window_id,
                    handle_id: handle,
                    handle_type: 'stream',
                    request_id: request_id,
                    payload: payload
                };

                ipcSend(startMessage);

                return obj;
            }
        }
    };

    function ensureResizeLayer() {
        if (window.pywebron && window.pywebron.attributes.show_title_bar === true) {
            const existing = document.getElementById('resize-area');
            if (existing) existing.style.display = 'none';
            return null;
        }

        // enable_resizable 为 false 时不创建调整大小层，避免出现调整大小指针
        if (enable_resizable === false) {
            const existing = document.getElementById('resize-area');
            if (existing) existing.style.display = 'none';
            return null;
        }

        if (!document.body) return null;

        if (!document.getElementById('pywebron-resize-style')) {
            const style = document.createElement('style');
            style.id = 'pywebron-resize-style';
            style.textContent = `
                .resize-area {
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    pointer-events: none;
                    z-index: 9999;
                }
                .resize-corner,
                .resize-edge {
                    position: absolute;
                    pointer-events: auto;
                }
                .resize-corner.top-left {
                    top: 0;
                    left: 0;
                    width: 10px;
                    height: 10px;
                    cursor: nw-resize;
                }
                .resize-corner.top-right {
                    top: 0;
                    right: 0;
                    width: 10px;
                    height: 10px;
                    cursor: ne-resize;
                }
                .resize-corner.bottom-left {
                    bottom: 0;
                    left: 0;
                    width: 10px;
                    height: 10px;
                    cursor: sw-resize;
                }
                .resize-corner.bottom-right {
                    bottom: 0;
                    right: 0;
                    width: 10px;
                    height: 10px;
                    cursor: se-resize;
                }
                .resize-edge.top {
                    top: 0;
                    left: 10px;
                    right: 10px;
                    height: 5px;
                    cursor: n-resize;
                }
                .resize-edge.bottom {
                    bottom: 0;
                    left: 10px;
                    right: 10px;
                    height: 5px;
                    cursor: s-resize;
                }
                .resize-edge.left {
                    top: 10px;
                    bottom: 10px;
                    left: 0;
                    width: 5px;
                    cursor: w-resize;
                }
                .resize-edge.right {
                    top: 10px;
                    bottom: 10px;
                    right: 0;
                    width: 5px;
                    cursor: e-resize;
                }
            `;
            document.head?.appendChild(style);
        }

        let area = document.getElementById('resize-area');
        if (!area) {
            area = document.createElement('div');
            area.id = 'resize-area';
            area.className = 'resize-area';
            area.innerHTML = `
                <div class="resize-corner top-left"></div>
                <div class="resize-corner top-right"></div>
                <div class="resize-corner bottom-left"></div>
                <div class="resize-corner bottom-right"></div>
                <div class="resize-edge top"></div>
                <div class="resize-edge bottom"></div>
                <div class="resize-edge left"></div>
                <div class="resize-edge right"></div>
            `;
            document.body.appendChild(area);
        }

        return area;
    }

    function setupWindowResizeHandles() {
        const area = ensureResizeLayer();
        if (!area) return;

        const HT = {
            'top': 12, 'bottom': 15, 'left': 10, 'right': 11,
            'topleft': 13, 'topright': 14, 'bottomleft': 16, 'bottomright': 17
        };

        area.querySelectorAll('.resize-edge, .resize-corner').forEach(el => {
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

                const pywebron = window.pywebron;
                if (!edge || !pywebron?.attributes?.window_id || !pywebron?.interfaces?.invoke) {
                    return;
                }

                pywebron.interfaces.invoke('__rust_start_resize', {
                    window_id: pywebron.attributes.window_id,
                    hit_test: HT[edge]
                });
            });
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', setupWindowResizeHandles, { once: true });
    } else {
        setupWindowResizeHandles();
    }
})();
