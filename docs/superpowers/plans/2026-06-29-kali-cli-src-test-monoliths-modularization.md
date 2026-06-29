# kali_cli co-located src test-monolith modularization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_cli's two co-located src unit-test monoliths — `src/build_tests.rs` (716 `#[test]`, 15,535 lines) and `src/output_tests.rs` (229 `#[test]`, 8,509 lines) — into a thin facade + per-concern `#[path] mod` submodules, via pure verbatim code-motion with zero behavior change.

**Architecture:** For each file `F`, the facade `src/F.rs` keeps every `use` line + all non-`#[test]` helpers + pinned `include_*!` fns + appended `#[path = "F/<mod>.rs"] mod <mod>;` decls; each submodule `src/F/<mod>.rs` is `use super::*;` + verbatim-moved `#[test]` fns. A deterministic Python mover (`move_fns.py`) performs the extraction; a verifier (`verify.py`) proves byte-identity. This is the 24th crate-modularization series entry (kali_cli sub-project 4 of 4, final). Spec: `docs/superpowers/specs/2026-06-29-kali-cli-src-test-monoliths-modularization-design.md`.

**Tech Stack:** Rust (cargo, edition 2021), Python 3 (mover/verifier scripts), git.

## Global Constraints

- **Pure relocation refactor, NOT TDD.** No new product code, no new tests, no renames, no reordering, no reformatting.
- **Verbatim moves only** — each `#[test]` fn's attribute block + body + one trailing blank relocate byte-for-byte. Never reformat/reorder/tidy moved code.
- **Submodule header is exactly `use super::*;`** (nothing else). Facade keeps every original `use` line. No per-submodule extern `use`s.
- **Facade ends with zero `#[test]` fns**, EXCEPT fns pinned for the `include_*!` gotcha (`output_tests` keeps exactly 2). Non-`#[test]` helpers (incl. `#[cfg]`-gated helpers lacking `#[test]`) always stay in the facade.
- **No `pub`/`pub(crate)` widening.** Child modules reach parent scope via `use super::*` (Rust descendant-visibility of private `use` imports).
- **Do NOT run `cargo fmt`.** The repo's `cargo fmt --all --check` gate is already red on baseline (10+ crates); accepted cosmetic minors (over-long signatures, stray blank lines from removals) are not regressions.
- **Integration: local-main ff-merge only — NEVER push to origin.** origin/main intentionally lags. Re-verify on merged main, then delete the branch.
- **SDD ledger** at `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite per task; durable recovery map.
- **`include_*!` pin gotcha:** `include_str!`/`include_bytes!`/`include!` resolve paths relative to the source file. A `#[test]` fn with a file-relative `include_*!` breaks when moved one dir deeper; pin it to the facade (mover's 3rd arg) rather than rewrite the path. `build_tests` has none; `output_tests` has exactly two (`published_cli_envelope_schema_matches_fixed_shape_validator_posture`, `published_diagnostic_schema_matches_fixed_shape_validator_posture`).
- **Sandbox baseline (corrected gates):** literal "0 warnings / fully green" does NOT hold. `cargo build -p kali_cli --tests 2>&1 | grep -c '^warning'` baseline == **2** (a grep artifact for the single pre-existing `build/mod.rs:40 profile_data_hash unused_imports` lib-test warning + cargo's "generated 1 warning" summary; one real warning, untouched here). Gate = warning count stays == 2.

---

## File Structure

**Created (scratch, git-ignored — `.superpowers/sdd/` is in `.gitignore`):**
- `.superpowers/sdd/move_fns.py` — the verbatim mover (full source in Task 1).
- `.superpowers/sdd/verify.py` — byte-identity verifier (full source in Task 1).
- `.superpowers/sdd/progress.md` — SDD ledger.
- `.superpowers/sdd/baseline-build_tests.txt`, `baseline-output_tests.txt` — `--list` basename multisets.

**Created (committed):**
- `crates/kali_cli/src/build_tests/{supports_math,supports_for,supports_misc,rejects,check,collect,validate,runtime,discover,misc}.rs` — 10 submodules.
- `crates/kali_cli/src/output_tests/{envelope,doctor,package,run,effects,test,payloads_misc,emit}.rs` — 8 submodules.

**Modified (committed):**
- `crates/kali_cli/src/build_tests.rs` — drained facade (0 `#[test]`) + 10 `#[path] mod` decls.
- `crates/kali_cli/src/output_tests.rs` — facade with 2 pinned `#[test]` + 8 `#[path] mod` decls.

**Untouched:** `src/build/mod.rs` (the `#[path="../build_tests.rs"] mod tests;` decl + cutoff re-exports at lines 37,47 stay), `src/lib.rs:559` (the `mod output_tests;` decl stays), all production code.

---

## Task 1: Branch, scratch tooling, baselines

**Files:**
- Create: `.superpowers/sdd/move_fns.py`, `.superpowers/sdd/verify.py`, `.superpowers/sdd/progress.md`, `.superpowers/sdd/baseline-build_tests.txt`, `.superpowers/sdd/baseline-output_tests.txt`
- No committed changes in this task (scratch only).

**Interfaces:**
- Produces: `move_fns.py` CLI `python3 move_fns.py <root_rs_relpath> "<groups-spec>" ["<pin1,pin2>"]` (run from `crates/kali_cli`) — writes `src/<stem>/<mod>.rs` submodules + rewrites facade; prints `name: count` per non-empty group. `verify.py` CLI `python3 verify.py <orig_rs> "<submodule_glob>"` — asserts `{name: body}` extracted from `<orig_rs>` equals that from the submodules; exits non-zero on any mismatch.

- [ ] **Step 1: Create the branch off the current main HEAD**

Run from repo root:
```bash
git checkout main && git rev-parse --short HEAD   # expect 77704c7e7 (or current main)
git checkout -b refactor/kali-cli-src-test-monoliths
mkdir -p .superpowers/sdd
```
Expected: on branch `refactor/kali-cli-src-test-monoliths`, working tree clean.

- [ ] **Step 2: Write the mover `.superpowers/sdd/move_fns.py`**

Write this file **verbatim** (it is the proven mover; the `FN_RE` / `IDENT_CHARS` / `find_close_line` lexer is string/comment/raw-string-aware and must not be altered):

```python
#!/usr/bin/env python3
"""move_fns.py — verbatim #[test]-fn extractor / facade splitter.

Usage (run from crate dir, e.g. crates/kali_cli):
    python3 move_fns.py <root_rs_relpath> "<groups-spec>"

groups-spec:  name=p1,p2;name2=p3;misc=*
  - Leading-prefix grouping: each #[test] fn joins the FIRST group (in spec
    order) any of whose prefixes is a leading prefix of the fn name
    (name.startswith(prefix)). '*' = catch-all, must be last.
  - Empty groups are auto-skipped.

Pure verbatim code-motion. Only #[test]-ATTRIBUTED fns move; every other line
(use-decls, non-#[test] helper fns, cfg-gated non-test helpers) stays byte-for-
byte in the facade. Each moved fn relocates with its full attribute block + body
+ exactly one trailing blank line.

Writes <dir>/<stem>/<mod>.rs for each non-empty group (header exactly
'use super::*;' then the verbatim fns), and rewrites <root_rs_relpath> to drop
the moved fns and append '#[path = "<stem>/<mod>.rs"] mod <mod>;' decls.
Submodule dir is derived RELATIVE TO THE INPUT FILE'S OWN DIRECTORY.

KEEP FN_RE / IDENT_CHARS / find_close_line byte-identical when editing — the
string/comment/raw-string-aware brace lexer is required (these files contain
r#"..."# JS/TS templates with '}' at column 0; a naive column-0 close-brace scan
breaks). Filter by the #[test] ATTRIBUTE, never name prefix alone.
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
    """[(name, (prefix,...))]; '*' becomes catch-all sentinel ('*',)."""
    groups = []
    for part in spec.split(';'):
        part = part.strip()
        if not part:
            continue
        name, _, prefixes = part.partition('=')
        name = name.strip()
        prefs = tuple(p.strip() for p in prefixes.split(',') if p.strip())
        groups.append((name, prefs))
    return groups


def group_for(fn_name, groups):
    for name, prefs in groups:
        if prefs == ('*',):
            return name
        if any(fn_name.startswith(p) for p in prefs):
            return name
    return None


def main():
    root_rel = sys.argv[1]
    spec = sys.argv[2]
    # Optional 3rd arg: comma-separated #[test] fn names to PIN in the facade
    # (not moved). Required for fns whose body has a FILE-RELATIVE include_str!/
    # include_bytes!/include! — moving them one dir deeper breaks the path, and
    # rewriting the path would violate the verbatim mandate.
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
cargo build -p kali_cli --tests 2>&1 | grep -c '^warning' | tee .superpowers/sdd/baseline-build-warnings.txt
```
Expected: `2`. (One real warning — `build/mod.rs:40 profile_data_hash unused_imports` — plus cargo's "generated 1 warning" summary; both lines begin with "warning".)

- [ ] **Step 5: Capture the `--list` basename multisets**

Run from repo root:
```bash
cargo test -p kali_cli --lib -- --list 2>/dev/null | grep ': test$' | grep '^build::tests::' \
  | sed -E 's/: test$//; s/^.*:://' | sort > .superpowers/sdd/baseline-build_tests.txt
cargo test -p kali_cli --lib -- --list 2>/dev/null | grep ': test$' | grep '^output_tests::' \
  | sed -E 's/: test$//; s/^.*:://' | sort > .superpowers/sdd/baseline-output_tests.txt
wc -l .superpowers/sdd/baseline-build_tests.txt .superpowers/sdd/baseline-output_tests.txt
```
Expected: `716` and `229`.

- [ ] **Step 6: Confirm `include_*!` pin inventory**

Run from repo root:
```bash
grep -rn 'include_str!\|include_bytes!\|include!' crates/kali_cli/src/build_tests.rs
grep -rn 'include_str!\|include_bytes!\|include!' crates/kali_cli/src/output_tests.rs
```
Expected: **build_tests → no output** (no pins). **output_tests → 2 lines**, both `include_str!("../../../schemas/...")`, inside `published_cli_envelope_schema_matches_fixed_shape_validator_posture` and `published_diagnostic_schema_matches_fixed_shape_validator_posture`. If build_tests unexpectedly shows any, STOP and add those fns to its pin list before Task 2.

- [ ] **Step 7: Write the SDD ledger `.superpowers/sdd/progress.md`**

Record: branch name, branch-base HEAD, plan/spec paths, mover/verifier locations + "KEEP lexer byte-identical", corrected gates (warnings==2; per-file `--list` basename-multiset diff empty; `include_*!` pin list), and a Task-status section (Task 1 COMPLETE, Tasks 2-4 PENDING). This file is git-ignored scratch — overwrite it as tasks complete.

- [ ] **Step 8: Verify branch + scratch state (no commit)**

```bash
git status --short        # expect clean (.superpowers/sdd is git-ignored)
git branch --show-current # expect refactor/kali-cli-src-test-monoliths
ls .superpowers/sdd/      # move_fns.py verify.py progress.md baseline-*.txt
```
Expected: clean tree, correct branch, all scratch files present. **No commit in Task 1.**

---

## Task 2: Split `build_tests.rs` (716 → 10 submodules)

**Files:**
- Create: `crates/kali_cli/src/build_tests/{supports_math,supports_for,supports_misc,rejects,check,collect,validate,runtime,discover,misc}.rs`
- Modify: `crates/kali_cli/src/build_tests.rs` (drain to 0 `#[test]` + 10 `#[path] mod` decls)

**Interfaces:**
- Consumes: `move_fns.py`, `verify.py`, `baseline-build_tests.txt` from Task 1.
- Produces: 10 submodules under `build::tests::<mod>::`. No public-surface change; `src/build/mod.rs` decl + cutoff re-exports unchanged.

- [ ] **Step 1: Run the mover (build_tests has NO pins)**

Run from `crates/kali_cli`:
```bash
python3 ../../.superpowers/sdd/move_fns.py src/build_tests.rs \
  "supports_math=build_source_file_supports_math;supports_for=build_source_file_supports_for;supports_misc=build_source_file_supports;rejects=build_source_file_rejects;check=check_source;collect=collect;validate=validate;runtime=runtime_entrypoint;discover=discover_dynamic;misc=*"
```
Expected stdout (group counts):
```
supports_math: 118
supports_for: 121
supports_misc: 121
rejects: 142
check: 43
collect: 53
validate: 46
runtime: 26
discover: 16
misc: 30
```
(Sum = 716. Spec-order matters: `supports_math`/`supports_for` precede the general `supports_misc` so the specific clusters bind first.)

- [ ] **Step 2: G1 — facade drained + decls present**

Run from `crates/kali_cli`:
```bash
grep -c '#\[test\]' src/build_tests.rs        # expect 0
grep -c '#\[path' src/build_tests.rs           # expect 10
for f in src/build_tests/*.rs; do printf '%s: ' "$f"; head -1 "$f"; done
```
Expected: `0` test attrs in facade; `10` `#[path]` decls; every submodule's first line exactly `use super::*;`.

- [ ] **Step 3: G-verbatim — byte-identity proof**

Run from repo root:
```bash
git show HEAD:crates/kali_cli/src/build_tests.rs > /tmp/orig_build_tests.rs
python3 .superpowers/sdd/verify.py /tmp/orig_build_tests.rs "crates/kali_cli/src/build_tests/*.rs"
```
Expected: `PROOF OK: all 716 #[test] bodies byte-identical, name sets equal`. If it reports a body or name mismatch, STOP — do not commit; the lexer mis-bounded a span.

- [ ] **Step 4: G3 — no new warnings**

Run from repo root:
```bash
cargo build -p kali_cli --tests 2>&1 | grep -c '^warning'
```
Expected: `2` (unchanged from baseline). Any higher number = a real regression; STOP.

- [ ] **Step 5: G4 — `--list` basename multiset unchanged**

Run from repo root:
```bash
cargo test -p kali_cli --lib -- --list 2>/dev/null | grep ': test$' | grep '^build::tests::' \
  | sed -E 's/: test$//; s/^.*:://' | sort | diff - .superpowers/sdd/baseline-build_tests.txt && echo "G4 OK (716 names match)"
```
Expected: no diff output, then `G4 OK (716 names match)`. (`sort` without `-u` = multiset; `s/^.*:://` strips the new `<mod>::` segment.)

- [ ] **Step 6: Commit**

Run from repo root:
```bash
git add crates/kali_cli/src/build_tests.rs crates/kali_cli/src/build_tests/
git commit -m "refactor(kali_cli): split build_tests.rs into per-concern test submodules [refactor]"
```
Then update `.superpowers/sdd/progress.md`: Task 2 COMPLETE with the commit hash + per-module counts.

---

## Task 3: Split `output_tests.rs` (229 → 8 submodules + 2 pinned)

**Files:**
- Create: `crates/kali_cli/src/output_tests/{envelope,doctor,package,run,effects,test,payloads_misc,emit}.rs`
- Modify: `crates/kali_cli/src/output_tests.rs` (retain 2 pinned `#[test]` + 8 `#[path] mod` decls)

**Interfaces:**
- Consumes: `move_fns.py`, `verify.py`, `baseline-output_tests.txt` from Task 1.
- Produces: 8 submodules under `output_tests::<mod>::`; facade retains the 2 `include_str!`-bearing fns at `output_tests::`. No public-surface change; `src/lib.rs:559` decl unchanged.

- [ ] **Step 1: Run the mover WITH the 2 pins**

Run from `crates/kali_cli` (the 3rd arg is the comma-separated pin list — keep on one line):
```bash
python3 ../../.superpowers/sdd/move_fns.py src/output_tests.rs \
  "envelope=validate_envelope,emit_envelope;doctor=validate_doctor;package=validate_package;run=validate_run;effects=validate_effects;test=validate_test;payloads_misc=validate_install,validate_init,validate_lint,validate_fmt,validate_check;emit=*" \
  "published_cli_envelope_schema_matches_fixed_shape_validator_posture,published_diagnostic_schema_matches_fixed_shape_validator_posture"
```
Expected stdout:
```
envelope: 67
doctor: 59
package: 22
run: 13
effects: 13
test: 11
payloads_misc: 18
emit: 24
```
(Sum = 227; the 2 pinned fns stay in the facade → 227 + 2 = 229.)

- [ ] **Step 2: G1 — facade keeps exactly the 2 pins + decls present**

Run from `crates/kali_cli`:
```bash
grep -c '#\[test\]' src/output_tests.rs          # expect 2
grep -oP '^\s*fn \K\w+' src/output_tests.rs | grep '^published_'   # expect the 2 pinned names
grep -c '#\[path' src/output_tests.rs             # expect 8
for f in src/output_tests/*.rs; do printf '%s: ' "$f"; head -1 "$f"; done
```
Expected: `2` test attrs (the two `published_*_schema_*` fns); `8` `#[path]` decls; every submodule's first line exactly `use super::*;`.

- [ ] **Step 3: G-verbatim — byte-identity proof (submodules + facade pins)**

Run from repo root (note the facade is passed as a second glob so the 2 pinned fns are included in the comparison):
```bash
git show HEAD:crates/kali_cli/src/output_tests.rs > /tmp/orig_output_tests.rs
python3 .superpowers/sdd/verify.py /tmp/orig_output_tests.rs "crates/kali_cli/src/output_tests/*.rs" "crates/kali_cli/src/output_tests.rs"
```
Expected: `PROOF OK: all 229 #[test] bodies byte-identical, name sets equal`. Mismatch → STOP, do not commit.

- [ ] **Step 4: G3 — no new warnings (this proves the pin fixed the `include_str!` paths)**

Run from repo root:
```bash
cargo build -p kali_cli --tests 2>&1 | tail -2
cargo build -p kali_cli --tests 2>&1 | grep -c '^warning'
```
Expected: build finishes (`Finished ...`), warning count `2`. If you see `couldn't read .../schemas/...` errors, a fn with a file-relative `include_*!` was NOT pinned — STOP, identify it (`grep -rn 'include_' src/output_tests/`), add it to the pin list, revert (`git checkout src/output_tests.rs && rm -rf src/output_tests`), and re-run Step 1.

- [ ] **Step 5: G4 — `--list` basename multiset unchanged**

Run from repo root:
```bash
cargo test -p kali_cli --lib -- --list 2>/dev/null | grep ': test$' | grep '^output_tests::' \
  | sed -E 's/: test$//; s/^.*:://' | sort | diff - .superpowers/sdd/baseline-output_tests.txt && echo "G4 OK (229 names match)"
```
Expected: no diff, then `G4 OK (229 names match)`. (The 2 pinned fns appear as `output_tests::published_*` — still matched by the `^output_tests::` filter and present in the baseline.)

- [ ] **Step 6: Commit**

Run from repo root:
```bash
git add crates/kali_cli/src/output_tests.rs crates/kali_cli/src/output_tests/
git commit -m "refactor(kali_cli): split output_tests.rs into per-payload test submodules [refactor]"
```
Then update `.superpowers/sdd/progress.md`: Task 3 COMPLETE with commit hash + per-module counts + the 2 pinned names.

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
git show $BASE:crates/kali_cli/src/build_tests.rs  > /tmp/base_build.rs
git show $BASE:crates/kali_cli/src/output_tests.rs > /tmp/base_output.rs
python3 .superpowers/sdd/verify.py /tmp/base_build.rs  "crates/kali_cli/src/build_tests/*.rs"
python3 .superpowers/sdd/verify.py /tmp/base_output.rs "crates/kali_cli/src/output_tests/*.rs" "crates/kali_cli/src/output_tests.rs"
```
Expected: `PROOF OK: all 716 ...` and `PROOF OK: all 229 ...`.

- [ ] **Step 2: Full build + targeted lib-test run**

Run from repo root:
```bash
cargo build -p kali_cli --tests 2>&1 | grep -c '^warning'   # expect 2
cargo test -p kali_cli --lib 2>&1 | tail -15
```
Expected: warnings == 2; lib unit tests run and pass (these are pure unit tests with no chromium-sandbox dependency, so expect a clean pass — note any pre-existing failures and confirm they are byte-for-byte the same test NAMES as on the branch base, not new).

- [ ] **Step 3: Request review (subagent-driven-development handles the per-task reviews; this is the finalize gate)**

Confirm with a fresh reviewer (opus for the whole-branch finalize) that: (a) both verbatim proofs pass, (b) facades are correctly drained (build 0, output 2 pinned), (c) all submodule headers are exactly `use super::*;`, (d) `src/build/mod.rs` and `src/lib.rs:559` decls are unchanged, (e) warnings == 2. Address any findings before merging.

- [ ] **Step 4: Fast-forward merge into local main (NEVER push origin)**

Run from repo root:
```bash
git checkout main
git merge --ff-only refactor/kali-cli-src-test-monoliths
git log --oneline -3
```
Expected: fast-forward; `main` now points at the output_tests commit. **Do NOT `git push`.**

- [ ] **Step 5: Re-verify on merged main**

Run from repo root:
```bash
cargo build -p kali_cli --tests 2>&1 | grep -c '^warning'   # expect 2
cargo test -p kali_cli --lib -- --list 2>/dev/null | grep ': test$' | grep -cE '^build::tests::|^output_tests::'   # expect 945
```
Expected: warnings == 2; total preserved at 945 (716 + 229).

- [ ] **Step 6: Delete the branch**

Run from repo root:
```bash
git branch -d refactor/kali-cli-src-test-monoliths
```

- [ ] **Step 7: Update the series memory**

Update `/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md`: record the 24th entry (kali_cli sub-project 4 of 4 — `build_tests.rs` + `output_tests.rs` co-located src unit-test split, DONE, merged to local main `<hash>`, with the `include_*!` pin gotcha and the 945-test byte-identical proof). Note kali_cli's co-located src tests are now fully modularized; the remaining frontier is other crates' co-located src test monoliths (kali_optimize, kali_types, kali_runtime, kali_codegen, …).

---

## Self-Review

- **Spec coverage:** Approach (facade + `use super::*` submodules) → Tasks 2-3 Step 1-2. Wiring / descendant-visibility → relied on by Step 1 mover output, proven by Step 4 build. `include_*!` pin gotcha → Task 1 Step 6 (inventory) + Task 3 Step 1/4 (pins). Module groupings (both tables) → Task 2/3 Step 1 group-specs, counts match spec exactly. Tooling (mover + 3rd pin arg, keep-lexer rule) → Task 1 Step 2. Gates G1/G3/G4/G-verbatim → Task 2/3 Steps 2-5; G5 (runtime) folded into Task 4 Step 2 (lib unit tests, no sandbox dep). Constraints (verbatim, no fmt, no widening, ff-merge-only) → Global Constraints + Task 4 Step 4. Out-of-scope (small files, other crates) → Task 4 Step 7 note. All spec sections covered.
- **Placeholder scan:** No TBD/TODO; every code/command step shows exact content and expected output. Mover and verifier embedded in full.
- **Type/name consistency:** `move_fns.py`/`verify.py` signatures, group names, pin names, and `--list` filters identical across Tasks 1-4. Group-spec strings in Task 2/3 match the spec tables and the de-risked run output (118/121/121/142/43/53/46/26/16/30 and 67/59/22/13/13/11/18/24). Pin names identical in Task 1 Step 6, Task 3 Step 1, and Constraints.
