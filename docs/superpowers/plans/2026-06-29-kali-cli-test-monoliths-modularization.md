# kali_cli Remaining Test-Monolith Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the 15 remaining ≥1000-line integration-test monoliths under `crates/kali_cli/tests/` into a thin facade + per-concern `#[path] mod` submodules, with zero behavior change and a byte-identical test name-set.

**Architecture:** This is the 23rd entry in the kali crate-modularization series and sub-project 3 of 3 for `kali_cli` (sub-project 2 applied the identical pattern to `runtime_smoke.rs`). Each monolith `tests/F.rs` becomes a facade keeping all `use` imports + non-`#[test]` helper fns + `#[path = "F/<mod>.rs"] mod <mod>;` declarations; each `tests/F/<mod>.rs` is exactly `use super::*;` followed by verbatim-moved `#[test]` fns. One test binary per file is preserved. Extraction is performed by a generalized version of the proven `.superpowers/sdd/move_fns.py` mover; correctness is guaranteed by a `cargo test --list` basename-multiset proof.

**Tech Stack:** Rust 2021 (cargo integration tests, `#[path]` modules), Python 3 (the `move_fns.py` mover), bash + `sed`/`diff` for the verification proofs.

## Global Constraints

- **Pure relocation refactor, not TDD.** No new product code, no new tests. The existing tests are the safety net; they must keep their exact names. "Show the code" for a relocation step means *name the file + the exact mover command + expected counts*, not paste thousands of moved lines — the operation is a mechanical, byte-faithful cut-paste performed by the mover.
- **Verbatim moves only.** `#[test]` fn bodies (and their attribute lines + one trailing blank line) relocate byte-for-byte. Never reformat, reorder, or "tidy".
- **Submodule header is exactly `use super::*;`** — nothing else. The root facade keeps every original `use`; descendant visibility makes the root's (private) imports and helpers reachable through the glob. No per-submodule extern `use`s.
- **Root facade ends with ZERO `#[test]` fns** per file. Non-`#[test]` helpers (including `#[cfg(...)]`-gated helpers that lack `#[test]`) stay in the facade.
- **No `pub`/`pub(crate)` widening.** `#[path] mod` children are descendants of the facade, so they already see the facade's private items.
- **Do NOT run `cargo fmt`.** The repo's `cargo fmt --all --check` gate is already red on baseline (10+ crates); the verbatim mandate forbids reformatting. Accepted cosmetic minors (>100-col lines, stray blanks) are not regressions.
- **Corrected baseline gates for this sandbox** (the literal "0 warnings / fully green" gates do NOT hold here):
  - **no-new-warnings** — `cargo build -p kali_cli --tests` warning count stays at the captured baseline (1 pre-existing `build/mod.rs:40 profile_data_hash unused_imports` lib-test warning); no NEW warnings.
  - **`--list` basename-multiset identical** — per file, strip the module-path prefix and the libtest suffix, `sort` *without* `-u`, diff against the captured baseline → empty. This is the mechanical completeness proof.
  - **runtime pass/fail-set unchanged** — there are pre-existing chromium-sandbox browser-bundle env failures (`No usable sandbox!`) on clean main; the *set of failing test names* must be identical before/after (diff the name-sets, expect empty). A shifted panic-site line number is expected (code-motion); the panic message is unchanged.
- **Integration policy:** work on `refactor/kali-cli-test-monoliths` off `main`. Integration is **local-`main` fast-forward merge ONLY — NEVER push to origin** (origin/main intentionally lags). Re-verify on merged main, then delete the branch.
- **SDD ledger:** `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite for this sub-project; it is the durable recovery map. Baseline `--list` snapshots live in `.superpowers/sdd/` (git-ignored).

---

## Task 1: Branch, generalize the mover, capture baselines

**Files:**
- Create branch: `refactor/kali-cli-test-monoliths`
- Modify (scratch, git-ignored): `.superpowers/sdd/move_fns.py`
- Create (scratch, git-ignored): `.superpowers/sdd/baseline-<file>.txt` (15 files), `.superpowers/sdd/baseline-build-warnings.txt`

**Interfaces:**
- Produces: a generalized mover invoked as
  `python3 .superpowers/sdd/move_fns.py <root_rs_relpath> "<groups-spec>"`
  run from `crates/kali_cli/`, where `<groups-spec>` is `name=p1,p2;name2=p3;misc=*` (`*` = catch-all, must be last). It derives the submodule dir from the root file stem, writes `tests/<stem>/<mod>.rs` (each `use super::*;` + verbatim fns), rewrites `tests/<stem>.rs` to drop moved fns and append `#[path]` mod decls, and **skips any group that captured zero fns**.

