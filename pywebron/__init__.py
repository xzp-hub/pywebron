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
from typing import Dict
from time import perf_counter


class App:
    def __init__(self, prewarm_webview: bool = False):
        t_start = perf_counter()
        print(f"[Performance] App.__init__ 开始")
        
        self.window = Window
        self.invoke = Invoke
        self.stream = Stream
        self.worker = Worker
        
        t_before_init = perf_counter()
        rust_init(prewarm_webview)
        t_after_init = perf_counter()
        
        print(f"[Performance] rust_init 耗时: {(t_after_init - t_before_init) * 1000:.2f}ms")
        print(f"[Performance] App.__init__ 总耗时: {(t_after_init - t_start) * 1000:.2f}ms")

    def run(self):
        t_start = perf_counter()
        print(f"[Performance] App.run 开始，准备启动事件循环")
        rust_run()  # 直接调用，rust_run 是同步函数
        t_end = perf_counter()
        print(f"[Performance] App.run 结束，总耗时: {(t_end - t_start) * 1000:.2f}ms")

    @staticmethod
    def get_windows() -> Dict[int, Dict]:
        return rust_get_windows()

    @staticmethod
    def get_handles() -> Dict[str, Dict[str, str]]:
        return rust_get_handles()


__all__ = ("App", "StreamSendModes")
