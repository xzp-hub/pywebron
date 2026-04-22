from types import SimpleNamespace
from typing import List
from .handler import Handle, Invoke, Stream


class Router:
    def __init__(self, title: str = ""):
        self.title = title
        self.handlers: List[tuple] = []
        self.invoke = SimpleNamespace(
            server=Invoke,
            struct=Handle.Struct,
            handle=lambda a=None: lambda f: (self.handlers.append((a or f.__name__, f, 'invoke')), f)[1]
        )
        self.stream = SimpleNamespace(
            server=Stream,
            struct=Handle.Struct,
            handle=lambda a=None: lambda f: (self.handlers.append((a or f.__name__, f, 'stream')), f)[1]
        )

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
                wrapper = Handle._create_wrapper_(handler_class, func)
                handlers_to_register.append({'name': name, 'type': htype, 'handler': wrapper})

            HANDLES.setdefault(router.title, []).extend(handlers_to_register)
            print(f"[Router] ✓ '{router.title}': {len(router.handlers)} 个处理器")

        print(f"[DEBUG] ========== 注册完成 ==========\n")
        print(f"[DEBUG] HANDLES 总览:")
        for group, handlers in HANDLES.items():
            print(f"[DEBUG]   [{group}]: {len(handlers)} 个处理器")
            for h in handlers:
                print(f"[DEBUG]     - {h['type']}: {h['name']}")
