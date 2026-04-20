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
        header = f"[{self.__class__.__name__}]-[{self.window_id}]-[{self.handle_id}]"
        if send_mode:
            print(f"{header}-[{send_mode}]: {payload}")
        else:
            print(f"{header}: {payload}")

    @classmethod
    def _handler_(cls, func: Callable, alias: str | None):
        params = signature(func).parameters

        def maker(param_name):
            annot, default = (param_proxy := params[param_name]).annotation, param_proxy.default
            annot_name = getattr(annot, '__name__', None)
            annot_class_name = getattr(annot.__class__, '__name__', None)
            
            match annot_name:
                case 'Invoke' | 'Stream':
                    return lambda req: (param_name, cls(req['handle_id'], req['window_id']))
                case 'Worker':
                    return lambda req: (param_name, Worker)
                case _ if annot_class_name in ('_InvokeRouter', '_StreamRouter'):
                    # 处理 router.invoke 和 router.stream 类型注解
                    return lambda req: (param_name, cls(req['handle_id'], req['window_id']))
                case _ if hasattr(annot, '__annotations__'):
                    return lambda req: (param_name, annot(**{
                        ann: req['payload'].get(ann, getattr(annot, ann, None))
                        for ann in annot.__annotations__
                    }))
                case _:
                    return lambda req: (param_name, req['payload'].get(param_name, default)
                    if default is not Parameter.empty else req['payload'][param_name])

        handles = [maker(param) for param in params]

        async def wrapper(req: dict):
            return await func(**dict(handle(req) for handle in handles))

        name = alias or func.__name__

        from ..configs import INVOKE_HANDLES, STREAM_HANDLES
        match cls.__name__:
            case 'Invoke':
                INVOKE_HANDLES[name] = wrapper
            case 'Stream':
                STREAM_HANDLES[name] = wrapper
        return wrapper
