from time import perf_counter

from pywebron import App
from pywebron.configs import PROJECT_ROOT_PATH, DwmCorners
from routers import create_window_router, create_monitor_router, create_chat_router, create_shortcut_router

print(f"[Performance] ========== 应用启动开始 ==========")
t_app_start = perf_counter()

app = App(prewarm_webview=False)
t_app_init_done = perf_counter()
print(
    f"[Performance] App 初始化完成，耗时: {(t_app_init_done - t_app_start) * 1000:.2f}ms"
)

# 注册路由分组
create_window_router(app)
create_monitor_router(app)
create_chat_router(app)
create_shortcut_router(app)

if __name__ == "__main__":
    t_register_start = perf_counter()
    app.window.register_window(
        title="PYWEBRON测试面板",
        width=1200,
        height=1200,
        show_title_bar=False,
        enable_resizable=True,
        window_radius=6,
        dwm_corner=DwmCorners.LITTLE_ROUND,
        link_content="http://localhost:5173/",
        # html_content=f"{PROJECT_ROOT_PATH}/assets/pywebron.html",
        # dist_content=f"{PROJECT_ROOT_PATH}/tests/uis/vues/dist",
        # icon_path=f"{PROJECT_ROOT_PATH}/assets/pywebron.png",
    )

    t_register_done = perf_counter()
    print(
        f"窗口注册完成，耗时: {(t_register_done - t_register_start) * 1000:.2f}ms")
    print(f"从应用启动到窗口注册完成，总耗时: {(t_register_done - t_app_start) * 1000:.2f}ms")
    app.run()
