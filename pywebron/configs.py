from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from typing import Dict, Callable, List
from enum import StrEnum, IntEnum
from pathlib import Path
from os import getenv

# 项目根目录路径
PROJECT_ROOT_PATH = str(Path(__file__).parents[1])


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
HANDLE_INDEX: Dict[str, Dict[str, Callable]] = {"invoke": {}, "stream": {}}

# HANDLES 改为延迟构建，减少运行时冗余存储
# 原结构按 router.title 分组；延迟构建时统一归入 "app" 组
_HANDLES_CACHE: Dict[str, List[Dict[str, str | Callable]]] = {}  # pyright: ignore[reportUnknownVariableType, reportDeprecated, reportMissingTypeArgument]
_HANDLES_BUILT = False

def build_handles() -> Dict[str, List[Dict[str, str | Callable]]]:
    """从 HANDLE_INDEX 延迟构建 handlers 列表"""
    global _HANDLES_BUILT, _HANDLES_CACHE
    if _HANDLES_BUILT:
        return _HANDLES_CACHE
    result: Dict[str, List[Dict[str, str | Callable]]] = {"app": []}
    for handler_type, mapping in HANDLE_INDEX.items():
        for name, handler in mapping.items():
            result["app"].append({'name': name, 'type': handler_type, 'handler': handler})
    _HANDLES_CACHE = result
    _HANDLES_BUILT = True
    return result

def invalidate_handles_cache():
    """注册新 handler 时调用，使缓存失效"""
    global _HANDLES_BUILT
    _HANDLES_BUILT = False

# 工作任务池（进程池或线程池）
WORKER_POOL: ProcessPoolExecutor | ThreadPoolExecutor | None = None
LOG_LEVEL = getenv("PYWEBRON_LOG_LEVEL", "error").strip().lower()
ENABLE_HANDLE_LOGS = LOG_LEVEL == "debug"
