from asyncio import sleep as asyncio_sleep, gather
from tools import SystemMonitoring, cpu_task
from pywebron.utils import save_file_dialog
from pywebron import App, StreamSendModes
from pywebron.configs import DEFAULT_DIR
from traceback import format_exc
from pathlib import Path
from time import time

app = App()


class WindowControlsStruct(app.invoke.struct):
    control_type: str


@app.invoke.handle("window_controls_invoke")
async def window_controls(invoke: app.invoke, struct: WindowControlsStruct):
    control_type = None
    try:
        res, control_type = None, struct.control_type
        match control_type:
            case "minimize_window":
                res = app.window.minimize_window(invoke.window_id)
            case "maximize_window":
                res = app.window.maximize_window(invoke.window_id)
            case "reappear_window":
                res = app.window.reappear_window(invoke.window_id)
            case "shutdown_window":
                res = app.window.shutdown_window(invoke.window_id)
        return await invoke.json_response(200, f'{control_type} 操作成功', res)
    except Exception:
        return await invoke.json_response(500, f'{control_type} 操作失败', format_exc())


@app.invoke.handle("cpu_intensive_task_invoke_command")
async def cpu_intensive_task(invoke: app.invoke, worker: app.worker):
    try:
        start = time()
        task1, task2 = await gather(worker.run(cpu_task, 1), worker.run(cpu_task, 2))
        res = {"res": str(task1 + task2), "time": time() - start}
        return await invoke.json_response(200, 'cpu 任务测试成功', res)
    except Exception:
        return await invoke.json_response(500, 'cpu 任务测试失败', format_exc())


@app.invoke.handle("running_create_window_invoke_handle")
async def running_create_window(invoke: app.invoke):
    try:
        res = app.window.register_window(
            window_title="运行时创建窗口",
            window_width=1200,
            window_height=1200,
            window_is_decorations=False,
        )
        return await invoke.json_response(200, f'运行时创建窗口成功：{res}', res)
    except Exception:
        return await invoke.json_response(500, '运行时创建窗口失败', format_exc())


@app.invoke.handle("file_download_invoke")
async def file_download(invoke: app.invoke):
    try:
        source_file_path = str(Path(DEFAULT_DIR) / 'assets' / 'pywebron.html')
        new_path = await save_file_dialog(str(source_file_path))
        return await invoke.json_response(200, '文件保存成功', new_path)
    except Exception:
        return await invoke.json_response(500, '文件保存失败', format_exc())


@app.stream.handle("system_monitoring_stream")
async def system_monitoring(stream: app.stream):
    try:
        res = await SystemMonitoring.run(fast_mode=True)
        await stream.send(200, '监控数据获取成功', res)
        await asyncio_sleep(1)
        while True:
            try:
                res = await SystemMonitoring.run()
                await stream.send(200, '监控数据获取成功', res)
                await asyncio_sleep(3)
            except Exception:
                await stream.send(500, '监控数据获取失败', format_exc())
    except Exception:
        await stream.send(500, '监控数据获取失败', format_exc())


class ChatRoomStruct(app.stream.struct):
    n: int = 42


@app.stream.handle("chat_room_stream")
async def chat_room(stream: app.stream, worker: app.worker, struct: ChatRoomStruct):
    try:
        await stream.send(200, '欢迎加入聊天室', {"type": "system"}, send_mode=StreamSendModes.BROADCAST)
        while True:
            match await stream.recv():
                case None:
                    await asyncio_sleep(0.1)
                case "multicast_test":
                    window_ids = list(app.get_windows().keys())
                    window_ids.remove(stream.window_id)
                    await stream.send(200, '组播功能测试', {"type": "chat"},
                                      send_mode=StreamSendModes.MULTICAST, mcast_win_ids=window_ids[0:1])
                case "worker_test":
                    n = struct.n
                    result = await worker.run(cpu_task, n)
                    await stream.send(200, f'Worker 任务完成，n: {n}, result: {result}',
                                      {"type": "chat", "result": str(
                                          result), "n": n},
                                      send_mode=StreamSendModes.UNITYCAST)
                case str() as res:
                    await stream.send(200, f'{res}, 收到', {"type": "chat"}, send_mode=StreamSendModes.UNITYCAST)
    except Exception:
        await stream.send(500, '聊天室错误', format_exc())


if __name__ == "__main__":
    app.window.register_window(
        window_title="PyWebron 控制面板 1",
        window_width=1800,
        window_height=1800,
        window_is_decorations=False,
        window_is_resizable=True,
    )
    app.run()
