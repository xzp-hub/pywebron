from types import SimpleNamespace
from typing import Callable, Any, List
from inspect import Parameter, signature
from .worker import Worker
from .window import Window


Struct = SimpleNamespace


class Handle:
    struct = Struct
    __slots__ = ("handle_id", "window_id")

    def __init__(self, handle_id: str, window_id: int):
        self.handle_id = handle_id
        self.window_id = window_id

    def _logger_(self, payload: dict, send_mode: str = None):
        header = f"[{self.__class__.__name__}]-[{self.window_id}]-[{self.handle_id}]"
        print(f"{header}-[{send_mode}]: {payload}" if send_mode else f"{header}: {payload}")


class Invoke(Handle):
    async def json_response(self, stat: bool, mssg: str, data: Any = None):
        self._logger_(payload := {'stat': stat, 'mssg': mssg, 'data': data})
        return {'window_id': self.window_id, 'handle_id': self.handle_id, 'payload': payload}


class Stream(Handle):
    async def send(self, code: int, mssg: str, data: Any, send_mode: str = "broadcast", mcast_win_ids: list[int] = None) -> bool:
        from .._pywebron_ import rust_stream_send
        payload, window_ids = {"code": code, "mssg": mssg, "data": data}, None
        self._logger_(payload, send_mode)
        window_ids = None if send_mode == "broadcast" else (mcast_win_ids if send_mode == "multicast" else [self.window_id])
        return await rust_stream_send(payload=payload, handle_id=self.handle_id, send_mode=send_mode, window_ids=window_ids)

    async def recv(self) -> Any:
        from .._pywebron_ import rust_stream_recv
        res = await rust_stream_recv(self.handle_id)
        return res["payload"] if res else None


class Router:
    def __init__(self, title: str = ""):
        self.title = title
        self.handlers: List[tuple] = []
        self.invoke = SimpleNamespace(
            server=Invoke,
            struct=Struct,
            handle=lambda a=None: lambda f: (self.handlers.append((a or f.__name__, f, 'invoke')), f)[1]
        )
        self.stream = SimpleNamespace(
            server=Stream,
            struct=Struct,
            handle=lambda a=None: lambda f: (self.handlers.append((a or f.__name__, f, 'stream')), f)[1]
        )
    
    @staticmethod
    def _create_wrapper_(handler_class: type, func: Callable):
        params = signature(func).parameters
        print(f"[DEBUG] 创建 wrapper for {func.__name__}, handler_class={handler_class.__name__}")
        print(f"[DEBUG] 参数列表: {list(params.keys())}")
        
        def maker(param_name):
            p = params[param_name]
            annot, default = p.annotation, p.default
            
            print(f"[DEBUG]   参数 '{param_name}': annotation={annot}, default={default}")
            
            # 如果没有类型注解，跳过
            if annot is Parameter.empty:
                print(f"[DEBUG]   -> 无类型注解，从 payload 获取")
                return lambda req: (param_name, req['payload'].get(param_name, default) if default is not Parameter.empty else req['payload'][param_name])
            
            type_name = getattr(annot, '__name__', None)
            print(f"[DEBUG]   -> type_name={type_name}")
            
            if type_name in ('Invoke', 'Stream'):
                print(f"[DEBUG]   -> 创建 {type_name} 实例")
                return lambda req: (param_name, handler_class(req['handle_id'], req['window_id']))
            elif type_name == 'Worker':
                print(f"[DEBUG]   -> 返回 Worker 类")
                return lambda req: (param_name, Worker)
            elif type_name == 'Window':
                print(f"[DEBUG]   -> 返回 Window 类")
                return lambda req: (param_name, Window)
            elif hasattr(annot, '__annotations__'):
                print(f"[DEBUG]   -> 创建结构体实例")
                return lambda req: (param_name, annot(**{k: req['payload'].get(k, getattr(annot, k, None)) for k in annot.__annotations__}))
            else:
                print(f"[DEBUG]   -> 从 payload 获取")
                return lambda req: (param_name, req['payload'].get(param_name, default) if default is not Parameter.empty else req['payload'][param_name])
        
        handles = [maker(p) for p in params]
        print(f"[DEBUG] Wrapper 创建完成，共 {len(handles)} 个参数处理器\n")
        
        async def wrapper(req: dict):
            print(f"[DEBUG] 调用 wrapper: func={func.__name__}, handle_id={req.get('handle_id')}, window_id={req.get('window_id')}")
            print(f"[DEBUG] 请求 payload: {req.get('payload')}")
            try:
                kwargs = dict(h(req) for h in handles)
                print(f"[DEBUG] 解析后的参数: {kwargs}")
                print(f"[DEBUG] 开始调用 handler 函数...")
                result = await func(**kwargs)
                print(f"[DEBUG] 执行成功，返回: {result} (type={type(result)})")
                # Stream handler 不应返回值（它们是无限循环）
                if handler_class == Stream:
                    print(f"[DEBUG] Stream handler 返回 None")
                    return None
                print(f"[DEBUG] Invoke handler 返回结果")
                return result
            except Exception as e:
                print(f"[ERROR] 执行失败: {type(e).__name__}: {e}")
                import traceback
                traceback.print_exc()
                raise
        
        return wrapper
    
    @classmethod
    def register_routers(cls, *routers: 'Router'):
        from ..configs import HANDLES
        print(f"\n[DEBUG] ========== 开始注册路由 ==========")
        for router in routers:
            print(f"[DEBUG] 注册路由组: '{router.title}'")
            print(f"[DEBUG] 处理器数量: {len(router.handlers)}")
            
            handlers_to_register = []
            for name, func, htype in router.handlers:
                print(f"[DEBUG]   - {htype}: {name} (func={func.__name__})")
                handler_class = Invoke if htype == 'invoke' else Stream
                wrapper = cls._create_wrapper_(handler_class, func)
                handlers_to_register.append({'name': name, 'type': htype, 'handler': wrapper})
            
            HANDLES.setdefault(router.title, []).extend(handlers_to_register)
            print(f"[Router] ✓ '{router.title}': {len(router.handlers)} 个处理器")
        
        print(f"[DEBUG] ========== 注册完成 ==========\n")
        print(f"[DEBUG] HANDLES 总览:")
        for group, handlers in HANDLES.items():
            print(f"[DEBUG]   [{group}]: {len(handlers)} 个处理器")
            for h in handlers:
                print(f"[DEBUG]     - {h['type']}: {h['name']}")
