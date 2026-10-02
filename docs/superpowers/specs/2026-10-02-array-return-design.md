# A returned array is a real array, or a refusal

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `368b5b5ea` (branch `array-return`: `main` at `00abb525c` plus the Rust `1.98.1` → `1.99.0` bump) |
| toolchain | `rustc 1.99.0 (b940084d7 2026-09-28)` |
| kali binary | `kali 0.1.0`, `target/debug/kali`, built fresh at the baseline (the container had no build cache) |
| oracle | `node v26.8.2` |
| measured on | 2026-10-02 |
| item picked | `docs/superpowers/followups/inline-allocation-value-position-discovered-defects.md` §1, "A returned or passed-through allocation, rebound, reads zeros — cross-reference R-14" |
| defects this closes | R-14 (both scopes, literal and allocation returns, bound and direct forms); the whole-array `console.log` handle print (§2.4) |

**Scope was chosen by the human partner:** R-14 alone (§1 of the item), not the
N1 family. R-48 (`inline-allocation-value-position-discovered-defects.md` §2) is
measured as a control only. A returned array becomes a **real value** for
allocations and integer literals, and a refusal everywhere else (option 1 of
three; the others were "allocations only, literals refuse" and "fail closed
only").

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** a `function` declaration (at any depth, A4) whose every `return` yields an array of
`I64` elements hands its caller a real runtime array. The caller can read it bound
(`const b = f(); b[i]`, `b.length`, `g(b)`) or directly (`f()[k]`, `f().length`).
Every other array-shaped return refuses with `E5506`, and so does every read that
today falls through to a placeholder `0`.

### 1.1 How it was chosen

1. The register's R-14 entry (`kali-silent-miscompile-register.md:2467`) carries
   an arena/escape-reclamation hypothesis at "low" mechanism confidence. Tracing
   the source refutes it (§2.1): no array is ever allocated, so there is nothing
   to reclaim.
2. The defect has two ends, and fixing either alone leaves the program wrong: the
   callee produces no handle, and the caller does not know it holds one.
3. The human partner chose real values over refusal for the two shapes the repo
   already has a runtime representation for: the `[len][elem…]` array the
   declarator lane allocates, and the handle a `new Array` binding already holds.

### 1.2 What this project does NOT claim

* It does **not** fix an array held in an object field (R-48, N1's other member).
  §4.2 pins R-48's two rows as controls at their current output.
* It does **not** give runtime arrays an `undefined`. An out-of-bounds read of a
  returned array prints `0` where node prints `undefined`, which is R-21's
  defect. It is pinned as a control, not fixed.
* It does **not** cover methods, arrow functions, function expressions or
  closures. An array return from any of those refuses (§3.1), and that refusal
  is the whole of the claim for them.
* It does **not** make `F64`, `String`, boolean or nested-array element arrays
  returnable. Those refuse.
* It does **not** make a local array literal a runtime array. `const a=[1,2,3];
  a[i]` keeps refusing as it does today; only a **returned** literal is
  materialized.
* No new diagnostic code, flag or schema. The CLI change packet
  (`specs/12/15/18/19`) is untouched: this project only routes more lanes to the
  existing `E5506`.

---

## 2. Measurements

### 2.1 The mechanism, traced from source

**Callee.** `emit_return` (`crates/kali_codegen/src/emit/control_flow.rs:177`)
has one materializing arm, and it is for `Repr::Object` returns of an object
literal. An array literal goes to `emit_node`, then to `emit_value`'s text-less
branch, and ends in `emit_aggregate_literal`
(`crates/kali_codegen/src/emit/literal.rs:13-45`). That function evaluates and
drops every element, then pushes `I64Const(0)`. The function returns `0`.

**Return repr.** `ReprTable::return_repr` defaults to `I64`
(`crates/kali_common/src/repr.rs:385`). `repr_infer` records `Repr::Object` for a
returned object literal (`crates/kali_types/src/repr_infer.rs:2732`) and nothing
for an array. `return <identifier>` is pushed onto `array_binding_returns`
(`:2738`), but that list feeds only two things: the I2 refusal of
String-element array returns (`:5645-5670`), and the exclusion of
array-returning functions from the numeric-return proof (`:6017`).

**Caller.** `const a = f()` adds only an object-flow edge from `f`'s return
(`:1947`). Codegen seeds `array_bindings` from **params only**
(`crates/kali_codegen/src/emitter.rs:657-662`), and the declarator lanes
(`control_flow.rs:1640-1720`) register a local only when its initializer is a
literal, a `new Array(n)` or a `.fill` call. So `a` is a plain `I64`. `a[0]`
misses `resolve_static_index_member` (which stops at the call node), misses
`dynamic_array_read_base` (`a` is not an array binding), and reaches the
numeric-index fallback in `emit_unary`
(`crates/kali_codegen/src/emit/operators.rs:563-571`). That fallback emits the
receiver, drops it, and pushes `I64Const(0)` with no diagnostic. **This is
where R-14's `0` comes from.**

**Arena.** `arena_gate` (`crates/kali_mir/src/analysis/arena_gate.rs:253-265`)
sets `has_returned_site` for every `return` whose value may be on the heap,
regardless of shape, which keeps the arena from opening. The source says a
returned allocation is already safe from loop-arena reclamation. §4.2 pins this
with a test rather than relying on that reading.

**Contrast, the object return.** `return {a:1}` gets `Repr::Object(shape)` and
a real allocation, and `const r = f(); r.a` reads it through
`emit_object_field_read`. This works because both ends have a lane. Arrays have
neither.

### 2.2 The silent rows, at the baseline

Each program was run with `kali run` against node. All exit 0 in kali.

| # | program | node | kali |
|---|---|---|---|
| S1 | `function f(){return [1,2,3];} console.log("r="+f()[0]);` | `r=1` | `r=0` |
| S2 | `function f(){return [1,2,3];} const a=f(); console.log(a[0]+","+a[2]);` | `1,3` | `0,0` |
| S3 | S2 with the caller inside `function main(){…} main();` | `1,3` | `0,0` |
| S4 | `function mk(){const a=new Array(3).fill(4); return a;} const b=mk(); console.log(b[1]);` | `4` | `0` |
| S5 | `function f(x){return x;} const a=f(new Array(3).fill(1)); console.log(a[0]);` | `1` | `0` |
| S6 | `function f(n){const a=new Array(n).fill(0); for(let i=0;i<n;i++)a[i]=i*i; return a;} …const a=f(4); console.log(a[3]);` (in `main`) | `9` | `0` |
| S7 | `function f(){const a=new Array(3); a[0]=7; return a;} …const b=f(); console.log(b[0]);` (in `main`) | `7` | `0` |
| S8 | `function f(){return new Array(3).fill(2);} …console.log(f()[0]);` (in `main`) | `2` | `0` |
| S9 | `function f(){return [1,2,3];} …const a=f(); console.log(a);` (in `main`) | `[ 1, 2, 3 ]` | `0` |
| S10 | `function main(){const a=new Array(3).fill(4); console.log(a);} main();` | `[ 4, 4, 4 ]` | `4104` |
| S11 | `function mk(){const a=new Array(2).fill("x"); return a;} function main(){const c=mk(); console.log(c[0]);} main();` (added by amendment A3) | `x` | `0` |

S10 involves no return at all. It is the pre-existing whole-array print defect of
a `new Array` binding, and it is listed here because this project makes the same
print reachable from a call-bound binding (§2.4).

### 2.3 Rows that already refuse or work, at the baseline

| program | kali today | why it matters |
|---|---|---|
| `function f(){const a=[]; a.push(1); return a;} …f()[0]` | `E5506` (growable array escaping via `return`) | the growable lane stays out of scope and keeps refusing |
| `function f(){return [1,2,3];} …const a=f(); let i=2; a[i]` | `E5506` (computed member) | becomes **correct** under this project |
| `function f(){const a=new Array(3).fill(4); return a;} …let b; b=f(); b[1]` | `E5506` (reassigning an array binding to a non-array) | assignment lane, kept refusing (§3.1) |
| `function f(){const a=new Array(2).fill(1.5); return a;} …b[0]` | `E4201` (invalid module) | §5 of the item's defects document. Becomes `E5506` here (F64 elements are not returnable) |
| `function f(){return {a:1};} const r=f(); r.a` | `1` | object-return control, must not move |
| `function main(){const a=new Array(3).fill(4); a[5]}` | `0` (node `undefined`) | R-21 control, must not move |
| `const o={arr:new Array(3).fill(6)}; o.arr[0]` (both scopes) | `0` (node `6`) | R-48 control, must not move |

### 2.4 Why the whole-array print joins this project

S9 prints `0` today. Once `a` is an array binding, it goes down the same lane as
S10 and prints the handle instead. Swapping one silent wrong value for another is
not a fix. Refusing a whole-array `console.log` of any runtime array binding with
`E5506` closes both, in one guard (§3.3).

---

## 3. The design

### 3.1 Inference: a return-array fact in `ReprTable` (`kali_types`, `kali_common`)

**Where it lives.** A new module, `crates/kali_types/src/array_return.rs`, with
tests in `array_return_tests.rs`, following `growable.rs`'s precedent of keeping
one lane's choke-point analysis outside the 6,862-line `repr_infer.rs`.
`repr_infer.rs` calls into it from `emit_table`, after array-element reprs are
solved, and writes the result into `ReprTable`.

**The table.** `ReprTable` gains `array_returns: BTreeMap<String, Repr>`
(function → element repr) and `array_return_tainted: BTreeMap<String,
&'static str>` (function → refusal reason), with `set_`/getter pairs in the
style of `set_array_binding`/`is_array_binding`. `return_repr` is **unchanged**
and stays `I64`, which is exactly what an array handle is at the Wasm level.
`Repr` gains no variant.

**Classifying a return argument.** Each `return` argument of a
`function` declaration (at any depth, A4) is classified as one of:

| class | shapes | element repr |
|---|---|---|
| *literal* | an array literal, empty or with only `I64`-solving elements; **or an identifier bound by `const` to such a literal** (amendment A2) | `I64` |
| *binding* | an identifier that is a **runtime** array of the function: a `new Array`/`.fill` local, an array param (`is_array_binding`), an array-fed param (amendment A3), or a call-bound array local (§3.1, call site) | that binding's solved `array_element` |
| *allocation* | `new Array(n)`, `new Array(n).fill(v)` | the allocation's solved element repr |
| *call* | a call to another array-returning function | that function's element repr |
| *bad-array* | an array literal with any non-`I64` element (float, string, boolean, nested array, object, spread, hole); an identifier bound by `let`/`var` to an array literal; a growable binding; any of the above whose element repr is not `I64` | — |
| *non-array* | anything else, including a bare `return;` | — |

The classifier is an **exhaustive match with no wildcard arm** over the
expression enum, as `growable.rs`'s scanner is. A new AST variant then forces a
decision, and the default is *non-array*.

**Classifying a function.** Over all of a function's returns, plus an implicit
*non-array* when control can fall off the end:

* **array-returning** with element repr `I64`: every return is *literal*,
  *binding*, *allocation* or *call*, and there is at least one.
* **array-tainted** with a reason: at least one return is array-shaped (any
  class but *non-array*), and the function is not array-returning. Reasons are
  `"it mixes array and non-array returns"`, `"an element is not an integer"`,
  `"it returns a growable array"`, and `"it returns a `let`/`var` binding of an
  array literal, which can be reassigned"`.
* otherwise, untouched. Every program with no array return is byte-identical.

Recursion and mutual recursion (`call` returns) are solved by a fixed point that
starts optimistic, with every candidate array-returning, and demotes until
stable. A cycle with no base case stays array-returning, which is sound: no
value ever comes back.

Methods, arrow functions, function expressions and closures are not candidates.
When one returns an array-shaped argument it is marked array-tainted with
reason `"only a `function` declaration with a unique name can return an array"`.

**The call site.** For `const b = f()` and `let b = f()`, where `f` is
array-returning, `set_array_binding(caller, b)` and `set_array_element(caller,
b, I64)`. This must happen **before** element-repr solving finishes, so that
`b` used as an array param of a further call (`g(b)`) or as a further `return b`
propagates. In practice the call-site facts and the function facts go in the
same fixed point. The existing I2 check and the numeric-return-proof exclusion
are left as they are; the I2 check now also covers a String-element binding in
the new lane, because that binding is *bad-array*.

Not added: assignment (`b = f()`), destructuring (`const [x] = f()`), and an
array-returning call stored into an object field or an array element. Each stays
on its existing lane: refuses today, or is R-48 territory.

### 3.2 Codegen: the callee (`kali_codegen`)

`emit_return` gets an array arm ahead of its object arm. The arm is taken when
`repr_table` names the current function:

* **array-returning, *literal* argument.** Allocate with
  `emit_array_allocation_static(len)` into a scratch slot, store each element at
  byte offset `8 + i*8`, and leave the handle as the return value. The
  allocate-and-store loop is **extracted** from the declarator lane
  (`control_flow.rs:1640-1683`) into one helper, `emit_static_array_materialize`,
  which both sites call. The declarator lane's object-element case
  (`emit_object_allocation` per child) moves into the helper unchanged, so the
  declarator's behaviour is byte-identical.
* **array-returning, any other argument.** Emit the argument as today. A
  *binding*, *allocation* or *call* already produces the handle.
* **array-tainted.** Every `return` whose argument is array-shaped refuses with
  `E5506`, and the message names the function and the taint reason. A
  *non-array* return in a tainted function emits as today, since the refusal on
  its array-shaped sibling already stops the build.

The arena-unwind and env-restore epilogue is shared with the existing arms,
unchanged.

### 3.3 Codegen: the caller

**Bound form.** At function entry, `array_bindings` is seeded from
`repr_table.is_array_binding` for **locals as well as params**
(`emitter.rs:657-662`), so the call-bound binding's membership comes from
inference and cannot drift from it. The declarator lane gets an arm for a call
initializer of an array-returning function: emit the call and `LocalSet` the
handle. From there, `b[k]`, `b[i]`, `b.length`, `b[i] = v`, `g(b)` and
`return b` all run through the existing array-binding lanes, unchanged.

**Direct form.** `f()[k]`, `f()[i]` and `f().length`, where `f` is
array-returning, evaluate the call **once** into a scratch slot this lane owns,
then read the element or the length from it. This follows the previous
project's scratch rule: one owner per slot, and the value is evaluated once.
These arms sit in the computed-member and `.length` lanes, next to the existing
array-binding arms they mirror.

**Backstops** (each one replaces a silent `0`):

1. The numeric-index fallback (`operators.rs:563-571`) refuses with `E5506`
   (`"no lane proves this receiver is an array"`) instead of emitting
   `I64Const(0)`.
2. `emit_aggregate_literal` (`literal.rs:13-45`) refuses with `E5506` when it is
   reached for an **array** literal in a position whose value is consumed. The
   object-literal branch is left alone. **Dropped as measured (§4.1, "Backstop 2,
   measured").** At full width it refused correct programs, this project's own
   lanes among them. Its narrowing needs the literal's parent node, and that
   site does not have it.
3. A whole-array `console.log` of a runtime array binding (single argument or
   one of several) refuses with `E5506`. That covers S9 and S10.

The spike (§4.1) measures how many existing cases each backstop moves before it
is committed. A backstop that turns a currently **correct** case into a refusal
is a capability loss. Such a backstop is narrowed to the call and return shapes
this project owns, and the remainder is filed in the discovered-defects
document. Each backstop's final width is recorded in that document, as measured.

### 3.4 Amendments found while planning (2026-10-02)

Reading the code for the implementation plan surfaced three things the design
above did not account for. Each was measured or read at `5e3d85bd2`.

**A1 — `kali check` has its own copy of codegen's runtime-array set.** The
resolver (`crates/kali_types/src/resolve/`) tracks
`scope.runtime_array_bindings` and answers `is_structural_runtime_array`
(`resolve/expression.rs:338`). It seeds array **params** from
`repr_table.is_array_binding` (`resolve/mod.rs:774`, `resolve/function.rs:43`)
and registers declarators through `declarator_registers_runtime_array`
(`resolve/expression.rs:746`). `kali check` refuses `a[i]` on anything that set
does not contain. Measured: `function f(){return [1,2,3];} function main(){const
a=f(); let i=2; console.log(a[i]);} main();` fails at `kali check` with that
`E5506`, not at codegen. If only codegen learned call-bound arrays, `check` would
refuse what `run` admits, the exact drift `resolve/expression.rs:761-765`
records killing a benchmark once. So:

* `declarator_registers_runtime_array`'s caller also registers a `const`/`let`
  declarator when `repr_table.is_array_binding(func, id)` holds and the
  initializer is a bare-identifier call to a function in
  `repr_table.array_return(callee)`. The function key comes from the existing
  `binding_repr_function_key`.
* The resolver's computed-member and `.length` admission for a **direct**
  `f()[i]` / `f().length` receiver accepts a bare-identifier call to an
  array-returning function. Each such site cites the codegen arm it mirrors.
* The **taint refusal is raised at check time**, through
  `ReprTable::add_shape_conflict`, the same channel the existing I2 check uses.
  `kali check` and `kali run` therefore refuse the same programs. The codegen
  return arm (§3.2) keeps a defensive `E5506` for a tainted function, which an
  admitted program never reaches.

**A2 — a `const` local bound to an array literal is fold-lane, not a runtime
array.** `repr_infer` gives `const a=[1,2,3]` an element node, so
`is_array_binding` is true. But codegen never allocates it: it folds reads at
compile time. `return a` would return `0`, exactly R-14's bug. Such an
identifier is therefore the *literal* class: codegen materializes it at the
return through `resolve_literal_aggregate`, which already follows a binding to
its literal, as the object arm does. This is sound only because a literal array
cannot be mutated: that already refuses with "mutating a literal array is
unavailable". A `let`/`var` literal binding is *bad-array*, since it can be
reassigned.

**A3 — S5's parameter is not an array binding today.** In `function f(x){return
x;}`, `x` is never subscripted, so it has no element node and
`is_array_binding(f, x)` is false. S5 needs a third inferred fact: an
**array-fed param**. That is a param of a top-level function with at least one
call edge, where **every** call edge passes an array-shaped argument at that
position:
* a runtime-array identifier, or an identifier that is itself array-fed;
* an allocation (`expression_is_array_allocation`'s shapes);
* a call to an array-returning function.

An array-fed param gets an element node unioned with each argument's, and so
becomes an ordinary array binding for everything downstream. Array-fed params,
array-returning functions and call-bound bindings are solved in **one**
optimistic fixed point, because each depends on the others. A param with any
non-array call edge, or none, is not array-fed and keeps its current lane.

**Measured alongside, and adding one row to §2.2:** returning a String-element
allocation is silent today even though the I2 check exists for it. `function
mk(){const a=new Array(2).fill("x"); return a;} function main(){const c=mk();
console.log(c[0]);} main();` passes `kali check` and prints `0` (node prints
`x`). Under this design it is *bad-array* (a String element) and refuses.
§4.2's refusals file pins it.

**A4 — nested `function` declarations are admitted.** The register's own
in-function R-14 repro (`crates/kali_cli/tests/cases/oracle/tier2.toml`,
`r14_function.js`) declares `f` *inside* `main`. Under "top-level only" it would
refuse rather than compute. `repr_infer` and codegen both key a declaration by
its own name, at any depth (`repr_infer.rs:2044`, `visit_stmt`'s
`FunctionDeclaration` arm), and `emit_return` runs the same epilogue for both.
So a `function` declaration at **any** depth is a candidate, provided its name
is declared **exactly once** in the program. `self.functions` is last-wins on a
repeated name, so a repeated name is not one function. Arrow functions,
function expressions, methods and class members stay excluded (§3.1). The
`nested_decl` probe pins it.

### 3.5 Data flow, end to end

```
return [1,2,3]   ──array_return.rs──▶  ReprTable.array_returns[f] = I64
     │                                              │
emit_return: array arm                     const b = f(): set_array_binding(main, b)
  emit_static_array_materialize                     │
  → handle on stack ──────── call ───────▶  emitter: array_bindings ∋ b (seeded)
                                                    │
                                            b[i] / b.length / g(b): existing lanes
```

---

## 4. Verification

### 4.1 The spike

Before any committed change, one throwaway patch (the §3 design, all three
backstops at full width) is applied to the working tree and measured two ways:

* **Probes:** node-oracle probe programs, at least the S1-S10 rows; the §2.3 rows;
  an allocation made inside a loop and returned from inside it; direct and
  mutual recursion; mixed returns; float, string, boolean and nested elements;
  an empty returned literal; a returned literal with computed (non-literal)
  integer elements; and `g(f())` (an array-returning call as an argument). Each
  probe is recorded as correct, refuses, or silent, at baseline and at spike.
* **Suite:** `bash scripts/test-gate.sh` (`cargo test --workspace
  --no-fail-fast`). Every newly failing test is classified: a correct case newly
  refusing (capability loss, which narrows a backstop), a pinned-zero case
  newly correct or refusing (expected, re-pin it), or something else
  (investigate).

The patch is then reverted with `git checkout` and is not committed.

The two remaining backstops (§3.3, 1 and 2) were then measured one at a time,
each at full width, by `bash scripts/test-gate.sh` and
`tools/array-return-probes/run.sh`. The oracle is node v26.10.0. "Correct
before" means the program printed node's output at the start commit `fefe62c60`.
A case pinned to a value that differs from node's is class (a) even if it was
passing.

#### Backstop 1, measured

The numeric-index fallback (`emit/operators.rs`, the floor of the numeric-index
arm) refuses with `E5506` (`"no lane proves this receiver is an array"`). It emits
the receiver first, so a refusal specific to the receiver wins. Measured alone, on
top of `fefe62c60`.

**27 failing tests: 27 (a), 0 (b), 0 (c).** Re-pinning them moved one more gate
(`every_zero_two_row_is_the_class_set_its_live_cases_assert`), because R-06's
r06c cases now assert `fail_closed`. R-06's §0.2 status cell follows its cases,
and the ranking's generated regions are re-spliced from the generator. With no
(b) rows there is no narrowing. **Final width: full**, committed as `d98f9dc49`.

| # | test (trials) | class | kali at `fefe62c60` | node v26.10.0 | now |
|---|---|---|---|---|---|
| 1-8 | `browser/promise_all_bundle::{promise_all_bundle__text, json_promise_all_bundle__json}` × ext js/ts/jsx/tsx | (a) | build OK; harness throws kali's own guard `Error: unexpected Promise.all results` (wasm `unreachable`), because every `X[0]`/`X[1]` read was a placeholder `0` | throws `TypeError: Promise.all called on non-object` (unbound `Object.freeze(Promise.all)(…)`) | build refuses, E5506 |
| 9-16 | `browser/promise_all_settled_bundle::{…__text, json_…__json}` × 4 ext | (a) | build OK; harness throws `Error: unexpected Promise.allSettled semantics` | throws `TypeError: Promise.allSettled called on non-object` | build refuses, E5506 |
| 17-20 | `runtime_smoke` `build::{,json_}build_emits_browser_bundle_promise_all_sequencing{,_in_js_input}` | (a) | build OK; harness traps (`values[0]`/`values[1]` read `0`, so the guard throws) | `promiseAllSmoke(1n, 2n)` returns `0n` | build refuses, E5506 |
| 21 | `object/computed_member_static_name::the_argv_index_lane_disagrees_with_itself_between_a_number_and_a_string_spelling` | (a) | `5 0 0` | `5 5 5` | refuses, E5506 |
| 22 | `object/property_key_identity::bigint_key_is_stored_under_zero_module_scope` | (a) | `false 0 1` | `false undefined 1` | refuses, E5506 |
| 23 | `object/property_key_identity::bigint_key_is_stored_under_zero_in_function` | (a) | `false 0 1` | `false undefined 1` | refuses, E5506 |
| 24 | `oracle/tier2::r06c_var_array_element_reads_module_scope` | (a) | `0 0` (silent) | `7 9` | `fail_closed` |
| 25 | `oracle/tier2::r06c_var_array_element_reads_in_function` | (a) | `0 0` (silent) | `7 9` | `fail_closed` |
| 26 | `runtime_argv::process_argv_huge_literal_index_never_flows_as_a_real_string` | (a) | `0` | `undefined` | refuses, E5506 |
| 27 | `runtime/array_return::async_function_stays_off_the_array_return_lane` | (a) | `0` | `undefined` | refuses, E5506 |

Its `_checks` twin still passes. `kali check` does not reach codegen, so it still
admits that async program, and check and run now disagree on it (a followup).
The new case `array_return_refusals.toml::a_literal_index_on_a_non_array_receiver_refuses`
pins the backstop. `function f() { return 3; } console.log("v=" + f()[0]);`
printed `v=0` at exit 0 at `fefe62c60`, where node prints `v=undefined`.

Probes: `r48_function_control` and `r48_module_control` move SILENT → REFUSES.
Both printed `0` where node prints `6`. No probe moves out of CORRECT.

#### Backstop 2, measured

`emit_aggregate_literal`'s array branch refuses with `E5506` when `want_value` is
set. It was measured at full width on top of backstop 1 (`d98f9dc49`).

**2501 failing tests.** The full list is not committed. To reproduce it, apply
the backstop 2 patch to `emit/literal.rs` at `d98f9dc49` and run
`bash scripts/test-gate.sh`. The patch: in `emit_aggregate_literal`'s array
(`else`) branch, when `want_value` is set, emit and drop each child, then return
`self.deny_e5506(...)`. By family:
`browser/` 1269, `misc/` 191, `array/` 146, `soundness/` 81, `oracle/` 79,
`runtime/` 68, `object/` 52, `string/` 7 (case trials), and 608 tests in
hand-written targets.

**(b) is not empty, and it includes this project's own lanes.** The rows below
were correct before: each printed node's output and now refuses with this
backstop's E5506.

| (b) row | evidence of "correct before" |
|---|---|
| `runtime/array_return::{allocation_returned_directly, bound_passed_on, computed_elements, dyn_index, empty_literal, if_else_both_return, mutate_call_bound, nested_decl, r14_register_in_function, s03_bound_in_main, s06_loop_filled, s07_store_then_return, s08_direct_fill}_computes`, `::const_literal_binding_return` (14) | each case pins node v26.10.0's exact output |
| probes `bound_passed_on` (3), `computed_elements` (5,10,6), `const_literal_return` (2), `dyn_index` (3), `empty_literal` (0), `if_else_both_return` (2), `mutate_call_bound` (9), `nested_decl` (1,3), `s03_bound_in_main` (1,3), `s06_loop_filled` (9), `s07_store_then_return` (7), `s08_direct_fill` (2) (12) | the probe runner's CORRECT verdict (stdout equal to node's, shown in parentheses) at backstop 1 |
| `nbody_runs_and_matches_canonical_output`, `spectral_norm_runs_and_matches_canonical_output`, `fannkuch_redux_runs_and_matches_canonical_output`, `mandelbrot_runs_and_matches_canonical_output`, `binary_trees_small_n_matches_canonical_output`, `binary_trees_canonical_n21_matches_output` (6) | the benchmark's canonical output |
| `acceptance_web_baseline_prefix_matches_node_byte_for_byte`, `acceptance_web_baseline_with_url_matches_node_byte_for_byte`, `delete_reinsert_enumeration_matches_node`, `quoted_and_numeric_like_keys_enumerate_in_es_order` (4) | node byte-for-byte or ES-order comparison |

One probe is (a): `r21_oob_control` moves SILENT → REFUSES (kali `0`, node
`undefined`). The remaining ~2480 rows were **not individually classified**.
Once (b) is non-empty, this backstop is narrowed or dropped whatever those rows
turn out to be.

**Narrowing is not available at this site, so backstop 2 is dropped entirely.**
§3.3's narrowing would refuse only when the literal's parent is a `Return`, a
`Call` argument or a declarator initializer. `emit_aggregate_literal` has no
parent to test: `LirNode` (`kali_lir/src/node.rs`) carries only
`kind/text/children/function_flavor`, and `FunctionEmitter` keeps no emit stack.
The backstop was reverted and is not committed. **Final width: none.** The
array-literal placeholder `0` stays as it was. It is to be filed in
`array-return-discovered-defects.md` (§5, Task 10) with this table, as a silent
lane with no backstop.

### 4.2 Tests that land

* **`crates/kali_cli/tests/cases/runtime/array_return.toml`.** The working
  lanes, each with node's exact output: S1-S8; the dynamic index and `.length`
  rows; a returned array passed on to a further function; a call-bound array
  mutated through `b[i] = v` and read back; recursion; the allocation-in-a-loop
  return (the arena pin); an empty literal; computed elements. Every rationale
  records kali's baseline output.
* **`crates/kali_cli/tests/cases/runtime/array_return_refusals.toml`.** The
  `E5506` lanes: each taint reason, each backstop, S9 and S10, and the
  non-top-level function forms.
* **Controls**, in the same files, each pinned at its current output and
  labelled as a control: the object return, R-21's out-of-bounds read, R-48's
  two rows, and the growable-escape refusal.
* **Unit tests** in `crates/kali_types/src/array_return_tests.rs`: each return
  class; each function class and taint reason; the recursion fixed point; the
  call-site binding facts; and that a program with no array return produces an
  empty table.
* **Oracle:** R-14 rows in `crates/kali_cli/tests/cases/oracle/tier2.toml`.

### 4.3 Done means

* S1-S8 print node's output in both scopes. S9, S10 and S11 refuse with `E5506`.
* `kali check` admits every program `kali run` runs correctly in
  `array_return.toml`, and refuses every taint-reason program `kali run`
  refuses (amendment A1).
* Every §2.3 row either keeps its baseline output or moves as that table says.
* `bash scripts/test-gate.sh` reports zero failures against the baseline
  measured at `368b5b5ea` under Rust 1.99.0: **12105 passed, 0 failed, 27
  ignored** (`cargo test --workspace --no-fail-fast`; the gate itself reported
  `GATE OK: 0 failing tests`). Every re-pinned case carries a
  rationale naming this project.
* `cargo clippy --workspace` is clean under 1.99.0.

---

## 5. Docs that move

* **`kali-silent-miscompile-register.md`.**
  * R-14 is marked CLOSED with the traced mechanism (§2.1), and its arena
    hypothesis is struck as refuted.
  * R-06-R1's "Real fix = R-14 escape stage" (`:1459`) and the
    `:6074` work-order row are corrected in place.
  * The whole-array `console.log` handle print is filed and retired as a new
    entry, as is any further silent lane the spike finds.
* **Blast-radius artifacts.** `tools/blast-radius/counts.json`, `clusters.json`
  and the ranking's generated regions are regenerated through `count.mjs` and
  `cargo run -p kali_blast_radius --example rank`, never hand-edited. Ranking
  churn is recorded in the ranking's commentary.
* **`inline-allocation-value-position-discovered-defects.md` §1** gets a
  resolution pointer to this spec.
* **New: `docs/superpowers/followups/array-return-discovered-defects.md`**,
  ranked by consequence: R-48's rows (re-measured), out-of-bounds reads (R-21),
  the excluded function forms, non-`I64` element arrays, assignment and
  destructuring from an array-returning call, and each backstop's narrowing, if
  any.

---

## 6. Risks

| risk | consequence | guard |
|---|---|---|
| A backstop turns a correct program into a refusal | capability loss, and the case files move | the spike counts it first; narrow and file |
| Inference and codegen disagree about which locals are arrays | a read on the wrong lane, possibly silent | codegen seeds from `repr_table` alone and has no recognizer of its own |
| The resolver and codegen disagree (`check` refuses what `run` admits, or the reverse) | a refused program that would have worked, or an admitted read on a `0` lane | the resolver registers from the same `repr_table` facts (amendment A1); the check-and-run pairs in §4.2 |
| A returned allocation lands in a loop arena | use after reclaim | the allocation-in-a-loop pin (§4.2) |
| The scratch slot for the direct form collides with an existing holder | a clobbered value | one owner per slot, as the previous project established; its unaudited §14 holders are not touched |
| The toolchain bump moves the baseline | a failure blamed on this project | the baseline is measured at `368b5b5ea` after the bump, before any change |
