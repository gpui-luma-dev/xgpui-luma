#!/usr/bin/env bash
# Rust suites only: live output, per-suite logs, and a final failure summary.
# Usage: just test [--list] [--only <name>] [Cargo options...]
# Override the log directory with TEST_ALL_LOG_DIR.
set -uo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." || exit 1
root=$PWD
case "$(uname -s)" in
    Darwin) os=macos ;;
    Linux) os=linux ;;
    MINGW* | MSYS* | CYGWIN*) os=windows ;;
    *) printf 'Unsupported test host: %s\n' "$(uname -s)" >&2; exit 2 ;;
esac

list_only=0
only=''
cargo_args=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --list) list_only=1 ;;
        --only)
            if [[ $# -lt 2 || -z "$2" || "$2" == --* ]]; then
                printf '%s\n' '--only requires a suite name or substring.' >&2
                exit 2
            fi
            only=$2
            shift
            ;;
        --help|-h)
            printf 'Usage: just test [--list] [--only <suite substring>] [Cargo options...]\n'
            exit 0
            ;;
        *) cargo_args+=("$1") ;;
    esac
    shift
done

names=()
modes=()
packages=()
suite() {
    [[ -z "$only" || "$1" == *"$only"* ]] || return 0
    names+=("$1")
    modes+=("$2")
    packages+=("$3")
}
suite release-tool tests 'luma-release'
suite sdk tests 'xgpui-luma'
suite color tests 'xgpui-luma-color'
suite look-shadcn tests 'xgpui-luma-look-shadcn'
suite look-radix tests 'xgpui-luma-look-radix'
suite plot tests 'xgpui-luma-plot'
suite common tests 'luma-app-common luma-shell-common luma-docs-embed'
suite lab bins 'xgpui-lab'
suite sample-apps tests 'luma-studio luma-radix-studio luma-plot-viz luma-neumorphic-demo luma-sdk-perf luma-shell-detached luma-shell-split-titlebar luma-shell-vscode luma-shell-2026'
suite doc-tests docs ''
if [[ ${#names[@]} -eq 0 ]]; then
    printf 'No test suites match "%s" on %s.\n' "$only" "$os" >&2
    exit 2
fi

logs=${TEST_ALL_LOG_DIR:-target/test-logs/luma}
case "$logs" in /* | [A-Za-z]:*) ;; *) logs="$root/$logs" ;; esac
if [[ "$list_only" -eq 0 ]]; then
    mkdir -p "$logs" || exit 1
fi
results=()
failed=0
start_all=$SECONDS
for i in "${!names[@]}"; do
    name=${names[$i]}
    command=(cargo test --locked --all-features --no-fail-fast)
    if [[ "${modes[$i]}" == docs ]]; then
        command+=(--workspace --doc)
    else
        if [[ "${modes[$i]}" == tests ]]; then command+=(--lib); fi
        command+=(--bins --tests)
        for package in ${packages[$i]}; do command+=(-p "$package"); done
    fi
    if [[ ${#cargo_args[@]} -gt 0 ]]; then command+=("${cargo_args[@]}"); fi
    if [[ "$list_only" -eq 1 ]]; then
        printf '%-24s ' "$name"
        printf '%q ' "${command[@]}"
        printf '\n'
        continue
    fi
    log="$logs/$name.log"
    printf '\n==> [%d/%d] %s\n    ' "$((i + 1))" "${#names[@]}" "$name"
    printf '%q ' "${command[@]}"
    printf '\n'
    start=$SECONDS
    "${command[@]}" 2>&1 | tee "$log"
    statuses=("${PIPESTATUS[@]}")
    status=${statuses[0]}
    # A failed log write must not be reported as a passing suite.
    if [[ "$status" -eq 0 && "${statuses[1]}" -ne 0 ]]; then status=${statuses[1]}; fi
    elapsed=$((SECONDS - start))
    if [[ "$status" -eq 0 ]]; then
        results+=("$(printf 'PASS  %-24s %5ss  %s' "$name" "$elapsed" "$log")")
    else
        results+=("$(printf 'FAIL  %-24s %5ss  (exit %s, %s)' "$name" "$elapsed" "$status" "$log")")
        failed=$((failed + 1))
    fi
done
[[ "$list_only" -eq 0 ]] || exit 0
printf '\n==== Test summary (%s, %ss) ====\n' "$os" "$((SECONDS - start_all))"
printf '%s\n' "${results[@]}"
printf '%d of %d suites failed.\n' "$failed" "${#names[@]}"
[[ "$failed" -eq 0 ]]
