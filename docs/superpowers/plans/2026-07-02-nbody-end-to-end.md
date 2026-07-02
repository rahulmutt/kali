# n-body End-to-End (First Heap-Object CLBG Slice) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `kali run` execute an idiomatic TS port of the CLBG n-body benchmark and print the two canonical `toFixed(9)` energy lines for `n = 1000`, by adding the first genuine runtime heap-object lane (fixed-shape objects: alloc, typed field load/store, arrays of refs, objects across calls) plus e-notation numeric literals.

**Architecture:** Extend the spectral-norm repr inference (`kali_types/src/repr_infer.rs`) with an object-shape axis: object literals seed per-slot field lists, aliasing flows (assignment / array element / arg↔param / return↔call) union per-field storage nodes in the existing `UnionFind`, and member accesses wire float edges through that shared storage. The solved table gains `Repr::Object(ShapeId)` entries, threaded through the existing `ResolutionResult → AnalyzedSource → CodegenCtx` path. Codegen materializes shaped object literals as headerless bump-allocated structs (field `i` at `base + i*8`) and lowers member reads/writes to typed loads/stores at static offsets. Fold-first: a literal with no write and no cross-boundary flow gets **no table entry** and keeps today's compile-time fold lane byte-identically. Everything outside the monomorphic fixed-shape surface is gated with `e5::FEATURE_UNAVAILABLE` (E5506), never miscompiled.

**Tech Stack:** Rust workspace (`kali_lexer`, `kali_common`, `kali_types`, `kali_codegen`, `kali_cli`), `wasm_encoder` instructions, wasmtime host runtime, Node.js for reference-output capture.

**Spec:** `docs/superpowers/specs/2026-07-02-nbody-end-to-end-design.md` (approved).

## Global Constraints

- Local `main` only. **Never `git push`** (this repo merges locally; no origin pushes).
- Gate for every task: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli` (the full `--workspace` gate has ~900 pre-existing chromium-sandbox failures — do not use it, do not try to fix those).
- Run `cargo fmt --all` before every commit.
- Reject-don't-miscompile: every gate uses `Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, …)` — code 5506, rendered `E5506`.
- No new dependencies. No `memory.grow`. Bump allocator only (no free/GC). Objects have **no header word**: `nfields × 8` bytes, field `i` at byte offset `i*8`.
- Existing object-fold fixtures (`object-enumeration-*`, `const-object-property-access-*`, `reflect-own-keys-*`, `object-literal-property-order-canonicalization-*`) and the fannkuch/spectral outputs must not change. A changed expected output is a regression to investigate, not re-baseline.
- Fixture metadata `buildModes` is exactly `["--fast", "--release", "--release-advanced"]`.
- Commit messages follow the repo's conventional style (`feat(codegen): …`, `test(cli): …`, `feat(types): …`).

## File Structure

| File | Responsibility |
|---|---|
| `crates/kali_lexer/src/number.rs` (modify) | e-notation in `lex_number` |
| `crates/kali_lexer/src/engine_tests.rs` (modify) | lexer tests |
| `crates/kali_common/src/repr.rs` (modify) | `ShapeId`, `Repr::Object`, `ReprTable` shape storage/conflicts |
| `crates/kali_common/src/repr_tests.rs` (modify) | shape-table unit tests |
| `crates/kali_types/src/repr_infer.rs` (modify) | object-shape inference (slots, flows, accesses, materialization, conflicts) |
| `crates/kali_types/src/repr_infer_tests.rs` (modify) | inference unit tests |
| `crates/kali_cli/src/build/compile.rs` (modify) | reject shape conflicts with E5506 |
| `crates/kali_codegen/src/emit/object.rs` (create) | `object_shape_of_node`, `emit_object_allocation`, `emit_object_field_read` |
| `crates/kali_codegen/src/emit/operators.rs` (modify) | member-read routing, `is_float_valued` field arm, arithmetic gate |
| `crates/kali_codegen/src/emit/literal.rs` (modify) | member-write routing in `emit_assignment`; `Object(_)` match arms |
| `crates/kali_codegen/src/emit/control_flow.rs` (modify) | declaration materialization (object literal, array-of-objects literal), return-literal materialization |
| `crates/kali_codegen/src/emit/call.rs` (modify) | `console.log(object)` gate; `Object(_)` match arms; static-length array allocation |
| `crates/kali_cli/tests/imperative_core_runtime.rs` (modify) | all micro-acceptance run-tests and gate tests |
| `crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.{ts,json}` (create) | vendored fixture + schema-v1 metadata |
| `crates/kali_cli/tests/clbg_nbody_runtime.rs` (create) | pinned end-to-end test |
| `crates/kali_cli/tests/runtime_smoke/misc.rs` (modify) | three-build-mode benchmark enrollment |
| `specs/19-feature-maturity.md` (modify) | narrow maturity rows |

Notes for implementers with zero context:
- The LIR is generic `LirNode { kind, text, children }` nodes; codegen re-derives structure. A **member read `p.x`** lowers as a 1-child `Value` node with `text = "x"` and `children[0] = p` — dispatched through the unary emitter in `emit/operators.rs`. A **member write `p.x = v`** reaches `emit_assignment` in `emit/literal.rs` with the same 1-child left node. An **array subscript `a[i]`** has the identical LIR shape (text = index literal or identifier); the two are distinguished by what the base is (registered array binding vs `Object`-repr binding).
- `FunctionEmitter` (constructed in `crates/kali_codegen/src/emitter.rs:100`) holds `locals: BTreeMap<String, u32>`, `bindings` (fold map), `array_bindings: HashSet<String>`, `repr_table`, `function_name`, and helpers `scalar_repr(name)`, `array_elem_repr(name)`, `is_float_valued(id)`, `unwrap_transparent(id)`, `assignment_target_name(node, id)`, `resolve_literal_aggregate(id)`, `is_object_literal(node)`, `object_literal_field(node, field)`, `is_array_literal(node)`.
- `lower.rs` reserves **two trailing i64 scratch locals** per function; `self.locals.len() as u32` is the first scratch slot (this idiom appears throughout).
- The `__heap` bump-pointer global is wasm global 0 (i32). Array layout: `[len:i64 @ +0][elems @ +8…]`.

---

### Task 1: e-notation numeric literals in the lexer

n-body's planetary constants are written `4.84143144246472090e+00` upstream. `lex_number` (`crates/kali_lexer/src/number.rs:5`) accepts only digits, one decimal point, and a bigint `n` suffix. The parser already converts token text with `value.parse::<f64>()` (`crates/kali_parser/src/expression/primary.rs:64`), which natively accepts e-notation — so the lexer is the only change.

**Files:**
- Modify: `crates/kali_lexer/src/number.rs`
- Test: `crates/kali_lexer/src/engine_tests.rs`, `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Produces: `NumericLiteral` tokens for `1e5`, `4.84e+00`, `2E-3`, `1.5e1`. `1e` (no digits) lexes as `1` + identifier `e`. `1e5n` lexes as `1e5` + identifier `n` (BigInt-with-exponent is a JS SyntaxError; the parser rejects the stray identifier).

- [ ] **Step 1: Write the failing lexer tests**

Append to `crates/kali_lexer/src/engine_tests.rs` (same style as `test_lexer_decimal_number` at line 42):

```rust
#[test]
fn test_lexer_exponent_number() {
    for source in ["1e5", "4.84e+00", "2E-3", "1.5e1"] {
        let lexer = Lexer::new(FileId::new(0), source.to_string());
        let result = lexer.lex_all();
        assert_eq!(
            result.tokens[0].kind,
            TokenType::NumericLiteral,
            "source: {source}"
        );
        assert_eq!(result.tokens[0].value, source, "source: {source}");
        assert!(result.diagnostics.is_empty(), "source: {source}");
    }
}

#[test]
fn test_lexer_exponent_without_digits_is_not_consumed() {
    let lexer = Lexer::new(FileId::new(0), "1e".to_string());
    let result = lexer.lex_all();
    assert_eq!(result.tokens[0].kind, TokenType::NumericLiteral);
    assert_eq!(result.tokens[0].value, "1");
    assert_eq!(result.tokens[1].kind, TokenType::Identifier);
}
```

If the `Token` struct's text field is not named `value`, mirror whatever the neighboring tests read (check `test_lexer_decimal_number`; drop the `value` assertions if no test asserts text).

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p kali_lexer test_lexer_exponent -- --nocapture`
Expected: FAIL — `1e5` currently lexes as NumericLiteral `1` followed by identifier `e5`.

- [ ] **Step 3: Implement the exponent branch**

In `crates/kali_lexer/src/number.rs`, insert between the fraction loop (ends line 29) and the bigint-suffix check (line 31):

```rust
        // Scientific-notation exponent: `e`/`E`, optional sign, then at least
        // one digit (`1e5`, `4.84e+00`, `2E-3`). Without a digit the suffix is
        // not part of the number (`1e` lexes as `1` then identifier `e`), and
        // an exponent never takes a bigint `n` suffix (`1e5n` leaves `n` to
        // the identifier lexer; the parser rejects it).
        if matches!(self.source.get(self.position), Some(&'e') | Some(&'E')) {
            let mut probe = self.position + 1;
            if matches!(self.source.get(probe), Some(&'+') | Some(&'-')) {
                probe += 1;
            }
            if self.source.get(probe).is_some_and(|c| c.is_ascii_digit()) {
                self.position = probe;
                while let Some(&c) = self.source.get(self.position) {
                    if c.is_ascii_digit() {
                        self.position += 1;
                    } else {
                        break;
                    }
                }
                return Token::new(TokenType::NumericLiteral, self.slice(_start), self.span());
            }
        }
