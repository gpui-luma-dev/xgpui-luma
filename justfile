# Run from the repository root. Windows: put Git for Windows Bash on PATH.
set shell := ["bash", "-cu"]
set windows-shell := ["bash", "-cu"]
set positional-arguments

# App launches and explicit builds default to release; checks/tests use Cargo defaults.
profile := "release"
python := if os() == "windows" { "python" } else { "python3" }
run-options := "--locked --profile " + profile
lib-packages := "-p xgpui-luma -p xgpui-luma-color -p xgpui-luma-look-radix -p xgpui-luma-look-shadcn -p xgpui-luma-plot"

# List development commands without launching an app.
default:
    @just --justfile "{{ justfile() }}" --list

# --- Apps ---

# Run Luma Studio; accepts a theme and --light/--dark.
app-studio *args:
    cargo run {{ run-options }} -p luma-studio -- "$@"

# Run Radix Studio.
app-radix *args:
    cargo run {{ run-options }} -p luma-radix-studio -- "$@"

# Run Plot Viz.
app-plot *args:
    cargo run {{ run-options }} -p luma-plot-viz -- "$@"

# Run the Neumorphic Demo.
app-neumorphic *args:
    cargo run {{ run-options }} -p luma-neumorphic-demo -- "$@"

# Run Luma Studio with panic backtraces enabled.
app-studio-backtrace *args:
    RUST_BACKTRACE=1 cargo run {{ run-options }} -p luma-studio -- "$@"

# --- Shells (optional theme; defaults to default) ---

# Run the detached shell.
shell-detached *args:
    cargo run {{ run-options }} -p luma-shell-detached -- "$@"

# Run the split-titlebar shell.
shell-split-titlebar *args:
    cargo run {{ run-options }} -p luma-shell-split-titlebar -- "$@"

# Run the VS Code-style shell.
shell-vscode *args:
    cargo run {{ run-options }} -p luma-shell-vscode -- "$@"

# Run the 2026 shell.
shell-2026 *args:
    cargo run {{ run-options }} -p luma-shell-2026 -- "$@"

# --- Profiling ---

# Run Luma Studio with the macOS Metal HUD enabled.
perf-studio *args:
    MTL_HUD_ENABLED=1 cargo run {{ run-options }} -p luma-studio -- "$@"

# Run the native SDK fixture: <scene> <look> <size> [--drive|--idle].
perf-sdk *args:
    cargo run {{ run-options }} -p luma-sdk-perf --features native -- "$@"

# --- SDK / Libraries ---

# Check the default SDK/library workspace members.
sdk-check *args:
    cargo check --locked "$@"

# Build the default SDK/library workspace members.
sdk-build *args:
    cargo build {{ run-options }} "$@"

# Run the library suites selected by CI, excluding benchmarks and demo binaries.
sdk-test *args:
    @{{ python }} -B scripts/quiet_check.py cargo test --locked --lib --bins --tests --all-features -p xgpui-luma -p xgpui-luma-color -p xgpui-luma-plot -p xgpui-luma-look-radix -p xgpui-luma-look-shadcn -p luma-app-common "$@"

# Lint default SDK/library members with every target and feature.
sdk-clippy *args:
    cargo clippy --locked --all-targets --all-features "$@"

# Run documentation tests for the library packages selected by CI.
sdk-doc-test *args:
    @{{ python }} -B scripts/quiet_check.py cargo test --locked --doc --all-features -p xgpui-luma -p xgpui-luma-look-radix -p xgpui-luma-look-shadcn -p xgpui-luma-plot "$@"

# --- Workspace ---

# Check all workspace targets and features, including the native profiling app.
workspace-check *args:
    cargo check --locked --workspace --all-targets --all-features "$@"

# Build the full workspace, including the native profiling app.
workspace-build *args:
    cargo build {{ run-options }} --workspace --all-features "$@"

# Run workspace unit/integration tests, excluding benchmark executables.
workspace-test *args:
    @{{ python }} -B scripts/quiet_check.py cargo test --locked --workspace --lib --bins --tests --all-features "$@"

# Lint the full workspace with the same targets and features as CI.
workspace-clippy *args:
    cargo clippy --locked --workspace --all-targets --all-features "$@"

