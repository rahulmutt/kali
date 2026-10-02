# Defects the array-return project measured and did NOT fix

**Filed** 2026-10-02 by the **array-return** project
(`docs/superpowers/specs/2026-10-02-array-return-design.md`), on the
convention `inline-allocation-value-position-discovered-defects.md` and its
predecessors use: a project that measures more than it fixes writes down what
it left, so the silence is not read as absence.

**Oracle:** `node v26.10.0` (the plan named v26.8.2, which is not installed on
this machine; the probe programs use no version-sensitive behaviour).
**Measured at:** the branch head `d2202ed4b`, on `target/debug/kali` built
from that commit (`cargo build -p kali_cli`), except where a row names an
earlier commit. Probe rows come from `tools/array-return-probes/run.sh` at
that commit; the baseline column of every probe row is
`tools/array-return-probes/baseline.tsv`, measured at the spec's baseline
`368b5b5ea`. Rows that are not probes were run by hand as
`node P.js` against `kali run P.js` (and `kali check P.js` where a row says
so), with the program text given in full.

This project's two register movements — **R-14** retired (CLOSED, both scopes
FIXED) and **R-68** filed (a whole runtime array prints its handle; its
console-argument lane closed, three lanes open) — are recorded in
`kali-silent-miscompile-register.md` and are not repeated here, except where
§2 below gives R-68's open lanes their programs.

**Ranked, most consequential first.** §1-§6 are silent wrong values: kali exits
0 and prints something node does not. §1 leads because it is the ordinary
"build an array with an arrow, hand it on" idiom and lands squarely beside
this project's own new lanes. §2-§4 are pre-existing silent defects of the
runtime-array lane that this project made reachable from a call. §5 and §6 are
cross-references to existing register entries (R-48, R-21). §7 is a silent
host-visible value with no in-program reader. §8 is the backstop that was
measured and dropped, so a silent placeholder lane stays open with no guard.
§9-§14 fail closed (an honest `E5506` or another diagnostic), and are ranked
by how likely a reader is to hit them; several of them carry a `kali check` /
`kali run` disagreement. §15 records the backstop that did land, and §16
answers the brief's item about `callback_escape`.

| brief item (Task 10 Step 4) | section |
|---|---|
| 1. R-48's two rows | §5 |
| 2. R-21 out-of-bounds row | §6 |
| 3. ternary array returns | §9 |
| 4. arrow and function-expression returns | §1 |
| 5. non-`I64` element arrays | §11 |
| 6. assignment and destructuring | §10 |
| 7. backstop widths and (b) rows | §8, §15 |
| 8. `callback_escape` | §16 |
| 9. computed booleans in runtime arrays | §4 |

---

## §1. An arrow or function-expression array return, passed to an array parameter, reads `0`

| program | node | kali |
|---|---|---|
| `const f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));` | `2` | `0`, exit 0 |
| `const f = function(){ return [1,2,3]; }; function g(x){return x[1];} console.log(g(f()));` | `2` | `0`, exit 0 |
| `const f = () => [1,2,3]; console.log(f()[0]);` | `1` | exit 1, `E5506` "an indexed read is unavailable … no lane proves this receiver is an array" |
| `const f = function(){ return [1,2,3]; }; console.log(f()[0]);` | `1` | exit 1, same `E5506` |
| probe `arrow_return`: `const f = () => [1, 2, 3]; function main() { const a = f(); console.log(a[0]); } main();` | `1` | exit 1, `E5506` "calling 'f' is unavailable … call through a first-class function value" (REFUSES at baseline too) |
| `const o = { f() { return [1,2,3]; } }; console.log(o.f()[0]);` | `1` | exit 1, `E3100` undefined identifier `f` |

`kali check` exits 0 on the first four rows. Only `function` declarations are
candidates (spec §3.1, A4). By the plan's pre-decided narrowing (Task 4), an
anonymous function (`__kali_fn_*`) is neither admitted nor tainted: it keeps
its pre-project lane, which is the literal placeholder `0`. When the result is
read directly, backstop 1 (§15) refuses. When it is passed to `g`, whose `x` is
subscripted and so counts as a runtime array (spec-sanctioned, see §13), the
callee reads index 1 of handle `0` and prints `0`. Task 6 measured row 1 as `0`
and backstop 1 did not change it.