- [ ] **Step 1: Create the branch**

```bash
cd /workspace
git checkout main
git checkout -b refactor/kali-cli-test-monoliths
```

- [ ] **Step 2: Capture the build-warning baseline**

```bash
cd /workspace
cargo build -p kali_cli --tests 2>&1 | grep -c '^warning' \
  > .superpowers/sdd/baseline-build-warnings.txt
cat .superpowers/sdd/baseline-build-warnings.txt
```
Expected: a small integer (expected `1` — the pre-existing `build/mod.rs:40` lib-test warning). Record whatever value prints; it is the gate ceiling.

- [ ] **Step 3: Capture the `--list` basename baseline for all 15 files**

```bash
cd /workspace/crates/kali_cli
for f in package_corpus late_compat_browser_js_input node_api_surface schema_docs \
         browser_runtime_summary_fallback_js_input browser_non_literal_iterator_sources \
         late_compat_browser_tsx_input browser_object_keys_iteration \
         browser_for_await_frozen_set_map_constructor_result late_compat_browser_jsx_input \
         browser_runtime_summary_fallback_tsx_input browser_runtime_summary_fallback_jsx_input \
         browser_runtime_summary_fallback_ts_input browser_math_atan2_bracketed_root \
         browser_reflect_own_keys; do
  cargo test -p kali_cli --test "$f" -- --list 2>/dev/null \
    | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
    > "/workspace/.superpowers/sdd/baseline-$f.txt"
  echo "$f: $(wc -l < /workspace/.superpowers/sdd/baseline-$f.txt) tests"
done
```
Expected (test counts): `package_corpus: 206`, `late_compat_browser_js_input: 118`, `node_api_surface: 45`, `schema_docs: 22`, `browser_runtime_summary_fallback_js_input: 34`, `browser_non_literal_iterator_sources: 90`, `late_compat_browser_tsx_input: 22`, `browser_object_keys_iteration: 25`, `browser_for_await_frozen_set_map_constructor_result: 40`, `late_compat_browser_jsx_input: 18`, `browser_runtime_summary_fallback_tsx_input: 28`, `browser_runtime_summary_fallback_jsx_input: 28`, `browser_runtime_summary_fallback_ts_input: 27`, `browser_math_atan2_bracketed_root: 29`, `browser_reflect_own_keys: 44`.

- [ ] **Step 4: Generalize the mover**

Edit `.superpowers/sdd/move_fns.py`. **Keep `FN_RE`, `IDENT_CHARS`, and the entire `find_close_line(...)` function exactly as they are in the current file** (the string/comment/raw-string-aware brace lexer — do not touch it). Replace the module docstring, the `ROOT`/`GROUPS` constants, and `main()` with the generalized versions below.

Replace the top constants block (`ROOT = ...` and the `GROUPS = [...]` literal) with:

```python
import os

ROOT = sys.argv[1]                       # e.g. "tests/package_corpus.rs" (run from crates/kali_cli)
STEM = os.path.splitext(os.path.basename(ROOT))[0]   # e.g. "package_corpus"
SUBDIR = os.path.join(os.path.dirname(ROOT), STEM)   # e.g. "tests/package_corpus"


def parse_groups(spec):
    """spec: 'name=p1,p2;name2=p3;misc=*'  -> [(name, prefixes_tuple_or_None, out_path)].
    '*' (sole token) marks the catch-all group; it must be last."""
    groups = []
    for part in spec.split(";"):
        part = part.strip()
        if not part:
            continue
        name, _, prefs = part.partition("=")
        name = name.strip()
        prefs = prefs.strip()
        prefixes = None if prefs == "*" else tuple(p.strip() for p in prefs.split(","))
        out = os.path.join(SUBDIR, name + ".rs")
        groups.append((name, prefixes, out))
    return groups


GROUPS = parse_groups(sys.argv[2])
```

Replace `main()` with (the first/second passes that build `selected`, `remove_ranges`, and `group_spans` are unchanged from the current file — reproduced here in full for a standalone task):

