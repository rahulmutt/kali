# Defects the unresolved-member-call project measured and did NOT fix

**Filed** 2026-10-03 by the **unresolved-member-call** project
(`docs/superpowers/specs/2026-10-03-unresolved-member-call-design.md`), on the
convention `literal-array-mutators-discovered-defects.md` uses: a project that
measures more than it fixes writes down what it left, so the silence is not
read as absence.

**Oracle:** `node v26.10.0`.
**Measured at:** `06655cc74` (branch `unresolved-member-call`), on
`target/debug/kali` built from that commit (`cargo build -p kali_cli`, the
`dev` profile). The gate landed at `e1920c32c`; `45d7dd46c` added the
enclosing-function-scope lookup (spec A-4). Probe rows come from
`tools/array-return-probes/run.sh`; a probe's baseline column is
`tools/array-return-probes/baseline-unres.tsv`, measured at the spec's
baseline `6f042548f`. The baseline `node` column carries machine-specific
stack traces, so diffs use `cut -f1,2,5`.

**Register:** no entry of `kali-silent-miscompile-register.md` moved lane, so
the register is not edited. `cargo test -p kali_cli --test cases -- oracle/`
was re-run at `06655cc74`: 177 passed, 0 failed. The two oracle cases whose
output changed (`oracle/tier2::r19t_tostring_method_form_module_scope` and
`..._in_function`) are still `FAIL_CLOSED`. Only the E5506 text moved, and
their rationales were updated in Task 7.

---

## §1. The `check` / `run` gap (spec §1.1)

`kali run` refuses every row below with E5506 (exit 1). `kali check` exits 0
on each, at HEAD and at the baseline. The type layer mirrors the refusal only
for a `const` object literal or a program-class instance whose member set
lacks the name (spec §3.3); the five mirrored rows (`objlit`, `strkey`,
`inmain`, `inst`, `inst_extends`) have `check` exit 1 and are not listed. Each
program is `tools/array-return-probes/probes/unres_<name>.js`; the node column
is node's `TypeError` message or its printed value, and the baseline column is
what `kali run` printed at `6f042548f` (exit 0).

| probe | program | node | baseline `run` | HEAD `run` | HEAD `check` exit |
|---|---|---|---|---|---|
| `alias` | `const o={k:1}; const p=o; console.log(p.zork());` | `TypeError: p.zork is not a function` | `0` | E5506 | 0 |
| `arr` | `const a=[1,2]; console.log(a.zork());` | `TypeError: a.zork is not a function` | `0` | E5506 | 0 |
| `call_pop` | `const a=[1,2,3]; a.pop.call(a); console.log(a.length);` | `2` | `3` | E5506 | 0 |
| `call_push` | `const a=[1,2,3]; a.push.call(a, 4); console.log(a.length);` | `4` | `3` | E5506 | 0 |
| `deep` | `const o={a:{b:{}}}; console.log(o.a.b.zork());` | `TypeError: o.a.b.zork is not a function` | `0` | E5506 | 0 |
| `f_call_push` | `function main(){ const a=[1,2,3]; a.push.call(a, 4); console.log(a.length); } main();` | `4` | `3` | E5506 | 0 |
| `fnres` | `function mk(){ return {k:1}; } const o=mk(); console.log(o.zork());` | `TypeError: o.zork is not a function` | `0` | E5506 | 0 |
| `hasown` | `const o={k:1}; console.log(o.hasOwnProperty("k"));` | `true` | `0` | E5506 | 0 |
| `letre` | `let o={k:1}; o={k:2}; console.log(o.zork());` | `TypeError: o.zork is not a function` | `0` | E5506 | 0 |
| `lit_obj` | `console.log(({k:1}).zork());` | `TypeError: {(intermediate value)}.zork is not a function` | `0` | E5506 | 0 |
| `lit_str` | `console.log("abc".zork());` | `TypeError: "abc".zork is not a function` | `0` | E5506 | 0 |
| `num` | `const n=5; console.log(n.zork());` | `TypeError: n.zork is not a function` | `0` | E5506 | 0 |
| `param` | `function g(x){ return x.zork(); } const o={k:1}; console.log(g(o));` | `TypeError: x.zork is not a function` | `0` | E5506 | 0 |
| `proto_pop` | `const a=[1,2,3]; Array.prototype.pop.call(a); console.log(a.length);` | `2` | `3` | E5506 | 0 |
| `proto_push` | `const a=[1,2,3]; Array.prototype.push.apply(a, [4]); console.log(a.length);` | `4` | `3` | E5506 | 0 |
| `str` | `const s="abc"; console.log(s.zork());` | `TypeError: s.zork is not a function` | `0` | E5506 | 0 |

