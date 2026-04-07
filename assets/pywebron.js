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
        enable_resizable,
        enable_devtools
    } = window.pywebron || {};

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

    function setupWindowResizeHandles() {
        if (window.pywebron && window.pywebron.attributes.show_title_bar === true) {
            const resizeArea = document.getElementById('resize-area');
            if (resizeArea) {
                resizeArea.style.display = 'none';
            }
            return;
        }

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
