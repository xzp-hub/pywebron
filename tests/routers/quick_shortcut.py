from asyncio import gather
from time import time
from traceback import format_exc

from pywebron import Router, Worker, Window
from pywebron.configs import PROJECT_ROOT_PATH
from pywebron.utils import save_file_dialog
from .tools import cpu_task

router = Router(title="快捷操作")
invoke = router.invoke
stream = router.stream


class QuickShortcutStruct(invoke.struct):
    n: int = 42


@invoke.handle("save_files_via_dialog_invoke")
async def save_files_via_dialog(server: invoke.server):
    try:
        source_path = f'{PROJECT_ROOT_PATH}/builtins/pywebron.html'
        new_path = await save_file_dialog(source_path)
        return await server.json_response(True, "文件保存成功", new_path)
    except Exception:
        return await server.json_response(False, "文件保存失败", format_exc())


@invoke.handle("execute_cpu_intensive_tasks_invoke")
async def execute_cpu_intensive_tasks(server: invoke.server, worker: Worker):
    try:
        start = time()
        task1, task2 = await gather(worker.run(cpu_task, 1), worker.run(cpu_task, 2))
        res = {"res": str(task1 + task2), "time": time() - start}
        return await server.json_response(True, "cpu 任务测试成功", res)
    except Exception:
        return await server.json_response(False, "cpu 任务测试失败", format_exc())


@invoke.handle("create_new_windows_at_runtime_invoke")
async def create_new_windows_at_runtime(server: invoke.server, window: Window):
    try:
        res = window.register_window(
            title="运行时创建窗口",
            width=1200,
            height=1200,
            show_title_bar=False,
            link_content="http://localhost:5173/",
        )
        return await server.json_response(True, f"运行时创建窗口成功：{res}", res)
    except Exception:
        return await server.json_response(False, "运行时创建窗口失败", format_exc())
