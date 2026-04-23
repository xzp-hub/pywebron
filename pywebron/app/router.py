from .handler import Handle, Invoke, Stream
from types import SimpleNamespace
from ..configs import HANDLES
from typing import Callable


class Router:
    __slots__ = ("title", "handles", "invoke", "stream")

    def __init__(self, title: str):
        self.title = title
        self.handles: list[tuple[str, Callable, str]] = []
        self.invoke = self.__create_namespace(Invoke, "invoke")
        self.stream = self.__create_namespace(Stream, "stream")

    def __create_namespace(self, cls, type_: str):
        return SimpleNamespace(
            server=cls,
            struct=Handle.struct,
            handle=lambda name=None: self.__register_handler(cls, type_, name)
        )

    def __register_handler(self, cls, type_: str, name: str | None):
        def decorator(func):
            self.handles.append((name or func.__name__, cls._create_wrapper_(func), type_))
            return func

        return decorator

    @staticmethod
    def register_routers(*routers: 'Router'):
        for router in routers:
            HANDLES.setdefault(router.title, []).extend(
                {'name': n, 'type': t, 'handler': w} for n, w, t in router.handles
            )
