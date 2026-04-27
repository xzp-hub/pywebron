from __future__ import annotations

import ast
from dataclasses import dataclass
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

from .installer import InstallerOptions, build_installer


BUILTIN_ICON = Path(__file__).resolve().parents[1] / "builtins" / "pywebron.png"


@dataclass(slots=True)
class BuildOptions:
    entry: Path
    standalone: bool = False
    installer: bool = False
    name: str | None = None
    icon: Path | None = None
    output: Path = Path("dist")
    extra_data: list[Path] | None = None
    no_confirm: bool = False


def build_app(options: BuildOptions) -> int:
    project_dir = Path.cwd()
    entry = _resolve_existing_file(options.entry, "entry file")
    output_dir = _resolve_output_dir(options.output)
    app_name = options.name or project_dir.name
    app_version = _read_project_version(project_dir)
    extra_data = options.extra_data or []

    icon = None
    if options.icon is not None:
        icon = _resolve_existing_file(options.icon, "icon file")
    else:
        app_icon = _find_entry_icon(entry, project_dir)
        if app_icon is not None:
            icon = _ensure_ico_icon(app_icon, output_dir, "app")

    if icon is None and BUILTIN_ICON.is_file():
        icon = _ensure_ico_icon(BUILTIN_ICON, output_dir, "default")

    _check_nuitka()
    if options.installer:
        _check_inno_setup()

    mode = "installer" if options.installer else "standalone" if options.standalone else "onefile"
    _print_plan(mode, entry, output_dir, app_name, icon, extra_data)
    if not options.no_confirm and not _confirm():
        print("Build cancelled.")
        return 1

    output_dir.mkdir(parents=True, exist_ok=True)
    command = _build_nuitka_command(
        entry=entry,
        output_dir=output_dir,
        app_name=app_name,
        app_version=app_version,
        icon=icon,
        extra_data=extra_data,
        mode=mode,
    )

    print("\nRunning Nuitka:")
    print(_format_command(command))
    result = subprocess.run(command, cwd=project_dir)
    if result.returncode != 0:
        return result.returncode

    _notify_icon_cache_changed()

    if options.installer:
        standalone_dir = _find_standalone_dir(output_dir, app_name, entry)
        return build_installer(
            InstallerOptions(
                app_name=app_name,
                entry_stem=entry.stem,
                standalone_dir=standalone_dir,
                output_dir=output_dir,
                icon=icon,
            )
        )

    print(f"\nBuild completed: {output_dir}")
    return 0


def _resolve_existing_file(path: Path, label: str) -> Path:
    resolved = path.expanduser().resolve()
    if not resolved.is_file():
        raise SystemExit(f"{label} not found: {path}")
    return resolved


def _resolve_existing_path(path: Path, label: str) -> Path:
    resolved = path.expanduser().resolve()
    if not resolved.exists():
        raise SystemExit(f"{label} not found: {path}")
    return resolved


def _resolve_output_dir(path: Path) -> Path:
    return path.expanduser().resolve()


def _find_entry_icon(entry: Path, project_dir: Path) -> Path | None:
    try:
        tree = ast.parse(entry.read_text(encoding="utf-8"), filename=str(entry))
    except (OSError, SyntaxError, UnicodeDecodeError):
        return None

    names = {
        "PROJECT_ROOT_PATH": str(project_dir),
    }
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        for keyword in node.keywords:
            if keyword.arg != "icon_path":
                continue
            value = _eval_icon_expr(keyword.value, names)
            if value is None or value == "__pywebron_builtin_icon__":
                return None
            icon_path = Path(value).expanduser()
            if not icon_path.is_absolute():
                icon_path = entry.parent / icon_path
            return icon_path.resolve() if icon_path.is_file() else None
    return None


def _eval_icon_expr(node: ast.AST, names: dict[str, str]) -> str | None:
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    if isinstance(node, ast.Name):
        return names.get(node.id)
    if isinstance(node, ast.JoinedStr):
        parts = []
        for value in node.values:
            if isinstance(value, ast.Constant) and isinstance(value.value, str):
                parts.append(value.value)
            elif isinstance(value, ast.FormattedValue):
                evaluated = _eval_icon_expr(value.value, names)
                if evaluated is None:
                    return None
                parts.append(evaluated)
            else:
                return None
        return "".join(parts)
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
        left = _eval_icon_expr(node.left, names)
        right = _eval_icon_expr(node.right, names)
        if left is not None and right is not None:
            return left + right
    return None


