# A block-scoped binding is its own binding, and each loop iteration gets its own

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `6345f082b` (`main`, the class-instances merge) |
| kali binary | `kali 0.1.0`, `target/debug/kali`, built at the baseline (`cargo build -p kali_cli`, `dev` profile) |
| oracle | `node v26.10.0` |
| measured on | 2026-10-04 |
| item picked | register R-10 ("Block-scoped `let`/`const` shadowing is unmodeled", `docs/superpowers/followups/kali-silent-miscompile-register.md`), as filed in `class-instances-discovered-defects.md` §6.11 and §6.15 |
| defects this closes | R-10; class-instances followups §6.11 and §6.15; literal-array-mutators followups §14, third bullet; unresolved-member-call followups §6.9; the per-iteration capture defect of §2.2 (new, filed in the register by this project) |

**Scope was chosen by the human partner:**

* **Item:** block scoping, over the plain-object lane (`mk().n`), the
  unresolved-member-call §1 `check` / `run` gap, and real growable mutators.
* **Depth:** real (option 1 of three; the others were "fail closed now" and
  "fail closed, then real as a second project").
* **Slice:** the core, plus all three neighbouring shapes offered:
  same-named function (and class) declarations, per-iteration loop bindings
  for closures, and sibling-block retyping. The design author had recommended
  refusing per-iteration bindings; the human partner took them in.
* **Mechanism:** approach A of three, a scope-aware AST rename pass before
  monomorphize (§4 records B and C), plus a MIR / codegen change for
  per-iteration env records (§3.3).
* **`check`:** mirrors every refusal the AST can see. The depth-2 capture
  refusal stays run-only, as it is at the baseline (§3.4), and is recorded as
  a `check` / `run` gap rather than claimed.

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** a `let`, `const`, `class` or function declaration inside a
block is a separate binding from any same-named binding outside it, and a
closure that outlives a loop iteration sees that iteration's bindings. At the
baseline every later stage of kali keys bindings by name, flat per function
(and, for functions and classes, flat per program), so an inner declaration
overwrites the outer one at exit 0 (§2). After this project, for the shapes in
§1 items 1 to 6, `kali run` gives node's output; for the shapes in §1.1 that
the lowering cannot prove, `kali run` refuses with `E5506`, and `kali check`
refuses wherever §3.4 says it does. Concretely:

1. A `let` / `const` / `class` in a bare block, an `if` arm, a loop body, a
   `for` / `for-in` / `for-of` head, or a `switch` that shadows an outer
   binding (a module binding, a function-level local or `var`, or a
   parameter) does not change the outer binding. At top level and in
   functions alike.
2. A function that reads a module binding reads the module-level one, not a
   same-named block binding declared later at top level (`closure_outer`).
3. Sibling blocks may declare the same name with different types
   (`{ let a=1 } { let a="s" }`); the baseline's
   "used as both a string and a number" refusal for that shape is lifted.
4. Two function declarations (or two class declarations) of one name, in
   different blocks or different enclosing functions, are separate
   functions (classes).
5. A closure that escapes its loop iteration (registered as a deferred
   callback, stored, or returned) and captures a `let` / `const` declared in
   that loop's head or body sees that iteration's value. For
   `for (let …; …; update)` the head bindings are copied into the next
   iteration's record before `update` runs, as ECMA-262's
   CreatePerIterationEnvironment does.
6. Diagnostics, and any function or class name the program can observe,
   show the name as written.
7. A program that shadows nothing and has no escaping loop capture compiles
   byte-identically to the baseline (§5.3).

### 1.1 What this project does NOT claim

* **No depth-2 captures.** A closure in an env-owning loop (§3.3) that also
  captures a binding of the enclosing function sees that binding at env depth
  2. The baseline's depth-2 refusal fires (`depth2`, `loopmix` in §2.2). It is
  run-only, and `check` exits 0 on it, as at the baseline.
* **No `await` / `yield` in an env-owning loop.** Refused under `run` and
  `check` (§3.4); `asyncloop` prints `2 2` silently at the baseline.
