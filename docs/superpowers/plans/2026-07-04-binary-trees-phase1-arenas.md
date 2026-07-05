# Binary-trees Phase 1: Dynamic Current-Arena Reclamation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `kali run --sandbox <fuel policy>` executes the canonical CLBG binary-trees benchmark at N=21 byte-for-byte, with per-loop/per-function arena reclamation keeping peak memory ~270MB against a ~9.4GB cumulative allocation.

**Architecture:** A single 64KB-page pool above `heap_base` with an intrusive free-page list. The active arena is three wasm globals (`__arena_page/cursor/limit`); opening a scope arena saves them into wasm locals and zeroes them (empty arena — first allocation fetches a page); reset walks the page list into the free list. Escape analysis in kali_mir gates which functions/loops get arenas, delivered to codegen as a name-keyed `ArenaTable` (the `ReprTable` precedent — no node ids survive lowering). Precursors: fuel-exhaustion diagnosability (P0a) and call-result-argument object-shape inference (P0b).

**Tech Stack:** Rust workspace (`crates/`), `wasm_encoder` for codegen, `wasmtime` runtime, `cargo test` per-crate gates.

**Spec:** `docs/superpowers/specs/2026-07-04-binary-trees-phase1-arenas-design.md` (read it before starting any task).

## Global Constraints

- **GC-less invariant:** no tracing/copying/generational GC machinery, no shadow stack, no write barriers. Reclamation is arena reset only.
- **REJECT-DON'T-MISCOMPILE:** unsupported surface → compile-time E5506, never a silent wrong answer.
- **FAIL CLOSED:** any `ArenaTable` miss or analysis ambiguity → no arena / `__alloc_global`. Wrong direction = correctness bug; conservative direction = only unreclaimed memory.
- **`__heap` stays wasm global index 0**, export name `"__heap"` unchanged (the host and browser glue resolve it by name; after Task 5 it means "page frontier").
- **Default 60M fuel guard unchanged** (`execute.rs:131-136`); benchmark-scale runs use scoped `--sandbox` policies.
- **No new host imports.** The 4 hand-mirrored browser JS import lists must not change (JS glue *implementations* may).
- **Existing CLBG fixtures stay output-identical** (n-body, spectral-norm, fannkuch, mandelbrot incl. the 5011-byte golden PBM). Wasm bytes will differ; canonical output must not.
- **Gate after every task:** `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli` green and `cargo fmt --check` clean.
- **Branch:** all work on `binary-trees-phase1-arenas` (created in Task 1, Step 1). Commit at the end of every task.
- Repro/scratch sources go in a temp dir, never committed. Temp fixture files in tests use the existing `AtomicU64`-counter slug helper idiom (see `heap_grow_runtime.rs`) — `as_nanos()+pid` alone collides on macOS CI.

## File Structure (net-new and load-bearing modifications)

```
crates/kali_runtime/src/execute.rs            # T1 fuel/trap diagnostics; T2 trap stdout
crates/kali_runtime/src/host/memory.rs        # T5 host strings via exported __alloc_global
crates/kali_types/src/repr_infer.rs           # T3 call-result arg → param obj_flow
crates/kali_types/src/repr_infer_tests.rs     # T3 unit tests
crates/kali_common/src/arena_table.rs         # T4 NEW: name-keyed ArenaTable
crates/kali_mir/src/analysis/arena_gate.rs    # T4 NEW: escape gate + loop ordinals
crates/kali_cli/src/build/compile.rs          # T4 plumbing: ctx.arena_table
crates/kali_codegen/src/ctx.rs                # T4 CodegenCtx.arena_table field
crates/kali_codegen/src/lower.rs              # T5 globals g1..g7, 4 synthetics, coverage set
crates/kali_codegen/src/emitter.rs            # T6 arena_frames stack; T7 function prologue
crates/kali_codegen/src/emit/control_flow.rs  # T6 loop open/reset/release, break/return unwind
crates/kali_codegen/src/emit/object.rs        # T3 E5506 backstop; T6 __alloc vs __alloc_global
crates/kali_cli/tests/trap_diagnostics_runtime.rs      # T1/T2 NEW
crates/kali_cli/tests/object_call_result_args_runtime.rs # T3 NEW
crates/kali_cli/tests/arena_reclamation_runtime.rs     # T6/T7 NEW
crates/kali_cli/tests/clbg_binary_trees_runtime.rs     # T8 NEW
crates/kali_cli/tests/fixtures/benchmarks/binary-trees-benchmark-v1.{ts,json,policy.json} # T8 NEW
specs/19-feature-maturity.md                  # T8 new row
```

Key constants (used consistently below):
- `PAGE = 65536`, `HEADER = 8` (`[0..4) next_page`, `[4..8) span_pages`), `PAYLOAD = PAGE - HEADER`.
- Globals: g0 `__heap` (frontier), g1 `__arena_page`, g2 `__arena_cursor`, g3 `__arena_limit`, g4 `__global_page`, g5 `__global_cursor`, g6 `__global_limit`, g7 `__free_list`. All i32, mutable, init 0 except g0 (init `heap_base`, unchanged).
- Boot state: all-zero trios ARE the open (empty) program arena and empty global arena — `_start` needs no prologue code.
- Synthetics, inserted after `_start`/before user functions in this order: `__alloc(i32)->i32`, `__alloc_global(i32)->i32` (exported), `__page_get(i32)->i32`, `__arena_reset()->()`.

---

### Task 1: P0a — fuel exhaustion gets a named E4003 diagnostic; trap kinds get wording

**Files:**
- Modify: `crates/kali_runtime/src/execute.rs:177-201` (the `start.call` error arm)
- Test: `crates/kali_cli/tests/trap_diagnostics_runtime.rs` (create)

**Interfaces:**
- Consumes: `wasmtime::Trap` variants (`OutOfFuel`, `MemoryOutOfBounds`, `UnreachableCodeReached`); `e4::RESOURCE_LIMIT_EXCEEDED` (=4003) from `kali_error/src/_error_codes.rs:80`; existing `runtime_error_diagnostic` helper.
- Produces: fuel exhaustion → `E4003` whose message contains `CPU fuel budget exhausted` and `resources.maxCpuTimeMs`; OOB/unreachable traps keep E4000 with a parenthesized kind.

- [ ] **Step 1: Create the branch**

```bash
git checkout -b binary-trees-phase1-arenas
```

- [ ] **Step 2: Write the failing test**

Create `crates/kali_cli/tests/trap_diagnostics_runtime.rs`. Copy the `kali_bin()` helper and the temp-source-file helper idiom from `crates/kali_cli/tests/heap_grow_runtime.rs` (including its `AtomicU64` slug counter). Then:

```rust
#[test]
fn fuel_exhaustion_reports_e4003_with_actionable_message() {
    // Runs forever; exhausts the 60M default fuel budget in well under a second.
    let source = write_temp_source(
        "fuel_runaway",
        "let i = 0;\nwhile (true) {\n  i = i + 1;\n}\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E4003"), "stderr: {stderr}");
    assert!(stderr.contains("CPU fuel budget exhausted"), "stderr: {stderr}");
    assert!(stderr.contains("resources.maxCpuTimeMs"), "stderr: {stderr}");
    assert!(
        !stderr.contains("E4000"),
        "fuel exhaustion must not present as a bare runtime trap: {stderr}"
    );
}
```

- [ ] **Step 3: Run it to verify it fails**

