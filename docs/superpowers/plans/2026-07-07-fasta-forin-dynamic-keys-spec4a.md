# fasta `for..in` + fixed-shape dynamic string-keyed access (Spec 4a) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compile fasta's two `for..in` sites (`makeCumulative`, `selectRandom`) by adding a fixed-shape `for..in` enumeration lane plus computed string-keyed get/set over an object whose shape is known at compile time.

**Architecture:** `for..in` gets a real counted-loop lowering over an ordinal `0..N-1` (`N` = the iterated object's known shape field count). The loop key is a runtime ordinal, repr-tracked as "for..in-key-of-shape-S"; computed access `obj[c]` lowers to an object field slot at `base + ord*8`; and when the key is used as a value it materializes an interned field-name string handle from a per-shape handle table. Everything unprovable fails closed with E5506.

**Tech Stack:** Rust; the Kali compiler pipeline (`kali_lexer` → `kali_parser` → `kali_hir` → `kali_mir` → `kali_lir` → `kali_codegen`), `kali_types` for repr inference + gates, `wasm-encoder` for emission, `wasmprinter`/`wasmparser` for codegen tests.

## Global Constraints

Copied verbatim from the spec (`docs/superpowers/specs/2026-07-07-fasta-forin-dynamic-keys-spec4a-design.md`). Every task's requirements implicitly include this section.

- **Fail-closed, never fail-open:** any receiver / key / target the analysis cannot prove safe rejects with a diagnostic — `e5::FEATURE_UNAVAILABLE = 5506` (`crates/kali_error/src/_error_codes.rs`), surfaced either through a resolver gate (`Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, …)`) or through `repr_infer::add_shape_conflict` → `ReprTable::shape_conflicts` → `crates/kali_cli/src/build/compile.rs:650-655`.
- **Both-walks ordinal safety:** `for..in` takes NO arena and gets NO loop ordinal. Do NOT add `"for-in"` to the codegen loop whitelist (`loop_preorder_ordinals_walk`, `crates/kali_codegen/src/lower.rs:1283`); leave the `kali_mir` walk's `ForInStmt` arm (`crates/kali_mir/src/analysis/walk.rs:304`) skipping `arena_enter_loop`. Both sides stay ordinal-less together — assigning an ordinal on only one side desyncs every real loop lexically after the `for..in`.
- **Both-sides oracle mirroring:** any new expression shape (computed key read/write with a key-provenance index, the `for..in` key as a string value) gets arms on BOTH the codegen recognizers AND the `kali_types` predicates in the SAME change, or it fails open. The four predicates to keep in sync live in `crates/kali_types/src/resolve/expression.rs`: `expression_is_string_typed` (:69), `operand_repr_is_string` (:362), `expression_is_length_fold_receiver` (:571), `expression_is_runtime_string_value` (:635).
- **Uniform-field-repr requirement:** dynamic computed access `obj[c]` (runtime ordinal index) is admitted only when ALL of `obj`'s shape fields share one repr (fasta: all `Repr::F64`). A runtime index cannot select a per-field type; mixed-repr shapes fail closed.
- **Object layout invariant:** fixed-shape objects are HEADERLESS — field `j` at byte `base + j*8`, load/store `MemArg { offset: 0, align: 3, memory_index: 0 }` with the address computed as `base + j*8`. (Contrast arrays, which have an 8-byte length header at offset 0 and elements at MemArg offset 8. Do NOT reuse the array element-address path's header offset for objects.)
- **Strings never dangle:** any runtime string allocation goes through interned data segments (`StringPool::intern`) / `__alloc_global`, NEVER the resettable `__alloc`. The per-shape handle table is allocated ONCE before the loop (never per-iteration) and holds interned handles.
- **No new host imports:** the 4 hand-mirrored `kali:rt` JS import lists (`crates/kali_runtime/src/browser/harness.rs`; `crates/kali_cli/src/bin/cmd_build.rs`) stay byte-identical — verify with `git diff` at the end.
- **Base-behavior invariants:** all CLBG fixtures byte-identical — nbody, fannkuch-redux, spectral-norm, mandelbrot, binary-trees. Static object-fold and numeric `for`/`while` loops unchanged. The both-walks fix leaves every existing loop's arena assignment identical (binary-trees is the guardrail).
- **Handle encoding:** `STRING_HANDLE_TAG (0x8000_0000_0000_0000) | (offset << 32) | len` where `len` is a BYTE count — via `encode_string_handle(offset, len)` (`crates/kali_codegen/src/lower.rs:2571`). Intern a compile-time string with `self.strings.intern(text) -> (offset, len)` (`crates/kali_codegen/src/ctx.rs:200`).
- **Conventions:** conventional-commit messages; commit after every task. The synthetic top-level function name is `"_start"` in repr_infer / resolver / codegen.
- **Per-task gate:** `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir` plus `cargo clippy -p <touched crates> -- -D warnings`. Final task adds `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`.
- **Integration:** push a PR + self-merge when CI is green, per the `kali-integration-convention` memory (`gh` authed as `rahulmutt`; run `gh auth setup-git` if git can't read credentials).

## File Structure

- `crates/kali_hir/src/lowering/statement.rs` (~:191) — give `ForInStmt` a `"for-in"` text discriminator (Task 1).
- `crates/kali_codegen/src/emit/control_flow.rs` — add `Some("for-in")` dispatch arm (~:683) + new `emit_for_in` (Task 1); dynamic object field read/write helpers + null-sentinel truthiness (Tasks 3/4).
- `crates/kali_codegen/src/emit/object.rs` — new `emit_object_field_read_dynamic` / `emit_object_field_write_dynamic` (Task 3); `emit_key_handle_table` + string materialization (Task 5).
- `crates/kali_types/src/scope.rs` (~:62) — new `for_in_key_bindings` registry (Task 2).
- `crates/kali_types/src/resolve/mod.rs` (~:407) — populate the registry in the `ForInStatement` arm (Task 2).
- `crates/kali_types/src/resolve/expression.rs` — consume predicate + the four mirrored oracles + computed-key gates (Tasks 2/3/4/5/6).
- `crates/kali_types/src/repr_infer.rs` (~:653) — seed the key node + key-string axis; uniform-repr shape check (Tasks 2/3).
- `crates/kali_cli/tests/runtime_forin.rs` — NEW e2e suite (all tasks).
- `crates/kali_types/src/repr_infer_tests.rs`, `crates/kali_mir/src/analysis/arena_gate_tests.rs`, `crates/kali_codegen/src/emit/*_tests.rs` — unit pins.

---

### Task 1: `for..in` identity + counted-loop skeleton (no key use yet)

Give `for..in` a real counted-loop lowering over `0..N-1` where `N` is the iterated object's known shape field count. The loop body executes `N` times; the key variable is bound to the ordinal (an `i64` local) but not yet usable as an index or string. Prove the desync-safety invariant.

**Files:**
- Modify: `crates/kali_hir/src/lowering/statement.rs:191-204` (the `ForInStatement` lowering)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs:683-697` (the `Branch` text dispatch) — add a `Some("for-in")` arm
- Create (function): `emit_for_in` in `crates/kali_codegen/src/emit/control_flow.rs`
- Test: `crates/kali_cli/tests/runtime_forin.rs` (new file)
- Test: `crates/kali_mir/src/analysis/arena_gate_tests.rs` (desync pin)

**Interfaces:**
- Consumes: `object_shape_of_node(&self, id: LirNodeId) -> Option<kali_common::ShapeId>` (`emit/object.rs:14`); `self.repr_table.shape_fields(shape) -> &[(String, Repr)]` (`kali_common/src/repr.rs:249`); the loop-frame scaffolding pattern in `emit_loop` (`control_flow.rs:158-348`); `self.emit_node(function, id, want_value)` (`control_flow.rs:489`).
- Produces: `fn emit_for_in(&mut self, function: &mut Function, id: LirNodeId, node: &LirNode) -> EmittedValue`. The LIR `for-in` node's `children` are `[left, right, body]` (left = key binding, right = iterated object, body = loop body).

- [ ] **Step 1: Write the failing e2e test**

Create `crates/kali_cli/tests/runtime_forin.rs` with the standard harness header (mirror `runtime_join.rs:1-24`, slug `"kali-forin"`) then this test:

```rust
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-forin-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

#[test]
fn for_in_over_fixed_shape_object_iterates_once_per_field() {
    // The body runs once per own field of the statically-shaped object.
    // Key not used yet; only the iteration count is observable.
    let out = run_source(
        "const table = { a: 1, c: 2, g: 3 };\nlet count = 0;\nfor (var c in table) {\n  count = count + 1;\n}\nconsole.log(count);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test runtime_forin for_in_over_fixed_shape_object_iterates_once_per_field`
Expected: FAIL — currently `for..in` mis-lowers to an `if` (`control_flow.rs:697` wildcard → `emit_branch`), so `count` prints `0` (body runs 0 or 1 times, not 3).

- [ ] **Step 3: Give `for..in` a `"for-in"` text discriminator in HIR**

In `crates/kali_hir/src/lowering/statement.rs`, the `ForInStatement` lowering (~:191) currently allocates the node with `alloc` (no text). Change it to `alloc_text` with `"for-in"`, mirroring the `ForOfStmt` treatment at statement.rs:212-216. Keep the children order `[left, right, body]` unchanged. Example (adapt to the exact surrounding code):

```rust
// crates/kali_hir/src/lowering/statement.rs — ForInStatement arm
let node = self.builder.alloc_text(HirNodeKind::ForInStmt, "for-in");
// ... push left, right, body children exactly as before ...
```

The text flows verbatim through MIR (`crates/kali_mir/src/lower.rs:45-48`) and LIR (`crates/kali_lir/src/lower.rs:37-40`); no changes needed there. Do NOT change `map_kind` (the node still collapses to `ControlFlow`/`Branch` by kind — recognition is now by text).

- [ ] **Step 4: Add the `Some("for-in")` dispatch arm + `emit_for_in`**

In `crates/kali_codegen/src/emit/control_flow.rs`, the `Branch` arm's inner `match node.text.as_deref()` (~:683-697) currently routes unknown text to `emit_branch` via the wildcard. Add, before the wildcard:

```rust
Some("for-in") => self.emit_for_in(function, id, &node),
```

Then add `emit_for_in`. It reads `N` from the iterated object's shape and emits a counted loop over the ordinal. The Block/Loop frame scaffolding mirrors `emit_loop` (`control_flow.rs:250-327`) but with NO arena logic (this loop is never granted an arena). Concrete implementation:

```rust
/// Lower `for (KEY in OBJ)` over a compile-time-known fixed-shape object.
/// children = [left(key binding), right(object), body]. The key is bound to
/// an ordinal 0..N-1 (N = OBJ's shape field count). No arena: this loop
/// allocates nothing per iteration.
pub(crate) fn emit_for_in(
    &mut self,
    function: &mut Function,
    _id: LirNodeId,
    node: &LirNode,
) -> EmittedValue {
    let left_id = node.children[0];
    let right_id = node.children[1];
    let body_id = node.children[2];

    // N from the object's shape. Fail closed if the object has no known shape.
    let shape = match self.object_shape_of_node(right_id) {
        Some(s) => s,
        None => {
            self.push_feature_unavailable_diagnostic(
                "for..in is only supported over an object with a compile-time-known shape",
            );
            return EmittedValue::none();
        }
    };
    let n = self.repr_table.shape_fields(shape).len() as i64;

    // Resolve the key variable's local slot (the same slot identifier reads
    // of the key resolve to). Extract the key name from the `left` child and
    // look up its local index via the emitter's binding->local map (the same
    // lookup identifier LocalGet uses). See emitter.rs for the accessor.
    let key_name = self.for_in_key_name(left_id); // helper: read left child's binding name
    let key_local = self.local_index_for(&key_name); // mirror identifier-load local lookup
    let ord_local = self.reserve_scratch_i64_local(); // fresh i64 scratch

    // preheader: ord = 0
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(ord_local));

    // block (break target) { loop (continue target) { ... } }
    function.instruction(&Instruction::Block(BlockType::Empty));
    function.instruction(&Instruction::Loop(BlockType::Empty));

    // break when ord >= N
    function.instruction(&Instruction::LocalGet(ord_local));
    function.instruction(&Instruction::I64Const(n));
    function.instruction(&Instruction::I64GeS);      // i32: 1 if ord >= N
    function.instruction(&Instruction::BrIf(1));      // -> break out of block

    // key = ord
    function.instruction(&Instruction::LocalGet(ord_local));
    function.instruction(&Instruction::LocalSet(key_local));

    // body
    self.emit_node(function, body_id, false);

    // ord = ord + 1
    function.instruction(&Instruction::LocalGet(ord_local));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(ord_local));

    function.instruction(&Instruction::Br(0));        // back to loop top
    function.instruction(&Instruction::End);          // end loop
    function.instruction(&Instruction::End);          // end block
    EmittedValue::none()
}
```

Notes for the implementer:
- `for_in_key_name`, `local_index_for`, `reserve_scratch_i64_local`, `push_feature_unavailable_diagnostic`, and `EmittedValue::none()` are the emitter's existing idioms under different names — resolve each by reading `crates/kali_codegen/src/emitter.rs` and how `emit_node`'s `Identifier` load finds a local slot and how `emit_object_field_read` (`emit/object.rs:124`) pushes the E5506 diagnostic (`FEATURE_UNAVAILABLE`). Reuse those exact APIs; do not invent new ones.
- If the key variable has no reserved local (verify against `collect_function_locals`), reserve one the same way `emit_object_allocation` reserves `scratch = self.locals.len() as u32` (`emit/object.rs:68`) and record it in the binding→local map so body reads of the key resolve to it.
- `Instruction`, `BlockType`, `MemArg` are re-exported via `use crate::*;` (see `emit/object.rs:6`).

- [ ] **Step 5: Run the e2e test to verify it passes**

Run: `cargo test -p kali_cli --test runtime_forin for_in_over_fixed_shape_object_iterates_once_per_field`
Expected: PASS — prints `3\n`.

- [ ] **Step 6: Write the desync-safety MIR pin**

Add to `crates/kali_mir/src/analysis/arena_gate_tests.rs` (mirror `loop_ordinals_are_preorder` at :147-170). This pins that a real loop lexically AFTER a `for..in` keeps its expected arena ordinal — i.e. the `for..in` consumed no ordinal:

```rust
#[test]
fn for_in_consumes_no_loop_ordinal() {
    // A for..in between two real loops must NOT shift the second loop's
    // ordinal. If for..in were (incorrectly) numbered, loop `1` below would
    // become `2` and this pin would break.
    let mir = analyze(
        "function mk(d) { return { v: d }; }
         function f(n) {
           let keep;
           for (let i = 0; i < n; i = i + 1) { const a = mk(1); let s = a.v; }
           const t = { a: 1, c: 2 };
           for (var c in t) { keep = c; }
           for (let j = 0; j < n; j = j + 1) { keep = mk(3); }
           return keep;
         }",
    );
    let table = compute_arena_table(&mir);
    // Loop 0 = first real for-loop; the for..in is skipped; loop 1 = last for-loop.
    assert!(table.loop_arena("f", 0));
    assert!(table.loop_arena("f", 1));
    assert!(!table.loop_arena("f", 2));
}
```

- [ ] **Step 7: Run the MIR pin to verify it passes**

Run: `cargo test -p kali_mir for_in_consumes_no_loop_ordinal`
Expected: PASS. (If it fails because the last loop is not arena-eligible for an unrelated reason, adjust the loop bodies so both real loops are arena-eligible per the existing `loop_ordinals_are_preorder` shape — the load-bearing assertion is that the second real loop's ordinal is `1`, not `2`.)

- [ ] **Step 8: Confirm no CLBG regression + gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir`
Then: `cargo clippy -p kali_hir -p kali_codegen -p kali_mir -p kali_cli -- -D warnings`
Expected: all pass; nbody/fannkuch/spectral/mandelbrot/binary-trees fixtures unchanged.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_hir/src/lowering/statement.rs crates/kali_codegen/src/emit/control_flow.rs crates/kali_cli/tests/runtime_forin.rs crates/kali_mir/src/analysis/arena_gate_tests.rs
git commit -m "feat(forin): counted-loop lowering for fixed-shape for..in (no key use)"
```

---

### Task 2: `for..in` key-provenance axis (analysis foundation)

Add the types-side tracking that a `for..in` key binding is a "key-of-shape-S", propagated through `last = c`, so later tasks can gate computed access and string materialization. Pure analysis: behavior-neutral (dormant until Task 3 consumes it). Modeled exactly on Spec 3's `runtime_array_bindings` registry.

**Files:**
- Modify: `crates/kali_types/src/scope.rs:62` (add the registry field), `:81` (init in `Scope::new`)
- Modify: `crates/kali_types/src/resolve/mod.rs:407-418` (populate in the `ForInStatement` arm)
- Modify: `crates/kali_types/src/resolve/expression.rs` (consume predicate ~near `is_structural_runtime_array` :238; register helper near `register_runtime_array_binding` :308; propagate on `last = c` in the `AssignmentExpression` path ~:1030)
- Modify: `crates/kali_types/src/repr_infer.rs:653` (seed a scalar node for the key so it has a stable identity)
- Test: `crates/kali_types/src/repr_infer_tests.rs` and a scope/resolve unit test

**Interfaces:**
- Consumes: `ReprTable` object-shape data (`shape_fields`); `is_structural_runtime_array` scope-walk pattern (`expression.rs:238-266`); `register_runtime_array_binding` (`expression.rs:308-326`).
- Produces:
  - `Scope::for_in_key_bindings: IndexMap<String, kali_common::ShapeId>` — binding name → the shape its key enumerates.
  - `fn register_for_in_key(&mut self, name: &str, shape: kali_common::ShapeId)` (resolver-side, walks to declaring scope like `register_runtime_array_binding`).
  - `fn for_in_key_shape(&self, name: &str) -> Option<kali_common::ShapeId>` (scope-chain walk mirroring `is_structural_runtime_array`, fail-closed across untracked-function boundaries).

- [ ] **Step 1: Write the failing unit test (provenance seeded + propagated)**

Add to `crates/kali_types/src/repr_infer_tests.rs` (or a resolve unit test module if the registry is only reachable through `TypeContext` — in that case mirror an existing resolve test). This test asserts the key and its `last=c` alias both resolve to the object's shape via `for_in_key_shape`. If `for_in_key_shape` is only exposed on `TypeContext`, write the test through the resolver entry point used by other `resolve/` tests. Minimal shape assertion via repr first:

```rust
#[test]
fn for_in_key_is_seeded_and_not_a_string_repr_by_default() {
    // The key binding exists after inference and defaults to a scalar repr
    // (I64 ordinal) until a string-use lifts it. This pins that seeding the
    // key node did not accidentally make it F64 or a shape.
    let t = reprs(
        "function m(table) { for (var c in table) { let z = c; } }\nm({ a: 1, c: 2 });\n",
    );
    assert_eq!(t.scalar("m", "c"), Repr::I64);
}
```

- [ ] **Step 2: Run to verify it fails or is unstable**

Run: `cargo test -p kali_types for_in_key_is_seeded_and_not_a_string_repr_by_default`
Expected: FAIL — currently the `ForInStatement` arm at `repr_infer.rs:653` never creates a node for the key `c`, so `t.scalar("m", "c")` returns the default `Repr::I64` only by accident (the binding is unknown). After Step 4 the key has an explicit seeded node. (If it already returns `I64`, keep this test as a guard and proceed; the real coverage is Steps 5-6.)

- [ ] **Step 3: Add the `Scope` registry**

In `crates/kali_types/src/scope.rs`, next to `runtime_array_bindings` (:62), add:

```rust
/// Bindings that hold a `for..in` key over a known-shape object: name ->
/// the enumerated object's ShapeId. Grow-only, per the runtime_array_bindings
/// convention. Seeded at the for..in left-hand var and at `last = c` aliases.
pub for_in_key_bindings: IndexMap<String, kali_common::ShapeId>,
```

Initialize it in `Scope::new` (:81) alongside the other registries: `for_in_key_bindings: IndexMap::new(),`. Do NOT clear it in `invalidate_static_binding` (:102-112) — grow-only, matching `runtime_array_bindings`.

- [ ] **Step 4: Seed the key node in repr_infer + populate the resolver registry**

(a) In `crates/kali_types/src/repr_infer.rs`, the `ForInStatement` arm at :653 currently only visits `right` and `body`. Add a scalar node for the key binding so it has a stable identity (mirror how a declarator seeds a scalar node). Keep it on the default (I64) axis — do NOT add a float or string seed here:

```rust
Statement::ForInStatement(stmt) => {
    self.visit_expr(func, &stmt.right);
    if let ForInLefthand::VariableDeclaration(decl) = &stmt.left {
        for d in &decl.declarations {
            let _ = self.scalar_node_for(func, &d.id); // seed identity, default axis
        }
    }
    self.visit_stmt(func, &stmt.body);
}
```

(b) In `crates/kali_types/src/resolve/mod.rs`, the `ForInStatement` arm (:407-418): after resolving `left` and `right`, derive `right`'s object shape and register the key. Add a `register_for_in_key` helper (mirror `register_runtime_array_binding` at `expression.rs:308-326`) and a shape resolver for `right` (reuse whatever resolves an identifier's `Repr::Object(shape)` — the `binding_repr_function_key` scope walk plus `repr_table.scalar`). Concretely:

```rust
Statement::ForInStatement(ForInStatement { left, right, body }) => {
    self.push_scope(ScopeType::Block);
    match left {
        ForInLefthand::VariableDeclaration(decl) => self.resolve_variable_declaration(decl),
        ForInLefthand::Expression(expr) => self.resolve_expression(expr),
    }
    self.resolve_expression(right);
    // NEW: tag the key binding with the enumerated object's shape when known.
    if let (Some(key_name), Some(shape)) =
        (for_in_key_binding_name(left), self.object_shape_of_expression(right))
    {
        self.register_for_in_key(&key_name, shape);
    }
    self.resolve_loop_body(body);
    self.pop_scope();
}
```

`for_in_key_binding_name(left)` extracts the identifier from a `ForInLefthand::VariableDeclaration` single declarator (return `None` for destructuring/expression LHS — fail closed). `object_shape_of_expression(right)` returns `Some(ShapeId)` only for a bare identifier whose `repr_table` scalar is `Repr::Object(shape)` (else `None`). Both are small helpers on `TypeContext`.

(c) Propagate on `last = c`: in `crates/kali_types/src/resolve/expression.rs` near the reassignment registration (:1030), when the RHS is a bare identifier carrying `for_in_key_shape`, register the LHS with the same shape:

```rust
// alongside the runtime-array reassignment registration (~expression.rs:1030)
if let Expression::Identifier(rhs_id) = rhs_unwrapped {
    if let Some(shape) = self.for_in_key_shape(&rhs_id.name) {
        self.register_for_in_key(&lhs_name, shape);
    }
}
```

- [ ] **Step 5: Add the consume predicate**

In `crates/kali_types/src/resolve/expression.rs`, add `for_in_key_shape(&self, name: &str) -> Option<kali_common::ShapeId>`, modeled on `is_structural_runtime_array` (:238-266): walk the scope chain from innermost, return the first `for_in_key_bindings` hit, stop at the tracked-function boundary (fail closed — do not cross into an enclosing function's scope except module/global under `_start`).

- [ ] **Step 6: Write the propagation + non-propagation unit test**

Add a resolve-layer unit test (mirror the structure of existing `resolve/` tests that build a `TypeContext`) asserting: (1) inside `for (var c in table)`, `for_in_key_shape("c")` is `Some(table's shape)`; (2) after `last = c`, `for_in_key_shape("last")` equals the same shape; (3) a `d` never assigned from a key has `for_in_key_shape("d") == None`. If the registry is not directly testable in isolation, cover it indirectly through the Task 3 gate tests and keep the repr-level test from Step 1 as the Task 2 pin. Note which path you used in the task report.

- [ ] **Step 7: Confirm behavior-neutrality + gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir`
Then: `cargo clippy -p kali_types -- -D warnings`
Expected: all pass; NO existing test changes (the registry is unconsumed → behavior-neutral). CLBG fixtures unchanged.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types/src/scope.rs crates/kali_types/src/resolve/mod.rs crates/kali_types/src/resolve/expression.rs crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs
git commit -m "feat(types): for..in key-provenance registry (dormant analysis)"
```

---

### Task 3: computed key get/set over a uniform-float shape (index use)

Relax the gate AND add the codegen lane together: `obj[c]`, `obj[c] = v`, `obj[c] += obj[k]` where `c`/`k` carry `for_in_key_shape(obj's shape)` and all of `obj`'s fields share one repr (f64 in fasta). Lowers to object field slots at `base + ord*8` (headerless, load offset 0). This is the makeCumulative index lane (without the `last`/null pattern, which is Task 4).

**Files:**
- Create (functions): `emit_object_field_read_dynamic`, `emit_object_field_write_dynamic` in `crates/kali_codegen/src/emit/object.rs`
- Modify: `crates/kali_codegen/src/emit/literal.rs:306-392` (route a computed store whose index is a for-in key to the dynamic object-write path) and the computed-read dispatch
- Modify: `crates/kali_types/src/resolve/expression.rs` (admit the computed key read/write in the store gate + the four oracles) and `crates/kali_types/src/repr_infer.rs` (uniform-repr shape check helper)
- Test: `crates/kali_cli/tests/runtime_forin.rs`; codegen emit test in `crates/kali_codegen/src/emit/object_tests.rs` (or the object test module)

**Interfaces:**
- Consumes: `self.repr_table.shape_fields(shape)` (uniform-repr check + field count); `object_shape_of_node` (`emit/object.rs:14`); the address arithmetic pattern from `emit_array_element_address_node` (`emit/call.rs:2844-2856`) but with load/store `MemArg { offset: 0 }`; `for_in_key_shape` (Task 2).
- Produces: `fn emit_object_field_read_dynamic(&mut self, function: &mut Function, base: LirNodeId, index: LirNodeId, elem_repr: Repr) -> EmittedValue` and `fn emit_object_field_write_dynamic(&mut self, function: &mut Function, base: LirNodeId, index: LirNodeId, value: LirNodeId, elem_repr: Repr)`; `fn shape_is_uniform_repr(&self, shape: ShapeId) -> Option<Repr>` on `ReprTable`.

- [ ] **Step 1: Write the failing e2e test (computed get/set, no `last`)**

Add to `crates/kali_cli/tests/runtime_forin.rs`. Golden derived from `node`:

```rust
#[test]
fn for_in_computed_key_read_write_doubles_each_field() {
    // makeCumulative-shaped index use without the `last`/null pattern:
    // read obj[c], write obj[c]. Sum after doubling proves both directions.
    let src = "const t = { a: 0.25, c: 0.25, g: 0.5 };\n\
function dbl(table) {\n  for (var c in table) {\n    table[c] = table[c] * 2;\n  }\n}\n\
function sum(table) {\n  let s = 0.0;\n  for (var c in table) {\n    s = s + table[c];\n  }\n  return s;\n}\n\
dbl(t);\nconsole.log(sum(t));\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node: (0.25+0.25+0.5)*2 = 2 -> "2\n"
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}
```

Independently re-derive the golden: run the same bytes through `node` and confirm `2`. Record the node version in the task report.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test runtime_forin for_in_computed_key_read_write_doubles_each_field`
Expected: FAIL — computed `table[c]` read/write over an object currently either rejects or reads a wrong slot (object has no length header; the array path would be off by one slot), or the write is ungated.

- [ ] **Step 3: Add the uniform-repr shape check**

In `crates/kali_common/src/repr.rs`, add to `ReprTable`:

```rust
/// If every field of `shape` shares one repr, return it; else None.
/// Dynamic (runtime-ordinal) computed access requires a single element type.
pub fn shape_is_uniform_repr(&self, shape: ShapeId) -> Option<Repr> {
    let fields = self.shape_fields(shape);
    let first = fields.first()?.1;
    if fields.iter().all(|(_, r)| *r == first) {
        Some(first)
    } else {
        None
    }
}
```

- [ ] **Step 4: Add the codegen dynamic object field read/write**

In `crates/kali_codegen/src/emit/object.rs`, add both helpers. Address = `base + index*8`, load/store `MemArg { offset: 0, align: 3, memory_index: 0 }` (headerless — offset 0, unlike arrays):

```rust
pub(crate) fn emit_object_field_read_dynamic(
    &mut self,
    function: &mut Function,
    base: LirNodeId,
    index: LirNodeId,
    elem_repr: kali_common::Repr,
) -> EmittedValue {
    // address: base (i32) + index*8
    self.emit_node(function, base, true);
    function.instruction(&Instruction::I32WrapI64);
    self.emit_node(function, index, true);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I32Const(8));
    function.instruction(&Instruction::I32Mul);
    function.instruction(&Instruction::I32Add);
    let memarg = MemArg { offset: 0, align: 3, memory_index: 0 };
    match elem_repr {
        kali_common::Repr::F64 => function.instruction(&Instruction::F64Load(memarg)),
        _ => function.instruction(&Instruction::I64Load(memarg)),
    };
    EmittedValue::scalar(elem_repr) // use the emitter's existing EmittedValue constructor
}

pub(crate) fn emit_object_field_write_dynamic(
    &mut self,
    function: &mut Function,
    base: LirNodeId,
    index: LirNodeId,
    value: LirNodeId,
    elem_repr: kali_common::Repr,
) {
    // address then value then store
    self.emit_node(function, base, true);
    function.instruction(&Instruction::I32WrapI64);
    self.emit_node(function, index, true);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I32Const(8));
    function.instruction(&Instruction::I32Mul);
    function.instruction(&Instruction::I32Add);
    self.emit_node(function, value, true); // value emission coerces to elem_repr as elsewhere
    let memarg = MemArg { offset: 0, align: 3, memory_index: 0 };
    match elem_repr {
        kali_common::Repr::F64 => function.instruction(&Instruction::F64Store(memarg)),
        _ => function.instruction(&Instruction::I64Store(memarg)),
    };
}
```

Adjust `EmittedValue::scalar`/`EmittedValue::none` to the emitter's real constructors (read `emitter.rs` / how `emit_object_field_read` builds its `EmittedValue` at `emit/object.rs:154-158`). For `+=` (compound assignment), the existing assignment lowering already decomposes `a op= b` into `a = a op b`; ensure both the read and write route through these dynamic helpers when the base is an object and the index is a for-in key.

- [ ] **Step 5: Route computed object access to the dynamic helpers**

In `crates/kali_codegen/src/emit/literal.rs` (the computed-store path ~:306-392) and the computed-read dispatch: when `object_shape_of_node(base)` is `Some(shape)` AND `shape_is_uniform_repr(shape)` is `Some(elem)` AND the index node is a for-in key (structurally: the index identifier's binding is the loop key — recognizable in codegen because its repr/local is the ordinal; the types gate in Step 6 is the authority, codegen mirrors it structurally), call `emit_object_field_write_dynamic` / `emit_object_field_read_dynamic` with `elem`. Otherwise fall through to existing behavior (which the Step-6 gate makes unreachable for unproven cases).

- [ ] **Step 6: Relax the types gate (mirror all four oracles)**

In `crates/kali_types/src/resolve/expression.rs`:
- Computed READ `obj[c]`: admit when `object_shape_of_expression(obj)` is `Some(shape)`, `shape_is_uniform_repr(shape)` is `Some(_)`, and `for_in_key_shape(c) == Some(shape)`. Add this arm to `operand_repr_is_string`? No — the element repr here is F64, not string; instead add recognition so the READ is NOT rejected and resolves to the field repr. Ensure `resolve_member_expression` (`resolve/member.rs:5-42`) does not reject it.
- Computed WRITE `obj[c] = v` / `obj[c] += ...`: in `reject_runtime_string_store` (:699-731) and the assignment dispatch (:987-989), admit the computed store when the base is a uniform-shape object and the index is a for-in key of that shape (do not emit E5506). A string VALUE into such a field still rejects (Task 6).
- Mirror the recognition in whichever of the four oracles a downstream consumer touches for this shape. Since fasta's fields are floats, the key oracle to update is the one gating the float element read/write, not the string ones — but add a guard arm to each of the four so none silently accepts or rejects inconsistently. Document in the task report exactly which arms were added, matching the codegen recognizer in Step 5 one-to-one.

- [ ] **Step 7: Run the e2e test to verify it passes**

Run: `cargo test -p kali_cli --test runtime_forin for_in_computed_key_read_write_doubles_each_field`
Expected: PASS — prints `2\n`.

- [ ] **Step 8: Write a codegen emit test (headerless offset)**

Add to the codegen object test module (mirror `operators_tests.rs:5-26`): lower `const t={a:1.0,c:2.0}; for (var c in t){ t[c]=t[c]; }` and assert the printed WAT contains an `f64.load`/`f64.store` and does NOT contain the array header `offset=8` pattern for this access (i.e. the store uses offset 0). Validate with `wasmparser::Validator`.

- [ ] **Step 9: Gate + commit**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir` then `cargo clippy -p kali_common -p kali_types -p kali_codegen -p kali_cli -- -D warnings`

```bash
git add crates/kali_common/src/repr.rs crates/kali_codegen/src/emit/object.rs crates/kali_codegen/src/emit/literal.rs crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/runtime_forin.rs
git commit -m "feat(forin): computed key get/set over uniform-float fixed shape"
```

---

### Task 4: null-sentinel key alias (`last` pattern) — completes `makeCumulative`

Handle `var last = null; ... if (last) table[c] += table[last]; last = c;`. A for-in-key-provenance binding uses `-1` for null; `if(last)` lowers to `last >= 0`; `table[last]` is guarded by that test. Completes `makeCumulative` byte-for-byte.

**Files:**
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (truthiness of a for-in-key binding: `!= 0` becomes `>= 0`) and the `null` literal store into such a binding (`-1` not `0`)
- Modify: `crates/kali_types/src/resolve/expression.rs` (the `last = c` propagation from Task 2 already tags `last`; ensure `var last = null` initialized then key-assigned is admitted)
- Test: `crates/kali_cli/tests/runtime_forin.rs`

**Interfaces:**
- Consumes: `for_in_key_shape` (Task 2) — a binding carrying it is a "key-or-null". Codegen must recognize the same set structurally (the binding is assigned from the loop key).
- Produces: no new public functions; a truthiness special-case and a null-init special-case keyed on for-in-key provenance.

- [ ] **Step 1: Write the failing e2e test (full makeCumulative)**

```rust
#[test]
fn make_cumulative_matches_node_byte_for_byte() {
    let src = "function makeCumulative(table) {\n  var last = null;\n  for (var c in table) {\n    if (last) table[c] += table[last];\n    last = c;\n  }\n}\n\
function dump(table) {\n  for (var c in table) { console.log(table[c]); }\n}\n\
const t = { a: 0.2, c: 0.3, g: 0.5 };\nmakeCumulative(t);\ndump(t);\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node cumulative: a=0.2, c=0.5, g=1 -> "0.2\n0.5\n1\n"
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0.2\n0.5\n1\n");
}
```

Independently re-derive the golden through `node` (watch float formatting — `console.log(1.0)` prints `1`). Record the node version.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test runtime_forin make_cumulative_matches_node_byte_for_byte`
Expected: FAIL — `if(last)` with `last` holding ordinal `0` (field `a`) is falsy under raw truthiness, so the first cumulative add is skipped incorrectly, or `var last = null` sets `0` and collides with ordinal `0`.

- [ ] **Step 3: null-init and truthiness special-cases in codegen**

In `crates/kali_codegen/src/emit/control_flow.rs`:
- When a `null`/`undefined` literal is stored into a binding that carries for-in-key provenance (structurally: it is later assigned from the loop key), emit `I64Const(-1)` instead of `0`.
- When such a binding is the condition of an `if` / `&&` / `||` / `!` (boolean context), lower its truthiness as `value >= 0` (`I64Const(0)`, `I64GeS`) instead of the default `!= 0`. Find the boolean-context lowering (how `emit_branch` at `control_flow.rs:1040-1114` evaluates the condition node) and branch on the condition identifier's for-in-key provenance.

Codegen must recognize "for-in-key provenance" structurally. Reuse the same signal the types side uses by threading a per-function set of for-in-key binding names into the emitter (populate it when lowering, mirroring how `array_bindings` is threaded from types to codegen). If that plumbing is heavy, the minimal structural signal is: the binding is assigned `= <loop key>` somewhere in the function; compute that set once per function in the emitter.

- [ ] **Step 4: Admit `var last = null` then key-assign in the types gate**

Confirm `crates/kali_types/src/resolve/expression.rs` does not reject `var last = null` followed by `last = c` (the Task 2 propagation tags `last`; the null init must not clobber the tag). Add a gate arm if needed so `table[last]` is admitted (same rule as `table[c]`: `for_in_key_shape(last) == Some(obj's shape)`).

- [ ] **Step 5: Run the e2e test to verify it passes**

Run: `cargo test -p kali_cli --test runtime_forin make_cumulative_matches_node_byte_for_byte`
Expected: PASS — prints `0.2\n0.5\n1\n`.

- [ ] **Step 6: Gate + commit**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir` then `cargo clippy -p kali_codegen -p kali_types -p kali_cli -- -D warnings`

```bash
git add crates/kali_codegen/src/emit/control_flow.rs crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/runtime_forin.rs
git commit -m "feat(forin): null-sentinel key alias completes makeCumulative"
```

---

### Task 5: key-as-string materialization (`return c`) — `selectRandom`

When a for-in key (or its alias) is used as a value, materialize the interned field-name string handle from a per-shape handle table indexed by the ordinal. Completes `selectRandom` (returns the nucleotide character string).

**Files:**
- Create (function): `emit_key_handle_table` (allocate + populate a per-shape table of interned handles, once before the loop) in `crates/kali_codegen/src/emit/object.rs`; wire it in `emit_for_in` (Task 1)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` / the identifier-in-string-context emission — a for-in-key binding used as a string value loads `handle_table_base + ord*8`
- Modify: `crates/kali_types/src/resolve/expression.rs` — the four oracles recognize a for-in-key binding as a runtime string value (so `return c`, `console.log(c)`, `c ==`/`+` are admitted and typed String)
- Modify: `crates/kali_types/src/repr_infer.rs` — a string-USE of a for-in key lifts its return/value axis to String (so `return c` yields `Repr::String`)
- Test: `crates/kali_cli/tests/runtime_forin.rs`

**Interfaces:**
- Consumes: `self.strings.intern(text) -> (offset, len)` (`ctx.rs:200`); `encode_string_handle(offset, len) -> i64` (`lower.rs:2571`); `self.alloc_callee_index()` + the bump-alloc pattern (`emit/object.rs:74-77`); `shape_fields(shape)` for the ordered field names.
- Produces: `fn emit_key_handle_table(&mut self, function: &mut Function, shape: ShapeId) -> u32` returning the local holding the table base pointer (i64), allocated once in the loop preheader.

- [ ] **Step 1: Write the failing e2e test (`selectRandom`-shaped `return c`)**

```rust
#[test]
fn for_in_key_returned_as_string_matches_node() {
    // selectRandom shape: return the key whose cumulative field first exceeds r.
    let src = "function selectRandom(table, r) {\n  for (var c in table) {\n    if (r < table[c]) return c;\n  }\n  return \"?\";\n}\n\
const t = { a: 0.3, c: 0.6, g: 1.0 };\n\
console.log(selectRandom(t, 0.1));\nconsole.log(selectRandom(t, 0.5));\nconsole.log(selectRandom(t, 0.9));\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node: 0.1<0.3 -> "a"; 0.5<0.6 -> "c"; 0.9<1.0 -> "g"
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nc\ng\n");
}
```

Re-derive the golden through `node`; record the version.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test runtime_forin for_in_key_returned_as_string_matches_node`
Expected: FAIL — `return c` returns the raw ordinal (or rejects), not the field-name string handle.

- [ ] **Step 3: Build the per-shape handle table in codegen**

Add `emit_key_handle_table` to `crates/kali_codegen/src/emit/object.rs`. Allocate `N*8` bytes once, store each field's interned handle at slot `j`, return the base local:

```rust
pub(crate) fn emit_key_handle_table(
    &mut self,
    function: &mut Function,
    shape: kali_common::ShapeId,
) -> u32 {
    let names: Vec<String> = self
        .repr_table
        .shape_fields(shape)
        .iter()
        .map(|(n, _)| n.clone())
        .collect();
    let base_local = self.reserve_scratch_i64_local();
    // bump-allocate N*8 bytes (constants -> may use __alloc; never per-iteration)
    function.instruction(&Instruction::I32Const((names.len() * 8) as i32));
    function.instruction(&Instruction::Call(self.alloc_callee_index()));
    function.instruction(&Instruction::I64ExtendI32U);
    function.instruction(&Instruction::LocalSet(base_local));
    for (j, name) in names.iter().enumerate() {
        let (offset, len) = self.strings.intern(name);
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
        function.instruction(&Instruction::I64Store(MemArg {
            offset: (j * 8) as u64,
            align: 3,
            memory_index: 0,
        }));
    }
    base_local
}
```

In `emit_for_in` (Task 1), call `emit_key_handle_table(function, shape)` in the preheader and store the returned base local on the emitter keyed by the key binding name, so string-use sites can find it.

- [ ] **Step 4: Emit the handle load at string-use sites**

Where an identifier is emitted in a string-value context, if it carries for-in-key provenance, emit `I64Load` at `handle_table_base + ord*8` (address = `base(i32) + ord*8`, offset 0) instead of the raw ordinal local. This mirrors `emit_object_field_read_dynamic` (Task 3) but reads from the handle table and yields a String handle.

- [ ] **Step 5: Lift the key's value axis to String in types (mirror the four oracles)**

- `repr_infer.rs`: when a for-in key (or alias) flows into a return / console.log / `+` / `==` position, add a string seed on that value node so `return c` gives `return_repr(func) == Repr::String`.
- `resolve/expression.rs`: add a for-in-key arm to `expression_is_string_typed` (:69), `operand_repr_is_string` (:362), and `expression_is_runtime_string_value` (:635) so a for-in-key identifier is recognized as a runtime string value. Do NOT add it to `expression_is_length_fold_receiver` (:571) — a for-in key is not a compile-time-constant fold receiver.

- [ ] **Step 6: Run the e2e test to verify it passes**

Run: `cargo test -p kali_cli --test runtime_forin for_in_key_returned_as_string_matches_node`
Expected: PASS — prints `a\nc\ng\n`.

- [ ] **Step 7: Gate + commit**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir` then `cargo clippy -p kali_codegen -p kali_types -p kali_cli -- -D warnings`

```bash
git add crates/kali_codegen/src/emit/object.rs crates/kali_codegen/src/emit/control_flow.rs crates/kali_types/src/resolve/expression.rs crates/kali_types/src/repr_infer.rs crates/kali_cli/tests/runtime_forin.rs
git commit -m "feat(forin): materialize for..in key as interned string handle"
```

---

### Task 6: fail-closed gates batch

Add a reject pin for every out-of-scope shape in the spec's fail-closed matrix, each producing an error exit (E5506), with the gate arm that guarantees it. No new accept lanes.

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs` and/or `crates/kali_types/src/resolve/mod.rs` (the `ForInStatement` arm) — add explicit reject arms
- Test: `crates/kali_cli/tests/runtime_forin.rs`

**Interfaces:**
- Consumes: `for_in_key_shape`, `object_shape_of_expression`, `shape_is_uniform_repr`.
- Produces: reject arms emitting `Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, …)`.

- [ ] **Step 1: Write the failing reject tests**

Add all of the following to `crates/kali_cli/tests/runtime_forin.rs` (reject convention: `assert!(!out.status.success(), "...")`):

```rust
#[test]
fn for_in_over_array_is_rejected() {
    let out = run_source("const a = new Array(2);\na[0]=1;\nfor (var c in a) { console.log(c); }\n");
    assert!(!out.status.success(), "for..in over an array must reject");
}

#[test]
fn computed_key_from_non_forin_string_is_rejected() {
    // A plain runtime string key not derived from for..in over `t` -> Spec 4b.
    let out = run_source(
        "function f(t, k) { return t[k]; }\nconst t = { a: 1.0, c: 2.0 };\nconsole.log(f(t, \"a\"));\n",
    );
    assert!(!out.status.success(), "general dynamic string key must reject");
}

#[test]
fn string_value_into_object_field_is_rejected() {
    let out = run_source(
        "function f(table, s) { for (var c in table) { table[c] = s; } }\nconst t = { a: 1.0 };\nf(t, \"x\");\n",
    );
    assert!(!out.status.success(), "storing a string into a field must reject");
}

#[test]
fn for_in_key_indexing_a_different_object_is_rejected() {
    let out = run_source(
        "function f(t, u) { for (var c in t) { console.log(u[c]); } }\nconst t = { a: 1.0 };\nconst u = { a: 9.0, b: 8.0 };\nf(t, u);\n",
    );
    assert!(!out.status.success(), "key used against a different object must reject");
}

#[test]
fn for_in_over_mixed_repr_shape_is_rejected() {
    // Non-uniform field reprs: dynamic index can't pick a per-field type.
    let out = run_source(
        "function f(table) { for (var c in table) { console.log(table[c]); } }\nconst t = { a: 1, c: 2.5 };\nf(t);\n",
    );
    assert!(!out.status.success(), "mixed-repr shape dynamic access must reject");
}
```

- [ ] **Step 2: Run to verify they fail (i.e. currently succeed or miscompile)**

Run: `cargo test -p kali_cli --test runtime_forin -- rejected is_rejected`
Expected: at least some of these currently PASS-the-program (exit 0) or miscompile — the test asserts `!success`, so they FAIL here until the gates are added. Note which already reject (those are already covered by earlier tasks' gates) and which need new arms.

- [ ] **Step 3: Add the reject arms**

In the `ForInStatement` resolver arm (`resolve/mod.rs:407`), reject when `object_shape_of_expression(right)` is `None` (covers arrays, non-objects, unknown/polymorphic shapes) with E5506. In the computed-access gate (`resolve/expression.rs`), reject when: the index is not a `for_in_key_shape` match for the base's shape (covers general dynamic keys and mismatched-object keys); the base shape is not uniform-repr (`shape_is_uniform_repr` is `None`); or the stored value is a runtime string (covers string-into-field). Each arm emits `Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, <specific message>)`. Keep messages specific per row.

- [ ] **Step 4: Run the reject tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_forin -- rejected is_rejected`
Expected: all PASS (all reject). Re-run the accept tests from Tasks 1/3/4/5 to confirm no over-rejection:
Run: `cargo test -p kali_cli --test runtime_forin`
Expected: every test passes (accepts accept, rejects reject).

- [ ] **Step 5: Gate + commit**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir` then `cargo clippy -p kali_types -p kali_cli -- -D warnings`

```bash
git add crates/kali_types/src/resolve/mod.rs crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/runtime_forin.rs
git commit -m "feat(forin): fail-closed gates for out-of-scope for..in shapes"
```

---

### Task 7: capstone + full verification + integration

`makeCumulative` + `selectRandom` over an IUB-style table driven by fasta's deterministic LCG, byte-for-byte vs `node`. Full workspace gate, CLBG guardrails, browser-glue diff, then PR + self-merge.

**Files:**
- Test: `crates/kali_cli/tests/runtime_forin.rs` (capstone)

**Interfaces:**
- Consumes: everything from Tasks 1-6.
- Produces: the shipped lane.

- [ ] **Step 1: Derive the capstone golden from `node`**

Write this exact program to a scratch file and run it through `node` (record the version), capturing stdout verbatim:

```js
function makeCumulative(table) {
  var last = null;
  for (var c in table) {
    if (last) table[c] += table[last];
    last = c;
  }
}
var rngLast = 42;
function random(max) {
  rngLast = (rngLast * 3877 + 29573) % 139968;
  return (max * rngLast) / 139968;
}
function selectRandom(table) {
  var r = random(1.0), c;
  for (c in table) if (r < table[c]) return c;
  return c;
}
var iub = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02 };
makeCumulative(iub);
var out = "";
for (var i = 0; i < 20; i = i + 1) out += selectRandom(iub);
console.log(out);
```

- [ ] **Step 2: Write the capstone test with the derived golden**

Add to `crates/kali_cli/tests/runtime_forin.rs`, pasting the program as a raw string (`let src = r#"..."#;`) and the captured stdout as `expected` (mirror `runtime_join.rs:308-338`). Use the standard two asserts. The golden MUST be independently re-derived twice (implementer + reviewer) per series convention; record both derivations and the node version in the task report.

- [ ] **Step 3: Run the capstone**

Run: `cargo test -p kali_cli --test runtime_forin`
Expected: PASS, byte-for-byte.

- [ ] **Step 4: Full workspace gate**

Run in order, all must exit 0:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

- [ ] **Step 5: CLBG + browser-glue guardrails**

Run the CLBG fixtures explicitly and confirm byte-identical:
```bash
cargo test -p kali_cli clbg
```
Confirm the 4 `kali:rt` import lists are untouched:
```bash
git diff --stat crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs
```
Expected: no diff to the import lists (empty or unrelated).

- [ ] **Step 6: Commit the capstone**

```bash
git add crates/kali_cli/tests/runtime_forin.rs
git commit -m "test(forin): fasta makeCumulative + selectRandom capstone byte-for-byte"
```

- [ ] **Step 7: Push a PR and self-merge**

Per `kali-integration-convention` (`gh` authed as `rahulmutt`; run `gh auth setup-git` first if git can't read credentials):
```bash
git push -u origin HEAD
gh pr create --title "Spec 4a: fasta for..in + fixed-shape dynamic string-keyed access" --body "<summary + task ledger>"
```
Watch CI; when all checks are green (build x2, clippy, determinism x2, fmt, browser-cdp-smoke, phase1-evidence, proof-check), merge:
```bash
gh pr merge --squash --delete-branch
```
Then sync local `main`.

---

## Self-Review

**1. Spec coverage** (each spec section → task):
- Scope: fixed-shape `for..in` + computed get/set, both key uses → Tasks 1 (loop), 3 (index get/set), 5 (key-as-string). ✓
- The two fasta `for..in` sites → `makeCumulative` (Tasks 3+4), `selectRandom` (Task 5), capstone (Task 7). ✓
- Problem (for-in mis-lowered as `if`; both-walks ordinal danger) → Task 1 (text discriminator, no ordinal, desync pin). ✓
- Runtime-ordinal + provenance approach → Task 2 (provenance), Task 3 (ordinal index = `base + ord*8`), Task 5 (handle table). ✓
- Four architecture pieces → Task 1 (recognition+loop), Task 3 (computed access), Task 5 (key-as-string), Task 2 (provenance axis). ✓
- Data flow incl. null sentinel `if(last) → last >= 0` → Task 4. ✓
- Both-sides oracle mirroring → called out in Tasks 3/5/6 gate steps. ✓
- Fail-closed matrix (every row) → Task 6 (array, non-object/unknown shape, general dynamic key, mismatched-object key, string-into-field, mixed-repr). ✓
- Deferred inventory (join receiver families, object string fields) → left rejecting; not touched (no task adds them). ✓
- Base-behavior invariants (CLBG, import lists, no-arena) → Task 1 (arena), Task 7 (CLBG + import diff). ✓
- Testing (unit tiers + capstone + fail-closed) → Tasks 1-7 each carry their tier; Task 7 full gate. ✓
- Integration → Task 7. ✓

**2. Placeholder scan:** No "TBD/TODO/handle appropriately". Codegen steps that touch private emitter APIs (`local_index_for`, `EmittedValue::scalar`, boolean-context lowering) name the exact existing function to read and mirror rather than inventing a signature — these are "read this, reuse that" instructions, not placeholders. The two known unknowns (does the for-in key get a reserved local; exact `EmittedValue` constructor names) are flagged with the file to check.

**3. Type consistency:** `ShapeId`, `Repr`, `EmittedValue`, `LirNodeId`, `MemArg { offset, align, memory_index }`, `encode_string_handle(offset, len) -> i64`, `self.strings.intern(text) -> (offset, len)`, `shape_fields`, `shape_is_uniform_repr` used consistently across tasks. `for_in_key_bindings` / `for_in_key_shape` / `register_for_in_key` names consistent Tasks 2→6. Object access is `MemArg { offset: 0 }` (headerless) everywhere, distinct from the array header offset 8 — consistent.

**Known-unknowns for the implementer to resolve by reading the named files (not placeholders — explicit investigation steps):**
- Whether a `for..in` key variable already receives a reserved local in `collect_function_locals` (Task 1 Step 4) — reserve one if not.
- The emitter's real constructor names for `EmittedValue` and the local-slot lookup for an identifier load (Tasks 1, 3) — read `crates/kali_codegen/src/emitter.rs`.
- How for-in-key provenance is threaded from `kali_types` into the codegen emitter for the structural recognition in Tasks 3-5 — mirror the existing `array_bindings` threading path.