# Build all native app binaries without launching them.
apps-build *args:
    cargo build {{ run-options }} --bins -p luma-studio -p luma-radix-studio -p luma-plot-viz -p luma-neumorphic-demo -p luma-shell-detached -p luma-shell-split-titlebar -p luma-shell-vscode -p luma-shell-2026 -p luma-sdk-perf -p xgpui-lab --features luma-sdk-perf/native "$@"

# --- Formatting ---

# Format Rust source across the workspace.
fmt:
    cargo fmt --all

# Check Rust formatting without modifying files.
fmt-check:
    cargo fmt --all --check

# --- Documentation / Supporting Checks ---

# Build and package the embedded WASM demos (profile: release or dev).
docs-build:
    sh apps/docs-embed/build.sh "{{ profile }}"

# Compile the control documentation examples using the Python/Rust checker.
docs-check: _control-docs-check

# Serve the documentation locally without opening a browser.
docs-serve port="8766":
    {{ python }} -m http.server "$1" --bind 127.0.0.1 --directory docs

# Test repository and embedded-demo Python helpers.
scripts-check:
    @{{ python }} -B scripts/quiet_check.py {{ python }} -B -m unittest discover -s scripts -p 'test_*.py'
    @{{ python }} -B scripts/quiet_check.py {{ python }} -B -m unittest discover -s apps/docs-embed -p 'test_*.py'

# Run optional workflow lint when actionlint is already available.
workflows-check:
    @if command -v actionlint >/dev/null; then actionlint; else echo "Skipped optional workflow lint: actionlint is not available."; fi

# Lint the WASM demos (requires the wasm32-unknown-unknown target).
wasm-check:
    env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS cargo clippy --locked -p luma-docs-embed --target wasm32-unknown-unknown -- -D warnings

# --- Aggregate Checks ---

# Run the local equivalents of CI formatting, script, lint and library/doc checks.
ci: fmt-check scripts-check workspace-clippy sdk-test sdk-doc-test _control-docs-check

# Check a PR without launching apps or building deployment artifacts.
pre-pr: ci workflows-check docs-check wasm-check
    git diff --check

