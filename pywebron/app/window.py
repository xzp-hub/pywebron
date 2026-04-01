from ..configs import DEFAULT_DIR
from pathlib import Path
from .._pywebron_ import (
    rust_register_window,
    rust_minimize_window,
    rust_maximize_window,
    rust_reappear_window,
    rust_shutdown_window,
    rust_start_drag_window,
)


class Window:
    @staticmethod
    def register_window(
            window_title: str = 'PyWebron App',
            window_content_path: str = None,
            window_content_url: str = None,
            window_width: int = 1200,
            window_height: int = 900,
            window_icon_path: str = None,
            window_is_decorations: bool = True,
            window_is_resizable: bool = True,
            window_is_devtools: bool = True,
            window_corner_radius: int = 1,  # DWMWCP_DONOTROUND = 1
    ) -> bool:
        if all((window_content_path, window_content_url)):
            raise 'window_content_path and window_content_url cannot be used at the same time'
        pather = lambda file_name: str(Path(DEFAULT_DIR) / 'assets' / file_name)
        return rust_register_window(
            title=window_title,
            width=window_width,
            height=window_height,
            content=window_content_path or window_content_url or pather('pywebron.html'),
            icon_path=window_icon_path or pather('pywebron.png'),
            decorations=window_is_decorations,
            resizable=window_is_resizable,
            devtools=window_is_devtools,
            corner=window_corner_radius,
        )

    @staticmethod
    def minimize_window(window_id: int) -> bool:
        return rust_minimize_window(window_id)

    @staticmethod
    def maximize_window(window_id: int) -> bool:
        return rust_maximize_window(window_id)

    @staticmethod
    def reappear_window(window_id: int) -> bool:
        return rust_reappear_window(window_id)

    @staticmethod
    def shutdown_window(window_id: int) -> bool:
        return rust_shutdown_window(window_id)

    @staticmethod
    def start_drag_window(window_id: int, button: int = 1) -> bool:
        return rust_start_drag_window(window_id, button, 0, 0)