Run: `cargo test -p kali_cli --test trap_diagnostics_runtime -- fuel_exhaustion`
Expected: FAIL — stderr currently contains `E4000: runtime trap` and no fuel wording.

- [ ] **Step 4: Implement the trap-kind match**

In `crates/kali_runtime/src/execute.rs`, replace the final fallback in the `start.call` error arm (`execute.rs:197-200`):

```rust
let diagnostic = match error.downcast_ref::<wasmtime::Trap>() {
    Some(wasmtime::Trap::OutOfFuel) => Diagnostic::error(
        e4::RESOURCE_LIMIT_EXCEEDED as u32,
        "CPU fuel budget exhausted: the program ran past the runaway guard \
         (default ~60s-equivalent when no sandbox policy is set); grant more \
         compute by raising `resources.maxCpuTimeMs` in a --sandbox policy"
            .to_string(),
    ),
    Some(wasmtime::Trap::MemoryOutOfBounds) => runtime_error_diagnostic(format!(
        "runtime trap (out-of-bounds memory access): {}",
        error
    )),
    Some(wasmtime::Trap::UnreachableCodeReached) => runtime_error_diagnostic(format!(
        "runtime trap (unreachable — allocation failure or an unsupported-path guard): {}",
        error
    )),
    _ => runtime_error_diagnostic(format!("runtime trap: {}", error)),
};
return Err(vec![diagnostic]);
```

Keep the `take_pending_exit_code()` and `pending_diagnostic` checks above it exactly as they are. Check `use` imports for `e4` in this file (it is already used at `execute.rs:172`).

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test -p kali_cli --test trap_diagnostics_runtime -- fuel_exhaustion`
Expected: PASS

- [ ] **Step 6: Add the raised-policy counterpart test (bounded workload passes)**

Append to the same test file. This pins the P0a re-diagnosis: the exact repro that looked like a wild-pointer bug is byte-correct with fuel granted, to ≥64MB cumulative (8,000 iterations × 8,176B = ~65MB):

```rust
#[test]
fn deep_object_workload_is_correct_to_64mb_under_raised_fuel_policy() {
    let source = write_temp_source(
        "trees_64mb",
        r#"function bottomUpTree(depth) {
  if (depth > 0) {
    return { left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) };
  }
  return { left: null, right: null };
}
function itemCheck(t) {
  if (t.left === null) {
    return 1;
  }
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}
function main() {
  let check = 0;
  for (let i = 1; i <= 8000; i = i + 1) {
    const tree = bottomUpTree(8);
    check = check + itemCheck(tree);
  }
  console.log(check);
}
main();
"#,
    );
    let policy = write_temp_policy_json(
        "fuel_grant",
        r#"{
  "schemaVersion": 1,
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": null },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": null, "maxActiveTimers": null },
    "eval": false,
    "random": false,
    "console": true
  },
  "resources": {
    "maxMemoryMB": null,
    "maxCpuTimeMs": 600000,
    "maxOpenFiles": null,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // 8000 iterations x itemCheck(depth-8 tree) = 8000 x 511
    assert_eq!(String::from_utf8_lossy(&output.stdout), "4088000\n");
}
```

Add a `write_temp_policy_json` helper next to `write_temp_source` (same slug-counter idiom, `.json` extension). Verify the `--sandbox` flag spelling against `clbg_mandelbrot_runtime.rs`, which already passes a policy.

- [ ] **Step 7: Run the full test file, then the gate**

Run: `cargo test -p kali_cli --test trap_diagnostics_runtime`
Expected: 2 passed.
Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli && cargo fmt --check`
Expected: green, no fmt diffs. (Watch specifically for existing tests that assert on `runtime trap:` wording — the mandelbrot unbudgeted-run test pins E4000 behavior; if it asserted exact message text, update the assertion to the new kind-qualified wording, keeping the E4000/E4003 code assertions intact.)

- [ ] **Step 8: Commit**

```bash
git add crates/kali_runtime/src/execute.rs crates/kali_cli/tests/trap_diagnostics_runtime.rs
git commit -m "fix(runtime): fuel exhaustion reports named E4003 diagnostic, not a bare E4000 trap; trap kinds get wording"
```

---

### Task 2: P0a — buffered stdout survives a trap

**Files:**
- Modify: `crates/kali_runtime/src/execute.rs` (trap arm), the `RuntimeOutcome` struct (grep `struct RuntimeOutcome` in `crates/kali_runtime/src/`), and the `kali run` command handler (grep `fn cmd_run\|"run"` under `crates/kali_cli/src/`)
- Test: `crates/kali_cli/tests/trap_diagnostics_runtime.rs` (extend)

