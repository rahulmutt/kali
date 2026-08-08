#!/usr/bin/env bash
# Per-pair verification for Task 18 batch 4.
#
# Runs, for one migrated pair, the checks the brief requires and reports each
# one's exit status explicitly -- the point is that a green here is necessary
# and NOT sufficient (U11), so every check's own output is printed rather than
# collapsed into a single OK.
#
#   1. the case file's trials actually run
#   2. audit-case-migration.py  (rule 3: absolute gate)
#   3. comment_coverage.py      (rule 12)
#   4. fidelity.py              (U14: BOTH directions printed)
#
# Usage: verify_pair.sh <stem> [--allow-empty]
#   stem: e.g. math_asinh_acosh_atanh_identities
#         -> browser_<stem>.rs  vs  cases/browser/<stem>.toml
set -uo pipefail

STEM="${1:?usage: verify_pair.sh <stem> [--allow-empty]}"
shift || true
CC_FLAGS=("$@")

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TESTS="$REPO/crates/kali_cli/tests"
RS="$TESTS/browser_$STEM.rs"
TOML="$TESTS/cases/browser/$STEM.toml"

[[ -f "$RS"   ]] || { echo "missing $RS"; exit 2; }
[[ -f "$TOML" ]] || { echo "missing $TOML"; exit 2; }

fail=0
note() { printf '\n=== %s ===\n' "$1"; }

note "TRIALS  browser/$STEM"
cargo_out=$(cd "$REPO" && cargo test -p kali_cli --test cases -- "browser/$STEM" 2>&1)
rc=$?; echo "$cargo_out" | grep -E "^test result" || echo "$cargo_out" | tail -3
(( rc )) && fail=1

note "AUDIT   (rule 3 -- absolute)"
( cd "$TESTS" && python3 "$REPO/scripts/audit-case-migration.py" "$RS" "$TOML" )
rc=$?; echo "audit exit=$rc"; (( rc )) && fail=1

note "COMMENT COVERAGE (rule 12)"
python3 "$REPO/tools/task-18-browser-pilot/comment_coverage.py" "${CC_FLAGS[@]+"${CC_FLAGS[@]}"}" "$RS" "$TOML"
rc=$?; echo "comment_coverage exit=$rc"; (( rc )) && fail=1

note "U8 (rationale prose is audited by nothing -- check its own citations)"
python3 "$REPO/tools/task-18-browser-pilot/check_rationale_fn_names.py" "$RS" "$TOML"
rc=$?; echo "check_rationale_fn_names exit=$rc"; (( rc )) && fail=1

note "FIXTURES (rule 9 -- every program text survives verbatim)"
python3 "$REPO/tools/task-18-browser-pilot/check_fixtures.py" "$RS" "$TOML"
rc=$?; echo "check_fixtures exit=$rc"; (( rc )) && fail=1

note "EXTRA CLAIMS (U14 extra direction -- rule 2, never invent)"
python3 "$REPO/tools/task-18-browser-pilot/check_extra_claims.py" "$RS" "$TOML"
rc=$?; echo "check_extra_claims exit=$rc"; (( rc )) && fail=1

note "FIDELITY (U14 -- raw string diff, BOTH directions, NOT truncated)"
# Fix round 1 (I6): this used to be `| head -4`, which discarded the entire
# EXTRA section -- U14: "a checker that computes `extra` and discards it has
# disabled the gate that catches inventions". fidelity.py is a report and always
# exits 0, so its status is recorded but the ENFORCING gate is
# check_extra_claims.py above; this stays for the raw both-directions view.
fidelity_out=$(python3 "$REPO/tools/task-18-browser-pilot/fidelity.py" "$RS" -- "$TOML")
fidelity_rc=$?
echo "$fidelity_out" | grep -E "^(source claims|MISSING \(|EXTRA \()" || true
echo "fidelity exit=$fidelity_rc (report only; enforcement is check_extra_claims)"
(( fidelity_rc )) && fail=1

printf '\n==== %s: %s ====\n' "$STEM" "$( ((fail)) && echo 'ATTENTION -- a gate exited non-zero' || echo 'gates exit 0' )"
exit $fail
