# R-35 boundary, re-derived after the parser-containment fix

Baseline commit: `5c9bbd051` (branch `r35-switch-lowering`) — includes **Task 2**
(`9db9150c0`, the switch's closing brace is now consumed) and **Task 3**
(`5c9bbd051`, all six required-token positions in `parse_switch_statement` report
`E2000`).
Binary: `./target/debug/kali`, rebuilt from clean at that commit (`cargo build --bin kali`,
`Finished dev profile ... in 29.27s`).
Oracle: `node v26.5.0` (verified with `node --version` → `v26.5.0`).
Measured: 2026-07-28.

Supersedes the boundary recorded in the register's §0.3 R-35 bullet, which was
measured THROUGH the parser leak and is therefore void.

---

## Why the old boundary was void, and what it actually measured

The register recorded: *"a `break` in a case → E5506; a local read in a case → E3100 —
so the silent window is exactly all-return/no-break/no-local."*

Both halves were measured while `parse_switch_statement` was leaking the switch's
closing brace, so every statement after the switch — and, in the leaked reading, the
clause tails themselves — was reparented to module scope. On the fixed parser:

- The `break` half survives, **but for a different reason and with a hole**. `break` in a
  clause of a switch that is *not* inside a loop is honest `E5506` ("break and continue
  are unavailable outside the supported static loop lowering path"), in both scopes. But
  `break` in a clause of a switch that **is** nested in a `for` loop is **not** rejected —
  it compiles into the *enclosing loop's* break and silently truncates the loop (cell 12,
  and `m10` below). The old boundary's "break is a wall" is false in exactly the shape
  where `switch` inside a loop is most idiomatic.
- The "local read → E3100" half is **falsified**. A clause that declares *and reads its
  own* `var`, `let` or `const` compiles fine and is silently wrong (cells 4, 5, 6, both
  scopes, all three declarator kinds byte-identical). `E3100` now appears only in the
  genuinely different shape where a clause-declared `let` is read *after* the switch
  (`m09`), which node accepts and kali rejects.

## Measured mechanism

`switch` has no lowering of its own anywhere in the pipeline.
`crates/kali_hir/src/lowering/statement.rs` allocates a `HirNodeKind::SwitchStmt` whose
children are `[discriminant, clause-block-0, clause-block-1, …]`, each clause block being
`[test?, stmts…]`. `crates/kali_mir/src/lower.rs` maps `SwitchStmt` into the generic
`MirNodeKind::ControlFlow` bucket alongside `IfStmt`, and codegen has no `Switch` arm at
all (`grep -rn "Switch" crates/kali_codegen/src/` returns one hit, an unrelated comment in
`emit/operators.rs:1474`). The generic control-flow arm therefore reads the node as an
**`if`**:

```
switch (d) { A B C D … }   ==>   if (d) { A-body } else { B-body }
```

Three consequences, each measured:

1. **Case tests are never consulted.** The test expression is just the clause block's
   first child and is discarded. `case y:` with `y` a binding (cell 14), duplicate tests
   (cell 11), `default` in a non-final slot (cell 10) — all measure identically to a
   plain two-clause switch.
2. **The discriminant is used as a truthiness condition.** `m01_fn` (five clauses,
   `case 1..4` + `default`) prints `100` for *every* truthy discriminant — `s(1) s(2)
   s(3) s(4) s(9)` all `100` — and `200` for `s(0)`. `m03_mod` (`x = 7`, only
   `case 1:`/`case 2:`, no `default`) prints `c=1` where node prints nothing;
   `m03b_mod` (`x = 0`) prints `c=2` where node prints nothing. A string discriminant is
   a nonzero handle and so is *always* truthy — even `""` (`m11_mod`: kali `hit=100`,
   node `hit=900`).
3. **Clauses beyond the second are never emitted at all.** `m01_fn` never produces `300`,
   `400` or `900` for any input. This makes R-35 a **Tier 1** defect (silently drops
   code), not only a Tier 2 wrong-value defect: with empty-clause grouping at module
   scope the switch produces **no output whatsoever** where node produces output
   (cell 9, module scope: kali stdout is empty, exit 0; node prints `hit=200`).

The **side effects of the wrong clause run** — this is not a value-selection bug.
`s01_mod_20.js` prints `hit=100` where node prints `hit=200`; the first clause's
`console.log` actually executes.

---

## The matrix

16 shapes × 2 scopes = **32 cells**. Every cell was probed with at least three
discriminants (four where a `default` and a coincidentally-correct value both exist), and
the kali answer was checked to *vary or fail to vary* with the discriminant as the
mechanism predicts. `s(10)`-style coincidentally-correct discriminants are marked ✓ and
were never the sole probe for any cell.

Scope key: **fn** = the `switch` is inside a function body, the function is called from
module scope. **mod** = the `switch` is written at module top level (top-level `return` is
legal in both runtimes under CommonJS and was verified to work in both).

Terminator note: the brief mandates `break` only for shape 3. `break` outside a loop is
independently `E5506` (see the control table below), which would have masked nine of the
shapes, so shapes 4, 5, 6, 9, 10, 11, 14, 15, 16 are measured in their **`return`-terminated**
form — the form Task 7's allowlist actually targets. The `break`-terminated variants of
those nine shapes are reported separately as a confound control.

**Fixtures and raw transcripts are tracked in this repository** at
`docs/superpowers/followups/r35-switch-boundary-fixtures/` — every `.js` file named in the
tables below, the five raw transcripts, a `README.md` mapping the naming scheme to the cells,
and `run.sh`, which re-runs the whole set under both runtimes and emits a transcript in the
same format so a re-run can be diffed against the checked-in one. Exit status is captured
**unpiped** (`cmd > out 2> err; x=$?`) on both sides, there and here.

*(Corrected in fix round 2: this line previously cited `scratchpad/r35/f/` and
`scratchpad/r35/results{,2,3}.txt`, an untracked scratch path that no longer existed — the
matrix was unauditable from artifacts while Stage 2 was being sized against it.)*

| # | shape | scope | fixture | kali stdout | kali exit | node stdout | node exit | verdict |
|---|---|---|---|---|---|---|---|---|
| 1 | all clauses `return`, numeric disc, `default` last | fn | `s01_fn.js` | `r20=100 r40=100 r0=200 r10=100` | 0 | `r20=200 r40=900 r0=900 r10=100` | 0 | **SILENT** |
| 1 | " | mod | `s01_mod_{20,40,0,10}.js` | `hit=100` / `hit=100` / `hit=200` / `hit=100`✓ | 0 | `hit=200` / `hit=900` / `hit=900` / `hit=100` | 0 | **SILENT** |
| 2 | all clauses `return`, **string** disc | fn | `s02_fn.js` | `ra=100`✓ `rb=100` `rz=100` | 0 | `ra=100 rb=200 rz=900` | 0 | **SILENT** |
| 2 | " | mod | `s02_mod_{a,b,z}.js` + `m11_mod.js` (`x=""`) | `hit=100`✓ / `hit=100` / `hit=100` / `hit=100` | 0 | `hit=100` / `hit=200` / `hit=900` / `hit=900` | 0 | **SILENT** |
| 3 | all clauses `r = n; break;` over a pre-declared `var` | fn | `s03_fn.js` | *(empty)* — `error[E5506]: break and continue are unavailable outside the supported static loop lowering path` ×2 | 1 | `r20=200 r40=900 r0=900 r10=100` | 0 | **FAIL-CLOSED** |
| 3 | " | mod | `s03_mod_{20,40,0,10}.js` | *(empty)* — same `E5506` ×2 | 1 | `r=200` / `r=900` / `r=900` / `r=100` | 0 | **FAIL-CLOSED** |
| 4 | clause declares **and** reads its own `var` | fn | `s04r_fn.js` | `r20=100 r40=100 r0=200 r10=100` | 0 | `r20=200 r40=900 r0=900 r10=100` | 0 | **SILENT** |
| 4 | " | mod | `s04r_mod_{20,40,0,10}.js` | `hit=100` / `hit=100` / `hit=200` / `hit=100`✓ | 0 | `hit=200` / `hit=900` / `hit=900` / `hit=100` | 0 | **SILENT** |
| 5 | clause declares **and** reads its own `let` | fn | `s05r_fn.js` | `r20=100 r40=100 r0=200 r10=100` | 0 | `r20=200 r40=900 r0=900 r10=100` | 0 | **SILENT** |
| 5 | " | mod | `s05r_mod_{20,40,0,10}.js` | `hit=100` / `hit=100` / `hit=200` / `hit=100`✓ | 0 | `hit=200` / `hit=900` / `hit=900` / `hit=100` | 0 | **SILENT** |
| 6 | clause declares **and** reads its own `const` | fn | `s06r_fn.js` | `r20=100 r40=100 r0=200 r10=100` | 0 | `r20=200 r40=900 r0=900 r10=100` | 0 | **SILENT** |
| 6 | " | mod | `s06r_mod_{20,40,0,10}.js` | `hit=100` / `hit=100` / `hit=200` / `hit=100`✓ | 0 | `hit=200` / `hit=900` / `hit=900` / `hit=100` | 0 | **SILENT** |
| 7 | clause ending in `throw` | fn | `s07_fn_{10,20,40}.js` | `hit=100`✓ / `hit=100` / `hit=100`; stderr `Uncaught t100` + `error[E4000]: runtime trap (unreachable — allocation failure or an unsupported-path guard)` | 1 | `hit=100` / `hit=200` / `hit=900`; `t100`/`t200`/`t900` | 1 | **FL-INTERNAL** |
| 7 | " | mod | `s07_mod_{10,20,40}.js` | `hit=100`✓ / `hit=100` / `hit=100`; same `Uncaught t100` + `E4000` | 1 | `hit=100` / `hit=200` / `hit=900` | 1 | **FL-INTERNAL** |
| 8 | true fallthrough (non-empty clause, no terminator) | fn | `s08_fn.js` | `hit=100` `call20=0` `hit=100` `call40=0` `hit=100` `call10=0` | 0 | `hit=200 hit=900 call20=0 hit=900 call40=0 hit=100 hit=200 hit=900 call10=0` | 0 | **SILENT** |
| 8 | " | mod | `s08_mod_{10,20,40}.js` | `hit=100 done=1` / `hit=100 done=1` / `hit=100 done=1` | 0 | `hit=100 hit=200 hit=900 done=1` / `hit=200 hit=900 done=1` / `hit=900 done=1` | 0 | **SILENT** |
| 9 | empty-clause grouping (`case 1:` then `case 2: return x;`) | fn | `s09r_fn.js` | `r1=1 r2=1 r5=1` | 0 | `r1=200 r2=200 r5=900` | 0 | **SILENT** |
| 9 | " | mod | `s09r_mod_{1,2,5}.js` | **(empty — no output at all)** | 0 | `hit=200` / `hit=200` / `hit=900` | 0 | **SILENT** (drops output) |
| 10 | `default` in a **non-final** position | fn | `s10r_fn.js` | `r20=100 r40=100 r10=100`✓ | 0 | `r20=200 r40=900 r10=100` | 0 | **SILENT** |
| 10 | " | mod | `s10r_mod_{10,20,40}.js` | `hit=100`✓ / `hit=100` / `hit=100` | 0 | `hit=100` / `hit=200` / `hit=900` | 0 | **SILENT** |
| 11 | duplicate case tests | fn | `s11r_fn.js` | `r10=100`✓ `r40=100` `r0=111` | 0 | `r10=100 r40=900 r0=900` | 0 | **SILENT** |
| 11 | " | mod | `s11r_mod_{10,40,0}.js` | `hit=100`✓ / `hit=100` / `hit=111` | 0 | `hit=100` / `hit=900` / `hit=900` | 0 | **SILENT** |
| 12 | switch nested in a `for` loop, clause contains `break` | fn | `s12_fn.js` | `r1=1`✓ `r2=1` `r3=1` | 0 | `r1=1 r2=11 r3=12` | 0 | **SILENT** |
| 12 | " | mod | `s12_mod_{1,2,3}.js`, `m10_mod.js` | `r=1`✓ / `r=1` / `r=1`; `m10`: `r=1` | 0 | `r=1` / `r=11` / `r=12`; `m10`: `r=505` | 0 | **SILENT** |
| 13 | switch nested in a `for` loop, clause contains `continue` | fn | `c13B_fn.js` | *(empty)* — `error[E4003]: CPU fuel budget exhausted: the program ran past the runaway guard` | 1 | `r4=3` | 0 | **FL-INTERNAL** |
| 13 | " | mod | `c13B_mod.js`, `c13E_mod.js` | *(empty)* — same `E4003` | 1 | `r=3` / `r=3` | 0 | **FL-INTERNAL** |
| 14 | non-literal case test (`case y:`, `y` a binding) | fn | `s14r_fn.js` | `r20=100 r40=100 r10=100`✓ | 0 | `r20=200 r40=900 r10=100` | 0 | **SILENT** |
| 14 | " | mod | `s14r_mod_{10,20,40}.js` | `hit=100`✓ / `hit=100` / `hit=100` | 0 | `hit=100` / `hit=200` / `hit=900` | 0 | **SILENT** |
| 15 | float discriminant | fn | `s15r_fn.js` | *(empty)* — `error[E4201]: failed to load WASM module: failed to compile: wasm[0]::function[40]` | 1 | `r25=200 r35=900 r15=100` | 0 | **FL-INTERNAL** |
| 15 | " | mod | `s15r_mod_{15,25,35}.js` | *(empty)* — `error[E4201]: ... wasm[0]::function[22]` | 1 | `r=100` / `r=200` / `r=900` | 0 | **FL-INTERNAL** |
| 16 | boolean discriminant | fn | `s16r_fn.js` | `rtrue=100 rfalse=200` | 0 | `rtrue=100 rfalse=200` | 0 | **CORRECT** |
| 16 | " | mod | `s16r_mod_{true,false}.js` | `hit=100` / `hit=200` | 0 | `hit=100` / `hit=200` | 0 | **CORRECT** |

**Tally (32 cells, baseline `5c9bbd051`): SILENT 22 · FAIL-CLOSED 2 · FL-INTERNAL 6 · CORRECT 2.**
*(Corrected 2026-07-28, fix round 1: cell 13 was first recorded SILENT/exit 0 in both scopes
from a fixture that never reached its own `continue`. See "Cell 13 — corrected" below. The
original tally read 24/2/4/2.)*

### Cell 13 — corrected, and it is **R-09**, not R-35

The first measurement of cell 13 used `s13_{fn,mod}.js`, whose `default` clause was
`default: break;`. Under R-35's `if (d) { clause-1 } else { clause-2 }` lowering that is
`if (i) { continue } else { break }`, so at `i = 0` the **`else` fires and breaks the
enclosing loop** — cell 12's defect — and the loop terminates during its first iteration.
The `continue` is never executed. That fixture was measuring cell 12 a second time.

Two controls prove it, both at `5c9bbd051`:

- `c13G_mod.js` — the same fixture with a `console.log("iter=" + i)` added at the end of the
  loop body: kali prints **no `iter=` line at all** and `r=0`, exit 0; node prints
  `iter=0 iter=2 iter=3 r=3`. Not one iteration of the body completes.
- `c13H_mod.js` — the same fixture with the `continue` clause replaced by `r = r + 10`: kali
  prints the **identical** `r=0`, exit 0 (node `r=14`). The answer does not depend on the
  `continue` being there.

With the `break` removed so the `continue` is actually reachable
(`case 1: continue; default: r = r + 1;`), the real behaviour appears in **both scopes**:

```
$ ./target/debug/kali run c13B_fn.js   → (no stdout)  exit 1
  error[E4003]: CPU fuel budget exhausted: the program ran past the runaway guard
  (default ~60s-equivalent when no sandbox policy is set); grant more compute by
  raising `resources.maxCpuTimeMs` in a --sandbox policy
$ node c13B_fn.js                       → r4=3        exit 0
$ ./target/debug/kali run c13B_mod.js  → (no stdout)  exit 1   (same E4003)
$ node c13B_mod.js                      → r=3         exit 0
```

**The switch is not the mechanism.** The same hang reproduces with no `switch` anywhere:

| fixture | shape | kali | node |
|---|---|---|---|
| `c13C_fn.js` / `c13C_mod.js` | `for (var i…) { if (i === 1) { continue; } else { r = r + 1; } }` — **no switch** | `E4003`, exit 1 | `r4=3` / `r=3`, exit 0 |
| `c13D_mod.js` | `for (var i…) { if (i === 1) continue; r = r + 1; }` — **no switch, no nesting** | `E4003`, exit 1 | `r=3`, exit 0 |
| `c13F_mod.js` | `for (let i=0; i<5; i++) { if (i % 2 === 0) continue; s = s + i; }` | `E4003`, exit 1 | `s=4`, exit 0 |
| `c13E_mod.js` | the switch form with `let` + `i++` instead of `var` + `i = i + 1` | `E4003`, exit 1 | `r=3`, exit 0 |

`c13F_mod.js` is **R-09's own recorded hang repro**. This is R-09 (*`continue` inside a
C-style `for` loop skips the update expression*) — already in the register as a §2 Tier-2
entry with the `E4003` hang form and "scopes affected: both" — surfacing through a switch
clause. **No new register ID was minted for it**; see the register's R-09 entry, whose
evidence this round widens (`var` + `i = i + 1` as well as `let` + `i++`; a `switch` clause
and an `if`/`else` block as well as a bare `if`). Cell 13 tells an allowlist author *nothing
about `switch`*: put a `continue` in a `for` loop by any route and the program hangs.

