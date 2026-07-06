# Runtime substring + .length, F1 store gate, F2 ternary — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compile fasta's `fastaRepeat` verbatim: runtime `substring` (zero-copy handle re-tag) and runtime `.length` on ASCII-provable string values, plus the F1 fail-closed store gate and full ternary (`?:`) support.

**Architecture:** Extend Spec 1's string-seed BFS (`kali_types::repr_infer`) with substring-call nodes as string sources and a non-ASCII provenance axis solved over the same string adjacency; relax the `kali_types` E5506 substring gate and add a `.length` gate. Codegen lowers runtime substring via a new hand-emitted synthetic guest function `__substring(h, s, e) -> i64` (pure i64 ALU clamp + re-tag — NO host import, NO browser-glue change) and `.length` as `handle & 0xFFFF_FFFF`. Ternary needs only a parser production plus a HIR marker text and one codegen arm (AST/HIR/resolver/escape already handle it). Escape flow gains a substring-aliases-receiver classification in `kali_mir`.

**Tech Stack:** Rust; crates `kali_common` (ReprTable), `kali_types` (inference + gates), `kali_parser` (ternary), `kali_hir` (marker), `kali_mir` (escape), `kali_codegen` (wasm emit via `wasm-encoder`), `kali_cli` (end-to-end `kali run` tests).

## Global Constraints

- Design doc: `docs/superpowers/specs/2026-07-06-substring-runtime-design.md` (Spec 2 of the 6-spec fasta series). Spec 1 context: `docs/superpowers/plans/2026-07-06-runtime-string-value-flow.md`.
- **Fail-closed, never fail-open:** a wrong runtime result is worse than a compile error. Any receiver/bound/arm the analysis cannot prove safe must reject with a diagnostic.
- **A gate relaxation and its codegen lane land in the SAME task** (Spec 1 lesson: relaxing without codegen = miscompile; codegen without relaxing = dead code).
- **No new host imports.** The 4 hand-mirrored `kali:rt` JS import lists (`kali_runtime/src/browser/harness.rs:198,530`; `kali_cli/src/bin/cmd_build.rs:1553,1817`) stay byte-identical — verify with `git diff` on those files at the end.
- **Base-behavior invariants:** static-literal substring/`.length`/relational folds byte-identical; interned-literal `==` byte-identical; float axis untouched; `kali check`-only programs with never-called functions keep compiling.
- Handle encoding: `STRING_HANDLE_TAG (0x8000_0000_0000_0000) | offset << 32 | len` (`kali_codegen/src/lib.rs:66`); `len` is a BYTE count — hence the ASCII gate.
- Full local gate per task: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir` ; final task adds `cargo test --workspace`, `cargo clippy --workspace -- -D warnings` (CI-exact, standing rule since PR #8), `cargo fmt --all -- --check`.
- Conventional-commit messages; commit after every task.
- The synthetic top-level function name is `"_start"` in repr_infer, the resolver, and codegen.

---

### Task 1: Non-ASCII provenance API on `ReprTable`

**Files:**
- Modify: `crates/kali_common/src/repr.rs`
- Test: `crates/kali_common/src/repr_tests.rs`

**Interfaces:**
- Produces: `ReprTable::mark_string_non_ascii(func, binding)`, `ReprTable::is_string_non_ascii(func, binding) -> bool`, `ReprTable::mark_string_non_ascii_return(func)`, `ReprTable::is_string_non_ascii_return(func) -> bool` — exact mirrors of the existing concat-taint quartet at `repr.rs:131-154`.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_common/src/repr_tests.rs`:

```rust
#[test]
fn repr_table_records_non_ascii_string_provenance() {
    let mut t = kali_common::ReprTable::default();
    assert!(!t.is_string_non_ascii("_start", "s"));
    t.mark_string_non_ascii("_start", "s");
    assert!(t.is_string_non_ascii("_start", "s"));
    assert!(!t.is_string_non_ascii("_start", "other"));

    assert!(!t.is_string_non_ascii_return("f"));
    t.mark_string_non_ascii_return("f");
    assert!(t.is_string_non_ascii_return("f"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_common repr_table_records_non_ascii_string_provenance`
Expected: FAIL to compile — no method `is_string_non_ascii`.

- [ ] **Step 3: Write minimal implementation**

In `crates/kali_common/src/repr.rs`, add two fields next to `string_concat_tainted` (line 56) / `string_concat_tainted_returns` (line 58):

```rust
    /// Bindings whose string value may contain non-ASCII text (byte-length
    /// handles diverge from JS UTF-16 semantics): `(function, binding)`.
    string_non_ascii: HashSet<(String, String)>,
    /// Functions whose string return value may contain non-ASCII text.
    string_non_ascii_returns: HashSet<String>,
```

Add four methods next to the taint quartet (`repr.rs:131-154`), identical shape:

```rust
    pub fn mark_string_non_ascii(&mut self, func: &str, binding: &str) {
        self.string_non_ascii
            .insert((func.to_string(), binding.to_string()));
    }

    pub fn mark_string_non_ascii_return(&mut self, func: &str) {
        self.string_non_ascii_returns.insert(func.to_string());
    }

    pub fn is_string_non_ascii(&self, func: &str, binding: &str) -> bool {
        self.string_non_ascii
            .contains(&(func.to_string(), binding.to_string()))
    }

    pub fn is_string_non_ascii_return(&self, func: &str) -> bool {
        self.string_non_ascii_returns.contains(func)
    }
```