`kali check` stays quiet on all of them because the mirror needs a `const`
binding whose member set the type layer knows. A parameter, an alias, a
function result, a reassigned `let`, a literal receiver, an array, string or
number receiver, and every `.call` / `.apply` spelling are outside it. A
program that passes `check` and is refused by `run` is a spec §8 twin
disagreement. Not fixed.

## §2. Host-provenance silences (spec §1.1, A-2)

Two programs whose receiver is a host value kali cannot prove host-derived
keep the baseline warn-and-`0` behaviour. Both rows are unchanged from the
baseline (probe diff, §3 below).

| probe | program | node | kali `run` (baseline and HEAD) | `check` exit |
|---|---|---|---|---|
| `unres_ok_host_alias` | `const t=globalThis.performance; console.log(typeof t.now());` | `number` | `0`, exit 0 | 0 |
| `unres_ok_import` | `import {m} from "./unres_lib/m.js"; console.log(m.zork());` (`m` is `export const m={k:1}` in `unres_lib/m.js`) | `TypeError: m.zork is not a function` | `0`, exit 0 | 0 |

`unres_ok_host_alias`: the initializer is `globalThis.performance`, which is a
member expression rooted at a name the program does not bind, so the
provenance walk could in principle prove it host, but the call is still
lowered to the fallback and the control keeps the escape hatch the spec chose
(§1.1): a host-rooted chain is not refused. `unres_ok_import`: codegen records
no import names, so an imported binding is a free global under §3.2's first
rule (A-2). The import case is wrong in the other direction: the imported
object is program-owned, and node throws. That is the local-import gap §1.1
discloses. Neither is fixed.

## §3. The probe diff

Task 6 Step 3, verbatim from `task-6-report.md`. Diff 1 is
`baseline-unres.tsv` against HEAD, `cut -f1,2,5`.

```
1,16c1,16
< unres_alias	SILENT	0
< unres_arr	SILENT	0
< unres_call_pop	SILENT	0
< unres_call_push	SILENT	0
< unres_deep	SILENT	0
< unres_f_call_push	SILENT	0
< unres_fnres	SILENT	0
< unres_hasown	SILENT	0
< unres_inmain	SILENT	0
< unres_inst_extends	SILENT	0
< unres_inst	SILENT	0
< unres_letre	SILENT	0
< unres_lit_obj	SILENT	0
< unres_lit_str	SILENT	0
< unres_num	SILENT	0
< unres_objlit	SILENT	0
---
> unres_alias	REFUSES	0
> unres_arr	REFUSES	0
> unres_call_pop	REFUSES	0
> unres_call_push	REFUSES	0
> unres_deep	REFUSES	0
> unres_f_call_push	REFUSES	0
> unres_fnres	REFUSES	0
> unres_hasown	REFUSES	0
> unres_inmain	REFUSES	1
> unres_inst_extends	REFUSES	1
> unres_inst	REFUSES	1
> unres_letre	REFUSES	0
> unres_lit_obj	REFUSES	0
> unres_lit_str	REFUSES	0
> unres_num	REFUSES	0
> unres_objlit	REFUSES	1
29,33c29,33
< unres_param	SILENT	0
< unres_proto_pop	SILENT	0
< unres_proto_push	SILENT	0
< unres_str	SILENT	0
< unres_strkey	SILENT	0
---
> unres_param	REFUSES	0
> unres_proto_pop	REFUSES	0
> unres_proto_push	REFUSES	0
> unres_str	REFUSES	0
> unres_strkey	REFUSES	1
```

