# kali_api_node Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose `kali_api_node/src/lib.rs` (1,511 lines) into 14 flat per-Node-API-family modules behind a glob-re-export facade, with co-located sibling tests, changing zero behavior.

**Architecture:** `kali_api_node` is an *independent-object pile* — ~40 self-contained Node API objects, each a `struct` + its own `impl` block(s) + (often) an error enum, plus free fns and family-local private helpers. There is **no shared mega-struct**, so **no Task-1 `pub(crate)` receiver-widening is required** (cross-family references are by already-`pub` public types; all 6 private top-level items are used only within their own family). Every item moves **byte-for-byte verbatim** into its family module; `lib.rs` becomes a thin facade of `mod` decls + alphabetical `pub use <mod>::*;` globs that preserve every `kali_api_node::Name` path.

**Tech Stack:** Rust (workspace crate), `cargo test` / `cargo clippy`, subagent-driven-development.

**Design:** `docs/superpowers/specs/2026-06-26-kali-api-node-modularization-design.md`

## Global Constraints

- **Zero behavior change.** Items relocate verbatim — including whitespace, blank-line separators between items, and relative source order. Never rewrite, reorder within a kept block, or "tidy" code.
- **This is a relocation refactor, not TDD.** No new product code and no new tests are written. The 16 existing tests are the safety net; they must stay green and keep their exact names. "Show the code" for a step here means *name the items and give their current line range*, not paste 1,511 lines — the operation is a mechanical cut-paste.
- **Test set is invariant at 16.** Proven by **basename-multiset**: strip module-path prefixes from `cargo test -p kali_api_node -- --list`, `sort` *without* `-u`, diff against the baseline → must be empty. (`--list` prefixes change as tests relocate; raw diff is non-empty by design.)
- **Green + clippy-clean at every commit:** `cargo test -p kali_api_node` passes and `cargo clippy -p kali_api_node --all-targets -- -D warnings` is clean after every task.
- **`kali_test_support` IS adopted (fixture CONVERT, hold count at 16).** `tests.rs` has 3 real `tempdir()` sites (all in the `fs` family). They convert to `kali_test_support::fixtures::tempdir()` in Task 15 (fs). No *added* fixture test — the suite stays strictly at 16. After conversion the `tempfile` dev-dep is unused crate-wide and is removed in the same task (commit the `Cargo.lock` delta with the `Cargo.toml` change).
- **Facade-only `lib.rs`.** When complete, `lib.rs` contains only: crate doc comment, alphabetical `mod` decls, alphabetical `pub use <mod>::*;` globs. No type/fn/static/const/use-of-private definitions remain. **Never** add a `#[cfg(test)] pub(crate) use …` re-export to feed a test glob — make the test file self-sufficient instead (the kali_api_web Task-15 facade-purity lesson).
- **Glob rule (clippy `-D warnings`):** every family module here exports public types, so each gets `pub use <mod>::*;`. If clippy ever flags a glob as re-exporting nothing, the module owns only `pub(crate)` items → switch that one to `pub(crate) use <mod>::*;`; if it genuinely re-exports nothing used, delete the glob (never `#[allow]` it). Not expected for any module in this plan.
- **Widening predicted NONE.** The 6 private top-level items (`resolve_node_path`, `path_root_key` → path; `NodeDigestAlgorithm` → crypto; `hex_digit` → buffer; `Listener`, `ListenerMap` → events) are each used only within their own family and stay **private** in their module. Escape hatch: if an extraction reveals a cross-family caller of a moved private item, widen *that one item* private→`pub(crate)` minimally (never `pub`) and record the coupling in the commit message.
- **Self-sufficient test files.** Every `*_tests.rs` header is `use crate::*;` plus the explicit `use` for any bare external name its tests reference (`serde_json::Value`, `std::sync::Arc`, `std::fs`, `kali_test_support::fixtures`, etc.). Do not rely on lib.rs's private imports leaking through the facade glob.

---

## Standard Extraction Procedure (every family task follows these 7 steps)

Each family task substitutes `<family>`, its **item list + current line ranges**, its **"neighbors that STAY" list**, and its **test names**. Line ranges in this plan are from the pre-refactor file and **shift after every prior extraction** — always re-grep the current item boundaries before cutting (`grep -nE '^(pub )?(struct|enum|fn|type|const|static|impl) ' crates/kali_api_node/src/lib.rs`). Move each struct/enum **together with all of its `impl` blocks** (including trait impls like `Display`/`Default`/`Error` that follow it).

