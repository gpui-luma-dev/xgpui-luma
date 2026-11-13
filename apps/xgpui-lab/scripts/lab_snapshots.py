#!/usr/bin/env python3
"""Capture and compare XGPUI Lab page snapshots for regression checks.

Builds xgpui-lab once, then opens the real Lab window on each page with
`--open PAGE --capture PNG`, in dark and light. Launching the Lab opens a
native window for each capture.

    # snapshot the lab screens (default pages) into a directory
    python3 apps/xgpui-lab/scripts/lab_snapshots.py capture target/lab-snapshots/before
    # ... change code ...
    python3 apps/xgpui-lab/scripts/lab_snapshots.py capture target/lab-snapshots/after
    python3 apps/xgpui-lab/scripts/lab_snapshots.py compare target/lab-snapshots/before target/lab-snapshots/after

    # other pages, themes or size (see `xgpui-lab --list-pages`)
    python3 .../lab_snapshots.py capture DIR --page "xgpui-media/Image Filters" --theme dark --size 1280x900

`compare` reports identical, changed and missing snapshots. Byte-identical PNGs
are identical renders. When ImageMagick's `compare` is installed it also counts
changed pixels and writes a highlighted diff image for each changed page.
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

# The screens that use the LabScreen design language.
DEFAULT_PAGES = [
    "xgpui-color/Color Spaces",
    "xgpui-color/Conversions",
    "xgpui-color/Interpolation",
    "xgpui-color/Gamut Mapping",
    "xgpui-color/CPU Gradients",
    "xgpui-gamut/Chromaticity & Gamut",
    "xgpui-gamut/Wide Color Fills",
    "xgpui-gamut/Wide Gradients",
    "xgpui-gamut/Gamut Mapping",
    "xgpui-gamut/Overlays",
    "xgpui-color/Italy Photo Comparison",
    "xgpui-media/Image Filters",
    "xgpui-media/Images",
    "xgpui-media/Textures & Mipmaps",
    "xgpui-media/Linear Adjustments",
    "xgpui-media/Float / P3",
    "xgpui-media/Tiled Canvas",
    "xgpui-media/Photo Viewer",
    "xgpui-media/Video Frames",
    "xgpui-media/Resource Limits",
    "xgpui-shell/Shell Lab",
    "xgpui-shell/Desktop Preferences",
    "xgpui-shader/Typography",
    "xgpui-shader/Gradients",
    "xgpui-shader/Gradient Paints",
    "xgpui-shader/WGSL Paints",
    "xgpui-shader/Instancing",
    "xgpui-shader/Mesh Editor",
    "xgpui-shader/Shader Depot: Grid",
    "xgpui-shader/Shadertoy: Seascape",
    "xgpui-shader/Pastel Waves",
    "xgpui-shader/Brushes",
    "xgpui-shader/Layer Effects",
    "xgpui-shader/Shadows",
    "xgpui-shader/Borders",
    "xgpui-parley/Text Shaping",
    "xgpui-parley/Line Layout",
    "xgpui-parley/Paragraphs",
    "xgpui-parley/Text Field",
    "xgpui-parley/Fonts & Fallback",
    "xgpui-parley/Rasterization",
    "xgpui-parley/Scrolling",
    "xgpui-svg/Rendering",
    "xgpui-svg/Scaling",
    "xgpui-svg/Path Effects",
    "xgpui-svg/Scheduling & Cache",
    "xgpui-canvas/Spatial Canvas",
    "xgpui-canvas/Mandelbrot",
]
THEMES = ["dark", "light"]
REPO = Path(__file__).resolve().parents[3]


def slug(page: str, theme: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", page.lower()).strip("-") + f"--{theme}.png"


def build() -> Path:
    subprocess.run(["cargo", "build", "-p", "xgpui-lab", "--bin", "xgpui-lab"], cwd=REPO, check=True)
    target = Path(os.environ.get("CARGO_TARGET_DIR", REPO / "target"))
    binary = target / "debug" / "xgpui-lab"
    if not binary.exists():
        sys.exit(f"built binary not found at {binary}")
    return binary


def capture(args) -> int:
    out = Path(args.dir)
    out.mkdir(parents=True, exist_ok=True)
    binary = build()
    failures = 0
    for page in args.page or DEFAULT_PAGES:
        for theme in args.theme or THEMES:
            png = out / slug(page, theme)
            command = [str(binary), "--open", page, "--theme", theme, "--size", args.size,
                       "--settle", str(args.settle), "--capture", str(png)] + (["--hide-nav"] if args.hide_nav else [])
            try:
                result = subprocess.run(command, cwd=REPO, capture_output=True, text=True, timeout=args.settle / 1000 + 60)
            except subprocess.TimeoutExpired:
                print(f"TIMEOUT  {page} ({theme})")
                failures += 1
                continue
            if result.returncode != 0 or not png.exists():
                print(f"FAILED   {page} ({theme}): {result.stderr.strip().splitlines()[-1:] or result.returncode}")
                failures += 1
            else:
                print(f"captured {png}")
    return 1 if failures else 0


def compare(args) -> int:
    before, after = Path(args.before), Path(args.after)
    names = sorted({p.name for p in before.glob("*.png")} | {p.name for p in after.glob("*.png")})
    magick = shutil.which("magick") or shutil.which("compare")
    diff_dir = after / "diff"
    changed = 0
    for name in names:
        a, b = before / name, after / name
        if not a.exists() or not b.exists():
            print(f"MISSING  {name} ({'before' if not a.exists() else 'after'})")
            changed += 1
            continue
        if a.read_bytes() == b.read_bytes():
            print(f"same     {name}")
            continue
        changed += 1
        detail = ""
        if magick:
            diff_dir.mkdir(exist_ok=True)
            tool = [magick, "compare"] if magick.endswith("magick") else [magick]
            result = subprocess.run(tool + ["-metric", "AE", "-fuzz", "1%", str(a), str(b), str(diff_dir / name)],
                                    capture_output=True, text=True)
            raw = (result.stderr or result.stdout).strip().split(" ")[0]
            try:
                pixels = f"{int(float(raw)):,}"
            except ValueError:
                pixels = raw
            detail = f"  {pixels} px differ · diff/{name}"
        print(f"CHANGED  {name}{detail}")
    print(f"{len(names) - changed} same, {changed} changed or missing")
    return 1 if changed else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    cap = commands.add_parser("capture", help="build once, then snapshot pages")
    cap.add_argument("dir")
    cap.add_argument("--page", action="append", help="page to capture (repeatable); default: the LabScreen pages")
    cap.add_argument("--theme", action="append", choices=THEMES, help="theme (repeatable); default: dark and light")
    cap.add_argument("--size", default="1440x900", help="window size, WIDTHxHEIGHT")
    cap.add_argument("--hide-nav", action="store_true", help="capture with the navigation panel closed")
    cap.add_argument("--settle", type=int, default=4000, help="milliseconds to wait before capturing")
    cmp = commands.add_parser("compare", help="compare two capture directories")
    cmp.add_argument("before")
    cmp.add_argument("after")
    args = parser.parse_args()
    return capture(args) if args.command == "capture" else compare(args)


if __name__ == "__main__":
    sys.exit(main())
