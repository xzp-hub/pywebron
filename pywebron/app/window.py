from ..configs import BUILTIN_ICON_PATH, DwmCorners
from ..utils import generate_window_id
from .._pywebron_ import (
    rust_register_window,
)


class Window:
    _window_ids: set[int] = set()

    @classmethod
    def register_window(
            cls,
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
            is_main: bool = False,
            window_id: int = None,
    ) -> int:
        """注册窗口，返回 window_id"""
        if sum(map(bool, (html_content, link_content, dist_content))) > 1:
            raise ValueError("html_content, link_content, and dist_content cannot be used at the same time")

        if window_id is None:
            window_id = generate_window_id()

        return rust_register_window(
            title=title,
            width=width,
            height=height,
            html_content=html_content,
            link_content=link_content,
            dist_content=dist_content,
            icon_path=icon_path or BUILTIN_ICON_PATH,
            show_title_bar=show_title_bar,
            window_radius=window_radius,
            enable_resizable=enable_resizable,
            enable_devtools=enable_devtools,
            dwm_corner=dwm_corner,
            is_main=is_main,
            window_id=window_id,
        )

    @classmethod
    def register_windows(cls, *window_ids):
        """注册一个或多个窗口"""
        for wid in window_ids:
            cls._window_ids.add(wid)
