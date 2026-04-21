from traceback import format_exc

from pywebron import Router
from pywebron.configs import PROJECT_ROOT_PATH

router = Router(title="窗口管理")


class WindowControlsStruct(router.invoke.struct):
    control_type: str


class SetupDragRegionStruct(router.invoke.struct):
    selector: str = ".header"


@router.invoke.handle("setup_drag_region_invoke")
async def setup_drag_region(invoke: router.invoke.server, window: router.invoke.window, struct: SetupDragRegionStruct):
    try:
        selector = struct.selector
        res = window.dragdrop_window(invoke.window_id, selector)
        return await invoke.json_response(True, "拖拽区域设置成功", {"selector": selector, "result": res})
    except Exception:
        return await invoke.json_response(False, "拖拽区域设置失败", format_exc())


@router.invoke.handle("window_controls_invoke")
async def window_controls(invoke: router.invoke.server, window:router.invoke.window, struct: WindowControlsStruct):
    control_type, res = None, None
    try:
        match control_type := struct.control_type:
            case "minimize_window":
                res = window.minimize_window(invoke.window_id)
            case "maximize_window":
                res = window.maximize_window(invoke.window_id)
            case "reappear_window":
                res = window.reappear_window(invoke.window_id)
            case "shutdown_window":
                res = window.shutdown_window(invoke.window_id)
        return await invoke.json_response(True, f"{control_type} 操作成功", res)
    except Exception:
        return await invoke.json_response(False, f"{control_type} 操作失败", format_exc())
