from ..configs import PROJECT_ROOT_PATH, DwmCorners
from .._pywebron_ import (
    rust_register_window,
    rust_minimize_window,
    rust_maximize_window,
    rust_reappear_window,
    rust_shutdown_window,
    rust_dragdrop_window,
)
from time import perf_counter


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
        enable_resizable: bool = True,
        enable_devtools: bool = True,
        dwm_corner: DwmCorners = DwmCorners.SYSTEM_ROUND,
    ) -> bool:
        t_start = perf_counter()
        print(f"[Performance] Window.register_window 开始: {title}")

        if sum(bool(x) for x in (html_content, link_content, dist_content)) > 1:
            raise ValueError(
                "html_content, link_content, and dist_content cannot be used at the same time"
            )

        def pather(file_name):
            return f"{PROJECT_ROOT_PATH}/assets/{file_name}"

        result = rust_register_window(
            title=title,
            width=width,
            height=height,
            html_content=html_content,
            link_content=link_content,
            dist_content=dist_content,
            icon_path=icon_path or pather("pywebron.png"),
            show_title_bar=show_title_bar,
            enable_resizable=enable_resizable,
            enable_devtools=enable_devtools,
            dwm_corner=dwm_corner,
        )

        t_end = perf_counter()
        print(
            f"[Performance] Window.register_window 完成: {title}, 耗时: {(t_end - t_start) * 1000:.2f}ms"
        )
        return result

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
