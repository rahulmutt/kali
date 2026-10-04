# Defects the literal-array-mutators project measured and did NOT fix

**Filed** 2026-10-03 by the **literal-array-mutators** project
(`docs/superpowers/specs/2026-10-03-literal-array-mutators-design.md`), on the
convention `array-bounds-discovered-defects.md` and its predecessors use: a
project that measures more than it fixes writes down what it left, so the
silence is not read as absence.

**Oracle:** `node v26.10.0`.
**Measured at:** `6e31db3b1` (branch `literal-array-mutators`), on
`target/debug/kali` built from that commit (`cargo build -p kali_cli`, the
`dev` profile). The backstop and the gates landed at `d46ee0dc7`. Probe rows
come from `tools/array-return-probes/run.sh`; a probe's baseline column is
`tools/array-return-probes/baseline-litmut.tsv`, measured at the spec's
baseline `9dc751cf8`. Rows that are not probes were run by hand as
`node P.js` against `kali run P.js` and `kali check P.js`, with the program
text given in full.

**Final-review update.** §1, §7 and §8 were re-measured, and §12-§14 added,
at the final-review fixes (`d8988ac76` and later, on top of `cd7a99637`), on
`target/debug/kali` built from the commit under test. Baseline columns in
those sections come from a `9dc751cf8` build, run by hand. §7 and §8 are
closed; see spec amendment A-10.

**Register:** no entry of `kali-silent-miscompile-register.md` moved lane, so
the register and `blast-radius-ranking.md` are not edited. R-21's `r21o`
oracle cases (`crates/kali_cli/tests/cases/oracle/tier2.toml`) were re-run
(`cargo test -p kali_cli --test cases -- oracle/`, 177 passed) and both are
still `silent`: they read a literal array out of range, which no mutator
gate touches. The register's mutator-on-literal entries are prose rows without
a §0.2 lane or an oracle case: R-39 (`.pop()` returns `0`) and R-40 (`.push` on
a const array literal is ignored). Their repros, `console.log([1,2,3].pop());`
and `const a=[1,2]; a.push(3); console.log(a.length);`, now refuse with
`E5506` on HEAD. They are not §0.2 lanes, so no lane moved, and the register
is left alone.

**Decision A-5, restated.** No human decision was taken on the capability loss
(§5). Array-bounds decision A-5 (accept fail-closed) therefore stands, and the
spec's §7 amendment A-5 records it. Every measured loss that remains is in
spec §5.4's two classes: an effect never observed, or dead code. The final
review found one loss outside them (§7, shadowing), and it was fixed.

---

## §1. The `check` / `run` gap (spec §1.1)

`kali check` exits 0 and `kali run` refuses when a mutator's receiver is
anything the type layer does not classify: it gates only a literal-array name,
an array literal and a plain runtime array. Alias and parameter receivers are
the two probe rows below. The final review measured more shapes; every row
here has `kali check` exit 0 and `kali run` exit 1:

| program | node | `kali run` E5506 text | `9dc751cf8` run |
|---|---|---|---|
| `const o={xs:[1,2,3]}; o.xs.pop(); console.log(o.xs.length);` | `2` | backstop ("could not prove which array") | `3`, exit 0 |
| `const m=[[1,2],[3,4]]; m[0].pop(); console.log(m[0].length);` | `1` | literal `.pop()` | `2`, exit 0 |
| `const o={a:1}; const k=Object.keys(o).sort(); console.log(k[0]);` | `a` | backstop, plus an indexed-read E5506 | exit 1 (E3100 placeholder, indexed-read E5506) |
| `const r=Array.from([1,2,3]).reverse(); console.log(r[0]);` | `3` | literal `.reverse()`, plus an indexed-read E5506 | exit 1 (same as above) |
| `const s="a,b"; const r=s.split(",").reverse(); console.log(r[0]);` | `b` | backstop, plus an indexed-read E5506 | exit 1 (same as above) |
| `const a=[1,2,3]; const p=a.pop; p(); console.log(a.length);` | throws `TypeError` | literal `.pop()` | `3`, exit 0 |
| `class S { constructor(){ this.length = 0; } } const s = new S(); s.length = 5; console.log(s.length);` | `5` | literal `.length` write (§13) | `1`, exit 0 |

