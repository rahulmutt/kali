# Defects the unresolved-member-read project measured and did NOT fix

**Filed** 2026-10-07 by the **unresolved-member-read** project
(`docs/superpowers/specs/2026-10-06-unresolved-member-read-design.md`). It
follows the convention of the other `*-discovered-defects.md` files: a project
that measures more than it fixes writes down what it left, so that the
silence is not read as absence.

**Oracle:** `node v26.10.0`, run as `env -u FORCE_COLOR node`.
**Measured at:** `06090515d` (branch `unresolved-member-read`), on
`target/debug/kali` built from that commit (`cargo build -p kali_cli`, `dev`
profile). The baseline is `28c97e4ed`. Probe rows come from
`tools/array-return-probes/run.sh`, and their baseline column is
`tools/array-return-probes/baseline-umr.tsv`. A row marked *hand* was run
directly with both binaries' outputs read side by side. It is not pinned by
any case.

**What the project did.** A member read that reaches codegen's generic
member-read fallback, or a plain `=` store that reaches the final binary
fallback, is refused with E5506 when the receiver chain is rooted at
something the program built. A host-rooted receiver keeps warn+0. Spec §3.8
records four amendments from execution:
- A-5: comma expressions are excluded from the gate;
- A-6: the `check` mirror defers to more specific refusals;
- A-7: `typeof o.z` is refused under `run`;
- A-8: R-29 measures BOTH_REJECT.

**Register:** §0.2 changed four rows:
- **R-21:** `r21f` moved to FAIL_CLOSED; six lanes are still SILENT.
- **R-25:** `r25l` moved to FAIL_CLOSED. The entry leaves the SILENT filter
  but is not fixed.
- **R-29:** retired at BOTH_REJECT.
- **R-27:** did not move (§1.1).

The ranking's §6 ELEVENTH amendment records the regeneration.

---

## §1. Residue: still silent, on purpose or out of reach

### §1.1 R-27 and the `(0, x)` comma family (spec A-5, human partner's ruling)

The read gate refused comma expressions (A-1) until the full `cases` run
showed it breaking correct programs that use the `(0, x)` indirection idiom
(92 trials across four files). The human partner ruled on 2026-10-07 that
comma/sequence expressions are excluded from the gate. Every comma expression
whose **value is read** therefore still evaluates to the placeholder.

| program | node | kali `run` (*hand*, `06090515d`) | `check` exit |
|---|---|---|---|
| R-27's repro: `let a = (1, 2); … let b = (bump(), 7); …` | `a=2`, `b=7`, `n=1` | `a=0`, `b=0`, `n=1`, exit 0 | 0 |
| ``const n="x"; const s=(0, `./${n}`); console.log(s);`` | `./x` | **`2`**, exit 0 | 0 |
| `let a = (console.log("x"), 7); console.log("a=" + a);` | `x`, `a=7` | `x`, `a=0`, exit 0 | 0 |
| `const o={a:1}; const w=(0,o); console.log(Object.hasOwn(w,"a"));` (control) | `true` | `true`, exit 0 | 0 |

- The template-literal row prints `2`, not `0`. The comma placeholder is a
  string-typed read here, so the wrong value is not even the familiar zero.
  It was not measured at the baseline `28c97e4ed`.
- `browser/template_literal_dynamic_import_harness` is unaffected because it
  only passes the value to `import()`.
- The `r27` oracle cases still assert `silent` (green at `06090515d`).
- `soundness/unresolved_member_read.toml` pins the `b=0` row WRONG ON PURPOSE.
  It also pins the `(0, o)` control as matching node.

A fix must tell a stored-and-forwarded sequence value apart from one that
is observed, or lower the sequence's last operand. Neither was attempted.

### §1.2 A member read rooted at an index expression: `t[0].v` is a silent `0`

The gate classifies a receiver by its chain root. An index expression
(`t[0]`) is not one of the roots in spec §3.1's table, so it keeps warn+0,
even when the element is an object the program built.

| program | node | kali `run` (*hand*) | `check` exit |
|---|---|---|---|
| `function taint(v){return {v:v};} function main(){ const c = 2; const t = new Array(1); t[0]=taint(c+5); console.log(t[0].v); } main();` | `7` | `0`, exit 0 | 0 |

