(function () {
    if (window.__pywebron_initialized) return;
    window.__pywebron_initialized = true;

    const pending = new Map(), streams = new Map();

    const interceptors = {
        response: [],
        error: []
    };

    const streamMessages = new Map();

    // 平台检测（通过 userAgent 判断）
    const isLinux = navigator.userAgent.toLowerCase().includes('linux') ||
        navigator.platform.toLowerCase().includes('linux');

    window.pywebron = {
        interceptors: {
            response: {
                use(fn) {
                    interceptors.response.push(fn);
                }
            },
            error: {
                use(fn) {
                    interceptors.error.push(fn);
                }
            }
        },
        // 平台标识
        isLinux: isLinux
    };

    if (existingWindowId !== undefined) {
        window.pywebron.window_id = existingWindowId;
    window.__pywebron_dispatch = function (msg) {
        const t = performance.now();
        const {handle_id, handle_type, request_id, payload} = msg;

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
                const elapsed = performance.now() - t;
                if (elapsed > 1) console.log(`[Timing][JS] __pywebron_dispatch(invoke) 处理耗时: ${elapsed.toFixed(2)}ms | handle=${handle_id}`);
            }
        } else if (handle_type === 'stream') {
            const s = streams.get(handle_id);
            if (s) {
                if (s.onData) {
                    handleStreamMessage(handle_id, payload);
                    s.onData(payload);
                    const elapsed = performance.now() - t;
                    console.log(`[Timing][JS] __pywebron_dispatch(stream) 处理耗时: ${elapsed.toFixed(2)}ms | handle=${handle_id}`);
                }
            }
        }
    };

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

    function handleResponse(response) {
        interceptors.response.forEach(fn => fn(response));
        return response;
    }

    function handleError(error) {
        interceptors.error.forEach(fn => fn(error));
        return error;
    }

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

    // 通过 wry IPC 发送消息到 Rust
    function ipcSend(message) {
        if (window.ipc && window.ipc.postMessage) {
            const t = performance.now();
            window.ipc.postMessage(JSON.stringify(message));
            const elapsed = performance.now() - t;
            if (elapsed > 0.5) console.log(`[Timing][JS] ipcSend 耗时: ${elapsed.toFixed(2)}ms | type=${message.handle_type}`);
        } else {
            console.error('[JS] window.ipc.postMessage 不可用');
        }
    }

    const savedWindowId = window.pywebron.window_id;
    Object.assign(window.pywebron, {
        window_id: savedWindowId || window.pywebron?.window_id,

        interceptors: {
            response: {
                use(fn) {
                    interceptors.response.push(fn);
                }
            },
            error: {
                use(fn) {
                    interceptors.error.push(fn);
                }
            }
        },
        generateRequestId(handleId) {
            const timestamp = Date.now();
            const random = Math.random().toString(36).slice(2, 8);
            return `${handleId}_${timestamp}_${random}`;
        },

        async invoke(handle, payload = {}, timeout = 6e4) {
            performance.now();
            const request_id = this.generateRequestId(handle);

            return new Promise((resolve, reject) => {
                pending.set(request_id, {resolve, reject});

                const message = {
                    window_id: this.window_id,
                    handle_id: handle,
                    handle_type: 'invoke',
                    request_id: request_id,
                    payload: payload
                };

                ipcSend(message);

                setTimeout(() => {
                    if (pending.delete(request_id)) {
                        console.log(`[JS] invoke 超时：${request_id}`);
                        reject(new Error('Timeout'))
                    }
                }, timeout);
            });
        },

        async stream(handle, payload = {}) {
            const t_start = performance.now();
            const hid = String(handle);
            const request_id = this.generateRequestId(hid);

            const self = this;
            const obj = {
                request_id,
                handle: hid,
                onData: null,
                onEnd: null,

                recv(cb) {
                    this.onData = cb;
                    return this
                },
                end(cb) {
                    this.onEnd = cb;
                    return this
                },
                close() {
                    streams.delete(this.handle)
                },
                send(data) {
                    const t = performance.now();
                    const message = {
                        window_id: self.window_id,
                        handle_id: hid,
                        handle_type: 'stream',
                        request_id: self.generateRequestId(hid),
                        payload: data
                    };

                    ipcSend(message);
                    const elapsed = performance.now() - t;
                    console.log(`[Timing][JS] stream.send 耗时: ${elapsed.toFixed(2)}ms | handle=${hid}`);
                    return this
                }
            };

            streams.set(hid, obj);

            const startMessage = {
                window_id: this.window_id,
                handle_id: handle,
                handle_type: 'stream',
                request_id: request_id,
                payload: payload
            };

            ipcSend(startMessage);
            console.log(`[Timing][Frontend] stream() 连接建立耗时: ${(performance.now() - t_start).toFixed(2)}ms | handle=${hid}`);

            return obj;
        },

        // Linux专用：启动窗口拖动（通过invoke调用后端）
        startDrag(button = 1) {
            if (!this.isLinux) return;
            this.invoke('__rust_start_drag_window', {
                window_id: this.window_id,
                button: button
            }).catch(e => console.warn('[Drag] Failed:', e.message));
        }

    // 注入 resize-area 样式和元素
    });