### Cell 16 is a coincidence, not a capability

The only two CORRECT cells are cell 16, and they are correct only because
`case true:` happens to occupy clause 1 and `case false:` clause 2, which is exactly the
truthiness test the buggy lowering performs. Reversing the clause order
(`m12_fn.js` / `m12_mod_{true,false}.js`: `case false:` first) inverts the answer —
kali `rtrue=200 rfalse=100`, node `rtrue=100 rfalse=200`, exit 0 both — i.e. **SILENT**.
The same coincidence explains `m13_fn.js` (`case 1:` + `default`, discriminants `1` and
`0`): kali `r1=100 r0=900`, matching node exactly. No cell in this matrix rests on such a
value alone.

### Confound control: `break`-terminated variants of the nine re-formed shapes

Every one of these is `error[E5506]: break and continue are unavailable outside the
supported static loop lowering path; use a supported loop form or the later compatibility
path`, exit 1, empty stdout, in **both** scopes — 18 further cells, all **FAIL-CLOSED**:

| shapes | fn fixture | mod fixtures |
|---|---|---|
| 4, 5, 6 (own `var`/`let`/`const`) | `s04_fn.js`, `s05_fn.js`, `s06_fn.js` | `s0{4,5,6}_mod_{20,40,0,10}.js` |
| 9 (empty-clause grouping) | `s09_fn.js` | `s09_mod_{1,2,5}.js` |
| 10 (`default` non-final) | `s10_fn.js` | `s10_mod_{10,20,40}.js` |
| 11 (duplicate tests) | `s11_fn.js` | `s11_mod_{10,40,0}.js` |
| 14 (non-literal test) | `s14_fn.js` | `s14_mod_{10,20,40}.js` |
| 15 (float disc) | `s15_fn.js` | `s15_mod_{15,25,35}.js` |
| 16 (boolean disc) | `s16_fn.js` | `s16_mod_{true,false}.js` |

