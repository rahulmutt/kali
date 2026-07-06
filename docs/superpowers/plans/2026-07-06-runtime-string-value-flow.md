# Runtime String Value Flow (fix E3200) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let string-typed variables, parameters, and function returns flow their runtime string handle through `+` and `console.log`, replacing the `E3200` rejection with real support.

**Architecture:** Add a `String` axis to the existing representation model (`kali_common::Repr`, today `I64 | F64 | Object`), inferred in `kali_types::repr_infer` by BFS reachability from string seeds over the *same* directed value-flow edge graph the float axis already uses. Codegen's `is_string_valued` consults the resulting `Repr::String` (mirroring `is_float_valued`); the front-end `E3200` gate is narrowed to reject only string sources not yet backed by that repr. The runtime plumbing (tagged handles, `string_concat`, handle-printing `console.log`) already exists and is unchanged.

**Tech Stack:** Rust; crates `kali_common` (repr model), `kali_types` (inference + resolve gates), `kali_codegen` (wasm emit), `kali_cli` (end-to-end `kali run` tests). Wasm via `wasm-encoder`; runtime via `wasmtime`.

## Global Constraints

- Design doc: `docs/superpowers/specs/2026-07-06-runtime-string-value-flow-design.md` (Spec 1 of the 6-spec fasta verbatim-upstream series).
- **Fail-closed, never fail-open:** any string source the repr axis cannot prove `Repr::String` must still be rejected (narrow the gate, never delete it). A wrong result is worse than a compile error.
- **Scope is scalar string bindings / params / returns only.** `substring` results, `Array.join`, and `for..in` dynamic keys stay gated (Specs 2–4).
- **No regressions:** the full workspace test suite (`cargo test --workspace`) must stay green — literal-rooted concat (`"x" + 3`), the float axis, and non-string programs are untouched.
- **Preserve the empty-table fast path:** a program with no strings and no floats must still yield `ReprTable::is_empty() == true`.
- Conventional-commit messages; commit after every task.
- The synthetic top-level function name is `"_start"` in both `repr_infer` (`TOP_LEVEL`) and codegen; string bindings at module scope key on `"_start"`.

---

### Task 1: Add the `String` repr axis to `kali_common`

**Files:**
- Modify: `crates/kali_common/src/repr.rs` (enum `Repr`, struct `ReprTable`, its setters, `is_empty`)
- Test: `crates/kali_common/src/repr_tests.rs`

