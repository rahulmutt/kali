# Defects the captured-bindings project measured and did NOT fix

**Filed** 2026-10-05 by the **captured-bindings** project
(`docs/superpowers/specs/2026-10-05-captured-bindings-design.md`), on the
convention `block-scoping-discovered-defects.md` uses: a project that measures
more than it fixes writes down what it left, so the silence is not read as
absence. Task 6 wrote the phase-1 triage (§1.1), §2, §3 and §5. Task 12 added
the phase-2 triage (§1.2), the phase-2 audit (§2.1), the new §3 rows and
§5.4–§5.11. Task 13 fills §4.

**Oracle:** `node v26.10.0`, always run as `env -u FORCE_COLOR node` (spec A-2.7).
**Measured at:** HEAD `b57e79e97` (branch `captured-bindings`), on
`target/debug/kali` built from that commit (the `dev` profile). The baseline
binary was built from `2ddf18c66` in a separate worktree. Probe rows come from
`tools/array-return-probes/probes/*.js` (runner
`tools/array-return-probes/run.sh`, every prefix). The baseline column of a
`cb_*` probe is `tools/array-return-probes/baseline-cb.tsv`. For a probe whose
verdict is OTHER (E4201), only the verdict column is compared, because the
function index in that message shifts.

**Phase 2 (Task 12)** was measured at HEAD `c7d7f22f0` against two older
binaries, each built in its own worktree: the baseline `2ddf18c66` and phase 1
`d640fbf26`, the last commit before phase 2. The phase-1 binary reproduces
Task 6's probe sweep row for row.

**What ships (phase 1).** Nothing new runs. A closure capture that the capture
lane cannot lower is refused with E5506 (`e5::FEATURE_UNAVAILABLE`). Before, it
read a zero placeholder or dropped a store. The message comes from
`kali_common::captured_binding_unavailable_message`, with one of three reasons:
- `its value type has no closure cell`
- `` `<name>` is two or more closures away``
- `` `<name>` is a parameter of `<owner>` ``

Codegen refuses under `run` (spec §3.1). The `build/capture_refusals` pass
mirrors that refusal under `check` and `run`, except for the residue in §3
(spec §3.4, A-2.6). No flag, schema or maturity row changes.

---

## §1. The triage tables

### §1.1 Phase 1 (Task 6)

At `b57e79e97`, `cargo test --workspace --no-fail-fast` failed 12 trials, all in
`kali_cli --test cases` (`soundness/`). It also failed one `kali_runtime` unit
test, and that one is not a mover:
`execute::execute_tests::host_env::runtime_reports_current_working_directory`
writes the tempdir path into a 64-byte buffer. It fails whenever `TMPDIR` is
longer than that, and passes with the default `TMPDIR`. The probe sweep moved
22 verdicts, all `cb_*`. Another 4 probes kept their verdict and changed only
their message or `check` exit.

