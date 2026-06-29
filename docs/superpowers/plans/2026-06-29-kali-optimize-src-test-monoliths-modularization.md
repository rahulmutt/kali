# kali_optimize co-located src test-monolith modularization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_optimize's two co-located src unit-test monoliths — `src/specialize_tests.rs` (37 `#[test]`, 5,344 lines) and `src/object_fold_tests.rs` (48 `#[test]`, 1,654 lines) — into a thin facade + per-concern `#[path] mod` submodules grouped on a semantic axis, via pure verbatim code-motion with zero behavior change.

**Architecture:** For each file `F`, the facade `src/F.rs` keeps its 3 original `use` lines + appended `#[path = "F/<mod>.rs"] mod <mod>;` decls and drains to **zero** fns (both files contain only `#[test]` fns at module level — no helpers to retain); each submodule `src/F/<mod>.rs` is `use super::*;` + verbatim-moved `#[test]` fns. A deterministic Python mover (`move_fns.py`) performs the extraction; a verifier (`verify.py`) proves byte-identity. This is the 25th crate-modularization series entry (first of the post-kali_cli frontier). Spec: `docs/superpowers/specs/2026-06-29-kali-optimize-src-test-monoliths-modularization-design.md`.

**Tech Stack:** Rust (cargo, edition 2021), Python 3 (mover/verifier scripts), git.

## Global Constraints

- **Pure relocation refactor, NOT TDD.** No new product code, no new tests, no renames, no reordering, no reformatting.
- **Verbatim moves only** — each `#[test]` fn's attribute block + body + one trailing blank relocate byte-for-byte. Never reformat/reorder/tidy moved code.
- **Submodule header is exactly `use super::*;`** (nothing else). Facade keeps every original `use` line. No per-submodule extern `use`s.
- **Both facades drain to zero fns.** Neither file has module-level non-`#[test]` helpers (shared helpers live in `src/test_support.rs`; `append_literal_chain`/`build_object` are nested *inside* test bodies and move with their parent test). No `include_*!` pins are needed (verified: 0 in both files).
- **Retained-import safety (empirically verified):** a fully-drained facade keeps its 3 `use` lines (`use crate::test_support::*;` / `use crate::*;` / `use kali_lir::{LirBuilder, LirNodeKind};`); these are consumed *only* through the children's `use super::*;` and do **not** warn-as-unused (Rust re-exports the facade's private `use` items through the child glob via descendant-visibility). No `#[allow]`, no import deletion, no `#[cfg(test)]` re-export.
- **No `pub`/`pub(crate)` widening.** Child modules reach parent scope via `use super::*`.
- **Do NOT run `cargo fmt`.** The repo's `cargo fmt --all --check` gate is already red on baseline (10+ crates); accepted cosmetic minors are not regressions.
- **Integration: local-main ff-merge only — NEVER push to origin.** origin/main intentionally lags. Re-verify on merged main, then delete the branch.
- **SDD ledger** at `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite per task; durable recovery map.
- **Baseline (this sandbox):** `cargo build -p kali_optimize --tests 2>&1 | grep -c '^warning'` baseline == **0** (confirmed). Gate = warning count stays == 0. Per-file lib `--list` counts: specialize_tests **37**, object_fold_tests **48** (confirmed).

---

## File Structure

**Created (scratch, git-ignored — `.superpowers/sdd/` is in `.gitignore`):**
- `.superpowers/sdd/move_fns.py` — the verbatim mover (full source in Task 1).
- `.superpowers/sdd/verify.py` — byte-identity verifier (full source in Task 1).
- `.superpowers/sdd/progress.md` — SDD ledger.
- `.superpowers/sdd/baseline-specialize_tests.txt`, `baseline-object_fold_tests.txt` — `--list` basename multisets.
- `.superpowers/sdd/baseline-warnings.txt` — the `0` warning count.

**Created (committed):**
- `crates/kali_optimize/src/specialize_tests/{mir_layout,tagged_budget,generic_reuse,literal_args,layout_bindings}.rs` — 5 submodules.
- `crates/kali_optimize/src/object_fold_tests/{enumeration,reflect_own_keys,object_has_own}.rs` — 3 submodules.

**Modified (committed):**
- `crates/kali_optimize/src/specialize_tests.rs` — drained facade (0 fns) + 5 `#[path] mod` decls.
- `crates/kali_optimize/src/object_fold_tests.rs` — drained facade (0 fns) + 3 `#[path] mod` decls.

**Untouched:** `src/specialize.rs:832` (the `mod specialize_tests;` decl stays), `src/object_fold.rs:379` (the `mod object_fold_tests;` decl stays), `src/test_support.rs`, all production code.

---

## Task 1: Branch, scratch tooling, baselines

**Files:**
- Create: `.superpowers/sdd/move_fns.py`, `.superpowers/sdd/verify.py`, `.superpowers/sdd/progress.md`, `.superpowers/sdd/baseline-specialize_tests.txt`, `.superpowers/sdd/baseline-object_fold_tests.txt`, `.superpowers/sdd/baseline-warnings.txt`
- No committed changes in this task (scratch only).