**Interfaces:**
- Consumes: Task 1's diagnostic construction.
- Produces: `RuntimeOutcome.trap: Option<Diagnostic>` (new field, `None` everywhere except the trap arm). On trap, `execute` returns `Ok(RuntimeOutcome { exit_code: 1, trap: Some(diagnostic), stdout/stdout_bytes/stderr: <captured state>, .. })` instead of `Err`. The CLI prints captured stdout first, then renders the diagnostic to stderr, exits nonzero.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn stdout_emitted_before_a_trap_is_not_lost() {
    let source = write_temp_source(
        "stdout_before_trap",
        "console.log(777);\nlet i = 0;\nwhile (true) {\n  i = i + 1;\n}\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("777"),
        "pre-trap stdout must be flushed; got stdout: {:?} stderr: {:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("E4003"));
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p kali_cli --test trap_diagnostics_runtime -- stdout_emitted_before_a_trap`
Expected: FAIL — stdout is empty today (the trap arm returns `Err` and drops the store's buffered stdout).

- [ ] **Step 3: Scout the blast radius (read-only)**

Run: `grep -rn "RuntimeOutcome {" crates/ --include="*.rs" | grep -v tests` and `grep -rn "execute" crates/kali_cli/src/ --include="*.rs" | grep -i "run\|outcome" | head -30`.
List every `RuntimeOutcome` literal constructor and every consumer that branches on `Ok`/`Err` from the runtime entry point (`kali run`, `kali test`, embed/API surfaces). You will add `trap: None` to every constructor and trap-aware handling ONLY to the `kali run` path (other surfaces keep their current behavior — the field is additive).

- [ ] **Step 4: Implement**

(a) Add the field to `RuntimeOutcome`:

```rust
/// Set when execution ended in a wasm trap: the run produced the captured
/// stdout/stderr up to the trap, `exit_code` is nonzero, and this holds the
/// diagnostic to render. `None` for clean completion.
pub trap: Option<Diagnostic>,
```

(b) In the Task-1 trap arm, replace `return Err(vec![diagnostic]);` with an `Ok` carrying state (mirror the `take_pending_exit_code` block at `execute.rs:178-193` verbatim, changing only `exit_code: 1` and `trap: Some(diagnostic)`).

(c) Add `trap: None` to every other `RuntimeOutcome` constructor found in Step 3.

(d) In the `kali run` handler: where it consumes a successful outcome, first print `outcome.stdout` / write `outcome.stdout_bytes` exactly as it does today, then if `outcome.trap` is `Some(diagnostic)`, render it through the same diagnostic-printing path used for `Err` diagnostics and exit with `outcome.exit_code`. If `kali test`'s handler shares the helper, give it the same treatment; do not touch embed/node/deno surfaces beyond the added field.

- [ ] **Step 5: Run the test file and the gate**

Run: `cargo test -p kali_cli --test trap_diagnostics_runtime`
Expected: 3 passed (Task 1's two tests still pass — E4003 still reaches stderr through the new path).
Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli && cargo fmt --check`
Expected: green. If any existing test asserted that a trap yields empty stdout, update it — the new behavior is the specified one.

- [ ] **Step 6: Commit**

```bash
git add -A crates/
git commit -m "fix(runtime,cli): preserve and flush buffered stdout when a run traps; trap diagnostic carried on RuntimeOutcome"
```

---

### Task 3: P0b — call-result arguments seed callee param object shapes; E5506 backstop

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs` (`CallEdge` at :40, `visit_call` identifier branch at :1031-1055, `resolve_calls` scalar branch at :1129-1140)
- Modify: `crates/kali_codegen/src/emit/object.rs` (or the shared member-read arm in `control_flow.rs` — located in Step 6) for the backstop
- Test: `crates/kali_types/src/repr_infer_tests.rs` (extend), `crates/kali_cli/tests/object_call_result_args_runtime.rs` (create)

**Interfaces:**
- Consumes: existing `ObjSlot::{Binding, ArrayElem, Return}` (repr_infer.rs:116-124), `obj_flows` fixpoint (orientation-insensitive), `record_object_flow_from_expr` (:293) as the pattern to mirror.
- Produces: `itemCheck(bottomUpTree(d))` correct with no other seeding call site; unclassifiable object-shaped member reads → E5506 at compile time.

- [ ] **Step 1: Write the failing e2e test**

Create `crates/kali_cli/tests/object_call_result_args_runtime.rs` (same helpers as Task 1):

```rust
#[test]
fn object_call_result_passed_directly_as_argument_is_correct() {
    // No bound-identifier call site anywhere: the param shape must come from
    // the call-result argument itself. Depth 10 => itemCheck = 2^11 - 1.
    let source = write_temp_source(
        "call_result_arg",
        r#"function bottomUpTree(depth) {
  if (depth > 0) {
    return { left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) };
  }
  return { left: null, right: null };
}
function itemCheck(t) {
  if (t.left === null) {
    return 1;
  }
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}
function main() {
  console.log(itemCheck(bottomUpTree(10)));
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run").arg(&source).output().expect("run kali");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2047\n");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p kali_cli --test object_call_result_args_runtime`
Expected: FAIL — prints `1\n` today (the known miscompile).

- [ ] **Step 3: Write the failing unit test**

Read the top of `crates/kali_types/src/repr_infer_tests.rs` first and reuse its parse/infer helpers verbatim. Then add (adjust helper names to what the file actually uses):

```rust
#[test]
fn call_result_argument_seeds_callee_param_object_shape() {
    let table = infer_from_source(
        r#"function mk() {
  return { left: null, right: null };
}
function check(t) {
  if (t.left === null) { return 1; }
  return 2;
}
function main() {
  console.log(check(mk()));
}
main();
"#,
    );
    assert!(
        matches!(table.param("check", 0), kali_common::Repr::Object(_)),
        "param must receive the object shape from the call-result argument"
    );
}
```

Verify the accessor name (`param` is what codegen calls per `call.rs:2242-2261`; confirm in `kali_common/src/repr.rs:33`).

- [ ] **Step 4: Implement the inference fix**

In `repr_infer.rs`:

(a) `CallEdge` (at :40) gains a field:

```rust
/// For each positional argument, the object slot the argument's value
/// aliases, when one exists: a bare identifier (binding), `arr[i]`
/// (array element), or a bare-identifier call (callee return).
arg_obj_slots: Vec<Option<ObjSlot>>,
```

(b) Add a helper beside `record_object_flow_from_expr` (:293), mirroring its match arms but *returning* the slot:

```rust
/// Object slot aliased by a call argument, when the expression can carry
/// an object reference (same recognized set as `record_object_flow_from_expr`).
fn arg_obj_slot(&mut self, func: &str, arg: &Expression) -> Option<ObjSlot> {
    match arg {
        Expression::Identifier(name) => {
            Some(ObjSlot::Binding(func.to_string(), name.clone()))
        }
        Expression::MemberExpression(member) if member.computed_index.is_some() => {
            match &member.object {
                Expression::Identifier(array) => {
                    Some(ObjSlot::ArrayElem(func.to_string(), array.clone()))
                }
                _ => None,
            }
        }
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(callee) => Some(ObjSlot::Return(callee.clone())),
            _ => None,
        },
        Expression::ParenthesizedExpression(inner) => {
            self.arg_obj_slot(func, &inner.expression)
        }
        _ => None,
    }
}
```

(c) In `visit_call`'s bare-identifier branch (:1031-1055), alongside the existing `arg_nodes`/`arg_array_names` pushes, add `arg_obj_slots.push(self.arg_obj_slot(func, arg));` and thread the vec into the `CallEdge` literal.

(d) In `resolve_calls`'s scalar-arg branch, replace the identifier-only block at :1134-1139 with the general form:

```rust
if let Some(Some(slot)) = edge.arg_obj_slots.get(k) {
    self.obj_flows.push((
        slot.clone(),
        ObjSlot::Binding(edge.callee.clone(), param_name.clone()),
    ));
}
```

(The identifier case previously handled there produces the identical `Binding→Binding` flow; `arg_array_names` keeps its remaining role in the array fixpoint untouched.)

- [ ] **Step 5: Run both tests to verify they pass**

Run: `cargo test -p kali_types repr_infer && cargo test -p kali_cli --test object_call_result_args_runtime`
Expected: PASS.

- [ ] **Step 6: E5506 backstop — locate, gate, verify no false positives**

Locate the member-read fallback that today silently mislowers a property read on a shapeless base: start from `object_shape_of_node` (`emit/object.rs:15`) and the "runtime-array-read arm" ambiguity note it references in `control_flow.rs`. Add a compile-time rejection in the member-property-READ path when ALL of: the base is a bound identifier whose repr is neither `Repr::Object(_)` nor a registered array binding, the property name is not `length`, and the node is not on the compile-time fold lane. Diagnostic (use the E5506 constant and phrasing family from repr_infer.rs:1037):

```rust
self.diagnostics.push(Diagnostic::error(
    e5::FEATURE_UNAVAILABLE as u32,
    format!(
        "reading property '{prop}' of a value with no statically inferred object shape is unavailable in the current phase"
    ),
));
```

Add a test to `object_call_result_args_runtime.rs`: a program whose object flows through a shape the inference still cannot classify must FAIL to compile with E5506 in stderr — use the module-scope nested-literal case, which remains unfixed by design:

```rust
#[test]
fn unclassified_object_shape_member_read_is_rejected_not_miscompiled() {
    let source = write_temp_source(
        "e5506_backstop",
        "const leafA = { left: null, right: null };\nconst leafB = { left: null, right: null };\nconst t = { left: leafA, right: leafB };\nconsole.log(t.left === null);\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run").arg(&source).output().expect("run kali");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("E5506"));
}
```

CAUTION: if that module-scope case turns out to now infer correctly (Task 3(d) may fix it incidentally), assert the correct output (`false`) instead and find/construct another still-unclassifiable shape for the rejection test; if none exists, delete the rejection test and note it in the commit message. Then run the FULL kali_codegen + kali_cli suites. If any pre-existing fixture legitimately reads properties off scalars (e.g. string `.length` variants), narrow the gate rather than weaken the suite — the gate must never fire on today's green surface.

- [ ] **Step 7: Gate and commit**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli && cargo fmt --check`

```bash
git add crates/kali_types crates/kali_codegen crates/kali_cli/tests/object_call_result_args_runtime.rs
git commit -m "fix(types,codegen): call-result arguments seed callee param object shapes; E5506 backstop for shapeless member reads"
```