| class | trials |
|---|---|
| wanted (silent wrong value became a refusal) | 20 (1 case, 19 probes) |
| other (explained per row) | 18 (11 cases, 7 probes) |
| capability loss in the sweep | 0 (but see §2) |
| **total** | **38** (under the stop rule's 50) |

Re-pin rules:
- **wanted:** the case asserts the refusal, and its rationale gets a dated `RE-PINNED 2026-10-05` paragraph quoting node, the baseline and HEAD.
- **other (already refused):** the needle moves to HEAD's message. The capturer's `__kali_fn_N` name is left out of the needle. The paragraph is the same kind, and the case name is kept.

#### Cases

| name | baseline (`2ddf18c66`) | phase 1 (HEAD) | class | node |
|---|---|---|---|---|
| `soundness/events::deferred_capture_of_bound_set_placeholder_tripwire` | exit 0, `sync=0` / `cb=0` | exit 1, E5506 `… captures `s` …: its value type has no closure cell` | **wanted**: the tripwire fired, and the case now asserts the refusal | `sync=3` / `cb=3` |
| `soundness/bitwise_compound::bitwise_compound_fails_closed_on_two_hop_captured_chain` | exit 1, E5506 compound assignment `'&='` | exit 1, E5506 `… captures `x` …: `x` is two or more closures away` | other: `check` now refuses the depth-2 capture first | `2` |
| `soundness/events::deferred_capture_nested_shadow_placeholder_denies` | exit 1, E5506 `a captured local binding …` | exit 1, E5506 `… captures `c` …: value type` | other: front end refuses before the deferred lane; the walk-stop claim still holds | `x=4` |
| `soundness/events::deferred_event_listener_captured_param_fails_closed` | exit 1, E5506 `a captured param binding …` | exit 1, E5506 `` `p` is a parameter of `main` `` | other: front end refuses first | `p=7` |
| `soundness/events::deferred_queuemicrotask_captured_param_fails_closed` | exit 1, E5506 `a captured param binding …` | exit 1, E5506 `` `i` is a parameter of `main` `` | other: front end refuses first | `i=3` |
| `soundness/events::deferred_settimeout_captured_param_fails_closed` | exit 1, E5506 `a captured param binding …` | exit 1, E5506 `` `i` is a parameter of `main` `` | other: front end refuses first | `i=6` |
| `soundness/events::deferred_settimeout_captured_float_fails_closed` | exit 1, E5506 `a captured float binding …` | exit 1, E5506 value type (`a`) | other: front end refuses first | `1.5` |
| `soundness/events::deferred_settimeout_captured_string_fails_closed` | exit 1, E5506 `a captured string binding …` | exit 1, E5506 value type (`s`) | other: front end refuses first | `hi` |
| `soundness/events::deferred_settimeout_captured_object_read_fails_closed` | exit 1, E5506 `a captured local binding …` | exit 1, E5506 value type (`o`) | other: object-literal initializer refused by `check` (R7) | `x=4` |
| `soundness/events::deferred_settimeout_captured_object_mutation_fails_closed` | exit 1, E5506 `a captured local binding …` | exit 1, E5506 value type (`o`) | other: as above | `x=5` |
| `soundness/events::deferred_settimeout_captured_object_self_contradiction_fails_closed` | exit 1, E5506 `a captured local binding …` | exit 1, E5506 value type (`o`) | other: as above | `sync=4` / `x=4` |
| `soundness/first_class_calls::calling_a_const_bound_arrow_by_name_fails_closed` | exit 1, E5506 `calling 'layer0' … first-class function value …` | exit 1, E5506 `… captures `layer0` …: value type` | other: the capture of an arrow-valued `const` is refused before the call; still fails closed | `lay=5` |

#### Probes

| probe | baseline verdict (kali) / `check` | phase 1 verdict / `check` | class | node |
|---|---|---|---|---|
| cb_a1 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `hi` |
| cb_a2 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `1.5` |
| cb_a3 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `6` |
| cb_a4 | SILENT (`5`) / 0 | REFUSES / 1 | wanted | `6` |
| cb_a6 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `1.5` |
| cb_a7 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `true` |
| cb_d02 | SILENT (`10`) / 0 | REFUSES / 1 | wanted | `15` |
| cb_e5 | SILENT (`5⏎0`) / 0 | REFUSES / **0** | wanted (`check` residue, §3) | `5⏎5` |
| cb_kb2 | SILENT (`1`) / 0 | REFUSES / 1 | wanted | `true` |
| cb_kb3 | SILENT (`1`) / 0 | REFUSES / 1 | wanted | `true` |
| cb_p1 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `5` |
| cb_p2 | SILENT (`0`) / 0 | REFUSES / **0** | wanted (`check` residue, §3) | `5` |
| cb_rw1 | SILENT (`3`) / 0 | REFUSES / 1 | wanted | `12` |
| cb_v4 | SILENT (`1 5`) / 0 | REFUSES / 1 | wanted | `6 5` |
| cb_v7 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `hi` |
| cb_v8 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `5` |
| cb_v9 | SILENT (`0`) / 0 | REFUSES / 1 | wanted | `5` |
| cb_w1 | SILENT (`a`) / 0 | REFUSES / 1 | wanted | `b` |
| cb_w6 | SILENT (`10`) / 0 | REFUSES / 1 | wanted | `15` |
| cb_fa | OTHER (E4201 invalid wasm) / 0 | REFUSES / 1 | other: baseline emitted invalid wasm; phase 1 refuses before emitting it | `3.5` |
| cb_fw | OTHER (E4201) / 0 | REFUSES / 1 | other: as fa | `2.75` |
| cb_w2 | OTHER (E4201) / 0 | REFUSES / 1 | other: as fa | `2.5` |
| cb_v2 | REFUSES (deferred-lane param message) / 0 | REFUSES / 1 | other: message is now the parameter reason; `check` refuses | `5` |
| cb_w3 | REFUSES (compound-assignment message) / 0 | REFUSES / 1 | other: message is now the value-type reason; `check` refuses | `2.5` |
| cb_w4 | REFUSES (update-expression message) / 0 | REFUSES / 1 | other: message is now the parameter reason; `check` refuses | `6` |
| bs_r_depth2 | REFUSES (deferred-lane scalar message) / 0 | REFUSES / 1 | other: message is now `` `a` is two or more closures away``; `check` refuses | `3` |

The `cb_*` rows are pinned by `crates/kali_cli/tests/cases/closure/captured_bindings.toml`
(Tasks 4 and 5).

### §1.2 Phase 2 (Task 12)

At `c7d7f22f0`, `env -u TMPDIR cargo test --workspace --no-fail-fast` failed 4
trials, all in `kali_cli --test cases`: 13020 passed, 4 failed, 1 ignored. The probe
sweep (`tools/array-return-probes/run.sh`) moved 15 verdicts against the phase-1
binary, counting the probe `cb_kb_ret`, which Task 10 added and the phase-1
binary refuses. Another 4 probes kept the REFUSES verdict and changed only their
message. Every move is a `cb_*` probe or one of the 4 failing cases.

| class | trials |
|---|---|
| wanted (refused → node-equal) | 13 (1 case, 12 probes) |
| other (explained per row) | 9 (3 cases, 6 probes) |
| capability loss | 0 |
| **total** | **22** (under the stop rule's 50) |

Byte identity holds. Three programs with nothing captured were built with the
baseline binary and with HEAD, and `cmp` found no difference in any of them:
- `bs_ok_no_shadow`;
- `function f(a){ return a+1; } console.log(f(2));`;
- `function o(){ let c=0; const i=()=>{c+=1;}; i(); return c; } console.log(o());`.

The `closure/captured_bindings.toml` cases (the run and `check` twins of the
`cb_*` probes) are not counted again here. Tasks 7–11 re-pinned them as each
task moved them, with dated rationales (ruling R2). Every one of them matches
its probe row below.

Two probes moved from refused to a wrong value at exit 0. Neither is a
capability loss: each prints what its uncaptured twin prints, and a ruling
covers it.
- `cb_a7`: ruling R11.
- `cb_kb_ret`: R11's principle and spec §1.1's R-34 non-goal (a boolean
  returned from a function renders `1`). The uncaptured twin
  `function f(){ const b=true; return b; } console.log(f());` prints `1` at
  the baseline, phase 1 and HEAD. Task 10 pinned this program to `1`.

#### Cases

| name | phase 1 (`d640fbf26`) | phase 2 (HEAD `c7d7f22f0`) | class | node |
|---|---|---|---|---|
| `soundness/closures::capture_gate_owner_f64_compound_assign_rejects_not_miscompiles`, renamed `…_lowers_not_miscompiles` | exit 1, E5506 `a closure `inc` that captures `c` …: its value type has no closure cell` / `check` 1 | exit 0, `2.5` / `check` 0 | **wanted**: the owner's F64 local now has an F64 cell (A-4), so the capturer's `c += 1` lowers. Re-pinned to node's stdout. | `2.5` |
| `soundness/events::deferred_settimeout_captured_param_fails_closed` | exit 1, E5506 `` `i` is a parameter of `main` `` / `check` 1 | exit 1, E5506 `a captured local binding without closure lowering would read a placeholder …` / `check` 0 | other: the Task 7 rewrite turns the parameter into a local, so the front end no longer refuses it. The deferred-lane allowlist (`Widening::Baseline`) refuses it instead. `check` admits it (R3, §3). The needle is re-pinned to the deferred-lane message. | `i=6` |
| `soundness/events::deferred_queuemicrotask_captured_param_fails_closed` | as above (`i`) | as above | other: as above | `i=3` |
| `soundness/events::deferred_event_listener_captured_param_fails_closed` | as above (`p`) | as above | other: as above | `p=7` |

The controller's brief expected a value-type line after the deferred-lane message in these three cases. There is none: stderr, and the JSON envelope's `errors[]`, carry only the deferred-lane message.

#### Probes

| probe | phase 1 verdict (kali) / `check` | phase 2 verdict (kali) / `check` | class | node |
|---|---|---|---|---|
| cb_a3 | REFUSES (`k` is a parameter of `f`) / 1 | CORRECT (`6`) / 0 | wanted | `6` |
| cb_a4 | REFUSES (parameter) / 1 | CORRECT (`6`) / 0 | wanted | `6` |
| cb_e5 | REFUSES (value type, `n`) / 0 | CORRECT (`5⏎5`) / 0 | wanted; also leaves the §3 residue | `5⏎5` |
| cb_kb2 | REFUSES (value type) / 1 | CORRECT (`true`) / 0 | wanted | `true` |
| cb_kb3 | REFUSES (value type) / 1 | CORRECT (`true`) / 0 | wanted | `true` |
| cb_p1 | REFUSES (parameter) / 1 | CORRECT (`5`) / 0 | wanted | `5` |
| cb_p2 | REFUSES (value type, `n`) / 0 | CORRECT (`5`) / 0 | wanted; also leaves the §3 residue | `5` |
| cb_rw1 | REFUSES (parameter) / 1 | CORRECT (`12`) / 0 | wanted | `12` |
| cb_v4 | REFUSES (parameter) / 1 | CORRECT (`6 5`) / 0 | wanted | `6 5` |
| cb_v8 | REFUSES (parameter) / 1 | CORRECT (`5`) / 0 | wanted | `5` |
| cb_w2 | REFUSES (value type) / 1 | CORRECT (`2.5`) / 0 | wanted | `2.5` |
| cb_w3 | REFUSES (value type) / 1 | CORRECT (`2.5`) / 0 | wanted | `2.5` |
| cb_w4 | REFUSES (parameter) / 1 | CORRECT (`6`) / 0 | wanted | `6` |
| cb_a7 | REFUSES (parameter) / 1 | SILENT (`1`) / 0 | other: a captured boolean parameter renders `1`, as the uncaptured one does (R11, A-2.1) | `true` |
| cb_kb_ret | REFUSES (value type) / 1 | SILENT (`1`) / 0 | other: a returned boolean renders `1`, as the uncaptured one does (R11's principle, R-34) | `true` |
| cb_a6 | REFUSES (parameter) / 1 | REFUSES (value type) / 1 | other: message only; a captured F64 parameter stays refused (A-4) | `1.5` |
| cb_v7 | REFUSES (parameter) / 1 | REFUSES (value type) / 1 | other: message only; a captured String parameter stays refused | `hi` |
| cb_v9 | REFUSES (parameter) / 1 | REFUSES (value type) / 1 | other: message only; the const-bound arrow has no call edge, so no numeric proof (R12) | `5` |
| cb_v2 | REFUSES (parameter) / 1 | REFUSES (deferred-lane message) / **0** | other: the deferred lane refuses the rewritten local; `check` admits it (R3, §3) | `5` |

---

## §2. Measured capability loss

The sweep has none: no trial or probe went from node-correct at the baseline to
refused. The human partner accepted one class found by hand, recorded as spec
**A-3**. Phase 1 refuses at compile time, so a capture that is never read at run
time is still refused. That covers a capture in code that never runs, and a
capture in a closure that is never called. All four programs print node's
output at the baseline, and all are refused at HEAD.

| id | program | node | baseline `run` / `check` | HEAD `run` / `check` |
|---|---|---|---|---|
| cl1 | `function f(k){ const g=()=>k; return g(); } console.log(1);` | `1` | exit 0, `1` / 0 | exit 1, E5506 `` `k` is a parameter of `f` `` / 1 |
| cl2 | `function f(k){ const g=()=>k; return k; } console.log(f(5));` | `5` | exit 0, `5` / 0 | exit 1, E5506 `` `k` is a parameter of `f` `` / 1 |
| cl3 | `function f(){ let x=1.5; const g=()=>x; return g(); } console.log(1);` | `1` | exit 0, `1` / 0 | exit 1, E5506 `its value type has no closure cell` / 1 |
| cl4 | `export default function(k){ const g=()=>k; return g(); } console.log(1);` | `1` | exit 0, `1` / 0 | exit 1, E5506 `` `k` is a parameter of `__kali_fn_0` `` / 1 |

There is a precedent: the deferred lane already refused this way at the
baseline. `function f(k){ setTimeout(()=>console.log(k),0); } console.log(1);`
prints `1` under node, and the baseline exits 1 with E5506. Phase 2 is expected
to lift cl2 (parameter-to-local rewrite plus a numeric proof). cl3 depends on
the F64 cells and is measured in plan Task 9. cl1 and cl4 have no numeric proof,
and the phase-2 triage (plan Task 12) settles them (A-3).

### §2.1 At phase 2 (Task 12)

Re-measured at HEAD `c7d7f22f0`; node, baseline and phase 1 (`d640fbf26`) as
above.

| id | phase 1 `run` / `check` | phase 2 `run` / `check` | outcome |
|---|---|---|---|
| cl1 | exit 1, E5506 `` `k` is a parameter of `f` `` / 1 | exit 1, E5506 `a closure `__kali_fn_0` that captures `k` …: its value type has no closure cell` / 1 | still refused. `f` has no call site, so `k` has no numeric proof. The rewritten local stays unpromoted, and the reason is now the value type. |
| cl2 | exit 1, E5506 `` `k` is a parameter of `f` `` / 1 | exit 0, `5` / 0 | **node-equal**: lifted by the rewrite plus the numeric proof, as A-3 expected |
| cl3 | exit 1, E5506 value type / 1 | exit 1, E5506 value type / 1 | still refused. The closure *reads* the F64 capture, and ruling R14 lowers only capturer writes (A-4). |
| cl4 | exit 1, E5506 `` `k` is a parameter of `__kali_fn_1` `` / 1 | exit 1, E5506 `a closure `__kali_fn_1` that captures `k` …: its value type has no closure cell` / 1 | still refused, as cl1: the anonymous default export is never called |

So A-3's accepted loss shrinks from four programs to three: cl1, cl3 and cl4.
Two more programs printed node's output at the baseline and have been refused
since phase 1. The baseline was right by coincidence: in the first, the
dropped store happened to equal the old value; in the second, nothing reads the
capture. Both stay refused at phase 2, and both are accepted under A-3's
reasoning:

| id | program | node | baseline `run` / `check` | phase 1 `run` / `check` | phase 2 `run` / `check` |
|---|---|---|---|---|---|
| p09 | `function f(){ let x=1.5; const g=()=>{ x = x; }; g(); return x; } console.log(f());` | `1.5` | exit 0, `1.5` / 0 | exit 1, E5506 value type / 1 | exit 1, E5506 value type / 1. The right-hand side reads the captured F64, which R14 refuses. |
| acu | `function f(){ let x=1.5; const C = class { static m(){ const y = x * 2; return 1; } }; console.log(C.m()); return x; } console.log(f());` | `1⏎1.5` | exit 0, `1⏎1.5` / 0 | exit 1, E5506 `a closure `m` that captures `x` …: value type` / 0 | exit 1, same / 0. The unused read of the captured F64 is still a read (R14). `check` admits it, because the frame has no plan key (§3). |

---

## §3. The `check` / `run` gap

`run` refuses each of these programs and `check` admits them (exit 0) at
`b57e79e97`. That is the A-2.6 residue, named. No case asserts the §3.4 property
over these programs.

| shape | program | node | HEAD `run` | HEAD `check` |
|---|---|---|---|---|
| `p2`: a TaggedVal local copied from a parameter | `function f(k){ let n=k; const g=()=>n; return g(); }` (probe `cb_p2`) | `5` | exit 1, E5506 value type (`n`) | 0 |
| `e5`: same shape, and the owner also logs `n` | probe `cb_e5` | `5⏎5` | exit 1, E5506 value type (`n`) | 0 |
| A non-scalar MIR layout reached through a non-literal initializer: a call result | `function makeArr(){ return [1,2,3]; } function f(){ let a=makeArr(); const g=()=>a[1]; return g(); } console.log(f());` | `2` | exit 1, E5506 value type (`a`) | 0 |
| Same, a `new Set()` local | `function f(){ const s=new Set([1,2,3]); const g=()=>s.size; return g(); } console.log(f());` | `3` | exit 1, E5506 value type (`s`) | 0 |
| Same, the Set tripwire | `soundness/events::deferred_capture_of_bound_set_placeholder_tripwire` | `sync=3⏎cb=3` | exit 1, E5506 value type (`s`) | 0 |
| Same, a `new Map()` local | `function f(){ let m=new Map(); m.set("a",2); const g=()=>m.get("a"); return g(); } console.log(f());` | `2` | exit 0, `0` (not refused; see §5.3) | 0 |
| A frame with no plan key: a method of an anonymous class expression | `function f(k){ const C = class { static m(){ return k; } }; return C.m(); } console.log(f(5));` | `5` | exit 1, E5506 `a closure `m` that captures `k` …: `k` is a parameter of `f``. `run` names the method by its bare name. | 0 |
| A frame with no plan key: a method of an anonymous `export default class` | `export default class { static m(){ let s=1.5; const g=()=>s; return g(); } } console.log(1);` | `1` | exit 0, `1`. The method is never called, and a single-file `run` cannot call it. | 0 |

### Residue at phase 2 (Task 11)

Measured over the 33 `tools/array-return-probes/probes/cb_*.js` probes at HEAD. `run` refuses 12 with E5506. `check` refuses 11 of those. The one `check` admits is the residue:

- `cb_v2`: a `setTimeout` callback capturing a rewritten parameter (`function f(k){ setTimeout(()=>console.log(k),0); } f(5);`). `run` refuses it through the deferred lane (its message reads "a captured local binding without closure lowering ...", not "that captures"); `check` exits 0 (R3).
- `cb_two` is refused by both commands with the string-and-number conflict, so it is outside the residue.

The classes of the A-2.6 residue, with no `cb_*` probe in them besides `cb_v2`: user-written parameter copies with a non-scalar or unproven layout; non-scalar layouts from non-literal initializers (call results, `new Map()`/`new Set()`; R7 remainder); arrows over a promoted object whose member access reaches the generic fallback (R13); frames with no plan key (anonymous class-expression methods); write-only deferred F64 captures. The shapes in the table below are the non-`cb` members.

Notes:
- **An anonymous `export default function` is not a gap.** `name_anon_functions` names it, so `check` and `run` both refuse its capture with the same line (cl4 in §2).
- **The loop-record test is a superset of MIR's.** `capture_refusals` treats a loop as one that "may own a record" when it textually contains a deferred-registration call, which is wider than MIR's registration rule. It skips the captures under such a loop and leaves them to block-scoping. So `check` can admit a depth refusal that `run` makes. A capture across a registering loop is already admitted by `check` and refused by `run` with block-scoping's message (`function m(){ let s="x"; for(let i=0;i<2;i++){ setTimeout(()=>console.log(i),0); const g=()=>s; g(); } } m();`: node `0⏎1`; `run` exits 1 with `… captures `s` through a per-iteration record …`; `check` exits 0). That sits beside block-scoping's existing gap (`block-scoping-discovered-defects.md` §3, §7.8). A program that shows the depth case was not found: a loop with a dead `setTimeout` and a depth-2 capture ran node-equal under both commands.

### Added at phase 2 (Task 12)

Measured at HEAD `c7d7f22f0`, with node, the baseline `2ddf18c66` and phase 1 `d640fbf26` for comparison.

| shape | program | node | phase 1 `run` / `check` | HEAD `run` / `check` |
|---|---|---|---|---|
| A deferred callback over a captured parameter, on every registration surface: the three `soundness/events::deferred_{settimeout,queuemicrotask,event_listener}_captured_param_fails_closed` cases, which join `cb_v2` (R3) | e.g. `function main(i){ setTimeout(function(){ console.log("i=" + i); }, 5); } main(6);` | `i=6` | exit 1, E5506 `` `i` is a parameter of `main` `` / 1 | exit 1, E5506 `a captured local binding without closure lowering would read a placeholder …` / **0** |
| A write-only deferred F64 capture (Task 9 minor, beside R3) | `function f(){ let x=1.5; setTimeout(()=>{ x = 2.5; },0); return x; } console.log(f());` | `1.5` | exit 1, E5506 value type / 1 | exit 1, E5506 `a captured float binding without closure lowering …` / **0**. R15 admits a capturer whose every use of the F64 is an assignment statement, and the deferred allowlist (`Widening::Baseline`) refuses F64. The baseline `check` also exited 0. |

**A class field initializer is not treated as a capture** (Task 7 minor). The
rename walk visits field initializers in the enclosing frame, so in
`function f(k){ class C { x = k } }` the `k` is neither a capture nor
rewritten. What the programs do, the same at the baseline, phase 1 and HEAD:
- `function f(k){ class C { x = k } return new C().x; } console.log(f(5));`
  (node `5`): `run` exits 1 with `error[E5506]: constructing class `C` is
  unavailable in the current phase: it reads a variable of an enclosing
  function; kali refuses rather than build an instance whose fields read 0`.
  `check` exits 1. The same holds with a `let k=3` local (node `3`). It fails
  closed, through the class-construction gate rather than the capture lane.
- `function f(k){ class C { x = k } return 1; } console.log(f(5));` prints `1`,
  node-equal, with `check` 0.
- `function f(k){ class C { static x = k } return C.x; } console.log(f(5));`
  prints **`0`** at exit 0 (node `5`), and `check` exits 0. A static field
  initializer reading an enclosing variable is silently wrong. It was already
  wrong at the baseline and is outside the capture lane (§5.11).

**The deferred-lane codegen backstop is only partly exercised.** The front end
pre-empts the codegen deferred-lane allowlist for the String, float and object
deferred cases (`deferred_settimeout_captured_{string,float,object_*}_fails_closed`).
Each is pinned to the front end's value-type message, so none of them reaches
the backstop's messages. At phase 2, the three captured-parameter cases above
and `cb_v2` reach the backstop's generic `a captured local binding …` arm
again, because the rewritten parameter is a local that `check` admits. The
backstop's float arm (`a captured float binding …`) is reached by the
write-only deferred F64 program above, but no case pins that text:
`closure/captured_bindings::a_deferred_callback_over_an_f64_local_stays_refused`
asserts only `E5506`. No case pins the String arm (`a captured string binding …`)
or the parameter arm (`a captured param binding …`). After the Task 7 rewrite,
a captured parameter reaches the deferred lane as a local.

---

## §4. Left for later

Filled by Task 13 from the ledger's rulings. Each item is refused today (or
renders as its uncaptured twin does), and none is claimed by the maturity row.

**Capabilities not built:**
- **A closure's read of a captured F64** (ruling R14, option A). `repr_infer`
  types a free identifier in a nested function by that function's own scalar
  node, which defaults to `I64` and is never joined to the owner's binding
  node. Its `parents` map records only `FunctionDeclaration` nesting, so it
  needs arrow parent edges first. This is the path to full F64 captures: `a2`,
  `a6`, `fa`, `fw`, `cl3` and `p09` all wait on it. Today they are refused
  (E5506, value type).
- **The other A-4 F64 operators.** `%=` (wasm has no f64 remainder), the
  bitwise compound operators and `++` / `--` on a captured F64 are refused in a
  closure (`frem`, `finc`; the bitwise and `--` forms were measured, and no
  case pins them).
- **String captures.** Arena handles cross G5 and the N1 arena-escape family
  (spec §1.1).
- **Object parameters.** A captured Object parameter becomes an Object local
  after the rewrite, and is refused (R9; `ob3`, `ob8`, `ol3`, `ol4`).
- **The depth-2 lowering**, together with block-scoping §4 (R-71).
- **Deferred F64.** The deferred allowlist keeps `Widening::Baseline` (spec
  §1.1, R1).
- **A captured `let` boolean**, and a captured boolean parameter: both render
  `1`, as uncaptured (A-2.1, R11). They need a boolean proof for non-`const`
  bindings.
- **The numeric proof for const-bound arrows** (R12, `v9`). An arrow bound to a
  `const` and called by name has no call edge, so its parameter gets no
  numeric proof and stays refused.
- **Boolean literal inflow should veto the numeric proof** (R11). `a7`
  (`f(true)`) passes the proof and renders `1`. A `kali_types` veto would turn
  that into a refusal, or into `true` once a boolean proof exists.

**`check` / `run` agreement:**
- **Frames with no plan key in `check`.** Methods of an anonymous class
  expression and of an anonymous `export default class` get no plan key, so
  `capture_refusals` admits them, and codegen alone decides (§3).
- **The A-2.6 residue** (§3): deferred callbacks over a rewritten parameter
  (`cb_v2`, R3), user-written parameter copies with a non-scalar or unproven
  layout (`ol2`), non-scalar layouts from non-literal initializers (R7
  remainder), member access that reaches the generic fallback (R13, `c1`), and
  write-only deferred F64 captures.

**Minor items from the ledger:**
- On the owner's side, `x %= 2` and `x |= 1` on a promoted F64 cell now give
  the generic compound-assignment message instead of the specific
  "'%=' on floating-point binding" one (still E5506).
- `do x += 1; while (x < 3);` over a captured F64 is refused by `run`. That is
  an over-refusal: it is correct or refused, and it should lower.
- `b||6` and `b?b:0` on a captured boolean `const` print `1`, as the uncaptured
  forms do.
- The case `closure/captured_bindings::a_captured_boolean_const_returned_still_renders_one`
  (`kb_ret`) pins `1`. It must flip to `true` when R-34 lands.

---

## §5. Pre-existing defects found, not fixed (outside this lane)

§5.1–§5.3 were measured at baseline `2ddf18c66` and at HEAD `b57e79e97`, with
the same result at both. Task 12 re-measured them at phase 1 `d640fbf26` and at
HEAD `c7d7f22f0`, and nothing changed. §5.4–§5.11 come from the ledger (Tasks 5,
8 and 9, rulings R12 and R16). Task 12 measured each at the baseline, phase 1 and
HEAD `c7d7f22f0`. A captured program that phase 1 refused and phase 2 no longer
refuses is listed when its result equals its uncaptured twin's: the capture
now does exactly what the uncaptured binding does (rulings R11 and R16). The
Task 13 maturity row must not claim any of them.

### §5.1 A fractional local captured in a static method of a named class expression is invalid wasm

`const D = class E { static m(){ let s=1.5; const g=()=>s; return g(); } }; console.log(D.m());`
- node prints `1.5`.
- kali `run` exits 1 with `error[E4201]: failed to load WASM module: failed to compile: …`. `check` exits 0.
- The likely cause is that the repr lookup under `E__m` falls back to I64.

### §5.2 A String binding declared in a loop that schedules a callback reads `0`

`function m(){ for(let i=0;i<2;i++){ let s="x"; setTimeout(()=>console.log(i),0); const g=()=>s; console.log(g()); } } m();`
- node prints `x⏎x⏎0⏎1`.
- kali prints `0⏎0⏎0⏎1` at exit 0. `check` exits 0.
- `s` is a String cell in an iteration record. Codegen and `capture_refusals` both leave iteration-owned refs to block-scoping (A-2.5), and nothing refuses this one. That is block-scoping's lane.

### §5.3 A `new Map()` read is a silent `0`, captured or not

`function f(){ let m=new Map(); m.set("a",2); return m.get("a"); } console.log(f());`
- node prints `2`.
- kali prints `0` at exit 0. `check` exits 0.
- The captured variant (§3) does the same. The zero comes from the Map lowering, not from the capture, so phase 1 has nothing to refuse. It is recorded here so that it is not read as covered.

### §5.4 A captured BigInt parameter prints without its `n` (R12)

`function f(k){ const g=()=>k; return g(); } console.log(f(7n));`
- node prints `7n`.
- The baseline printed `0`. Phase 1 refused it (`` `k` is a parameter of `f` ``). HEAD prints `7` at exit 0.
- The uncaptured `function f(k){ return k; } console.log(f(7n));` prints `7` at all three binaries. BigInt rendering is not the capture's defect.

### §5.5 `Math.PI` and `Math.E` read as `0` (R16)

- `console.log(Math.PI); console.log(Math.E);`: node prints `3.141592653589793⏎2.718281828459045`, and kali prints `0⏎0` at all three binaries.
- `function f(){ let x=1.5; x = Math.PI; return x; } console.log(f());` prints `0` at all three binaries.
- The captured `function f(){ let x=1.5; const g=()=>{ x = Math.PI; }; g(); return x; } console.log(f());`, and the same with `Math.E`, behaves as follows:
  - the baseline printed `1.5`, because the store was dropped;
  - phase 1 refused it;
  - HEAD prints `0`, as the uncaptured program does.

### §5.6 Large literals and non-literal right-hand sides (R16 extended)

`repr_infer` does not seed a literal such as `1e20` as a float. A binding
initialized from one is `I64`, so a store or read through it emits invalid wasm
(E4201) or reinterprets saturated integer bits.

| program | node | baseline | phase 1 | HEAD | uncaptured twin (all three binaries) |
|---|---|---|---|---|---|
| `let x=1.5; const g=()=>{ x = 1e20 + 1; }; g(); return x;` | `100000000000000000000` | E4201 | E5506 | E4201 | `let x=1.5; x = 1e20 + 1;` → E4201 |
| `let x=1.5; const g=()=>{ const q = 1e20; x = q; }; g(); return x;` | same | `1.5` | E5506 | E4201 | `const q = 1e20; x = q;` → E4201 |
| `let x=1.5; const g=()=>{ let q = 1e20; x = q; }; g(); return x;` | same | `1.5` | E5506 | `-9223354444668731000` | `let q = 1e20; x = q;` → `-9223354444668731000` |
| `let x=1.5; let q=1e20; const g=()=>{ x = q; }; g(); return x;` | same | `1.5` | E5506 | `-9223354444668731000` | as above |
| `let x=1.5; const q=1e20; const g=()=>{ x = q; }; g(); return x;` | same | `1.5` | E5506 | `-9223354444668731000` | `const q = 1e20; x = q;` → E4201 (the captured `const` takes the `let` path's garbage, not E4201) |
| `let x=1.5; let NaN=3; const g=()=>{ x = NaN; }; g(); return x;` | `3` | `1.5` | E5506 | E4201 | `let NaN=3; x = NaN;` → E4201 |
| `let x=1.5; console.log((x = 2.5, 7)); return x;` (comma expression) | `7⏎2.5` | `2⏎1.5` | `2⏎1.5` | `2⏎1.5` | (this is the uncaptured program) |
| the same comma expression in a static method of an anonymous class expression capturing `x` | `7⏎2.5` | `2⏎1.5` | `2⏎1.5` | `2⏎1.5` | as above |

Each program is wrapped as `function f(){ … } console.log(f());`. When a capturer's
comma expression is used as a value, as in `const g=()=>(x = 2.5, 7)`, it is
refused at phase 1 and at HEAD (E5506, value type, R14). The baseline gave E4201
for it.

### §5.7 An uncaptured `let x=1.5; x=1e20` is invalid wasm

`function f(){ let x=1.5; x = 1e20; return x; } console.log(f());`
- node prints `100000000000000000000`.
- kali gives E4201 at all three binaries, with `check` 0.
- The same `repr_infer` seeding as §5.6. The captured literal store `x = 1e20` is node-equal at HEAD, because a capturer's literal right-hand side is stored as its f64 constant (A-4).

### §5.8 y9: a capture initialized from a large literal

`function f(){ let x=1e20; const g=()=>{ x = 2.5; }; g(); return x; } console.log(f());`
- node prints `2.5`.
- kali gives E4201 at all three binaries, with `check` 0.
- The cell is `I64` because `1e20` is not seeded as float, and the closure's float write hits it.
- The uncaptured `let x=1e20; x = 2.5;` is E4201 at all three binaries too.

### §5.9 The owner's `Math.floor` on an F64 is invalid wasm (R16)

`function f(){ let x=1.5; const g=()=>{ x = 2.5; }; g(); return Math.floor(x); } console.log(f());` (p36)
- node prints `2`.
- The baseline gave E4201, phase 1 refused it (E5506), and HEAD gives E4201.
- The uncaptured `function f(){ let x=1.5; return Math.floor(x); } console.log(f());` (node `1`) is E4201 at all three binaries.

### §5.10 Captured objects read through a call, an argument or a computed key are silent `0`

These are the same at all three binaries, with `check` 0. None involves a captured
parameter, so phase 1 had nothing to refuse. R13's member-fallback refusal covers
only a static member chain rooted at the capture.

| id | program | node | kali |
|---|---|---|---|
| c8 | `function mk(){ return {a:1}; } function f(){ let o=mk(); const g=()=>o; return g().a; } console.log(f());` | `1` | `0` |
| c9 | `function mk(){ return {a:1}; } function show(z){ console.log(z.a); } function f(){ let o=mk(); const g=()=>{ show(o); }; g(); } f();` | `1` | `0` |
| e1 | `function show(z){ return z.n * 10; } function outer(p){ const obj = p; function rd(){ return show(obj); } console.log(rd()); } const x={n:4}; outer(x);` | `40` | `0` |
| m6 | `function f(p){ const o=p; const g=()=>o["a"]; return g(); } const x={a:1}; console.log(f(x));` | `1` | `0` |
| q8 | `function mk(){ return {a:1, s:"xy", arr:[1,2]}; } function f(){ let o=mk(); const g=()=>o["a"]; return g(); } console.log(f());` | `1` | `0` |

### §5.11 A static class field reading an enclosing variable is silent `0`

`function f(k){ class C { static x = k } return C.x; } console.log(f(5));`
- node prints `5`.
- kali prints `0` at exit 0 at all three binaries, with `check` 0.
- Field initializers are walked in the enclosing frame, so this is not a capture (§3). Construction is not involved, so the class-construction gate does not refuse it either.