**What it would cost:** either admit anonymous functions as candidates (the
classifier already handles their return statements; the work is codegen's
`emit_return` arm for the anonymous lowering, and the call-through-value lane
for the `arrow_return` shape), or taint them and refuse at the argument. The
second option is cheap, but it may refuse programs that work today, so it needs
the spike's capability-loss count first.

## §2. A whole runtime array still prints its handle through a template literal, a concat, or an alias (R-68's open lanes)

| program | node | kali |
|---|---|---|
| `` const a = new Array(3).fill(4); console.log(`x${a}`); `` | `x4,4,4` | `x4112`, exit 0 |
| `const a = new Array(3).fill(4); console.log("v=" + a);` | `v=4,4,4` | `v=4112`, exit 0 |
| `const a = new Array(3).fill(4); const b = a; console.log(b);` | `[ 4, 4, 4 ]` | `4104`, exit 0 |
| `` function f(){return [1,2,3];} const a=f(); console.log(`x${a}`); `` | `x1,2,3` | `x4112`, exit 0 |
| `function f(){return [1,2,3];} const a=f(); const b=a; console.log(b);` | `[ 1, 2, 3 ]` | `4104`, exit 0 |

Every row reproduces identically inside `function main(){…} main();`. `kali
check` exits 0 on all of them. The console-argument guard
(`crates/kali_codegen/src/emit/call.rs:31`, `:78`) sees only a console
argument that is itself a runtime-array identifier or an array-returning call.
A template substitution and a `+` operand take the string-coercion ladder,
whose terminal arm renders the i64 handle as an integer. An alias is not an
array binding, so the guard does not recognize it. All three lanes are
pre-existing on plain `new Array` bindings. They were left open by ruling R13,
because widening the string paths without a measured backstop risks
capability loss. The handle values are allocation-dependent.

**What it would cost:** the template and concat lanes need the same
`is_runtime_array_value` check at the string ladder's entry. That ladder is
shared by every `+`, so a spike must count what it moves first. The alias lane
needs `const b = a` to register `b` as an array binding (in codegen and in the
resolver together, per amendment A1), which would also make `b[i]` work.

## §3. Old runtime-array defects now reachable through an array return

| program | node | kali |
|---|---|---|
| `function f(){ return [1,2,3]; } console.log(f()[-1]);` | `undefined` | `3`, exit 0 (the length header) |
| `function f(){ return [1,2,3]; } function main(){ const b=f(); b.push(4); console.log(b[3]); } main();` | `4` | `0`, exit 0 |
| `function f(){ return [1,2,3]; } console.log(f()[5]);` | `undefined` | `0`, exit 0 (R-21, §6) |
| `function f(){ return [1,2,3]; } function main(){ const b=f(); const h=()=>b[2]; console.log(h()); } main();` | `3` | exit 1: `warning[E3100]` undefined identifier `b` … zero placeholder, then `E5506` "no lane proves this receiver is an array" |

`kali check` exits 0 on every row. None of these is new code. The
`[len][elem…]` runtime array has no bounds check, no negative-index guard, and
no growth. A returned array is now that kind of array, so every one of its
defects is reachable from a call. The `push` row writes past the
fixed-length allocation and reads back whatever the heap holds there. The
closure row printed `0` at exit 0 when Task 7 measured it. Backstop 1 (§15)
moved it to a refusal, and the `E3100` warning shows that the capture itself
is not resolved.

**What it would cost:** the negative and out-of-range reads need R-21's
`undefined` (§6). `push` on a fixed-length array needs a refusal at the call
(the growable lane is separate and already refuses escaping through a
`return`, probe `growable_escape_control`). The closure capture of a
call-bound binding is the closure-environment lane's work.

## §4. A computed boolean stored into a runtime array reads back `1`/`0`

