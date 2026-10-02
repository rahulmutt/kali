# Anonymous Array Return Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A `const`-bound or immediately-invoked arrow or function expression that returns an `I64` array hands its caller a real runtime array. Every other directly-called anonymous array return refuses with `E5506`. Nothing is left that prints node's value as a silent `0`.

**Architecture:** `repr_infer` builds a per-scope `const` function-alias table in its Phase A2 pre-pass. Each array-return fact keyed on a callee name is then resolved through the table to the arrow's synthetic `__kali_fn_N` id, so the pure fixed point in `array_return.rs` sees anonymous bodies as ordinary candidates. Codegen's `array_return_call_elem` resolves its callee the same way codegen already resolves the call itself (`resolve_bound_member_callable_node`). Callee-side emission is unchanged because it is already keyed on `__kali_fn_N`.

**Tech Stack:** Rust (`kali_types`, `kali_codegen`, `kali_common`), black-box `.toml` CLI cases, `node v26.10.0` as oracle, bash probe runner.

**Spec:** `docs/superpowers/specs/2026-10-02-anon-array-return-design.md`

## Global Constraints

- Baseline commit: `068b29950`. Branch: `anon-array-return`. Oracle: `node v26.10.0`. Toolchain: `rustc 1.99.0`.
- Every line reference in this plan is as of the baseline commit. Re-locate by the quoted code if lines have moved.
- Rust unit tests live in sibling `*_tests.rs` files, never inline `#[cfg(test)]` bodies.
- Black-box CLI tests are `.toml` case files under `crates/kali_cli/tests/cases/`, run by the single `cases` target. Add no new `tests/*.rs` target.
- Only arrows, and function expressions whose id starts with `__kali_fn_`, become candidates. Named function expressions keep their pre-project lane.
- There is no fall-through to module scope. A `const f` resolves only for calls made in the same function (scope) that declares it, because codegen's `bindings` is per function.
- Callbacks (anonymous functions that are never directly called) produce no fact and no refusal.
- Backstop 2 (`array-return-discovered-defects.md` §8) is **not** reopened.
- No CLI, schema, diagnostic-code or maturity change. `E5506` already exists.
- Commit messages use the `(anon-array-return)` scope, e.g. `fix(anon-array-return): …`.

## Review Focus

1. **A called anonymous function that is also passed as a callback** (`const f = () => [1]; f(); xs.forEach(f);`). It becomes array-returning, so the callback lane now receives a real handle where it got `0`. Expected: the output matches node or refuses, and is never silent. Pinned in Task 6 (`called_and_passed_as_callback.js`).
2. **A named function expression called immediately** (`(function h(){ return [1,2]; })()`). Its id is `h`, not `__kali_fn_N`, so it is not a candidate. Now that it's in `called` it taints `FORM` and refuses. Expected: it refuses at both `check` and `run`, and the spike counts it. Pinned in Task 6 (`named_fn_expr_iife.js`).
3. **An arrow whose elements depend on its params** (`const f = (n) => [n, n]; g(f(3))`). No call edge targets `__kali_fn_N`, so the R15 param proof cannot discharge and the arrow taints `ELEMENT`. Expected: a refusal, not a silent `0`, with the gap filed as a followup. Pinned in Task 6 (`param_dependent_elements.js`).
4. **A `const f` declared inside a `switch` case.** Phase A2's `collect_local_names` does not descend into switch case bodies, so the alias table never sees it, while codegen still resolves it. Expected: this blind spot is measured in Task 7 and filed. It is not fixed, because extending the shared A2 walk changes `local_names` for every lane.
5. **The same name declared twice in one function** (`const f = () => [1]; { const f = () => 2; }`). The alias is `Blocked`, so inference declines while codegen's flat `bindings` resolves to whichever `f` it emitted last. Expected: measured in Task 7 and filed with its verdict. Pinned in Task 6 at its measured output (`redeclared_alias.js`).

---

## File map

| file | responsibility in this project |
|---|---|
| `crates/kali_types/src/array_return.rs` | pure lane: add `direct_callee`; narrow the anonymous exemption in `solve` |
| `crates/kali_types/src/array_return_tests.rs` | unit tests for the two pure changes |
| `crates/kali_types/src/repr_infer.rs` | `FnAlias` table (Phase A2), `array_return_callee` resolver, IIFE callees, anonymous candidate forms, display names for refusals |
| `crates/kali_types/src/repr_infer_tests.rs` | inference unit tests over the alias table and anonymous candidates |
| `crates/kali_common/src/messages.rs` | `array_return_refused_message_anonymous` |
| `crates/kali_codegen/src/emitter.rs` | `array_return_call_elem` resolves through `bindings` |
| `crates/kali_codegen/src/emit/control_flow_tests.rs` | codegen unit test for the resolution |
| `tools/array-return-probes/probes/anon_*.js`, `tools/array-return-probes/baseline-anon.tsv` | probes and their baseline at `068b29950` |
| `crates/kali_cli/tests/cases/runtime/anon_array_return.toml` | node-oracle cases (new file in the existing `cases` target) |
| `docs/superpowers/followups/anon-array-return-discovered-defects.md` | what was measured and not fixed |
| `docs/superpowers/followups/array-return-discovered-defects.md` | §1 marked FIXED |
| `docs/superpowers/specs/2026-10-02-anon-array-return-design.md` | §7 amendments recording where the plan narrowed the spec |

---

### Task 1: Probes and their baseline

**Files:**
- Create: `tools/array-return-probes/probes/anon_*.js` (16 files, listed below)
- Create: `tools/array-return-probes/baseline-anon.tsv`

**Interfaces:**
- Produces: `baseline-anon.tsv`, read by Task 3 (spike) and Task 7 (final comparison). Its columns are those of `run.sh`: `name  verdict  node  kali`.

- [ ] **Step 1: Confirm the tree is at the baseline**

Run: `git merge-base --is-ancestor 068b29950 HEAD && echo ok; git diff --stat 068b29950 -- crates tools`
Expected: `ok`, and an empty diff (only docs have changed since the baseline).

- [ ] **Step 2: Write the probe files**

Create each file with exactly this content (one program per file):

```text
anon_arrow_passed_on.js           const f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));
anon_fnexpr_passed_on.js          const f = function(){ return [1,2,3]; }; function g(x){return x[1];} console.log(g(f()));
anon_iife_passed_on.js            function g(x){return x[1];} console.log(g((() => [1,2,3])()));
anon_nested_const.js              function main(){ const f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f())); } main();
anon_alias_chain.js               const f = () => [1,2,3]; const h = f; function g(x){return x[1];} console.log(g(h()));
anon_side_effect_body.js          const f = () => { console.log("ran"); return [1,2]; }; function g(x){return x[1];} console.log(g(f()));
anon_direct_index.js              const f = () => [1,2,3]; console.log(f()[0]);
anon_direct_length.js             const f = () => [1,2,3]; console.log(f().length);
anon_bound.js                     const f = () => [1,2,3]; const a = f(); console.log(a[2]);
anon_allocation_control.js        const f = (n) => new Array(n).fill(4); function g(x){return x[1];} console.log(g(f(3)));
anon_scalar_control.js            const f = () => 7; console.log(f());
anon_module_from_main_control.js  const f = () => [1,2,3]; function main(){ function g(x){return x[1];} console.log(g(f())); } main();
anon_let_control.js               let f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));
anon_mixed_return.js              const f = (c) => { if (c) { return [1]; } return 0; }; function g(x){return x[0];} console.log(g(f(true)));
anon_boolean_elements.js          const f = () => [true, false]; function g(x){return x[0];} console.log(g(f()));
anon_map_callback_control.js      const xs = new Array(3).fill(1); const ys = xs.map(x => x + 1); console.log(ys[0]);
```