`.call` / `.apply` spellings are not in this table: neither layer refuses
them (§12).

The type layer cannot tell which calls
codegen will fail to resolve, so it cannot soundly mirror the backstop. This
follows the precedent of array-bounds followups §4. The probe row `litmut_alias`
reads the alias to its literal in codegen, so `run` refuses with the literal
message. `litmut_param` reaches the backstop text.

`litmut_alias`:

```js
function main(){ const a=[1,2,3]; const b=a; b.pop(); console.log(a.length); } main();
```

node prints `2`. At `9dc751cf8` `run` printed `3` at exit 0. At HEAD `check`
exits 0 and `run` exits 1 with `E5506 calling `.pop()` on a literal array…`,
because codegen traces `b` back to the literal. It does not get the backstop
text.

`litmut_param`:

```js
function g(x){ x.pop(); } function main(){ const a=[1,2,3]; g(a); console.log(a.length); } main();
```

node prints `2`. At `9dc751cf8` `run` exited 1 already, with an `E3100`
placeholder warning for `pop`. At HEAD `check` exits 0 and `run` exits 1 with
`E5506 calling `.pop()` is unavailable in the current phase: kali could not
prove which array the receiver is…` (the backstop text), plus the pre-existing
literal-argument `E5506`.

Not fixed. The gap holds: `run` refuses, and node's output is not produced.

## §2. Warnings are discarded on a successful build

`compile_source_file_uncached` (`kali_cli/src/build/compile.rs:431-517`)
returns diagnostics only on its `Err` path. A build with no error drops every
warning.

```js
var o={k:1}; console.log(o.zork(4));
```

`kali build --output json` reports `"warnings":[]` and `"success":true`, and the
built program prints `0`. node throws `TypeError: o.zork is not a function`.
Measured at HEAD. Not fixed.

## §3. Any unresolved member call evaluates to `0` at exit 0

**Fixed** by the unresolved-member-call project (`docs/superpowers/specs/2026-10-03-unresolved-member-call-design.md`), at `e1920c32c` (the gate; `45d7dd46c` added the enclosing-function-scope lookup, spec A-4): `kali run` refuses with E5506, and `kali check` mirrors the refusal for a `const` object-literal or program-class receiver.

`var o={k:1}; console.log(o.zork(4));` and `const o={k:1}; console.log(o.zork(4));`
both print `0` at exit 0 under `kali run`, and `kali check` exits 0 (node
throws `TypeError`). The terminal placeholder fallback of `emit_call` drops the
arguments and pushes `i64.const 0`. The backstop (spec §3.4) covers only the
nine mutator names (`push`, `pop`, `shift`, `unshift`, `splice`, `reverse`,
`sort`, `fill`, `copyWithin`). Not fixed.

## §4. The probe diff and the triage table (at `d46ee0dc7`)

The probe diff, as recorded when the probes were re-run at `d46ee0dc7`.

#### 4.a. The diff against the four committed baselines

`baseline-litmut` (recorded at `9dc751cf8`) moved as follows:

```
litmut_alias SILENT -> REFUSES          litmut_t_copywithin SILENT -> REFUSES
litmut_closure SILENT -> REFUSES        litmut_t_empty_push SILENT -> REFUSES
litmut_f_copywithin SILENT -> REFUSES   litmut_t_fill SILENT -> REFUSES
litmut_f_fill SILENT -> REFUSES         litmut_t_length SILENT -> REFUSES
litmut_f_length SILENT -> REFUSES       litmut_t_let_push SILENT -> REFUSES
litmut_f_obj_pop SILENT -> REFUSES      litmut_t_pop SILENT -> REFUSES
litmut_f_pop SILENT -> REFUSES          litmut_t_push SILENT -> REFUSES
litmut_f_reverse SILENT -> REFUSES      litmut_t_reverse SILENT -> REFUSES
litmut_f_shift SILENT -> REFUSES        litmut_t_shift SILENT -> REFUSES
litmut_f_sort SILENT -> REFUSES         litmut_t_sort SILENT -> REFUSES
litmut_f_splice SILENT -> REFUSES       litmut_t_splice SILENT -> REFUSES
litmut_f_str_reverse SILENT -> REFUSES  litmut_t_unshift SILENT -> REFUSES
litmut_f_unshift SILENT -> REFUSES      litmut_w_as SILENT -> REFUSES
litmut_nameless SILENT -> REFUSES       litmut_w_optcall SILENT -> REFUSES
litmut_plain_copywithin SILENT -> REFUSES  litmut_w_optmember SILENT -> REFUSES
litmut_plain_optcall SILENT -> REFUSES  litmut_w_paren SILENT -> REFUSES
litmut_plain_reverse SILENT -> REFUSES  litmut_w_strkey SILENT -> REFUSES
litmut_plain_sort SILENT -> REFUSES
```

No row moved other than these 37. All four rows that already refused at
baseline still refuse (`t_var_pop`, `param`, `f_length_compound`, `push_pop`).
No `litmut_ok_*` row moved.

The diffs against `baseline.tsv`, `baseline-anon.tsv` and
`baseline-bounds.tsv` show many moves, such as `bound_length REFUSES->CORRECT` and
`bounds_a1 SILENT->REFUSES`. Those files were recorded before earlier projects
(array-return, anon, array-bounds), so the moves are history and this project
did not cause them. The `grep -F` substring match also pulls `anon_*_passed_on`
rows into the `baseline` diff. §4.b gives the comparison that decides this.

#### 4.b. The comparison that decides it: every probe, `9dc751cf8` binary vs HEAD binary

`run.sh` was run with `KALI=<9dc751cf8 build>` over the same probe set and
diffed columns 1 and 2 against `final.tsv`. The only rows that moved are the
37 `litmut_*` rows listed above. No `baseline`, `anon`, `bounds_*` or `r21`
row moved. Every `CORRECT`/`REFUSES`/`TRAPS` verdict outside `litmut_*` is
identical. The check-column (col 5) diff has no non-`litmut_` line either.
Re-running `baseline-litmut` on the `9dc751cf8` binary reproduces the committed
`baseline-litmut.tsv` verdicts exactly.

#### 4.c. Check-column listing (litmut rows, HEAD)

```
litmut_alias REFUSES check=0
litmut_closure REFUSES check=1
litmut_f_copywithin REFUSES check=1
litmut_f_fill REFUSES check=1
litmut_f_length_compound REFUSES check=1
litmut_f_length REFUSES check=1
litmut_f_obj_pop REFUSES check=1
litmut_f_pop REFUSES check=1
litmut_f_reverse REFUSES check=1
litmut_f_shift REFUSES check=1
litmut_f_sort REFUSES check=1
litmut_f_splice REFUSES check=1
litmut_f_str_reverse REFUSES check=1
litmut_f_unshift REFUSES check=1
litmut_nameless REFUSES check=1
litmut_ok_class_sort CORRECT check=0
litmut_ok_empty_push CORRECT check=0
litmut_ok_growable_join CORRECT check=0
litmut_ok_growable_push CORRECT check=0
litmut_ok_indexof CORRECT check=0
litmut_ok_plain_fill CORRECT check=0
litmut_ok_push_loop CORRECT check=0
litmut_ok_read CORRECT check=0
litmut_param REFUSES check=0
litmut_plain_copywithin REFUSES check=1
litmut_plain_optcall REFUSES check=1
litmut_plain_reverse REFUSES check=1
litmut_plain_sort REFUSES check=1
litmut_push_pop REFUSES check=1
litmut_t_copywithin REFUSES check=1
litmut_t_empty_push REFUSES check=1
litmut_t_fill REFUSES check=1
litmut_t_length REFUSES check=1
litmut_t_let_push REFUSES check=1
litmut_t_pop REFUSES check=1
litmut_t_push REFUSES check=1
litmut_t_reverse REFUSES check=1
litmut_t_shift REFUSES check=1
litmut_t_sort REFUSES check=1
litmut_t_splice REFUSES check=1
litmut_t_unshift REFUSES check=1
litmut_t_var_pop REFUSES check=1
litmut_w_as REFUSES check=1
litmut_w_optcall REFUSES check=1
litmut_w_optmember REFUSES check=1
litmut_w_paren REFUSES check=1
litmut_w_strkey REFUSES check=1
```

