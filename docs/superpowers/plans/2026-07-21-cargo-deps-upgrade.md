# Cargo Dependency Upgrade + CI Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade every external dependency in the Kali workspace to its latest stable version (measured 2026-07-21), fix all resulting build/clippy/fmt errors, and bump GitHub Actions to current majors — with zero behavior change (byte-for-byte fixtures stay byte-for-byte).

**Architecture:** Six risk-ordered stages, one commit each, on branch `deps-upgrade-2026-07`. After every stage the full workspace gate must show **zero failing tests** (the main baseline is 0-failed; Task 1 verifies that before anything changes). A nasty stage gets stopped and reported, not hacked green.

**Tech Stack:** Rust 1.97.1 (mise-pinned, current stable), cargo workspace (24 crates), GitHub Actions.

## Global Constraints

- Branch: `deps-upgrade-2026-07` (already exists, from main `df1c919a9`; spec committed as `27057b6f1`).
- Gate after EVERY stage: `cargo build --workspace` clean, `cargo clippy --workspace -- -D warnings` clean, `cargo fmt --all -- --check` clean, `bash scripts/test-gate.sh` prints `GATE OK: 0 failing tests`.
- No `#[allow]` to silence clippy unless the lint is a known false positive; never loosen `-D warnings`.
- No behavior changes: if a dep upgrade changes any fixture output or test expectation, STOP and report — do not update expectations to match.
- Fix API breakage at the call site following the new crate's idioms. If an API change is unclear, consult the crate's changelog/release notes (WebFetch of its GitHub releases page is fine).
- If a stage is disproportionately expensive (> ~2h of migration work), STOP and report; staged commits let us defer just that dep.
- Commit messages: `chore(deps): <stage summary> [depsUpgrade]` (Stage 6 uses `ci:` prefix).

---

### Task 1: Gate script + baseline verification

**Files:**
- Create: `scripts/test-gate.sh`

**Interfaces:**
- Produces: `scripts/test-gate.sh` — no args; runs the full workspace test enumeration; exits 0 and prints `GATE OK: 0 failing tests` iff nothing fails, else exits 1 and prints the failing-test list. Every later task runs this.

- [ ] **Step 1: Write the gate script**

```bash
#!/usr/bin/env bash
# Full-workspace test gate: enumerate every failing test (never stop at the
# first red binary) and fail unless the count is zero.
#
# Parses the bare `failures:` summary lists (reliable under parallel output
# interleaving, unlike per-test `... FAILED` lines).
set -uo pipefail

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

cargo test --workspace --no-fail-fast >"$log" 2>&1
status=$?

failures="$(awk '
    /^failures:$/ { collecting = 1; next }
    collecting && /^    [A-Za-z_]/ { print $1; next }
    collecting { collecting = 0 }
' "$log" | sort -u)"

if [ -n "$failures" ]; then
    echo "GATE FAILED — failing tests:"
    echo "$failures"
    exit 1
fi

if [ "$status" -ne 0 ]; then
    echo "GATE FAILED — cargo test exited $status with no parsed failures (build error?). Full log:"
    tail -n 40 "$log"
    exit 1
fi

echo "GATE OK: 0 failing tests"
```

Save as `scripts/test-gate.sh`, then: `chmod +x scripts/test-gate.sh`

- [ ] **Step 2: Verify the pre-upgrade baseline is green**

Run: `bash scripts/test-gate.sh`
Expected: `GATE OK: 0 failing tests` (matches the P4 close-out baseline). If it prints failures instead, STOP — the baseline is poisoned; report the list before touching any dependency.

- [ ] **Step 3: Commit**

```bash
git add scripts/test-gate.sh
git commit -m "chore(ci): add full-workspace test-gate script [depsUpgrade]"
```

---

### Task 2: Stage 1 — lockfile refresh + semver-compatible requirement bumps

**Files:**
- Modify: `Cargo.toml` (workspace `[workspace.dependencies]` block, lines ~68-90)
- Modify: `Cargo.lock` (via `cargo update`)

