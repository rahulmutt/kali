#!/usr/bin/env bash
# Full-workspace test gate: enumerate every failing test (never stop at the
# first red binary) and fail unless the count is zero.
#
# Parses the bare `failures:` summary lists (reliable under parallel output
# interleaving, unlike per-test `... FAILED` lines).
#
# ---------------------------------------------------------------------------
# `--gates-only` (batch 8-inst-2) RUNS THE TASK 18 MIGRATION GATES INSTEAD, and
# a BARE INVOCATION IS UNCHANGED FROM BASE -- same cargo suites, same runtime,
# same output shape. That is not a style choice: the plan's Global Constraints
# name this file as one of four that must not be modified, and ~50 plan
# documents call the bare command as their per-task gate and assume its cost.
# The human partner's ruling for this dispatch was "keep the wiring, but a bare
# `bash scripts/test-gate.sh` must behave exactly as it did before", so the
# migration gates are OPT-IN and reachable only through the flag.
# `scripts/check-determinism.sh` and `mise.toml` remain untouched and still
# under the original prohibition.
#
# Why the wiring exists at all: until batch 8-inst-2 this script matched none of
# those gates --
#
#     $ grep -c "citation_sweep\|batch5_crosscheck\|check_fixtures\|classify_drift" \
#           scripts/test-gate.sh
#     0
#
# -- and neither did anything under `.github/`. Eight batches built them and
# every one ran by hand or not at all, which is the same failure class as a
# figure with no command beside it: a check nobody re-runs is indistinguishable
# from a check that was deleted (ruling 15).
#
# `.github/workflows/ci.yml`'s `migration-gates` job invokes exactly
# `--gates-only`, so the gate SET lives here, in one place, rather than being
# listed a second time in YAML where the two copies would drift. That job is
# also the only checkout in `ci.yml` given `fetch-depth: 0`, because
# `citation_sweep.sh` resolves a deleted source's citations against a historical
# blob and `actions/checkout` is shallow by default.
#
# WHAT IS DELIBERATELY NOT IN THE SET. `classify_drift.py` with no arguments
# (the census: run all 14 generators and require each to be a fixed point) needs
# a built `libkali_common` rlib and REWRITES `cases/browser/` while it runs,
# refusing to start unless that directory is clean. Wiring it in would make the
# flag unusable to the one person most likely to pass it -- someone with an edit
# open in exactly that directory. Its `--selftest` is here; the census stays a
# deliberate, clean-tree invocation.
# ---------------------------------------------------------------------------
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PILOT="$REPO/tools/task-18-browser-pilot"
MODE="${1:-tests}"

run_gates() {
    local fail=0 rc
    # Each entry runs from a clean checkout, needs no build, and GATES (U12).
    # The order is cheapest-first so a broken corpus is reported in seconds.
    local -a gates=(
        "python3 $PILOT/check_fixtures.py --argv-correspondence --census"
        "python3 $PILOT/batch5_crosscheck.py --selftest"
        "python3 $PILOT/find_fixture_self_inspection.py --selftest"
        "python3 $PILOT/classify_drift.py --selftest"
        "bash    $PILOT/citation_sweep.sh"
        "python3 $PILOT/inst2_probes.py"
        "python3 $PILOT/source_ref_rehearsal.py"
    )
    for g in "${gates[@]}"; do
        printf '\n=== MIGRATION GATE: %s ===\n' "$g"
        ( cd "$REPO" && $g )
        rc=$?
        echo "exit=$rc"
        (( rc )) && fail=1
    done
    if (( fail )); then
        echo "GATE FAILED — a Task 18 migration gate exited non-zero"
        return 1
    fi
    echo "MIGRATION GATES OK"
    return 0
}

run_tests() {
    local log status failures
    log="$(mktemp)"
    trap 'rm -f "$log"' RETURN

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
        return 1
    fi

    if [ "$status" -ne 0 ]; then
        echo "GATE FAILED — cargo test exited $status with no parsed failures (build error?). Full log:"
        tail -n 40 "$log"
        return 1
    fi

    echo "GATE OK: 0 failing tests"
    return 0
}

case "$MODE" in
    # No argument: EXACTLY what this script did at BASE, and nothing else.
    tests)        run_tests || exit 1 ;;
    --gates-only) run_gates || exit 1 ;;
    *)            echo "usage: test-gate.sh [--gates-only]"; exit 2 ;;
esac
