# Live documentation examples

The documentation runs primary controls, color controls, and plots in an xgpui-owned
canvas. Each marked `data-live-example="<control>-basic"` Rust block in `docs/`
is the source of truth: `build.rs` decodes it into an include, so the browser runs
the displayed constructor and callbacks. `src/web.rs` provides application setup,
bounds, theme synchronization, and callback readouts.

## Build and preview

The workspace uses the published `xgpui` and `xgpui-platform` 1.0.0 crates.
See [xgpui](https://xgpui.dev/) for framework documentation.
This browser app uses `xgpui-web` 1.0.0, aliased as `gpui_web`; Rust imports
retain the `gpui` name through the workspace dependency alias.

Requirements: Rust, Python 3, the `wasm32-unknown-unknown` target, and a
`wasm-bindgen-cli` version matching `wasm-bindgen` in `Cargo.lock`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
sh apps/docs-embed/build.sh
just docs-serve
```

Open <http://127.0.0.1:8766/button.html>. `just docs-build` builds the runtime;
`sh apps/docs-embed/build.sh dev` uses a faster development profile. The build
script clears `RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS` so native-only linker flags
such as `-lc++` do not reach the WASM linker.

Generated files in `apps/docs-embed/pkg/` and `docs/examples/pkg/` are ignored.
The packager publishes only one content-versioned JavaScript/WASM pair and an
uncached manifest. Rebuilding prunes obsolete generated versions and unversioned
copies. Reload local preview pages after rebuilding. Production deploys publish
the complete directory together; no runtime files are committed to Git.

## Add or change a sample

1. Edit the marked Rust block in its documentation page.
2. For a new control, add its page name to `build.rs` and its adapter/route to
   `src/web.rs`. Retained-control adapters supply bounds and callback labels;
   retained-view adapters preserve the snippet's own `Render` implementation.
3. Add one preview iframe and an open-separately link to the page. When examples
   share a page, use the existing HTML tab strip to navigate one shared iframe.
4. Rebuild and check interaction and light/dark theme changes.

Studio and docs share `luma_app_common::color_picker::ColorPickerPopover`. The
Swatch sample subscribes to its color changes; it is a working picker composition,
not just a click notification. Control Group remains documentation-only as a base
class.

Explicit Look constructors should reuse the ambient Look with a standalone
fallback. SDK controls and Look builders own interactive chrome. The host embeds
Inter from `apps/common/assets` and IBM Plex Sans from `assets/ibm-plex-sans`;
the latter is copied unmodified from GPUI Web 1.24.2, supplies GPUI's `.ZedSans`
fallback, and includes its OFL license.

The runtime uses `WebPlatform::new(false)` with worker threads disabled, so the
site does not require cross-origin isolation headers. It provides embedded Lucide
assets, binds SDK control keys, and shares one cached bundle across previews.
Chart tab changes release the previous iframe document; plots retain pointer input.

## Validation

Run `just pre-pr` before publishing a PR. It includes the native library,
WASM, documentation example, JavaScript, workflow, and Python checks. Install
Node.js and actionlint on `PATH` and the `wasm32-unknown-unknown` Rust target.
It does not launch applications or build the deployment runtime.

For focused runtime validation and deployment packaging:

```sh
cargo fmt --all --check
cargo clippy --locked -p luma-docs-embed --target wasm32-unknown-unknown -- -D warnings
cargo test --locked -p xgpui-luma --lib --features test-support
python3 -B scripts/check_control_docs.py
python3 -B -m unittest discover -s apps/docs-embed -p 'test_*.py'
sh apps/docs-embed/build.sh
python3 -B apps/docs-embed/package_runtime.py --check
```

Browser checks cover control activation, callbacks, text entry, nested tree
expansion, table selection, pager synchronization, progress animation, color
adjustments, chart tabs, and state retention across theme changes. The SDK's
context-menu regression uses a headless GPUI TestWindow, without launching Studio.

## Deployment

`.github/workflows/docs.yml` runs WASM Clippy, packaging tests, and JavaScript
syntax checks on relevant pull requests. PRs do not install wasm-bindgen CLI,
compile a release runtime, or upload a site artifact. Superseded PR checks cancel.

Main-branch pushes and manual runs on `main` build and validate the release
runtime, upload the full site as `docs-site`, and deploy through GitHub Pages.
The wasm-bindgen CLI cache is keyed by its locked version and runner platform;
check and release Rust caches are separate.

In `.github/workflows/ci.yml`, changes confined to `docs/` and `apps/docs-embed/`
keep formatting and compile all documentation snippets in a focused job. They
skip native workspace Clippy and test suites. Shared Rust, dependency, script,
and workflow changes retain full native CI. Missing comparison history defaults
to full checks; a source file moved into docs also retains native coverage.
Routing tests run with `python3 -B -m unittest discover -s scripts -p 'test_*.py'`.

For the initial rollout, after the workflow reaches `main`, set repository
Settings → Pages → Build and deployment → Source to **GitHub Actions**, retain
`gpui-luma.dev`, and run the workflow on `main`. Branch-based Pages deployment
cannot publish ignored generated runtime assets.

## Follow-ups

- Layout, scrolling containers, sidebars, and window/panel composition are excluded.
- The release runtime is about 21 MiB before transfer compression. Bundle splitting
  and size optimization remain follow-ups.
- Variable-font rendering needs further fidelity work.
- Canvas accessibility and WebGL2/other-browser coverage need further verification.
- Native pager templates omit their page-size popup; the sample composes an SDK
  Selector explicitly for page sizes 10, 25, and 50.
