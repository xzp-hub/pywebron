from ._pywebron_ import rust_save_file_dialog
import sys


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
