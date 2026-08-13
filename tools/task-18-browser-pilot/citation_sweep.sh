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
#   citation_sweep.sh --print-specs   # one line per spec, the population only,
#                                     # no crosscheck. Each line is
#                                     #   <stem> <PROVENANCE> <ref> <source file>
#                                     # where the last two are FACTS about what
#                                     # was resolved -- which sha, whose text --
#                                     # and PROVENANCE is DERIVED from them by
#                                     # `provenance()` rather than stamped on at
#                                     # each construction site (ruling 18 #1).
#                                     # `source_ref_rehearsal.py` diffs this
#                                     # against `citation_tiers.py --specs`, so
#                                     # the two loops cannot agree on a label
#                                     # while disagreeing about which blob they
#                                     # read.
# ---------------------------------------------------------------------------
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TESTS="$REPO/crates/kali_cli/tests"
cd "$TESTS"
PRINT_SPECS=0
# --------------------------------------------------------------------------
# THE FAMILY (Task 19 instruments, §2). Default `browser`, so a bare
# invocation -- which is what `test-gate.sh --gates-only` and
# `.github/workflows/ci.yml` make -- sweeps exactly the population it always
# did, byte for byte. `--family <name>` moves the case-file glob, the source
# filename and the `Migrated from` grep together.
#
# THE PREFIX IS ASKED FOR, NOT ASSUMED (`families.py`): `misc/`'s sources carry
# no family prefix at all, so `<family>_` is wrong for a family that already
# exists. A family whose case files disagree about their prefix makes
# `families.py` exit non-zero, and so does this.
# --------------------------------------------------------------------------
FAMILY=browser
FAMILY_ARG=()          # empty unless --family was given; see below
ARGS=()
while (( $# )); do
  case "$1" in
    --print-specs) PRINT_SPECS=1; shift ;;
    --family)      FAMILY="${2:?--family needs a name}"; FAMILY_ARG=(--family "$FAMILY"); shift 2 ;;
    --family=*)    FAMILY="${1#*=}"; FAMILY_ARG=(--family "$FAMILY"); shift ;;
    *)             ARGS+=("$1"); shift ;;
  esac