The spec expected `misc/arena_reclamation_runtime_sandboxed::function_scratch_is_reclaimed`
to move through this shape. It did not. The fixture was changed anyway so
that it no longer reads the field, because its subject is reclamation. This
is a gap in the gate's coverage, not a ruling. Closing it means giving the
§3.1 table an index-expression row that follows the array's element
provenance.

### §1.3 R-60 / `Object.fromEntries` and host-object reads (spec §5)

- `const o = Object.fromEntries([["a", 1]]); console.log(o.a);`: node `1`,
  kali `0` at exit 0, `check` 0 (*hand*). The initializer resolves to the
  free global `Object`, which is host, so the read keeps warn+0. R-60's §0.2
  row and the `property_key_identity` from-entries pins did not move.
- `const t=globalThis.performance; console.log("v="+t.zork);`: node
  `v=undefined`, kali `v=0` at exit 0 (*hand*). Every read off a genuine
  host object, or a value a host call returned, is out of scope (spec §1).

### §1.4 The `check` / `run` gap

`kali check` mirrors the read refusal only for an absent field on a `const`
object literal or program-class binding, and only where no more specific
refusal applies (spec §3.4, A-6). Every row below is refused by `run` with
E5506 and exits 0 under `check`. The probe rows are the `umr_` probes
(`check` column of `run.sh`).

| shape | example | `run` | `check` |
|---|---|---|---|
| member on a call result | `umr_c8`, `umr_mkcall` (`mk().a`), `umr_idcall` (`id(x).a`) | E5506 read | 0 |
| read through a parameter | `umr_c9`, `umr_e1` | E5506 read | 0 |
| computed / bracket read on a local from a parameter or a user call | `umr_m6`, `umr_q8` | E5506 read | 0 |
| identifier store to a `const` (R-29) | `umr_const_store` | E5506 store | 0 |
| array spread (R-25) | `umr_spread` | E5506 neutral | 0 |
| `typeof` operand (A-7) | `umr_typeof`: `const o={a:1}; console.log(typeof o.z);` | E8001 warning, then E5506 read | 0 |
| **numeric name on a `const` object literal** (restored by A-6) | `const o={a:1}; console.log(o[0]);` (*hand*) | E5506 `no lane proves this receiver is an array` | 0 |
| **`.length` on a `const` object literal** (restored by A-6) | `const o={a:1}; console.log(o.length);` (*hand*) | E5506 `` `.length` is unavailable … `` | 0 |

- **The last two rows were restored by A-6, not newly opened.** `check`
  exited 0 on them at the baseline. For a few commits on this branch it
  refused them through the mirror. It now exits 0 again, because the mirror
  skips names that have their own codegen floor. No case row pins this
  restored gap.
- Mirrored rows (`check` exits 1): `umr_absent`, `umr_absent_bracket`,
  `umr_absent_nullish`, `umr_absent_optional`, and `o.version` / `o.pid` on
  a `const` literal (their codegen arms do not refuse, so the mirror keeps
  them).
- A program that `check` accepts and `run` refuses is a spec §8 twin
  disagreement. Not fixed.

---

## §2. Capability loss: `o.z ?? d` on a `const` object literal (spec A-4)

| program | node | kali at `28c97e4ed` | kali at `06090515d` | `check` |
|---|---|---|---|---|
| `const o={a:1}; console.log(o.z ?? 5);` (`umr_absent_nullish`) | `5` | `5` (CORRECT) | E5506 read, exit 1 | 1 |

At the baseline this program printed node's value only by accident: kali
stores `undefined` and `0` alike, so the placeholder read as nullish. The
read reaches the fallback with a program-built root, so it now refuses under
both commands. This is a correct program that is now refused. The same shape
on a parameter already refused at the baseline (`unknown field 'z' on
fixed-shape object`). Real `undefined` (register R-21, G4) would restore it.

---

## §3. No longer residue: `typeof` on a program-built value (spec A-7)

Spec A-3 listed `typeof o.z` as residue that stays silent. Measured, it does
not: the `typeof` arm evaluates its operand first, and the operand's member
read reaches the read gate. `const o={a:1}; console.log(typeof o.z);` now
gives `warning[E8001] unsupported unary operator 'typeof'` and then E5506
`reading `.z` …` at exit 1 (node prints `undefined`). This was accepted by
controller ruling and pinned in `soundness/unresolved_member_read.toml`. It
is fail-closed, not fixed, and `check` still exits 0 (§1.4).

