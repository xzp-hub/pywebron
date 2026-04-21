from .configs import StreamSendModes
from ._pywebron_ import (
    rust_init,
    rust_run,
    rust_get_windows,
    rust_get_handles,
)
from .app.window import Window
from .app.handler import Invoke, Stream, Router
from .app.worker import Worker
from typing import Dict
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
        
        # 单文件应用：使用内部 Router
        self._app_router = Router(title="app")
        self.invoke = self._app_router.invoke
        self.stream = self._app_router.stream
        
        # 其他属性
        self.window = Window
        self.worker = Worker
        self.router = Router

    def run(self):
        t_start = perf_counter()
        print(f"[Performance] App.run 开始，准备启动事件循环")
        
        # 自动注册单文件应用的处理器
        if self._app_router.handlers:
            Router.register_routers(self._app_router)
        
        rust_run()
        t_end = perf_counter()
        print(f"[Performance] App.run 结束，总耗时: {(t_end - t_start) * 1000:.2f}ms")

    @staticmethod
    def get_windows() -> Dict[int, Dict]:
        return rust_get_windows()

    @staticmethod
    def get_handles() -> Dict[str, Dict[str, str]]:
        return rust_get_handles()
    
    @staticmethod
    def get_all_handlers():
        """获取所有已注册的处理器（按路由分组）"""
        from .configs import HANDLES
        return HANDLES


__all__ = ("App", "Window", "Invoke", "Stream", "Worker", "Router", "StreamSendModes")
