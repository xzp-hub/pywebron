from .configs import StreamSendModes
from ._pywebron_ import rust_init, rust_run, rust_get_windows, rust_get_handles
from .app.window import Window
from .app.handler import Invoke, Stream
from .app.router import Router
from .app.worker import Worker
from typing import Dict


class App:
    def __init__(self, prewarm_webview: bool = False):
        rust_init(prewarm_webview)
        self._app_router = Router(title="app")
        self.invoke = self._app_router.invoke
        self.stream = self._app_router.stream
        self.window = Window
        self.worker = Worker
        self.router = Router

    def run(self):
        if self._app_router.handlers:
            Router.register_routers(self._app_router)
        rust_run()

    @staticmethod
    def get_windows() -> Dict[int, Dict]:
        return rust_get_windows()

    @staticmethod
    def get_handles() -> Dict[str, Dict[str, str]]:
        return rust_get_handles()

    @staticmethod
    def get_all_handlers():
        from .configs import HANDLES
        return HANDLES


__all__ = ("App", "Window", "Invoke", "Stream", "Worker", "Router", "StreamSendModes")