* **No `eval`.** With `--compat eval` on, a program the rename pass changes
  refuses (§3.4).
* **No `catch` or `switch`-fallthrough shapes.** `try` / `catch` and true
  fallthrough are refused at the baseline for unrelated reasons, and still
  are. The rename pass handles `catch` params and `switch` scopes so that a
  later project lifting those refusals inherits correct scoping, but no
  claim is made for them here.
* **No sloppy-mode Annex B.** A function declared in a block is visible only
  in that block (ES module semantics). A call from outside the block is an
  undefined identifier (E3100), as the resolver already reports.
* **Per-iteration records are never freed.** One record per iteration of an
  env-owning loop is allocated in the never-reset global region, the same
  class as the baseline's one record per activation
  (`kali_codegen/src/closure.rs:1-20`). A long-running loop grows memory
  linearly.
* **No proof-boundary change.** `proofs/BOUNDARY.md` does not cover this
  lowering.

---

## 2. What was measured

Every program below becomes a probe `tools/array-return-probes/probes/bs_*.js`
in Task 0 (§5.1). Exit codes were measured separately from the output.

### 2.1 The silent surface: shadowing

| probe | program | node | kali `run` | `check` |
|---|---|---|---|---|
| `top_if` | `let x=1; if(true){ let x=2; console.log(x); } console.log(x);` | `2 1` | `2 2`, exit 0 | 0 |
| `top_block_const` | `const x=1; { const x=2; console.log(x); } console.log(x);` | `2 1` | `2 2`, exit 0 | 0 |
| `fn_if` | `function f(){ let x=1; if(true){ let x=2; console.log(x); } return x; } console.log(f());` | `2 1` | `2 2`, exit 0 | 0 |
| `fn_param` | `function f(c){ { const c=7; console.log(c); } return c; } console.log(f(2));` | `7 2` | `7 7`, exit 0 | 0 |
| `for_let` | `let i=100; for(let i=0;i<2;i++){ console.log(i); } console.log(i);` | `0 1 100` | `0 1 2`, exit 0 | 0 |
| `for_body` | `let s=0; const v=9; for(let i=0;i<2;i++){ const v=i*10; s+=v; } console.log(s, v);` | `10 9` | `10 10`, exit 0 | 0 |
| `while_shadow` | `let n=3; let k=0; while(k<2){ let n=k; k++; } console.log(n);` | `3` | `1`, exit 0 | 0 |
| `arr_shadow` | `const a=[1,2,3]; { const a=[9]; console.log(a.length); } console.log(a.length);` | `1 3` | `1 1`, exit 0 | 0 |
| `nested_fn` | `let x=1; { let x=2; function g(){ return x; } console.log(g()); } console.log(x);` | `2 1` | `2 2`, exit 0 | 0 |
| `closure_outer` | `let x=1; function g(){ return x; } { let x=2; console.log(g()); }` | `1` | `2`, exit 0 | 0 |

Refused at the baseline because of the flat naming (lifted by §1 item 3):

| probe | program | node | kali `run` |
|---|---|---|---|
| `sibling` | `function f(){ { let a=1; console.log(a); } { let a="s"; console.log(a); } } f();` | `1 s` | E5506 `binding a in f is used as both a string and a number` |

Not silent, invalid wasm (fixed by §1 item 4):

| probe | program | node | kali `run` | `check` |
|---|---|---|---|---|
| `samefn` | `function a(){ function h(){ return 1; } return h(); } function b(){ function h(){ return 2; } return h(); } console.log(a(), b());` | `1 2` | E4201 invalid wasm | 0 |
| `blockfn` | `{ function h(){ return 1; } console.log(h()); } { function h(){ return 2; } console.log(h()); }` | `1 2` | E4201 invalid wasm | 0 |

Refused for unrelated reasons, out of scope (§1.1): `catch_shadow`
(try/catch), `switch_case` (fallthrough), `str_shadow` (string `+`),
`sibling_types` (non-ASCII-provable `.length`), `loop_closure` (literal-array
`.push`). Correct at the baseline: `tdz_var` (`var` hoisted out of a block).