- [ ] **Step 3: Measure the baseline**

Run: `cargo build -p kali_cli && tools/array-return-probes/run.sh /tmp/claude-anon-all.tsv && grep '^anon_' /tmp/claude-anon-all.tsv > tools/array-return-probes/baseline-anon.tsv && cut -f1,2 tools/array-return-probes/baseline-anon.tsv`

Expected (as measured during brainstorming; record whatever is actually printed):
`anon_arrow_passed_on SILENT`, `anon_fnexpr_passed_on SILENT`, `anon_iife_passed_on SILENT`, `anon_nested_const SILENT`, `anon_alias_chain SILENT`, `anon_side_effect_body SILENT`, `anon_direct_index REFUSES`, `anon_direct_length REFUSES`, `anon_bound REFUSES`, `anon_allocation_control CORRECT`, `anon_scalar_control CORRECT`, `anon_module_from_main_control REFUSES`, `anon_let_control REFUSES`. The last three rows (`anon_mixed_return`, `anon_boolean_elements`, `anon_map_callback_control`) are recorded as measured.

If any row differs from that expectation, stop and report it before continuing. The spec's §2 tables were measured at this commit.

- [ ] **Step 4: Commit**

```bash
git add tools/array-return-probes/probes/anon_*.js tools/array-return-probes/baseline-anon.tsv
git commit -m "test(anon-array-return): probes for anonymous array returns and their baseline at 068b29950"
```

---

### Task 2: The pure lane — `direct_callee` and the narrowed exemption

**Files:**
- Modify: `crates/kali_types/src/array_return.rs` (`classify_return_arg` call arm ~`:335`, `arg_shape` ~`:378`, `classify_init` ~`:400`, `arg_array_proof` ~`:292`, `solve` exemption `:630-638`)
- Test: `crates/kali_types/src/array_return_tests.rs`

**Interfaces:**
- Produces: `pub(crate) fn direct_callee(call: &kali_ast::CallExpression) -> Option<String>`. It returns the bare identifier callee's name, or an immediately-invoked arrow's / function expression's `id`. Tasks 3 and 4 rely on IIFE callees arriving as `ReturnArg::Call(id)`, `ArgShape::Call(id)`, `InitKind::Call(id)` and `ArgArrayProof::Call(id)`.
- Produces: `solve` exempts a non-candidate `__kali_fn_*` from taint **only when it is not in `facts.called`**.

- [ ] **Step 1: Write the failing tests**

Append to `crates/kali_types/src/array_return_tests.rs`:

```rust
#[test]
fn iife_call_is_call_to_its_synthetic_id() {
    let mut parsed = parse("function f() { return (() => [1, 2])(); }");
    let Statement::FunctionDeclaration(decl) = &mut parsed[0] else {
        panic!("not a declaration");
    };
    let Statement::ReturnStatement(ret) = &mut decl.body.body[0] else {
        panic!("not a return");
    };
    let Some(Expression::CallExpression(call)) = ret.argument.as_mut() else {
        panic!("not a call");
    };
    let mut callee = &mut call.callee;
    while let Expression::ParenthesizedExpression(inner) = callee {
        callee = &mut inner.expression;
    }
    let Expression::ArrowFunctionExpression(arrow) = callee else {
        panic!("not an arrow: {callee:?}");
    };
    arrow.id = Some("__kali_fn_0".into());
    let arg = ret.argument.clone();
    assert_eq!(
        classify_return_arg(arg.as_ref(), &|_| false, &|_| false),
        ReturnArg::Call("__kali_fn_0".into())
    );
    let arg = arg.expect("argument");
    assert_eq!(arg_shape(&arg), ArgShape::Call("__kali_fn_0".into()));
    assert_eq!(classify_init(&arg), InitKind::Call("__kali_fn_0".into()));
    assert_eq!(
        arg_array_proof("f", &arg),
        ArgArrayProof::Call("__kali_fn_0".into())
    );
}

#[test]
fn unnamed_iife_is_not_a_call() {
    // Without the pre-pass id there is nothing to key on.
    assert_eq!(
        classify("function f() { return (() => [1, 2])(); }"),
        ReturnArg::NonArray
    );
}

#[test]
fn called_anonymous_non_candidate_is_tainted_form() {
    let mut facts = facts_one("__kali_fn_0", vec![ReturnArg::Literal(None)]);
    facts.candidate_forms.clear();
    let escaping: BTreeSet<String> = ["__kali_fn_0".to_string()].into();
    let s = solve(&facts, &[], &BTreeMap::new(), &escaping, &no_base);
    assert_eq!(s.tainted.get("__kali_fn_0"), Some(&kali_common::ARRAY_RETURN_FORM));
}

#[test]
fn uncalled_escaping_anonymous_function_is_never_tainted() {
    // A callback: escaping (so not R8-exempt) but never directly called.
    let mut facts = facts_one("__kali_fn_0", vec![ReturnArg::Literal(None)]);
    facts.candidate_forms.clear();
    facts.called.clear();
    let escaping: BTreeSet<String> = ["__kali_fn_0".to_string()].into();
    let s = solve(&facts, &[], &BTreeMap::new(), &escaping, &no_base);
    assert_eq!(s.tainted.get("__kali_fn_0"), None);
}

#[test]
fn called_anonymous_candidate_is_array_returning() {
    let facts = facts_one("__kali_fn_0", vec![ReturnArg::Literal(None)]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.contains("__kali_fn_0"));
    assert!(s.tainted.is_empty());
}
```

Also edit the second half of the existing `non_candidate_form_with_array_return_is_tainted_form`. It builds `facts_one("__kali_fn_0", …)`, which marks the function called, so under the new rule the result is a `FORM` taint, not `None`. Replace that half with:

```rust
    // An anonymous non-candidate that is never directly called is exempt
    // (anon-array-return spec §3.1); `called_anonymous_non_candidate_is_tainted_form`
    // covers the called one.
    let mut facts = facts_one("__kali_fn_0", vec![ReturnArg::Literal(None)]);
    facts.candidate_forms.clear();
    facts.called.clear();
    let escaping: BTreeSet<String> = ["__kali_fn_0".to_string()].into();
    let s = solve(&facts, &[], &BTreeMap::new(), &escaping, &no_base);
    assert_eq!(s.tainted.get("__kali_fn_0"), None);
```

If `ArgArrayProof`/`InitKind` do not derive `PartialEq`, they do (`array_return.rs:279`, `:394`). `parse` returns `Vec<Statement>`, so it is mutable after `let mut`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_types array_return_tests -- --nocapture`
Expected: `iife_call_is_call_to_its_synthetic_id` fails with left `NonArray` (or `Other`/`Unknown`), and `called_anonymous_non_candidate_is_tainted_form` fails with left `None`. The others pass.

- [ ] **Step 3: Implement `direct_callee` and use it**

In `crates/kali_types/src/array_return.rs`, add the import `CallExpression` to the `kali_ast` use line, and add after `fill_value`:

```rust
/// The function a call reaches as written at the call site: a bare-identifier
/// callee's name, or an immediately-invoked arrow's or function expression's
/// synthetic `__kali_fn_N` id (named in place by `name_anon_functions`).
/// Resolving a `const f = () => …` alias needs scope facts, so that is
/// `repr_infer`'s job (anon-array-return spec §3.1).
pub(crate) fn direct_callee(call: &CallExpression) -> Option<String> {
    match unparen(&call.callee) {
        Expression::Identifier(name) => Some(name.clone()),
        Expression::ArrowFunctionExpression(arrow) => arrow.id.clone(),
        Expression::FunctionExpression(func) => func.id.clone(),
        _ => None,
    }
}
```

Replace the four call arms:

```rust
// classify_return_arg
        Expression::CallExpression(call) => match direct_callee(call) {
            Some(callee) => ReturnArg::Call(callee),
            None => ReturnArg::NonArray,
        },
