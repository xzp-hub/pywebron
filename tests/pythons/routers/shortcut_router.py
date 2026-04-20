from asyncio import gather
from time import time
from traceback import format_exc

from pywebron import Router, Worker
from pywebron.configs import PROJECT_ROOT_PATH
from pywebron.utils import save_file_dialog
from tools import cpu_task

router = Router()


@router.invoke.handle("save_files_via_dialog_invoke")
async def save_files_via_dialog(invoke: router.invoke):
    try:
        source_path = f'{PROJECT_ROOT_PATH}/assets/pywebron.html'
        new_path = await save_file_dialog(str(source_path))
        return await invoke.json_response(True, "文件保存成功", new_path)
    except Exception:
        return await invoke.json_response(False, "文件保存失败", format_exc())


@router.invoke.handle("execute_cpu_intensive_tasks_invoke")
async def execute_cpu_intensive_tasks(invoke: router.invoke, worker: Worker):
    try:
        start = time()
        task1, task2 = await gather(worker.run(cpu_task, 1), worker.run(cpu_task, 2))
        res = {"res": str(task1 + task2), "time": time() - start}
        return await invoke.json_response(True, "cpu 任务测试成功", res)
    except Exception:
        return await invoke.json_response(False, "cpu 任务测试失败", format_exc())


def create_shortcut_router():
    """快捷操作分组：文件保存、CPU任务测试等"""
    return router
