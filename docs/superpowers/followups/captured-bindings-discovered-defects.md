# Defects the captured-bindings project measured and did NOT fix

**Filed** 2026-10-05 by the **captured-bindings** project
(`docs/superpowers/specs/2026-10-05-captured-bindings-design.md`), on the
convention `block-scoping-discovered-defects.md` uses: a project that measures
more than it fixes writes down what it left, so the silence is not read as
absence. Task 6 wrote the phase-1 triage (§1), §2, §3 and §5. Task 13 fills §4.

**Oracle:** `node v26.10.0`, always run as `env -u FORCE_COLOR node` (spec A-2.7).
**Measured at:** HEAD `b57e79e97` (branch `captured-bindings`), on
`target/debug/kali` built from that commit (the `dev` profile). The baseline
binary was built from `2ddf18c66` in a separate worktree. Probe rows come from
`tools/array-return-probes/probes/*.js` (runner
`tools/array-return-probes/run.sh`, every prefix). The baseline column of a
`cb_*` probe is `tools/array-return-probes/baseline-cb.tsv`. For a probe whose
verdict is OTHER (E4201), only the verdict column is compared, because the
function index in that message shifts.

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

## §1. The phase-1 triage table

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

### Cases

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

### Probes

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

---

## §4. Left for later

(Empty. Task 13 fills it.)

---

## §5. Pre-existing defects found, not fixed (outside this lane)

Each was measured at baseline `2ddf18c66` and at HEAD `b57e79e97`, with the same
result at both.

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
