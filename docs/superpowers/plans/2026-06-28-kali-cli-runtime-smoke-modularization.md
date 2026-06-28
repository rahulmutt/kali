# kali_cli `runtime_smoke.rs` modularization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `tests/runtime_smoke.rs` (73,625 lines, 1,816 `#[test]` fns) into a root (shared helpers + `use` block + `mod` decls) plus 8 per-command submodules under `tests/runtime_smoke/`, keeping it a single integration-test binary with zero behavior change.

**Architecture:** Pure code-motion. `runtime_smoke.rs` stays one integration-test binary; tests relocate verbatim into `tests/runtime_smoke/{run,test,build,check,package,effects,install,misc}.rs` via `#[path] mod` declarations in the root, each submodule beginning with `use super::*;`. The ~183 shared helpers and the `use` block stay in the root untouched (children see parent privates, so no `pub(crate)` widenings). Executed in 3 size-ordered move-tasks (run+test → build+check+package → effects+install+misc) so the biggest payoff lands first and each task ends green. (The spec framed this as a "single task"; the plan decomposes it into 3 move-tasks for SDD reviewability and intermediate verification gates on a 73K-line file — each task ends green with the count + name-set invariant re-verified.)

**Tech Stack:** Rust (edition 2021), cargo workspace. Crate `kali_cli` at `crates/kali_cli/`. Integration test binary `runtime_smoke` at `tests/runtime_smoke.rs` (drives the compiled `kali` binary via `CARGO_BIN_EXE_kali`).

## Global Constraints

