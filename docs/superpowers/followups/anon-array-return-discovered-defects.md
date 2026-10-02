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
§2 and §3 were re-measured by Task 8 at `4898f3994`, on `target/debug/kali`
built from that commit, and their `cb63f9909` column on a `kali` built from
`cb63f9909` in a throwaway worktree. Each says so at its head.
§1, §7 and §9 (the final review's) and the callback rows of §11 were measured
at `a9496b001`, on `target/debug/kali` built from that commit, with every
baseline cell measured on a `kali` built from `068b29950` in a throwaway
worktree (removed afterwards). Each says so at its head.

**Register:** no entry of `kali-silent-miscompile-register.md` moved lane, so
the register and `blast-radius-ranking.md` are not edited. Read at
`97c008bc5`, R-14's entry still says "An arrow or function-expression array
return passed on to an array parameter still reads `0`" and points at
`array-return-discovered-defects.md` §1. That sentence was true at
`d2202ed4b`. It is superseded by that §1's FIXED note, and R-14's own lane (a
`function` declaration) is unchanged. §5 and §6 below are R-10's class
(block-scoped shadowing is unmodeled, §0.2 SILENT when read at `97c008bc5`).
§3 is a silent lane of R-05's class, whose §0.2 row reads FAIL_CLOSED. None of
them is filed in the register by this project.

**Ranked, most consequential first.** §1-§7 are silent wrong values: kali
exits 0 and prints something node does not. Within that group, this branch's
own regressions lead. §1 is five forms that refused at the baseline and are
silent now (the runtime-array defects the declaration lane already had,
reached through an anonymous return). §2 and §3 are the branch's earlier
regressions: forms of each refused at the baseline and were silent at
`97c008bc5`. Task 8 closed both of those (§2 entirely, and §3's direct-index
row), re-measured at `4898f3994`. §3's remaining silent rows are pre-existing
member-lowering rows (silent at the baseline, or not measured there), like
§4-§6. §7 is silent too, but it is not a regression: its forms print the
baseline's value. It ranks after the other silent rows because it is a gap in
this project's own claim rather than a declined shape. The final review
re-ranked the sections: the old §1-§9 became §2-§6, §8, §10, §11 and §12, and
the spec's amendments, the `anon_array_return.toml` rationales, a comment in
`emitter.rs` and the array-return followups' cross-references were renumbered
with them. §2 is a FIXED record. §8 is a debug-build panic that precedes the
switch-case blind spot the plan asked about. §9, §10 and §11 fail closed: §9
is an over-refusal (the resolver refuses a read the declaration computes),
§10 is the out-of-scope shapes that need first-class function values, and §11
is a `kali check` / `kali run` disagreement on callback rows. §12 records the
capability-loss spike and the final probe diff.

| brief item (Task 7 Step 3, plus controller rulings) | section |
|---|---|
| 1. module-scope `const` from another function; `let`-bound arrow | §10 |
| 2. param-dependent elements | §2 |
| 3. named function expression called immediately | §4 |
| 4. switch-case blind spot | §8 |
| 5. redeclared alias; callback-plus-call | §5, §11 |
| 6. spike capability-loss list, and Task 4's restoration | §12 |
| (c) block-level `function` shadowing a `const` arrow | §6 |
| (d) member call resolved to a same-named declaration | §3 |
| (e) `check` / `run` disagreement on callbacks | §11 |
| final review: runtime-array defects reached through an anonymous return | §1 |
| final review: TypeScript-wrapped `const` arrow or IIFE | §7 |
| final review: the resolver's bare-name consumer | §9 |

---

## §1. Runtime-array defects now reachable through an anonymous return

This is the anonymous-return counterpart of
`array-return-discovered-defects.md` §3 ("Old runtime-array defects now
reachable through an array return", read at `a9496b001`). The `[len][elem…]`
runtime array has no bounds check, no negative-index guard, no growth and no
length write. An admitted anonymous return hands its caller that same array,
so each of those defects is reachable from `f()` as it is from a declaration.
Spec §1.1 says returned arrays "inherit all of those, exactly as declaration
returns do". That inheritance is real, and it is not free: at the baseline
every one of these forms refused.

**This branch turned baseline refusals into silent wrong values.** Measured at
`a9496b001` against node v26.10.0 on `target/debug/kali` built from that
commit, and at `068b29950` on a `kali` built from that commit in a throwaway
worktree. `kali check` exits 0 on every row, at both commits. Each `run` cell
below that prints a value exits 0.

| program | node | kali `run` at `a9496b001` | baseline `068b29950` (`run`) |
|---|---|---|---|
| `const f = () => [1,2,3]; console.log(f()[-1]);` | `undefined` | **`3`**, exit 0 (the length header) | exit 1, `E5506` "an indexed read is unavailable … no lane proves this receiver is an array" |
| `const f = () => [1,2,3]; console.log(f()[5]);` | `undefined` | **`0`**, exit 0 | exit 1, same `E5506` |
| `const f = () => [1,2,3]; const a = f(); a.push(4); console.log(a.length);` | `4` | **`3`**, exit 0 | exit 1: `warning[E3100]` undefined call target `push`, then `E5506` "`.length` is unavailable … no lane proves its length" |
| `const f = () => [1,2,3]; const a = f(); a.push(4); console.log(a[3]);` | `4` | **`0`**, exit 0 | exit 1: the same `E3100` warning, then `E5506` "an indexed read is unavailable …" |
| `const f = () => [1,2,3]; const a = f(); a.length = 1; console.log(a.length);` | `1` | **`3`**, exit 0 | exit 1: `E5506` "`.length` is unavailable …", `warning[E8001]` unsupported binary operator `=`, then the same `E5506` |

The declaration twins, measured at the same two commits (`function f(){ return
[1,2,3]; }` in place of the arrow; the `push` rows bind `const a = f()` at
module scope, as above). Every cell is identical at `a9496b001` and
`068b29950`, so the declaration lane has been silent here all along:

| program | node | `run` at `a9496b001` and at `068b29950` |
|---|---|---|
| `function f(){ return [1,2,3]; } console.log(f()[-1]);` | `undefined` | **`3`**, exit 0 |
| `function f(){ return [1,2,3]; } console.log(f()[5]);` | `undefined` | **`0`**, exit 0 |
| `function f(){ return [1,2,3]; } const a = f(); a.push(4); console.log(a.length);` | `4` | **`3`**, exit 0 |
| `function f(){ return [1,2,3]; } const a = f(); a.push(4); console.log(a[3]);` | `4` | **`0`**, exit 0 |
| `function f(){ return [1,2,3]; } const a = f(); a.length = 1; console.log(a.length);` | `1` | **`3`**, exit 0 |

`a.length = 1` is not in the declaration lane's §3 list (it lists `[-1]`,
`push`, `[5]` and a closure capture). It is silent there too, so it is the
same class and not specific to anonymous returns. The declaration lane's §3 says its `push` row writes past the
fixed-length allocation. For `a.length = 1`, only the outcome was measured:
the assignment does not change what `.length` reads, and how it is lowered
was not traced (at the baseline the refusing row carries an `E8001`
"unsupported binary operator" warning for it).

**The probe gate does not cover these.** No `anon_*` probe indexes out of
bounds, pushes, or assigns `.length`, so §12's "0 SILENT rows among the
`anon_*` probes" does not speak to them.

**What it would cost:** the same as the declaration lane's §3. The negative
and out-of-range reads need R-21's `undefined`
(`array-return-discovered-defects.md` §6). `push` on a fixed-length array and
a `.length` write need a refusal at the call or the assignment. Fixing them
once for the declaration lane fixes the anonymous lane, since both hand over
the same runtime array. A fail-closed alternative for this project alone
would refuse those operations on a call-bound anonymous return, which would
leave the declaration twin silent.

## §2. An arrow whose elements are its params admits any argument (ruling R15 discharges vacuously) — FIXED by Task 8

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

**Fix round 1 (`615af7548`), measured at `f45f3c4ac`.** These rows were
measured with `kali` built from each named commit (`068b29950` and
`cb63f9909` in throwaway worktrees). Each cell is `run`, then `check`'s exit
in parentheses. `G` is `function g(x){return x*2;} console.log(g(1));`.

| program | node | `f45f3c4ac` | `b9faa27f2` | `cb63f9909` | baseline `068b29950` |
|---|---|---|---|---|---|
| `G const f = (k) => new Array(g(k)).fill(7); console.log(f(1.5).length);` | `2`, `3` | `E5506` "returning an array from `f` … an element is not an integer" (1) | `E4201` "failed to load WASM module" (0) | `E4201` (0) | `E5506` "`.length` is unavailable … no lane proves its length" (0) |
| the same, block-bodied: `const f = (k) => { return new Array(g(k)).fill(7); };` | `2`, `3` | same `E5506` (1) | `E4201` (0) | `E4201` (0) | same `E5506` `.length` (0) |
| the same, a declaration: `function f(k){ return new Array(g(k)).fill(7); }` | `2`, `3` | same `E5506` (1) | `E4201` (0) | `E4201` (0) | `E4201` (0) |
| `function g(x){return x[1];} console.log(g(((n) => [n, n])(1.5)));` | `1.5` | `E5506` "returning an array from an immediately-invoked function … an element is not an integer" (1) | same (1) | `E4201` (0) | `E4201` (0) |
| `console.log(((n) => [n, n])(1.5)[0]);` | `1.5` | same `E5506` (1) | same (1) | `E4201` (0) | `E5506` "an indexed read … no lane proves this receiver is an array" (0) |

The allocation arm of `visit_array_return_value` visited only the fill value.
A call in the length (`g(k)`) recorded no call edge, so `g`'s param proof
held vacuously and the length was "proven" an integer. Both the `return`
statement and, after Task 8, the concise arrow body reached that arm. The
length arguments are now visited (`array_return::allocation_length_args`,
shared with `allocation_proof`), and every allocation row refuses `E5506` at
check and run. The declaration row's `E4201` predates this project: it
failed at load at the baseline too. The two anonymous allocation rows
refused at the baseline (an honest `E5506`), failed at load at `cb63f9909`,
and refuse again now.

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

## §3. A member call `o.f()` runs a same-named top-level declaration

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

## §4. A named function expression keeps its silent pre-project lane

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

## §5. A `const` alias redeclared in an inner block runs the inner body

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

## §6. A block-level `function` declaration shadowing a `const` arrow

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

**What it would cost:** the same as §5. Block-scoped function declarations
need R-10's scope frames.

## §7. A TypeScript-wrapped `const` arrow or IIFE stays silent

Each program is a `.ts` file (node v26.10.0 runs them directly). Measured at
`a9496b001` on `target/debug/kali` built from that commit, and at `068b29950`
on a `kali` built from that commit in a throwaway worktree. Each cell is `run`;
`check` exits 0 on every row, at both commits except where stated.

| program | node | `run` at `a9496b001` | baseline `068b29950` |
|---|---|---|---|
| `const f = (() => [1,2,3]) as any; function g(x: any){return x[1];} console.log(g(f()));` | `2` | **`0`**, exit 0 | `0`, exit 0 |
| `const f = (() => [1,2,3]) satisfies any; function g(x: any){return x[1];} console.log(g(f()));` | `2` | **`0`**, exit 0 | `0`, exit 0 |
| `function g(x: any){return x[1];} console.log(g(((() => [1,2,3]) as any)()));` | `2` | **`0`**, exit 0 | `0`, exit 0 |
| `const f = (() => [true,false]) as any; function g(x: any){return x[0];} console.log(g(f()));` | `true` | **`0`**, exit 0 | `0`, exit 0 |
| `const f = ((n: number) => { if (n) { return [1,2]; } return 5; }) as any; function g(x: any){return x[1];} console.log(g(f(1)));` | `2` | **`0`**, exit 0 | `0`, exit 0 |
| control, scalar: `const f = (() => 7) as any; console.log(f());` | `7` | `7`, exit 0 | `7`, exit 0 |
| control, unwrapped: `const f = () => [1,2,3]; function g(x: any){return x[1];} console.log(g(f()));` | `2` | `2`, exit 0 | `0`, exit 0 |
| control, unwrapped boolean: `const f = () => [true,false]; function g(x: any){return x[0];} console.log(g(f()));` | `true` | exit 1, `E5506` "returning an array from `f` … an element is not an integer" (`check` exit 1) | `0`, exit 0 |
| control, unwrapped mixed: `const f = (n: number) => { if (n) { return [1,2]; } return 5; }; function g(x: any){return x[1];} console.log(g(f(1)));` | `2` | exit 1, `E5506` "returning an array from `f` … it mixes array and non-array returns" (`check` exit 1) | `0`, exit 0 |

The wrapped forms print the baseline's `0` at every commit. They are not a
regression. They are the one place spec §1's claim ("a `const`-bound … arrow
… hands its caller a real runtime array", with the refusal for an array-shaped
return that is not admitted) does not hold, and the unwrapped controls show
the contrast: the same body refuses or computes once the wrapper is gone.