```python
def main():
    with open(ROOT, "r") as f:
        content = f.read()
    lines = content.split("\n")  # file ends with \n -> last elem is ''

    # First pass: attribute-tracked selection of #[test] fns into groups.
    attr_lines = []
    selected = []
    i = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        s = line.strip()
        if s.startswith("#["):
            attr_lines.append((i, s))
            i += 1
            continue
        m = FN_RE.match(s)
        if m:
            name = m.group(1)
            has_test = attr_lines and any("test]" in a[1] for a in attr_lines)
            group = None
            if has_test:
                for (gname, prefixes, _out) in GROUPS:
                    if prefixes is None:        # catch-all (must be last)
                        group = gname
                        break
                    if name.startswith(prefixes):
                        group = gname
                        break
            if group is not None:
                attr_start = attr_lines[0][0] if attr_lines else i
                selected.append((attr_start, i, name, group))
            attr_lines = []
            i += 1
            continue
        if s and not s.startswith("//") and not s.startswith("#"):
            attr_lines = []
        i += 1

    # Second pass: span capture via the verbatim lexer.
    remove_ranges = []
    group_spans = {gname: [] for (gname, _p, _o) in GROUPS}
    for (attr_start, fn_line, name, group) in selected:
        close_line = find_close_line(lines, fn_line)
        if close_line is None:
            print(f"ERROR: no closing brace for fn {name} at line {fn_line + 1}", file=sys.stderr)
            sys.exit(1)
        end = close_line
        if end + 1 < n and lines[end + 1] == "":
            end = end + 1  # include one trailing blank line (inter-fn separator)
        span = (attr_start, end)
        remove_ranges.append(span)
        group_spans[group].append(span)

    # Build submodule files: header + verbatim spans. Skip EMPTY groups.
    def build(spans):
        out = ["use super::*;", ""]
        for (s, e) in spans:
            for k in range(s, e + 1):
                out.append(lines[k])
        text = "\n".join(out)
        if not text.endswith("\n"):
            text += "\n"
        return text

    os.makedirs(SUBDIR, exist_ok=True)
    nonempty = [(gname, out) for (gname, _p, out) in GROUPS if group_spans[gname]]
    for (gname, out_path) in nonempty:
        with open(out_path, "w") as f:
            f.write(build(group_spans[gname]))
        print(f"moved {gname} -> {out_path}: {len(group_spans[gname])} fns", file=sys.stderr)
    for (gname, _p, _o) in GROUPS:
        if not group_spans[gname]:
            print(f"skipped empty group: {gname}", file=sys.stderr)

    # Rewrite the facade: drop removed ranges, append #[path] mod decls (non-empty only).
    removed = [False] * n
    for (s, e) in remove_ranges:
        for k in range(s, e + 1):
            removed[k] = True
    root_lines = [lines[k] for k in range(n) if not removed[k]]
    while root_lines and root_lines[-1] == "":
        root_lines.pop()

    mod_block = [""]
    for (gname, out_path) in nonempty:
        rel = os.path.relpath(out_path, os.path.dirname(ROOT))  # e.g. "package_corpus/run.rs"
        mod_block.append(f'#[path = "{rel}"]')
        mod_block.append(f"mod {gname};")
        mod_block.append("")
    root_lines.extend(mod_block)
    root_text = "\n".join(root_lines)
    if not root_text.endswith("\n"):
        root_text += "\n"
    with open(ROOT, "w") as f:
        f.write(root_text)

    print(f"root lines removed: {sum(e - s + 1 for s, e in remove_ranges)}", file=sys.stderr)


if __name__ == "__main__":
    main()
```

- [ ] **Step 5: Smoke-test the mover on a clean copy WITHOUT mutating the tree**

```bash
cd /workspace/crates/kali_cli
cp tests/browser_math_atan2_bracketed_root.rs /tmp/atan2_orig.rs
python3 /workspace/.superpowers/sdd/move_fns.py tests/browser_math_atan2_bracketed_root.rs \
  "run=run,json_run;build=build,json_build;check=check,json_check;test=test,json_test;misc=*"
# Expect stderr: "moved build ...", "moved run ...", "skipped empty group: check", "skipped empty group: test", "skipped empty group: misc"
ls tests/browser_math_atan2_bracketed_root/
grep -c '#\[test\]' tests/browser_math_atan2_bracketed_root.rs   # expect 0
# revert the smoke test — Task 16 will redo it for real:
git checkout -- tests/browser_math_atan2_bracketed_root.rs
rm -rf tests/browser_math_atan2_bracketed_root/
```
Expected: two submodules (`build.rs`, `run.rs`) created, three empty groups skipped, facade has 0 `#[test]`, then a clean revert. This validates the generalized mover before any real task.

