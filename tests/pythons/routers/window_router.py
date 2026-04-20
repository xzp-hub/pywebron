from traceback import format_exc

from pywebron import Router, Window
from pywebron.configs import PROJECT_ROOT_PATH

router = Router()


class WindowControlsStruct(router.invoke.struct):
    control_type: str


class SetupDragRegionStruct(router.invoke.struct):
    selector: str = ".header"


@router.invoke.handle("setup_drag_region_invoke")
async def setup_drag_region(invoke: router.invoke.server, struct: SetupDragRegionStruct):
    try:
        selector = struct.selector
        res = Window.dragdrop_window(invoke.window_id, selector)
        return await invoke.json_response(True, "拖拽区域设置成功", {"selector": selector, "result": res})
    except Exception:
        return await invoke.json_response(False, "拖拽区域设置失败", format_exc())


@router.invoke.handle("window_controls_invoke")
async def window_controls(invoke: router.invoke.server, struct: WindowControlsStruct):
    control_type, res = None, None
    try:
        match control_type := struct.control_type:
            case "minimize_window":
                res = Window.minimize_window(invoke.window_id)
            case "maximize_window":
                res = Window.maximize_window(invoke.window_id)
            case "reappear_window":
                res = Window.reappear_window(invoke.window_id)
            case "shutdown_window":
                res = Window.shutdown_window(invoke.window_id)
        return await invoke.json_response(True, f"{control_type} 操作成功", res)
    except Exception:
        return await invoke.json_response(False, f"{control_type} 操作失败", format_exc())


@router.invoke.handle("create_new_windows_at_runtime_invoke")
async def create_new_windows_at_runtime(invoke: router.invoke.server):
    try:
        res = Window.register_window(
            title="运行时创建窗口",
            width=1200,
            height=1200,
            show_title_bar=False,
            dist_content=f"{PROJECT_ROOT_PATH}/tests/uis/vues/dist",
        )
        return await invoke.json_response(True, f"运行时创建窗口成功：{res}", res)
    except Exception:
        return await invoke.json_response(False, "运行时创建窗口失败", format_exc())


def create_window_router():
    """窗口控制分组：拖拽设置、窗口控制、运行时创建窗口"""
    return router
