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
# `--gates-only` IS A DEVELOPER COMMAND, RUN BY HAND. NOTHING IN CI INVOKES IT.
#
# This paragraph used to say `.github/workflows/ci.yml`'s `migration-gates` job
# invoked exactly `--gates-only`, and that the job was "the only checkout in
# `ci.yml` given `fetch-depth: 0`". Both claims are now false and are corrected
# here rather than left to rot:
#
#   * there is no `migration-gates` job. It was added on this branch and removed
#     again before merge because it could not pass on a runner -- it installed no
#     Rust toolchain, while two of the 14 gates run the compiled `kali` binary
#     and fail rather than skip when it is absent. `ci.yml` now carries a `NO
#     `migration-gates` JOB` comment recording that, and no job at all;
#   * `ci.yml` has no `fetch-depth: 0` anywhere. `release.yml:40` is the only
#     `fetch-depth: 0` in the repository.
#
# The depth requirement itself is unchanged and still real: `citation_sweep.sh`
# resolves a deleted source's citations against a historical blob, so it cannot
# run against a shallow clone. That is now a precondition on the human running
# the flag (a normal `git clone` satisfies it), not something a workflow
# arranges. The gate SET still lives here, in one place, which is why it is
# worth keeping the flag even with no CI caller: a set listed a second time in
# YAML would be a second copy to drift.
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
        # ADDED BY THE TASK 19 INSTRUMENT DISPATCH, and the reason is this
        # file's own header two paragraphs up: "a check nobody re-runs is
        # indistinguishable from a check that was deleted". This is the
        # regression suite for `audit-case-migration.py` -- the gate rule 3
        # calls ABSOLUTE, in which six separate bugs have been found, every
        # one by a human rather than by a script -- and NOTHING ran it.
        # Neither this file nor `.github/workflows/ci.yml` mentioned it:
        #
        #     $ grep -c audit-case-migration_test .github/workflows/ci.yml \
        #           scripts/test-gate.sh          # -> 0, 0 before this line
        #
        # It is stdlib `unittest`, needs no build, and takes under a second.
        # It also carries the corpus-wide census that pins the Task 19
        # `[constants]` fix ("no shipped case file has an unreferenced
        # constant"), so that figure is gated rather than recorded.
        "python3 $REPO/scripts/audit-case-migration_test.py"
        # ALSO ADDED BY THE TASK 19 INSTRUMENT DISPATCH. `families.py` derives
        # each family's source-filename prefix from that family's own case
        # files, and `citation_sweep.sh`, `verify_pair.sh`,
        # `batch5_crosscheck.py --family` and `source_ref_rehearsal.py --family`
        # all resolve their source paths through it. A browser run would notice
        # a broken derivation (the sweep exits 2), but no other family's would,
        # and batch 2 is ~47 targets across several families. Its selftest seeds
        # a known positive AND two poisons, per the rule that an instrument
        # validated in one direction only passes trivially.
        "python3 $PILOT/families.py --selftest"
        "python3 $PILOT/check_fixtures.py --argv-correspondence --census"
        "python3 $PILOT/batch5_crosscheck.py --selftest"
        "python3 $PILOT/find_fixture_self_inspection.py --selftest"
        "python3 $PILOT/classify_drift.py --selftest"
        "bash    $PILOT/citation_sweep.sh"
        "python3 $PILOT/inst2_probes.py"
        "python3 $PILOT/source_ref_rehearsal.py"
        # ADDED BY TASK 19 BATCH 2, on the controller's instruction, and for the
        # reason this file's header gives: a generator whose fixed point nobody
        # re-runs is a fixed point nobody checks -- ruling 15's own shape.
        # Its default mode is the CHECK direction: it re-renders all 17 of that
        # batch's case files from their spec and requires each to be
        # byte-identical to what is shipped, so a hand-edit to a generated case
        # file fails here instead of silently diverging from the mapping a
        # reviewer reads. It also requires every `EXPECTED-RED (rc=N)` paragraph
        # in those headers to AGREE with the gate it names (ruling 18 #3) --
        # which caught three headers claiming a red that had gone green.
        "python3 $REPO/tools/migration/gen_task19_batch2.py"
        # ADDED IN TASK 19 BATCH 2 FIX ROUND 2, and the reason is that batch's
        # own report claimed it was already here and it was not:
        #
        #     $ grep -c screen_candidates scripts/test-gate.sh   # -> 0 before this line
        #
        # `screen_candidates.py` is the instrument every remaining batch draws
        # its work list from, and its `--selftest` carries both directions of
        # ground truth (a self-documented retention must not score CLEAN; an
        # already-migrated target must still score CLEAN) plus the retention
        # cross-check that closes the `runtime_monomorphize` masquerade, with
        # its own known positive. All of that was re-run by whoever remembered.
        "python3 $REPO/tools/migration/screen_candidates.py --selftest"
        # ADDED BY TASK 19 BATCH 3, for the reason the batch-2 entry above gives
        # and one more. Its default mode is the CHECK direction: it re-renders
        # all 7 of that batch's case files -- 123 cases, one per source `#[test]`
        # fn -- from their spec and requires each to be byte-identical to what is
        # shipped, and requires every `EXPECTED-RED (rc=N)` paragraph to AGREE
        # with the gate it names (ruling 18 #3).
        #
        # The extra reason: batch 3's fidelity argument is that
        # `t19b3_extract.claims_of` RAISES on an assertion shape it does not
        # model rather than skipping it, so every claim in those 123 cases is
        # derived rather than hand-listed. That guarantee is only worth
        # anything while something re-runs the derivation against the sources --
        # which is what this line is. A source that grows an assertion shape the
        # extractor cannot model now fails HERE.
        #
        # This takes the gate set from 11 to 12. Batch 3's brief asked for
        # `--gates-only` 11/11 AND for the batch's generator to be wired into
        # the lane; those two are not simultaneously satisfiable, and the batch
        # took the wiring, which is the requirement about its own work. The
        # arithmetic is 11 + 1 = 12 and is stated in that batch's report.
        "python3 $REPO/tools/migration/gen_task19_batch3.py"
        # ADDED BY TASK 19 BATCH 4, for the same reason as the line above and
        # with one addition of its own. `gen_task19_batch4.py`'s default mode is
        # the CHECK direction: it re-derives all nine of its case files from
        # `t19b4_extract.py` and requires the shipped bytes to match, so a source
        # that grows a construct outside the extractor's closed statement,
        # expression or CLAIM language fails HERE rather than migrating short.
        # It also re-runs all five of ruling 19's gates and requires every
        # EXPECTED-RED declaration in every header to agree with the gate it
        # names, in BOTH directions -- an undeclared red and a declared red that
        # has gone green are both failures.
        #
        # It further RE-RUNS the real binary for both of the batch's rule-11
        # cross-stream resolutions, so a pin that says "the needle is on stderr"
        # cannot go stale silently. That is the one gate in this list that needs
        # a built `kali`; it degrades to a loud failure, never to a skip.
        #
        # This takes the gate set from 12 to 13. The arithmetic is 12 + 1 = 13
        # and is stated in batch 4's report; the count moves by construction
        # whenever a batch wires its generator in, which is what batch 3 flagged.
        "python3 $REPO/tools/migration/gen_task19_batch4.py"
        # ADDED BY TASK 19 BATCH 5, the last migration batch, for the same
        # reason and with three additions of its own.
        # `gen_task19_batch5.py`'s default mode is the CHECK direction: it
        # re-derives all seven of its case files from `t19b5_extract.py` and
        # requires the shipped bytes to match, so a source that grows a
        # construct outside the extractor's closed CLAIM language fails HERE
        # rather than migrating short. It re-runs all five of ruling 19's gates
        # and requires every EXPECTED-RED declaration to agree with the gate it
        # names, in BOTH directions -- including the THREE-COLUMN red-list in
        # the one U4 trim's retained `.rs` header, every cell of which is a live
        # run rather than a prediction.
        #
        # The three additions:
        #   * it RE-RUNS the real binary for all eighteen of this batch's
        #     rule-11 disjunction resolutions and raises if a cell moves branch,
        #     or if BOTH disjuncts ever hold (ruling 17's tie, which it refuses
        #     to break silently). Like batch 4's, this needs a built `kali` and
        #     degrades to a loud failure, never to a skip;
        #   * it RE-MEASURES the ground for declining the four `array_from_*`
        #     spec §5.11 retentions -- every `json_output` branch still
        #     unreachable -- so a source that grew a reachable call site fails
        #     here instead of staying silently declined;
        #   * it runs ruling 10's `find_fixture_self_inspection.py` over all
        #     seven targets and requires zero UNADJUDICATED instances.
        #
        # This takes the gate set from 13 to 14. The arithmetic is 13 + 1 = 14
        # and is stated in batch 5's report; the count moves by construction
        # whenever a batch wires its generator in.
        "python3 $REPO/tools/migration/gen_task19_batch5.py"
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