**Mechanism.** `note_fn_alias` and `direct_callee`
(`crates/kali_types/src/repr_infer.rs`, `crates/kali_types/src/array_return.rs`)
see through `TypeAssertion` and `SatisfiesExpression` by way of `unparen`, so
the synthetic `__kali_fn_N` id of the wrapped arrow becomes `called`.
`visit_expr` (`repr_infer.rs`) has an arm for `ArrowFunctionExpression` and
`FunctionExpression`, and none for `TypeAssertion` or `SatisfiesExpression`:
those fall to its final `_ => self.new_node()` arm (read at `a9496b001`), so
the wrapped expression is never walked. The body is therefore never visited
as an anonymous form (`anon_fn_forms`) and its `return` statements are never
recorded (`returns`). The id is `called` and has no admission and no taint,
and codegen still resolves the call (the scalar control prints `7`), so the
body keeps the literal placeholder `0`. This is spec §3.4's agreement risk:
inference and codegen resolve different sets of calls. The `.js`-only probe
gate (spec A-4) could not see it, because `as` and `satisfies` are TypeScript
syntax. This is a spec §3.4 agreement gap the probe gate missed.

**What it would cost:** walk the wrapped expression in `visit_expr`, with arms
for `TypeAssertion` and `SatisfiesExpression` that delegate to the inner
expression. That arm is shared by every inference lane (numeric proofs,
object slots, escape marking), so it touches all of them and needs the whole
suite re-run. The cheaper alternative is a fail-closed taint: a `called` id
whose body inference never walked is refused. That would also refuse the
scalar control (`(() => 7) as any`), which computes correctly today.