All four pass conditions hold:

* Every non-ok `litmut_*` row is REFUSES.
* `check` is non-zero on every one of them except `alias` and `param`, the disclosed §1.1 gap.
* No `litmut_ok_*` row moved.
* No row of the older baselines moved between `9dc751cf8` and HEAD.

On `litmut_w_as`, node's column is a SyntaxError (R5). It is judged on kali alone: run REFUSES, check=1.

The triage table for the same re-run.

#### 4.t. Triage table

The "before" column is the `9dc751cf8` binary, and "after" is HEAD `d46ee0dc7`.

#### 4.ta. Moved tests

| name | before | after | class |
|---|---|---|---|
| growable_array_core[js,ts]::run_rejects_growable_array_mixed_i64_and_string_push… | exit 1, `E5506 elements of `o` in `main` are used as both strings and numbers` | exit 1, `E5506 calling `.push()` on a literal array…` (printed twice) | wanted (refusal kept, message moved per spec A-3); re-pinned |
| growable_array_fail_closed_push_diagnostics::malformed_push_…_object_literal_arg | exit 1, `E5506 growable array `o` in `m` has a `.push` call the growable-array lane does not support…` | exit 1, `E5506 calling `.push()` on a literal array…` | wanted (message move, A-3); re-pinned |
| growable_array_fail_closed_push_diagnostics::malformed_push_…_wrong_arity | same as above | same as above | wanted (message move, A-3); re-pinned |
| set_iteration_runtime::run_supports_set_constructor_iteration_in_{js,ts,jsx,tsx} (4) | exit 1, the self-check throws because the top-level `nullishValues.push` was dropped, giving an `E4000` trap (stdout `1 2 1 …`) | exit 1 at compile time, `E5506 calling `.push()` on a literal array…` | wanted (dropped mutator); re-pinned |
| bitwise_compound::bitwise_compound_fails_closed_on_growable_array_rhs | exit 1, `warning E3100 'push' … zero placeholder` then `E5506 '&=' on a non-integer binding 'n'` | exit 1, `E5506 calling `.push()` on a literal array…` | wanted (dropped mutator); re-pinned |
| structured_clone::named_growable_alias_is_broken_tripwire | exit 0, stdout `1,2,3` (silently wrong) | exit 1, `E5506 calling `.push()` on a literal array…` | wanted (silent wrong value); re-pinned |

Re-pin notes:

* Every re-pin keeps the original rationale. Each appends a dated "Re-pinned
  2026-10-03 by the literal-array-mutators project…" paragraph that says what
  moved and quotes node v26.10.0's output.
* New assertions:
  * Every case now has `exit = "failure"` and `stderr_contains = [<E5506 form already used by the case>, "calling `.push()` on a literal array"]`.
  * `structured_clone` also gains `stdout = ""`.
  * The `stderr_absent = ["appears in a position"]` in the malformed-push cases is kept, and it still holds.
* Node output quoted in each rationale:

  | case | node v26.10.0 output |
  |---|---|
  | mixed push | `2` |
  | `o.push({a:1})` | `1` |
  | `o.push(1,2)` | `2` |
  | bitwise | `1` |
  | structured_clone | `1,2,3,4` |
  | set_iteration | `SyntaxError: missing ) after argument list`, because of nested single quotes. The rationale quotes the isolated push half, which prints `2 1 2`. |