- [ ] **Step 6: No commit (scratch only)**

The mover and baseline files are git-ignored scratch; there is nothing to commit in Task 1. The branch is ready.

---

## Per-File Extraction Recipe (steps R1–R6)

Every file task (Tasks 2–16) executes these six steps. The task supplies only: the **file stem `F`**, the **groups-spec**, the **expected populated modules**, and the **commit message**. All commands run from `/workspace/crates/kali_cli` unless noted.

- **R1 — Run the mover:**
  `python3 /workspace/.superpowers/sdd/move_fns.py tests/F.rs "<groups-spec>"`
  Read the stderr summary; note which groups were populated vs skipped.
- **R2 — Verify the facade is drained:**
  `grep -c '#\[test\]' tests/F.rs` → **must print `0`**.
- **R3 — Build with no new warnings:**
  `cd /workspace && cargo build -p kali_cli --tests 2>&1 | grep -c '^warning'`
  → must equal `cat .superpowers/sdd/baseline-build-warnings.txt`. If higher, a NEW warning was introduced — investigate before continuing.
- **R4 — `--list` basename-multiset proof:**
  ```bash
  cd /workspace/crates/kali_cli
  cargo test -p kali_cli --test F -- --list 2>/dev/null \
    | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
    | diff - /workspace/.superpowers/sdd/baseline-F.txt
  ```
  → **must print nothing** (identical multiset; the new `<mod>::` prefix is stripped by `s/^.*:://`).
- **R5 — Runtime fail-set confirmation (best-effort):**
  ```bash
  cd /workspace
  cargo test -p kali_cli --test F 2>&1 \
    | grep -E '^test .* \.\.\. FAILED$' | sed -E 's/^test //; s/ \.\.\. FAILED$//; s/^.*:://' | sort \
    > /tmp/F-fails-after.txt
  ```
  Compare against a before-snapshot taken on the branch base for the same file (capture it the same way before R1, or reuse a baseline fail snapshot). Expect an identical name-set (pre-existing `No usable sandbox!` browser-bundle failures are unchanged). If the file has no browser-bundle tests, expect an empty fail-set both before and after.
- **R6 — Commit:**
  ```bash
  cd /workspace
  git add crates/kali_cli/tests/F.rs crates/kali_cli/tests/F/
  git commit -m "<commit message>"
  ```

**Command-axis groups-spec (used verbatim by all command-axis files):**
```
run=run,json_run;build=build,json_build;check=check,json_check;test=test,json_test;misc=*
```
Empty groups auto-skip, so the populated modules are whatever subset the file actually exercises.

---

## Task 2 (TG1): Split `package_corpus.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/package_corpus.rs`
- Create → submodules: `crates/kali_cli/tests/package_corpus/{browser_runtime,browser_corpus,utility,node,misc}.rs`

**Groups-spec (corpus-kind axis — fn names do not lead with a command verb):**
```
browser_runtime=browser_runtime,json_browser_runtime;browser_corpus=browser_corpus;utility=utility_corpus;node=node_;misc=*
```

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = package_corpus` and the groups-spec above.
  - Expected populated modules: `browser_runtime`, `browser_corpus`, `utility`, `node`, `misc` (the catch-all `misc` collects `jsr_*`/`deno_*`/`binary_*`/`package_*`/`native_*`/`inherited_*`/`default_*` + any stray `json_*`).
  - R2 expects `0`; R4 diffs against `baseline-package_corpus.txt` (206 names) → empty.
  - R6 commit message: `refactor(kali_cli): split package_corpus.rs into corpus-kind test submodules [refactor]`

---

## Task 3 (TG2): Split `late_compat_browser_js_input.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/late_compat_browser_js_input.rs`
- Create → submodules: `crates/kali_cli/tests/late_compat_browser_js_input/{run,build,check,test,misc}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = late_compat_browser_js_input` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `build`, `check`, `test`, `misc` (misc = `browser_late_*` / `sequence_wrapped_*` etc.).
  - R4 diffs against `baseline-late_compat_browser_js_input.txt` (118 names) → empty.
  - R6 commit message: `refactor(kali_cli): split late_compat_browser_js_input.rs into per-command test submodules [refactor]`

---

## Task 4 (TG2): Split `node_api_surface.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/node_api_surface.rs`
- Create → submodules: `crates/kali_cli/tests/node_api_surface/{core,explicit,inherited,process,misc}.rs`

**Groups-spec (semantic axis):**
```
core=node_api;explicit=explicit_node;inherited=inherited_node;process=process_kill,process_;misc=*
```

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = node_api_surface` and the groups-spec above.
  - Expected populated modules: `core`, `explicit`, `inherited`, `process`, and `misc` if any names remain.
  - R4 diffs against `baseline-node_api_surface.txt` (45 names) → empty.
  - R6 commit message: `refactor(kali_cli): split node_api_surface.rs into per-category test submodules [refactor]`

