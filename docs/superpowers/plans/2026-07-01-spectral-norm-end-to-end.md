# spectral-norm End-to-End Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `kali run` execute the `spectral-norm` Computer Language Benchmarks Game program and print its exact canonical output for a pinned `n = 100`, by adding a floating-point representation lane: an interprocedural int-vs-float (i64-vs-f64) inference in `kali_types`, plumbed to codegen, plus f64 arithmetic + promotion, f64 arrays, `.length`, `.fill`, runtime `Math.sqrt`, and runtime float→fixed-decimal formatting.

**Architecture:** Kali lowers TS/JS through AST → HIR → MIR → LIR → wasm (`crates/kali_{ast,hir,mir,lir,codegen}`) and runs the wasm on a wasmtime host with custom `kali:rt` imports (`crates/kali_runtime`). Codegen today receives only a `LirProgram` and emits **every** wasm param/result/local as `i64`; it re-derives all shape knowledge (string/array) structurally from the LIR. This plan adds the first analysis result plumbed across the `kali_types` → codegen boundary: a `ReprTable` (shared type in `kali_common`) computed by the resolver, carried on `ResolutionResult` → `AnalyzedSource` → `CodegenCtx` → `FunctionEmitter`. That table drives both **wasm signature/local generation** and **per-operand instruction selection** (i64 vs f64), keeping the shipped integer slice byte-identical (it has zero float seeds).

**Tech Stack:** Rust; `wasm_encoder` (module emission) and `wasmparser` (validation); `wasmtime` (execution engine); existing `kali` CLI integration tests via `Command::new(CARGO_BIN_EXE_kali)`.

**Design spec:** `docs/superpowers/specs/2026-07-01-spectral-norm-end-to-end-design.md`.

## Global Constraints

Every task's requirements implicitly include all of these (copied from the spec):

- **Value model, unchanged for integers/strings/arrays.** A JS integer is a raw two's-complement `i64` (bit 63 = 0). String handles set bit 63 (`STRING_HANDLE_TAG = 0x8000_0000_0000_0000`, packed `TAG | (offset << 32) | len`). Array handles are raw byte offsets into linear memory (bit 63 = 0), disambiguated statically. **f64 slots in as a second machine representation for `number`, chosen statically per value.** An f64 scalar is a native wasm `f64`; an array whose *elements* are float stores/loads `f64` at each slot, but the array **handle stays i64** (offsets are never f64).
- **Default int; float only where inferred.** Every `number`-typed program point is `i64` unless the representation inference unifies it with a float seed (`/`, a float literal, `Math.sqrt`, a `.toFixed` receiver). The shipped fannkuch/integer slice has **zero float seeds**, so its lowering must stay byte-identical.
- **No float→int coercion.** spectral-norm never lets a float index an array or drive a counter. If the inference ever forces a node into both a float and an integer-only context (e.g. an array index), that stays on the current reject/gated path — do **not** silently truncate.
- **No tracing/background GC.** Allocation is bump-only (no free), reusing the existing `__heap` global. Permitted; tracing GC is not.
- **Pure-Rust toolchain.** Add no dependency that vendors/compiles C/C++/asm (`-sys`, `cc`, `build.rs` translation units). f64 formatting uses Rust `std` only.
- **AOT-only.** No language-level JIT; all lowering is ahead-of-time.
- **Engine = wasmtime.** Output runs via the existing `crates/kali_runtime` embedding; the module exports `memory` and the `__heap` global.
- **Preserve observable outputs.** Existing programs must print exactly what they print today. `cargo test --workspace` must stay green at the end of every task. Optimizer fixtures that assert on wasm *size/instruction counts* (`crates/kali_cli/tests/runtime_smoke.rs`) may shift; update those evidence numbers if and only if a real lowering change moves them, and never change an expected **stdout** string.
- **`.toFixed` conformance is bounded.** Rust `format!("{:.N}")` rounds half-to-even; ECMAScript `toFixed` rounds half up. They differ only on exact ties at the requested decimal — not hit by spectral-norm. Document this as a known limitation; do not claim full `toFixed` conformance.
- **Additive maturity claims only.** When you update `specs/19-feature-maturity.md`, describe exactly the supported f64 spectral-norm slice; do not over-claim. **Do not touch `proofs/BOUNDARY.md`.**
- **Hygiene.** `cargo clippy --workspace` clean and `cargo fmt` applied before each commit.
- **Type annotations are out of scope.** TS `: number` annotations fail name resolution (`E3100`); the port is annotation-free, matching the upstream JS submission.

---

## Pipeline Orientation (read before Task 1)

Facts established by reading the codebase (2026-07-01 state); every task relies on them. Line numbers are anchors — confirm with the surrounding code, which may have shifted by a few lines.

### Crate dependency facts (for the shared `Repr` type)
- `kali_codegen` depends on `kali_common`, `kali_error`, `kali_lir` (`crates/kali_codegen/Cargo.toml`).
- `kali_types` depends on `kali_common`, `kali_error`, `kali_ast`, `kali_lexer`, `kali_parser`, `kali_npm`.
- `kali_types` does **not** depend on codegen/hir/mir/lir. So the shared `Repr`/`ReprTable` types go in **`kali_common`** — both producer and consumer already depend on it, no cycle, no new edge.

### The compile pipeline (`crates/kali_cli/src/build/compile.rs`)
- `compile_source_file_uncached` (~line 423): `analyze_source_file` (435) → `HirLowerer::lower_statements` (443) → `MirLowerer::lower_hir_result` (453) → `LirLowerer::lower_program` (457) → `optimizer.optimize_program_with_mir` (473) → `CodegenCtx::new(...)` (475) → `lower_lir_to_wasm(&mut ctx, &lir)` (481).
- `analyze_source_file` (~line 576) parses and, at 632-644, runs the resolver: `TypeContext::…resolve_statements_in_file(source_path, &parsed.statements)` → `resolved`. **Today only `resolved.diagnostics` is kept (640); `resolved` and the resolver are dropped.** It returns `AnalyzedSource { statements, diagnostics }` (struct ~752-755) *before HIR/MIR/LIR exist*.
- **Consequence:** the `ReprTable` must be produced by the resolver, returned on `ResolutionResult`, carried out of `analyze_source_file` on a new `AnalyzedSource` field, then out of `compile_source_file_uncached` into `CodegenCtx`.

### `CodegenCtx` (`crates/kali_codegen/src/ctx.rs:93-108`)
```rust
pub struct CodegenCtx {
    pub target: TargetConfig,     // max_specializations, compat_eval, coverage
    pub source_path: Option<PathBuf>,
}
```
Constructed at `compile.rs:475`. This is where the repr table is added (`pub repr_table: ReprTable`).

### `kali_types` resolver (`crates/kali_types/src/resolve/mod.rs`)
- Entry: `resolve_statements_at_path` (252-274) → `push_scope(Module)` → `resolve_statement_list(statements)` (single whole-program walk) → returns `ResolutionResult { diagnostics, scopes, global_scope }`. `ResolutionResult` is defined in `crates/kali_types/src/context.rs:6-11` and derives `Clone`.
- `resolve_statement` (323) recurses into every function body; `FunctionDeclaration` arm (457-476) binds the fn name in the **parent** scope (`bind_current_scope(name)` ~465), then `push_scope(Function)` (~466), binds params, resolves the body. **The Function scope does not record its own name** — capture the name↔scope association yourself if needed.
- `resolve_call_expression` (`crates/kali_types/src/resolve/call.rs:5-78`) resolves callee + each arg independently (48-51); **no arg↔param or return↔call-site linkage exists** — the interprocedural edges are new.
- AST shapes: `FunctionDeclaration { name: String, params: Vec<String>, body, is_async, generator }` (`crates/kali_ast/src/declaration.rs:11-17`); `CallExpression { callee: Expression, args: Vec<Expression> }` (`crates/kali_ast/src/expression.rs:104-107`). Function-declaration params are bare `Vec<String>` (positional).
- `Scope` (`crates/kali_types/src/scope.rs:20-40`) already has `static_string_typed: IndexMap<String,bool>` (33) and `static_numeric_values: IndexMap<String,String>` (34, **no int/float distinction**). `invalidate_static_binding` (74-83) `shift_remove`s from every static map on reassignment — the flow-aware invalidation to mirror.

### Codegen consumption points
- `FunctionEmitter::new(...)` (`crates/kali_codegen/src/emitter.rs:78-93`) takes, among others, `params: &[String]`, `local_names: &[String]`. Add a per-function repr slice param here. Called per function at `lower.rs:285-300` inside `for (coverage_id, function) in all_functions.iter().enumerate()`; the function name is `function.name`.
- `ValueShape` (`emitter.rs:16-23`): `Unknown | Scalar | Boolean | String` — **no int/float variant**. `EmittedValue { produced: bool, shape: ValueShape }` (46-50).
- Name-keyed fields (`emitter.rs:67-70`): `locals: BTreeMap<String,u32>`, `bindings: BTreeMap<String,LirNodeId>`, `array_bindings: HashSet<String>`. `locals` populated in `new` (94-100) from `params` then `local_names`. Binding names come from **LIR node `text`** (the original JS identifiers), carried verbatim through HIR→MIR→LIR.
- **wasm signatures/locals are all i64 today:** function type-section generation and per-function local decls in `crates/kali_codegen/src/lower.rs` emit `ValType::I64` for every param/result/local (~lines 225-245 for the type section, ~284 for the `Function::new(vec![(count, ValType::I64)])` local decl). The repr table must flip selected slots to `ValType::F64`.
- `all_functions` = synthetic `_start` (name `"_start"`) + `collect_functions(lir)`; `FunctionPlan.name` = LIR instruction `text` (`lower.rs` `function_plan` ~684), `.params` from `Value` children `text` (~695-700), `.locals` from `collect_function_locals` (~719-730). `function_name_to_index` map (134-136) keys by `function.name` — the user's source function name, matching `FunctionDeclaration.name`.

