from types import SimpleNamespace
from typing import Callable, Any, List
from inspect import Parameter, signature
from .worker import Worker
from .window import Window
from .._pywebron_ import rust_stream_send
from ..configs import StreamSendModes
from .._pywebron_ import rust_stream_recv

class Handle:
    struct = SimpleNamespace
    __slots__ = ("handle_id", "window_id")

    _BUILTIN_INJECTORS = {
        'Invoke': lambda req, hc: hc(req['handle_id'], req['window_id']),
        'Stream': lambda req, hc: hc(req['handle_id'], req['window_id']),
        'Worker': lambda req, _: Worker,
        'Window': lambda req, _: Window,
    }

    def __init__(self, handle_id: str, window_id: int):
        self.handle_id = handle_id
        self.window_id = window_id

    def _logger_(self, payload: dict, send_mode: str = None):
        header = f"[{self.__class__.__name__}]-[{self.window_id}]-[{self.handle_id}]"
        print(f"{header}-[{send_mode}]: {payload}" if send_mode else f"{header}: {payload}")

    @classmethod
    def _create_wrapper_(cls, func: Callable):
        params = signature(func).parameters

        def maker(param_name):
            p = params[param_name]
            annot, default = p.annotation, p.default

            if annot is Parameter.empty:
                return lambda req, pn=param_name, d=default: (pn,
                                                              req['payload'].get(pn, d) if d is not Parameter.empty else
                                                              req['payload'][pn])

            type_name = getattr(annot, '__name__', None)

            if type_name in Handle._BUILTIN_INJECTORS:
                inject = Handle._BUILTIN_INJECTORS[type_name]
                return lambda req, pn=param_name: (pn, inject(req, cls))

            if hasattr(annot, '__annotations__'):
                return lambda req, a=annot, pn=param_name: (pn, a(
                    **{k: req['payload'].get(k, getattr(a, k, None)) for k in a.__annotations__}))

            return lambda req, pn=param_name, d=default: (pn,
                                                          req['payload'].get(pn, d) if d is not Parameter.empty else
                                                          req['payload'][pn])

        handles = [maker(p) for p in params]

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