done
if ((${#ARGS[@]})); then
  echo "citation_sweep.sh: unrecognised argument(s): ${ARGS[*]}" >&2
  echo "usage: citation_sweep.sh [--family <name>] [--print-specs]" >&2
  exit 2
fi
PREFIX=$(python3 "$REPO/tools/task-18-browser-pilot/families.py" --prefix "$FAMILY") || exit 2
CASES_REL="cases/$FAMILY"
[[ -d "$CASES_REL" ]] || { echo "citation_sweep.sh: no $TESTS/$CASES_REL" >&2; exit 2; }
SPECS=()
RESOLVED=()
FAILURES=()
VALIDATED=0
TMPDIR_=$(mktemp -d)
# Every exit path, including the hard `exit 2`s below: a `SOURCE REF:`
# reproduction is ~10MB, so a leaked scratch dir is no longer free.
trap 'rm -rf "$TMPDIR_"' EXIT
# SEPARATE SUBDIRECTORIES, and not for tidiness. A `--pretrim` blob and a
# `SOURCE REF:` reproduction used to sit side by side in one scratch dir, which
# put `$TMPDIR_/x.rs` where a pre-trim carrier's `mod x;` could in principle
# resolve it. Neither can now see the other's files (minor 6).
mkdir -p "$TMPDIR_/pretrim" "$TMPDIR_/refs"

# Materialise a PRE-TRIM blob AND, for a `#[path]` carrier, its pre-trim sibling
# submodule directory. The blob alone is not enough for a carrier: its `#[path]`
# declarations resolve relative to the blob's own directory, and a U4 trim that
# MIGRATED some submodules has deleted them from the tree, so every qualified
# `<file>.rs:N` citation in the pair is reported as naming a non-submodule -- a
# correct pair turned red by an artefact of the materialisation. Same
# reproduction the `SOURCE REF:` arm does for a deleted source, applied to a
# trimmed one. The submodule list is enumerated from the BLOB, so a submodule
# the trim removed is still found.
materialise_pretrim() {
  local ref="$1" rel="$2" out="$3"
  git -C "$REPO" show "$ref:crates/kali_cli/tests/$rel" > "$out" \
    || { echo "cannot read $ref:$rel -- this sweep needs FULL history: in CI, actions/checkout must be given \`fetch-depth: 0\`; locally, git fetch --unshallow"; exit 2; }
  local sub
  for sub in $(grep -oP '(?<=#\[path = ")[^"]+(?="\])' "$out" 2>/dev/null); do
    mkdir -p "$(dirname "$out")/$(dirname "$sub")"
    git -C "$REPO" show "$ref:crates/kali_cli/tests/$sub" \
      > "$(dirname "$out")/$sub" 2>/dev/null \
      || { echo "cannot read $ref:$sub -- the pre-trim submodule a qualified citation needs"; exit 2; }
  done
}

# The case file's HEADER: leading `#` lines, blank lines skipped, stop at the
# first content line. This is `batch5_crosscheck.py`'s `_header()`, so a
# `#`-prefixed line inside a `[source]` fixture body is not part of it.
header_of() { awk 'substr($0,1,1)=="#"{print;next} NF{exit}' "$1"; }

# `crates/kali_cli/tests` AS OF $1, materialised once per ref and echoed.
# THE WHOLE DIRECTORY, not just the one file -- see the SOURCE-DELETED arm.
ref_tree() {
  local ref=$1 dir="$TMPDIR_/refs/$ref"
  if [[ ! -d "$dir" ]]; then
    rm -rf "$dir.part"
    mkdir -p "$dir.part"
    git -C "$REPO" archive "$ref" crates/kali_cli/tests \
      | tar -x -C "$dir.part" --strip-components=3 || { rm -rf "$dir.part"; return 1; }
    mv "$dir.part" "$dir"
  fi
  printf '%s\n' "$dir"
}

# Is $2 a commit this repository has, and does it carry crates/kali_cli/tests/$3?
# Appends to FAILURES and returns 1 if not.
ref_carries() {
  local stem=$1 ref=$2 name=$3
  if ! git -C "$REPO" rev-parse -q --verify "$ref^{commit}" >/dev/null 2>&1; then
    FAILURES+=("$stem: \`SOURCE REF: $ref\` is not reachable in this repository. This sweep needs FULL history: in CI, actions/checkout must be given \`fetch-depth: 0\` (ci.yml's checkout is the default shallow one and cannot resolve it); locally, git fetch --unshallow.")
    return 1
  fi
  if ! git -C "$REPO" cat-file -e "$ref:crates/kali_cli/tests/$name" 2>/dev/null; then
    FAILURES+=("$stem: \`SOURCE REF: $ref\` resolves, but that commit does not contain crates/kali_cli/tests/$name. The ref must name a commit where the source still EXISTS -- the deletion commit's PARENT, not the deletion commit.")
    return 1
  fi
  return 0
}

# ---------------------------------------------------------------------------
# VALIDATE A DECLARATION WHILE ITS SOURCE IS STILL HERE TO VALIDATE IT AGAINST.
#
# The SOURCE-DELETED arm can only ever check that the ref EXISTS and carries the
# path -- on deletion day there is nothing left to compare the blob to. But the
# intended workflow is DECLARE FIRST, DELETE LATER, so every declaration passes
# through a window in which the source is still in the tree and the ref is
# cheaply falsifiable by content. A ref naming an older revision of the same
# file would satisfy every existence check and silently shift every `:N` in the
# case file on the day the source went away.
#
# So: whenever a case file declares a `SOURCE REF:` AND the source its citations
# resolve against is still available, the declared blob is compared BYTE FOR
# BYTE against that source. "The source its citations resolve against" is the
# gate's own answer, not the working-tree file: for a U4 trim it is the
# `PRE-TRIM REF:` blob, because that is what every `:N` in the case file was
# written against and the trimmed tree file is not.
check_ref_content() {
  local stem=$1 ref=$2 name=$3 path=$4
  [[ -z "$ref" ]] && return 0
  ref_carries "$stem" "$ref" "$name" || return 1
  if ! git -C "$REPO" cat-file blob "$ref:crates/kali_cli/tests/$name" | cmp -s - "$path"; then
    FAILURES+=("$stem: \`SOURCE REF: $ref\` names a commit whose crates/kali_cli/tests/$name DIFFERS from the source this case file's citations resolve against today. Existence is not enough: every \`:N\` here would shift silently on the day the source is deleted. Compare with: git diff $ref -- crates/kali_cli/tests/$name")
    return 1
  fi
  VALIDATED=$((VALIDATED + 1))
  return 0
}

# PROVENANCE IS DERIVED, NOT STAMPED (ruling 18 #1). Its inputs are the two
# facts recorded beside every spec -- the ref whose blob the gate will read (or
# `-`) and the source file whose text it will read (or `-`) -- plus the state of
# the tree. Nothing decides it at the point a spec is appended, so the two
# population loops cannot agree on a label while resolving different things.
provenance() {
  local stem=$1 ref=$2 name=$3
  if [[ "$name" == "-" ]]; then echo RETENTION
  elif [[ ! -f "$TESTS/$name" ]]; then echo SOURCEREF
  elif [[ "$ref" != "-" ]]; then echo PRETRIM
  elif [[ "$name" == "$PREFIX$stem.rs" ]]; then echo TREE
  else echo SPLIT
  fi
}

for t in "$CASES_REL"/*.toml; do
  s=$(basename "$t" .toml)
  rs="$PREFIX$s.rs"
  # NOT ANCHORED ON THE FAMILY PREFIX, and deliberately so: this grep exists to
  # find a U2 SPLIT's source, whose stem differs from the case file's. Anchoring
  # it on `$PREFIX` would still work for browser but would silently return
  # nothing for a `misc/` split (empty prefix, `\S+` already covers it) --
  # matching `families.MIGRATED_FROM` keeps one answer to "which source".
  src=$(grep -m1 -oP '(?<=Migrated from tests/)[A-Za-z0-9_]+(/[A-Za-z0-9_]+)*\.rs' "$t" || true)

  # THE DECLARATION IS READ FOR EVERY CASE FILE, not only for a sourceless one.
  # Round 1 read it only inside the SOURCE-DELETED arm, which made a declaration
  # on a case file whose `.rs` is still present completely inert -- unparsed and
  # unchecked -- for the whole declare-first-delete-later window. Declaring MORE
  # THAN ONE is an error rather than a silent first-wins: which blob the
  # citations were written against would then be ambiguous.
  mapfile -t refs < <(header_of "$t" | grep -oP '(?<=SOURCE REF:)\s*\S+' | tr -d ' ')
  if ((${#refs[@]} > 1)); then
    FAILURES+=("$s: $t declares ${#refs[@]} \`SOURCE REF:\` lines (${refs[*]}). Which blob the citations were written against is then ambiguous; keep one.")
    continue
  fi
  ref=${refs[0]:-}
  if [[ -n "$ref" && ! "$ref" =~ ^[0-9a-f]{40}$ ]]; then
    FAILURES+=("$s: \`SOURCE REF: $ref\` is not a full 40-char sha. A branch name or an abbreviation names a different commit as the branch moves or the repository grows; resolve it with git rev-parse.")
    continue
  fi

  if [[ -f "$rs" ]]; then
    pt=$(grep -oP '(?<=PRE-TRIM REF:)\s*\S+' "$rs" | head -1 | tr -d ' ')
    if [[ -n "$pt" ]]; then
      blob="$TMPDIR_/pretrim/$s.rs"
      # The remedy is on this message too, because a shallow clone hits THIS arm
      # first (`PRE-TRIM REF:` precedes the SOURCE-DELETED arm in the glob) and
      # "cannot read <ref>" alone does not say that full history is the fix.
      materialise_pretrim "$pt" "$rs" "$blob"
      SPECS+=("$s=$blob"); RESOLVED+=("$s $pt $rs")
      check_ref_content "$s" "$ref" "$rs" "$blob"
    else
      SPECS+=("$s"); RESOLVED+=("$s - $rs")
      check_ref_content "$s" "$ref" "$rs" "$TESTS/$rs"
    fi
    continue
  fi
  if [[ -n "$src" && -f "$src" ]]; then
    # THE NAMED SOURCE MAY ITSELF BE A TRIMMED CARRIER (batch 8A). A U2 split
    # names one source from two case files, so the stem here is the CASE FILE's
    # (`reflect_own_keys_explicit_api`) and never matches a `.rs`; this arm
    # resolves it by the `Migrated from` name. If that file declares a
    # `PRE-TRIM REF:`, every `:N` in the case file is a PRE-TRIM number
    # (ruling 9) and the live file is the wrong side to resolve against -- for a
    # `#[path]` carrier it also resolves only the RETAINED submodules, so
    # citations into the migrated ones look like bad names. Take the same
    # pre-trim route the same-named arm above takes.
    srcpt=$(grep -oP '(?<=PRE-TRIM REF:)\s*\S+' "$TESTS/$src" | head -1 | tr -d ' ')
    if [[ -n "$srcpt" ]]; then
      srcblob="$TMPDIR_/pretrim/$s.rs"
      materialise_pretrim "$srcpt" "$src" "$srcblob"
      SPECS+=("$s=$srcblob"); RESOLVED+=("$s $srcpt $src")
      check_ref_content "$s" "$ref" "$src" "$srcblob"
      continue
    fi
    SPECS+=("$s=$TESTS/$src"); RESOLVED+=("$s - $src")
    check_ref_content "$s" "$ref" "$src" "$TESTS/$src"
    continue
  fi
  # ------------------------------------------------------------------
  # SOURCE DELETED FROM THE TREE (batch 8). Resolve against the historical
  # blob, by the same route `PRE-TRIM REF:` already takes for a U4 trim.
  #
  # This arm used to be `SPECS+=("$s")` -- the bare stem, which reaches
  # `batch5_crosscheck.py`'s no-source branch and runs the GATEDNESS arm alone.
  # That cost nothing while the sourceless population carried no resolvable
  # citation between them; task 18's last step deletes the whole `browser_*.rs`
  # family in one commit, at which point the bare stem would silently stop
  # reading the family's citations altogether. So a missing declaration is an
  # ERROR here, not a fall-through: ruling 18's point is that failure-to-match
  # must not be indistinguishable from nothing-to-check, and the state is
  # DERIVED (`the .rs named by the header is not in the tree`) rather than
  # matched out of prose.
  #
  # WHY THE WHOLE `crates/kali_cli/tests` TREE IS MATERIALISED, not the one
  # blob: U10 targets keep their `#[test]` fns behind `#[path]` submodules in a
  # sibling directory, and the family deletion takes the sibling directory with
  # the carrier. `batch5_crosscheck.py` resolves a carrier's submodules against
  # a TREE base (fix round 1, I5) -- after the deletion there is no tree base
  # left, so a qualified `<file>.rs:N` citation in such a pair would collapse
  # into one loud "declares `#[path]` submodule(s) but none could be resolved".
  # Reproducing the directory the carrier sat in gives the existing resolver its
  # tree base back unchanged, rather than teaching the resolver a second lookup
  # mode. Reproducing only a directory NAMED AFTER THE SOURCE would not have
  # been enough -- a plain `mod x;` carrier's files sit under a directory named
  # after the MOD:
  #   $ python3 -c 'import sys; sys.path.insert(0, "tools/task-18-browser-pilot")
  #     from submodules import submodule_paths
  #     print(submodule_paths("crates/kali_cli/tests/browser_cdp_smoke.rs"))'
  #   [.../cdp_driver/mod.rs, .../cdp_driver/driver.rs, .../cdp_driver/protocol.rs]
  # `git archive` of the whole tests tree costs ~50ms and ~10MB per distinct
  # ref, and refs repeat across stems, so the cache makes this cheaper than
  # per-file `git show` anyway.
  if [[ -z "$src" ]]; then
    FAILURES+=("$s: no $PREFIX$s.rs in the tree and no \`Migrated from tests/<file>.rs\` line in $t, so the source its citations resolve against cannot even be named. Add the header line.")
    continue
  fi
  if [[ -z "$ref" ]]; then
    FAILURES+=("$s: $src is absent from the tree and $t declares no \`SOURCE REF:\`. Every citation in this file is unreadable without one. Derive it: git log --diff-filter=D -1 --format=%H -- crates/kali_cli/tests/$src ; then git rev-parse <that>^ -- the ref names a commit where the file still EXISTS.")
    continue
  fi
  ref_carries "$s" "$ref" "$src" || continue
  tree=$(ref_tree "$ref") || {
    FAILURES+=("$s: cannot materialise crates/kali_cli/tests at $ref (git archive | tar failed).")
    continue
  }
  # THE REF CONTAINS IT (checked above) AND THE REPRODUCTION HAS IT. Not the
  # same question, and the difference is not hypothetical: `ref_tree` caches per
  # ref, so a reproduction that is incomplete for ANY reason is reused by every
  # later stem sharing that ref. Without this, `batch5_crosscheck.py` receives an
  # override path that does not exist and silently falls into its no-source
  # branch -- gatedness arm only, the exact silent degradation this arm was
  # written to end. Found by mutating `ref_tree` to materialise a single blob
  # instead of the tree; the sweep reported `gatedness arm only` for the second
  # stem onward and nothing failed.
  if [[ ! -f "$tree/$src" ]]; then
    FAILURES+=("$s: $ref contains crates/kali_cli/tests/$src but the reproduction at $tree does not -- the materialisation is incomplete.")
    continue
  fi
  SPECS+=("$s=$tree/$src"); RESOLVED+=("$s $ref $src")
done
# WHOLE-FILE RETENTIONS: a `$PREFIX*.rs` carrying a `//!` header, with no case
# file of its own.
#
# AN EMPTY PREFIX MAKES THIS GLOB THE WHOLE DIRECTORY, and the case-file test
# below is NOT enough to keep it honest: `browser_promise_any_bundle.rs` is a
# browser retention with a `//!` header and no `cases/misc/*.toml`, so a
# `--family misc` sweep would adopt it. A gate that silently widens its
# population when pointed at a different family is the failure this whole
# generalisation exists to avoid, so the other families' prefixes are subtracted
# explicitly. `families.py --list` is the one place they are derived, and a
# family whose prefix is itself empty contributes nothing to subtract (there is
# at most one such family by construction -- two would make every unprefixed
# `.rs` ambiguous, which is a corpus problem, not a gate problem).
# WITH AN EMPTY PREFIX THE ARM IS REFUSED, NOT WIDENED (ruling 18 #3). Filtering
# out the other families' prefixes is not enough and this was measured, not
# assumed: `--family misc --print-specs` with that filter in place adopted TEN
# unprefixed `.rs` files as "retentions" -- `arena_reclamation_runtime.rs`,
# `closure_return_isolation.rs`, `inprocess.rs`, and `cases.rs`, which is the
# HARNESS ITSELF. Every one of them is an unmigrated target whose `//!` is
# ordinary module documentation, not a U3 retention header. There is no fact in
# the tree that distinguishes them, so the sweep says so and checks the case
# files only, rather than guessing and quietly sweeping a wider population than
# its banner claims.
if [[ -z "$PREFIX" ]]; then
  RETENTION_ARM="SKIPPED"
else
  RETENTION_ARM="ran"
fi
for rs in ${PREFIX}*.rs; do
  [[ -z "$PREFIX" ]] && break
  [[ -f "$rs" ]] || continue
  s=${rs#"$PREFIX"}; s=${s%.rs}
  [[ -f "$CASES_REL/$s.toml" ]] && continue
  grep -q '^//!' "$rs" || continue
  SPECS+=("$s"); RESOLVED+=("$s - -")
done

if ((${#FAILURES[@]})); then
  echo
  echo "SWEEP CANNOT RESOLVE ITS POPULATION"
  for f in "${FAILURES[@]}"; do echo "  $f"; done
  echo "SWEEP EXIT=2"
  exit 2
fi

if ((PRINT_SPECS)); then
  for r in "${RESOLVED[@]}"; do
    read -r st rf nm <<< "$r"
    printf '%s %s %s %s\n' "$st" "$(provenance "$st" "$rf" "$nm")" "$rf" "$nm"
  done
  # THE POPULATION THE SWEEP PROPER WOULD PASS TO THE CROSSCHECK, printed from
  # `SPECS` while the loop above printed from `RESOLVED`. Two arrays appended in
  # lockstep by every arm, and nothing checked that they stay that way; now
  # `source_ref_rehearsal.population_agreement` compares this figure against the
  # `sweep over N stems` banner AND against the number of lines above, so a stem
  # that reaches one array and not the other is a failure rather than a silently
  # shorter sweep. It also turns citation_tiers.py's "its printed stem count must
  # equal that script's banner" from a sentence into a comparison.
  printf '#population %s\n' "${#SPECS[@]}"
  # HOW MANY DECLARATIONS WERE CHECKED BY CONTENT, not just by existence. It is
  # reported HERE rather than in the sweep proper for a reason the rehearsal
  # depends on: the sweep's own output has to be byte-identical either side of
  # the family deletion, and this number is 0 by definition once the sources are
  # gone. `source_ref_rehearsal.py` asserts it on both sides, so the arm cannot
  # be silently unwired.
  printf '#validated %s\n' "$VALIDATED"
  # ONLY WHEN THE ARM DID NOT RUN. `source_ref_rehearsal.population_agreement`
  # diffs this output against `citation_tiers.py --specs` line for line, so an
  # unconditional extra line is a disagreement between the two population loops
  # -- which is what that gate exists to catch, and it caught this. Printing it
  # only in the skipped case keeps the browser output byte-identical AND keeps
  # the skip audible, instead of choosing between the two.
  [[ "$RETENTION_ARM" == "SKIPPED" ]] && printf '#retention-arm SKIPPED\n'
  exit 0
fi

if [[ "$RETENTION_ARM" == "SKIPPED" ]]; then
  echo "note: family '$FAMILY' has an EMPTY source prefix, so a whole-file"
  echo "      retention cannot be told from any other unprefixed .rs in"
  echo "      crates/kali_cli/tests. The whole-file-retention arm is SKIPPED;"
  echo "      this sweep covers $FAMILY's case files only."
fi

echo "sweep over ${#SPECS[@]} stems"
# `--family` IS FORWARDED ONLY WHEN IT WAS GIVEN. `batch5_crosscheck.py`
# defaults to browser and prints a one-line family banner when told a family
# explicitly, so a bare `citation_sweep.sh` produces output that is BYTE
# IDENTICAL to what it produced before this file learned about families --
# which is the evidence that the finished, deleted browser corpus did not
# regress, and it is worth more than a banner nobody needed.
python3 "$REPO/tools/task-18-browser-pilot/batch5_crosscheck.py" --citations-only \
  ${FAMILY_ARG[@]+"${FAMILY_ARG[@]}"} "${SPECS[@]}"
rc=$?
echo "SWEEP EXIT=$rc"
exit $rc