**Interfaces:**
- Consumes: `scripts/test-gate.sh` from Task 1.
- Produces: fully refreshed lockfile; later stages only change requirement strings.

- [ ] **Step 1: Raise the requirement strings that have drifted behind reality**

In `/workspace/Cargo.toml`, apply exactly these edits (current line → new line):

```toml
clap = { version = "4.4", features = ["derive", "cargo"] }   # → version = "4.6"
indexmap = "2.2"                                             # → "2.14"
once_cell = "1.19"                                           # → "1.21"
flate2 = "1.0"                                               # → "1.1"
```

Leave all other requirement strings alone in this stage (they are either already at the latest floor — ahash 0.8, log 0.4, serde 1.0, serde_json 1.0, url 2.5, form_urlencoded 1.2, base64 0.22, semver 1.0, tar 0.4, urlencoding 2.1, ryu-js 1.0 — or are major bumps handled in later stages).

- [ ] **Step 2: Refresh the lockfile**

Run: `cargo update`
Expected: many `Updating` lines (wasmtime 24.0.7→24.0.11, wast, zerocopy, etc. — all within existing requirements), exit 0.

- [ ] **Step 3: Run the full gate**

```bash
cargo build --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check && bash scripts/test-gate.sh
```
Expected: build + clippy + fmt clean, `GATE OK: 0 failing tests`. If clippy finds new lints from updated deps, fix them at the call site per Global Constraints.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(deps): stage 1 — cargo update + compatible requirement bumps (clap 4.6, indexmap 2.14, once_cell 1.21, flate2 1.1) [depsUpgrade]"
```

---

### Task 3: Stage 2 — easy majors (thiserror removal, reqwest 0.13, getrandom 0.4, tungstenite 0.30)

**Files:**
- Modify: `Cargo.toml` (workspace deps)
- Modify: `Cargo.lock`
- Possible call-site fixes: `crates/kali_api_node/src/http.rs`, `crates/kali_npm/src/lib.rs`, `crates/kali_runtime/src/lib.rs` (reqwest); `crates/kali_api_node/src/crypto.rs`, `crates/kali_api_web/src/crypto.rs` (getrandom); `crates/kali_cli/tests/cdp_driver/protocol.rs`, `crates/kali_cli/tests/cdp_driver/driver.rs` (tungstenite)

**Interfaces:**
- Consumes: `scripts/test-gate.sh`.
- Produces: nothing new for later tasks; each stage is independent.

- [ ] **Step 1: Remove the dead thiserror entry**

`thiserror = "1.0"` in `[workspace.dependencies]` is referenced by **zero** crates (verified: no `crates/*/Cargo.toml` lists it, no `.rs` file mentions `thiserror`). Delete the line entirely instead of upgrading it.

- [ ] **Step 2: Bump the three real majors**

In `/workspace/Cargo.toml`:

```toml
reqwest = { version = "0.12", ... }   # → version = "0.13" (keep default-features = false, features = ["blocking", "rustls-tls"]; if 0.13 renamed a feature, cargo will error naming it — adjust to the new name from the reqwest 0.13 changelog)
getrandom = "0.3"                     # → "0.4"
tungstenite = { version = "0.24", default-features = false }   # → version = "0.30"
```

Run: `cargo update -p reqwest -p getrandom -p tungstenite` (or plain `cargo update`), then `cargo build --workspace 2>&1 | head -60`.

- [ ] **Step 3: Fix compile errors at the call sites**

Expected churn, by dep:
- **getrandom 0.4** — call sites use only `getrandom::fill` and `getrandom::Error` (`kali_api_node/src/crypto.rs:3,116-133`, `kali_api_web/src/crypto.rs:9-69`). These names survived 0.3→0.4; expect zero or trivial fixes.
- **reqwest 0.13** — call sites use the `blocking` client only. Expect at most builder-method renames.
- **tungstenite 0.30** — CDP test driver only. Known churn in 0.25-0.30: `Message::Text` now wraps `Utf8Bytes` (use `Message::text(s)` constructor and `.to_string()`/`as_str()` on read), and `Error`/`Bytes` types moved to `tungstenite::Bytes`. Fix per compiler guidance; behavior (CDP smoke driver) must be unchanged.

- [ ] **Step 4: Run the full gate**

```bash
cargo build --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check && bash scripts/test-gate.sh
```
Expected: all clean, `GATE OK: 0 failing tests`.

Also run the tungstenite-affected lane explicitly (it is `--ignored` so the gate skips it):
`cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
Expected: pass (requires chromium or node per harness; if the environment lacks both, note it in the commit body and rely on CI's browser-cdp-smoke job).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/
git commit -m "chore(deps): stage 2 — reqwest 0.13, getrandom 0.4, tungstenite 0.30; drop unused thiserror [depsUpgrade]"
```

---

### Task 4: Stage 3 — RustCrypto set (sha1/sha2 0.11, hmac 0.13)

**Files:**
- Modify: `Cargo.toml` (workspace deps), `Cargo.lock`
- Possible call-site fixes: `crates/kali_api_web/src/crypto.rs` (Sha1/Sha256 digest), `crates/kali_api_node/src/crypto.rs` (Hmac<Sha256/384/512> via `new_from_slice`), `crates/kali_npm/src/tarball.rs` + `crates/kali_npm/src/lib.rs` (sha256_hex), `crates/kali_cli/src/build/compile.rs`, `crates/kali_cli/src/build/paths.rs`, `crates/kali_runtime/src/host/imports_default.rs`, and the `Sha256::digest`-using test files under `crates/kali_cli/tests/` (clbg_*_runtime.rs, runtime_smoke, package_corpus, schema_docs)

**Interfaces:**
- Consumes: `scripts/test-gate.sh`.

- [ ] **Step 1: Bump all three together**

These share the `digest` ecosystem — mixing 0.10-line and 0.11-line halves does not compile, so one atomic edit in `/workspace/Cargo.toml`:

```toml
sha1 = "0.10"    # → "0.11"
sha2 = "0.10"    # → "0.11"
hmac = "0.12"    # → "0.13"
```

Run: `cargo update`, then `cargo build --workspace 2>&1 | head -80`.

- [ ] **Step 2: Fix compile errors at the call sites**

The used surface is small: `Sha1::digest(..)`, `Sha256::digest(..)`, `Hmac::<ShaN>::new_from_slice(..)` + `mac.update(..)` + `finalize().into_bytes()`. In digest 0.11 the trait names/paths moved (`digest::Digest`, `digest::KeyInit`/`Mac` re-exports); expect import-path and trait-bound fixes rather than algorithmic changes. Crypto OUTPUTS must be identical — the CLBG fixture tests hash outputs with Sha256 and will catch any change; `kali_api_node` hmac tests pin hex vectors.

- [ ] **Step 3: Run the full gate**

```bash
cargo build --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check && bash scripts/test-gate.sh
```
Expected: all clean, `GATE OK: 0 failing tests`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock crates/
git commit -m "chore(deps): stage 3 — RustCrypto digest-0.11 line (sha1/sha2 0.11, hmac 0.13) [depsUpgrade]"
```

---

### Task 5: Stage 4 — wasm-tools set (wasm-encoder / wasmparser / wit-component 0.254)

**Files:**
- Modify: `Cargo.toml` (workspace deps), `Cargo.lock`
- Possible call-site fixes: `crates/kali_codegen/` (main emitter — imports `BlockType, CodeSection, ConstExpr, CustomSection, DataSection, EntityType, ExportKind, ExportSection, Function, FunctionSection, GlobalSection, GlobalType, ImportSection, Instruction, MemArg, MemorySection, MemoryType, Module, TypeSection, ValType` from wasm_encoder; `Validator`, `ExternalKind::Func`, `TypeRef::Func` from wasmparser), `crates/kali_cli/` (`Component, ComponentSectionId, CustomSection, RawSection, Section` + wit-component)

**Interfaces:**
- Consumes: `scripts/test-gate.sh`.

- [ ] **Step 1: Bump all three together**

They are released in lockstep from the wasm-tools repo; one atomic edit in `/workspace/Cargo.toml`:

```toml
wasm-encoder = "0.240"   # → "0.254"
wit-component = "0.240"  # → "0.254"
wasmparser = "0.240"     # → "0.254"
```

Run: `cargo update`, then `cargo build --workspace 2>&1 | head -80`.

CAUTION: wasmtime 24 internally depends on its own pinned wasmparser — that is a separate lockfile entry and fine; do NOT try to unify them.

- [ ] **Step 2: Fix compile errors at the call sites**

The section/instruction encoder API is historically stable across 0.24x→0.25x; expect at most enum-variant or constructor-signature drift on `Instruction`/`MemArg`/`ValType` and `Validator` config. Consult https://github.com/bytecodealliance/wasm-tools/releases if an error is not self-explanatory. The emitted bytes feed byte-for-byte fixture tests — the gate proves output identity.

- [ ] **Step 3: Run the full gate**

```bash
cargo build --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check && bash scripts/test-gate.sh
```
Expected: all clean, `GATE OK: 0 failing tests`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock crates/
git commit -m "chore(deps): stage 4 — wasm-tools 0.254 (wasm-encoder/wasmparser/wit-component) [depsUpgrade]"
```

---

### Task 6: Stage 5 — wasmtime 24 → 47

**Files:**
- Modify: `Cargo.toml` (workspace deps), `Cargo.lock`
- Possible call-site fixes: `crates/kali_runtime/src/` only (`lib.rs`, `execute.rs`, `host/imports_node.rs`, `host/imports_default.rs`, `execute_tests/`) — the only crate with a wasmtime dependency

**Interfaces:**
- Consumes: `scripts/test-gate.sh`.

- [ ] **Step 1: Bump the requirement**

In `/workspace/Cargo.toml`:

```toml
wasmtime = "24"   # → "47"
```

Run: `cargo update`, then `cargo build --workspace 2>&1 | head -100`.

- [ ] **Step 2: Fix compile errors in kali_runtime**

Measured usage surface: `wasmtime::Result` (96×), `wasmtime::Error::msg` (77×), Engine/Store/Linker/Module basics, `Store::set_fuel`/fuel APIs, `StoreLimits`/`StoreLimitsBuilder`, `Trap::{UnreachableCodeReached, OutOfFuel, MemoryOutOfBounds}`, `ValType`/`Val`, one `Global`. Across 24→47 expect: possible `wasmtime::Error` type change (anyhow re-export was replaced by a dedicated error type in later majors — `Error::msg` call sites may need adjusting), possible `Config`/feature-flag reshuffles (wasmtime split cargo features; if the build fails on missing functionality, add the needed feature to the workspace dep, e.g. `features = ["cranelift"]`), and trap/renames. Consult https://github.com/bytecodealliance/wasmtime/releases for the exact major that broke a given API.

Behavioral invariants that MUST hold (the tests pin them): fuel exhaustion still surfaces as `Trap::OutOfFuel` → E4000-class diagnostics; memory limits via StoreLimits unchanged; deterministic execution (no new nondeterminism from JIT changes).

- [ ] **Step 3: Run the full gate**

```bash
cargo build --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check && bash scripts/test-gate.sh
```
Expected: all clean, `GATE OK: 0 failing tests`.

- [ ] **Step 4: Run the determinism smoke lane (extra gate for this stage)**

Run: `bash scripts/check-determinism.sh`
Expected: exit 0 (same pass criteria as the CI determinism job). wasmtime executes the fixtures, so this stage gets the extra lane.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/kali_runtime/
git commit -m "chore(deps): stage 5 — wasmtime 24 -> 47 [depsUpgrade]"
```

---

### Task 7: Stage 6 — GitHub Actions refresh

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: nothing; independent of the cargo stages.

- [ ] **Step 1: Bump action versions in both workflows**

Latest majors (fetched live from the GitHub releases API on 2026-07-21) — apply everywhere each action appears:

| Action | From | To |
|---|---|---|
| actions/checkout | v4 | v7 |
| actions/setup-node | v4 | v7 |
| actions/cache | v4 | v6 |
| actions/upload-artifact | v4 | v7 |
| actions/download-artifact | v4 | v8 |
| softprops/action-gh-release | v2 | v3 |
| dorny/paths-filter | v3 | v4 |
| slsa-framework/slsa-github-generator | v2.1.0 | v2.1.0 (already latest — no change) |
| leanprover/lean-action | v1 | v1 (major tag already latest — no change) |
| dtolnay/rust-toolchain | @stable | @stable (no change, per spec) |

- [ ] **Step 2: Clean up node24 shim and stale cache keys**

In `ci.yml`:
- Delete the `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true` env line — the bumped action majors run on node24+ natively, the shim is obsolete.
- Bump every cache key `-cargo-v2-` → `-cargo-v3-` (both `key:` and `restore-keys:`, all 5 occurrences) so post-upgrade CI starts from a clean `target/` instead of restoring artifacts built against the old dependency graph.

- [ ] **Step 3: Validate the workflow files**

Run: `python3 -c "import yaml,sys; [yaml.safe_load(open(f)) for f in ['.github/workflows/ci.yml','.github/workflows/release.yml']]; print('YAML OK')"`
Expected: `YAML OK`. If `actionlint` is available (`command -v actionlint`), run it too; otherwise CI itself is the validator.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/
git commit -m "ci: stage 6 — bump actions to latest majors (checkout v7, setup-node v7, cache v6, artifacts v7/v8, gh-release v3, paths-filter v4); drop node24 shim; roll cache key [depsUpgrade]"
```

---

### Task 8: PR, CI verification, merge

**Files:** none (integration only)

**Interfaces:**
- Consumes: all prior commits on `deps-upgrade-2026-07`.

- [ ] **Step 1: Final local verification sweep**

```bash
cargo build --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check && bash scripts/test-gate.sh && bash scripts/check-determinism.sh
```
Expected: everything clean, `GATE OK: 0 failing tests`, determinism exit 0.

- [ ] **Step 2: Push and open the PR**

```bash
git push -u origin deps-upgrade-2026-07
gh pr create --title "chore(deps): upgrade all cargo dependencies to latest + CI actions refresh" --body "$(cat <<'EOF'
Six risk-ordered stages, each gated at 0 failing workspace tests:

1. cargo update + compatible bumps (clap 4.6, indexmap 2.14, once_cell 1.21, flate2 1.1)
2. reqwest 0.13, getrandom 0.4, tungstenite 0.30; drop unused thiserror
3. RustCrypto digest-0.11 line (sha1/sha2 0.11, hmac 0.13)
4. wasm-tools 0.254 (wasm-encoder/wasmparser/wit-component)
5. wasmtime 24 -> 47 (+ determinism smoke lane)
6. GitHub Actions to latest majors; node24 shim removed; cache key rolled

No behavior changes: byte-for-byte fixtures unchanged; per-stage gate log in commit messages.
Spec: docs/superpowers/specs/2026-07-21-cargo-deps-upgrade-design.md
EOF
)"
```
(If `gh` cannot authenticate git, run `gh auth setup-git` first — standing convention.)

- [ ] **Step 3: Wait for CI, then review**

Watch `gh pr checks --watch`. All jobs (build ubuntu+macos, clippy, fmt, phase1-evidence, determinism ubuntu+macos, browser-cdp-smoke, proof-check) must be green. If a CI-only failure appears (e.g., a bumped action's breaking input change), fix it in a follow-up commit on the branch and re-verify.

Then request code review per superpowers:requesting-code-review before merging.

- [ ] **Step 4: Merge and close out**

```bash
gh pr merge --merge --delete-branch
```
Per standing convention (push + merge myself once reviewed and green). Then update memory with any lessons (e.g., which wasmtime majors actually broke which APIs).