* `misc/growable_array_core.toml` and `misc/set_iteration_runtime.toml` are
  GENERATED by `tools/migration/gen_task19_batch4.py`. They were hand-edited, with
  a dated "HAND-EDITED … generator check above is therefore RED" header
  note, following the `runtime/join.toml` precedent. The generator check was
  already red at `d46ee0dc7` (`GenError: set_iteration_runtime…: 'E4000' is NOT
  on stderr`). That check is a developer gate and no CI job runs it.

#### 4.tb. Moved probes

| name | before (9dc751cf8: verdict, kali out) | after (HEAD) | class |
|---|---|---|---|
| litmut_t_push / t_pop / t_length / t_shift / t_unshift / t_splice / t_reverse / t_sort / t_fill / t_copywithin / t_empty_push / t_let_push | SILENT (wrong value; e.g. t_push `3` vs node `4`) | REFUSES, check=1 | wanted (probe; measurement only, nothing to re-pin) |
| litmut_f_pop / f_length / f_shift / f_unshift / f_splice / f_reverse / f_sort / f_fill / f_copywithin / f_str_reverse / f_obj_pop | SILENT | REFUSES, check=1 | wanted (probe) |
| litmut_w_paren / w_strkey / w_optcall / w_optmember / w_as | SILENT | REFUSES, check=1 | wanted (probe; w_as judged on kali only) |
| litmut_nameless / closure | SILENT | REFUSES, check=1 | wanted (probe) |
| litmut_alias | SILENT `3` vs node `2` | REFUSES (literal message, R9), check=0 | wanted (probe; disclosed §1.1 check gap) |
| litmut_plain_reverse / plain_sort / plain_copywithin / plain_optcall | SILENT | REFUSES, check=1 | wanted (probe; followups §2/§13) |

Probes that did not move and must stay REFUSES:

* `litmut_t_var_pop` still refuses. HEAD shows the literal `.pop()` message.
* `litmut_param` still refuses. HEAD shows the backstop "could not prove which array" message plus the pre-existing literal-argument E5506, with check=0.
* `litmut_f_length_compound` still refuses. HEAD shows the literal `.length` write message.
* `litmut_push_pop` still refuses. HEAD shows the literal `.push()` message, where baseline showed the growable-scan message (A-3).

## §5. Measured capability loss, and decision A-5

No human decision was taken. Array-bounds decision A-5 (accept fail-closed)
is restated and applies. Every measured loss is in spec §5.4's two classes
("effect never observed" and "dead code"). Table, as measured at `d46ee0dc7`:

| program | node | 9dc751cf8 run | 9dc751cf8 check | HEAD run | HEAD check | §5.4 class |
|---|---|---|---|---|---|---|
| `const a=[1,2,3]; a.fill(9); console.log("done");` | `done` | `done`, exit 0 | 0 | exit 1, `E5506 calling `.fill()` on a literal array…` | 1 | effect never observed |
| `const a=[3,1,2]; a.sort(); console.log("done");` | `done` | `done`, exit 0 | 0 | exit 1, `E5506 calling `.sort()` on a literal array…` | 1 | effect never observed |
| `function main(){ const a=[1,2,3]; if (a.length > 5) { a.pop(); } console.log(a[0]); } main();` | `1` | `1`, exit 0 | 0 | exit 1, `E5506 calling `.pop()` on a literal array…` | 1 | dead code |
| `function f(){ const a=[1,2,3]; a.reverse(); } console.log(1);` | `1` | `1`, exit 0 | 0 | exit 1, `E5506 calling `.reverse()` on a literal array…` | 1 | dead code (uncalled function) |

All four losses are in the two §5.4 classes, so A-5 accepts them as fail-closed.

None of the 10 moved tests is a capability loss: every one was a refusal, a
trap or a silent wrong value at baseline.

**Extra sweep.** Beyond the probe set, base and HEAD were compared (node in brackets) on
shapes most likely to show a non-§5.4 loss:

