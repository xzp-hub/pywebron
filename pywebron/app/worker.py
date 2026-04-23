from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from typing import Any, Callable, Dict
from asyncio import get_running_loop
from ..utils import get_gil_status
from os import cpu_count
from os import getenv
from .. import configs


class Worker:
    @classmethod
    def __func__(cls, func: Callable, args: tuple, kwargs: Dict) -> Any:
        return func(*args, **kwargs)

    @classmethod
    def init_worker_pool(cls) -> ProcessPoolExecutor | ThreadPoolExecutor | None:
        if configs.WORKER_POOL is not None:
            return configs.WORKER_POOL
        configured = getenv("PYWEBRON_WORKER_MAX")
        configured_workers = int(configured) if configured and configured.isdigit() else None
        if get_gil_status():
            configs.WORKER_POOL = ProcessPoolExecutor(
                max_workers=configured_workers or min(max((cpu_count() or 1) - 1, 1), 4),
            )
        else:
            workers = configured_workers or min(max(cpu_count() or 1, 2), 8)
            configs.WORKER_POOL = ThreadPoolExecutor(max_workers=workers)
        return configs.WORKER_POOL

    @classmethod
    async def run(cls, func: Callable, *args, **kwargs) -> Any:
        return await get_running_loop().run_in_executor(
            cls.init_worker_pool(), cls.__func__, func, args, kwargs
        )
