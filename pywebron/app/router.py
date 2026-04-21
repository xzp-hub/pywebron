from typing import Callable, List, TYPE_CHECKING

if TYPE_CHECKING:
    from .. import App

from .invoke import Invoke
from .stream import Stream
from .window import Window


class Router:
    """类似 FastAPI 的 APIRouter，支持分组注册处理器。

    用法:
        router = Router()

        @router.invoke.handle("window_controls")
        async def window_controls(invoke: router.invoke.server, struct: WindowControlsStruct):
            ...

        @router.stream.handle("system_monitoring")
        async def system_monitoring(stream: router.stream.server):
            ...

        # 在主应用中注册
        app.router.register_routers(router)
    """

    def __init__(self, title: str = "", app: 'App' = None):
        self.title = title
        self._app = app
        self._routers: List['Router'] = []
        self._pending_invoke: list[tuple[str, Callable]] = []
        self._pending_stream: list[tuple[str, Callable]] = []
        self.invoke = _InvokeRouter(self)
        self.stream = _StreamRouter(self)
    
    def register_routers(self, *routers: 'Router'):
        """注册一个或多个路由器"""
        for router in routers:
            self._routers.append(router)
            # 注册路由器中的所有 invoke 和 stream 处理器
            for name, handler in router._pending_invoke:
                print(f"[Router] 注册 Invoke 处理器: {name}")
            for name, handler in router._pending_stream:
                print(f"[Router] 注册 Stream 处理器: {name}")


class _InvokeRouter:
    """Invoke 路由器辅助类"""
    server = Invoke  # 类型注解用：invoke: router.invoke.server
    window = Window  # 窗口操作：router.invoke.window

    def __init__(self, router: Router):
        self._router = router
        self.struct = Invoke.struct  # 数据结构
        self.server = Invoke  # 服务端类型

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
    server = Stream  # 类型注解用：stream: router.stream.server

    def __init__(self, router: Router):
        self._router = router
        self.struct = Stream.struct  # 数据结构
        self.server = Stream  # 服务端类型

    def handle(self, alias: str = None):
        """注册 Stream 处理器到路由组。alias 为空时使用函数名。"""
        def decorator(func):
            name = alias or func.__name__
            wrapper = Stream._handler_(func, name)
            self._router._pending_stream.append((name, wrapper))
            return wrapper
        return decorator