* No loss:
  * `"abc".split("").reverse().join("")` [`cba`] refused at both base and HEAD (pre-existing join refusal).
  * `Object.keys(o).sort()` [`a`], `[...a].sort()` [`1`] and `Array.from([1,2,3]).reverse()` [`3`] refused at both base and HEAD. Base: E3100 placeholder plus an indexed-read E5506. HEAD: literal-mutator E5506.
  * A user class with its own `sort`/`reverse` [`4`/`9`] printed node's output at both.
  * An object property `api.sortIt(1)` [`2`] printed node's output at both.
  * A user class with `push`/`pop`/`fill`, and an object literal with `fill()`/`sort()` methods, refused or errored identically at both, for pre-existing unrelated reasons.
* Browser and host API tests in the workspace (the browser harness targets) all pass.

The 10 moved tests were not capability losses: every one was a refusal, a
trap or a silent wrong value at baseline. This sweep found no row outside
§5.4's two classes. The final review found one (§7), which was fixed.

## §6. Follow-up feature: real mutators on the growable lane and top-level promotion

Not started. This is the real fix for every refusal in this file: let the
growable lane implement `pop`, `shift`, `unshift`, `splice`, `reverse`, `sort`,
`fill`, `copyWithin` and `.length` writes, and let a top-level literal be
promoted to a growable array the way an in-function one already is. It would
turn the §5 refusals and the array-bounds §5 and §12 refusals into node-correct
output. Until it is built, the refusals stand as accepted (fail-closed).

## §7. Shadowing: the literal gate walked past a nearer non-literal binding (closed)

**Closed** by the final-review fix (spec A-10). `resolve_array_literal_binding_name`
walked past a nearer binding of the same name to an outer literal array, so an
inner class instance or parameter named like an outer literal got the
literal-array refusal. That was a capability loss outside spec §5.4: kali at
`9dc751cf8` ran these programs as node does. The walk now stops at the first
scope that binds the name.

```js
const a=[1,2,3]; class Box { sort(){ return 7; } } function main(){ const a=new Box(); console.log(a.sort()); } main(); console.log(a[0]);
```

node prints `7` then `1`. `9dc751cf8`: the same, exit 0. `cd7a99637`: `check`
and `run` exit 1 with the literal `.sort()` E5506. Now: `check` exits 0, `run`
prints `7` then `1`. Pinned by the `an_inner_class_instance_shadowing_*` cases
in `cases/array/literal_array_mutators.toml` and a unit test in
`kali_types/src/resolve/member_tests.rs`.

The example this section first gave, a `Stack` whose `push` does
`this.n=this.n+x`, now passes `check`, and `run` prints `0` (node `3`). That is
§14's class-field defect, not the gate: kali prints `0` for it at `9dc751cf8`
too, and for any method name. The parameter spelling
(`function g(s){ s.push(3); return s.n; } console.log(g(new Stack()));`) is
refused by the pre-existing "passing an array literal to function" E5506,
because LIR cannot tell `new Stack()` from `[x]`.

## §8. `(a.pop)?.()` passed `check` and `run` skipped the call (closed)

**Closed** by the final-review fix (spec A-10). The optional-chain gate in
`resolve_optional_chain` matched the member without `unwrap_transparent`.

```js
const a=[1,2,3]; (a.pop)?.(); console.log(a.length);
```

node prints `2`. `9dc751cf8` and `cd7a99637`: `check` exits 0 and `run` prints
`3` at exit 0. Now: `check` and `run` exit 1 with the literal `.pop()` E5506.
Pinned by `a_parenthesized_optional_call_on_a_literal_refuses_under_{check,run}`.

Still true, and not fixed: the parser drops an optional call's arguments and
then continues the postfix chain, so `a.push?.(4).toString()` and
`a.push?.name` both parse to `Member(OptionalChain(a.push), …)`. The gate cannot
tell them apart, so `a.push?.name` (no call, node `push`) refuses with the
literal `.push()` message under `check` and `run`. Narrowing the gate would let
`a.push?.(4).toString()` through, and codegen has no optional-call gate to
catch it, so the over-refusal stays. Codegen cannot tell an optional call from
a plain `a.push` read either, so the backstop cannot see it.

## §9. Wrong-lane wording

