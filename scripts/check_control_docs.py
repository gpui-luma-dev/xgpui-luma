"""Compile event callback HTML examples without launching an application."""

from html.parser import HTMLParser
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = {
    "motion.html": [0, 1, 2, 3, 4],
    "drag-drop.html": 0,
    "plot-interaction.html": 0,
    "plot-view-state.html": 0,
    "plot-performance.html": 0,
    "plot-pie.html": 0,
    "plot-radar.html": 0,
    "plot-radial.html": 0,
    "plots.html": 1,
    "plot-cartesian.html": 0,
    "plot-styling.html": 0,
    "layout.html": 0,
    "composition.html": 0,
    "button.html": [0, 1],
    "icon-button.html": 0,
    "tooltip.html": 0,
    "listbox.html": [0, 1],
    "checkbox.html": 0,
    "switch.html": 0,
    "toggle.html": 0,
    "index.html": 1,  # Cargo.toml precedes the complete Rust application.
    "look-radix.html": [0, 1],
    "look-shadcn.html": 0,
    "architecture.html": 0,
    "looks.html": 0,
    "create-look.html": [0, 1, 2],
    "accordion.html": [0, 1],
    "autocomplete.html": 0,
    "context-menu.html": 0,
    "control-group.html": 0,
    "dock-splitter.html": 0,
    "icon-group.html": 0,
    "overlay-window.html": 0,
    "pager.html": 0,
    "progress.html": 0,
    "popover-button.html": 0,
    "popup-menu.html": 0,
    "radio-button.html": 0,
    "radio-group.html": 0,
    "resizable-panels.html": 0,
    "scrollbar.html": 0,
    "search-selector.html": 0,
    "selection-panel.html": 0,
    "selector.html": 0,
    "sidebar.html": [0, 1],
    "stepper.html": 0,
    "slider.html": 0,
    "colors.html": 1,
    "color-slider.html": [0, 1],
    "color-field.html": 0,
    "color-ring.html": 0,
    "color-arc.html": 0,
    "color-swatch.html": 0,
    "split-button.html": 0,
    "control-specialization.html": 0,
    "floating-menu.html": 0,
    "selector-list.html": 0,
    "slide-panel.html": 0,
    "split-view.html": 0,
    "table.html": [0, 1],
    "tabs.html": [0, 1, 2],
    "textarea.html": 0,
    "textfield.html": 0,
    "toolbar.html": 0,
    "tree-view.html": [0, 1],
    "combobox.html": 0,
}


class CodeBlocks(HTMLParser):
    def __init__(self):
        super().__init__()
        self.blocks = []
        self.current = None

    def handle_starttag(self, tag, attrs):
        if tag == "pre":
            self.current = []

    def handle_data(self, data):
        if self.current is not None:
            self.current.append(data)

    def handle_endtag(self, tag):
        if tag == "pre" and self.current is not None:
            self.blocks.append("".join(self.current))
            self.current = None


def example_source():
    modules = ["#![allow(unused)]"]
    for page, selection in EXAMPLES.items():
        indices = selection if isinstance(selection, list) else [selection]
        parser = CodeBlocks()
        parser.feed((ROOT / "docs" / page).read_text())
        if any(index >= len(parser.blocks) for index in indices):
            raise ValueError(f"Missing code examples {indices} in {page}")
        code = "\n".join(parser.blocks[index] for index in indices)
        if page in {"architecture.html", "looks.html"}:
            code = (
                "struct Example { save_requests: usize }\n"
                "fn build(cx: &mut gpui::Context<Example>) {\n" + code + "\n}"
            )
        modules.append(f"mod {page.removesuffix('.html').replace('-', '_')} {{\n{code}\n}}")
    return "\n".join(modules)


def main():
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"], cwd=ROOT,
    ))
    dependencies = {
        dependency.get("rename") or dependency["name"]: dependency
        for package in metadata["packages"] for dependency in package["dependencies"]
    }
    with tempfile.TemporaryDirectory(prefix="luma-control-docs-") as directory:
        package = Path(directory)
        manifest = ['[package]', 'name = "luma-control-doc-check"', 'version = "0.0.0"',
                    'edition = "2024"', '[dependencies]']
        for name in ["gpui", "gpui_platform"]:
            spec = dependencies[name]
            fields = [f'package = {json.dumps(spec["name"])}',
                      f'version = {json.dumps(spec["req"])}',
                      f'features = {json.dumps(spec["features"])}']
            manifest.append(f'{name} = {{ {", ".join(fields)} }}')
        for name, path in [("xgpui-luma", "sdk"), ("xgpui-luma-look-radix", "look-radix"),
                           ("xgpui-luma-look-shadcn", "look-shadcn"), ("xgpui-luma-plot", "plot"),
                           ("xgpui-luma-color", "luma-color")]:
            manifest.append(f'{name} = {{ path = {json.dumps(str(ROOT / "crates" / path))} }}')
        manifest.append(f'luma-app-common = {{ path = {json.dumps(str(ROOT / "apps/common"))} }}')
        (package / "Cargo.toml").write_text("\n".join(manifest) + "\n")
        (package / "src").mkdir()
        (package / "src/lib.rs").write_text(example_source())
        # Retain the workspace's dependency resolutions. Cargo adds only this
        # temporary package; no repository manifests or lockfiles are modified.
        shutil.copyfile(ROOT / "Cargo.lock", package / "Cargo.lock")
        subprocess.run(["cargo", "check", "--manifest-path", str(package / "Cargo.toml"),
                        "--target-dir", str(ROOT / "target")], check=True, cwd=ROOT)
    count = sum(len(value) if isinstance(value, list) else 1 for value in EXAMPLES.values())
    print(f"Compiled {count} documentation examples.")


if __name__ == "__main__":
    main()
