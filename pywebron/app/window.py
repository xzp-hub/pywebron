from ..configs import PROJECT_ROOT_PATH
from .._pywebron_ import (
    rust_register_window,
    rust_minimize_window,
    rust_maximize_window,
    rust_reappear_window,
    rust_shutdown_window,
    rust_dragdrop_window,
)


class Window:
    @staticmethod
    def register_window(
        title: str = "PyWebron App",
        content_path: str = None,
        content_url: str = None,
        width: int = 1200,
        height: int = 900,
        icon_path: str = None,
        show_title_bar: bool = True,
        enable_resizable: bool = True,
        enable_devtools: bool = True,
    ) -> bool:
        if all((content_path, content_url)):
            raise "content_path and content_url cannot be used at the same time"

        def pather(file_name):
            return f"{PROJECT_ROOT_PATH}/assets/{file_name}"

        return rust_register_window(
            title=title,
            width=width,
            height=height,
            content=content_path or content_url or pather("pywebron.html"),
            icon_path=icon_path or pather("pywebron.png"),
            show_title_bar=show_title_bar,
            enable_resizable=enable_resizable,
            enable_devtools=enable_devtools,
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
    def dragdrop_window(window_id: int, selector: str) -> bool:
        return rust_dragdrop_window(window_id, selector)