**Interfaces:**
- Produces: `kali_common::Repr::String` variant; `ReprTable` records and returns it via the existing `scalar` / `param` / `return_repr` / `set_scalar` / `set_param` / `set_return` API; `ReprTable::is_empty()` returns `false` once any `String` repr is recorded.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_common/src/repr_tests.rs`:

```rust
#[test]
fn repr_table_records_string_and_is_non_empty() {
    let mut t = kali_common::ReprTable::default();
    assert!(t.is_empty());
    t.set_scalar("_start", "s", kali_common::Repr::String);
    assert_eq!(t.scalar("_start", "s"), kali_common::Repr::String);
    assert!(!t.is_empty(), "a string decision makes the table non-empty");
    // A string decision must not spuriously mark the program as containing floats.
    let mut t2 = kali_common::ReprTable::default();
    t2.set_return("f", kali_common::Repr::String);
    assert_eq!(t2.return_repr("f"), kali_common::Repr::String);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_common repr_table_records_string_and_is_non_empty`
Expected: FAIL to compile — `no variant named `String` found for enum `Repr``.

- [ ] **Step 3: Write minimal implementation**

In `crates/kali_common/src/repr.rs`, add the variant to the `Repr` enum:

```rust
pub enum Repr {
    #[default]
    I64,
    F64,
    Object(ShapeId),
    /// Tagged linear-memory string handle (`STRING_HANDLE_TAG | offset << 32 | len`).
    String,
}
```

Add a field to `ReprTable` next to `any_float`:

```rust
    any_float: bool,
    any_string: bool,
```

In each of `set_scalar`, `set_param`, `set_return`, add a string arm alongside the existing float arm (example for `set_scalar`; apply the same two lines to all three):

```rust
    pub fn set_scalar(&mut self, func: &str, binding: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        if repr == Repr::String {
            self.any_string = true;
        }
        self.scalars
            .insert((func.to_string(), binding.to_string()), repr);
    }
```

Update `is_empty`:

```rust
    pub fn is_empty(&self) -> bool {
        !self.any_float && !self.any_string && self.shapes.is_empty() && self.shape_conflicts.is_empty()
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p kali_common repr`
Expected: PASS (new test plus all existing `repr_tests` green).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/repr.rs crates/kali_common/src/repr_tests.rs
git commit -m "feat(repr): add Repr::String axis and any_string flag to ReprTable"
```

---

### Task 2: Infer `Repr::String` in `kali_types::repr_infer`

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs` (struct `ReprInfer`, `visit_expr`, the solve/`emit_table` section)
- Test: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Consumes: `Repr::String` (Task 1).
- Produces: `infer_reprs(statements)` returns a `ReprTable` where a scalar/param/return reached by a string seed is `Repr::String`; a program point reached by *both* a string and a float seed records a shape conflict via `ReprTable::add_shape_conflict`.

String seeds are string literals and template literals. Propagation reuses the existing `+`/assignment/param/return/callsite edges (a `+` already adds `left -> result` and `right -> result` edges, so a string operand strings the result exactly as a float operand floats it).

- [ ] **Step 1: Write the failing tests**

Add to `crates/kali_types/src/repr_infer_tests.rs`:

```rust
#[test]
fn string_literal_binding_is_string_repr() {
    let t = reprs("let s = \"hi\";\n");
    assert_eq!(t.scalar("_start", "s"), Repr::String);
}

#[test]
fn string_flows_through_concat_reassignment() {
    // a starts as a string literal and accumulates string concatenations.
    let t = reprs("let a = \"\";\na = a + \"y\";\n");
    assert_eq!(t.scalar("_start", "a"), Repr::String);
}

#[test]
fn string_flows_through_param_and_return() {
    let src = "\
function f(s) { return s + \"!\"; }\n\
let out = f(\"hi\");\n";
    let t = reprs(src);
    assert_eq!(t.param("f", 0), Repr::String);
    assert_eq!(t.return_repr("f"), Repr::String);
    assert_eq!(t.scalar("_start", "out"), Repr::String);
}

#[test]
fn plain_integer_program_has_no_string_repr() {
    let t = reprs("let a = 1 + 2;\n");
    assert_eq!(t.scalar("_start", "a"), Repr::I64);
    assert!(t.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_types string_literal_binding_is_string_repr string_flows_through_concat_reassignment string_flows_through_param_and_return`
Expected: FAIL — string bindings resolve to `Repr::I64` (default), assertions mismatch.

- [ ] **Step 3: Write the implementation**

In `crates/kali_types/src/repr_infer.rs`:

**(a)** Add a seed vector to the `ReprInfer` struct, next to `seeds: Vec<usize>`:

```rust
    /// Directed reachability seeds for the STRING axis (string/template literals).
    string_seeds: Vec<usize>,
```

**(b)** Add a seeding helper next to `add_seed`:

```rust
    fn add_string_seed(&mut self, node: usize) {
        self.string_seeds.push(node);
    }
```

**(c)** In `visit_expr`, replace the string-literal and add the template-literal seeding. The current catch-all `Expression::Literal(_) => self.new_node(),` swallows strings; split the string case out:

```rust
            Expression::Literal(LiteralValue::String(_)) => {
                let node = self.new_node();
                self.add_string_seed(node);
                node
            }
            Expression::Literal(_) => self.new_node(),
            Expression::TemplateLiteral(_) => {
                let node = self.new_node();
                self.add_string_seed(node);
                node
            }
```

(Keep the existing `Expression::Literal(LiteralValue::Number(n))` arm above this — do not remove it. If a `TemplateLiteral` arm already exists elsewhere in the match, add the `add_string_seed` seeding into it instead of adding a second arm.)

**(d)** Generalize the solver. Rename the reachability core so both axes share the adjacency build. Replace `solve_float` with a shared helper plus two callers:

```rust
    /// BFS reachability over the directed edge graph from `seed_nodes`,
    /// endpoints canonicalised through the array-element union-find. Shared by
    /// the float and string axes. Consumes nothing (adjacency rebuilt by caller).
    fn solve_reach(&mut self, adj: &[Vec<usize>], seed_nodes: &[usize]) -> Vec<bool> {
        let n = self.node_count;
        let mut hit = vec![false; n];
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        for &s in seed_nodes {
            let r = self.uf.find(s);
            if !hit[r] {
                hit[r] = true;
                queue.push_back(r);
            }
        }
        while let Some(u) = queue.pop_front() {
            for &v in &adj[u] {
                if !hit[v] {
                    hit[v] = true;
                    queue.push_back(v);
                }
            }
        }
        hit
    }

    /// Build the canonicalised adjacency list once (consumes `self.edges`).
    fn build_adjacency(&mut self) -> Vec<Vec<usize>> {
        let n = self.node_count;
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let edges = std::mem::take(&mut self.edges);
        for (from, to) in edges {
            let f = self.uf.find(from);
            let t = self.uf.find(to);
            adj[f].push(t);
        }
        adj
    }
```

**(e)** In `emit_table`, replace the `let float = self.solve_float();` line and drive both axes from the shared adjacency:

```rust
        let adj = self.build_adjacency();
        let float_seeds = std::mem::take(&mut self.seeds);
        let string_seeds = std::mem::take(&mut self.string_seeds);
        let float = self.solve_reach(&adj, &float_seeds);
        let string = self.solve_reach(&adj, &string_seeds);
        let mut table = ReprTable::default();
```

Delete the now-unused `solve_float` method.

**(f)** In `emit_table`, update the scalar, return, and param materialization loops to consider the string axis and detect conflicts. For scalars:

```rust
        for ((func, name), node) in scalars {
            match (string[node], float[node]) {
                (true, true) => table.add_shape_conflict(format!(
                    "binding `{name}` in `{func}` is used as both a string and a number"
                )),
                (true, false) => table.set_scalar(&func, &name, Repr::String),
                (false, true) => table.set_scalar(&func, &name, Repr::F64),
                (false, false) => {}
            }
        }
```

Apply the same `match (string[node], float[node])` shape to the **returns** loop (`set_return`) and the **params** loop (`set_param`), using an appropriate conflict message for each. Leave the array-element loop unchanged (strings are scalar-only in this spec; array-of-string is out of scope).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_types repr_infer`
Expected: PASS (new string tests plus all existing float/array/object repr tests green).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs
git commit -m "feat(repr-infer): infer Repr::String via string-seed reachability; conflict on string+number"
```

---

### Task 3: Make the repr table available during resolution + track the current function

**Files:**
- Modify: `crates/kali_types/src/context.rs` (resolver struct: add `repr_table` and `current_function` fields)
- Modify: `crates/kali_types/src/resolve/mod.rs` (compute table before `resolve_statement_list`; push/pop current function around `FunctionDeclaration`)
- Test: `crates/kali_types/src/repr_infer_tests.rs` (or the resolve test module) — behavior-preservation check

**Interfaces:**
- Produces: during resolution the resolver exposes `self.repr_table: kali_common::ReprTable` (populated from `infer_reprs` before any statement is resolved) and `self.current_function_name() -> &str` returning the enclosing function name (`"_start"` at module scope). Consumed by Task 4's gate.

This task is a pure plumbing refactor: it computes the repr table earlier and threads function context. It must not change any diagnostic yet — the full suite stays green.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_types/src/repr_infer_tests.rs` a check that resolution still returns the same table it did before (guards the reorder):

```rust
#[test]
fn resolution_result_carries_string_reprs() {
    // End-to-end through the resolver (not infer_reprs directly): the reordered
    // table must reach ResolutionResult unchanged.
    let parsed = crate::test_support::parse_statements("let s = \"hi\";\n");
    let mut resolver = crate::resolve::Resolver::default();
    let result = resolver.resolve_statements_at_path(None::<&std::path::Path>, &parsed);
    assert_eq!(result.repr_table.scalar("_start", "s"), Repr::String);
}
```

(If `Resolver` is constructed differently in this crate, mirror the constructor used by the existing resolve tests in `crates/kali_types/src/` — grep for `resolve_statements_at_path` in tests.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_types resolution_result_carries_string_reprs`
Expected: FAIL to compile if `Resolver`/constructor path differs, or FAIL if the field isn't wired. (Adjust the constructor to the crate's actual test pattern; the assertion itself should pass only after Step 3 wires `self.repr_table`.)

- [ ] **Step 3: Write the implementation**

**(a)** In `crates/kali_types/src/context.rs`, add two fields to the resolver struct (next to existing state like `suppress_string_addition_rejection`):

```rust
    pub(crate) repr_table: kali_common::ReprTable,
    /// Stack of enclosing function names; module scope is `_start`.
    current_function: Vec<String>,
```

Initialize `current_function` to `vec!["_start".to_string()]` wherever the struct is constructed (its `Default`/`new`). `repr_table` defaults to `ReprTable::default()`.

Add the accessor (in `context.rs` or `resolve/mod.rs`):

```rust
    pub(crate) fn current_function_name(&self) -> &str {
        self.current_function.last().map(String::as_str).unwrap_or("_start")
    }
```

**(b)** In `resolve/mod.rs::resolve_statements_at_path`, compute the table BEFORE resolving statements, store it on `self`, and reuse it in the result:

```rust
        self.push_scope(ScopeType::Module);
        self.repr_table = crate::repr_infer::infer_reprs(statements);
        self.resolve_statement_list(statements);
        self.emit_pending_generator_function_lowering_diagnostic();
        self.scope_stack.clear();

        ResolutionResult {
            diagnostics: self.diagnostics.clone(),
            scopes: self.scopes.clone(),
            global_scope: self.global_scope.clone(),
            repr_table: self.repr_table.clone(),
        }
```

Remove the old `let repr_table = crate::repr_infer::infer_reprs(statements);` line further down.

**(c)** In `resolve/mod.rs`, in the `Statement::FunctionDeclaration` arm (around line 460), push/pop the function name, mirroring the existing `in_generator_function` save/restore:

```rust
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::Function);
                self.current_function.push(name.clone());
                let previous_generator = self.in_generator_function;
                self.in_generator_function = *generator;
                if *generator {
                    self.record_generator_function_lowering(*is_async);
                }
                self.bind_name_list(params);
                self.resolve_block_body(body);
                self.in_generator_function = previous_generator;
                self.current_function.pop();
                self.pop_scope();
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_types`
Expected: PASS — the new test plus every existing `kali_types` test (no diagnostic changed).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_types/src/context.rs crates/kali_types/src/resolve/mod.rs crates/kali_types/src/repr_infer_tests.rs
git commit -m "refactor(resolve): compute repr table before resolution and track current function"
```

---

### Task 4: Consult `Repr::String` in codegen and narrow the `E3200` gate

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`is_string_valued`)
- Modify: `crates/kali_types/src/resolve/expression.rs` (`reject_unsupported_string_variable_addition`, add `operand_repr_is_string`)
- Test: `crates/kali_cli/tests/runtime_string_value_flow.rs` (new, end-to-end `kali run`)

**Interfaces:**
- Consumes: `Repr::String` in the `ReprTable` (Tasks 1–3), `self.current_function_name()` (Task 3), codegen's `self.scalar_repr(name)` / `self.repr_table.return_repr(name)` / `self.function_name` (existing).
- Produces: string-typed variables/params and calls to string-returning functions lower correctly through `+` and `console.log`; the `E3200` diagnostic fires only for string sources not backed by `Repr::String`.

The codegen half and the gate half MUST land together: relaxing the gate without codegen support would miscompile; adding codegen support without relaxing the gate leaves the program rejected. They are one reviewable unit.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/runtime_string_value_flow.rs`:

```rust
use std::process::Command;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    let dir = std::env::temp_dir().join(format!(
        "kali-strflow-{}-{}",
        std::process::id(),
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
fn string_variable_concat_prints() {
    let out = run_source("let x = \"GG\";\nx = x + \"CC\";\nconsole.log(x);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "GGCC\n");
}

#[test]
fn string_param_return_roundtrip_prints() {
    let out = run_source("function f(s) { return s + \"!\"; }\nconsole.log(f(\"hi\"));\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hi!\n");
}

#[test]
fn string_accumulation_loop_prints() {
    let out = run_source("let a = \"\";\nfor (let i = 0; i < 3; i = i + 1) { a = a + \"y\"; }\nconsole.log(a);\n");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "yyy\n");
}

#[test]
fn string_then_number_binding_is_rejected() {
    // A binding used as both string and number must fail to compile (fail-closed),
    // not silently miscompile.
    let out = run_source("let x = \"a\";\nx = 5;\nconsole.log(x);\n");
    assert!(!out.status.success(), "mixed string/number binding must be rejected");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_string_value_flow`
Expected: FAIL — `string_variable_concat_prints`, `string_param_return_roundtrip_prints`, and `string_accumulation_loop_prints` fail today with the `E3200` diagnostic (non-zero exit); `string_then_number_binding_is_rejected` may already pass (it should stay passing).

- [ ] **Step 3: Extend codegen `is_string_valued`**

In `crates/kali_codegen/src/emit/operators.rs`, extend `is_string_valued` to consult the repr for identifiers and calls, mirroring `is_float_valued`. Add arms to the existing `match node.kind`:

```rust
    pub(crate) fn is_string_valued(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let node = self.node(id);
        match node.kind {
            LirNodeKind::Literal => node.text.as_deref().is_some_and(|text| {
                let trimmed = text.trim();
                let mut chars = trimmed.chars();
                matches!(
                    (chars.next(), trimmed.chars().last()),
                    (Some('"'), Some('"')) | (Some('\''), Some('\'')) | (Some('`'), Some('`'))
                )
            }),
            LirNodeKind::Value if node.children.len() == 2 && node.text.as_deref() == Some("+") => {
                self.is_string_valued(node.children[0]) || self.is_string_valued(node.children[1])
            }
            // Bare identifier read: string iff its binding's repr is String.
            // Mirror is_float_valued's local-vs-module-const resolution: a name
            // not declared locally in a non-_start function reads the module
            // (_start) binding.
            LirNodeKind::Value if node.children.is_empty() => {
                node.text.as_deref().is_some_and(|name| {
                    if !self.locals.contains_key(name) && self.function_name != "_start" {
                        self.repr_table.scalar("_start", name) == kali_common::Repr::String
                    } else {
                        self.scalar_repr(name) == kali_common::Repr::String
                    }
                })
            }
            // Call to a string-returning function.
            LirNodeKind::Call => self
                .call_callee_name(id)
                .is_some_and(|name| self.repr_table.return_repr(&name) == kali_common::Repr::String),
            _ => false,
        }
    }
```

If a `call_callee_name(id) -> Option<String>` helper does not already exist, use whatever helper `is_float_valued`'s call arm uses to get the callee name (grep `is_float_valued` in the same file for the exact accessor — it resolves the callee identifier of a `Call` node and reads `return_repr`). Reuse that accessor rather than adding a new one.

- [ ] **Step 4: Narrow the `E3200` gate**

In `crates/kali_types/src/resolve/expression.rs`, add a repr-backed predicate and use it to narrow the rejection:

```rust
    /// True when `operand`'s runtime representation is proven `Repr::String` by
    /// the repr inference — the SAME signal codegen's `is_string_valued` uses,
    /// so the gate and codegen never disagree. Covers a string-typed identifier
    /// (variable/param) and a call to a string-returning function.
    fn operand_repr_is_string(&self, operand: &Expression) -> bool {
        use kali_common::Repr;
        match operand {
            Expression::Identifier(name) => {
                let func = self.current_function_name();
                self.repr_table.scalar(func, name) == Repr::String
                    || self.repr_table.scalar("_start", name) == Repr::String
            }
            Expression::CallExpression(call) => {
                if let Expression::Identifier(callee) = &call.callee {
                    self.repr_table.return_repr(callee) == Repr::String
                } else {
                    false
                }
            }
            Expression::ParenthesizedExpression(inner) => self.operand_repr_is_string(&inner.expression),
            _ => false,
        }
    }
```

Then narrow the rejection closure in `reject_unsupported_string_variable_addition`:

```rust
        let operand_is_unsupported_string = |operand: &Expression| {
            self.expression_is_string_typed(operand)
                && !self.expression_is_codegen_string_valued(operand)
                && !self.operand_repr_is_string(operand)
        };
```

(Match the actual `CallExpression` field names in this AST — grep `pub struct CallExpression` in `kali_ast`; `callee` may be boxed. Adjust the pattern accordingly.)

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_string_value_flow`
Expected: PASS — all four tests green (`GGCC`, `hi!`, `yyy` printed; mixed binding rejected).

- [ ] **Step 6: Run the full suite to confirm no regressions**

Run: `cargo test --workspace`
Expected: PASS — no existing test regresses (literal-rooted concat, float axis, string const-fold behaviors all unchanged).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/runtime_string_value_flow.rs
git commit -m "feat(strings): flow Repr::String vars/params/returns through + and console.log; narrow E3200"
```

---

## Self-Review

**1. Spec coverage:**
- Design §"Approach" step 1 (`kali_common` axis) → Task 1. ✓
- Step 2 (`kali_types` repr inference: seed, unify, conflict) → Task 2. ✓
- Step 3 (codegen consults axis) → Task 4 Step 3. ✓
- Step 4 (relax `E3200`) → Task 4 Step 4. ✓
- Design "Testing" cases: string-variable concat+print → `string_variable_concat_prints`; param round-trip + return consumed by caller → `string_param_return_roundtrip_prints`; accumulation loop → `string_accumulation_loop_prints`; conflict guard → `string_then_number_binding_is_rejected`; no regressions → Task 4 Step 6. ✓
- Design "Scope/non-goals" (substring/join/dynamic keys stay gated): enforced by Task 4's narrowing keying strictly on `Repr::String`, which Task 2 only sets for literal/template-seeded scalar flow — call results from `substring`/`join` and member reads are not seeded, so they stay rejected. ✓
- The ordering discovery (gate runs before `infer_reprs`) is resolved by Task 3 (compute table first). This is plan-added plumbing the spec's "consult the repr" implies; flagged here as the one non-obvious dependency.

**2. Placeholder scan:** No TBD/TODO/"handle edge cases"/"similar to". Two places instruct the implementer to grep for an exact accessor/field name (`is_float_valued`'s callee-name helper; `CallExpression` field shape) rather than guess — these are real-code lookups, not placeholders, because the precise symbol must match the codebase, and the fallback behavior is fully specified.

**3. Type consistency:** `Repr::String` used identically across Tasks 1–4. `ReprTable` methods (`scalar`, `param`, `return_repr`, `set_scalar`, `set_param`, `set_return`, `add_shape_conflict`, `is_empty`) match the existing API read in the investigation. `current_function_name() -> &str` defined in Task 3, consumed in Task 4. `is_string_valued` / `scalar_repr` / `return_repr` / `function_name` / `locals` are existing codegen members confirmed present. `solve_reach` / `build_adjacency` defined and consumed within Task 2.
