(function () {
    console.log('[Resize JS] ========== resize 脚本开始执行 ==========');
    console.log('[Resize JS] window.pywebron:', window.pywebron);

    if (window.pywebron && window.pywebron.hasSystemTitleBar === true) {
        console.log('[Resize JS] 检测到系统标题栏，隐藏 resize-area');
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

    console.log('[Resize JS] 设置 resize 事件监听');
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

            console.log('[Resize JS] mousedown on edge:', edge);
            if (!edge || !window.pywebron?.window_id || !window.pywebron?.invoke) {
                console.log('[Resize JS] 条件不满足，跳过');
                return;
            }

            console.log('[Resize JS] 调用 __rust_start_resize, window_id:', window.pywebron.window_id, 'hit_test:', HT[edge]);
            window.pywebron.invoke('__rust_start_resize', {
                window_id: window.pywebron.window_id,
                hit_test: HT[edge]
            });
        });
    });

    console.log('[Resize JS] ========== resize 脚本执行完毕 ==========');
})();