(Copied verbatim from the spec `docs/superpowers/specs/2026-06-28-kali-cli-runtime-smoke-modularization-design.md`, with the `--list` invariant corrected — see Verification. Every task's requirements implicitly include these.)

- **Pure code-motion, zero behavior change.** Same 1,816 tests exist and pass before and after.
- **`--list` invariant (corrected).** The spec's literal "`--list` diff = EMPTY" is **not** achievable with the `#[path] mod` mechanism: `runtime_smoke`'s tests are top-level today (bare names in `--list`), and nesting them in `mod run`/`mod test`/… adds a module-path prefix (verified — `build_tests` lists as `build::tests::fn_name`). The corrected invariant is: **(a) test COUNT unchanged (1,816); (b) test FN-NAME SET unchanged, modulo the new module prefix — `run_supports_x` becomes `run::run_supports_x`; (c) pass/fail unchanged.** This is behavior-neutral: `cargo test --test runtime_smoke -- <filter>` substring-matches against the full path, so `run_supports_x` still matches `run::run_supports_x`. (Phase-1's "diff EMPTY" held only because `build_tests`/`output_tests` were already nested in `mod tests` before and after.)
- **Allowed changes only:** `#[path] mod <name>;` declarations in the root; `use super::*;` as the first line of each submodule; verbatim relocation of `#[test]` fn bodies (attribute lines intact). No rewriting of bodies. **No `pub(crate)`** (none needed — child modules see parent privates; `use super::*` reaches helpers + the root `use` block natively).
- **No `cargo fmt`.** Verbatim moves + the root's existing >100-col lines are already red on baseline — not a regression.
- **0 warnings gate.** Every task ends with `cargo build -p kali_cli --tests` at 0 warnings. Glob `use super::*` does not trigger `unused_imports` for unused items.
- **Verbatim-move convention.** "Move" = cut a `#[test]` fn's attribute line(s) (`#[test]`, and `#[cfg(unix)]` where present) + body byte-for-byte from the root and paste into the target submodule after the `use super::*;` line. **Helpers (fns WITHOUT a `#[test]` attribute — including the cfg-gated `run_browser_entrypoint_smoke`, `test_browser_entrypoint_smoke`, `browser_entrypoint_smoke`, `shell_quote_path`) STAY in the root — never move them.** The enumeration script below filters by the `#[test]` attribute, so it auto-excludes these helpers; do not move by name-prefix alone.
- **Assignment rule (deterministic, exhaustive):** `json_<cmd>_…` → the `<cmd>` module; `<cmd>_…` for `cmd ∈ {run, test, build, check, package, effects, install}` → that module; everything else (`fmt`/`lint`/`doctor`/`init` incl. `json_` variants, plus the long-tail `browser_*`/`late_*`/`node_*`/`permission_*`/…) → `misc.rs`. No test straddles two modules.
- **Integration:** local-main ff-merge only — **NEVER push to origin** (origin/main intentionally lags). Re-verify build+test on merged main, then delete the branch.
- **SDD ledger:** overwrite `.superpowers/sdd/progress.md` per task (git-ignored scratch) — it's the durable recovery map.

## File Structure

```
tests/runtime_smoke.rs          root: use block + ~183 helpers (UNCHANGED) + 8 #[path] mod decls
tests/runtime_smoke/run.rs      run_*   + json_run_*     (582 tests)  — starts with `use super::*;`
tests/runtime_smoke/test.rs     test_*  + json_test_*    (473 tests)  — starts with `use super::*;`
tests/runtime_smoke/build.rs    build_* + json_build_*   (320 tests)  — starts with `use super::*;`
tests/runtime_smoke/check.rs    check_* + json_check_*   (164 tests)  — starts with `use super::*;`
tests/runtime_smoke/package.rs  package_* + json_package_* (123 tests)— starts with `use super::*;`
tests/runtime_smoke/effects.rs  effects_* + json_effects_* (70 tests)— starts with `use super::*;`
tests/runtime_smoke/install.rs  install_* + json_install_* (23 tests)— starts with `use super::*;`
tests/runtime_smoke/misc.rs     long-tail + fmt/lint/doctor/init + json_{fmt,lint,init} (61 tests)— starts with `use super::*;`
```

## Verification commands (used in every move-task)

```bash
cd /workspace
# 0 warnings (compile the test binary)
cargo build -p kali_cli --tests 2>&1 | tee /tmp/rs_build.txt
# test count unchanged (expect 1816)
cargo test -p kali_cli --test runtime_smoke -- --list 2>/dev/null | grep -c ': test$'
# fn-name set unchanged: strip the new module prefix + ': test' suffix, diff vs baseline
cargo test -p kali_cli --test runtime_smoke -- --list 2>/dev/null \
  | sed -E 's/^.*:://; s/: test$//' | sort > /tmp/rs_names_now.txt
diff /tmp/rs_baseline_names.txt /tmp/rs_names_now.txt && echo "NAME SET UNCHANGED"
# run the suite (runtime_smoke is fully green on baseline — the 2 build_bundles_* failures
# live in tests/array_from_bracketed_root_wrappers.rs, a DIFFERENT binary)
cargo test -p kali_cli --test runtime_smoke 2>&1 | tee /tmp/rs_test.txt
```

The `sed` keeps only the fn name (last path segment): baseline lines have no `::` so `^.*::` doesn't match and only `: test` is stripped; after the refactor `run::run_supports_x: test` becomes `run_supports_x`. Either way the output is the bare fn name, so the diff is empty iff no test was added/removed/renamed.

## Move-enumeration script (parameterized; used in every move-task)

Lists the **exact** `#[test]` fns currently in the root whose name starts with any given prefix, plus a count on stderr. Run it before cutting to get the authoritative move-list and to confirm the expected count. It filters by the `#[test]` attribute, so cfg-gated helpers (`run_browser_entrypoint_smoke`, etc.) are automatically excluded.

```bash
python3 - "$@" <<'PY'
import re, sys
prefixes = tuple(sys.argv[1:])           # e.g. run json_run
src = open('tests/runtime_smoke.rs').read().splitlines()
attrs = []
n = 0
for l in src:
    s = l.strip()
    if s.startswith('#['):
        attrs.append(s); continue
    m = re.match(r'fn ([A-Za-z0-9_]+)\b', s)
    if m:
        name = m.group(1)
        if any('test]' in a for a in attrs) and name.startswith(prefixes):
            print(name); n += 1
        attrs = []; continue
    if s and not s.startswith('//') and not s.startswith('#'):
        attrs = []
print(f"# {n} test fns to move (prefixes={prefixes})", file=sys.stderr)
PY
```

For `misc.rs` (Task 3), enumerate ALL remaining `#[test]` fns in the root (no prefix filter) — see Task 3 for that variant.

---

### Task 0: Baseline & branch setup

**Files:**
- Modify: `.superpowers/sdd/progress.md` (git-ignored scratch — overwrite)

**Interfaces:** Produces the baseline name-set snapshot (`/tmp/rs_baseline_names.txt`) and the `refactor/kali-cli-runtime-smoke-modularization` branch that all later tasks build on.

- [ ] **Step 1: Create the refactor branch off main**

```bash
cd /workspace
git checkout main
git checkout -b refactor/kali-cli-runtime-smoke-modularization
git rev-parse HEAD   # record as branch-base in the ledger
```

- [ ] **Step 2: Capture the baseline name-set + confirm runtime_smoke is fully green**

```bash
cargo test -p kali_cli --test runtime_smoke -- --list 2>/dev/null \
  | sed -E 's/^.*:://; s/: test$//' | sort > /tmp/rs_baseline_names.txt
wc -l /tmp/rs_baseline_names.txt     # expect 1816
cargo build -p kali_cli --tests 2>&1 | tee /tmp/rs_baseline_build.txt   # expect 0 warnings
cargo test -p kali_cli --test runtime_smoke 2>&1 | tee /tmp/rs_baseline_test.txt
```

Expected: build clean (0 warnings); `--list` name count = 1816; the `runtime_smoke` test run is **fully green** (0 failures). The 2 pre-existing `build_bundles_*` baseline failures live in `tests/array_from_bracketed_root_wrappers.rs` — a DIFFERENT test binary — and do NOT appear here. If a `runtime_smoke` failure appears, STOP — the baseline is not what the spec assumes; reconcile before proceeding.

- [ ] **Step 3: Record the ledger**

Write `.superpowers/sdd/progress.md` with: branch-base HEAD, `runtime_smoke` `--list` count (1816), confirmation that `runtime_smoke` is fully green on baseline, and the per-module expected counts (run 582 / test 473 / build 320 / check 164 / package 123 / effects 70 / install 23 / misc 61).

---

### Task 1: `run.rs` + `test.rs` (1,055 tests)

**Files:**
- Create: `tests/runtime_smoke/run.rs`, `tests/runtime_smoke/test.rs`
- Modify: `tests/runtime_smoke.rs` (remove the moved `#[test]` fns; add 2 `#[path] mod` decls)

**Interfaces:**
- Consumes: the root's `use` block + all ~183 helpers, reached via `use super::*;` (no per-module `use` block).
- Produces: nothing (leaf test submodules).

- [ ] **Step 1: Enumerate the run + test test fns to move**

```bash
cd /workspace/crates/kali_cli
python3 - run json_run <<'PY'
import re, sys
prefixes = tuple(sys.argv[1:])
src = open('tests/runtime_smoke.rs').read().splitlines()
attrs = []; n = 0
for l in src:
    s = l.strip()
    if s.startswith('#['): attrs.append(s); continue
    m = re.match(r'fn ([A-Za-z0-9_]+)\b', s)
    if m:
        name = m.group(1)
        if any('test]' in a for a in attrs) and name.startswith(prefixes):
            print(name); n += 1
        attrs = []; continue
    if s and not s.startswith('//') and not s.startswith('#'): attrs = []
print(f"# {n} test fns to move (prefixes={prefixes})", file=sys.stderr)
PY
```

Expected stderr: `# 582 test fns to move (prefixes=('run', 'json_run'))`. Repeat with `test json_test` → expect `# 473 test fns to move`. If the counts differ, STOP and reconcile (a fn may have been mis-attributed by the assignment rule).

- [ ] **Step 2: Create the two submodule files with their `use super::*;` header**

```bash
cd /workspace/crates/kali_cli
mkdir -p tests/runtime_smoke
printf 'use super::*;\n\n' > tests/runtime_smoke/run.rs
printf 'use super::*;\n\n' > tests/runtime_smoke/test.rs
```

- [ ] **Step 3: Move the run + test `#[test]` fns verbatim from the root into the submodules**

For each fn name listed in Step 1 (run/json_run group): cut its attribute line(s) (`#[test]`, and `#[cfg(unix)]` where present) + body byte-for-byte from `tests/runtime_smoke.rs` and append it to `tests/runtime_smoke/run.rs`. Do the same for the test/json_test group into `tests/runtime_smoke/test.rs`. Do NOT move any helper (fns without `#[test]`, e.g. `run_browser_entrypoint_smoke`, `start_registry_metadata_server`, `assert_json_run_supports_*`) — they stay in the root. After this step, `tests/runtime_smoke.rs` contains no `#[test]` fn whose name starts with `run_`/`json_run_`/`test_`/`json_test_`.

- [ ] **Step 4: Wire the `#[path] mod` declarations in the root**

Append to `tests/runtime_smoke.rs` (after the last helper, at EOF):

```rust
#[path = "runtime_smoke/run.rs"]
mod run;

#[path = "runtime_smoke/test.rs"]
mod test;
```

- [ ] **Step 5: Verify**

Run the Verification commands block above. Expected: build 0 warnings; `grep -c ': test$'` = 1816; `diff /tmp/rs_baseline_names.txt /tmp/rs_names_now.txt` empty (NAME SET UNCHANGED); `cargo test -p kali_cli --test runtime_smoke` fully green. Spot-run a moved test to confirm it resolves in its new module:

```bash
cargo test -p kali_cli --test runtime_smoke -- run:: 2>&1 | tail -5   # runs all run-module tests
cargo test -p kali_cli --test runtime_smoke -- json_run_ 2>&1 | tail -5  # substring filter still matches
```

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add crates/kali_cli/tests/runtime_smoke.rs crates/kali_cli/tests/runtime_smoke/run.rs crates/kali_cli/tests/runtime_smoke/test.rs
git commit -m "refactor(kali_cli): extract runtime_smoke run/test submodules [refactor]"
```

Update `.superpowers/sdd/progress.md`.

---

### Task 2: `build.rs` + `check.rs` + `package.rs` (607 tests)

**Files:**
- Create: `tests/runtime_smoke/build.rs`, `tests/runtime_smoke/check.rs`, `tests/runtime_smoke/package.rs`
- Modify: `tests/runtime_smoke.rs` (remove moved fns; add 3 `#[path] mod` decls)

**Interfaces:**
- Consumes: root helpers + `use` block via `use super::*;`.
- Produces: nothing.

- [ ] **Step 1: Enumerate the build + check + package test fns to move**

Run the enumeration script from Task 1 Step 1 three times with these prefixes and verify the counts:

| group | prefixes | expected count |
|---|---|---:|
| build | `build json_build` | 320 |
| check | `check json_check` | 164 |
| package | `package json_package` | 123 |

```bash
cd /workspace/crates/kali_cli
python3 - build json_build <<'PY'
import re, sys
prefixes = tuple(sys.argv[1:])
src = open('tests/runtime_smoke.rs').read().splitlines()
attrs = []; n = 0
for l in src:
    s = l.strip()
    if s.startswith('#['): attrs.append(s); continue
    m = re.match(r'fn ([A-Za-z0-9_]+)\b', s)
    if m:
        name = m.group(1)
        if any('test]' in a for a in attrs) and name.startswith(prefixes):
            print(name); n += 1
        attrs = []; continue
    if s and not s.startswith('//') and not s.startswith('#'): attrs = []
print(f"# {n} test fns to move (prefixes={prefixes})", file=sys.stderr)
PY
```

(Repeat for `check json_check` → 164, `package json_package` → 123.) If any count differs, STOP and reconcile.

- [ ] **Step 2: Create the three submodule files**

```bash
cd /workspace/crates/kali_cli
printf 'use super::*;\n\n' > tests/runtime_smoke/build.rs
printf 'use super::*;\n\n' > tests/runtime_smoke/check.rs
printf 'use super::*;\n\n' > tests/runtime_smoke/package.rs
```

- [ ] **Step 3: Move the build/check/package `#[test]` fns verbatim**

Cut each enumerated fn's attribute line(s) + body from `tests/runtime_smoke.rs` and append to the matching submodule (`build.rs` / `check.rs` / `package.rs`). Helpers (e.g. `assert_build_supports_*`, `package_audit_metadata_body*`) stay in the root.

- [ ] **Step 4: Wire the `#[path] mod` declarations in the root**

Append to `tests/runtime_smoke.rs`:

```rust
#[path = "runtime_smoke/build.rs"]
mod build;

#[path = "runtime_smoke/check.rs"]
mod check;

#[path = "runtime_smoke/package.rs"]
mod package;
```

- [ ] **Step 5: Verify**

Run the Verification commands block. Expected: build 0 warnings; count 1816; NAME SET UNCHANGED; suite fully green. Spot-run: `cargo test -p kali_cli --test runtime_smoke -- build:: 2>&1 | tail -5`.

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add crates/kali_cli/tests/runtime_smoke.rs crates/kali_cli/tests/runtime_smoke/build.rs crates/kali_cli/tests/runtime_smoke/check.rs crates/kali_cli/tests/runtime_smoke/package.rs
git commit -m "refactor(kali_cli): extract runtime_smoke build/check/package submodules [refactor]"
```

Update the ledger.

---

### Task 3: `effects.rs` + `install.rs` + `misc.rs` (154 tests)

**Files:**
- Create: `tests/runtime_smoke/effects.rs`, `tests/runtime_smoke/install.rs`, `tests/runtime_smoke/misc.rs`
- Modify: `tests/runtime_smoke.rs` (remove ALL remaining `#[test]` fns; add 3 `#[path] mod` decls)

**Interfaces:**
- Consumes: root helpers + `use` block via `use super::*;`.
- Produces: nothing. After this task the root contains **no `#[test]` fns** — only the `use` block + helpers + 8 `#[path] mod` decls.

- [ ] **Step 1: Enumerate the effects + install test fns, then ALL remaining (misc)**

For effects and install, use the prefix script (Task 1 Step 1) with prefixes `effects json_effects` (expect 70) and `install json_install` (expect 23). For `misc`, enumerate **every `#[test]` fn still in the root** (no prefix filter) — these are the long-tail (`browser_*`, `late_*`, `node_*`, `permission_*`, `release_*`, singletons) plus `fmt`/`lint`/`doctor`/`init` and `json_fmt`/`json_lint`/`json_init`:

```bash
cd /workspace/crates/kali_cli
python3 - <<'PY'
import re, sys
src = open('tests/runtime_smoke.rs').read().splitlines()
attrs = []; n = 0
for l in src:
    s = l.strip()
    if s.startswith('#['): attrs.append(s); continue
    m = re.match(r'fn ([A-Za-z0-9_]+)\b', s)
    if m:
        name = m.group(1)
        if any('test]' in a for a in attrs):
            print(name); n += 1
        attrs = []; continue
    if s and not s.startswith('//') and not s.startswith('#'): attrs = []
print(f"# {n} test fns remaining (all -> misc)", file=sys.stderr)
PY
```

Expected: the **first** run (before any effects/install moves) prints `# 154` (effects 70 + install 23 + misc 61). Move effects and install into their files FIRST (Steps 3–4), then re-run this "all remaining" script — it must then print `# 61` (the misc group only). If the re-run count is not 61, STOP and reconcile (a fn was mis-attributed by the assignment rule).

- [ ] **Step 2: Create the three submodule files**

```bash
cd /workspace/crates/kali_cli
printf 'use super::*;\n\n' > tests/runtime_smoke/effects.rs
printf 'use super::*;\n\n' > tests/runtime_smoke/install.rs
printf 'use super::*;\n\n' > tests/runtime_smoke/misc.rs
```

- [ ] **Step 3: Move the effects `#[test]` fns verbatim into `effects.rs`** (70 fns, prefixes `effects`/`json_effects`).

- [ ] **Step 4: Move the install `#[test]` fns verbatim into `install.rs`** (23 fns, prefixes `install`/`json_install`).

- [ ] **Step 5: Move ALL remaining `#[test]` fns verbatim into `misc.rs`** — re-run the "all remaining" script from Step 1; it must list exactly 61 fns. Cut each one's attribute line(s) + body from the root and append to `misc.rs`. After this, `tests/runtime_smoke.rs` contains **zero** `#[test]` fns (verify: `grep -c 'fn .*(' tests/runtime_smoke.rs` shows only helpers; `grep -c '^#\[test\]' tests/runtime_smoke.rs` = 0).

- [ ] **Step 6: Wire the `#[path] mod` declarations in the root**

Append to `tests/runtime_smoke.rs`:

```rust
#[path = "runtime_smoke/effects.rs"]
mod effects;

#[path = "runtime_smoke/install.rs"]
mod install;

#[path = "runtime_smoke/misc.rs"]
mod misc;
```

- [ ] **Step 7: Verify**

Run the Verification commands block. Expected: build 0 warnings; count 1816; NAME SET UNCHANGED; suite fully green. Confirm the root is now test-free:

```bash
cd /workspace/crates/kali_cli
grep -cE '^\s*#\[(test|cfg\(unix\))\]' tests/runtime_smoke.rs   # 0 #[test]; may show #[cfg(unix)] on the 4 cfg-gated HELPERS that stay — that's fine
grep -nE '^\s*#\[test\]' tests/runtime_smoke.rs                  # expect no matches
```

(Note: the 4 cfg-gated helpers `shell_quote_path`, `browser_entrypoint_smoke`, `run_browser_entrypoint_smoke`, `test_browser_entrypoint_smoke` keep their `#[cfg(unix)]` attributes in the root — they are helpers, not tests.)

- [ ] **Step 8: Commit**

```bash
cd /workspace
git add crates/kali_cli/tests/runtime_smoke.rs crates/kali_cli/tests/runtime_smoke/effects.rs crates/kali_cli/tests/runtime_smoke/install.rs crates/kali_cli/tests/runtime_smoke/misc.rs
git commit -m "refactor(kali_cli): extract runtime_smoke effects/install/misc submodules; root test-free [refactor]"
```

Update the ledger.

---

### Task 4: Final whole-branch verification & integration

**Files:** none (verification + merge only)

**Interfaces:** —

- [ ] **Step 1: Whole-branch verification on the refactor branch**

```bash
cd /workspace
cargo build -p kali_cli --tests 2>&1 | tee /tmp/rs_final_build.txt        # 0 warnings
cargo test -p kali_cli --test runtime_smoke -- --list 2>/dev/null | grep -c ': test$'   # 1816
cargo test -p kali_cli --test runtime_smoke -- --list 2>/dev/null \
  | sed -E 's/^.*:://; s/: test$//' | sort > /tmp/rs_final_names.txt
diff /tmp/rs_baseline_names.txt /tmp/rs_final_names.txt && echo "NAME SET UNCHANGED"
cargo test -p kali_cli --test runtime_smoke 2>&1 | tee /tmp/rs_final_test.txt   # fully green
# full crate still matches baseline (the 2 build_bundles_* failures are in a different binary, unaffected)
cargo build -p kali_cli 2>&1 | tee /tmp/rs_final_crate_build.txt           # 0 warnings
```

Expected: 0 warnings everywhere; count 1816; NAME SET UNCHANGED; `runtime_smoke` fully green; full crate build 0 warnings.

- [ ] **Step 2: ff-merge into local main (NEVER push to origin)**

```bash
cd /workspace
git checkout main
git merge --ff-only refactor/kali-cli-runtime-smoke-modularization
git rev-parse HEAD   # record merged HEAD
```

- [ ] **Step 3: Re-verify on merged main**

```bash
cd /workspace
cargo build -p kali_cli --tests 2>&1 | tee /tmp/rs_main_build.txt          # 0 warnings
cargo test -p kali_cli --test runtime_smoke -- --list 2>/dev/null | grep -c ': test$'  # 1816
cargo test -p kali_cli --test runtime_smoke 2>&1 | tee /tmp/rs_main_test.txt         # fully green
```

- [ ] **Step 4: Delete the refactor branch**

```bash
cd /workspace
git branch -d refactor/kali-cli-runtime-smoke-modularization
```

- [ ] **Step 5: Final ledger update**

Record in `.superpowers/sdd/progress.md`: merged main HEAD, final `--list` count (1816), NAME SET UNCHANGED confirmation, suite green. Sub-project 2 complete.

---

## Out of scope

- The other `tests/` monoliths (`package_corpus.rs`, `node_api_surface.rs`, `schema_docs.rs`, `late_compat_browser_*`) — each its own future spec.
- Sub-project 3 (subdirectory grouping for the ~370 small per-behavior `tests/` files).
