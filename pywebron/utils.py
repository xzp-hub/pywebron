from ._pywebron_ import rust_save_file_dialog
import sys
import threading

_window_id_counter = 0
_window_id_lock = threading.Lock()

# JavaScript Number.MAX_SAFE_INTEGER = 2^53 - 1 = 9007199254740991
# Window IDs must stay within this limit to avoid precision loss when
# passed through JSON and handled in the browser.
_JS_MAX_SAFE_INTEGER = (1 << 53) - 1


def generate_window_id() -> int:
    global _window_id_counter
    with _window_id_lock:
        _window_id_counter += 1
        if _window_id_counter > _JS_MAX_SAFE_INTEGER:
            _window_id_counter = 1
        return _window_id_counter


def get_gil_status():
    if sys.version_info < (3, 13):
        return True
    return getattr(sys, "_is_gil_enabled", lambda: True)()


async def save_file_dialog(
        source_file_path: str,
        new_file_name: str | None = None,
        is_del_source_file: bool = False
) -> str:
    return await rust_save_file_dialog(
        source_file_path=source_file_path,
        new_file_name=new_file_name,
        is_del_source_file=is_del_source_file
    )