- [ ] **Step A — Create `crates/kali_api_node/src/<family>.rs`.** Start with a module doc line and the precise `use` imports the moved code needs — a subset of `lib.rs`'s header (`base64::…`, `getrandom::…`, `hmac::…`, `serde_json::Value`, `sha2::…`, `std::{…}`, `url::Url`) plus `use crate::{SiblingType, …};` for any types owned by other family modules. Let the compiler tell you exactly which (`cargo build -p kali_api_node`, add what's missing, remove what's unused).
- [ ] **Step B — Move the listed items verbatim** from `lib.rs` into `<family>.rs` (cut, not copy), preserving order and blank-line separators. Leave every "neighbor that STAYS" untouched in place.
- [ ] **Step C — Wire the facade.** Add `mod <family>;` and `pub use <family>::*;` to `lib.rs`, both in alphabetical position. Remove any `use` import in `lib.rs`'s header that the moved items took with them and that nothing remaining uses (compiler-driven — fix the resulting `unused_imports` warning in this same commit).
- [ ] **Step D — Relocate this family's tests** (skip if the family has none). Cut the listed test fns verbatim from `src/tests.rs` into a new `crates/kali_api_node/src/<family>_tests.rs`; change its header from `use super::*;` to `use crate::*;` and add any specific extra import the moved tests reference (compiler-driven). Add to the bottom of `<family>.rs`:
  ```rust
  #[cfg(test)]
  #[path = "<family>_tests.rs"]
  mod <family>_tests;
  ```
- [ ] **Step E — Verify green + clippy-clean.**
  Run: `cargo test -p kali_api_node` → Expected: PASS (still 16 tests).
  Run: `cargo clippy -p kali_api_node --all-targets -- -D warnings` → Expected: no warnings.
- [ ] **Step F — Verify the basename-multiset proof holds.**
  Run:
  ```bash
  cargo test -p kali_api_node -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
    | diff - <(sort .superpowers/sdd/kali_api_node_list_before.txt)
  ```
  Expected: empty output (identical 16-name multiset).
- [ ] **Step G — Commit.**
  ```bash
  git add crates/kali_api_node/src/
  git commit -m "refactor(kali_api_node): extract <family> module [refactor]"
  ```

---

## Task 1: Baseline snapshot (controller-run, no commit)

**Files:** none modified. Produces `.superpowers/sdd/kali_api_node_list_before.txt`.

- [ ] **Step 1:** Capture the test baseline:
  ```bash
  mkdir -p .superpowers/sdd
  cargo test -p kali_api_node -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort > .superpowers/sdd/kali_api_node_list_before.txt
  wc -l .superpowers/sdd/kali_api_node_list_before.txt   # Expected: 16
  ```
- [ ] **Step 2:** Confirm green at HEAD: `cargo test -p kali_api_node` → 16 passed.
- [ ] **Step 3:** Record the fixture sites for Task 15: `grep -nE 'tempdir|fs::write|fs::read|fs::create|NamedTempFile|tempfile' crates/kali_api_node/src/tests.rs` → expect `use tempfile::tempdir;` (line 7) + 3 `tempdir()` calls (≈377, ≈419, ≈518) + `std::fs::create_dir` / `std::fs::read_to_string` (stay as-is, not `write`). All 3 `tempdir()` sites belong to `fs`-family tests.

This task is run directly by the controller (trivial, no commit).

---

## Task 2: Extract `util`

**Files:** Create `crates/kali_api_node/src/util.rs`; Modify `crates/kali_api_node/src/lib.rs`. (No test file — see below.)

**Items to move** (re-grep current ranges):
- `node_api_init` (≈22, the empty crate-entry free fn) — folded here, mirroring kali_api_web's `web_api_init`→`util`.
- `util_format` (≈1379), `util_inspect` (≈1388), `NodeUtil` + `impl NodeUtil` (≈1394–1415), `util_promisify` (≈1416–1435).

**Neighbors that STAY:** everything else.

**Tests to move:** NONE. `NodeUtil`/buffer behavior is covered by `buffer_and_util_helpers_round_trip`, which is assigned to the `buffer` family (Task 5). Skip Step D.

Follow the Standard Extraction Procedure (Steps A–C, E–G) with `<family>` = `util`.

---

## Task 3: Extract `url`

**Files:** Create `crates/kali_api_node/src/url.rs`; Modify `crates/kali_api_node/src/lib.rs`. (No test file.)

**Items to move** (re-grep current ranges):
- `parse_url` (≈1355), `resolve_url` (≈1360), `NodeUrl` + `impl NodeUrl` (≈1366–1378).