---

### Task 4: ArenaTable + escape-gate analysis (no codegen behavior change)

**Files:**
- Create: `crates/kali_common/src/arena_table.rs`; register in `crates/kali_common/src/lib.rs`
- Create: `crates/kali_mir/src/analysis/arena_gate.rs`; register in `crates/kali_mir/src/analysis/mod.rs`
- Modify: `crates/kali_codegen/src/ctx.rs:94` (add field), `crates/kali_cli/src/build/compile.rs` (~:453-481, plumbing)
- Test: unit tests inline in `arena_gate.rs` submodule file following the `ownership_analysis_tests/` directory idiom

**Interfaces:**
- Consumes: `MirProgram.functions: Vec<MirFunction>` / `MirBinding { name, kind, ownership, layout, escapes, captured_by }` (`kali_mir/src/binding.rs:16`), the `OwnershipAnalyzer` walk (`analysis/mod.rs:258`, `walk.rs:11`), `LayoutDescriptor` (`analysis/infer.rs:8`).
- Produces (relied on by Tasks 6/7):

```rust
// kali_common::ArenaTable — name-keyed, additive, misses fail closed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArenaTable { /* three BTreeSets, private */ }
impl ArenaTable {
    pub fn set_arena_eligible(&mut self, func: &str);
    pub fn arena_eligible(&self, func: &str) -> bool;      // sites may call __alloc (current arena)
    pub fn set_opens_arena(&mut self, func: &str);
    pub fn opens_arena(&self, func: &str) -> bool;          // function-body arena (Task 7)
    pub fn set_loop_arena(&mut self, func: &str, ordinal: u32);
    pub fn loop_arena(&self, func: &str, ordinal: u32) -> bool; // per-loop arena, pre-order ordinal
}
// kali_mir::analysis::arena_gate
pub fn compute_arena_table(mir: &MirProgram) -> kali_common::ArenaTable;
```

- `CodegenCtx.arena_table: kali_common::ArenaTable` (default empty), set in `compile.rs` beside `ctx.repr_table` (:481).

- [ ] **Step 1: Write ArenaTable + its unit tests (kali_common)**

