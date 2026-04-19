from .._pywebron_ import rust_stream_send, rust_stream_recv
from ..configs import StreamSendModes
from .handle import Handle
from typing import Any


class Stream(Handle):
    _log_count = {}
    _limited_streams = {'system_monitoring'}

    @classmethod
    def handle(cls, alias: str = None):
        def decorator(func):
            return cls._handler_(func, alias)

        return decorator

    async def send(
            self,
            code: int,
            mssg: str,
            data: Any,
            send_mode: StreamSendModes = StreamSendModes.BROADCAST,
            mcast_win_ids: list[int] = None,
    ) -> bool:
        payload, window_ids = {"code": code, "mssg": mssg, "data": data}, None
        self._logger_(payload, send_mode)
        match send_mode:
            case StreamSendModes.BROADCAST:
                window_ids = None
            case StreamSendModes.MULTICAST:
                window_ids = mcast_win_ids
            case StreamSendModes.UNITYCAST:
                window_ids = [self.window_id]
        return await rust_stream_send(
            payload=payload,
            handle_id=self.handle_id,
            send_mode=send_mode,
            window_ids=window_ids,
        )

    async def recv(self) -> Any:
        if (res := await rust_stream_recv(self.handle_id)) is not None:
            return res["payload"]
        return res