```

- [ ] **Step 4: Run lexer tests to verify they pass**

Run: `cargo test -p kali_lexer`
Expected: PASS (all, including pre-existing).

- [ ] **Step 5: Write the failing end-to-end micro test**

Append to `crates/kali_cli/tests/imperative_core_runtime.rs`:

```rust
#[test]
fn exponent_notation_literals_run() {
    assert_eq!(run_js("console.log(2e3);"), "2000\n");
    assert_eq!(run_js("console.log((1.5e1).toFixed(1));"), "15.0\n");
    assert_eq!(run_js("console.log((1e-2).toFixed(2));"), "0.01\n");
}
```

- [ ] **Step 6: Run it**

Run: `cargo test -p kali_cli exponent_notation_literals_run`
Expected: PASS (the parser stores literals as `f64`, downstream never sees the raw text). If the `2e3` case fails with output other than `2000`, the AST→LIR literal text kept e-notation; fix by checking where LIR literal text is produced (grep `LirNodeKind::Literal` construction in `kali_mir`/`kali_lir`) and confirm it renders from the parsed `f64` — report findings rather than hacking codegen.

- [ ] **Step 7: Full gate + commit**

```bash
cargo fmt --all
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
git add -A && git commit -m "feat(lexer): scientific-notation numeric literals (e/E exponent)"
```

---

### Task 2: Shape model in `kali_common` (`ShapeId`, `Repr::Object`, table storage)

**Files:**
- Modify: `crates/kali_common/src/repr.rs`
- Test: `crates/kali_common/src/repr_tests.rs`
- Modify (mechanical): every `match` on `Repr` the compiler flags in `kali_codegen`/`kali_types`

**Interfaces (later tasks rely on these exact signatures):**
- `pub struct ShapeId(pub u32)` — `Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord`
- `Repr::Object(ShapeId)` — new variant; default stays `I64`
- `ReprTable::intern_shape(&mut self, fields: Vec<(String, Repr)>) -> ShapeId`
- `ReprTable::shape_fields(&self, shape: ShapeId) -> &[(String, Repr)]`
- `ReprTable::shape_field(&self, shape: ShapeId, name: &str) -> Option<(usize, Repr)>` — (field index, field repr)
- `ReprTable::add_shape_conflict(&mut self, message: String)` / `shape_conflicts(&self) -> &[String]`
- `ReprTable::is_empty` now also false when any shape or conflict exists

- [ ] **Step 1: Write the failing unit tests**

Append to `crates/kali_common/src/repr_tests.rs` (mirror the file's existing imports/style):

```rust
#[test]
fn shape_interning_dedupes_identical_field_lists() {
    let mut table = ReprTable::default();
    let a = table.intern_shape(vec![("x".into(), Repr::F64), ("m".into(), Repr::I64)]);
    let b = table.intern_shape(vec![("x".into(), Repr::F64), ("m".into(), Repr::I64)]);
    let c = table.intern_shape(vec![("x".into(), Repr::I64), ("m".into(), Repr::I64)]);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(table.shape_field(a, "m"), Some((1, Repr::I64)));
    assert_eq!(table.shape_field(a, "nope"), None);
    assert_eq!(table.shape_fields(a).len(), 2);
}