### Binary operators (`crates/kali_codegen/src/emit/operators.rs`)
- `emit_binary(&mut self, function, node: &LirNode) -> EmittedValue` (407-608). op = `node.text.as_deref()`, operands `node.children[0]/[1]`. Both operands pushed at 463-466, then a match at 468 selects the instruction: `+`→`I64Add` (470), `-`→`I64Sub` (477), `*`→`I64Mul` (484), `/`→`I64DivS` (491), `%`→`I64RemS` (498), `== ===`→`I64Eq`+`I64ExtendI32U` (505), `< <= > >=`→`I64LtS/LeS/GtS/GeS`+`I64ExtendI32U` (521-552), `&& ||`→`I64And/I64Or` (554/561), `**`→`emit_exponentiation_expression` (567), `??` (572). **No per-operand type info today.** Unary `-` (74-82) emits `I64Const(0); …; I64Sub`. `is_string_valued` (374-391) is the structural type-predicate to mirror.
- Compound assignments (`+=` etc.) are handled in `crates/kali_codegen/src/emit/literal.rs` (~400-408), also hard-coded `I64*`.

### Arrays
- Alloc/read live in `crates/kali_codegen/src/emit/call.rs`: `resolve_array_alloc_call(id) -> Option<Option<LirNodeId>>` (2167-2179); `emit_array_allocation(function, size_arg) -> EmittedValue` (2184-2234) — `GlobalGet(0)`=`__heap`, layout `[len:i64 @ +0][elem0 @ +8]…`, length `I64Store offset:0` (2210), heap bump `GlobalSet(0)` (2216-2226), returns i64 base; `emit_dynamic_array_read` (2252-2268) → `I64Load offset:8`; `emit_array_element_address` / `emit_array_element_address_node` (2272-2299) compute `base + index*8`.
- Store path `a[i]=v` in `crates/kali_codegen/src/emit/literal.rs`: `emit_assignment` (180-415), array-element store at 248-299, `I64Store { offset:8, align:3, memory_index:0 }` (287-291), gated on `self.array_bindings.contains(&base_name)` (273).
- Array bindings registered in `crates/kali_codegen/src/emit/control_flow.rs:268-282` (`resolve_array_alloc_call` → `emit_array_allocation` → `LocalSet` → `array_bindings.insert(name)`). **All element loads/stores are i64.**

### `__heap` + host helpers
- `__heap`: `GlobalType { val_type: I32, mutable: true }`, init `heap_base = (string_pool.next_offset + 7) & !7` (`lower.rs:323-332`), exported `export("__heap", ExportKind::Global, 0)` (263). Global index 0.
- Host bump: `crates/kali_runtime/src/host/memory.rs` `alloc_guest_string(caller, bytes) -> Result<i64>` (73-94) returns tagged handle. `STRING_HANDLE_TAG` = `0x8000_0000_0000_0000` (156). `decode_string_handle_bytes` (45-56).
- Host helper registration: `crates/kali_runtime/src/host/imports_default.rs` — `int_to_string` (623-632, `(i64)->i64`), `string_concat` (634-644, `(i64,i64)->i64`), module `"kali:rt"`.
- Import wiring: `crates/kali_codegen/src/lower.rs:138-183` — type section (138-157) + import section (158-183). Import-index constants `crates/kali_codegen/src/lib.rs:42-62`: `INT_TO_STRING_IMPORT_INDEX=17`, `STRING_CONCAT_IMPORT_INDEX=18`, `COVERAGE_HIT_IMPORT_INDEX=19`, `FUNCTION_INDEX_OFFSET=19`. Adding a fixed import shifts `COVERAGE_HIT_IMPORT_INDEX`/`FUNCTION_INDEX_OFFSET` and the conditional env-import offsets — verify with a full-suite run after that step alone.

### Math.sqrt
- `math_sqrt_constant_root(&self, arg) -> Option<i64>` (`crates/kali_codegen/src/intrinsics/math.rs:216-229`) — perfect-square i64 root only. Runtime lowering `crates/kali_codegen/src/emit/call.rs:1750-1796` (inside `if let Some(method) = self.math_member_method(&callee_node)` starting ~1319): `if method == "sqrt" || method == "cbrt"`, `Some(root)` → `I64Const(root)`, **`None` → pushes `E5 FEATURE_UNAVAILABLE` + `Unreachable` (1784-1795)**. Replace that `None` bail with an f64 path.

### console.log
- `crates/kali_runtime/src/host/io.rs:22-30` `format_console_value(caller, value: i64)`: string handle → UTF-8; else `value.to_string()`. **No float case needed** — floats reach `console.log` only after `.toFixed` produces a string handle. A raw f64 bit-pattern reaching `console_log` as i64 would print garbage, so all float values must be `.toFixed`-converted before printing (the port and micro-tests observe floats via boolean comparisons or `.toFixed`, never raw).

### Test harness (`crates/kali_cli/tests/`)
- Micro-acceptance file `imperative_core_runtime.rs`: `run_js(source) -> String` (9-30) writes to `tempdir()/main.js`, runs `kali run`, asserts success, returns stdout. `run_js_expect_failure` (34-56) for rejection tests.
- CLBG file `clbg_fannkuch_runtime.rs`: `fixture(name)` → `tests/fixtures/benchmarks/name`; runs `kali run <fixture>`, asserts stdout; a metadata test parses the JSON and asserts `sha256-{:x}` of the source == `sourceSha256`.
- Benchmark fixture pair: `fannkuch-redux-benchmark-v1.{ts,json}`. `assert_optimization_benchmark_fixture(stem, name)` (`crates/kali_cli/tests/runtime_smoke.rs:5928-`, called from `runtime_smoke/misc.rs:1541`) validates `sourceSha256` (5950) and compiles in three modes.
- New f64 micro-tests go in `imperative_core_runtime.rs` (integer float-repr observed via boolean comparison output until `.toFixed` exists in Task 8; float `.toFixed` output thereafter).

### Repr model used throughout this plan
Two independent axes per program point, both defaulting to `I64`:
- **Scalar repr** of a binding/param/return/local: `I64` or `F64`.
- **Array element repr** of an array binding/param: `I64` or `F64` (the handle is always `I64`).

`Repr::F64` is assigned to a scalar node iff it unifies with a float seed. An array's element repr is `F64` iff any element read/written on it (in any function, after arg↔param unification) is a float scalar.

---

## Task 1: `Repr` types + union-find (in `kali_common`)

Add the shared representation types and a small union-find, with no consumers yet. Pure data-structure task, unit-tested in isolation.

**Files:**
- Create: `crates/kali_common/src/repr.rs`
- Create: `crates/kali_common/src/repr_tests.rs`
- Modify: `crates/kali_common/src/lib.rs` (register the module + re-export)

**Interfaces:**
- Produces:
  - `pub enum Repr { I64, F64 }` (derives `Clone, Copy, PartialEq, Eq, Debug, Default`; `#[default] I64`).
  - `pub struct ReprTable` with lookups:
    - `scalar(&self, func: &str, binding: &str) -> Repr` — repr of a scalar binding/param/local; defaults `I64`.
    - `array_element(&self, func: &str, binding: &str) -> Repr` — element repr of an array binding/param; defaults `I64`.
    - `return_repr(&self, func: &str) -> Repr` — the function's return-value repr; defaults `I64`.
    - `param(&self, func: &str, index: usize) -> Repr` — repr of a positional parameter; defaults `I64`.
    - setters `set_scalar/set_array_element/set_return/set_param`.
    - `pub fn is_empty(&self) -> bool` (true when no float was ever recorded — lets codegen fast-path the all-i64 case).
  - `pub struct UnionFind` over `usize` node ids: `new()`, `fresh() -> usize`, `union(a, b)`, `find(&mut self, x) -> usize`, `seed_float(&mut self, x)` / `is_float(&mut self, x) -> bool` (a node is float iff its root is seeded). Float-ness must survive `union` (union of a float set with any set stays float).

- [ ] **Step 1: Write the failing test**

Create `crates/kali_common/src/repr_tests.rs`:
```rust
use super::repr::{Repr, ReprTable, UnionFind};

#[test]
fn union_find_propagates_float_through_union() {
    let mut uf = UnionFind::new();
    let a = uf.fresh();
    let b = uf.fresh();
    let c = uf.fresh();
    uf.seed_float(a);
    uf.union(a, b); // b joins a's float set
    assert!(uf.is_float(a));
    assert!(uf.is_float(b));
    assert!(!uf.is_float(c)); // untouched node stays int
}

#[test]
fn union_find_float_survives_union_order() {
    // Seeding one member then unioning must make the whole set float
    // regardless of which node is the resulting root.
    let mut uf = UnionFind::new();
    let x = uf.fresh();
    let y = uf.fresh();
    uf.union(x, y);
    uf.seed_float(y);
    assert!(uf.is_float(x));
    assert!(uf.is_float(y));
}

#[test]
fn repr_table_defaults_int_and_records_float() {
    let mut t = ReprTable::default();
    assert_eq!(t.scalar("f", "x"), Repr::I64);
    assert_eq!(t.array_element("f", "u"), Repr::I64);
    assert_eq!(t.return_repr("f"), Repr::I64);
    assert!(t.is_empty());
    t.set_scalar("f", "x", Repr::F64);
    t.set_array_element("f", "u", Repr::F64);
    t.set_return("f", Repr::F64);
    t.set_param("f", 0, Repr::F64);
    assert_eq!(t.scalar("f", "x"), Repr::F64);
    assert_eq!(t.array_element("f", "u"), Repr::F64);
    assert_eq!(t.return_repr("f"), Repr::F64);
    assert_eq!(t.param("f", 0), Repr::F64);
    assert!(!t.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_common repr_tests 2>&1 | head -20`
