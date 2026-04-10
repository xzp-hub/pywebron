# PyWebron

基于 **Rust + WebView** 的高性能 Python 桌面 GUI 框架。

Python 编写业务逻辑，Rust 管理窗口与渲染，HTML/CSS/JS 构建界面——三者通过 IPC 机制无缝协作。

## 特性

- **高性能引擎**：Rust 核心驱动，默认 mimalloc 内存分配器，LTO + 代码裁剪优化
- **双通信模式**：Invoke（请求-响应）+ Stream（双向流式通信），支持单播/组播/广播
- **智能并发**：自动检测 GIL 状态，GIL 存在时使用多进程，否则使用多线程
- **参数自动注入**：通过函数签名分析，自动将请求参数注入到 handler
- **无边框窗口**：内置圆角、拖拽区域、调整大小手柄、自定义标题栏
- **多窗口管理**：支持运行时动态创建/关闭窗口，每个窗口独立 WebView 实例
- **前端桥接库**：自动注入 `pywebron.js`，提供 `invoke` / `stream` API
- **三种内容加载模式**：HTML 文件 / URL 链接 / 前端构建产物（自定义协议）
- **跨平台**：Windows（WebView2）+ Linux（WebKitGTK + GTK）

## 架构

```
┌─────────────────────────────────┐
│   前端 (HTML/CSS/JS)             │  pywebron.js 提供 IPC 桥接
│   pywebron.interfaces.invoke()   │  pywebron.interfaces.stream()
└──────────────┬──────────────────┘
               │ IPC (wry WebView postMessage)
               ▼
┌─────────────────────────────────┐
│   Rust 中间层 (核心引擎)          │  TAO 窗口管理 + WRY WebView
│   IPC 消息分发 / 线程池 / 订阅管理 │  Invoke 线程池 + Stream 线程池
└──────────────┬──────────────────┘
               │ PyO3 (Python GIL 管理)
               ▼
┌─────────────────────────────────┐
│   Python 业务层                  │  App / Invoke / Stream / Worker
│   装饰器注册 / 参数自动注入 / async │  GIL 感知的 Worker 线程池
└─────────────────────────────────┘
```

## 安装

### 快速安装（pip）

```bash
pip install pywebron
```

> **运行时依赖**：Windows 需安装 [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)（Windows 10/11 已内置），Linux 需安装 WebKitGTK + GTK 开发库。

### 从源码构建（自定义开发）

适用于需要修改框架源码或本地调试的场景。

**环境要求**：

