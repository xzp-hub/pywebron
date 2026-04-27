from __future__ import annotations

import argparse
from pathlib import Path
from typing import Sequence

from .build import BuildOptions, build_app


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="pywebron",
        description="PyWebron command line tools",
    )
    subparsers = parser.add_subparsers(dest="command")

    build_parser = subparsers.add_parser(
        "build",
        help="Build a PyWebron app into a distributable desktop application",
    )
    build_parser.add_argument(
        "entry",
        nargs="?",
        default="app.py",
        help="Entry Python file path. Defaults to app.py",
    )
    mode_group = build_parser.add_mutually_exclusive_group()
    mode_group.add_argument(
        "--standalone",
        action="store_true",
        help="Build a standalone folder instead of a single executable",
    )
    mode_group.add_argument(
        "--installer",
        action="store_true",
        help="Build an installer with Inno Setup",
    )
    build_parser.add_argument(
        "--name",
        help="Application name. Defaults to current project directory name",
    )
    build_parser.add_argument("--icon", help="Application icon path")
    build_parser.add_argument(
        "--output",
        default="dist",
        help="Output directory. Defaults to ./dist",
    )
    build_parser.add_argument(
        "--extra-data",
        action="append",
        default=[],
        help="Extra file or directory to bundle. Can be used multiple times",
    )
    build_parser.add_argument(
        "--no-confirm",
        action="store_true",
        help="Skip confirmation before building",
    )

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = create_parser()
    args = parser.parse_args(argv)

    if args.command == "build":
        options = BuildOptions(
            entry=Path(args.entry),
            standalone=args.standalone,
            installer=args.installer,
            name=args.name,
            icon=Path(args.icon) if args.icon else None,
            output=Path(args.output),
            extra_data=[Path(item) for item in args.extra_data],
            no_confirm=args.no_confirm,
        )
        return build_app(options)

    parser.print_help()
    return 0


__all__ = ("main", "create_parser")
