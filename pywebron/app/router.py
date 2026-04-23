from types import SimpleNamespace
from typing import Callable
from .handler import Handle, Invoke, Stream


class Router:
    __slots__ = ("title", "handlers", "invoke", "stream")

    def __init__(self, title: str):
        self.title = title
        self.handlers: list[tuple[str, Callable, str]] = []
        self.invoke = self._create_namespace(Invoke, "invoke")
        self.stream = self._create_namespace(Stream, "stream")

    def _create_namespace(self, cls, type_: str):
        return SimpleNamespace(
            server=cls,
            struct=Handle.struct,
            handle=lambda name=None: self._register_handler(cls, type_, name)
        )

    def _register_handler(self, cls, type_: str, name: str | None):
        def decorator(func):
            self.handlers.append((name or func.__name__, cls._create_wrapper_(func), type_))
            return func
        return decorator

    @staticmethod
    def register_routers(*routers: 'Router'):
        from ..configs import HANDLES
        for r in routers:
            HANDLES.setdefault(r.title, []).extend(
                {'name': n, 'type': t, 'handler': w} for n, w, t in r.handlers
            )
