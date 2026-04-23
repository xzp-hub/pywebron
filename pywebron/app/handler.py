from .._pywebron_ import rust_stream_send
from .._pywebron_ import rust_stream_recv
from inspect import Parameter, signature
from ..configs import StreamSendModes
from types import SimpleNamespace
from typing import Callable, Any
from .worker import Worker
from .window import Window


class Handle:
    __slots__ = ("handle_id", "window_id")
    struct = SimpleNamespace

    def __init__(self, handle_id: str, window_id: int):
        self.handle_id = handle_id
        self.window_id = window_id

    def _logger_(self, payload: dict, send_mode: str = None):
        header = f"[{self.__class__.__name__}]-[{self.window_id}]-[{self.handle_id}]"
        print(f"{header}-[{send_mode}]: {payload}" if send_mode else f"{header}: {payload}")

    __TYPE_INJECTORS = {
        'Invoke': lambda req, klass: klass(req['handle_id'], req['window_id']),
        'Stream': lambda req, klass: klass(req['handle_id'], req['window_id']),
        'Worker': lambda req, klass: Worker,
        'Window': lambda req, klass: Window,
        'Struct': lambda req, klass: klass(**{ann: req['payload'].get(
            ann, getattr(klass, ann, None)) for ann in klass.__annotations__
        })
    }

    @classmethod
    def _maker_(cls, func: Callable):
        handles, injectors = [], cls.__TYPE_INJECTORS

        for pk, pv in signature(func).parameters.items():
            if (ann := pv.annotation) is not Parameter.empty:
                type_name = getattr(ann, '__name__', None)
                injector = injectors.get(type_name) if isinstance(type_name, str) else None
                target = cls if injector else ann if hasattr(ann, '__annotations__') else None
                if target is not None:
                    ier = injector or injectors['Struct']
                    handles.append(lambda req, pak=pk, inj=ier, tgt=target: (pak, inj(req, tgt)))
                    continue

            handles.append(
                lambda req, dk=pk, dv=pv.default: (
                    dk, req['payload'][dk] if dv is Parameter.empty else req['payload'].get(dk, dv)
                )
            )

        return handles

    @classmethod
    def _create_wrapper_(cls, func: Callable):
        handles = cls._maker_(func)

        async def wrapper(req: dict):
            return await func(**dict(handle(req) for handle in handles))

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