## §8. A `switch` case body: the alias blind spot is unreachable behind a debug-build panic

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

## §9. `kali_types`' resolver keys the array-return fact by the bare callee name

Measured at `a9496b001` on `target/debug/kali` built from that commit, and at
`068b29950` on a `kali` built from that commit in a throwaway worktree. Each
cell is `kali check` (exit and first diagnostic), then `kali run`'s result.
This is an over-refusal: no row prints a wrong value.

| program | node | `check` / `run` at `a9496b001` | baseline `068b29950` |
|---|---|---|---|
| `const F = () => [1,2,3]; const b = F(); let i = 1; console.log(b[i]);` | `2` | exit 1, `E5506` "computed member access `o[k]` is unavailable … unless the index is a literal or a compile-time-constant `const` binding, or the receiver is a runtime array …"; `run` refuses with the same `E5506` | same refusal at `check` and `run` |
| the declaration twin: `function F(){ return [1,2,3]; } const b = F(); let i = 1; console.log(b[i]);` | `2` | `check` exit 0, `run` prints `2` | `2` |
| `const F = () => [1,2,3]; const b = F(); let s = 0; for (let i = 0; i < b.length; i++) { s += b[i]; } console.log(s);` | `6` | exit 1, the same computed-member `E5506`; `run` refuses the same way | same refusal |
| the declaration twin of the loop (`function F(){ return [1,2,3]; }`) | `6` | `check` exit 0, `run` prints `6` | `6` |
| `const F = () => [1,2,3]; console.log(F()?.[1]);` | `2` | `check` exit 0; `run` exit 1, `E5506` "an indexed read is unavailable … no lane proves this receiver is an array" | same `run` refusal, `check` exit 0 |
| the declaration twin: `function F(){ return [1,2,3]; } console.log(F()?.[1]);` | `2` | `check` exit 0, `run` prints `2` | `2` |

