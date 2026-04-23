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
            handle_name = name or func.__name__
            if any(existing_name == handle_name and existing_type == type_ for existing_name, _, existing_type in self.handles):
                raise ValueError(f"Duplicate {type_} handler in router '{self.title}': {handle_name}")
            self.handles.append((handle_name, cls._create_wrapper_(func), type_))
            return func

        return decorator

    @staticmethod
    def register_routers(*routers: 'Router'):
        existing = {
            (item['type'], item['name'])
            for handlers in HANDLES.values()
            for item in handlers
        }
        pending: set[tuple[str, str]] = set()

        for router in routers:
            for name, _, type_ in router.handles:
                key = (type_, name)
                if key in existing or key in pending:
                    raise ValueError(f"Duplicate {type_} handler registration: {name}")
                pending.add(key)

        for router in routers:
            HANDLES.setdefault(router.title, []).extend(
                {'name': n, 'type': t, 'handler': w} for n, w, t in router.handles
            )