**Interfaces:**
- Produces: `move_fns.py` CLI `python3 move_fns.py <root_rs_relpath> "<groups-spec>" ["<pin1,pin2>"]` (run from `crates/kali_optimize`) — writes `src/<stem>/<mod>.rs` submodules + rewrites facade; prints `name: count` per non-empty group. Grouping is **exact `#[test]`-name set membership** (not leading-prefix). `verify.py` CLI `python3 verify.py <orig_rs> "<submodule_glob>"` — asserts `{name: body}` extracted from `<orig_rs>` equals that from the submodules; exits non-zero on any mismatch.

- [ ] **Step 1: Create the branch off the current main HEAD**

Run from repo root:
```bash
git checkout main && git rev-parse --short HEAD   # expect fe2878947 (or current main)
git checkout -b refactor/kali-optimize-src-test-monoliths
mkdir -p .superpowers/sdd
```
Expected: on branch `refactor/kali-optimize-src-test-monoliths`, working tree clean.

- [ ] **Step 2: Write the mover `.superpowers/sdd/move_fns.py`**

Write this file **verbatim**. The `FN_RE` / `IDENT_CHARS` / `find_close_line` lexer is the proven string/comment/raw-string-aware extractor and **must not be altered**. The only difference from prior series entries is `group_for` (and the docstring): grouping is **exact name equality**, because the semantic token sits mid-name here (every fn starts `release_`/`release_advanced_`/`fast_`), so leading-prefix grouping cannot partition these files.

