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
            html_content: str = None,
            link_content: str = None,
            dist_content: str = None,
            width: int = 1200,
            height: int = 900,
            icon_path: str = None,
            show_title_bar: bool = True,
            window_radius: int = 5,
            enable_resizable: bool = True,
            enable_devtools: bool = True,
            dwm_corner: DwmCorners = DwmCorners.SYSTEM_ROUND,
    ) -> bool:
        pather = lambda name: fr"{PROJECT_ROOT_PATH}\assets\{name}"

        if sum(map(bool, (html_content, link_content, dist_content))) > 1:
            raise ValueError("html_content, link_content, and dist_content cannot be used at the same time")

        if not tuple(filter(None, (html_content, link_content, dist_content))):
            html_content = pather("pywebron.html")

        return rust_register_window(
            title=title,
            width=width,
            height=height,
            html_content=html_content,
            link_content=link_content,
            dist_content=dist_content,
            icon_path=icon_path or pather("pywebron.png"),
            show_title_bar=show_title_bar,
            window_radius=window_radius,
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
