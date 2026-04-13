from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from typing import Any, Callable, Dict
from asyncio import get_running_loop
from ..utils import get_gil_status
from os import cpu_count
from .. import configs


class Worker:
    @classmethod
    def __func__(cls, func: Callable, args: tuple, kwargs: Dict) -> Any:
        return func(*args, **kwargs)

    @classmethod
    def init_worker_pool(cls) -> ProcessPoolExecutor | ThreadPoolExecutor | None:
        if configs.WORKER_POOL is not None:
            return configs.WORKER_POOL
        if get_gil_status():
            configs.WORKER_POOL = ProcessPoolExecutor(
                max_workers=5,
            )
        else:
            configs.WORKER_POOL = ThreadPoolExecutor(max_workers=cpu_count() * 2)
        return configs.WORKER_POOL

    @classmethod
    async def run(cls, func: Callable, *args, **kwargs) -> Any:
        return await get_running_loop().run_in_executor(
            cls.init_worker_pool(), cls.__func__, func, args, kwargs
        )
