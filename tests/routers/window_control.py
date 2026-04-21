from traceback import format_exc

from pywebron import Router, Window

router = Router(title="窗口管理")
invoke = router.invoke


class WindowControlsStruct(invoke.struct):
    control_type: str


class SetupDragRegionStruct(invoke.struct):
    selector: str = ".header"


@invoke.handle("setup_drag_region_invoke")
async def setup_drag_region(server: invoke.server, window: Window, struct: SetupDragRegionStruct):
    try:
        selector = struct.selector
        res = window.dragdrop_window(server.window_id, selector)
        return await server.json_response(True, "拖拽区域设置成功", {"selector": selector, "result": res})
    except Exception:
        return await server.json_response(False, "拖拽区域设置失败", format_exc())


@invoke.handle("window_controls_invoke")
async def window_controls(server: invoke.server, window: Window, struct: WindowControlsStruct):
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