```python
#!/usr/bin/env python3
"""move_fns.py — verbatim #[test]-fn extractor / facade splitter (exact-name groups).

Usage (run from crate dir, e.g. crates/kali_optimize):
    python3 move_fns.py <root_rs_relpath> "<groups-spec>" ["<pin1,pin2>"]

groups-spec:  name=fnA,fnB;name2=fnC;misc=*
  - EXACT-NAME grouping: each #[test] fn joins the FIRST group (in spec order)
    whose member list contains the fn name verbatim (fn_name == member). This
    differs from prior series entries (leading-prefix startswith) because the
    semantic token here is mid-name. '*' = catch-all, must be last if used.
  - Empty groups are auto-skipped.
  - A fn matching no group → 'WARN: no group for <name>' and exit 2 (loud).

Optional 3rd arg: comma-separated #[test] fn names to PIN in the facade (not
moved) — for fns whose body has a FILE-RELATIVE include_str!/include_bytes!/
include!. Unused for kali_optimize (0 such macros), kept for tool generality.

Pure verbatim code-motion. Only #[test]-ATTRIBUTED fns move; every other line
(use-decls, nested helper fns inside test bodies) stays byte-for-byte. Each
moved fn relocates with its full attribute block + body + one trailing blank.

Writes <dir>/<stem>/<mod>.rs for each non-empty group (header exactly
'use super::*;' then the verbatim fns), and rewrites <root_rs_relpath> to drop
the moved fns and append '#[path = "<stem>/<mod>.rs"] mod <mod>;' decls.
Submodule dir is derived RELATIVE TO THE INPUT FILE'S OWN DIRECTORY.

KEEP FN_RE / IDENT_CHARS / find_close_line byte-identical when editing — the
string/comment/raw-string-aware brace lexer is required (these files contain
r#"..."# JS/TS templates with '}' at column 0; a naive column-0 close-brace scan
breaks). Filter by the #[test] ATTRIBUTE, never name alone.
"""
import os
import re
import sys

FN_RE = re.compile(r'^\s*(?:pub\s+(?:\([^)]*\)\s+)?)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)')
IDENT_CHARS = set('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_')


def find_close_line(lines, start):
    """Return the index of the line holding the brace that closes the fn body
    opened at/after line `start`. Char-level scan that respects //-comments,
    /*...*/ comments, "..." and '...' literals, and r#"..."# raw strings."""
    depth = 0
    seen_open = False
    i = start
    n = len(lines)
    while i < n:
        line = lines[i]
        j = 0
        L = len(line)
        while j < L:
            c = line[j]
            # line comment
            if c == '/' and j + 1 < L and line[j + 1] == '/':
                break  # rest of line is comment
            # block comment
            if c == '/' and j + 1 < L and line[j + 1] == '*':
                # scan to closing */ possibly across lines
                j += 2
                while True:
                    end = line.find('*/', j)
                    if end != -1:
                        j = end + 2
                        break
                    i += 1
                    if i >= n:
                        return n - 1
                    line = lines[i]
                    L = len(line)
                    j = 0
                continue
            # raw string r#"..."# / r"..."
            if c == 'r' and (j == 0 or line[j - 1] not in IDENT_CHARS):
                k = j + 1
                hashes = 0
                while k < L and line[k] == '#':
                    hashes += 1
                    k += 1
                if k < L and line[k] == '"':
                    # raw string opened: close on  "###... (hashes count)
                    closer = '"' + ('#' * hashes)
                    j = k + 1
                    while True:
                        end = line.find(closer, j)
                        if end != -1:
                            j = end + len(closer)
                            break
                        i += 1
                        if i >= n:
                            return n - 1
                        line = lines[i]
                        L = len(line)
                        j = 0
                    continue
            # normal string
            if c == '"':
                j += 1
                while j < L:
                    if line[j] == '\\':
                        j += 2
                        continue
                    if line[j] == '"':
                        j += 1
                        break
                    j += 1
                continue
            # char literal vs lifetime: 'x'  '\n'  vs  'a (lifetime)
            if c == "'":
                if j + 1 < L and line[j + 1] == '\\':
                    # escaped char literal '\n' '\'' '\\'
                    j += 2
                    while j < L and line[j] != "'":
                        j += 1
                    j += 1
                    continue
                if j + 2 < L and line[j + 2] == "'":
                    # simple char literal 'x'
                    j += 3
                    continue
                # lifetime — just advance past the quote
                j += 1
                continue
            if c == '{':
                depth += 1
                seen_open = True
            elif c == '}':
                depth -= 1
                if seen_open and depth == 0:
                    return i
            j += 1
        i += 1
    return n - 1


def parse_spec(spec):
    """[(name, (member,...))]; '*' becomes catch-all sentinel ('*',)."""
    groups = []
    for part in spec.split(';'):
        part = part.strip()
        if not part:
            continue
        name, _, members = part.partition('=')
        name = name.strip()
        mems = tuple(m.strip() for m in members.split(',') if m.strip())
        groups.append((name, mems))
    return groups


def group_for(fn_name, groups):
    for name, members in groups:
        if members == ('*',):
            return name
        if fn_name in members:
            return name
    return None


def main():
    root_rel = sys.argv[1]
    spec = sys.argv[2]
    # Optional 3rd arg: comma-separated #[test] fn names to PIN in the facade.
    pinned = set()
    if len(sys.argv) > 3 and sys.argv[3].strip():
        pinned = {p.strip() for p in sys.argv[3].split(',') if p.strip()}
    groups = parse_spec(spec)

    with open(root_rel, 'r') as f:
        text = f.read()
    lines = text.split('\n')  # keeps trailing-newline structure via final '' element

    # Identify spans of #[test] fns: (block_start, end_inclusive, fn_name)
    spans = []
    attr_start = None
    has_test = False
    i = 0
    n = len(lines)
    while i < n:
        stripped = lines[i].strip()
        if stripped.startswith('#['):
            if attr_start is None:
                attr_start = i
                has_test = False
            if stripped.startswith('#[test]') or stripped == '#[test]':
                has_test = True
            i += 1
            continue
        m = FN_RE.match(lines[i])
        if m:
            block_start = attr_start if attr_start is not None else i
            is_test = has_test
            end = find_close_line(lines, i)
            if is_test and m.group(1) not in pinned:
                spans.append((block_start, end, m.group(1)))
            attr_start = None
            has_test = False
            i = end + 1
            continue
        # any other non-attribute content resets a pending attr block
        if stripped:
            attr_start = None
            has_test = False
        i += 1

    if not spans:
        print('no #[test] fns found', file=sys.stderr)
        sys.exit(1)

    # Partition into groups, preserving source order within each group.
    assigned = {name: [] for name, _ in groups}
    for (s, e, name) in spans:
        g = group_for(name, groups)
        if g is None:
            print(f'WARN: no group for {name}', file=sys.stderr)
            sys.exit(2)
        # include one trailing blank line if present
        end = e
        if end + 1 < n and lines[end + 1].strip() == '':
            end = end + 1
        assigned[g].append((s, end))

    # Build the set of line indices that move out.
    moved = set()
    for g, ivs in assigned.items():
        for (s, e) in ivs:
            for k in range(s, e + 1):
                moved.add(k)

    stem = os.path.splitext(os.path.basename(root_rel))[0]
    d = os.path.dirname(root_rel)
    subdir = os.path.join(d, stem)

    # Write submodules (non-empty groups only), in spec order.
    decls = []
    for name, _ in groups:
        ivs = assigned[name]
        if not ivs:
            continue
        os.makedirs(subdir, exist_ok=True)
        body = []
        for (s, e) in ivs:
            body.extend(lines[s:e + 1])
        # one trailing newline at EOF; strip extra trailing blanks to exactly one
        while body and body[-1].strip() == '':
            body.pop()
        content = 'use super::*;\n\n' + '\n'.join(body) + '\n'
        with open(os.path.join(subdir, name + '.rs'), 'w') as f:
            f.write(content)
        decls.append(f'#[path = "{stem}/{name}.rs"]\nmod {name};')

    # Rewrite facade: keep every non-moved line VERBATIM, then append decls.
    # No blank-collapsing of surviving lines — surviving content stays byte-for-byte.
    facade = [lines[k] for k in range(n) if k not in moved]
    while facade and facade[-1].strip() == '':
        facade.pop()
    facade_text = '\n'.join(facade) + '\n\n' + '\n\n'.join(decls) + '\n'
    with open(root_rel, 'w') as f:
        f.write(facade_text)

    for name, _ in groups:
        if assigned[name]:
            print(f'{name}: {len(assigned[name])}')


if __name__ == '__main__':
    main()
```

