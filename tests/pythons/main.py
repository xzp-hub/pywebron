from pywebron.configs import DwmCorners
import routers.window_control as wc
import routers.system_monitor as sm
import routers.online_chatbox as oc
import routers.quick_shortcut as qs
from pywebron import App


def main():
    app = App(prewarm_webview=False)
    # 显式注册路由
    app.router.register_routers(wc.router, sm.router, oc.router, qs.router)
    main_win = app.window.register_window(
        title="PYWEBRON测试面板",
        width=1200,
        height=1200,
        is_main=True,
        show_title_bar=False,
        enable_resizable=True,
        window_radius=6,
        dwm_corner=DwmCorners.LITTLE_ROUND,
        # link_content="http://localhost:5173/",
        # html_content=f"{PROJECT_ROOT_PATH}/assets/pywebron.html",
        dist_content=f"{PROJECT_ROOT_PATH}/forntend/dist_content/dist",
        # icon_path=f"{PROJECT_ROOT_PATH}/assets/pywebron.png",
    )
    app.window.register_windows(main_win)
    app.run()


if __name__ == "__main__":
    main()