---

## Task 5 (TG2): Split `schema_docs.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/schema_docs.rs`
- Create → submodules: `crates/kali_cli/tests/schema_docs/{plan,proof,misc}.rs`

**Groups-spec (semantic axis):**
```
plan=active_plan;proof=proof_,collect_proof,specialized_artifact;misc=*
```

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = schema_docs` and the groups-spec above.
  - Expected populated modules: `plan`, `proof`, `misc` (misc holds the scattered remainder).
  - R4 diffs against `baseline-schema_docs.txt` (22 names) → empty.
  - R6 commit message: `refactor(kali_cli): split schema_docs.rs into grouped test submodules [refactor]`

---

## Task 6 (TG3): Split `browser_non_literal_iterator_sources.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_non_literal_iterator_sources.rs`
- Create → submodules: `crates/kali_cli/tests/browser_non_literal_iterator_sources/{run,build,check,test,misc}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_non_literal_iterator_sources` and the **command-axis groups-spec**.
  - Expected populated modules: `build`, `check` (the bulk), plus small `run`/`test` if present; `misc` if any remain.
  - R4 diffs against `baseline-browser_non_literal_iterator_sources.txt` (90 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_non_literal_iterator_sources.rs into per-command test submodules [refactor]`

---

## Task 7 (TG3): Split `browser_reflect_own_keys.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_reflect_own_keys.rs`
- Create → submodules: `crates/kali_cli/tests/browser_reflect_own_keys/{run,build,check,test}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_reflect_own_keys` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `test`, `build`, `check` (`misc` expected empty → skipped).
  - R4 diffs against `baseline-browser_reflect_own_keys.txt` (44 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_reflect_own_keys.rs into per-command test submodules [refactor]`

---

## Task 8 (TG3): Split `browser_for_await_frozen_set_map_constructor_result.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_for_await_frozen_set_map_constructor_result.rs`
- Create → submodules: `crates/kali_cli/tests/browser_for_await_frozen_set_map_constructor_result/{run,build,test}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_for_await_frozen_set_map_constructor_result` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `test`, `build` (`check`/`misc` expected empty → skipped).
  - R4 diffs against `baseline-browser_for_await_frozen_set_map_constructor_result.txt` (40 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_for_await_frozen_set_map_constructor_result.rs into per-command test submodules [refactor]`

---

## Task 9 (TG3): Split `browser_runtime_summary_fallback_js_input.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_runtime_summary_fallback_js_input.rs`
- Create → submodules: `crates/kali_cli/tests/browser_runtime_summary_fallback_js_input/{run,test}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_runtime_summary_fallback_js_input` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `test` (`build`/`check`/`misc` expected empty → skipped).
  - R4 diffs against `baseline-browser_runtime_summary_fallback_js_input.txt` (34 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_runtime_summary_fallback_js_input.rs into per-command test submodules [refactor]`

---

## Task 10 (TG4): Split `late_compat_browser_tsx_input.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/late_compat_browser_tsx_input.rs`
- Create → submodules: `crates/kali_cli/tests/late_compat_browser_tsx_input/{run,build,check,test,misc}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = late_compat_browser_tsx_input` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `build`, `check`, `test`, `misc` (misc = `browser_late_*`).
  - R4 diffs against `baseline-late_compat_browser_tsx_input.txt` (22 names) → empty.
  - R6 commit message: `refactor(kali_cli): split late_compat_browser_tsx_input.rs into per-command test submodules [refactor]`

---

## Task 11 (TG4): Split `late_compat_browser_jsx_input.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/late_compat_browser_jsx_input.rs`
- Create → submodules: `crates/kali_cli/tests/late_compat_browser_jsx_input/{run,build,check,misc}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = late_compat_browser_jsx_input` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `check`, `build` and/or `misc` per what is present (`test` expected empty → skipped; `run_and_test_*` names lead with `run` → `run`).
  - R4 diffs against `baseline-late_compat_browser_jsx_input.txt` (18 names) → empty.
  - R6 commit message: `refactor(kali_cli): split late_compat_browser_jsx_input.rs into per-command test submodules [refactor]`

---

## Task 12 (TG4): Split `browser_runtime_summary_fallback_tsx_input.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_runtime_summary_fallback_tsx_input.rs`
- Create → submodules: `crates/kali_cli/tests/browser_runtime_summary_fallback_tsx_input/{run,test}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_runtime_summary_fallback_tsx_input` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `test`.
  - R4 diffs against `baseline-browser_runtime_summary_fallback_tsx_input.txt` (28 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_runtime_summary_fallback_tsx_input.rs into per-command test submodules [refactor]`

---

## Task 13 (TG4): Split `browser_runtime_summary_fallback_jsx_input.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_runtime_summary_fallback_jsx_input.rs`
- Create → submodules: `crates/kali_cli/tests/browser_runtime_summary_fallback_jsx_input/{run,test}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_runtime_summary_fallback_jsx_input` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `test`.
  - R4 diffs against `baseline-browser_runtime_summary_fallback_jsx_input.txt` (28 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_runtime_summary_fallback_jsx_input.rs into per-command test submodules [refactor]`

---

## Task 14 (TG4): Split `browser_runtime_summary_fallback_ts_input.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_runtime_summary_fallback_ts_input.rs`
- Create → submodules: `crates/kali_cli/tests/browser_runtime_summary_fallback_ts_input/{run,test}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_runtime_summary_fallback_ts_input` and the **command-axis groups-spec**.
  - Expected populated modules: `run`, `test`.
  - R4 diffs against `baseline-browser_runtime_summary_fallback_ts_input.txt` (27 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_runtime_summary_fallback_ts_input.rs into per-command test submodules [refactor]`

---

## Task 15 (TG4): Split `browser_object_keys_iteration.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_object_keys_iteration.rs`
- Create → submodules: `crates/kali_cli/tests/browser_object_keys_iteration/{build,build_json}.rs`

**Groups-spec (single-command → output-mode split):**
```
build_json=json_build;build=build;misc=*
```

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_object_keys_iteration` and the groups-spec above.
  - Expected populated modules: `build_json` (the `json_build_*` tests), `build` (the plain `build_*` tests); `misc` expected empty → skipped.
  - R4 diffs against `baseline-browser_object_keys_iteration.txt` (25 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_object_keys_iteration.rs by output mode [refactor]`

---

## Task 16 (TG4): Split `browser_math_atan2_bracketed_root.rs`

**Files:**
- Modify → facade: `crates/kali_cli/tests/browser_math_atan2_bracketed_root.rs`
- Create → submodules: `crates/kali_cli/tests/browser_math_atan2_bracketed_root/{build,run}.rs`

- [ ] **Step 1:** Execute recipe steps **R1–R6** with `F = browser_math_atan2_bracketed_root` and the **command-axis groups-spec**.
  - Expected populated modules: `build`, `run` (`check`/`test`/`misc` expected empty → skipped). This is the file used for the Task-1 smoke test; this task performs the real, committed split.
  - R4 diffs against `baseline-browser_math_atan2_bracketed_root.txt` (29 names) → empty.
  - R6 commit message: `refactor(kali_cli): split browser_math_atan2_bracketed_root.rs into per-command test submodules [refactor]`

---

## Task 17: Whole-branch finalize verification + integration

**Files:**
- No source changes. Verification + merge only.

**Interfaces:**
- Consumes: the 15 split commits from Tasks 2–16 on `refactor/kali-cli-test-monoliths`.

- [ ] **Step 1: Confirm every targeted facade is drained**

```bash
cd /workspace/crates/kali_cli
for f in package_corpus late_compat_browser_js_input node_api_surface schema_docs \
         browser_runtime_summary_fallback_js_input browser_non_literal_iterator_sources \
         late_compat_browser_tsx_input browser_object_keys_iteration \
         browser_for_await_frozen_set_map_constructor_result late_compat_browser_jsx_input \
         browser_runtime_summary_fallback_tsx_input browser_runtime_summary_fallback_jsx_input \
         browser_runtime_summary_fallback_ts_input browser_math_atan2_bracketed_root \
         browser_reflect_own_keys; do
  printf '%-55s %s\n' "$f" "$(grep -c '#\[test\]' tests/$f.rs)"
done
```
Expected: every count is `0`.

- [ ] **Step 2: Whole-crate build — no new warnings**

```bash
cd /workspace
cargo build -p kali_cli --tests 2>&1 | grep -c '^warning'
```
Expected: equals `cat .superpowers/sdd/baseline-build-warnings.txt`.

- [ ] **Step 3: Re-run all 15 `--list` basename proofs**

```bash
cd /workspace/crates/kali_cli
fail=0
for f in package_corpus late_compat_browser_js_input node_api_surface schema_docs \
         browser_runtime_summary_fallback_js_input browser_non_literal_iterator_sources \
         late_compat_browser_tsx_input browser_object_keys_iteration \
         browser_for_await_frozen_set_map_constructor_result late_compat_browser_jsx_input \
         browser_runtime_summary_fallback_tsx_input browser_runtime_summary_fallback_jsx_input \
         browser_runtime_summary_fallback_ts_input browser_math_atan2_bracketed_root \
         browser_reflect_own_keys; do
  d=$(cargo test -p kali_cli --test "$f" -- --list 2>/dev/null \
        | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
        | diff - "/workspace/.superpowers/sdd/baseline-$f.txt")
  [ -n "$d" ] && { echo "MISMATCH: $f"; echo "$d"; fail=1; }
done
[ "$fail" = 0 ] && echo "ALL 15 BASENAME-MULTISETS MATCH"
```
Expected: `ALL 15 BASENAME-MULTISETS MATCH`.

- [ ] **Step 4: Request the whole-branch review**

Use superpowers:requesting-code-review for the full branch diff (`git diff main...refactor/kali-cli-test-monoliths`). The reviewer confirms: verbatim moves (no body edits), facades drained, submodule headers are exactly `use super::*;`, no `pub`/`pub(crate)` widening, no `cargo fmt` reflow. Save the diff to `.superpowers/sdd/` per series convention.

- [ ] **Step 5: Fast-forward merge to local main (NEVER push origin)**

```bash
cd /workspace
git checkout main
git merge --ff-only refactor/kali-cli-test-monoliths
```
Expected: fast-forward succeeds (15 + spec commits ahead of origin).

- [ ] **Step 6: Re-verify on merged main, then delete the branch**

Re-run Steps 1–3 on `main`. When green, delete the branch:
```bash
git branch -d refactor/kali-cli-test-monoliths
```

- [ ] **Step 7: Update the memory ledger**

Update `/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md` and `MEMORY.md`: mark kali_cli sub-project 3 (23rd) DONE with the merged-main commit hash, note all 15 monoliths split, and that the generalized `move_fns.py` (CLI groups-spec) is reusable for future test-grouping work.

---

## Self-Review

**1. Spec coverage.** Every spec section maps to a task: method/wiring → Per-File Recipe + Global Constraints; the 15-file scheme table → Tasks 2–16 (one per file; `package_corpus` uses the corrected corpus-kind axis, `node_api_surface`/`schema_docs` semantic, `browser_object_keys_iteration` output-mode, the rest command-axis); sequencing TG1–TG4 → Task grouping (TG1=Task2, TG2=Tasks3–5, TG3=Tasks6–9, TG4=Tasks10–16); verification gates → R3/R4/R5 + Task 17; integration policy → Task 17 Steps 5–6 + ledger update Step 7. No spec requirement is unaddressed.

**2. Placeholder scan.** No "TBD"/"TODO"/"handle edge cases". Each task carries its exact groups-spec, expected modules, expected baseline file + count, and commit message. The mover code in Task 1 is complete except `find_close_line`/`FN_RE`/`IDENT_CHARS`, which are explicitly preserved verbatim from the existing file (not a placeholder — a deliberate "do not touch" instruction with an exact pointer).

**3. Type/name consistency.** The mover CLI contract (`move_fns.py <root_rs> "<groups-spec>"`, `*`=catch-all-last, empty-group-skip, `use super::*;` headers) is defined once in Task 1 and referenced identically by R1 and every task. Groups-spec grammar (`name=p1,p2;...;misc=*`) is consistent across all 15 specs. Baseline filenames (`baseline-<F>.txt`) match between Task 1 Step 3 (capture) and R4/Task 17 (proof). Module names in each task's **Files** block match the groups-spec group names.