// arg_shape
        Expression::CallExpression(call) => match direct_callee(call) {
            Some(callee) => ArgShape::Call(callee),
            None => ArgShape::Other,
        },
// classify_init
        Expression::CallExpression(call) => match direct_callee(call) {
            Some(callee) => InitKind::Call(callee),
            None => InitKind::Other,
        },
// arg_array_proof
        Expression::CallExpression(call) => match direct_callee(call) {
            Some(callee) => ArgArrayProof::Call(callee),
            None => ArgArrayProof::Unknown,
        },
```

Leave `num_proof`'s call arm unchanged: an IIFE element stays `NumProof::No`, which is conservative.

- [ ] **Step 4: Narrow the exemption in `solve`**

Replace the block at `array_return.rs:630-638` with:

```rust
                // An anonymous `__kali_fn_N` that is never directly called is a
                // callback (`xs.flatMap(x => [x])`): the array-method lanes
                // consume its result, and tainting it refused working programs
                // (plan Task 4 step 10 of the array-return project). One that IS
                // directly called, through a `const` alias or immediately
                // (anon-array-return spec §3.1), is tainted like a declaration.
                if !facts.is_candidate(f)
                    && f.starts_with("__kali_fn_")
                    && !facts.called.contains(f)
                {
                    continue;
                }
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p kali_types array_return`
Expected: all pass, including the existing `repr_infer_tests::array_return_arrow_is_not_tainted_under_the_narrowing`. Its arrow is never called, so `called` does not contain it.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/array_return.rs crates/kali_types/src/array_return_tests.rs
git commit -m "feat(anon-array-return): direct_callee keys an IIFE on its synthetic id; only an uncalled anonymous function is exempt from taint"
```

---

### Task 3: The alias table, callee resolution, and the capability-loss spike (GATE)

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs`
  - struct fields near `array_return_facts` (`:1070`)
  - `collect_local_names_in_stmt` (`:2972-3040`)
  - `register_nested_fn` `LocalNames` arm (`:2713-2721`)
  - `classify_array_return` (`:1803-1808`)
  - `visit_declarator_init` `InitKind::Call` arm (`:3677-3690`)
  - `visit_call` fallback `other` arm (`~:5390`)
  - `resolve_array_returns` feeds (`:5485-5490`) and `called` (`:5536-5556`)
  - proof discharge: `ParamArray` `ArgArrayProof::Call` (`:573-583`) and `NumProof::Call` (`:657-660`)
- Test: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Consumes: `array_return::direct_callee` (Task 2).
- Produces:
  - `fn array_return_callee(&self, func: &str, name: &str) -> Option<String>` on `ReprInfer`. It returns the array-return key a direct call `name(…)` made in `func` reaches, or `None` when this lane can name nothing.
  - `fn_alias_names: BTreeMap<String, String>`, mapping `__kali_fn_N` to the first `const` name bound to it. Task 4 reads it for messages.
  - `iife_callees: BTreeSet<String>`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/kali_types/src/repr_infer_tests.rs`, after `array_return_arrow_is_not_tainted_under_the_narrowing`:

```rust
// ---- Anonymous array returns (spec 2026-10-02-anon-array-return-design.md §3.1) ----

#[test]
fn anon_alias_call_reaches_the_arrow_and_taints_it_before_candidates_exist() {
    // Task 3 state: the call through `const f` now reaches `__kali_fn_0`, which
    // is directly called but not yet a candidate form, so it taints FORM.
    // Task 4 turns this into admission and rewrites this test.
    let t = reprs_with_fn_id(
        "const f = () => [1, 2, 3];\nfunction g(x) { return x[1]; }\nconsole.log(g(f()));\n",
        "__kali_fn_0",
    );
    assert_eq!(
        t.array_return_taint("__kali_fn_0"),
        Some(kali_common::ARRAY_RETURN_FORM)
    );
}

#[test]
fn anon_let_binding_is_never_an_alias() {
    let t = reprs_with_fn_id(
        "let f = () => [1, 2, 3];\nfunction g(x) { return x[1]; }\nconsole.log(g(f()));\n",
        "__kali_fn_0",
    );
    assert_eq!(t.array_return_taint("__kali_fn_0"), None);
    assert_eq!(t.array_return("__kali_fn_0"), None);
}

#[test]
fn anon_alias_does_not_fall_through_to_module_scope() {
    let t = reprs_with_fn_id(
        "const f = () => [1, 2, 3];\nfunction main() { function g(x) { return x[1]; } console.log(g(f())); }\nmain();\n",
        "__kali_fn_0",
    );
    assert_eq!(t.array_return_taint("__kali_fn_0"), None);
    assert_eq!(t.array_return("__kali_fn_0"), None);
}

#[test]
fn anon_redeclared_name_is_not_an_alias() {
    let t = reprs_with_fn_id(
        "const f = () => [1, 2, 3];\nif (true) { const f = 2; }\nfunction g(x) { return x[1]; }\nconsole.log(g(f()));\n",
        "__kali_fn_0",
    );
    assert_eq!(t.array_return_taint("__kali_fn_0"), None);
    assert_eq!(t.array_return("__kali_fn_0"), None);
}
```

- [ ] **Step 2: Run the tests to verify the first one fails**

Run: `cargo test -p kali_types anon_`
Expected: `anon_alias_call_reaches_the_arrow_and_taints_it_before_candidates_exist` FAILS (left `None`). The other three pass already, and they guard against over-resolving.

- [ ] **Step 3: Add the alias table**

In `repr_infer.rs`, next to the other module-private enums (near `ArrayOrigin`), add:

```rust
/// What a locally declared name is, as a direct callee, to the array-return
/// lane (anon-array-return spec §3.1). Mirrors codegen's per-function, flat
/// `bindings`: only a `const` is an alias, and a name declared twice in one
/// scope is ambiguous there, so it is never one here.
#[derive(Clone, Debug, PartialEq, Eq)]
enum FnAlias {
    /// `const name = <arrow | anonymous function expression>`: its `__kali_fn_N`.
    Function(String),
    /// `const name = other`: resolve `other` in the same scope.
    Binding(String),
    /// Declared any other way, or more than once.
    Blocked,
}
```

Add fields to `ReprInfer` beside `array_return_facts`:

```rust
    /// Anon-array-return §3.1: `(scope, name)` -> what the name is as a callee.
    /// Built in Phase A2, so it is complete before any body is walked.
    fn_aliases: BTreeMap<(String, String), FnAlias>,
    /// `__kali_fn_N` -> the first `const` name bound to it, for refusal messages.
    fn_alias_names: BTreeMap<String, String>,
    /// `__kali_fn_N` ids called immediately (`(() => [1])()`).
    iife_callees: BTreeSet<String>,
```

Add methods next to `callee_is_shadowed`:

```rust
    /// Phase A2: record one declaration of `name` in `func`. A second
    /// declaration of the same name, of any kind, blocks it.
    fn note_fn_alias(&mut self, func: &str, kind: &str, name: &str, init: Option<&Expression>) {
        let key = (func.to_string(), name.to_string());
        if self.fn_aliases.contains_key(&key) {
            self.fn_aliases.insert(key, FnAlias::Blocked);
            return;
        }
        let alias = match (kind, init.map(crate::array_return::unparen)) {
            ("const", Some(Expression::ArrowFunctionExpression(a))) => {
                a.id.clone().map(FnAlias::Function)
            }
            ("const", Some(Expression::FunctionExpression(f))) => f
                .id
                .clone()
                .filter(|id| id.starts_with("__kali_fn_"))
                .map(FnAlias::Function),
            ("const", Some(Expression::Identifier(other))) => {
                Some(FnAlias::Binding(other.clone()))
            }
            _ => None,
        };
        if let Some(FnAlias::Function(id)) = &alias {
            self.fn_alias_names
                .entry(id.clone())
                .or_insert_with(|| name.to_string());
        }
        self.fn_aliases.insert(key, alias.unwrap_or(FnAlias::Blocked));
    }

    /// Phase A2: a param, catch param or loop binding is never an alias.
    fn block_fn_alias(&mut self, func: &str, name: &str) {
        self.fn_aliases
            .insert((func.to_string(), name.to_string()), FnAlias::Blocked);
    }

    /// The `__kali_fn_N` a `const` alias `name` in `func` names, following
    /// `const h = f` chains in the same scope.
    fn fn_alias_target(&self, func: &str, name: &str) -> Option<String> {
        let mut seen = BTreeSet::new();
        let mut current = name.to_string();
        loop {
            if !seen.insert(current.clone()) {
                return None;
            }
            match self.fn_aliases.get(&(func.to_string(), current.clone()))? {
                FnAlias::Function(id) => return Some(id.clone()),
                FnAlias::Binding(next) => current = next.clone(),
                FnAlias::Blocked => return None,
            }
        }
    }

    /// The array-return key a direct call `name(…)` made in `func` reaches: the
    /// arrow a `const` alias names, else `name` itself unless `func` shadows it
    /// (`None`). An IIFE's `__kali_fn_N` is never declared, so it passes through.
    fn array_return_callee(&self, func: &str, name: &str) -> Option<String> {
        if let Some(id) = self.fn_alias_target(func, name) {
            return Some(id);
        }
        (!self.callee_is_shadowed(func, name)).then(|| name.to_string())
    }
```

- [ ] **Step 4: Populate the table in Phase A2**

In `collect_local_names_in_stmt`, make each arm record into the table alongside `local_names`:
- In the `FunctionDeclaration` arm, after inserting params: `for param in &decl.params { self.block_fn_alias(&decl.name, param); }`. Collect the names first if the borrow checker needs it: `let params = decl.params.clone();`.
- In the `VariableDeclaration` arm, after the `entry.insert` loop:

```rust
                for d in &decl.declarations {
                    self.note_fn_alias(func, &decl.kind, &d.id, d.init.as_ref());
                }
```

- In the `ForStatement`, `ForInStatement` and `ForOfStatement` declaration arms, add `self.block_fn_alias(func, &d.id)` for each declarator.
- In the `TryStatement` handler arm, add `self.block_fn_alias(func, &handler.param)`.

In `register_nested_fn`'s `NestedFnWalk::LocalNames` arm, after the param inserts: `for param in params { self.block_fn_alias(id, param); }`.

- [ ] **Step 5: Resolve at every fact site**

`classify_array_return`, replacing its final `match class { … }`:

```rust
        match class {
            ReturnArg::Call(callee) => match self.array_return_callee(func, &callee) {
                Some(key) => ReturnArg::Call(key),
                None => ReturnArg::NonArray,
            },
            other => other,
        }
```

`visit_declarator_init`'s `InitKind::Call(callee)` arm: keep the `array_origins` push, then replace the `if kind != "var" && !self.callee_is_shadowed(func, &callee) {` guard with:

```rust
                if kind != "var" {
                    if let Some(callee) = self.array_return_callee(func, &callee) {
                        if kind == "let" {
                            self.let_call_bound_bindings
                                .insert((func.to_string(), id.to_string()));
                        }
                        self.array_return_facts.call_bound.push((
                            func.to_string(),
                            id.to_string(),
                            callee,
                        ));
                    }
                }
```

`visit_call`'s fallback `other =>` arm: record an IIFE before visiting:

```rust
            other => {
                if let Some(id) = crate::array_return::direct_callee(call) {
                    self.iife_callees.insert(id);
                }
                self.visit_expr(func, other);
                for arg in &call.args {
                    self.visit_expr(func, arg);
                }
                self.new_node()
            }
```

`direct_callee` returns an identifier's name too, but this arm is never reached for a bare identifier: the `Expression::Identifier` arm above takes it. Check that `call.callee` here is not wrapped in a way that makes `other` an identifier. If parenthesized identifiers reach this arm, guard with `if !matches!(crate::array_return::unparen(&call.callee), Expression::Identifier(_))`.

`resolve_array_returns`, in the feeds loop, replace

```rust
                    Some(ArgShape::Call(g)) if self.callee_is_shadowed(&edge.caller, g) => {
                        ArgShape::Other
                    }
```

with

```rust
                    Some(ArgShape::Call(g)) => match self.array_return_callee(&edge.caller, g) {
                        Some(key) => ArgShape::Call(key),
                        None => ArgShape::Other,
                    },
```

`resolve_array_returns`, in the `called` set: keep the raw edge callees (so R8 never gets weaker for declarations), and add the resolved ones and the IIFEs:

```rust
        let mut called: BTreeSet<String> =
            self.calls.iter().map(|edge| edge.callee.clone()).collect();
        called.extend(
            self.calls
                .iter()
                .filter_map(|edge| self.array_return_callee(&edge.caller, &edge.callee)),
        );
        called.extend(self.iife_callees.iter().cloned());
```

(The rest of the existing `called.extend(...)` lines stay. They read `call_bound`, `ReturnArg::Call` and feed shapes, which now carry resolved keys.)

Proof discharge, `NumFact::ParamArray`'s `ArgArrayProof::Call(g)` arm:

```rust
                        Some(crate::array_return::ArgArrayProof::Call(g)) => self
                            .infer
                            .array_return_callee(&edge.caller, g)
                            .and_then(|key| {
                                self.infer.array_elem_node.get(&(
                                    key,
                                    crate::array_return::RETURN_ARRAY_KEY.to_string(),
                                ))
                            })
                            .is_some_and(|&node| self.class_of(node, deps)),
```

`proof_holds`'s `NumProof::Call` arm:

```rust
            NumProof::Call { caller, callee } => self
                .infer
                .array_return_callee(caller, callee)
                .is_some_and(|key| self.assume(NumFact::Return(key), deps)),
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p kali_types`
Expected: all pass, including the four `anon_` tests.

- [ ] **Step 7: The capability-loss spike (GATE)**

This tree is the spec's §5.1 spike: anonymous functions are resolved and taintable but not yet candidates, so every directly-called anonymous array return now refuses.

Run:
```bash
cargo build -p kali_cli
tools/array-return-probes/run.sh /tmp/claude-anon-spike.tsv
git stash -q && cargo build -q -p kali_cli && tools/array-return-probes/run.sh /tmp/claude-anon-pre.tsv; git stash pop -q && cargo build -q -p kali_cli
join -t$'\t' <(cut -f1,2 /tmp/claude-anon-pre.tsv | sort) <(cut -f1,2 /tmp/claude-anon-spike.tsv | sort) | awk -F'\t' '$2!=$3'
cargo test --workspace 2>&1 | grep -E '^test .* FAILED|^---- |panicked|test result' | tee /tmp/claude-anon-spike-tests.txt
```

Expected:
- Probe diff: the six SILENT `anon_*` rows move to `REFUSES`. `anon_mixed_return` and `anon_boolean_elements` move to `REFUSES` if they were SILENT. No row moves away from `CORRECT`.
- Test failures: only cases that pinned a §2.1 program at `0`, or that pinned an anonymous IIFE's or alias call's output.

