# Throw-fallout Stage 5 — dynamic-import member typeof + AST module-link Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drain throw-fallout bucket #7 (32 tests) honestly: make `typeof ns.member` fold to the real member kind AND make `ns.member()` an actual call into the imported module's function (today both fall open to constant `0`), for provenance-proven module-namespace bindings (`import * as ns from "./x"` and `const ns = await import(<foldable spec>)`).

**Architecture:** A new pre-resolver AST pass (`module_link.rs` in `kali_cli/src/build/`, modeled on the Spec-5 monomorphization playbook) collects namespace provenance, parses the target module, purity-gates it, clones its exported functions into the entry AST under mangled `__link{N}_{name}` names, rewrites `typeof ns.member` to a string literal and `ns.member(...)` to a direct call, and default-denies every leftover use of the namespace binding. Zero kali_types/kali_codegen changes for these lanes by construction — the resolver and codegen see ordinary functions, calls, and literals. Separately, the generic codegen `typeof` fail-open (`I64Const(0)` + warning) is flipped to a fail-closed E5506 error, gated on a measured census (user decision: measure, close if cheap).

**Tech Stack:** Rust workspace; crates `kali_cli` (build pipeline, tests), `kali_ast` (AST structs), `kali_parser`, `kali_lexer`, `kali_codegen` (Task 2 only), `kali_error` (E-codes). Node v26.5.0 as the semantic oracle.

## Global Constraints

- Branch `soundness-batch1-pra`; PR #16 stays **draft**; nothing pushed/merged (program policy).
- **PRIMARY GATE (every checkpoint):** `cargo test --workspace --no-fail-fast` enumeration diffed against BOTH the Task-1 stage-entry snapshot (`stage5-pre.txt`, expected 783±3) and a **main-worktree** enumeration — `comm -13 pre post` must print NOTHING (memory: `ci-gate-vs-poisoned-baseline`).
- **Zero test edits** on the 32 target tests (no re-pins). New tests may be added.
- **Distinguishable-value rule:** every call-lane test/probe must use a return value ≠ 0 (e.g. `7n`) AND a body side effect (`console.log("inside lazyValue")`), asserted against node — `0n` fixtures cannot distinguish a real call from the fail-open `0`.
- **Fail-closed everywhere:** any input outside a proven lane gets a compile-time `e5::FEATURE_UNAVAILABLE` (5506) / existing E-code — never a silent wrong value. Warnings + placeholder-0 are forbidden in new code.
- GC-less invariant: no GC machinery (memory: `kali-gc-less-invariant`). This stage is compile-time only; no new `kali:rt` host imports (the 4 hand-mirrored import lists stay untouched — re-confirm at Task 9).
- Commit after every green task step-cycle; tag messages `[stage5]`.
- Enumeration pipeline (verbatim from Stage-4 triage `docs/superpowers/followups/throw-fallout-stage4-triage.md:296-302`):
  ```bash
  cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort > <out>.txt
  ```
  Output interleaving can FALSE-DRAIN (drop FAILED lines) but cannot fabricate newly-red; reconcile drains by isolation runs.

## Verified code facts (read before implementing)

- `import(<expr>)` parses to `Expression::ImportExpression(Box<ImportExpression { source: Expression }>)` (`crates/kali_ast/src/expression.rs:314`, parser `crates/kali_parser/src/expression/primary.rs:185-193`).
- `export function f() {}` parses to a **plain** `Statement::FunctionDeclaration` — the parser consumes `export` and drops the marker (`crates/kali_parser/src/module.rs:136-138`). True export names need a token scan.
- `FunctionDeclaration { name: String, params: Vec<String>, body: Box<BlockStatement>, is_async: bool, generator: bool }` (`crates/kali_ast/src/declaration.rs:11`).
- `VariableDeclarator { id: String, init: Option<Expression> }`, `VariableDeclaration { declarations, kind: String }` (`declaration.rs:44-55`).
- `MemberExpression { object: Expression, property: String, computed_index: Option<Box<Expression>> }` (`expression.rs:111`); `UnaryExpression { operator: String, argument: Expression }` (`:97`); `CallExpression { callee: Expression, args: Vec<Expression> }` (`:104`); `AwaitExpression { argument: Expression }` (`:277`).
- `ImportSpecifier::Namespace(local)` binding: `crates/kali_types/src/resolve/mod.rs:1023-1025`.
- codegen typeof arm: `crates/kali_codegen/src/emit/operators.rs:152-227`; generic fallback (warning `e8::UNIMPLEMENTED` + `I64Const(0)`) at `operators.rs:214-226`; `typeof_static_text` at `:795-841`.
- E-codes: `DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH = 4008`, `FEATURE_UNAVAILABLE = 5506` (`crates/kali_error/src/_error_codes.rs:86,102`). Reject-diagnostic pattern: `crates/kali_types/src/resolve/expression.rs:2283-2289`.
- Dynamic-chunk discovery + token fold: `crates/kali_cli/src/build/eval.rs` — `pub fn discover_dynamic_import_targets(source: &Path, contents: &str) -> Result<Vec<DynamicImportTarget>, Diagnostic>` (`:264`), `DynamicImportTarget { specifier: String, target: PathBuf }` (`:12`), `fn resolve_dynamic_import_target(source: &Path, specifier: &str) -> Option<PathBuf>` (`:333`, currently private).
- Pass insertion point: `crates/kali_cli/src/build/compile.rs:648` calls `kali_types::monomorphize::monomorphize_statements(&mut parsed.statements)` AFTER `validate_unique_export_names_from_statements` and BEFORE the resolver. The link pass runs immediately BEFORE that monomorphize call.
- Callee-rename precedent: `pub fn rewrite_callees_in_body(...)` `crates/kali_types/src/monomorphize.rs:171` (pattern to mirror; write a local walk, do not force-fit its signature).
- Empirical baseline (fresh binary, this branch): `import { f } from './m'; f()`, `import * as ns; ns.f()`, and `const c = await import('./m'); c.f()` ALL skip the body and yield `0`; `ns.notAnExport()` yields `0`; `console.log(chunk)` prints the specifier string. Reproducers in `/tmp/claude-1000/-workspace/918642bd-8901-448e-a544-86d45e2ec010/scratchpad/dyn5/` (main9-main11).

