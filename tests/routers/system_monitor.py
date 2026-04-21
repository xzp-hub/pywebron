from asyncio import sleep as asyncio_sleep
from traceback import format_exc

from pywebron import Router, StreamSendModes
from .tools import SystemMonitoring, TerminalLogger

router = Router(title="系统监控")


@router.stream.handle("system_monitoring_stream")
async def system_monitoring(stream: router.stream.server):
    try:
        res = await SystemMonitoring.run(fast_mode=True)
        await stream.send(200, "监控数据获取成功", res, send_mode=StreamSendModes.BROADCAST)
        await asyncio_sleep(1)
        while True:
            try:
                res = await SystemMonitoring.run()
                await stream.send(200, "监控数据获取成功", res, send_mode=StreamSendModes.BROADCAST)
                await asyncio_sleep(3)
            except Exception:
                await stream.send(500, "监控数据获取失败", format_exc())
    except Exception:
        await stream.send(500, "监控数据获取失败", format_exc())


@router.stream.handle("terminal_log_stream")
async def terminal_log(stream: router.stream.server):
    try:
        with TerminalLogger.pause():
            await stream.send(200, "历史日志", {"logs": TerminalLogger.get_history_logs()})
        while True:
            try:
                if new_logs := TerminalLogger.get_current():
                    with TerminalLogger.pause():
                        await stream.send(200, "终端日志", {"logs": new_logs}, send_mode=StreamSendModes.UNITYCAST)
                await asyncio_sleep(0.3)
            except Exception:
                await stream.send(500, "终端日志错误", format_exc())
    except Exception:
        await stream.send(500, "终端日志错误", format_exc())