**Imports:** `url.rs` needs `use url::Url;` (and `url::ParseError` is referenced fully-qualified in the moved signatures). Inside `url.rs`, bare `use url::…` resolves to the **external** `url` crate (a child module is not self-shadowed by `mod url;`). The facade glob `pub use url::*;` resolves to the **local** module. Verify `lib.rs`'s `use url::Url;` header line is removed once these move (nothing else in lib.rs uses it after this task — confirm via the `unused_imports` warning).

**Neighbors that STAY:** everything else.

**Tests to move:** NONE. `NodeUrl` behavior is covered by `os_and_url_helpers_expose_expected_views`, assigned to the `os` family (Task 10). Skip Step D.

Follow the Standard Extraction Procedure (Steps A–C, E–G) with `<family>` = `url`.

---

## Task 4: Extract `assert`

**Files:** Create `crates/kali_api_node/src/assert.rs`, `crates/kali_api_node/src/assert_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges):
- `NodeAssert` + `impl NodeAssert` (≈1436–1504), `assert_true` (≈1505–1510).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `assert_helpers_produce_clear_results`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `assert`.

---

## Task 5: Extract `buffer`

**Files:** Create `crates/kali_api_node/src/buffer.rs`, `crates/kali_api_node/src/buffer_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges):
- `NodeBuffer` (tuple struct, ≈830) + `impl NodeBuffer` (≈832–897), `hex_digit` (private fn, ≈898–908 — stays **private** in `buffer.rs`).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `buffer_and_util_helpers_round_trip` — this test exercises both `NodeBuffer` **and** `NodeUtil`; `NodeUtil` was extracted in Task 2 and is reachable via the facade glob through the test's `use crate::*;`. Add any explicit import the test body needs (compiler-driven).

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `buffer`.

---

## Task 6: Extract `child_process`

**Files:** Create `crates/kali_api_node/src/child_process.rs`, `crates/kali_api_node/src/child_process_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges — the struct `NodeChildProcess` and its `impl` are **non-contiguous**, separated by the Output/Error types):
- `NodeChildProcess` (unit struct, ≈1235), `NodeChildProcessOutput` + `impl` (≈1239–1259), `NodeChildProcessError` + `impl` + `Display` + `Error` (≈1260–1279), `impl NodeChildProcess` (≈1280–1317). Move the whole ≈1235–1317 block verbatim (preserves order; no reorder needed).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `child_process_helpers_capture_command_output`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `child_process`.

---

## Task 7: Extract `crypto`

**Files:** Create `crates/kali_api_node/src/crypto.rs`, `crates/kali_api_node/src/crypto_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges — the ≈417–559 block, contiguous):
- `sha256_hex` (≈417), `NodeCryptoError` + `impl` + `Display` + `Error` (≈423–449), `NodeDigestAlgorithm` (private enum) + `impl` (≈450–506 — stays **private** in `crypto.rs`), `NodeCrypto` + `impl` (≈507–534), `random_bytes` (≈535), `random_uuid_v4` (≈542–559).

**Imports:** `crypto.rs` needs `base64::{engine::general_purpose::STANDARD, Engine as _}`, `getrandom::fill as fill_random_bytes`, `hmac::{Hmac, Mac}`, `sha2::{Digest, Sha256, Sha384, Sha512}` (compiler-driven; remove these from `lib.rs`'s header if nothing remaining uses them).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `crypto_helpers_produce_expected_formats`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `crypto`.

---

## Task 8: Extract `events`

**Files:** Create `crates/kali_api_node/src/events.rs`, `crates/kali_api_node/src/events_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges — the ≈745–829 block):
- `NodeEvent` + `impl` (≈745–773), `type Listener` (≈774), `type ListenerMap` (≈775 — both type aliases stay **private** in `events.rs`), `EventEmitter` + `impl` (≈779–829).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `event_emitter_invokes_listeners_in_order`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `events`.

---

## Task 9: Extract `http`

**Files:** Create `crates/kali_api_node/src/http.rs`, `crates/kali_api_node/src/http_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges — `NodeHttp` struct and `impl NodeHttp` are **non-contiguous**, with `NodeHttpResponse` between them; move the whole ≈1161–1234 block verbatim):
- `NodeHttpError` + `impl` + `Display` + `Error` (≈1161–1182), `NodeHttp` (unit struct, ≈1183), `NodeHttpResponse` + `impl` (≈1187–1205), `impl NodeHttp` (≈1206–1234).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `http_helpers_fetch_local_response_body`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `http`.

---

## Task 10: Extract `os`

