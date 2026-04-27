# PyWebron CLI 方案

## 概述

为 PyWebron 添加 `pywebron build` CLI 命令，将用户的 PyWebron 应用打包为可分发的桌面应用。

## CLI 命令

### `pywebron build`

将 PyWebron 项目打包为可执行文件。

```bash
# 单文件模式（默认）— 输出单个 exe，免安装，适合轻量工具分发
pywebron build

# standalone 模式 — 输出文件夹（exe + 依赖），启动更快
pywebron build --standalone

# installer 模式 — 输出安装包（setup.exe），带安装界面，适合正式产品分发
pywebron build --installer
```

#### 参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `entry` | 入口文件路径 | `app.py` |
| `--standalone` | 输出为文件夹而非单文件 | `False` |
| `--installer` | 输出为安装包（基于 Inno Setup） | `False` |
| `--name` | 应用名称 | 项目目录名 |
| `--icon` | 应用图标路径 | 无 |
| `--output` | 输出目录 | `./dist` |
| `--extra-data` | 额外需要打包的资源文件/目录 | 无 |
| `--no-confirm` | 跳过确认直接打包 | `False` |

#### 使用示例

```bash
# 最简单的用法
pywebron build app.py

# 指定名称和图标
pywebron build app.py --name "MyApp" --icon ./static/icon.ico

# 打包额外资源（前端 dist 目录、图片等）
pywebron build app.py --extra-data ./frontend/dist --extra-data ./static/logo.png

# 生成安装包
pywebron build app.py --installer --name "MyApp" --icon ./static/icon.ico
```

## 三种打包模式对比

| | onefile（默认） | standalone | installer |
|---|---|---|---|
| 产物 | 单个 exe | 文件夹（exe + dll） | setup.exe 安装包 |
| 启动速度 | 首次稍慢（需解压） | 快 | 快 |
| 分发方式 | 直接发文件 | 压缩后发 | 发安装包 |
| 卸载 | 删文件 | 删文件夹 | 控制面板卸载 |
| 桌面快捷方式 | 无 | 无 | 有 |
| 适合场景 | 内部工具、小工具 | 需要快速启动的应用 | 面向外部用户的正式产品 |

## 技术实现

### 底层工具：Nuitka

选择 Nuitka 而非 PyInstaller，原因：
- Nuitka 将 Python 编译为 C 再编译为机器码，运行性能更好
- 产物体积更小
- 对 pyd 扩展（`_pywebron_`）支持良好

### 打包流程

```
pywebron build app.py
    │
    ├─ 1. 解析参数，确定打包模式
    │
    ├─ 2. 检查依赖（Nuitka、C 编译器；installer 模式还需 Inno Setup）
    │
    ├─ 3. 扫描项目
    │     ├─ 入口文件及其 import 的所有 .py 文件
    │     ├─ --extra-data 指定的资源文件/目录
    │     └─ venv 中的依赖（Nuitka 自动处理）
    │
    ├─ 4. 生成 Nuitka 编译命令并执行
    │     ├─ onefile:    --onefile
    │     ├─ standalone: --standalone
    │     └─ installer:  --standalone（先生成文件夹）
    │
    ├─ 5.（仅 installer 模式）生成 Inno Setup 脚本并编译为 setup.exe
    │
    └─ 6. 输出到 dist/ 目录
```

### 核心实现

CLI 入口通过 `pyproject.toml` 注册：

```toml
[project.scripts]
pywebron = "pywebron.cli:main"
```

主要模块：

```
pywebron/
└── cli/
    ├── __init__.py      # CLI 入口，argparse 解析
    ├── build.py         # 打包逻辑（调用 Nuitka）
    └── installer.py     # Inno Setup 脚本生成（installer 模式）
```

### Nuitka 命令生成示例

```bash
# onefile 模式
python -m nuitka --onefile --output-dir=./dist \
    --include-data-dir=./frontend/dist=frontend/dist \
    --include-data-file=./static/icon.png=static/icon.png \
    --windows-icon-from-ico=./static/icon.ico \
    --product-name="MyApp" \
    app.py

# standalone 模式
python -m nuitka --standalone --output-dir=./dist \
    --include-data-dir=./frontend/dist=frontend/dist \
    app.py
```

### Installer 脚本自动生成

installer 模式下，自动生成 Inno Setup 的 `.iss` 脚本：

```iss
[Setup]
AppName=MyApp
AppVersion=1.0.0
DefaultDirName={autopf}\MyApp
OutputDir=dist
OutputBaseFilename=MyApp_setup
SetupIconFile=static\icon.ico

[Files]
Source: "dist\standalone\*"; DestDir: "{app}"; Flags: recursesubdirs

[Icons]
Name: "{autodesktop}\MyApp"; Filename: "{app}\app.exe"
Name: "{group}\MyApp"; Filename: "{app}\app.exe"
```

## 依赖要求

| 打包模式 | 需要安装 |
|---------|---------|
| onefile / standalone | Nuitka (`pip install nuitka`)、C 编译器（Windows 上用 MSVC 或 MinGW） |
| installer | 以上 + Inno Setup |

`pywebron build` 执行前会检查这些依赖是否存在，缺失时给出安装提示。