- [ ] **Step 3: Write the verifier `.superpowers/sdd/verify.py`**

Write this file verbatim. It re-extracts `{fn_name: body_text}` (test fns only) from an original file and from a glob of submodule files using the SAME lexer, and asserts equality:

```python
#!/usr/bin/env python3
"""Prove verbatim move: {name: body} from original == {name: body} from submodules.

Usage:  python3 verify.py <orig_rs> "<submodule_glob>" [extra_glob_for_facade_pins]
Exits non-zero on any name-set or body mismatch.
"""
import sys, glob, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from move_fns import FN_RE, find_close_line


def extract(path):
    with open(path) as f:
        lines = f.read().split('\n')
    out = {}
    attr_start = None
    has_test = False
    i = 0
    n = len(lines)
    while i < n:
        s = lines[i].strip()
        if s.startswith('#['):
            if attr_start is None:
                attr_start = i; has_test = False
            if s.startswith('#[test]'):
                has_test = True
            i += 1; continue
        m = FN_RE.match(lines[i])
        if m:
            bs = attr_start if attr_start is not None else i
            end = find_close_line(lines, i)
            if has_test:
                out[m.group(1)] = '\n'.join(lines[bs:end + 1])
            attr_start = None; has_test = False; i = end + 1; continue
        if s:
            attr_start = None; has_test = False
        i += 1
    return out


def collect(globs):
    d = {}
    for g in globs:
        for p in sorted(glob.glob(g)):
            for k, v in extract(p).items():
                assert k not in d, f"dup {k} in {p}"
                d[k] = v
    return d


orig = extract(sys.argv[1])
sub = collect(sys.argv[2:])
print(f"original #[test] fns: {len(orig)}   collected: {len(sub)}")
assert set(orig) == set(sub), f"name mismatch: only-orig={sorted(set(orig)-set(sub))} only-sub={sorted(set(sub)-set(orig))}"
bad = [k for k in orig if orig[k] != sub[k]]
if bad:
    print(f"BODY MISMATCH for {len(bad)} fns, e.g. {bad[:3]}")
    sys.exit(1)
print(f"PROOF OK: all {len(orig)} #[test] bodies byte-identical, name sets equal")
```

- [ ] **Step 4: Capture the warnings baseline**

Run from repo root:
```bash
cargo build -p kali_optimize --tests 2>&1 | grep -c '^warning' | tee .superpowers/sdd/baseline-warnings.txt
```
Expected: `0`. (kali_optimize's tests build is clean on baseline.) If non-zero, record the actual number — the gate becomes "warning count stays == that baseline".

- [ ] **Step 5: Capture the `--list` basename multisets**

Run from repo root (module paths are `specialize::specialize_tests::<fn>` and `object_fold::object_fold_tests::<fn>`; the `s/^.*:://` strips everything up to the final `::`, leaving the bare fn name):
```bash
cargo test -p kali_optimize --lib -- --list 2>/dev/null | grep ': test$' | grep 'specialize_tests::' \
  | sed -E 's/: test$//; s/^.*:://' | sort > .superpowers/sdd/baseline-specialize_tests.txt
cargo test -p kali_optimize --lib -- --list 2>/dev/null | grep ': test$' | grep 'object_fold_tests::' \
  | sed -E 's/: test$//; s/^.*:://' | sort > .superpowers/sdd/baseline-object_fold_tests.txt
wc -l .superpowers/sdd/baseline-specialize_tests.txt .superpowers/sdd/baseline-object_fold_tests.txt
```
Expected: `37` and `48`.

- [ ] **Step 6: Confirm there are no `include_*!` macros (no pins needed)**

Run from repo root:
```bash
grep -c 'include_str!\|include_bytes!\|include!' crates/kali_optimize/src/specialize_tests.rs crates/kali_optimize/src/object_fold_tests.rs
```
Expected: both report `0`. If either is non-zero, STOP — inventory which `#[test]` fn embeds a *file-relative* `include_*!` and add it to that file's pin list (the mover's 3rd arg) before splitting.

- [ ] **Step 7: Confirm both facades will drain fully (no module-level helper fns)**

Run from repo root (every module-level `fn` should be a `#[test]` fn; the count of col-0 `fn` must equal the count of col-0 `#[test]`):
```bash
for F in specialize_tests object_fold_tests; do
  printf '%s: col0-fn=%s col0-test=%s\n' "$F" \
    "$(grep -c '^fn ' crates/kali_optimize/src/$F.rs)" \
    "$(grep -c '^#\[test\]' crates/kali_optimize/src/$F.rs)"
done
```
Expected: `specialize_tests: col0-fn=37 col0-test=37` and `object_fold_tests: col0-fn=48 col0-test=48`. Equal counts confirm there are no module-level helpers to retain → both facades drain to 0 fns. If `col0-fn` > `col0-test`, a module-level helper exists; it must stay in the facade (it is not `#[test]`, so the mover already leaves it — but note it, as the facade will then keep that helper rather than draining to 0).

