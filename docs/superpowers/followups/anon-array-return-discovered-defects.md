# Defects the anon-array-return project measured and did NOT fix

**Filed** 2026-10-02 by the **anon-array-return** project
(`docs/superpowers/specs/2026-10-02-anon-array-return-design.md`), on the
convention `array-return-discovered-defects.md` and its predecessors use: a
project that measures more than it fixes writes down what it left, so the
silence is not read as absence.

**Oracle:** `node v26.10.0`.
**Measured at:** `97c008bc5` (branch `anon-array-return`), on
`target/debug/kali` built from that commit (`cargo build -p kali_cli`, the
`dev` profile, so `debug_assert!` is live). Probe rows come from
`tools/array-return-probes/run.sh` at that commit. A probe's baseline column
is `tools/array-return-probes/baseline-anon.tsv`, measured at the spec's
baseline `068b29950`. Rows that are not probes were run by hand as `node P.js`
against `kali run P.js` and `kali check P.js`, with the program text given in
full. Their **baseline** column was measured on a `kali` built from
`068b29950` in a throwaway worktree. A row that says "not measured at the
baseline" was not.
§1 and §2 were re-measured by Task 8 at `4898f3994`, on `target/debug/kali`
built from that commit, and their `cb63f9909` column on a `kali` built from
`cb63f9909` in a throwaway worktree. Each says so at its head.

**Register:** no entry of `kali-silent-miscompile-register.md` moved lane, so
the register and `blast-radius-ranking.md` are not edited. Read at
`97c008bc5`, R-14's entry still says "An arrow or function-expression array
return passed on to an array parameter still reads `0`" and points at
`array-return-discovered-defects.md` §1. That sentence was true at
`d2202ed4b`. It is superseded by that §1's FIXED note, and R-14's own lane (a
`function` declaration) is unchanged. §4 and §5 below are R-10's class
(block-scoped shadowing is unmodeled, §0.2 SILENT when read at `97c008bc5`).
§2 is a silent lane of R-05's class, whose §0.2 row reads FAIL_CLOSED. None of
them is filed in the register by this project.