def _ensure_ico_icon(source: Path, output_dir: Path, name: str) -> Path:
    if source.suffix.lower() == ".ico":
        return source

    icon_dir = output_dir / ".pywebron"
    icon_dir.mkdir(parents=True, exist_ok=True)
    icon_hash = hashlib.sha1(source.read_bytes()).hexdigest()[:12]
    icon_path = icon_dir / f"{name}-{icon_hash}.ico"

    for old_icon in icon_dir.glob(f"{name}-*.ico"):
        if old_icon != icon_path:
            old_icon.unlink(missing_ok=True)

    if icon_path.is_file():
        return icon_path

    if source.suffix.lower() == ".png":
        png_data = source.read_bytes()
        width, height = _read_png_size(png_data, source)
        icon_path.write_bytes(_png_to_ico(png_data, width, height))
    else:
        _convert_image_to_ico(source, icon_path)
    return icon_path


def _read_png_size(data: bytes, source: Path) -> tuple[int, int]:
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"Icon is not a valid PNG: {source}")
    width = int.from_bytes(data[16:20], "big")
    height = int.from_bytes(data[20:24], "big")
    return width, height


def _convert_image_to_ico(source: Path, target: Path) -> None:
    if sys.platform != "win32":
        raise SystemExit(f"Only .ico or .png icons are supported on this platform: {source}")

    script = r'''
param([string]$Source, [string]$Target)
Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Image]::FromFile($Source)
try {
    $size = [Math]::Min(256, [Math]::Max($img.Width, $img.Height))
    $bmp = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    try {
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        try {
            $g.Clear([System.Drawing.Color]::Transparent)
            $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
            $scale = [Math]::Min($size / $img.Width, $size / $img.Height)
            $w = [int]($img.Width * $scale)
            $h = [int]($img.Height * $scale)
            $x = [int](($size - $w) / 2)
            $y = [int](($size - $h) / 2)
            $g.DrawImage($img, $x, $y, $w, $h)
        } finally { $g.Dispose() }
        $handle = $bmp.GetHicon()
        $icon = [System.Drawing.Icon]::FromHandle($handle)
        try {
            $fs = [System.IO.File]::Create($Target)
            try { $icon.Save($fs) } finally { $fs.Dispose() }
        } finally {
            $icon.Dispose()
            Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class NativeIconCleanup {
    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool DestroyIcon(IntPtr hIcon);
}
'@
            [NativeIconCleanup]::DestroyIcon($handle) | Out-Null
        }
    } finally { $bmp.Dispose() }
} finally { $img.Dispose() }
'''
    result = subprocess.run(
        [
            "powershell.exe",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            f"& {{ {script} }}",
            str(source),
            str(target),
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if result.returncode != 0 or not target.is_file():
        raise SystemExit(f"Failed to convert icon to .ico: {source}")


def _png_to_ico(png_data: bytes, width: int, height: int) -> bytes:
    icon_width = width if width < 256 else 0
    icon_height = height if height < 256 else 0
    header = (0).to_bytes(2, "little") + (1).to_bytes(2, "little") + (1).to_bytes(2, "little")
    image_offset = 6 + 16
    entry = b"".join(
        [
            icon_width.to_bytes(1, "little"),
            icon_height.to_bytes(1, "little"),
            (0).to_bytes(1, "little"),
            (0).to_bytes(1, "little"),
            (1).to_bytes(2, "little"),
            (32).to_bytes(2, "little"),
            len(png_data).to_bytes(4, "little"),
            image_offset.to_bytes(4, "little"),
        ]
    )
    return header + entry + png_data


def _read_project_version(project_dir: Path) -> str:
    pyproject_path = project_dir / "pyproject.toml"
    if not pyproject_path.is_file():
        return "1.0.0.0"

    try:
        data = tomllib.loads(pyproject_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError):
        return "1.0.0.0"

    version = str(data.get("project", {}).get("version", "1.0.0"))
    return _windows_version(version)


def _windows_version(version: str) -> str:
    parts = []
    for part in version.replace("-", ".").split("."):
        if part.isdigit():
            parts.append(str(int(part)))
        else:
            break

    while len(parts) < 4:
        parts.append("0")
    return ".".join(parts[:4])


def _check_nuitka() -> None:
    try:
        result = subprocess.run(
            [sys.executable, "-m", "nuitka", "--version"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except OSError as exc:
        raise SystemExit(f"Failed to check Nuitka: {exc}") from exc

    if result.returncode != 0:
        raise SystemExit(
            "Nuitka is required. Install it with: python -m pip install 'pywebron[build]'"
        )

    if sys.platform == "win32" and shutil.which("cl") is None and shutil.which("gcc") is None:
        print(
            "Warning: no C compiler found in PATH. Nuitka needs MSVC Build Tools or MinGW on Windows."
        )


def _check_inno_setup() -> None:
    if shutil.which("iscc") is None and shutil.which("ISCC.exe") is None:
        raise SystemExit(
            "Inno Setup is required for --installer. Install it and ensure ISCC.exe is in PATH."
        )


def _notify_icon_cache_changed() -> None:
    if sys.platform != "win32":
        return

    script = r"""
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ShellRefresh {
    [DllImport("shell32.dll")]
    public static extern void SHChangeNotify(int wEventId, uint uFlags, IntPtr dwItem1, IntPtr dwItem2);
}
'@
[ShellRefresh]::SHChangeNotify(0x08000000, 0x0000, [IntPtr]::Zero, [IntPtr]::Zero)
"""
    subprocess.run(
        ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def _build_nuitka_command(
    *,
    entry: Path,
    output_dir: Path,
    app_name: str,
    app_version: str,
    icon: Path | None,
    extra_data: list[Path],
    mode: str,
) -> list[str]:
    command = [
        sys.executable,
        "-m",
        "nuitka",
        "--assume-yes-for-downloads",
        f"--output-dir={output_dir}",
        f"--output-filename={app_name}",
        f"--product-name={app_name}",
        f"--product-version={app_version}",
        f"--file-version={app_version}",
    ]
    command.append("--standalone" if mode in {"standalone", "installer"} else "--onefile")

    if icon is not None:
        command.append(f"--windows-icon-from-ico={icon}")

    for item in extra_data:
        source = _resolve_existing_path(item, "extra data")
        target = _data_target(source)
        if source.is_dir():
            command.append(f"--include-data-dir={source}={target}")
        else:
            command.append(f"--include-data-file={source}={target}")

    command.append(str(entry))
    return command


def _data_target(source: Path) -> str:
    try:
        return source.relative_to(Path.cwd()).as_posix()
    except ValueError:
        return source.name


def _find_standalone_dir(output_dir: Path, app_name: str, entry: Path) -> Path:
    candidates = [
        output_dir / f"{app_name}.dist",
        output_dir / f"{entry.stem}.dist",
    ]
    candidates.extend(output_dir.glob("*.dist"))
    for candidate in candidates:
        if candidate.is_dir():
            return candidate
    raise SystemExit(f"Nuitka standalone output folder not found in: {output_dir}")


def _print_plan(
    mode: str,
    entry: Path,
    output_dir: Path,
    app_name: str,
    icon: Path | None,
    extra_data: list[Path],
) -> None:
    print("PyWebron build plan")
    print(f"  mode:       {mode}")
    print(f"  entry:      {entry}")
    print(f"  name:       {app_name}")
    print(f"  output:     {output_dir}")
    print(f"  icon:       {icon or '-'}")
    if extra_data:
        print("  extra data:")
        for item in extra_data:
            print(f"    - {item}")
    else:
        print("  extra data: -")


def _confirm() -> bool:
    answer = input("\nContinue build? [y/N]: ").strip().lower()
    return answer in {"y", "yes"}


def _format_command(command: list[str]) -> str:
    return " ".join(f'"{part}"' if " " in part else part for part in command)
