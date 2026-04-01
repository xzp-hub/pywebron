from ..configs import INVOKE_HANDLES, STREAM_HANDLES
from dataclasses import dataclass, asdict
from inspect import Parameter, signature
from typing import Callable

from .worker import Worker


@dataclass
class Struct:
    def __init__(self, **kwargs):
        self.__dict__.update(kwargs)

    def __repr__(self):
        return repr(asdict(self))


class Handle:
    struct = Struct
    __slots__ = ("handle_id", "window_id")

    def __init__(self, handle_id: str, window_id: int):
        self.handle_id = handle_id
        self.window_id = window_id

    def _logger_(self, payload: dict, send_mode: str = None):
        header = f"[Stream]-[{self.window_id}]-[{self.handle_id}]"
        if send_mode:
            print(f"[Stream]-{header}-[{send_mode}]: {payload}")
        else:
            print(f"[Invoke]-{header}: {payload}")

    @classmethod
    def _handler_(cls, func: Callable, alias: str):
        params = signature(func).parameters

        def maker(param_name):
            annot, default = (param_proxy := params[param_name]).annotation, param_proxy.default
            match getattr(annot, '__name__', None):
                case 'Invoke' | 'Stream':
                    return lambda req: (param_name, cls(req['handle_id'], req['window_id']))
                case 'Worker':
                    return lambda req: (param_name, Worker)
                case _ if hasattr(annot, '__annotations__'):
                    return lambda req: (param_name, annot(**{
                        ann: req['payload'].get(ann, getattr(annot, ann, None)) for ann in annot.__annotations__
                    }))
                case _:
                    return lambda req: (param_name, req['payload'].get(param_name, default)
                    if default is not Parameter.empty else req['payload'][param_name])

        handlers = [maker(param) for param in params]

        async def wrapper(req: dict):
            return await func(**dict(handler(req) for handler in handlers))

        match cls.__name__:
            case 'Invoke':
                INVOKE_HANDLES[alias or func.__name__] = wrapper
            case 'Stream':
                STREAM_HANDLES[alias or func.__name__] = wrapper
        return wrapper