`const o={a:1}; console.log(Object.keys(o).sort());` (node `[ 'a' ]`) and
`Array.from([1,2,3]).reverse()` (node `3` for element 0) refuse at HEAD. The
`Object.keys(o).sort()` case reaches the backstop ("could not prove which array
the receiver is"). `Array.from([1,2,3]).reverse()` says "on a literal array",
though the receiver is the result of a call. Both refused at baseline too, so
only the wording is off. Related to the Task 4 note that
`resolve_literal_aggregate` follows `Array.from` and other folds. Not fixed.

## §10. Coverage lost

As recorded at `d46ee0dc7`:

* **The `&=` refusal for a growable-array RHS.**
  * Uncovered for a named binding, and unreachable on HEAD for every shape tried.
  * A push-built named array cannot reach the `&=` check. At top level, the push refuses at the literal gate. In a function, the `&=` use is a non-growable use, so the push refuses at the literal gate too.
  * Shapes tried, all refused at the `.push()` literal gate:
    * `function f(){ const a=[]; a.push(1); let n=-1; n &= a; } f();`
    * the same with a `for` loop of pushes
    * the same with extra `.length` / `a[0]` reads
    * `n = n & a`
  * Still covered for a growable object-field handle by the new case `soundness/bitwise_compound::bitwise_compound_fails_closed_on_growable_object_field_rhs`: `n &= o.xs` refuses with `E5506 bitwise compound assignment '&=' …`.
* **Set-constructor iteration under `run`.**
  * The `set_iteration_runtime` `run_supports_*` cases (4) now pin only the literal `.push()` refusal.
  * The fixture never reaches the Set-iteration code at `run`, so `run` coverage of `new (null ?? Set)(…)`, `new (false || Set)(…)` and the frozen-alias constructors is gone.
  * The `test_supports_*` siblings in the same file still pass.
* **repr_infer's argument-specific diagnostic "has a `.push` call the growable-array lane does not support".**
  * Unreachable from the CLI on HEAD for every in-function shape tried. A malformed push leaves the binding unpromoted, so the literal gate fires first, and resolve errors return before `repr_infer`'s shape conflicts are reported (A-3).
  * Tried, all giving the literal `.push()` E5506: `o.push(1,2)`, `o.push({a:1})` on `[1]`, `o.push([1])`, `o.push()`, `o.push(...[1])`.
  * The routing is still unit-tested in `crates/kali_types/src/growable_tests.rs` (`reject_kind_routes_position_vs_malformed_push`). No CLI case shows the message.
  * The same applies to the growable scan's "used as both strings and numbers" message for a mixed i64/string push.

## §11. The developer-only migration gate is red, with a new reason

`scripts/test-gate.sh --gates-only`, through
`tools/migration/gen_task19_batch4.py`, is red. It was already red before this
project (`GenError: set_iteration_runtime…: 'E4000' is NOT on stderr`). After
the §4.ta re-pins it is red with a new reason: the "used as both strings and
numbers" claim is absent from the re-pinned mixed-push case. `misc/growable_array_core.toml` and
`misc/set_iteration_runtime.toml` are generated and were hand-edited, with a
dated HAND-EDITED header note, following the `runtime/join.toml` precedent. This
gate is not run by `cargo test` or by CI. Not fixed.

## §12. `.call` / `.apply` spellings of a mutator are silent

**Fixed** by the unresolved-member-call project (`docs/superpowers/specs/2026-10-03-unresolved-member-call-design.md`), at `e1920c32c` (the gate; `45d7dd46c` added the enclosing-function-scope lookup, spec A-4): `kali run` refuses with E5506, and `kali check` mirrors the refusal for a `const` object-literal or program-class receiver.

Neither the type layer nor codegen sees a mutator spelled through `.call` or
`.apply`: the callee's method name is `call` / `apply`, not the mutator.
Measured at the final-review fixes, with the `9dc751cf8` result alongside:

| program | node | HEAD `check` | HEAD `run` | `9dc751cf8` run |
|---|---|---|---|---|
| `const a=[1,2,3]; a.push.call(a, 4); console.log(a.length);` | `4` | exit 0 | `3`, exit 0 | `3`, exit 0 |
| `const a=[1,2,3]; Array.prototype.push.apply(a, [4]); console.log(a.length);` | `4` | exit 0 | `3`, exit 0 | `3`, exit 0 |
| `const a=[1,2,3]; a.pop.call(a); console.log(a.length);` | `2` | exit 0 | `3`, exit 0 | `3`, exit 0 |
| `const a=[1,2,3]; Array.prototype.pop.call(a); console.log(a.length);` | `2` | exit 0 | `3`, exit 0 | `3`, exit 0 |

Silent wrong output, unchanged from baseline. It falls under the claim of spec
§1 ("never silently does nothing") but not under its spelled forms (`a.m()`,
`a["m"]()`, `a.m?.()`). Not fixed.

## §13. A class instance's `.length` write refuses with the literal wording

```js
class S { constructor(){ this.length = 0; } } const s = new S(); s.length = 5; console.log(s.length);
```

node prints `5`. `9dc751cf8`: `run` printed `1` at exit 0. HEAD: `check` exits
0, and `run` exits 1 with `assigning to `.length` of a literal array is
unavailable…`. The codegen `.length` write gate (`emit/literal.rs`) calls
`is_literal_array_value`, which cannot tell `new S()` from `[x]` in LIR, and
it runs before any resolution, unlike the mutator gate, which runs only at the
placeholder fallback. The program was silently wrong at baseline, so this is
not a capability loss; the wording names the wrong lane, and `check` misses it
(§1). The same over-approximation is noted on `is_literal_array_value`. Not
fixed.

## §14. Pre-existing miscompiles met while verifying the final-review fixes

None of these involves a mutator gate. Each gives the same output at
`9dc751cf8` and at HEAD, and each is silent (exit 0, wrong output).

* **A method call on a nameless constructed value evaluates to `0`.**
  **Fixed for classes kali lowers** by the class-instances project
  (`docs/superpowers/specs/2026-10-04-class-instances-design.md`), at `b5dc0c095` (the rewrite),
  case `object/class_instances::14a_matches_node` (prints `2`). `S` is a stateless in-slice
  class, so `new S()` is a factory call before IR and `.push(1)` is a method call on it. A
  different shape, `new C().v` (a field read off a nameless instance), is refused with E5506 (R-15).
  **Fail-closed for every other program class** (ruling R-29): `new X().m()` of a class
  kali does not lower (one with a `static` member, a class expression, a stateless class
  in an `extends` chain) still evaluated to `0` after the rewrite, and is now refused with
  E5506 (`it is outside the class-instances slice and its new result is used in the same
  expression`), cases `object/class_instances::r29_*`.
  `class S { push(v){ return v+1; } } console.log(new S().push(1));` prints `0`
  (node `2`). `foo` and `sort` give `0` the same way, so the method name does
  not matter. Binding the instance first (`const s=new S(); s.push(1)`) prints
  `2`; that spelling is the control case
  `a_user_class_method_named_push_still_matches_node`.
* **A field write in a method is lost.**
  **Fixed** by the class-instances project, at `b5dc0c095`, cases
  `object/class_instances::a_field_write_in_a_method_is_kept` (js) and
  `object/class_instances::ts_14b_matches_node` (ts); both print `3`.
  `class Stack{ constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } function main(){ const s=new Stack(); s.add(3); console.log(s.n); } main();`
  prints `0` (node `3`).
* **A block-scoped shadow in the same function clobbers the outer array.**
  `const a=[1,2,3]; class Box { foo(){ return 7; } } { const a=new Box(); console.log(a.foo()); } console.log(a[0]);`
  prints `7` then `0` (node `7` then `1`). Codegen's `bindings` map is flat
  per function, so the inner `a` replaces the outer one. With a mutator name
  (`sort`) in place of `foo`, `cd7a99637` refused the program through the
  §7 defect; after the §7 fix it prints `7` then `0` again, as at baseline.
  Related to array-bounds followups §11.

Not fixed.
