from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from multiprocessing import get_context
from typing import Any, Callable, Dict
from asyncio import get_running_loop
from ..utils import get_gil_status
from os import cpu_count
from .. import configs


def _worker_func(func: Callable, args: tuple, kwargs: Dict) -> Any:
    return func(*args, **kwargs)


class Worker:
    @classmethod
    def __func__(cls, func: Callable, args: tuple, kwargs: Dict) -> Any:
        return _worker_func(func, args, kwargs)

    @classmethod
    def get_worker(cls) -> ProcessPoolExecutor | ThreadPoolExecutor | None:
        if configs.WORKER is not None:
            return configs.WORKER
        if get_gil_status():
            mp_context = get_context("spawn")
            configs.WORKER = ProcessPoolExecutor(
                max_workers=5,
                mp_context=mp_context,
            )
        else:
            configs.WORKER = ThreadPoolExecutor(max_workers=cpu_count() * 2)
        return configs.WORKER

    @classmethod
    async def run(cls, func: Callable, *args, **kwargs) -> Any:
        return await get_running_loop().run_in_executor(
            cls.get_worker(), cls.__func__, func, args, kwargs
        )
