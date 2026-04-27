from pywebron.configs import DwmCorners, PROJECT_ROOT_PATH
import routers.system_monitor as sm
import routers.online_chatbox as oc
import routers.quick_shortcut as qs
from pywebron import App


def main():
    print("[DEBUG] 开始创建 App...")
    app = App(prewarm_webview=False)
    print("[DEBUG] App 创建成功")

    print("[DEBUG] 注册路由...")
    app.router.register_routers(sm.router, oc.router, qs.router)
    print("[DEBUG] 路由注册成功")
    
    print("[DEBUG] 注册窗口...")
    main_win = app.window.register_window(
        title="PYWEBRON测试面板",
        width=1200,
        height=1200,
        show_title_bar=False,
        enable_resizable=True,
        window_radius=6,
        is_main=True,
        dwm_corner=DwmCorners.LITTLE_ROUND,
        # link_content="http://localhost:5173/",
        # html_content=f"{PROJECT_ROOT_PATH}/frontend/html_content/pywebron.html",
        dist_content=f"{PROJECT_ROOT_PATH}/frontend/dist_content/dist",
        icon_path=f"{PROJECT_ROOT_PATH}/frontend/test.jpg",
    )
    print(f"[DEBUG] 窗口注册成功，ID: {main_win}")
    
    print("[DEBUG] 添加窗口到应用...")
    app.window.register_windows(main_win)  # pyright: ignore[reportUnknownMemberType]
    print("[DEBUG] 窗口添加成功，准备运行...")
    
    print("[DEBUG] 启动应用...")
    app.run()


if __name__ == "__main__":
    main()