---

## §4. Moves found after Task 7, outside the spec §6.1 table (NOT re-pinned)

Task 8 found these. Task 7's runs covered the `cases` target and
`kali_cli --lib`. `cargo test -j 8 -p kali_cli --tests --no-fail-fast`,
run under the watchdog at `06090515d`, fails **5 tests in 3 hand-written
targets**. All 5 fail on this project's new E5506 messages. The spec's stop
rule covers them, so none was re-pinned or edited.

| target::test | program (essence) | was | now |
|---|---|---|---|
| `imperative_core_runtime::unknown_field_read_is_fold_first_until_materialized` | `const p = { x: 1.0 }; console.log(p.y);` | pins kali's silent `"0\n"` (node prints `undefined`) | E5506 ``reading `.y` …`` under `run` (and under `check`, via the mirror) |
| `runtime_smoke::build::build_rejects_function_declaration_export_aliases_for_library_artifact_in_all_input_classes` and the `json_build_…` twin | `export function main(input) { return 1; } export { main as alias };` under `kali build --lib` | builds | E5506 ``reading `.alias` …``: **a valid library no longer builds** |
| `runtime_smoke::misc::optimization_benchmark_suite_tracks_compile_time_size_and_speed` | the `object-enumeration-delete-reinsert` benchmark: `const literal = {1: 4, 2: 2, b: 1}; delete literal.b; literal.b = 3; …` | builds in every mode | E5506 ``assigning to `.b` …`` |
| `inprocess::benchmark_execution::every_benchmark_fixture_runs_and_agrees_with_node` | the same benchmark, whose KNOWN-BROKEN entry pins `--fast` printing node's `15` | `--fast` builds and agrees with node | `--fast` refuses with the same store E5506 |

- The first row is the intended kind of move: a pinned silent `0` becomes a
  refusal. It is also the corpus program that left the blast-radius accept
  set (ranking §6 ELEVENTH amendment).
- The other two are **correct programs that now refuse**: an export alias,
  and a store after `delete`. In both, the gate's "no lowering" premise
  looks wrong, because the store or read was lowered (or harmless) before.
  These need a ruling before the branch can be called green.
- **RESOLVED 2026-10-07 (human partner's ruling, spec A-9).** The four
  export-alias and delete-reinsert tests pass unchanged after two narrowings
  (`d3ec2bf13`): codegen no longer emits export specifiers, and a store to a
  module `const` plain-data object literal that nothing else names is
  exempt. The `imperative_core_runtime` test is re-pinned to the E5506 read
  refusal (`5798ae62f`). `kali_cli --tests` is green (70/70 targets).

---

## §5. Other items recorded during execution

- **A class-rewrite error drops every held read refusal (spec A-6).**
  `compile.rs` adds the mirror's held class-instance refusals only when
  `rewrite_class_instances` reports no error. An error for one class drops
  the held refusals for every other receiver. The program is still refused
  (fail-closed), but it shows fewer diagnostics than it has defects.
- **The store gate's free-global keep branch is unreachable in practice.**
  `zz = 3` in sloppy code is refused earlier (E3100, undeclared name), so
  the identifier-host branch of spec A-2 has no reachable test.
- **`read_mirror_skipped_members` is not cleared** in
  `resolve_statements_at_path` beside its sibling resets
  (`crates/kali_types/src/resolve/mod.rs`). This is latent: it would fail
  open only if a context were reused.
- **The parameter-rooted nested read has no case of its own.**
  `soundness/unresolved_member_read.toml`'s `nested.js` is refused by an
  earlier gate (object literal as a call argument), not the read gate.
- **`oracle/tier3.toml`'s header prose** still says R-29 "did NOT move",
  which contradicts its re-pinned rows.
- **The §6.1 argv-index row was stale.**
  `object/computed_member_static_name::the_argv_index_lane_disagrees_…`
  already refused through the array-return backstop (`process.argv` is a
  host root) and did not move. The spec's table is corrected (§6.1, struck
  through).