---

### Task 1: Stage-entry snapshot (denominator capture)

**Files:**
- Create: `$SCRATCH/stage5-pre.txt` (scratch artifact, referenced by every later gate; `$SCRATCH` = the session scratchpad dir)
- Create: `docs/superpowers/followups/throw-fallout-stage5-triage.md` (running triage doc)

**Interfaces:**
- Produces: `stage5-pre.txt` — sorted failing-test-name set at stage entry; the triage doc all later tasks append evidence to.

- [ ] **Step 1: Fresh build + two enumerations**

```bash
cd /workspace && cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort > "$SCRATCH/stage5-pre-run1.txt"
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort > "$SCRATCH/stage5-pre-run2.txt"
comm -3 "$SCRATCH/stage5-pre-run1.txt" "$SCRATCH/stage5-pre-run2.txt"
```

Expected: each file ~783 lines (±3 from interleaving); `comm -3` diff small (interleaving noise only). Union the two runs into `stage5-pre.txt`:

```bash
sort -u "$SCRATCH/stage5-pre-run1.txt" "$SCRATCH/stage5-pre-run2.txt" > "$SCRATCH/stage5-pre.txt"
wc -l "$SCRATCH/stage5-pre.txt"
```

- [ ] **Step 2: Confirm the 32 targets are in the set**

```bash
grep -c "dynamic_import" "$SCRATCH/stage5-pre.txt"
```

Expected: ≥ 32 (26 harness + 6 runtime_smoke names; see bucket list in `docs/superpowers/followups/throw-fallout-stage0-denominator.md:1061-1102`).

- [ ] **Step 3: Start the triage doc and commit**

Create `docs/superpowers/followups/throw-fallout-stage5-triage.md` with: stage-entry count, the exact enumeration commands, the 32-name target list, and a "Baseline reproducers" section recording the main9/main10/main11 mirage evidence (distinguishable `7n` probe: body never runs, prints `0`; node prints `inside lazyValue` / `7`).

```bash
git add docs/superpowers/followups/throw-fallout-stage5-triage.md
git commit -m "docs(soundness): stage5 entry snapshot + call-lane mirage reproducers [stage5]"
```

---

### Task 2: Generic typeof fallback → fail-closed E5506 (+ census)

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs:214-226` (the `"typeof"` arm's final fallback ONLY — do not touch the `delete`/`void` arms or other operators' fallbacks)
- Test: `crates/kali_codegen/src/emit/operators.rs` (inline `#[cfg(test)]` if the file has one; otherwise the crate's existing test home for emit — follow the file's current test convention)
- Modify: `docs/superpowers/followups/throw-fallout-stage5-triage.md` (census section)

**Interfaces:**
- Consumes: nothing from other tasks (independent; runs before the link pass exists).
- Produces: `typeof` on an unproven operand is a compile error `E5506` with message `"typeof is only supported on statically-provable operands in the current direct-runtime path (this operand's type cannot be proven; a silent placeholder would miscompile comparisons)"`. Decision recorded in triage doc: KEPT or REVERTED.

- [ ] **Step 1: Write the failing reproducer test**

A compile-level test: `typeof someRuntimeObj.member` (unproven operand) must produce an E5506 **error**, not a warning + successful compile. Use the crate's existing compile-and-collect-diagnostics test helper (grep `e8::UNIMPLEMENTED` in existing tests for the pattern to invert). The test asserts: diagnostics contain code 5506 with severity error; compilation does NOT produce a wasm artifact.

- [ ] **Step 2: Run it — expect FAIL** (today: warning + successful compile).

- [ ] **Step 3: Implement the flip**

Replace `operators.rs:214-226`:

```rust
// Fail-closed (throw-fallout Stage 5): an unproven `typeof` operand used to
// compile to a silent `I64Const(0)` placeholder — never equal to any interned
// type-name string, so every `typeof x === '...'` guard silently took the
// wrong branch (bucket #7's root enabler). Reject at compile time instead.
self.diagnostics.push(Diagnostic::error(
    e5::FEATURE_UNAVAILABLE as u32,
    "typeof is only supported on statically-provable operands in the current direct-runtime path (this operand's type cannot be proven; a silent placeholder would miscompile comparisons)".to_string(),
));
function.instruction(&Instruction::I64Const(0));
EmittedValue {
    produced: true,
    shape: ValueShape::Unknown,
}
```

(Keep emitting the placeholder instruction so the wasm stays structurally valid — the error diagnostic already fails the build; mirror how other emit-time E5506 rejects in this file handle the instruction stream. Check `e5` is imported in this file; add `use` if needed.)

- [ ] **Step 4: Run the new test — expect PASS.** Also run `cargo test -p kali_codegen` — expect no new failures in-crate.

- [ ] **Step 5: CENSUS — full workspace**

```bash
cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort > "$SCRATCH/stage5-typeof-census.txt"
comm -13 "$SCRATCH/stage5-pre.txt" "$SCRATCH/stage5-typeof-census.txt" > "$SCRATCH/stage5-typeof-newlyred.txt"
wc -l "$SCRATCH/stage5-typeof-newlyred.txt"; cat "$SCRATCH/stage5-typeof-newlyred.txt"
```

- [ ] **Step 6: Decision rule (user-approved: measure, close if cheap)**

- **≤ ~8 newly-red, all explainable as "green test compiled an unproven typeof and silently took the 0 branch":** KEEP the flip. Isolation-run each newly-red name, record mechanism per bucket in the triage doc, and file each as an explicit fix-or-extend item; if a fix is a one-liner provable-lane extension (e.g. a new literal kind in `typeof_static_text`), do it inside this task with its own red-test-first cycle. Newly-red names that remain must be FIXED before Task 9's gate (the checkpoint demands 0 newly-red) — if any can't be fixed in-stage, that forces the REVERT branch instead.
- **Otherwise (large/unclear blast radius):** REVERT Steps 3-4 to the warning fallback (`git checkout -p` the hunk), keep the reproducer test but mark it `#[ignore = "generic typeof fail-open closure deferred; census attached in stage5 triage"]`, and paste the census into the triage doc as the follow-up's sizing evidence. The namespace-member typeof surface still closes structurally via Task 6 (the AST rewrite eliminates those operands before codegen).

- [ ] **Step 7: Record census + decision in the triage doc; commit**

```bash
git add -A && git commit -m "fix(codegen): typeof unproven-operand fallback fail-closed E5506 + census [stage5]"
```

(or `docs(soundness): typeof fail-open census — closure deferred [stage5]` on the revert branch.)

---

### Task 3: `module_link.rs` — provenance collection + AST specifier fold

