from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import shutil
import subprocess


@dataclass(slots=True)
class InstallerOptions:
    app_name: str
    entry_stem: str
    standalone_dir: Path
    output_dir: Path
    icon: Path | None = None


def build_installer(options: InstallerOptions) -> int:
    compiler = shutil.which("iscc") or shutil.which("ISCC.exe")
    if compiler is None:
        raise SystemExit(
            "Inno Setup is required for --installer. Install it and ensure ISCC.exe is in PATH."
        )

    exe_path = _find_executable(options)
    script_path = _write_script(options, exe_path)

    print("\nRunning Inno Setup:")
    print(_format_command([compiler, str(script_path)]))
    result = subprocess.run([compiler, str(script_path)], cwd=options.output_dir)
    if result.returncode == 0:
        print(f"\nInstaller completed: {options.output_dir / (options.app_name + '_setup.exe')}")
    return result.returncode


def _find_executable(options: InstallerOptions) -> Path:
    candidates = [
        options.standalone_dir / f"{options.app_name}.exe",
        options.standalone_dir / f"{options.entry_stem}.exe",
    ]
    candidates.extend(sorted(options.standalone_dir.glob("*.exe")))

    for candidate in candidates:
        if candidate.is_file():
            return candidate

    raise SystemExit(f"Executable not found in standalone folder: {options.standalone_dir}")


def _write_script(options: InstallerOptions, exe_path: Path) -> Path:
    options.output_dir.mkdir(parents=True, exist_ok=True)
    script_path = options.output_dir / f"{options.app_name}.iss"
    exe_name = exe_path.name
    app_name = _inno_value(options.app_name)
    escaped_exe_name = _inno_value(exe_name)
    output_dir = _inno_path(options.output_dir)
    standalone_dir = _inno_path(options.standalone_dir)
    icon_line = (
        f'SetupIconFile="{_inno_path(options.icon)}"\n'
        if options.icon is not None
        else ""
    )

    content = "\n".join(
        [
            "[Setup]",
            f"AppName={app_name}",
            "AppVersion=1.0.0",
            rf"DefaultDirName={{autopf}}\{app_name}",
            f"DefaultGroupName={app_name}",
            f"OutputDir={output_dir}",
            f"OutputBaseFilename={app_name}_setup",
            f"{icon_line}Compression=lzma",
            "SolidCompression=yes",
            "",
            "[Files]",
            rf'Source: "{standalone_dir}\*"; DestDir: "{{app}}"; Flags: recursesubdirs ignoreversion',
            "",
            "[Icons]",
            rf'Name: "{{autodesktop}}\{app_name}"; Filename: "{{app}}\{escaped_exe_name}"',
            rf'Name: "{{group}}\{app_name}"; Filename: "{{app}}\{escaped_exe_name}"',
            "",
        ]
    )
    script_path.write_text(content, encoding="utf-8")
    return script_path


def _inno_path(path: Path) -> str:
    return str(path.resolve()).replace("/", "\\").replace('"', '""')


def _inno_value(value: str) -> str:
    return value.replace('"', '""')


def _format_command(command: list[str]) -> str:
    return " ".join(f'"{part}"' if " " in part else part for part in command)
