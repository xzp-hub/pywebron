from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from contextvars import ContextVar
from typing import Dict, Callable, List
from enum import StrEnum, IntEnum
from pathlib import Path

# 项目根目录路径
PROJECT_ROOT_PATH = str(Path(__file__).parents[1])


# 当前 handler 执行的窗口 ID（用于终端日志按窗口归属路由）
CURRENT_WINDOW_ID: ContextVar[int | None] = ContextVar('CURRENT_WINDOW_ID', default=None)

# 流消息发送模式
class StreamSendModes(StrEnum):
    UNITYCAST = "unitycast"  # 单播：回复给最近发消息的来源窗口（通过 recv 动态更新）
    MULTICAST = "multicast"  # 组播：发送到指定的多个窗口
    BROADCAST = "broadcast"  # 广播：发送到所有窗口


# Windows DWM 窗口圆角模式（对应 DWM_WINDOW_CORNER_PREFERENCE）
class DwmCorners(IntEnum):
    SYSTEM_ROUND = 0  # 系统默认圆角
    ZEROES_ROUND = 1  # 不圆角（直角）
    NORMAL_ROUND = 2  # 正常圆角
    LITTLE_ROUND = 3  # 小圆角


# 统一的处理器注册表
# 结构: {
#     'app': [{'name': 'handler_name', 'type': 'invoke'|'stream', 'handler': <function>}],
#     'router_title': [{'name': 'handler_name', 'type': 'invoke'|'stream', 'handler': <function>}],
# }
HANDLES: Dict[str, List[Dict[str, str | Callable]]] = {}  # pyright: ignore[reportUnknownVariableType, reportDeprecated, reportMissingTypeArgument]

# 工作任务池（进程池或线程池）
WORKER_POOL: ProcessPoolExecutor | ThreadPoolExecutor | None = None