**Files:**
- Create: `crates/kali_cli/src/build/module_link.rs`
- Modify: `crates/kali_cli/src/build/mod.rs` (or `lib.rs` — wherever sibling modules like `eval` are declared; add `pub mod module_link;`)
- Modify: `crates/kali_cli/src/build/eval.rs:333` (`fn resolve_dynamic_import_target` → `pub(crate) fn`)
- Test: `crates/kali_cli/src/build/module_link.rs` (inline `#[cfg(test)] mod tests`, matching eval.rs's convention — check whether eval.rs uses inline tests or a sibling `_tests.rs` file and follow it)

**Interfaces:**
- Consumes: `eval::resolve_dynamic_import_target(source, specifier)`, `eval::discover_dynamic_import_targets(source, contents)`.
- Produces:
  ```rust
  pub struct NamespaceProvenance {
      /// binding name → linked module
      pub bindings: BTreeMap<String, LinkedModule>,
  }
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct LinkedModule {
      pub path: PathBuf,
      /// stable per-module ordinal, assigned in first-seen order; mangled
      /// names are `__link{index}_{export}`
      pub index: usize,
  }
  pub fn collect_namespace_provenance(
      source_path: &Path,
      source_contents: &str,
      statements: &[Statement],
  ) -> NamespaceProvenance
  ```
  Internal: `fn fold_import_specifier(expr: &Expression, consts: &BTreeMap<String, String>) -> Option<String>`.

- [ ] **Step 1: Write the failing unit tests** (parse a source string with `kali_parser`, call `collect_namespace_provenance`, assert the map). Cover, minimally:

```rust
// positive: each red-fixture specifier shape must yield provenance
// 1. import * as ns from "./util.js";
// 2. const c = await import("./lazy.js");
// 3. const name = "lazy.js"; const c = await import(`./${name}`);
// 4. const c = await import((0, `./${name}`));
// 5. const c = await import(Object.freeze((null ?? "./lazy.js")));
// 6. const c = await import(Object.freeze((true && "./lazy.js")));
// 7. const c = await import(Object.freeze((false || "./lazy.js")));
// negative: NO provenance (binding absent from map) for
// 8. const c = await import(runtimeName());        // non-foldable
// 9. const c = await import(`./${runtimeVar}`);    // non-const part
// 10. import * as fs from "fs";                    // non-relative source
// 11. let c = await import("./lazy.js");           // `let`, not const — mutable rebinding unproven
```

Tests need real files on disk for target resolution — use `tempfile::tempdir()` (already a dev-dependency of kali_cli; confirm in `crates/kali_cli/Cargo.toml`, add if test-scope-missing) and write `util.js`/`lazy.js` stubs.

- [ ] **Step 2: Run — expect FAIL** (`module_link` doesn't exist): `cargo test -p kali_cli module_link -- --test-threads=4` → compile error.

- [ ] **Step 3: Implement**

```rust
//! Pre-resolver AST module-linking pass (throw-fallout Stage 5).
//!
//! Detects provenance-proven module-namespace bindings, links the target
//! module's exported functions into the entry AST under mangled names, and
//! rewrites the proven member uses. Everything outside the proven lane is
//! fail-closed E5506 — the binding otherwise holds the raw specifier string
//! (a silent-miscompile leak).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use kali_ast::*;

pub fn collect_namespace_provenance(
    source_path: &Path,
    source_contents: &str,
    statements: &[Statement],
) -> NamespaceProvenance {
    // (a) module-scope single-declarator string consts, for template parts:
    let mut consts: BTreeMap<String, String> = BTreeMap::new();
    for statement in statements {
        if let Statement::VariableDeclaration(decl) = statement {
            if decl.kind == "const" {
                for d in &decl.declarations {
                    if let Some(Expression::Literal(LiteralValue::String(value))) = &d.init {
                        consts.insert(d.id.clone(), unquote(value));
                    }
                }
            }
        }
    }
    // NOTE: also walk into top-level `async function` bodies for their local
    // consts — the fixtures declare `const name = "lazy.js"` INSIDE main().
    // Collect per-function consts when scanning that function's declarators
    // (a declarator and its specifier-part const live in the same body in
    // every fixture; keep scoping simple: consts visible to a declarator are
    // module-scope consts + consts declared earlier in the SAME function body).

    // (b) provenance sources:
    let mut bindings = BTreeMap::new();
    let mut next_index = 0usize;
    let mut path_index: BTreeMap<PathBuf, usize> = BTreeMap::new();
    visit_declarators_and_imports(statements, &mut |site| match site {
        Site::NamespaceImport { local, source } => {
            if source.starts_with("./") || source.starts_with("../") {
                if let Some(target) =
                    crate::build::eval::resolve_dynamic_import_target(source_path, source)
                { register(&mut bindings, &mut path_index, &mut next_index, local, target); }
            }
        }
        Site::ConstAwaitImport { local, specifier_expr, scope_consts } => {
            if let Some(spec) = fold_import_specifier(specifier_expr, scope_consts) {
                if let Some(target) =
                    crate::build::eval::resolve_dynamic_import_target(source_path, &spec)
                { register(&mut bindings, &mut path_index, &mut next_index, local, target); }
            }
        }
    });
    NamespaceProvenance { bindings }
}
```

`fold_import_specifier` handles exactly: `Literal(String)` (template literals arrive as string literals — reuse `kali_common::template::resolve_interpolated_template_literal` with a lookup into `consts`, mirroring `eval.rs:648-670`), `ParenthesizedExpression`, `SequenceExpression` → last element, `CallExpression{callee: MemberExpression{object: Identifier("Object"), property: "freeze"}}` → fold single arg, `BinaryExpression`/logical forms for `??`/`&&`/`||` with foldable operands (fold both sides, apply JS truthiness on the literal LHS: `null ?? x` → x, `true && x` → x, `false || x` → x; anything unprovable → `None`), `Identifier` → `consts` lookup, `+` concat of foldables. **Check first how the parser represents `??`/`&&`/`||`** — there is no LogicalExpression node (memory: `kali-runtime-join-spec3`); grep `kali_parser` for how `??` parses (likely `BinaryExpression` with those operator strings) and match that. Everything else → `None` (no provenance — downstream default-deny handles uses).

`ConstAwaitImport` detection: `VariableDeclaration { kind: "const" }` declarator with `init = Some(AwaitExpression { argument: ImportExpression { source } })`, unwrapping `ParenthesizedExpression` around either layer.

- [ ] **Step 4: Run — expect PASS:** `cargo test -p kali_cli module_link -- --test-threads=4`

- [ ] **Step 5: Commit** — `git add -A && git commit -m "feat(build): module_link provenance collection + AST specifier fold [stage5]"`

---

### Task 4: Target-module load, purity gate, true-export census

**Files:**
- Modify: `crates/kali_cli/src/build/module_link.rs`
- Test: same file's test module

**Interfaces:**
- Consumes: `LinkedModule` from Task 3.
- Produces:
  ```rust
  pub struct LinkedModuleAst {
      pub index: usize,
      /// export name → the parsed function declaration
      pub exports: BTreeMap<String, FunctionDeclaration>,
      /// ALL top-level function names (exports + private helpers), for
      /// sibling-callee renames in Task 5
      pub all_functions: BTreeMap<String, FunctionDeclaration>,
  }
  /// Err(diagnostic) = purity-gate reject (E5506, names the module + reason)
  pub fn load_linked_module(module: &LinkedModule) -> Result<LinkedModuleAst, Diagnostic>
  ```

- [ ] **Step 1: Failing unit tests:**

```rust
// PASS the gate:  "export function lazyValue() { return 7n; }"
// PASS: two functions, one exported: "function helper() { return 1n; } export function f() { return helper(); }"
// REJECT (each asserts an Err with code 5506 and a message naming the module path + offending construct):
//   top-level statement:      "console.log('boot'); export function f() {}"
//   top-level import:         "import { x } from './other.js'; export function f() {}"
//   non-function export:      "export const value = 7;"
//   async export:             "export async function f() {}"
//   generator export:         "export function* f() {}"
//   class:                    "export class C {}"
```

- [ ] **Step 2: Run — expect FAIL** (function not defined).

- [ ] **Step 3: Implement.** Read the file (`std::fs::read_to_string`), lex + parse (mirror `eval.rs:625`: `Parser::new(FileId::new(0), lexed.tokens)`). Purity gate: every top-level `Statement` must be `Statement::FunctionDeclaration` with `is_async == false && generator == false`; anything else → `Err(Diagnostic::error(e5::FEATURE_UNAVAILABLE, format!("module '{}' cannot be linked for namespace member access: its top level contains {} — only plain `export function` declarations are supported in the current direct-runtime path", path.display(), what)))`. True-export census: token-scan the module source (`Lexer`) for `Export` immediately followed by `Function` (or `Async`+`Function` — already rejected) and take the following `Identifier` token as an export name; `exports` = that set ∩ `all_functions`. (The parser erases the `export` marker — `kali_parser/src/module.rs:136-138` — hence the token scan.)

- [ ] **Step 4: Run — expect PASS.**

- [ ] **Step 5: Commit** — `git commit -am "feat(build): linked-module purity gate + true-export census [stage5]"`

---

### Task 5: Clone + mangle + sibling-callee rename + append + collision guard

**Files:**
- Modify: `crates/kali_cli/src/build/module_link.rs`
- Test: same file's test module

**Interfaces:**
- Consumes: `LinkedModuleAst` (Task 4), `NamespaceProvenance` (Task 3).
- Produces:
  ```rust
  /// Appends mangled clones of `module.all_functions` to `statements`.
  /// Mangle: `__link{module.index}_{original_name}`. Sibling references
  /// inside cloned bodies are renamed to their mangled forms.
  /// Err = mangled-name collision with an already-declared entry name (E5506).
  pub fn append_linked_functions(
      statements: &mut Vec<Statement>,
      module: &LinkedModuleAst,
  ) -> Result<(), Diagnostic>
  ```

- [ ] **Step 1: Failing unit tests:** (a) single export appended as `__link0_lazyValue`, body byte-identical apart from the name; (b) module with `helper()` called from exported `f()`: BOTH appended, the call inside `__link0_f`'s body renamed to `__link0_helper`; (c) entry AST already containing `function __link0_lazyValue() {}` → `Err` 5506.

- [ ] **Step 2: Run — expect FAIL.**

- [ ] **Step 3: Implement.** Collision guard: collect every `FunctionDeclaration` name + declarator id already in `statements`; if any equals a mangled name → Err. Sibling rename: a recursive `fn rename_callees(statement/expression, renames: &BTreeMap<String, String>)` walk that rewrites `CallExpression { callee: Expression::Identifier(name), .. }` when `name ∈ renames` — mirror the traversal shape of `rewrite_callees_in_body` (`kali_types/src/monomorphize.rs:171`) but keep it local to `module_link.rs` (its signature is plan-indexed, not name-keyed). Bare non-call references to a sibling function name (e.g. `const g = helper;`) inside a cloned body → return `Err` 5506 ("linked module function aliases a sibling export — unsupported"): detect during the walk (an `Identifier(name ∈ renames)` in non-callee position).

- [ ] **Step 4: Run — expect PASS.**

- [ ] **Step 5: Commit** — `git commit -am "feat(build): linked-function clone/mangle/append + sibling rename [stage5]"`

---

### Task 6: Rewrite walk — typeof fold, member-call rewrite, non-export rejects

**Files:**
- Modify: `crates/kali_cli/src/build/module_link.rs`
- Test: same file's test module

**Interfaces:**
- Consumes: `NamespaceProvenance`, `LinkedModuleAst` maps.
- Produces:
  ```rust
  /// Rewrites proven uses of namespace bindings in place; pushes E5506
  /// diagnostics for uses of non-exported members.
  pub fn rewrite_namespace_uses(
      statements: &mut Vec<Statement>,
      provenance: &NamespaceProvenance,
      modules: &BTreeMap<usize, LinkedModuleAst>,
      diagnostics: &mut Vec<Diagnostic>,
  )
  ```

- [ ] **Step 1: Failing unit tests** (parse entry source → run Tasks 3-6 pipeline → assert on the rewritten AST and diagnostics):

```rust
// FOLD:    typeof ns.lazyValue     → Literal string "function"   (member ∈ exports)
// FOLD:    typeof ns.missing       → Literal string "undefined"  (member ∉ exports; sealed namespace)
// REWRITE: ns.lazyValue(a, b)      → CallExpression{callee: Identifier("__link0_lazyValue"), args:[a,b]}
// REWRITE: await ns.lazyValue()    → await __link0_lazyValue()   (await wrapper preserved untouched)
// REJECT:  ns.notAnExport()        → E5506 "module '<path>' does not export 'notAnExport'" (node: TypeError)
// REJECT:  ns.helper()  (private)  → same E5506 (helper linked for sibling calls but NOT exported)
// FOLD literal equality: the produced "function" literal must be constructed
//   identically to how the parser builds the literal in `x !== 'function'`
//   — write the assertion by PARSING `("function")` and comparing the two
//   Expression values for equality, so the quoting convention is proven, not
//   assumed.
```

- [ ] **Step 2: Run — expect FAIL.**

- [ ] **Step 3: Implement.** Deep pre-order walk over all statements/expressions (including function bodies, arrow bodies, template parts, call args, declarator inits). At each node:
  - `UnaryExpression { operator: "typeof", argument: MemberExpression { object: Identifier(ns), property, computed_index: None } }` with `ns ∈ provenance` → replace whole node with the string-literal Expression (constructed via the parser-equality convention proven in Step 1).
  - `CallExpression { callee: MemberExpression { object: Identifier(ns), property, computed_index: None }, args }` with `ns ∈ provenance` → `property ∈ module.exports` ? replace callee with `Identifier(mangled)` : push E5506 reject (leave node — the build fails on the error).
  - Computed access `ns[expr]` (`computed_index: Some(_)`) in either position → push E5506 ("computed member access on a module namespace is unavailable…").
  - Nothing else is rewritten here — leftover `ns` uses are Task 7's default-deny.

- [ ] **Step 4: Run — expect PASS.**

- [ ] **Step 5: Commit** — `git commit -am "feat(build): namespace typeof fold + member-call rewrite + non-export rejects [stage5]"`

---

### Task 7: Default-deny leftovers + shadowing guard + pipeline wiring

**Files:**
- Modify: `crates/kali_cli/src/build/module_link.rs` (add `deny_unrewritten_uses` + top-level `pub fn link_provable_module_namespaces`)
- Modify: `crates/kali_cli/src/build/compile.rs` (insert the pass call immediately before the `monomorphize_statements` call at `compile.rs:648`)
- Test: `module_link.rs` test module + one compile-level test

**Interfaces:**
- Consumes: everything from Tasks 3-6.
- Produces:
  ```rust
  /// The single public entry point compile.rs calls. Runs collect → load →
  /// append → rewrite → deny. Pushes diagnostics; never silently rewrites
  /// partially. No provenance found → guaranteed no-op (statements untouched).
  pub fn link_provable_module_namespaces(
      source_path: &Path,
      source_contents: &str,
      statements: &mut Vec<Statement>,
      diagnostics: &mut Vec<Diagnostic>,
  )
  ```

- [ ] **Step 1: Failing unit tests:**

```rust
// DENY (each → E5506 naming the binding and the allowed positions, message
// modeled on the Spec-4a for-in reject at kali_types/src/resolve/expression.rs:2283-2289):
//   console.log(chunk);            // value leak — today prints "./lazy.js"
//   const s = chunk + '';          // string coercion leak
//   const alias = chunk;           // alias copy
//   f(chunk);                      // argument escape
//   return chunk;                  // return escape
// SHADOWING GUARD: any SECOND binding of a provenance name anywhere in the
//   entry AST (let/const/var/function/param) → E5506 (provenance would be
//   scope-ambiguous):
//   const chunk = await import("./lazy.js"); { const chunk = 5; }  → E5506
// NO-OP GUARANTEE: source with no namespace bindings → statements deep-equal
//   before/after the pass (assert with PartialEq).
// ALLOWED (no diagnostic): the binding's own declarator; statement-form
//   `await import("./x.js");` with NO binding (must stay untouched — 39
//   green tests depend on it).
```

- [ ] **Step 2: Run — expect FAIL.**

- [ ] **Step 3: Implement.** `deny_unrewritten_uses`: walk the post-rewrite AST; every remaining `Identifier(name ∈ provenance)` OUTSIDE (a) its own declarator id / import-specifier position pushes E5506. Shadowing guard: collect all declared names (function names, declarator ids, params) with a counter; count > 1 for a provenance name → E5506. Then wire `compile.rs`:

```rust
// AST module-linking (throw-fallout Stage 5). Runs BEFORE monomorphize so
// linked functions participate in specialization, and AFTER export-name
// validation (mangled `__link` names must not collide — the pass re-checks).
crate::build::module_link::link_provable_module_namespaces(
    source_path,
    &source_contents,
    &mut parsed.statements,
    &mut diagnostics,
);
if has_errors(&diagnostics) {
    return Err(diagnostics);
}
kali_types::monomorphize::monomorphize_statements(&mut parsed.statements);
```

(Check what `source_contents` variable is in scope at `compile.rs:648` — the parsed source text is read earlier in the same function; reuse it. If only a path is available, `fs::read_to_string` it once.)

- [ ] **Step 4: Run unit tests — expect PASS.** Then the two mirage reproducers by hand:

```bash
cd $SCRATCH/dyn5 && /workspace/target/debug/kali run main9.js   # static import * as, 7n probe
```
Expected NOW: `inside lazyValue` / `7` / `main loaded` — byte-identical to `node main9.js`.

```bash
KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node /workspace/target/debug/kali run --api browser --max-threads 0 --max-spawned-processes 0 main10.js
```
Expected: `inside lazyValue` / `7` / `main loaded`, exit 0.

- [ ] **Step 5: Commit** — `git commit -am "feat(build): default-deny namespace leftovers + wire link pass into compile [stage5]"`

---

### Task 8: Acceptance test file + adversarial re-mask probes

**Files:**
- Create: `crates/kali_cli/tests/module_namespace_link.rs` (standalone integration file, modeled on `browser_template_literal_dynamic_import_harness.rs`: `kali_bin()` helper, `tempdir()`, `Command`)
- Modify: `docs/superpowers/followups/throw-fallout-stage5-triage.md` (re-mask evidence)

**Interfaces:**
- Consumes: the full pipeline from Task 7.
- Produces: the stage's distinguishable-value acceptance suite (the 32 legacy tests CANNOT distinguish a real call from fail-open 0 — this file is the load-bearing evidence).

- [ ] **Step 1: Write the tests** (all must pass on the Task-7 build; write them, run them, and fix the product — not the tests — if any fail):

```rust
// GREEN (each asserts exact stdout lines + exit 0, values chosen ≠ 0):
//  static_namespace_member_call_runs_body_and_returns_value
//      util.js: export function lazyValue() { console.log("inside lazyValue"); return 7n; }
//      main.js: import * as ns from "./util.js";
//               if (typeof ns.lazyValue !== 'function') { throw new Error('missing'); }
//               console.log(String(ns.lazyValue())); console.log("main loaded");
//      expect stdout: "inside lazyValue\n7\nmain loaded\n" via plain `kali run`
//  dynamic_import_member_call_runs_body_and_returns_value
//      same body via: const chunk = await import("./lazy.js"); … await chunk.lazyValue()
//      browser lane: KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node, --api browser
//  dynamic_import_template_literal_specifier_variant (const name = "lazy.js"; `./${name}`)
//  typeof_missing_member_is_undefined_string
//      main.js: import * as ns from "./util.js";
//               console.log(typeof ns.missing); console.log(typeof ns.lazyValue);
//      expect stdout: "undefined\nfunction\n" (compare against node)
//  two_modules_same_export_name_route_to_respective_bodies
//      a.js: export function tag() { console.log("inside A"); return 1n; }
//      b.js: export function tag() { console.log("inside B"); return 2n; }
//      main: import * as a from "./a.js"; import * as b from "./b.js";
//            console.log(String(a.tag())); console.log(String(b.tag()));
//      expect: "inside A\n1\ninside B\n2\n"
// REJECT (assert exit != 0 AND stderr contains "E5506"):
//  namespace_value_leak_rejected            (console.log(chunk))
//  non_export_member_call_rejected          (chunk.notAnExport())
//  impure_target_module_rejected            (chunk module with top-level console.log,
//                                            imported WITH a binding — the statement-form
//                                            sibling test below stays green)
// GREEN guard (unchanged behavior):
//  statement_form_side_effect_import_stays_green
//      main.js: async function main(){ await import("./lazy.js"); console.log("main loaded"); } main();
//      lazy.js WITH top-level console.log — plain `kali run` — expect exit 0, "main loaded"
//      (documents the pre-existing chunk-never-runs divergence; do NOT assert "lazy loaded")
```

- [ ] **Step 2: Run the file:** `cargo test -p kali_cli --test module_namespace_link -- --test-threads=4` — expect ALL PASS. Any failure = product bug; fix in the responsible task's code with a red-test-first cycle before proceeding.

- [ ] **Step 3: Adversarial re-mask probes** (temporary local sabotage, evidence recorded then REVERTED — nothing committed):
  1. In `rewrite_namespace_uses`, make the typeof fold return `"undefined"` for exported functions → rebuild → the harness fixture reproducer (`main.js` from `$SCRATCH/dyn5`) must FAIL with `missing lazyValue export` (honest trap). Revert.
  2. In `append_linked_functions`, skip appending (link to nothing — calls resolve to an unknown identifier) → rebuild → `module_namespace_link` distinguishable tests must FAIL (no `inside lazyValue`). Revert.
  Record both outcomes (command + observed failure) in the triage doc.

- [ ] **Step 4: Commit** — `git add -A && git commit -m "test(cli): stage5 distinguishable-value acceptance suite + re-mask evidence [stage5]"`

---

### Task 9: Bucket verification + full-workspace gate CHECKPOINT

**Files:**
- Modify: `docs/superpowers/followups/throw-fallout-stage5-triage.md` (gate numbers, drain table)
- Create: memory file `/home/dev/.claude/projects/-workspace/memory/kali-throw-fallout-stage5.md` + MEMORY.md index line (controller does this, not a subagent)

**Interfaces:**
- Consumes: everything; `stage5-pre.txt` from Task 1.

- [ ] **Step 1: Isolation-run the 32 targets on a fresh build**

```bash
cargo build -p kali_cli
cargo test -p kali_cli --test browser_template_literal_dynamic_import_harness -- --test-threads=4
cargo test -p kali_cli --test runtime_smoke dynamic_import -- --test-threads=4
```

Expected: harness 26/26 pass; runtime_smoke dynamic_import 45/45 (39 previously green + 6 drained), 0 failed.

- [ ] **Step 2: Full-workspace enumeration ×2 + PRIMARY GATE**

```bash
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort > "$SCRATCH/stage5-post.txt"
comm -13 "$SCRATCH/stage5-pre.txt" "$SCRATCH/stage5-post.txt"   # PRIMARY GATE: must print NOTHING
comm -23 "$SCRATCH/stage5-pre.txt" "$SCRATCH/stage5-post.txt"   # drain list
```

Run twice; union. PRIMARY GATE non-empty → STOP, triage each name (WAT-census / isolation-run to distinguish product regression from test-census desync — Stage-4 lesson — e.g. check `count_tag_boxing_ops` `SYNTHETIC_FUNCTIONS` at `runtime_smoke.rs:802` even though no new synthetic is expected), fix, re-run. Do not proceed red.

- [ ] **Step 3: Main-worktree cross-check** (memory: `ci-gate-vs-poisoned-baseline`)

```bash
git worktree add /tmp/kali-main-gate main 2>/dev/null || true
cd /tmp/kali-main-gate && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort > "$SCRATCH/main-post.txt"; cd /workspace
comm -13 "$SCRATCH/main-post.txt" "$SCRATCH/stage5-post.txt" | comm -13 "$SCRATCH/stage5-pre.txt" -
```

Expected: empty (nothing red on-branch that is green on main, beyond what stage entry already had).

- [ ] **Step 4: Reconcile the drain** — isolation-run every name in the drain list (exact-name filters per owning test binary); classify real vs interleaving false-drain in a per-bucket table in the triage doc (Stage-4 format, `throw-fallout-stage4-triage.md:326+`).

- [ ] **Step 5: Confirm no import-list drift** — `git diff main -- crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs | grep -c "kali:rt"` expected 0 (no host-import changes this stage).

- [ ] **Step 6: Commit checkpoint + write stage memory**

```bash
git add -A && git commit -m "docs(soundness): stage5 checkpoint — bucket #7 drained, gate numbers + drain table [stage5]"
```

Memory file records: denominator before→after, the call-lane-mirage lesson (0n coincidence — distinguishable-value probes are mandatory), the AST-link primitive, the typeof-census decision, and follow-up inventory (statement-form chunk-never-runs divergence; static named-import `import { f }; f()` fail-open if the census deferred it; non-function export kinds; `let`-bound namespaces).

---

## Self-review notes (spec-coverage check, done at plan-write time)

- Spec component 1 (provenance) → Task 3. Component 2 (typeof fold both-sides) → Tasks 6 + 2 (the AST fold makes the codegen/kali_types twin-arm question moot for namespace operands — they never reach codegen; the generic closure is Task 2). Component 3 (positional allowlist default-deny) → Task 7. Component 4 (measured generic closure) → Task 2. Component 5 (call lane via AST linking) → Tasks 4-7. Component 6 (static named-import `import { f }` fail-open) → NOT fixed by this plan's lane (it has no namespace binding); Task 2's census + Task 9's follow-up inventory carry it, per the spec's "census decides, otherwise documented follow-up".
- Spec testing section → Task 8 (distinguishable probes, re-mask), Task 9 (gates). Spec error-handling table → Tasks 4 (purity), 6 (non-export, computed), 7 (leftovers, shadowing), 2 (generic typeof).
- Type-consistency: `NamespaceProvenance`/`LinkedModule` (Task 3) → consumed by Tasks 4-7 with matching signatures; mangle scheme `__link{index}_{name}` used identically in Tasks 5, 6, 8.