(Match the taint methods' exact signature/allocation style — copy them and rename. Do NOT touch `is_empty`: non-ASCII marks only ever accompany a `Repr::String` decision, which already flips `any_string`.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p kali_common repr`
Expected: PASS (new test plus all existing repr tests).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/repr.rs crates/kali_common/src/repr_tests.rs
git commit -m "feat(repr): add non-ASCII string provenance marks to ReprTable"
```

---

### Task 2: repr_infer — substring nodes as string sources + non-ASCII axis

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs`
- Test: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Consumes: Task 1's `mark_string_non_ascii*`.
- Produces: `infer_reprs` marks a binding fed by `x.substring(...)` as `Repr::String` AND concat-tainted; bindings/params/returns reached by a non-ASCII seed carry `is_string_non_ascii*`. Existing behavior for programs without substring/non-ASCII is byte-identical.

Background (verified, merged state): `ReprInfer` has `string_seeds` (line 83) and `runtime_string_nodes` (line 92, the taint seed pool — filtered to string-reachable in `emit_table` at 1557-1576 and solved with `solve_reach` over the string-axis adjacency). `visit_call`'s member-method match is at 1087-1137 with arms for `sqrt|cbrt`/`toFixed`/`fill` and a fallthrough `_` that returns a fresh i64 node — that fallthrough is why substring results are invisible today.

- [ ] **Step 1: Write the failing tests**

Add to `crates/kali_types/src/repr_infer_tests.rs` (mirror the existing `reprs(...)` helper used by `string_literal_binding_is_string_repr`):

```rust
#[test]
fn substring_result_binding_is_string_and_tainted() {
    let t = reprs("let a = \"GGCC\";\nlet s = a.substring(1, 3);\n");
    assert_eq!(t.scalar("_start", "s"), Repr::String);
    assert!(
        t.is_string_concat_tainted("_start", "s"),
        "a runtime substring result is a non-interned string and must be concat-tainted"
    );
    assert!(!t.is_string_non_ascii("_start", "s"));
}

#[test]
fn substring_flows_through_param_and_return() {
    let src = "\
function f(seq) { return seq.substring(0, 2); }\n\
let out = f(\"GGCC\");\n";
    let t = reprs(src);
    assert_eq!(t.return_repr("f"), Repr::String);
    assert_eq!(t.scalar("_start", "out"), Repr::String);
}

#[test]
fn non_ascii_literal_marks_non_ascii_through_flow() {
    let t = reprs("let a = \"héllo\";\nlet b = a + \"!\";\n");
    assert_eq!(t.scalar("_start", "a"), Repr::String);
    assert!(t.is_string_non_ascii("_start", "a"));
    assert!(t.is_string_non_ascii("_start", "b"), "non-ASCII propagates through +");
}

#[test]
fn ascii_only_flow_is_not_marked_non_ascii() {
    let t = reprs("let a = \"GG\" + \"CC\";\nlet b = a + 5;\n");
    assert!(!t.is_string_non_ascii("_start", "a"));
    assert!(!t.is_string_non_ascii("_start", "b"));
}

#[test]
fn interpolated_template_is_marked_non_ascii_fail_closed() {
    // Interpolations are not modeled as value-flow edges in repr_infer
    // (pre-existing), so an interpolated template's contents cannot be
    // proven ASCII. Fail closed.
    let t = reprs("let n = 3;\nlet a = `x${n}y`;\n");
    assert_eq!(t.scalar("_start", "a"), Repr::String);
    assert!(t.is_string_non_ascii("_start", "a"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_types repr_infer -- substring_result non_ascii ascii_only interpolated_template`
Expected: FAIL — `s` resolves `Repr::I64` (substring fallthrough), and `is_string_non_ascii` doesn't exist as behavior (always false).

- [ ] **Step 3: Write the implementation**

In `crates/kali_types/src/repr_infer.rs`:

**(a)** Add a seed vector to `ReprInfer` next to `runtime_string_nodes` (line 92):

```rust
    /// Directed reachability seeds for the NON-ASCII provenance axis:
    /// non-ASCII string literals and interpolated template results (whose
    /// interpolations are not modeled, so their contents are unprovable).
    non_ascii_seeds: Vec<usize>,
```

**(b)** In `visit_expr`'s string-literal arm (the `Expression::Literal(LiteralValue::String(..))` arm that calls `add_string_seed`), bind the value and seed non-ASCII:

```rust
            Expression::Literal(LiteralValue::String(value)) => {
                let node = self.new_node();
                self.add_string_seed(node);
                if !value.is_ascii() {
                    self.non_ascii_seeds.push(node);
                }
                node
            }
```

(If the current arm pattern ignores the payload with `_`, change it to bind `value`. If the payload type is not a plain `String`, use its string accessor — grep `LiteralValue::String` in `kali_ast` for the field shape.)

**(c)** In the `Expression::TemplateLiteral` arm (the one that calls `add_string_seed` and, when interpolated, pushes to `runtime_string_nodes` — line ~781): seed non-ASCII when any raw quasi text chunk is non-ASCII, and ALWAYS when the template has interpolations (fail-closed, contents unprovable):

```rust
                if template.expressions.is_empty() {
                    if template.quasis.iter().any(|quasi| !quasi.is_ascii()) {
                        self.non_ascii_seeds.push(node);
                    }
                } else {
                    self.non_ascii_seeds.push(node);
                }
```

(Match the actual `TemplateLiteral` field names — grep `pub struct TemplateLiteral` in `kali_ast`; quasis may be a cooked/raw pair. Use whichever text the existing arm renders.)

**(d)** In `visit_call`'s member-method match (line 1092), add a `"substring"` arm before the `_` fallthrough, next to `"toFixed"`:

```rust
                    "substring" => {
                        let recv = self.visit_expr(func, &member.object);
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        let result = self.new_node();
                        // A slice of a string is a string: receiver -> result.
                        self.add_edge(recv, result);
                        // A runtime substring result is a non-interned runtime
                        // string: taint-seed it (like `+` results). Static-
                        // foldable slices never consult the repr, so this
                        // over-approximation costs nothing there.
                        self.runtime_string_nodes.push(result);
                        result
                    }
```

**(e)** In `emit_table` (the `has_strings` block at 1557-1576), solve the third axis and return it alongside `(string, tainted)`:

```rust
            let non_ascii_seeds = std::mem::take(&mut self.non_ascii_seeds);
            let non_ascii = self.solve_reach(&string_adj, &non_ascii_seeds);
            (string, tainted, non_ascii)
```

and change the `else` arm to `(vec![false; n], vec![false; n], vec![false; n])` and the binding to `let (string, tainted, non_ascii) = ...`.

**(f)** Materialize: at EVERY site that calls `table.mark_string_concat_tainted(...)` or `mark_string_concat_tainted_return(...)` (scalars loop 1685-1700, returns loop 1726-1752, params loop 1760-1785), add the analogous non-ASCII mark guarded by the same node:

```rust
                    if non_ascii[node] {
                        table.mark_string_non_ascii(&func, &name);
                    }
```

(returns: `mark_string_non_ascii_return(&func)`; params: keyed by param NAME exactly as the taint mark at that site is.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_types repr_infer`
Expected: PASS — new tests plus every existing repr_infer test (string/float/taint behavior unchanged for substring-free programs; note `substring_result_binding_is_string_and_tainted` relies on the E5506 gate NOT running here — `reprs()` calls `infer_reprs` directly, not the resolver).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs
git commit -m "feat(repr-infer): substring results as tainted string sources; non-ASCII provenance axis"
```

---

### Task 3: Synthetic `__substring` guest function

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (FunctionPlan list at 177-236, type section at 242+, signature map ~361-368, `local_decls` at 487-530, new `emit_substring_body`)
- Modify: `crates/kali_codegen/src/emitter.rs` (index accessor next to `alloc_fn_index` at ~251-261)
- Test: existing suites (every compiled module now contains and validates `__substring`) + Task 4's e2e tests exercise its behavior.

**Interfaces:**
- Produces: a guest function `__substring(h: i64, s: i64, e: i64) -> i64` in every module, resolvable by callsites via `FunctionEmitter::substring_fn_index()`. Semantics: `len = h & 0xFFFF_FFFF`; clamp `s` and `e` to `[0, len]` (JS ToLength-style clamp for integer inputs); swap if `s > e`; return `TAG | ((off + s) << 32) | (e - s)` where `off = (h >> 32) & 0x7FFF_FFFF`. Passing `e = i64::MAX` yields "to end of string" (clamps to `len`) — the callsite default for the 0/1-arg forms.

Why a synthetic function and not inline ALU: every lowered function reserves exactly TWO trailing i64 scratch locals (`lower.rs:489-500`), the general-purpose one is clobbered by nested emissions (e.g. the `??` temp at `operators.rs:1254`), and the clamp/swap needs three concurrent temps. A hand-emitted function has its own locals — the exact pattern of the page-pool quartet (`__alloc`/`__alloc_global`/`__page_get`/`__arena_reset`, `lower.rs:177-235`), whose comment block documents that inserting synthetics is safe because ALL callee indices resolve through `function_name_to_index`.

- [ ] **Step 1: Add the FunctionPlan and index accessor**

In `crates/kali_codegen/src/lower.rs`, after the `__arena_reset` push (line 227-235) and BEFORE `all_functions.extend(function_plans)`:

```rust
    // Synthetic runtime-substring `__substring(h: i64, s: i64, e: i64) -> i64`:
    // pure-ALU zero-copy slice re-tag over a tagged string handle (Spec 2).
    // Same inert-placeholder pattern as the four allocator synthetics above;
    // body hand-emitted by `emit_substring_body`. Pass `e = i64::MAX` for the
    // "to end of string" 0/1-arg forms — the clamp folds it to `len`.
    all_functions.push(FunctionPlan {
        name: "__substring".to_string(),
        params: vec!["h".to_string(), "s".to_string(), "e".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
```

Also update the synthetic-names list at `lower.rs:37` (`&["__alloc", "__alloc_global", "__page_get", "__arena_reset"]`) to include `"__substring"` — read the list's uses first and mirror whatever exclusions the four allocator names get (export skipping, LIR-lowering skipping, etc.).

In `crates/kali_codegen/src/emitter.rs`, next to `alloc_fn_index` (~251-261), copy its exact shape:

```rust
    /// Function index of the synthetic runtime-substring helper.
    pub(crate) fn substring_fn_index(&self) -> u32 {
        self.functions["__substring"]
    }
```

(Mirror the real field/lookup used by `alloc_fn_index` — read it first; the map may be named differently than `functions`.)

- [ ] **Step 2: Wire the type/signature and locals**

In `lower.rs`:
- Type section (~242+): add a `(i64, i64, i64) -> i64` function type alongside the existing entries, and in the signature-selection match (~361-368, where `"__alloc" | "__alloc_global" | "__page_get"` pick their types) add an arm mapping `"__substring"` to the new type index. Read the existing wiring to see whether user functions share one generic signature — mirror precisely.
- `local_decls` (~521+): add an arm reserving ONE i64 temp for the swap:

```rust
        } else if function.name == "__substring" {
            local_decls.push((1, ValType::I64));
        }
```

- [ ] **Step 3: Hand-emit the body**

In `lower.rs`, next to `emit_bump_body`/`emit_page_get_body`, add (and dispatch it where those bodies are dispatched by function name — find the `match function.name` that calls `emit_bump_body`):

```rust
/// `__substring(h, s, e) -> i64`: zero-copy slice of a tagged string handle.
/// Locals: 0 = h, 1 = s, 2 = e (params), 3 = swap temp.
/// len is recomputed from `h` (2 instructions) rather than stored.
fn emit_substring_body(body: &mut Function) {
    // s = max(s, 0)
    body.instruction(&Instruction::I64Const(0));
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::I64Const(0));
    body.instruction(&Instruction::I64LtS);
    body.instruction(&Instruction::Select);
    body.instruction(&Instruction::LocalSet(1));
    // s = min(s, len)
    body.instruction(&Instruction::LocalGet(0));
    body.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    body.instruction(&Instruction::I64And);
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::LocalGet(0));
    body.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    body.instruction(&Instruction::I64And);
    body.instruction(&Instruction::I64GtS);
    body.instruction(&Instruction::Select);
    body.instruction(&Instruction::LocalSet(1));
    // e = max(e, 0)
    body.instruction(&Instruction::I64Const(0));
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::I64Const(0));
    body.instruction(&Instruction::I64LtS);
    body.instruction(&Instruction::Select);
    body.instruction(&Instruction::LocalSet(2));
    // e = min(e, len)
    body.instruction(&Instruction::LocalGet(0));
    body.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    body.instruction(&Instruction::I64And);
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::LocalGet(0));
    body.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    body.instruction(&Instruction::I64And);
    body.instruction(&Instruction::I64GtS);
    body.instruction(&Instruction::Select);
    body.instruction(&Instruction::LocalSet(2));
    // if s > e { t = s; s = e; e = t }   (JS substring swaps its bounds)
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::I64GtS);
    body.instruction(&Instruction::If(BlockType::Empty));
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::LocalSet(3));
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::LocalSet(1));
    body.instruction(&Instruction::LocalGet(3));
    body.instruction(&Instruction::LocalSet(2));
    body.instruction(&Instruction::End);
    // TAG | (off + s) << 32 | (e - s)   where off = (h >> 32) & 0x7FFF_FFFF
    body.instruction(&Instruction::LocalGet(0));
    body.instruction(&Instruction::I64Const(32));
    body.instruction(&Instruction::I64ShrU);
    body.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    body.instruction(&Instruction::I64And);
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::I64Add);
    body.instruction(&Instruction::I64Const(32));
    body.instruction(&Instruction::I64Shl);
    body.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    body.instruction(&Instruction::I64Or);
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::I64Sub);
    body.instruction(&Instruction::I64Or);
    body.instruction(&Instruction::End);
}
```

(Note the `Select` operand order: `[val1, val2, cond]` pops cond then val2 then val1, yielding `cond != 0 ? val1 : val2`. Each clamp above pushes the REPLACEMENT first, the current value second, then the condition — re-derive each one when reviewing. If wasmtime validation later complains about local indices, check whether params occupy locals 0-2 in this crate's convention — they do for `__alloc` (`size` is local 0).)

- [ ] **Step 4: Run the codegen + cli suites (validation check)**

Run: `cargo test -p kali_codegen -p kali_cli`
Expected: PASS — every existing test now compiles a module containing `__substring`; wasmtime validates it on every `kali run` test. A validation error (bad local index / type mismatch) fails loudly here.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/lower.rs crates/kali_codegen/src/emitter.rs
git commit -m "feat(codegen): synthetic __substring(h,s,e) guest function — clamp + zero-copy handle re-tag"
```

---

### Task 4: Runtime substring end-to-end — types gate + codegen lane

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs` (new predicates)
- Modify: `crates/kali_types/src/static_analysis/string.rs` (`resolve_string_substring_member_call`, lines 399-433)
- Modify: `crates/kali_codegen/src/emit/call.rs` (dispatch at line ~448, new recognizer + emitter)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`is_string_valued` ~553-601, `is_runtime_concat_string` ~613-651)
- Test: `crates/kali_cli/tests/runtime_substring_length.rs` (new)

**Interfaces:**
- Consumes: Tasks 1-3 (`is_string_non_ascii*`, repr substring flow, `substring_fn_index()`).
- Produces:
  - `kali_types` predicates `expression_repr_is_ascii_string(&Expression) -> bool` and `expression_is_int_repr_bound(&Expression) -> bool` (also consumed by Task 5's `.length` gate and Task 6's store gate).
  - Codegen `runtime_substring_call_parts(&LirNode) -> Option<(LirNodeId, Option<LirNodeId>, Option<LirNodeId>)>` (also consumed by Task 5's `.length` ordering test and referenced conceptually by Task 9).
  - `seq.substring(a, b)` / `seq.substring(a)` / `seq.substring(0, x)` compile and run for ASCII-provable string receivers with int bounds; everything else keeps E5506.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/runtime_substring_length.rs`. Copy the `run_source` helper VERBATIM from `crates/kali_cli/tests/runtime_string_value_flow.rs:8-27` (it has the `static AtomicU64` counter that fixed the macOS temp-slug collision flake — change only the slug prefix, e.g. `kali-strsub-`). Then:

```rust
#[test]
fn substring_two_arg_runtime_bounds_prints() {
    let out = run_source(
        "let a = \"GGCCAATT\";\nlet i = 2;\nconsole.log(a.substring(i, i + 4));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "CCAA\n");
}

#[test]
fn substring_one_arg_and_concat_roundtrip() {
    // The fastaRepeat wrap shape: substring-to-end + `+` + substring prefix.
    let out = run_source(
        "function wrap(seq, i) { return seq.substring(i) + seq.substring(0, i); }\nconsole.log(wrap(\"GGCCAATT\", 6));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "TTGGCCAA\n");
}

#[test]
fn substring_swaps_and_clamps_bounds_like_js() {
    // JS substring: start > end swaps; negative -> 0; > len -> len.
    let out = run_source(
        "let a = \"GGCCAATT\";\nlet hi = 99;\nlet lo = 0 - 5;\nconsole.log(a.substring(6, 2));\nconsole.log(a.substring(lo, 3));\nconsole.log(a.substring(4, hi));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "CCAA\nGGC\nAATT\n");
}

#[test]
fn chained_substring_prints() {
    let out = run_source(
        "let a = \"GGCCAATT\";\nlet i = 1;\nconsole.log(a.substring(i).substring(i));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "CCAATT\n");
}

#[test]
fn substring_on_non_ascii_receiver_is_rejected() {
    // Byte-offset slicing of non-ASCII text diverges from JS code-unit
    // semantics: must reject, never miscompile.
    let out = run_source("let a = \"héllo\";\nlet i = 1;\nconsole.log(a.substring(i, 3));\n");
    assert!(!out.status.success(), "non-ASCII receiver must be rejected");
}

#[test]
fn substring_with_float_bound_is_rejected() {
    // JS ToInteger on fractional bounds is deliberately unimplemented.
    let out = run_source("let a = \"GGCC\";\nlet f = 1 / 2;\nconsole.log(a.substring(f, 3));\n");
    assert!(!out.status.success(), "float-repr bound must be rejected");
}

#[test]
fn substring_result_equality_is_rejected() {
    // A slice is a non-interned runtime string: handle-identity == would be
    // wrong. Pin as a rejection.
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet s = a.substring(0, i);\nif (s == \"G\") { console.log(1); }\n",
    );
    assert!(!out.status.success(), "substring == must be rejected, not compared by handle");
}

#[test]
fn static_substring_fold_still_prints() {
    // Base fold lane byte-identical.
    let out = run_source("console.log(\"GGCCAATT\".substring(2, 4));\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "CC\n");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_substring_length`
Expected: the four positive tests FAIL with E5506 on stderr (non-zero exit); the three reject tests already pass (they must STAY passing); `static_substring_fold_still_prints` already passes.

- [ ] **Step 3: Add the `kali_types` predicates**

In `crates/kali_types/src/resolve/expression.rs`, next to `operand_repr_is_string` (line ~180):

```rust
    /// True when `name`'s string value may contain non-ASCII text. Checks BOTH
    /// the current-function and module scopes (over-approximate: either scope
    /// non-ASCII rejects — fail-closed against the scope-resolution ambiguity
    /// `identifier_repr_is_string` handles precisely for the String bit).
    fn identifier_string_may_be_non_ascii(&self, name: &str) -> bool {
        let func = self.current_function_name();
        self.repr_table.is_string_non_ascii(func, name)
            || self.repr_table.is_string_non_ascii("_start", name)
    }

    /// True when `expr` is proven an ASCII-only runtime string: `Repr::String`
    /// via the inference AND never reached by a non-ASCII seed. The receivers
    /// the substring/.length lanes accept. Fail-closed: unknown shapes are false.
    pub(crate) fn expression_repr_is_ascii_string(&self, expr: &Expression) -> bool {
        use kali_common::Repr;
        match expr {
            Expression::Identifier(name) => {
                self.identifier_repr_is_string(name)
                    && !self.identifier_string_may_be_non_ascii(name)
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => {
                    self.repr_table.return_repr(callee) == Repr::String
                        && !self.repr_table.is_string_non_ascii_return(callee)
                }
                // A chained substring: ASCII iff ITS receiver is.
                Expression::MemberExpression(member)
                    if member.computed_index.is_none()
                        && member.property.as_str() == "substring" =>
                {
                    self.expression_repr_is_ascii_string(&member.object)
                }
                _ => false,
            },
            Expression::ParenthesizedExpression(inner) => {
                self.expression_repr_is_ascii_string(&inner.expression)
            }
            _ => false,
        }
    }

    /// True when `arg` is safe as a runtime substring bound: provably integer-
    /// repr at runtime. Float/string/unknown shapes reject (JS ToInteger on
    /// NaN/fractions is unimplemented). Fail-closed.
    pub(crate) fn expression_is_int_repr_bound(&self, arg: &Expression) -> bool {
        use kali_common::Repr;
        match arg {
            Expression::Literal(LiteralValue::Number(n)) => n.is_finite() && n.fract() == 0.0,
            Expression::Identifier(name) => {
                let func = self.current_function_name();
                self.repr_table.scalar(func, name) == Repr::I64
                    && self.repr_table.scalar("_start", name) == Repr::I64
            }
            Expression::BinaryExpression(binary)
                if matches!(binary.operator.as_str(), "+" | "-" | "*" | "%") =>
            {
                self.expression_is_int_repr_bound(&binary.left)
                    && self.expression_is_int_repr_bound(&binary.right)
            }
            Expression::UnaryExpression(unary) if unary.operator == "-" => {
                self.expression_is_int_repr_bound(&unary.argument)
            }
            Expression::ParenthesizedExpression(inner) => {
                self.expression_is_int_repr_bound(&inner.expression)
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => {
                    self.repr_table.return_repr(callee) == Repr::I64
                }
                _ => false,
            },
            _ => false,
        }
    }
```

(Match the actual `BinaryExpression`/`UnaryExpression` field types — `operator` may be `String`; adjust `.as_str()` accordingly, mirroring `reject_unsupported_string_variable_addition`'s `expr.operator != "+"` comparison style. Untracked identifiers default `Repr::I64` — trusting that is sound because the float and string axes are complete for their kinds; anything float- or string-reachable is recorded.)

- [ ] **Step 4: Relax the substring gate**

In `crates/kali_types/src/static_analysis/string.rs`, `resolve_string_substring_member_call` (399-433): after the existing static-fold accept block (which must stay FIRST and byte-identical), insert the runtime lane before the diagnostic push:

```rust
        let receiver_is_runtime_ascii_string =
            self.expression_repr_is_ascii_string(&member.object);
        let bounds_are_int_repr = expr
            .args
            .iter()
            .all(|argument| self.expression_is_int_repr_bound(argument));
        if supported_arg_count && receiver_is_runtime_ascii_string && bounds_are_int_repr {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }
```

Extend the E5506 message's first clause to name the new lane:

```
"String.prototype.substring is unavailable unless the receiver is a statically-known ASCII string literal with statically-known finite numeric bounds, or an ASCII-provable runtime string value with integer-typed bounds, in the current direct-runtime path; non-ASCII receivers and float-typed bounds are rejected"
```

(If `resolve_string_substring_member_call` lives on a different type than the `expression.rs` impl block, the predicates are `pub(crate)` — same crate, fine. Check whether some OTHER gate also rejects `x.substring(...)` for a runtime receiver by running the Step 1 positive tests after this step alone — if a second diagnostic fires, find and narrow it the same repr-backed way before proceeding.)

- [ ] **Step 5: Codegen — recognizer, dispatch, emitter, string-oracle arms**

**(a)** In `crates/kali_codegen/src/emit/call.rs`, next to `to_fixed_call_parts` (~2462):

```rust
    /// Recognizes a RUNTIME `x.substring(a?, b?)` member call: Call node whose
    /// callee is a member node with text "substring" and a string-valued
    /// receiver. Returns (receiver, start_arg, end_arg). Static-foldable
    /// slices are handled by `resolve_static_string_substring_call` FIRST and
    /// never reach this.
    pub(crate) fn runtime_substring_call_parts(
        &self,
        node: &LirNode,
    ) -> Option<(LirNodeId, Option<LirNodeId>, Option<LirNodeId>)> {
        if node.kind != LirNodeKind::Call || !(1..=3).contains(&node.children.len()) {
            return None;
        }
        let callee = self.resolve_transparent_callable_node(node.children[0])?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("substring") {
            return None;
        }
        let receiver = callee_node.children.first().copied()?;
        if !self.is_string_valued(receiver) {
            return None;
        }
        Some((
            receiver,
            node.children.get(1).copied(),
            node.children.get(2).copied(),
        ))
    }
```

(Mirror `resolve_static_string_substring_call`'s callee unwrapping exactly — it uses `resolve_transparent_callable_node`, `intrinsics/string.rs:295-303`.)

**(b)** Dispatch immediately AFTER the static substring fold block (call.rs ~441-448):

```rust
        if let Some((receiver, start, end)) = self.runtime_substring_call_parts(node) {
            return self.emit_runtime_substring(function, receiver, start, end);
        }
```

**(c)** The emitter (same file, near `emit_to_fixed` ~2481):

```rust
    /// Runtime `x.substring(a?, b?)`: push handle + clamped-later bounds, call
    /// the synthetic `__substring`. Defaults: start 0, end i64::MAX (the
    /// helper clamps it to len — the "to end of string" 0/1-arg forms).
    fn emit_runtime_substring(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        start: Option<LirNodeId>,
        end: Option<LirNodeId>,
    ) -> EmittedValue {
        let recv = self.emit_node(function, receiver, true);
        if !recv.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        self.emit_substring_bound(function, start, 0);
        self.emit_substring_bound(function, end, i64::MAX);
        function.instruction(&Instruction::Call(self.substring_fn_index()));
        EmittedValue {
            produced: true,
            shape: ValueShape::String,
        }
    }

    /// Emits one substring bound as i64, defaulting when absent. Codegen-side
    /// fail-closed backstop behind the types gate: a float- or string-valued
    /// bound gets a diagnostic, never a silent reinterpret.
    fn emit_substring_bound(
        &mut self,
        function: &mut Function,
        arg: Option<LirNodeId>,
        default: i64,
    ) {
        let Some(arg) = arg else {
            function.instruction(&Instruction::I64Const(default));
            return;
        };
        if self.is_float_valued(arg) || self.is_string_valued(arg) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.substring bounds must be integer-typed in the current direct-runtime path".to_string(),
            ));
            function.instruction(&Instruction::I64Const(default));
            return;
        }
        let value = self.emit_node(function, arg, true);
        if !value.produced {
            function.instruction(&Instruction::I64Const(default));
        }
    }
```

(Match this file's actual diagnostic imports — grep `FEATURE_UNAVAILABLE` or `Diagnostic::error` in `kali_codegen` for the crate-local path; the `+=` reject in `emit/literal.rs:501-523` is the template.)

**(d)** In `crates/kali_codegen/src/emit/operators.rs`, add a substring arm to BOTH string oracles, placed BEFORE each one's existing generic `LirNodeKind::Call` arm:

In `is_string_valued` (~589):

```rust
            // Runtime substring: a slice of a string is a string.
            LirNodeKind::Call
                if self.runtime_substring_call_parts(node).is_some() => true,
```

In `is_runtime_concat_string` (~613-651), same guard but ALSO excluding the static fold lane (a folded slice behaves like an interned literal):

```rust
            // A runtime substring result is a non-interned runtime string.
            LirNodeKind::Call
                if self.runtime_substring_call_parts(node).is_some()
                    && self.resolve_static_string_substring_call(node).is_none() => true,
```

(If these `match` arms take `node.kind` rather than binding `node`, adapt to the surrounding style — both functions already have multi-arm matches on the node; `runtime_substring_call_parts` takes `&LirNode`, which both have in scope. Recursion terminates: the parts-recognizer recurses only into the receiver child via `is_string_valued`.)

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_substring_length`
Expected: PASS — all eight tests.

- [ ] **Step 7: Run the crate gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli`
Expected: PASS — in particular `string_substring_static_ascii` (fold lane) and `runtime_string_value_flow` (Spec 1 surface) unchanged.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types/src/resolve/expression.rs crates/kali_types/src/static_analysis/string.rs crates/kali_codegen/src/emit/call.rs crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/runtime_substring_length.rs
git commit -m "feat(strings): runtime substring on ASCII-provable receivers via __substring; relax E5506"
```

---

### Task 5: Runtime `.length` end-to-end — types gate + codegen arm

**Files:**
- Modify: `crates/kali_types/src/resolve/member.rs` (`resolve_member_expression`, lines 5-40)
- Modify: `crates/kali_types/src/resolve/expression.rs` (one small gate helper)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (the `length` block at ~820-846)
- Test: `crates/kali_cli/tests/runtime_substring_length.rs` (extend)

**Interfaces:**
- Consumes: Task 4's `expression_repr_is_ascii_string`, codegen `is_string_valued`.
- Produces: `s.length` on ASCII-provable string values compiles to `handle & 0xFFFF_FFFF` (i64, byte==code-unit count for ASCII); non-ASCII-provable string receivers reject E5506; array `.length` and the static-literal `.length` fold (`emit_unary`, which counts UTF-16 units and is correct even for non-ASCII literals) are byte-identical.

Background: today the resolver accepts any `.length` silently (`resolve_member_expression` has no length rule) and codegen's `length` block fires only for `array_bindings` (`control_flow.rs:820-846`); a string local falls through to `emit_unary`'s fallback → silent `0`. The string arm must be checked BEFORE the array interpretation because repr_infer's `visit_member` registers ANY `.length` receiver as an array binding (repr_infer.rs:1018-1022) — including string vars.

- [ ] **Step 1: Write the failing tests**

Add to `crates/kali_cli/tests/runtime_substring_length.rs`:

```rust
#[test]
fn string_param_length_prints() {
    let out = run_source(
        "function f(seq) { return seq.length; }\nconsole.log(f(\"GGCCAATT\"));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "8\n");
}

#[test]
fn substring_result_length_prints() {
    // The fastaRepeat shape: `seqi = lenOut - s.length` on a slice.
    let out = run_source(
        "let a = \"GGCCAATT\";\nlet i = 6;\nlet s = a.substring(i);\nconsole.log(10 - s.length);\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "8\n");
}

#[test]
fn non_ascii_string_length_is_rejected() {
    // handle len is a byte count; "héllo".length must be 5, the handle says 6.
    let out = run_source("let a = \"héllo\";\nlet b = a + \"\";\nconsole.log(b.length);\n");
    assert!(!out.status.success(), "non-ASCII runtime .length must be rejected");
}

#[test]
fn static_non_ascii_literal_length_still_prints_utf16_count() {
    // Base fold lane: emit_unary counts UTF-16 units — correct for literals.
    let out = run_source("console.log(\"héllo\".length);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n");
}

#[test]
fn array_length_still_prints() {
    let out = run_source("let a = [1, 2, 3];\nconsole.log(a.length);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_substring_length -- length`
Expected: `string_param_length_prints` and `substring_result_length_prints` FAIL (print `0` today — the silent lane); `non_ascii_string_length_is_rejected` FAILS (compiles and prints a wrong count today — this is the latent silent-wrong the spec §2 names); the two `still_prints` tests already PASS.

- [ ] **Step 3: Add the types gate**

In `crates/kali_types/src/resolve/expression.rs`:

```rust
    /// `.length` gate: a runtime string receiver must be ASCII-provable
    /// (handle len is a byte count; JS counts UTF-16 units — they agree only
    /// for ASCII). Static-foldable receivers stay on the base fold lane,
    /// which counts UTF-16 units and is correct for ANY literal.
    pub(crate) fn reject_unprovable_string_length(&mut self, expr: &MemberExpression) {
        if expr.computed_index.is_some() || expr.property.as_str() != "length" {
            return;
        }
        if self.resolve_static_string_expression(&expr.object).is_some() {
            return;
        }
        let object_is_string = self.expression_is_string_typed(&expr.object)
            || self.operand_repr_is_string(&expr.object);
        if object_is_string && !self.expression_repr_is_ascii_string(&expr.object) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "'.length' on a runtime string value is unavailable unless the string is ASCII-provable in the current direct-runtime path; non-ASCII strings would report a byte count, not a JS character count".to_string(),
            ));
        }
    }
```

Call it at the top of `resolve_member_expression` (`resolve/member.rs:5`), before the late-member chain:

```rust
        self.reject_unprovable_string_length(expr);
```

(`resolve_static_string_expression` may take `&mut self` — if so make the gate take `&mut self` accordingly, which it already does. Import path for `e5` matches the other gates in `static_analysis/string.rs`; if `expression.rs` doesn't already import it, mirror `reject_unsupported_string_variable_addition`'s `e3` import.)

- [ ] **Step 4: Add the codegen arm**

In `crates/kali_codegen/src/emit/control_flow.rs`, inside the `text == Some("length")` block (~line 820), BEFORE the `array_bindings` check:

```rust
                if node.text.as_deref() == Some("length") {
                    let base_id = node.children[0];
                    // Runtime string length: low 32 bits of the tagged handle.
                    // MUST win before the array interpretation — repr_infer
                    // registers any `.length` receiver as an array binding,
                    // and the array lane would read garbage memory through a
                    // tagged handle.
                    if self.is_string_valued(base_id) {
                        let base = self.emit_node(function, base_id, true);
                        if !base.produced {
                            function.instruction(&Instruction::I64Const(0));
                        }
                        function.instruction(&Instruction::I64Const(0xFFFF_FFFF));
                        function.instruction(&Instruction::I64And);
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }
                    // ... existing array_bindings path unchanged ...
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_substring_length`
Expected: PASS — all tests in the file.

- [ ] **Step 6: Run the crate gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli`
Expected: PASS (array `.length`, `process.argv.length`, static-string `.length` untouched).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_types/src/resolve/member.rs crates/kali_types/src/resolve/expression.rs crates/kali_codegen/src/emit/control_flow.rs crates/kali_cli/tests/runtime_substring_length.rs
git commit -m "feat(strings): runtime .length on ASCII-provable strings; string arm wins over array lane"
```

---

### Task 6: F1 — fail-closed runtime-string store gate

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs` (shared predicate + assignment/array-literal gates)
- Modify: `crates/kali_types/src/static_analysis/string.rs` OR `resolve/call.rs` (the `fill` gate, registered in the `resolve_call_expression` chain at `resolve/call.rs:62-76`)
- Test: `crates/kali_cli/tests/runtime_substring_length.rs` (extend)

**Interfaces:**
- Consumes: Task 4's predicates, `resolve_static_string_expression` (static fold lane detector), `operand_repr_is_string`, `expression_is_string_typed`.
- Produces: `expression_is_runtime_string_value(&Expression) -> bool`; storing a runtime string into an array element (`a[i] = v`, `a.fill(v)`), an array literal element, or an object field rejects E5506. Static-foldable strings (the join fold lane) stay green.

Background: repr_infer's store edges already carry the string axis (`add_edge`, both axes — `repr_infer.rs:928-942` for `a[i]=v`, `1111-1129` for `fill`, `693-712` for array literals, `1364-1367` for field writes), but `emit_table`'s element/field loops read only the float bit — the string silently drops to the int lane and the READ side prints handles as numbers or compares them wrongly. The gate lands in the RESOLVER (AST level) because only there can the static fold lane (`resolve_static_string_expression`) be distinguished from runtime strings.

- [ ] **Step 1: Write the failing tests**

Add to `crates/kali_cli/tests/runtime_substring_length.rs`:

```rust
#[test]
fn storing_tainted_concat_into_element_is_rejected() {
    // The F1 launder: base prints 0 for this (element read loses stringness).
    let out = run_source(
        "let x = \"x\";\nlet t = x + \"y\";\nlet arr = [0];\narr[0] = t;\nif (arr[0] == \"xy\") { console.log(1); }\n",
    );
    assert!(!out.status.success(), "runtime-string element store must be rejected");
}

#[test]
fn storing_substring_into_element_is_rejected() {
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet arr = [0];\narr[0] = a.substring(0, i);\nconsole.log(arr[0]);\n",
    );
    assert!(!out.status.success(), "substring element store must be rejected");
}

#[test]
fn storing_string_param_into_field_is_rejected() {
    let out = run_source(
        "function f(s) { let o = { v: 1 };\no.v = s;\nreturn o.v; }\nconsole.log(f(\"hi\"));\n",
    );
    assert!(!out.status.success(), "runtime-string field store must be rejected");
}

#[test]
fn array_literal_with_runtime_string_element_is_rejected() {
    let out = run_source(
        "function f(s) { let a = [s];\nreturn a.length; }\nconsole.log(f(\"hi\"));\n",
    );
    assert!(!out.status.success(), "runtime-string array-literal element must be rejected");
}

#[test]
fn fill_with_runtime_string_is_rejected() {
    let out = run_source(
        "function f(s) { let a = [0, 0];\na.fill(s);\nreturn a.length; }\nconsole.log(f(\"hi\"));\n",
    );
    assert!(!out.status.success(), "runtime-string fill must be rejected");
}

#[test]
fn static_string_array_join_stays_green() {
    // The REQUIRED-GREEN fold lane: fully static elements + static join.
    let out = run_source("const a = [\"x\", \"y\"];\nconsole.log(a.join(\",\"));\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x,y\n");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_substring_length -- store rejected green`
Expected: the five reject tests FAIL (each compiles and prints something wrong or nothing today); `static_string_array_join_stays_green` already PASSES (it must stay green after the gate too — if it fails NOW, stop and investigate before writing the gate).

- [ ] **Step 3: Write the shared predicate + gates**

In `crates/kali_types/src/resolve/expression.rs`:

```rust
    /// True when `expr` produces a RUNTIME string value — one whose handle
    /// exists only at run time (concat results, string-typed vars/params,
    /// string-returning calls, substring slices). Statically-foldable strings
    /// return false: the const-fold lane (e.g. `const a = ["x","y"]` + static
    /// `join`) must stay green, and interned-literal stores keep base
    /// behavior. The F1 store gate keys on this.
    pub(crate) fn expression_is_runtime_string_value(&mut self, expr: &Expression) -> bool {
        if self.resolve_static_string_expression(expr).is_some() {
            return false;
        }
        if self.expression_is_string_typed(expr) || self.operand_repr_is_string(expr) {
            return true;
        }
        if let Expression::CallExpression(call) = expr {
            if let Expression::MemberExpression(member) = &call.callee {
                return member.computed_index.is_none()
                    && member.property.as_str() == "substring";
            }
        }
        false
    }

    /// F1: reject storing a runtime string into an array element or object
    /// field. Element/field reads are int-lane (per-edge string-axis
    /// exclusion, Spec 1) — a stored runtime string would read back as a raw
    /// number or compare by meaningless handle identity.
    pub(crate) fn reject_runtime_string_store(&mut self, assign: &AssignmentExpression) {
        let Expression::MemberExpression(_) = &assign.left else {
            return;
        };
        if !self.expression_is_runtime_string_value(&assign.right) {
            return;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "storing a runtime string value into an array element or object field is unavailable in the current direct-runtime path; element and field reads have no string lane yet".to_string(),
        ));
    }
```

Call `self.reject_runtime_string_store(assign);` inside `resolve_expression`'s `Expression::AssignmentExpression` arm (line ~277-287), after the existing resolution calls.

In the `Expression::ArrayExpression` arm (~309-375), add an element check (adapt to the arm's actual iteration — it may already loop `elements`):

```rust
                for element in arr.elements.iter().flatten() {
                    if let kali_ast::ExpressionOrSpread::Expression(element_expr) = element {
                        if self.expression_is_runtime_string_value(element_expr) {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                "a runtime string value is unavailable as an array element in the current direct-runtime path; element reads have no string lane yet".to_string(),
                            ));
                        }
                    }
                }
```

(Mirror the exact `elements` iteration shape from repr_infer's `visit_declarator_init` at `repr_infer.rs:693-712`.)

The `fill` gate — add to `crates/kali_types/src/static_analysis/string.rs` (self-guarded like every gate there) and register it in the `resolve_call_expression` chain (`resolve/call.rs:62-76`):

```rust
    pub(crate) fn resolve_array_fill_runtime_string(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };
        if member.computed_index.is_some() || member.property.as_str() != "fill" {
            return;
        }
        let first_is_runtime_string = expr
            .args
            .first()
            .is_some_and(|argument| self.expression_is_runtime_string_value(argument));
        if first_is_runtime_string {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Array.prototype.fill with a runtime string value is unavailable in the current direct-runtime path; element reads have no string lane yet".to_string(),
            ));
        }
    }
```

(`is_some_and` with a `&mut self` closure won't borrow-check — if so, restructure to `if let Some(first) = expr.args.first() { if self.expression_is_runtime_string_value(first) { ... } }`. Same caveat in the array-literal loop.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_substring_length`
Expected: PASS — five rejects reject, the join fold stays green.

- [ ] **Step 5: Run the crate gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli`
Expected: PASS. Watch specifically for regressions in array/object fixture tests that store STATIC strings (they must stay green via the fold-lane escape hatch).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/resolve/expression.rs crates/kali_types/src/static_analysis/string.rs crates/kali_types/src/resolve/call.rs crates/kali_cli/tests/runtime_substring_length.rs
git commit -m "fix(types): F1 — reject runtime-string stores into elements/fields/array literals/fill"
```

---

### Task 7: Ternary — parser production

**Files:**
- Modify: `crates/kali_parser/src/expression/mod.rs` (lines 16-34)
- Test: the parser's existing expression test module (grep `mod_tests` under `crates/kali_parser/src/expression/` and mirror its parse-helper convention)

**Interfaces:**
- Consumes: `kali_ast::ConditionalExpression` (exists: `expression.rs:248-254`, all fields `Box<Expression>`), `TokenType::Question` / `TokenType::Colon` (`kali_lexer/src/token.rs:58,39`; `?.` is already a separate `QuestionDot` token so no ambiguity).
- Produces: `b ? x : y` parses to `Expression::ConditionalExpression { test, consequent, alternate }`, right-associative, between the binary layer and assignment. Consumed by Task 8.

Background: today `parse_assignment_expression` (mod.rs:20-34) goes `parse_binary_expression(0)` → assignment-operator check; `Question` matches nothing, so `? x : y` is left unconsumed and re-dispatched as garbage statements — the silent drop. Everything downstream (HIR lowering `kali_hir/src/lowering/expression.rs:132`, resolver arm `resolve/expression.rs:380`, repr_infer arm `repr_infer.rs:856` with merge_nodes, escape classify `escape_flow.rs:410`) already handles the variant.

- [ ] **Step 1: Write the failing tests**

Add to the parser's expression test module (mirror its existing parse-source helper):

```rust
#[test]
fn parses_conditional_expression() {
    // however this module parses a program and digs out the first declarator
    // init — mirror the neighboring binary-expression tests' helper.
    let init = first_declarator_init("let x = a ? 1 : 2;");
    let Expression::ConditionalExpression(cond) = init else {
        panic!("expected ConditionalExpression, got {init:?}");
    };
    assert!(matches!(*cond.test, Expression::Identifier(ref n) if n == "a"));
    assert!(matches!(*cond.consequent, Expression::Literal(_)));
    assert!(matches!(*cond.alternate, Expression::Literal(_)));
}

#[test]
fn conditional_is_right_associative() {
    let init = first_declarator_init("let x = a ? 1 : b ? 2 : 3;");
    let Expression::ConditionalExpression(outer) = init else {
        panic!("expected outer conditional");
    };
    assert!(
        matches!(*outer.alternate, Expression::ConditionalExpression(_)),
        "alternate must nest the second conditional"
    );
}

#[test]
fn conditional_nests_inside_assignment_rhs() {
    // `x = a ? 1 : 2` — the ternary binds tighter than `=`.
    let expr = first_expression_statement("x = a ? 1 : 2;");
    let Expression::AssignmentExpression(assign) = expr else {
        panic!("expected assignment");
    };
    assert!(matches!(assign.right, Expression::ConditionalExpression(_)));
}

#[test]
fn optional_chain_is_not_a_conditional() {
    // `a?.b` lexes QuestionDot — must stay a chain, not a ternary.
    let expr = first_expression_statement("a?.b;");
    assert!(!matches!(expr, Expression::ConditionalExpression(_)));
}
```

(Replace `first_declarator_init` / `first_expression_statement` with this module's real helpers — read two neighboring tests first and copy their extraction pattern exactly.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_parser conditional`
Expected: FAIL — the init is `Identifier("a")` (tail dropped), not a ConditionalExpression.

- [ ] **Step 3: Write the implementation**

In `crates/kali_parser/src/expression/mod.rs`, change `parse_assignment_expression`'s first line and add the production:

```rust
    pub(crate) fn parse_assignment_expression(&mut self) -> Expression {
        let left = self.parse_conditional_expression();

        let Some(operator) = self.parse_assignment_operator() else {
            return left;
        };

        let _ = self.stream.advance();
        let right = self.parse_assignment_expression();
        Expression::AssignmentExpression(Box::new(AssignmentExpression {
            operator,
            left,
            right,
        }))
    }

    /// `ConditionalExpression : ShortCircuit ('?' AssignmentExpression ':' AssignmentExpression)?`
    /// Right-associative via the recursive `parse_assignment_expression` arms.
    /// `?.` never reaches here (it lexes as `QuestionDot`).
    fn parse_conditional_expression(&mut self) -> Expression {
        let test = self.parse_binary_expression(0);

        if self.stream.current_kind() != Some(&TokenType::Question) {
            return test;
        }
        let _ = self.stream.advance();
        let consequent = self.parse_assignment_expression();
        if self.stream.current_kind() == Some(&TokenType::Colon) {
            let _ = self.stream.advance();
        }
        let alternate = self.parse_assignment_expression();
        Expression::ConditionalExpression(Box::new(ConditionalExpression {
            test: Box::new(test),
            consequent: Box::new(consequent),
            alternate: Box::new(alternate),
        }))
    }
```

Add `ConditionalExpression` to the `kali_ast` import list at the top of the file. (`current_kind()` returns `Option<&TokenType>` — the file compares via `.copied()?` in `parse_assignment_operator`; `== Some(&TokenType::Question)` works if `TokenType: PartialEq`, which the lexer derives — if not, use `matches!(self.stream.current_kind(), Some(TokenType::Question))`. For a missing `:`, this parser's house style is lenient recovery (see `parse_expression_statement`'s silent `accept(Semicolon)`); if the parser has a diagnostic-push convention for required tokens, use it instead of the silent skip — grep `expected` in the parser first.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_parser`
Expected: PASS — new tests plus the whole parser suite (nothing asserted the drop behavior; verified by exploration).

- [ ] **Step 5: Run the full crate gate — watch for newly-flowing ternaries**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_hir -p kali_mir`
Expected: PASS. NOTE: any pre-existing test source containing `?:` previously parsed degenerately (test dropped); it now produces a real ConditionalExpression that codegen still miscompiles to `0` until Task 8. If a test regresses here, it was silently wrong before — record it and confirm Task 8 fixes it rather than papering over. (Exploration found no such test, but this is the checkpoint.)

- [ ] **Step 6: Commit**

```bash
git add crates/kali_parser/src/expression/mod.rs crates/kali_parser/src/expression/mod_tests.rs
git commit -m "feat(parser): parse conditional (ternary) expressions — fixes silent ?:-tail drop"
```

(Adjust the test-file path to the module's real name.)

---

### Task 8: Ternary — HIR marker + branch-selecting codegen

**Files:**
- Modify: `crates/kali_hir/src/lowering/expression.rs` (line 132-138)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (`emit_value` dispatch at ~709-717; new `emit_conditional`)
- Modify: `crates/kali_mir/src/analysis/arena_gate_tests.rs` (stale comments at 231-235, 305-309)
- Test: `crates/kali_cli/tests/runtime_ternary.rs` (new)

**Interfaces:**
- Consumes: Task 7's parse; `emit_branch` (control_flow.rs:964-1038) as the template; `reject_string_condition` (operators.rs:660); `is_float_valued` / `is_string_valued`; `emit_float_operand` (operators.rs:852-858).
- Produces: `test ? a : b` evaluates its condition by shape-directed truthiness and exactly ONE arm (wasm `if`/`else`, NOT `select`), typed i64 or f64 by arm repr. String arms flow handles. Float+string arm mix rejects fail-closed.

Background: HIR lowers `ConditionalExpr` with `text = None` (`lowering/expression.rs:132`), MIR maps it to `MirNodeKind::Expr` and LIR to `Value` — the discriminator is erased, and `emit_value` routes any text-less Value to `emit_aggregate_literal`, which evaluates children for side effects, DROPS them, and pushes `I64Const(0)`. Fix: a marker `text` (`"?"`) at the single HIR site (text flows verbatim through MIR/LIR), and a dedicated `emit_value` arm ahead of the `text.is_none()` check ordering (the arm keys on the marker, so place it right after that check — the marker makes text `Some`).

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/runtime_ternary.rs` (copy `run_source` verbatim again, slug `kali-ternary-`):

```rust
#[test]
fn int_ternary_selects_branch() {
    let out = run_source("let a = 1;\nconsole.log(a > 0 ? 10 : 20);\nconsole.log(a < 0 ? 10 : 20);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n20\n");
}

#[test]
fn float_ternary_selects_and_prints_float() {
    let out = run_source("let a = 1;\nlet x = a > 0 ? 1.5 : 2.5;\nconsole.log(x);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1.5\n");
}

#[test]
fn mixed_int_float_arms_promote_to_float() {
    let out = run_source("let a = 0;\nlet x = a > 0 ? 1.5 : 2;\nconsole.log(x);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

#[test]
fn string_arms_ternary_prints() {
    let out = run_source("let a = 1;\nlet s = \"x\";\nconsole.log(a > 0 ? s + \"1\" : s + \"2\");\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x1\n");
}

#[test]
fn only_taken_arm_evaluates() {
    // Laziness pin: the untaken arm's side effect must not run.
    let out = run_source(
        "let n = 0;\nfunction inc() { n = n + 1;\nreturn n; }\nlet a = 1;\nlet x = a > 0 ? 5 : inc();\nconsole.log(x);\nconsole.log(n);\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n0\n");
}

#[test]
fn nested_ternary_selects() {
    let out = run_source("let a = 2;\nconsole.log(a == 1 ? 10 : a == 2 ? 20 : 30);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "20\n");
}

#[test]
fn string_and_number_arms_are_rejected() {
    // Repr conflict (merge_nodes) or codegen guard — either way: no compile.
    let out = run_source("let a = 1;\nlet s = \"x\";\nlet v = a > 0 ? s : 5;\nconsole.log(v);\n");
    assert!(!out.status.success(), "string/number arm mix must be rejected");
}

#[test]
fn string_and_float_arms_are_rejected() {
    // A float-typed result block would promote a handle to f64 — reject.
    let out = run_source("let a = 1;\nlet s = \"x\";\nconsole.log(a > 0 ? s + \"!\" : 1.5);\n");
    assert!(!out.status.success(), "string/float arm mix must be rejected");
}

#[test]
fn ternary_in_never_called_function_still_compiles() {
    let out = run_source("function unused(a) { return a > 0 ? 1 : 2; }\nconsole.log(7);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_ternary`
Expected: the positive tests FAIL (every ternary evaluates to `0` via `emit_aggregate_literal` — e.g. `int_ternary_selects_branch` prints `0\n0\n`; `only_taken_arm_evaluates` prints `5`? No — prints `0\n1\n` because BOTH arms run for side effects). The two reject tests may already fail-to-compile via `merge_nodes` — note which; they must still reject after.

- [ ] **Step 3: HIR marker**

In `crates/kali_hir/src/lowering/expression.rs:132-138`, set marker text on the node (mirror EXACTLY how the `MemberExpression` arm at 46-58 attaches its `text` — same builder method):

```rust
            Expression::ConditionalExpression(expr) => {
                // Marker text "?": MIR/LIR erase the node KIND (Expr -> Value)
                // but preserve text — this is how codegen tells a ternary from
                // an aggregate literal (both are otherwise text-less Values).
                let id = self.builder.alloc_text(HirNodeKind::ConditionalExpr, None, "?");
                push_child!(self, id, self.lower_expression(&expr.test));
                push_child!(self, id, self.lower_expression(&expr.consequent));
                push_child!(self, id, self.lower_expression(&expr.alternate));
                id
            }
```

(If the builder's text-attaching method has a different name/signature, copy the MemberExpr arm's call verbatim and substitute `"?"`.)

- [ ] **Step 4: Codegen — dispatch + `emit_conditional`**

In `crates/kali_codegen/src/emit/control_flow.rs`, `emit_value` (~709), after the `text.is_none()` check:

```rust
        // Ternary `test ? a : b` — marker text "?" set by the HIR lowering.
        if node.text.as_deref() == Some("?") && node.children.len() == 3 {
            return self.emit_conditional(function, node, want_value);
        }
```

New function next to `emit_branch` (copy its truthiness table VERBATIM — lines 986-999):

```rust
    /// `test ? consequent : alternate`: value-producing if/else. Only the
    /// taken arm evaluates (JS semantics — never `select`). Result block type
    /// is repr-directed: f64 when either arm is float-valued (the other arm
    /// promotes), i64 otherwise (ints, booleans, string handles).
    pub(crate) fn emit_conditional(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        let cond = node.children[0];
        let cons = node.children[1];
        let alt = node.children[2];

        self.reject_string_condition(cond);

        let float_result =
            want_value && (self.is_float_valued(cons) || self.is_float_valued(alt));
        let string_result =
            want_value && (self.is_string_valued(cons) || self.is_string_valued(alt));
        if float_result && string_result {
            // A float result block would reinterpret a string handle as f64.
            self.diagnostics.push(Diagnostic::error(
                e3::TYPE_MISMATCH as u32,
                "a conditional expression mixing string and float branches is unavailable in the current direct-runtime path".to_string(),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        let condition = self.emit_node(function, cond, true);
        if !condition.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        match condition.shape {
            ValueShape::Boolean => {
                function.instruction(&Instruction::I32WrapI64);
            }
            ValueShape::Scalar | ValueShape::Unknown | ValueShape::String => {
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
            }
            ValueShape::Float => {
                function.instruction(&Instruction::F64Const(0.0.into()));
                function.instruction(&Instruction::F64Ne);
            }
        }

        let if_index = self.push_control_frame(ControlFlowLabelKind::If);
        function.instruction(&Instruction::If(if want_value {
            BlockType::Result(if float_result { ValType::F64 } else { ValType::I64 })
        } else {
            BlockType::Empty
        }));
        self.emit_conditional_arm(function, cons, want_value, float_result);
        function.instruction(&Instruction::Else);
        self.emit_conditional_arm(function, alt, want_value, float_result);
        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::If);

        EmittedValue {
            produced: want_value,
            shape: if !want_value {
                ValueShape::Unknown
            } else if float_result {
                ValueShape::Float
            } else if string_result {
                ValueShape::String
            } else {
                ValueShape::Unknown
            },
        }
    }

    fn emit_conditional_arm(
        &mut self,
        function: &mut Function,
        arm: LirNodeId,
        want_value: bool,
        float_result: bool,
    ) {
        if want_value && float_result {
            // Emits the arm and inserts F64ConvertI64S when it isn't already
            // float — the same promotion `+` uses for mixed operands.
            self.emit_float_operand(function, arm);
            return;
        }
        let produced = self.emit_node(function, arm, want_value);
        if want_value && !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        } else if !want_value && produced.produced {
            function.instruction(&Instruction::Drop);
        }
    }
```

(Check `emit_float_operand`'s real signature at operators.rs:852-858 — if it returns a value or takes different args, adapt; if it doesn't pad a non-produced arm, add the `I64Const(0)`+convert fallback it uses internally. Check the `e3`/`Diagnostic` import path used elsewhere in this file — `reject_string_condition` pushes the same way. `F64Const(0.0.into())` matches emit_branch's exact form.)

- [ ] **Step 5: Update the stale MIR test comments**

In `crates/kali_mir/src/analysis/arena_gate_tests.rs` at 231-235 and 305-309, the comments say "kali_parser has no ternary surface today, so this source does not reach HirNodeKind::ConditionalExpr". The parser now produces ternaries; the hand-built HIR in those tests is still valid. Reword each comment to say the HIR is hand-built to isolate the analyzer arm (drop the "parser has no ternary" claim). No behavioral change.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_ternary`
Expected: PASS — all nine.

- [ ] **Step 7: Run the crate gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_hir -p kali_mir -p kali_parser`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_hir/src/lowering/expression.rs crates/kali_codegen/src/emit/control_flow.rs crates/kali_mir/src/analysis/arena_gate_tests.rs crates/kali_cli/tests/runtime_ternary.rs
git commit -m "feat(codegen): branch-selecting ternary via HIR marker — kills the aggregate-literal miscompile"
```

---

### Task 9: escape_flow — substring aliases its receiver

**Files:**
- Modify: `crates/kali_mir/src/analysis/escape_flow.rs` (`classify_value` CallExpr arm, lines 477-491)
- Modify: `crates/kali_mir/src/analysis/arena_gate.rs` (`arena_note_call_expr` MemberExpr arm, lines 396-409)
- Test: `crates/kali_mir/src/analysis/escape_flow_tests.rs` and/or `arena_gate_tests.rs` (mirror their hand-built-HIR conventions)

**Interfaces:**
- Consumes: HIR shape — `s.substring(a, b)` is `CallExpr` whose `children[0]` is a `MemberExpr` with `text = Some("substring")` and `children[0]` = receiver.
- Produces: a substring result classifies `ValueClass::DependsOn(receiver's nodes)` — result may-heap iff receiver may-heap; result escaping taints the receiver (backward taint through embedded nodes). A substring member call no longer counts as an unknown call in arena accounting (it retains nothing — pure ALU).

Background/soundness note: host-allocated strings (concat results) live in the never-reset GLOBAL arena (`escape_flow.rs:431-438` classifies string `+` as `Scalar` for exactly that reason), and interned literals are static — so no CURRENT string can dangle across `__arena_reset`, and this task is a PRECISION fix (substring today falls to `None => ValueClass::heap()` with empty embeds, plus `arena_note_unknown_call()` taint that can suppress arenas in functions using substring). The alias edge is nonetheless the spec's keystone (§4): it makes the invariant structural instead of incidental, so a future resettable-arena string source cannot fail open.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_mir/src/analysis/arena_gate_tests.rs` (hand-build HIR exactly as the neighboring tests at 231/305 do — read one first and copy its builder scaffolding):

```rust
#[test]
fn substring_member_call_is_not_an_unknown_call() {
    // A pure-ALU slice retains nothing: a function whose only member call is
    // `s.substring(0, 1)` must keep whatever arena eligibility it otherwise
    // has (unknown-call taint previously suppressed it).
    // Build: fn f() { let o = { v: 1 }; let s = p.substring(0, 1); return 0; }
    // -- adapt to the scaffolding of the neighboring hand-built-HIR tests and
    // assert the same fact they assert for a whitelisted call (e.g. the
    // function still qualifies for its loop/function arena, or the
    // unknown-call fact is absent from the collected facts).
    todo_build_and_assert();
}
```

and to `escape_flow_tests.rs`:

```rust
#[test]
fn substring_result_aliases_receiver_for_taint() {
    // fn f(p) { g = p.substring(0, 1) }  (g global/escaping sink)
    // The result's escape must taint `p` (param_escapes(f, 0) == true), NOT
    // vanish into an empty-embed heap class.
    todo_build_and_assert();
}
```

**These two test bodies are the one deliberately-unexpanded part of this plan:** the hand-built-HIR builder scaffolding is idiosyncratic (span/text/children tuples per node) and must be copied from the neighboring tests at implementation time, not reproduced here from memory. The FACTS to assert are fully specified in the test comments above. Write them first, watch them fail.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_mir substring`
Expected: FAIL — `param_escapes` is false today (empty embeds), and the unknown-call fact is present.

- [ ] **Step 3: Write the implementation**

**(a)** `escape_flow.rs`, `classify_value`'s `HirNodeKind::CallExpr` arm (477-491) — add the substring alias BEFORE the ident-callee target resolution:

```rust
            HirNodeKind::CallExpr => {
                // Runtime `substring` is a zero-copy slice: the result ALIASES
                // its receiver's memory. Heap-ness defers to the receiver's
                // nodes (DependsOn), and result-escape taints the receiver
                // backward through them. An interned/global-arena receiver
                // carries no nodes -> the slice stays effectively scalar.
                // (Spec 2 §4 — the soundness keystone for slice handles.)
                if let Some(callee) = node
                    .children
                    .first()
                    .map(|id| &self.nodes[id.0 as usize])
                    .filter(|callee| {
                        callee.kind == HirNodeKind::MemberExpr
                            && callee.text.as_deref() == Some("substring")
                    })
                {
                    return match callee.children.first() {
                        Some(receiver) => {
                            ValueClass::DependsOn(self.classify_value(*receiver).take_nodes())
                        }
                        None => ValueClass::heap(),
                    };
                }
                let target = node
                    .children
                    .first()
                    // ... existing body unchanged ...
```

(Check `take_nodes()` exists on `ValueClass` — the MemberExpr arm at line 520 calls it; if `DependsOn` needs a `BTreeSet` and `take_nodes` returns one, this composes directly.)

**(b)** `arena_gate.rs`, `arena_note_call_expr`'s `HirNodeKind::MemberExpr` arm (396-409):

```rust
                if is_whitelisted_host_method(base_object.as_deref(), &method) {
                    whitelisted = true;
                } else if method == "substring" {
                    // Pure-ALU slice: retains nothing, poisons nothing. The
                    // result's aliasing is modeled by classify_value.
                } else {
                    self.arena_note_unknown_call();
                }
```

(The arg loop below then runs with `known_target = None` and NOT whitelisted: substring's int bounds classify `Scalar` and are skipped, and the receiver lives inside the callee node, not in `children[1..]` — so no spurious taint. Verify by reading the loop at 420-445.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_mir`
Expected: PASS — both new tests plus the whole mir suite (`member_read_carries_base_identity_for_taint` and the arena-gate pins unchanged).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_mir/src/analysis/escape_flow.rs crates/kali_mir/src/analysis/arena_gate.rs crates/kali_mir/src/analysis/escape_flow_tests.rs crates/kali_mir/src/analysis/arena_gate_tests.rs
git commit -m "feat(escape-flow): substring result aliases its receiver; substring is not an unknown call"
```

---

### Task 10: fastaRepeat fixture + full verification gate

**Files:**
- Test: `crates/kali_cli/tests/runtime_substring_length.rs` (extend with the capstone test)

**Interfaces:**
- Consumes: everything above.
- Produces: the spec's success criterion #1 — a fastaRepeat-shaped program runs byte-for-byte against `node`.

- [ ] **Step 1: Generate the golden with node**

The fixture source (upstream fastaRepeat's exact control flow, `let`-styled, pinned n=200 — argv arrives in Spec 5):

```js
const ALU = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGACCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAATACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCAGCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGGAGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCCAGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA";
function fastaRepeat(n, seq) {
  let seqi = 0;
  let lenOut = 60;
  while (n > 0) {
    if (n < lenOut) {
      lenOut = n;
    }
    if (seqi + lenOut < seq.length) {
      console.log(seq.substring(seqi, seqi + lenOut));
      seqi = seqi + lenOut;
    } else {
      let s = seq.substring(seqi);
      seqi = lenOut - s.length;
      console.log(s + seq.substring(0, seqi));
    }
    n = n - lenOut;
  }
}
fastaRepeat(200, ALU);
```

Run: `node -e '<the source above>' > /tmp/claude-1000/-workspace/*/scratchpad/fasta-repeat-golden.txt 2>/dev/null; wc -c` (or via a scratch file). Capture stdout EXACTLY.

- [ ] **Step 2: Write the test with the node-generated golden embedded**

```rust
#[test]
fn fasta_repeat_shape_matches_node_byte_for_byte() {
    let src = r#"const ALU = "GGCC...(full string)...AAAAA";
function fastaRepeat(n, seq) {
  let seqi = 0;
  let lenOut = 60;
  while (n > 0) {
    if (n < lenOut) {
      lenOut = n;
    }
    if (seqi + lenOut < seq.length) {
      console.log(seq.substring(seqi, seqi + lenOut));
      seqi = seqi + lenOut;
    } else {
      let s = seq.substring(seqi);
      seqi = lenOut - s.length;
      console.log(s + seq.substring(0, seqi));
    }
    n = n - lenOut;
  }
}
fastaRepeat(200, ALU);
"#;
    let out = run_source(src);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let expected = "GGCCGGGCGCGG...(the node-captured golden, embedded verbatim)...";
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected);
}
```

(Embed the FULL ALU string and the FULL node-captured stdout — inline golden strings are this repo's convention, `clbg_nbody_runtime.rs`. The `kali` source and the `node -e` source must be the same bytes.)

- [ ] **Step 3: Run the test**

Run: `cargo test -p kali_cli --test runtime_substring_length fasta_repeat`
Expected: PASS byte-for-byte. If it mismatches, diff line-by-line — a wrap-boundary error means the clamp/swap in `__substring` or the `s.length` lane; debug with `systematic-debugging`, don't tweak the golden.

- [ ] **Step 4: Full workspace verification**

Run, in order, all CI-exact:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
git diff --exit-code crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs
```

Expected: all PASS; the last command confirms zero browser-glue changes (no new imports — the spec's promise).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/runtime_substring_length.rs
git commit -m "test(cli): fastaRepeat-shaped fixture runs byte-for-byte against node golden"
```

---

## Self-Review

**1. Spec coverage:**
- §1 repr layer (substring nodes as sources + taint; ASCII-provenance bit) → Tasks 1-2. ✓
- §2 gate relaxation (substring runtime lane; int-repr bounds; `.length` gate incl. the array-bias callout) → Tasks 4-5. ✓
- §3 codegen (retag ALU, clamp/swap/defaults, `.length` low-32) → Tasks 3-5. The spec sketches INLINE ALU; the plan uses a synthetic `__substring` guest function because lowered functions have only two clobberable scratch locals (`lower.rs:489`) — same instructions, same zero-copy/zero-import properties, hand-emittable with its own locals. Deviation documented in Task 3.
- §4 escape alias edges (substring → receiver; ternary → both arms) → Task 9 (substring; incl. the discovered fact that today's global-arena strings make it a precision/structural fix) + ternary arms already join in `classify_value` (verified line 410-420; nothing to do). ✓
- §5 F1 (two element-store paths = `a[i]=v` + `a.fill(v)`, array literal, field store; fold-lane green) → Task 6. ✓
- §6 F2 full ternary (parse/resolve/codegen; if-else not select; never-called compiles) → Tasks 7-8; resolver/repr arms pre-exist (verified). ✓
- §7 fail-closed matrix → every row has a test: non-ASCII receiver (T4/T5), float bound (T4), `==` on slice (T4), F1 rows (T6), arm conflict + never-called (T8), alias escalation (T9). Relational/bitwise on slices: covered structurally — the existing `emit_binary` reject at operators.rs:1010-1047 keys on `is_string_valued`, which Task 4(d) teaches about slices; no new test strictly needed but T4's `==` pin guards the family.
- §8 testing + success criteria → Tasks 4-10; browser: no imports added, `git diff` check in Task 10. ✓

**2. Placeholder scan:** Task 9's two test bodies are explicitly deferred to the neighboring hand-built-HIR scaffolding with fully-specified assertions — flagged in-plan as deliberate (the builder tuples cannot be faithfully reproduced without copying them at implementation time). All other steps carry complete code. Remaining grep-lookups (builder text method, `first_declarator_init` helper, `e5` import paths, `emit_float_operand` signature) are exact-symbol matches against the codebase with fallbacks specified — the Spec 1 plan's accepted pattern.

**3. Type consistency:** `mark/is_string_non_ascii(_return)` defined T1, consumed T2/T4/T5. `expression_repr_is_ascii_string` / `expression_is_int_repr_bound` defined T4, consumed T4/T5. `expression_is_runtime_string_value` defined and consumed T6. `runtime_substring_call_parts` defined T4(a), consumed T4(b,d). `substring_fn_index()` defined T3, consumed T4(c). `__substring` param order `(h, s, e)` consistent between T3 body (locals 0/1/2) and T4(c) push order (receiver, start, end). Ternary marker `"?"` set T8 HIR, matched T8 codegen. `EmittedValue`/`ValueShape` usages match `emitter.rs:16-65`.
