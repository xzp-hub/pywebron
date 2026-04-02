from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from typing import Dict, Callable
from pathlib import Path
from enum import StrEnum

# 项目根目录路径
PROJECT_ROOT_PATH = str(Path(__file__).parents[1])

# 资源目录路径
ASSETS_SRC_DIR = f'{PROJECT_ROOT_PATH}/assets/src'


# 流消息发送模式
class StreamSendModes(StrEnum):
    UNITYCAST = 'unitycast'  # 单播：发送给单个接收者
    MULTICAST = 'multicast'  # 组播：发送给指定的多个接收者
    BROADCAST = 'broadcast'  # 广播：发送给所有接收者


# 调用处理器注册表
INVOKE_HANDLES: Dict[str, Callable] = {}

# 流处理器注册表
STREAM_HANDLES: Dict[str, Callable] = {}

# 工作线程池（进程池或线程池）
WORKER_POOL: ProcessPoolExecutor | ThreadPoolExecutor | None = None