Expected: FAIL — `unresolved import super::repr` / module does not exist.

- [ ] **Step 3: Implement `repr.rs`**

Create `crates/kali_common/src/repr.rs`:
```rust
//! Shared integer-vs-float representation model for the `number` type.
//!
//! Every `number`-typed program point is `I64` unless the representation
//! inference in `kali_types` unifies it with a float seed. The resulting
//! `ReprTable` is threaded to codegen, which uses it to pick wasm signatures,
//! locals, and per-operand arithmetic instructions.

use std::collections::HashMap;

/// Machine representation chosen for a `number` value.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Repr {
    /// Two's-complement 64-bit integer (the default for every `number`).
    #[default]
    I64,
    /// IEEE-754 double.
    F64,
}

/// Representation decisions for a whole program, keyed by function + binding.
///
/// All lookups default to [`Repr::I64`]; only float decisions are stored, so an
/// empty table means "no floats anywhere" and codegen can keep its i64 fast path.
#[derive(Clone, Debug, Default)]
pub struct ReprTable {
    scalars: HashMap<(String, String), Repr>,
    array_elements: HashMap<(String, String), Repr>,
    returns: HashMap<String, Repr>,
    params: HashMap<(String, usize), Repr>,
    any_float: bool,
}

impl ReprTable {
    pub fn scalar(&self, func: &str, binding: &str) -> Repr {
        self.scalars
            .get(&(func.to_string(), binding.to_string()))
            .copied()
            .unwrap_or_default()
    }

    pub fn array_element(&self, func: &str, binding: &str) -> Repr {
        self.array_elements
            .get(&(func.to_string(), binding.to_string()))
            .copied()
            .unwrap_or_default()
    }

    pub fn return_repr(&self, func: &str) -> Repr {
        self.returns.get(func).copied().unwrap_or_default()
    }

    pub fn param(&self, func: &str, index: usize) -> Repr {
        self.params
            .get(&(func.to_string(), index))
            .copied()
            .unwrap_or_default()
    }

    pub fn set_scalar(&mut self, func: &str, binding: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        self.scalars
            .insert((func.to_string(), binding.to_string()), repr);
    }

    pub fn set_array_element(&mut self, func: &str, binding: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        self.array_elements
            .insert((func.to_string(), binding.to_string()), repr);
    }

    pub fn set_return(&mut self, func: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        self.returns.insert(func.to_string(), repr);
    }

    pub fn set_param(&mut self, func: &str, index: usize, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        self.params.insert((func.to_string(), index), repr);
    }

    /// True when no float representation was ever recorded.
    pub fn is_empty(&self) -> bool {
        !self.any_float
    }
}

/// Disjoint-set forest whose sets carry a sticky "is float" bit.
#[derive(Default)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
    float: Vec<bool>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new singleton node and return its id.
    pub fn fresh(&mut self) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.rank.push(0);
        self.float.push(false);
        id
    }

    pub fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        // Path compression.
        let mut cur = x;
        while self.parent[cur] != cur {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }

    pub fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        let float = self.float[ra] || self.float[rb];
        let root = if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
            rb
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
            ra
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
            ra
        };
        self.float[root] = float;
    }

    pub fn seed_float(&mut self, x: usize) {
        let r = self.find(x);
        self.float[r] = true;
    }

    pub fn is_float(&mut self, x: usize) -> bool {
        let r = self.find(x);
        self.float[r]
    }
}
```

- [ ] **Step 4: Register the module**

In `crates/kali_common/src/lib.rs`, add alongside the other `mod x; pub use x::*;` lines (e.g. after `mod number; pub use number::*;`):
```rust
mod repr;
pub use repr::*;
```
And add the test-module registration following the existing `*_tests` convention (search `lib.rs`/the crate for how `number_tests` is wired — it is typically `#[cfg(test)] mod repr_tests;` in `lib.rs` or a `#[path]` include; match whatever pattern `number_tests.rs` uses).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p kali_common repr_tests 2>&1 | tail -20`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**
```bash
cargo fmt && cargo clippy -p kali_common 2>&1 | tail -5
git add crates/kali_common/src/repr.rs crates/kali_common/src/repr_tests.rs crates/kali_common/src/lib.rs
git commit -m "feat(common): shared Repr/ReprTable + union-find for int-vs-float inference"
```

---

## Task 2: Representation inference in `kali_types`

Compute the `ReprTable` during the existing whole-program resolve walk: allocate a union-find node per scalar binding/param/return and per array-element, add float seeds, add intra- and inter-procedural equality edges, solve, and populate a `ReprTable`. Return it on `ResolutionResult`. No codegen consumer yet — unit-tested against source snippets.

**Files:**
- Create: `crates/kali_types/src/repr_infer.rs` (the pass)
- Create: `crates/kali_types/src/repr_infer_tests.rs`
- Modify: `crates/kali_types/src/context.rs` (add `repr_table: ReprTable` to `ResolutionResult`)
- Modify: `crates/kali_types/src/resolve/mod.rs` (run the pass in `resolve_statements_at_path`, attach to the result)
- Modify: `crates/kali_types/src/lib.rs` (register `repr_infer` + tests module)

**Interfaces:**
- Consumes: `kali_common::{Repr, ReprTable, UnionFind}`; the parsed `&[Statement]` (`kali_ast`).
- Produces:
  - `pub fn infer_reprs(statements: &[kali_ast::Statement]) -> ReprTable` (the whole-program pass).
  - `ResolutionResult.repr_table: ReprTable` (populated in `resolve_statements_at_path`).

### The inference algorithm (implement in `repr_infer.rs`)

Model each `number` program point as a union-find node. Maintain, while walking each function:
- `scalar_node: HashMap<(func, name), usize>` — one node per scalar binding/param/local, created lazily.
- `array_elem_node: HashMap<(func, name), usize>` — one node per array binding/param's element repr.
- `return_node: HashMap<func, usize>` — one node per function's return value.
- `param_node: HashMap<(func, index), usize>` — aliased to the same node as that param's `scalar_node`/`array_elem_node` entry (a param is just a binding).

**Two passes over the whole program** (functions are visited in source order, but the union-find makes ordering irrelevant):

1. **Collect nodes + intra-procedural constraints + seeds**, walking each function body with the current function name in hand:
   - For each `number` binding/param, ensure a `scalar_node`.
   - **Float seeds** (`uf.seed_float`): the result node of any `/` binary op; a numeric literal whose text contains `.`, `e`/`E`, or otherwise is non-integer-valued (a float literal — reuse `kali_common` number parsing to decide "not an exact integer"); the result of a `Math.sqrt(...)` / `Math.cbrt(...)` call; and the receiver node of a `.toFixed(...)` member call.
   - **Assignment / init edges** (`uf.union`): `let/const/var x = <expr>` and `x = <expr>` union `scalar_node[x]` with the expression's result node (flow-aware: reuse the reassignment-invalidation shape from `static_string_typed`, but for repr you *union* rather than invalidate — a variable that is ever float is float throughout, matching the spec).
   - **Arithmetic result nodes:** `+ - * ` and unary `-` produce a fresh result node unioned with **both** operand nodes (so int+float ⇒ the whole cluster is float, and the promotion is realized at codegen by converting the i64 operand). `/` produces a fresh result node that is **seeded float** (and, per JS, not unioned into its operands' int-ness — but do union operands' result cluster into the float result so that e.g. `t = t + A(i,j)*u[j]` makes `t` float). Comparisons `< <= > >= == === != !==` produce a **boolean** (always i64) result node — do not union it with operands.
   - **Array element read/write edges:** `a[i] = v` unions `array_elem_node[a]` with `v`'s result node; `a[i]` (read) produces a result node unioned with `array_elem_node[a]`. Index `i`'s node is untouched (indices stay i64).
   - **`.length`** produces an i64 result node (untouched). **`.fill(v)`** unions `array_elem_node[receiver]` with `v`'s node.
   - **`return <expr>`** unions `return_node[func]` with the expr's result node.
2. **Inter-procedural constraints** (second walk, or collected as call edges in pass 1 and unioned after all nodes exist): for every `CallExpression` whose callee is a bare identifier naming a user `FunctionDeclaration` `f`:
   - For each positional argument `k`, union the **argument's result node** with `param_node[(f, k)]`. If the argument is an array binding, union `array_elem_node[arg]` with `array_elem_node[(f, param_name_k)]` (arrays flow by element repr, not scalar). Determine per-parameter whether it is used as an array or scalar inside `f` from which node map `f` created for that param name.
   - Union the **call-site result node** with `return_node[f]` (so `A(i,j)` at a call site is float because `A`'s return is float).

   Because arrays pass by element-repr edges, `spectralnorm`'s `w` (only ever `AtAu(u,v,w)`) unions its element node with `Au`'s `v` param element node, which is float (`v[i] = t`), so `w` is correctly f64.

3. **Solve → populate `ReprTable`:** for every recorded node, `table.set_*` to `F64` iff `uf.is_float(node)`. Leave int nodes unset (they default to `I64`).

**Determinism:** iterate node maps in a stable order (e.g. `BTreeMap` or sort keys) before writing the table so output is reproducible.

- [ ] **Step 1: Write the failing test**

Create `crates/kali_types/src/repr_infer_tests.rs`. Parse source via the crate's existing parser entry (match how other `kali_types` tests parse — typically `kali_parser::parse(...)` returning statements; use the same helper the resolver tests use):
```rust
use super::repr_infer::infer_reprs;
use kali_common::Repr;

