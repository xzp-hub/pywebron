from ..configs import ASSETS_SRC_DIR
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
            title: str = 'PyWebron App',
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
            raise 'content_path and content_url cannot be used at the same time'
        content = content_path or content_url or f'{ASSETS_SRC_DIR}/index.html'
        icon = icon_path or f'{ASSETS_SRC_DIR}/index.png'
        print(f'[Window] 注册窗口 - content: {content}')
        print(f'[Window] 注册窗口 - icon: {icon}')
        return rust_register_window(
            title=title,
            width=width,
            height=height,
            content=content,
            icon_path=icon,
            decorations=show_title_bar,
            resizable=enable_resizable,
            devtools=enable_devtools,
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
