

# PyWebron

PyWebron 是一个基于 WebView 的现代 Python 桌面 GUI 框架，采用 Python + Rust 混合架构实现。

## 特性

- 🖥️ **跨平台窗口管理** - 支持自定义标题栏、窗口最大化、最小化、拖拽移动
- 🔄 **Invoke 机制** - Python 后端与前端的双向异步调用
- 📡 **Stream 流** - 实时数据流推送，支持多人聊天/系统监控场景
- 🧵 **Worker 线程池** - 支持 CPU 密集型任务的异步执行
- 💾 **资源缓存** - 高效的 HTML/JS/CSS 资源缓存机制
- 🎨 **现代化 UI** - 内置精致的监控仪表盘和聊天界面模板

## 技术栈

- **前端**: HTML5 + CSS3 + JavaScript (WebView)
- **Python**: asyncio, concurrent.futures
- **后端**: Rust (via PyO3)
- **WebView**: webview2 (Windows) / webkit2gtk (Linux)

## 安装

```bash
pip install pywebron
```

## 快速开始

```python
import pywebron as app

# 初始化应用
pywebron = app.App()

# 注册窗口
app.Window.register_window(
    title="PyWebron 应用",
    content_path="./dist/index.html",  # 或 content_url
    width=1200,
    height=900,
)

# 运行应用
pywebron.run()
```

## 核心模块

### Window - 窗口管理

```python
from pywebron import Window

# 创建窗口
Window.register_window(title="标题", content_path="index.html")

# 窗口控制
Window.minimize_window(window_id)
Window.maximize_window(window_id)
Window.reappear_window(window_id)
Window.shutdown_window(window_id)
```

### Invoke - 异步调用

```python
import pywebron.app as app

@app.invoke.handle("my_command")
async def my_handler(invoke: app.invoke, struct):
    await invoke.json_response(200, "成功", {"data": "..."})
```

### Stream - 数据流

```python
import pywebron.app as app

@app.stream.handle("my_stream")
async def my_stream(stream: app.stream):
    # 发送数据
    await stream.send(200, "消息", {"value": 1})
    
    # 接收数据
    data = await stream.recv()
```

### Worker - 异步任务

```python
import pywebron.app as app

@app.invoke.handle("cpu_task")
async def cpu_task(invoke: app.invoke, worker: app.worker):
    result = await worker.run(heavy_computation, arg1, arg2)
```

## 示例项目

项目目录 `tests/pythons/main.py` 包含完整的示例：

- `window_controls` - 自定义窗口拖拽区域和控制按钮
- `system_monitoring` - 实时 CPU/RAM/VRM/IO 监控
- `chat_room` - 实时聊天房间
- `cpu_intensive_task` - CPU 密集型任务演示
- `file_download` - 文件保存对话框

## 目录结构

```
pywebron/
├── pywebron/           # Python 包
│   ├── __init__.py    # 主入口
│   ├── app/           # 应用模块
│   │   ├── invoke.py  # 调用机制
│   │   ├── stream.py  # 流机制
│   │   ├── window.py  # 窗口管理
│   │   └── worker.py  # 线程池
│   ├── configs.py      # 配置
│   └── utils.py       # 工具函数
├── rusts/             # Rust 后端
│   ├── lib.rs         # 模块入口
│   ├── configs.rs    # 配置定义
│   ├── utils.rs       # 底层工具
│   └── app/           # 核心模块
│       ├── invoke.rs # 调用处理
│       ├── stream.rs # 流处理
│       └── window.rs # 窗口管理
└── assets/           # 前端资源
```

## 依赖要求

- Python >= 3.10
- Rust toolchain (用于编译)
- Windows: WebView2 Runtime
- Linux: webkit2gtk-4.1

## 许可证

MIT License