fn reprs(src: &str) -> kali_common::ReprTable {
    // Use the same parse helper the other kali_types tests use to get Vec<Statement>.
    let parsed = crate::test_support::parse_statements(src);
    infer_reprs(&parsed)
}

#[test]
fn division_is_float_addition_of_ints_is_int() {
    let t = reprs("let a = 1 + 2;\nlet b = 1 / 2;\n");
    assert_eq!(t.scalar("_start", "a"), Repr::I64);
    assert_eq!(t.scalar("_start", "b"), Repr::F64);
}

#[test]
fn float_flows_through_accumulator_reassignment() {
    // t starts int-literal 0 but accumulates a float => float throughout.
    let t = reprs("let t = 0;\nt = t + 1 / 2;\n");
    assert_eq!(t.scalar("_start", "t"), Repr::F64);
}

#[test]
fn array_element_float_from_store_and_interprocedural_param() {
    let src = "\
function store(v) { v[0] = 1 / 2; }\n\
function main() { const w = new Array(2); store(w); }\n";
    let t = reprs(src);
    // store's param v has float elements (v[0] = float).
    assert_eq!(t.array_element("store", "v"), Repr::F64);
    // w flows into store's v => w has float elements too, even though main
    // never touches w with a float op.
    assert_eq!(t.array_element("main", "w"), Repr::F64);
}

#[test]
fn function_return_repr_propagates_to_call_site() {
    let src = "\
function half(x) { return 1 / x; }\n\
function main() { let y = half(4); }\n";
    let t = reprs(src);
    assert_eq!(t.return_repr("half"), Repr::F64);
    assert_eq!(t.scalar("main", "y"), Repr::F64);
}

#[test]
fn pure_integer_program_has_empty_table() {
    let t = reprs("let s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + i; }\n");
    assert!(t.is_empty(), "integer-only program must record no floats");
}
```
> If `kali_types` has no `test_support::parse_statements`, add a tiny module-local helper in the test file that calls the same parser the resolver tests use (grep the existing `resolve` tests for the parse call and copy it). Do not invent a new parse API.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_types repr_infer 2>&1 | head -20`
Expected: FAIL — `infer_reprs` unresolved.

- [ ] **Step 3: Implement `infer_reprs`**