| program | node | kali |
|---|---|---|
| `function f(x){ const t = x > 1; return [t]; } console.log(f(2)[0]);` | `true` | `1`, exit 0 |
| `function f(x){ const t = x > 1; return [t, x]; } function main(){ const b = f(2); console.log(b[0]); } main();` | `true` | `1`, exit 0 |
| `function main(){ const t = 3 > 1; const a = new Array(2).fill(t); console.log(a[0]); } main();` | `true` | `1`, exit 0 |
| probe `boolean_elements`: `function f() { return [true, false]; } function main() { const a = f(); console.log(a[0]); } main();` | `true` | exit 1, `E5506` "returning an array from `f` … an element is not an integer" |

The literal `true` is refused because the element check (spec §3.1) sees a
Boolean literal. An identifier whose value is a comparison is integer-shaped
to that check, so it is admitted, stored as `1`, and read back as an integer.
The `.fill(t)` row involves no return at all, so the gap is in the
runtime-array element repr, not in this project's classifier. It is the R-30
family (no `Repr::Boolean` axis) reaching array elements.

**What it would cost:** narrowing the element check to refuse any element
whose inferred repr is boolean is cheap and closes the return rows. The
`.fill` row needs the same check on the fill value. Rendering a boolean
element correctly needs a per-array element repr.

## §5. R-48: an array held in an object field (cross-reference, not refiled)

| program | node | kali at baseline `368b5b5ea` | kali at `d2202ed4b` |
|---|---|---|---|
| probe `r48_module_control`: `const o = { arr: new Array(3).fill(6) }; console.log(o.arr[0]);` | `6` | `0`, exit 0 | exit 1, `E5506` "no lane proves this receiver is an array" |
| probe `r48_function_control`: the same inside `function main(){…} main();` | `6` | `0`, exit 0 | exit 1, same `E5506` |
| oracle `r48_…_module_scope`: `let o={a:6}; o.a=[1,2]; console.log(o.a);` | `[ 1, 2 ]` | `0`, exit 0 | `0`, exit 0 (still `silent`) |
| oracle `r48_…_in_function`: the same inside `function main(){…} main();` | `[ 1, 2 ]` | `0`, exit 0 | `0`, exit 0 (still `silent`) |
| `function f(){ let o={a:6}; o.a=[1,2]; return o.a; } console.log(f()[0]);` | `1` | not measured | exit 1, same `E5506` |

These are the two §2 rows of `inline-allocation-value-position-discovered-defects.md`
(the probes) plus R-48's own oracle pair. The project measured R-48 as a
control only (spec §1.2). Backstop 1 moved the element reads from a silent `0`
to a refusal (spec §4.1), which is why the probes read REFUSES rather than
their baseline SILENT. The whole-value print of the field is not an element
read and does not reach that backstop, so R-48's register repro and both of
its oracle cases still measure SILENT. R-48 stays open in the register, in
cluster N1, which it now holds alone.

## §6. R-21: an out-of-bounds read of a runtime array prints `0` (cross-reference, not refiled)

