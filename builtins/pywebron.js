(function () {
    if (window.__pywebron_initialized) return;
    window.__pywebron_initialized = true;

    const pending = new Map(), streams = new Map();
    const interceptors = { response: [], error: [] };
    const streamMessages = new Map();
    let _reqCounter = 0;

    const config = window.pywebron || {};
    const {
        window_id,
        show_title_bar,
        window_radius,
        enable_resizable,
        toast_success = false,
        toast_error = true
    } = config;

    if (!show_title_bar) {
        const radiusStyle = `html,body{border-radius:${window_radius}px!important;overflow:hidden!important;background:transparent!important;margin:0!important;padding:0!important;width:100%!important;height:100%!important;box-sizing:border-box!important}html{border:1px solid transparent!important}body{border:1px solid var(--border-color)!important}#app,#app>[id='app'],.app,main{border-radius:${window_radius}px!important}`;

        const injectRadiusStyle = () => {
            if (document.getElementById('__pywebron_radius_style__')) return true;
            if (!document.head) return false;
            const style = document.createElement('style');
            style.id = '__pywebron_radius_style__';
            style.textContent = radiusStyle;
            document.head.insertBefore(style, document.head.firstChild);
            return true;
        };

        if (!injectRadiusStyle()) {
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', injectRadiusStyle, { once: true });
            } else {
                let retries = 0;
                const retry = setInterval(() => {
                    if (injectRadiusStyle() || ++retries > 10) clearInterval(retry);
                }, 50);
            }
        }
    }

    function generateRequestId(handleId) {
        return `${handleId}_${++_reqCounter}`;
    }

    function ipcSend(message) {
        window.ipc?.postMessage(JSON.stringify(message));
    }

    function handleResponse(response) {
        for (let i = 0; i < interceptors.response.length; i++) interceptors.response[i](response);
        return response;
    }

    function handleError(error) {
        for (let i = 0; i < interceptors.error.length; i++) interceptors.error[i](error);
        return error;
    }

    const MESSAGE_COLORS = { success: '#00B42A', error: '#F59E0B' };
    let messageStyleInjected = false;

    function injectMessageStyle() {
        if (messageStyleInjected) return;
        messageStyleInjected = true;
        if (document.head) {
            const style = document.createElement('style');
            style.textContent = '@keyframes slideIn{from{transform:translateY(-100%);opacity:0}to{transform:translateY(0);opacity:1}}';
            document.head.appendChild(style);
        }
    }

    function showMessage(mssg, stat = true, streamId = null) {
        if (streamId) {
            if (streamMessages.get(streamId) === mssg) return;
            streamMessages.set(streamId, mssg);
        }

        if (!document.body) {
            setTimeout(() => showMessage(mssg, stat, streamId), 100);
            return;
        }

        injectMessageStyle();

        let container = document.getElementById('pywebron-message-container');
        if (!container) {
            container = document.createElement('div');
            container.id = 'pywebron-message-container';
            container.style.cssText = 'position:fixed;top:20px;left:50%;transform:translateX(-50%);z-index:9999;display:flex;flex-direction:column;gap:10px;max-width:80%';
            document.body.appendChild(container);
        }

        const messageEl = document.createElement('div');
        const color = stat ? MESSAGE_COLORS.success : MESSAGE_COLORS.error;

        messageEl.style.cssText = `padding:12px 24px;background:${color};color:white;border-radius:4px;box-shadow:0 2px 12px rgba(0,0,0,0.15);animation:slideIn .3s ease;cursor:pointer;transition:opacity .3s`;
        messageEl.textContent = mssg;
        messageEl.onclick = () => removeMessage(messageEl);

        container.appendChild(messageEl);
        setTimeout(() => removeMessage(messageEl), 3000);
    }

    function removeMessage(el) {
        el.style.opacity = '0';
        setTimeout(() => el.remove(), 300);
    }

    interceptors.response.push((response) => {
        if (response && response.stat === false) {
            if (toast_error && response.mssg) showMessage(response.mssg, false);
            if (response.data) console.error(response.data);
        } else if (toast_success && response && response.stat === true && response.mssg) {
            showMessage(response.mssg, true);
        }
    });

    window.__pywebron_dispatch = function (msg) {
        const { handle_id, handle_type, request_id, payload } = msg;

        if (handle_type === 'invoke') {
            const h = pending.get(request_id);
            if (h) {
                pending.delete(request_id);
                if (h.timerId) clearTimeout(h.timerId);
                if (payload && payload.stat === false) {
                    handleError(new Error(payload.mssg || payload.data));
                    h.reject(new Error(payload.mssg || payload.data));
                } else {
                    handleResponse(payload);
                    h.resolve(payload);
                }
            }
        } else if (handle_type === 'stream') {
            const s = streams.get(handle_id);
            if (s && s.onData) {
                if (payload && payload.__history_batch__ && Array.isArray(payload.messages)) {
                    for (let i = 0; i < payload.messages.length; i++) s.onData(payload.messages[i]);
                    return;
                }
                if (payload && payload.stat !== undefined && payload.mssg) {
                    if ((payload.stat && toast_success) || (payload.stat === false && toast_error)) {
                    showMessage(payload.mssg, payload.stat, handle_id);
                    }
                }
                s.onData(payload);
            }
        }
    };

    window.pywebron = {
        attributes: { ...config },

        interfaces: {
            resolveAssetUrl(filePath) {
                if (!filePath || typeof filePath !== 'string') return '';
                const normalized = filePath.replace(/\\/g, '/');
                // 保留完整路径，让协议处理器在 dist 目录找不到时回退到绝对路径
                return 'http://app.' + normalized;
            },

            async invoke(handle, payload = {}, timeout = 6e4) {
                const request_id = generateRequestId(handle);

                return new Promise((resolve, reject) => {
                    // 防止 pending Map 无限增长
                    if (pending.size > 1000) {
                        const oldest = pending.keys().next().value;
                        const dropped = oldest !== undefined ? pending.get(oldest) : null;
                        if (oldest !== undefined) {
                            pending.delete(oldest);
                            if (dropped?.timerId) clearTimeout(dropped.timerId);
                        }
                        dropped?.reject?.(new Error('Pending queue overflow'));
                    }
                    const timerId = setTimeout(() => {
                        if (pending.delete(request_id)) reject(new Error('Timeout'));
                    }, timeout);
                    pending.set(request_id, { resolve, reject, timerId });

                    ipcSend({
                        window_id,
                        handle_id: handle,
                        handle_type: 'invoke',
                        request_id,
                        payload
                    });
                });
            },

            windows: {
                minimize: () => window.pywebron.interfaces.invoke('__rust_window_minimize'),
                maximize: () => window.pywebron.interfaces.invoke('__rust_window_maximize'),
                reappear: () => window.pywebron.interfaces.invoke('__rust_window_reappear'),
                shutdown: () => window.pywebron.interfaces.invoke('__rust_window_shutdown'),
                dragdrop: (selector = '.header') => window.pywebron.interfaces.invoke('__rust_window_dragdrop', { selector }),
            },

            async stream(handle, payload = {}) {
                const hid = String(handle);
                const request_id = generateRequestId(hid);

                const obj = {
                    request_id,
                    handle: hid,
                    onData: null,

                    recv(cb) {
                        this.onData = cb;
                        return this;
                    },
                    close() {
                        ipcSend({
                            window_id,
                            handle_id: hid,
                            handle_type: 'stream_close',
                            request_id: generateRequestId(hid),
                            payload: null
                        });
                        streams.delete(this.handle);
                        streamMessages.delete(this.handle);
                    },
                    send(data) {
                        ipcSend({
                            window_id,
                            handle_id: hid,
                            handle_type: 'stream',
                            request_id: generateRequestId(hid),
                            payload: data
                        });
                        return this;
                    }
                };

                streams.get(hid)?.close();
                streams.set(hid, obj);

                ipcSend({
                    window_id,
                    handle_id: handle,
                    handle_type: 'stream',
                    request_id,
                    payload
                });

                return obj;
            }
        }
    };

    function ensureResizeLayer() {
        if (show_title_bar || enable_resizable === false) {
            const existing = document.getElementById('resize-area');
            if (existing) existing.style.display = 'none';
            return null;
        }

        if (!document.body) return null;

        if (!document.getElementById('pywebron-resize-style')) {
            const style = document.createElement('style');
            style.id = 'pywebron-resize-style';
            style.textContent = `.resize-area{position:fixed;inset:0;pointer-events:none;z-index:9999}.resize-corner,.resize-edge{position:absolute;pointer-events:auto}.resize-corner{width:10px;height:10px}.resize-corner.top-left{top:0;left:0;cursor:nw-resize}.resize-corner.top-right{top:0;right:0;cursor:ne-resize}.resize-corner.bottom-left{bottom:0;left:0;cursor:sw-resize}.resize-corner.bottom-right{bottom:0;right:0;cursor:se-resize}.resize-edge.top,.resize-edge.bottom{left:10px;right:10px;height:5px}.resize-edge.top{top:0;cursor:n-resize}.resize-edge.bottom{bottom:0;cursor:s-resize}.resize-edge.left,.resize-edge.right{top:10px;bottom:10px;width:5px}.resize-edge.left{left:0;cursor:w-resize}.resize-edge.right{right:0;cursor:e-resize}`;
            document.head?.appendChild(style);
        }

        let area = document.getElementById('resize-area');
        if (!area) {
            area = document.createElement('div');
            area.id = 'resize-area';
            area.className = 'resize-area';
            area.innerHTML = '<div class="resize-corner top-left" data-edge="topleft"></div><div class="resize-corner top-right" data-edge="topright"></div><div class="resize-corner bottom-left" data-edge="bottomleft"></div><div class="resize-corner bottom-right" data-edge="bottomright"></div><div class="resize-edge top" data-edge="top"></div><div class="resize-edge bottom" data-edge="bottom"></div><div class="resize-edge left" data-edge="left"></div><div class="resize-edge right" data-edge="right"></div>';
            document.body.appendChild(area);
        }

        return area;
    }

    function setupWindowResizeHandles() {
        const area = ensureResizeLayer();
        if (!area) return;

        const HIT_TEST = {
            top: 12,
            bottom: 15,
            left: 10,
            right: 11,
            topleft: 13,
            topright: 14,
            bottomleft: 16,
            bottomright: 17
        };

        area.addEventListener('mousedown', (e) => {
            const target = e.target.closest('[data-edge]');
            if (!target) return;
            e.preventDefault();
            e.stopPropagation();

            const edge = target.dataset.edge;
            if (edge && HIT_TEST[edge] !== undefined) {
                ipcSend({
                    window_id,
                    handle_id: '__rust_start_resize',
                    handle_type: 'invoke',
                    request_id: generateRequestId('__rust_start_resize'),
                    payload: { window_id, hit_test: HIT_TEST[edge] }
                });
            }
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', setupWindowResizeHandles, { once: true });
    } else {
        setupWindowResizeHandles();
    }
})();