The declaration and the arrow compute the same array at run time (the direct and bound reads
and `g(F())` all work, per the `anon_*` cases). What differs is a read the
resolver must classify at check time: a computed index through a variable, and
an optional-chain index. Those refuse for the arrow and compute for the
declaration. The baseline refused the same programs, so this is not a
regression.

**Mechanism.** `kali_types`' resolver asks "does this call return a runtime
array?" by source name. `call_returns_runtime_array`
(`crates/kali_types/src/resolve/member.rs`, about line 381 at `a9496b001`)
takes the callee `Expression::Identifier` and looks up
`repr_table.array_return(callee)`. The call-bound registration of
`const b = F()` as a runtime-array binding
(`crates/kali_types/src/resolve/mod.rs`, about line 990) goes through that
function. An anonymous function's fact is keyed by the synthetic
`__kali_fn_N` id (spec §3.1), so `array_return("F")` is `None`: the resolver
never learns that `F` is the alias of an admitted body, and the binding `b`
is not registered as a runtime array. The index through `i` then falls to the
computed-member gate, which refuses. The two `b[i]` rows refuse at `check`
and `run` with the resolver's diagnostic. The `?.[1]` row passes `check` and
refuses only at `run`, with the indexed-read backstop's message. Where that
row's refusal originates was not traced, and it is filed with this section
because it is a read that computes for the declaration and refuses for the
arrow, at both commits.

