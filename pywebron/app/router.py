from typing import Callable
from .invoke import Invoke
from .stream import Stream


class Router:
    """类似 FastAPI 的 APIRouter，支持分组注册处理器。

    用法:
        router = Router()

        @router.invoke.handle("window_controls")
        async def window_controls(invoke: router.invoke, struct: WindowControlsStruct):
            ...

        @router.stream.handle("system_monitoring")
        async def system_monitoring(stream: router.stream):
            ...

        # 在主应用中注册
        app.include_router(router)
    """

    def __init__(self):
        self._pending_invoke: list[tuple[str, Callable]] = []
        self._pending_stream: list[tuple[str, Callable]] = []
        self.invoke = _InvokeRouter(self)
        self.stream = _StreamRouter(self)


class _InvokeRouter:
    """Invoke 路由器辅助类"""
    def __init__(self, router: Router):
        self._router = router
        self.struct = Invoke.struct  # 添加 struct 属性

    def handle(self, alias: str = None):
        """注册 Invoke 处理器到路由组。alias 为空时使用函数名。"""
        def decorator(func):
            name = alias or func.__name__
            wrapper = Invoke._handler_(func, name)
            self._router._pending_invoke.append((name, wrapper))
            return wrapper
        return decorator


class _StreamRouter:
    """Stream 路由器辅助类"""
    def __init__(self, router: Router):
        self._router = router
        self.struct = Stream.struct  # 添加 struct 属性

    def handle(self, alias: str = None):
        """注册 Stream 处理器到路由组。alias 为空时使用函数名。"""
        def decorator(func):
            name = alias or func.__name__
            wrapper = Stream._handler_(func, name)
            self._router._pending_stream.append((name, wrapper))
            return wrapper
        return decorator
