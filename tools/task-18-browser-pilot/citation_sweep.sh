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
# Exits 1 when any stem reports a problem, so it gates.
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
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO/crates/kali_cli/tests"
SPECS=()
TMPDIR_=$(mktemp -d)
for t in cases/browser/*.toml; do
  s=$(basename "$t" .toml)
  rs="browser_$s.rs"
  if [[ ! -f "$rs" ]]; then
    src=$(grep -m1 -oP '(?<=Migrated from tests/)browser_\S+\.rs' "$t" || true)
    # NO SOURCE IN THE TREE (deleted after its migration shipped): pass the stem
    # anyway. `batch5_crosscheck.py` runs the GATEDNESS arm on it, which needs no
    # source. This used to `continue` -- 23 of 104 case files skipped outright,
    # with two ungated citations hiding in them (batch 7 fix round 1, I2).
    if [[ -n "$src" && -f "$src" ]]; then
      SPECS+=("$s=$REPO/crates/kali_cli/tests/$src")
    else
      SPECS+=("$s")
    fi
    continue
  fi
  ref=$(grep -oP '(?<=PRE-TRIM REF:)\s*\S+' "$rs" | head -1 | tr -d ' ')
  if [[ -n "$ref" ]]; then
    blob="$TMPDIR_/$s.rs"
    git -C "$REPO" show "$ref:crates/kali_cli/tests/$rs" > "$blob" || { echo "cannot read $ref:$rs"; exit 2; }
    SPECS+=("$s=$blob")
  else
    SPECS+=("$s")
  fi
done
for rs in browser_*.rs; do
  s=${rs#browser_}; s=${s%.rs}
  [[ -f "cases/browser/$s.toml" ]] && continue
  grep -q '^//!' "$rs" || continue
  SPECS+=("$s")
done
echo "sweep over ${#SPECS[@]} stems"
python3 "$REPO/tools/task-18-browser-pilot/batch5_crosscheck.py" --citations-only "${SPECS[@]}"
rc=$?
rm -rf "$TMPDIR_"
echo "SWEEP EXIT=$rc"
exit $rc