For **every** test failure and every `CORRECT → REFUSES` probe, write one line in `/tmp/claude-anon-spike-losses.md`: the program, its baseline output, its spike output, and whether Task 4's admission is expected to restore it (an `I64` literal or allocation return reached through a `const` alias or IIFE) or not.

**GATE:** If any loss is *not* expected to be restored by Task 4, stop and bring the list to the human partner before continuing. If every loss is expected to be restored, continue, keeping the file for Task 7.

- [ ] **Step 8: Commit**

Commit even though the tree refuses more than the baseline. Task 4 is next, and the spike is part of the record.

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs
git commit -m "feat(anon-array-return): resolve const function aliases and IIFEs to their synthetic id in every array-return fact"
```

If `cargo test --workspace` has failures that Task 4 restores, say so in the commit body, listing the failing case names.

---

### Task 4: Anonymous candidate forms and readable refusals

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs`
  - `visit_expr` `FunctionExpression` arm (`~:4490`) and `ArrowFunctionExpression` arm (`~:4520`)
  - `resolve_array_returns` (after `called` is computed)
  - `emit_table` taint loop (`:6752-6755`)
- Modify: `crates/kali_common/src/messages.rs` (after `array_return_refused_message`, `:134`)
- Test: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Consumes: `fn_alias_names`, `iife_callees`, `array_return_callee` (Task 3).
- Produces:
  - `pub fn array_return_refused_message_anonymous(reason: &str) -> String` in `kali_common`. Re-export it the way `array_return_refused_message` is exported (check `crates/kali_common/src/lib.rs`).
  - `ReprTable::array_return("__kali_fn_N") == Some(Repr::I64)` for an admitted anonymous function. Task 5 reads it.

- [ ] **Step 1: Write the failing tests**

In `repr_infer_tests.rs`, **replace** `anon_alias_call_reaches_the_arrow_and_taints_it_before_candidates_exist` with:

```rust
#[test]
fn anon_alias_call_admits_the_arrow() {
    let t = reprs_with_fn_id(
        "const f = () => [1, 2, 3];\nfunction g(x) { return x[1]; }\nconsole.log(g(f()));\n",
        "__kali_fn_0",
    );
    assert_eq!(t.array_return("__kali_fn_0"), Some(Repr::I64));
    assert_eq!(t.array_return_taint("__kali_fn_0"), None);
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
}

#[test]
fn anon_alias_call_bound_binding_is_an_array() {
    let t = reprs_with_fn_id(
        "const f = () => [1, 2, 3];\nconst a = f();\nconsole.log(a[2]);\n",
        "__kali_fn_0",
    );
    assert_eq!(t.array_return("__kali_fn_0"), Some(Repr::I64));
    assert!(t.is_call_bound_array_binding("_start", "a"));
}

#[test]
fn anon_block_bodied_function_expression_is_admitted() {
    let t = reprs_with_fn_id(
        "const f = function () { return [1, 2, 3]; };\nconst a = f();\n",
        "__kali_fn_0",
    );
    assert_eq!(t.array_return("__kali_fn_0"), Some(Repr::I64));
}

#[test]
fn anon_mixed_return_refuses_under_its_binding_name() {
    let t = reprs_with_fn_id(
        "const f = function (c) { if (c) { return [1]; } return 0; };\nconst a = f(true);\nconsole.log(a[0]);\n",
        "__kali_fn_0",
    );
    assert_eq!(
        t.array_return_taint("__kali_fn_0"),
        Some(kali_common::ARRAY_RETURN_MIXED)
    );
    assert!(
        t.shape_conflicts()
            .iter()
            .any(|m| m.contains("returning an array from `f`")
                && m.contains(kali_common::ARRAY_RETURN_MIXED)),
        "{:?}",
        t.shape_conflicts()
    );
}

#[test]
fn anon_async_arrow_is_not_a_candidate_and_never_tainted() {
    // Ruling R12, as for declarations: the call yields a Promise.
    let mut parsed = crate::test_support::parse_statements(
        "const f = async () => [1, 2];\nconst p = f();\n",
    );
    if let kali_ast::Statement::VariableDeclaration(decl) = &mut parsed[0] {
        if let Some(kali_ast::Expression::ArrowFunctionExpression(a)) =
            decl.declarations[0].init.as_mut()
        {
            a.id = Some("__kali_fn_0".into());
        }
    }
    let t = infer_reprs(&parsed);
    assert_eq!(t.array_return("__kali_fn_0"), None);
    assert_eq!(t.array_return_taint("__kali_fn_0"), None);
}

#[test]
fn anonymous_refusal_message_names_an_immediately_invoked_function() {
    let m = kali_common::array_return_refused_message_anonymous(kali_common::ARRAY_RETURN_MIXED);
    assert!(m.starts_with("returning an array from an immediately-invoked function is unavailable in the current phase"));
}
```

If `reprs_with_fn_id`'s `reprs` wrapper normally does more than `infer_reprs(&parsed)` (check `fn reprs` at the top of the file), use the same call in the async test.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_types anon`
Expected: `anon_alias_call_admits_the_arrow`, `anon_alias_call_bound_binding_is_an_array`, `anon_block_bodied_function_expression_is_admitted` fail (left `None`). `anon_mixed_return_refuses_under_its_binding_name` fails on the message. `anonymous_refusal_message_…` fails to compile, because the function doesn't exist yet. Add the message function first (Step 3) if the compile error blocks the others.

- [ ] **Step 3: The message**

In `crates/kali_common/src/messages.rs` after `array_return_refused_message`:

```rust
/// [`array_return_refused_message`] for an anonymous function that has no
/// source name: one called immediately (anon-array-return spec §3.3).
pub fn array_return_refused_message_anonymous(reason: &str) -> String {
    format!(
        "returning an array from an immediately-invoked function is unavailable in the current phase: {reason}"
    )
}
```

Export it wherever `array_return_refused_message` is exported.

- [ ] **Step 4: Record anonymous forms during the walk**

Add a field beside `iife_callees`:

```rust
    /// Non-async, non-generator `__kali_fn_N` bodies: candidate forms once
    /// directly called (anon-array-return spec §3.1).
    anon_fn_forms: BTreeSet<String>,
```

and a method:

```rust
    /// An anonymous function's form facts. A concise arrow body is one
    /// `return`, so it never falls off the end (`body` is `None`).
    fn note_anon_fn_form(
        &mut self,
        id: &str,
        is_async: bool,
        generator: bool,
        body: Option<&[Statement]>,
    ) {
        if is_async || generator {
            // Ruling R12: never a candidate, never tainted.
            self.array_return_facts
                .non_taintable
                .insert(id.to_string());
            self.array_return_facts
                .declaration_counts
                .insert(id.to_string(), 1);
            return;
        }
        self.anon_fn_forms.insert(id.to_string());
        if body.is_some_and(crate::array_return::body_falls_off_end) {
            self.array_return_facts
                .falls_off_end
                .insert(id.to_string());
        }
    }
```

In `visit_expr`'s `FunctionExpression` arm, inside `if let (Some(id), Some(body)) = …`, before `self.visit_block(id, body)`:

```rust
                    if id.starts_with("__kali_fn_") {
                        self.note_anon_fn_form(id, f.is_async, f.generator, Some(&body.body));
                    }
```

In the `ArrowFunctionExpression` arm, inside `if let Some(id) = a.id.as_deref()`, before `self.visit_expr(id, &a.body)`: `self.note_anon_fn_form(id, a.is_async, false, None);`. Also update that arm's comment "An arrow is never a candidate form, so an array-shaped body taints it (`ARRAY_RETURN_FORM`)" to: "A directly-called arrow is a candidate form (anon-array-return §3.1); an uncalled one is exempt."