**Files:** Create `crates/kali_api_node/src/os.rs`, `crates/kali_api_node/src/os_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges):
- `NodeOs` + `impl NodeOs` (≈1318–1354).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `os_and_url_helpers_expose_expected_views` — exercises both `NodeOs` **and** `NodeUrl`; `NodeUrl` was extracted in Task 3 and is reachable via the facade glob through `use crate::*;`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `os`.

---

## Task 11: Extract `path`

**Files:** Create `crates/kali_api_node/src/path.rs`, `crates/kali_api_node/src/path_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges — the ≈241–416 block, contiguous):
- `normalize_path` (≈241), `join_path` (≈268), `resolve_path` (≈275), `relative_path` (≈288), `resolve_node_path` (private, ≈330), `path_root_key` (private, ≈340), `dirname` (≈350), `basename` (≈358), `extname` (≈367), `NodePath` + `impl NodePath` (≈384–416). The two private helpers stay **private** in `path.rs`.

**Imports:** `path.rs` needs `std::path::{Path, PathBuf}` (and whatever else the bodies use — compiler-driven).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `path_helpers_are_lexical_and_deterministic`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `path`.

---

## Task 12: Extract `process`

**Files:** Create `crates/kali_api_node/src/process.rs`, `crates/kali_api_node/src/process_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges):
- `NodeProcess` (≈26) + `impl Default for NodeProcess` (≈37) + `impl NodeProcess` (≈52–240). Move the whole ≈26–240 block verbatim.

**Imports:** `process.rs` needs `serde_json::Value` and the relevant `std::{…}` items (compiler-driven).

**Neighbors that STAY:** everything else.

**Tests to move (2):** `default_process_context_uses_node_as_argv0`, `process_context_tracks_env_and_output`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `process`.

---

## Task 13: Extract `runtime`

**Files:** Create `crates/kali_api_node/src/runtime.rs`, `crates/kali_api_node/src/runtime_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges):
- `NodeRuntimeProjection` + `impl NodeRuntimeProjection` (≈560–744).

**Imports:** `runtime.rs` references sibling types it bundles (e.g. `NodeProcess`, `NodeFs`, …) — these are owned by other family modules by now, so add `use crate::{…};` as the compiler directs.

**Neighbors that STAY:** everything else.

**Tests to move (3):** `runtime_projection_bundles_common_node_surfaces`, `runtime_projection_exposes_deterministic_env_snapshot`, `runtime_projection_preserves_host_argv0_projection`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `runtime`.

---

## Task 14: Extract `stream`

**Files:** Create `crates/kali_api_node/src/stream.rs`, `crates/kali_api_node/src/stream_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (re-grep current ranges):
- `NodeStream` + `impl NodeStream` (≈1132–1160).

**Neighbors that STAY:** everything else.

**Tests to move (1):** `stream_helpers_concatenate_bytes`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `stream`.

---

## Task 15: Extract `fs` (fixture conversion; empties and deletes `tests.rs`)

**Files:** Create `crates/kali_api_node/src/fs.rs`, `crates/kali_api_node/src/fs_tests.rs`; Modify `lib.rs`, `Cargo.toml`, root `Cargo.lock`; Delete `crates/kali_api_node/src/tests.rs`.

**Items to move** (re-grep current ranges — the ≈909–1131 block, contiguous):
- `NodeFs` + `impl NodeFs` (≈909–1011), `NodeFsPromises` + `impl` (≈1012–1085), `NodeFsMetadata` + `impl` (≈1086–1131).

**Neighbors that STAY:** after this task, only `lib.rs`'s facade decls should remain (this is the last family).

**Tests to move (2):** `fs_helpers_round_trip_files_and_directories`, `fs_promises_helpers_match_sync_helpers`. These are the **last two tests in `tests.rs`** and contain all 3 `tempdir()` sites.

**Extra steps for this task (in addition to Standard Extraction Procedure A–G):**

- [ ] **Step A2 — Add the `kali_test_support` dev-dep.** In `crates/kali_api_node/Cargo.toml` under `[dev-dependencies]`, add:
  ```toml
  kali_test_support = { workspace = true }
  ```
  (Match the form used by `crates/kali_npm/Cargo.toml` / `crates/kali_optimize/Cargo.toml`.)
- [ ] **Step D2 — Convert the fixture sites in `fs_tests.rs`.** In the moved test bodies, replace each `tempdir().expect("tempdir")` with `fixtures::tempdir()` and add `use kali_test_support::fixtures;` to the `fs_tests.rs` header. Leave `std::fs::create_dir` and `std::fs::read_to_string` as-is (not UTF-8 `write`, so no `write_file` conversion applies). Keep `std::fs` / `std::env` imports the bodies still need (compiler-driven, self-sufficient header).
- [ ] **Step D3 — Remove the now-unused `tempfile` dev-dep.** The only `tempfile` usage was the converted `tempdir()` calls. Delete the `tempfile = …` line from `crates/kali_api_node/Cargo.toml` `[dev-dependencies]` and confirm no `use tempfile` remains anywhere in the crate (`grep -rn tempfile crates/kali_api_node` → empty). Regenerate the lockfile (`cargo build -p kali_api_node`) so root `Cargo.lock` reflects the removal.
- [ ] **Step D4 — Delete the emptied `tests.rs`.** After moving the last two tests, `crates/kali_api_node/src/tests.rs` is empty (only its header remains). Delete it and remove the `mod tests;` line from `lib.rs`. (Git renders the final test relocation + deletion; ensure no `#[path = "tests.rs"]` reference remains.)
- [ ] **Step E2 — Verify.** `cargo test -p kali_api_node` → 16 passed; `cargo clippy -p kali_api_node --all-targets -- -D warnings` → clean; `grep -rn 'tempfile' crates/kali_api_node` → empty.
- [ ] **Step G2 — Commit** (include `Cargo.toml` and `Cargo.lock`):
  ```bash
  git add crates/kali_api_node/ Cargo.lock
  git commit -m "refactor(kali_api_node): extract fs module, adopt fixtures, drop tempfile, delete tests.rs [refactor]"
  ```

