from .handle import Handle
from typing import Any


class Invoke(Handle):
    @classmethod
    def handle(cls, alias: str = None):
        def decorator(func):
            return cls._handler_(func, alias)

        return decorator

    async def json_response(self, stat: bool, mssg: str, data: Any = None):
        self._logger_(payload := {'stat': stat, 'mssg': mssg, 'data': data})
        return {'window_id': self.window_id, 'handle_id': self.handle_id, 'payload': payload}