Note `id: &str` borrows from `a`/`f` while `note_anon_fn_form` takes `&mut self`. The AST is not owned by `self`, so this compiles. If it does not, clone `id` first.

- [ ] **Step 5: Admit called anonymous forms in Phase C0**

In `resolve_array_returns`, immediately after `called` is fully built and **before** `self.array_return_facts.called = called;`:

```rust
        // Anon-array-return §3.1: a directly-called anonymous body is a
        // candidate form, exactly like a `function` declaration.
        for id in &self.anon_fn_forms {
            if called.contains(id) {
                self.array_return_facts
                    .candidate_forms
                    .insert(id.clone());
                self.array_return_facts
                    .declaration_counts
                    .insert(id.clone(), 1);
            }
        }
```

- [ ] **Step 6: Readable refusal messages**

Add a method:

```rust
    /// The refusal for `f`'s array return, naming an anonymous function by the
    /// `const` it is bound to (anon-array-return spec §3.3).
    fn array_return_refusal(&self, f: &str, reason: &str) -> String {
        match self.fn_alias_names.get(f) {
            Some(name) => kali_common::array_return_refused_message(name, reason),
            None if f.starts_with("__kali_fn_") => {
                kali_common::array_return_refused_message_anonymous(reason)
            }
            None => kali_common::array_return_refused_message(f, reason),
        }
    }
```

In `emit_table`, change `table.add_shape_conflict(kali_common::array_return_refused_message(f, reason));` to `table.add_shape_conflict(self.array_return_refusal(f, reason));`. Then grep for any other `array_return_refused_message(` call in `kali_types` that builds a shape conflict from a function key, and route it through the same method:

Run: `grep -rn "array_return_refused_message(" crates/kali_types/src`

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p kali_types`
Expected: all pass.

- [ ] **Step 8: Re-run the spike comparison**

Run the Step 7 commands of Task 3 again (writing `/tmp/claude-anon-task4.tsv`), comparing against `tools/array-return-probes/baseline-anon.tsv`:

```bash
cargo build -p kali_cli && tools/array-return-probes/run.sh /tmp/claude-anon-task4.tsv
join -t$'\t' <(cut -f1,2 tools/array-return-probes/baseline-anon.tsv | sort) <(grep '^anon_' /tmp/claude-anon-task4.tsv | cut -f1,2 | sort) | awk -F'\t' '$2!=$3'
```

Expected at this point, with codegen's caller side not yet changed:
- The `g(f())` rows (`anon_arrow_passed_on`, `anon_fnexpr_passed_on`, `anon_iife_passed_on`, `anon_nested_const`, `anon_alias_chain`, `anon_side_effect_body`) are `CORRECT` or `REFUSES`. The callee now materializes the array and the param `x` reads it, so `CORRECT` is likely.
- `anon_direct_index`, `anon_direct_length` and `anon_bound` are still `REFUSES` (Task 5).
- No row is `SILENT`.

Any `SILENT` row is a defect in this task. Debug it with superpowers:systematic-debugging before continuing.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs crates/kali_common/src/messages.rs crates/kali_common/src/lib.rs
git commit -m "feat(anon-array-return): a directly-called anonymous function is an array-return candidate; refusals name its const binding"
```

---

### Task 5: Codegen reads a call through a `const` alias or an IIFE

**Files:**
- Modify: `crates/kali_codegen/src/emitter.rs:835-850` (`array_return_call_elem`, and its doc comment above)
- Test: `crates/kali_codegen/src/emit/control_flow_tests.rs`

**Interfaces:**
- Consumes: `ReprTable::array_return("__kali_fn_N")` (Task 4); `FunctionEmitter::resolve_bound_member_callable_node` (`emit/call.rs`), `unwrap_transparent_value_node`, `bare_identifier_name`, `self.functions`.
- Produces: `array_return_call_elem` returns `Some(elem)` for a call whose callee codegen resolves to an admitted `__kali_fn_N`. Every existing consumer (direct index, `.length`, the console guard, argument passing, call-bound registration at `control_flow.rs:1751`) picks this up unchanged.

- [ ] **Step 1: Write the failing test**

Append to `crates/kali_codegen/src/emit/control_flow_tests.rs`:

```rust
/// Compiles `source`, optionally admitting the first anonymous function
/// (`__kali_fn_0`, HIR's synthetic name when the CLI pre-pass has not run) as
/// array-returning, and returns the E5506 messages codegen raised
/// (anon-array-return spec §3.2).
fn anon_array_return_e5506_messages(source: &str, admit: bool) -> Vec<String> {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    if admit {
        ctx.repr_table
            .set_array_return("__kali_fn_0", kali_common::Repr::I64);
    }
    let result = lower_lir_to_wasm(&mut ctx, &program);
    result
        .diagnostics
        .iter()
        .filter(|d| d.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32))
        .map(|d| d.message.clone())
        .collect()
}

#[test]
fn anon_alias_direct_index_reads_the_admitted_return() {
    let src = "const f = () => [1, 2, 3]; console.log(f()[0]);";
    let refused = anon_array_return_e5506_messages(src, false);
    assert!(
        refused.iter().any(|m| m.contains("indexed read")),
        "not admitted: backstop 1 refuses ({refused:?})"
    );
    let admitted = anon_array_return_e5506_messages(src, true);
    assert!(admitted.is_empty(), "admitted: {admitted:?}");
}

#[test]
fn anon_iife_length_reads_the_admitted_return() {
    let src = "console.log((() => [1, 2, 3])().length);";
    assert!(!anon_array_return_e5506_messages(src, false).is_empty());
    assert!(anon_array_return_e5506_messages(src, true).is_empty());
}

#[test]
fn anon_let_alias_is_not_resolved() {
    let src = "let f = () => [1, 2, 3]; console.log(f()[0]);";
    assert!(!anon_array_return_e5506_messages(src, true).is_empty());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_codegen anon_`
Expected: `anon_alias_direct_index_reads_the_admitted_return` and `anon_iife_length_reads_the_admitted_return` fail on the admitted assertion. `anon_let_alias_is_not_resolved` passes.

If the not-admitted assertion fails instead (no E5506 at all), the harness's synthetic name is not `__kali_fn_0`. Print `program.nodes.iter().filter_map(|n| n.text.clone()).filter(|t| t.starts_with("__kali_fn_")).collect::<Vec<_>>()` and use the printed name.

- [ ] **Step 3: Implement**

Replace the body of `array_return_call_elem`:

```rust
    pub(crate) fn array_return_call_elem(&self, id: LirNodeId) -> Option<kali_common::Repr> {
        let target = self.unwrap_transparent_value_node(id);
        let node = self.node(target);
        if node.kind != LirNodeKind::Call {
            return None;
        }
        let callee = *node.children.first()?;
        let source_name = self.bare_identifier_name(callee);
        if let Some(name) = &source_name {
            if self
                .repr_table
                .is_array_return_callee_shadowed(&self.function_name, name)
                || self.locals.contains_key(name)
            {
                return None;
            }
        }
        // Resolve the callee exactly as the call itself is lowered
        // (`emit/call.rs`, `resolve_bound_member_callable_node`): a `const`
        // alias or an IIFE lands on the anonymous body's `__kali_fn_N`
        // (anon-array-return spec §3.2).
        let bound = self
            .resolve_bound_member_callable_node(callee)
            .unwrap_or_else(|| self.unwrap_transparent_value_node(callee));
        let key = self
            .node(bound)
            .text
            .clone()
            .filter(|text| self.functions.contains_key(text))
            .or(source_name)?;
        self.repr_table.array_return(&key)
    }
```

Extend the doc comment above it with one paragraph:

```rust
    /// The callee is resolved the way the call is lowered, so a `const f = () =>
    /// …` alias (followed through `const h = f` chains by `bindings`) and an
    /// immediately-invoked arrow reach the anonymous body's `__kali_fn_N` key
    /// (anon-array-return spec §3.2). Inference resolves the same aliases in
    /// `repr_infer::array_return_callee`; the two must agree, and the
    /// `anon_*` probes are the gate.
```

If `self.functions` is not visible from `emitter.rs` (it is a field of the emitter used in `emit/call.rs:891`, so it should be), use the same accessor `call.rs` uses.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_codegen`
Expected: all pass.

- [ ] **Step 5: Probe check**

```bash
cargo build -p kali_cli && tools/array-return-probes/run.sh /tmp/claude-anon-task5.tsv
grep '^anon_' /tmp/claude-anon-task5.tsv | cut -f1,2
```

Expected:
- `CORRECT`: every §2.1 row (the six passed-on rows, `anon_direct_index`, `anon_direct_length`, `anon_bound`), `anon_allocation_control`, `anon_scalar_control`, and `anon_map_callback_control` (if it was CORRECT at baseline).
- `REFUSES`: `anon_module_from_main_control`, `anon_let_control`, `anon_mixed_return`, `anon_boolean_elements`.
- No `SILENT`.

Also confirm no pre-existing probe changed verdict:

```bash
join -t$'\t' <(grep -v '^anon_' /tmp/claude-anon-pre.tsv | cut -f1,2 | sort) <(grep -v '^anon_' /tmp/claude-anon-task5.tsv | cut -f1,2 | sort) | awk -F'\t' '$2!=$3'
```

Expected: empty. (`/tmp/claude-anon-pre.tsv` is from Task 3 Step 7. If it is gone, regenerate it from a `git worktree` at `068b29950`.)

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emitter.rs crates/kali_codegen/src/emit/control_flow_tests.rs
git commit -m "feat(anon-array-return): a call through a const alias or an IIFE reads the admitted anonymous return"
```

---

### Task 6: Node-oracle cases

**Files:**
- Create: `crates/kali_cli/tests/cases/runtime/anon_array_return.toml`
- Modify: any case file Task 3's spike listed as pinning a §2.1 program at `0` (re-pin to node's value)

**Interfaces:**
- Consumes: the behaviour of Tasks 2-5.
- Produces: the pinned oracle for the branch.

- [ ] **Step 1: Measure every program against node first**

For each program in Step 2, run `node P.js` and `target/debug/kali run P.js` (and `kali check P.js` for the refusals). The `stdout` in each case is **node's** output. A refusal case asserts kali's `E5506` text. If kali's output for a "computes" case differs from node's, the case must not be written to match kali: stop and debug.

- [ ] **Step 2: Write the case file**

Create `crates/kali_cli/tests/cases/runtime/anon_array_return.toml`:

```toml
# Cases for the anon-array-return project (spec
# docs/superpowers/specs/2026-10-02-anon-array-return-design.md, sections 2-3).
#
# A `const`-bound or immediately-invoked anonymous function that returns an I64
# array hands its caller a real runtime array. Every `_computes` case prints
# node v26.10.0's exact output; each rationale records kali's output at the
# baseline `068b29950` (tools/array-return-probes/baseline-anon.tsv).
#
# `[source]` keys are one file per program (see this directory's README).

[constants]
REFUSED = "is unavailable in the current phase"

[source]
"arrow_passed_on.js" = '''
const f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));
'''
"fnexpr_passed_on.js" = '''
const f = function(){ return [1,2,3]; }; function g(x){return x[1];} console.log(g(f()));
'''
"iife_passed_on.js" = '''
function g(x){return x[1];} console.log(g((() => [1,2,3])()));
'''
"nested_const.js" = '''
function main(){ const f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f())); } main();
'''
"alias_chain.js" = '''
const f = () => [1,2,3]; const h = f; function g(x){return x[1];} console.log(g(h()));
'''
"side_effect_body.js" = '''
const f = () => { console.log("ran"); return [1,2]; }; function g(x){return x[1];} console.log(g(f()));
'''
"direct_index.js" = '''
const f = () => [1,2,3]; console.log(f()[0]);
'''
"direct_length.js" = '''
const f = () => [1,2,3]; console.log(f().length);
'''
"bound.js" = '''
const f = () => [1,2,3]; const a = f(); console.log(a[2]);
'''
"allocation_return.js" = '''
const f = (n) => new Array(n).fill(4); function g(x){return x[1];} console.log(g(f(3)));
'''
"mixed_return.js" = '''
const f = (c) => { if (c) { return [1]; } return 0; }; function g(x){return x[0];} console.log(g(f(true)));
'''
"boolean_elements.js" = '''
const f = () => [true, false]; function g(x){return x[0];} console.log(g(f()));
'''
"let_alias.js" = '''
let f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));
'''
"module_alias_called_from_main.js" = '''
const f = () => [1,2,3]; function main(){ function g(x){return x[1];} console.log(g(f())); } main();
'''
"named_fn_expr_iife.js" = '''
function g(x){return x[1];} console.log(g((function h(){ return [1,2]; })()));
'''
"param_dependent_elements.js" = '''
const f = (n) => [n, n]; function g(x){return x[1];} console.log(g(f(3)));
'''
"called_and_passed_as_callback.js" = '''
const f = () => [1, 2]; console.log(f()[1]); const xs = new Array(2).fill(0); xs.forEach(f); console.log("done");
'''
"redeclared_alias.js" = '''
const f = () => [1,2,3]; { const f = () => 2; } function g(x){return x[1];} console.log(g(f()));
'''
"map_callback_returning_array.js" = '''
const xs = new Array(2).fill(1); const ys = xs.map(x => x + 1); console.log(ys[0]);
'''
```

Then the cases. For each `_computes` row the shape is:

```toml
[[case]]
name = "arrow_alias_passed_on_computes"
rationale = """At `068b29950` kali printed `0` at exit 0 where node v26.10.0 prints `2`: the arrow was never an array-return candidate, so its literal return was the placeholder `0`. Spec §2.1 row 1."""
args = ["run", "arrow_passed_on.js"]
exit = "success"
stdout = "2\n"
```

Write one `_computes` case each for `arrow_passed_on.js` (`2\n`), `fnexpr_passed_on.js` (`2\n`), `iife_passed_on.js` (`2\n`), `nested_const.js` (`2\n`), `alias_chain.js` (`2\n`), `side_effect_body.js` (`ran\n2\n`), `direct_index.js` (`1\n`), `direct_length.js` (`3\n`), `bound.js` (`3\n`), `allocation_return.js` (`4\n`, rationale "a pin: already correct at the baseline"), and `map_callback_returning_array.js` (node's output, rationale "a control: a callback is never a candidate"). Each rationale states the baseline output from `baseline-anon.tsv` and the spec row.

The refusal cases come in `check`/`run` pairs (amendment A1 of the array-return project: check and run refuse the same programs), shaped like:

```toml
[[case]]
name = "anon_mixed_return_refuses_at_check"
rationale = """At `068b29950` kali printed node's value or `0` (baseline-anon.tsv `anon_mixed_return`). A directly-called anonymous function that mixes array and non-array returns refuses like a declaration (spec §3.3)."""
args = ["check", "mixed_return.js"]
exit = "failure"
stderr_contains = ["E5506", "returning an array from `f`", "${REFUSED}", "it mixes array and non-array returns"]

