from asyncio import sleep as asyncio_sleep, gather
from tools import SystemMonitoring, cpu_task
from pywebron.utils import save_file_dialog
from pywebron import App, StreamSendModes
from pywebron.configs import PROJECT_ROOT_PATH, DwmCornerPreference
from traceback import format_exc
from pathlib import Path
from time import time

app = App(prewarm_webview=False)


class WindowControlsStruct(app.invoke.struct):
    control_type: str


class SetupDragRegionStruct(app.invoke.struct):
    selector: str = ".header"


@app.invoke.handle("setup_drag_region_invoke")
async def setup_drag_region(invoke: app.invoke, struct: SetupDragRegionStruct):
    try:
        selector = struct.selector
        res = app.window.dragdrop_window(invoke.window_id, selector)
        return await invoke.json_response(
            200, "拖拽区域设置成功", {"selector": selector, "result": res}
        )
    except Exception:
        return await invoke.json_response(500, "拖拽区域设置失败", format_exc())


@app.invoke.handle("window_controls_invoke")
async def window_controls(invoke: app.invoke, struct: WindowControlsStruct):
    control_type, res = None, None
    try:
        match control_type := struct.control_type:
            case "minimize_window":
                res = app.window.minimize_window(invoke.window_id)
            case "maximize_window":
                res = app.window.maximize_window(invoke.window_id)
            case "reappear_window":
                res = app.window.reappear_window(invoke.window_id)
            case "shutdown_window":
                res = app.window.shutdown_window(invoke.window_id)
        return await invoke.json_response(200, f"{control_type} 操作成功", res)
    except Exception:
        return await invoke.json_response(500, f"{control_type} 操作失败", format_exc())


@app.invoke.handle("cpu_intensive_task_invoke_command")
async def cpu_intensive_task(invoke: app.invoke, worker: app.worker):
    try:
        start = time()
        task1, task2 = await gather(worker.run(cpu_task, 1), worker.run(cpu_task, 2))
        res = {"res": str(task1 + task2), "time": time() - start}
        return await invoke.json_response(200, "cpu 任务测试成功", res)
    except Exception:
        return await invoke.json_response(500, "cpu 任务测试失败", format_exc())


@app.invoke.handle("running_create_window_invoke_handle")
async def running_create_window(invoke: app.invoke):
    try:
        res = app.window.register_window(
            title="运行时创建窗口",
            width=1200,
            height=1200,
            show_title_bar=False,
        )
        return await invoke.json_response(200, f"运行时创建窗口成功：{res}", res)
    except Exception:
        return await invoke.json_response(500, "运行时创建窗口失败", format_exc())


@app.invoke.handle("file_download_invoke")
async def file_download(invoke: app.invoke):
    try:
        source_path = str(Path(PROJECT_ROOT_PATH) / "assets" / "src" / "index.html")
        new_path = await save_file_dialog(str(source_path))
        return await invoke.json_response(200, "文件保存成功", new_path)
    except Exception:
        return await invoke.json_response(500, "文件保存失败", format_exc())


@app.stream.handle("system_monitoring_stream")
async def system_monitoring(stream: app.stream):
    try:
        res = await SystemMonitoring.run(fast_mode=True)
        await stream.send(200, "监控数据获取成功", res)
        await asyncio_sleep(1)
        while True:
            try:
                res = await SystemMonitoring.run()
                await stream.send(200, "监控数据获取成功", res)
                await asyncio_sleep(3)
            except Exception:
                await stream.send(500, "监控数据获取失败", format_exc())
    except Exception:
        await stream.send(500, "监控数据获取失败", format_exc())


class ChatRoomStruct(app.stream.struct):
    n: int = 42


@app.stream.handle("chat_room_stream")
async def chat_room(stream: app.stream, worker: app.worker, struct: ChatRoomStruct):
    try:
        await stream.send(
            200,
            "欢迎加入聊天室",
            {"type": "system"},
            send_mode=StreamSendModes.BROADCAST,
        )
        while True:
            match res := await stream.recv():
                case None:
                    await asyncio_sleep(0.1)
                case "multicast_test":
                    (wids := list(app.get_windows().keys())).remove(stream.window_id)
                    await stream.send(
                        200,
                        "组播功能测试",
                        {"type": "chat"},
                        send_mode=StreamSendModes.MULTICAST,
                        mcast_win_ids=wids[0:1],
                    )
                case "worker_test":
                    res = await worker.run(cpu_task, n := struct.n)
                    await stream.send(
                        200,
                        f"Worker 任务完成，n: {n}, result: {res}",
                        {"type": "chat", "result": res, "n": n},
                        send_mode=StreamSendModes.UNITYCAST,
                    )
                case _:
                    await stream.send(
                        200,
                        f"收到 {res}",
                        {"type": "chat"},
                        send_mode=StreamSendModes.UNITYCAST,
                    )
    except Exception:
        await stream.send(500, "聊天室错误", format_exc())


if __name__ == "__main__":
    app.window.register_window(
        title="PyWebron 控制面板 1",
        width=1200,
        height=1200,
        show_title_bar=True,
        enable_resizable=True,
        dwm_corner_preference=DwmCornerPreference.DO_NOT_ROUND,  # noqa: F821
        # content_url= 'http://localhost:5173/'
    )
    app.run()
