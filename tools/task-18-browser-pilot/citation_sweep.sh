#!/usr/bin/env bash
# Family-wide citation sweep -- the driver behind the "N stems / M problems"
# figure every batch report quotes.
#
# COMMITTED IN BATCH 6B'S FIX ROUND 1 (M7). It had been re-improvised per batch
# and kept in a scratch directory, so the residual figure in each report was not
# reproducible from the tree -- which is U12's whole point ("any committed script
# must actually run from a clean checkout and must gate rather than merely
# report") applied to the one number the citation gate is judged by.
#
# Exits 1 when any stem reports a problem, and 2 when the population itself
# cannot be resolved, so it gates.
#
# WIREABLE INTO CI AS OF BATCH 7 (item 2). It used to exit 1 on a CLEAN tree,
# because `browser_generator_default_export_rejection.rs` carried seven bare
# `:N` header citations with no adjacent backticked construct -- disclosed, but
# with no disposition, and a gate nobody can wire in is a gate that will drift.
# The disposition taken is REWORD, not a red-list: all seven were rewordable
# (they cite `source.contains(...)` and `errors.iter().all(...)` constructs that
# were simply not named beside the number), so the artifact was made gateable
# rather than the gate made blind. A clean tree exits 0; the kill power is
# unchanged and is demonstrated in `batch5_crosscheck.py --selftest` plus the
# batch-7 report's poison runs (un-backticking a citation, drifting one off its
# construct, and stranding a red-list entry are all still exit 1).
#  * every case-file stem whose browser_<stem>.rs exists;
#  * every case file from a U2 SPLIT, whose stem differs from its source's --
#    the source is read out of the file's own `Migrated from tests/browser_X.rs`
#    header line, so nothing here is hardcoded;
#  * every whole-file retention (a browser_*.rs with a //! header, no case file).
# A stem whose .rs declares a PRE-TRIM REF resolves against that blob (ruling 9).
#
# ---------------------------------------------------------------------------
# PRECONDITION (batch 8): FULL GIT HISTORY.
#
# A case file whose source `.rs` is no longer in the tree resolves its citations
# against the historical blob, named by a `SOURCE REF:` line in its own header
# (see the SOURCE-DELETED arm below). That needs the referenced commit and its
# `crates/kali_cli/tests` tree to be present locally, so this script cannot run
# against a shallow clone. `.github/workflows/ci.yml` currently checks out with
# `actions/checkout`'s DEFAULT depth, which is shallow; wiring this sweep into
# CI therefore requires `with: fetch-depth: 0` on that step. That edit is a
# separate backlog item and is deliberately NOT made here -- changing what CI
# checks out changes what CI runs.
#
# Usage:
#   citation_sweep.sh                 # the sweep; exit 0/1/2
#   citation_sweep.sh --print-specs   # one `<stem> <PROVENANCE>` line per spec,
#                                     # the population only, no crosscheck. Used
#                                     # by source_ref_rehearsal.py to gate that
#                                     # `citation_tiers.py`'s second copy of this
#                                     # loop still builds the same population.
# ---------------------------------------------------------------------------
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO/crates/kali_cli/tests"
PRINT_SPECS=0
[[ "${1:-}" == "--print-specs" ]] && PRINT_SPECS=1
SPECS=()
KINDS=()
FAILURES=()
TMPDIR_=$(mktemp -d)
# Every exit path, including the hard `exit 2`s below: a `SOURCE REF:`
# reproduction is ~10MB, so a leaked scratch dir is no longer free.
trap 'rm -rf "$TMPDIR_"' EXIT

# `crates/kali_cli/tests` AS OF $1, materialised once per ref under $TMPDIR_ and
# echoed. THE WHOLE DIRECTORY, not just the one file, and that is the design
# decision -- see the SOURCE-DELETED arm for why.
ref_tree() {
  local ref=$1 dir="$TMPDIR_/ref-$ref"
  if [[ ! -d "$dir" ]]; then
    rm -rf "$dir.part"
    mkdir -p "$dir.part"
    git -C "$REPO" archive "$ref" crates/kali_cli/tests \
      | tar -x -C "$dir.part" --strip-components=3 || { rm -rf "$dir.part"; return 1; }
    mv "$dir.part" "$dir"
  fi
  printf '%s\n' "$dir"
}