### 2.2 The silent surface: per-iteration bindings

| probe | program | node | kali `run` | `check` |
|---|---|---|---|---|
| `defer1` | `for(let i=0;i<3;i++){ queueMicrotask(()=>console.log(i)); }` | `0 1 2` | `0 0 0`, exit 0 | 0 |
| `defer2` | `function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();` | `0 1 2` | `3 3 3`, exit 0 | 0 |
| `defer5` | `function m(){ for(let i=0;i<2;i++){ const k=i*3; queueMicrotask(()=>console.log(k)); } } m();` | `0 3` | `3 3`, exit 0 | 0 |
| `defer4` | `function m(){ for(const x of [5,6]){ queueMicrotask(()=>console.log(x)); } } m();` | `5 6` | E5506 (captured local without closure lowering) | 0 |
| `loopmix` | `function m(){ let a=10; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a+i),0); } } m();` | `10 11` | `12 12`, exit 0 | 0 |
| `asyncloop` | `async function m(){ for(let i=0;i<2;i++){ await null; setTimeout(()=>console.log(i),0); } } m();` | `0 1` | `2 2`, exit 0 | 0 |
| `depth2` | `function m(){ let a=1; function mid(){ let b=2; queueMicrotask(()=>console.log(a+b)); } mid(); } m();` | `3` | E5506 (depth-2 capture), exit 1 | 0 |
| `loopc3` (control) | `function m(){ let s=0; for(let i=0;i<3;i++){ const g=()=>i; s+=g(); } return s; } console.log(m());` | `3` | `3`, exit 0 | 0 |

`defer4` should become node-correct (§1 item 5). `loopc1`, `loopc2`, `loopc4`
and `loopc5` (a closure stored in a variable and called after the loop) are
refused at the baseline by the first-class-call refusal (`calling 'f0' is
unavailable …`) and stay refused; they become probes so a movement shows.
`defer3` (`Promise.resolve().then(…)` in a loop) is refused by the
unresolved-callee refusal and stays refused.

### 2.3 Where the binding is lost

The resolver models block scopes and gives each declaration a fresh id
(`kali_types/src/context.rs:214-228`, `:268`, `:299`), but
`analyze_source_file` (`kali_cli/src/build/compile.rs:768-773`) keeps only
its diagnostics and repr table. Every later stage is keyed by name:

* **codegen locals.** `FunctionEmitter::locals` and `bindings`
  (`kali_codegen/src/emitter.rs:345-346`) are flat per function;
  `collect_function_locals_from_node` (`lower.rs:6361`) dedupes by name. The
  declarator emit (`emit/control_flow.rs` about 1890-1925) does `LocalSet`
  into whatever slot holds the name, so `fn_param`'s inner `const c` writes
  the parameter.
* **module globals.** `collect_module_scalar_globals` (`lower.rs:4138`) and
  `module_const_inits` / `module_binding_names` (`lower.rs:1359-1392`) descend
  into blocks, so a top-level block's `let x` is the module's `x`.
* **functions.** `function_name_to_index` (`lower.rs:917-919`) is one map over
  the program, built by `collect_functions_from_node` (`lower.rs:2808`).
* **repr, monomorphize, class rewrite, MIR, optimizer.** `ReprTable` keys on
  `(function, name)` (`kali_common/src/repr.rs:83`); monomorphize's env is
  flat per function (`monomorphize.rs:462-590`); the class rewrite's
  `Scopes` is per frame (`class_instances/scopes.rs`); MIR's
  `ScopeState.binding_index` is per function (`kali_mir/src/analysis/mod.rs:147`);
  the optimizer's constant envs are name-keyed (`object_fold.rs:447`,
  `specialize.rs:53-115`).
* **per-iteration.** An env record is allocated once per activation of an
  env-owning function (`kali_codegen/src/closure.rs:1-20`), so every
  iteration's closures share one cell.

Fixing each stage separately is approach B (§4). Renaming once, before all of
them, is approach A.

---

## 3. Design