The guard fires at *compile* time on the `break` alone, before anything about the shape's
own axis can matter — which is why these variants cannot be used to measure `let`/`const`,
grouping, `default` position, duplicate tests, non-literal tests, floats or booleans.
Note the guard's message names *loops*: it is the loop-lowering guard catching a `break`
with no enclosing loop frame, not a switch-aware check.

### Supplementary probes

| probe | question | kali | node | note |
|---|---|---|---|---|
| `m01_fn.js` | are clauses 3+ emitted? | `r1=100 r2=100 r3=100 r4=100 r9=100 r0=200`, exit 0 | `r1=100 r2=200 r3=300 r4=400 r9=900 r0=900`, exit 0 | `300`/`400`/`900` are unreachable for **every** input — clauses beyond the second are never emitted |
| `m02b_mod.js` | 4 clauses, `return`-terminated, `x=3` | `c=1`, exit 0 | `c=3`, exit 0 | wrong clause's side effect runs |
| `m03_mod.js` / `m03b_mod.js` | 2 clauses, no `default`, `x=7` / `x=0` | `c=1` / `c=2`, exit 0 | *(nothing)* / *(nothing)*, exit 0 | a clause runs where node runs none |
| `m04_mod.js` | single clause `case 1:`, `x=0` | `after=1`, exit 0 | `after=1`, exit 0 | correct — the empty `else` |
| `m05_fn.js` | `throw` in a **non-selected** clause | `r10=100`✓ `r40=100`, exit 0 | `r10=100 r40=900`, exit 0 | a `throw` clause is *not* a barrier: it silently never fires |
| `m08_fn.js` | `let v` in a clause shadowing an outer `let v` | `r10=100 r0=1`, **exit 0** | `r10=100` then **`ReferenceError: Cannot access 'v' before initialization`**, **exit 1** | kali invents an answer where node throws on the switch block's TDZ |
| `m08b_fn.js` | same, but the clause body is explicitly braced | `r10=100 r0=1`, exit 0 | `r10=100 r0=1`, exit 0 | correct — the hazard is specifically the *un-braced* clause sharing the switch block scope |
| `m09_fn.js` | read a clause-declared `let` **after** the switch | *(empty)* — `error[E3100]: undefined identifier 'q'`, exit 1 | `r10=1`, exit 0 | kali **over**-rejects here; this, not "a local read", is where E3100 now lives |
| `m10_mod.js` | is a clause `break` inside a loop the *loop's* break? | `r=1`, exit 0 | `r=505`, exit 0 | yes — the loop is truncated at iteration 0 and the post-switch `r = r + 100` never runs |
| `m14_fn.js` | float discriminant with **integer** case tests | *(empty)* — `error[E4201]: ... wasm[0]::function[40]`, exit 1 | `r25=900 r0=900`, exit 0 | the E4201 tracks the *discriminant's* float-ness, not the case tests' |