for t in cases/browser/*.toml; do
  s=$(basename "$t" .toml)
  rs="browser_$s.rs"
  if [[ ! -f "$rs" ]]; then
    src=$(grep -m1 -oP '(?<=Migrated from tests/)browser_\S+\.rs' "$t" || true)
    if [[ -n "$src" && -f "$src" ]]; then
      SPECS+=("$s=$REPO/crates/kali_cli/tests/$src"); KINDS+=("$s SPLIT")
      continue
    fi
    # ------------------------------------------------------------------
    # SOURCE DELETED FROM THE TREE (batch 8). Resolve against the historical
    # blob, by the same route `PRE-TRIM REF:` already takes for a U4 trim.
    #
    # This arm used to be `SPECS+=("$s")` -- the bare stem, which reaches
    # `batch5_crosscheck.py`'s no-source branch and runs the GATEDNESS arm
    # alone. That cost nothing while the sourceless population carried no
    # resolvable citation between them; task 18's last step deletes the whole
    # `browser_*.rs` family in one commit, at which point the bare stem would
    # silently stop reading the family's citations altogether. So a missing
    # declaration is an ERROR here, not a fall-through: ruling 18's point is
    # that failure-to-match must not be indistinguishable from nothing-to-check,
    # and the state is DERIVED (`the .rs named by the header is not in the
    # tree`) rather than matched out of prose.
    #
    # WHY THE WHOLE `crates/kali_cli/tests` TREE IS MATERIALISED, not the one
    # blob: U10 targets keep their `#[test]` fns behind `#[path]` submodules in
    # a sibling directory, and the family deletion takes the sibling directory
    # with the carrier. `batch5_crosscheck.py` resolves a carrier's submodules
    # against a TREE base (fix round 1, I5) -- after the deletion there is no
    # tree base left, so every qualified `<file>.rs:N` citation in such a pair
    # would collapse into one loud "declares `#[path]` submodule(s) but none
    # could be resolved". Reproducing the directory the carrier sat in gives the
    # existing resolver its tree base back unchanged, rather than teaching the
    # resolver a second lookup mode. Reproducing only a directory NAMED AFTER
    # THE SOURCE would not have been enough -- a plain `mod x;` carrier's files
    # sit under a directory named after the MOD:
    #   $ python3 -c 'import sys; sys.path.insert(0, "tools/task-18-browser-pilot")
    #     from submodules import submodule_paths
    #     print(submodule_paths("crates/kali_cli/tests/browser_cdp_smoke.rs"))'
    #   [.../cdp_driver/mod.rs, .../cdp_driver/driver.rs, .../cdp_driver/protocol.rs]
    # `git archive` of the whole tests tree costs ~50ms and ~10MB per distinct
    # ref, and refs repeat across stems, so the cache below makes this cheaper
    # than per-file `git show` anyway.
    if [[ -z "$src" ]]; then
      FAILURES+=("$s: no browser_$s.rs in the tree and no \`Migrated from tests/<file>.rs\` line in $t, so the source its citations resolve against cannot even be named. Add the header line.")
      continue
    fi
    # READ OUT OF THE CASE FILE'S HEADER, not out of the file. The awk clause is
    # `batch5_crosscheck.py`'s `_header()` -- leading `#` lines, blank lines
    # skipped, stop at the first content line -- so a `#`-prefixed line inside a
    # `[source]` fixture body cannot declare a ref for the file that quotes it.
    # A file declaring MORE THAN ONE is an error rather than a silent
    # first-wins: two refs is an ambiguity about which blob the citations were
    # written against, and picking one hides the other.
    mapfile -t refs < <(awk 'substr($0,1,1)=="#"{print;next} NF{exit}' "$t" \
                        | grep -oP '(?<=SOURCE REF:)\s*\S+' | tr -d ' ')
    if ((${#refs[@]} > 1)); then
      FAILURES+=("$s: $t declares ${#refs[@]} \`SOURCE REF:\` lines (${refs[*]}). Which blob the citations were written against is then ambiguous; keep one.")
      continue
    fi
    ref=${refs[0]:-}
    if [[ -z "$ref" ]]; then
      FAILURES+=("$s: $src is absent from the tree and $t declares no \`SOURCE REF:\`. Every citation in this file is unreadable without one. Derive it: git log --diff-filter=D -1 --format=%H -- crates/kali_cli/tests/$src ; then git rev-parse <that>^ -- the ref names a commit where the file still EXISTS.")
      continue
    fi
    if [[ ! "$ref" =~ ^[0-9a-f]{40}$ ]]; then
      FAILURES+=("$s: \`SOURCE REF: $ref\` is not a full 40-char sha. A branch name or an abbreviation names a different commit as the branch moves or the repository grows; resolve it with git rev-parse.")
      continue
    fi
    if ! git -C "$REPO" rev-parse -q --verify "$ref^{commit}" >/dev/null 2>&1; then
      FAILURES+=("$s: \`SOURCE REF: $ref\` is not reachable in this repository. This sweep needs FULL history: in CI, actions/checkout must be given \`fetch-depth: 0\` (ci.yml's checkout is the default shallow one and cannot resolve it); locally, git fetch --unshallow.")
      continue
    fi
    if ! git -C "$REPO" cat-file -e "$ref:crates/kali_cli/tests/$src" 2>/dev/null; then
      FAILURES+=("$s: \`SOURCE REF: $ref\` resolves, but that commit does not contain crates/kali_cli/tests/$src. The ref must name a commit where the source still EXISTS -- the deletion commit's PARENT, not the deletion commit.")
      continue
    fi
    tree=$(ref_tree "$ref") || {
      FAILURES+=("$s: cannot materialise crates/kali_cli/tests at $ref (git archive | tar failed).")
      continue
    }
    # THE REF CONTAINS IT (checked above) AND THE REPRODUCTION HAS IT. Not the
    # same question, and the difference is not hypothetical: `ref_tree` caches
    # per ref, so a reproduction that is incomplete for ANY reason is reused by
    # every later stem sharing that ref. Without this, `batch5_crosscheck.py`
    # receives an override path that does not exist and silently falls into its
    # no-source branch -- gatedness arm only, the exact silent degradation this
    # arm was written to end. Found by mutating `ref_tree` to materialise a
    # single blob instead of the tree; the sweep reported `gatedness arm only`
    # for the second stem onward and nothing failed.
    if [[ ! -f "$tree/$src" ]]; then
      FAILURES+=("$s: $ref contains crates/kali_cli/tests/$src but the reproduction at $tree does not -- the materialisation is incomplete.")
      continue
    fi
    SPECS+=("$s=$tree/$src"); KINDS+=("$s SOURCEREF")
    continue
  fi
  ref=$(grep -oP '(?<=PRE-TRIM REF:)\s*\S+' "$rs" | head -1 | tr -d ' ')
  if [[ -n "$ref" ]]; then
    blob="$TMPDIR_/$s.rs"
    # The remedy is on this message too, because a shallow clone hits THIS arm
    # first (`PRE-TRIM REF:` precedes the SOURCE-DELETED arm in the glob) and
    # "cannot read <ref>" alone does not say that full history is the fix.
    git -C "$REPO" show "$ref:crates/kali_cli/tests/$rs" > "$blob" \
      || { echo "cannot read $ref:$rs -- this sweep needs FULL history: in CI, actions/checkout must be given \`fetch-depth: 0\`; locally, git fetch --unshallow"; exit 2; }
    SPECS+=("$s=$blob"); KINDS+=("$s PRETRIM")
  else
    SPECS+=("$s"); KINDS+=("$s TREE")
  fi
done
for rs in browser_*.rs; do
  s=${rs#browser_}; s=${s%.rs}
  [[ -f "cases/browser/$s.toml" ]] && continue
  grep -q '^//!' "$rs" || continue
  SPECS+=("$s"); KINDS+=("$s RETENTION")
done

if ((${#FAILURES[@]})); then
  echo
  echo "SWEEP CANNOT RESOLVE ITS POPULATION"
  for f in "${FAILURES[@]}"; do echo "  $f"; done
  echo "SWEEP EXIT=2"
  exit 2
fi

if ((PRINT_SPECS)); then
  printf '%s\n' "${KINDS[@]}"
  exit 0
fi

echo "sweep over ${#SPECS[@]} stems"
python3 "$REPO/tools/task-18-browser-pilot/batch5_crosscheck.py" --citations-only "${SPECS[@]}"
rc=$?
echo "SWEEP EXIT=$rc"
exit $rc
