# Stage P2 — `structuredClone` deep-clone lane — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `structuredClone` evaluate a real deep clone over the smallest sound value envelope (flat objects of i64 scalars + growable-i64 array fields), closing the two pre-existing silent miscompiles (array-valued object fields; object `===` fail-open) that block its acceptance fixture.

**Architecture:** Three default-deny allowlists, each at one choke point. Lane 1 teaches the shape model + field-read + growable-array dispatch that an object field can be a growable-i64 array (new nullary `Repr::GrowableArrayI64`). Lane 2 adds a per-shape `__clone_shape_N` WASM synthetic and a 3-way call-site dispatch. Lane 3 replaces the object `===`/`!==` gate with a same-shape-only pointer-compare allow lane. Everything outside each envelope fails closed (E5506) or, for the ratified placeholder-construct build case, warns-and-builds.

**Tech Stack:** Rust (`kali_common`, `kali_types`, `kali_codegen`, `kali_cli`); WASM (`wasm-encoder`); integration tests drive `kali run` and diff against `node`.

## Global Constraints

- **Soundness ethos:** every new capability is an ALLOWLIST at a single choke point, default-deny. No denylists. (Standing lesson: denylists leak forever.)
- **Fail-closed diagnostic:** out-of-envelope shapes emit `E5506` (`e5::FEATURE_UNAVAILABLE`), never a miscompile. Exception: the ratified zero-placeholder-construct argument (`new Blob(...)`) warns-and-builds, preserving today's `check`/`build --bundle` success on the corpus pins.
- **Clone envelope (exact):** flat fixed-shape objects whose fields are i64 scalars or growable-i64 arrays. Strings, F64 fields, nested objects, aliasing, cycles → fail closed.
- **Every test asserts the program RUNS** (via `kali run` / `kali test`), not just AST shape. Acceptance = byte-for-byte vs `node`.
- **Gate discipline:** `cargo test --workspace --no-fail-fast`, diffed against a **main worktree**, enumerated twice with zero drift; **0 newly-red** is the only gate that counts. Re-run reproducers on a **freshly built** binary (fix reports are unreliable).
- **Census sync:** any new `__*` synthetic must appear in BOTH `crates/kali_codegen/src/lower.rs` `SYNTHETIC_FUNCTIONS` (line 38) AND the test census `crates/kali_cli/tests/runtime_smoke.rs` `SYNTHETIC_FUNCTIONS` (line 806), or the census test reds.
- **Build the binary before any `kali run` probe:** `cargo build -p kali_cli` and invoke `target/debug/kali`.
- **Spec:** `docs/superpowers/specs/2026-07-17-p2-structured-clone-design.md` (lanes, probes p2a–p2e, §6 summary table).

---

## File Structure

**Modified:**
- `crates/kali_common/src/repr.rs` — add `Repr::GrowableArrayI64` variant + shape helpers.
- `crates/kali_types/src/repr_infer.rs` — infer a growable-i64 array field → `GrowableArrayI64` in the interned shape; conflict otherwise.
- `crates/kali_codegen/src/emit/object.rs` — `emit_object_field_read` returns an array-handle-valued result for a `GrowableArrayI64` field; a new predicate `object_field_is_growable_array`.
- `crates/kali_codegen/src/emit/growable.rs` — growable dispatch (`push`/`join`/`length`/index/`for..of`) accepts a `GrowableArrayI64` field-read receiver, not only a named binding.
- `crates/kali_codegen/src/emit/operators.rs` — Lane 3: same-shape object/array `===`/`!==` allow lane replacing the blanket gate (line 1494–1531).
- `crates/kali_codegen/src/emit/call.rs` — Lane 2: `structuredClone` call recognizer + 3-way dispatch.
- `crates/kali_codegen/src/emit/clone.rs` — **new** — the per-shape `__clone_shape_N` synthetic emitter.
- `crates/kali_codegen/src/lower.rs` — register `__clone_shape_*` synthetics (`SYNTHETIC_FUNCTIONS`, line 38; emission wiring).
- `crates/kali_cli/tests/runtime_smoke.rs` — test-side `SYNTHETIC_FUNCTIONS` census (line 806).

**New tests:**
- `crates/kali_cli/tests/soundness_structured_clone.rs` — all P2 acceptance + tripwire integration tests.

---

## Task ordering rationale

Lane 1 is a hard prerequisite (probes p2d/p2e prove array fields are a live silent miscompile). It ships first and is independently valuable. Lane 3 (`===`) is independent of Lane 1/2 and small; it ships second so the fail-open (p2a) is closed early. Lane 2 (the clone itself) depends on Lane 1's shape/field machinery and lands last. The final task runs the whole-workspace gate + adversarial review.

---

### Task 1: Add the `GrowableArrayI64` field repr variant

**Files:**
- Modify: `crates/kali_common/src/repr.rs:16-28` (the `Repr` enum) and near `:405` (`shape_field`)
- Test: `crates/kali_common/src/repr_tests.rs`

**Interfaces:**
- Produces: `kali_common::Repr::GrowableArrayI64` (nullary, keeps `Repr: Copy`); helper `Repr::is_growable_array(&self) -> bool`.

- [ ] **Step 1: Write the failing test**

In `crates/kali_common/src/repr_tests.rs`, add:

```rust
#[test]
fn growable_array_field_repr_round_trips_in_a_shape() {
    let mut table = ReprTable::default();
    let shape = table.intern_shape(vec![
        ("count".to_string(), Repr::I64),
        ("values".to_string(), Repr::GrowableArrayI64),
    ]);
    assert_eq!(table.shape_field(shape, "count"), Some((0, Repr::I64)));
    assert_eq!(
        table.shape_field(shape, "values"),
        Some((1, Repr::GrowableArrayI64))
    );
    assert!(Repr::GrowableArrayI64.is_growable_array());
    assert!(!Repr::I64.is_growable_array());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_common growable_array_field_repr_round_trips_in_a_shape`
Expected: FAIL — `no variant named GrowableArrayI64` / `no method is_growable_array`.

- [ ] **Step 3: Add the variant + helper**

In `crates/kali_common/src/repr.rs`, extend the enum (keep the existing `Copy` derive):

```rust
pub enum Repr {
    #[default]
    I64,
    F64,
    Object(ShapeId),
    String,
    /// A growable-i64 runtime array stored as an `ARRAY_HANDLE_TAG` handle in
    /// one 8-byte slot (binding local or object field). The ONLY array shape a
    /// fixed-shape object field may carry (Stage P2, Lane 1). Element repr is
    /// fixed to i64; string/float/nested array fields fail closed.
    GrowableArrayI64,
}
```

Add the helper in the `impl Repr` block (create one if absent):

```rust
impl Repr {
    pub fn is_growable_array(&self) -> bool {
        matches!(self, Repr::GrowableArrayI64)
    }
}
```

- [ ] **Step 4: Run test to verify it passes (and nothing else breaks)**

Run: `cargo test -p kali_common`
Expected: PASS. If a non-exhaustive `match self` over `Repr` now errors elsewhere in `kali_common`, add a `Repr::GrowableArrayI64 => ...` arm mirroring the `Object(_)` arm's intent (a heap-handle i64 slot). Fix each `error[E0004]` before moving on.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/repr.rs crates/kali_common/src/repr_tests.rs
git commit -m "feat(repr): add GrowableArrayI64 field repr variant [stageP2]"
```

---

### Task 2: Make `kali_codegen` exhaustive over the new variant (no behavior change)

**Files:**
- Modify: any `match repr { ... }` over `kali_common::Repr` in `crates/kali_codegen/src` that is now non-exhaustive (compiler will list them).
- Test: `cargo build -p kali_codegen` is the gate.

**Interfaces:**
- Consumes: `Repr::GrowableArrayI64` (Task 1).
- Produces: no new public interface — this task only keeps existing sites compiling with the SAFE default (treat an unexpected `GrowableArrayI64` at a scalar site as a fail-closed E5506 or the i64 arm, per the site).

- [ ] **Step 1: Build to surface every non-exhaustive match**

Run: `cargo build -p kali_codegen 2>&1 | grep -A3 "E0004\|non-exhaustive"`
Expected: a list of `match` sites (e.g. in `emit/object.rs`, `emit/literal.rs`). If empty (all matches use `_ =>`), skip to Step 3.

- [ ] **Step 2: Add fail-closed arms**

For each surfaced site, add an explicit arm. At a SCALAR-only site (field store/read of a value expected to be a number), emit the fail-closed pattern rather than silently treating a handle as i64:

```rust
kali_common::Repr::GrowableArrayI64 => {
    self.diagnostics.push(Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        "growable-array field is only supported through structuredClone and array methods in the current phase".to_string(),
    ));
    function.instruction(&Instruction::I64Const(0));
}
```

At sites whose `_ =>` arm already does an i64 load/store of the slot (the handle IS an i64), leaving it in the existing `_ =>` arm is correct — do NOT add a special arm there. Only add explicit arms where the compiler forces it.

- [ ] **Step 3: Verify the crate builds green**

Run: `cargo build -p kali_codegen`
Expected: builds with no `E0004`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_codegen/src
git commit -m "chore(codegen): exhaustive Repr match arms for GrowableArrayI64 (fail-closed) [stageP2]"
```

---

