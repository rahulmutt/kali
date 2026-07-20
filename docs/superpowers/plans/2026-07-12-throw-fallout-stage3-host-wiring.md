# throw-fallout Stage 3 — Host wiring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drain the host-wiring buckets of the throw-fallout residual (`performance.now`, web crypto, `coverage_hit` browser LinkError, `process.kill(0)` probe = 45 tests) plus the F-Stage1-2/3 env-equality soundness holes and the Stage-0 CDP crash-lane residual — all with real node-parity implementations, zero flips.

**Architecture:** Each host intrinsic gets a codegen recognizer that emits a conditional `kali:rt` import + call (host Rust impls already exist for perf.now / getRandomValues / randomUUID; `subtle.digest` needs a new one), mirrored by a kali_types admission arm, and a JS entry in all four browser `importObject` lists guarded by a divergence test. A narrow "synchronously-settled `await`" lowering makes the async-wrapped fixtures run without the full Stage-7 microtask queue.

**Tech Stack:** Rust workspace; `wasm-encoder`/`wasmprinter`/`wasmparser` (validation); wasmtime host lane; node `.mjs` + Chromium/CDP browser harness lanes.

## Global Constraints

- **Branch:** `soundness-batch1-pra`. Stage base commit: `5815fef08`. Denominator entering this stage: **923**.
- **The one hard gate:** `cargo test --workspace --no-fail-fast` on the branch → capture the FAILED set → diff against the persistent main worktree at `/workspace/.worktrees/kali-main` (built at merge-base, 0 failures). A stage/task is green only when its target tests pass **and** the global failing set strictly shrank **and** zero main-green tests turned red. Plain `cargo test --workspace` fail-fasts at the first failing binary — always enumerate with `--no-fail-fast`; exit-code verdicts use the exact CI command.
- **Fix, never flip.** Every target test gets a real implementation matching node's observable behavior. No construct is rejected/trapped to pass. No self-check `throw` may be re-silenced (no re-masking).
- **Both-sides hand-mirror discipline (non-negotiable).** Every new recognizer needs an arm in **both** kali_codegen (emit oracle) and kali_types (resolve predicate), or it fails open. Reviewed per recognizer.
- **Parity is defined by node**, same fixture, byte-for-byte.
- **GC-less** stays true — nothing here introduces tracing GC.
- **Design doc:** `docs/superpowers/specs/2026-07-12-throw-fallout-stage3-host-wiring-design.md`.
- **The conditional-import 10-step lockstep** (from the mechanics map; every new `kali:rt` conditional import touches all ten in lockstep):
  1. A `const`/`TypeSection` entry if the signature is new (`crates/kali_codegen/src/lower.rs:362-382`).
  2. `import_section.import("kali:rt", …)` appended **after** `args_get` at `lower.rs:473` (keeps all existing import indices stable).
  3. A `program_uses_*` probe fn + its call at `lower.rs:77-84` (make it a **superset** of the emit recognizer — over-inclusive is the safe side).
  4. An index formula `let X_import_index = if uses_X { Some(COVERAGE_HIT_IMPORT_INDEX + <all prior conditional flags>) } else { None };` in the `lower.rs:99-196` block.
  5. **The same flag added to `function_index_offset` at `lower.rs:86-95`** — the easy-to-miss step; miss it and every user-function call index shifts wrong.
  6. A `FunctionEmitter` `Option<u32>` field + ctor param + threading (`crates/kali_codegen/src/emit/emitter.rs:72-79` fields, `:184-191` ctor; `lower.rs:712-736` construction).
  7. A `*_import_index` recognizer in `crates/kali_codegen/src/intrinsics/host.rs` (template: `env_get_import_index` at `host.rs:93-111`).
  8. An emit arm in `emit_call` (`crates/kali_codegen/src/emit/call.rs`, before the generic fall-through at `:2300`).
  9. A kali_types admission arm (`crates/kali_types/src/late_host.rs` + dispatch in `resolve/call.rs:56-82` or `resolve/member.rs`).
  10. JS entries in all four browser `importObject` lists (List A `harness.rs:220-327`, List B `harness.rs:592-703`, List C `cmd_build.rs:1552-1649`, List D `cmd_build.rs:1850-1948`).

---

## File map

| File | Responsibility | Tasks |
|---|---|---|
| `docs/superpowers/followups/throw-fallout-stage3-triage.md` | Target-set enumeration + per-bucket empirical failure pin | 1, 10 |
| `crates/kali_runtime/src/browser/harness.rs` | Browser JS import lists A & B | 2, 5, 6, 7 |
| `crates/kali_cli/src/bin/cmd_build.rs` | Browser JS import lists C (ESM) & D (CJS) | 2, 5, 6, 7 |
| `crates/kali_runtime/src/browser/harness_tests/…` (new mirror-sync test) | Guard: the four lists declare the same host-wired `kali:rt` members | 2 |
| `crates/kali_codegen/src/intrinsics/host.rs` | Recognizers: process.kill widen, perf.now, crypto_* import-index fns | 3, 5, 6, 7 |
| `crates/kali_codegen/src/emit/call.rs` | `emit_call` dispatch arms | 3, 5, 6, 7 |
| `crates/kali_codegen/src/emit/emitter.rs` | Emitter `Option<u32>` import-index fields | 5, 6, 7 |
| `crates/kali_codegen/src/lower.rs` | Import declaration, `uses_*` probes, index arithmetic, `function_index_offset` | 5, 6, 7 |
| `crates/kali_types/src/late_host.rs`, `resolve/call.rs` | kali_types admission arms | 3, 5, 6, 7 |
| `crates/kali_codegen/src/emit/operators.rs` | env equality (`is_env_get_string_call`, streq guard) | 8 |
| `crates/kali_codegen/src/emit/control_flow.rs` or await lowering site | synchronously-settled `await` lane | 4 |
| `crates/kali_runtime/src/host/imports_default.rs` | NEW `crypto_subtle_digest` host import | 7 |
| `crates/kali_runtime/src/browser/*` (CDP/HTML driver) | CDP crash-lane propagation | 9 |

---

## Task 1: Stage-3 triage — enumerate the exact target set and pin every bucket's failure mode empirically

**No code changes. Deliverable: the triage doc + repro transcripts.** Per the program's twice-learned lesson (Stage 1 and Stage 2 forecasts both falsified), no fix is written against an assumed failure mode.