**Ranked, most consequential first.** §1-§5 were filed as silent wrong
values: kali exits 0 and prints something node does not. §1 and §2 led
because this branch made them: forms of each refused at the baseline and were
silent at `97c008bc5`. Task 8 closed both of this branch's own regressions
(§1 entirely, and §2's direct-index row), re-measured at `4898f3994`. §2's
remaining silent rows are pre-existing member-lowering rows (silent at the
baseline, or not measured there), like §3-§5. The section numbers are kept,
not re-ranked, because the spec's amendments and the `anon_array_return.toml`
rationales cite them. §1 is now a FIXED record. §6 is a debug-build panic that
precedes the switch-case blind spot the plan asked about. §7 and §8 fail
closed: §7 is the out-of-scope shapes that need first-class function values,
and §8 is a `kali check` / `kali run` disagreement on callback rows. §9 records
the capability-loss spike and the final probe diff.

| brief item (Task 7 Step 3, plus controller rulings) | section |
|---|---|
| 1. module-scope `const` from another function; `let`-bound arrow | §7 |
| 2. param-dependent elements | §1 |
| 3. named function expression called immediately | §3 |
| 4. switch-case blind spot | §6 |
| 5. redeclared alias; callback-plus-call | §4, §8 |
| 6. spike capability-loss list, and Task 4's restoration | §9 |
| (c) block-level `function` shadowing a `const` arrow | §5 |
| (d) member call resolved to a same-named declaration | §2 |
| (e) `check` / `run` disagreement on callbacks | §8 |

---

## §1. An arrow whose elements are its params admits any argument (ruling R15 discharges vacuously) — FIXED by Task 8

**FIXED** by Task 8 (`fef5de60a`). Re-measured at `4898f3994` against node
v26.10.0 on `target/debug/kali` built from that commit. The `cb63f9909` column
is the record before the fix. The baseline column is the one measured at
`068b29950` when this section was filed.

| program | node | kali `run` at `4898f3994` | kali `check` at `4898f3994` | `run` at `cb63f9909` (`check`) | baseline `068b29950` |
|---|---|---|---|---|---|
| `const f = (n) => [n, n]; function g(x){return x[1];} console.log(g(f(3)));` | `3` | `3`, exit 0 | exit 0 | `3`, exit 0 (exit 0) | `0`, exit 0 |
| `const f = (n) => [n, n]; console.log(f(3)[0]);` | `3` | `3`, exit 0 | exit 0 | `3`, exit 0 (exit 0) | exit 1, `E5506` "no lane proves this receiver is an array" |
| `const f = (n) => [n, n]; const a = f(3); console.log(a[0]);` | `3` | `3`, exit 0 | exit 0 | `3`, exit 0 (exit 0) | not measured at the baseline |
| `function g(x){return x[1];} console.log(g(((n) => [n, n])(3)));` | `3` | `3`, exit 0 | exit 0 | `3`, exit 0 (exit 0) | not measured at the baseline |
| `const f = (n) => [n, n]; function g(x){return x[1];} console.log(g(f(true)));` | `true` | exit 1, `E5506` "returning an array from `f` … an element is not an integer" | exit 1, same | **`1`, exit 0** (exit 0) | `0`, exit 0 |
| `const f = (n) => [n, n]; console.log(f(true)[0]);` | `true` | exit 1, same `E5506` | exit 1, same | **`1`, exit 0** (exit 0) | exit 1, `E5506` "no lane proves this receiver is an array" |
| `const f = (n) => [n, n]; const a = f(true); console.log(a[0]);` | `true` | exit 1, same `E5506` | exit 1, same | **`1`, exit 0** (exit 0) | exit 1, same `E5506` |
| `function g(x){return x[1];} console.log(g(((n) => [n, n])(true)));` | `true` | exit 1, `E5506` "returning an array from an immediately-invoked function … an element is not an integer" | exit 1, same | **`1`, exit 0** (exit 0) | `0`, exit 0 |
| `const f = (n) => [n, n]; function g(x){return x[1];} console.log(g(f("a")));` | `a` | exit 1, `E5506` "returning an array from `f` … an element is not an integer" | exit 1, same | `a`, exit 0 (exit 0) | `0`, exit 0 |
| `const f = (n) => [n, n]; console.log(f("a")[0]);` | `a` | exit 1, same `E5506` | exit 1, same | `a`, exit 0 (exit 0) | exit 1, same `E5506` |
| `const f = (n) => [n, n]; function g(x){return x[1];} console.log(g(f(1.5)));` | `1.5` | exit 1, same `E5506` | exit 1, same | exit 1, `E4201` "failed to load WASM module: failed to compile" (exit 0) | exit 1, same `E4201` |
| `const f = (n) => [n, n]; console.log(f(1.5)[0]);` | `1.5` | exit 1, same `E5506` | exit 1, same | exit 1, same `E4201` (exit 0) | exit 1, `E5506` "no lane proves this receiver is an array" |
| `const f = (n) => [n, n]; const a = f(1.5); console.log(a[0]);` | `1.5` | exit 1, same `E5506` | exit 1, same | exit 1, same `E4201` (exit 0) | exit 1, same `E5506` |
| `const f = function(n){ return [n, n]; }; console.log(f(true)[0]);` | `true` | exit 1, same `E5506` | exit 1, same | **`1`, exit 0** (exit 0) | not measured at the baseline |
| `const f = (v) => new Array(2).fill(v); console.log(f(true)[0]);` | `true` | exit 1, same `E5506` | exit 1, same | **`1`, exit 0** (exit 0) | not measured at the baseline |
| `const f = (v) => new Array(2).fill(v); console.log(f(3)[0]);` | `3` | `3`, exit 0 | exit 0 | `3`, exit 0 (exit 0) | not measured at the baseline |
| control: `function f(n){ return [n, n]; } function g(x){return x[1];} console.log(g(f(true)));` | `true` | exit 1, `E5506` "returning an array from `f` … an element is not an integer" | exit 1, same | exit 1, same (exit 1) | exit 1, same |
| control: the same with `f(1.5)` | `1.5` | exit 1, same `E5506` | exit 1, same | exit 1, same (exit 1) | exit 1, same |

Every row prints node's value or refuses with `E5506` at both `check` and
`run`, as spec §1 claims. The `3` rows still compute. The string rows printed
node's `a` at `cb63f9909` with nothing proving it. They now refuse, which is
no loss against the baseline, where they printed `0` or refused. The
`function` declaration controls are unchanged.

**Mechanism, as it was at `cb63f9909`.** Two gaps made ruling R15's element
proof hold vacuously for a called anonymous body.

1. The param fact. `NumFact::Param` (`crates/kali_types/src/repr_infer.rs`,
   `NumProofCheck::fact_holds`) holds when `call_sites_enumerable(func)` and
   every edge in `edges_to(func)` proves its argument. A called anonymous body
   joins the candidates with a declaration count of 1, so
   `call_sites_enumerable` was true. The R15 edges were snapshotted from
   `self.calls` under the source callee name `f` (amendment A-1), and an IIFE
   recorded no edge, so `edges_to("__kali_fn_N")` was empty and `.all(…)` was
   vacuously true. This is the only gap for a block-bodied function
   expression (the `function(n){ return [n, n]; }` row).
2. The concise arrow body. The arrow arm of `visit_expr` classified the body
   as a return but recorded none of a `return` statement's other facts: no
   element obligation for a returned literal, no fill proof for a returned
   allocation, and no return number proof. With no obligation on the return
   element class there was nothing for the param fact to refute, so even a
   correct edge would not have refused. It also left the elements' repr
   unwired, which is why `f(1.5)` failed at WASM load instead of refusing.

**The fix (`fef5de60a`).** The R15 snapshot keys each edge by
`array_return_callee(caller, callee)`, the same resolution the array-return
facts use, and skips an edge it returns `None` for (a shadowed callee, as
before). Every IIFE pushes its own edge with its argument proofs.
`CallEdge.callee` and `resolve_calls` are unchanged (A-1). An anonymous id
with no edge is not call-site-enumerable, a fail-closed guard no measured
program reaches, since every way an anonymous body becomes `called` now
records an edge. A concise arrow body records a return's facts through the
same helpers as a `return` statement (`note_array_return`,
`visit_array_return_value`). See spec amendment A-9.

## §2. A member call `o.f()` runs a same-named top-level declaration

The direct-index row is closed by Task 8 (`4898f3994`). The scalar and
passed-on rows are not. Re-measured at `4898f3994` against node v26.10.0. The
`cb63f9909` column is the record before Task 8, and the baseline column is
the one measured at `068b29950` when this section was filed.

| program | node | kali `run` at `4898f3994` | kali `check` at `4898f3994` | `run` at `cb63f9909` | baseline `068b29950` |
|---|---|---|---|---|---|
| `function f(){return 7;} const o = {f: () => 5}; console.log(o.f());` | `5` | **`7`, exit 0** | exit 0 | `7`, exit 0 | `7`, exit 0 |
| `function f(){return [1,2,3];} const o = {f: () => [4,5,6]}; console.log(o.f()[0]);` | `4` | exit 1, `E5506` "an indexed read is unavailable … no lane proves this receiver is an array" | exit 0 | **`1`, exit 0** | exit 1, `E5506` "no lane proves this receiver is an array" |
| `function f(){return [1,2,3];} const o = {f: () => [4,5,6]}; function g(x){return x[1];} console.log(g(o.f()));` | `5` | **`2`, exit 0** | exit 0 | `2`, exit 0 | `2`, exit 0 |
| `function f(){return [1,2,3];} const o = {f: () => [4,5,6]}; const h = o.f; console.log(h()[0]);` | `4` | exit 1, `E5506` "an indexed read is unavailable … no lane proves this receiver is an array" | exit 0 | exit 1, same `E5506` | not measured at the baseline |
| `function f(){return [1,2,3];} const o = {f: () => [4,5,6]}; const h = o.f; function g(x){return x[1];} console.log(g(h()));` | `5` | **`2`, exit 0** | exit 0 | `2`, exit 0 | not measured at the baseline |
| control: `const o = {f: () => 5}; console.log(o.f());` | `5` | exit 1, `E5506` "calling 'f' is unavailable … call through a first-class function value" | exit 0 | exit 1, same `E5506` | not measured at the baseline |
| control: `const o = {f: () => [4,5,6]}; console.log(o.f()[0]);` | `4` | exit 1, same `E5506` | exit 0 | exit 1, same `E5506` | not measured at the baseline |

Found in Task 5's review. Codegen lowers the member callee `o.f` to the
top-level declaration `f`, so the wrong function runs. With no declaration
named `f` (the controls), the call refuses, as register R-05's lane does
(FAIL_CLOSED at its §0.2 row). `kali check` exits 0 on every row: each
refusal here is codegen's.

**What Task 8 closed.** Task 5 (`c2c2eb1b9`) made `array_return_call_elem`
(`crates/kali_codegen/src/emitter.rs`) resolve its callee "exactly as the
call itself is lowered", through `resolve_bound_member_callable_node`. That
resolver lands on the declaration `f`, an admitted array return, so
`o.f()[0]` read `f`'s real array and printed `1`. Task 8 (`4898f3994`)
narrows `array_return_call_elem` to a bare-identifier callee (through
`bindings`) or a callee that is itself an anonymous function expression (an
IIFE). Any other callee is not an array-return call, so the direct-index row
refuses through backstop 1 again, as at the baseline (spec amendment A-10).
The `anon_member_call_decl` probe and the
`member_call_with_same_named_declaration_refuses_at_run` case pin it.

**What remains.** The scalar and passed-on rows were silent at the baseline,
and are unchanged at `4898f3994`. They are R-05's class, not this project's.
Task 8 also measured a `const h = o.f` alias: its passed-on row prints `2` at
`cb63f9909` and at `4898f3994`, by the same member lowering.

**What it would cost:** the member-call lowering must not resolve a property
name to a same-named top-level declaration. It should refuse as R-05's lane
does without the declaration. That one fix closes the three remaining silent
rows.

## §3. A named function expression keeps its silent pre-project lane

| program | node | kali `run` at `97c008bc5` | kali `check` | baseline `068b29950` |
|---|---|---|---|---|
| `function g(x){return x[1];} console.log(g((function h(){ return [1,2]; })()));` | `2` | **`0`, exit 0** | exit 0 | `0`, exit 0 |
| `const f = function h(){ return [1,2,3]; }; function g(x){return x[1];} console.log(g(f()));` | `2` | **`0`, exit 0** | exit 0 | `0`, exit 0 |
| `console.log((function h(){ return [1,2]; })()[0]);` | `1` | exit 1, `E5506` "no lane proves this receiver is an array" | exit 0 | exit 1, same |

Ruling (Task 2, spec amendment A-5): `direct_callee`
(`crates/kali_types/src/array_return.rs:425-436`) keys an arrow or function
expression only by a synthetic `__kali_fn_N` id. A named function
expression's id is its own name, so it is neither a candidate nor in
`called`, and it keeps the literal placeholder `0`. The plan (Review Focus #2)
expected a `FORM` refusal, and Task 6 measured the silent `0` instead. The
`const`-bound form (row 2) is not an IIFE, but it takes the same path: the
alias table keys only a synthetic id, so `f` resolves to nothing.

**What it would cost:** a named function expression needs a key that cannot
collide with a declaration of the same name. One option is for
`name_anon_functions` to give it a synthetic id as well, with the
self-reference to `h` scoped to its body. With that key it can be admitted
like an arrow. A cheaper fail-closed step taints it (`FORM`) under a
collision-free key, so rows 1 and 2 refuse.

## §4. A `const` alias redeclared in an inner block runs the inner body

| program | node | kali `run` at `97c008bc5` | kali `check` | baseline `068b29950` |
|---|---|---|---|---|
| `const f = () => [1,2,3]; { const f = () => 2; } function g(x){return x[1];} console.log(g(f()));` | `2` | **`0`, exit 0** | exit 0 | `0`, exit 0 |
| `const f = () => { console.log("outer"); return 1; }; { const f = () => { console.log("inner"); return 2; }; } console.log(f());` | `outer` `1` | **`inner` `2`, exit 0** | exit 0 | not measured at the baseline |

Spec §3.1 makes inference decline a name declared twice in one function
(`FnAlias::Blocked`, `repr_infer.rs:1786-1807`), because codegen's
`self.bindings` is flat. The second row shows what codegen does with the flat
map: the outer call runs the **inner** body, even for scalars. This is
register R-10's class (an inner block declaration aliases the outer binding),
in the function-value lane. Since inference declines, `__kali_fn_0` is not in
`called` and keeps its placeholder. Whichever body runs, the array row cannot
print node's value. This is the agreement risk spec §3.4 names: inference
declines a shape that codegen resolves, so the old silent lane survives.

**What it would cost:** the real fix is R-10's, which is block-scoped
bindings in codegen and the resolver. A fail-closed step for this lane alone
would have codegen refuse a call whose callee name inference reports as
`Blocked`. That needs the blocked set published in `ReprTable`, and it would
refuse the scalar row too.

## §5. A block-level `function` declaration shadowing a `const` arrow

| program | node | kali `run` at `97c008bc5` | kali `check` | baseline `068b29950` |
|---|---|---|---|---|
| `const f = () => [1, 2, 3]; function g(x) { return x[1]; } { function f() { return [7, 8, 9]; } console.log(g(f())); }` | `8` | **`0`, exit 0** | exit 0 | `0`, exit 0 |

Task 3's fix round (`b86bffa9e`, spec amendment A-7) makes a same-scope
`function` declaration block the `const` alias. Task 3's review predicted
that without it, Task 4 would admit the outer arrow here and print `2`. That
prediction was not measured. Now inference declines, and the program prints
`0` as it did at the baseline. The ledger records the same `0` at
`02ff6a9ec`. Two names `f` share one function scope in codegen. Which body
`f()` calls was not traced. This is R-10's class again.

**What it would cost:** the same as §4. Block-scoped function declarations
need R-10's scope frames.

## §6. A `switch` case body: the alias blind spot is unreachable behind a debug-build panic

| program | node | kali `run` / `check` at `97c008bc5` | baseline `068b29950` |
|---|---|---|---|
| `function g(x){return x[1];} switch (1) { case 1: { const f = () => [1,2,3]; console.log(g(f())); } }` | `2` | exit 101, panic at `repr_infer.rs:1594` "F-AB-2 lockstep violation: Phase-B (walk 4) seeded __kali_fn ids the shared Phase-A descent (walks 1-3) never registered: [\"__kali_fn_0\"]", both commands | exit 101, same panic (`repr_infer.rs:1572`) |
| the same without the case block braces | `2` | exit 101, same panic | exit 101, same |
| the same with `const f = function(){ return [1,2,3]; };` | `2` | exit 101, same panic | exit 101, same |
| `switch (1) { case 1: { const f = () => [1,2,3]; console.log(f()[1]); } }` | `2` | exit 101, same panic | exit 101, same |
| `switch (1) { case 1: { const f = () => 7; console.log(f()); } }` | `7` | exit 101, same panic | exit 101, same |
| `switch (1) { case 1: { const xs = new Array(2).fill(1); console.log(xs.length); const h = () => 7; } }` | `2` | exit 101, same panic | exit 101, same |

Plan Review Focus #4 predicted a silent blind spot: Phase A2's
`collect_local_names` does not descend into switch case bodies, so the alias
table never sees a `const f` there, while codegen resolves it. That prediction
cannot be observed. **Any** arrow or function expression in a case body,
called or not and array-returning or not, trips the F-AB-2 lockstep
`debug_assert!` (`repr_infer.rs:1580-1600`), at the baseline as at
`97c008bc5`. Phase A's `descend_expr_fns` does not reach a case body, and
Phase B's walk 4 does. `docs/superpowers/followups/stageAB-followups.md`
§F-AB-2 planted this tripwire and does not list a switch case as a known
gap. Only the `dev`-profile binary was measured. A build without debug
assertions skips the assert, and what it then does with these programs was
not measured.

**What it would cost:** teach the shared Phase-A descent (walks 1-3) to visit
switch case bodies. That changes `local_names` and the fn registrations for
every lane at once, which is why the plan kept it out of scope. After it
lands, re-measure the first row. If A2 still skips the case body, the alias
table misses `f` and the row is the silent `0` the plan predicted.

## §7. Out of scope: these shapes need first-class function values, and refuse

| program | node | kali `run` at `97c008bc5` | kali `check` | baseline `068b29950` |
|---|---|---|---|---|
| probe `arrow_return`: `const f = () => [1, 2, 3]; function main() { const a = f(); console.log(a[0]); } main();` | `1` | exit 1, `E5506` "calling 'f' is unavailable … call through a first-class function value" | exit 0 | exit 1, same |
| probe `anon_module_from_main_control`: `const f = () => [1,2,3]; function main(){ function g(x){return x[1];} console.log(g(f())); } main();` | `2` | exit 1, same `E5506` | exit 0 | exit 1, same |
| probe `anon_let_control`: `let f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));` | `2` | exit 1, same `E5506` | exit 0 | exit 1, same |
| `var f = () => [1,2,3]; function g(x){return x[1];} console.log(g(f()));` | `2` | exit 1, same `E5506` | exit 0 | exit 1, same |
| `const o = { f() { return [1,2,3]; } }; console.log(o.f()[0]);` | `1` | exit 1, `E3100` undefined identifier `f` | exit 1, same | exit 1, same |

Spec §1.1 excludes these, and each one refuses as it did at the baseline.
Codegen's `bindings` is per function, so a module-scope `const` is invisible
inside `main`. A `let` or `var` binding is a mutable function value, so it is
not resolved statically. The first four refuse only at `kali run`, which is
where the first-class-callee guard runs, at codegen (register R-02's lane,
FAIL_CLOSED at its §0.2 row).

**What it would cost:** first-class function values: a closure representation,
a function table and an indirect call (register R-02, cluster G2). After that,
the array-return facts would need a callee set per call site, not one name.

## §8. A callback lane: `kali check` admits what `kali run` refuses

| program | node | kali `run` at `97c008bc5` | kali `check` | baseline `068b29950` |
|---|---|---|---|---|
| `const f = () => [1, 2]; console.log(f()[1]); const xs = new Array(2).fill(0); xs.forEach(f); console.log("done");` | `2` `done` | exit 1, `E5506` "calling 'forEach' is unavailable … call through a first-class function value" | exit 0 | exit 1, `E5506` "no lane proves this receiver is an array", then the same `forEach` `E5506` |
| probe `anon_map_callback_control`: `const xs = new Array(2).fill(1); const ys = xs.map(x => x + 1); console.log(ys[0]);` | `2` | exit 1, `E5506` "calling 'map' is unavailable …", then "no lane proves this receiver is an array" | exit 0 | exit 1, same |

Plan Review Focus #1 asked whether an anonymous function that is both called
and passed as a callback could become silent. It cannot today. `f()[1]` now
computes (it refused at the baseline), and `forEach(f)` still refuses at
`kali run`, so the program still exits 1. Both rows are the pre-existing
callback lane (register R-03, FAIL_CLOSED), and `kali check` exits 0 on both,
at the baseline and now. They are pinned as run-only refusals in
`crates/kali_cli/tests/cases/runtime/anon_array_return.toml`
(`anon_called_and_passed_as_callback_refuses_at_run`,
`anon_map_callback_control_refuses_at_run`).

**What it would cost:** the first-class-callee guard must run inside
`kali check` (an inference-side fact for "callee does not resolve to a
compiled function"). That is shared with §7 and R-02, and is not specific to
this project.

## §9. The capability-loss spike, and the final probe diff

**Spike (spec §5.1, the Task 3 tree `dce70f00e` on base `02ff6a9ec`).**
`cargo test --workspace` had 0 failures. The probes had one loss
(CORRECT → REFUSES):

| probe | program | node | baseline | spike | `97c008bc5` |
|---|---|---|---|---|---|
| `anon_allocation_control` | `const f = (n) => new Array(n).fill(4); function g(x){return x[1];} console.log(g(f(3)));` | `4` | `4` (CORRECT) | `E5506` "returning an array from `__kali_fn_0` … only a `function` declaration with a unique name can return an array" | `4` (CORRECT) |

Task 4 (`42e3a45ec`) restored it, measured CORRECT in the Task 4 probe run and
again at `97c008bc5`. The spike's other probe moves were SILENT → REFUSES and
were not losses: `anon_alias_chain`, `anon_arrow_passed_on`,
`anon_boolean_elements`, `anon_fnexpr_passed_on`, `anon_mixed_return`,
`anon_nested_const`, `anon_side_effect_body`. Seven of them are now CORRECT,
and `anon_boolean_elements` and `anon_mixed_return` still refuse (below).

**Final probe run at `97c008bc5`.** There are 0 SILENT rows among the 16
`anon_*` probes, and no non-`anon_*` probe moved. Against
`baseline-anon.tsv` (`068b29950`):

| probe | baseline | `97c008bc5` |
|---|---|---|
| `anon_alias_chain` | SILENT | CORRECT |
| `anon_arrow_passed_on` | SILENT | CORRECT |
| `anon_fnexpr_passed_on` | SILENT | CORRECT |
| `anon_iife_passed_on` | SILENT | CORRECT |
| `anon_nested_const` | SILENT | CORRECT |
| `anon_side_effect_body` | SILENT | CORRECT |
| `anon_bound` | REFUSES | CORRECT |
| `anon_direct_index` | REFUSES | CORRECT |
| `anon_direct_length` | REFUSES | CORRECT |
| `anon_boolean_elements` | SILENT | REFUSES (`E5506` "an element is not an integer") |
| `anon_mixed_return` | SILENT | REFUSES (`E5506` "it mixes array and non-array returns") |

`anon_allocation_control`, `anon_scalar_control`, `anon_let_control`,
`anon_map_callback_control` and `anon_module_from_main_control` did not move.
The pre-change run at `02ff6a9ec` gives the same diff, except that
`anon_iife_passed_on` reads REFUSES there, because Task 2 already tainted the
called IIFE `FORM`. The probe gate missed §1 at `97c008bc5`: no probe passed
a non-integer to a param-dependent arrow.

**Task 8 probe run at `4898f3994`.** Task 8 added two probes,
`anon_param_true` (§1's `f(true)[0]` row) and `anon_member_call_decl` (§2's
direct-index row). Both read SILENT on a `kali` built from `cb63f9909` and
REFUSES at `4898f3994`. There are 0 SILENT rows among the 18 `anon_*` probes,
and no other probe moved between `cb63f9909` and `4898f3994`.