| program | node | kali at baseline | kali at `d2202ed4b` |
|---|---|---|---|
| probe `r21_oob_control`: `function main() { const a = new Array(3).fill(4); console.log(a[5]); } main();` | `undefined` | `0`, exit 0 | `0`, exit 0 |
| `function f(){ return [1,2,3]; } console.log(f()[5]);` | `undefined` | `0`, exit 0 (S1's lane) | `0`, exit 0 |

The control did not move, as spec §1.2 requires. A returned array's
out-of-range read inherits the same `0` (§3). Backstop 2 at full width would
have refused the control (spec §4.1, the one class-(a) probe). Backstop 2 was
dropped (§8), so the control is unchanged.

## §7. A host-exported function that returns an array hands the host a handle, or its old `0`

| program (`kali build`, the export called from node through `WebAssembly.instantiate` with stub imports) | node calling the JS directly | `f()` from the wasm export |
|---|---|---|
| `// kali-tree-shake: f` / `function f() { return [7, 8, 9]; }` | `[ 7, 8, 9 ]` | `4104n` (a heap handle) |
| `// kali-tree-shake: f` / `function f() { return [1.5, 2.5]; }` | `[ 1.5, 2.5 ]` | `0n` (the pre-project placeholder) |

Ruling R8: a function with no in-program call edge that does not escape (a
tree-shake root, an entry point, dead code) is never tainted. It keeps its
pre-project lane, because there is no kali reader to protect, and tainting it
refused `browser/math_sqrt_cbrt_frozen_aliases` (a correct program, 8 trials)
during Task 4. Admission is unaffected. An integer-element return is therefore
admitted and hands the host a raw heap handle, where the baseline handed it
`0`. A float-element return is neither admitted nor tainted, and hands the host
`0` as before. Neither value is the array node would produce. `kali check`
exits 0 on both.

**What it would cost:** the host ABI has no array return type. Until it has
one, the honest move is to refuse an array-shaped return from an *exported*
function. That needs an "is exported" fact that inference does not have
today, which is why R8 chose the call-edge proxy.

## §8. Backstop 2 was dropped: the array-literal placeholder `0` stays silent, with no guard

Spec §3.3's backstop 2 refuses in `emit_aggregate_literal`'s array branch when
a value is wanted, instead of pushing `I64Const(0)`. It was measured at full
width on top of backstop 1 (`d98f9dc49`), and **2501 tests failed** (spec
§4.1, by family: `browser/` 1269, `misc/` 191, `array/` 146, `soundness/` 81,
`oracle/` 79, `runtime/` 68, `object/` 52, `string/` 7 case trials, and 608
tests in hand-written targets). Class (b), meaning a row that was correct
before and now refuses, is not empty:

| (b) row | evidence of "correct before" |
|---|---|
| `runtime/array_return::{allocation_returned_directly, bound_passed_on, computed_elements, dyn_index, empty_literal, if_else_both_return, mutate_call_bound, nested_decl, r14_register_in_function, s03_bound_in_main, s06_loop_filled, s07_store_then_return, s08_direct_fill}_computes`, `::const_literal_binding_return` (14) | each case pins node v26.10.0's exact output |
| probes `bound_passed_on`, `computed_elements`, `const_literal_return`, `dyn_index`, `empty_literal`, `if_else_both_return`, `mutate_call_bound`, `nested_decl`, `s03_bound_in_main`, `s06_loop_filled`, `s07_store_then_return`, `s08_direct_fill` (12) | CORRECT in the probe runner at backstop 1 |
| `nbody_…`, `spectral_norm_…`, `fannkuch_redux_…`, `mandelbrot_…`, `binary_trees_small_n_…`, `binary_trees_canonical_n21_…` canonical-output tests (6) | the benchmark's canonical output |
| `acceptance_web_baseline_prefix_matches_node_byte_for_byte`, `acceptance_web_baseline_with_url_matches_node_byte_for_byte`, `delete_reinsert_enumeration_matches_node`, `quoted_and_numeric_like_keys_enumerate_in_es_order` (4) | node byte-for-byte or ES-order comparison |

The one class-(a) probe was `r21_oob_control` (§6). The other ~2480 rows were
not individually classified. The full list is not committed; spec §4.1 gives
the patch and the command that reproduces it.

**Final width: none.** The backstop cannot be narrowed at its site. Narrowing
would refuse only when the literal's parent is a `Return`, a `Call` argument or
a declarator initializer, but `LirNode` (`kali_lir/src/node.rs`) carries only
`kind/text/children/function_flavor`, and `FunctionEmitter` keeps no emit
stack, so `emit_aggregate_literal` cannot see the parent. The placeholder `0`
therefore stays a **silent lane with no backstop**: any array literal in a
value position that no materializing lane claims evaluates to `0`. This
project's own lanes (the return arm and the A2 `const` literal) claim theirs
before the placeholder is reached. Every other position is unaudited.

**What it would cost:** an emit-time parent (a stack in `FunctionEmitter`, or
a parent link on `LirNode`), then the narrowed backstop re-measured against
the (b) table above. Why the unnarrowed refusal reached 2501 tests was not
investigated.

## §9. A ternary array return refuses

| program | node | kali |
|---|---|---|
| `function f(c){ return c ? [1] : [2]; } console.log(f(true)[0]);` | `1` | exit 1, `E5506` "no lane proves this receiver is an array" |
| `function f(c){ return c ? [1,2] : [3,4]; } function main(){ const b = f(false); console.log(b[0]); } main();` | `3` | exit 1, same `E5506` |

`kali check` exits 0 on both, and only codegen refuses (backstop 1). The
classifier calls `c ? [..] : [..]` `NonArray`, so `f` is not array-returning
and its literals keep the placeholder lane. The refusal is honest. The work to
admit it would be a `Conditional` arm in the return classifier that requires
both branches to be admitted literals or allocations, plus a materializing
return arm per branch.

## §10. Assignment and destructuring from an array-returning call refuse

| program | node | kali `run` | kali `check` |
|---|---|---|---|
| `function f(){ return [1,2,3]; } function main(){ let b = [0,0,0]; b = f(); console.log(b[1]); } main();` | `2` | exit 1, `E5506` "no lane proves this receiver is an array" | exit 0 |
| the same at module scope | `2` | exit 1, same `E5506` | exit 0 |
| `function f(){ return [1,2,3]; } function main(){ let b; b = f(); console.log(b[1]); } main();` | `2` | exit 1, `E5506` "reassigning an array binding to a non-array value …" | exit 1, same |
| `function f(){ return [1,2,3]; } function main(){ const [x] = f(); console.log(x); } main();` | `1` | exit 1, `E5506` "a reserved word cannot be used as a binding name" | exit 1, same |
| `function f(){ return [1,2,3]; } const [x, y] = f(); console.log(x + "," + y);` | `1,2` | exit 1, same `E5506` | exit 1, same |

Spec §3.1 keeps assignment on its refusing lane, and so does probe
`reassign_control`. The first two rows refuse only in codegen, so `check` and
`run` disagree. The destructuring rows refuse with a message that does not
describe the program, since there is no reserved word in it. The parser's
`parse_variable_declarator` (`crates/kali_parser/src/statement.rs:143`)
accepts only a binding-name token, so every array pattern gets this message
whatever its initializer. It is not this project's refusal, and destructuring
is unsupported in general.

## §11. Non-`I64` element arrays refuse

| probe | program | node | kali (baseline → now) |
|---|---|---|---|
| `float_elements` | `function f() { return [1.5, 2.5]; } function main() { const a = f(); console.log(a[0]); } main();` | `1.5` | SILENT → REFUSES (`E5506` "… an element is not an integer") |
| `float_fill_return` | `function f() { const a = new Array(2).fill(1.5); return a; } …const b = f(); console.log(b[0]);` | `1.5` | OTHER (`E4201`) → REFUSES |
| `string_literal_elements` | `function f() { return ["a", "b"]; } …const a = f(); console.log(a[0]);` | `a` | SILENT → REFUSES |
| `s11_string_elements` | `function mk() { const a = new Array(2).fill("x"); return a; } …const c = mk(); console.log(c[0]);` | `x` | SILENT → REFUSES |
| `boolean_elements` | `function f() { return [true, false]; } …const a = f(); console.log(a[0]);` | `true` | SILENT → REFUSES |
| `nested_array_elements` | `function f() { return [[1], [2]]; } …const a = f(); console.log(a[0][0]);` | `1` | SILENT → REFUSES |

All refuse as spec §1.2 says. A computed boolean is not caught (§4). Making any
of these returnable needs a per-element repr on the runtime array (float
slots, string handles, nested handles), which is a separate project.

## §12. Async and generator functions: not candidates, and `check`/`run` disagree on async

| program | node | kali `run` | kali `check` |
|---|---|---|---|
| `async function f(){ return [1,2,3]; } console.log(f()[0]);` | `undefined` | exit 1, `E5506` "no lane proves this receiver is an array" | exit 0 |
| `function* f(){ yield 1; return [1,2,3]; } console.log(f()[0]);` | `undefined` | exit 1, `E5506` "generator function lowering is unavailable …" | exit 1, same |

Ruling R12 excludes async and generator functions from the candidates and from
the taint. Before backstop 1 the async row printed `0` (Task 7 measured `1`
while async functions were still admitted, then `0` once R12 excluded them).
It now refuses, but only in codegen, so `kali check` admits a program that
`kali run` refuses. That is the same pattern as the `.length` floor. Node's
`undefined` comes from indexing a Promise, and kali has no lane for it.

## §13. Smaller fail-closed and check/run gaps

| program | node | kali `run` | kali `check` | note |
|---|---|---|---|---|
| `function f(x){ x[0]; return x; } console.log(f(5));` | `5` | exit 1, `E5506` "printing a whole runtime array …" | exit 0 | A param subscripted in its body counts as a runtime array (spec-sanctioned), so `f` is array-returning. It is admitted although `5` is passed, and only the console guard stops it. |
| `function f(){ return [1,2,3]; } console.log(f()["length"]);` | `3` | exit 1, `E5506` "rendering a String() result bound to a variable … or returned from a function …" | exit 0 | A string-literal index. The message does not describe the program. Pre-existing for `new Array` bindings. |
| `function f(){ return [1,2,3]; } console.log(f()["2"]);` | `3` | exit 1, same `E5506` | exit 0 | same |
| `function f(){ return [1,2,3]; } function main(){ const b=f(); console.log(b["length"]); } main();` | `3` | exit 1, same `E5506` | exit 0 | same |
| `class C {} function f(){ return [1,2,3]; } function main(){ let b=f(); console.log(b[0]); } main();` | `1` | exit 1, `E5506` "no lane proves this receiver is an array" | exit 0 | Any class anywhere in the program declines every `let` call-bound binding (Task 4, coarse and fail-closed). `const b` still computes (`1`), and so does `let b` without the class. |

## §14. Mutual recursion is out of scope

| probe | program | node | kali |
|---|---|---|---|
| `mutual_recursion` | `function f(n) { if (n === 0) { return [5]; } return g(n - 1); } function g(n) { return f(n); } console.log(f(2)[0]);` | `5` | OTHER: exit 1, `E3100` undefined identifier `g` (baseline: the same) |

Ruling R7: kali has no forward function reference, even for scalars. The
solver's mutual-recursion unit test stays, but the program cannot be compiled
until forward references land. The probe stays OTHER.

## §15. Backstop 1 landed at full width

The numeric-index fallback (`crates/kali_codegen/src/emit/operators.rs`, the
floor of the numeric-index arm) refuses with `E5506` ("no lane proves this
receiver is an array"). It measured **27 failing tests, all class (a)**, with
no (b) and no (c), so it was not narrowed (spec §4.1, committed `d98f9dc49`).
The 27 were re-pinned: 16 browser `promise_all{,_settled}_bundle` trials
(re-generated from `gen_batch8a.py`), 4 `runtime_smoke` Promise.all builds, 3
`object/` cases, the two `r06c` oracle cases (R-06's §0.2 lane now reads
FAIL_CLOSED), `runtime_argv::process_argv_huge_literal_index_…`, and
`runtime/array_return::async_function_stays_off_the_array_return_lane`.
Probes `r48_module_control` and `r48_function_control` moved SILENT →
REFUSES (§5). It is codegen-only, which is the source of every `check`-exit-0
/ `run`-refuses row in §9, §10 and §12.

## §16. `callback_escape` refuses

Probe `callback_escape` (`function mk(x) { return x; } function main() { const a = mk(new Array(2).fill(3)); const r = [1, 2].map(mk); console.log(a[0] + "," + r[1]); } main();`,
node `3,2`) reads REFUSES at `d2202ed4b` (`E5506` "array callback method 'map'
is unavailable …"), as it did at Task 7. Brief item 8 asks for an entry only if
the verdict is neither CORRECT nor REFUSES, so this section records the
measurement and nothing more.
