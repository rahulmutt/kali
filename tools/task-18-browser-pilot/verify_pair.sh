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
# --rs AND --companion EXIST FOR A SOURCE SPLIT ACROSS SEVERAL CASE FILES.
# U2 forces one `.rs` into two or more `.toml`s whenever a fixture's presence is
# a case's whole point (a `kali.json` manifest, a lockfile, a sibling module) --
# `[source]` is file-wide, so a `[[case]]` cannot opt out of one. When that
# happens the case files no longer share a stem with the `.rs` (--rs names it),
# and the two LITERAL-COVERAGE arms have to see the whole set at once
# (--companion adds them): auditing one half alone reports the other half's
# argv tokens and fixtures as dropped claims. Batch 3 ran that joint audit by
# hand for `bundle_cjs_source_classes` + `_inherited`; this makes it a flag.
# The other six arms are per-file by construction and take only this file.
#
# Usage: verify_pair.sh <stem> [--rs <stem>] [--companion <stem>]...
#                              [--pretrim <ref>] [--structure] [--allow-empty]
#   stem: e.g. math_asinh_acosh_atanh_identities
#         -> browser_<stem>.rs  vs  cases/browser/<stem>.toml
#   --rs <stem>: take the source from browser_<stem>.rs instead (for a case file
#         whose stem differs from its source's, i.e. a U2 split).
#   --companion <stem>: cases/browser/<stem>.toml is part of the same migration
#         from the same source; passed to the AUDIT and FIXTURE arms so their
#         left-hand side is the whole migrated set. Repeatable.
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
RS_STEM=""
COMPANIONS=()
XCHECK_FLAGS=(--citations-only)
CC_FLAGS=()
while (( $# )); do
  case "$1" in
    --pretrim)   PRETRIM="${2:?--pretrim needs a git ref}"; shift 2 ;;
    --rs)        RS_STEM="${2:?--rs needs a stem}"; shift 2 ;;
    --companion) COMPANIONS+=("${2:?--companion needs a stem}"); shift 2 ;;
    --structure) XCHECK_FLAGS=(); shift ;;
    *)           CC_FLAGS+=("$1"); shift ;;
  esac
done

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TESTS="$REPO/crates/kali_cli/tests"
RS="$TESTS/browser_${RS_STEM:-$STEM}.rs"
TOML="$TESTS/cases/browser/$STEM.toml"

[[ -f "$TOML" ]] || { echo "missing $TOML"; exit 2; }

# --------------------------------------------------------------------------
# SOURCE DELETED FROM THE TREE -- DELEGATE, DO NOT RE-RESOLVE (batch 8-inst-2).
#
# All eight arms take a `.rs` path. Once a source is deleted there is none, and
# this script used to stop at `missing <path>` and exit 2 -- measured at BASE,
# that is already the state of 23 of the 128 case-file stems (their sources went
# in batches 6B/7), and 8C deletes the rest of the family in one commit. So the
# tool that verifies a migrated pair would be unrunnable on every migrated pair,
# at exactly the moment 8C needs it.
#
# WHAT IT DELEGATES TO, AND WHY IT DOES NOT RE-IMPLEMENT IT. `citation_tiers.py`
# already resolves this, per stem, for the sweep: working-tree `.rs`, a
# `PRE-TRIM REF:` blob for a U4 trim, a U2 split's `Migrated from` source, or the
# `SOURCE REF:` reproduction of the whole `crates/kali_cli/tests` tree (whole
# tree, because a U10 `#[path]` carrier's submodules must still resolve beside
# it). `8-inst-1` established SEVEN live readers of the pre-trim ref; a bash
# eighth that re-parsed the headers here could disagree with the gate about which
# blob a pair's `:N` citations mean, and each reader would look locally correct.
# `--resolve-source` hands back the string `sweep_specs` would have built, so
# there is one resolver and this file holds none.
#
# The scratch dir is OURS: `--into` tells the resolver to materialise there and
# not clean up, because the eight arms run after it exits.
# --------------------------------------------------------------------------
SCRATCH=""
cleanup() { [[ -n "$SCRATCH" ]] && rm -rf "$SCRATCH"; }
trap cleanup EXIT
RESOLVED_PROV=""
if [[ ! -f "$RS" ]]; then
  SCRATCH="$(mktemp -d -t "verify_pair_${STEM}_XXXXXX")"
  if ! line=$(python3 "$REPO/tools/task-18-browser-pilot/citation_tiers.py" \
                --resolve-source "$STEM" --into "$SCRATCH"); then
    echo "cannot resolve a source for $STEM: $line"; exit 2
  fi
  read -r _stem RESOLVED_PROV RESOLVED_REF RESOLVED_NAME RS <<< "$line"
  echo "browser_${RS_STEM:-$STEM}.rs is not in the tree; resolved $RESOLVED_NAME"
  echo "  as $RESOLVED_PROV at ref $RESOLVED_REF -> $RS"
  [[ -f "$RS" ]] || { echo "resolver returned $RS, which is not a file"; exit 2; }
  # A NON-MATCH IS AN ERROR (ruling 18 #3). `--rs` is the caller's claim about
  # which source this case file was migrated from; the resolver derives the same
  # thing from the case file's own `Migrated from` line. If they disagree, one of
  # them is verifying the wrong pair and there is no way to tell which, so stop.
  if [[ -n "$RS_STEM" && "$RESOLVED_NAME" != "browser_$RS_STEM.rs" ]]; then
    echo "--rs $RS_STEM says browser_$RS_STEM.rs, the case file's own header"
    echo "resolves to $RESOLVED_NAME -- these cannot both be this pair's source"
    exit 2
  fi