- Python >= 3.11
- Rust 工具链（[rustup](https://rustup.rs/)）
- [maturin](https://github.com/PyO3/maturin) 构建工具

```bash
# 克隆仓库
git clone https://github.com/xzp-hub/pywebron.git
cd pywebron

# 创建虚拟环境（推荐）
python -m venv .venv
.venv\Scripts\activate   # Windows
# source .venv/bin/activate  # Linux

# 安装 maturin
pip install maturin

# 开发模式安装（编译 Rust 扩展 + 安装 Python 包）
maturin develop

# 生产模式安装（启用 LTO 和优化）
maturin develop --release
```

## 快速开始

### 最小示例

```python
from pywebron import App

app = App()

@app.invoke.handle("greet")
async def greet(invoke: app.invoke):
    return await invoke.json_response(200, "Hello!", {"message": "Hello, PyWebron!"})

app.window.register_window(
    title="My First App",
    width=800,
    height=600,
)

app.run()
```

### 完整示例

```python
from pywebron import App, StreamSendModes
from pywebron.configs import DwmCorners
from pywebron.utils import save_file_dialog

app = App()

# ---- 自定义参数结构体 ----
class MyStruct(app.invoke.struct):
    name: str
    age: int = 0

# ---- Invoke: 请求-响应模式 ----
@app.invoke.handle("get_user_info")
async def get_user_info(invoke: app.invoke, struct: MyStruct):
    return await invoke.json_response(
        200, "查询成功", {"name": struct.name, "age": struct.age}
    )

# ---- Stream: 双向流式通信 ----
@app.stream.handle("chat_stream")
async def chat_stream(stream: app.stream):
    # 广播欢迎消息
    await stream.send(200, "欢迎!", {"type": "system"})

    while True:
        match res := await stream.recv():
            case None:
                await asyncio.sleep(0.1)
            case _:
                # 单播回复
                await stream.send(
                    200, f"收到: {res}", {"type": "chat"},
                    send_mode=StreamSendModes.UNITYCAST,
                )

# ---- Worker: CPU 密集任务 ----
@app.invoke.handle("heavy_task")
async def heavy_task(invoke: app.invoke, worker: app.worker):
    result = await worker.run(my_cpu_func, arg1, arg2)
    return await invoke.json_response(200, "完成", result)

# ---- 注册窗口并运行 ----
app.window.register_window(
    title="PyWebron App",
    width=1200,
    height=900,
    show_title_bar=False,       # 无边框窗口
    window_radius=5,            # 圆角半径
    enable_resizable=True,      # 可调整大小
    dwm_corner=DwmCorners.LITTLE_ROUND,  # Windows DWM 圆角
    # 三种内容模式（三选一）：
    # html_content="/path/to/index.html",        # 加载 HTML 文件
    # link_content="http://localhost:5173/",      # 加载 URL
    # dist_content="/path/to/dist",               # 加载前端构建产物
)

app.run()
```

## API 参考

### App

应用入口类，负责初始化 Rust 引擎和启动事件循环。

```python
from pywebron import App

app = App(prewarm_webview=False)  # prewarm_webview: 是否预热 WebView2 引擎（仅 Windows）
app.run()                          # 启动事件循环（阻塞主线程）
app.get_windows()                  # 获取所有窗口信息 Dict[int, Dict]
app.get_handles()                  # 获取所有已注册的 handler Dict[str, Dict]
```

### Window

窗口管理，支持多窗口和运行时动态创建。

| 方法 | 说明 | 返回值 |
|---|---|---|
| `register_window(...)` | 注册新窗口 | `bool` |
| `minimize_window(window_id)` | 最小化窗口 | `bool` |
| `maximize_window(window_id)` | 最大化窗口 | `bool` |
| `reappear_window(window_id)` | 还原窗口 | `bool` |
| `shutdown_window(window_id)` | 关闭窗口 | `bool` |
| `dragdrop_window(window_id, selector)` | 设置拖拽区域 | `bool` |

#### register_window 参数

| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `title` | `str` | `"PyWebron App"` | 窗口标题 |
| `html_content` | `str` | `None` | HTML 文件路径 |
| `link_content` | `str` | `None` | URL 地址（如 Vite 开发服务器） |
| `dist_content` | `str` | `None` | 前端构建产物目录路径 |
| `width` | `int` | `1200` | 窗口宽度 |
| `height` | `int` | `900` | 窗口高度 |
| `icon_path` | `str` | `None` | 窗口图标路径 |
| `show_title_bar` | `bool` | `True` | 是否显示系统标题栏 |
| `window_radius` | `int` | `5` | 无边框窗口圆角半径 |
| `enable_resizable` | `bool` | `True` | 是否允许调整窗口大小 |
| `enable_devtools` | `bool` | `True` | 是否启用开发者工具 |
| `dwm_corner` | `DwmCorners` | `SYSTEM_ROUND` | Windows DWM 窗口圆角偏好 |

> **注意**：`html_content`、`link_content`、`dist_content` 三者只能选其一，不指定则使用内置默认页面。

### Invoke

请求-响应模式，前端调用 Python 函数并等待返回结果。

```python
# 注册 handler
@app.invoke.handle("handle_name")
async def my_handler(invoke: app.invoke, struct: MyStruct, worker: app.worker):
    return await invoke.json_response(code, message, data)
```

**参数自动注入**：

| 参数类型注解 | 注入内容 |
|---|---|
| `app.invoke` | Invoke 实例（含 `window_id`、`handle_id`） |
| `app.worker` | Worker 类（可调用 `worker.run()`） |
| 自定义 Struct | 自动从 `payload` 中提取对应字段 |
| 普通参数 | 直接从 `payload` 中取值 |

### Stream

双向流式通信，Python 和前端可持续互发数据。

```python
@app.stream.handle("stream_name")
async def my_stream(stream: app.stream, worker: app.worker, struct: MyStruct):
    # 发送数据
    await stream.send(code, message, data, send_mode=..., mcast_win_ids=...)
    # 接收数据（返回 None 表示无数据）
    result = await stream.recv()
```

**StreamSendModes 发送模式**：

| 模式 | 值 | 说明 |
|---|---|---|
| `UNITYCAST` | `"unitycast"` | 单播：仅发送到当前窗口 |
| `MULTICAST` | `"multicast"` | 组播：发送到 `mcast_win_ids` 指定的窗口 |
| `BROADCAST` | `"broadcast"` | 广播：发送到所有订阅窗口 |

### Worker

CPU 密集任务执行器，自动选择最优并发策略。

```python
result = await worker.run(func, *args, **kwargs)
```

- GIL 存在时（CPython < 3.13 或 GIL 未禁用）：使用 `ProcessPoolExecutor`
- GIL 不存在时（Free-threaded Python 3.13+）：使用 `ThreadPoolExecutor`

### DwmCorners

Windows DWM 窗口圆角偏好枚举。

| 值 | 说明 |
|---|---|
| `SYSTEM_ROUND` (0) | 系统默认圆角 |
| `ZEROES_ROUND` (1) | 不圆角（直角） |
| `NORMAL_ROUND` (2) | 正常圆角 |
| `LITTLE_ROUND` (3) | 小圆角 |

### 工具函数

```python
from pywebron.utils import save_file_dialog, get_gil_status

# 保存文件对话框（异步）
new_path = await save_file_dialog(
    source_file_path="/path/to/source.txt",
    new_file_name="output.txt",        # 可选，默认使用源文件名
    is_del_source_file=False,           # 可选，保存后是否删除源文件
)

# 检测 GIL 状态
has_gil = get_gil_status()  # True / False
```

## 前端 API

框架自动向每个 WebView 注入 `pywebron.js`，提供以下接口：

### invoke — 请求-响应

```javascript
// 调用 Python handler，返回 Promise
const response = await window.pywebron.interfaces.invoke(
    'handle_name',   // handler 名称
    { key: 'value' }, // payload 对象
    5000              // 超时时间（毫秒），可选
);
// response: { window_id, handle_id, payload: { code, mssg, data } }
```

### stream — 双向流

```javascript
const stream = await window.pywebron.interfaces.stream('stream_name', { key: 'value' });

// 接收数据
stream.recv(data => {
    console.log('收到:', data);
});

// 发送数据
stream.send('hello from frontend');

// 关闭流
stream.close();
```

### 全局属性

```javascript
// 窗口配置属性（只读）
window.pywebron.attributes.window_id       // 窗口 ID
window.pywebron.attributes.show_title_bar   // 是否显示标题栏
window.pywebron.attributes.window_radius    // 圆角半径
window.pywebron.attributes.enable_resizable // 是否可调整大小
```

## 内容加载模式

### 1. HTML 文件模式

直接加载 HTML 文件，适合简单页面：

```python
app.window.register_window(
    html_content="/path/to/index.html",
)
```

### 2. URL 链接模式

加载 URL，适合 Vite 等开发服务器热重载开发：

```python
app.window.register_window(
    link_content="http://localhost:5173/",
)
```

### 3. 构建产物模式

通过自定义 `app://` 协议加载前端构建产物，适合生产部署：

```python
app.window.register_window(
    dist_content="/path/to/vue-app/dist",
)
```

前端代码中直接使用 `app://` 协议引用资源：

```html
<link rel="stylesheet" href="app://assets/index.css">
<script src="app://assets/index.js"></script>
```

## 无边框窗口

设置 `show_title_bar=False` 即可创建无边框窗口，框架自动处理：

- **圆角渲染**：通过 `window_radius` 控制圆角半径
- **拖拽区域**：调用 `dragdrop_window(window_id, selector)` 设置可拖拽区域
- **调整大小**：`enable_resizable=True` 时自动注入调整大小手柄
- **自定义标题栏**：内置默认标题栏 HTML 模板（`assets/pywebron.html`）

### 自定义标题栏示例

```html
<div class="header" id="windowHeader">
    <div class="h-left">
        <svg class="h-icon">...</svg>
        <span class="h-title">My App</span>
    </div>
    <div class="h-ctrls">
        <div class="win-btn" onclick="windowAction('min')">─</div>
        <div class="win-btn" onclick="windowAction('toggle')">□</div>
        <div class="win-btn close" onclick="windowAction('shut')">✕</div>
    </div>
</div>
```

```javascript
async function windowAction(type) {
    const map = {
        min: 'minimize_window',
        max: 'maximize_window',
        rep: 'reappear_window',
        shut: 'shutdown_window'
    };
    const action = type === 'toggle' ? (isMaximized ? 'rep' : 'max') : type;
    await window.pywebron.interfaces.invoke('window_controls_invoke', {
        control_type: map[action]
    }, 5000);
}
```

## IPC 通信机制

PyWebron 的核心是 Rust 层的 IPC 消息分发系统，包含两个独立的线程池：

### Invoke 线程池

- 线程数：自动检测 CPU 核心数（最少 2 个）
- 处理流程：前端请求 → Rust 接收 → 获取 Python GIL → 执行 handler → 返回结果
- 每个请求独立处理，互不阻塞
- 内置 Handler 缓存，避免重复查找

### Stream 线程池

- 线程数：8 个常驻线程
- 每个 stream handler 通常为无限循环（持续收发数据）
- 独立于 invoke 线程池，避免流式通信阻塞请求-响应
- 支持订阅管理：一个 stream handler 可被多个窗口同时订阅
- 有界消息队列（100 条/窗口），防止内存溢出

### 数据流转路径

```
前端 invoke() → wry postMessage → Rust IPC 线程池 → Python GIL → handler 执行
                                                               ↓
前端 ← wry evaluateScript ← Rust 事件循环 ← json_response()
```

## 项目结构

```
pywebron/
├── assets/                     # 内置资源
│   ├── pywebron.html           # 默认窗口页面模板
│   ├── pywebron.js             # 前端 IPC 桥接库
│   └── pywebron.png            # 默认窗口图标
├── pywebron/                   # Python 模块
│   ├── __init__.py             # App 入口类
│   ├── configs.py              # 全局注册表 & 枚举定义
│   ├── utils.py                # GIL 检测、文件对话框
│   └── app/
│       ├── handle.py           # Handle 基类（参数自动注入）
│       ├── invoke.py           # Invoke 处理器
│       ├── stream.py           # Stream 处理器
│       ├── window.py           # 窗口管理 API
│       └── worker.py           # Worker 线程/进程池
├── rusts/                      # Rust 源码
│   ├── lib.rs                  # PyO3 模块入口（14 个导出函数）
│   ├── configs.rs              # 事件类型枚举、DWM 圆角枚举
│   ├── utils.rs                # DPI、窗口 ID、图标、圆角、对话框
│   └── app/
│       ├── window.rs           # 窗口创建、事件循环、WebView 管理
│       ├── invoke.rs           # IPC 线程池、invoke 请求分发
│       └── stream.rs           # Stream 订阅管理、数据队列
├── tests/                      # 测试应用
│   ├── pythons/main.py         # 完整功能演示
│   └── uis/                    # 前端测试页面
├── Cargo.toml                  # Rust 依赖配置
├── pyproject.toml              # Python 项目配置
└── README.md
```

## Rust 核心依赖

| 依赖 | 用途 |
|---|---|
| [tao](https://github.com/tauri-apps/tao) | 跨平台窗口管理 |
| [wry](https://github.com/tauri-apps/wry) | 跨平台 WebView |
| [pyo3](https://github.com/PyO3/pyo3) | Python ↔ Rust 绑定 |
| [tokio](https://tokio.rs/) | 异步运行时 |
| [dashmap](https://github.com/xacrimon/dashmap) | 并发 HashMap |
| [parking_lot](https://github.com/Amanieu/parking_lot) | 高性能锁 |
| [crossbeam](https://github.com/crossbeam-rs/crossbeam) | 无锁队列 |
| [mimalloc](https://github.com/purpleprotocol/mimalloc_rust) | 高性能内存分配器 |
| [rfd](https://github.com/PolyMeilex/rfd) | 原生文件对话框 |

## 构建 & 发布

```bash
# 开发模式（快速编译，无优化）
maturin develop

# 发布模式（LTO + 优化，编译较慢但运行更快）
maturin develop --release

# 构建 wheel 包
maturin build --release

# 发布到 PyPI
maturin publish
```

## License

MIT License
