#!/usr/bin/env bash
# Exercise runner failure handling and filtering without building or launching apps.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
export TEST_ALL_LOG_DIR="$tmp/logs" RUNNER_CALLS="$tmp/calls"
: > "$RUNNER_CALLS"
cargo() {
    printf '%s\n' "$*" >> "$RUNNER_CALLS"
    printf 'test output stays visible\n'
    [[ " $* " == *' --no-fail-fast '* ]] || return 98
    if [[ " $* " == *' -p xgpui-lab '* && " $* " == *' --lib '* ]]; then
        printf 'Lab has no library target.\n' >&2
        return 97
    fi
    case " $* " in
        *' --doc '*) return "${FAKE_DOC_STATUS:-0}" ;;
        *' -p xgpui-luma '*) return "${FAKE_SDK_STATUS:-0}" ;;
        *) return 0 ;;
    esac
}
export -f cargo
bash scripts/test.sh --list > "$tmp/list"
[[ ! -s "$RUNNER_CALLS" && ! -e "$TEST_ALL_LOG_DIR" ]]
[[ $(wc -l < "$tmp/list") -eq 10 ]]

# A failure in an early suite must survive later successes.
export FAKE_SDK_STATUS=101 FAKE_DOC_STATUS=0
status=0
bash scripts/test.sh > "$tmp/early" 2>&1 || status=$?
[[ "$status" -eq 1 && $(wc -l < "$RUNNER_CALLS") -eq 10 ]]
grep -q '^FAIL  sdk .*exit 101' "$tmp/early"
grep -q '^PASS  doc-tests ' "$tmp/early"
grep -q '^1 of 10 suites failed\.' "$tmp/early"
grep -q '^test output stays visible' "$tmp/early"
grep -q '^test output stays visible' "$TEST_ALL_LOG_DIR/sdk.log"

export FAKE_SDK_STATUS=0 FAKE_DOC_STATUS=101
status=0
bash scripts/test.sh > "$tmp/late" 2>&1 || status=$?
[[ "$status" -eq 1 ]]
grep -q '^FAIL  doc-tests .*exit 101' "$tmp/late"

export FAKE_SDK_STATUS=101
status=0
bash scripts/test.sh > "$tmp/both" 2>&1 || status=$?
[[ "$status" -eq 1 ]]
grep -q '^2 of 10 suites failed\.' "$tmp/both"

export FAKE_SDK_STATUS=0 FAKE_DOC_STATUS=0
bash scripts/test.sh > "$tmp/pass" 2>&1
grep -q '^0 of 10 suites failed\.' "$tmp/pass"

: > "$RUNNER_CALLS"
bash scripts/test.sh --only look --offline > "$tmp/filter" 2>&1
[[ $(wc -l < "$RUNNER_CALLS") -eq 2 ]]
grep -q -- '--offline' "$RUNNER_CALLS"
grep -q '^0 of 2 suites failed\.' "$tmp/filter"
for args in '--only' '--only missing-suite'; do
    status=0
    # Intentional splitting: exercise both missing-value and no-match errors.
    bash scripts/test.sh $args > "$tmp/invalid" 2>&1 || status=$?
    [[ "$status" -eq 2 ]]
done
printf 'Runner self-test passed: listing, filtering, logs, live output, argument forwarding, early/late/combined failures.\n'
