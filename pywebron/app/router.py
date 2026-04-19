from typing import Callable
from .invoke import Invoke
from .stream import Stream


class Router:
    """类似 FastAPI 的 APIRouter，支持分组注册处理器。

    用法:
        router = Router()

        @router.invoke("window_controls")
        async def window_controls(invoke: invoke, struct: WindowControlsStruct):
            ...

        @router.stream()
        async def system_monitoring(stream: app.stream):
            ...

        # 在主应用中注册
        app.include_router(router)
    """

    def __init__(self):
        self._pending_invoke: list[tuple[str, Callable]] = []
        self._pending_stream: list[tuple[str, Callable]] = []

    def invoke(self, alias: str = None):
        """注册 Invoke 处理器到路由组。alias 为空时使用函数名。"""
        def decorator(func):
            name = alias or func.__name__
            wrapper = Invoke._handler_(func, name)
            self._pending_invoke.append((name, wrapper))
            return wrapper
        return decorator

    def stream(self, alias: str = None):
        """注册 Stream 处理器到路由组。alias 为空时使用函数名。"""
        def decorator(func):
            name = alias or func.__name__
            wrapper = Stream._handler_(func, name)
            self._pending_stream.append((name, wrapper))
            return wrapper
        return decorator