### Task 3: Infer a growable-i64 array object field as `GrowableArrayI64`

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs` (the object-shape field-repr assignment path; grep `intern_shape` and the field-repr resolution feeding it)
- Test: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Consumes: `Repr::GrowableArrayI64`, `growable::growable_array_candidates` (existing).
- Produces: interned shapes whose array-literal-initialized i64 fields carry `Repr::GrowableArrayI64`; a `shape_conflict` for any non-i64-array field (string/float/nested array).

- [ ] **Step 1: Write the failing tests**

In `crates/kali_types/src/repr_infer_tests.rs`:

```rust
#[test]
fn object_field_growable_int_array_infers_growable_array_repr() {
    // { count: 1, values: [1,2,3] } read via o.count and o.values.push(4)
    let src = "const o = { count: 1, values: [1, 2, 3] };\n\
               o.values.push(4);\n\
               console.log(o.count, o.values.length);\n";
    let t = infer_repr_table(src); // existing test helper in this module
    let shape = shape_of_binding(&t, "root", "o"); // existing helper
    assert_eq!(t.shape_field(shape, "count").map(|(_, r)| r), Some(Repr::I64));
    assert_eq!(
        t.shape_field(shape, "values").map(|(_, r)| r),
        Some(Repr::GrowableArrayI64)
    );
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn object_field_string_array_conflicts() {
    let src = "const o = { vals: ['a', 'b'] };\n\
               o.vals.push('c');\n\
               console.log(o.vals.length);\n";
    let t = infer_repr_table(src);
    assert!(!t.shape_conflicts().is_empty());
}
```

If `infer_repr_table` / `shape_of_binding` helpers do not exist under those names, use whichever the surrounding tests already use (grep the file for the pattern the neighboring `shape_field` tests use) and mirror it exactly.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p kali_types object_field_growable_int_array_infers_growable_array_repr object_field_string_array_conflicts`
Expected: FAIL — the array field currently infers `I64` (or `Object`/none), not `GrowableArrayI64`; the string case does not conflict yet.

- [ ] **Step 3: Teach the field-repr resolver**

At the point in `repr_infer.rs` where each object-literal field's `Repr` is decided before `intern_shape`, resolve an array-literal-initialized field as follows:
- If the field's initializer is a growable-i64 array candidate (reuse `growable::growable_array_candidates` provenance — the field binding proves growable AND its element node solves to i64) → assign `Repr::GrowableArrayI64`.
- If the field initializer is an array (literal / `new Array` / array-typed) but NOT provably growable-i64 (string/float element, or non-growable usage) → push a `shape_conflict` (`format!("object field '{name}' is an unsupported array shape; only growable integer array fields are available")`), so compilation fails closed instead of interning a wrong field repr.
- Scalar/object fields: unchanged.

Follow the existing conflict-push idiom already used in this file for contradictory fields (grep `shape_conflicts` pushes near the shape-building code and mirror it).

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p kali_types object_field_growable_int_array_infers_growable_array_repr object_field_string_array_conflicts`
Expected: PASS.

- [ ] **Step 5: Full crate regression**

Run: `cargo test -p kali_types`
Expected: PASS (no existing shape test regressed).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs
git commit -m "feat(types): infer growable-i64 object array fields as GrowableArrayI64; conflict otherwise [stageP2]"
```

---

### Task 4: Field-read produces a growable-array handle; add `object_field_is_growable_array`

**Files:**
- Modify: `crates/kali_codegen/src/emit/object.rs:406-446` (`emit_object_field_read`); add predicate near `object_shape_of_node:14`.
- Test: `crates/kali_cli/tests/soundness_structured_clone.rs` (new file) — read-only field method.

**Interfaces:**
- Consumes: `Repr::GrowableArrayI64`, `shape_field` (Task 1/3).
- Produces: `fn object_field_is_growable_array(&self, node: LirNodeId) -> bool` — true when `node` is a `base.field` read whose base has a known shape and whose `field` repr is `GrowableArrayI64`. `emit_object_field_read` loads the handle slot (i64) and returns it unchanged (the slot already holds the tagged handle); this predicate is what downstream array dispatch consults.

- [ ] **Step 1: Write the failing integration test**

Create `crates/kali_cli/tests/soundness_structured_clone.rs`. Mirror the harness pattern from `crates/kali_cli/tests/soundness_events.rs` (copy its `run_source` / `kali_bin` helper imports and `node`-diff style verbatim). Add:

```rust
#[test]
fn object_array_field_read_only_join_round_trips() {
    let src = "const o = { count: 1, values: [1, 2, 3] };\n\
               console.log(o.values.join(','));\n";
    let out = run_kali_run(src);           // helper: builds+runs, returns stdout
    assert_eq!(out.trim(), "1,2,3");       // node prints 1,2,3; kali prints 0 today
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_structured_clone object_array_field_read_only_join_round_trips`
Expected: FAIL — kali prints `0` (probe p2e).

- [ ] **Step 3: Add the predicate**

In `crates/kali_codegen/src/emit/object.rs`, add:

```rust
/// True when `id` is a `base.field` read whose base carries a known shape and
/// whose `field` is a `GrowableArrayI64` slot — i.e. the loaded i64 is an
/// ARRAY_HANDLE_TAG handle, not a scalar. Lets the growable-array dispatch
/// (push/join/length/index/for-of) accept a field-read receiver, not only a
/// named binding. Any other field shape returns false (fail closed at the
/// dispatch site).
pub(crate) fn object_field_is_growable_array(&self, id: LirNodeId) -> bool {
    let id = self.unwrap_transparent(id);
    let node = self.node(id);
    if node.kind != LirNodeKind::Value || node.children.len() != 1 {
        return false;
    }
    let Some(field) = node.text.as_deref().filter(|t| !t.is_empty()) else {
        return false;
    };
    let Some(shape) = self.object_shape_of_node(node.children[0]) else {
        return false;
    };
    matches!(
        self.repr_table.shape_field(shape, field),
        Some((_, kali_common::Repr::GrowableArrayI64))
    )
}
```

`emit_object_field_read` needs no change to its load (the `_ =>` arm already `I64Load`s the slot — the handle). Confirm its returned `ValueShape` is acceptable to the dispatch consumer in Task 5; if the dispatch needs a distinct marker, thread it there rather than here.

- [ ] **Step 4: (dispatch lands in Task 5) — build only**

Run: `cargo build -p kali_codegen`
Expected: builds (the predicate is unused until Task 5; add `#[allow(dead_code)]` only if the build warns-as-errors, and remove it in Task 5).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/emit/object.rs crates/kali_cli/tests/soundness_structured_clone.rs
git commit -m "feat(codegen): object_field_is_growable_array predicate; field-read handle load [stageP2]"
```

---

### Task 5: Growable dispatch accepts a `GrowableArrayI64` field-read receiver

**Files:**
- Modify: `crates/kali_codegen/src/emit/growable.rs` — the push recognizer (`:465-471`) and the join/length/index/for-of receiver resolution (`:540-576`).
- Test: `crates/kali_cli/tests/soundness_structured_clone.rs`

**Interfaces:**
- Consumes: `object_field_is_growable_array` (Task 4), the existing growable emit paths.
- Produces: `o.values.push(v)` / `.join` / `.length` / `o.values[i]` / `for (const x of o.values)` all lower through the growable synthetics when `o.values` is a `GrowableArrayI64` field.

- [ ] **Step 1: Write the failing tests**

Add to `soundness_structured_clone.rs`:

```rust
#[test]
fn object_array_field_push_join_length_round_trip() {
    let src = "const o = { count: 1, values: [1, 2, 3] };\n\
               o.values.push(4);\n\
               console.log(o.count, o.values.join(','), o.values.length);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "1 1,2,3,4 4"); // matches node
}

#[test]
fn object_array_field_index_and_for_of() {
    let src = "const o = { values: [10, 20, 30] };\n\
               let s = 0;\n\
               for (const x of o.values) { s += x; }\n\
               console.log(o.values[1], s);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "20 60");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p kali_cli --test soundness_structured_clone object_array_field_push_join_length_round_trip object_array_field_index_and_for_of`
Expected: FAIL — kali prints `0`-ish (probe p2d), the receiver is a member expression the dispatch rejects.

- [ ] **Step 3: Extend the push recognizer**

In `growable.rs`, the push guard currently is (line ~471):

```rust
if !receiver_node.children.is_empty() || !self.is_growable_array(base) {
    return None;
}
```

A field-read receiver HAS children (`base.field` is a 1-child `Value`). Broaden the guard to also admit a growable-array field-read receiver:

```rust
let receiver_is_named_growable =
    receiver_node.children.is_empty() && self.is_growable_array(base);
let receiver_is_field_growable = self.object_field_is_growable_array(receiver_id);
if !receiver_is_named_growable && !receiver_is_field_growable {
    return None;
}
```

Then, where the named-binding path resolves the array HANDLE by binding name, the field path must instead emit the field read (`emit_object_field_read`) to push the handle on the stack. Follow the existing handle-materialization in the push body: for the named case it does a `LocalGet` of the binding's handle local; for the field case, call `self.emit_object_field_read(function, base_of_field, shape, field)` (or `emit_node` on the receiver) to leave the handle i64 on the stack in the same position.

- [ ] **Step 4: Extend join/length/index/for-of receiver resolution**

At `growable.rs:540-576` the `is_growable_array(base_name)` name checks decide whether a receiver is growable. For each of these read sites, add a parallel branch: if the receiver is a `GrowableArrayI64` field read (`object_field_is_growable_array`), treat it as growable and materialize its handle via the field read rather than a named-local lookup. Keep the two branches symmetric so both produce an i64 handle before the shared growable emit.

- [ ] **Step 5: Run to verify pass**

Run: `cargo test -p kali_cli --test soundness_structured_clone object_array_field_push_join_length_round_trip object_array_field_index_and_for_of`
Expected: PASS.

- [ ] **Step 6: Add the Lane 1 tripwire test**

```rust
#[test]
fn structured_clone_string_array_field_fails_closed() {
    let src = "const o = { vals: ['a', 'b'] };\n\
               o.vals.push('c');\n\
               console.log(o.vals.length);\n";
    let stderr = run_kali_run_expect_error(src); // helper: expects nonzero exit
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
```

Run: `cargo test -p kali_cli --test soundness_structured_clone structured_clone_string_array_field_fails_closed`
Expected: PASS (string array field conflicts in Task 3 → E5506).

- [ ] **Step 7: Regression + commit**

Run: `cargo test -p kali_codegen && cargo test -p kali_cli --test soundness_structured_clone`
Expected: PASS.

```bash
git add crates/kali_codegen/src/emit/growable.rs crates/kali_cli/tests/soundness_structured_clone.rs
git commit -m "feat(codegen): growable-array dispatch accepts GrowableArrayI64 field receivers (closes p2d/p2e) [stageP2]"
```

---

### Task 6: Lane 3 — same-shape object/array `===`/`!==`

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs:1494-1531` (object-misuse gate + `===` arm).
- Test: `crates/kali_cli/tests/soundness_structured_clone.rs`

**Interfaces:**
- Consumes: `object_shape_of_node` (object.rs:14), `object_field_is_growable_array` (Task 4).
- Produces: `p === q` lowers to `I64Eq`/`I64Ne` when both operands are proven same-shape heap refs; all other object-involving `===`/`!==` → E5506.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn same_shape_object_identity_alias_is_true() {
    let src = "const p = { x: 1 };\nconst q = p;\nconst r = { x: 2 };\n\
               console.log(p === q, p === r);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "true false");
}

#[test]
fn structured_clone_cross_shape_identity_fails_closed() {
    let src = "const a = { x: 1 };\nconst b = { y: 1, z: 2 };\n\
               console.log(a === b);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p kali_cli --test soundness_structured_clone same_shape_object_identity_alias_is_true structured_clone_cross_shape_identity_fails_closed`
Expected: FAIL — alias case currently E5506s (probe p2b) instead of `true false`; cross-shape currently also E5506s but for the wrong reason (blanket gate) — this test will pass by accident, so ALSO assert the alias case to prove the allow lane exists.

- [ ] **Step 3: Replace the blanket object gate for equality ops**

In `operators.rs`, before the existing blanket gate at line 1500, insert a same-shape allow lane for `===`/`!==`:

```rust
if matches!(op, "===" | "!==") {
    let left_shape = self.object_shape_of_node(left);
    let right_shape = self.object_shape_of_node(right);
    let both_growable_field =
        self.object_field_is_growable_array(left) && self.object_field_is_growable_array(right);
    let same_object_shape =
        matches!((left_shape, right_shape), (Some(a), Some(b)) if a == b);
    if same_object_shape || both_growable_field {
        // proven same-shape heap refs: real pointer identity
        self.emit_node(function, left, true);
        self.emit_node(function, right, true);
        function.instruction(&Instruction::I64Eq); // handles are i64
        if op == "!==" {
            function.instruction(&Instruction::I64Eqz); // negate: eq==0
        }
        // I64Eq yields i32; extend to the i64 boolean convention used here
        function.instruction(&Instruction::I64ExtendI32U);
        return EmittedValue { produced: true, shape: ValueShape::Boolean };
    }
    // one-object-one-not, cross-shape, or unknown-repr → fall through to the
    // blanket gate below, which E5506s (closes the p2a fail-open: an
    // unknown-repr operand no longer reaches the scalar `===` arm).
}
```

Confirm the exact boolean encoding the surrounding code uses (`I64Eq` returns i32; the file's other comparisons show whether it `I64ExtendI32U`s or leaves i32 — match the adjacent `<`/`<=` handling and the `Boolean` shape convention). Keep the blanket gate at line 1500 as the fall-through so any object operand NOT matching the allow lane still E5506s — this is what closes the p2a fail-open (unknown-repr `===` no longer slips to the scalar arm).

- [ ] **Step 4: Ensure the unknown-repr fail-open is truly closed**

Add:

```rust
#[test]
fn object_identity_against_unknown_repr_fails_closed() {
    // structuredClone result has unknown repr BEFORE Lane 2 lands; force the
    // p2a shape with a param of unknown repr compared to an object.
    let src = "function f(u) { const o = { x: 1 }; return o === u; }\n\
               console.log(f(0));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
```

Run: `cargo test -p kali_cli --test soundness_structured_clone object_identity_against_unknown_repr_fails_closed`
Expected: PASS — one operand is a known object shape, the other unknown → falls to the blanket gate → E5506 (no `1` fail-open).

- [ ] **Step 5: Run all Lane 3 tests + regression**

Run: `cargo test -p kali_cli --test soundness_structured_clone same_shape_object_identity_alias_is_true structured_clone_cross_shape_identity_fails_closed object_identity_against_unknown_repr_fails_closed && cargo test -p kali_codegen`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/soundness_structured_clone.rs
git commit -m "feat(codegen): same-shape object/array === allow lane; close p2a fail-open [stageP2]"
```

---

### Task 7: Lane 2a — the `__clone_shape_N` synthetic emitter

**Files:**
- Create: `crates/kali_codegen/src/emit/clone.rs`
- Modify: `crates/kali_codegen/src/emit/mod.rs` (module decl), `crates/kali_codegen/src/lower.rs:38` (`SYNTHETIC_FUNCTIONS`) + synthetic emission wiring.
- Test: exercised end-to-end in Task 8; a unit build check here.

**Interfaces:**
- Consumes: `shape_fields` (repr.rs:385), the object allocation path (`emit_object_allocation` layout: `__alloc(nfields*8)`, field `i` at `i*8`), the growable-array element-copy layout (`[len][cap][data_ptr]`, from `growable.rs` docs).
- Produces: `fn clone_shape_synthetic_name(shape: ShapeId) -> String` returning `format!("__clone_shape_{}", shape.0)`; a synthetic WASM function `(param $src i64) (result i64)` that deep-clones one shape and is registered so `self.functions["__clone_shape_N"]` resolves at call sites.

- [ ] **Step 1: Define the naming + registration contract (build-first, no test yet)**

Create `crates/kali_codegen/src/emit/clone.rs`:

```rust
//! Stage P2 Lane 2: per-shape deep-clone synthetic `__clone_shape_<ShapeId>`.
//! Allocates a fresh object of the same shape, copies scalar slots verbatim,
//! and deep-copies growable-i64 array fields into fresh handles so the clone
//! shares no mutable storage with the source. Emitted only for shapes whose
//! every field is in the P2 allowlist (scalar or GrowableArrayI64); the call
//! site (Task 8) gates that before requesting emission.
use crate::*;

pub(crate) fn clone_shape_synthetic_name(shape: kali_common::ShapeId) -> String {
    format!("__clone_shape_{}", shape.0)
}
```

Add `mod clone;` (or `pub(crate) mod clone;`) to `crates/kali_codegen/src/emit/mod.rs`.

- [ ] **Step 2: Register the synthetic prefix in both censuses**

`SYNTHETIC_FUNCTIONS` (lower.rs:38 and runtime_smoke.rs:806) are exact-name lists, but clone names are shape-parameterized. Add a prefix check helper used wherever `SYNTHETIC_FUNCTIONS.contains(name)` is consulted:

In `crates/kali_codegen/src/lower.rs`, add near `SYNTHETIC_FUNCTIONS`:

```rust
/// A synthetic function name is either an exact entry in `SYNTHETIC_FUNCTIONS`
/// or a shape-parameterized clone synthetic `__clone_shape_<n>` (Stage P2).
pub fn is_synthetic_function(name: &str) -> bool {
    SYNTHETIC_FUNCTIONS.contains(&name) || name.starts_with("__clone_shape_")
}
```

Replace the two `SYNTHETIC_FUNCTIONS.contains(&...)` call sites in `lower.rs` (`:1356`, `:1521`) and the one in `env_safety.rs:240` with `is_synthetic_function(...)`. In the test census (`runtime_smoke.rs:806`), mirror the same prefix acceptance in its filter.

- [ ] **Step 3: Emit the clone body**

In `clone.rs`, add the emitter (invoked by lower.rs during synthetic emission). Follow `emit_object_allocation` (object.rs:62) for the alloc + per-field store shape, and the growable header layout for the array deep-copy:

```rust
// Pseudocode of the body per field (name, repr) in shape order:
//   dst = __alloc(nfields*8)
//   for i, (name, repr):
//     I64: dst[i*8] = src[i*8]                      (verbatim slot copy)
//     GrowableArrayI64:
//       h = src[i*8]                                (source handle)
//       len = load len(h)
//       nh = __alloc_growable(len)                  (fresh handle, cap>=len)
//       copy len i64 elements h.data -> nh.data
//       dst[i*8] = nh
//   return dst
```

Use the SAME allocator + growable-init helpers the existing growable `push`/array-literal-seed path uses (grep `growable.rs` for the header field offsets and the fresh-handle allocation it emits for an array literal; reuse those exact offsets so the census stays at zero tag-boxing). Do NOT hand-roll a new memory layout.

- [ ] **Step 4: Build check**

Run: `cargo build -p kali_codegen`
Expected: builds.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/emit/clone.rs crates/kali_codegen/src/emit/mod.rs crates/kali_codegen/src/lower.rs crates/kali_codegen/src/env_safety.rs
git commit -m "feat(codegen): __clone_shape_N deep-clone synthetic + prefix-aware synthetic census [stageP2]"
```

---

### Task 8: Lane 2b — `structuredClone` call recognizer + 3-way dispatch

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (add a `structuredClone` recognizer in the call-dispatch chain).
- Test: `crates/kali_cli/tests/soundness_structured_clone.rs`

**Interfaces:**
- Consumes: `object_shape_of_node`, `shape_fields`, `clone_shape_synthetic_name` (Task 7), `declarator_init_is_placeholder_construct` (lower.rs:2145).
- Produces: `structuredClone(arg)` lowers per the 3-way allowlist; the clone synthetic for the resolved shape is requested for emission (registered in `self.functions`).

- [ ] **Step 1: Write the acceptance test (the fixture body)**

```rust
#[test]
fn structured_clone_deep_clones_scalar_and_array_object() {
    let src = "const original = { count: 1, values: [1, 2, 3] };\n\
               const cloned = structuredClone(original);\n\
               original.values.push(4);\n\
               console.log(cloned.count, cloned.values.join(','), original.values.join(','));\n";
    let out = run_kali_run(src);
    // clone unaffected by the push into original.values
    assert_eq!(out.trim(), "1 1,2,3 1,2,3,4");
}

#[test]
fn structured_clone_result_identity_is_false() {
    let src = "const original = { count: 1, values: [1, 2, 3] };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned === original, cloned.values === original.values);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "false false");
}

#[test]
fn structured_clone_of_unproven_argument_fails_closed() {
    let src = "function f(u) { return structuredClone(u); }\nconsole.log(1);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_structured_clone structured_clone_deep_clones_scalar_and_array_object structured_clone_result_identity_is_false structured_clone_of_unproven_argument_fails_closed`
Expected: FAIL — `structuredClone` currently has no lowering (traps E4000 / prints wrong).

- [ ] **Step 3: Add the recognizer + dispatch**

In `call.rs`, add a recognizer alongside the other bare-builtin call handlers (grep how `scheduling_surface` / host builtins are dispatched in the call chain and insert a `structuredClone` arm there). Dispatch:

```rust
// structuredClone(arg): 3-way default-deny allowlist (Stage P2 Lane 2).
if self.is_structured_clone_call(callee_node) {
    let args = /* the call's argument children */;
    if args.len() != 1 {
        return self.deny_e5506(function, "structuredClone expects exactly one argument in the current phase");
    }
    let arg = args[0];
    // Lane 1: in-envelope object (every field scalar or GrowableArrayI64)
    if let Some(shape) = self.object_shape_of_node(arg) {
        if self.shape_is_clone_envelope(shape) {
            self.request_clone_synthetic(shape); // registers __clone_shape_N for emission
            self.emit_node(function, arg, true); // push src handle
            let idx = self.functions[&clone_shape_synthetic_name(shape)];
            function.instruction(&Instruction::Call(idx));
            return EmittedValue { produced: true, shape: ValueShape::Scalar };
            // NOTE: also propagate Repr::Object(shape) onto the result binding
            // so Lane 3 `cloned === original` proves same-shape (see Step 4).
        }
    }
    // Lane 2 entry 2: zero-placeholder construct arg (new Blob(...)) → warn+build,
    // keep today's placeholder-0 behavior (corpus build pins stay green).
    if self.arg_is_zero_placeholder_construct(arg) {
        self.diagnostics.push(Diagnostic::warning(
            /* e-code for "unsupported-but-builds" warn used elsewhere */,
            "structuredClone of an unsupported construct is a no-op placeholder in the current phase".to_string(),
        ));
        let _ = self.emit_node(function, arg, true); // placeholder 0 lowering
        return EmittedValue { produced: true, shape: ValueShape::Scalar };
    }
    // Lane 2 entry 3: everything else → fail closed.
    return self.deny_e5506(function, "structuredClone argument of unproven or unsupported shape");
}
```

Add the helpers: `is_structured_clone_call` (bare unshadowed `structuredClone` callee — mirror `scheduling_surface`'s shadowing checks at host.rs:951), `shape_is_clone_envelope` (every field repr is `I64`/`F64`-scalar or `GrowableArrayI64`), `arg_is_zero_placeholder_construct` (reuse `declarator_init_is_placeholder_construct` on a `new X()` arg, excluding `Array`/`Uint8Array`/`EventTarget`), `request_clone_synthetic` (registers the synthetic name so `lower.rs` emits it and `self.functions` resolves it). Use the exact warning-code convention already used for "unsupported construct still builds" (grep the Stage D C-1 placeholder path for the code it uses).

- [ ] **Step 4: Propagate the clone result's shape for Lane 3**

Ensure `const cloned = structuredClone(original)` records `cloned`'s repr as `Repr::Object(shape)` (same shape as `original`) so `cloned === original` lands in the Lane 3 allow lane and `cloned.values` is a `GrowableArrayI64` field. This is a repr-inference edge in `repr_infer.rs`: a `structuredClone(x)` initializer copies `x`'s object shape onto the LHS binding. Add it next to the object-shape propagation for aliasing assignments (grep the object-aliasing flow, repr.rs comment at `:209`).

- [ ] **Step 5: Run acceptance tests**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_structured_clone structured_clone_deep_clones_scalar_and_array_object structured_clone_result_identity_is_false structured_clone_of_unproven_argument_fails_closed`
Expected: PASS.

- [ ] **Step 6: Re-masking checks (prove the lanes are real)**

Manually verify (not committed — a scratch probe): temporarily make `request_clone_synthetic`'s array-field copy SHARE the source handle → `structured_clone_deep_clones_scalar_and_array_object` must go RED (clone would see the `push(4)`). Revert. Temporarily make the clone return the src pointer → `structured_clone_result_identity_is_false` must go RED. Revert. This confirms neither test is coincidence-green.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen/src/emit/call.rs crates/kali_types/src/repr_infer.rs crates/kali_cli/tests/soundness_structured_clone.rs
git commit -m "feat(codegen): structuredClone 3-way dispatch + clone-result shape propagation (acceptance green) [stageP2]"
```

---

### Task 9: Lane 2 tripwire — placeholder-construct build preservation

**Files:**
- Test: `crates/kali_cli/tests/soundness_structured_clone.rs`
- Verify: `crates/kali_cli/tests/package_corpus.rs`, `crates/kali_cli/tests/package_corpus/browser_corpus.rs` (existing pins, unchanged).

**Interfaces:**
- Consumes: the Lane 2 entry-2 warn-build path (Task 8).
- Produces: a deliberate tripwire pin that reds if `Blob`/`File` ever gain a real-value lowering without the exclusion-list update.

- [ ] **Step 1: Write the tripwire + build-preservation tests**

```rust
#[test]
fn structured_clone_of_placeholder_construct_still_builds() {
    // Corpus shape: structuredClone(new Blob([...])) must BUILD (check/bundle).
    let src = "structuredClone(new Blob(['x']));\nexport default function root() { return 1; }\n";
    // helper that runs `kali build --bundle` and asserts success
    assert!(build_bundle_succeeds(src));
}

#[test]
fn structured_clone_of_placeholder_construct_tripwire() {
    // DELIBERATE tripwire (not a correctness claim): kali returns placeholder 0
    // for structuredClone(new Blob(...)). node returns a real Blob clone. This
    // pins kali's current same-0 behavior; it goes RED the day Blob gains a real
    // lowering, forcing the declarator_init_is_placeholder_construct exclusion
    // list (lower.rs:2145) to add Blob. See spec §2.3.
    let src = "const b = structuredClone(new Blob(['x']));\nconsole.log(typeof b);\n";
    let out = run_kali_run(src);
    // kali: placeholder 0 → prints its scalar rendering; node: 'object'.
    // Assert the DIVERGENCE is the expected placeholder, not node's 'object'.
    assert_ne!(out.trim(), "object", "Blob gained a real lowering — update the exclusion list");
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p kali_cli --test soundness_structured_clone structured_clone_of_placeholder_construct_still_builds structured_clone_of_placeholder_construct_tripwire`
Expected: PASS (Task 8 entry 2 preserves build + placeholder-0).

- [ ] **Step 3: Verify the corpus build pins are still green (fresh binary)**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test package_corpus && cargo test -p kali_cli --test package_corpus browser_corpus`
Expected: PASS — zero re-pins; the `structuredClone(new Blob([...]))` corpus cases still `check`/`build --bundle` successfully.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_cli/tests/soundness_structured_clone.rs
git commit -m "test(codegen): structuredClone placeholder-construct build preservation + Blob tripwire [stageP2]"
```

---

### Task 10: Census, existing-fixture flip, whole-workspace gate, adversarial review

**Files:**
- Verify/modify: `crates/kali_cli/tests/runtime_smoke.rs` (`structured_clone_and_event_primitives_source` :424, census :806), `crates/kali_cli/tests/runtime_smoke/test.rs:663-783` (the deliberate-flip comments).
- No product code unless the gate surfaces a regression.

**Interfaces:**
- Consumes: everything above.
- Produces: certified 0-newly-red gate + updated fixture disposition.

- [ ] **Step 1: WAT-census the acceptance hot path**

Build the acceptance fixture to WASM and confirm the census test counts ZERO tag-boxing ops and that `__clone_shape_*` is accepted by the prefix-aware synthetic filter:

Run: `cargo test -p kali_cli --test runtime_smoke -- --nocapture 2>&1 | grep -i "synthetic\|tag_boxing\|census"`
Expected: census test passes; `__clone_shape_*` accepted.

- [ ] **Step 2: Re-examine the deliberate-flip fixture**

`structured_clone_and_event_primitives_source` (runtime_smoke.rs:424) previously trapped at `structuredClone` (E4000, the §8.4 flip). With P2, the `structuredClone` prefix now SUCCEEDS; the fixture next hits `AbortController` (P3, still unsupported). Update `runtime_smoke/test.rs:663-783` so the assertion expects the trap/deny to shift from `structuredClone` to the first P3 primitive (`AbortController`), NOT build/run success of the whole fixture. Keep the `success == false` invariant. Adjust the comment naming the "first genuinely-unsupported primitive" from `structuredClone` to `AbortController`.

Run: `cargo test -p kali_cli --test runtime_smoke structured_clone_and_event_primitives`
Expected: PASS with the shifted expectation.

- [ ] **Step 3: Enumerate the full workspace gate twice against main**

```bash
git worktree list   # confirm the main worktree exists (see [[ci-gate-vs-poisoned-baseline]])
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/p2-gate-1.txt
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/p2-gate-2.txt
```

Diff the FAILED sets against the main worktree's baseline. Expected: **0 newly-red**; the two enumerations agree (zero drift). If a test is newly-red, triage whether it is a real regression or a census/count desync (per [[kali-throw-fallout-stage4]]) before proceeding — do not certify with any unexplained newly-red.

- [ ] **Step 4: Whole-stage adversarial review**

Dispatch a review (most-capable model) over the full P2 diff with the mandate: attack each allowlist's boundary (a field that is array-shaped but not i64; a `structuredClone` arg that is object-shaped but has an out-of-envelope field; a cross-shape `===` that could pointer-compare incompatible layouts; the placeholder-construct exclusion list). Every prior stage's whole-stage review caught a silent miscompile no per-task review saw. Fix findings with allowlist tightening (never a denylist patch), re-running the original reproducer AND probing the allowed side.

- [ ] **Step 5: Final fresh-binary acceptance re-run**

```bash
cargo build -p kali_cli
cargo test -p kali_cli --test soundness_structured_clone
cargo test -p kali_cli --test runtime_smoke structured_clone
cargo test -p kali_cli --test package_corpus
```

Expected: all PASS on the freshly built binary.

- [ ] **Step 6: Commit the close-out**

```bash
git add crates/kali_cli/tests/runtime_smoke.rs crates/kali_cli/tests/runtime_smoke/test.rs
git commit -m "test(soundness): P2 close-out — census sync, structuredClone-prefix flip to AbortController, 0-newly-red gate [stageP2]"
```

- [ ] **Step 7: Update the follow-up inventory + memory**

In `docs/superpowers/followups/stageD-triage.md` §8.6, mark "Stage P2 — structuredClone" as SHIPPED with the commit range; note P3's newly-relevant items (the clone envelope widening to string/nested fields; the `webBaselineSmoke` end-to-end flip still pending P5). Write a `kali-structured-clone-p2` memory file per the memory convention and add its MEMORY.md pointer.

---

## Self-Review

**1. Spec coverage:**
- Lane 1 array fields → Tasks 1–5. ✓
- Lane 2 clone synthetic + 3-way dispatch → Tasks 7–8. ✓
- Lane 3 `===` → Task 6. ✓
- p2a fail-open closed → Task 6 Step 4. ✓
- p2d/p2e miscompile closed → Task 5. ✓
- Placeholder-construct build preservation + Blob tripwire (§2.3) → Task 9. ✓
- Cross-shape identity tripwire (§3) → Task 6. ✓
- String-array-field tripwire (§1) → Task 5 Step 6. ✓
- Census sync (§2.1) → Tasks 7, 10. ✓
- Deliberate-flip fixture shift (§0/§5) → Task 10 Step 2. ✓
- Re-masking checks (§5) → Task 8 Step 6. ✓
- 0-newly-red gate + adversarial review (§5) → Task 10. ✓
- `JSON.stringify`/arity/diagnostics (§4) → arity in Task 8 Step 3; `JSON.stringify` explicitly out of scope (happy path never evaluates it) per spec §4. ✓
- Non-goal: `webBaselineSmoke` end-to-end stays red (P3–P5) → stated in Task 10 Step 7. ✓

**2. Placeholder scan:** No "TBD"/"add error handling"/"similar to Task N". Where an exact instruction encoding or helper name depends on an adjacent convention (boolean i32/i64 encoding in operators.rs; the warn e-code), the step names the exact grep target and analog to copy — this is direction to an existing pattern, not a placeholder. The deep inference edits (Tasks 3, 8 Step 4) cite the exact file, the existing idiom to mirror, and the conflict-push pattern.

**3. Type consistency:** `Repr::GrowableArrayI64` (Task 1) used identically in Tasks 2–8. `object_field_is_growable_array` (Task 4) consumed in Tasks 5–6. `clone_shape_synthetic_name` (Task 7) consumed in Task 8. `is_synthetic_function` (Task 7) replaces the three `SYNTHETIC_FUNCTIONS.contains` sites consistently. Test helper names (`run_kali_run`, `run_kali_run_expect_error`, `build_bundle_succeeds`) are introduced in Task 4 Step 1 (mirroring `soundness_events.rs`) and reused verbatim thereafter.

Known risk flagged for execution: Lane 1's inference edit (Task 3) and the growable-field dispatch (Task 5) are the deepest changes; the growable header offsets must be reused verbatim from `growable.rs` (Task 7 Step 3) to keep the census at zero tag-boxing. These are called out in-task rather than hand-waved.
