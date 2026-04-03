(function() {
    if (window.__pywebron_resize_initialized) return;
    window.__pywebron_resize_initialized = true;

    if (window.pywebron && window.pywebron.hasSystemTitleBar === true) {
        return;
    }

    var HT = {
        'top': 12, 'bottom': 15, 'left': 10, 'right': 11,
        'topleft': 13, 'topright': 14, 'bottomleft': 16, 'bottomright': 17
    };

    function createResizeArea() {
        if (document.getElementById('pywebron-resize-area')) return;

        var resizeArea = document.createElement('div');
        resizeArea.id = 'pywebron-resize-area';
        resizeArea.style.cssText = 'position:absolute;top:0;left:0;right:0;bottom:0;pointer-events:none;z-index:9999;';

        var edges = [
            {name: 'top', style: 'top:0;left:8px;right:8px;height:8px;cursor:n-resize;'},
            {name: 'bottom', style: 'bottom:0;left:8px;right:8px;height:8px;cursor:s-resize;'},
            {name: 'left', style: 'left:0;top:8px;bottom:8px;width:8px;cursor:w-resize;'},
            {name: 'right', style: 'right:0;top:8px;bottom:8px;width:8px;cursor:e-resize;'}
        ];

        var corners = [
            {name: 'topleft', style: 'top:0;left:0;width:8px;height:8px;cursor:nw-resize;'},
            {name: 'topright', style: 'top:0;right:0;width:8px;height:8px;cursor:ne-resize;'},
            {name: 'bottomleft', style: 'bottom:0;left:0;width:8px;height:8px;cursor:sw-resize;'},
            {name: 'bottomright', style: 'bottom:0;right:0;width:8px;height:8px;cursor:se-resize;'}
        ];

        edges.forEach(function(edge) {
            var el = document.createElement('div');
            el.className = 'pywebron-resize-edge';
            el.dataset.edge = edge.name;
            el.style.cssText = 'position:absolute;pointer-events:auto;background:transparent;' + edge.style;
            resizeArea.appendChild(el);
        });

        corners.forEach(function(corner) {
            var el = document.createElement('div');
            el.className = 'pywebron-resize-corner';
            el.dataset.edge = corner.name;
            el.style.cssText = 'position:absolute;pointer-events:auto;background:transparent;' + corner.style;
            resizeArea.appendChild(el);
        });

        document.body.appendChild(resizeArea);

        resizeArea.addEventListener('mousedown', function(e) {
            var target = e.target;
            if (!target.classList.contains('pywebron-resize-edge') && 
                !target.classList.contains('pywebron-resize-corner')) return;

            e.preventDefault();
            e.stopPropagation();

            var edge = target.dataset.edge;
            if (!edge || !window.pywebron?.window_id) return;

            if (window.pywebron.invoke) {
                window.pywebron.invoke('__rust_start_resize', {
                    window_id: window.pywebron.window_id,
                    hit_test: HT[edge]
                }).catch(function(err) {
                    console.warn('[PyWebron Resize] Failed:', err.message);
                });
            }
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', createResizeArea);
    } else {
        createResizeArea();
    }

    setTimeout(createResizeArea, 100);
})();
