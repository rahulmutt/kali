# A `const`-bound or immediately-invoked anonymous function returns a real array, or refuses

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `068b29950` (`main`, merge of PR #45 `array-return`) |
| branch | `anon-array-return` |
| toolchain | `rustc 1.99.0 (b940084d7 2026-09-28)` |
| kali binary | `kali 0.1.0`, `target/debug/kali`, `cargo build -p kali_cli` at the baseline |
| oracle | `node v26.10.0` |
| measured on | 2026-10-02 |
| item picked | `docs/superpowers/followups/array-return-discovered-defects.md` §1, "An arrow or function-expression array return, passed to an array parameter, reads `0`" |
| defects this closes | that §1, plus the nested-`const`, `const`-alias-chain and IIFE rows §2.1 adds |

**Scope was chosen by the human partner:** real values wherever codegen
already resolves the callee statically to an anonymous function body, and a
refusal (`E5506`) for every other directly-called anonymous array return.
The two other options were "admit everything callable" and "fail closed only".
The approach (resolve `const` function aliases when inference records its facts) was
chosen over rewriting call sites in the `name_anon_functions` pre-pass and over
registering `const f = () => …` as a declaration named `f` (§4).

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** an arrow or function expression that is (a) bound by a `const`
declarator, directly or through a chain of `const` identifier aliases, in the
same function as the call, or (b) called immediately (an IIFE), and whose every
`return` yields an array of `I64` elements, hands its caller a real runtime
array. The caller can read it bound or directly, or pass it on, exactly as
for a `function` declaration under the array-return project
(`2026-10-02-array-return-design.md` §1). Such a function whose returns are
array-shaped but not admitted refuses with `E5506` at check time, as a
declaration does.

### 1.1 What this project does NOT claim

* It does **not** resolve a `let`/`var`-bound function value, a module-scope
  `const` function called from inside another function, an object method, or
  any call through a first-class function value. Every one of those refuses
  today (`E5506` "call through a first-class function value", or `E3100`) and
  keeps refusing.
* It does **not** change callbacks. An anonymous function that is never
  directly called (`xs.map(x => [x])`, `setTimeout(cb)`) produces no fact, is
  never tainted, and keeps its pre-project lane.
* It does **not** reopen backstop 2 (`array-return-discovered-defects.md` §8):
  an unadmitted, uncalled anonymous body that returns a literal still emits the
  placeholder `0`, with no guard.
* It does **not** fix R-68's open lanes (§2 of the followups), R-21 (an
  out-of-bounds read prints `0`), R-48, or the runtime-array defects of §3 of
  the followups. Returned arrays inherit all of those, exactly as declaration
  returns do.

---

## 2. What was measured

### 2.1 The silent surface at the baseline

`kali check` exits 0 on every SILENT row.

| program | node | kali | verdict |
|---|---|---|---|
| `const f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));` | `2` | `0`, exit 0 | SILENT |
| `const f = function(){ return [1,2,3]; }; function g(x){return x[1];} console.log(g(f()));` | `2` | `0`, exit 0 | SILENT |
| `function g(x){return x[1];} console.log(g((() => [1,2,3])()));` | `2` | `0`, exit 0 | SILENT |
| `function main(){ const f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f())); } main();` | `2` | `0`, exit 0 | SILENT |
| `const f = () => [1,2,3]; const h = f; function g(x){return x[1];} console.log(g(h()));` | `2` | `0`, exit 0 | SILENT |
| `const f = () => { console.log("ran"); return [1,2]; }; function g(x){return x[1];} console.log(g(f()));` | `ran` `2` | `ran` `0`, exit 0 | SILENT (the body runs) |
| `const f = () => [1,2,3]; console.log(f()[0]);` | `1` | `E5506` indexed read, exit 1 | REFUSES (backstop 1) |
| `const f = () => [1,2,3]; console.log(f().length);` | `3` | `E5506` `.length`, exit 1 | REFUSES |
| `const f = () => [1,2,3]; const a = f(); console.log(a[2]);` | `3` | `E5506` indexed read, exit 1 | REFUSES |

### 2.2 Controls at the baseline

| program | node | kali | why it is a control |
|---|---|---|---|
| `const f = () => 7; console.log(f());` | `7` | `7` | codegen already resolves `f` to `__kali_fn_N` and runs it |
| `const f = (n) => new Array(n).fill(4); function g(x){return x[1];} console.log(g(f(3)));` | `4` | `4` | an allocation return already produces its handle |
| `const f = () => 7; function main(){ console.log(f()); } main();` | `7` | `E5506` first-class call | out of scope; must keep refusing |
| `let f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));` | `2` | `E5506` first-class call | out of scope; must keep refusing |
| probe `arrow_return` (`f` module-scope, called in `main`) | `1` | `E5506` | out of scope; must keep refusing |

### 2.3 Mechanism

Three facts combine:

1. **Codegen resolves the call.** A `const` declarator records
   `name → init node` in the per-function, flat `self.bindings`
   (`crates/kali_codegen/src/emit/control_flow.rs:1906-1911`), and
   `resolve_bound_node` (`emit/call.rs:6699`) follows it, so `f()` runs
   `__kali_fn_N`. A module-scope binding is not visible inside another
   function's emitter, which is why §2.2's third row refuses.
2. **Inference does not.** The array-return lane keys every fact on a bare
   callee name (`crates/kali_types/src/array_return.rs`, `ReturnArg::Call`,
   `ArgShape::Call`, `InitKind::Call`, `ArgArrayProof::Call`,
   `NumProof::Call`). Only a `function` declaration is a candidate form
   (`repr_infer.rs:3186-3205`). `Call("f")` matches nothing, and
   `__kali_fn_N` is never admitted.
3. **Nothing taints it.** `solve` exempts every `__kali_fn_*` that is not a
   candidate (`array_return.rs:630-638`, the plan's pre-decided narrowing),
   so the body keeps the literal placeholder `0`. Since `g`'s `x` is
   subscripted, it counts as a runtime array (followups §13), so `x[1]` reads
   through handle `0` and prints `0`.

HIR lowers a concise arrow body to a synthetic `return <expr>`
(`crates/kali_hir/src/lowering/function.rs:42-70`), so `emit_return` already
runs under `function_name == "__kali_fn_N"` for both arrow forms.

---

## 3. Design

### 3.1 Inference (`kali_types`)

**Alias table.** During the body walk, `repr_infer` records
`(func, name) → __kali_fn_N` for a declarator that is `const` and whose
initializer, after `array_return::unparen`, is either

* an `ArrowFunctionExpression` or `FunctionExpression` (key: its synthetic
  `id`), or
* a bare identifier already in the table **for the same `func`** (the
  `const h = f` chain).

A name declared more than once in `func`, by any declarator kind, a param, or
a nested `function` declaration, is removed from the table. Codegen's
`bindings` is flat, not block-scoped, so a re-declared name is ambiguous, and
inference declines to guess. There is **no** fall-through to module scope
(unlike `binding_scope`, `repr_infer.rs:1815`), because codegen's per-function
`bindings` has none.

**Resolution.** One helper, `resolve_callee(func, name) -> String`, returns
the table's `__kali_fn_N` or `name` unchanged. It is applied at every point
where a bare-identifier callee becomes an array-return fact: return
classification (`ReturnArg::Call`), feeds (`ArgShape::Call`), call-bound
declarators (`InitKind::Call`), argument array proofs
(`ArgArrayProof::Call`), number proofs (`NumProof::Call`), call edges, and
`ArrayReturnFacts::called`. `array_return.rs` stays pure. It sees resolved
keys and needs no knowledge of aliases.

**IIFE.** A call whose callee, after `unparen`, is itself an arrow or
function expression records that expression's `__kali_fn_N` as the callee in
the same places.

**Candidates.** A non-async, non-generator `__kali_fn_N` that is in `called`
joins `candidate_forms` with a declaration count of 1, and
`body_falls_off_end` applies to its body. A concise arrow body is a single
`return` and never falls off. An async arrow joins `non_taintable` (ruling
R12).

**`solve`.** The fixed point is unchanged. The narrowing at
`array_return.rs:630-638` becomes:

> an anonymous `__kali_fn_N` **not in `called`** is never tainted, which keeps
> callbacks on their lane. One that **is** in `called` is admitted or tainted
> exactly like a declaration (`MIXED`, `ELEMENT`, `LET_LITERAL`, `FORM`).

**Shadow fact.** `is_array_return_callee_shadowed(func, name)` (published via
`ReprTable`) declines when `name` is a param or a `let`/`var` binding of
`func`, so a `let f = () => …` never resolves, matching codegen's `locals`
belt.

**Published.** `array_return("__kali_fn_N")`,
`array_return_taint("__kali_fn_N")`, and `call_bound` / `array_fed_params`
computed from resolved facts. These are the same `ReprTable` keys a declaration
uses.

### 3.2 Codegen (`kali_codegen`)

**Callee side.** No change. `emit_return` (`emit/control_flow.rs:217`), the
scratch reservation (`lower.rs:3658`) and literal materialization key on
`self.function_name`, which is `__kali_fn_N` inside an anonymous body.

**Caller side.** `array_return_call_elem` (`emitter.rs:835`) gains one
resolution step:

1. Run the existing shadow decline on the **source** callee name
   (`is_array_return_callee_shadowed` plus the `locals` belt).
2. If the callee node resolves through `resolve_bound_node` and
   `unwrap_transparent_value_node` to a function-expression value whose text
   is `__kali_fn_N`, look up `array_return("__kali_fn_N")`. Otherwise look up
   the bare name as today.
3. An IIFE's callee node already is that value, so step 2 covers it.

Every consumer goes through this function, so the direct index, `.length`,
the console-argument guard and argument passing all read real memory with no
per-consumer edits. The call-bound lane reads `is_call_bound_array_binding`,
which inference now computes from resolved facts.

### 3.3 Refusals

* **At check time.** A directly-called anonymous function whose returns are
  array-shaped but not admitted becomes a shape conflict and `E5506`, through
  `kali_common::array_return_refused_message`. The message names the source
  binding (`f`), not `__kali_fn_N`. An IIFE has no source name, so its message
  says "an immediately-invoked function".
* **Callbacks** (not in `called`) produce no fact and no refusal.
* `emit_return`'s taint arm stays as the belt for any path that skips the
  shape-conflict check.

### 3.4 The agreement risk

Inference's alias table and codegen's `bindings` must resolve the same calls.
If inference declines a shape that codegen resolves, the old silent `0`
survives on that shape. This is the A1 discipline of the array-return
project: check, run and inference must not disagree. It is guarded by a
both-sides test (§5.3) and by the oracle rows (§5.2). Backstop 2 is
deliberately not used as a net (§1.1).

---

## 4. Approaches not taken

| approach | why not |
|---|---|
| Rewrite `f()` to `__kali_fn_N()` in the `name_anon_functions` pre-pass | A global AST rewrite. Every other lane (shadow guards, string-result taint, diagnostics that name `f`) would see a different program. The blast radius is far larger than the defect. |
| Register `const f = () => …` as a declaration named `f` | Codegen emits the body as `__kali_fn_N`, so `emit_return` would look up the wrong key, and `f` would collide with the shadow machinery that treats `const f` as a shadow of a `function f`. |
| Fail closed only (taint every called anonymous array return) | Chosen against by the human partner: the runtime representation and the callee-side materialization already exist, so the real-value half is mostly callee resolution. |

---

## 5. Testing and measurement

### 5.1 The capability-loss spike (gate)

Before the real-value half lands, apply only the narrowed exemption (§3.1
`solve`) with alias resolution feeding `called`. Then run `cargo test --workspace` and
`tools/array-return-probes/run.sh`, and list every test or probe that moves to
an `E5506` that did not refuse at the baseline. Each one is either admitted by
the full change or brought back to the human partner before proceeding. The
same diff is recomputed at the end of the branch and recorded in the
followups file (§6).

### 5.2 Probes and oracle cases

* New probes `tools/array-return-probes/probes/anon_*.js`: every §2.1 row,
  every §2.2 control, and `flatMap`/`map` callbacks returning `[x]` (their
  output must not change). `baseline.tsv` gains their baseline column,
  measured at `068b29950`.
* `crates/kali_cli/tests/cases/runtime/array_return.toml`: the §2.1 rows at
  node's output, including the alias chain, nested `const`, IIFE and
  side-effecting bodies, the bound (`const a = f()`) form, and `.length`.
* `crates/kali_cli/tests/cases/runtime/array_return_refusals.toml`: a called
  anonymous function with a mixed return, a boolean element, a `let`
  literal, and an async arrow, with `kali check` and `kali run` both
  refusing. The §2.2 out-of-scope controls, still refusing.
* Any `oracle/` case that pins a §2.1 row at `0` is re-pinned to node's value.

No new `tests/*.rs` integration target.

### 5.3 Unit tests (sibling `*_tests.rs` files)

* `array_return_tests.rs`: `solve` admits a called anonymous candidate, taints
  a called mixed one, and leaves an uncalled one untainted.
* `repr_infer` tests: the alias table covers a single alias, a chain, a
  re-declaration that drops, a `let` that never enters, no module-scope
  fall-through, and an IIFE callee.
* `control_flow_tests.rs` / emitter tests: `array_return_call_elem` resolves
  through `bindings` and declines on a shadowed source name.
* **Agreement test:** over the §5.2 probe sources, every call that codegen's
  `resolve_bound_node` resolves to an `__kali_fn_N` whose returns are
  array-shaped has a matching resolved inference fact.

### 5.4 Gates

`cargo test --workspace` passes, including
`kali_blast_radius::ranking::ranking_tests::spliced_document_matches_the_generator`.
If any ranked entry moves, the ranking is regenerated with
`cargo run -p kali_blast_radius --example rank`.

---

## 6. Bookkeeping

* `docs/superpowers/followups/array-return-discovered-defects.md` §1: marked
  FIXED with the closing commit. Its §9, §13 and §16 cross-references are
  re-checked.
* A new `docs/superpowers/followups/anon-array-return-discovered-defects.md`
  records what was measured and not fixed, at minimum the out-of-scope
  controls of §2.2 (they need first-class function values) and the §5.1
  capability-loss diff.
* `kali-silent-miscompile-register.md` is amended only if an entry's lane
  moves.
* No CLI, schema, diagnostic-code or maturity change. `E5506` and its
  array-return message family already exist, so the AGENTS.md §6 CLI change
  packet does not apply.