# Check release changes before committing; clean candidates also get final archive verification.
preflight-release: _preflight-source-check _release-registry-check ci
    #!/usr/bin/env bash
    set -euo pipefail
    release_package_id=$(cargo pkgid --locked --package xgpui-luma)
    VERSION=${release_package_id##*@}
    GITHUB_SHA=$(git rev-parse HEAD)
    export VERSION GITHUB_SHA
    export LIB_PACKAGES="xgpui-luma xgpui-luma-color xgpui-luma-look-radix xgpui-luma-look-shadcn xgpui-luma-plot"
    export PUBLISH_RELEASE=false
    if [[ -n "$(git status --porcelain)" ]]; then
        cargo package --locked --allow-dirty --registry crates-io {{ lib-packages }}
        echo "Local pre-flight checks passed for uncommitted changes."
        echo "Final release archive verification is pending: commit changes and rerun just preflight-release."
        exit 0
    fi
    cargo package --locked --registry crates-io {{ lib-packages }}
    # With PUBLISH_RELEASE=false this only verifies archives, including already published versions.
    {{ python }} -B scripts/release.py publish
    echo "Release pre-flight passed. Publishing remains a separate action."

# Build and verify the release Pages artifact locally; never deploy or open a browser.
preflight-pages: _preflight-source-check fmt-check scripts-check workflows-check docs-check wasm-check
    just --justfile "{{ justfile() }}" _wasm-bindgen-check
    sh apps/docs-embed/build.sh release
    {{ python }} -B apps/docs-embed/package_runtime.py --check
    test -s docs/CNAME
    git diff --check
    @echo "Pages pre-flight passed. Deployment remains a separate action."

# --- Utilities ---

# Count Rust source lines (requires tokei).
loc:
    tokei --types Rust

# --- Internal Prerequisites ---

_release-registry-check:
    #!/usr/bin/env bash
    set -euo pipefail
    release_package_id=$(cargo pkgid --locked --package xgpui-luma)
    VERSION=${release_package_id##*@}
    GITHUB_SHA=$(git rev-parse HEAD)
    export VERSION GITHUB_SHA
    export LIB_PACKAGES="xgpui-luma xgpui-luma-color xgpui-luma-look-radix xgpui-luma-look-shadcn xgpui-luma-plot"
    PUBLISH_RELEASE=false {{ python }} -B scripts/release.py preflight --require-unpublished

_preflight-source-check:
    #!/usr/bin/env {{ python }}
    import json
    import subprocess
    import sys
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1"], text=True))
    members = set(metadata["workspace_members"])
    overrides = [p["name"] for p in metadata["packages"] if p["id"] not in members and not (p.get("source") or "").startswith("registry+")]
    if overrides:
        sys.exit("Pre-flight requires registry dependencies; remove local/git overrides: " + ", ".join(sorted(overrides)))

_wasm-bindgen-check:
    #!/usr/bin/env {{ python }}
    import shutil
    import subprocess
    import sys
    import tomllib
    if not shutil.which("wasm-bindgen"):
        sys.exit("Source checks completed; Pages artifact verification is blocked because wasm-bindgen is unavailable.")
    with open("Cargo.lock", "rb") as lockfile:
        expected = next(p["version"] for p in tomllib.load(lockfile)["package"] if p["name"] == "wasm-bindgen")
    actual = subprocess.check_output(["wasm-bindgen", "--version"], text=True).split()[-1]
    if actual != expected:
        sys.exit(f"Source checks completed; Pages artifact verification requires wasm-bindgen {expected}, but found {actual}.")

_control-docs-check:
    {{ python }} -B scripts/check_control_docs.py

# --- XGPUI Lab ---

# Run XGPUI Lab.
app-lab *args:
    cargo run {{ run-options }} -p xgpui-lab -- "$@"

# Build the lab without launching it.
lab-build:
    cargo build {{ run-options }} -p xgpui-lab

# Run the lab headless tests.
lab-test:
    cargo test --locked -p xgpui-lab --all-targets --features test-support

# Lint the lab and its test targets.
lab-clippy:
    cargo clippy --locked -p xgpui-lab --all-targets --features test-support -- -D warnings

# --- XGPUI Lab: Parley text field (apps/xgpui-lab/src/parley_lab) ---

# Open the standalone Parley window on the text field tabs.
lab-parley-field *args:
    cargo run {{ run-options }} -p xgpui-lab -- --native-text-backend=parley --text-field "$@"

# Run the whole lab on the Parley text system (xgpui-parley › Text Field).
lab-parley *args:
    cargo run {{ run-options }} -p xgpui-lab -- --text-system=parley "$@"

# Run the Parley text field headless tests; extra args filter or pass through.
lab-parley-test *args:
    cargo test --locked -p xgpui-lab --features test-support --bin xgpui-lab text_field "$@"

# Run the ignored Parley text field performance measurements in release.
lab-parley-perf:
    cargo test --release --locked -p xgpui-lab --features test-support --bin xgpui-lab measure_ -- --ignored --nocapture

lab-chromaticity *args:
    cargo run {{ run-options }} -p xgpui-lab --bin chromaticity-lab -- "$@"

lab-coverage-report:
    {{ python }} apps/xgpui-lab/scripts/render_objc2_coverage.py

# Explicit native GUI regressions; run only when app launches are intended.
lab-regression-tests:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p artifacts/regression-tests
    output=$(mktemp -d "$PWD/artifacts/regression-tests/$(date -u +%Y%m%dT%H%M%SZ)-XXXXXX")
    printf 'Regression results: %s\n' "$output"
    cargo build --locked -p xgpui-lab 2>&1 | tee "$output/build.log"
    status=0
    ./target/debug/xgpui-lab --media-benchmark atlas --output "$output/atlas" 2>&1 | tee "$output/atlas.log" || status=1
    ./target/debug/xgpui-lab --gamut-benchmark texture --output "$output/gamut" 2>&1 | tee "$output/gamut.log" || status=1
    # The video workload includes the independent native PaintSurface assertions.
    ./target/debug/xgpui-lab --media-benchmark video --output "$output/native-surface" 2>&1 | tee "$output/native-surface.log" || status=1
    ./target/debug/xgpui-lab --media-benchmark lifecycle --output "$output/lifecycle" 2>&1 | tee "$output/lifecycle.log" || status=1
    printf 'Regression results: %s\n' "$output"
    exit "$status"