- [ ] **Step 8: Write the SDD ledger `.superpowers/sdd/progress.md`**

Record: branch name, branch-base HEAD, plan/spec paths, mover/verifier locations + "KEEP lexer byte-identical; grouping is EXACT-NAME", corrected gates (warnings==0; per-file `--list` basename-multiset diff empty; no `include_*!` pins; both facades drain to 0 fns), and a Task-status section (Task 1 COMPLETE, Tasks 2-4 PENDING). This file is git-ignored scratch — overwrite it as tasks complete.

- [ ] **Step 9: Verify branch + scratch state (no commit)**

```bash
git status --short        # expect clean (.superpowers/sdd is git-ignored)
git branch --show-current # expect refactor/kali-optimize-src-test-monoliths
ls .superpowers/sdd/      # move_fns.py verify.py progress.md baseline-*.txt
```
Expected: clean tree, correct branch, all scratch files present. **No commit in Task 1.**

---

## Task 2: Split `specialize_tests.rs` (37 → 5 submodules)

**Files:**
- Create: `crates/kali_optimize/src/specialize_tests/{mir_layout,tagged_budget,generic_reuse,literal_args,layout_bindings}.rs`
- Modify: `crates/kali_optimize/src/specialize_tests.rs` (drain to 0 fns + 5 `#[path] mod` decls)

**Interfaces:**
- Consumes: `move_fns.py`, `verify.py`, `baseline-specialize_tests.txt` from Task 1.
- Produces: 5 submodules under `specialize::specialize_tests::<mod>::`. No public-surface change; `src/specialize.rs:832` decl unchanged.

- [ ] **Step 1: Run the mover (exact-name groups; no pins)**

Run from `crates/kali_optimize`. The groups-spec is one long single-line argument (exact `#[test]` names, validated against source — counts 4/5/8/14/6):
```bash
python3 ../../.superpowers/sdd/move_fns.py src/specialize_tests.rs \
"mir_layout=release_specializes_large_function_using_mir_layouts,release_recursively_specializes_nested_mir_call_sites,release_specializes_same_binding_name_in_distinct_function_scopes,release_specializes_literal_shaped_mir_call_sites_without_layout_metadata;tagged_budget=release_specializes_tagged_parameters_from_concrete_arguments,release_respects_zero_specialization_budget_for_tagged_parameters,release_advanced_limits_specialization_to_one_distinct_call_site_after_root_inlining,release_specializes_tagged_parameters_for_non_inlined_functions,release_specializes_concrete_arguments_without_mir_layouts;generic_reuse=release_allows_generic_specialization_inside_mir_specialized_clones,release_advanced_allows_generic_specialization_inside_mir_specialized_clones,release_reuses_generic_specializations_across_layout_specialized_owners,release_advanced_reuses_generic_specializations_across_layout_specialized_owners,release_specializes_identical_generic_call_sites_across_owners_once,release_reuses_generic_specializations_across_reexport_chain,release_advanced_partially_specializes_reexport_chain,release_reuses_existing_mir_specializations_after_an_owner_spends_its_budget;literal_args=release_specializes_array_literal_arguments_by_shape,release_specializes_string_literal_arguments,release_specializes_quoted_string_and_template_literal_arguments_distinctly,release_specializes_regex_literal_arguments,release_specializes_regex_literal_arguments_with_mir_layouts,release_specializes_nullish_literal_arguments,release_advanced_specializes_nullish_literal_arguments,fast_keeps_nullish_literal_arguments_unspecialized,release_specializes_infinity_and_nan_literal_arguments,release_specializes_boolean_literal_arguments,release_specializes_numeric_literal_arguments,release_specializes_negative_zero_literal_arguments,release_specializes_bigint_literal_arguments,release_advanced_specializes_bigint_literal_arguments;layout_bindings=release_specializes_shared_closure_layout_bindings,release_specializes_distinct_closure_capture_bindings,release_specializes_nested_mir_bound_bindings_inside_object_literals,release_specializes_shared_struct_layout_bindings,release_specializes_distinct_struct_layout_bindings,release_specializes_distinct_array_layout_bindings"
```
Expected stdout (group counts):
```
mir_layout: 4
tagged_budget: 5
generic_reuse: 8
literal_args: 14
layout_bindings: 6
```
(Sum = 37.) If the mover prints `WARN: no group for <name>` and exits 2, a `#[test]` fn was omitted from the spec — STOP, add the exact name to the right group, revert (`git checkout src/specialize_tests.rs && rm -rf src/specialize_tests`), and re-run.

- [ ] **Step 2: G1 — facade drained + decls present**

