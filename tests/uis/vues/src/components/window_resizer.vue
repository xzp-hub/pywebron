<script>
export default {
  name: 'window_resizer'
}
</script>

<template>
  <div id="resize-area" class="window-resize-overlay">
    <div class="window-resize-corner-top-left"></div>
    <div class="window-resize-corner-top-right"></div>
    <div class="window-resize-corner-bottom-left"></div>
    <div class="window-resize-corner-bottom-right"></div>
    <div class="window-resize-edge-top"></div>
    <div class="window-resize-edge-bottom"></div>
    <div class="window-resize-edge-left"></div>
    <div class="window-resize-edge-right"></div>
  </div>
</template>

<style scoped>
.window-resize-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  z-index: 9999;
}

.window-resize-corner-top-left,
.window-resize-corner-top-right,
.window-resize-corner-bottom-left,
.window-resize-corner-bottom-right,
.window-resize-edge-top,
.window-resize-edge-bottom,
.window-resize-edge-left,
.window-resize-edge-right {
  position: absolute;
  pointer-events: auto;
}

.window-resize-corner-top-left { top: 0; left: 0; width: 10px; height: 10px; cursor: nw-resize; }
.window-resize-corner-top-right { top: 0; right: 0; width: 10px; height: 10px; cursor: ne-resize; }
.window-resize-corner-bottom-left { bottom: 0; left: 0; width: 10px; height: 10px; cursor: sw-resize; }
.window-resize-corner-bottom-right { bottom: 0; right: 0; width: 10px; height: 10px; cursor: se-resize; }

.window-resize-edge-top { top: 0; left: 10px; right: 10px; height: 5px; cursor: n-resize; }
.window-resize-edge-bottom { bottom: 0; left: 10px; right: 10px; height: 5px; cursor: s-resize; }
.window-resize-edge-left { top: 10px; left: 0; width: 5px; bottom: 10px; cursor: w-resize; }
.window-resize-edge-right { top: 10px; right: 0; width: 5px; bottom: 10px; cursor: e-resize; }
</style>