Create `crates/kali_types/src/repr_infer.rs` implementing the algorithm above. Structure it as a `ReprInfer { uf: UnionFind, scalar_node, array_elem_node, return_node, param_kind, calls: Vec<CallEdge>, ... }` struct with:
- `fn infer_reprs(statements: &[Statement]) -> ReprTable` — top-level: create the infer state, walk top-level statements under the synthetic function name `"_start"` (matching codegen's `_start`), then walk each `FunctionDeclaration` under its own name, then resolve `calls` interprocedurally, then emit the table.
- Per-expression `fn visit_expr(&mut self, func: &str, expr: &Expression) -> usize` returning the expr's result node (creating fresh nodes for operators/calls/reads as specified).
- Per-statement `fn visit_stmt(&mut self, func: &str, stmt: &Statement)`.
- A `CallEdge { callee: String, arg_nodes: Vec<usize>, arg_array_names: Vec<Option<(String,String)>>, result_node: usize }` recorded during `visit_expr` for each user-function call and drained after all function bodies are walked.

Match against the real `kali_ast` node kinds (`Statement`, `Expression`, `BinaryExpression`, `CallExpression`, `MemberExpression`, `VariableDeclaration`, `AssignmentExpression`, `ReturnStatement`, `FunctionDeclaration`, array-subscript/`new Array` shapes). Grep `crates/kali_ast/src/{expression,statement,declaration}.rs` for exact variant names and fields; reuse `resolve/`'s existing matchers where one already extracts, e.g., a member method name or a `new Array` call, rather than re-deriving.

For the float-literal seed, reuse `kali_common`'s number parsing (the `number` module) to classify a numeric literal token as "exact integer" vs "float"; a literal that is not an exact integer (has a fractional/exponent part) seeds float.

- [ ] **Step 4: Attach to `ResolutionResult` and run the pass**

In `crates/kali_types/src/context.rs`, add to `ResolutionResult` (struct ~6-11):
```rust
pub repr_table: kali_common::ReprTable,
```
In `crates/kali_types/src/resolve/mod.rs`, in `resolve_statements_at_path` (252-274), before constructing the result, compute:
```rust
let repr_table = crate::repr_infer::infer_reprs(statements);
```
and add `repr_table` to the returned `ResolutionResult { … }`. Update every other place that constructs a `ResolutionResult` literal (grep `ResolutionResult {`) to include `repr_table: Default::default()`.

Register the module in `crates/kali_types/src/lib.rs`: `mod repr_infer;` (+ `#[cfg(test)] mod repr_infer_tests;` following the crate's test convention). Add `pub use` if other crates need `infer_reprs` (they don't — only `ResolutionResult.repr_table` is public API).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p kali_types repr_infer 2>&1 | tail -20`
Expected: PASS (5 tests).

- [ ] **Step 6: Full-crate regression + commit**
```bash
cargo test -p kali_types 2>&1 | tail -15   # existing resolver tests still green
cargo fmt && cargo clippy -p kali_types 2>&1 | tail -5
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs crates/kali_types/src/context.rs crates/kali_types/src/resolve/mod.rs crates/kali_types/src/lib.rs
git commit -m "feat(types): interprocedural int-vs-float representation inference -> ReprTable"
```

---

## Task 3: Thread `ReprTable` to codegen; emit f64 signatures, locals, arithmetic

Plumb the table from the resolver into `CodegenCtx` and `FunctionEmitter`, make wasm function signatures/locals repr-directed (so f64 params/results/locals exist), and lower f64 scalar arithmetic + int→float promotion + f64 comparison. Observable via boolean-comparison output (no `.toFixed` yet).

**Files:**
- Modify: `crates/kali_cli/src/build/compile.rs` (carry `repr_table` out of `analyze_source_file` → `AnalyzedSource` → into `CodegenCtx`)
- Modify: `crates/kali_codegen/src/ctx.rs` (`CodegenCtx.repr_table`)
- Modify: `crates/kali_codegen/src/lower.rs` (repr-directed type-section + local decls; pass a per-function repr view into `FunctionEmitter::new`)
- Modify: `crates/kali_codegen/src/emitter.rs` (store the repr view; add `ValueShape::Float`; helper to look up a node's repr)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`emit_binary`/unary: repr-directed instruction selection + promotion)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: `kali_common::{Repr, ReprTable}`; `ResolutionResult.repr_table` (Task 2).
- Produces:
  - `CodegenCtx.repr_table: ReprTable`.
  - `FunctionEmitter` gains `repr: FnRepr<'a>` — a lightweight per-function view exposing `scalar(name) -> Repr`, `array_element(name) -> Repr`, `return_repr() -> Repr`, `param(i) -> Repr`, backed by `&ReprTable` + the current function name.
  - `ValueShape::Float` variant.
  - `wasm_type(repr: Repr) -> ValType` helper (`I64` or `F64`).

- [ ] **Step 1: Write the failing test**
```rust
#[test]
fn f64_scalar_arithmetic_observed_via_comparison() {
    // Division yields a float; 1.5 < 2 is true (=> 1).
    assert_eq!(run_js("console.log((3 / 2) < 2);\n"), "1\n");
    assert_eq!(run_js("console.log((3 / 2) < 1);\n"), "0\n");
    // int promoted into a float add: 1 + 0.5 = 1.5 < 2.
    assert_eq!(run_js("console.log((1 + 1 / 2) < 2);\n"), "1\n");
    // f64 local round-trips through local.set/get.
    assert_eq!(run_js("let x = 3 / 2;\nconsole.log(x < 2);\n"), "1\n");
    // f64-returning function + f64 param propagation across a call.
    assert_eq!(
        run_js("function half(x) { return 1 / x; }\nconsole.log(half(4) < 1);\n"),
        "1\n"
    );
}

#[test]
fn integer_programs_are_unchanged_by_repr_plumbing() {
    // Regression guard: pure-int program still prints the same.
    assert_eq!(
        run_js("let s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + i; }\nconsole.log(s);\n"),
        "10\n"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime f64_scalar_arithmetic -- --nocapture 2>&1 | tail -25`
Expected: FAIL — `3 / 2` lowers as `I64DivS` (= 1), `1 < 2` path prints `1` but `(1 + 1/2)` etc. mis-evaluate; the `half` case returns via i64. At minimum the promotion/return cases fail.

- [ ] **Step 3: Add `repr_table` to `CodegenCtx` and thread from the driver**

In `crates/kali_codegen/src/ctx.rs`, add to `CodegenCtx`:
```rust
pub repr_table: kali_common::ReprTable,
```
Initialize it in `CodegenCtx::new` to `Default::default()` (keep `new`'s signature; callers set the field after construction, matching how `source_path` is set).

In `crates/kali_cli/src/build/compile.rs`:
- Add `repr_table: kali_common::ReprTable` to `AnalyzedSource` (struct ~752-755).
- In `analyze_source_file`, capture the table from the resolver (632-644):
  ```rust
  let mut repr_table = kali_common::ReprTable::default();
  if !is_declaration_only_source_file(source_path) {
      let mut resolver = TypeContext::…;
      resolver.set_sandbox_policy_attached(sandbox_policy_attached);
      let resolved = resolver.resolve_statements_in_file(source_path, &parsed.statements);
      diagnostics.extend(resolved.diagnostics);
      if has_errors(&diagnostics) { return Err(diagnostics); }
      repr_table = resolved.repr_table;
  }
  ```
  and return it: `Ok(AnalyzedSource { statements: parsed.statements, diagnostics, repr_table })`.
- In `compile_source_file_uncached`, after `let analyzed = analyze_source_file(...)?;` and after `CodegenCtx::new(...)` (475), set:
  ```rust
  ctx.repr_table = analyzed.repr_table.clone();
  ```
  (Clone because `analyzed` may be borrowed later; if not, move it.)

- [ ] **Step 4: Repr-directed signatures + locals in `lower.rs`**

In `crates/kali_codegen/src/lower.rs`, wherever function param/result ValTypes are built for the type section (~225-245) and the per-function local decl is built (~284):
- Replace the fixed `ValType::I64` for each **param** `k` of function `name` with `wasm_type(ctx.repr_table.param(&name, k))`.
- Replace the fixed result `ValType::I64` (for a value-returning function) with `wasm_type(ctx.repr_table.return_repr(&name))`. A function that returns nothing keeps an empty result vector.
- For **locals**, the `Function::new(vec![(count, ValType::I64)])` shape must be split by repr: build the local decl as grouped `(count, ValType)` runs, where each local index's ValType is `wasm_type(ctx.repr_table.scalar(&name, local_name))` for named locals (array bindings and the scratch locals stay `I64` — array handles are i64, and the alloc scratch is i64). Preserve the existing extra scratch local(s).

Add the helper (in `lower.rs` or `emitter.rs`):
```rust
pub(crate) fn wasm_type(repr: kali_common::Repr) -> wasm_encoder::ValType {
    match repr {
        kali_common::Repr::F64 => wasm_encoder::ValType::F64,
        kali_common::Repr::I64 => wasm_encoder::ValType::I64,
    }
}
```
Pass a per-function repr view into `FunctionEmitter::new` — add a parameter `repr_table: &'a kali_common::ReprTable` and `function_name: &str` (or a prebuilt `FnRepr`). The call site (285-300) has `function.name` and `&ctx.repr_table` in scope.

- [ ] **Step 5: Store the repr view + add `ValueShape::Float`**

In `crates/kali_codegen/src/emitter.rs`:
- Extend `ValueShape` (16-23) with `Float`.
- Add fields to `FunctionEmitter`: `repr_table: &'a kali_common::ReprTable`, `function_name: String`. Populate in `new`.
- Add methods:
  ```rust
  pub(crate) fn scalar_repr(&self, name: &str) -> kali_common::Repr {
      self.repr_table.scalar(&self.function_name, name)
  }
  pub(crate) fn array_elem_repr(&self, name: &str) -> kali_common::Repr {
      self.repr_table.array_element(&self.function_name, name)
  }
  ```
- Add a structural predicate mirroring `is_string_valued`: `fn is_float_valued(&self, id: LirNodeId) -> bool` that returns true when a LIR value node's repr is F64 — for an identifier, `self.scalar_repr(name) == F64`; for a `/` op, `true`; for `+ - *`, true if either operand is float-valued; for an array read `a[i]`, `self.array_elem_repr(a) == F64`; for a call, `self.repr_table.return_repr(callee) == F64`; for a float literal, true; else false. This is the per-operand oracle `emit_binary` consults.

- [ ] **Step 6: Repr-directed `emit_binary` + promotion**

In `crates/kali_codegen/src/emit/operators.rs`, in `emit_binary` (407-608):
- Before pushing operands (463-466), compute `let float_op = self.is_float_valued(node.children[0]) || self.is_float_valued(node.children[1]);` for arithmetic/relational ops (`+ - * / % < <= > >=`). (`%` on floats is out of scope for spectral-norm; keep `%` i64.)
- When emitting **each operand**, if `float_op` and that operand is **not** itself float-valued, append `F64ConvertI64S` right after emitting it (promotion). If `float_op` and the operand is already float, emit as-is. Factor the per-operand emit so the conversion is inserted per side.
- In the instruction match (468+), when `float_op`, select the f64 instruction instead of the i64 one:
  - `+`→`F64Add`, `-`→`F64Sub`, `*`→`F64Mul`, `/`→`F64Div`.
  - `<`→`F64Lt`, `<=`→`F64Le`, `>`→`F64Gt`, `>=`→`F64Ge`, then `I64ExtendI32U` (comparison result is a boolean i64, same as the i64 path).
  - `== ===`/`!= !==` on floats → `F64Eq`/`F64Ne` + `I64ExtendI32U` (not needed by spectral-norm but cheap and correct; include for completeness).
  - Return `EmittedValue { produced: true, shape: if is_comparison { ValueShape::Boolean } else { ValueShape::Float } }` for the float arithmetic case.
- Unary `-` (74-82): if the operand is float-valued, emit `F64Neg` instead of `I64Const(0); …; I64Sub`.

> Division subtlety: `1 / x` — the repr inference seeds `/` float, so `float_op` is true; both `1` and `x` (if int) get `F64ConvertI64S`, then `F64Div`. This yields real floating division (0.25 for `1/4`), matching JS.

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test imperative_core_runtime f64_scalar_arithmetic -- --nocapture 2>&1 | tail -25`
Run: `cargo test -p kali_cli --test imperative_core_runtime integer_programs_are_unchanged -- --nocapture 2>&1 | tail -10`
Expected: PASS both.

- [ ] **Step 8: Workspace regression + commit**
```bash
cargo test --workspace 2>&1 | tail -20   # nothing else regresses; integer slice byte-identical
cargo fmt && cargo clippy --workspace 2>&1 | tail -5
git add crates/kali_cli/src/build/compile.rs crates/kali_codegen/src/ctx.rs crates/kali_codegen/src/lower.rs crates/kali_codegen/src/emitter.rs crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): plumb ReprTable; emit f64 signatures/locals + f64 arithmetic and promotion"
```

---

## Task 4: f64 arrays (repr-directed element load/store)

Make array element read/write use `F64Store`/`F64Load` when the array's element repr is `F64`, keeping i64 arrays byte-identical. The `[len:i64 @ +0][elem @ +8…]` layout and 8-byte stride are unchanged (f64 and i64 are both 8 bytes).

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (`emit_dynamic_array_read` → repr-directed load)
- Modify: `crates/kali_codegen/src/emit/literal.rs` (`emit_assignment` array-element store → repr-directed store)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (if array reads are also emitted there, 434/468 — apply the same repr-directed load)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: `self.array_elem_repr(name)` (Task 3).
- Produces: array element access selects `F64Load`/`F64Store { offset: 8, align: 3, memory_index: 0 }` vs the i64 forms, keyed on the base array binding's element repr.

- [ ] **Step 1: Write the failing test**
```rust
#[test]
fn f64_arrays_store_and_load() {
    // a is a float array (element written from a division).
    assert_eq!(
        run_js("const a = new Array(2);\na[0] = 3 / 2;\nconsole.log(a[0] < 2);\n"),
        "1\n"
    );
    // read-modify across elements stays float.
    assert_eq!(
        run_js("const a = new Array(2);\na[0] = 1 / 2;\na[1] = a[0] + a[0];\nconsole.log(a[1] < 2);\n"),
        "1\n" // 1.0 < 2
    );
    // integer arrays are unchanged.
    assert_eq!(
        run_js("const a = new Array(2);\na[0] = 10;\na[1] = 20;\nconsole.log(a[0] + a[1]);\n"),
        "30\n"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime f64_arrays_store_and_load -- --nocapture 2>&1 | tail -20`
Expected: FAIL — float elements are stored/loaded as i64 bit-nonsense; `a[0] < 2` compares a truncated/garbage value.

- [ ] **Step 3: Repr-directed store**

In `crates/kali_codegen/src/emit/literal.rs`, `emit_assignment` array-element store (248-299): where it currently emits `I64Store { offset: 8, align: 3, memory_index: 0 }` (287-291), branch on the base binding's element repr:
```rust
let elem_repr = self.array_elem_repr(&base_name);
// value already on stack; for a float array the RHS is float-valued by inference,
// so no conversion is needed here (RHS emission already produced an f64).
match elem_repr {
    kali_common::Repr::F64 => function.instruction(&Instruction::F64Store(MemArg { offset: 8, align: 3, memory_index: 0 })),
    kali_common::Repr::I64 => function.instruction(&Instruction::I64Store(MemArg { offset: 8, align: 3, memory_index: 0 })),
};
```
> The RHS value is emitted just above this store; because inference unified the array element with the RHS, the RHS is float-valued whenever `elem_repr == F64`, so its emitted value is already an `f64` on the stack (no `F64ConvertI64S` needed). If the RHS is an integer literal being stored into a float array (e.g. `.fill(1)` in Task 6, or `a[i] = 1` on a float array), that literal must be emitted as `F64Const` / converted — handle that in the literal/fill path (Task 6) and in `emit_assignment` by converting an i64-valued RHS when `elem_repr == F64`.

Add the conversion guard in the store path:
```rust
if elem_repr == kali_common::Repr::F64 && !self.is_float_valued(rhs_id) {
    function.instruction(&Instruction::F64ConvertI64S);
}
```
(emit this immediately after the RHS is pushed and before the `F64Store`).

- [ ] **Step 4: Repr-directed load**

In `crates/kali_codegen/src/emit/call.rs`, `emit_dynamic_array_read` (2252-2268): where it emits `I64Load { offset: 8, … }`, branch on the base array's element repr and emit `F64Load` for `F64`. The function needs the base binding name to look up the repr — it already resolves the base to read from `array_bindings`; use that name with `self.array_elem_repr(name)`. Apply the same branch anywhere else an array element is loaded (`control_flow.rs` 434/468 if applicable — grep for `I64Load` with `offset: 8`).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test imperative_core_runtime f64_arrays_store_and_load -- --nocapture 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 6: Workspace regression + commit**
```bash
cargo test --workspace 2>&1 | tail -20
cargo fmt && cargo clippy --workspace 2>&1 | tail -5
git add crates/kali_codegen/src/emit/call.rs crates/kali_codegen/src/emit/literal.rs crates/kali_codegen/src/emit/control_flow.rs crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): repr-directed f64 array element load/store"
```

---

## Task 5: `.length`

Add `a.length` → load the array's length header at `offset 0`. Works for i64 and f64 arrays alike (the length is always an i64 header).

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` or `crates/kali_codegen/src/emit/operators.rs` (wherever member access `a.length` is lowered — grep for existing `"length"` handling; today `.length` on a runtime array is unimplemented per the maturity doc)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: `array_bindings` membership; the base handle.
- Produces: `a.length` emits `<base handle i32>; I64Load { offset: 0, align: 3, memory_index: 0 }` → an i64 count. Result `ValueShape::Scalar` (i64).

- [ ] **Step 1: Write the failing test**
```rust
#[test]
fn array_length_reads_header() {
    assert_eq!(run_js("const a = new Array(3);\nconsole.log(a.length);\n"), "3\n");
    // length drives a loop bound (the spectral-norm idiom).
    assert_eq!(
        run_js("const a = new Array(4);\nlet n = 0;\nfor (let i = 0; i < a.length; i = i + 1) { n = n + 1; }\nconsole.log(n);\n"),
        "4\n"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime array_length_reads_header -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `.length` is unrecognized (member-access fallback / `E3100` or wrong value).

- [ ] **Step 3: Implement `.length`**

Find where member access `obj.prop` is lowered for a runtime value (grep `emit/call.rs`/`emit/member*`/`operators.rs` for `"length"`; there is likely a static-array `.length` fold already — extend it to the runtime array case). When the receiver is a name in `self.array_bindings` and the property is `length`:
```rust
// emit the base handle (array offset) as i32 address, then load the i64 header.
self.emit_array_base_address(function, base_id); // however the base handle is materialized elsewhere
function.instruction(&Instruction::I64Load(MemArg { offset: 0, align: 3, memory_index: 0 }));
```
Reuse the existing helper that pushes the array base address for element access (the one used by `emit_array_element_address`), but with `offset: 0` and no index term. Return `EmittedValue { produced: true, shape: ValueShape::Scalar }`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test imperative_core_runtime array_length_reads_header -- --nocapture 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Workspace regression + commit**
```bash
cargo test --workspace 2>&1 | tail -20
cargo fmt && cargo clippy --workspace 2>&1 | tail -5
git add crates/kali_codegen/src/emit/ crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): runtime array .length via header load"
```

---

## Task 6: `.fill(v)`

Lower `new Array(n).fill(v)` (and `a.fill(v)`) to an init loop storing `v` at each of the `n` element slots, with repr-directed element width. In the spectral-norm port this is `new Array(n).fill(1)` on a float array, so `1` is stored as `1.0`.

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (recognize `.fill(v)` on an array receiver; emit the fill loop) and/or `crates/kali_codegen/src/emit/control_flow.rs` (declarator-init path, so `const u = new Array(n).fill(1)` registers `u` as an array binding AND fills it)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: `emit_array_allocation` (Task's existing alloc), `array_elem_repr`.
- Produces: a `.fill(v)` call whose receiver is a freshly allocated (or bound) array writes `v` into every slot `0..len` via a wasm loop; returns the array handle so `const u = new Array(n).fill(1)` binds `u` to the filled array.

- [ ] **Step 1: Write the failing test**
```rust
#[test]
fn array_fill_initializes_all_elements() {
    // integer fill
    assert_eq!(
        run_js("const a = new Array(3).fill(7);\nconsole.log(a[0] + a[1] + a[2]);\n"),
        "21\n"
    );
    // float fill: a is a float array (used in a float add), fill(1) stores 1.0.
    assert_eq!(
        run_js("const a = new Array(2).fill(1);\nconsole.log((a[0] + 1 / 2) < 2);\n"),
        "1\n" // 1.0 + 0.5 = 1.5 < 2
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime array_fill_initializes_all_elements -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `.fill` unrecognized; elements read as 0/garbage.

- [ ] **Step 3: Implement `.fill(v)`**

Recognize a member call `.fill(v)` whose receiver emits an array handle (either a fresh `new Array(n)` or an array binding). Emit:
1. Materialize the array base into a scratch i32 local (reuse the alloc scratch local from `lower.rs` `+2` reservation) and its length (from the header, `I64Load offset:0`, or from the size arg if fresh).
2. A counter local `i = 0`.
3. A `block { loop { <i >= len ⇒ br out>; addr = base + i*8; push v (as f64 or i64 per element repr, converting an i64 literal to `F64Const`/`F64ConvertI64S` when the array is float); store; i += 1; br loop } }`, mirroring the loop shape used by `emit_for_of_array_iteration` / the real-loop lowering.
4. Leave the array handle on the stack as the expression result.

For `const u = new Array(n).fill(1)` in the declarator path (`control_flow.rs:268-282`), ensure the binding is registered in `array_bindings` (as `new Array` already does) AND the fill loop runs at initialization. The element repr for the stored `v` comes from `self.array_elem_repr(binding_name)`; when `F64` and `v` is an integer literal, emit `F64Const(v as f64)` (or push `I64Const(v)` then `F64ConvertI64S`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test imperative_core_runtime array_fill_initializes_all_elements -- --nocapture 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Workspace regression + commit**
```bash
cargo test --workspace 2>&1 | tail -20
cargo fmt && cargo clippy --workspace 2>&1 | tail -5
git add crates/kali_codegen/src/emit/ crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): Array.prototype.fill as a repr-directed init loop"
```

---

## Task 7: Runtime `Math.sqrt` → `F64Sqrt`

Replace the `None` bail in the `Math.sqrt`/`cbrt` runtime path with a real f64 lowering: emit the argument as f64 and `F64Sqrt`. Keep the perfect-square constant-fold fast path.

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (the `method == "sqrt" || method == "cbrt"` block, 1750-1796; replace the `None` branch at 1784-1795)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: the argument node; `is_float_valued`.
- Produces: `Math.sqrt(x)` where `x` is not a perfect-square constant emits `<x as f64>; F64Sqrt`, a float-valued result. (`cbrt` stays constant-fold-only for now — its runtime form is out of scope; leave `cbrt`'s `None` branch as the existing diagnostic, i.e. only `sqrt` gets the runtime path.)

- [ ] **Step 1: Write the failing test**
```rust
#[test]
fn math_sqrt_runtime_f64() {
    // non-perfect-square: was FEATURE_UNAVAILABLE, now a real f64 sqrt.
    assert_eq!(run_js("console.log(Math.sqrt(2) < 2);\n"), "1\n");   // 1.414… < 2
    assert_eq!(run_js("console.log(Math.sqrt(2) < 1);\n"), "0\n");
    // perfect square still constant-folds correctly.
    assert_eq!(run_js("console.log(Math.sqrt(9) < 4);\n"), "1\n");   // 3 < 4
    // sqrt of a computed float (the spectral-norm shape).
    assert_eq!(run_js("let r = 1 / 4;\nconsole.log(Math.sqrt(r) < 1);\n"), "1\n"); // 0.5 < 1
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime math_sqrt_runtime_f64 -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `Math.sqrt(2)` hits the `None` branch → `E5 FEATURE_UNAVAILABLE` + `Unreachable` (runtime trap).

- [ ] **Step 3: Implement the runtime `F64Sqrt` path**

In `crates/kali_codegen/src/emit/call.rs`, in the `sqrt`/`cbrt` block, when `method == "sqrt"` and `math_sqrt_constant_root` returns `None`, instead of the diagnostic + `Unreachable`:
```rust
// Runtime sqrt: emit the argument as f64, then F64Sqrt.
let arg_id = /* the single argument node id, as the block already computed it */;
self.emit_node(function, arg_id); // pushes the arg
if !self.is_float_valued(arg_id) {
    function.instruction(&Instruction::F64ConvertI64S);
}
function.instruction(&Instruction::F64Sqrt);
return EmittedValue { produced: true, shape: ValueShape::Float };
```
Keep the `Some(root)` fast path (`I64Const(root)`) unchanged. Leave `cbrt`'s `None` branch as-is (still the diagnostic) since runtime cbrt is out of scope.

> Note the inference (Task 2) already seeds `Math.sqrt(...)` float, so a variable assigned `Math.sqrt(x)` is f64 and its local/return signature is f64 — consistent with this emit.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test imperative_core_runtime math_sqrt_runtime_f64 -- --nocapture 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Workspace regression + commit**
```bash
cargo test --workspace 2>&1 | tail -20
cargo fmt && cargo clippy --workspace 2>&1 | tail -5
git add crates/kali_codegen/src/emit/call.rs crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat(codegen): runtime Math.sqrt via F64Sqrt (perfect-square fold retained)"
```

---

## Task 8: `.toFixed(d)` float→string host helper

Add a `kali:rt` host import `float_to_fixed(f64, i32) -> i64` (string handle) and lower a `.toFixed(d)` member call to it. This is the output path for the benchmark's single printed line.

**Files:**
- Modify: `crates/kali_runtime/src/host/imports_default.rs` (register `float_to_fixed`, next to `int_to_string`)
- Modify: `crates/kali_codegen/src/lib.rs` (add `FLOAT_TO_FIXED_IMPORT_INDEX`; bump `COVERAGE_HIT_IMPORT_INDEX` and `FUNCTION_INDEX_OFFSET`)
- Modify: `crates/kali_codegen/src/lower.rs` (add the import's type + import entry; keep the conditional-import offset arithmetic consistent)
- Modify: `crates/kali_codegen/src/emit/call.rs` (lower `.toFixed(d)` → emit receiver as f64, digits as i32, `Call(FLOAT_TO_FIXED_IMPORT_INDEX)`)
- Test: `crates/kali_cli/tests/imperative_core_runtime.rs`

**Interfaces:**
- Consumes: the receiver node (float-valued) + the digit-count literal.
- Produces:
  - Host import `"kali:rt" "float_to_fixed" : (f64, i32) -> i64`, formatting `format!("{:.*}", digits as usize, value)` and returning `alloc_guest_string(caller, s.as_bytes())`.
  - `x.toFixed(d)` lowers to `<x as f64>; I32Const(d); Call(FLOAT_TO_FIXED_IMPORT_INDEX)`, result `ValueShape::String`.
  - New constant `FLOAT_TO_FIXED_IMPORT_INDEX = 19`; `COVERAGE_HIT_IMPORT_INDEX` and `FUNCTION_INDEX_OFFSET` become `20`.

- [ ] **Step 1: Write the failing test**
```rust
#[test]
fn to_fixed_formats_floats() {
    assert_eq!(run_js("console.log((1.5).toFixed(1));\n"), "1.5\n");
    assert_eq!(run_js("console.log((1 / 3).toFixed(6));\n"), "0.333333\n");
    assert_eq!(run_js("console.log((1 / 2).toFixed(9));\n"), "0.500000000\n");
    // integer value formatted to fixed decimals.
    assert_eq!(run_js("console.log((2 / 1).toFixed(3));\n"), "2.000\n");
    // sqrt then format (spectral-norm output shape).
    assert_eq!(run_js("console.log(Math.sqrt(2).toFixed(9));\n"), "1.414213562\n");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_cli --test imperative_core_runtime to_fixed_formats_floats -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `.toFixed` unrecognized.

- [ ] **Step 3: Register the host import**

In `crates/kali_runtime/src/host/imports_default.rs`, add next to `int_to_string` (623-632):
```rust
linker.func_wrap(
    "kali:rt",
    "float_to_fixed",
    |mut caller: Caller<'_, HostState>, value: f64, digits: i32| -> i64 {
        let d = digits.clamp(0, 100) as usize;
        let s = format!("{:.*}", d, value);
        alloc_guest_string(&mut caller, s.as_bytes()).unwrap_or(0)
    },
)?;
```
(Match the exact closure/registration style used by `int_to_string`/`string_concat` in this file — arg types, `Caller` generics, and error handling.)

- [ ] **Step 4: Add the import index + wiring**

In `crates/kali_codegen/src/lib.rs` (42-62):
```rust
pub const FLOAT_TO_FIXED_IMPORT_INDEX: u32 = 19;
pub const COVERAGE_HIT_IMPORT_INDEX: u32 = 20;   // was 19
pub const FUNCTION_INDEX_OFFSET: u32 = 20;       // was 19
```
In `crates/kali_codegen/src/lower.rs` (138-183): add a function type `(f64, i32) -> i64` to the type section and an import entry `"kali:rt" "float_to_fixed"` at index 19, shifting `coverage_hit` and any conditional imports by one. Re-audit the conditional env-import offset arithmetic (the `function_index_offset` computation at 48-118) so every downstream index still lines up. **After this step alone**, run the full suite — an off-by-one here corrupts every call index.

- [ ] **Step 5: Lower `.toFixed(d)`**

In `crates/kali_codegen/src/emit/call.rs`, recognize a member call `<recv>.toFixed(<digits>)`:
```rust
// receiver must be float-valued; digits is an integer literal.
self.emit_node(function, recv_id);
if !self.is_float_valued(recv_id) {
    function.instruction(&Instruction::F64ConvertI64S);
}
let digits = /* parse the integer-literal argument */;
function.instruction(&Instruction::I32Const(digits));
function.instruction(&Instruction::Call(FLOAT_TO_FIXED_IMPORT_INDEX));
return EmittedValue { produced: true, shape: ValueShape::String };
```
The result is a string handle, so `console.log(x.toFixed(d))` prints via the existing string-handle path — no `io.rs` change.

- [ ] **Step 6: Run tests + full suite (import-index audit)**

Run: `cargo test -p kali_cli --test imperative_core_runtime to_fixed_formats_floats -- --nocapture 2>&1 | tail -20`
Run: `cargo test --workspace 2>&1 | tail -25`
Expected: PASS; no other test regresses (confirms the import-index shift is correct).

- [ ] **Step 7: Commit**
```bash
cargo fmt && cargo clippy --workspace 2>&1 | tail -5
git add crates/kali_runtime/src/host/imports_default.rs crates/kali_codegen/src/lib.rs crates/kali_codegen/src/lower.rs crates/kali_codegen/src/emit/call.rs crates/kali_cli/tests/imperative_core_runtime.rs
git commit -m "feat: runtime float->fixed-decimal via kali:rt float_to_fixed + .toFixed lowering"
```

---

## Task 9: Vendor spectral-norm and assert end-to-end

Add the vendored fixture, the end-to-end run test, and schema-v1 benchmark metadata.

**Files:**
- Create: `crates/kali_cli/tests/fixtures/benchmarks/spectral-norm-benchmark-v1.ts`
- Create: `crates/kali_cli/tests/fixtures/benchmarks/spectral-norm-benchmark-v1.json`
- Create: `crates/kali_cli/tests/clbg_spectral_norm_runtime.rs`
- Modify: `crates/kali_cli/tests/runtime_smoke/misc.rs` (call `assert_optimization_benchmark_fixture("spectral-norm-benchmark-v1", "spectral-norm")` alongside the fannkuch call at ~1541)

**Interfaces:**
- Consumes: everything from Tasks 1-8.
- Produces: a passing `kali run` byte-match on the canonical output line and a passing metadata/compile-in-three-modes assertion.

- [ ] **Step 1: Capture the reference output**

Write the port to a scratch file and run it under Node (the reference), to pin the expected line for `n = 100`:
```bash
cat > /tmp/claude-1000/-workspace/de34f0be-09fe-4d16-9957-944f64cfa5b6/scratchpad/spectral-norm.js <<'EOF'
function A(i, j) { return 1 / ((i + j) * (i + j + 1) / 2 + i + 1); }
function Au(u, v) { for (let i = 0; i < u.length; i = i + 1) { let t = 0; for (let j = 0; j < u.length; j = j + 1) { t = t + A(i, j) * u[j]; } v[i] = t; } }
function Atu(u, v) { for (let i = 0; i < u.length; i = i + 1) { let t = 0; for (let j = 0; j < u.length; j = j + 1) { t = t + A(j, i) * u[j]; } v[i] = t; } }
function AtAu(u, v, w) { Au(u, w); Atu(w, v); }
function spectralnorm(n) {
  const u = new Array(n).fill(1); const v = new Array(n); const w = new Array(n);
  for (let i = 0; i < 10; i = i + 1) { AtAu(u, v, w); AtAu(v, u, w); }
  let vBv = 0; let vv = 0;
  for (let i = 0; i < n; i = i + 1) { vBv = vBv + u[i] * v[i]; vv = vv + v[i] * v[i]; }
  return Math.sqrt(vBv / vv);
}
console.log(spectralnorm(100).toFixed(9));
EOF
node /tmp/claude-1000/-workspace/de34f0be-09fe-4d16-9957-944f64cfa5b6/scratchpad/spectral-norm.js
```
Expected (canonical CLBG value for n=100): `1.274219991`. **Record the exact printed line** — use whatever Node prints verbatim as the expected string in Step 3.

- [ ] **Step 2: Create the vendored fixture**

Create `crates/kali_cli/tests/fixtures/benchmarks/spectral-norm-benchmark-v1.ts` with the CLBG attribution header + the exact port (annotation-free, `n = 100`):
```ts
// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// spectral-norm — idiomatic TS port of the Node.js / JavaScript submission,
// normalized to Kali's pipeline (no intrinsic tuning). Retains upstream attribution.
function A(i, j) {
  return 1 / ((i + j) * (i + j + 1) / 2 + i + 1);
}
function Au(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) {
      t = t + A(i, j) * u[j];
    }
    v[i] = t;
  }
}
function Atu(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) {
      t = t + A(j, i) * u[j];
    }
    v[i] = t;
  }
}
function AtAu(u, v, w) {
  Au(u, w);
  Atu(w, v);
}
function spectralnorm(n) {
  const u = new Array(n).fill(1);
  const v = new Array(n);
  const w = new Array(n);
  for (let i = 0; i < 10; i = i + 1) {
    AtAu(u, v, w);
    AtAu(v, u, w);
  }
  let vBv = 0;
  let vv = 0;
  for (let i = 0; i < n; i = i + 1) {
    vBv = vBv + u[i] * v[i];
    vv = vv + v[i] * v[i];
  }
  return Math.sqrt(vBv / vv);
}
console.log(spectralnorm(100).toFixed(9));
```

- [ ] **Step 3: Write the end-to-end test**

Create `crates/kali_cli/tests/clbg_spectral_norm_runtime.rs`, mirroring `clbg_fannkuch_runtime.rs`:
```rust
use std::process::Command;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

#[test]
fn spectral_norm_runs_and_matches_canonical_output() {
    let source = fixture("spectral-norm-benchmark-v1.ts");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "kali run failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "1.274219991\n", // <-- replace with the exact line Node printed in Step 1
    );
}
```

- [ ] **Step 4: Run the end-to-end test**

Run: `cargo test -p kali_cli --test clbg_spectral_norm_runtime -- --nocapture 2>&1 | tail -30`
Expected: PASS. If the value differs in the last digit(s), that is a real floating-point discrepancy to debug via `superpowers:systematic-debugging` — do **not** just rebase the expected string. Likely suspects: a missing `F64ConvertI64S` promotion, an i64 store into a float array, or associativity from evaluating an operand in the wrong repr.

- [ ] **Step 5: Create the benchmark metadata JSON**

Compute the source hash and write the fixture JSON:
```bash
python3 - <<'EOF'
import hashlib, pathlib
p = pathlib.Path("crates/kali_cli/tests/fixtures/benchmarks/spectral-norm-benchmark-v1.ts")
print("sha256-" + hashlib.sha256(p.read_bytes()).hexdigest())
EOF
```
Create `crates/kali_cli/tests/fixtures/benchmarks/spectral-norm-benchmark-v1.json` (fill in the hash from above):
```json
{
  "benchmark": "spectral-norm",
  "version": 1,
  "sourceFile": "spectral-norm-benchmark-v1.ts",
  "sourceSha256": "sha256-<hash from the command above>",
  "buildModes": ["--fast", "--release", "--release-advanced"]
}
```
Add attribution/notes fields only if the fannkuch JSON has them (match its schema exactly — compare against `fannkuch-redux-benchmark-v1.json`).

- [ ] **Step 6: Wire the compile-in-three-modes assertion**

In `crates/kali_cli/tests/runtime_smoke/misc.rs` (~1541, next to the fannkuch call), add:
```rust
assert_optimization_benchmark_fixture("spectral-norm-benchmark-v1", "spectral-norm");
```
Optionally add a metadata-consistency test to `clbg_spectral_norm_runtime.rs` mirroring `fannkuch_redux_metadata_is_consistent` (parse JSON, assert fields + recompute the sha).

- [ ] **Step 7: Run the benchmark-fixture assertion + full suite**

Run: `cargo test -p kali_cli spectral 2>&1 | tail -25`
Run: `cargo test --workspace 2>&1 | tail -25`
Expected: PASS.

- [ ] **Step 8: Commit**
```bash
cargo fmt && cargo clippy --workspace 2>&1 | tail -5
git add crates/kali_cli/tests/fixtures/benchmarks/spectral-norm-benchmark-v1.ts crates/kali_cli/tests/fixtures/benchmarks/spectral-norm-benchmark-v1.json crates/kali_cli/tests/clbg_spectral_norm_runtime.rs crates/kali_cli/tests/runtime_smoke/misc.rs
git commit -m "test(clbg): vendor spectral-norm fixture and assert canonical output end-to-end"
```

---

## Task 10: Honest maturity-doc updates and final verification

Record exactly the supported f64 slice in the feature-maturity spec, and run the full verification gate.

**Files:**
- Modify: `specs/19-feature-maturity.md` (add narrow rows for the f64 slice)
- Modify: `SPEC.md` and/or `plan/phase-24/README.md` only if they reference the CLBG lane and need the second fixture noted (match how fannkuch was recorded — grep for `fannkuch` to find the spots)

- [ ] **Step 1: Add feature-maturity rows**

In `specs/19-feature-maturity.md`, add Phase-1 rows (mirroring the fannkuch rows' honesty and phrasing) for:
- **Runtime f64 arithmetic + int→float promotion** — `+ - * /` and unary `-` on `number` values inferred to floating representation lower to `F64*`; mixed int/float operands promote the integer via `f64.convert_i64_s`. Scoped to the values the interprocedural representation inference marks float (seeded by `/`, float literals, `Math.sqrt`, `.toFixed`); everything else stays `i64`. Demonstrated by the spectral-norm end-to-end fixture.
- **f64 arrays** — `new Array(n)` whose elements are inferred float store/load `f64` at the same `[len@+0][elem@+8…]` layout; the handle stays `i64`. No growth/free.
- **`Array.prototype.length` and `Array.prototype.fill(v)`** — `.length` reads the i64 header; `.fill(v)` initializes all slots via a loop (repr-directed width). Fixed-size arrays only.
- **Runtime `Math.sqrt`** — non-constant `Math.sqrt` lowers to `f64.sqrt`; the perfect-square constant fold is retained. `cbrt` remains constant-fold-only.
- **`Number.prototype.toFixed(d)`** — runtime float→fixed-decimal via the `kali:rt float_to_fixed` host helper. **Known limitation:** formatting uses Rust half-to-even rounding, which differs from ECMAScript `toFixed` half-up only on exact ties at the requested decimal; full `toFixed` conformance is not claimed.
- Note the interprocedural int-vs-float **representation inference** (in `kali_types`, threaded to codegen via `kali_common::ReprTable`) as the mechanism, and that it is **not yet a general MIR layout pass** (future follow-up).

Keep every row scoped to this slice; do not imply general objects, growable arrays, float→int coercion, general `Math.*`, stdin, or byte stdout. **Do not touch `proofs/BOUNDARY.md`.**

- [ ] **Step 2: Final verification gate**

Run and confirm each is green:
```bash
cargo test --workspace 2>&1 | tail -30
cargo clippy --workspace 2>&1 | tail -10
cargo fmt --check 2>&1 | tail -5
cargo test -p kali_cli --test clbg_spectral_norm_runtime 2>&1 | tail -10
cargo test -p kali_cli --test clbg_fannkuch_runtime 2>&1 | tail -10   # fannkuch still byte-identical
```
Expected: all PASS; fannkuch output unchanged (proves the integer slice is untouched).

- [ ] **Step 3: Commit**
```bash
git add specs/19-feature-maturity.md SPEC.md plan/phase-24/README.md
git commit -m "docs(spec): record the f64 spectral-norm execution slice in feature maturity"
```

---

## Self-Review Notes (for the executor)

- **Spec coverage:** Task 1-2 cover the repr inference (spec §4.1); Task 3 covers f64 arithmetic + promotion + signatures (§4.2 piece 1); Task 4 covers f64 arrays (piece 2); Task 5 `.length` (piece 3); Task 6 `.fill` (piece 4); Task 7 `Math.sqrt` (piece 5); Task 8 `.toFixed` (piece 6); Task 9 the end-to-end fixture + metadata (§5 criteria 1-3); Task 10 the maturity rows + full-suite gate (§5 criterion 4, §6 honesty).
- **Observability without `.toFixed`:** Tasks 3-7 deliberately observe floats via boolean comparison output (`(expr) < k` prints `1`/`0`), because `console.log` has no raw-float path and `.toFixed` only arrives in Task 8. This keeps every task independently runtime-testable.
- **Dependency order:** 1 → 2 (types before the pass that fills them) → 3 (plumbing + arithmetic; depends on 2's table) → 4 (arrays; depends on 3's `is_float_valued`/repr view) → 5, 6 (arrays; depend on 4) → 7 (sqrt; depends on 3) → 8 (`.toFixed`; depends on 3) → 9 (needs all) → 10 (last). Execute in numeric order.
- **Highest-risk step:** Task 8 Step 4 (import-index shift) — a single off-by-one corrupts every `Call` index. The plan runs the full suite immediately after. Second-highest: Task 3's signature/local ValType generation — an f64 local declared as i64 (or vice-versa) produces a wasm validation error; the wasmparser validator will reject the module loudly rather than miscompile silently.
- **Determinism:** Task 2 must iterate node maps in a stable order before writing the table.
- **The one place to debug, not rebase:** Task 9 Step 4 — if the last decimal differs, it is a real promotion/store-width bug, not an expected-value nit.
