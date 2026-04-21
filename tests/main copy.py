from traceback import format_exc
from pywebron.configs import DwmCorners, PROJECT_ROOT_PATH
from pywebron import App
from pywebron.app.window import Window

app = App(prewarm_webview=False)


class WindowControlsStruct(app.invoke.struct):
    control_type: str


class SetupDragRegionStruct(app.invoke.struct):
    selector: str = ".header"


@app.invoke.handle("setup_drag_region_invoke")
async def setup_drag_region(server: app.invoke.server, window: Window, struct: SetupDragRegionStruct):
    try:
        selector = struct.selector
        res = window.dragdrop_window(server.window_id, selector)
        return await server.json_response(True, "拖拽区域设置成功", {"selector": selector, "result": res})
    except Exception:
        return await server.json_response(False, "拖拽区域设置失败", format_exc())


@app.invoke.handle("window_controls_invoke")
async def window_controls(server: app.invoke.server, window: Window, struct: WindowControlsStruct):
    control_type, res = None, None
    try:
        match control_type := struct.control_type:
            case "minimize_window":
                res = window.minimize_window(server.window_id)
            case "maximize_window":
                res = window.maximize_window(server.window_id)
            case "reappear_window":
                res = window.reappear_window(server.window_id)
            case "shutdown_window":
                res = window.shutdown_window(server.window_id)
        return await server.json_response(True, f"{control_type} 操作成功", res)
    except Exception:
        return await server.json_response(False, f"{control_type} 操作失败", format_exc())


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
        html_content=f"{PROJECT_ROOT_PATH}/frontend/html_content/pywebron.html",
        # dist_content=f"{PROJECT_ROOT_PATH}/tests/uis/dist_content/dist",
        # icon_path=f"{PROJECT_ROOT_PATH}/builtins/pywebron.png",
    )
    app.window.register_windows(main_win)
    app.run()


if __name__ == "__main__":
    main()