---

## Task 16: Final verification & fmt (controller-run)

**Files:** Modify `crates/kali_api_node/src/*` only if `cargo fmt` produces a diff.

- [ ] **Step 1 — Confirm `lib.rs` is a pure facade.** It should contain only: crate doc comment, 14 alphabetical `mod` decls, 14 alphabetical `pub use <mod>::*;` globs. No type/fn/static/const definitions, no `#[cfg(test)]` re-exports. `wc -l crates/kali_api_node/src/lib.rs` → expect ≈30. `grep -nE '^(pub )?(struct|enum|fn|const|static|type|impl) ' crates/kali_api_node/src/lib.rs` → empty.
- [ ] **Step 2 — Public API inventory unchanged.** `grep -cE '^pub ' crates/kali_api_node/src/lib.rs` is now 0 (all moved); instead verify the 14 modules collectively re-export the original 39 public items at flat paths — spot-check a downstream consumer still builds: `cargo build` (whole workspace) → success.
- [ ] **Step 3 — Run `cargo fmt`.** `cargo fmt -p kali_api_node`; if it produces a diff (e.g. a signature pushed past 100 cols by no change here, or import alphabetization), inspect it is behavior-neutral and commit as `style(kali_api_node): cargo fmt [refactor]`. If clean, no commit.
- [ ] **Step 4 — Final full verification.**
  ```bash
  cargo test -p kali_api_node                                   # 16 passed
  cargo clippy -p kali_api_node --all-targets -- -D warnings    # clean
  cargo build                                                   # whole workspace builds
  cargo test -p kali_api_node -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
    | diff - <(sort .superpowers/sdd/kali_api_node_list_before.txt)   # empty
  ```
- [ ] **Step 5 — Integration (local main only, per spec).** Fast-forward merge `refactor/kali-api-node-modularization` into local `main`, re-verify on merged main (`cargo test -p kali_api_node` + clippy + `cargo build`), delete the feature branch. Do **not** push to origin this cycle.

---

## Self-Review

- **Spec coverage:** shape (independent-object-pile, no Task-1 widening) ✓ Task list intro + Global Constraints; 14-family decomposition ✓ Tasks 2–15; pure-facade lib.rs ✓ Tasks 2–15 Step C + Task 16 Step 1; no `url`-shadow hazard ✓ Task 3; widening-NONE + escape hatch ✓ Global Constraints; fixture CONVERT + tempfile removal ✓ Task 15 Steps A2/D2/D3; test split into ≤14 self-sufficient sibling files with combined-test assignment ✓ Tasks 4–15 (util/url carry none; buffer/os carry the combined tests); basename-multiset proof ✓ Step F + Task 16 Step 4; invariants (verbatim, green+clippy every commit, API preserved) ✓ Global Constraints + Task 16; integration local-main-only ✓ Task 16 Step 5.
- **Placeholder scan:** no TBD/TODO; every family task names exact items + line ranges + test names; line numbers flagged as "re-grep, shifts after each extraction" (intentional, not a placeholder).
- **Type consistency:** family/module names and `<family>_tests.rs` filenames are consistent across the Standard Procedure and every task; the 12 test-bearing families (assert, buffer, child_process, crypto, events, fs, http, os, path, process, runtime, stream) sum to all 16 tests; url + util carry none (covered by combined tests). Last test-bearing family extracted = fs (Task 15) → correctly the one that empties and deletes `tests.rs`.