Run from `crates/kali_optimize`:
```bash
grep -c '#\[test\]' src/specialize_tests.rs        # expect 0
grep -c '^#\[path' src/specialize_tests.rs          # expect 5
for f in src/specialize_tests/*.rs; do printf '%s: ' "$f"; head -1 "$f"; done
```
Expected: `0` test attrs in facade; `5` `#[path]` decls; every submodule's first line exactly `use super::*;`.

- [ ] **Step 3: G-verbatim — byte-identity proof**

Run from repo root:
```bash
git show HEAD:crates/kali_optimize/src/specialize_tests.rs > /tmp/orig_specialize_tests.rs
python3 .superpowers/sdd/verify.py /tmp/orig_specialize_tests.rs "crates/kali_optimize/src/specialize_tests/*.rs"
```
Expected: `PROOF OK: all 37 #[test] bodies byte-identical, name sets equal`. If it reports a body or name mismatch, STOP — do not commit; the lexer mis-bounded a span.

- [ ] **Step 4: G3 — no new warnings (proves retained facade imports + drained facade compile clean)**

Run from repo root:
```bash
cargo build -p kali_optimize --tests 2>&1 | tail -2
cargo build -p kali_optimize --tests 2>&1 | grep -c '^warning'
```
Expected: build finishes (`Finished ...`), warning count `0` (unchanged from baseline). Any higher number = a real regression; STOP. (An `unused_imports` warning on the facade's `use` lines would mean the descendant-visibility re-export assumption failed — but it is verified not to; investigate before any `#[allow]`.)

- [ ] **Step 5: G4 — `--list` basename multiset unchanged**

Run from repo root:
```bash
cargo test -p kali_optimize --lib -- --list 2>/dev/null | grep ': test$' | grep 'specialize_tests::' \
  | sed -E 's/: test$//; s/^.*:://' | sort | diff - .superpowers/sdd/baseline-specialize_tests.txt && echo "G4 OK (37 names match)"
```
Expected: no diff output, then `G4 OK (37 names match)`. (`sort` without `-u` = multiset; `s/^.*:://` strips the new `<mod>::` segment.)

- [ ] **Step 6: Commit**

Run from repo root:
```bash
git add crates/kali_optimize/src/specialize_tests.rs crates/kali_optimize/src/specialize_tests/
git commit -m "refactor(kali_optimize): split specialize_tests.rs into per-concern test submodules [refactor]"
```
Then update `.superpowers/sdd/progress.md`: Task 2 COMPLETE with the commit hash + per-module counts.

---

## Task 3: Split `object_fold_tests.rs` (48 → 3 submodules)

**Files:**
- Create: `crates/kali_optimize/src/object_fold_tests/{enumeration,reflect_own_keys,object_has_own}.rs`
- Modify: `crates/kali_optimize/src/object_fold_tests.rs` (drain to 0 fns + 3 `#[path] mod` decls)

**Interfaces:**
- Consumes: `move_fns.py`, `verify.py`, `baseline-object_fold_tests.txt` from Task 1.
- Produces: 3 submodules under `object_fold::object_fold_tests::<mod>::`. No public-surface change; `src/object_fold.rs:379` decl unchanged.

- [ ] **Step 1: Run the mover (exact-name groups; no pins)**

Run from `crates/kali_optimize`. One long single-line groups-spec (exact `#[test]` names, validated — counts 20/16/12):
```bash
python3 ../../.superpowers/sdd/move_fns.py src/object_fold_tests.rs \
"enumeration=release_folds_object_keys_calls_over_literal_object_shapes,release_folds_object_entries_calls_over_literal_object_shapes,release_folds_object_from_entries_calls_over_literal_entry_arrays,release_folds_global_this_object_from_entries_calls_over_literal_entry_arrays,release_folds_object_values_calls_over_literal_object_shapes,release_folds_object_enumeration_calls_over_string_literals,release_folds_bracketed_global_this_object_enumeration_calls_over_string_literals,release_folds_global_this_object_enumeration_calls_over_string_literals,release_advanced_folds_global_this_object_enumeration_calls_over_string_literals,release_folds_bracketed_global_this_object_enumeration_calls_over_literal_object_shapes,release_advanced_folds_object_enumeration_calls_over_string_literals,release_advanced_folds_bracketed_global_this_object_enumeration_calls_over_string_literals,fast_folds_object_enumeration_calls_over_literal_object_shapes,release_folds_object_enumeration_calls_over_const_bound_literal_object_shapes,release_folds_object_enumeration_calls_over_wrapped_const_bound_literal_object_shapes,release_folds_object_enumeration_calls_over_const_alias_chains,release_advanced_folds_object_enumeration_calls_over_const_alias_chains,release_advanced_folds_object_enumeration_calls_over_const_bound_literal_object_shapes,release_advanced_folds_object_enumeration_calls_over_frozen_literal_object_shapes,release_advanced_folds_object_enumeration_calls_over_literal_object_shapes;reflect_own_keys=fast_folds_reflect_own_keys_calls_over_literal_object_shapes,release_folds_reflect_own_keys_calls_over_literal_object_shapes,fast_folds_bracketed_reflect_own_keys_calls_over_literal_object_shapes,release_folds_bracketed_reflect_own_keys_calls_over_literal_object_shapes,release_advanced_folds_reflect_own_keys_calls_over_literal_object_shapes,release_advanced_folds_bracketed_reflect_own_keys_calls_over_literal_object_shapes,fast_folds_mixed_bracketed_reflect_own_keys_calls_over_literal_object_shapes,release_folds_mixed_bracketed_reflect_own_keys_calls_over_literal_object_shapes,release_advanced_folds_mixed_bracketed_reflect_own_keys_calls_over_literal_object_shapes,fast_folds_global_this_reflect_bracketed_own_keys_calls_over_literal_object_shapes,release_folds_global_this_reflect_bracketed_own_keys_calls_over_literal_object_shapes,release_advanced_folds_global_this_reflect_bracketed_own_keys_calls_over_literal_object_shapes,release_folds_reflect_own_keys_calls_over_frozen_literal_object_shapes,release_advanced_folds_reflect_own_keys_calls_over_const_bound_literal_object_shapes,release_folds_reflect_own_keys_calls_over_const_alias_chains,release_advanced_folds_reflect_own_keys_calls_over_const_alias_chains;object_has_own=release_folds_object_has_own_calls_over_literal_object_shapes,release_folds_object_has_own_calls_through_optional_chain_wrappers,release_folds_object_has_own_calls_through_frozen_optional_chain_wrappers,release_folds_object_has_own_calls_over_frozen_from_entries_shapes,release_folds_object_has_own_calls_over_frozen_bracketed_from_entries_shapes,release_folds_object_has_own_calls_through_frozen_callable_wrappers,release_advanced_folds_object_has_own_calls_through_frozen_callable_wrappers,release_advanced_folds_object_has_own_calls_over_literal_object_shapes,release_advanced_folds_object_has_own_calls_over_frozen_from_entries_shapes,release_advanced_folds_object_has_own_calls_over_frozen_bracketed_from_entries_shapes,release_advanced_folds_bracketed_object_has_own_calls_over_literal_object_shapes,release_folds_object_has_own_calls_over_const_bound_literal_object_shapes"
```
Expected stdout:
```
enumeration: 20
reflect_own_keys: 16
object_has_own: 12
```
(Sum = 48.) A `WARN: no group for <name>` / exit 2 means a fn was omitted — STOP, fix the spec, revert (`git checkout src/object_fold_tests.rs && rm -rf src/object_fold_tests`), re-run.

- [ ] **Step 2: G1 — facade drained + decls present**

Run from `crates/kali_optimize`:
```bash
grep -c '#\[test\]' src/object_fold_tests.rs        # expect 0
grep -c '^#\[path' src/object_fold_tests.rs          # expect 3
for f in src/object_fold_tests/*.rs; do printf '%s: ' "$f"; head -1 "$f"; done
```
Expected: `0` test attrs in facade; `3` `#[path]` decls; every submodule's first line exactly `use super::*;`.

- [ ] **Step 3: G-verbatim — byte-identity proof**

Run from repo root:
```bash
git show HEAD:crates/kali_optimize/src/object_fold_tests.rs > /tmp/orig_object_fold_tests.rs
python3 .superpowers/sdd/verify.py /tmp/orig_object_fold_tests.rs "crates/kali_optimize/src/object_fold_tests/*.rs"
```
Expected: `PROOF OK: all 48 #[test] bodies byte-identical, name sets equal`. Mismatch → STOP, do not commit.

- [ ] **Step 4: G3 — no new warnings**

Run from repo root:
```bash
cargo build -p kali_optimize --tests 2>&1 | tail -2
cargo build -p kali_optimize --tests 2>&1 | grep -c '^warning'
```
Expected: `Finished ...`, warning count `0`. Higher = regression; STOP.

- [ ] **Step 5: G4 — `--list` basename multiset unchanged**

Run from repo root:
```bash
cargo test -p kali_optimize --lib -- --list 2>/dev/null | grep ': test$' | grep 'object_fold_tests::' \
  | sed -E 's/: test$//; s/^.*:://' | sort | diff - .superpowers/sdd/baseline-object_fold_tests.txt && echo "G4 OK (48 names match)"
```
Expected: no diff, then `G4 OK (48 names match)`.

- [ ] **Step 6: Commit**

Run from repo root:
```bash
git add crates/kali_optimize/src/object_fold_tests.rs crates/kali_optimize/src/object_fold_tests/
git commit -m "refactor(kali_optimize): split object_fold_tests.rs into per-operation test submodules [refactor]"
```
Then update `.superpowers/sdd/progress.md`: Task 3 COMPLETE with commit hash + per-module counts.

---

## Task 4: Finalize — whole-branch proof, run tests, ff-merge, cleanup

**Files:** none created/modified (verification + integration only).

**Interfaces:**
- Consumes: the two refactor commits from Tasks 2-3.
- Produces: branch fast-forward-merged into local `main`; branch deleted; memory updated.

- [ ] **Step 1: Whole-branch verbatim proof (base → head)**

Run from repo root (compares the pre-branch original files against the final split layout):
```bash
BASE=$(git merge-base main HEAD)   # the branch base
git show $BASE:crates/kali_optimize/src/specialize_tests.rs  > /tmp/base_specialize.rs
git show $BASE:crates/kali_optimize/src/object_fold_tests.rs > /tmp/base_object_fold.rs
python3 .superpowers/sdd/verify.py /tmp/base_specialize.rs  "crates/kali_optimize/src/specialize_tests/*.rs"
python3 .superpowers/sdd/verify.py /tmp/base_object_fold.rs "crates/kali_optimize/src/object_fold_tests/*.rs"
```
Expected: `PROOF OK: all 37 ...` and `PROOF OK: all 48 ...`.

- [ ] **Step 2: Full build + lib-test run**

Run from repo root:
```bash
cargo build -p kali_optimize --tests 2>&1 | grep -c '^warning'   # expect 0
cargo test -p kali_optimize --lib 2>&1 | tail -15
```
Expected: warnings == 0; lib unit tests run and pass (these are pure unit tests with no chromium-sandbox dependency → clean pass). If any pre-existing failure appears, confirm it is byte-for-byte the same test NAME as on the branch base, not a new one (code-motion shifts panic line numbers; the message is the invariant).

- [ ] **Step 3: Request review (finalize gate)**

Confirm with a fresh reviewer (opus for the whole-branch finalize) that: (a) both verbatim proofs pass, (b) both facades are correctly drained to 0 fns, (c) all 8 submodule headers are exactly `use super::*;`, (d) `src/specialize.rs:832` and `src/object_fold.rs:379` decls are unchanged, (e) warnings == 0, (f) the facades retain their original 3 `use` lines (no `#[allow]`, no import deletion). Address any findings before merging.

- [ ] **Step 4: Fast-forward merge into local main (NEVER push origin)**

Run from repo root:
```bash
git checkout main
git merge --ff-only refactor/kali-optimize-src-test-monoliths
git log --oneline -3
```
Expected: fast-forward; `main` now points at the object_fold_tests commit. **Do NOT `git push`.**

- [ ] **Step 5: Re-verify on merged main**

Run from repo root:
```bash
cargo build -p kali_optimize --tests 2>&1 | grep -c '^warning'   # expect 0
cargo test -p kali_optimize --lib -- --list 2>/dev/null | grep ': test$' | grep -cE 'specialize_tests::|object_fold_tests::'   # expect 85
```
Expected: warnings == 0; total preserved at 85 (37 + 48).

- [ ] **Step 6: Delete the branch**

Run from repo root:
```bash
git branch -d refactor/kali-optimize-src-test-monoliths
```

- [ ] **Step 7: Update the series memory**

Update `/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md`: record the 25th entry (kali_optimize co-located src unit-test split — `specialize_tests.rs` 37→5 + `object_fold_tests.rs` 48→3, DONE, merged to local main `<hash>`, with the **exact-name mover generalization** and the 85-test byte-identical proof). Note: both facades drained fully to 0 fns; retained `use` lines consumed only via `use super::*;` compiled clean (descendant-visibility); no `include_*!` pins. Remaining frontier: kali_types, kali_runtime, kali_codegen co-located src test monoliths.

---

## Self-Review

- **Spec coverage:** Approach (facade + `use super::*` submodules, both drain to 0) → Tasks 2-3 Steps 1-2; drained-facade import safety → Global Constraints + Task 2 Step 4 note. Module groupings (both tables, exact-name) → Task 2/3 Step 1 group-specs (validated against source: 4/5/8/14/6 and 20/16/12). Mover exact-name generalization → Task 1 Step 2 (`group_for` uses `==`; lexer byte-identical). No `include_*!` pins → Task 1 Step 6. Both facades fully drain → Task 1 Step 7 (col0-fn == col0-test check). Gates G1/G3/G4/G-verbatim → Task 2/3 Steps 2-5; G5 (runtime) folded into Task 4 Step 2. Constraints (verbatim, no fmt, no widening, ff-merge-only) → Global Constraints + Task 4 Step 4. Out-of-scope (small files, other crates) → Task 4 Step 7 note. All spec sections covered.
- **Placeholder scan:** No TBD/TODO; every code/command step shows exact content and expected output. Mover and verifier embedded in full. The only `<hash>`/`<name>` tokens are runtime values recorded into scratch/memory, not unresolved plan content.
- **Type/name consistency:** `move_fns.py`/`verify.py` signatures, group names (`mir_layout`/`tagged_budget`/`generic_reuse`/`literal_args`/`layout_bindings`; `enumeration`/`reflect_own_keys`/`object_has_own`), `--list` filter strings (`specialize_tests::` / `object_fold_tests::`), and baseline filenames identical across Tasks 1-4. Group-spec strings in Task 2/3 match the spec tables and the validated counts. Module paths (`specialize::specialize_tests::` / `object_fold::object_fold_tests::`) confirmed against real `--list` output.
