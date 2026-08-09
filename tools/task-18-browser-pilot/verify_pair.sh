#!/usr/bin/env bash
# Per-pair verification for Task 18 batch 4.
#
# Runs, for one migrated pair, the checks the brief requires and reports each
# one's exit status explicitly -- the point is that a green here is necessary
# and NOT sufficient (U11), so every check's own output is printed rather than
# collapsed into a single OK.
#
#   1. the case file's trials actually run
#   2. audit-case-migration.py       (rule 3: absolute gate)
#   3. comment_coverage.py           (rule 12)
#   4. check_rationale_fn_names.py   (U8: rationale prose is audited by nothing)
#   5. check_fixtures.py             (rule 9: program text survives verbatim)
#   6. check_extra_claims.py         (U14 `extra` / rule 2: never invent)
#   7. fidelity.py                   (U14: BOTH directions printed; report only)
#   8. batch5_crosscheck.py          (ruling 11: every `:N` citation resolves)
#
# That list said "four" and named only 1-4 even after batch 4 added two gates and
# batch 6 added a seventh; corrected in batch 6's fix round 1. Eight steps, of
# which seven enforce -- fidelity.py always exits 0 and is a report, its
# enforcing counterpart being check_extra_claims.py.
#
# STEP 8 WAS WIRED IN BY BATCH 6, and the reason is not tidiness. Ruling 11
# exempts `:N` code citations from "no figure an edit can move" ONLY because
# they are mechanically gated -- "a pointer nothing re-resolves is a figure in
# disguise". Until now the citation gate ran when someone remembered to run it,
# which made that exemption unearned; it had already caught seven stale
# citations that hand-verification missed, and wiring it immediately surfaced
# three more (two adjudicated retentions whose header citations were shifted by
# a retroactive batch-5 edit, and one that had been silently wrong for longer).
#
# --pretrim <ref> IS MANDATORY FOR A U4 TRIM-AND-KEEP RETENTION PAIR. Every `:N`
# in such a case file is a PRE-TRIM line number, so resolving it against the
# post-trim working tree reports failures that are artefacts of the trim -- the
# exact confusion ruling 9 exists to prevent. Each retained `.rs` names its own
# ref in its CONSEQUENCE FOR THE GATES block; use that one. Measured across the
# nine retention pairs in this family: all nine exit 0 against their own ref,
# and five of the nine are red without it.
#
# Usage: verify_pair.sh <stem> [--pretrim <ref>] [--structure] [--allow-empty]
#   stem: e.g. math_asinh_acosh_atanh_identities
#         -> browser_<stem>.rs  vs  cases/browser/<stem>.toml
#   --pretrim <ref>: resolve the case file's citations against
#         `<ref>:crates/kali_cli/tests/browser_<stem>.rs` instead of the working
#         tree. The retention header's OWN citations are always resolved against
#         the working tree regardless -- it describes the shipped file.
#   --structure: also run the batch-5 header-section-order arm. Off by default,
#         because pilot/batch-2/3/4 headers predate those section names and
#         would fail an arm that is about batch-5-and-later house style, not
#         about correctness.
#   remaining flags are passed to comment_coverage.py (e.g. --allow-empty).
set -uo pipefail

STEM="${1:?usage: verify_pair.sh <stem> [--pretrim <ref>] [--structure] [--allow-empty]}"
shift || true

PRETRIM=""
XCHECK_FLAGS=(--citations-only)
CC_FLAGS=()
while (( $# )); do
  case "$1" in
    --pretrim)   PRETRIM="${2:?--pretrim needs a git ref}"; shift 2 ;;
    --structure) XCHECK_FLAGS=(); shift ;;
    *)           CC_FLAGS+=("$1"); shift ;;
  esac
done

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

note "CITATIONS (ruling 11 -- :N is exempt ONLY because it is mechanically gated)"
xcheck_spec="$STEM"
if [[ -n "$PRETRIM" ]]; then
  pretrim_rs="$(mktemp -t "verify_pair_${STEM}_pretrim_XXXXXX.rs")"
  if git -C "$REPO" show "$PRETRIM:crates/kali_cli/tests/browser_$STEM.rs" > "$pretrim_rs" 2>/dev/null; then
    xcheck_spec="$STEM=$pretrim_rs"
    echo "resolving case-file citations against pre-trim ref $PRETRIM"
  else
    # Do NOT fall back to the working tree: that would silently run the very
    # comparison the --pretrim flag exists to avoid, and report its artefacts as
    # real drift. Fail loudly instead.
    echo "cannot read browser_$STEM.rs at ref $PRETRIM"; rm -f "$pretrim_rs"; exit 2
  fi
fi
python3 "$REPO/tools/task-18-browser-pilot/batch5_crosscheck.py" \
  "${XCHECK_FLAGS[@]+"${XCHECK_FLAGS[@]}"}" "$xcheck_spec"
rc=$?; echo "batch5_crosscheck exit=$rc"; (( rc )) && fail=1
[[ -n "${pretrim_rs:-}" ]] && rm -f "$pretrim_rs"

printf '\n==== %s: %s ====\n' "$STEM" "$( ((fail)) && echo 'ATTENTION -- a gate exited non-zero' || echo 'gates exit 0' )"
exit $fail