**Files:**
- Create: `docs/superpowers/followups/throw-fallout-stage3-triage.md`
- Scratch: `$SCRATCH/stage3-pre.txt` (branch failing set), `$SCRATCH/stage3-main.txt` (main worktree failing set)

**Interfaces:**
- Produces: the confirmed 45-name target set (buckets #5/#6/H/K), and for each bucket a pinned "current behavior on the branch binary" (exact stdout/stderr/exit + diagnostic code) that Tasks 3–9 assert against.

- [ ] **Step 1: Verify the main worktree is clean.**

Run: `cd /workspace/.worktrees/kali-main && git log --oneline -1 && cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort > "$SCRATCH/stage3-main.txt" && wc -l "$SCRATCH/stage3-main.txt"`
Expected: 0 lines (the gate baseline; the empty `stage3-main.txt` is consumed by Task 10's diff). If non-zero, STOP — the gate is poisoned (see memory `ci-gate-vs-poisoned-baseline`).

- [ ] **Step 2: Enumerate the branch failing set.**

Run: `cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort > "$SCRATCH/stage3-pre.txt" && wc -l "$SCRATCH/stage3-pre.txt"`
Expected: 923 lines (the entering denominator). If drift, record it in the triage doc before proceeding.

- [ ] **Step 3: Extract the four target buckets by name pattern and confirm counts.**

Run these greps against `$SCRATCH/stage3-pre.txt` and record the exact matching names in the triage doc:
- `#5 performance.now`: `grep performance_now` → expect 21.
- `#6 web crypto`: `grep -E 'crypto|random_uuid|subtle|get_random_values'` → expect 18.
- `H coverage_hit`: `grep -E 'reports_function_coverage.*browser_api_surface'` → expect 2 (`test_reports_function_coverage_in_json_output_when_browser_api_surface_is_{configured,inherited}`, `runtime_smoke/test.rs:6960/6994`).
- `K process.kill`: `grep -E 'process_kill_zero_probe|optional_chain_wrapped_process_kill'` → expect 4 (`node_api_surface::core`, `core.rs:1269/1381/1501/1615`).

Expected: 45 total. Any deviation is recorded and reconciled against the denominator doc.

- [ ] **Step 4: Pin the perf.now failure mode (resolve the placeholder-vs-E5506 conflict).**

The two source explorations disagreed on whether an unrecognized `performance.now()` yields placeholder-0 or a fail-closed E5506. Pin it. Build the branch binary once (`cargo build -p kali_cli`), then:

Run: write `async function main(){ const a = performance.now(); await Promise.resolve(); const b = performance.now(); if (typeof a !== 'number' || b < a) throw new Error('x'); console.log('perf ok'); } main();` to a temp `.js` and run `./target/debug/kali run <file>` (wasmtime lane) AND with `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node ./target/debug/kali --output json run --api browser <file>` (browser lane). Record exact stdout/stderr/exit + any diagnostic code for BOTH lanes.

Also isolate the async wrapper: run `console.log(performance.now());` (no async, no await) and `async function m(){ await Promise.resolve(); console.log('ok'); } m();` (await, no perf.now) separately. This tells you whether the blocker is the missing recognizer, the `await Promise.resolve()` shape, or the async wrapper itself. Record which.

Expected: a definitive statement in the triage doc: "perf.now currently → {E5506 reject | placeholder-0 | async-wrapper trap}; the async wrapper {does | does not} already flatten+run; `await Promise.resolve()` {works | traps | is a no-op}." This decides whether Task 4 (await lane) is a prerequisite for Tasks 5–7 or shrinks to nothing.

- [ ] **Step 5: Pin the crypto failure mode.**

Run the two crypto fixtures from `runtime_smoke/run.rs:8850` (subtle.digest+randomUUID) and the getRandomValues fixture (`run.rs:2234` area) on the branch binary (browser lane, `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`). Record exact behavior. Confirm the mechanics finding that **`crypto_subtle_digest` has no `kali:rt` host import** (only `kali:node` `create_hash` at `imports_node.rs:202`) — grep `imports_default.rs` for `subtle`/`digest` and record absence. This confirms Task 7 must add a host import.

- [ ] **Step 6: Pin the process.kill failure mode (which shapes actually fail).**

The codegen recognizer already passes a 25-case Object.freeze/globalThis/bracket transparent-wrapper unit test (`intrinsics/host_tests/process.rs:175+`). So the 4 bucket-K failures are NOT the freeze shapes. Inspect the fixture sources: `process_kill_zero_probe_node_api_surface_run_source` (`kali_common/src/process_kill.rs:348`) uses `sequenceKill(0)`, `bracketedRootSequenceKill(0)`, `process.kill(zeroAlias)` (static-zero-alias arg), and optional-chain forms (`core.rs:1381`). Run each failing test with `cargo test -p kali_cli --test <bin> <name> -- --nocapture` and record the exact stderr/diagnostic. Determine which receiver/arg shapes (sequence-expression receiver, static-zero-alias arg, optional-chain receiver) the recognizer misses. This scopes Task 3 precisely.

- [ ] **Step 7: Confirm the coverage_hit LinkError.**

Run `test_reports_function_coverage_in_json_output_when_browser_api_surface_is_configured` (`runtime_smoke/test.rs:6960`) with `--nocapture` and confirm the failure is a browser `WebAssembly.instantiate` LinkError for a missing `coverage_hit` import (not something else). Record the exact error text.

- [ ] **Step 8: Write the triage doc and commit.**

Write `docs/superpowers/followups/throw-fallout-stage3-triage.md` capturing: the 45-name target set (by bucket), the four pinned failure modes (Steps 4–7), the async-wrapper decision (Step 4), and the confirmed `crypto_subtle_digest` host-import gap (Step 5). Structure it like the Stage-2 triage doc.

```bash
git add docs/superpowers/followups/throw-fallout-stage3-triage.md
git commit -m "docs(soundness): throw-fallout Stage 3 triage — target set + per-bucket failure pins"
```

---

## Task 2: coverage_hit browser import-list entries + the mirror-sync guard test (bucket H, 2 tests)

Smallest self-contained drain. `coverage_hit` is already declared conditionally guest-side (`lower.rs:419`) and wired host-side (`imports_default.rs:208`); it is only missing from the four browser JS lists, causing the LinkError. This task also stands up the divergence-guard infra Tasks 5–7 reuse.

**Files:**
- Modify: `crates/kali_runtime/src/browser/harness.rs` (List A ~`:220-327`, List B ~`:592-703`)
- Modify: `crates/kali_cli/src/bin/cmd_build.rs` (List C ~`:1552-1649`, List D ~`:1850-1948`)
- Create/Modify test: a mirror-sync guard test (e.g. `crates/kali_runtime/src/browser/harness_tests.rs` or an existing browser test module — follow the crate's test-placement convention)

**Interfaces:**
- Produces: `coverage_hit` present in all four lists; a reusable `HOST_WIRED_KALIRT_IMPORTS` concept the guard test checks each list against.

- [ ] **Step 1: Write the failing guard test.**

Add a test that extracts the `kali:rt` member names from each of the four `format!` templates and asserts every host-wired conditional import this program adds is present in all four. Start with the members that MUST be in all four today plus `coverage_hit`:

```rust
#[test]
fn browser_import_lists_declare_all_host_wired_kalirt_members() {
    // Members every browser importObject must expose (conditional imports the guest may emit).
    const REQUIRED: &[&str] = &["coverage_hit"];
    for (label, src) in browser_import_list_sources() {
        for member in REQUIRED {
            assert!(
                src.contains(&format!("{member}(")) || src.contains(&format!("{member} (")),
                "browser import list {label} is missing kali:rt member `{member}`"
            );
        }
    }
}
```

Provide `browser_import_list_sources()` returning `[("harness.A", LIST_A_SRC), ("harness.B", LIST_B_SRC), ("cmd_build.esm", LIST_C_SRC), ("cmd_build.cjs", LIST_D_SRC)]`. If the four templates aren't already `const`/`fn`-accessible strings, expose them (extract each `format!` body to a `const` template or a `pub(crate) fn …_import_object_js() -> String` that the emit sites also call — single-sourcing each list's *text* is fine even though we are not single-sourcing across the four).

- [ ] **Step 2: Run it to verify it fails.**

Run: `cargo test -p kali_runtime browser_import_lists_declare_all_host_wired_kalirt_members -- --nocapture`
Expected: FAIL — `coverage_hit` missing from all four.

- [ ] **Step 3: Add the `coverage_hit` JS entry to all four lists.**

Mirror the `args_get` byte-writing / direct-host-call pattern (`harness.rs:254-261`). `coverage_hit` takes an `i32` coverage id and records it; in the browser harness the simplest real implementation collects ids into a JS array the summary reads. Add to each list:

```js
      coverage_hit(id) {{
        coverageHits.push(id);
      }},
```

Declare `const coverageHits = [];` in each harness scope and include it in the emitted coverage summary the harness already produces (follow how the harness reports `registeredTestFailures`, `harness.rs:342-359`). For the two `cmd_build.rs` bundle lists, match the bundle's existing summary-plumbing shape. (Task 1 Step 7 pinned exactly what the coverage test reads back — satisfy that shape.)

- [ ] **Step 4: Run the guard test — expect PASS.**

Run: `cargo test -p kali_runtime browser_import_lists_declare_all_host_wired_kalirt_members`
Expected: PASS.

- [ ] **Step 5: Run the two bucket-H target tests — expect PASS.**

Run: `cargo test -p kali_cli --test runtime_smoke test_reports_function_coverage_in_json_output_when_browser_api_surface`
Expected: both `_configured` and `_inherited` PASS (byte-for-byte coverage JSON vs node).

- [ ] **Step 6: Commit.**

```bash
git add crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs crates/kali_runtime/src/browser/harness_tests.rs
git commit -m "fix(runtime): wire coverage_hit into all four browser import lists + mirror-sync guard (throw-fallout Stage 3 bucket H)"
```

---

## Task 3: process.kill(0) receiver/arg widening (bucket K, 4 tests)

Pure codegen + kali_types recognizer widening — no imports, no lists, no await. Independent of Tasks 4–7. Task 1 Step 6 pinned the exact missing shapes (sequence-expression receiver, static-zero-alias arg, optional-chain receiver).

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (`is_process_kill` at `:446-462` and/or the zero-arg check in the `emit_call` arm at `call.rs:2258-2288`)
- Modify: `crates/kali_types/src/late_host.rs` (`resolve_process_kill_call` at `:39-77`) if types-side also rejects the shapes
- Test: `crates/kali_codegen/src/intrinsics/host_tests/process.rs`

**Interfaces:**
- Consumes: the pinned shape list from Task 1 Step 6.
- Produces: `process.kill(0)` accepted for sequence-expression receivers, static-zero-alias args, and optional-chain receivers → lowers to `i64.const 1`, no `process_exit` import; unsupported (non-zero / non-static / extra-arg) shapes keep their `FEATURE_UNAVAILABLE` (E5506) reject.

- [ ] **Step 1: Write the failing unit tests for the pinned missing shapes.**

Extend `process.rs` mirroring `process_kill_zero_probe_lowers_through_transparent_wrappers_…` (`:175`). Use the exact shapes Task 1 pinned as failing. Example (adjust to the pinned set):

```rust
#[test]
fn process_kill_zero_probe_accepts_static_zero_alias_and_sequence_receivers() {
    for source in [
        "const z = 0; process.kill(z);",
        "const z = 0; const z2 = z; process.kill(z2);",
        "(0, process.kill)(0);",
        "process?.kill(0);",
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig { max_specializations: 16, compat_eval: false, coverage: false });
        let result = lower_lir_to_wasm(&mut ctx, &program);
        assert!(result.diagnostics.is_empty(), "source {source:?}: {:?}", result.diagnostics);
        let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
        Validator::new().validate_all(&result.wasm_bytes).expect("validate");
        assert!(printed.contains("i64.const 1"), "source {source:?}: {printed}");
        assert!(!printed.contains("process_exit"), "source {source:?}: {printed}");
    }
}
```

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p kali_codegen process_kill_zero_probe_accepts_static_zero_alias_and_sequence_receivers -- --nocapture`
Expected: FAIL (diagnostics non-empty or missing `i64.const 1`) for the shapes the recognizer misses.

- [ ] **Step 3: Widen the recognizer's receiver-unwrap and arg-resolution.**

In the `emit_call` process.kill arm (`call.rs:2258`), resolve the receiver through the same transparent/bound/sequence unwrap the fixture needs, and resolve the argument through static-zero resolution. The receiver is already resolved through `resolve_bound_member_callable_node` at `emit_call`'s top (`call.rs:57-60`), which unwraps `??`/`&&`/`||`/transparent — confirm it also covers sequence `(0, expr)` receivers; if not, extend it or add a sequence-unwrap. For the static-zero **argument**, reuse the existing static-zero recognizer the transparent-wrapper test already relies on (find it near the `is_process_kill` arm — the `+0`/`(0)` cases pass today, so a static-value resolver exists; extend it to follow `const z = 0` alias bindings via `resolve_bound_node`, `call.rs:3211`). Keep non-zero/non-static args on the `FEATURE_UNAVAILABLE` path.

- [ ] **Step 4: Mirror on the kali_types side if needed.**

If Task 1 pinned that the reject originates in `resolve_process_kill_call` (`late_host.rs:39-77`) rather than codegen, widen its `resolve_static_callable_name` / arg check symmetrically so types admits exactly what codegen now emits. Verify no admit/emit desync: a shape admitted by types must emit in codegen and vice versa.

- [ ] **Step 5: Run the unit test — expect PASS.**

Run: `cargo test -p kali_codegen process_kill_zero_probe_accepts_static_zero_alias_and_sequence_receivers`
Expected: PASS.

- [ ] **Step 6: Run the 4 bucket-K target tests — expect PASS.**

Run: `cargo test -p kali_cli --test node_api_surface process_kill_zero_probe optional_chain_wrapped_process_kill`
Expected: all 4 PASS.

- [ ] **Step 7: Confirm the negative pins still reject.**

Run the existing `FEATURE_UNAVAILABLE` process.kill rejection tests (grep `process_kill.*unavailable` / the non-zero-arg pins). Expected: still reject — the widening did not over-admit.

- [ ] **Step 8: Commit.**

```bash
git add crates/kali_codegen/src/intrinsics/host.rs crates/kali_codegen/src/emit/call.rs crates/kali_types/src/late_host.rs crates/kali_codegen/src/intrinsics/host_tests/process.rs
git commit -m "fix(codegen+types): process.kill(0) accepts sequence/optional-chain receivers + static-zero aliases (throw-fallout Stage 3 bucket K)"
```

---

## Task 4: The synchronously-settled `await` lane (prerequisite for #5/#6 iff Task 1 says so)

**Conditional task.** If Task 1 Step 4 pinned that the async wrapper already flattens+runs and `await Promise.resolve()` already works, this task shrinks to "add a regression pin" (Steps 1–2, 6) and the recognizer tasks proceed directly. If the async wrapper or `await Promise.resolve()` blocks, implement the narrow lowering here first.

**Files:**
- Modify: the await-lowering site (identify via Task 1 — likely the async-flatten lane from Stage 0; `crates/kali_codegen/src/emit/` — grep for how `AwaitExpression` / async bodies lower). kali_types `resolve/mod.rs:273` handles `AwaitExpression` in resolution.
- Test: a codegen or runtime_smoke test for `await Promise.resolve()` + `await <host intrinsic>`.

**Interfaces:**
- Produces: `await <operand that settles synchronously>` (a host-intrinsic call OR `Promise.resolve(v)`) evaluates as direct synchronous evaluation, yielding the operand's value; `await Promise.resolve()` (no value) is a synchronous no-op sequencing point. Consumed by Tasks 5–7's async-wrapped fixtures.

- [ ] **Step 1: Write the failing test (minimal, isolates await from host intrinsics).**

```rust
// runtime_smoke style — assert stdout parity with node
// fixture: async function m(){ const v = await Promise.resolve(7); if (v !== 7) throw new Error('x'); console.log(v); } m();
```
Run kali on the fixture; expect `7`. Also `async function m(){ await Promise.resolve(); console.log('ok'); } m();` → expect `ok`.

- [ ] **Step 2: Run to verify current behavior matches Task 1's pin.**

Run the fixtures on the branch binary. Expected: matches what Task 1 Step 4 recorded. If already green, note it and skip to Step 6 (regression pin only).

- [ ] **Step 3: Implement the synchronously-settled await lowering.**

At the await-lowering site, when the awaited operand is (a) a recognized host-intrinsic call, or (b) `Promise.resolve(<expr>)`, lower it as direct evaluation of the inner value (for `Promise.resolve(v)`, unwrap to `v`; for a bare `Promise.resolve()`, produce unit/no value and continue). Do NOT attempt general Promise/microtask semantics — those are Stage 7. Add a kali_types arm so this shape is admitted symmetrically. Any await whose operand is NOT provably synchronously-settled keeps today's behavior (do not regress it to a new wrong answer — leave it for Stage 7).

- [ ] **Step 4: Mirror in kali_types.**

Ensure `resolve/mod.rs:273` (`AwaitExpression`) admits the synchronously-settled shapes without demanding the full async machinery, symmetric with codegen.

- [ ] **Step 5: Run the test — expect PASS.**

Run: `cargo test -p kali_cli --test runtime_smoke <await test names>`
Expected: PASS with node-parity stdout.

- [ ] **Step 6: Commit.**

```bash
git add -A
git commit -m "feat(codegen+types): synchronously-settled await lane (Promise.resolve + host intrinsics) for the async-wrapped host-wiring fixtures (throw-fallout Stage 3)"
```

---

## Task 5: performance.now recognizer end-to-end (bucket #5, 21 tests)

Follows the 10-step lockstep. Host impls already exist (`imports_default.rs:86/92`); this makes them reachable. Depends on Task 4 iff Task 1 flagged the async wrapper as a blocker (the #5 fixtures are async-wrapped with `await Promise.resolve()`).

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (type entry, import decl, `uses_performance_now` probe, index formula, `function_index_offset`, emitter threading)
- Modify: `crates/kali_codegen/src/emit/emitter.rs` (field + ctor param)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (`performance_now_import_index` recognizer)
- Modify: `crates/kali_codegen/src/emit/call.rs` (emit arm)
- Modify: `crates/kali_types/src/late_host.rs` + `resolve/call.rs` (admission arm)
- Modify: all four browser JS lists (`harness.rs`, `cmd_build.rs`) + extend the Task 2 guard's `REQUIRED` with `performance_now`
- Test: `crates/kali_codegen/src/intrinsics/host_tests/` (new perf.now unit test)

**Interfaces:**
- Consumes: `COVERAGE_HIT_IMPORT_INDEX` base + all prior conditional flags (`lower.rs:99-196` pattern); Task 4's await lane.
- Produces: `program_uses_performance_now(lir) -> bool`; `FunctionEmitter.performance_now_import_index: Option<u32>`; `performance_now_import_index(&self, callee_node: &LirNode) -> Option<u32>`; a `resolve_performance_now_call` types arm. `performance.now()` emits `import "kali:rt" "performance_now"` and leaves an f64 (`ValueShape::Float`) on the stack.

- [ ] **Step 1: Write the failing codegen unit test.**

```rust
#[test]
fn performance_now_lowers_to_kalirt_import_returning_float() {
    let program = parse_and_lower_lir("performance.now();");
    let mut ctx = CodegenCtx::new(TargetConfig { max_specializations: 16, compat_eval: false, coverage: false });
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new().validate_all(&result.wasm_bytes).expect("validate");
    assert!(printed.contains("import \"kali:rt\" \"performance_now\""), "{printed}");
}
```

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p kali_codegen performance_now_lowers_to_kalirt_import_returning_float -- --nocapture`
Expected: FAIL (import absent; diagnostics may show E5506 or placeholder per Task 1's pin).

- [ ] **Step 3: Add the `() -> f64` type + conditional import declaration.**

In `lower.rs`: add a `PERFORMANCE_NOW_TYPE_INDEX` for `() -> f64` in the `TypeSection` block (`:362-382`, alongside `ARGS_GET_TYPE_INDEX`). Append `import_section.import("kali:rt", "performance_now", EntityType::Function(PERFORMANCE_NOW_TYPE_INDEX));` **after** `args_get` (`:473`), gated on `uses_performance_now`.

- [ ] **Step 4: Add the `program_uses_performance_now` probe + index arithmetic + function_index_offset term.**

Add `program_uses_performance_now(lir)` (walk: `node.kind == Call`, callee text `"now"`, object text `"performance"`) and call it at `lower.rs:77-84`. Add `let performance_now_import_index = if uses_performance_now { Some(COVERAGE_HIT_IMPORT_INDEX + <all eight prior conditional flags> + if uses_args_get {1} else {0}) } else { None };`. Add `+ if uses_performance_now {1} else {0}` to `function_index_offset` (`:86-95`). (New conditional imports go **after** `args_get`, so perf.now's index sums all prior flags including `uses_args_get`.)

- [ ] **Step 5: Thread the field through the emitter.**

Add `performance_now_import_index: Option<u32>` to `FunctionEmitter` (`emitter.rs:72-79`), its ctor param (`:184-191`), and pass it at construction (`lower.rs:712-736`).

- [ ] **Step 6: Add the recognizer + emit arm.**

In `host.rs`, add `performance_now_import_index(&self, callee_node: &LirNode) -> Option<u32>` mirroring `env_get_import_index` (`:93-111`): method text `"now"`, object text `"performance"`, return `self.performance_now_import_index`. In `emit_call` (`call.rs`, before `:2300`), add an arm: `if let Some(idx) = self.performance_now_import_index(&callee_node) { function.instruction(&Instruction::Call(idx)); return Ok(EmittedValue { produced: true, shape: ValueShape::Float, .. }); }` (match the exact `EmittedValue` construction used by other float-producing arms, e.g. `call.rs:154-158`).

- [ ] **Step 7: Add the kali_types admission arm.**

In `resolve/call.rs` dispatch (`:56-82`), add `self.resolve_performance_now_call(expr)`. Implement it in `late_host.rs` mirroring `resolve_process_kill_call` (`:39-77`): admit `performance.now` (no args) when `self.api_surface` permits; reject unsupported arg shapes with `e5::FEATURE_UNAVAILABLE`. Symmetric with the codegen recognizer.

- [ ] **Step 8: Add `performance_now` to all four browser JS lists + extend the guard.**

Add to List A, B, C, D:
```js
      performance_now() {{
        return performance.now();
      }},
```
(Returns a JS number; wasm result is f64, so no BigInt wrapping — unlike the `math_*` i64 entries.) Add `"performance_now"` to the Task 2 guard's `REQUIRED` array.

- [ ] **Step 9: Run the unit test + guard — expect PASS.**

Run: `cargo test -p kali_codegen performance_now_lowers_to_kalirt_import_returning_float && cargo test -p kali_runtime browser_import_lists_declare_all_host_wired_kalirt_members`
Expected: both PASS.

- [ ] **Step 10: Run the 21 bucket-#5 target tests — expect PASS.**

Run: `cargo test -p kali_cli --test runtime_smoke performance_now`
Expected: all 21 PASS (build/run/test, js/ts, wasmtime + browser-harness lanes), node-parity stdout `performance.now ok`.

- [ ] **Step 11: Commit.**

```bash
git add -A
git commit -m "feat(codegen+types+runtime): wire performance.now through the kali:rt import lane end-to-end (throw-fallout Stage 3 bucket #5)"
```

---

## Task 6: crypto.getRandomValues + crypto.randomUUID recognizers (part of bucket #6)

Both host impls exist (`imports_default.rs:98`, `:151`). getRandomValues fills a buffer in place and returns it; randomUUID writes a string. Same 10-step lockstep, two imports. Depends on Task 4 iff async-wrapped (the getRandomValues fixture is not always async; the digest fixture in Task 7 is).

**Files:** same set as Task 5, for two imports (`crypto_get_random_values` type 7 `(i32,i32)->i32`; `crypto_random_uuid` type 7 too).

**Interfaces:**
- Produces: `program_uses_crypto_get_random_values`, `program_uses_crypto_random_uuid` probes; two emitter `Option<u32>` fields; two `*_import_index` recognizers; a `resolve_crypto_call` types arm covering both. getRandomValues returns the same buffer handle it was given (fills in place); randomUUID returns a tagged string handle.

- [ ] **Step 1: Write the failing unit tests.**

```rust
#[test]
fn crypto_get_random_values_lowers_to_kalirt_import() {
    let program = parse_and_lower_lir("const b = new Uint8Array(8); crypto.getRandomValues(b);");
    let mut ctx = CodegenCtx::new(TargetConfig { max_specializations: 16, compat_eval: false, coverage: false });
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print");
    Validator::new().validate_all(&result.wasm_bytes).expect("validate");
    assert!(printed.contains("import \"kali:rt\" \"crypto_get_random_values\""), "{printed}");
}
#[test]
fn crypto_random_uuid_lowers_to_kalirt_import() {
    let program = parse_and_lower_lir("const u = crypto.randomUUID(); console.log(u.length);");
    let mut ctx = CodegenCtx::new(TargetConfig { max_specializations: 16, compat_eval: false, coverage: false });
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print");
    Validator::new().validate_all(&result.wasm_bytes).expect("validate");
    assert!(printed.contains("import \"kali:rt\" \"crypto_random_uuid\""), "{printed}");
}
```

- [ ] **Step 2: Run to verify they fail.**

Run: `cargo test -p kali_codegen crypto_get_random_values_lowers crypto_random_uuid_lowers -- --nocapture`
Expected: FAIL (imports absent).

- [ ] **Step 3: Declare both conditional imports (both type 7 `(i32,i32)->i32`).**

Append after `performance_now` in the import section; add `uses_crypto_get_random_values` / `uses_crypto_random_uuid` probes; add both to the index formulas (each sums all prior flags incl. `uses_performance_now`) and to `function_index_offset`. Type index 7 already exists (`lower.rs:362-364`) — reuse it.

- [ ] **Step 4: Thread both emitter fields (field + ctor param + construction).**

- [ ] **Step 5: Add both recognizers + emit arms.**

`crypto_get_random_values_import_index`: method `"getRandomValues"`, object `"crypto"`. Emit arm: decode the `Uint8Array` argument to `(ptr, len)` and pass both, then `Call` — **mirror the args_get buffer-passing emit at `operators.rs:325-380`** (the buffer handle → ptr/len split; array handles are i64 linear-memory handles). getRandomValues returns `out_len`; the fixture expects the SAME buffer back, so re-produce the original buffer handle as the arm's value (the host filled it in place). `crypto_random_uuid_import_index`: method `"randomUUID"`, object `"crypto"`. Emit arm: allocate a fixed-cap buffer via `__alloc_global` (≥36 bytes), pass `(buf, cap)`, call, then build a tagged string handle from `buf` + returned len — **mirror the env_get string-handle construction at `call.rs:2131-2178`** (`I64ExtendI32U` the byte count, subtract 1, `I64Or STRING_HANDLE_TAG`).

- [ ] **Step 6: Add the kali_types admission arm.**

`resolve_crypto_call` in `late_host.rs`, dispatched from `resolve/call.rs:56-82`: admit `crypto.getRandomValues(<buffer>)` and `crypto.randomUUID()`; reject unsupported shapes with `e5::FEATURE_UNAVAILABLE`. `crypto` is already a builtin global (`builtins.rs:122`) so the object resolves.

- [ ] **Step 7: Add both to all four browser JS lists + guard.**

```js
      crypto_get_random_values(outPtr, outLen) {{
        if (wasmMemory === null) {{ return 0; }}
        crypto.getRandomValues(new Uint8Array(wasmMemory.buffer, outPtr, outLen));
        return outLen;
      }},
      crypto_random_uuid(outPtr, outCap) {{
        if (wasmMemory === null) {{ return 0; }}
        const bytes = new TextEncoder().encode(crypto.randomUUID());
        if (bytes.length > outCap) {{ return -1; }}
        new Uint8Array(wasmMemory.buffer, outPtr, bytes.length).set(bytes);
        return bytes.length;
      }},
```
Add `"crypto_get_random_values"`, `"crypto_random_uuid"` to the guard's `REQUIRED`.

- [ ] **Step 8: Run unit tests + guard — expect PASS.**

Run: `cargo test -p kali_codegen crypto_get_random_values_lowers crypto_random_uuid_lowers && cargo test -p kali_runtime browser_import_lists_declare_all_host_wired_kalirt_members`

- [ ] **Step 9: Run the getRandomValues/randomUUID subset of bucket #6 — expect PASS.**

Run: `cargo test -p kali_cli --test runtime_smoke get_random_values random_uuid`
Expected: the non-digest crypto tests PASS. (Digest tests drain in Task 7.)

- [ ] **Step 10: Commit.**

```bash
git add -A
git commit -m "feat(codegen+types+runtime): wire crypto.getRandomValues + crypto.randomUUID end-to-end (throw-fallout Stage 3 bucket #6 part 1)"
```

---

## Task 7: crypto.subtle.digest — NEW host import + recognizer (rest of bucket #6)

The one intrinsic with **no `kali:rt` host import today** (only `kali:node` `create_hash`, `imports_node.rs:202`). Needs a new host func + the full lockstep. The fixtures are `await`-wrapped, so this depends on Task 4.

**Files:**
- Modify: `crates/kali_runtime/src/host/imports_default.rs` (new `crypto_subtle_digest` host import) + `kali_api_web/src/crypto.rs` already has the `SubtleCrypto::digest` impl (`:85-99`) to call.
- Modify: same codegen/types/lists set as Tasks 5–6.

**Interfaces:**
- Consumes: `kali_api_web`'s SHA digest (`crypto.rs:85-99`, SHA-256/512 etc.); Task 4's await lane.
- Produces: `crypto_subtle_digest(algo_ptr, algo_len, in_ptr, in_len, out_ptr, out_cap) -> i32` host import (returns digest byte length); a codegen recognizer for `crypto.subtle.digest(algo, bytes)`; a types arm. Result is a buffer whose `byteLength` the fixture reads (32 for SHA-256, 64 for SHA-512).

- [ ] **Step 1: Write the failing host-side runtime test.**

Mirror `crates/kali_runtime/src/execute_tests/crypto_random.rs` (the `runtime_exposes_performance_now` style, which hand-writes a `.wat` importing `kali:rt` and asserts the host provides it). Write a test that imports `kali:rt` `crypto_subtle_digest` and asserts instantiation + a SHA-256 digest of known input yields 32 bytes with the correct first bytes.

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p kali_runtime <digest host test> -- --nocapture`
Expected: FAIL — link error, `crypto_subtle_digest` not provided.

- [ ] **Step 3: Add the host import in `imports_default.rs`.**

Mirror `crypto_random_uuid` (`:151-163`) for the guest-memory read/write plumbing (`guest_memory`, `checked_offset`, `write_guest_bytes`). Read the algorithm name string (`algo_ptr`/`algo_len`) and input bytes (`in_ptr`/`in_len`) from guest memory, call `kali_api_web`'s `SubtleCrypto::digest(algo, &input)` (`crypto.rs:85-99`), write the digest to `out_ptr` (bounded by `out_cap`), return the digest length. Gate under `HostOperation` policy consistent with the other crypto imports (`enforce_operation(..., HostOperation::Random)` or a digest-appropriate op — match the existing crypto gating).

- [ ] **Step 4: Run the host test — expect PASS.**

Run: `cargo test -p kali_runtime <digest host test>`
Expected: PASS.

- [ ] **Step 5: Write the failing codegen unit test.**

```rust
#[test]
fn crypto_subtle_digest_lowers_to_kalirt_import() {
    let program = parse_and_lower_lir("const b = new Uint8Array(4); crypto.subtle.digest('SHA-256', b);");
    let mut ctx = CodegenCtx::new(TargetConfig { max_specializations: 16, compat_eval: false, coverage: false });
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print");
    Validator::new().validate_all(&result.wasm_bytes).expect("validate");
    assert!(printed.contains("import \"kali:rt\" \"crypto_subtle_digest\""), "{printed}");
}
```

- [ ] **Step 6: Run to verify it fails; then implement the codegen lockstep.**

Run: `cargo test -p kali_codegen crypto_subtle_digest_lowers -- --nocapture` (Expected FAIL). Then: add a new type `(i32,i32,i32,i32,i32,i32)->i32` to the `TypeSection`; declare the conditional import after the Task-6 crypto imports; add `program_uses_crypto_subtle_digest` (Call → member chain `crypto.subtle.digest`); add the index formula + `function_index_offset` term; thread the emitter field; add `crypto_subtle_digest_import_index` recognizer (matches the `crypto.subtle.digest` member chain) + emit arm passing `(algo_ptr, algo_len, in_ptr, in_len, out_ptr, out_cap)` — algo is a string literal handle (decode to ptr/len via the string-handle convention `operators.rs:367-376`), input is a Uint8Array (ptr/len like Task 6), output is an allocated buffer whose handle+returned-len becomes the result (a byte buffer with a readable `byteLength`).

- [ ] **Step 7: Add the kali_types arm.**

Extend `resolve_crypto_call` (Task 6) to admit `crypto.subtle.digest(<string>, <buffer>)`; reject unsupported algos/shapes with `e5::FEATURE_UNAVAILABLE`. `crypto.subtle` must resolve as a member — verify `subtle` resolves off the `crypto` builtin; if not, add the member resolution in `resolve/member.rs`.

- [ ] **Step 8: Add `crypto_subtle_digest` to all four browser JS lists + guard.**

```js
      crypto_subtle_digest(algoPtr, algoLen, inPtr, inLen, outPtr, outCap) {{
        // Browser lane: synchronous within the harness — see Task 4's synchronously-settled await.
        // Decode algo + input from wasmMemory, call crypto.subtle.digest, write bytes to outPtr.
        // NOTE: crypto.subtle.digest is async in browsers; the node harness resolves it before the
        // guest reads the buffer because the await lane sequences the host call. If Task 1 pinned
        // that the browser lane needs a sync digest, use a sync SHA (e.g. node:crypto in the node
        // harness) — decide from the Task 1 pin, do not guess here.
      }},
```
This entry's exact implementation is the one place gated on Task 1's async-wrapper pin — the node harness has `node:crypto` available synchronously; the plan's Task 1 Step 5 determines whether the browser lane can complete the digest before the guest reads `byteLength`. Add `"crypto_subtle_digest"` to the guard's `REQUIRED`.

- [ ] **Step 9: Run the codegen test + guard + digest target tests — expect PASS.**

Run: `cargo test -p kali_codegen crypto_subtle_digest_lowers && cargo test -p kali_runtime browser_import_lists_declare_all_host_wired_kalirt_members && cargo test -p kali_cli --test runtime_smoke subtle_digest`
Expected: PASS, digest `byteLength` 32 (SHA-256) / 64 (SHA-512) matching node.

- [ ] **Step 10: Commit.**

```bash
git add -A
git commit -m "feat(runtime+codegen+types): crypto.subtle.digest host import + end-to-end wiring (throw-fallout Stage 3 bucket #6 part 2)"
```

---

## Task 8: env-equality soundness — F-Stage1-2 (env-vs-env) + F-Stage1-3 (bound-alias)

Two `operators.rs` fixes. Near-zero test-count drain (soundness); carried here because it's the host env lane and Stage 7 builds on it. No new import.

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`is_env_get_string_call` `:912-923`, the streq equality guard `:1418-1428`)
- Test: a runtime_smoke test running under `Deno.env`/`process.env` with two distinct env vars and an aliased getter.

**Interfaces:**
- Produces: env-vs-env `==`/`===` compares by content (distinct scratch regions), and bound-alias `const g = Deno.env.get; g("K") === "y"` routes through `__streq`.

- [ ] **Step 1: Write the failing tests.**

Two runtime_smoke fixtures, asserting stdout parity with node:
- env-vs-env: set `A=foo`, `B=bar` (same length), fixture `if ((Deno.env.get('A') === Deno.env.get('B'))) console.log('eq'); else console.log('ne');` → node prints `ne`. (Today: silently excluded / wrong.)
- bound-alias: set `K=y`, fixture `const g = Deno.env.get; if (g('K') === 'y') console.log('eq'); else console.log('ne');` → node prints `eq`. (Today: `ne`, silent raw-handle compare — confirmed on branch in the Stage-1 triage.)

- [ ] **Step 2: Run to verify they fail.**

Run: `cargo test -p kali_cli --test runtime_smoke <env-vs-env test> <bound-alias test> -- --nocapture`
Expected: FAIL (kali prints the wrong branch).

- [ ] **Step 3: Fix F-Stage1-3 (bound-alias recognition).**

In `is_env_get_string_call` (`operators.rs:912-923`), resolve the callee through bound aliases before checking the env_get recognizer — mirror `is_runtime_concat_string`'s `let id = self.resolve_bound_node(id);` (`operators.rs:937`; `resolve_bound_node` at `call.rs:3211`). For member-callable aliases, use `resolve_bound_member_callable_node` (the path `emit_call` uses at `call.rs:57-60`).

- [ ] **Step 4: Fix F-Stage1-2 (env-vs-env distinct buffers).**

The equality guard excludes `left_env && right_env` because both env.get results land in the single reserved buffer `[0, 4096)` (`ENV_GET_BUFFER_RESERVED`, `lib.rs:75`). Give the second env.get its own scratch region so `__streq` reads the right bytes: either copy the first result out to a distinct offset before the second call, or allocate a second scratch slot. Then admit `left_env && right_env` in the guard (`:1428`). Keep the guard's other invariants intact.

- [ ] **Step 5: Run the tests — expect PASS.**

Run: `cargo test -p kali_cli --test runtime_smoke <env-vs-env test> <bound-alias test>`
Expected: PASS, node-parity.

- [ ] **Step 6: Run the existing env-equality pins — no regressions.**

Run: `cargo test -p kali_cli --test runtime_smoke deno_env env_get`
Expected: all still PASS (the Task 6 env accept-lane tests etc.).

- [ ] **Step 7: Commit.**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/runtime_smoke/
git commit -m "fix(codegen): env-vs-env content equality + bound-alias Deno.env.get streq routing (throw-fallout Stage 3, F-Stage1-2/3)"
```

---

## Task 9: CDP crash-lane reproducer (Stage-0 residual)

Harness integrity, zero drain. `browser_tests_failed` (`execute.rs:374-384`) already counts a non-success harness exit with zero reported failures as 1 failure — lane-agnostic, provided the CDP/HTML driver propagates an in-page trap to a non-zero process exit. Stage 0's reproducer only covered the node `.mjs` lane.

**Files:**
- Create test: an integration reproducer for the Chromium/HTML CDP entrypoint (mirror `browser_harness_failing_test_propagates_failure.rs` from Stage 0, but forcing the HTML lane via a Chromium-named harness command; `browser_harness_uses_html_entrypoint`, `command.rs:285`).
- Modify (only if the reproducer shows swallowing): the CDP/HTML driver in `crates/kali_runtime/src/browser/` to surface an in-page guest trap as a non-zero exit.

**Interfaces:**
- Produces: a passing reproducer proving an in-page guest trap in the CDP/HTML lane yields `success:false` / non-zero exit (not a swallowed `passed:1`).

- [ ] **Step 1: Write the failing (or regression-pin) reproducer.**

Mirror the Stage-0 node-lane reproducer but drive the HTML entrypoint (Chromium-named harness command, gated/skipped if no Chromium is available in CI — follow how existing CDP tests guard on `chromium`/`google-chrome` presence; see memory `browser-cdp-smoke-driver`). The fixture is a `Kali.test` whose body traps (`RuntimeError: unreachable`). Assert the run reports `success:false` and non-zero exit.

- [ ] **Step 2: Run it.**

Run: `cargo test -p kali_runtime <cdp reproducer> -- --nocapture` (or the CDP smoke lane, `mise run browser-smoke`)
Expected: if the CDP driver already propagates, PASS (it becomes a regression pin — the desired outcome). If it swallows (reports `passed:1`/exit 0), FAIL — proceed to Step 3.

- [ ] **Step 3: If swallowing, fix the CDP driver propagation.**

In the HTML/CDP driver, ensure an in-page guest trap sets a non-zero process exit / `harness_status_success = false` so `browser_tests_failed` counts it. Re-run Step 2 to green.

- [ ] **Step 4: Commit.**

```bash
git add -A
git commit -m "test(runtime): CDP/HTML in-page trap surfaces as run failure — Stage-0 residual crash-lane pin (throw-fallout Stage 3)"
```

---

## Task 10: Stage-3 checkpoint — flipped-pin re-verification + full gate + drain snapshot

**Files:**
- Modify: the runtime_smoke "flipped-pin" fixture(s) embedding `crypto.getRandomValues` (Stage-2 triage: currently GREEN asserting E5506/failure)
- Modify: `docs/superpowers/followups/throw-fallout-stage3-triage.md` (drain snapshot)

- [ ] **Step 1: Re-verify the crypto flipped-pin fixture honestly.**

The runtime_smoke enumeration flipped-pin fixture embedding `crypto.getRandomValues` (Stage-2 triage §"Flipped pin") currently asserts E5506/failure. Now that crypto is recognized, run it and observe which diagnostic (if any) fires. Re-pin to the honest post-fix outcome — **never let it flip green by accident** (Invariant: no silent flip). If the fixture's whole point was the crypto reject and crypto now works, reshape the assertion to the genuine remaining rejection cause or convert it to a success pin matching node, documenting the change.

- [ ] **Step 2: Run the full enumeration gate.**

Run: `cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort > "$SCRATCH/stage3-post.txt" && wc -l "$SCRATCH/stage3-post.txt"`
Expected: ≈878 (923 − 45). Record the exact number.

- [ ] **Step 3: Diff against main + verify zero new-red.**

Run: `comm -13 <(sort "$SCRATCH/stage3-main.txt") <(sort "$SCRATCH/stage3-post.txt")` (names failing on branch but not main) and `comm -23 <(sort "$SCRATCH/stage3-pre.txt") <(sort "$SCRATCH/stage3-post.txt")` (drained names).
Expected: the drained set = the 45 target names (± the honest flipped-pin adjustments); **zero** names that are green on main are red on branch. If any main-green test is red, it is a stage-introduced regression — bisect and fix before closing (Stage-2 lesson: per-task gates are necessary but not sufficient; the full enumeration is the only real gate).

- [ ] **Step 4: Verify the exact CI command's exit behavior.**

Run: `cargo test --workspace` (the exact CI command, fail-fast). Expected: still exit 101 mid-program (later buckets remain) — record the first failing binary for continuity, consistent with prior checkpoints.

- [ ] **Step 5: Snapshot the drain into the triage doc.**

Update `throw-fallout-stage3-triage.md` with the pre/post counts, the drained-name attribution per bucket, the honest note if any #5/#6 attributed forward to Stage 7 (per Task 1's async-wrapper pin), and any newly-prominent follow-ups. Mirror the Stage-2 checkpoint section style.

- [ ] **Step 6: Commit.**

```bash
git add -A
git commit -m "docs(soundness)+test(cli): throw-fallout Stage 3 checkpoint — 923→~878 drain snapshot + crypto flipped-pin re-verification"
```

---

## Self-review notes (addressed in this plan)

- **Placeholder-vs-E5506 conflict** between the two source explorations for unrecognized `performance.now()` is not assumed — Task 1 Step 4 pins it empirically and downstream tasks branch on it.
- **`crypto.subtle.digest` needs a NEW host import** (Task 7) — the other three intrinsics reuse existing host funcs.
- **`function_index_offset` (`lower.rs:86-95`)** is called out as the easy-to-miss lockstep step in the Global Constraints and in each recognizer task.
- **Both-sides mirror** (codegen + kali_types) is a step in every recognizer task (3, 4, 5, 6, 7).
- **The four import lists** = `harness.rs` ×2 + `cmd_build.rs` ×2 (matches memory `kali-browser-harness-import-sync`); the guard test (Task 2) closes the divergence class by construction.
- **No silent flips**: Task 10 Step 1 re-verifies the crypto flipped-pin honestly; Task 3 Step 7 confirms process.kill negative pins still reject.
- **The gate** is the full `--no-fail-fast` enumeration diffed against the main worktree (Task 10), never a per-task subset.
