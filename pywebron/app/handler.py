from types import SimpleNamespace
from typing import Callable, Any
from inspect import Parameter, signature
from .worker import Worker
from .window import Window
from .._pywebron_ import rust_stream_send
from ..configs import StreamSendModes
from .._pywebron_ import rust_stream_recv


class Handle:
    __slots__ = ("handle_id", "window_id")
    struct = SimpleNamespace

    def __init__(self, handle_id: str, window_id: int):
        self.handle_id = handle_id
        self.window_id = window_id

    def _logger_(self, payload: dict, send_mode: str = None):
        header = f"[{self.__class__.__name__}]-[{self.window_id}]-[{self.handle_id}]"
        print(f"{header}-[{send_mode}]: {payload}" if send_mode else f"{header}: {payload}")

    _BUILTIN_INJECTORS = {
        'Invoke': lambda req, handle_cls: handle_cls(req['handle_id'], req['window_id']),
        'Stream': lambda req, handle_cls: handle_cls(req['handle_id'], req['window_id']),
        'Worker': lambda req, _target: Worker,
        'Window': lambda req, _target: Window,
        'Struct': lambda req, annotation: annotation(
            **{
                k: req['payload'].get(k, getattr(annotation, k, None))
                for k in annotation.__annotations__
            }
        ),
    }

    @classmethod
    def _build_param_handles(cls, func: Callable):
        handles, injectors = [], cls._BUILTIN_INJECTORS

        for param_name, param in signature(func).parameters.items():
            annotation, default = param.annotation, param.default
            if annotation is not Parameter.empty:
                type_name = getattr(annotation, '__name__', None)
                injector = injectors.get(type_name) if isinstance(type_name, str) else None
                target = cls if injector else annotation if hasattr(annotation, '__annotations__') else None
                if target is not None:
                    handles.append(
                        lambda req, k=param_name, inj=injector or injectors['Struct'], tgt=target: (k, inj(req, tgt))
                    )
                    continue

            handles.append(
                lambda req, k=param_name, dv=default: (
                    k, req['payload'][k] if dv is Parameter.empty else req['payload'].get(k, dv)
                )
            )

        return handles

    @classmethod
    def _create_wrapper_(cls, func: Callable):
        handles = cls._build_param_handles(func)

        async def wrapper(req: dict):
            try:
                kwargs = dict(h(req) for h in handles)
                result = await func(**kwargs)
                if cls is Stream:
                    return None
                return result
            except Exception:
                raise

        return wrapper


class Invoke(Handle):
    async def json_response(self, stat: bool, mssg: str, data: Any = None):
        self._logger_(payload := {'stat': stat, 'mssg': mssg, 'data': data})
        return {'window_id': self.window_id, 'handle_id': self.handle_id, 'payload': payload}


class Stream(Handle):
    async def send(
            self, stat: bool, mssg: str, data: Any,
            send_mode: str = StreamSendModes.BROADCAST,
            mcast_wids: list[int] = None,
            save_history: bool = False
    ) -> bool:
        self._logger_(pld := {"stat": stat, "mssg": mssg, "data": data}, send_mode)
        match send_mode:
            case StreamSendModes.UNITYCAST:
                wids = [self.window_id]
            case StreamSendModes.MULTICAST:
                wids = mcast_wids
            case _:
                wids = None
        return await rust_stream_send(
            payload=pld,
            handle_id=self.handle_id,
            send_mode=send_mode,
            window_ids=wids,
            save_history=save_history
        )

    async def recv(self) -> Any:
        if res := await rust_stream_recv(self.handle_id):
            self.window_id = res['window_id']
            return res["payload"]
        return None
