from .configs import StreamSendModes
from ._pywebron_ import (
    rust_init,
    rust_run,
    rust_get_windows,
    rust_get_handles,
)
from .app.window import Window
from .app.invoke import Invoke
from .app.stream import Stream
from .app.worker import Worker
from .app.router import Router
from typing import Dict, List
from time import perf_counter


class App:
    def __init__(self, prewarm_webview: bool = False):
        t_start = perf_counter()
        print(f"[Performance] App.__init__ 开始")
        
        t_before_init = perf_counter()
        rust_init(prewarm_webview)
        t_after_init = perf_counter()
        
        print(f"[Performance] rust_init 耗时: {(t_after_init - t_before_init) * 1000:.2f}ms")
        print(f"[Performance] App.__init__ 总耗时: {(t_after_init - t_start) * 1000:.2f}ms")
        
        self._routers: List[Router] = []

    def include_router(self, *routers: Router):
        """注册一个或多个路由器"""
        for router in routers:
            self._routers.append(router)
            # 注册路由器中的所有 invoke 和 stream 处理器
            for name, handler in router._pending_invoke:
                print(f"[Router] 注册 Invoke 处理器: {name}")
            for name, handler in router._pending_stream:
                print(f"[Router] 注册 Stream 处理器: {name}")

    def include_window(self, *window_ids):
        """注册一个或多个窗口"""
        for wid in window_ids:
            print(f"[Window] 注册窗口 ID: {wid}")

    def register_window(self, **kwargs) -> bool:
        """注册窗口，参数同 Window.register_window"""
        return Window.register_window(**kwargs)

    def run(self):
        t_start = perf_counter()
        print(f"[Performance] App.run 开始，准备启动事件循环")
        rust_run()
        t_end = perf_counter()
        print(f"[Performance] App.run 结束，总耗时: {(t_end - t_start) * 1000:.2f}ms")

    @staticmethod
    def get_windows() -> Dict[int, Dict]:
        return rust_get_windows()

    @staticmethod
    def get_handles() -> Dict[str, Dict[str, str]]:
        return rust_get_handles()


__all__ = ("App", "Window", "Invoke", "Stream", "Worker", "Router", "StreamSendModes")
