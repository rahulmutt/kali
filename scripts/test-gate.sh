#!/usr/bin/env bash
# Full-workspace test gate: enumerate every failing test (never stop at the
# first red binary) and fail unless the count is zero.
#
# Parses the bare `failures:` summary lists (reliable under parallel output
# interleaving, unlike per-test `... FAILED` lines).
set -uo pipefail

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

cargo test --workspace --no-fail-fast >"$log" 2>&1
status=$?

failures="$(awk '
    /^failures:$/ { collecting = 1; next }
    collecting && /^    [A-Za-z_]/ { print $1; next }
    collecting { collecting = 0 }
' "$log" | sort -u)"

if [ -n "$failures" ]; then
    echo "GATE FAILED — failing tests:"
    echo "$failures"
    exit 1
fi

if [ "$status" -ne 0 ]; then
    echo "GATE FAILED — cargo test exited $status with no parsed failures (build error?). Full log:"
    tail -n 40 "$log"
    exit 1
fi

echo "GATE OK: 0 failing tests"