The struct is three `BTreeSet`s with the setters/getters above (10 minutes; write tests for set/get/miss-is-false in the same file's `#[cfg(test)]` module). Run: `cargo test -p kali_common arena_table` → PASS. Commit checkpoint is at task end.

- [ ] **Step 2: Read the analysis engine before designing against it**

Read in full: `crates/kali_mir/src/analysis/mod.rs` (esp. `BindingState` :23, `UseContext` :16, `finalise_binding` :102, `ScopeState` :144), `walk.rs`, `resolve.rs`, and `crates/kali_mir/src/binding.rs`. Map where you can observe, during or after the walk: (i) per-function allocation sites (object/array literal layouts — `LayoutDescriptor`), (ii) call targets per function, (iii) per-loop scopes with the bindings assigned inside them and where those bindings were declared, (iv) `escapes`/`captured_by`/`returned` verdicts. If the existing walk does not expose loops, add loop-scope tracking to it (a scope kind + a per-function pre-order loop counter) rather than writing a second walker.

- [ ] **Step 3: Write failing unit tests for the gate (the contract, exactly)**

Create the tests first, in a `arena_gate_tests.rs` next to `ownership_analysis_tests.rs`, mirroring its "lower source → analyze → assert" helper chain. Test matrix (one test per line; sources are small function/loop snippets you write inline):

```text
eligible_when_all_sites_only_returned:      factory returning fresh literals        => arena_eligible("factory")
ineligible_on_module_binding_store:         fn stores fresh obj into module let      => !arena_eligible(f)
ineligible_on_capture:                      fresh obj captured by inner closure      => !arena_eligible(f)
ineligible_on_store_into_preexisting:       p.field = fresh (p is a param)           => !arena_eligible(f)
eligible_child_joins_parent_fate:           fresh obj embedded in returned literal   => arena_eligible(f)
uniform_per_function:                       one global site + one local site         => !arena_eligible(f)  (v1 coarseness)
loop_arena_when_no_outflow:                 loop: const tree = mk(); scalar += ...   => loop_arena(f, 0)
loop_veto_on_outer_binding_assignment:      loop assigns obj into outer-declared let => !loop_arena(f, 0)
loop_veto_on_unknown_call:                  loop calls a closure/indirect target     => !loop_arena(f, 0)
loop_whitelist_console_log:                 loop only console.log(fresh-string/obj)  => loop_arena(f, 0)
loop_ordinals_are_preorder:                 two sequential + one nested loop         => ordinals 0,1,2 in source order
opens_arena_only_with_local_sites:          fn with non-escaping local alloc         => opens_arena(f)
no_arena_for_nonallocating_fn:              fn with no reachable alloc (itemCheck)   => !opens_arena(f)
```

Run: `cargo test -p kali_mir arena_gate` → all FAIL (module doesn't exist yet).

- [ ] **Step 4: Implement `compute_arena_table`**

Algorithm (all sets per function name; conservative at every join):

1. **Per-function facts from the walk:** `allocates: bool`; `calls: BTreeSet<String>` (bare-identifier targets; any non-identifier/closure/indirect call sets `has_unknown_call = true`); per allocation site, its fresh-value fate.
2. **Fate lattice** `ScopeLocal < Returned < Global`, computed per function over fresh allocations using the machinery behind `BindingState.returned` / `escaped_via_flow` / `captured_by`: store into module-scope binding, capture, or store into any pre-existing object/array (param, outer binding, or anything not itself a same-function fresh allocation) ⇒ `Global`. Embedding as an object-literal field value joins the child's fate to the parent literal's fate. `return` ⇒ at most `Returned`.
3. `arena_eligible(f)` ⇔ `allocates(f)` and NO site's fate is `Global`. (Uniform per function — one global site poisons all, v1.)
4. **Reaches-alloc** = transitive closure of `allocates` over `calls`, with `has_unknown_call` treated as reaching-alloc AND tainting (unknown callees might allocate and might leak).
5. `loop_arena(f, ordinal)` ⇔ the loop body (including nested non-arena'd scopes) (a) reaches an allocation transitively through only known, `arena_eligible` callees, (b) has NO heap-typed value outflow: no assignment inside the loop to an object/array-layout binding declared outside it, no `return` of a heap-typed value from inside it, and no store of a heap-typed value into any binding/object that outlives the loop; scalar-layout bindings are exempt (treat unknown layout as heap-typed), (c) contains no unknown/closure/indirect calls and no host calls except the non-retaining whitelist `console.log` (plus any stdout-write intrinsics you find registered in the same family).
6. `opens_arena(f)` ⇔ `arena_eligible(f)` and at least one site's fate is `ScopeLocal` (fresh values that die inside `f`). (`main` in binary-trees qualifies; `bottomUpTree` — all `Returned` — does not need one and must not get one.)
7. Loop ordinals: pre-order per function over the same walk order the LIR emitter will use (loop nodes only; nested loops number after their parent's position).

Wire into `compile.rs` after MIR lowering (:453): `ctx.arena_table = kali_mir::analysis::arena_gate::compute_arena_table(&mir_program);` next to where `ctx.repr_table` is set (:481). Add the `arena_table: kali_common::ArenaTable` field (with `Default`) to `CodegenCtx` (`ctx.rs:94`).

- [ ] **Step 5: Run tests; then verify zero behavior change**

Run: `cargo test -p kali_mir arena_gate` → PASS.
Run the full gate: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli && cargo fmt --check` → green (codegen reads nothing from the table yet; this proves pure additivity).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_common crates/kali_mir crates/kali_codegen/src/ctx.rs crates/kali_cli/src/build/compile.rs
git commit -m "feat(mir,common): escape-gate analysis producing name-keyed ArenaTable; plumbed to CodegenCtx (no codegen behavior change)"
```

---

### Task 5: Page pool + synthetics; host strings via exported `__alloc_global` (behavior-preserving)

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (globals :507-515, FunctionPlan inserts :174-182, type table :303-331, locals special-case :437-442, body dispatch :479-490, export :349, coverage filter :533-534, `emit_alloc_body` :1527 replaced)
- Modify: `crates/kali_runtime/src/host/memory.rs:110-133` (`write_bytes_at_heap`)
- Modify: the 4 browser JS glue sites (grep `__heap` in `crates/kali_runtime/src/browser/harness.rs` and `crates/kali_cli/src/**/cmd_build*`)
- Test: existing full suite is the regression harness; add `crates/kali_codegen/src/emit/call_tests/alloc_helper.rs` updates + one new e2e span test in `crates/kali_cli/tests/heap_grow_runtime.rs`

**Interfaces:**
- Consumes: ArenaTable exists but is still unused; Phase-0 geometric-grow logic from the old `emit_alloc_body`.
- Produces: globals g1–g7 (indices/names in File Structure block); synthetics `__alloc(i32)->i32`, `__alloc_global(i32)->i32` **exported as `"__alloc_global"`**, `__page_get(i32)->i32`, `__arena_reset()->()`; `SYNTHETIC_FUNCTIONS: &[&str]` const used by the coverage filter; page layout `{next:i32@0, span_pages:i32@4, payload@8}`. After this task every allocation flows through pages, boot trios are all-zero (open empty program arena), and behavior is output-identical everywhere.

- [ ] **Step 1: Write the failing span e2e test first**

In `heap_grow_runtime.rs` add: a program allocating a >64KB array per loop iteration, checking element correctness (this exercises the multi-page span path that does not exist yet — it must PASS before AND after; it fails only if the new span path is wrong, so run it now to record the green baseline):

```rust
#[test]
fn multi_page_array_allocations_are_correct() {
    let source = write_temp_source(
        "span_arrays",
        r#"function main() {
  let sum = 0;
  for (let round = 0; round < 4; round = round + 1) {
    const a = new Array(20000);
    for (let i = 0; i < 20000; i = i + 1) {
      a[i] = i + round;
    }
    sum = sum + a[19999];
  }
  console.log(sum);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run").arg(&source).output().expect("run kali");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "80002\n"); // 4*19999 + (0+1+2+3)
}
```

Run it; record PASS (baseline). (Arrays this size already work via today's bump — the test's value is catching span-path regressions in Step 5.)

- [ ] **Step 2: Declare globals and synthetics**

In `lower.rs`:
(a) After the existing `__heap` global (:507-515), append seven `i32` mutable globals initialized to 0, in the exact order g1..g7 listed in File Structure. Add `pub const SYNTHETIC_FUNCTIONS: &[&str] = &["__alloc", "__alloc_global", "__page_get", "__arena_reset"];`
(b) Push three more `FunctionPlan`s after `__alloc`'s (:174-182), same inert-placeholder pattern, names/params: `__alloc_global(size)`, `__page_get(pages)`, `__arena_reset()`.
(c) Type table (:303-331): `__alloc`, `__alloc_global`, `__page_get` reuse the existing `(i32)->i32` entry; `__arena_reset` needs `()->()` — add it if absent.
(d) Locals special-case (:437-442): `__alloc`/`__alloc_global` need 2 scratch i32 locals (`cur`, `p`); `__page_get` needs 4 (`head`, `base`, `need`, plus one for the grow loop — match what the moved grow logic consumed); `__arena_reset` needs 2 (`p`, `next`).
(e) Body dispatch (:479-490): route each name to its emitter (Step 3).
(f) Exports (:349): add `export "__alloc_global"` (function export, index via `function_name_to_index`).
(g) Coverage filter (:533-534): replace the `!= "__alloc"` check with `!SYNTHETIC_FUNCTIONS.contains(&name)`.

- [ ] **Step 3: Emit the synthetic bodies**

Replace `emit_alloc_body` with the following family (constants: `PAGE=65536`, `HEADER=8`, `PAYLOAD=PAGE-8`). One parameterized bump emitter serves both alloc flavors:

```rust
/// Bump allocator against one arena trio. Locals: 1=cur, 2=p. Param 0=size.
fn emit_bump_body(func: &mut Function, g_page: u32, g_cur: u32, g_lim: u32, page_get: u32) {
    // fast path: cur = g_cur; if cur+size <= g_lim { g_cur = cur+size; return cur }
    func.instruction(&Instruction::GlobalGet(g_cur));
    func.instruction(&Instruction::LocalTee(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalGet(g_lim));
    func.instruction(&Instruction::I32LeU);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(g_cur));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // span path: size > PAYLOAD
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Const(PAYLOAD));
    func.instruction(&Instruction::I32GtU);
    func.instruction(&Instruction::If(BlockType::Empty));
    //   n = (size + HEADER + PAGE - 1) / PAGE ; p = __page_get(n)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Const(HEADER + PAGE - 1));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::I32Const(PAGE));
    func.instruction(&Instruction::I32DivU);
    func.instruction(&Instruction::Call(page_get));
    func.instruction(&Instruction::LocalSet(2));
    //   p.next = g_page; g_page = p; return p + HEADER  (cursor/limit untouched:
    //   the previous page keeps filling; the span is fully consumed)
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::GlobalGet(g_page));
    func.instruction(&Instruction::I32Store(MemArg { offset: 0, align: 2, memory_index: 0 }));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::GlobalSet(g_page));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32Const(HEADER));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // fresh single page: p = __page_get(1); link; install cursor/limit; return p+HEADER
    func.instruction(&Instruction::I32Const(1));
    func.instruction(&Instruction::Call(page_get));
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::GlobalGet(g_page));
    func.instruction(&Instruction::I32Store(MemArg { offset: 0, align: 2, memory_index: 0 }));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::GlobalSet(g_page));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32Const(HEADER));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(g_cur));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32Const(PAGE));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(g_lim));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32Const(HEADER));
    func.instruction(&Instruction::I32Add);
    // falls through as the function result
}
```

Notes that MUST hold: the empty-arena boot state (all-zero trio) takes the slow path on first allocation (`cur+size <= 0` is false for size>0) and correctly links `next=0`. `__alloc` = `emit_bump_body(f, 1, 2, 3, page_get_idx)`; `__alloc_global` = `emit_bump_body(f, 4, 5, 6, page_get_idx)`.

`__page_get` (param 0 = n pages; returns page base). Free-list pop with span-split for n==1; frontier + geometric grow otherwise. Move the Phase-0 grow logic (old `emit_alloc_body`'s `memory.grow` block, including the `-1 → Unreachable` check and geometric `max(deficit, cur_pages)` sizing) into the frontier path here, growing to cover `__heap + n*PAGE`:

```text
if n == 1 && g7 != 0:
    head = g7
    if head.span == 1: g7 = head.next; return head
    rem = head + PAGE
    rem.next = head.next; rem.span = head.span - 1; g7 = rem
    head.span = 1; return head
base = g0; need = n * PAGE; new_top = base + need
<Phase-0 grow-to-new_top logic; unreachable on grow == -1>
g0 = new_top
base.next = 0; base.span = n
return base
```

Emit that as instructions following the same style as `emit_bump_body` (wrap the free-list branch in `If`, the inner span==1 check in a nested `If` ending in `Return`).

`__arena_reset` (no params): walk the current-arena page list into the free list, zero the trio:

```text
p = g1
block:
  loop:
    if p == 0: br 1
    next = p.next
    p.next = g7; g7 = p
    p = next
    br 0
g1 = 0; g2 = 0; g3 = 0
```

Frontier init: `heap_base` (g0 init, :506) is already 8-aligned; pages are 64KB *chunks*, not 64KB-*aligned* — no alignment change needed.

- [ ] **Step 4: Move the host string bump onto `__alloc_global`**

In `crates/kali_runtime/src/host/memory.rs:110-133`, change the allocate-and-write helper: resolve `caller.get_export("__alloc_global")` as a `TypedFunc<i32, i32>`, call it with the byte length rounded up to a multiple of 8, and write the bytes at the returned pointer. Keep the old direct-`__heap`-bump code as a fallback branch when the export is absent (stale cached modules built pre-Task-5), with a comment naming this task. Then update the browser JS glue: grep `__heap` across `crates/kali_runtime/src/browser/harness.rs` and the `cmd_build` bundle glue; wherever the JS implementation bumps `__heap` to place a runtime string, switch to `instance.exports.__alloc_global(len)` with the same absent-export fallback. The import *lists* must not change — assert this by diffing the import-name arrays before/after.

- [ ] **Step 5: Run everything (this task's test IS the full suite)**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli && cargo fmt --check`
Expected: green — specifically `clbg_nbody_runtime`, `clbg_spectral_norm_runtime`, `clbg_fannkuch_runtime`, `clbg_mandelbrot_runtime` (canonical outputs incl. 5011-byte PBM), all of `heap_grow_runtime` (growth now via `__page_get`), the Step-1 span test, and the template-literal/browser bundle tests (host strings now via export). The codegen unit tests that pinned the old `__alloc` body (`emit/call_tests/alloc_helper.rs`) will need their expected instruction sequences updated — update them to the new body, do not weaken them.
Run browser smoke: `mise run browser-smoke` → green.

- [ ] **Step 6: Commit**

```bash
git add -A crates/
git commit -m "feat(codegen,runtime): page-pool allocator — 4 synthetics, 7 new globals, host strings via exported __alloc_global (behavior-preserving)"
```

---

### Task 6: Loop arenas — emission hooks, scope-exit unwinding, reclamation proof

**Files:**
- Modify: `crates/kali_codegen/src/emitter.rs` (arena_frames stack beside `loop_frames` :82; locals provisioning), `crates/kali_codegen/src/emit/control_flow.rs` (`emit_loop` :108-207, break/continue :10-60, `emit_return` :62-93), `crates/kali_codegen/src/lower.rs` (fall-through end :491), allocation-site callee choice in `emit/object.rs:74-75` + `emit/call.rs:2390`
- Test: `crates/kali_cli/tests/arena_reclamation_runtime.rs` (create)

**Interfaces:**
- Consumes: `ArenaTable::{arena_eligible, loop_arena}` (Task 4), synthetics + globals (Task 5), `loop_frames: Vec<LoopFrame>` and `control_frame_depth` (emitter.rs).
- Produces: `struct ArenaFrame { saved_page_local: u32, saved_cursor_local: u32, saved_limit_local: u32, loop_frame_index: Option<usize> }` and `arena_frames: Vec<ArenaFrame>` on `FunctionEmitter`; helper `emit_arena_release(&mut self, function: &mut Function, frame: &ArenaFrame)` = `Call(__arena_reset)` + three `LocalGet(saved_*)`→`GlobalSet(1..3)`. Allocation sites call `__alloc` iff `arena_eligible(current_fn)`, else `__alloc_global`.

- [ ] **Step 1: Write the failing reclamation-proof test**

Create `arena_reclamation_runtime.rs` (helpers as before; `write_temp_policy_json` from Task 1 — policies here set BOTH `maxMemoryMB` and `maxCpuTimeMs`):

```rust
#[test]
fn per_iteration_loop_allocations_are_reclaimed() {
    // 400 iterations x ~256KB of fresh objects = ~100MB cumulative, under an
    // 8MB memory cap: passes only if iteration arenas recycle pages.
    let source = write_temp_source(
        "reclaim_loop",
        r#"function mkRow() {
  return { a: 1, b: 2, c: 3, d: 4 };
}
function main() {
  let sum = 0;
  for (let round = 0; round < 400; round = round + 1) {
    let last = 0;
    for (let i = 0; i < 8000; i = i + 1) {
      const row = mkRow();
      last = row.a + row.d;
    }
    sum = sum + last;
  }
  console.log(sum);
}
main();
"#,
    );
    // Task 1's policy JSON verbatim, with these two resource values instead:
    //   "maxMemoryMB": 8, "maxCpuTimeMs": 600000
    let policy = write_temp_policy_json("mem8", MEM8_POLICY_JSON);
    let output = std::process::Command::new(kali_bin())
        .arg("run").arg("--sandbox").arg(&policy).arg(&source)
        .output().expect("run kali");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2000\n"); // 400 * (1+4)
}
```

Run: `cargo test -p kali_cli --test arena_reclamation_runtime` → FAIL today (unreachable trap: 100MB cumulative > 8MB cap, no reclamation). Verify the failure is the allocation trap, not fuel (raise `maxCpuTimeMs` in the policy if E4003 appears — Task 1's diagnostic now makes them distinguishable).

- [ ] **Step 2: Provision saved-state locals and the arena_frames stack**

In `collect_function_locals` (`lower.rs:972`): for the function being planned, count its arena'd loops (`ctx.arena_table.loop_arena(name, ordinal)` over the same pre-order loop numbering — implement one shared `loop_preorder_ordinals(nodes, body)` helper in codegen and use it both here and in emission so the two cannot diverge) and append 3 synthetic i32 locals per arena'd loop, named `"__arena_save_page#{k}"` etc. so `FunctionEmitter::new`'s name→slot mapping picks them up. Add `arena_frames: Vec<ArenaFrame>` (empty) to `FunctionEmitter`.

- [ ] **Step 3: Hook `emit_loop`**

In `emit_loop` (`control_flow.rs:108`), maintain a per-function loop counter (pre-order — increment on entry, before descending). When `loop_arena(fn, ordinal)`:
- **Open** (before the `block`/`loop` opcodes, after for-init at :148): `GlobalGet(1)→LocalSet(saved_page)`, same for cursor/limit; then `I32Const(0)→GlobalSet(1..3)` ×3. Push the `ArenaFrame` (with `loop_frame_index = Some(index of the LoopFrame pushed at :159)`).
- **Per-iteration reset**: immediately after the `Instruction::Loop` opcode (:158) — i.e. the top of every iteration, before condition/body — emit `Call(__arena_reset_index)`. (Top placement makes `continue` correct with zero unwinding: every re-entry passes the reset; the outflow rule guarantees nothing live spans iterations.)
- **Release** (after the loop's `End`s at :203-207): `emit_arena_release(frame)`, pop `arena_frames`.

- [ ] **Step 4: Unwind on break and return**

- `break` (`control_flow.rs:48-56` before `Br(depth)`): release every `ArenaFrame` whose `loop_frame_index` is at or inside the branch-target loop frame (unlabeled break targets the innermost loop: release the top arena frame iff it belongs to that loop). Do NOT pop `arena_frames` permanently — the release code executes on the break path only; the loop's normal-exit release at Step 3 still emits. Emit releases, then `Br`.
- `continue`: no arena code (top-of-iteration reset covers it; unlabeled continue cannot cross another loop's arena).
- `emit_return` (`control_flow.rs:62`, before `Instruction::Return` at :93): release ALL live `arena_frames`, newest→oldest (emit-only, no pop).
- Fall-through function end (`lower.rs:491`): nothing needed for loop arenas (loops are closed constructs) — this hook matters in Task 7.

- [ ] **Step 5: Route allocation sites through the gate**

In `emit/object.rs:74-75` and `emit/call.rs:2390`, replace `self.alloc_fn_index()` with a `self.alloc_callee_index()` helper on `FunctionEmitter`: `functions["__alloc"]` when `ctx.arena_table.arena_eligible(self.function_name)`, else `functions["__alloc_global"]`.

- [ ] **Step 6: Extend the test file with the behavioral matrix, run everything**

Add these tests (same helper idioms; each asserts exact stdout):

```text
mini_binary_trees_output_exact:      the full binary-trees program shape at n=8, expected
                                     output computed from the CLBG formula (stretch 9 =>
                                     1023; levels 4,6,8; longLived 8 => 511) — pins the
                                     long-lived-before-loop + per-iteration-tree mix.
break_inside_arena_loop_is_sound:    loop allocs per iteration, breaks at i==50, prints a
                                     scalar accumulated before break + an object built
                                     AFTER the loop (proves state restore on break path).
return_from_arena_loop_is_sound:     function returns a SCALAR from inside an arena'd
                                     loop; caller continues allocating correctly.
module_global_store_fails_closed:    fn called in a loop stores fresh obj to module let;
                                     values correct after.
nested_arena_loops_correct:          outer arena'd loop + inner arena'd loop, exact sums
                                     (concrete program below).
spans_inside_arena_loop:             per-iteration `new Array(20000)` (~160KB span) x 200
                                     iterations under maxMemoryMB: 8 — passes only if
                                     spans return to the free list on reset.
```

The two soundness-critical ones, in full (the rest follow the same shape):

```rust
#[test]
fn store_to_outer_fails_closed() {
    // The fresh object escapes the iteration into an outer-declared binding:
    // the gate must veto this loop's arena; the value must survive the loop.
    let source = write_temp_source(
        "store_outer",
        r#"function mk(v) {
  return { a: v, b: v + 1 };
}
function main() {
  let last = mk(0);
  for (let i = 1; i <= 100; i = i + 1) {
    last = mk(i);
  }
  console.log(last.a + last.b);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run").arg(&source).output().expect("run kali");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "201\n"); // 100 + 101
}

#[test]
fn nested_arena_loops_correct() {
    let source = write_temp_source(
        "nested_loops",
        r#"function mk(v) {
  return { a: v, b: 2 * v };
}
function main() {
  let total = 0;
  for (let outer = 0; outer < 50; outer = outer + 1) {
    let rowSum = 0;
    for (let inner = 0; inner < 200; inner = inner + 1) {
      const cell = mk(inner);
      rowSum = rowSum + cell.a + cell.b;
    }
    total = total + rowSum;
  }
  console.log(total);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run").arg(&source).output().expect("run kali");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    // rowSum = 3 * (0..199 sum) = 3 * 19900 = 59700; total = 50 * 59700
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2985000\n");
}
```

Run: `cargo test -p kali_cli --test arena_reclamation_runtime` → all PASS (Step 1's test flips to green here).
Full gate + CLBG outputs + `mise run browser-smoke` → green.

- [ ] **Step 7: Commit**

```bash
git add -A crates/
git commit -m "feat(codegen): per-loop iteration arenas — open/reset/release hooks, break/return unwinding, escape-gated allocation routing"
```

---

### Task 7: Function-body arenas

**Files:**
- Modify: `crates/kali_codegen/src/emitter.rs` (`emit_function_body` :219 — prologue), `crates/kali_codegen/src/emit/control_flow.rs` (`emit_return` already unwinds all frames — verify the function frame is included), `crates/kali_codegen/src/lower.rs` (:489-491 — fall-through release before the trailing `End`), `collect_function_locals` (3 more locals when `opens_arena`)
- Test: `crates/kali_cli/tests/arena_reclamation_runtime.rs` (extend)

**Interfaces:**
- Consumes: `ArenaTable::opens_arena`, `ArenaFrame` with `loop_frame_index: None` marking the function-level frame.
- Produces: functions with `opens_arena` save/zero the trio on entry and release on every exit path (explicit returns via the Task-6 unwinding; fall-through via a release emitted before the final `End`).

- [ ] **Step 1: Write the failing tests**

```text
function_scratch_is_reclaimed:  a function builds ~64KB of scratch objects, returns a
    scalar; called 2000 times in a (non-arena'd — make the call target unknown-free but
    store results to an outer array so the LOOP is vetoed) loop under maxMemoryMB: 8.
    ~128MB cumulative; passes only if per-call function arenas reclaim.
recursive_function_arena_sound: a recursive function with opens_arena (local scratch per
    frame, scalar return), depth 500 — pins that saved-trio locals nest correctly
    through recursion.
factory_functions_get_no_arena: bottomUpTree-shaped factory (all sites Returned) called
    from an arena'd loop — exact output; pins that opens_arena is NOT set for factories
    (their allocations must land in the caller's arena, not a per-call arena that would
    dangle on return).
```

Write them concretely in the Task-6 style with exact expected stdout; run → the first FAILS (memory cap), the others pass pre-change or fail-to-compile — record the baseline.

- [ ] **Step 2: Implement**

- `collect_function_locals`: +3 i32 saved-state locals when `opens_arena(name)`.
- `emit_function_body` (emitter.rs:219): if `opens_arena`, emit save-trio-to-locals + zero-trio BEFORE `self.emit_node(body)`, pushing an `ArenaFrame { loop_frame_index: None, .. }` as the bottom of `arena_frames`.
- Fall-through: in `lower.rs` just before the trailing `Instruction::End` (:491), if the function opened an arena, emit the release (reset + restore). Wire this through a small `FunctionEmitter::emit_function_epilogue` so the logic lives beside the prologue.
- `emit_return`'s Task-6 unwinding already walks ALL frames including the function frame — verify by reading, don't assume.

- [ ] **Step 3: Run tests + full gate + browser smoke; commit**

Run: `cargo test -p kali_cli --test arena_reclamation_runtime && cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli && cargo fmt --check && mise run browser-smoke`

```bash
git add -A crates/
git commit -m "feat(codegen): per-call function-body arenas with prologue open and all-exit-path release"
```

---

### Task 8: Binary-trees fixture — canonical N=21 acceptance

**Files:**
- Create: `crates/kali_cli/tests/fixtures/benchmarks/binary-trees-benchmark-v1.ts`, `.json`, `.policy.json`
- Create: `crates/kali_cli/tests/clbg_binary_trees_runtime.rs`
- Modify: `specs/19-feature-maturity.md` (:237 paragraph area — new evidence row)

**Interfaces:**
- Consumes: everything above.
- Produces: the fifth vendored CLBG fixture; canonical N=21 byte-for-byte via `kali run --sandbox`.

- [ ] **Step 1: Write the fixture source**

`binary-trees-benchmark-v1.ts` — mirror the annotation style of `nbody-benchmark-v1.ts` (read it first); the program (canonical CLBG shape, kali-normalized idioms only):

```ts
function bottomUpTree(depth: number) {
  if (depth > 0) {
    return { left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) };
  }
  return { left: null, right: null };
}

function itemCheck(t): number {
  if (t.left === null) {
    return 1;
  }
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}

function main(): void {
  const n = 21;
  const minDepth = 4;
  const maxDepth = n;
  const stretchDepth = maxDepth + 1;
  console.log(`stretch tree of depth ${stretchDepth}\t check: ${itemCheck(bottomUpTree(stretchDepth))}`);
  const longLivedTree = bottomUpTree(maxDepth);
  for (let depth = minDepth; depth <= maxDepth; depth = depth + 2) {
    const iterations = 1 << (maxDepth - depth + minDepth);
    let check = 0;
    for (let i = 1; i <= iterations; i = i + 1) {
      check = check + itemCheck(bottomUpTree(depth));
    }
    console.log(`${iterations}\t trees of depth ${depth}\t check: ${check}`);
  }
  console.log(`long lived tree of depth ${maxDepth}\t check: ${itemCheck(longLivedTree)}`);
}

main();
```

If the `t` parameter's missing annotation trips the TS surface, mirror however nbody annotates object params; do NOT add intermediate bindings — the direct call-result args are the point (P0b).

- [ ] **Step 2: Policy + metadata**

`.policy.json`: copy `mandelbrot-benchmark-v1.policy.json`, set `"maxCpuTimeMs": 60000000` (≈60B fuel; N=21 is estimated ~30B — headroom, trimmed in Step 5) and `"maxMemoryMB": null`.
`.json`: mirror nbody's shape (`benchmark: "binary-trees"`, `version: 1`, `sourceFile`, `buildModes: ["--fast", "--release", "--release-advanced"]`), computing `sourceSha256` with:

```bash
sha256sum crates/kali_cli/tests/fixtures/benchmarks/binary-trees-benchmark-v1.ts
```

- [ ] **Step 3: Write the runtime test (failing until run end-to-end)**

`clbg_binary_trees_runtime.rs`, mirroring `clbg_mandelbrot_runtime.rs`'s policy-passing pattern:

- `binary_trees_small_n_matches_canonical_output` (always-on): writes a temp n=10 variant of the fixture logic (temp-source helper) and asserts the exact 6-line output (compute the expected lines: stretch 11 → 4095; 1024/d4 → 31744; 256/d6 → 32512; 64/d8 → 32704; 16/d10 → 32752; long lived 10 → 2047; **real tab characters** — write them as `\t` in the Rust expected-string literal).
- `binary_trees_canonical_n21_matches_output` marked `#[ignore = "multi-minute canonical run; invoke via: cargo test -p kali_cli --release --test clbg_binary_trees_runtime -- --ignored"]`: runs the vendored fixture with the policy; asserts the full 11-line canonical output byte-for-byte. Expected `check` values for N=21 (CLBG-canonical; verify the first three against a quick n=10/n=12 run before trusting the pattern): stretch depth 22 → `8388607`; depth-d line → `iterations * (2^(d+1) - 1)`; long lived 21 → `4194303`. Compute each line's number in the test as Rust constants with the formula, not hand-typed literals.
- `binary_trees_metadata_is_consistent`: copy nbody's metadata test, adjusted.

- [ ] **Step 4: Run the acceptance**

```bash
cargo test -p kali_cli --release --test clbg_binary_trees_runtime            # small-N + metadata
time cargo test -p kali_cli --release --test clbg_binary_trees_runtime -- --ignored   # N=21
```

Expected: all pass; note the N=21 wall-clock. If N=21 fails on memory (unreachable trap): the arena gate vetoed a load-bearing loop — debug via the Task 4 unit tests before touching codegen. If it fails on fuel (E4003 — now legible): raise the policy value.

- [ ] **Step 5: Right-size the policy and CI placement**

Trim `maxCpuTimeMs` to ~2× the observed fuel need (observe by bisecting or just leave 2× the failing threshold if measured). If observed wall-clock < 30s, remove `#[ignore]` (the spec's empirical rule); otherwise keep it and add a named mise task `clbg-binary-trees` (mirror how `browser-smoke` is registered in `mise.toml`) running the ignored test.

- [ ] **Step 6: Feature-maturity row**

In `specs/19-feature-maturity.md`, extend the CLBG evidence paragraph (:237) in the established voice: binary-trees is the fifth vendored fixture, the first exercising **heap-object reclamation** (per-loop/per-function arena reset over the page pool — no GC), canonical N=21 via a scoped policy raising `maxCpuTimeMs` (mandelbrot precedent), execution-correctness coverage, not a throughput claim.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/tests specs/19-feature-maturity.md
git commit -m "feat(cli): binary-trees CLBG fixture — canonical N=21 via scoped fuel policy, arena reclamation end-to-end"
```

---

### Task 9: Full regression sweep, perf sanity, integration

**Files:** none new — verification + PR.

- [ ] **Step 1: Full gate, fmt, browser smoke**

```bash
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
cargo fmt --check
mise run browser-smoke
```

Expected: all green.

- [ ] **Step 2: Perf sanity (the bump fast path must not have regressed)**

```bash
cargo build --release -p kali_cli
time target/release/kali run crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.ts
time target/release/kali run --sandbox crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.policy.json crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts
```

Run each 3×; compare against the same commands on `main` (checkout `main` in a scratch worktree: `git worktree add /tmp/kali-main main`). More than ~10% regression = investigate before merging (the fast path is shape-identical by design; a regression means an arena was opened where the gate should have declined, or the bump body grew).

- [ ] **Step 3: Self-review the diff, then PR**

Re-read the full branch diff against the spec's Global Constraints (GC-less: no trace/copy machinery crept in; fail-closed defaults; `__heap` still g0/exported). Then follow the repo integration convention: push the branch, open a PR with a body summarizing P0a re-diagnosis, P0b, the arena architecture, and the fixture; merge after CI is green (per `kali-integration-convention`: the agent pushes and merges).

```bash
git push -u origin binary-trees-phase1-arenas
gh pr create --title "binary-trees Phase 1: dynamic current-arena reclamation" --body-file <PR body>
```

---

## Plan Self-Review (performed at write time)

- **Spec coverage:** P0a (T1/T2), P0b (T3), ArenaTable + escape gate + ordinal keying (T4), pool/synthetics/host-strings/boot (T5), loop arenas + adversarial + spans + reclamation proof (T6), function arenas + recursion (T7), fixture/policy/maturity/N=21 acceptance + test tiers (T8), regression + perf sanity + browser smoke (T9). Loop-ordinal stability is pinned structurally (T6 Step 2's single shared `loop_preorder_ordinals` helper used by both locals-provisioning and emission) plus T4's pre-order unit test.
- **Known open risk carried into execution:** the arena-gate walk (T4 Step 2/4) is specified as an algorithm against read-confirmed structures, not line-level code — the implementer must read the analysis engine first; the unit-test matrix is the contract. The E5506 backstop (T3 Step 6) may need narrowing against the live suite; the test names the fallback procedure.
- **Type consistency:** `ArenaTable` accessor names match between T4 (definition) and T6/T7 (uses); `ArenaFrame` fields defined in T6 and reused in T7; synthetic names/global indices fixed once in File Structure and referenced verbatim throughout.