### 3.1 The rename pass: placement

A new AST pass, `crates/kali_cli/src/build/block_scope_rename.rs`, called from
`analyze_source_file` after `link_provable_module_namespaces`
(`compile.rs:730`) and before `monomorphize_statements` (`compile.rs:747`).
Every later stage (monomorphize, `name_anonymous_functions`, the resolver,
`repr_infer`, the class rewrite and its re-inference, HIR, MIR, the optimizer,
codegen) sees the renamed program. `kali check` calls the same
`analyze_source_file`, so `check` and `run` see the same names.

The walk enumerates every `Statement`, `Expression`, pattern and JSX variant
with no `_ =>` arm, using `module_link.rs`'s `deny_import_positions_*` as the
checklist, as `name_anon_functions.rs` does (its module doc states the
discipline: a missed field is a silent miscompile, so the walk is checked by
inspection and test as well as by the compiler).

### 3.2 The rename pass: the rule

**Scopes.** The walk keeps a scope chain matching the resolver's
(`kali_types/src/resolve/mod.rs`):

* a module scope; a function scope holding the parameters and every `var`
  and function-level declaration (`var` hoists to it, as
  `variable_binding_scope`, `context.rs:414`, does);
* a block scope for a bare block, each `if` arm, each loop body, each
  `for` / `for-in` / `for-of` head, `catch` (holding its param), and one per
  `switch` (all cases share it, as the resolver's `resolve/mod.rs:858` does);
* function and class declarations bind in the block that contains them and
  are hoisted within it.

Each identifier reference is resolved through the chain and rewritten to its
binding's final name. A reference that resolves to nothing is left as
written (a free global, or an undefined identifier for the resolver to
report).

**Which bindings are renamed.**

* **Variables** (`let`, `const`, `var`, parameters, `catch` params, and names
  bound by destructuring). Consider one frame (a function body, or the
  module body) together with its enclosing frames. Group their declarations
  by spelling. If a spelling has more than one declaration in that set, the
  **keeper** is the one in the outermost scope (a module-level binding over
  a function-level one, a function-level one (including a parameter or a
  hoisted `var`) over a block one); among siblings at the same depth, the
  first in source order. Every other declaration of that spelling is
  renamed. Because enclosing frames are in the set, a nested function's
  parameter or local that shares a spelling with a module or enclosing
  function binding is renamed too (`const n=1; function f(n){…}` renames
  `f`'s `n`). That is deliberate: a capture or module-global lookup by name
  can then never reach the wrong binding. Sibling frames (two separate
  functions each declaring `i`) are not grouped and are not renamed.
* **Function and class declarations.** Codegen's function map and the class
  rewrite's generated names (`C__new`, `C__m`) are program-wide, so the
  group is the whole program: a function or class spelling declared more
  than once anywhere is renamed everywhere except one keeper (the top-level
  declaration if there is one, otherwise the first in source order).
* **Never renamed:** a module-top-level binding (so exports, imports, module
  globals and library-mode artifacts are unchanged); two declarations in the
  **same** scope (left for the resolver to report E3101,
  `context.rs:383`, `:407`); property keys, member names and labels.

The rule is a function of the AST alone, so the pass is deterministic.
Numbering is a single per-program counter in a pre-order walk.

**Name shape.** A renamed binding is spelled `<name>{b<N>}`. `{` and `}` are
not identifier characters in the lexer, so the name cannot collide with
anything the program spells: not R-30r's `Scopes::spelled()` set, not
`__kali_fn_N`, not `__link…`, and not monomorphize's `<name>${<idx>}`
(`monomorphize.rs:162`), whose shape differs. A renamed function that is then
monomorphized becomes `<name>{b<N>}${<idx>}`.

**Shapes that need care.**

* Shorthand property `{x}` where `x` is renamed becomes `{x: x{b1}}`; the key
  stays `x`.
* A renamed name in a destructuring pattern, `const {x} = o`, becomes
  `const {x: x{b1}} = o`; an array pattern element is renamed in place.
* `export { x }` names a module-top-level binding, which is never renamed.

### 3.3 Per-iteration env records

**When a loop owns an env.** In MIR, a loop (any `for`, `for-in`, `for-of`,
`while` or `do`) is an **iteration owner** when both hold:

1. a `let` or `const` declared in its head or body is captured by a closure;
2. that closure escapes the iteration by `escape_flow`'s verdict
   (`kali_mir/src/analysis/escape_flow.rs`): registered as a deferred
   callback, stored, or returned. A closure `escape_flow` cannot classify
   counts as escaping. (Per-iteration records are always correct; the cost
   of a false "escapes" is only that a capture of a function binding becomes
   depth 2 and refuses.)

A loop whose closures are all only called synchronously within their own
iteration (`loopc3`) is not an owner: one shared cell and per-iteration cells
are indistinguishable there, so it keeps the baseline lowering and nothing is
lost.

**Plans.** An iteration owner gets a synthetic scope label `<fn>{iter<N>}` in
`MirProgram::parent_labels` (`analysis/scope.rs::push_scope`), between its
enclosing function's label and the labels of the closures inside it.
`derive_env_plans` (`env_plan.rs`) then assigns the loop-declared captured
bindings to that label as cells, exactly as it does for a function. Depth
counting (`env_owning_hops`) and owner-keyed promotion
(`closure::cell_is_promotable`) are unchanged.

**Codegen.** For an iteration owner:

* At the start of each iteration: save `g8` (`CURRENT_ENV_GLOBAL`) into a
  reserved local (named as `env_save_local_name` names its own, with an
  unrepresentable suffix), allocate a record whose parent is the saved `g8`,
  and set `g8`.
* On every exit from the iteration (falling off the body, `continue`,
  `break`, a labelled jump, `return`): restore `g8`.
* For `for (let …; test; update)`: after the body, allocate the next
  iteration's record and copy the head bindings' current values into it
  before `update` runs. The closures of earlier iterations keep their
  records.
* At top level the loop record's parent is `0`. Captured module bindings
  stay module globals (the module root never owns an env, `env_plan.rs`);
  only the loop's own bindings move into the record. This is the `defer1`
  fix.

### 3.4 Refusals

Every refusal is `E5506` with a message naming the binding as written.

| shape | raised where | `run` | `check` |
|---|---|---|---|
| `--compat eval` is on and the rename pass renamed a binding (direct `eval` could name the original spelling) | rename pass | refuses | refuses |
| an iteration owner whose body contains `await` or `yield` | rename pass, conservatively: a loop with a head- or body-declared `let` / `const` captured by any closure in the loop and an `await` / `yield` in the body | refuses | refuses |
| a closure in an iteration owner that also captures a binding of the enclosing function (env depth 2) | the baseline's depth-2 refusal (`env_safety.rs`, `closure_access.rs` `env_walk_depth_for`) | refuses | **exits 0** (as at the baseline) |

The `await` / `yield` refusal is decided in the AST because `check` never runs
MIR. Its AST predicate is wider than "iteration owner" (it does not know
whether the closure escapes), so it can refuse a loop MIR would not have made
an owner. Each such refusal found by the sweep (§5.4) is classified; a
class-1 capability loss stops the project for a ruling.

The depth-2 row is a `check` / `run` gap. It is recorded in the followups
file, not claimed as mirrored.

### 3.5 Display names

A single `kali_common::display_name(&str) -> Cow<str>` strips a trailing
`{b<N>}` (and leaves every other name alone). It is applied:

* to every diagnostic message, once, at the boundary in `compile.rs` where
  `analyze_source_file` and `compile_source_file_uncached` return
  diagnostics. A golden diagnostic whose text changes because of the rename
  is a mapping bug, not a re-pin (§5.4).
* wherever codegen emits a function or class name as a runtime string (a
  `.name` read, function or class rendering such as `[Function: h]` /
  `[class C]`). The plan's first task lists those sites by search; each gets
  a case.

### 3.6 Docs

* `specs/15-errors.md`: the two new E5506 texts of §3.4 (the depth-2 text
  exists). No new codes.
* `specs/19-feature-maturity.md`: a new row, worded as narrowly as §1 and
  §1.1: what is claimed (§1 items 1 to 6) and what is not (depth-2 captures,
  `await` / `yield` in an env-owning loop, `eval`, the never-freed
  per-iteration records). Row 203 (mutable locals) is unchanged.
* `specs/18-schemas.md`, `specs/12-cli.md`, `README.md`: no change (no
  command, flag, schema field or usage moves). The plan's last task re-reads
  them to confirm.
* `docs/superpowers/followups/kali-silent-miscompile-register.md`: R-10 marked
  fixed with the commit and cases; the per-iteration defect (§2.2) filed as a
  new entry, recorded fixed in the same change, so the register keeps that it
  was silent. `blast-radius-ranking.md` is a measured snapshot and is not
  edited.
* "Fixed by" notes, in the established style, on
  `class-instances-discovered-defects.md` §6.11 and §6.15,
  `literal-array-mutators-discovered-defects.md` §14 (third bullet) and
  `unresolved-member-call-discovered-defects.md` §6.9.
* A new `docs/superpowers/followups/block-scoping-discovered-defects.md`,
  holding the triage table (§5.4), the depth-2 `check` / `run` gap, the
  never-freed records, the out-of-scope `catch` / fallthrough shapes, and
  anything else measured and not fixed.
* `proofs/BOUNDARY.md`: no change.

---

## 4. Approaches not taken

* **B. Thread the resolver's binding ids through HIR, MIR, LIR and codegen.**
  The principled fix, but over twenty passes look names up as strings (§2.3),
  and monomorphize and `repr_infer` run on the AST before the resolver's ids
  exist, so some would stay name-keyed. It touches nearly every stage for the
  same observable result.
* **C. Rename at HIR lowering, from the resolver's scopes.** Too late:
  monomorphize, `repr_infer` and the class rewrite have already run over the
  merged names.
* **Per-iteration by AST desugaring** (Babel's loop-body function). It reuses
  per-activation records, but needs `break` / `continue` / `return` / label
  translation through a function boundary and puts every outer capture at
  depth 2, which refuses. Rejected for the MIR label of §3.3, which has the
  same depth-2 limit but needs no control-flow translation.

---

## 5. Testing and measurement

### 5.1 Probes

Task 0 commits every program of §2 as
`tools/array-return-probes/probes/bs_<name>.js` and records
`tools/array-return-probes/baseline-bs.tsv` with the baseline binary, using
`tools/array-return-probes/run.sh`. The final diff of that table against HEAD
is the project's headline evidence.

### 5.2 Cases

New `.toml` case files under `crates/kali_cli/tests/cases/scope/`, each with
node's output in its rationale:

* `block_shadowing.toml`: every §2.1 silent row and `sibling`, `samefn`,
  `blockfn`, node-correct under `run`, `check` exit 0; a `ts` variant of the
  main shapes; diagnostics naming a renamed binding show the written name
  (an E3101 in a block, an E5506 on a shadowed binding).
* `per_iteration.toml`: `defer1`, `defer2`, `defer4`, `defer5`, the
  `for-of` and `while`-body variants, at top level and in functions,
  node-correct; `loopc3` as the control that must not move; `loopmix`
  refused under `run` with `check` exit 0 (pinning the gap); `asyncloop`
  refused under `run` and `check`; a `for (let …)` whose closure writes the
  loop variable (the copy-before-update order).
* `eval_refusal.toml`: the `--compat eval` refusal, and its control (a
  program that shadows nothing under `--compat eval` is unchanged).

Re-pins: `oracle/tier2.toml` `r10_block_scope_shadowing_module_scope` and
`r10_block_scope_shadowing_in_function` flip from `verdict = "silent"` to
correct. `oracle/classifier_ground_truth.toml` uses R-10 as its only SILENT
specimen (rationale at lines 194-198); the replacement is chosen from the
register's entries still measured silent at HEAD and brought to the human
partner before it is pinned.

### 5.3 Unit tests

In sibling `*_tests.rs` files:

* `block_scope_rename_tests.rs`: the keeper rule (outermost, `var` hoisting,
  sibling order, parameters); `for` heads, `catch`, `switch`; function and
  class names unique program-wide; shorthand and destructuring expansion;
  same-scope duplicates untouched; module-top-level and exported names
  untouched; idempotence; a program that shadows nothing comes back
  unchanged (and the pass allocates no rename table); an exhaustiveness
  checklist test in the style of `name_anon_functions`.
* `display_name` tests in `kali_common`.
* `kali_mir` `env_plan_tests.rs`: an iteration owner exists only for an
  escaping capture; the loop label's depth from a closure; a capture of a
  function binding from inside an owner lands at depth 2.
* Determinism: a case builds the same program twice and compares the wasm
  bytes. The incremental cache's compiler identity
  (`docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md`)
  is checked to change with this pass, so stale artifacts cannot be reused.

### 5.4 Blast radius and capability loss

After the change, `cargo test --workspace --no-fail-fast` and
`cargo test -p kali_cli --test cases` (with `-- --ignored` as well). Every
trial whose outcome moved goes in a triage table in the followups file,
classed as one of:

* **wanted**: a silent wrong value became correct, or became a refusal;
* **capability loss**, in one of three classes: (1) a program node runs
  correctly that kali ran correctly and now refuses; (2) the same, where the
  refused code is dead; (3) a program whose output never depended on the
  changed binding;
* **rationale only** / **stderr only**: the verdict is unchanged and only
  text moved.

**Stop rule:** more than 50 moved trials, or any class-1 capability loss,
stops implementation for a ruling from the human partner.

The sweep re-runs, with the baseline binary, every `cli` and `oracle` step
whose HEAD stderr carries one of §3.4's messages or the depth-2 refusal, so
each new refusal of a previously exit-0 program is classified (as the
class-instances project's §5 did).

Known movers: the two `r10_*` oracle cases; the classifier ground-truth
fixture; any case whose program shadows a binding and pinned the wrong
output, which the sweep finds.

---

## 6. Amendments

Found while planning, at the baseline `6345f082b`. Each overrides the section
it names.

* **A-1. The AST has no patterns and `catch` is unreachable (§3.2).** Every
  binding in `kali_ast` is a plain `String`: declarators, parameters
  (`Vec<String>` / `Vec<FunctionParam { name }>`), `catch` params and import
  locals. The parser refuses destructuring, default and rest parameters
  (`kali_parser/src/declaration.rs:82-104`, `statement.rs:139-170`), and it
  refuses `try` (`statement.rs:640-655`). So §3.2's destructuring rule has
  nothing to act on. Shorthand `{x}` is parsed as
  `{ key: PropertyName::Identifier("x"), value: Expression::Identifier("x") }`
  (`kali_parser/src/expression/object.rs:35-40`), so renaming the value alone
  gives `{x: x{b1}}`. The walk still handles `catch` params and `switch`
  scopes.
* **A-2. "Escapes the iteration" means "a deferred registration is in the
  loop" (§3.3).** `escape_flow` gives no verdict per closure
  (`kali_mir/src/analysis/escape_flow.rs`: a may-heap fixpoint over
  bindings, params and returns only). The only way a closure made in an
  iteration can run after that iteration is a deferred registration, because
  a stored or returned closure called later is refused at the baseline by the
  first-class-call refusal (`loopc1`, `loopc2`, `loopc4`, `loopc5`). A loop
  is therefore an iteration owner when (a) a `let` / `const` declared
  directly in it (not in a nested loop or function) is captured by a closure
  created in it, and (b) the loop contains a call to a deferred-registration
  callee (`queueMicrotask`, `setTimeout`, `setInterval`, or a member
  `addEventListener`). That callee list moves from
  `kali_codegen/src/env_safety.rs:158-179` into one
  `kali_common::is_deferred_registration_callee`, which both MIR and
  `env_safety` call. `Kali.test` is not on the list, and a loop registering
  only `Kali.test` callbacks keeps the baseline lowering.
* **A-3. No `await` / `yield` refusal (§1.1, §3.4).** `await` is lowered as a
  synchronous pass-through (`kali_codegen/src/emit/control_flow.rs:2341`), so
  nothing suspends inside a loop and `g8` cannot be observed mid-iteration.
  Generators are refused at the baseline (`kali_codegen/src/lower.rs:31`).
  The refusal is dropped, and `asyncloop` is claimed node-correct (`0 1`).
  With it goes the only AST-side refusal besides `eval`.
* **A-4. A capture of depth 2 or more through an iteration record gets its
  own refusal (§1.1, §3.4).** At the baseline, a capture of depth 2 or more is
  not always refused. `env_walk_depth_for` (`emit/closure_access.rs:172`)
  returns `None` and the access falls back to the older behaviour, which is
  refused only on the deferred path (`intrinsics/host.rs:1733`). In an
  iteration owner, a closure called synchronously that captures a binding of
  the enclosing function would reach that fallback. So `derive_env_plans`
  marks every `CapturedRef` whose hop path crosses an iteration record, and
  codegen refuses any such reference of depth 2 or more with E5506
  (`a closure in a loop that captures `a` through a per-iteration record is
  unavailable …`). It is run-only, like every MIR-derived refusal. The same
  rule refuses **nested owner loops** whose closure reads the outer loop's
  binding (`nested_loops`: node `0 0` / `0 1` / `1 0` / `1 1`). That binding
  is two records away, through the inner one, so §1 item 5 does not claim
  this shape. Lowering walks of depth 2 or more is a future item.
* **A-5. Two more run-only refusals (§3.4).**
  - **An iteration-owner `for` whose body contains a `continue`** (not one in a
    nested loop or function). The copy into the next iteration's record sits
    at the end of the body, where the baseline's update also sits. `continue`
    branches to the loop top past both (register R-09 for the update), so the
    next iteration would reuse the record its closures share.
  - **An iteration-owner `for…of` lowered by compile-time unrolling.** That is
    every `for…of` except the growable-array runtime loop
    (`intrinsics/array.rs:~1405-1505`). The unrolled loop variable is
    substituted, not stored, so it has no cell to copy.
  
  `defer4` (`for (const x of [5,6])`) therefore stays refused, with the new
  message, and is not claimed. The growable-array runtime `for…of`,
  `for…in`, `while`, `do` and `for` are claimed.
* **A-6. Owner loops are matched by cell name, with a backstop (§3.3).** MIR
  and LIR share no loop id, and the existing pre-order loop ordinal excludes
  `for…in` (`kali_mir/src/analysis/walk.rs:324`). After the rename, the
  `let` / `const` names declared directly in a loop are unique within their
  function, so codegen finds an iteration plan's loop as the loop that
  directly declares that plan's cells. Both sides walk declarations "directly
  in this loop": MIR from HIR `VarDecl`, LIR from `Instruction` nodes whose
  text is `let` / `const` (`kali_codegen/src/lower.rs:6378`). They are kept in
  step by a backstop: an iteration plan that codegen never emitted refuses
  the program with E5506. Silently dropping it is not an option.
* **A-7. Nothing at run time shows a binding's name, and the cache needs no
  work (§3.5, §5.3).** No code path turns a function or class name into a
  runtime string. There is no `.name`, no `[Function: f]` and no `[class C]`;
  logging an instance is refused. `display_name` therefore applies to
  diagnostics only (`message`, `suggestion`, `notes`). A renamed nested
  function's wasm export name keeps the `{b<N>}` spelling. The host never
  looks a function up by its source name, and the spelling cannot collide.
  The incremental cache's compiler identity is a fingerprint of the
  executable (`kali_cli/src/build/fingerprint.rs:28`), so any rebuild changes
  it. Determinism is pinned by the pass's idempotence unit test and by the
  existing
  `runtime_smoke::build::build_artifacts_are_deterministic_across_repeated_invocations`,
  whose fixture gains a shadowed binding.