**What it would cost:** publish the resolved key in `ReprTable` (for example
the `(func, name) → __kali_fn_N` alias table, or a ready-made "this callee
name returns a runtime array" fact), and route the resolver's two lookups
through it, so that it resolves the callee exactly as inference does.

## §10. Out of scope: these shapes need first-class function values, and refuse

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

## §11. A callback lane: `kali check` admits what `kali run` refuses

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
`anon_map_callback_returning_scalar_refuses_at_run`). The `map` row's callback
`x => x + 1` returns a scalar, so it says nothing about an array-returning
callback. Those are the rows below.

Measured at `a9496b001` against node v26.10.0 on `target/debug/kali` built
from that commit, and at `068b29950` on a `kali` built from that commit in a
throwaway worktree. Cells are `run`'s first diagnostics, and `check`'s exit.

| program | node | `run` at `a9496b001` | `check` | baseline `068b29950` (`run`, `check`) |
|---|---|---|---|---|
| `const xs = new Array(2).fill(1); const ys = xs.map(x => [x]); console.log(ys[1]);` | `[ 1 ]` | exit 1, `E5506` "calling 'map' is unavailable …", then "an indexed read is unavailable … no lane proves this receiver is an array" | exit 0 | same two refusals, exit 0 |
| `const xs = new Array(2).fill(1); const ys = xs.flatMap(x => [x, x * 10]); console.log(ys[1]);` | `10` | exit 1, `E5506` "calling 'flatMap' is unavailable …", then the same indexed-read `E5506` | exit 0 | same two refusals, exit 0 |
| the `forEach(f)` row above, re-run | `2` `done` | exit 1, `E5506` "calling 'forEach' is unavailable …" | exit 0 | exit 1, `E5506` "calling 'forEach' …" (the indexed-read `E5506` comes first at the baseline), exit 0 |

In both array-returning rows the callback is never called directly, so it is
never an array-return candidate (spec §1.1) and the refusal is the callback
lane's own, unchanged from the baseline. Neither is silent, so they are
pinned as run refusals (`anon_map_callback_returning_array_refuses_at_run`,
`anon_flatmap_callback_returning_array_refuses_at_run`).

**What it would cost:** the first-class-callee guard must run inside
`kali check` (an inference-side fact for "callee does not resolve to a
compiled function"). That is shared with §10 and R-02, and is not specific to
this project.

## §12. The capability-loss spike, and the final probe diff

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
`anon_*` probes, and no non-`anon_*` probe moved. That count is of the
probe set, which does not cover the runtime-array defects an anonymous return
inherits from the declaration lane (§1: negative and out-of-range reads,
`push`, a `.length` write), nor the TypeScript-wrapped forms (§7). Against
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
called IIFE `FORM`. The probe gate missed §2 at `97c008bc5`: no probe passed
a non-integer to a param-dependent arrow.

**Task 8 probe run at `4898f3994`.** Task 8 added two probes,
`anon_param_true` (§2's `f(true)[0]` row) and `anon_member_call_decl` (§3's
direct-index row). Both read SILENT on a `kali` built from `cb63f9909` and
REFUSES at `4898f3994`. Both read REFUSES at the baseline `068b29950`
(rows added to `baseline-anon.tsv` by the final review, measured with
`run.sh` on a `kali` built from `068b29950`). There are 0 SILENT rows among the 18 `anon_*` probes (a set that does not
cover §1's or §7's shapes),
and no other probe moved between `cb63f9909` and `4898f3994`.
