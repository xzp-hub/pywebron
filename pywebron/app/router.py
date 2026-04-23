from .handler import Handle, Invoke, Stream
from types import SimpleNamespace
from ..configs import HANDLES, HANDLE_INDEX
from typing import Callable


class Router:
    __slots__ = ("title", "handles", "invoke", "stream", "_handle_keys")

    def __init__(self, title: str):
        self.title = title
        self.handles: list[tuple[str, Callable, str]] = []
        self._handle_keys: set[tuple[str, str]] = set()
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
            key = (type_, handle_name)
            if key in self._handle_keys:
                raise ValueError(f"Duplicate {type_} handler in router '{self.title}': {handle_name}")
            self._handle_keys.add(key)
            self.handles.append((handle_name, cls._create_wrapper_(func), type_))
            return func

        return decorator

    @staticmethod
    def register_routers(*routers: 'Router'):
        existing = {
            (handler_type, name)
            for handler_type, mapping in HANDLE_INDEX.items()
            for name in mapping
        }
        pending: set[tuple[str, str]] = set()

        for router in routers:
            for name, _, type_ in router.handles:
                key = (type_, name)
                if key in existing or key in pending:
                    raise ValueError(f"Duplicate {type_} handler registration: {name}")
                pending.add(key)

        for router in routers:
            bucket = HANDLES.setdefault(router.title, [])
            for name, wrapper, handler_type in router.handles:
                bucket.append({'name': name, 'type': handler_type, 'handler': wrapper})
                HANDLE_INDEX[handler_type][name] = wrapper
