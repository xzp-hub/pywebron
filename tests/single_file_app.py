from pywebron.configs import DwmCorners, PROJECT_ROOT_PATH
from pywebron import App

app = App(prewarm_webview=False)


def main():
    main_win = app.window.register_window(
        title="PYWEBRON测试面板",
        width=1200,
        height=1200,
        show_title_bar=False,
        enable_resizable=True,
        window_radius=6,
        dwm_corner=DwmCorners.LITTLE_ROUND,
        # link_content="http://localhost:5173/",
        # html_content=f"{PROJECT_ROOT_PATH}/frontend/html_content/pywebron.html",
        # dist_content=f"{PROJECT_ROOT_PATH}/tests/uis/dist_content/dist",
        icon_path=f"{PROJECT_ROOT_PATH}/frontend/test.jpg",
    )
    app.window.register_windows(main_win)
    app.run()


if __name__ == "__main__":
    main()
