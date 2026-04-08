# PyWebron

PyWebron is a modern Python desktop GUI framework based on WebView, implemented with a Python + Rust hybrid architecture.

## Features

- 🖥️ **Cross-platform Window Management** - Supports custom title bars, window maximize/minimize, and drag-to-move
- 🔄 **Invoke Mechanism** - Bidirectional asynchronous calls between Python backend and frontend
- 📡 **Stream Streaming** - Real-time data stream push, suitable for multi-user chat/system monitoring scenarios
- 🧵 **Worker Thread Pool** - Supports asynchronous execution of CPU-intensive tasks
- 💾 **Resource Caching** - Efficient caching mechanism for HTML/JS/CSS resources
- 🎨 **Modern UI** - Built-in elegant dashboard and chat interface templates

## Technology Stack

- **Frontend**: HTML5 + CSS3 + JavaScript (WebView)
- **Python**: asyncio, concurrent.futures
- **Backend**: Rust (via PyO3)
- **WebView**: webview2 (Windows) / webkit2gtk (Linux)

## Installation

```bash
pip install pywebron
```

## Quick Start

```python
import pywebron as app

# Initialize the application
pywebron = app.App()

# Register a window
app.Window.register_window(
    title="PyWebron App",
    content_path="./dist/index.html",  # or content_url
    width=1200,
    height=900,
)

# Run the application
pywebron.run()
```

## Core Modules

### Window - Window Management

```python
from pywebron import Window

# Create a window
Window.register_window(title="Title", content_path="index.html")

# Window control
Window.minimize_window(window_id)
Window.maximize_window(window_id)
Window.reappear_window(window_id)
Window.shutdown_window(window_id)
```

### Invoke - Asynchronous Invocation

```python
import pywebron.app as app

@app.invoke.handle("my_command")
async def my_handler(invoke: app.invoke, struct):
    await invoke.json_response(200, "Success", {"data": "..."})
```

### Stream - Data Streaming

```python
import pywebron.app as app

@app.stream.handle("my_stream")
async def my_stream(stream: app.stream):
    # Send data
    await stream.send(200, "Message", {"value": 1})
    
    # Receive data
    data = await stream.recv()
```

### Worker - Asynchronous Tasks

```python
import pywebron.app as app

@app.invoke.handle("cpu_task")
async def cpu_task(invoke: app.invoke, worker: app.worker):
    result = await worker.run(heavy_computation, arg1, arg2)
```

## Example Projects

The project directory `tests/pythons/main.py` contains complete examples:

- `window_controls` - Custom window drag regions and control buttons
- `system_monitoring` - Real-time CPU/RAM/VRM/IO monitoring
- `chat_room` - Real-time chat room
- `cpu_intensive_task` - CPU-intensive task demonstration
- `file_download` - File save dialog

## Directory Structure

```
pywebron/
├── pywebron/           # Python package
│   ├── __init__.py    # Main entry
│   ├── app/           # Application modules
│   │   ├── invoke.py  # Invoke mechanism
│   │   ├── stream.py  # Stream mechanism
│   │   ├── window.py  # Window management
│   │   └── worker.py  # Thread pool
│   ├── configs.py      # Configuration
│   └── utils.py       # Utility functions
├── rusts/             # Rust backend
│   ├── lib.rs         # Module entry
│   ├── configs.rs    # Configuration definitions
│   ├── utils.rs       # Low-level utilities
│   └── app/           # Core modules
│       ├── invoke.rs # Invoke handling
│       ├── stream.rs # Stream handling
│       └── window.rs # Window management
└── assets/           # Frontend assets
```

## Dependencies

- Python >= 3.10
- Rust toolchain (required for compilation)
- Windows: WebView2 Runtime
- Linux: webkit2gtk-4.1

## License

MIT License