Pass conditions: every non-ok `unres_*` row REFUSES; `check` exits 1 on
`objlit`, `strkey`, `inmain`, `inst` and `inst_extends`; no `unres_ok_*` row
moved. Diff 2 (every non-`unres` row, baseline binary `6f042548f` against
HEAD, `cut -f1,2,5`) is empty: no older row moved.

Task 6 also compared every baseline fact in the plan's case table with what it
measured (including `hasown` `true`, `call_push` 3 against 4, `proto_pop` 3
against 2) and found no disagreement.

## §4. The triage table

From Task 7 Step 3. `cargo test --workspace --no-fail-fast` at `8e4ce5e46`
failed two cases and no Rust unit or integration test, including the
package-corpus targets. The stop rules (more than about 50 moved tests, or a
loss outside spec §5.4's classes) were not triggered. No gate fired on a
host-provenance root.

| name | before (`6f042548f`) | after (HEAD) | class |
|---|---|---|---|
| `soundness/abort::abort_signal_static_call_with_shadowed_receiver_not_denied` | `ran:0`, exit 0 (node: `TypeError: AbortSignal.timeout is not a function`, exit 1) | E5506 `calling `.timeout()` … the receiver is a value this program built …`, exit 1 | **wanted**: pinned a silent `0`. The root is `const AbortSignal = 5`. Re-pinned. |
| `soundness/events::event_unknown_receiver_non_capturing_listener_still_builds` | `built`, exit 0 (node: `built`, exit 0) | E5506 `calling `.addEventListener()` …`, exit 1 | **capability loss, §5.4 class 2 (dead code)**: `attach(signal)` is never called. The receiver is a parameter, which §1 item 1 says to refuse. Re-pinned. |
| `oracle/tier2::r19t_tostring_method_form_module_scope` | E5506 deny-set text `calling 'toString' … recognized builtin with no implemented lowering …` x4, exit 1 | E5506 `calling `.toString()` … the receiver is a value this program built …` x4, exit 1 | **rationale only**: verdict still FAIL_CLOSED. Dated note appended. |
| `oracle/tier2::r19t_tostring_method_form_in_function` | same (in `main`) | same | **rationale only**: same update. |
| `soundness/unimplemented_builtins::to_string_method_fails_closed` | E5506 deny-set `toString` text, exit 1 | E5506 new gate text, exit 1 | **rationale only**: dated note that the gate refuses first. Assertions unchanged. |
| `soundness/events::event_target_reassigned_binding_fails_closed` | E5506 EventTarget-binding refusal + `warning[E3100]` for `addEventListener` | the same E5506, and the E3100 warning is now a second E5506 | **stderr only, no edit**: still exit 1, still asserts E5506. |
| `soundness/textcodec::assigned_encoder_construction_fails_closed` | E5506 TextEncoder-construction refusal + E3100 `encode` warning | the same E5506 + a second E5506 for `.encode()` | **stderr only, no edit**: still fails closed. |

After the re-pins: `cargo test --workspace` exit 0 (124 `test result: ok`
lines, 0 FAILED) and `cargo test -p kali_cli --test cases` at
`6222 passed; 0 failed; 2 ignored`.

## §5. Measured capability loss

Exactly one, in spec §5.4 class 2 (dead code):
`soundness/events::event_unknown_receiver_non_capturing_listener_still_builds`.
The program declares a function that calls `.addEventListener()` on its
parameter, and never calls that function. node builds and runs it. kali
before this project built it too; now `run` refuses, because a parameter is
not a proven host value (spec §1 item 1) and the call reaches the fallback.
The case was re-pinned to the refusal and its rationale states the class.

What was swept. Task 7 ran every `cli` and `oracle` step of every case file
(6,224 trials, with matrix and constants expanded) with HEAD's binary. For
each trial whose stderr contained "the receiver is a value this program
built", it re-ran the trial with the baseline binary (`6f042548f`). Because
the gate always prints that text, the sweep covers every trial where the gate
fires. 25 trials print it: 18 are this project's own new cases, 2 failed
(the abort case, wanted, and the events case, the loss above) and 5 still
pass with different stderr (rows 3 to 7 of §4's table). The Rust unit and
integration targets, and the package corpus (whose fixtures contain
`signal.addEventListener`), did not move. A capability loss that no case,
fixture or corpus program exercises was not measured.

## §6. Anything else measured and not fixed

1. **`unres_ok_getter` is silent and unchanged.**
   `class G { get f(){ return () => 7; } } const g=new G(); console.log(g.f());`
   prints `0` where node prints `7`, at the baseline and at HEAD (`check`
   exit 0 both times). The spec expected a refusal (spec §5.1 names a
   getter-backed control), but the call reaches a route other than the
   terminal fallback (a getter likely lowers as a method), so the gate never
   sees it. Not fixed.
2. **Misleading refusal text for a parameter receiver.** The message says "the
   receiver is a value this program built", but for a parameter (the events
   case in §5) the program never supplies the value in that shape. The wording
   follows spec §1 item 1 and is pinned by cases, so it was not changed.
3. **Case name.** `event_unknown_receiver_non_capturing_listener_still_builds`
   no longer builds; the name was kept so the trial id stays stable. The
   rationale explains it.
4. **Class heritage with type parameters (Task 2).** `parse_class_heritage`
   takes an `extends` inside type parameters (`class B<T extends Foo> {}`) as
   the base, so `B` is counted host-derived and its instances keep warn+0.
   Fail-open, rare. A fix takes `extends` only at angle depth 0.
5. **Field collector (Task 2).** `previous_kind` is stale after a parsed
   method, and the collector over-collects names from union and function
   types. Over-collecting only makes `check` quieter.
6. **`const K = class Foo extends X {}` (Task 3).** It is recorded as `Foo`
   only, not as `K`. After the Task 5 fix, instances of a `const K = class {}`
   binding are outside the `check` mirror, so `check` is quieter there.
7. **Cycle test (Task 3).** The `host_derived` cycle test asserts only `A`.
8. **Array or object literal root with a host element (Task 4).**
   `[performance,1].zork()` (a 2-element literal) is walked as a computed
   member, counts as host, and stays silent; this is baseline behaviour and a
   gap in §3.1 rule 2. `[performance].zork()` (1 element) is also silent,
   because a 1-element array unwraps to its element in LIR.
9. **Block scope (Task 4).** The declarator walk ignores block scope:
   `{ const o={k:1}; o.zork(); } const o=globalThis.performance;` stays silent
   at the baseline's `0`, because the module-level `o` is found as host.
10. **`scope.rs` doc comment (Task 5).** The `MemberReceiver` enum sits
    between `/// A lexical scope.` and `pub struct Scope`, so the doc comment
    reads as attached to the enum. Cosmetic.
11. **Redundant guard (Task 5).** `is_program_class` in `known_member_set`
    (`member.rs:512`) is redundant. Harmless.
12. **Check-control rationales (Task 6).** Some `check` control cases quote
    run-time output in their rationales. Cosmetic.
13. **Baseline TSV (Task 0).** `baseline-unres.tsv`'s node column holds
    machine-specific stack traces, like the older baselines. Diffs use
    `cut -f1,2,5`.
14. **Duplicated class name (ruling R6, spec A-5).** A program that declares
    one class name twice, one host-derived, loses its host-method calls to an
    E5506 refusal on `run` (fail-closed), and `check` stays quiet.
15. **Task 3 and Task 4 test discipline.** Task 3's tests were written with
    the code and no RED run was captured; they pass and pin the behaviour.
