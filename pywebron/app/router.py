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

    def _create_namespace(self, handle_cls, handle_type: str):
        def handle(name: str | None = None):
            return self._register_handler(handle_cls, handle_type, name)

        return SimpleNamespace(server=handle_cls, struct=Handle.struct, handle=handle)

    def _register_handler(self, handle_cls, handle_type: str, name: str | None = None):
        def decorator(func):
            self.handlers.append((name or func.__name__, handle_cls._create_wrapper_(func), handle_type))
            return func

        return decorator

    @staticmethod
    def register_routers(*routers: 'Router'):
        from ..configs import HANDLES

        for router in routers:
            HANDLES.setdefault(router.title, []).extend(
                {'name': name, 'type': handle_type, 'handler': wrapper}
                for name, wrapper, handle_type in router.handlers
            )