### Instrument compliance

Every fixture obeys the register's instrument rules: exactly **one** argument per
`console.log`, always literal-rooted concatenation (`"r=" + r`), and **no default
parameters anywhere** (R-01 truncation). No `console.log` receives an array, an object or
a bare boolean, so R-30/R-31 rendering defects cannot contaminate a cell. Exit status was
recorded **unpiped** for both sides of every run.

---

## Consequences for the Stage 2 allowlist

Read with the fact that Task 6 makes `emit_switch` **deny-by-default with `E5506`**, so
any shape Tasks 7-10 do not admit becomes FAIL-CLOSED rather than staying SILENT.

- **Rule 4 (clause terminators)** — `return`-terminated clauses are exactly where the
  silent lane lives: cells 1, 2, 4, 5, 6, 9, 10, 11, 14 are all SILENT in both scopes in
  their `return` form. Task 7's choice to admit `return` first is aimed at the live
  damage. Deferring `break` to Task 9 is *not* cost-free the way the non-loop `E5506`
  suggests: cell 12 (and `m10_mod`) shows `break` in a clause of a loop-nested switch is
  SILENT and truncates the enclosing loop. **`continue` is a different hazard and Task 9
  must not size it against `break`**: cell 13 is `E4003` fuel exhaustion (FL-INTERNAL) in
  both scopes, it is **R-09, not R-35** — the identical hang reproduces with no `switch`
  anywhere (`c13C`, `c13D`, `c13F`) — and no `switch` allowlist can fix it. Deferring empty-clause
  grouping to Task 10 leaves the single worst cell (9/mod, which drops all output)
  longest — though Task 6 converts it to `E5506` in the meantime. `throw` (cell 7) is
  FL-INTERNAL where it fires and SILENT where it does not (`m05_fn`), so it is a real
  hazard, not a quiet one; deferring it is defensible on cost, not on safety.
- **Rule 5 (`let`/`const` in a clause body)** — the denial's *premise* is confirmed:
  `m08_fn` shows an un-braced `let` in a clause shadowing an outer binding gives kali
  exit 0 with a fabricated answer where node raises a TDZ `ReferenceError`, exactly R-10's
  unmodeled block scoping. But the denial is *load-bearing only because Task 6 denies by
  default*: cells 5 and 6 are SILENT today and measure byte-identical to cell 4 (`var`),
  so `let`/`const` in a clause body does **not** fail closed on its own. Rule 5 must be
  reached through the `E5506` deny path, never left to the status quo.
- Two shapes the allowlist should be sized against explicitly: **cell 16** is the only
  CORRECT pair in the matrix and Rule 2 (numeric-literal case tests) would move it to
  `E5506` — an acceptable trade given `m12` shows its correctness is an ordering
  coincidence, but it should be a deliberate, recorded regression. **Cell 15** is
  FL-INTERNAL `E4201` at module load; if Rule 2 admits float literals, Task 7's fixture
  set needs a float cell or the E4201 will simply move rather than close.