fi
[[ -f "$RS"   ]] || { echo "missing $RS"; exit 2; }

# Case files migrated from the same source, for the two arms whose subject is
# the whole migration rather than this one file.
JOINT=("$TOML")
for c in ${COMPANIONS[@]+"${COMPANIONS[@]}"}; do
  ct="$TESTS/cases/browser/$c.toml"
  [[ -f "$ct" ]] || { echo "missing companion $ct"; exit 2; }
  JOINT+=("$ct")
done

fail=0
note() { printf '\n=== %s ===\n' "$1"; }

note "TRIALS  browser/$STEM"
cargo_out=$(cd "$REPO" && cargo test -p kali_cli --test cases -- "browser/$STEM" 2>&1)
rc=$?; echo "$cargo_out" | grep -E "^test result" || echo "$cargo_out" | tail -3
(( rc )) && fail=1

note "AUDIT   (rule 3 -- absolute)"
( cd "$TESTS" && python3 "$REPO/scripts/audit-case-migration.py" "$RS" "${JOINT[@]}" )
rc=$?; echo "audit exit=$rc"; (( rc )) && fail=1

note "COMMENT COVERAGE (rule 12)"
python3 "$REPO/tools/task-18-browser-pilot/comment_coverage.py" "${CC_FLAGS[@]+"${CC_FLAGS[@]}"}" "$RS" "$TOML"
rc=$?; echo "comment_coverage exit=$rc"; (( rc )) && fail=1

note "U8 (rationale prose is audited by nothing -- check its own citations)"
python3 "$REPO/tools/task-18-browser-pilot/check_rationale_fn_names.py" "$RS" "$TOML"
rc=$?; echo "check_rationale_fn_names exit=$rc"; (( rc )) && fail=1

note "FIXTURES (rule 9 -- every program text survives verbatim)"
python3 "$REPO/tools/task-18-browser-pilot/check_fixtures.py" "$RS" "${JOINT[@]}"
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
# A --rs split pair has no browser_<case stem>.rs, so the citation gate is given
# the real source through the same `=PATH` override --pretrim uses. A pair whose
# source was resolved out of the tree above needs the same override, and for the
# same reason: without it the gate falls into its no-source branch and runs the
# GATEDNESS arm alone -- green, and reading none of this pair's citations.
[[ -n "$RS_STEM" || -n "$RESOLVED_PROV" ]] && xcheck_spec="$STEM=$RS"
if [[ -n "$PRETRIM" ]]; then
  # THE BLOB IS TAKEN FROM THE SOURCE STEM, NOT THE CASE STEM (fix round 1, I5).
  # This used to read browser_$STEM.rs unconditionally, so `--rs` + `--pretrim`
  # together looked for a `.rs` named after the CASE file -- which, for a U2
  # split, is exactly the file that does not exist -- and exited 2. No shipped
  # pair combines the two flags today; batches 7-8 meet `#[path]` carriers with
  # retentions, which is that combination.
  pretrim_stem="${RS_STEM:-$STEM}"
  pretrim_rs="$(mktemp -t "verify_pair_${STEM}_pretrim_XXXXXX.rs")"
  if git -C "$REPO" show "$PRETRIM:crates/kali_cli/tests/browser_$pretrim_stem.rs" > "$pretrim_rs" 2>/dev/null; then
    xcheck_spec="$STEM=$pretrim_rs"
    echo "resolving case-file citations against pre-trim ref $PRETRIM (browser_$pretrim_stem.rs)"
  else
    # Do NOT fall back to the working tree: that would silently run the very
    # comparison the --pretrim flag exists to avoid, and report its artefacts as
    # real drift. Fail loudly instead.
    echo "cannot read browser_$pretrim_stem.rs at ref $PRETRIM"; rm -f "$pretrim_rs"; exit 2
  fi
fi
python3 "$REPO/tools/task-18-browser-pilot/batch5_crosscheck.py" \
  "${XCHECK_FLAGS[@]+"${XCHECK_FLAGS[@]}"}" "$xcheck_spec"
rc=$?; echo "batch5_crosscheck exit=$rc"; (( rc )) && fail=1
[[ -n "${pretrim_rs:-}" ]] && rm -f "$pretrim_rs"

printf '\n==== %s: %s ====\n' "$STEM" "$( ((fail)) && echo 'ATTENTION -- a gate exited non-zero' || echo 'gates exit 0' )"
exit $fail
