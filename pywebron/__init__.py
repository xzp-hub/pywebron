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


class App:
    def __init__(self):
        self.window = Window
        self.invoke = Invoke
        self.stream = Stream
        self.worker = Worker
        rust_init()

    def run(self):
        rust_run()  # 直接调用，rust_run 是同步函数

    @staticmethod
    def get_windows() -> Dict[int, Dict]:
        return rust_get_windows()

    @staticmethod
    def get_handles() -> Dict[str, Dict[str, str]]:
        return rust_get_handles()


__all__ = ("App", "StreamSendModes")
