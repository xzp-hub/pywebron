from ..configs import PROJECT_ROOT_PATH, DwmCorners
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
        content_dist: str = None,
        width: int = 1200,
        height: int = 900,
        icon_path: str = None,
        show_title_bar: bool = True,
        enable_resizable: bool = True,
        enable_devtools: bool = True,
        dwm_corner: DwmCorners = DwmCorners.SYSTEM_ROUND,
    ) -> bool:
        if sum(bool(x) for x in (content_path, content_url, content_dist)) > 1:
            raise ValueError(
                "content_path, content_url, and content_dist cannot be used at the same time"
            )

        if content_url is not None and not show_title_bar:
            raise ValueError("when using content_url, show_title_bar must be True")

        def pather(file_name):
            return f"{PROJECT_ROOT_PATH}/assets/{file_name}"

        return rust_register_window(
            title=title,
            width=width,
            height=height,
            content_path=content_path,
            content_url=content_url,
            content_dist=content_dist,
            icon_path=icon_path or pather("pywebron.png"),
            show_title_bar=show_title_bar,
            enable_resizable=enable_resizable,
            enable_devtools=enable_devtools,
            dwm_corner=dwm_corner,
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
