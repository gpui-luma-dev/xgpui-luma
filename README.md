# xgpui-luma

Component library built on [xgpui](https://xgpui.dev/), with a lookless
control SDK, Shadcn and Radix look adapters, color controls, and desktop example apps. Documentation is hosted at
[gpui-luma.dev](https://gpui-luma.dev). Source lives at
[gpui-luma-dev/xgpui-luma](https://github.com/gpui-luma-dev/xgpui-luma).

Requires a recent Rust stable toolchain (`rust-toolchain.toml` tracks `stable`).

## Build an app

Luma now uses xgpui 2.0.1 instead of gpui-unofficial. The dependency aliases
`gpui` and `gpui_platform` keep existing Rust imports unchanged. Browser builds
use `xgpui-web`, aliased as `gpui_web`.

The examples below target this repository’s crate names and versions. Once these
versions are published to crates.io, use them directly with the commands below.
To build from this checkout, see [Example programs](#example-programs).

```bash
cargo new my-luma-app
cd my-luma-app
cargo add xgpui-luma@1.0.1 xgpui-luma-look-shadcn@1.0.1
cargo add xgpui@=2.0.1 --rename gpui
cargo add xgpui-platform@=2.0.1 --rename gpui_platform --features font-kit
```

Add the [complete app example](https://gpui-luma.dev/#sample) to `src/main.rs`, then
run `cargo build` or `cargo run`. Choose the Radix look instead of Shadcn, or use
the SDK’s built-in styling. Add color controls and plots as needed.

Use `1.0.1` for Luma dependencies to accept compatible `1.x` releases. Run
`cargo update` to refresh versions already recorded in `Cargo.lock`.

```toml
[dependencies]
gpui = { package = "xgpui", version = "=2.0.1" }
gpui_platform = { package = "xgpui-platform", version = "=2.0.1", features = ["font-kit"] }
xgpui-luma = "1.0.1"
xgpui-luma-look-shadcn = "1.0.1"
# Alternatively: xgpui-luma-look-radix = "1.0.1"
# Optional: xgpui-luma-color = "1.0.1"
# Optional: xgpui-luma-plot = "1.0.1"
```

## Library crates

| Crate | Purpose |
|---|---|
| [`xgpui-luma`](https://crates.io/crates/xgpui-luma) | Lookless controls, layouts, focus, interaction, motion, and shared theme contracts |
| [`xgpui-luma-look-shadcn`](https://crates.io/crates/xgpui-luma-look-shadcn) | Shadcn themes, CSS tokens, styled builders, and optional inspection tooling |
| [`xgpui-luma-look-radix`](https://crates.io/crates/xgpui-luma-look-radix) | Radix themes, palette scales, custom colors, and styled builders |
| [`xgpui-luma-color`](https://crates.io/crates/xgpui-luma-color) | Color fields, rings, sliders, swatches, and color rendering primitives |
| [`xgpui-luma-plot`](https://crates.io/crates/xgpui-luma-plot) | Cartesian, pie, radar, and radial charts |

Depend on **`xgpui-luma`**, then choose a look adapter. Add `xgpui-luma-color` for
color controls. The links above point to the registry destinations for these crates.

Import the SDK with `use xgpui_luma::…`. Look adapter imports use
`xgpui_luma_look_shadcn` or `xgpui_luma_look_radix`.

## Styling

Use the SDK’s built-in light/dark styling, or choose the
[Shadcn](https://gpui-luma.dev/look-shadcn.html) or
[Radix](https://gpui-luma.dev/look-radix.html) Look. See the
[web docs](https://gpui-luma.dev/looks.html) for theme configuration.

## Example programs

Clone the repository to run its desktop examples:

```bash
git clone https://github.com/gpui-luma-dev/xgpui-luma.git
cd xgpui-luma
```

**Use release builds for optimal app performance. Debug builds are intended for
development and can make rendering and interactions noticeably slower.**

The [justfile](justfile) runs apps and explicit builds in release mode by default.
Use `just --set profile dev <recipe>` for a development build. Run `just` to list
recipes without launching an app. Without `just`, run the corresponding
`cargo run -p <package> --release` command.

**Luma Studio** — Shadcn look workbench (control docs, theme inspection):

![Luma Studio](docs/screenshots/luma-shadcn.png)

```bash
just app-studio
# Debug: just --set profile dev app-studio
```

**Luma Radix Studio** — Radix style guide, custom palette editor, and control previews:

![Luma Radix Studio](docs/screenshots/luma-radix.png)

```bash
just app-radix
# Debug: just --set profile dev app-radix
```

**Luma Plot Viz** — chart galleries using `xgpui-luma-plot` and the SDK’s built-in styling, with line, area, bar, pie, radar, and radial examples:

![Luma Plot Viz showing bar charts in dark mode](docs/screenshots/luma-plot.png)

```bash
just app-plot
# Debug: just --set profile dev app-plot
```

Additional examples (also release builds by default):

```bash
just app-neumorphic
just shell-detached
just shell-split-titlebar
just shell-vscode
just shell-2026
```

For profiling, use `just perf-studio` to enable the Metal HUD, or
`just --set profile dev app-studio-backtrace` to run a debug build with Rust backtraces.

The shell demos accept `default` or a built-in Shadcn theme ID,
such as `retro-arcade`, as their first argument. Recipes forward arguments to the
app and use the default theme when none is provided:

```bash
just shell-vscode retro-arcade
```

## Prebuilt executables

The [Build Luma Studio](https://github.com/gpui-luma-dev/xgpui-luma/actions/workflows/build.yml)
workflow is disabled in its YAML configuration. All GitHub workflows have their
jobs gated off, and automatic push and pull-request triggers are removed. Manual
dispatch skips every job until the YAML configuration is re-enabled.

## Development checks

From the repository root, use the library checks during development:

```bash
just sdk-check
just sdk-build
just sdk-test
just sdk-clippy
just sdk-doc-test
```

Check, build, and Clippy target Cargo’s five default library members. The test
recipes use CI’s selected library suites; `sdk-test` also includes `luma-app-common`.
For the full workspace, use `just workspace-check`, `just workspace-build`, or
`just workspace-test`. Run `just apps-build` to compile all native apps without
launching them. `just ci` runs formatting, Python helper tests, workspace Clippy,
selected library tests and doc-tests, and control documentation checks. Run
`just fmt` to format the workspace.
Before sending a PR, run:

```bash
just pre-pr
```

This runs formatting, workflow lint, native and WASM Clippy, library tests and
doc-tests, documentation example compilation, Python helper/packaging tests,
JavaScript syntax checks, and whitespace checks. It checks all app targets with
Clippy but does not run sample app tests, launch apps, or build deployment assets.
It stops at the first failure.

Requires Node.js and [actionlint](https://github.com/rhysd/actionlint) on `PATH`,
plus the WASM Rust target (`rustup target add wasm32-unknown-unknown`). On macOS,
install the tools with `brew install node actionlint`. Native-only Rust linker
flags are cleared for the WASM check.

The `test-support` features enable headless GPUI interaction tests.

To package and compile-verify the five release archives without publishing:

```bash
cargo package --locked --registry crates-io \
  -p xgpui-luma -p xgpui-luma-color -p xgpui-luma-look-radix \
  -p xgpui-luma-look-shadcn -p xgpui-luma-plot
```

Add `--allow-dirty` to check uncommitted changes. Keep registry access enabled:
with Cargo 1.99.1, `--offline` fails to verify unpublished workspace dependencies
with a temporary-registry `no hash listed` error. Archives are written to
`target/package/`. This command does not publish crates.

## License

Apache-2.0. See [LICENSE](LICENSE). Bundled third-party material retains its own
license terms; attribution files are included with the relevant crates and assets.
