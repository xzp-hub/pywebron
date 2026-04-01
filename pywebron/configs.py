from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from enum import StrEnum
from typing import Dict, Callable
from pathlib import Path

DEFAULT_DIR = str(Path(__file__).parents[1])


class StreamSendModes(StrEnum):
    UNITYCAST = 'unitycast'
    MULTICAST = 'multicast'
    BROADCAST = 'broadcast'


INVOKE_HANDLES: Dict[str, Callable] = {}
STREAM_HANDLES: Dict[str, Callable] = {}

WORKER: ProcessPoolExecutor | ThreadPoolExecutor | None = None