#[test]
fn object_entries_and_conflicts_make_the_table_non_empty() {
    let mut table = ReprTable::default();
    assert!(table.is_empty());
    let s = table.intern_shape(vec![("x".into(), Repr::I64)]);
    table.set_scalar("_start", "p", Repr::Object(s));
    assert!(!table.is_empty());
    assert_eq!(table.scalar("_start", "p"), Repr::Object(s));

    let mut conflicted = ReprTable::default();
    conflicted.add_shape_conflict("boom".into());
    assert!(!conflicted.is_empty());
    assert_eq!(conflicted.shape_conflicts(), ["boom".to_string()]);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p kali_common shape_`
Expected: FAIL to compile (`ShapeId`/`Object` don't exist).

- [ ] **Step 3: Implement in `crates/kali_common/src/repr.rs`**

Add above `Repr`:

```rust
/// Interned identity of a fixed object layout: an ordered list of
/// `(field name, field repr)`. Field `i` lives at byte offset `i * 8`
/// (every field is one 8-byte slot; objects have no header word).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct ShapeId(pub u32);
```

Extend `Repr`:

```rust
pub enum Repr {
    /// Two's-complement 64-bit integer (the default for every `number`).
    #[default]
    I64,
    /// IEEE-754 double.
    F64,
    /// Pointer (i64) to a fixed-shape heap object in linear memory.
    Object(ShapeId),
}
```

Add fields to `ReprTable`:

```rust
    /// Interned object layouts; `ShapeId` indexes this list.
    shapes: Vec<Vec<(String, Repr)>>,
    /// Gate messages from the shape inference (contradictory or unsupported
    /// object usage). Any entry makes compilation fail with E5506.
    shape_conflicts: Vec<String>,
```

Add methods to `impl ReprTable`:

```rust
    pub fn intern_shape(&mut self, fields: Vec<(String, Repr)>) -> ShapeId {
        if let Some(index) = self.shapes.iter().position(|shape| *shape == fields) {
            return ShapeId(index as u32);
        }
        self.shapes.push(fields);
        ShapeId((self.shapes.len() - 1) as u32)
    }

    pub fn shape_fields(&self, shape: ShapeId) -> &[(String, Repr)] {
        &self.shapes[shape.0 as usize]
    }

    /// `(field index, field repr)` for `name` in `shape`; `None` for an
    /// unknown field (callers gate, never miscompile).
    pub fn shape_field(&self, shape: ShapeId, name: &str) -> Option<(usize, Repr)> {
        self.shape_fields(shape)
            .iter()
            .enumerate()
            .find(|(_, (field, _))| field == name)
            .map(|(index, (_, repr))| (index, *repr))
    }

    pub fn add_shape_conflict(&mut self, message: String) {
        self.shape_conflicts.push(message);
    }

    pub fn shape_conflicts(&self) -> &[String] {
        &self.shape_conflicts
    }
```

Change `is_empty` (keep its doc honest):

```rust
    /// True when no float representation, object shape, or shape conflict was
    /// ever recorded (codegen may keep its all-i64 fast paths).
    pub fn is_empty(&self) -> bool {
        !self.any_float && self.shapes.is_empty() && self.shape_conflicts.is_empty()
    }
```

`Vec<(String, Repr)>` equality requires `Repr: PartialEq` — already derived. The `set_*` float guards (`if repr == Repr::F64`) are unchanged: `Object` entries must NOT set `any_float`.

- [ ] **Step 4: Fix every compiler-flagged `match` on `Repr`**

Run: `cargo build -p kali_codegen -p kali_types -p kali_cli 2>&1 | head -60`

For each `non-exhaustive patterns` error, add an `Object(_)` arm with **pointer semantics = i64 semantics** (object references are i64 pointers). Known sites:
- `crates/kali_codegen/src/emit/literal.rs` — `match self.array_elem_repr(&base_name)` in the array-element write (line ~282): make the `I64` arm `kali_common::Repr::I64 | kali_common::Repr::Object(_)` (storing a pointer element is a plain i64 store).
- The array-element **read** path and any wasm signature/local `ValType` selection driven by `Repr` (in `lower.rs` and/or `emitter.rs`): `Object(_)` picks `ValType::I64` / `I64Load` exactly like `I64`.
- Any `kali_types` site the compiler flags: same treatment.

Do NOT change behavior for `I64`/`F64` arms anywhere.

- [ ] **Step 5: Run the full gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli`
Expected: PASS — the new variant is never produced yet, so all behavior is unchanged.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add -A && git commit -m "feat(common): Repr::Object(ShapeId) and shape storage on ReprTable"
```

---

### Task 3: Object-shape inference in `kali_types` + conflict gate in the driver

Extends `crates/kali_types/src/repr_infer.rs` (read its module doc first — the directional-float / bidirectional-array design carries over: object field storage is bidirectional shared storage exactly like array elements).

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs`
- Modify: `crates/kali_cli/src/build/compile.rs` (after `repr_table = resolved.repr_table;`, line ~646)
- Test: `crates/kali_types/src/repr_infer_tests.rs`, `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: Task 2's `ShapeId`, `Repr::Object`, `intern_shape`, `add_shape_conflict`.
- Produces (read by codegen in Tasks 4–6): `table.scalar(func, binding) == Repr::Object(s)` for materialized object bindings/params; `table.param(func, idx) == Repr::Object(s)`; `table.return_repr(func) == Repr::Object(s)` for factories; `table.array_element(func, arr) == Repr::Object(s)` (plus `set_array_binding`) for arrays of refs. **Unmaterialized** (write-free, non-flowing) literals get NO entry — that is the fold-first contract.

- [ ] **Step 1: Write the failing inference unit tests**

Append to `crates/kali_types/src/repr_infer_tests.rs`:

```rust
#[test]
fn written_object_literal_binding_gets_a_shape() {
    let t = reprs("const p = { x: 1.5, y: 2 };\np.x = p.x + 1.0;\nconsole.log(p.y);\n");
    let Repr::Object(shape) = t.scalar("_start", "p") else {
        panic!("p should be an object binding");
    };
    assert_eq!(t.shape_field(shape, "x"), Some((0, Repr::F64)));
    assert_eq!(t.shape_field(shape, "y"), Some((1, Repr::I64)));
}

#[test]
fn read_only_local_object_literal_stays_on_the_fold_lane() {
    let t = reprs("const p = { x: 1.5 };\nconsole.log(p.x);\n");
    assert_eq!(t.scalar("_start", "p"), Repr::I64); // no entry == fold lane
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn field_float_flows_to_reader_binding() {
    let t = reprs("const p = { x: 1 };\np.x = 2.5;\nconst d = p.x;\n");
    assert_eq!(t.scalar("_start", "d"), Repr::F64);
}

#[test]
fn array_of_objects_shares_shape_across_factory_param_and_alias() {
    let src = "\
function mk(v) { return { x: v, m: 1.5 }; }\n\
function bump(arr) { const b = arr[0]; b.x = b.x + arr[1].m; }\n\
const bodies = [mk(1.0), mk(2.0)];\n\
bump(bodies);\n";
    let t = reprs(src);
    let Repr::Object(elem) = t.array_element("_start", "bodies") else {
        panic!("bodies elements should be objects");
    };
    assert_eq!(t.array_element("bump", "arr"), Repr::Object(elem));
    assert_eq!(t.return_repr("mk"), Repr::Object(elem));
    assert_eq!(t.scalar("bump", "b"), Repr::Object(elem));
    assert_eq!(t.param("bump", 0), Repr::Object(elem));
    assert_eq!(t.shape_field(elem, "x"), Some((0, Repr::F64)));
    assert!(t.is_array_binding("_start", "bodies"));
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn shape_mismatch_reassignment_is_a_conflict() {
    let t = reprs("let p = { x: 1.0 };\np = { y: 2.0 };\np.y = 3.0;\n");
    assert!(!t.shape_conflicts().is_empty());
}

#[test]
fn unknown_field_access_is_a_conflict() {
    let t = reprs("const p = { x: 1.0 };\np.x = 2.0;\np.z = 1.0;\n");
    assert!(t.shape_conflicts().iter().any(|m| m.contains("'z'")));
}

#[test]
fn object_literal_as_direct_call_argument_is_a_conflict() {
    let t = reprs("function f(o) { return o.x; }\nf({ x: 1.0 });\n");
    assert!(!t.shape_conflicts().is_empty());
}

#[test]
fn float_and_array_programs_gain_no_shapes() {
    let t = reprs("function f(a) { a[0] = 1 / 2; }\nconst w = new Array(2);\nf(w);\n");
    assert!(t.shape_conflicts().is_empty());
    assert_eq!(t.array_element("_start", "w"), Repr::F64);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p kali_types repr_infer`
Expected: new tests FAIL (`scalar` returns `I64`, no conflicts recorded); all pre-existing tests still PASS.

- [ ] **Step 3: Add the object machinery to `ReprInfer`**

In `crates/kali_types/src/repr_infer.rs`, add after the `CallEdge` struct:

```rust
/// Identity of an object-holding slot for shape/aliasing purposes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum ObjSlot {
    /// `(func, binding)` — a named binding or parameter.
    Binding(String, String),
    /// `(func, array_binding)` — every element of the named array (elements
    /// of one array share one shape and one per-field storage cluster).
    ArrayElem(String, String),
    /// `func` — the function's return value.
    Return(String),
}

/// A recorded `<base>.field` access: read (`other` = the result node) or
/// write (`other` = the stored-value node). Wired to shared field storage
/// after object propagation (`resolve_objects`).
struct ObjAccess {
    base: ObjSlot,
    field: String,
    other: usize,
    is_write: bool,
}
```

Add fields to `struct ReprInfer`:

```rust
    /// Ordered field names of each slot directly initialized by an object literal.
    obj_literal_fields: BTreeMap<ObjSlot, Vec<String>>,
    /// Bidirectional object-aliasing flows (assignment, array element,
    /// arg↔param, return↔call-site). Harmless for scalar slots: flows only
    /// take effect for slots proven to hold object literals.
    obj_flows: Vec<(ObjSlot, ObjSlot)>,
    /// Deferred member accesses (wired in `resolve_objects`).
    obj_accesses: Vec<ObjAccess>,
    /// Per-(slot, field) storage node, unioned across aliased slots.
    obj_field_node: BTreeMap<(ObjSlot, String), usize>,
    /// Slots that must lower as runtime heap objects (any write, any flow).
    obj_materialized: BTreeSet<ObjSlot>,
    /// Object slots with their propagated field lists (set by `resolve_objects`).
    obj_fields_of: BTreeMap<ObjSlot, Vec<String>>,
    /// Gate messages (unsupported or contradictory object usage).
    obj_conflicts: Vec<String>,
```

(`#[derive(Default)]` on `ReprInfer` covers them.) Add helper methods to `impl ReprInfer`, next to `array_elem_node_for`:

```rust
    fn obj_field_node_for(&mut self, slot: &ObjSlot, field: &str) -> usize {
        let key = (slot.clone(), field.to_string());
        if let Some(&n) = self.obj_field_node.get(&key) {
            return n;
        }
        let n = self.new_node();
        self.obj_field_node.insert(key, n);
        n
    }

    /// Record an object literal initializing `slot`: remember its ordered
    /// field names, visit each value, and wire `value -> field storage`
    /// float edges. Unsupported property forms become gate conflicts.
    fn record_object_literal(
        &mut self,
        func: &str,
        slot: ObjSlot,
        obj: &kali_ast::ObjectExpression,
    ) {
        let mut names = Vec::new();
        for prop in &obj.properties {
            let kali_ast::PropertyName::Identifier(key) = &prop.key else {
                self.obj_conflicts.push(format!(
                    "object literal for {slot:?} uses a non-identifier property name, which is unavailable in the current phase"
                ));
                return;
            };
            if !matches!(prop.kind, kali_ast::ObjectPropertyKind::Init) {
                self.obj_conflicts.push(format!(
                    "object literal for {slot:?} uses a getter/setter, which is unavailable in the current phase"
                ));
                return;
            }
            if matches!(prop.value, Expression::ObjectExpression(_)) {
                self.obj_conflicts.push(format!(
                    "nested object field '{key}' is unavailable in the current phase"
                ));
                return;
            }
            let value_node = self.visit_expr(func, &prop.value);
            let field_node = self.obj_field_node_for(&slot, key);
            self.add_edge(value_node, field_node);
            names.push(key.clone());
        }
        match self.obj_literal_fields.entry(slot.clone()) {
            std::collections::btree_map::Entry::Occupied(existing) => {
                if *existing.get() != names {
                    self.obj_conflicts.push(format!(
                        "conflicting object shapes assigned to {slot:?}"
                    ));
                }
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(names);
            }
        }
    }

    /// Record an aliasing flow `dst ~ <expr>` when the expression can carry an
    /// object reference: identifier, `arr[i]`, or bare-identifier call.
    fn record_object_flow_from_expr(&mut self, func: &str, dst: ObjSlot, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => self
                .obj_flows
                .push((dst, ObjSlot::Binding(func.to_string(), name.clone()))),
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                if let Expression::Identifier(array) = &member.object {
                    self.obj_flows
                        .push((dst, ObjSlot::ArrayElem(func.to_string(), array.clone())));
                }
            }
            Expression::CallExpression(call) => {
                if let Expression::Identifier(callee) = &call.callee {
                    self.obj_flows.push((dst, ObjSlot::Return(callee.clone())));
                }
            }
            Expression::ParenthesizedExpression(inner) => {
                self.record_object_flow_from_expr(func, dst, &inner.expression)
            }
            _ => {}
        }
    }

    /// Slot for a member-access base: a bare identifier (binding) or a
    /// subscript of a bare identifier (array element). Registers the array's
    /// element node in the subscript case (the base is an array) and visits
    /// the index for its own edges.
    fn member_base_slot(&mut self, func: &str, base: &Expression) -> Option<ObjSlot> {
        match base {
            Expression::Identifier(name) => {
                Some(ObjSlot::Binding(func.to_string(), name.clone()))
            }
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                if let Some(index) = &member.computed_index {
                    self.visit_expr(func, index);
                }
                match &member.object {
                    Expression::Identifier(array) => {
                        self.array_elem_node_for(func, array);
                        Some(ObjSlot::ArrayElem(func.to_string(), array.clone()))
                    }
                    _ => None,
                }
            }
            Expression::ParenthesizedExpression(inner) => {
                self.member_base_slot(func, &inner.expression)
            }
            _ => None,
        }
    }
```

Note: `member.computed_index` is `Option<…>` — check the actual field type in `kali_ast` (`MemberExpression`) and adapt the `if let Some(index)` accordingly (it may be `Option<Box<Expression>>`).

- [ ] **Step 4: Hook the walk**

Four hook points, all in `repr_infer.rs`:

**(a) `visit_declarator_init`** — replace the function head with:

```rust
    fn visit_declarator_init(&mut self, func: &str, id: &str, init: &Expression) {
        if let Expression::ObjectExpression(obj) = init {
            self.record_object_literal(
                func,
                ObjSlot::Binding(func.to_string(), id.to_string()),
                obj,
            );
            return;
        }
        self.record_object_flow_from_expr(
            func,
            ObjSlot::Binding(func.to_string(), id.to_string()),
            init,
        );
        if self.init_is_array(init) {
```

and inside the existing array-literal element loop, handle object elements before the scalar edge:

```rust
                for element in arr.elements.iter().flatten() {
                    if let kali_ast::ExpressionOrSpread::Expression(expr) = element {
                        if let Expression::ObjectExpression(obj) = expr {
                            self.record_object_literal(
                                func,
                                ObjSlot::ArrayElem(func.to_string(), id.to_string()),
                                obj,
                            );
                            continue;
                        }
                        self.record_object_flow_from_expr(
                            func,
                            ObjSlot::ArrayElem(func.to_string(), id.to_string()),
                            expr,
                        );
                        let en = self.visit_expr(func, expr);
                        self.add_edge(en, elem);
                    }
                }
```

**(b) `visit_stmt` ReturnStatement arm** — replace with:

```rust
            Statement::ReturnStatement(stmt) => {
                if let Some(arg) = &stmt.argument {
                    if let Expression::ObjectExpression(obj) = arg {
                        self.record_object_literal(func, ObjSlot::Return(func.to_string()), obj);
                    } else {
                        self.record_object_flow_from_expr(
                            func,
                            ObjSlot::Return(func.to_string()),
                            arg,
                        );
                        let rn = self.visit_expr(func, arg);
                        let ret = self.return_node_for(func);
                        self.add_edge(rn, ret);
                    }
                }
            }
```

**(c) `visit_assignment`** — two changes. At the top, before the member-store handling:

```rust
        // Whole-object (re)assignment through a plain identifier target.
        if let Expression::Identifier(name) = &assign.left {
            if matches!(assign.operator, AssignmentOperator::Assign) {
                if let Expression::ObjectExpression(obj) = &assign.right {
                    let slot = ObjSlot::Binding(func.to_string(), name.clone());
                    self.record_object_literal(func, slot.clone(), obj);
                    // A reassigned literal is observable through the binding:
                    // the fold lane cannot represent it, so materialize.
                    self.obj_materialized.insert(slot);
                    return self.scalar_node_for(func, name);
                }
                self.record_object_flow_from_expr(
                    func,
                    ObjSlot::Binding(func.to_string(), name.clone()),
                    &assign.right,
                );
            }
        }
```

Then replace the non-computed member branch (currently the comment `// `.length`/`.field =` — visit both sides, no numeric edge`):

```rust
            // Non-computed member store: `<base>.field = v` — deferred object
            // field access, wired after object propagation.
            let rn = self.visit_expr(func, &assign.right);
            if let Some(base) = self.member_base_slot(func, &member.object) {
                self.obj_accesses.push(ObjAccess {
                    base,
                    field: member.property.clone(),
                    other: rn,
                    is_write: true,
                });
            } else {
                self.visit_expr(func, &member.object);
            }
            return rn;
```

**(d) `visit_member`** — after the computed-index branch and the `.length` special case, before the final fallthrough, insert:

```rust
        // Non-computed member read `<base>.field` — deferred object access.
        if let Some(base) = self.member_base_slot(func, &member.object) {
            let result = self.new_node();
            self.obj_accesses.push(ObjAccess {
                base,
                field: member.property.as_str().to_string(),
                other: result,
                is_write: false,
            });
            return result;
        }
```

(Recording a read on a non-object base is harmless: `resolve_objects` ignores accesses whose base never becomes an object, so `str.length`-style reads keep today's behavior. Keep the `.length` special case ABOVE this insert so `bodies.length` still registers the array.)

**(e) `visit_call`** — in the bare-identifier branch's argument loop, gate direct object-literal arguments:

```rust
                for arg in &call.args {
                    if matches!(arg, Expression::ObjectExpression(_)) {
                        self.obj_conflicts.push(
                            "an object literal passed directly as a call argument is unavailable in the current phase; bind it to a const first"
                                .to_string(),
                        );
                    }
                    arg_nodes.push(self.visit_expr(func, arg));
```

- [ ] **Step 5: Interprocedural flows + `resolve_objects`**

In `resolve_calls` step 2 (the drained-calls loop), add object flows alongside the existing wiring:

```rust
                if is_array_param {
                    if let Some(Some((caller, name))) = edge.arg_array_names.get(k) {
                        let caller_elem = self.array_elem_node_for(caller, name);
                        let param_elem = self.array_elem_node_for(&edge.callee, param_name);
                        self.uf.union(caller_elem, param_elem);
                        // Elements of the two arrays are the same objects.
                        self.obj_flows.push((
                            ObjSlot::ArrayElem(caller.clone(), name.clone()),
                            ObjSlot::ArrayElem(edge.callee.clone(), param_name.clone()),
                        ));
                    }
                } else if let Some(&arg_node) = edge.arg_nodes.get(k) {
                    let pnode = self.scalar_node_for(&edge.callee, param_name);
                    self.add_edge(arg_node, pnode);
                    // Object aliasing arg ~ param (no-op unless proven object).
                    if let Some(Some((caller, name))) = edge.arg_array_names.get(k) {
                        self.obj_flows.push((
                            ObjSlot::Binding(caller.clone(), name.clone()),
                            ObjSlot::Binding(edge.callee.clone(), param_name.clone()),
                        ));
                    }
                }
```

Add the new phase, called from `infer_reprs` between `resolve_calls()` and `emit_table()`:

```rust
    // Phase C2: object-shape propagation (field lists across flows, shared
    // field storage unions, deferred member-access wiring, materialization).
    infer.resolve_objects();
```

```rust
    fn resolve_objects(&mut self) {
        // 1. Propagate field lists across flows to a fixpoint (copy into
        //    unknown sides only; mismatches are flagged once, afterwards).
        let mut fields_of: BTreeMap<ObjSlot, Vec<String>> = self.obj_literal_fields.clone();
        loop {
            let mut changed = false;
            for (a, b) in &self.obj_flows {
                match (fields_of.contains_key(a), fields_of.get(b).cloned()) {
                    (false, Some(fields)) => {
                        fields_of.insert(a.clone(), fields);
                        changed = true;
                    }
                    (true, None) => {
                        fields_of.insert(b.clone(), fields_of[a].clone());
                        changed = true;
                    }
                    _ => {}
                }
            }
            if !changed {
                break;
            }
        }
        for (a, b) in &self.obj_flows {
            if let (Some(fa), Some(fb)) = (fields_of.get(a), fields_of.get(b)) {
                if fa != fb {
                    self.obj_conflicts.push(format!(
                        "conflicting object shapes flow between {a:?} and {b:?}"
                    ));
                }
            }
        }

        // 2. Union per-field storage across flows between object slots; both
        //    endpoints of an object flow are observable through an alias, so
        //    they materialize.
        let flows = self.obj_flows.clone();
        for (a, b) in &flows {
            let Some(names) = fields_of.get(a).cloned() else { continue };
            if !fields_of.contains_key(b) {
                continue;
            }
            for name in &names {
                let x = self.obj_field_node_for(a, name);
                let y = self.obj_field_node_for(b, name);
                self.uf.union(x, y);
            }
            self.obj_materialized.insert(a.clone());
            self.obj_materialized.insert(b.clone());
        }

        // 3. Wire deferred member accesses through canonical field storage.
        let accesses = std::mem::take(&mut self.obj_accesses);
        for access in accesses {
            let Some(names) = fields_of.get(&access.base) else {
                continue; // not an object: fold lane / existing behavior
            };
            if !names.contains(&access.field) {
                self.obj_conflicts.push(format!(
                    "unknown field '{}' on fixed-shape object {:?}",
                    access.field, access.base
                ));
                continue;
            }
            let field_node = self.obj_field_node_for(&access.base, &access.field);
            if access.is_write {
                self.add_edge(access.other, field_node);
                self.obj_materialized.insert(access.base.clone());
            } else {
                self.add_edge(field_node, access.other);
            }
        }

        self.obj_fields_of = fields_of;
    }
```

- [ ] **Step 6: Emit shapes into the table**

In `emit_table`, after the existing params loop and before `table` is returned:

```rust
        // Object shapes: one interned Shape per materialized object slot.
        // Unmaterialized (write-free, non-flowing) literals get NO entry —
        // codegen keeps its compile-time fold lane for them.
        let fields_of = std::mem::take(&mut self.obj_fields_of);
        let materialized = std::mem::take(&mut self.obj_materialized);
        for (slot, names) in &fields_of {
            if !materialized.contains(slot) {
                continue;
            }
            let fields: Vec<(String, Repr)> = names
                .iter()
                .map(|name| {
                    let node = self.obj_field_node_for(slot, name);
                    let rep = self.uf.find(node);
                    let repr = if float[rep] { Repr::F64 } else { Repr::I64 };
                    (name.clone(), repr)
                })
                .collect();
            let shape = table.intern_shape(fields);
            match slot {
                ObjSlot::Binding(func, name) => {
                    // A binding both object and float-unified is contradictory.
                    if let Some(&node) = self.scalar_node.get(&(func.clone(), name.clone())) {
                        if float[node] {
                            self.obj_conflicts.push(format!(
                                "binding '{name}' in '{func}' is used both as an object and as a number"
                            ));
                            continue;
                        }
                    }
                    table.set_scalar(func, name, Repr::Object(shape));
                }
                ObjSlot::ArrayElem(func, name) => {
                    table.set_array_binding(func, name);
                    table.set_array_element(func, name, Repr::Object(shape));
                }
                ObjSlot::Return(func) => table.set_return(func, Repr::Object(shape)),
            }
        }
        // Object params mirror the binding entry positionally.
        let functions: Vec<(String, Vec<String>)> = self
            .functions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (func, params) in functions {
            for (index, name) in params.iter().enumerate() {
                if let Repr::Object(shape) = table.scalar(&func, name) {
                    table.set_param(&func, index, Repr::Object(shape));
                }
            }
        }
        for message in std::mem::take(&mut self.obj_conflicts) {
            table.add_shape_conflict(message);
        }
```

Borrow-checker note: `emit_table` computes `float` first via `solve_float()`; the object block needs `&mut self` for `obj_field_node_for` while iterating `fields_of` — that's why `fields_of`/`materialized` are `std::mem::take`n into locals first. If a flagged borrow remains, collect `(slot, names)` pairs into a `Vec` before the loop (the existing code does exactly this dance for scalars/elems).

- [ ] **Step 7: Run the inference tests**

Run: `cargo test -p kali_types repr_infer`
Expected: all Step 1 tests PASS; pre-existing tests unchanged.

- [ ] **Step 8: Reject conflicts in the driver + e2e test**

In `crates/kali_cli/src/build/compile.rs`, immediately after `repr_table = resolved.repr_table;` (inside the same block, line ~646):

```rust
        for message in repr_table.shape_conflicts() {
            diagnostics.push(Diagnostic::error(
                kali_error::e5::FEATURE_UNAVAILABLE as u32,
                message.clone(),
            ));
        }
        if has_errors(&diagnostics) {
            return Err(diagnostics);
        }
```

(Match the file's existing `Diagnostic`/error-code imports — grep `use kali_error` in that file; `e5` lives at `kali_error::_error_codes::e5` re-exported as `kali_error::e5`, value 5506.)

Append to `crates/kali_cli/tests/imperative_core_runtime.rs`:

```rust
#[test]
fn object_shape_mismatch_is_rejected() {
    let combined = run_js_expect_failure(
        "let p = { x: 1.0 };\np = { y: 2.0 };\np.y = 3.0;\nconsole.log(p.y);\n",
    );
    assert!(combined.contains("5506"), "expected E5506 gate, got: {combined}");
}
```

Run: `cargo test -p kali_cli object_shape_mismatch_is_rejected`
Expected: PASS.

- [ ] **Step 9: Full gate + commit**

```bash
cargo fmt --all
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
git add -A && git commit -m "feat(types): object-shape axis on repr inference; reject shape conflicts (E5506)"
```

At this point behavior is unchanged for every compiling program (codegen ignores `Object` entries until Task 4) except that contradictory object programs are now rejected instead of silently mis-folded.

---

### Task 4: Codegen object lane — materialization, field read, field write

**Files:**
- Create: `crates/kali_codegen/src/emit/object.rs`
- Modify: `crates/kali_codegen/src/emit/operators.rs` (unary `_` arm + `is_float_valued`), `crates/kali_codegen/src/emit/literal.rs` (`emit_assignment`), `crates/kali_codegen/src/emit/control_flow.rs` (declaration branch), the `emit` module list (add `mod object;` next to `mod literal;` — find via `grep -rn "mod literal" crates/kali_codegen/src`)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: Task 3's table entries; existing `unwrap_transparent` (make it `pub(crate)` — it is currently private), `assignment_target_name`, `resolve_literal_aggregate`, `is_object_literal`, `object_literal_field`, `is_float_valued`, `scalar_repr`, `array_elem_repr`, `array_bindings`, scratch-local idiom.
- Produces (used by Tasks 5–7):
  - `object_shape_of_node(&self, id: LirNodeId) -> Option<kali_common::ShapeId>`
  - `emit_object_allocation(&mut self, function: &mut Function, literal: &LirNode, shape: kali_common::ShapeId) -> EmittedValue`
  - `emit_object_field_read(&mut self, function: &mut Function, base: LirNodeId, shape: kali_common::ShapeId, field: &str) -> EmittedValue`

- [ ] **Step 1: Write the failing micro tests**

Append to `crates/kali_cli/tests/imperative_core_runtime.rs`:

```rust
#[test]
fn object_field_write_and_read_round_trip() {
    assert_eq!(
        run_js("const p = { x: 1.0 };\np.x = p.x + 1.5;\nconsole.log(p.x.toFixed(1));\n"),
        "2.5\n"
    );
}

#[test]
fn object_field_read_through_alias() {
    assert_eq!(
        run_js(
            "const p = { x: 1.0, y: 2.5 };\np.x = 4.0;\nconst q = p;\nconsole.log((q.x + q.y).toFixed(1));\n"
        ),
        "6.5\n"
    );
}

#[test]
fn integer_object_field_round_trip() {
    assert_eq!(
        run_js("const p = { n: 3 };\np.n = p.n + 4;\nconsole.log(p.n);\n"),
        "7\n"
    );
}
```

Run: `cargo test -p kali_cli object_field` — Expected: FAIL (today the literal lowers to drop-and-`I64Const(0)`).

- [ ] **Step 2: Create `crates/kali_codegen/src/emit/object.rs`**

```rust
//! Runtime fixed-shape heap objects: bump-allocated headerless structs in
//! linear memory (field `i` at `base + i*8`), lowered type-directed off the
//! `Repr::Object(ShapeId)` entries the shape inference recorded. Object
//! literals with no table entry keep the compile-time fold lane in
//! `intrinsics/object.rs` untouched.
use crate::*;

impl<'a> FunctionEmitter<'a> {
    /// Static shape of the object reference produced by `id`, when known:
    /// a bare identifier whose binding repr is `Object(_)`, or a subscript
    /// `a[i]` of a registered array binding whose element repr is `Object(_)`.
    /// A field read of a scalar field returns `None` (nested objects are
    /// gated by the inference).
    pub(crate) fn object_shape_of_node(&self, id: LirNodeId) -> Option<kali_common::ShapeId> {
        let id = self.unwrap_transparent(id);
        let node = self.node(id);
        if node.kind != LirNodeKind::Value {
            return None;
        }
        if node.children.is_empty() {
            let name = node.text.as_deref()?;
            if let kali_common::Repr::Object(shape) = self.scalar_repr(name) {
                return Some(shape);
            }
            return None;
        }
        // Subscript `a[index]`: 1-child with the index in `text`, or 2-child
        // computed (non-operator text). Field reads have the same 1-child
        // shape but their base is not an array binding.
        if node.children.len() == 2
            && is_binary_operator_text(node.text.as_deref().unwrap_or_default())
        {
            return None;
        }
        if node.children.len() > 2 || node.text.as_deref().is_none_or(str::is_empty) {
            return None;
        }
        let base = node.children[0];
        let base_name = self.assignment_target_name(node, base)?;
        if !self.array_bindings.contains(&base_name) {
            return None;
        }
        if let kali_common::Repr::Object(shape) = self.array_elem_repr(&base_name) {
            return Some(shape);
        }
        None
    }

    /// Bump-allocate a fixed-layout object for `literal` (an object-literal
    /// LIR node) with layout `shape`, leaving the i64 base pointer on the
    /// stack. Field values are emitted in shape order via the literal's own
    /// field lookup, promoted to the field's repr.
    pub(crate) fn emit_object_allocation(
        &mut self,
        function: &mut Function,
        literal: &LirNode,
        shape: kali_common::ShapeId,
    ) -> EmittedValue {
        let scratch = self.locals.len() as u32;
        let fields = self.repr_table.shape_fields(shape).to_vec();

        // base = __heap; __heap += nfields * 8.
        function.instruction(&Instruction::GlobalGet(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(scratch));
        function.instruction(&Instruction::GlobalGet(0));
        function.instruction(&Instruction::I32Const((fields.len() * 8) as i32));
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::GlobalSet(0));

        for (index, (name, repr)) in fields.iter().enumerate() {
            let Some(value_id) = self.object_literal_field(literal, name) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "object literal is missing field '{name}' required by its inferred shape"
                    ),
                ));
                continue;
            };
            function.instruction(&Instruction::LocalGet(scratch));
            function.instruction(&Instruction::I32WrapI64);
            let produced = self.emit_node(function, value_id, true);
            let mem = MemArg {
                offset: (index * 8) as u64,
                align: 3,
                memory_index: 0,
            };
            match repr {
                kali_common::Repr::F64 => {
                    if !produced.produced {
                        function.instruction(&Instruction::F64Const(0.0.into()));
                    } else if !self.is_float_valued(value_id) {
                        function.instruction(&Instruction::F64ConvertI64S);
                    }
                    function.instruction(&Instruction::F64Store(mem));
                }
                _ => {
                    if !produced.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                    function.instruction(&Instruction::I64Store(mem));
                }
            }
        }

        function.instruction(&Instruction::LocalGet(scratch));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// `<base>.field` read on a shaped base: typed load at the field's static
    /// offset. Unknown fields are gated, never miscompiled.
    pub(crate) fn emit_object_field_read(
        &mut self,
        function: &mut Function,
        base: LirNodeId,
        shape: kali_common::ShapeId,
        field: &str,
    ) -> EmittedValue {
        let Some((index, repr)) = self.repr_table.shape_field(shape, field) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "unknown field '{field}' on a fixed-shape object; only declared fields are available"
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        };
        let produced = self.emit_node(function, base, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I32WrapI64);
        let mem = MemArg {
            offset: (index * 8) as u64,
            align: 3,
            memory_index: 0,
        };
        match repr {
            kali_common::Repr::F64 => function.instruction(&Instruction::F64Load(mem)),
            _ => function.instruction(&Instruction::I64Load(mem)),
        };
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }
}
```

Register the module (`mod object;`) next to the other `emit` submodules, and change `unwrap_transparent`'s visibility in `operators.rs` from private to `pub(crate)`. If `is_binary_operator_text` is not already importable there, use the same path `emit/literal.rs:259` uses (it is a crate-level helper).

- [ ] **Step 3: Route member reads (operators.rs)**

In the unary emitter's `_` fallback arm (the one that currently tries `object_literal_field` then warns `unsupported unary operator`, `crates/kali_codegen/src/emit/operators.rs:334`), insert at the very top of the arm:

```rust
            _ => {
                if let Some(shape) = self.object_shape_of_node(arg) {
                    return self.emit_object_field_read(function, arg, shape, op);
                }
```

This fires only for shaped bases (materialized objects); array subscripts on non-object arrays and fold-lane literals fall through unchanged. `bodies[i]` itself (op = index, arg = the array identifier) is untouched: `object_shape_of_node(arg)` is `None` because the array binding's *scalar* repr is not `Object`.

- [ ] **Step 4: `is_float_valued` field arm (operators.rs)**

At the top of `is_float_valued` (`operators.rs:431`), after the `unwrap_transparent`/`node` lines, before the `match`:

```rust
        // Fixed-shape object field read: the repr comes from the shape table.
        if node.kind == LirNodeKind::Value && node.children.len() == 1 {
            if let (Some(field), Some(shape)) = (
                node.text.as_deref().filter(|text| !text.is_empty()),
                self.object_shape_of_node(node.children[0]),
            ) {
                return matches!(
                    self.repr_table.shape_field(shape, field),
                    Some((_, kali_common::Repr::F64))
                );
            }
        }
```

- [ ] **Step 5: Declaration materialization (control_flow.rs)**

In the declarator loop (`crates/kali_codegen/src/emit/control_flow.rs`, before the `resolve_array_alloc_call` branch at line ~275), insert:

```rust
                        // Materialized object-literal binding: `const p = {…}`
                        // whose inferred repr is Object(shape) — allocate the
                        // fixed-layout struct and bind the base pointer.
                        // Unmaterialized literals keep the fold lane below.
                        if let Some(name) = declarator.text.clone() {
                            if let kali_common::Repr::Object(shape) = self.scalar_repr(&name) {
                                let aggregate = self
                                    .resolve_literal_aggregate(init)
                                    .map(|id| self.node(id).clone())
                                    .filter(|node| self.is_object_literal(node));
                                if let Some(aggregate) = aggregate {
                                    let allocated =
                                        self.emit_object_allocation(function, &aggregate, shape);
                                    if !allocated.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    if let Some(index) = self.locals.get(&name).copied() {
                                        function.instruction(&Instruction::LocalSet(index));
                                    } else {
                                        function.instruction(&Instruction::Drop);
                                    }
                                    continue;
                                }
                                // A shaped binding aliasing an existing object
                                // (identifier / element / call): the generic
                                // emission below yields the i64 pointer.
                            }
                        }
```

The `continue` also skips the fold-map registration for materialized bindings (the declarator handling after this point is what inserts into `self.bindings` — verify with `grep -n "bindings.insert" crates/kali_codegen/src/emit/control_flow.rs` that the insert happens *after* the insertion point, so folds can never see stale field values for materialized objects).

- [ ] **Step 6: Member writes (literal.rs `emit_assignment`)**

Insert BEFORE the dynamic-array-element-write block (before the comment `// Dynamic array element write` at `crates/kali_codegen/src/emit/literal.rs:244`):

```rust
        // Fixed-shape object field store: `<base>.field = v` (including
        // through an array element: `bodies[0].vx = v`). Must precede the
        // array-write path: both lower as a 1-child member node, but here the
        // BASE (not the whole target) carries the object shape.
        if op == "=" {
            let left_node = self.node(left).clone();
            if left_node.kind == LirNodeKind::Value && left_node.children.len() == 1 {
                if let Some(field) = left_node.text.clone().filter(|text| !text.is_empty()) {
                    let base_id = left_node.children[0];
                    if let Some(shape) = self.object_shape_of_node(base_id) {
                        let Some((index, repr)) = self.repr_table.shape_field(shape, &field)
                        else {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                format!(
                                    "unknown field '{field}' on a fixed-shape object; only declared fields can be assigned"
                                ),
                            ));
                            function.instruction(&Instruction::I64Const(0));
                            return true;
                        };
                        let scratch = self.locals.len() as u32;
                        let produced = self.emit_node(function, base_id, true);
                        if !produced.produced {
                            function.instruction(&Instruction::I64Const(0));
                        }
                        function.instruction(&Instruction::LocalTee(scratch));
                        function.instruction(&Instruction::I32WrapI64);
                        let mem = MemArg {
                            offset: (index * 8) as u64,
                            align: 3,
                            memory_index: 0,
                        };
                        let rhs = self.emit_node(function, right, true);
                        match repr {
                            kali_common::Repr::F64 => {
                                if !rhs.produced {
                                    function.instruction(&Instruction::F64Const(0.0.into()));
                                } else if !self.is_float_valued(right) {
                                    function.instruction(&Instruction::F64ConvertI64S);
                                }
                                function.instruction(&Instruction::F64Store(mem));
                                // Assignment expression result: reload the field.
                                function.instruction(&Instruction::LocalGet(scratch));
                                function.instruction(&Instruction::I32WrapI64);
                                function.instruction(&Instruction::F64Load(mem));
                            }
                            _ => {
                                if !rhs.produced {
                                    function.instruction(&Instruction::I64Const(0));
                                }
                                function.instruction(&Instruction::I64Store(mem));
                                function.instruction(&Instruction::LocalGet(scratch));
                                function.instruction(&Instruction::I32WrapI64);
                                function.instruction(&Instruction::I64Load(mem));
                            }
                        }
                        return true;
                    }
                }
            }
        }
```

(The scratch local is i64 and the pointer is i64 — `LocalTee` is well-typed, unlike the array path which must extend an i32 address first.)

- [ ] **Step 7: Run the micro tests**

Run: `cargo test -p kali_cli object_field integer_object_field`
Expected: all three PASS. If `object_field_write_and_read_round_trip` fails with a diagnostic about an unsupported unary operator, the member read routed to the fold before your arm — re-check Step 3's insertion point is the first statement of the `_` arm.

- [ ] **Step 8: Fold-lane preservation check + full gate**

Run: `cargo test -p kali_codegen && cargo test -p kali_cli`
Expected: PASS with **zero** changed expectations in `object-enumeration-*` / `const-object-property-access-*` / `reflect-own-keys-*` fixtures or any optimizer wasm-size notes. Any change there = regression; stop and fix (most likely cause: a fold-lane binding got a table entry — re-check Task 3's materialization rule).

- [ ] **Step 9: Commit**

```bash
cargo fmt --all
git add -A && git commit -m "feat(codegen): runtime fixed-shape heap objects — alloc, typed field load/store"
```

---

### Task 5: Arrays of object references

**Files:**
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (array-literal-of-objects declaration branch), `crates/kali_codegen/src/emit/call.rs` (static-length array allocation)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: `emit_array_allocation` (`emit/call.rs:2266`), `emit_object_allocation`, `object_shape_of_node` (already handles the subscript-base case), Task 3's `array_element(func, name) == Repr::Object(s)`.
- Produces: array literals of object refs lower to a real `[len@+0][ptrs@+8…]` array; `a[i].f` reads/writes work end-to-end.

- [ ] **Step 1: Write the failing micro tests**

```rust
#[test]
fn array_of_object_literals_reads_and_writes() {
    assert_eq!(
        run_js(
            "const a = [{ x: 1.0 }, { x: 2.0 }];\na[1].x = 5.0;\nconsole.log((a[0].x + a[1].x).toFixed(1));\n"
        ),
        "6.0\n"
    );
}

#[test]
fn array_element_alias_mutation_is_shared() {
    assert_eq!(
        run_js(
            "const a = [{ x: 1.5 }, { x: 2.0 }];\nconst b = a[0];\nb.x = b.x + 1.0;\nconsole.log(a[0].x.toFixed(1));\n"
        ),
        "2.5\n"
    );
}
```

Run: `cargo test -p kali_cli array_of_object array_element_alias` — Expected: FAIL.

- [ ] **Step 2: Static-length array allocation**

In `crates/kali_codegen/src/emit/call.rs`, next to `emit_array_allocation`, add a static-length twin that reuses its exact body shape but emits the length as a constant. Copy `emit_array_allocation`'s body and replace the size-argument emission (`self.emit_array_length_value(function, size_arg); LocalSet(size_scratch)`) with:

```rust
    /// Bump-allocate an array of statically-known length (array literals),
    /// leaving the i64 base handle on the stack. Same layout as
    /// `emit_array_allocation`.
    pub(crate) fn emit_array_allocation_static(
        &mut self,
        function: &mut Function,
        len: usize,
    ) -> EmittedValue {
        // …identical to emit_array_allocation, with the size step replaced by:
        function.instruction(&Instruction::I64Const(len as i64));
        function.instruction(&Instruction::LocalSet(size_scratch));
        // …rest identical (length-header store, __heap advance, handle push).
    }
```

(Duplicate the ~40 lines rather than threading an enum through the existing signature — two call sites, zero risk to the `new Array(n)` path.)

- [ ] **Step 3: Declaration branch for array literals of objects**

In `control_flow.rs`, immediately after Task 4's object-binding branch (still before `resolve_array_alloc_call`):

```rust
                        // Array literal of object references:
                        // `const bodies = [ … ]` with element repr
                        // Object(shape) — allocate the array, then
                        // materialize/store each element pointer.
                        if let Some(name) = declarator.text.clone() {
                            if let kali_common::Repr::Object(elem_shape) =
                                self.array_elem_repr(&name)
                            {
                                let aggregate = self
                                    .resolve_literal_aggregate(init)
                                    .map(|id| self.node(id).clone())
                                    .filter(|node| self.is_array_literal(node));
                                if let (Some(aggregate), Some(index)) =
                                    (aggregate, self.locals.get(&name).copied())
                                {
                                    let allocated = self.emit_array_allocation_static(
                                        function,
                                        aggregate.children.len(),
                                    );
                                    if !allocated.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    function.instruction(&Instruction::LocalSet(index));
                                    self.array_bindings.insert(name.clone());
                                    for (i, child) in
                                        aggregate.children.iter().copied().enumerate()
                                    {
                                        function.instruction(&Instruction::LocalGet(index));
                                        function.instruction(&Instruction::I32WrapI64);
                                        let child_node = self.node(child).clone();
                                        let produced = if self.is_object_literal(&child_node) {
                                            self.emit_object_allocation(
                                                function,
                                                &child_node,
                                                elem_shape,
                                            )
                                        } else {
                                            // Factory call / identifier: already
                                            // an i64 pointer.
                                            self.emit_node(function, child, true)
                                        };
                                        if !produced.produced {
                                            function.instruction(&Instruction::I64Const(0));
                                        }
                                        function.instruction(&Instruction::I64Store(MemArg {
                                            offset: (8 + i * 8) as u64,
                                            align: 3,
                                            memory_index: 0,
                                        }));
                                    }
                                    continue;
                                }
                            }
                        }
```

- [ ] **Step 4: Run the micro tests**

Run: `cargo test -p kali_cli array_of_object array_element_alias`
Expected: PASS. (`a[1].x = 5.0` uses Task 4's write path with a subscript base via `object_shape_of_node`; `const b = a[0]` is a plain i64 element load via the `Object(_)` match arm from Task 2.)

- [ ] **Step 5: Full gate + commit**

```bash
cargo fmt --all
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
git add -A && git commit -m "feat(codegen): array literals of object references with element read/write"
```

---

### Task 6: Objects across function boundaries (params + factory returns)

**Files:**
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (return-statement emission)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: `return_repr(func)`, `emit_object_allocation`. Object params need no new code: the inference gives them `Object` scalar/param entries, wasm signatures treat `Object` as i64 (Task 2 match arms), and `object_shape_of_node` resolves them via `scalar_repr`.

- [ ] **Step 1: Write the failing micro tests**

```rust
#[test]
fn objects_cross_function_boundaries() {
    let src = "\
function mk(v) { return { x: v }; }\n\
function getx(p) { return p.x; }\n\
const a = mk(3.5);\nconsole.log(getx(a).toFixed(1));\n";
    assert_eq!(run_js(src), "3.5\n");
}

#[test]
fn factory_array_advance_shape_miniature() {
    let src = "\
function mk(x, vx) { return { x: x, vx: vx }; }\n\
function advance(bs, dt) {\n\
  for (let i = 0; i < bs.length; i = i + 1) {\n\
    const b = bs[i];\n\
    b.x = b.x + dt * b.vx;\n\
  }\n\
}\n\
const bs = [mk(1.0, 2.0), mk(0.5, 4.0)];\n\
advance(bs, 0.5);\n\
console.log((bs[0].x + bs[1].x).toFixed(2));\n";
    assert_eq!(run_js(src), "4.50\n");
}
```

Run: `cargo test -p kali_cli objects_cross factory_array` — Expected: FAIL (returned literals still lower to drop-and-zero).

- [ ] **Step 2: Materialize returned object literals**

Find the return-statement emission in `crates/kali_codegen/src/emit/control_flow.rs` (`grep -n "Instruction::Return" crates/kali_codegen/src/emit/control_flow.rs`). Where the return argument is emitted, insert before the generic argument emission:

```rust
                // A function whose return repr is Object(shape) returning an
                // object literal materializes it (factory functions). Only the
                // direct return argument routes here — other literals in the
                // body keep their own lanes.
                if let kali_common::Repr::Object(shape) =
                    self.repr_table.return_repr(&self.function_name)
                {
                    if let Some(aggregate_id) = self.resolve_literal_aggregate(argument_id) {
                        let aggregate = self.node(aggregate_id).clone();
                        if self.is_object_literal(&aggregate) {
                            let produced =
                                self.emit_object_allocation(function, &aggregate, shape);
                            if !produced.produced {
                                function.instruction(&Instruction::I64Const(0));
                            }
                            function.instruction(&Instruction::Return);
                            return /* match the surrounding function's return convention */;
                        }
                    }
                }
```

Adapt the tail (`Instruction::Return` + the surrounding function's return value) to mirror exactly what the existing return-argument path does after emitting a value — copy its post-emission shape verbatim.

- [ ] **Step 3: Run the micro tests**

Run: `cargo test -p kali_cli objects_cross factory_array`
Expected: PASS. If `factory_array_advance_shape_miniature` fails while `objects_cross_function_boundaries` passes, the array-param registration is the gap: check that `bs`'s element repr reached `advance` (inference test `array_of_objects_shares_shape_across_factory_param_and_alias` covers this — if that passes but the run fails, the issue is in `emitter.rs:122`'s array-binding registration, which keys off `is_array_binding` and already covers object-element arrays).

- [ ] **Step 4: Full gate + commit**

```bash
cargo fmt --all
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
git add -A && git commit -m "feat(codegen): objects across function boundaries — factory returns and object params"
```

---

### Task 7: Gates — reject object misuse with E5506

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (binary-op gate), `crates/kali_codegen/src/emit/call.rs` (console gate)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

- [ ] **Step 1: Write the failing gate tests**

```rust
#[test]
fn console_log_of_object_reference_is_rejected() {
    let combined =
        run_js_expect_failure("const p = { x: 1.0 };\np.x = 2.0;\nconsole.log(p);\n");
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}

#[test]
fn object_in_arithmetic_is_rejected() {
    let combined =
        run_js_expect_failure("const p = { x: 1.0 };\np.x = 2.0;\nconsole.log(p + 1);\n");
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}

#[test]
fn unknown_field_write_is_rejected() {
    let combined = run_js_expect_failure("const p = { x: 1.0 };\np.x = 2.0;\np.z = 1.0;\n");
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}

#[test]
fn object_literal_direct_argument_is_rejected() {
    let combined = run_js_expect_failure(
        "function f(o) { return o.x; }\nconsole.log(f({ x: 1.0 }).toFixed(1));\n",
    );
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}
```

Run: `cargo test -p kali_cli _is_rejected` — `unknown_field_write` and `object_literal_direct_argument` should already PASS (inference conflicts from Task 3); the console and arithmetic tests FAIL (they currently miscompile silently — the pointer prints / pointer arithmetic).

- [ ] **Step 2: Arithmetic gate**

At the top of `emit_binary` in `operators.rs` (find with `grep -n "fn emit_binary" crates/kali_codegen/src/emit/operators.rs`), after left/right ids are known but before any emission:

```rust
        if self.object_shape_of_node(left).is_some() || self.object_shape_of_node(right).is_some()
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "operator '{op}' on an object reference is unavailable in the current phase; operate on its fields instead"
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }
```

(Adapt `op` to the local variable naming in that function.)

- [ ] **Step 3: Console gate**

The `console.log` call resolves its import via `intrinsics/host.rs:34`; find where the call's arguments are emitted in `emit/call.rs` (grep for the function that consumes that import index — follow `host.rs:34`'s caller). Before emitting each console argument:

```rust
                if self.object_shape_of_node(arg_id).is_some() {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "printing an object reference is unavailable in the current phase; print its fields instead"
                            .to_string(),
                    ));
                }
```

(Adapt `arg_id` to the local naming; an error diagnostic fails the build, so the subsequent emission is unreachable in practice — no need to alter the emission itself.)

- [ ] **Step 4: Run + full gate + commit**

```bash
cargo test -p kali_cli _is_rejected
cargo fmt --all
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
git add -A && git commit -m "feat(codegen): gate object misuse (console/arithmetic/unknown-field) with E5506"
```

---

### Task 8: Vendored n-body fixture, pinned canonical output, maturity rows

**Files:**
- Create: `crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.ts`, `crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.json`, `crates/kali_cli/tests/clbg_nbody_runtime.rs`
- Modify: `crates/kali_cli/tests/runtime_smoke/misc.rs` (three-build-mode enrollment), `specs/19-feature-maturity.md`

- [ ] **Step 1: Write the fixture**

`crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.ts` — planetary constants are the upstream CLBG values, digit-for-digit. Constants are **inlined per factory** (module-level consts read from inside functions are not part of the supported slice); `4 * 3.141592653589793 * 3.141592653589793` preserves the upstream `4 * PI * PI` left-associated evaluation bit-for-bit.

```ts
// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// n-body — idiomatic TS port of the Node.js / JavaScript submission,
// normalized to Kali's pipeline (no intrinsic tuning). Retains upstream attribution.
// SOLAR_MASS = 4 * PI * PI and DAYS_PER_YEAR = 365.24 are inlined at each use.
function Sun() {
  return {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    vx: 0.0,
    vy: 0.0,
    vz: 0.0,
    mass: 4 * 3.141592653589793 * 3.141592653589793
  };
}
function Jupiter() {
  return {
    x: 4.84143144246472090e+00,
    y: -1.16032004402742839e+00,
    z: -1.03622044471123109e-01,
    vx: 1.66007664274403694e-03 * 365.24,
    vy: 7.69901118419740425e-03 * 365.24,
    vz: -6.90460016972063023e-05 * 365.24,
    mass: 9.54791938424326609e-04 * (4 * 3.141592653589793 * 3.141592653589793)
  };
}
function Saturn() {
  return {
    x: 8.34336671824457987e+00,
    y: 4.12479856412430479e+00,
    z: -4.03523417114321381e-01,
    vx: -2.76742510726862411e-03 * 365.24,
    vy: 4.99852801234917238e-03 * 365.24,
    vz: 2.30417297573763929e-05 * 365.24,
    mass: 2.85885980666130812e-04 * (4 * 3.141592653589793 * 3.141592653589793)
  };
}
function Uranus() {
  return {
    x: 1.28943695621391310e+01,
    y: -1.51111514016986312e+01,
    z: -2.23307578892655734e-01,
    vx: 2.96460137564761618e-03 * 365.24,
    vy: 2.37847173959480950e-03 * 365.24,
    vz: -2.96589568540237556e-05 * 365.24,
    mass: 4.36624404335156298e-05 * (4 * 3.141592653589793 * 3.141592653589793)
  };
}
function Neptune() {
  return {
    x: 1.53796971148509165e+01,
    y: -2.59193146099879641e+01,
    z: 1.79258772950371181e-01,
    vx: 2.68067772490389322e-03 * 365.24,
    vy: 1.62824170038242295e-03 * 365.24,
    vz: -9.51592254519715870e-05 * 365.24,
    mass: 5.15138902046611451e-05 * (4 * 3.141592653589793 * 3.141592653589793)
  };
}

function offsetMomentum(bodies) {
  let px = 0;
  let py = 0;
  let pz = 0;
  for (let i = 0; i < bodies.length; i = i + 1) {
    const b = bodies[i];
    px = px + b.vx * b.mass;
    py = py + b.vy * b.mass;
    pz = pz + b.vz * b.mass;
  }
  bodies[0].vx = -px / (4 * 3.141592653589793 * 3.141592653589793);
  bodies[0].vy = -py / (4 * 3.141592653589793 * 3.141592653589793);
  bodies[0].vz = -pz / (4 * 3.141592653589793 * 3.141592653589793);
}

function advance(bodies, dt) {
  for (let i = 0; i < bodies.length; i = i + 1) {
    const bi = bodies[i];
    for (let j = i + 1; j < bodies.length; j = j + 1) {
      const bj = bodies[j];
      const dx = bi.x - bj.x;
      const dy = bi.y - bj.y;
      const dz = bi.z - bj.z;
      const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
      const mag = dt / (distance * distance * distance);
      bi.vx = bi.vx - dx * bj.mass * mag;
      bi.vy = bi.vy - dy * bj.mass * mag;
      bi.vz = bi.vz - dz * bj.mass * mag;
      bj.vx = bj.vx + dx * bi.mass * mag;
      bj.vy = bj.vy + dy * bi.mass * mag;
      bj.vz = bj.vz + dz * bi.mass * mag;
    }
  }
  for (let i = 0; i < bodies.length; i = i + 1) {
    const b = bodies[i];
    b.x = b.x + dt * b.vx;
    b.y = b.y + dt * b.vy;
    b.z = b.z + dt * b.vz;
  }
}

function energy(bodies) {
  let e = 0;
  for (let i = 0; i < bodies.length; i = i + 1) {
    const bi = bodies[i];
    e = e + 0.5 * bi.mass * (bi.vx * bi.vx + bi.vy * bi.vy + bi.vz * bi.vz);
    for (let j = i + 1; j < bodies.length; j = j + 1) {
      const bj = bodies[j];
      const dx = bi.x - bj.x;
      const dy = bi.y - bj.y;
      const dz = bi.z - bj.z;
      const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
      e = e - (bi.mass * bj.mass) / distance;
    }
  }
  return e;
}

const bodies = [Sun(), Jupiter(), Saturn(), Uranus(), Neptune()];
offsetMomentum(bodies);
console.log(energy(bodies).toFixed(9));
for (let i = 0; i < 1000; i = i + 1) {
  advance(bodies, 0.01);
}
console.log(energy(bodies).toFixed(9));
```

- [ ] **Step 2: Capture the reference output under Node and verify it is canonical**

```bash
node crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.ts
```

Expected output (the published CLBG reference for `n = 1000`):

```
-0.169075164
-0.169087605
```

If Node prints anything else, a constant was mistranscribed — fix the fixture until Node produces exactly these two lines. Also confirm neither value is a `toFixed(9)` half-tie (they are not: the 10th significant digits are not `5`-exact), so the Rust half-to-even vs JS half-up divergence (spectral spec §6) cannot bite.

- [ ] **Step 3: Metadata + end-to-end test**

```bash
sha256sum crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.ts
```

`crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.json` (substitute the real digest):

```json
{
  "benchmark": "nbody",
  "version": 1,
  "sourceFile": "nbody-benchmark-v1.ts",
  "sourceSha256": "sha256-<hex digest from sha256sum>",
  "buildModes": ["--fast", "--release", "--release-advanced"]
}
```

`crates/kali_cli/tests/clbg_nbody_runtime.rs` (mirror of `clbg_spectral_norm_runtime.rs`):

```rust
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

#[test]
fn nbody_runs_and_matches_canonical_output() {
    let source = fixture("nbody-benchmark-v1.ts");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "-0.169075164\n-0.169087605\n"
    );
}

#[test]
fn nbody_metadata_is_consistent() {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(fixture("nbody-benchmark-v1.json")).expect("read metadata"),
    )
    .expect("parse metadata");
    assert_eq!(meta["benchmark"], "nbody");
    assert_eq!(meta["version"], 1);
    assert_eq!(meta["sourceFile"], "nbody-benchmark-v1.ts");
    assert_eq!(
        meta["buildModes"],
        serde_json::json!(["--fast", "--release", "--release-advanced"])
    );
    let src = fs::read(fixture("nbody-benchmark-v1.ts")).expect("read source");
    let digest = format!("sha256-{:x}", Sha256::digest(&src));
    assert_eq!(
        meta["sourceSha256"], digest,
        "metadata sha256 must match the source file"
    );
}
```

- [ ] **Step 4: Run the end-to-end test**

Run: `cargo test -p kali_cli --test clbg_nbody_runtime`
Expected: both tests PASS, with stdout byte-identical to the Node capture. This is the slice's acceptance gate — if the values differ from Node, debug with `superpowers:systematic-debugging` (do not adjust the pinned expectation; wasmtime and Node must agree bit-for-bit on this IEEE-754 `+ - * / sqrt neg` program).

- [ ] **Step 5: Three-build-mode enrollment**

Find spectral's enrollment: `grep -n "spectral-norm" crates/kali_cli/tests/runtime_smoke/misc.rs` (the call lands on `assert_optimization_benchmark_fixture(fixture_stem, benchmark_name)` at `misc.rs:1542`). Clone that test/entry with `("nbody-benchmark-v1", "nbody")`, mirroring its naming.

Run: `cargo test -p kali_cli nbody`
Expected: PASS (compiles in all three build modes with deterministic artifacts).

- [ ] **Step 6: Feature-maturity rows**

Read the rows the spectral slice added: `grep -n "toFixed\|f64\|float" specs/19-feature-maturity.md`. Append rows in the same table format, scoped exactly to this slice's surface (wording to adapt to the table's columns):

- Runtime fixed-shape object literals: bump-allocated headerless structs, monomorphic statically-inferred shapes, typed field load/store (f64/i64), field writes including through aliases and array elements. Evidence: `imperative_core_runtime.rs` object tests + `clbg_nbody_runtime.rs`.
- Object references in arrays and across calls: array literals of object refs, element aliasing, object params/factory returns. Same evidence.
- Scientific-notation numeric literals (`1e5`, `4.84e+00`). Evidence: lexer tests + `exponent_notation_literals_run`.
- Gated (not claimed): classes/`new`/`this`/prototypes, dynamic/polymorphic shapes, nested objects, printing object refs, object arithmetic, object literals as direct call arguments — all E5506.

Do NOT touch `proofs/BOUNDARY.md`.

- [ ] **Step 7: Full gate + commit**

```bash
cargo fmt --all
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
git add -A && git commit -m "test(cli),docs(spec): vendored n-body fixture with pinned canonical output; maturity rows"
```

---

## Final verification (after all tasks)

- [ ] `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli` — green.
- [ ] `cargo run -p kali_cli --bin kali -- run crates/kali_cli/tests/fixtures/benchmarks/nbody-benchmark-v1.ts` prints exactly the two canonical lines (use `superpowers:verification-before-completion`).
- [ ] `git log --oneline` shows one commit per task; working tree clean; **nothing pushed**.
- [ ] Spot-check fold-lane preservation: `git diff HEAD~8 -- crates/kali_cli/tests/fixtures/benchmarks/object-enumeration-benchmark-v1.json` shows no changes to any pre-existing fixture expectation.