[[case]]
name = "anon_mixed_return_refuses_at_run"
rationale = """The `run` twin of the case above."""
args = ["run", "mixed_return.js"]
exit = "failure"
stderr_contains = ["E5506", "returning an array from `f`", "${REFUSED}", "it mixes array and non-array returns"]
```

Write pairs for:
- `mixed_return.js`
- `boolean_elements.js` (reason: whichever `ARRAY_RETURN_*` text Step 1 measured)
- `named_fn_expr_iife.js` (expected `returning an array from `h``, plus the `ARRAY_RETURN_FORM` text)
- `param_dependent_elements.js` (expected an `ELEMENT` refusal; if Step 1 measures it CORRECT, write a `_computes` case instead)

Write single `run` cases (`exit = "failure"`, `stderr_contains = ["E5506", "first-class function value"]`) for `let_alias.js` and `module_alias_called_from_main.js`. Their rationale: "out of scope (spec §1.1); must keep refusing".

For `called_and_passed_as_callback.js` and `redeclared_alias.js`, write the case at whatever Step 1 measured, **provided** it is either node's output or an `E5506` refusal. If either is SILENT (exit 0, output differs from node), do not write a case that pins the silent output. Record it instead for Task 7's followups file, and tell the human partner at the end.

- [ ] **Step 3: Re-pin pre-existing cases the spike listed**

For each case named in `/tmp/claude-anon-spike-losses.md` or in Task 3's commit body: if it now prints node's value, update its `stdout` and append to its rationale: "Re-pinned by anon-array-return (spec §2.1): kali now prints node's value." If it now refuses and did not before, update it to the refusal and list it for Task 7.

- [ ] **Step 4: Run the cases**

Run: `cargo test -p kali_cli --test cases`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cases
git commit -m "test(anon-array-return): node-oracle cases for const-bound and immediately-invoked anonymous array returns"
```

---

### Task 7: Final measurement, bookkeeping, spec amendments

**Files:**
- Create: `docs/superpowers/followups/anon-array-return-discovered-defects.md`
- Modify: `docs/superpowers/followups/array-return-discovered-defects.md` (§1 heading and body, and §9/§13/§16 cross-references if they mention arrows)
- Modify: `docs/superpowers/specs/2026-10-02-anon-array-return-design.md` (append §7 amendments)
- Modify (only if an entry's lane moved): `docs/superpowers/followups/kali-silent-miscompile-register.md`, `docs/superpowers/followups/blast-radius-ranking.md` (regenerated)

- [ ] **Step 1: Full verification**

```bash
cargo test --workspace 2>&1 | tail -5
cargo build -p kali_cli && tools/array-return-probes/run.sh /tmp/claude-anon-final.tsv
grep -c SILENT <(grep '^anon_' /tmp/claude-anon-final.tsv)
join -t$'\t' <(cut -f1,2 /tmp/claude-anon-pre.tsv | sort) <(cut -f1,2 /tmp/claude-anon-final.tsv | sort) | awk -F'\t' '$2!=$3'
```

Expected:
- `cargo test --workspace`: all pass, including `kali_blast_radius::ranking::ranking_tests::spliced_document_matches_the_generator`.
- SILENT count on `anon_*` rows: `0`.
- The diff lists only `anon_*` rows moving `SILENT → CORRECT`, `REFUSES → CORRECT`, or `SILENT → REFUSES`.

- [ ] **Step 2: Measure the Review Focus blind spots**

Run each against node and kali, and record the verdict:

```text
switch_case_alias.js   function g(x){return x[1];} switch (1) { case 1: { const f = () => [1,2,3]; console.log(g(f())); } }
redeclared_alias.js    (from Task 6)
called_and_passed_as_callback.js (from Task 6)
```

- [ ] **Step 3: Write the followups file**

Create `docs/superpowers/followups/anon-array-return-discovered-defects.md`, following the header convention of `array-return-discovered-defects.md`:
- "Filed 2026-10-02 by the anon-array-return project".
- Oracle and measured-at commit.
- Ranked sections, each with a `| program | node | kali |` table and a "What it would cost" paragraph.

Include at minimum:
1. A module-scope `const` arrow called from inside another function, and a `let`-bound arrow. Both REFUSE and need first-class function values.
2. Param-dependent elements (`const f = (n) => [n, n]`), with its measured verdict. Cost: give `__kali_fn_N` call edges, so the R15 param proof can enumerate call sites.
3. A named function expression called immediately refuses `FORM`.
4. The switch-case blind spot (Step 2), with its verdict. Cost: Phase A2's `collect_local_names` does not descend into switch case bodies, which is shared by every lane.
5. The redeclared alias and callback-plus-call rows, with their verdicts.
6. The Task 3 spike's capability-loss list (`/tmp/claude-anon-spike-losses.md`), and which entries Task 4 restored.

- [ ] **Step 4: Mark the picked item FIXED**

In `array-return-discovered-defects.md`, append ` — FIXED by anon-array-return` to the §1 heading, and add a first paragraph to §1:

```markdown
**FIXED** on branch `anon-array-return` (spec
`docs/superpowers/specs/2026-10-02-anon-array-return-design.md`): a `const`-bound
or immediately-invoked anonymous function is an array-return candidate, and its
rows print node's value. The `arrow_return` probe (module-scope `const` called
from `main`) and the object-method row still refuse, and are filed in
`anon-array-return-discovered-defects.md`. The table below is the record as
measured at `d2202ed4b`.
```

- [ ] **Step 5: Append spec amendments**

Append to the spec:

```markdown
---

## 7. Amendments made during planning

* **A-1 (§3.1 "call edges").** `CallEdge.callee` is not rewritten. Only the
  array-return facts resolve it (`called`, feeds, `call_bound`, returns, and the
  R15 discharge), so `resolve_calls`' param-repr inference is not widened to
  anonymous bodies. Consequence: an arrow whose returned elements depend on its
  params cannot discharge the R15 proof and refuses `ELEMENT` (followups §2).
* **A-2 (§3.1 "shadow fact").** No new `ReprTable` fact. `let`/`var`/param names
  are `Blocked` in the alias table, and codegen's existing `locals` belt and
  `is_array_return_callee_shadowed` decline on the source name.
* **A-3 (§5.2 async arrow).** An async arrow follows ruling R12: it is
  `non_taintable`, keeps its pre-project lane, and is not a refusal case.
* **A-4 (§5.3 agreement test).** The both-sides agreement check is realized as
  the `anon_*` probe gate (no `anon_*` probe is SILENT) plus the
  `runtime/anon_array_return.toml` shape matrix, not as a unit test. Codegen's
  resolution is not reachable from `kali_types`.
* **A-5 (§3.3 named function expressions).** An IIFE of a *named* function
  expression is directly called but not a candidate (its id is not
  `__kali_fn_N`), so it refuses `FORM`.
* **A-6 (§5.2 case files).** The cases live in a new
  `runtime/anon_array_return.toml` in the same `cases` target, rather than being
  appended to `array_return.toml` / `array_return_refusals.toml`, so this
  project's baseline (`068b29950`) is not mixed with that one's (`368b5b5ea`).
```

Add any further deviation that Tasks 3-6 forced, in the same form.

- [ ] **Step 6: Register and ranking**

Run: `grep -n "R-14\|anonymous\|arrow" docs/superpowers/followups/kali-silent-miscompile-register.md | head -40`.

If any open register entry's lane is one of this project's rows, amend that entry's §0.2 row with the new verdict and commit. Then run `cargo run -p kali_blast_radius --example rank` and, if the generated region changed, splice it in and confirm the ranking test passes. If nothing in the register moved, write that sentence in the followups file's header instead.

- [ ] **Step 7: Commit**

```bash
git add docs/superpowers
git commit -m "docs(anon-array-return): followups §1 FIXED; file what was measured and not fixed; spec amendments A-1..A-5"
```
