from asyncio import sleep as asyncio_sleep
from traceback import format_exc

from pywebron import Router, StreamSendModes
from tests.utills.tools import SystemMonitoring, TerminalLogger

router = Router(title="系统监控")
stream = router.stream


@stream.handle("system_monitoring_stream")
async def system_monitoring(server: stream.server):
    try:
        # 首次快速获取数据并发送
        res = await SystemMonitoring.run(fast_mode=True)
        await server.send(200, "监控数据获取成功", res, send_mode=StreamSendModes.BROADCAST)
        # 进入正常监控循环
        while True:
            try:
                await asyncio_sleep(3)  # 先等待，再获取数据
                res = await SystemMonitoring.run()
                await server.send(200, "监控数据获取成功", res, send_mode=StreamSendModes.BROADCAST)
            except Exception:
                await server.send(500, "监控数据获取失败", format_exc())
    except Exception:
        await server.send(500, "监控数据获取失败", format_exc())


@stream.handle("terminal_log_stream")
async def terminal_log(server: stream.server):
    try:
        with TerminalLogger.pause():
            await server.send(200, "历史日志", {"logs": TerminalLogger.get_history_logs()})
        while True:
            try:
                if new_logs := TerminalLogger.get_current():
                    with TerminalLogger.pause():
                        await server.send(200, "终端日志", {"logs": new_logs}, send_mode=StreamSendModes.UNITYCAST)
                await asyncio_sleep(0.3)
            except Exception:
                await server.send(500, "终端日志错误", format_exc())
    except Exception:
        await server.send(500, "终端日志错误", format_exc())
