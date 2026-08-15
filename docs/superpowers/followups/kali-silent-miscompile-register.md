# Kali silent-miscompile register (canonical)

Branch `soundness-batch1-pra`. Oracle: `node v26.5.0`. Binary: `./target/debug/kali`.

This document consolidates four independent adversarial sweeps into one deduplicated,
severity-ranked register of **silent miscompiles** — cases where kali exits 0, emits no
diagnostic, and produces an answer that differs from node.

Source registers (superseded by this file; retained for their full probe logs):

| sweep | surface | raw defects | file |
|---|---|---|---|
| A | output / rendering / coercion | 13 | `.superpowers/sdd/sweep-a-output-coercion.md` |
| B | operators / control flow | 8 | `.superpowers/sdd/sweep-b-operators-controlflow.md` |
| C | functions / calls / scope | 8 | `.superpowers/sdd/sweep-c-functions-calls.md` |
| D | objects / arrays / strings | 13 | `.superpowers/sdd/sweep-d-data-structures.md` |

Repro files: `/tmp/claude-1000/-workspace/3882ed8e-3d1f-4182-91f6-6b9ace78f5f9/scratchpad/sweep-{a,b,c,d}/`
and `.../scratchpad/consolidate/` (controller re-verification).

**Verification status vocabulary** used on every entry:

- `CONFIRMED-BY-CONTROLLER` — independently re-run by the consolidating controller on a
  freshly built binary, transcript reproduced in this file.
- `sweep-only` — one sweep's transcript, both scopes probed, not re-run here.
- `sweep-only-top-level-only` — one sweep's transcript, **module scope only**. Given this
  repo's history of scope-dependent defects, the "scopes affected" line on these is a
  hypothesis, not a finding.

---

## 0. RE-DERIVATION 2026-07-24 (baseline `62d786e74`) — READ THIS FIRST

The register below was written against branch `soundness-batch1-pra` and is now
**substantially stale**. Every entry was re-verified on a freshly-built binary
(`./target/debug/kali`) at **commit `62d786e74`** against `node v26.5.0` on 2026-07-24 by
four independent surface sweeps (A output/coercion, B operators/control-flow,
C functions/calls, D data-structures). **Where a per-entry headline below conflicts
with this section, this section wins** — except where this section is itself superseded by a
later measurement, which is now the case for **R-11** (closed `28f18b3ff`) and for **R-35**
(boundary re-derived 2026-07-28 on `5c9bbd051`; the §0.3 bullet below carries the corrected
text and `r35-switch-boundary-rederived.md` carries the matrix). Full per-surface probe logs:
`scratchpad/resweep/sweep-{a,b,c,d}-rederived.md`.

**`62d786e74` is a named baseline, not "main".** This section was originally headed "HEAD
`62d786e74`, main"; `main` has since moved (`28f18b3ff` R-11, `372a3f440` this section). Applying
this document's own lesson to itself — *a number without a named baseline is not a measurement* —
the baseline is stated as the commit, and the section is **superseded for R-11 by `28f18b3ff`**,
which closed it. Rows and bullets re-measured after `62d786e74` name the commit they were
measured on (`372a3f440`) inline.

**Regeneration 2026-08-15.** §0.2 is no longer a hand-maintained table. It is
generated from the oracle cases under `crates/kali_cli/tests/cases/oracle/`,
which assert a derived verdict class and therefore fail when an entry moves.
Where this section's prose and §0.2 disagree about a class, **§0.2 wins** — it
is measured and this prose is not.

### 0.1 Headline

The register's own top priority — the **Group-1 evidence-corrupting defects
(R-01, R-04, R-07)** — are all resolved, and the entire **functions/calls/scope
surface** (R-02, R-05) has moved from silent-miscompile to honest **fail-closed
E5506**. That validates the "allowlist at the call-lowering choke" interim fix the
register recommended for cluster G2. ~~As a result the silent-miscompile frontier has
**moved**: the highest-blast-radius *silent* defect on the current binary is no
longer any original Tier-1 entry but **R-35 (`switch` selects the wrong clause)**,
newly found this re-derivation.~~ **STRUCK 2026-07-29 (`64438bf0ef`) — R-35 is closed and
this sentence is false; see the second amendment below for the re-derived frontier.**

**Amendment 2026-07-28 (`5c9bbd051`, branch `r35-switch-lowering`).** Probing R-35 uncovered
a *parser* defect strictly worse than R-35 itself: **R-49**, in which
`parse_switch_statement` never consumed the switch's closing brace and every statement after
a `switch` was reparented to module scope and executed at module load. R-49 is **CLOSED**
(`9db9150c0`). R-35's recorded boundary was measured through R-49 and is **void**; the true
boundary is a 32-cell both-scopes matrix (22 SILENT / 2 FAIL-CLOSED / 6 FL-INTERNAL /
2 CORRECT) in `docs/superpowers/followups/r35-switch-boundary-rederived.md`, and R-35 is
**Tier 1**, not Tier 2 — clauses beyond the second are never emitted at all. ~~R-35 itself
remains **SILENT and open**; Stage 2 of that branch is the lowering fix.~~ **STRUCK
2026-07-29 (`64438bf0ef`) — false; see the second amendment immediately below.**

**Amendment 2026-07-29 (`64438bf0ef`, branch `r35-switch-lowering`) — R-35 IS CLOSED, AND
THE FRONTIER CLAIM IS RE-DERIVED.** This amendment supersedes both struck sentences above
and takes this section's own precedence over everything below it.

**1. R-35 is CLOSED.** Stage 2 landed: `switch` is lowered by an **allowlist** at
`crates/kali_codegen/src/emit/switch.rs`'s `switch_plan`, which admits only shapes it can
*prove* and returns `Err(reason)` otherwise, surfacing as honest `E5506`. The admitted set
matches `node v26.5.0` byte-for-byte; **no silent lane remains in `switch`**. The
authoritative boundary — admitted set, fourteen-item residual, two accepted regressions,
three standing couplings — is **§7.11**, not §0.2's row and not §0.3's bullet. R-35 is the
second Tier-1 entry closed by this project, after R-49 (`9db9150c0`).

**2. The frontier does NOT pass to R-51, R-52 or R-53 — and it is UNRANKED.** The
temptation on closing a headline entry is to promote the newest finding into the vacancy.
That would be a confident wrong answer, so it is not given here. Measured at
`64438bf0ef` against `node v26.5.0`:

- The three new close-out entries all reproduce, all silent, all exit 0 — but each is a
  **narrow** construct with a correct sibling, which is the opposite of high blast radius.
  **R-51** needs the optional-call form specifically (`s?.(7)` → `w=0`/`c=0`, node
  `w=7`/`c=1`; the plain `s(7)` control is correct on both). **R-52** needs a `for` with an
  omitted clause *and* a present later one (`for (var i = 0; ;)` → kali prints only `s=0`,
  node prints six `iter=` lines and `s=15`). **R-53**'s silent lane is narrower than its
  own headline says — see point 3.
- Meanwhile **four pre-existing SILENT entries reproduce at this same HEAD on far more
  ordinary constructs**, each re-measured here rather than carried over:
  - **R-13** — `var o = {a:1,b:2}; var k = "a"; o[k]` → kali `read=0`, node `read=1`, exit 0.
  - **R-31** — `console.log(o)` → kali `0`, node `{ a: 1 }`; `console.log(a)` → kali `0`,
    node `[ 1, 2, 3 ]`, exit 0. (Note the array lane printed `0`, not the length §0.2's row
    records — one more reason the table below needs re-measuring, not re-reading.)
  - **R-10** — `var x=1; { let x=2; } x` → kali `outer=2`, node `outer=1`, exit 0.
  - **R-14** — `function f(){return [1,2,3];} f()[0]` → kali `e0=0`, node `e0=1`, exit 0.

  A computed property read, `console.log` of an object, a block-scoped `let`, and indexing a
  returned array are each vastly more common in real JS than `s?.(x)` or `for (init; ;)`.
  So the frontier falls **back into the pre-existing SILENT set**, not onto the new entries.

  **Which member of that set is highest is not established by any measurement in this
  document, and this section will not assert one.** Two reasons, both structural: (a)
  "blast radius" has never been given an operational definition here — it has been used
  informally to mean *tier × construct frequency*, and no frequency model over real JS has
  ever been built for this project; (b) the ~26 SILENT verdicts in §0.2's table are dated
  **2026-07-24 / `62d786e74`** and have **not** been re-measured wholesale since, and at
  least one has already moved — R-21's absent-field lane (`o.b` for undeclared `b`) now
  **fails closed** `E5506 unknown field 'b' on fixed-shape object` at `64438bf0ef`, where
  §0.2 still records it SILENT. Ranking a stale table is not a measurement.

  **What would settle it, in order:** (i) write down an operational definition of blast
  radius (proposed: tier × a counted frequency of the triggering construct over a fixed JS
  corpus, so the ranking is reproducible rather than argued); (ii) re-run the four surface
  sweeps at the current HEAD to refresh every §0.2 verdict, since entries have demonstrably
  moved in both directions; (iii) *then* rank. Until (i)-(iii) are done the honest statement
  is the one made here: ~~**the frontier is unranked, and it is somewhere in
  {R-10, R-13, R-14, R-31, and the rest of the pre-existing SILENT set} — not in
  {R-51, R-52, R-53}.**~~ **STRUCK 2026-08-15 — (i), (ii) and (iii) are all done; see the
  third amendment below. The exclusion of {R-51, R-52, R-53} did NOT survive the
  measurement, and it failed on ALL THREE names: R-51 and R-52 are in the reachable axis's
  band 1 on tier, and R-53 is in it too, through its cluster G4. None of the three is there
  on frequency — all three count 0 reachable. The amendment below gives the three routes.**

**3. R-53 is WIDER than its §0.2 headline: `let` is affected, not only `var`.** Measured at
`64438bf0ef`, switch-free: `for (let v of [1,2,3,4]) { console.log("iter=" + v); s = s + v; }`
→ kali `iter=0` ×4 and `s=0`, node `iter=1/2/3/4` and `s=10`, **exit 0 both sides**. The
`const` form is correct on the identical fixture (`s=10`). The silent lane is also bounded on
the other axis: over a **binding** iterable rather than an array literal
(`var a=[1,2,3]; for (const v of a)`) kali fails closed with an honest `E5506`. So R-53's
silent surface is precisely *for-of over an **array literal** with a **`var` or `let`**
loop variable*. This matters beyond bookkeeping: it is why this branch's own fail-closed
message names `const` explicitly (see §7.11's note) — recommending a bare "`for...of`" would
have routed users out of an honest denial and into R-53.

**Amendment 2026-08-15 — THE FRONTIER IS NOW RANKED.** The 2026-07-29 amendment's point 2
said the frontier was unranked and named the three things that would settle it: an
operational definition of blast radius, a re-measurement of every §0.2 verdict, and then a
ranking. All three are done.

- The definition is `docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md` §3:
  the pair `(tier, reachable_frequency)`, where frequency is counted only over corpus
  programs kali accepts.
- §0.2 was regenerated 2026-08-15 from live cases under
  `crates/kali_cli/tests/cases/oracle/`, measured at `4cfa218814`.
- The ranking is `docs/superpowers/followups/blast-radius-ranking.md`, added 2026-08-15.

~~the frontier is unranked, and it is somewhere in {R-10, R-13, R-14, R-31, and the rest of
the pre-existing SILENT set}~~ — superseded. The measured band 1 is in the ranking document;
that document, not this paragraph, is authoritative.

**Two corrections this amendment owes the paragraph above, both of them against it.**

1. **The "not in {R-51, R-52, R-53}" half did not survive — on all three names.** On the
   ranking's reachable axis, **R-51** (cluster G2) and **R-52** are both in band 1, not
   because they are frequent — both count **0** reachable and both are
   `present-but-unreachable` — but because they are the only Tier-1 clusters left, and no
   Tier-2 cluster dominates a Tier-1 one at any frequency. **R-53** is in band 1 as well, by
   a third route: its cluster **G4** contains R-21, which has no predicate at all, so G4 has
   no frequency and cannot be dominated. R-53's own reachable count is also 0. None of the
   three is on the frontier because it turned out to be common; all three are there because
   a partial order over a thin measurement leaves them uncompared. That is the Pareto
   definition working, and it is exactly the kind of result an argued frontier gets wrong.
2. **Of the four entries this paragraph nominated, three are off the measured frontier.**
   Bands are over *clusters*, so read each nominee through its cluster: R-13 → **G3**, band
   2; R-14 → the escape/provenance-loss pair with R-48, band 3; R-10 → **G7**, band 4. Only
   R-31 is in a band-1 cluster (**G8**), and G8 is there on R-30's 57 and R-23's tier, not on
   R-31's own count of 2. R-13's number in particular does not mean what it looks like: the
   ranking's §3.2 shows only 2 of its 45 reachable sites have the receiver shape R-13's own
   repro describes. The nomination was a reasonable guess and the measurement disagrees with
   it, which is the whole reason the measurement was built.

**Read the ranking's §1.1 before its bands.** `kali check` accepts **1 of the 40** corpus
programs written to do jobs rather than to probe the compiler (2.5%), so 126 of the 127
reachable programs are anchor micro-snippets and every reachable frequency is, in substance,
a frequency over test snippets. That is a finding about kali, not a defect of the corpus.

### 0.2 Current status of every register entry

**Regenerated 2026-08-15.** Every row below is produced from the oracle cases in
`crates/kali_cli/tests/cases/oracle/`, measured at commit `4cfa218814` against
`node v26.7.0`. A row is not prose a reader must re-derive: it is the verdict
a live case asserts, and a change of class is a red test. The prior table was
dated 2026-07-24 / `62d786e74` and had been stale for weeks — see §1 of
`docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md`.

FIXED = kali matches node, exit 0. FAIL-CLOSED = honest Enn nonzero (acceptable —
not a silent defect). SILENT = exit 0, no diagnostic, wrong (the dangerous class).
FL-INTERNAL = nonzero but wrong *kind* (E4201/E4003 internal, not honest E5506).
*(This paragraph is the register's older hyphenated spelling of four of the eight
classes below — `FAIL-CLOSED` is `FAIL_CLOSED` and `FL-INTERNAL` is `FL_INTERNAL`.
It is kept because it is where those four are defined; it is not a complete list,
and the classifier's vocabulary is the one the table uses.)*

Verdict classes are the classifier's, defined in
`crates/kali_blast_radius/src/verdict.rs`: `FIXED`, `SILENT`, `FAIL_CLOSED`,
`FL_INTERNAL`, `ACCEPTS_INVALID`, `BOTH_REJECT`, `TIMEOUT`, `NONDETERMINISTIC`.

**How to read a row.** Rows are ordered by entry number, which is the extraction's
order, not the 2026-07-24 table's. The status column names **lanes**, and each lane
name is the `rNN…` prefix of the cases that assert it: every lane below is measured
by **two** cases, module scope and in-function, and **the two scopes agreed on the
class for every lane of every entry** — there is no entry in this table whose class
depends on scope. Five entries (R-07, R-09, R-13, R-20, R-54) carry an additional
case in `classifier_ground_truth.toml`, which measures the classifier on that
entry's own repro; those cases agree with the tier files. **143 cases back the 41
rows.** The oracle directory holds 147; the other four carry
`register_entry = "GROUND-TRUTH"`, measure the classifier rather than any entry,
and are therefore attributable to no row — `agree.js`, `both_reject.js`,
`hang.js` and `nondeterministic.js`. A reader auditing the mapping should expect
those four to be unattributed, and should treat any *other* unattributed case as
a defect.

| entry | status measured 2026-08-15 at `4cfa218814` | note |
|---|---|---|
| R-01 default param truncates module | **FAIL_CLOSED** (both scopes) | E5506 "a default parameter is not supported", all forms; no truncation. Class unchanged since the 2026-07-24 row — this is the first time a case has held it. kali's stdout is empty where node prints `A`/`B`, so nothing is truncated *and* nothing is printed. |
| R-02 call through fn value → 0 | **FAIL_CLOSED** (both scopes) | every broken lane E5506 (the recommended G2 interim fix); callee never runs, but honestly. Supported set unchanged (direct call, const-arrow/fnlit, IIFE, sibling capture). The refusal is preceded by a `warning[E3100] undefined identifier … lowered through a zero placeholder compatibility fallback` — a warning, not the verdict. |
| R-03 forEach / expr-arrow filter | **FAIL_CLOSED** (both scopes) | E5506 via the first-class-fn-value guard; the diagnostic text is word for word R-02's and R-05's, differing only in the quoted callee. `reduce`/`map` are a different program and are not what this row measures. |
| R-04 console drops later args | **FIXED** (both scopes) | all sinks, both scopes; multi-arg routes booleans through `emit_as_string` correctly. The case measures one cell of R-04's boundary (a `var` reference in the middle position); five further boundary shapes were re-measured by hand at `4cfa218814` and **all agreed with node**, so the entry is fixed, not merely the cell. |
| R-05 object-literal method / `this` → 0 | **FAIL_CLOSED** (both scopes) | ~~`E3100` "undefined identifier 'm'"~~ — at `4cfa218814` the §2 repro refuses with the same `E5506` first-class-callee message R-02 and R-03 produce; fail-loud either way, so the verdict class is unchanged. BUT class-method `this.field` is a different program: it is **R-36**, has no case here, and this row does not speak for it. |
| R-06 var/let composite init | **FIXED** (declarator-init lane `r06a`) / **SILENT** (whole-object reassignment `r06b`, R-06-R2) / **SILENT** (array elements `r06c`, R-06-R3) | objects-half closed PR #26 covers the **declarator initializer only**. Whole-object *reassignment* (`var o={f:1}; o={f:2}; o.f`→`0`, node `2`) is **R-06-R2**, still silent. Arrays-half is **R-06-R3**, still silent; both elements of `var a=[7,9]` read `0`. The three lanes are three programs and get three cases; the entry is not fixed. |
| R-07 `const` is not a binding | **FIXED** (both scopes) | `const` is a real binding now. The two scopes use the register's own two repros (Repro A "classic swap" in-function, Repro B "stale read" at module scope) rather than one repro wrapped twice; a third case in `classifier_ground_truth.toml` pins the FIXED class on Repro A. Under the defect these printed a plausible wrong answer at exit 0, so a regression would classify SILENT and name R-07. |
| R-08 `===`/`!==`/`==`/`!=` half | **FAIL_CLOSED** (`r08eq`, both scopes) | ~~FIXED — conflation cases all correct; null-guard now fail-closed~~ **CHANGED at this regeneration: FIXED → FAIL_CLOSED.** The move is narrower than the class name suggests and must not be read as a regression: the repro's first three comparisons still agree with node, and the whole program now exits 1 only because its **fourth** comparison (a `let`-bound `0` against `null`) is refused with `E5506 operator '===' cannot be decided here`. One refused comparison takes the program's verdict; the three conflation cases are unaffected. |
| R-08 `??` half | **SILENT** (`r08nc`, both scopes) | `let a=0; a??9`→9, and the `var`, parameter and call-return operands all →9/10 against node's `0`. All four operand kinds reproduce digit for digit. Unchanged — this half is untouched by the `===` half's move. |
| R-09 `continue` skips for-update | **SILENT** (skip-ahead form `r09s`) / **FL_INTERNAL** (hang form `r09h`, `E4003` fuel trap) | two lanes of one entry, not a contradiction: the skip-ahead form is silent-wrong (kali `s=13`, node `s=10`, exit 0 both) and the `i%2` form runs away to `E4003`. `E4003` is documented as *internal*, so it is FL_INTERNAL and not an honest denial. The evidence-widening recorded on this row in 2026-07 stands and is not re-measured here: as of `61c2d48ea9` the register records the hang as independent of `let`/`var`, of `%`, and of nesting, reproducing under `do`/`while` and `for…in`, with `while`, `for…of` and a C-style `for` with no update clause the only faithful forms. **R-09 is the owning ID for the switch-clause `continue` hang**; it is *not* an R-35 defect and no `switch` allowlist can fix it. See §2's R-09 entry and `r35-switch-boundary-rederived.md`. |
| R-10 block-scope shadowing | **SILENT** (both scopes) | the inner declaration still aliases the outer binding: kali `r=2`, node `r=1`, exit 0. One of the three frontier candidates, and still silent at `4cfa218814`. |
| R-11 bitwise compound assign | **FIXED** (local-scalar lane, both scopes) | ~~CLOSED 2026-07-25 (`28f18b3ff`)~~ — the class is now stated in the classifier's vocabulary rather than as a project event: **FIXED**. All six operators match (`and=2 or=14 xor=7 shl=24 shr=3 ushr=3`). **This case measures the local-scalar lane only** — the entry's own repro. R-11's guard-bypass shapes (object field, array element, parameter) are different programs with no case here, and as of `61c2d48ea9` §2 records some of them refusing with `E5506`; this row does not speak for them. The `&=`/`+=` relation is **INVERTED** on the object-field lane — as of `61c2d48ea9`, `o.a &= 3` lowers to `2` while `o.a += 1` refuses with `E5506`; see §3's G3 edit. |
| R-12 alias defeats array-store guard | **SILENT** (both scopes) | the store vanishes and the read-back through the alias reports the pre-store value (kali `b0=1`, node `b0=7`). The discriminator is **SCOPE, not declarator kind**, per the 2026-07-25 correction on `372a3f440`; both scopes measure SILENT here because both cases carry the alias. |
| R-13 computed var-key get/set | **SILENT** (read `r13r`) / **SILENT** (write `r13w`), both scopes | read →`v=0` where node reads `2`; write vanishes (kali `dot=2`, node `dot=8`). Two repros, two lanes, one class. A third case in `classifier_ground_truth.toml` pins the SILENT class on the read repro. One of the three frontier candidates, and still silent at `4cfa218814`. |
| R-14 returned array reads zeros | **SILENT** (both scopes) | kali `r=0`, node `r=1`, exit 0. One of the three frontier candidates, and still silent at `4cfa218814`. The "object-return is correct" control that FLIPPED is **R-44**, a different entry with no case here. Arrays are broken even when bound, not only when indexed off the call expression. |
| R-15 `.split()` result | **SILENT** (both scopes) | element-read shape → `len=0` plus a leaked handle (`1=-9223354418898927615`, node `1=b`). The `STATUS 2026-07-20` partial closure added the *runtime* `.split()` fallback to the deny-set; the register's own repro binds a string **literal**, so it reaches the preserved static-ASCII fold lane and the deny-set never sees it. Partial closure, live defect. |
| R-16 per-method string repr leak | **SILENT** (both scopes) | `.slice()`/`.charAt()`/`.toUpperCase()`/`.repeat()` leak the raw handle in concat position (kali `c=-9223354388834156541`, node `c=hel`). The handle's bit pattern is allocation-dependent and differs from the one recorded in 2026-07; the two kali runs of the case agree with each other, so the pair does not rank NONDETERMINISTIC. |
| R-17 string handles escape as ints | **SILENT** (both scopes) | join/element/`Object.keys` concat lanes; both handles match the recorded bit patterns digit for digit and both consumers still leak, so neither lane was closed and neither masks the other. |
| R-18 string literal `&&`/`\|\|` leaks handle | **SILENT** (both scopes) | two leaked handles plus case-3's inverted truthiness, all four lines reproducing exactly as recorded. |
| R-19 `String(x)` / `.toString()` → 0 | **FIXED** (`String()` lane `r19s`) / **FAIL_CLOSED** (`.toString()` lane `r19t`), both scopes | Stage P5 gain: `String()` of a proven scalar/string COMPUTES (var-bound too, un-poisons concat); all four `.toString()` spellings fail closed with `E5506`, one refusal each, so no receiver kind slips past the deny-set. No silent path. **§2's own STATUS line contradicts this row about the `String(x)` lane; HEAD agrees with the row, not with §2** — as of `61c2d48ea9`, §2 is the stale text. |
| R-20 `JSON.stringify` → 0 | **FAIL_CLOSED** (both scopes) | `E5506`; the message names the callee `stringify` rather than `JSON.stringify`, which the register already recorded as cosmetic and which is confirmed here. A second case in `classifier_ground_truth.toml` pins the FAIL_CLOSED class on the same repro. **§2's section TITLE is stale** — as of `61c2d48ea9` it still reads "silently returns 0 for every input" while its own `STATUS 2026-07-20` line and this row say fail-closed; HEAD confirms the STATUS line. The residual aliased-receiver lane (`const j = JSON; j.stringify(o)`) is a different program with no case here. |
| R-21 no `undefined` value | **FAIL_CLOSED** (absent field, `let`/`var` receiver — `r21fl`) / **SILENT** (all seven other lanes: `r21bn`, `r21bu`, `r21c`, `r21v`, `r21a`, `r21f`, `r21o`), both scopes | ~~SILENT, all forms~~ — the absent-field read moved to `E5506 unknown field` by `64438bf0ef`; §0.2 recorded SILENT until this regeneration. **The discriminator is the RECEIVER'S DECLARATOR KIND, and §2's own repro takes the silent side:** `const o={a:1}; "z="+o.z` → `z=0` at exit 0 (`r21f`, SILENT), while the identical program with a `let` or `var` receiver refuses (`r21fl`, FAIL_CLOSED). That is a lane discriminator, not a move of the entry — eight lanes across sixteen cases, and seven of them are still silent (`null`/`undefined` through a binding, concat, void return, `undefined+1`→1, out-of-bounds literal array read →`false`). |
| R-22 `==` cross-type coercion | **SILENT** (both scopes) | `1=="1"`→`false`, node `true`. `"1"==true` fails closed and `1==true`/`null==undefined`/`1==1.0` are correct; those are different programs and are not what this row measures. |
| R-23 `typeof` non-literal | **SILENT** (both scopes) | any binding/expression → `0` — a *number* where node produces `boolean`, not the string `"0"` and not `"undefined"`, which is why `typeof x === "string"` dispatch silently never matches. Literal control correct. |
| R-24 `Object.freeze` no-op | **SILENT** (both scopes) | the write goes through (kali `x=99`, node `x=1`) and `isFrozen`→`0` where node prints `true`. Both halves reproduce. |
| R-25 array spread `[...a]` | **FAIL_CLOSED** (`.length`/index-fold lane `r25i`) / **SILENT** (`console.log` residual `r25l`), both scopes | `b.length`/`b[i]` → `E5506`, one refusal per consumer, behind a `warning[E8001] unsupported unary operator 'spread'`; `console.log([...a])`→`0` (node `[ 1, 2 ]`) is the named residual of the 2026-07-20 partial close and is still silent. |
| R-26 unary `+` on non-numeric string | **SILENT** (both scopes) | `+"abc"`→`5451`, node `NaN` — digit for digit the register's recorded value, and exactly what an unvalidated byte accumulator predicts (49·100 + 50·10 + 51). `+"42"`→42 is a different program. |
| R-27 comma operator → 0 | **SILENT** (both scopes) | value lost (`a=0`, `b=0` against node's `a=2`, `b=7`); the side effect still fires exactly once — both engines agree on `n=1`, so only the value is lost. |
| R-28 `-0` | **SILENT** (reciprocal `r28v`) / **SILENT** (direct log `r28r`), both scopes | `1/-0`→`Infinity` (node `-Infinity`) and `console.log(-0)`→`0` (node `-0`). Two lanes, one class. The "`-0` folds to the integer `0`" mechanism is the register's hypothesis as of `61c2d48ea9`; these cases record the divergence, not the mechanism. |
| R-29 assign to `const` | **ACCEPTS_INVALID** (both scopes) | ~~SILENT (node throws)~~ **RECLASSIFIED at this regeneration: SILENT → ACCEPTS_INVALID. The entry did not move; only the name of its class did.** kali prints `r=1` at exit 0 with no diagnostic; node exits 1 with `TypeError: Assignment to constant variable.` — kali accepting a program node refuses is ACCEPTS_INVALID by definition, and the old row's own parenthetical "(node throws)" ruled out the class the row named. §2's R-54 already files R-29 as "the same class" as R-54, which this table spells ACCEPTS-INVALID. Write still discarded, no const-write guard: the defect is unchanged and unfixed. |
| R-30 booleans render 1/0 in direct log | **SILENT** (`var` binding `r30a`; `const` object field `r30c`) / **FIXED** (`const` scalar `r30b`; concat and template `r30d`), both scopes | narrowed by the R-04 fix and narrowed again by the 2026-07-19 correction: among plain bindings only `var` is still wrong (`console.log(b)`→`1`, node `true`), `const` **object fields** remain wrong, and the concat/template sinks are correct for operands kali can prove. Four lanes, two classes. The two FIXED lanes are the entry's own declared controls — **they do not retire the entry**. |
| R-31 log array→len / object→0 | **SILENT** (direct log `r31a`) / **SILENT** (concat `r31b`), both scopes | direct: array→`2` (its length — a deceptive answer for a 2-element array), object→`0`, against node's `[ 1, 2 ]` and `{ f: 1 }`. Concat: both collapse to `v=0` against node's `v=1,2` and `v=[object Object]`. Two sinks, two lanes, because node renders differently on each. Both silent. |
| R-32 no exponential notation | **SILENT** (past-threshold direct log `r32a`) / **FIXED** (just-inside `r32b`; concat `r32c`), both scopes | `1e21`/`1e100`/`1e-7` render as expanded digits in the direct-log sink where node uses exponential; `1e20`/`1e-6` are correct, pinning the boundary exactly, and the concat path implements the small-number threshold the direct-log path does not. Two independent formatters, and they still disagree. |
| R-33 `console.warn` `[warn]` prefix | **SILENT** (`console.warn` lane `r33a`, observed on **stderr**) / **FIXED** (`console.error` control `r33b`, also stderr), both scopes | ~~SILENT/WARN~~ — the class is `SILENT`; the prefix persists (kali `[warn] hi`, node `hi`, exit 0 both) and `console.error` is correct. **Both lanes are measured on stderr, which is where `console.warn` renders**; an earlier stdout-only reading of this entry measured FIXED by comparing two empty strings, and that FIXED was an artifact of the observed stream, not a fix. The four R-33 cases are the only ones in the oracle directory that set `observe = "stderr"`. The cases compare whole streams; that the difference is *exactly* the `[warn] ` prefix is the hand observation, not something the class alone establishes. |
| R-34 bool user-fn renders 1/0 (concat & multi-arg) | **SILENT** (both scopes) | live; concat AND multi-arg both render `1` where node renders `true` — all three lines of the repro diverge, so both named lanes are still broken. |
| R-47 `for..of` over a `let` array iterates the binding's NAME | **SILENT** (`let` lane `r47l`) / **FAIL_CLOSED** (`var` lane `r47v`) / **FIXED** (`const` lane `r47c`), both scopes | added 2026-07-25, originally measured on `372a3f440`. `let a=[1,2,3]; for (const x of a) log(x)` still prints the single line `a` (node `1 2 3`) — the identifier's own text is the iterand and the trip count follows the identifier's LENGTH. `var` refuses with the `E5506` for-of-iteration message word for word; `const` matches node. **The `const` lane's FIXED is a LANE result the entry itself declares as its control — it does not retire R-47.** |
| R-48 array stored into an `I64` object field reads `0` | **SILENT** (both scopes) | added 2026-07-25, originally measured on `372a3f440`. `o.a=[1,2]; o.a`→`0` (node `[ 1, 2 ]`); the store still vanishes and the slot still reads its zero. |
| R-49 `parse_switch_statement` reparented every post-switch statement to module scope | **FAIL_CLOSED** (by R-35's switch allowlist, not R-49's defect), both scopes | ~~CLOSED 2026-07-28 (`9db9150c0`) — Tier 1, cluster **G1**~~ — the closure stands; what changed is the measured class. **CHANGED at this regeneration: FIXED/CLOSED → FAIL_CLOSED, AND NOT BY THIS ENTRY'S GATE.** The reparenting defect is not back. The decisive repro's discriminant is a parameter, and it is now refused *before execution* by **R-35's switch-lowering allowlist** — ``E5506: this `switch` is not in the supported lowering set (the discriminant is not a proven integer or string)`` — which names R-35's admitted set, not R-49's defect. Verified by hand at `4cfa218814`: the identical program with a locally-bound `var x = 1;` discriminant compiles and prints `g=0`, so **the containment property still holds wherever the switch is admitted**. Two case names in `tier1.toml` promise "containment"; those cases observe a refusal that happens first, and this row is written from the verdict and the rationale, not from the names. |
| R-51 optional call `s?.(x)` returns `0` and never runs the callee | **SILENT** (both scopes) | added 2026-07-29, originally measured on `58234e87c7`. `s?.(7)` → `w=0` (node `w=7`) and a side-effect counter in the callee stays `0` where node reads `1`, so the body still never runs. Exit 0, empty stderr — completely silent. The non-optional control `s(7)` is correct, so the defect is the optional-call route specifically. Carries a **standing coupling warning to R-35's parameter proof** — see §2's R-51 entry and §7.11. **One of only two Tier-1 entries still measuring SILENT at `4cfa218814`; the other is R-52.** |
| R-52 `for`-clause arity misclassification (omitted clauses) | **SILENT** (Repro A `r52a`; Repro B `r52b`) / **FL_INTERNAL** (Repro C `r52c`, `E4003`), both scopes | added 2026-07-29, originally measured on `58234e87c7`. Three labelled repros, three declared severities, three lanes — collapsing them would record one class for an entry the register itself records as carrying three. A: `for (var i = 0; ;)` skips the loop entirely (kali `s=0`; node six `iter=` lines and `s=15`). B: `for (init; ; update)` drops iteration zero and **the sums still agree**, which is why the per-iteration log is load-bearing. C: `for (; test; update)` runs away to `E4003` after ~1.36M lines in ~2.7s, reproducibly (two kali runs compared byte for byte, so the pair does not rank NONDETERMINISTIC). Distinct from R-09, which is about update PLACEMENT, not clause identification. Carries a **standing coupling to `continue_is_faithful`** — see §2's R-52 entry. |
| R-53 `for (var v of […])` — **and `for (let v of […])`** — binds every element to `0` | **SILENT** (`var` loop variable `r53v`; `let` loop variable `r53l`) / **FIXED** (`const` loop variable `r53c`), both scopes | the 2026-07-29 widening holds at `4cfa218814`: `let` is affected as well as `var`, measured on the entry's own separately-dated four-element fixture. In every silent lane **the trip count is correct and only the bound value is lost** (`iter=0` ×3 or ×4, `t=0`/`s=0`, against node's `1..3`/`t=6` and `1..4`/`s=10`). The silent surface remains *for-of over an **array literal** with a **`var` or `let`** loop variable*; over a binding iterable kali refuses. **The `const` lane's FIXED is a LANE result the entry itself declares as its control — it does not retire R-53.** Distinct from **R-47**, which is `for..of` over a `let`-declared array BINDING iterating the binding's NAME; this is the loop VARIABLE's declarator kind over an array LITERAL. Consequence for probe design is unchanged: `for (var v of …)` must not be used as a faithful-loop control. |
| R-54 a second `default` clause is absorbed into the first (node: `SyntaxError`) | **ACCEPTS_INVALID** (both scopes) | added 2026-07-29, originally measured on `58234e87c7`. Both halves still reproduce: kali prints `v=d2` **and** `g=5` at exit 0, so the clauses are still MERGING rather than replacing; node refuses the whole file with `SyntaxError: More than one default clause in switch statement` at exit 1. `g=5` is the load-bearing half — `v=d2` alone would be consistent with replacement. A second case in `classifier_ground_truth.toml` pins the ACCEPTS_INVALID class on the same repro. Only invalid JS is affected. Cluster **G1**, same function as R-49 and independent of it. |

**Two entries a reader may look for and not find.** Neither is a §2 entry, so
neither has an oracle case, and a row with no case behind it is what this
regeneration exists to eliminate. **R-35 was a row here and is not one now. R-50
never was one, and is named here so a reader does not go looking for it.**

- **R-35 `switch` selects the wrong clause** — closed by allowlist 2026-07-29
  (`64438bf0ef`). Its authoritative boundary is **§7.11**: the admitted set, the
  fourteen-item fail-closed residual, two accepted regressions, three standing
  couplings. It was never §0.2's to summarise and the prior row said so itself.
  R-35's allowlist is, however, the gate that now refuses R-49's repro — see that
  row.
- **R-50** — filed in **§7** as a fail-loudly defect, not a §2 entry.

**Net, measured 2026-08-15 at `4cfa218814` against `node v26.7.0`.** Of the 41 §2
entries, **29 carry at least one SILENT lane** and **12 carry none**:

- **No silent lane (12):** R-01, R-02, R-03, R-04, R-05, R-07, R-11, R-19, R-20,
  R-29, R-49, R-54.
- **At least one silent lane (29):** R-06, R-08, R-09, R-10, R-12, R-13, R-14,
  R-15, R-16, R-17, R-18, R-21, R-22, R-23, R-24, R-25, R-26, R-27, R-28, R-30,
  R-31, R-32, R-33, R-34, R-47, R-48, R-51, R-52, R-53.
- **Tier 1's silent population is 2** — R-51 and R-52 — down from the eight
  entries Tier 1 holds. R-01, R-02, R-03 and R-05 fail closed; R-04 is fixed;
  R-49 fails closed by R-35's gate.
- **Movement since the 2026-07-24 net:** R-29 leaves the silent set (reclassified
  ACCEPTS_INVALID, same behaviour), taking the count from 30 entries to 29.
  R-08's `===` half and R-21's absent-field-with-`let`-receiver lane moved to
  FAIL_CLOSED; both entries still carry silent lanes and neither is retired.

**The 2026-07-24 sweep's own net is preserved below, unrewritten,** because it is
that sweep's record and the table above supersedes it rather than editing it. It
is dated `62d786e74` and must not be read as current.

> **Net (2026-07-24 sweep, `62d786e74` — SUPERSEDED by the table above):** of the
> register's ~29 silent-class entries, the sweep confirms **FIXED/fail-closed:
> R-01, R-02, R-03, R-04, R-05, R-07, R-08(=== half), R-19, R-20**, plus **R-11,
> CLOSED after this section's baseline** (`28f18b3ff`); **still SILENT: R-06-R2,
> R-06-R3, R-08(?? half), R-09, R-10, R-12, R-13, R-14, R-15, R-16, R-17, R-18,
> R-21, R-22, R-23, R-24, R-25(residual), R-26, R-27, R-28, R-29, R-30, R-31,
> R-32, R-33, R-34.** Added post-sweep 2026-07-25 and also **SILENT: R-47, R-48.**
>
> **Update 2026-07-29 (R-35 close-out, branch `r35-switch-lowering`).** Two changes
> to the sentence above, recorded here rather than rewritten into it so the sweep's
> own record stays legible:
>
> - **R-35 leaves the silent class.** Its admitted set is FIXED and everything else
>   is honest `E5506`. It is the second Tier-1 entry (after R-49) closed by this
>   project. Its residual is FAIL-CLOSED, which this register counts as
>   *acceptable*, not as a defect — but the residual is a real limit on what kali
>   compiles, and it is enumerated in **§7.11**, which is the authoritative list.
>   Neither this row nor §0.3's bullet is.
> - **Three new SILENT entries came out of the close-out's own probing: R-51, R-52,
>   R-53.** None of them involves `switch`. All three were found while building
>   switch-free **controls** for the loop-faithfulness and escape-analysis questions
>   R-35 raised — the recurring pattern this register documents: *the control is
>   where the new defect lives.* All three have a correct sibling form (`s(x)` for
>   R-51, a full four-clause `for` for R-52, `const` for R-53). R-52 and R-53
>   additionally **invalidate probe shapes**: `for (init; ;)` runs zero iterations
>   and `for (var v of …)` yields all-zero elements, so a fixture built on either
>   measures nothing while *looking* like it passed.

### 0.3 NEW silent miscompiles found this re-derivation (exit 0, no diagnostic, wrong)

- **R-35 — `switch` selects the wrong clause (HEADLINE, high blast radius). Tier 1.**
  **STATUS 2026-07-29: CLOSED-BY-ALLOWLIST on branch `r35-switch-lowering`. Everything
  below this paragraph describes the PRE-FIX behaviour and is retained as the historical
  record — do not read it as current.** The admitted set is now FIXED (matches node
  byte-for-byte) and every unadmitted shape is honest `E5506`; **there is no silent lane
  left in `switch`**. The authoritative statement of what is admitted, what is refused, the
  two accepted regressions and the three standing couplings is **§7.11**, not this bullet
  and not §0.2's row. The one shape where the *parser* is narrower than the allowlist is
  **R-50** (a sequence-expression discriminant, §7). Historical detail follows:
  codegen has **no `Switch` arm** (`grep -rn Switch crates/kali_codegen/src/` = 1 hit, an
  unrelated comment). `kali_hir` allocates `SwitchStmt` with children
  `[discriminant, clause-block-0, clause-block-1, …]`, `kali_mir` folds it into the same
  generic `ControlFlow` bucket as `IfStmt`, and the generic arm reads it as
  **`if (discriminant) { clause-1 } else { clause-2 }`**.
  `switch(x){case 10:return"A";case 20:return"B";default:return"D"}` → `s(20)`="A" (node "B"),
  `s(40)`="A" (node "D"), `s(0)`="B" (node "D"). `s(10)` coincidentally correct.
  **BOUNDARY RE-DERIVED 2026-07-28 on `5c9bbd051`** (branch `r35-switch-lowering`, after the
  R-49 parser-containment fix). The previously recorded boundary — *"a `break` in a case →
  E5506; a local read in a case → E3100, so the silent window is exactly
  all-return/no-break/no-local"* — was **measured THROUGH the R-49 parser leak and is void**;
  that `break` was a leaked break evaluated at module scope with no loop frame and that
  `E3100` was a leaked identifier read resolved against module scope. The true boundary,
  from a 32-cell both-scopes matrix (22 SILENT / 2 FAIL-CLOSED / 6 FL-INTERNAL / 2 CORRECT):
  - **Clauses beyond the second are never emitted at all** — so R-35 **silently drops code**
    (Tier 1), not merely a wrong value (Tier 2). A five-clause switch can never produce its
    3rd, 4th or `default` answer for *any* input; empty-clause grouping at **module scope**
    produces **no output whatsoever** where node prints.
  - **The wrong clause's side effects RUN**, not just its value: a `console.log` in clause 1
    prints where node prints clause 2's (`hit=100` vs node `hit=200`).
  - **String discriminants are affected** — a string is a nonzero handle, hence always
    truthy, so a string switch *always* takes clause 1 (even `""`, where node takes
    `default`).
  - `break`/`continue` in a clause is honest `E5506` **only when the switch is not inside a
    loop**; a `break` nested in a `for` loop compiles and silently breaks the **enclosing
    loop** (`r=1` where node prints `r=505`). A `continue` nested in a `for` loop is a
    *different* defect and **belongs to R-09, not R-35** — it hangs to `E4003` fuel
    exhaustion, and the identical hang reproduces with no `switch` anywhere (corrected in
    fix round 1; see R-09 and the matrix file's "Cell 13 — corrected"). A clause declaring
    **and reading its own** `var`,
    `let` or `const` does **not** fail closed — all three measure identically SILENT.
    `throw` in a clause is `E4000` where it fires and SILENT where it does not.
  - Superseded by **`docs/superpowers/followups/r35-switch-boundary-rederived.md`**, which
    carries the full matrix, the fixtures and both runtimes' stdout/exit per cell.
- **R-36 — class instance fields round-trip to `0` (REGRESSION).** `constructor(){this.v=3}` then
  `this.v` / `c.v` reads →0; the method body runs, only the field value is lost. The register had
  class `this` as FAIL-CLOSED (E4201); it is now silent, exit 0. Single-field only (2+ fields →
  FL-04). `class C{constructor(){this.v=3}} new C().v`→0 (node 3).
- **R-37 — `new Map()` is a silent 0-stub.** `m.set("k",5); m.get("k")`→0 (node 5); `m.size`→0.
- **R-38 — `new Set()` is a silent 0-stub, value-wrong in control flow.** `s.add(3); s.has(3)`→0
  (node true), and `if(s.has(3))` takes the ELSE branch — a silent branch flip, not just a value.
- **R-39 — `Array.prototype.pop()` returns `0`.** `[1,2,3].pop()`→0 (node 3).
- **R-40 — `.push` on a const array-literal is silently ignored.** `const a=[1,2]; a.push(3);
  a.length`→2 (node 3); `a[2]`→undefined. (The supported growable-array lane is fine
  **in-function only** — `function f(){const g=[]; g.push(7); return g.length} f()`→`1` ✓; at
  **module scope** it is a silent no-op too, `const g=[]; g.push(7); g.length`→`0` (node `1`),
  which is §7.9's "Module-scope growable `push` is a silent no-op"
  (`P5-R-modulescope-growable-push`). Both re-measured on merged `main` `372a3f440`,
  2026-07-25. The literal-array lane swallows the push in either scope.)
- **R-41 — `Array.prototype.concat` is ignored.** `[1,2].concat([3,4]).length`→2 (node 4); result
  is just the receiver.
- **R-42 — `Array.prototype.slice` element reads `0` — in the BOUND form.**
  `const a=[1,2,3]; const b=a.slice(1); b[0]`→`0` (node `2`), while `b.length`→`2` is correct:
  contents zeroed (R-14-flavored). **Repro corrected 2026-07-25** (measured on merged `main`
  `372a3f440`): the originally-recorded literal form `[1,2,3].slice(1)[0]` does **NOT**
  reproduce — it folds statically and prints `2`, correct, as does
  `[1,2,3].slice(1).length`→`2`. The defect is real only once the slice result is bound.
- **R-43 — array destructuring ASSIGNMENT is a no-op.** `let a=1,b=2; [a,b]=[b,a]`→`1,2` (node
  `2,1`); `let a=0n; [a]=[1n]; a`→`0` (node `1n`). (Destructuring DECLARATION fails closed, but
  with a *misdiagnosed* "reserved word" message — see 0.5.) **R-43 is the owning ID for
  §7.9's `P5-R-destructuring-assign` bullet**, which is the same defect (both repros re-measured
  identical on merged `main` `372a3f440`, 2026-07-25) and whose claim that "no register entry
  covers destructuring-assignment drop specifically" R-43 falsifies; that bullet is retained
  for its AST-decay mechanism datum. Cluster **G1**.
- **R-44 — chained member on a function-CALL result → `0`.** `function f(){return{a:1}} f().a`→0
  (node 1); `const r=f(); r.a`→1 is correct. This is the R-06-R1 "member-on-call hole" and it
  FALSIFIES R-14's old "object-return is correct" control. `"a,b,c".split(",").length`→0 is the
  same shape (method-chain result). Arrays are strictly worse (`const a=f(); a[0]`→0 too).
- **R-45 — `NaN` is not represented in a slot.** `var x=NaN; log(x)`→0 (node NaN); `NaN+1`→1
  (node NaN). A real value silently collapses to `0` (distinct from R-28's `-0`). (The LITERAL
  concat form instead crashes — FL-02.)
- **R-46 — `-Infinity` rendering / handling.** `console.log(-Infinity)`→`-inf` (C-style, silent);
  a `var`-bound `-Infinity` instead crashes (FL-02). Positive `Infinity`/`NaN` direct-log correct.

Added 2026-07-29 by the **R-35 close-out** (Task 11), all measured on `58234e87c7` with
`node v26.5.0` as oracle, all switch-free, all exit 0 unless stated. Full entries in §2.

- **R-51 — an optional call `s?.(x)` returns `0` and never runs the callee. Tier 1.**
  `var hits = 0; function s(x) { hits = hits + 1; return x; } console.log("w=" + s?.(7));
  console.log("hits=" + hits);` → kali `w=0` / `hits=0`, node `w=7` / `hits=1`. Exit 0 with
  **no diagnostic at all** — not on `kali run`, not on `kali build`. The callee body does
  not execute, so this drops code, not just a value. The non-optional control `s(7)` is
  correct on both engines. **Standing coupling to R-35**: `s?.(x)` is an invocation route
  invisible to *both* halves of R-35's switch-parameter proof — see §2's R-51 entry.
- **R-52 — `for`-clause arity misclassification silently skips or truncates the loop.
  Tier 1 (+ an FL-INTERNAL manifestation).** kali's HIR omits absent `for` clauses and
  codegen classifies the survivors **by count**, so it cannot tell which clause is missing.
  `for (var i = 0; ;) { i = i+1; log("iter="+i); if (i>5) break; s = s+i; }` → kali prints
  **only `s=0`** (exit 0, no diagnostic) where node prints six `iter=` lines and `s=15`:
  the `var i = 0` *declaration* is used as the loop test and is falsy, so the body never
  runs. `for (init; ; update)` drops the **first** iteration (node `iter=0..5`, kali
  `iter=1..5`, both exit 0). `for (; test; update)` runs away to `E4003` (loud). Distinct
  from R-09, which is about update PLACEMENT, not clause identification.
- **R-53 — `for (var v of […])` binds every element to `0`. Tier 2.**
  `var t = 0; for (var v of [1,2,3]) { log("iter="+v); t = t+v; }` → kali `iter=0` three
  times and `t=0`, node `iter=1/2/3` and `t=6`. Exit 0, no diagnostic, no `break`,
  `continue` or `switch` involved. **`const` is correct** on the byte-identical fixture.
  Distinct from **R-47** (`for..of` over a `let`-declared array BINDING iterates the
  binding's own NAME): this is the loop VARIABLE's declarator kind over an array LITERAL.
  **Probe-design consequence: `for (var v of …)` is not a usable faithful-loop control.**
- **R-54 — a second `default` clause is absorbed into the first; kali accepts a file node
  rejects. Tier 3.** Found while completing the acceptance matrix's `default` axis — the
  "two or more `default`s" **denied** cell would not deny.
  ```js
  var g = 0;
  function s(x) {
    switch (x) {
      case 1: return "one";
      default: g = 5;
      default: return "d2";
    }
  }
  console.log("v=" + s(9));
  console.log("g=" + g);
  ```
  **node**: `SyntaxError: More than one default clause in switch statement`, whole file
  refused, exit 1. **kali**: `v=d2` / `g=5`, exit 0 — **both `default` bodies ran**, merged
  into one clause. Traced: `parse_switch_statement`'s **`default`** arm stops its statement
  loop on `Case | RightBrace` only, omitting `Default`, where the sibling **`case`** arm
  (`crates/kali_parser/src/statement.rs:536-541`) correctly stops on
  `Case | Default | RightBrace`. **Only invalid JS is affected** — no valid program can
  contain two `default`s — which is why this is Tier 3 and not Tier 1. Cluster **G1**, in
  the **same function as R-49** and independent of it.

### 0.4 NEW fail-loud-INTERNAL crashes (exit 1, wrong error KIND — belong with §7 FL family)

These exit nonzero (so no silent-trust is at stake) but via an internal `E4201`/`E4003`
("WebAssembly translation error" / fuel) instead of an honest `E5506` naming the limit.

- **FL-02 — non-finite float literal in string concat → E4201.** `"v="+NaN`, `"v="+Infinity`,
  `"v="+(-Infinity)`, and `var x=-Infinity; log(x)`. Finite-float concat is fine. The
  float→string sink cannot render a non-finite f64 and traps module load.
- **FL-03 — `NaN === NaN` / `NaN < 1` → E4201.** (`isNaN(NaN)` and `x=0/0; x!==x` are correct.)
- **FL-04 — class with 2+ instance fields → E4201.** `constructor(x){this.v=x;this.w=10}`.
- **FL-05 — excess-arity call → E4201.** `function f(a){} f(1,2,3)`.
- **FL-06 — spread in call → E4201.** `add(...[1,2,3])`.

### 0.5 Diagnostic-quality note (fails closed, but wrong reason)

`const [x,y]=[1,2]` / `const {a,b}={a:1,b:2}` → `E5506 "a reserved word cannot be used as a
binding name"`. The names are not reserved words; destructuring *declaration* is simply
unsupported and the message misdiagnoses it. Honest (exit 1) but misleading.

---

## 1. Executive summary

**42 raw defects → 33 after deduplication** *(the original four-sweep intake — correct when
written, for the 33 entries R-01..R-33 that then existed; stale since R-34 landed. See the note
under the table. The register now holds **49** numbered entries: R-01..R-49.)* Nine entries were
folded into siblings that
share a demonstrated or strongly-inferred root cause (noted per entry).

Severity split (each entry ranked at the most severe class it carries):

| tier | class | count (historical R-01..R-34 / now) |
|---|---|---|
| 1 | **silently drops code or output** — statements never run, calls never fire, output vanishes | 5 / **8** |
| 2 | **silently produces a wrong value** | 23 / **26** |
| 3 | **silently wrong control flow only** (value otherwise intact) | 1 / **2** |
| 4 | **rendering-only** (in-memory value is correct) | 4 (see note) / 5 |

The left-hand counts are the original R-01..R-34 sweep, left as the historical record. Since
then §0.3 added **R-35..R-46** (2026-07-24 re-derivation), §2 added **R-47** and **R-48**
(2026-07-25, promoted from §7.10 sightings), and §2 added **R-49** (2026-07-28, from the R-35
switch-lowering stage). R-47 and R-48 are filed in Tier 2, and R-47 additionally carries a
**Tier-3** wrong-trip-count half (its entry says so); R-49 is filed in **Tier 1**, which is
the change recorded in this table's Tier-1 cell. **R-35..R-46 are excluded from these counts entirely** — the re-derivation recorded
them as §0.3 bullets and never tier-ranked them, so tier-ranking them here would be an
unmeasured claim. (R-35's 2026-07-28 re-derivation now *establishes* that it is Tier 1 — it
drops clauses, not just values — but it remains an un-ranked §0.3 bullet and is still outside
these counts; see §0.3 and `r35-switch-boundary-rederived.md`.) **R-49** was added
2026-07-28 as a tier-ranked §2 **Tier 1** entry and *is* counted, which is the only change to
the right-hand column since 2026-07-25.

**Updated 2026-07-29 (R-35 close-out, Task 11).** The right-hand column moved twice more:
**R-51** and **R-52** were added as tier-ranked §2 **Tier 1** entries, **R-53** as a
tier-ranked §2 **Tier 2** entry and **R-54** as a tier-ranked §2 **Tier 3** entry — R-51,
R-52 and R-53 new silent miscompiles found by the close-out's own switch-free control
fixtures, R-54 an accepts-invalid parser fail-open found by the acceptance matrix's own
`default` axis (see §0.3 and §7.11). **R-35 itself is now
CLOSED-BY-ALLOWLIST** — its admitted set matches node and everything else is honest
`E5506` — but it remains an un-ranked §0.3 bullet and stays outside these counts for the
same reason it always did, so **closing it changes no cell in this table**. R-50 is filed
in §7 and is likewise not counted (see its own numbering note).

Right-hand column = **41** tier-ranked entries in §2 (8 + 26 + 2 + 5), re-counted 2026-07-29
by `### R-` headers per tier heading; the register holds **54** numbered entries in total
(R-01..R-54), the other 13 being the un-ranked §0.3 set (R-35..R-46) plus §7's R-50. **The historical Tier-4 cell reads `4` where Tier 4 now holds five
entries (R-30..R-34) — but it was CORRECT when written and went stale afterwards, not an
off-by-one.** Verified in history 2026-07-25: `ee0225f37`, the commit that created this table,
had Tier 4 = R-30..R-33, exactly four entries, and no R-34 anywhere in the file; `2727252f6`
(the first and only commit introducing `### R-34`) appended it to Tier 4 without updating the
cell. The same applies to the "33 after deduplication" headline — correct for the 33 entries
R-01..R-33 that existed at authoring, one short only once R-34 landed. Both are recorded rather
than silently corrected, since the left-hand column is the historical record.

Every entry in this document is an **exit-0, no-diagnostic** divergence unless the entry
says otherwise. Fail-closed behavior (`E5506`, `E3100`, `E4201`, traps) is recorded only as
context, because refusing to compile is the correct outcome and not a defect of this class.

### The five a reader must know first

1. **R-01 — a default parameter silently truncates the module.** `function g(b=5){}` causes
   every later statement in the file to be dropped, exit 0, no diagnostic. This is
   *evidence-corrupting*: any fixture or probe in this repository that contains a default
   parameter has been silently truncated, so conclusions drawn from it may be invalid.
2. **R-07 — `const` is not a binding.** Its initializer expression is re-emitted at every
   read site, so `const tmp=a; a=b; b=tmp` yields `a=2 b=2`. Every "snapshot a mutable
   value" idiom in JS is wrong, and side effects fire once per *read*.
3. **R-08 — `===` conflates `0`, `null`, `undefined` and `false`.** `0 === null` is `true`.
   Every null-guard in every program fires for the perfectly valid value `0`.
4. **R-02 — calling a function through a function *value* returns `0` and never runs the
   callee.** Callbacks, returned closures, function tables and object methods all silently
   evaluate to `0`; a dropped call flips branches.
5. **R-12 — one alias binding defeats a fail-closed guard.** `const b=a; b[0]=7` compiles
   and silently no-ops, while the un-aliased `a[0]=7` correctly fails closed with `E5506`.

Two further items every future investigator needs before running any probe at all:

- **R-04 — `console.log` (and `.error`/`.warn`/`.info`) silently discards every argument
  after the first whenever any argument is non-literal.** This is the primary instrument of
  every sweep. It must be validated before use, and probes must pass exactly one argument.
- **R-11 — ~~every bitwise compound assignment (`&= |= ^= <<= >>= >>>=`) is a silent no-op~~ —
  CLOSED 2026-07-25** (branch `r11-bitwise-compound-assign`, `0104f5baf`..`9dcdcc3c1`). The six
  operators now compute correct values on proven-integer targets and fail closed `E5506`
  everywhere else. Re-measured over the final 49-target × 6-op audit matrix: on the pre-R-11
  binary `e416b22a1`, **209 of 294 cells printed the unmodified operand at exit 0**; on
  `9dcdcc3c1`, **0** — 144 MATCH, 150 `E5506`, 0 `WRONG`, 0 `E4201`, and **no cell moved into
  `WRONG` or `E4201`**. See the R-11 entry in §2 for the full close note and §7.10 for the
  sightings, accepted costs and lessons this project produced.

---

## 2. Deduplicated, severity-ranked register

Ranking rule: an entry is placed at the most severe class it carries — silently drops
code/output > silently wrong value > silently wrong control flow > rendering-only. Within a
tier, ordering is by blast radius.

---

## Tier 1 — silently drops code or output

### R-01: A default parameter silently truncates the rest of the module

- **Folds in**: D-C-1.
- **Verification**: `CONFIRMED-BY-CONTROLLER`.
- **Root-cause group**: G1 (parser fail-open recovery).
- **Repro** (`scratchpad/consolidate/dp.js`):
  ```js
  console.log("A");
  function g(b=5){ return b; }
  console.log("B");
  ```
- **node**: `A` / `B` (exit 0) — **kali**: `A` (exit 0), nothing on stderr, no E-code.
- **Scopes affected**: both. Also fires for function *expressions*
  (`const g = function (b=1) {...}`). When the declaration is the first statement, the
  *entire* program is dropped and kali prints nothing at exit 0. Arrow functions with
  defaults fail **closed** (`E3100`) instead — the truncation is specific to `function` forms.
- **Severity**: silent-missing-output — the worst class. An arbitrary suffix of the program
  vanishes.
- **Blast radius**: very high, and uniquely corrosive. Default parameters are ordinary
  modern JS. Beyond miscompiling user programs, this is a **silencing** bug: it can mask any
  other defect in any file that contains a default parameter, including this repository's own
  fixtures and every probe written during past investigations.
- **Mechanism**: `crates/kali_parser/src/declaration.rs:13-35`, `parse_parameter_list`. After
  consuming identifier `a`, the next token is `=`, so neither `accept(RightParen)` (line 25)
  nor `accept(Comma)` (line 28) matches; lines 29-30 do
  `let _ = self.stream.accept(RightParen); break;` — a *silent* recovery leaving the token
  stream parked on `=`. The parser desynchronizes and the remaining statements are dropped
  with no diagnostic. The discarded `accept` result on line 29 is the fail-open.
- **Confidence**: high on behavior (6 sweep transcripts + controller re-run); high on
  mechanism (source is unambiguous).

### R-02: Calling a function through any first-class function value returns `0` and never runs the callee

- **Folds in**: D-C-2, plus D-C-2's closure sub-cases (c01–c12) **as corrected below**.
- **Verification**: `CONFIRMED-BY-CONTROLLER`, **with a correction to sweep C**.
- **Root-cause group**: G2 (call lowering: unresolvable callee → constant `0`).
- **Repro**:
  ```js
  function boom() { console.log("CALLEE RAN"); return 5; }
  var g = boom;
  console.log("r=" + g());
  ```
- **node**: `CALLEE RAN` then `r=5` (exit 0) — **kali**: `r=0` (exit 0). `CALLEE RAN` is
  absent, proving the callee is **never invoked**.
- **Control-flow escalation** (sweep C b15/z1): `function t(){return 1;} var g=t;
  if (g()) {...} else {...}` — node prints `then`, kali prints `else`. A dropped call
  silently flips a branch.
- **CORRECTION — sweep C's "every closure shape is broken / closures are effectively
  nonexistent" is OVERSTATED.** Controller re-run on a fresh binary:

  Direct sibling capture is **CORRECT** — this is the shipped Stage C env-pointer lane
  (`scratchpad/consolidate/c1.js`):
  ```js
  function outer(){ let n=1; function inc(){ n=n+1; } inc(); console.log("captured="+n); }
  outer();
  ```
  node `captured=2` (exit 0) — kali `captured=2` (exit 0). **Match.**

  **Returned** closures are silently wrong (`c2.js`):
  ```js
  function mk(){ let n=0; return function(){ n=n+1; return n; }; }
  const f=mk();
  console.log("returned="+f());
  ```
  node `returned=1` (exit 0) — kali `returned=0` (exit 0). **Silent, exit 0.**

  Both shapes in one file (`c3.js`, `c4.js`) still produce `captured=2` / `returned=0` at
  exit 0. The controller separately observed an `E4201` (malformed WASM, exit 1) for a
  mixed-shape file; the controller's shape was not reproduced by the two mixed variants
  re-run here, so **the E4201 is shape-sensitive and the silent `returned=0` form is the
  common one**. Recorded as a discrepancy rather than resolved: both outcomes are real, and
  a fix must not assume the loud one.
- **Supported vs broken boundary** (sweep C b9, exhaustive):
  - **CORRECT**: direct named call `dbl(21)`; `const g = <arrow or function literal>` then
    `g(21)` (expression- and block-bodied, both scopes); IIFEs in both forms; sibling
    closures called directly by name inside their definer (above).
  - **SILENTLY WRONG (→ `0`)**: `let g = <fn literal>`; `var g = <fn literal>`;
    `const g = existingName` (alias); a function passed as a **parameter** and called
    (`function apply(h,x){return h(x);}`), *even when the argument is a `const` arrow*; a
    function **returned** from a function and called; a reassigned function var
    (`let g=a; g=b; g()`).
  - Note the `let`/`var` vs `const` polarity here — it is the same polarity as R-06, and
    that coincidence is the basis of cluster G7.
- **Severity**: silent-wrong-value + silent-missing-output + silent-wrong-control-flow.
- **Blast radius**: extreme. Callbacks, higher-order functions, function tables, strategy
  objects and returned closures. Note the interaction with R-01: a codebase using default
  parameters never reaches these calls, so the two defects hide each other.
- **Mechanism hypothesis**: not pinned to a line. Consistent with call lowering resolving the
  callee by *name* and, on static-resolution failure, emitting a constant `0` for the call
  expression instead of failing closed. Per this repo's own repeated lesson the fix shape is
  an **allowlist at the call-lowering choke point** (emit only for statically-resolved
  callees or admitted closure lanes, `E5506` otherwise), not a denylist of value shapes.
- **Confidence**: high on behavior (20+ sweep transcripts + 4 controller re-runs); medium on
  the single-root claim.

### R-03: `Array.prototype.forEach` / expression-arrow `filter` silently no-op

- **Folds in**: D-C-4.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G3 (guard denylist with sibling holes); possibly G2.
- **Repro**:
  ```js
  const a = [1, 2, 3];
  a.forEach((x) => { console.log("saw" + x); });
  console.log("done");
  ```
- **node**: `saw1` `saw2` `saw3` `done` (exit 0) — **kali**: `done` (exit 0).
- **Second shape**: `[1,2,3,4].filter((x) => x > 2).length` → node `2`, kali `0`, exit 0.
- **Why this is distinct from R-02**: the array-callback lane **has** a fail-closed guard.
  `map` correctly emits `E5506` ("array callback method 'map' is unavailable"), and `filter`
  with a **block-bodied** callback also emits `E5506`. But `forEach` is not on that denylist
  at all, and `filter` with an **expression-bodied** arrow slips past the body check. This is
  a denylist with holes — exactly the class this repo has repeatedly had to close with an
  allowlist.
- **Severity**: silent-missing-output (`forEach`) / silent-wrong-value (`filter`).
- **Blast radius**: high. `forEach` is ubiquitous and fails in the most dangerous direction:
  work silently not done.
- **Correct neighbor**: `reduce` is genuinely correct, verified on two non-degenerate folds.
- **Confidence**: high on behavior; high on the "denylist hole" characterization (the E5506
  for `map` is direct evidence the guard exists and is incomplete).

### R-04: The whole `console` family drops every argument after the first when any argument is non-literal

- **Folds in**: D-A-3 (boundary map of a known defect, plus a genuine extension).
- **Verification**: `sweep-only` (both scopes).
- **Root-cause group**: G8 (per-sink rendering divergence).
- **The rule, precisely**: if *every* argument is a compile-time constant literal, all
  arguments print correctly. If *any one* argument is not a literal, kali prints **argument 0
  only** (correctly evaluated) and **silently discards all remaining arguments**. It drops; it
  never reorders; argument 0 is never lost.
- **Position-independence** (it is "any argument", not "a later argument"):
  - `console.log(1+1, 5)` → `2` (node `2 5`)
  - `console.log(5, 1+1)` → `5` (node `5 2`)
  - `var x=3; console.log(1, x, 2)` → `1` (node `1 3 2`)
  - `var x=3; console.log(1, 2, x)` → `1` (node `1 2 3`)
  - `console.log(1, 2+0, 3, 4+0)` → `1` (node `1 2 3 4`) — three arguments lost in one call
- **Literal** (call is correct): number, string, `true`/`false`, `null`, `undefined`, a
  negative numeric literal, a parenthesized literal, a template literal with no substitution.
  Zero-arg `console.log()` is correct.
- **Non-literal** (triggers the drop): arithmetic, string concatenation, a plain variable
  reference *including a `var` bound to a literal*, a function call, a template literal
  *with* a substitution.
- **EXTENSION (new, materially wider than "console.log")**: the same drop affects **every**
  console sink. `console.error(1, x)` → `1`; `console.warn(1, x)` → `[warn] 1`;
  `console.info(1, x)` → `1`. A fix targeting `console.log` alone leaves three sinks broken.
- **Scopes affected**: both.
- **Severity**: silent-missing-output.
- **Blast radius**: very high. `console.log(label, value)` is the single most common debug and
  report shape in JS, and the dropped case — a variable or expression as the value — is
  precisely the useful one. **This defect is also the primary instrument of every sweep in
  this repository**; see §4.
- **Confidence**: high on behavior and boundary (25+ transcripts, no exceptions found).

### R-05: Object-literal method calls return `0`, never run the body; `this` yields `0`

- **Folds in**: D-C-3.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G2.
- **Repro**:
  ```js
  const o = { f: function () { console.log("RAN"); return 7; } };
  console.log("r=" + o.f());
  ```
- **node**: `RAN` then `r=7` (exit 0) — **kali**: `r=0` (exit 0), body never runs. Same with
  an arrow value (`{ f: () => 7 }`).
- **`this` specifically**: `const o = { v: 3, f: function () { return this.v; } }; o.f()` →
  kali `r=0` at exit 0; node `3`. **`this` in an object-literal method silently miscompiles.**
  By contrast `this.v` inside a *class* method fails **closed** (`E4201`), so the two `this`
  surfaces disagree — one lies, one refuses.
- **Severity**: silent-wrong-value + silent-missing-output.
- **Blast radius**: high — a function stored in an object field is the most common JS
  namespace/module-object idiom.
- **Mechanism hypothesis**: probably the same unresolvable-callee fallback as R-02 (a member
  expression can never resolve to a name). If so, one allowlist fixes both. `this` → `0` is
  consistent with `this` being an unbound identifier that also falls back to `0`.
- **Confidence**: high on behavior; medium on sharing R-02's root.
- **Fail-closed context**: method shorthand `{ f() {...} }` → `E3100`; class methods
  *without* `this` are correct including arguments and side effects.

---

### R-49: `parse_switch_statement` silently reparented every post-switch statement to module scope — **CLOSED 2026-07-28**

- **Added**: 2026-07-28, from the R-35 switch-lowering project (branch
  `r35-switch-lowering`). **This is not R-35** — different layer (parser, not codegen),
  different blast radius (every statement after *any* `switch`, not the switch's own
  clauses), and higher severity.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — traced in source, reproduced on a freshly
  built binary, closed with a regression test in the same commit.
- **Root-cause group**: G1 (parser fail-open recovery).
- **Mechanism (traced)**: `parse_switch_statement` ended its clause loop by *inspecting*
  `TokenType::RightBrace` without consuming it. The switch's closing brace was therefore
  still on the stream when control returned to the enclosing block parser, which took it as
  **its own** terminator and stopped. Everything after the `switch` — to the end of the
  enclosing function — was reparented into the module body.
  `parse_block_statement`, `parse_class_body` and `parse_arrow_function_body_expression`
  all already `accept()` their closer; this was the **unique** non-consuming closer site in
  the parser.
- **Decisive repro** (a function that is never called still runs):
  ```js
  var g = 0;
  function f(x) {
    switch (x) { case 1: g = 1; break; }
    g = 99;
  }
  console.log("g=" + g);
  ```
  **node**: `g=0` (exit 0) — `f` is never called. **kali (pre-fix)**: `g=99` (exit 0) — the
  `g = 99` was hoisted out of `f` and executed at module load. A function *declared* after a
  switch-containing function disappeared entirely by the same mechanism.
- **Severity**: Tier 1 — silently drops and silently *relocates* code. Worse than a wrong
  value: statements execute that the program never reached, and statements the program did
  reach never execute.
- **Evidence-integrity consequence**: **every probe in this repository that placed a
  statement after a `switch` was measuring the leak, not the feature.** R-35's originally
  recorded boundary is the known casualty (see §0.3 and
  `r35-switch-boundary-rederived.md`); any other pre-2026-07-28 finding whose fixture
  contains a `switch` should be re-derived before it is relied on.
- **CLOSED** by `9db9150c0` ("fix(parser): consume the switch closing brace — stop
  reparenting post-switch statements to module scope") on branch `r35-switch-lowering`.
- **Regression cover — citation corrected 2026-07-29 (R-35 close-out).** This entry
  previously named `crates/kali_cli/tests/switch_parser_containment.rs`. **That file no
  longer exists**: Task 6 of the R-35 stage deleted it and moved its pins **up a layer**,
  from an end-to-end `kali run` harness into the parser's own integration suite, where the
  defect actually lives. A reader following the old path found nothing and could reasonably
  have concluded the closure was unpinned. The live pins are in
  **`crates/kali_parser/tests/parser_integration.rs`, `mod switch`**:
  - `test_switch_does_not_leak_following_statements_out_of_a_function` — the containment
    property itself.
  - `test_function_declared_after_a_switch_function_survives_as_sibling_statement` — the
    "a whole later declaration disappeared" half.
  - `test_call_to_a_switch_containing_function_leaves_module_scope_statement_intact` — the
    decisive repro above, as an assertion.
  - `test_parse_switch_statement`, plus
    `test_switch_missing_paren_reports_expected_token`,
    `test_switch_missing_case_colon_reports_expected_token` and
    `test_well_formed_switch_reports_no_expected_token` — the `expect(kind)` hardening from
    `5c9bbd051` noted below.
  **Lesson worth keeping**: a register citation that names a *file path* rots when the file
  moves. Where a test's name is stable, cite the test name and the module, as above.
- **Related, same stage**: `5c9bbd051` added the parser's missing `expect(kind)` helper and
  routed all six required-token positions in `parse_switch_statement` through it — see §4's
  note on `e2::EXPECTED_TOKEN`.

### R-51: An optional call `s?.(x)` returns `0` and never runs the callee

- **Added**: 2026-07-29, by the **R-35 close-out** (Task 11, branch `r35-switch-lowering`).
  Found while enumerating every route by which a function can be *invoked*, in order to
  check whether R-35's switch-parameter proof could miss one. It can.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — measured on a freshly built binary at
  `58234e87c7`, against `node v26.5.0`, with a paired non-optional control.
- **Root-cause group**: G2 (call lowering: unresolvable callee folds to constant `0`) — the
  symptom is G2's exactly, though the *route* is the optional-chain lowering rather than an
  unresolvable callee. Recorded as G2 by symptom; the mechanism is named below.
- **Repro** (one argument per `console.log`, literal-rooted, no default parameters):
  ```js
  var hits = 0;
  function s(x) { hits = hits + 1; return x; }
  console.log("w=" + s?.(7));
  console.log("hits=" + hits);
  ```
  **node**: `w=7` / `hits=1` (exit 0). **kali**: `w=0` / `hits=0` (exit 0).
- **The side-effect counter is the load-bearing half of the repro.** `w=0` alone would be
  consistent with "the call ran and its result was lost" — the ordinary G2 shape. `hits=0`
  proves the **callee body never executed**, which makes this Tier 1 (silently drops code),
  not Tier 2. Any observable the callee was responsible for — a write, a log, a push — is
  simply gone.
- **Control**: the identical program with `s(7)` instead of `s?.(7)` gives `w=7` / `hits=1`
  on **both** engines. So this is the optional-call route specifically, not the call
  lowering generally and not this fixture's shape.
- **There is no diagnostic, on either subcommand.** `kali run` prints nothing on stderr and
  exits 0; `kali build` prints only `Built executable artifact at d7.wasm` and exits 0.
  Worth stating explicitly because an early note on this defect described it as emitting
  "only a warning" — it does not emit even that. It is fully silent.
- **Mechanism (partly traced)**: `kali_hir`'s `lower_optional_chain`
  (`crates/kali_hir/src/lowering/expression.rs:244`) handles `OptionalChainInner::NonNull`
  and the call form does not survive it as a call. Downstream, `kali_types`' `repr_infer`
  has an `OptionalChainExpression` arm that **descends into the object only**
  (`crates/kali_types/src/repr_infer.rs:2343-2344`), so the invocation is never seen as an
  invocation by the inference pass either. Not fully traced to the emit site.
- **STANDING COUPLING TO R-35 — this is why the entry exists, and it must not be dropped
  when this defect is fixed.** `s?.(x)` is an invocation route invisible to **both** halves
  of R-35's switch-parameter proof:
  1. it produces **no escape mark**, because the escape walk has no
     `OptionalChainExpression` arm; and
  2. it produces **no `CallEdge`** (`crates/kali_types/src/repr_infer.rs:351`, built at
     `:4473` for a bare-identifier `CallExpression` only), so it contributes no argument
     evidence.
  R-35 admits a **parameter** discriminant only when the parameter's inflow is proven and
  the enclosing function does not escape. An invisible invocation site therefore satisfies
  that proof *vacuously* — exactly the leak `new s(true)` produced before Task 7 closed it
  (`crates/kali_codegen/src/emit/switch.rs:438`,
  `crates/kali_types/src/repr_infer.rs:3236,3577`, pinned by
  `a_new_invocation_site_of_the_enclosing_function_is_fail_closed` and
  `a_new_expression_call_site_denies_a_string_parameter_discriminant`).
  **It is latent TODAY only because optional calls are dropped entirely** — the call never
  happens, so it cannot deliver a discriminant of the wrong domain. The two defects mask
  each other.
  **WARNING, standing: if optional-call lowering is ever implemented, the escape gate must
  be extended in or before that same change.** Fixing R-51 alone un-masks the R-35
  parameter leak *verbatim* — a `switch` on a parameter would be admitted on a proof that
  never saw the call site that supplies it, and the result is a silent miscompile in
  territory the allowlist believes it has proven. Do not treat R-51 as an isolated
  call-lowering fix. See §7.11's "design note for whoever makes cross-module calls real",
  which specifies the shape the extension should take (**extend the escape notion once, at
  the walk**, covering `export`, dynamic `import()` and the optional call together — do not
  add per-route checks).
- **Severity**: Tier 1 — silently drops the callee's execution and yields `0`.
- **Blast radius**: bounded by how often `?.()` appears in the corpus, which this entry did
  not measure — but the failure is total (no execution, no value, no diagnostic) wherever it
  does appear, and `?.()` is the idiomatic spelling for optional callbacks.
- **Confidence**: high on behavior (paired control, side-effect counter, both subcommands);
  medium on mechanism (the HIR and inference gaps are traced; the emit site is not).

### R-52: `for`-clause arity misclassification — an omitted clause silently skips or truncates the loop

- **Added**: 2026-07-29, by the **R-35 close-out** (Task 11, branch `r35-switch-lowering`).
  Found while building switch-free **controls** for the R-09 loop-faithfulness question.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — traced in source, measured on a freshly
  built binary at `58234e87c7` against `node v26.5.0`, three arities differentially compared
  against the correct four-clause form.
- **Root-cause group**: unclustered (an isolated lowering/emit contract mismatch), but it is
  a textbook instance of the pattern §3's G-clusters keep circling: **two passes with an
  unwritten agreement about a positional encoding**.
- **Mechanism (traced, both halves)**:
  1. `kali_hir`'s `for` lowering pushes a child **only for clauses that are present**
     (`crates/kali_hir/src/lowering/statement.rs:189-209`: `if let Some(init) … if let
     Some(test) … if let Some(update) …`, then unconditionally the body). An absent clause
     leaves **no hole** — nothing marks its position.
  2. `kali_codegen`'s `emit_loop` recovers the clauses **by counting children**
     (`crates/kali_codegen/src/emit/control_flow.rs:280-297`, whose own comment says
     *"[init?, test?, update?, body] — body is always last; classify by count"*): 2 children
     ⇒ `(None, first, None)`; 3 ⇒ `(first, second, None)`; 4 ⇒ `(first, second, third)`.
  Count cannot distinguish *which* clause is missing, so every arity with an omitted clause
  and a present later one is misread. The four-clause form is correct, which is why this
  survived: the overwhelmingly common `for (a; b; c)` shape is fine.
- **Repro A — the loop is skipped ENTIRELY, silently** (`for (init; ;)`, 2 children, so the
  `init` **declaration** is used as the test and is falsy):
  ```js
  var s = 0;
  for (var i = 0; ;) {
    i = i + 1;
    console.log("iter=" + i);
    if (i > 5) break;
    s = s + i;
  }
  console.log("s=" + s);
  ```
  **node**: `iter=1` … `iter=6` then `s=15` (exit 0). **kali**: **`s=0` and nothing else** —
  exit 0, no diagnostic. Zero iterations. This is the Tier-1 cell: the body is *emitted* but
  never *entered*, and the program looks like it ran.
- **Repro B — the FIRST iteration is dropped, silently** (`for (init; ; update)`, 3 children
  ⇒ `(init, update-as-test, None)`, so the update runs as the test and advances the counter
  once before the body is ever entered):
  ```js
  var s = 0;
  for (var i = 0; ; i = i + 1) {
    console.log("iter=" + i);
    if (i > 4) break;
    s = s + i;
  }
  console.log("s=" + s);
  ```
  **node**: `iter=0` … `iter=5`, `s=10` (exit 0). **kali**: `iter=1` … `iter=5`, `s=10`
  (exit 0). **Note the sums agree** — `s=10` on both sides — which is exactly why the
  per-iteration `console.log` is mandatory: a fixture asserting only the final value would
  have scored this cell CORRECT. The `iter=` lines are the whole evidence.
- **Repro C — runaway, loud** (`for (; test; update)`, 3 children ⇒ `(test-as-init,
  update-as-test, None)`: the real test is evaluated once and discarded, and the update is
  used as a test that is always truthy):
  ```js
  var s = 0;
  var i = 0;
  for (; i < 4; i = i + 1) {
    console.log("iter=" + i);
    s = s + i;
  }
  console.log("s=" + s);
  ```
  **node**: `iter=0..3`, `s=6` (exit 0). **kali**: `iter=1`, `iter=2`, … to
  `error[E4003]: CPU fuel budget exhausted` (exit 1). Lower severity — it is loud.
- **Control**: `for (; i < 3; )` (2 children ⇒ `(None, test, None)`, correctly classified)
  matches node byte-for-byte, as does the full four-clause `for (var i = 0; i < 3; i = i+1)`
  used throughout `switch_runtime.rs`. So this is arity-specific, not a general `for` defect.
- **Distinct from R-09**, and the distinction matters when fixing either. R-09 is about
  where the update is *placed* relative to `continue`'s branch target — the clauses are
  identified correctly and the loop runs. R-52 is about the clauses being *identified*
  wrongly in the first place; `continue` need not appear at all.
- **STANDING COUPLING — `continue_is_faithful` is currently right for the wrong reason.**
  `crates/kali_codegen/src/emit/control_flow.rs:348` computes
  `let continue_is_faithful = update.is_none() && kind != "do-while";` **from the
  misclassified triple**. In all three broken arities above the real update has been
  consumed as the `test`, so `update` is `None` and the loop is flagged
  `continue_is_faithful = true`. That flag is what R-35's `switch` allowlist consumes to
  decide whether a clause's `continue` may be admitted (denial constant
  `UNFAITHFUL_CONTINUE`). It is **harmless today only because those loops are already
  broken** — Repro A never enters the body, so no `continue` inside it can execute; Repros B
  and C are already wrong or already trapping.
  **WARNING, standing: if this arity bug is ever fixed, `control_flow.rs:348` must be
  re-derived in the same change.** The moment the triple is classified correctly, `update`
  becomes `Some(…)` for these shapes and the flag must flip to `false` — otherwise a
  `switch` clause's `continue` will be **admitted into a loop that skips its update**, which
  is R-09's silent-wrong-value form reached *through* a construct the allowlist certified.
  This is the third documented instance in this project of a fix un-masking a leak that a
  second defect was covering; see also R-51's coupling to the R-35 escape gate.
- **Severity**: Tier 1 for Repro A (drops the entire loop body's execution), Tier 2 for
  Repro B (wrong value / wrong trip count), FL-INTERNAL for Repro C.
- **Blast radius**: bounded — `for` loops with omitted clauses are a minority of `for`
  loops. But the shapes are idiomatic (`for (var i = 0; ;)` with an internal `break` is a
  standard reader loop) and the Repro A failure is total and silent.
- **Probe-design consequence**: **`for (init; ;)` cannot be used as a fixture loop.** It
  runs zero iterations, so any assertion about its body vacuously "passes" while measuring
  nothing. Prefer `while`, or a C-style `for` with **no init and no update** (`for (; t; )`,
  verified correct above), when an update-free faithful loop is what is wanted.
- **Confidence**: high on behavior and high on mechanism (both halves traced to named
  lines; the arity table predicts all three observed failures and the two observed
  successes).

---

## Tier 2 — silently produces a wrong value

### R-06: `var` / `let` object and array literal initializers are dropped wholesale; `const` works

- **Folds in**: sweep A's out-of-surface sighting (rated by sweep A above all of its own
  findings).
- **Verification**: `sweep-only` (both scopes).
- **Root-cause group**: G7 (binding storage: `const` inlined, non-`const` composite
  initializers lost).
- **Repro**: `var o={f:7}; console.log(o.f);` → node `7`, kali `0` (exit 0).
- **Detail**: `var o={a:7,b:9}` → both fields `0`. `var a=[7,9]; a[0]`→`0`, `a[1]`→`0`. String
  values too: `var o={f:"hi"}; o.f` → `0`. `let` behaves identically to `var`.
  `const o={f:7}` → `7` ✓ and `const a=[7,9]` → `7` ✓.
- **Scopes affected**: both — `function g(){var o={f:7}; return o.f;} g()` → `0`.
- **Why this is NOT the known module-scope element-store defect**: it affects both scopes, it
  is the *initializer* that is lost rather than a later store, and assigning after declaration
  **repairs** it (`var o={f:false}; o.f=true; if(o.f)` → `T` ✓) — the opposite polarity from
  the known defect.
- **Severity**: silent-wrong-value.
- **Blast radius**: very high. This silently returns `0` at exit 0 in the single most common
  object shape in JS.
- **Cross-sweep link**: R-02's boundary shows the *same* polarity for function values
  (`const g = <fn literal>` correct, `let`/`var` → `0`). Two sweeps found the same
  `const`-works / `let`-`var`-lose-the-initializer split on unrelated surfaces. See G7.
- **Confidence**: high on behavior; mechanism not investigated.
- **STATUS — objects-half CLOSED 2026-07-24** (branch `r06-object-init-materialization`, commits `acf7c5c2c`..`3146b9653`). Fix is entirely in `crates/kali_types/src/repr_infer.rs`: a new `mutable_object_literal_bindings: BTreeSet<ObjSlot>` records every non-`const` object-literal declarator binding; a read-materialization block in `resolve_objects` marks such a binding materialized on a field READ (the treatment a write already gave it), so it lowers through the real `Repr::Object` allocation instead of the silent-`0` fold fallback. `const` is absent from the set → byte-identical fold-first lowering (verified 0-newly-red, `const o={f:7}`→`7`, `const o={f:true}`→`1` unchanged).
  - **Admission is an ALLOWLIST at the materialization choke** (`object_field_value_is_safe_for_materialization`), NOT a denylist: a mutable object-literal binding materializes ONLY IF every field value provably lowers to a safe repr — a numeric literal, a string literal, or a unary `+`/`-` on a **numeric** literal. Everything else fails the WHOLE binding closed with `E5506`: Boolean in any form (literal, variable, `!x`, comparison, logical), BigInt literal, `null`/`undefined`, unary `+`/`-` on a **string** literal, numeric/string **expressions** and identifiers (honest over-deny), nested object/array, function. This closed two review-caught fail-opens (see below).
  - **Falsifies G7's "R-06 falls out of the R-07 fix" inference**: R-07 is fixed and R-06 still reproduced on fresh `main`, so R-06 was an independent defect (a fold-vs-materialize gap: read-only mutable objects were neither foldable — not `const` — nor materialized — no write), not a symptom of R-07.
  - **Two whole-stage-review CRITICALs (the signature "denylist leaks; only an allowlist at the choke closes the class" lesson, twice):** (1) an initial bare-`Literal(Boolean)` denylist leaked — `var t=true; var o={f:t}`, `{f:!0}`, `{f:1>0}` → new nonzero-wrong `1`; and `{f:7n}` → `7`. Converted to the allowlist above. (2) the allowlist's unary arm recursed into ITS argument unconditionally, admitting unary-`+`-on-string: `{f:+"hi"}`→`617` (node NaN), `{f:+"3.5"}`→`285`; decimal strings `{f:+"3"}`→3 coincidentally matched and masked it. Closed by restricting the unary operand to a numeric literal.
  - **Residuals (out of scope this stage; left no-worse, tracked):**
    - **R-06-R1 — returned/escaping objects.** `function h(){var o={f:7}; return o;} h().f` → silent-`0` today (the member-on-call hole, R-14 territory) — even for `const`/write objects. Verified no-worse (no new crash, no new nonzero) after this fix. Real fix = R-14 escape stage.
    - **R-06-R2 — whole-object reassignment.** `var o={f:1}; o={f:2}; o.f` → silent-`0`; the object-literal-RHS assignment store is a distinct mechanism from the declarator init. Unchanged. **Re-measured on merged `main` (`372a3f440`) 2026-07-25, and the `let` spelling measures IDENTICAL**: `var o={f:1}; o={f:2}; o.f` → `0` (node `2`) and `let o={a:6}; o={a:9}; o.a` → `0` (node `9`), both exit 0, no diagnostic. `var` and `let` are one lane here, not two — see §7.10, where the `let` sighting is now a cross-reference to this residual.
    - **R-06-R3 — arrays.** `var a=[7,9]` / `var a=[1,2]; a[0]=9` read back `0` — var-array runtime storage largely unimplemented. Own later stage (entangled with R-12/R-13/arena lanes). **Re-measured on merged `main` (`372a3f440`) 2026-07-25; on the indexing and `.length` shapes the `let` spelling measures IDENTICAL to the `var` one, so `var` and `let` are ONE lane there:**
      - store: `var a=[1,2]; a[0]=9; a[0]` → `0` and `let a=[1,2]; a[0]=9; a[0]` → `0`, node `9` for both; `let a=[1,2,3]; a[1]=5; a[1]` → `0`, node `5`.
      - element read: `let a=[1,2,3]; a[0]` → `0`, node `1`.
      - **`.length` read (datum this residual previously lacked): `let a=[1,2,3]; a.length` → `0`, node `3`.** So the binding does not merely lose its stores — the whole thing reads back as an empty/zero array.
      - **`const` is NOT a clean control — it is correct on READS only.** `const a=[1,2,3]; a[0]` → `1` ✓ and `a.length` → `3` ✓, but a module-scope `const` **store** is silently dropped exactly like `var`/`let`, only from a *correct* starting value rather than `0`: `const a=[1,2]; a[0]=9; a[0]` → **`1`** (node `9`) and `const a=[1,2,3]; a[1]=5; a[1]` → **`2`** (node `5`), both exit 0, no diagnostic. In-function the `const` store instead fails closed `E5506` — as do `let` and `var`. **See R-12**, whose entry and §0.2 row record the same scope-not-declarator discriminator. (Correction 2026-07-25: an earlier revision of this residual said "`const` is the discriminator that behaves correctly", which is false for stores.)
      - **the `for..of` consumer is where `let` and `var` DIVERGE**, so the one-lane claim above is scoped to indexing/`.length`: on the same `let`-array storage gap `for..of` is worse than `0` — it iterates the binding's NAME — while the `var` spelling fails closed `E5506`. That is **R-47**, promoted to its own entry in Tier 2.
    - **R-06-R4 — object string-field value-SINK corruption (PRE-EXISTING, const-reproducible; broader than first thought).** A materialized object's String field reads back correctly ONLY in sole-`console.log`-arg / `==` / assignment / return positions; it CORRUPTS to its raw i64 handle through `+` concat, template `${}`, multi-arg `console.log`, and `.length` — e.g. `console.log("x"+o.f)` → `x-9223354444668731390`. `const o={f:"hi"}; console.log("x"+o.f)` corrupts IDENTICALLY (const never touches R-06), proving it is a downstream sink bug, not something R-06 introduces in kind; R-06 merely routes read-only var string objects to the same broken sinks. Its real fix is an object-field-String repr/sink stage. (Single-arg string fields ARE supported and shipped green.)
    - **R-06-R5 — non-literal-valued fields honest over-deny.** `var n=5; var o={a:n}`, `var o={f:3+4}`, `var o={f:null}`, leading-dot float `{f:.5}` → `E5506` even though several would read correctly if materialized. The literal-only allowlist is conservative by design (default-deny on unprovable repr). A later refinement can query each field value's inferred repr and admit provably-{I64,F64,String} non-literals.

### R-07: `const` is not a binding — its initializer is re-emitted at every read site (CRITICAL)

- **Folds in**: D-B-1, **and the previously-registered "`const a = bump()` double-evaluates"**,
  which is a *symptom* of this defect, not an independent bug.
- **Verification**: `CONFIRMED-BY-CONTROLLER` (swap repro).
- **Root-cause group**: G7.
- This is not double evaluation. It is **textual re-evaluation of the initializer expression
  at every read**, so (a) side effects fire once per read, and (b) the value read is computed
  from the **current** values of any variables the initializer mentions, not the values at
  binding time.
- **Repro A — classic swap** (`sweep-b/p47_const_swap.js`), in-function:
  ```js
  function t() {
    let a = 1, b = 2;
    const tmp = a;
    a = b;
    b = tmp;
    console.log("a=" + a + " b=" + b);
  }
  t();
  ```
  **node**: `a=2 b=1` (exit 0) — **kali**: `a=2 b=2` (exit 0).
- **Repro B — stale read**, top level (`p04_stale.js`):
  `let n = 5; const x = n; n = 99; console.log("x=" + x);` → node `x=5`, kali `x=99`.
- **Repro C — `const` over a param, param later reassigned** (`p45_const_param.js`):
  ```js
  function f(x) { const y = x; x = 99; return y; }             // node 5,  kali 99
  function g(x) { const y = x * 2; x = 99; return y + y; }     // node 20, kali 396
  function h(a, b) { const s = a + b; a = 0; b = 0; return s; } // node 3, kali 0
  ```
  `g` shows both failure modes at once: `y` is read twice and each read recomputes `x*2` with
  the *new* `x` → `99*2 + 99*2 = 396`.
- **Repro D — loop-carried temp** (`p46_const_loopcarry.js`):
  `let i=0, acc=0; while (i<3) { const cur=i; i=i+1; acc=acc+cur; }` → node `acc=3`,
  kali `acc=6` (`cur` is read *after* `i` was bumped).
- **Repro E — side effects scale with read count** (`p03_multiread.js`): `const x = bump();`
  then 3 reads → node `n=1`, kali `n=4`. With **zero** reads kali is correct (`n=1`) — which
  is exactly why the old "double-evaluates" framing understated the defect.
- **Repro F — shape survey** (`p06_shapes.js`): every non-literal initializer form is affected
  — identifier, binary, unary, parenthesized, ternary.
- **Scopes affected**: both, verified independently.
- **Not affected** (bounds the damage): `let` and `var` are correct in every shape probed; a
  `const` read in the same iteration with no intervening mutation is correct; a `const` bound
  to a literal is correct.
- **Severity**: silent-wrong-value, escalating to silent-wrong-control-flow via
  `if (constFlag)`.
- **Blast radius**: **maximal.** `const tmp = a`, `const old = this.x`, `const n = arr.length`,
  `const start = Date.now()` — every snapshot idiom in idiomatic JS is wrong. Existing
  fixtures escape it only because they were written to suit the compiler.
- **Mechanism**: `crates/kali_codegen/src/emit/control_flow.rs:1284-1286` — a `const`
  declarator that did not receive a local slot does
  `self.bindings.insert(name, declarator.children[1]); … Drop`, storing the **initializer LIR
  node id** instead of a value. The identifier read path at
  `crates/kali_codegen/src/emit/control_flow.rs:1614-1616` then does
  `if let Some(bound) = self.bindings.get(text) { return self.emit_node(function, bound, want_value) }`
  — re-emitting the initializer inline at the use site with **no purity gate**. Note the
  asymmetry: the module-scope inline path 20 lines below (`:1625-1628`) *does* gate on
  `is_pure_module_const_init(init, 0)`. The local-`const` path has no gate at all.
- **Confidence**: high on behavior (8 transcripts, both scopes, + controller re-run); high on
  mechanism — the two sites explain every observation including "zero reads ⇒ correct" and the
  purity asymmetry.

### R-08: `===` conflates `null`, `undefined`, `false` and `0`; `??` treats `0`/`false` as nullish

- **Folds in**: D-B-3 + D-B-4 (sweep B states they share the root; both are the scalar-`0`
  conflation seen from two operators).
- **Verification**: `CONFIRMED-BY-CONTROLLER` (`0===null` → `true`, `0===false` → `true`).
- **Root-cause group**: G4 (no value distinct from scalar `0`).
- **Repro** (`p54_nulleq.js`, top level):
  ```js
  console.log("1=" + (0 === null));
  console.log("2=" + (0 === undefined));
  console.log("3=" + (false === null));
  let z = 0;
  console.log("4=" + (z === null));
  ```
  **node**: `1=false 2=false 3=false 4=false` — **kali**: `1=true 2=true 3=true 4=true` (exit 0).
- **Control-flow form** (`p53_nullguard.js`, the realistic shape):
  ```js
  function t(x) { if (x === null) { return "isnull"; } return "notnull"; }
  let u;
  console.log("1=" + t(u));    // node notnull, kali isnull
  console.log("2=" + t(null)); // node isnull,  kali isnull
  console.log("3=" + t(0));    // node notnull, kali isnull   <-- 0 mistaken for null
  ```
- Also `true === 1` → kali `true` (node `false`); `false === 0` → kali `true` (node `false`);
  `null !== undefined` → kali `false` (node `true`).
- **`??` half** (`p21_nullish2.js`): `let a=0; a ?? 9` → kali `9` (node `0`);
  `0 ?? 9` → kali `9`; `let f=false; f ?? 9` → kali `9` (node `false`). kali makes `??`
  behave as `||`, defeating the entire purpose of the operator.
- **Scopes affected**: both, for both halves.
- **Severity**: silent-wrong-value **and** silent-wrong-control-flow — the worst combination,
  because the program takes a whole different path and still exits 0.
- **Blast radius**: very high. `if (x === null) return default;` and `if (v === undefined)`
  are everywhere; under kali they fire for the perfectly valid value `0`. Any "0 is a legal
  value, null means absent" API is inverted.
- **Mechanism hypothesis**: `null`, `undefined` and `false` all lower to the scalar `0`, and
  `===` on scalars is a plain `i64.eq` with no tag discrimination. **The `??=` lowering
  carries an explicit `E5506` admitting exactly this** ("null and 0 are indistinguishable for
  a scalar value") — so the unsoundness is *known* at that one site and fails **open**
  everywhere else, including in the plain `??` operator. Not code-located.
- **Confidence**: high on behavior; medium on mechanism (the `??=` diagnostic text is strong
  corroboration). Raising it: find the `===` emit arm and confirm there is no repr guard.

- **UPDATE 2026-07-19 (soundness-batch1-pra, commit `4949d79ec`, "fix 4"): the `===`/`!==`/
  `==`/`!=` majority of this entry is CLOSED.** `crates/kali_codegen/src/emit/equality.rs`
  now classifies both operands into a compile-time JS type class (`EqClass`) and decides by
  TYPE rather than bit pattern: `0 === null`, `0 === false`, `true === 1` all now match node.
  **The `??` half is CLOSED ONLY where the compiler can PROVE a type class for the left
  operand — see residual 4 below for the precise proof condition and its non-exhaustive
  illustrations, and residual 5 for a second, independent way `??` still diverges from node
  even when that proof succeeds.** (An earlier version of this addendum claimed `??` was
  "closed for a literal or a `const`-bound operand"; that headline generalized past what the
  mechanism actually proves and was falsified by probing — see residual 4.) Re-verified on a
  freshly built binary as part of this addendum (2026-07-19): `console.log("1=" + (0 === null))`
  → `1=false` (was `1=true`); `0 ?? 9` → `0` (matches node); `const c = 0; c ?? 9` → `0` (matches
  node). Pinned by `crates/kali_cli/tests/soundness_strict_equality.rs` (12+ tests).

  **This entry is NOT fully closed.** Fix 4 documents (in `equality.rs`'s own doc comments) and
  this wave (soundness-batch1-pra wave 0, across four addendum rounds) additionally pins six
  residuals. Residuals 1-4 exist because kali cannot prove a `Repr::Boolean` axis for an
  arbitrary expression and the type-directed table therefore leaves the pre-existing unsound
  bit-pattern `i64.eq` in place rather than regressing a large swath of the corpus by failing
  everything closed; residuals 5 and 6 are independent print-sink defects that fire even when the
  type-directed table's/`??`'s branch decision is correct — **residual 5 is single-argument
  `console.log`-only (R-30's own mechanism, closes when R-30 closes) and residual 6 is the
  string-concat and multi-argument console lanes, a genuinely `??`-specific defect that does
  NOT close with R-30** (round 4 correction — round 3 wrongly retired residual 6's work as a
  duplicate of residual 5/R-30; see residual 6 below for why they are different):

  1. An `UntypedObjectField` operand (an object-shape field with the untyped `I64` repr, which
     may hold a pointer, a number or a boolean) against a proven `null`/`undefined`/boolean
     keeps the pre-existing lowering rather than proving anything.
  2. An unprovable operand against a proven **boolean** (`f() === true` where `f`'s return type
     is not provable) keeps the pre-existing lowering. Cost of closing it: 33 pinned corpus
     programs of the shape `Object.is(a, b) !== true`. Pinned by
     `unprovable_operand_against_boolean_is_a_known_residual`.
  3. **CRITICAL-2 (new finding, this wave)**: an unprovable operand against a proven **number**
     — including a bare number LITERAL — never even reaches the decision table, because
     `EqClass::arms_the_gate` (the gate that decides whether the type-directed machinery
     engages at all) recognizes only `null`/`undefined`/boolean, not `Number`. Repro,
     re-verified on a freshly built binary:
     ```js
     function f(b) { return b; }
     if (f(false) === 0) { console.log(111); } else { console.log(222); }
     ```
     kali prints `111` (exit 0) — node prints `222` (exit 0). `f(false)`'s parameter is
     unprovable and `0` is a proven `Number` literal, so `arms_the_gate()` is `false` for both
     sides and `equality_decision` returns `Runtime` at its very first check, before the
     asymmetric one-side-classified branch that handles residuals 1 and 2 is ever reached. This
     is wrong CONTROL FLOW (a whole different `if` branch taken), not just a wrong printed
     value, at exit 0 with no diagnostic — the same severity class the rest of R-08 was in
     before fix 4. Pinned honestly (as a residual, not a correctness claim) by
     `unprovable_operand_against_number_literal_is_a_known_residual` in
     `soundness_strict_equality.rs`. **Not fixed in this wave** — the real fix needs the same
     `Repr::Boolean` axis residual 2 is blocked on; this is inventory + pin only, per maintainer
     ruling.
  4. **CRITICAL — restated 2026-07-19 (second addendum round) as a MECHANISM, not a shape
     list, after a round-2 probe falsified the round-1 restatement of this residual** (round 1
     claimed, in the entry headline above, that `??` was "closed for a literal or a
     `const`-bound operand"; that is a *description of two symptoms*, not the proof condition,
     and round-2 probing found counterexamples the headline's own words technically permitted
     — see family (a) below).
     **The actual proof condition**: `??`'s left-operand branch is decided at compile time,
     correctly, if and only if `static_equality_class`
     (`crates/kali_codegen/src/emit/equality.rs:228`) returns `Some(class)` for it, AND that
     class actually arms `??`'s check (`is_nullish_class`/`is_never_nullish`,
     `operators.rs:2181-2208`) — see the `UntypedObjectField` caveat below for one of **three**
     places those two conditions come apart (**corrected 2026-07-19, round 4**: `equality.rs:
     140-152` shows `is_never_nullish` covers only `Number | BigInt | Boolean | String` and
     `is_nullish_class` only `Null | Undefined`, so `ObjectOrNull` and `EnvGetResult` are ALSO
     `Some` without arming the gate — three non-arming classes, not one. No miscompile follows
     for the other two: the runtime `i64.eqz` zero-test is independently exact for an object
     pointer (`ObjectOrNull`) and for a `Deno.env.get` unset-`0` result (`EnvGetResult`), so the
     *outcome* only diverges for `UntypedObjectField`).
     **Corrected 2026-07-19 (third addendum round): a round-2 restatement of this condition as
     "exactly two cases" was itself an UNDER-claim** — verified false on a freshly built binary
     (`(a < b) ?? 9` over two function PARAMETERS, and `(a - a) ?? 9` over a `let`-bound float,
     both agree with node) — `static_equality_class` returns `Some` for considerably more than a
     literal or a literal-terminated `const` chain; reading `equality.rs:228-329` end to end, the
     full set is:
     - (i) a literal, or an identifier whose ENTIRE initializer chain resolves, at compile time,
       all the way down to such a literal, via `resolve_literal_aggregate`/`self.bindings` (the
       `const`-alias chain) — round 2's two cases, still correct as far as they go.
     - (ii) an operand-INDEPENDENT operator form: the unary `void`, `!`, `typeof`, `delete`,
       numeric `-`/`~` round 2 already listed, **plus every relational/equality operator** (`<
       <= > >= == != === !== in instanceof`, `equality.rs:280-289`) — these always produce a
       `Boolean` regardless of what their operands are, which is exactly why `(a < b) ?? 9` over
       two unprovable parameters is proven.
     - (iii) a statically-folded CALL result whose rendered text is
       `"true"`/`"false"`/`"undefined"`/`"null"` (`equality.rs:297-304`), via
       `render_static_value` (`crates/kali_codegen/src/intrinsics/host.rs:358-411`) — e.g.
       `arr.at(oob)`/`str.at(oob)`/`str.codePointAt(oob)` (`"undefined"` on an out-of-range
       index) or `Object.freeze(<literal>)` recursing into a literal operand.
       **Corrected 2026-07-19 (round 4): the previous three rounds' named examples for this case
       — `Object.is(a, b)` and the `Number.is*` predicates — are WRONG.** Read end to end,
       `render_static_value`'s `Call` arm (`host.rs:375-411`) has no case for either: it only
       folds `Object.freeze`, `arr.at`/`str.at`/`str.codePointAt`, and `require`/semver calls.
       `Object.is`/`Number.isFinite`/`isNaN`/`isInteger`/`isSafeInteger` DO get
       `shape: ValueShape::Boolean` when actually emitted (`crates/kali_codegen/src/emit/
       call.rs:1398-1494` and `:1496-1559`), but that is a completely different code path from
       `static_equality_class`'s textual fold, and the two disagree: `static_equality_class`
       returns `None` for these calls, not `Some(Boolean)`. This is the exact mechanism behind
       residual 6 below — traced while investigating that residual, not asserted from the old
       text.
     - (iv) a bare global identifier lowered as a childless `Value` node — `undefined`, `NaN`,
       `Infinity` (`equality.rs:307-313`).
     - (v) a REPR-BACKED proof, which is what makes `(a - a) ?? 9` provable even though `a` is a
       genuine runtime `let` slot: an object-shaped value (`object_shape_of_node`), a bigint-
       literal-valued node, a float-valued node, a string-valued node, or a
       `Deno.env.get(...)` result (`is_env_get_string_call`) — none of these require the operand
       to be a literal or a `const`, only that the codegen repr proves the JS type.
     - (vi) a `base.field` read whose shape-table repr is a TYPED float, string, or object field
       (`object_field_equality_class`) — **witness required, round 4**: a bare object-literal
       `const` binding never reaches this arm at all (see the reverted illustration below), so
       this case needs a base whose SHAPE is independently resolved. Verified witness: routing a
       shape-tracked object through a same-shape function PARAMETER —
       ```js
       function mk() { return { a: 1.5 }; }
       function chk(o) { return o.a === null; }
       console.log(chk(mk()));   // kali 0 (exit 0), node false — classified, not E5506
       ```
       — proves this arm is real code, reachable, and behaves as documented (contrast the
       direct-binding case, which fails closed with `E5506` because it never gets here). A `??`
       witness over the identical shape (`o.a ?? 9` in place of `o.a === null`) instead hits an
       unrelated `error[E4201]` (malformed WASM) both as a function return and as a parameter
       read — a separate, pre-existing defect in this arm's `??` interaction with a typed float
       field, out of scope for this round; not chased further here. But **not** the untyped
       `I64` default: that case still
       returns `Some`, just of the special `UntypedObjectField` class, which
       `is_nullish_class`/`is_never_nullish` both reject (`equality.rs:345-348`,
       `operators.rs:2201-2208`), so it falls through to the runtime `i64.eqz` test exactly as if
       it had returned `None`. **This is genuinely true for an object whose SHAPE is resolved**
       (e.g. the CLBG binary-trees `{ left, right }` case) — but it does NOT apply to the
       const-bound member-read illustration below. **Reverted 2026-07-19 (round 4): round 3
       changed this illustration's classification from `None` to
       `Some(EqClass::UntypedObjectField)`; round 2 was correct and round 3's change is false,
       verified on a freshly built binary.** `const o = { a: 0 }; console.log(o.a === null)`
       fails CLOSED with `E5506` (exit 1) — if the class were `Some(UntypedObjectField)`,
       `strict_decision`'s `is_unproven` arm (`equality.rs:184`) would route it to `Runtime`
       (silent bit-pattern compare, exit 0), not `FailClosed`. The actual class is `None`:
       `object_field_equality_class` requires `object_shape_of_node(base)` to resolve
       (`equality.rs:334-351`), which for a bare identifier bottoms out in `scalar_repr(name)`
       being `Repr::Object(shape)` (`crates/kali_codegen/src/emit/object.rs:14-25`) — and a
       `const` bound directly to an object literal is never given a resolved shape this way (no
       write/escape/call-return path materializes it). This is a **separate gap** from the
       untyped-object-field residual elsewhere in this register (residual 1 / R-08's
       `UntypedObjectField` note): arming `UntypedObjectField` would still leave
       `const o={a:0}; o.a ?? 9` broken, because that class is never reached for this program in
       the first place. (This is the same mis-grouping error round 4 exists to fix in residual 5
       — a real defect wrongly retired by asserting it is "the same as" a sibling that, on
       inspection, is never reached.)
     Anything else — any operand read back from a runtime storage slot that is none of (i)-(vi)
     above (a plain `let`/`var`/parameter/call-return binding with no repr proof, or the untyped-
     I64-field case in (vi)) — returns `None` (or `Some(UntypedObjectField)`, which behaves
     identically to `None` for `??`), and `operators.rs`'s `??` arm falls through to the
     pre-existing `i64.eqz` bit-pattern test, which conflates a runtime `0`/`false` with nullish
     (`??` degrades to `||`). **The shape lists below (this round's and round 1's) are non-
     exhaustive illustrations of that one rule — not an enumeration of what is broken; do not
     read either list as a boundary.**
     - **Illustration set 1 (round 1, still valid): a genuine runtime slot, no `const` in the
       chain at all.** A `let`-bound, `var`-bound, function-PARAMETER, or call-RETURN-VALUE
       operand. Re-verified on a freshly built binary (2026-07-19):
       ```js
       let a = 0;
       console.log(a ?? 9);                          // kali 9,  node 0
       var v = 0;
       console.log(v ?? 9);                          // kali 9,  node 0
       function opt(n) { return n ?? 10; }
       console.log(opt(0));                           // kali 10, node 0
       function zero() { return 0; }
       console.log(zero() ?? 9);                      // kali 9,  node 0
       ```
       Pinned by `nullish_coalescing_over_let_binding_is_a_known_residual`,
       `nullish_coalescing_over_var_binding_is_a_known_residual`,
       `nullish_coalescing_over_parameter_is_a_known_residual`, and
       `nullish_coalescing_over_call_return_is_a_known_residual` in
       `soundness_strict_equality.rs` (all four now pinned; previously only the `let` and
       parameter shapes were pinned while the header prose also claimed `var` and call-return —
       that prose/pin mismatch is fixed by adding the two missing pins, not by narrowing the
       prose).
     - **Illustration set 2, FAMILY (a) (new this round): a `const` binding IS present, but its
       initializer chain does not bottom out at a literal.** `resolve_literal_aggregate` will
       follow a `const`'s binding, but if what sits at the end of the chain is a call, a folded
       runtime expression, or a further (non-literal) binding, `static_equality_class` still
       returns `None` there — `const` the keyword proves nothing by itself; only a chain that
       terminates in a literal does. The fourth shape (an object-field read) reaches the SAME
       `None` outcome for a different reason (**reverted 2026-07-19, round 4 — see the
       `UntypedObjectField` caveat above for the full correction**): `o.a` where field `a` only
       ever holds the untyped integer literal `0` returns plain `None`, because
       `object_field_equality_class` never even fires for it — `o`'s base is a const-bound
       object LITERAL, whose shape is never resolved (`object_shape_of_node` requires
       `scalar_repr("o")` to be `Repr::Object(shape)`, which a bare object-literal binding never
       gets). This is precisely what falsifies
       the round-1 headline ("closed for a literal or a `const`-bound operand"): all four operands
       below ARE `const`-bound, and all four are still wrong. Re-verified on a freshly built
       binary (2026-07-19), all four shapes, exit 0, no diagnostic, kali `9` vs node `0`:
       ```js
       function zero() { return 0; }
       const c1 = zero();      console.log(c1 ?? 9);   // const bound to a CALL result
       const c2 = 1 - 1;       console.log(c2 ?? 9);   // const bound to a FOLDED expression
       let d = 0;
       const c3 = d;           console.log(c3 ?? 9);   // const bound to a LET-ALIAS
       const o = { a: 0 };     console.log(o.a ?? 9);   // const-bound MEMBER READ
       ```
       Pinned by `nullish_coalescing_over_const_bound_call_result_is_a_known_residual`,
       `nullish_coalescing_over_const_bound_folded_expression_is_a_known_residual`,
       `nullish_coalescing_over_const_bound_let_alias_is_a_known_residual`, and
       `nullish_coalescing_over_const_bound_member_read_is_a_known_residual` in
       `soundness_strict_equality.rs`. By contrast, `const c = 0; c ?? 9` → kali `0` (matches
       node) — a chain of length one that terminates directly at a literal, which IS proven.
     Neither illustration set is fixed in this wave — both need the same `Repr::Boolean`/null-
     axis architectural blocker as residuals 2 and 3; per maintainer ruling, do not attempt it
     here.
     - **Blast radius: LARGER than residuals 2 and 3.** Residuals 2 and 3 are triggered by
       comparatively narrow shapes (a proven-boolean or proven-number-literal compare against an
       unprovable operand). `x ?? default` over anything that isn't a literal or a
       literal-terminated `const` chain is `??`'s ORDINARY usage — this is the common case of the
       operator in idiomatic JS, not an edge case.
  - **Severity of the residual, downgraded from the original entry — but ONLY for the
    `===`/`!==`/`==`/`!=` half.** For those operators it is no longer "every null-guard in every
    program"; narrowed to residuals 1-3 above (an untyped object field, an unprovable-vs-boolean
    compare, or a proven-number operand compared against an operand whose type kali cannot prove
    at compile time). **The `??` half is NOT downgraded**: residual 4 above is `??`'s ordinary-
    usage shape, so for `??` the original severity recorded at the top of this entry — silent-
    wrong-value **and** silent-wrong-control-flow, the worst combination — still stands,
    essentially untouched by fix 4. Residuals 5 and 6 below are further, independent
    divergences on top of the cases residual 4 *does* prove correctly (a print-sink rendering
    defect, not a value/control-flow defect — the in-memory branch selection stays correct in
    both).

  5. **FAMILY (b), single-argument `console.log` ONLY (scope corrected 2026-07-19, round 4 — see
     residual 6 below for the part of family (b) this scope-narrowing carves OUT): a `??` whose
     selected result is a BOOLEAN loses its boolean-ness at the single-argument print sink, for
     every binding kind including a bare literal operand — even when `??`'s branch selection is
     itself correct.**
     **Corrected 2026-07-19 (third addendum round): this IS R-30 ("Computed booleans render
     `1`/`0` in direct `console.log` argument position", Tier 4 below) observed through `??`,
     not a `??`-specific defect** — the round-2 mechanism trace immediately below (no `Boolean`
     shape arm in the single-argument console sink) is correct, but it is R-30's mechanism
     verbatim, and `??` is simply one more producer feeding it: `??`'s branch decision hands the
     console sink a provably-boolean value the same way a bare `!`/comparison/ternary result
     does, and the sink drops the shape identically in every case. Residual 5 therefore **closes
     when R-30 closes** (the console-formatter-unification fix, priority row 9 in this
     register's fix-priority table) — it is **not** blocked on the `Repr::Boolean`/null axis
     that blocks residuals 2-4, and no `??`-specific work is needed for it. `??` has been added
     to R-30's producer list below. This is not a proof-condition gap in `static_equality_class`;
     it fires ON TOP OF a correct decision. Mechanism: when `??`'s left operand is provably
     `Boolean`-classed (never nullish) or the branch resolves to a provably `Boolean`-classed
     right operand, the selected
     operand's `EmittedValue` correctly carries `shape: ValueShape::Boolean` (via
     `selected_nullish_operand`, `equality.rs:433-436`). But the SINGLE-ARGUMENT
     `console.log`/`.error`/`.warn`/`.info` sink (`emit_console_argument`,
     `crates/kali_codegen/src/emit/call.rs:23-41`) — which is what a `??` expression falls to
     whenever the WHOLE call isn't statically renderable — never inspects `shape` except for
     `Float`; it hands the raw i64 straight to the host import, which does `value.to_string()`
     for anything that is not a string handle
     (`crates/kali_runtime/src/host/io.rs::format_console_value`). A bare `console.log(false)`
     prints correctly ONLY because the entire call is folded to the literal string `"false"` by
     a SEPARATE, independent constant-folder (`render_console_call`/`render_static_value`,
     `crates/kali_codegen/src/intrinsics/host.rs:345-`), which has no case for a `??` (or any
     other binary-operator) node and therefore never folds a `??` expression at all — the same
     "hand-mirrored oracle" class of bug this repo has hit before (two independent notions of
     "is this a boolean" — `??`'s own branch decision and console's static-fold decision — that
     disagree).
     **The multi-argument console lane and string-concat, for a PROVABLE operand ONLY (a bare
     literal, or anything else that satisfies (i)-(vi) above), DO honor `shape: Boolean` and are
     NOT affected by residual 5** — `console.log("x:", false ?? 9)` correctly prints `x: false`.
     Re-verified on a freshly built binary (2026-07-19):
     ```js
     console.log(false ?? 9);        // kali 0, node false — left operand selected, provably Boolean
     console.log(true ?? 9);         // kali 1, node true
     console.log(null ?? false);     // kali 0, node false — right operand selected, provably Boolean
     console.log(null ?? true);      // kali 1, node true
     console.log("x:", false ?? 9);  // kali "x: false", node "x: false" — multi-arg lane is fine
                                      // for a PROVABLE operand (see residual 6 for the
                                      // UNPROVABLE-operand case, which this pin does NOT cover)
     ```
     Pinned honestly (recording current WRONG behaviour, not a correctness claim) by
     `nullish_coalescing_boolean_literal_result_loses_shape_is_a_known_residual` and
     `nullish_coalescing_right_operand_boolean_loses_shape_is_a_known_residual` in
     `soundness_strict_equality.rs`. **Not fixed in this wave** — but, per the correction above,
     it is **not** blocked on the `Repr::Boolean`/null-axis architectural blocker that covers
     the rest of this entry; it is blocked on R-30's own fix (unify the two console formatters).
     The note above is diagnostic (single-argument console sink lacks a `Boolean` shape arm and
     the static console folder has no `??` arm), not a repair.
     - **Note the masking hazard this residual corrects**: the pre-existing
       `nullish_coalescing_does_not_treat_falsy_as_nullish` test's `n3` case
       (`"n3:" + (false ?? true))`) routes through string concatenation over a PROVABLE (literal)
       operand, i.e. `emit_as_string`'s correct path, and passed throughout both prior rounds —
       which is exactly why a green suite did not surface this residual until it was probed
       directly through the single-argument sink. **Round 4 correction: do not read this as
       "concat is unconditionally fine" — see residual 6, which is exactly the case this masking
       note's own logic predicts once the operand stops being provable.**

  6. **`??`-SPECIFIC (new residual, round 4 — split out of what round 3 wrongly retired as "the
     same as residual 5 / R-30"; scope corrected round 5, see the note below the repro): the
     string-concat (`+`) and multi-argument `console.log` lanes ALSO lose a `??` result's
     boolean-ness, whenever the LEFT OPERAND is a CALL whose OWN emission already tags its result
     `shape: ValueShape::Boolean` (a hand-cased intrinsic such as `Number.isInteger`/`Object.is`)
     but which `static_equality_class` cannot prove — this fires ON TOP OF a value the call site
     already got right, and it is blocked on neither R-30's fix nor the `Repr::Boolean`/null axis
     that blocks residuals 1-4.** Verified on a freshly built binary (2026-07-19):
     ```js
     console.log("s:" + (Number.isInteger(5)));       // kali s:true   node s:true   BASELINE OK
     console.log("w:" + (Number.isInteger(5) ?? 9));  // kali w:1      node w:true   DIVERGES
     console.log("x:", Object.is(1,1));               // kali x: true  node x: true  BASELINE OK
     console.log("x:", Object.is(1,1) ?? 9);          // kali x: 1     node x: true  DIVERGES
     ```
     The baselines (no `??`) are correct on exactly the same lanes that diverge once `??` is
     introduced — `??` is what breaks them, and the value it hands to the sink is never a
     `console.log` argument at all in the concat case (`"w:" + (...)`), so this is **not** R-30
     (R-30's own text is explicit that it is the single-argument DIRECT `console.log` position;
     unifying the console formatters, R-30's fix, cannot repair a value that never reaches a
     console sink).
     **Round 5 correction — an `isEven`-style ordinary user function was wrongly added here in
     round 4 as a third pair, annotated `BASELINE OK`. Re-verified on a freshly built binary
     (2026-07-19): the baseline is already wrong** —
     `function isEven(n){return n%2===0;} console.log("a:"+(isEven(4)))` prints kali `a:1`,
     node `a:true`, **with no `??` anywhere in the program**. `??` is therefore not what breaks
     this row, and folding it in here both over-scoped this residual (its "baselines are correct
     until `??` is introduced" conclusion is false for a plain function call) and mis-sent a
     future maintainer (fixing `??`'s own runtime-fallback lowering, this residual's fix, leaves
     `isEven(4) ?? 9` printing `a:1`, because the call's shape was never `Boolean` in the first
     place — see the mechanism correction at step 2 below). The row and the class it actually
     exposes — a boolean-returning **user function**, no `??` involved — are now tracked
     separately as **R-34** (Tier 4, below), which also carries the corrected mechanism trace and
     the reproducers verbatim.
     **Mechanism, traced (not inference) — this is the substantive finding of round 4:**
     1. `??`'s codegen (`crates/kali_codegen/src/emit/operators.rs:2170-2229`) only attaches a
        proven shape to its result via `selected_nullish_operand` on the two PROOF-DRIVEN
        branches (`static_equality_class(left)` returns `Some(class)` that arms the gate,
        `operators.rs:2184` and `:2194`). When `static_equality_class(left)` returns `None`, `??`
        falls to the untyped runtime fallback (`operators.rs:2210-2229`), which **unconditionally
        returns `EmittedValue { shape: ValueShape::Unknown }`** (`:2226-2229`) — it never
        inspects `left_result.shape`/`right_result.shape`, which it already computed one line
        earlier (`:2210`, `:2219`) and simply discards, unlike the sibling bitwise-operator arm a
        few lines above it (`:2153-2159`), which DOES propagate `Boolean` when both operands
        agree. This is a real, if narrow, asymmetry within `??`'s own lowering: the runtime
        fallback throws away shape information it already has in hand.
     2. `Number.isInteger(5)` and `Object.is(1,1)` both hit this fallback because
        `static_equality_class(left)` returns `None` for them — **not** because they are
        "unprovable" in some deep sense, but because of the exact hand-mirrored-oracle gap traced
        under case (iii) above: `static_equality_class`'s only route to prove a CALL result
        Boolean is `render_static_value` (`intrinsics/host.rs:358-411`), and that function's
        `Call` arm has no case for `Object.is`/`Number.is*` — verified by reading its complete
        match arm end to end (only `Object.freeze`, `arr.at`/`str.at`/`str.codePointAt`, and
        `require`/semver fold). Meanwhile the ACTUAL emission of these same calls (`call.rs:1398-
        1494`, `:1496-1559` — their own hand-cased intrinsic arms) correctly reports `shape:
        ValueShape::Boolean` on its own `EmittedValue` — the two are simply different code paths
        that were never kept in sync, the same "hand-mirrored oracle" class this register has
        flagged before (see G5). **Round 5 correction: this does NOT extend to "any ordinary
        function body"** as round 4 claimed — an ordinary user function (e.g. `isEven`) does not
        go through either of these hand-cased intrinsic arms at all. It hits the GENERIC resolved-
        call path (`crates/kali_codegen/src/emit/call.rs:3112-3123`), which computes its
        `EmittedValue.shape` as `ValueShape::Float` when `repr_table.return_repr(callee) ==
        Repr::F64` and `ValueShape::Unknown` otherwise — there is no `Boolean` arm here at all,
        for any function, because `kali_common::Repr` has no `Boolean` variant to test for (see
        step 3 below). So an ordinary function's call-site shape is `Unknown` **before `??` or
        any other consumer ever sees it** — there is no already-correct `Boolean` shape for `??`'s
        fallback (or anything else) to discard. This is why `isEven` does not belong in this
        residual: this residual's mechanism (step 1) is "a value that started `Boolean` gets
        thrown away"; `isEven`'s value never started `Boolean`. See R-34.
     3. Contrast with a CALL that IS provable: `function greet(){return "hi";} greet() ?? "x"`
        prints correctly in concat (`"g:"+(greet()??"x")` → `g:hi`), because `is_string_valued`
        (`crates/kali_codegen/src/emit/operators.rs:1012-1020`) proves `greet`'s return via
        `self.repr_table.return_repr(name) == Repr::String` — a real, whole-program, DATA-FLOW
        repr axis that both `is_string_valued` and `static_equality_class`'s repr-backed section
        (case (v) above) consult directly, independent of any local textual folding. **No
        equivalent axis exists for booleans**: `kali_common::Repr`
        (`crates/kali_common/src/repr.rs:18-38`) has variants for `I64`, `F64`, `Object(ShapeId)`,
        `String`, `GrowableArrayI64`, `AbortHandle` — **no `Boolean`** — so a call's booleanness
        can ONLY ever be proven by `static_equality_class`'s local, syntactic cases (i)-(iv),
        never by a cross-function data-flow proof the way String/Float/Object are. This is the
        same `Repr::Boolean`-axis gap residuals 2-4 are blocked on, but it manifests here as a
        DIFFERENT failure mode (shape loss on a correct decision, not a wrong decision), which is
        exactly why this is tracked as its own residual rather than folded into 2-4.
     4. Downstream, `emit_as_string` (`operators.rs:1537-1572` — the shared coercion ladder used
        by BOTH `+` string concatenation and the multi-argument console lane via
        `emit_console_argument_as_string`, `call.rs:60-69`) keys its boolean-formatting arm
        (`:1561-1564`) on exactly `emitted.shape == ValueShape::Boolean`. Since the `??` node's
        shape is `Unknown` per step 1, that arm is skipped and the value falls to
        `int_to_string`, printing the raw `1`/`0` bit pattern instead of `"true"`/`"false"`.
     Not fixed in this wave (out of scope — "do not attempt to fix `??` itself"). Pinned honestly
     (recording current WRONG behaviour, not a correctness claim) by
     `nullish_coalescing_boolean_result_loses_shape_in_concat_is_a_known_residual` and
     `nullish_coalescing_boolean_result_loses_shape_in_multi_arg_console_is_a_known_residual` in
     `soundness_strict_equality.rs`. **Update trigger: this residual is specific to `??`'s own
     runtime-fallback lowering (step 1 above) — it goes RED when THAT code path starts deriving
     its `EmittedValue.shape` from the operands it already emits, not when R-30 closes and not
     when a `Repr::Boolean` axis lands** (though the latter would also happen to fix it, by
     routing `Number.isInteger`/`Object.is` through the proof-driven branches instead). This
     residual no longer includes `isEven`-style ordinary functions — see the round-5 correction
     above and R-34.

### R-09: `continue` inside a C-style `for` loop skips the update expression

- **Folds in**: D-B-6.
- **Verification**: `sweep-only` (both scopes, both manifestations).
- **Root-cause group**: unclustered (isolated lowering bug).
- **Repro — silent (exit 0) form** (`p28b.js`, in-function):
  ```js
  function t() {
    let s = 0;
    for (let i = 0; i < 6; i++) { if (i === 2) { i++; continue; } s = s + i; }
    console.log("s=" + s);
  }
  t();
  ```
  **node**: `s=10` — **kali**: `s=13` (exit 0). The arithmetic confirms the mechanism exactly:
  node visits `i = 0,1,(2→3 skipped),4,5` ⇒ `0+1+4+5 = 10`; kali, never running `i++` after
  `continue`, visits `0,1,(2→3),3,4,5` ⇒ `0+1+3+4+5 = 13`.
- **Repro — hang form** (`p27a.js`, `p27d.js`):
  `for (let i=0; i<5; i++) { if (i%2===0) continue; s = s + i; }` → node `s=4` (exit 0);
  kali `error[E4003]: CPU fuel budget exhausted` (exit 1) — infinite loop, because the only
  thing advancing `i` is the skipped update.
- **Evidence widened 2026-07-28** on `5c9bbd051` (branch `r35-switch-lowering`, fix round 1),
  while re-deriving the R-35 boundary. The hang is **not** specific to `let`, to `i++`, to a
  `%` test, or to any nesting depth. Seven fixtures, both scopes, **every one** exits 1 with
  `error[E4003]: CPU fuel budget exhausted: the program ran past the runaway guard`:
  | fixture | shape | node |
  |---|---|---|
  | `c13D_mod.js` | `for (var i=0;i<4;i=i+1) { if (i === 1) continue; r = r + 1; }` — bare, un-nested | `r=3` |
  | `c13C_{fn,mod}.js` | the same guarded by an `if`/`else` **block** | `r4=3` / `r=3` |
  | `c13B_{fn,mod}.js` | the same with the `continue` inside a **`switch` clause** | `r4=3` / `r=3` |
  | `c13E_mod.js` | switch form with `let` + `i++` | `r=3` |
  | `c13F_mod.js` | this entry's own recorded hang repro, re-run | `s=4` |
  **R-09 is the owning ID for the `continue`-in-a-`switch`-clause hang**, which the R-35
  boundary re-derivation first recorded (wrongly) as an R-35 cell. It is **independent of
  R-35**: the switch is not in the causal path, the no-switch controls hang identically, and
  no `switch` allowlist can fix it. Cross-referenced from cell 13 of
  `docs/superpowers/followups/r35-switch-boundary-rederived.md`. **No new `R-nn` was minted**
  — the next free ID was R-50, but this entry already covers the mechanism, both
  manifestations and both scopes, and a duplicate would recreate exactly the split-entry
  problem PRs #28 and #29 existed to clean up.
- **Evidence widened again 2026-07-29** on `58234e87c7` (R-35 close-out, Task 11), while
  re-deriving which loop forms a `switch` clause's `continue` may be admitted into. **The
  "Not affected" line below was wrong in TWO directions and is corrected in place.** Both
  corrections are measured switch-free, one argument per `console.log`, per-iteration
  logging, exit status captured unpiped:
  - **`do`/`while` IS affected** — it was listed as unaffected and is not.
    ```js
    var i = 0;
    do {
      i = i + 1;
      console.log("iter=" + i);
      if (i >= 3) { continue; }
    } while (i < 3);
    console.log("i=" + i);
    ```
    **node**: `iter=1` / `iter=2` / `iter=3` / `i=3` (exit 0). **kali**: `iter=1`,
    `iter=2`, `iter=3`, `iter=4`, … runaway to `error[E4003]: CPU fuel budget exhausted`
    (exit 1). The mechanism is the *mirror image* of the `for` case rather than the same
    one: `continue` branches to the loop **top**, and `do`/`while` puts its test at the
    **bottom**, so the test is skipped for that pass and the loop cannot terminate. Same
    owning ID because it is the same root question — *what does `continue` branch to, and
    what sits between there and the back edge* — and splitting it would recreate the
    split-entry problem PRs #28 and #29 existed to clean up.
  - **`for…in` IS affected** and was **missing from the line entirely**, in neither
    direction.
    ```js
    var o = { a: 1, b: 2 };
    var n = 0;
    for (var k in o) {
      console.log("iter=" + k);
      if (k === "a") { continue; }
      n = n + 1;
    }
    console.log("n=" + n);
    ```
    **node**: `iter=a` / `iter=b` / `n=1` (exit 0). **kali**: `iter=a` repeated to
    `error[E4003]` (exit 1) — the key cursor is not advanced past the `continue`.
  - **Re-derived: the genuinely unaffected forms are `while`, `for…of`, and a C-style `for`
    with NO update clause — those three, and no others.** Each verified switch-free with
    the same shape (`if (i === 2) { continue; }` inside a three-iteration loop with a
    per-iteration `console.log`), each matching node byte-for-byte at exit 0. The unifying
    property is stated positively, so it can be checked rather than remembered: **a form is
    faithful exactly when nothing the loop needs for progress or termination sits between
    `continue`'s branch target and the back edge.** `while` has nothing there; `for…of`
    advances its cursor *before* the body; an update-free `for` has no update to skip. A
    C-style `for` *with* an update has the update there; `do`/`while` has the test there;
    `for…in` has the key advance there.
  - This is the predicate `crates/kali_codegen/src/emit/control_flow.rs:348` now encodes
    (`update.is_none() && kind != "do-while"`) and that R-35's `switch` allowlist consumes
    to decide whether a clause's `continue` is admissible (denial constant
    `UNFAITHFUL_CONTINUE`). **`for…in` is covered by that predicate only because `for-in`
    is a different HIR node kind that reaches a different emit path, not because the
    predicate names it** — anyone editing that line must re-check `for…in` explicitly.
    See also **R-52**, which records a coupling in the opposite direction: three `for`
    arities are currently flagged faithful *only because the loops are already broken*.
- **Scopes affected**: both.
- **Not affected**: `continue` in **`while`**, **`for…of`**, and a **C-style `for` with no
  update clause** is correct; `break` in `for`/`for…of`/nested loops is correct.
  **Corrected 2026-07-29** — this line previously read "`while`, `do/while` and `for…of`",
  which was wrong twice over: `do/while` **is** affected (verified above) and `for…in`
  **is** affected and was absent from the line altogether. The original claim was never
  measured; it was asserted alongside the `for` finding.
- **Severity**: silent-wrong-value (the `p28b` form) degrading to a hang when the body does
  not otherwise advance the loop variable.
- **The SILENT variant is the dangerous one and it is easy to miss — R-35 → R-09
  cross-reference.** Readers arriving here from R-35 (whose `switch` clauses may contain a
  `continue`) tend to take away "R-09 is the `E4003` hang", because that is the loud, memorable
  manifestation and the one every R-35-stage fixture hit. **That is only half the entry.**
  When the loop body *also* mutates the counter, the loop still terminates and R-09 produces
  a **wrong answer at exit 0 with no diagnostic** — documented in this entry's own
  "Repro — silent (exit 0) form" above (`s=13` where node gives `s=10`, and the arithmetic
  trace matches digit for digit), and again in the "Blast radius" note below (skip-ahead
  scanners, tokenizers, run-length loops). A `switch` clause that does `i = i + 1; continue;`
  inside a C-style `for` with an update is exactly that shape. R-35's allowlist refuses it
  (`UNFAITHFUL_CONTINUE`) rather than emitting it, so the silent form is currently
  unreachable *through a switch* — but that is a refusal, not a fix, and it evaporates the
  moment anyone widens the faithfulness predicate.
- **Blast radius**: **very high.** `for (…;…;i++) { if (cond) continue; … }` is one of the
  most common loop shapes in JS. Most instances will *hang* rather than mis-answer, which is
  at least loud — but any loop whose body also mutates the counter (skip-ahead scanners,
  tokenizers, run-length loops) silently produces a wrong result at exit 0.
- **Mechanism hypothesis**: `continue` is lowered as a branch to the loop's header/test label
  rather than to a dedicated continue target placed before the update expression. Not located.
- **Confidence**: high on behavior (4 transcripts, both scopes, both manifestations, and the
  arithmetic trace matches digit for digit); medium on mechanism.

### R-10: Block-scoped `let`/`const` shadowing is unmodeled — the inner declaration aliases the outer binding

- **Folds in**: D-C-5.
- **Verification**: `sweep-only` (both scopes) for the full shape inventory, **upgraded to
  `CONFIRMED-BY-CONTROLLER` for the core repro**: the Repro line below was directly re-measured
  on a freshly-built binary at merged `main` (`372a3f440`) against node v26.5.0 on 2026-07-25 and
  still reproduces verbatim (kali `r=2`, node `r=1`), as did the declaration-only form (see §7.10).
- **Root-cause group**: G7.
- **Repro**: `let x = 1; { let x = 2; } console.log("r=" + x);` → node `r=1`, kali `r=2` (exit 0).
- **Worse variant — writes inside the block escape**: `let x=1; { let x=2; x=99; } return x;`
  → node `1`, kali `99`. The inner block's private variable and the outer variable are the
  same storage cell, so ordinary block-local scratch work corrupts the enclosing scope.
- **All block forms affected**: bare block, `if` body (node 1 / kali 2), `for` body
  (node 1 / kali 5), and `const` inner as well as `let`. A later *read* also observes the
  corruption (`let y = x + 10` → node 11, kali 12).
- **Scopes affected**: both, identically.
- **Severity**: silent-wrong-value.
- **Blast radius**: very high and insidious. Reusing a short name like `i`, `x`, `tmp` or `n`
  inside an `if` or loop body is everyday JS, and the corruption is action-at-a-distance with
  no diagnostic.
- **Mechanism hypothesis**: the resolver keys bindings on name within the enclosing *function*
  scope rather than the lexical *block*. Supporting evidence: the `var` analogue fails closed
  with `E3101: duplicate binding 'x'`, suggesting one flat per-function binding table where
  `let` is permitted to re-declare (and therefore overwrite) while `var` is rejected.
- **Correct neighbor**: *parameter* shadowing of a module name is correct; a distinct inner
  name in a loop body is correct. The bug is specifically same-name re-declaration.
- **Confidence**: high on behavior (7 transcripts, both scopes, 4 block forms); medium on
  mechanism.
- **Re-measured 2026-07-25** on `fc777af54`, on the `main`-identical `e416b22a1`, and on merged
  `main` `372a3f440`, in the declaration-only form (`let n = 6; { let n = 7; … }` → `7`/`7`,
  node `7`/`6`) — see §7.10. Identical on all three, so the defect is pre-existing and
  unrelated to R-11: the repro contains no assignment operator at all, which makes this a
  **binding-storage** defect, not an assignment one.

### R-11: Every bitwise compound assignment (`&= |= ^= <<= >>= >>>=`) is a silent no-op — **CLOSED 2026-07-25**

- **Folds in**: D-B-2.
- **Verification**: `sweep-only` (both scopes, 4 target kinds).
- **Root-cause group**: G3 (guard denylist with sibling holes).
- **Repro** (`p13_bitcompound.js`, top level):
  ```js
  let a = 6; a &= 3; console.log("and=" + a);
  let b = 6; b |= 8; console.log("or=" + b);
  let c = 6; c ^= 1; console.log("xor=" + c);
  let d = 6; d <<= 2; console.log("shl=" + d);
  let e = 6; e >>= 1; console.log("shr=" + e);
  let f = 6; f >>>= 1; console.log("ushr=" + f);
  ```
  **node**: `and=2 or=14 xor=7 shl=24 shr=3 ushr=3` — **kali**: `and=6 or=6 xor=6 shl=6 shr=6
  ushr=6` (exit 0). The operand is never written back.
- **Scopes affected**: both.
- **Guard-bypass extension — the more dangerous half**:
  - `const o = {a:6}; o.a &= 3;` → kali `6`, exit **0**. But the *arithmetic* form `o.a += 3`
    on the same target fails **closed** with `E5506 "compound assignment lowering is
    unavailable unless the target is a mutable local binding"`. The bitwise path skips the
    fail-closed check entirely.
  - `const arr=[6]; arr[0] |= 8;` → kali `6`, exit 0 — the `E5506 "mutating a literal array
    is unavailable"` guard that fires for `arr[0] += 3` does **not** fire.
  - Parameter: `function u(x){ x &= 3; return x; }` `u(6)` → kali `6`, node `2`.
- **Severity**: silent-wrong-value.
- **Blast radius**: high. Hash/checksum/flag-mask code (`h ^= x`, `mask |= BIT`, `v >>= 8`) is
  the canonical use and is exactly the code that silently produces a plausible-looking wrong
  number. The non-local cases are worse because the arithmetic siblings there *are*
  fail-closed, so a reviewer would reasonably assume the whole compound-assign family is gated.
- **Mechanism hypothesis**: the compound-assign lowering handles the arithmetic operator set
  and silently falls through for the bitwise set — the write-back is skipped rather than the
  statement rejected. Project memory lists "compound bitwise-assign" as *deferred*; **the
  deferral was implemented as a silent no-op, not a diagnostic.**
- **Confidence**: high on behavior (11 transcripts); low on mechanism.
- **Not affected**: the bitwise *binary* operators (`& | ^ ~ << >> >>>`) are correct,
  including shift-count masking and 32-bit wraparound. Only the assignment forms are no-ops.
- **STATUS — CLOSED 2026-07-25** (branch `r11-bitwise-compound-assign`,
  `0104f5baf`..`9dcdcc3c1`; oracle node v26.5.0). Bitwise result semantics now live in exactly
  one place, `FunctionEmitter::emit_bitwise_i32_op_extend`
  (`crates/kali_codegen/src/emit/operators.rs`): it applies the JS op to two `i32` operands and
  extends back to `i64`, **sign**-extended for every op and **zero**-extended (uint32) only for
  `>>>`/`>>>=`. The plain binary operators (`emit_bitwise`) and all four compound-assign target
  arms route through it, so the two forms cannot desynchronize. The four lowering sites are:
  scalar local/param (`emit/literal.rs`, `emit_local_compound_assignment`), module-scope integer
  global (`emit/literal.rs`, `emit_module_global_assignment`), captured scalar env cell
  (`emit/closure_access.rs`, `try_emit_captured_assign`), and static dot-field on a fixed-shape
  object (`emit/object.rs`, `emit_object_field_bitwise_compound_assign`). Every other target —
  array element, computed/for-in-key member, `const`, non-scalar, class field, growable-array
  element, handle members, a base that is a call/nested member — and every non-integer target
  or RHS (float, string, BigInt, boolean, `null`, template, concat, call, member, index, and
  every non-literal identifier) fails closed `E5506`, never `E4201`. The
  `TypeContext::resolve_expression` gate (`crates/kali_types/src/resolve/expression.rs`) now
  admits the six ops through two narrow structural predicates
  (`bitwise_compound_target_is_admitted_local_scalar`,
  `bitwise_compound_dot_field_target_is_admitted`) and denies everything else with the operator
  text in the message; the local-scalar arm's `_ => false` fail-open — which the caller turned
  into a silent bare read of the target, i.e. *this defect* — is now a default-deny that emits
  `E5506` instead. Admission is positive-evidence only: the target must be `Repr::I64` **and**
  in `ReprTable::numeric_bindings` (`binding_is_proven_numeric`), plus per-lane BigInt and float
  taint scans (`module_global_bigint_targets`, `module_global_float_targets`,
  `captured_cell_bigint_targets`, `captured_cell_float_targets`, `shape_field_bigint_targets`);
  the RHS must be positively proven by `bitwise_compound_rhs_is_provably_i64`.
  **Headline, precisely.** Re-derived for this close on a freshly built `e416b22a1` binary
  against the final 49-target × 6-op matrix (294 cells), oracle node v26.5.0. `e416b22a1` is
  the correct stand-in for `main` here: `62d786e74..e416b22a1` touches only two `docs/` files,
  so the two are **code-identical**.

  | binary | MATCH | `E5506` | WRONG | node-throws | `E4201` | **prints the unmodified operand at exit 0** |
  |---|---|---|---|---|---|---|
  | `e416b22a1` (pre-R-11) | 2 | 42 | 232 | 12 | 6 | **209** |
  | `9dcdcc3c1` (HEAD) | 144 | 150 | 0 | 0 | 0 | **0** |

  252 cells moved, **0 of them into `WRONG` or `E4201`** (144 `WRONG→MATCH`, 88
  `WRONG→E5506`, 12 `node-throws→E5506`, 6 `E4201→E5506`, 2 `MATCH→E5506`). No R-11 signature
  failure survives in any independently-run corpus (the 1596-row laundering corpus, the
  390-program object-inflow corpus, the 85-row read-route corpus, or the Task-7 review sweeps).
  **The 2 `MATCH→E5506` cells are the total main-relative cost of this project over the
  294-cell matrix, and both are coincidences**, not working programs: they are
  `member-of-string` with `&=` and `|=`
  (`const s="abc"; let n=s.length; n&=3;` → `main` `3`, node `3`) — the R-11 silent no-op
  matched node only because `3&3 == 3` and `3|3 == 3`. The identical target with
  `^= <<= >>= >>>=` was WRONG on `main`. Those two are also the ONLY `MATCH` cells `main` scored
  in the whole 294-cell matrix, so **`main` never once computed a bitwise compound assignment
  correctly**. The scope qualifier is load-bearing, not hedging: §7.10 records two *further*
  programs outside this matrix that matched node on `main` and now return `E5506`
  (`let a=3, b=3; let n=a*b; n |= 0;` → `9` and `let o={a:3}; let n=o.a; n |= 1;` → `3`), and
  they are coincidence matches of the same kind (`9|0 == 9`, `3|1 == 3`). The claim to carry
  forward is therefore the **direction** — every main-relative move is silently-wrong or
  already-refused → fail-closed, and no measured corpus contains a program `main` genuinely got
  right that HEAD refuses — not any single total. Any later claim that this project "lost
  working behavior" should be checked against that fact first — see §7.10, where an earlier
  revision of this very entry made exactly that error.
  *Note on an earlier figure*: the Task-6 report's "143" was measured over the round-1 222-cell
  corpus under a slightly narrower signature definition; over that same 37-target subset this
  re-derivation counts 149. The corpus-bound count is not the claim — the **direction** is: no
  cell of any measured corpus prints the unmodified operand at exit 0 on HEAD, and no cell moved
  into a wrong value.
- **PLAN-DEFECT FINDING — the stated root cause was wrong, and the way it was wrong is the
  lesson.** The plan's mechanism hypothesis (recorded above: "the compound-assign lowering
  handles the arithmetic operator set and silently falls through for the bitwise set") named a
  codegen fix site. That site was **unreachable**: the six operators never tokenized at all.
  `crates/kali_lexer/src/punctuation.rs` had no rules for `&= |= ^= <<= >>= >>>=`, and
  `kali_ast::AssignmentOperator` had no bitwise variants, so `n &= 3` lexed as `&` followed by
  `=` and the operator never reached codegen in any form. An inserted prerequisite task (T1.5,
  `2f9d14dfe`) had to build the whole lexer → AST → parser → HIR → types path before the
  planned fix had any input to act on. **A root-cause trace that starts at the fix site and
  never verifies that the input arrives there is not a trace** — it is a plausible story about
  a code path, confirmed only against itself. The cheap falsifier was one token dump.
- **Deliberate scope boundaries** (fail-closed, pinned, recovery work — not defects): the
  arithmetic sibling of the object-field lane is still unclaimed
  (`o.a += 1` → `E5506`; `emit_object_field_compound_assign_dynamic` still covers only the
  computed for-in-key form); a BigInt-literal target on the **local** lane is treated as a plain
  i64 (`let n=7n; n&=3` → `3`, which is exactly what kali's own plain `n & 3` prints on every
  binary back to `e416b22a1`; node throws) — pinned by
  `bitwise_compound_tripwire_local_scalar_bigint_target_matches_the_plain_operator`.
  **The PARAMETER lane has the same divergence and is NOT covered by that pin**, nor by the
  param-inflow pin in §7.10 (`bitwise_compound_fails_closed_on_bigint_via_parameter_argument_inflow`
  covers a parameter flowing INTO a module-global/captured target, not a parameter used AS the
  target): `function f(p){ p &= 3; return p; } console.log(f(7n));` → kali `3` at exit 0
  (`main`/`e416b22a1`: `7`), node throws `TypeError: Cannot mix BigInt`. Same class and no valid
  program is miscompiled — kali's own plain `function f(p){ return p & 3; }` also returns `3`
  on every binary back to `e416b22a1` — so R-11 makes the compound form agree with the plain
  form rather than introducing new wrongness; recorded here because it is un-pinned. See §7.10
  for the measured over-denial costs and their recovery routes.
- **Pins**: `crates/kali_cli/tests/soundness_bitwise_compound.rs` — 66 tests, all green
  (`test result: ok. 66 passed; 0 failed`).

### R-12: One alias binding defeats the fail-closed array-element-store guard, in BOTH scopes

- **Folds in**: D-D-4.
- **Verification**: `CONFIRMED-BY-CONTROLLER`.
- **Root-cause group**: G3.
- **Repro** (`scratchpad/consolidate/al.js`):
  ```js
  function f(){ const a=[1,2]; const b=a; b[0]=7; console.log("b0="+b[0]); }
  f();
  ```
  **node**: `b0=7` (exit 0) — **kali**: `b0=1` (exit 0). The store vanished.
  Sweep D's fuller form also reads back through the original: `a0=7` node / `a0=1` kali.
- **The un-aliased control fails CLOSED, correctly** (`al2.js`):
  ```js
  function f(){ const a=[1,2]; a[0]=7; console.log("a0="+a[0]); }
  ```
  **kali**: `error[E5506]: mutating a literal array is unavailable in the current
  direct-runtime path unless the whole access folds statically; use new Array(n) for runtime
  mutation` (exit 1).
- **Correction 2026-07-25 — the un-aliased control fails closed only IN-FUNCTION; the
  discriminator is SCOPE, not declarator kind.** Re-measured on merged `main` (`372a3f440`)
  against node v26.5.0:
  - module scope, un-aliased: `const a=[1,2]; a[0]=7; console.log(a[0]);` → kali `1`, node `7`,
    **exit 0, no diagnostic — SILENT**, not fail-closed.
  - in-function, un-aliased: `const` **and** `let` **and** `var` all give the `E5506` above
    (exit 1) — the declarator kind makes no difference.
  - aliased (`const b=a; b[0]=7`) is silent in **both** scopes (module `1`, in-function `1`;
    node `7`).
  So the "correctly fail-closed control" above is a statement about the in-function lane only.
- So **interposing a single binding (`const b=a`) converts a correctly-refused program into a
  silently-wrong one** — in-function. At module scope there is nothing to defeat: the
  un-aliased store is already silent. Aliasing an array into a shorter local name is ubiquitous.
- **Scopes affected**: both.
- **Contrast**: the **object** equivalent is CORRECT — object aliasing propagates mutation
  properly in both scopes. The defect is array-specific.
- **Severity**: silent-wrong-value (dropped side effect).
- **Mechanism hypothesis**: the literal-array mutation guard keys on the *declaration site* of
  the identifier being indexed. `b`'s declaration is an identifier initializer, not an array
  literal, so `b` is neither recognized as a literal array (→ no guard) nor tracked as
  pointing at one (→ no real store). Classic denylist-shaped guard.
- **Confidence**: high on behavior; medium on mechanism.

### R-13: Computed member access with a variable key — reads return `0`, writes silently no-op

- **Folds in**: D-D-2 + D-D-3 (sweep D states one shared root: admittance keyed on key
  *shape* rather than key *repr*).
- **Verification**: `sweep-only` (both scopes).
- **Root-cause group**: G3.
- **Read repro** (`o06_computed.js` in-function, `o10_computed_top.js` top level):
  `const o={a:1,b:2}; const k="b"; console.log("v=" + o[k]);` → node `v=2`, kali `v=0` (exit 0).
- **Write repro** (`o12_computed_write.js`, `o15_computed_write_top.js`):
  `const o={a:1,b:2}; const k="b"; o[k]=8; console.log("dot=" + o.b);` → node `dot=8`,
  kali `dot=2` (exit 0). The store vanished; the read-back uses `.b`, a lane known good.
- **Scopes affected**: both.
- **Severity**: silent-wrong-value; the write half is a dropped side effect.
- **Blast radius**: high. The literal-key form `o["b"]` is CORRECT, and the for-in-key form is
  the shipped Spec 4a lane — so the gap is exactly "key held in an ordinary variable", the
  most common dynamic-lookup shape in real JS. The write half is worse than the read half
  because the read-back path is correct, so the program looks internally consistent while
  silently discarding writes.
- **Mechanism hypothesis**: the computed-member lane admits only a string-literal key or a
  for-in key binding; any other key expression falls through to a default-`0` read / dropped
  store instead of failing closed.
- **Confidence**: high on behavior; medium on mechanism.

### R-14: An array returned from a function reads back as all zeros

- **Folds in**: D-C-6.
- **Verification**: `sweep-only` (both scopes).
- **Root-cause group**: unclustered (arena/escape suspicion, untraced).
- **Repro**: `function f() { return [1, 2, 3]; } console.log("r=" + f()[0]);` → node `r=1`,
  kali `r=0` (exit 0).
- **Scopes affected**: both — including fully in-function
  (`function main(){ const a=f(); return a[0]+","+a[2]; }` → node `1,3`, kali `0,0`).
- **Why this is NOT the known module-scope defect**: the known register covers module-scope
  *growable* arrays built with `.push` and module-scope element *stores*. This is a plain
  array **literal** crossing a **return**, with no push and no store. Two discriminating
  controls separate them: the same literal bound directly at top level is CORRECT
  (`const a=[1,2,3]; a[0]` → 1), and an **object** literal returned from a function is CORRECT
  (`f().a` → 1).
- **Severity**: silent-wrong-value.
- **Blast radius**: high — "build an array, return it" is a basic idiom.
- **Mechanism hypothesis**: consistent with the array's backing storage living in a
  callee-local region reclaimed at return (or whose pointer is not propagated), so the caller
  reads a zeroed slot. The arena reclamation lane is the natural suspect: a returned array
  must be promoted out of the callee's scratch arena, and objects evidently are while arrays
  are not. Raising it: check whether the escape/arena analysis treats array literals as
  returned-heap.
- **Confidence**: high on behavior (3 transcripts + 2 discriminating controls); low on
  mechanism.

### R-15: `.split()` returns a length-0 array plus handle garbage

- **Folds in**: D-D-10.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G6 (unimplemented builtin folds to a default instead of failing closed).
- **Repro** (`s06_split.js`):
  `const s="a,b,c"; const p=s.split(","); console.log("len="+p.length); console.log("1="+p[1]);`
  → node `len=3` / `1=b`; kali `len=0` / `1=-9223354418898927615` (exit 0).
- **Severity**: silent-wrong-value (a wrong length *and* a leaked handle).
- **Blast radius**: high. `split` is one of the most common string operations in JS, and
  `len=0` means every downstream loop over the result silently does nothing.
- **Mechanism hypothesis**: `split` is unimplemented and falls through to a default empty
  array rather than failing closed.
- **Confidence**: high on behavior.
- **STATUS 2026-07-20 (G6 item 4, shipped)**: PARTIALLY CLOSED. The runtime `.split()`
  fallback is now in the Stream-A value-builtin deny-set (`split`) → E5506 fail-closed where
  it reaches the terminal; the static-ASCII fold lane (`console.log("abc".split("")[0])` → `a`)
  is preserved. RESIDUAL R-A4-4: the static-split element in a `+` concat position
  (`"r=" + "abc".split("")[0]`) still leaks a raw tagged string-handle i64 (`-9223354436078796799`)
  at exit 0 — a per-lane repr leak (G5-flavored), pre-existing, not closed. Note: the `split`
  deny-set entry is belt-and-suspenders; the primary fail-close for constructible member forms
  is upstream (`String.prototype.split` receiver guard).

### R-16: Per-method string-repr gap — `.slice()` / `.charAt()` / `.toUpperCase()` / `.repeat()` leak the handle in concat position

- **Folds in**: D-D-7.
- **Verification**: `sweep-only` (both scopes).
- **Root-cause group**: G5 (string handle reaches a consumer that never proved it was a string).
- **Repro** (`s02_substr.js`): `const s="hello world"; console.log("c=" + s.slice(0,3));` →
  node `c=hel`, kali `c=-9223354328704614397` (exit 0). Same for `.toUpperCase()`,
  `.repeat()`, `.charAt()`.
- **Position-dependent, which makes it especially treacherous**: `console.log(s.slice(0,3))`
  alone prints `hel` correctly, and returning the slice from a function and logging it is
  correct. **Only the concat position corrupts.** A program can print a value correctly on one
  line and print its raw handle on the next.
- **`.substring()` is CORRECT in both positions** — so this is a per-method repr-tracking gap,
  i.e. the hand-mirrored-oracle hazard already recorded in project memory
  (`kali-substring-runtime-spec2`): `substring` got its repr arm, its siblings did not.
- **Severity**: silent-wrong-value.
- **Mechanism**: the String repr axis is populated per method name. `slice`/`repeat`/`charAt`/
  case-conversion are lowered by `crates/kali_codegen/src/intrinsics/string.rs`
  (slice:247, repeat:353, charAt:582, case:812), but the corresponding predicate in
  `crates/kali_types/src/static_analysis/string.rs` (slice:365, repeat:469, charAt:661) does
  not mark the result as `Repr::String` for the concat consumer.
- **Confidence**: high on behavior; medium on mechanism (file:line found by reading, not
  proven by a fix).

### R-17: String handles escape as raw integers from the plain-array and `Object.keys` lanes

- **Folds in**: D-D-5 + D-D-6 + D-D-8 + D-D-11 (sweep D asserts one shared mechanism; the
  escaping bit patterns are all in the same `-92233543…` range).
- **Verification**: `sweep-only` (D-D-5/6/8 both scopes; D-D-11 top level only).
- **Root-cause group**: G5.
- **Repros**, all exit 0:
  - `.join()` on a plain string array (`g18`, `g21`): `const a=["p","q"]; "j=" + a.join("-")`
    → node `j=p-q`, kali `j=-9223354427488862205`. Single-element `["p"].join("-")` returns
    `0` instead — a *different* wrong value from the same lane.
  - element read of a plain string array (`g23`): `const a=["p","q"]; "0=" + a[0]` →
    node `0=p`, kali `0=-9223354444668731391`. In-function the same value is correct when it
    reaches `console.log` directly; only concat is wrong.
  - `.join()` on a never-pushed empty array (`g11`, `g19`): `const a=[]; "j=" + a.join(",")` →
    node `j=`, kali `j=-9223354436078796800`. Note a **dynamically** empty *growable* array is
    CORRECT, so this is the plain-literal `[]` lane, not the growable lane.
  - `Object.keys` elements (`m03_objkeys.js`): `const k=Object.keys(o); "0=" + k[0]` →
    node `0=a`, kali `0=-9223354444668731391`. **`k.length` is CORRECT (2)** — partial
    correctness is the dangerous pattern: an iteration over keys runs the right number of
    times with garbage in hand.
- **Severity**: silent-wrong-value; leaks an internal representation into user-visible output.
- **Blast radius**: high — string arrays are everywhere.
- **Contrast**: the numeric sibling `[1,2,3].join(",")` fails **closed**, with a *misleading*
  message (`elements of 'a' … are used as both strings and numbers`). The numeric case is safe
  but confusing; the string case is unsafe.
- **Mechanism hypothesis**: one allowlist gap at the concat/repr choke point — a string handle
  reaching a consumer that never proved it was a string, rendered as an i64.
- **Confidence**: high on behavior; medium-high on the shared-root merge.

### R-18: String **literal** operands of `&&`/`||` leak a raw handle as a number

- **Folds in**: D-B-5.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G5 + G3 (it is a hole in an existing guard).
- **Repro** (`p20_lit_or.js`):
  ```js
  console.log("1=" + ("" || 7));
  console.log("2=" + ("hi" || 7));
  console.log("3=" + ("" && 7));
  console.log("4=" + ("hi" && 7));
  ```
  **node**: `1=7  2=hi  3=(empty)  4=7` — **kali**: `1=-9223354436078796800
  2=-9223354427488862206  3=7  4=7` (exit 0). Cases 1 and 2 leak a tagged string handle into
  numeric position; case 3 additionally has the truthiness backwards.
- **This is precisely a hole in an existing guard.** The equivalent through a *variable* fails
  **closed**: `let s = ""; s || 7` → `E5506 "a runtime string value is unavailable as an
  operand of '&&'/'||' … truthiness of a runtime string is not evaluated correctly"`. The
  guard keys on the operand being a runtime string *value*; a string *literal* operand slips
  past it into the very miscompile the guard's own message describes.
- **Severity**: silent-wrong-value.
- **Blast radius**: medium — `"" || x` / `"lit" && x` appear in defaulting code, though the
  variable form is more common. The guard-hole pattern is the interesting part.
- **Mechanism hypothesis**: the `&&`/`||` deny check inspects the operand's inferred `Repr`
  for a runtime-string axis; a `Literal` string node is not routed through that inference, so
  it reaches the scalar lowering and its interned handle is used as the i64 result.
- **Confidence**: high on behavior; medium on mechanism.
- **Important non-finding**: the short-circuit fix `b5bae4e10` **HOLDS**. 30 shapes probed by
  sweep B and re-affirmed by the controller — value position, assigned, nested, chained,
  mixed, as `if`/`while`/`for` conditions, in ternaries, as return values, call arguments,
  array elements, object-literal values, under `!`, feeding `+` and `===`. **No surviving
  short-circuit hole; no regression.**

### R-19: `String(x)` and `x.toString()` silently return `0` for every input, in every scope

- **Folds in**: D-A-1 + D-D-1 (the same defect found independently by two sweeps).
- **Verification**: `sweep-only` (both scopes, both sweeps).
- **Root-cause group**: G6.
- **Repro**: `console.log(String(42));` → node `42`, kali `0` (exit 0).
- **Total, not partial**: `String(42)`→0, `String(-7)`→0, `String(1.5)`→0, `String("hi")`→0,
  `String(true)`→0, `String(null)`→0, `String(undefined)`→0, `String(0/0)`→0, `String(1/0)`→0,
  `String(-1/0)`→0, `String(1e-7)`→0. Same for the method form: `(42).toString()`→0,
  `(1.5).toString()`→0, `var n=42; n.toString()`→0, `var s="hi"; s.toString()`→0. It poisons
  downstream concat: `"x" + String(42)` → `x0`.
- **A near-miss trap worth recording**: `console.log(String(42).length)` prints `2`, which
  *matches* node and looks like evidence the call works. It does not —
  `String("hello").length` also prints `2` and `String(12345).length` also prints `2`
  (node: `5` for both). The `.length` of a `String(...)` result is a constant `2` regardless
  of input; the agreement at `String(42)` is coincidence. Meanwhile
  `var s=String("hello"); s.length` prints `0` — the direct-member and via-binding paths
  disagree with each other as well. **Any future "String() works" claim resting on `.length`
  is invalid.**
- **Severity**: silent-wrong-value.
- **Blast radius**: very high. `String(x)` and `.toString()` are the two most common explicit
  conversions in JS, and it is the natural thing a user reaches for when `+` concat is
  rejected by `E3200`. Anything that formats a value, builds a key, or normalizes input is
  affected, and it fails silently at exit 0 with a plausible-looking `0`.
- **Mechanism hypothesis**: a uniform `0` independent of argument type reads like the call
  resolving to an absent builtin whose result slot is never written. **Contrast `Number(...)`,
  which fails honestly** with `E3100: undefined identifier 'Number'`. Whatever makes `Number`
  fail closed is the behavior `String` should have.
- **Confidence**: high on behavior (20+ transcripts, two sweeps, both scopes); low on mechanism.
- **STATUS 2026-07-20 (G6 item 4, shipped `acfc9c87b`..`20790621c`)**: CLOSED for the canonical
  spellings via the Stream-A value-builtin deny-set. `String(x)`, `x.toString()`, computed
  `n["toString"]()`, and the concat/template/array/push/arg positions of `String(x)` now
  fail closed E5506 (several of these were silent-0 before and were CLOSED by this work).
  Program-defined same-name functions are unaffected (gate-1 pre-empts the deny-set).
  RESIDUALS (pre-existing NAME-deny-set leaks, NOT closed — closable only by an allowlist at
  the resolve choke point, Group 3): R-A4-1 `globalThis.String(x)` → silent `0`; R-A4-2
  `globalThis["String"](x)` → silent `0`. New pin file: `crates/kali_cli/tests/soundness_unimplemented_builtins.rs`.

### R-20: `JSON.stringify(x)` silently returns `0` for every input

- **Folds in**: D-A-5.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G6.
- **Repro**: `const o={f:1}; console.log(JSON.stringify(o));` → node `{"f":1}`, kali `0` (exit 0).
- **Total, like R-19**: `JSON.stringify(42)`→`0`, `("hi")`→`0`, `([1,2])`→`0`, `({f:1})`→`0`.
  It does **not** fail closed with an E-code, which is what makes it a defect rather than a
  missing-feature note.
- **Severity**: silent-wrong-value.
- **Blast radius**: moderate-to-high; universal in real JS, though arguably "unimplemented"
  territory — which is precisely the point: unimplemented must mean *refuse*, not *return 0*.
- **Mechanism hypothesis**: likely the same root as R-19. Sweep A flags this as its
  **highest-value structural suspicion**: one choke-point fix (make unknown builtin calls fail
  closed) would convert R-19, R-20 and R-15 from silent-wrong into honest errors at once.
- **Confidence**: high on behavior; low on mechanism.
- **STATUS 2026-07-20 (G6 item 4, shipped)**: CLOSED for the canonical spellings via the
  Stream-A deny-set. `JSON.stringify(o)` and computed `JSON["stringify"](o)` fail closed
  E5506 (JSON-receiver-gated). RESIDUAL R-A4-3 (pre-existing): an ALIASED receiver
  `const j = JSON; j.stringify(o)` escapes the receiver gate → silent `0` at exit 0
  (Group-3 allowlist-at-resolve). NOTE: the E5506 message names the callee `stringify`
  (not `JSON.stringify`) — cosmetic.

### R-21: There is no `undefined` value — absent, void and `undefined` reads render as `0` or `false`

- **Folds in**: D-A-6 + D-A-11 + D-C-7 + D-D-12 (four sweeps' views of one missing repr axis).
- **Verification**: `sweep-only` (both scopes for D-A-6 and D-C-7; top level only for D-D-12's
  out-of-bounds cases and D-A-11).
- **Root-cause group**: G4.
- **Repros**, all exit 0:
  - binding: `var x=null; console.log(x)` → `0` (node `null`); `var x=undefined;
    console.log(x)` → `0` (node `undefined`). Direct literal position is CORRECT
    (`console.log(null)` → `null`).
  - concat: `console.log("v=" + null)` → `v=0` (node `v=null`). And **inconsistently**,
    `console.log("v=" + undefined)` → **`v=false`** — `undefined` renders as the string
    `false` in concat but as `0` through a binding. Two different wrong answers for one value.
  - void return: `function f(){} console.log("r=" + f())` → `r=0` (node `r=undefined`); bare
    `console.log(f())` → `0`. A function falling off the end of a non-taken `if` behaves the
    same.
  - arithmetic: `undefined + 1` → `1` (node `NaN`). Note `null + 1` → `1` is **correct** per
    JS, so it is specifically the `undefined`→number rung that is wrong.
  - absent reads, three paths and three *different* wrong renderings: missing object field
    `const o={a:1}; "z="+o.z` → `z=0` (node `undefined`); out-of-bounds literal-array read
    `const a=[1,2]; "oob="+a[5]` → `oob=false` (node `undefined`); out-of-bounds growable read
    → `0` (node `undefined`).
- **Important nuance — comparison is CORRECT while rendering is wrong**: `f() === undefined`
  takes the true branch, and `if (f())` correctly takes the falsy branch. So an `undefined`
  sentinel genuinely exists and compares correctly against `undefined`; only its *rendering*
  collapses. (This does **not** rescue R-08: the sentinel is indistinguishable from `0`.)
- **Severity**: silent-wrong-value.
- **Blast radius**: high. "Function returned nothing prints `0`" is a particularly nasty shape
  because `0` is a legitimate value a reader accepts without suspicion, and a missing property
  silently contributes `0` to a sum instead of poisoning it to `NaN`, so the error never
  surfaces. The three-different-wrong-renderings inconsistency suggests each absent path is an
  independent uninitialized default rather than one modelled `undefined`.
- **Confidence**: high on behavior; medium on the single-root merge.

### R-22: Loose equality `==` does not coerce across types

- **Folds in**: D-A-7.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: unclustered (missing coercion-table rung; *not* G4 — the special-case
  table is present, one rung is absent).
- **Repro**: `console.log("v=" + (1=="1"));` → node `v=true`, kali `v=false` (exit 0). Concat
  position used deliberately, since direct-log boolean rendering is separately broken (R-30).
- **Detail**: `"1"==1` → `false` in both operand orders. Same-type comparisons are correct
  (`1==1.0` → `true`), and `null==undefined` → `true` is correct — so the table is not simply
  absent; it is the number/string coercion rung that is missing.
- **Severity**: silent-wrong-value, escalating to silently wrong control flow wherever such a
  comparison guards a branch.
- **Blast radius**: moderate. `==` across number/string is common in loosely-typed input
  handling; a wrong `false` in a guard takes the wrong branch at exit 0.
- **Confidence**: high on behavior.

### R-23: `typeof` returns `0` for anything but a bare literal

- **Folds in**: D-A-8, plus sweep B's `p38_misc.js` and sweep C's b6/u4 sightings.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G8 (per-sink rendering) / G4.
- **Repro**: `var b=true; console.log(typeof b);` → node `boolean`, kali `0` (exit 0).
- **Detail**: correct for direct literals — `typeof true`→`boolean`, `typeof 1`→`number`,
  `typeof "a"`→`string`, `typeof undefined`→`undefined`. Wrong for everything else:
  `typeof b` (binding)→`0`, `typeof o` (object)→`0`, `typeof f` (function)→`0`,
  `typeof (1<2)`→`0`, and in concat `"t=" + typeof (1<2)` → `t=0`. For a void-call result
  `typeof x` yields the *number* rendering, not even the string `"undefined"`.
- **Severity**: silent-wrong-value.
- **Blast radius**: moderate. `typeof x === "string"` style dispatch is a common guard and it
  will now silently never match.
- **Mechanism note**: project memory records that a `typeof` codegen flip was **REVERTED** in
  throw-fallout Stage 5 per the decision rule. It is worth checking whether that revert is
  what leaves this open, i.e. whether the revert traded a test regression for a live silent
  miscompile.
- **Confidence**: high on behavior.

### R-24: `Object.freeze()` is silently ignored — writes to a frozen object go through

- **Folds in**: D-D-9.
- **Verification**: `CONFIRMED-BY-CONTROLLER`, **with an important probe caveat**.
- **Root-cause group**: G6.
- **Repro** (`scratchpad/consolidate/fz1.js`):
  ```js
  const o={x:1}; Object.freeze(o); o.x=99;
  console.log("x="+o.x); console.log("isFrozen="+Object.isFrozen(o));
  ```
  **node**: `x=1` / `isFrozen=true` (exit 0) — **kali**: `x=99` / `isFrozen=0` (exit 0).
- **PROBE CAVEAT — the weaker probe HIDES the defect** (`fz2.js`):
  `const o=Object.freeze({x:1}); o.x=99; console.log("x="+o.x);` → node `x=1`, kali `x=1`.
  **They agree.** Written that way, the object literal folds and the write is dropped for
  unrelated reasons, so the probe reports a match while the defect is live. Any future
  `Object.freeze` verification must bind first and freeze second.
- **Severity**: silent-wrong-value.
- **Blast radius**: medium. `Object.freeze` is common in config/constant modules, and it is
  the standard *hardening* idiom — a program that freezes to protect an invariant gets no
  protection and no diagnostic. `Object.isFrozen` additionally reports `0`.
- **Mechanism hypothesis**: `Object.freeze(x)` is modelled purely as an identity wrapper for
  intrinsic-hardening recognition and never given write-barrier semantics.
- **Confidence**: high on behavior; medium on mechanism.
- **STATUS 2026-07-20 (G6 item 4)**: DEFERRED — NOT closed. Attempted under Stream C; the
  plan's escape hatch fired. A receiver-SHAPE-only classifier cannot distinguish the unsound
  `Object.freeze(o); o.x=99` (write leaks) from the SOUND `Object.freeze(o); …read-only /
  Object.is / Reflect.ownKeys` — both are a bare program-bound object identifier at the freeze
  site. Failing closed on the shape regressed `object_is_freeze.rs` (8→0) and 7 lib passthrough
  tests (Object.is alias-chain / Reflect.ownKeys const-bound-iterable). Cleanly separating them
  needs the write-barrier/dataflow analysis the fail-closed direction forbids. Becomes its own
  follow-up plan (ledger item 8). R-24 STAYS OPEN.

### R-25: Array spread `[...a]` yields `len=1` and element `0`

- **Folds in**: D-D-13 (an EXTENSION of the registered `[...Object.values(o)] → 0` defect).
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G6.
- **Repro** (`m06_spread_arr.js`):
  `const a=[1,2]; const b=[...a]; console.log("len="+b.length); console.log("0="+b[0]);` →
  node `len=2` / `0=1`; kali `len=1` / `0=0` (exit 0).
- **Why an extension and not a duplicate**: materially different shape (spread of a plain
  array-literal binding, not of an intrinsic call result) and a materially different wrong
  answer (`len=1` — the spread element counted as one slot and left zero — rather than `0`).
  **The blast radius of the registered bug is therefore wider than "spread of
  `Object.values`": it is spread of *anything*.** Object spread `{...o}` by contrast fails
  CLOSED (`E5506`).
- **Severity**: silent-wrong-value.
- **Confidence**: high on behavior.
- **STATUS 2026-07-20 (G6 item 4, shipped `acfc9c87b`)**: PARTIALLY CLOSED. `[...a]` now fails
  closed E5506 at the guarded fold sites: `.length` fold + numeric-index fold
  (`emit/operators.rs`), the static-slice resolver (`emit/call.rs`), and the console static
  length-render (`intrinsics/host.rs`); object spread `{...o}` already failed closed.
  RESIDUAL (pre-existing, NOT closed — `array_literal_contains_spread` is consulted at only ~4
  of ~30 `is_array_literal` consumers, so `is_array_literal` still returns true for a spread
  literal at the unguarded sites): `console.log([...a])` → `0` at exit 0 (node `[ 1, 2 ]`);
  `new Map([...a])` / `new Set([...a])` → `size=0` at exit 0. A fuller close is the
  choke-point form (a single shared spread guard across the ~30 consumers, or make
  `is_array_literal`'s consumers spread-aware) — deferred as a Group-3-style follow-up; the
  per-site guarding is itself the "denylist of shapes leaks" pattern. New pin file:
  `crates/kali_cli/tests/soundness_array_spread.rs`.

### R-26: Unary `+` on a non-numeric string yields garbage integers instead of `NaN`

- **Folds in**: D-A-2.
- **Verification**: `sweep-only` (both scopes, plus via bindings and parameters).
- **Root-cause group**: unclustered (missing range guard in one lowering).
- **Repro**: `console.log(+"abc");` → node `NaN`, kali `5451` (exit 0).
- **The rule is a naive unvalidated digit accumulator**: outputs are exactly
  `acc = acc*10 + (byte - 0x30)` over every byte, with no digit check and no `NaN` exit:
  - `+"a"` → `49` (`'a'`=97, 97−48=49)
  - `+"abc"` → `5451` (49·100 + 50·10 + 51)
  - `+"12x"` → `192` (1, 2, then `'x'`−48=72 → 12·10+72)
  - `+" "` → `-16` (`' '`=32, 32−48=−16) — it goes **negative**
  - `+"  7  "` → `-175476` (node `7`; JS trims whitespace)
  - `+"0x10"` → `7210` (node `16`)
  Correct cases: `+"42"`→42, `+"-5"`→−5, `+"1.5"`→1.5, `+""`→0, `+true`→1.
- **Severity**: silent-wrong-value.
- **Blast radius**: high, **and it lands on a lane this project already depends on**.
  `+process.argv[2]` is the documented argv→number primitive (Spec 5). Today a malformed
  argument does not produce `NaN` and does not fail closed — it produces a large, sometimes
  negative, plausible-looking integer that flows straight into loop bounds and allocation
  sizes. Leading/trailing whitespace alone is enough, and that is an entirely ordinary thing
  for an argv- or file-derived string to contain.
- **Mechanism hypothesis**: the string→i64 lowering for unary `+` accumulates digits without a
  `0..=9` range guard and without a non-digit/whitespace path.
- **Confidence**: high on behavior; **high on mechanism** — the arithmetic model predicts all
  six divergent outputs exactly, which is evidence rather than a guess.

### R-27: The comma operator evaluates to `0`

- **Folds in**: D-B-7.
- **Verification**: `sweep-only` (top level + one in-function sighting).
- **Root-cause group**: unclustered.
- **Repro** (`p39_comma.js`):
  ```js
  let n = 0;
  function bump() { n = n + 1; return 5; }
  let a = (1, 2);   console.log("a=" + a);
  let b = (bump(), 7); console.log("b=" + b);
  console.log("n=" + n);
  ```
  **node**: `a=2  b=7  n=1` — **kali**: `a=0  b=0  n=1` (exit 0). The side effect fires exactly
  once; only the *value* of the sequence expression is lost.
- **Severity**: silent-wrong-value.
- **Blast radius**: low-to-medium — uncommon in hand-written modern JS, but pervasive in
  minified/transpiled output and in `for (i = 0, j = n; …)` headers.
- **Mechanism hypothesis**: the sequence expression is emitted as a statement sequence with
  `want_value=false`, dropping every operand and pushing the `I64Const(0)` placeholder.
- **Confidence**: high on behavior; medium on mechanism.

### R-28: `-0` is not represented — `1 / -0` yields `+Infinity`

- **Folds in**: D-B-8 + D-A-12 (the value half and the rendering half of one representational
  gap).
- **Verification**: `sweep-only` (top level; the mechanism is representational so both scopes
  are expected).
- **Root-cause group**: unclustered.
- **Repro** (`p15_negzero.js`): `let mz = -0; 1/mz` → node `-Infinity`, kali `Infinity`.
  Same for `let z=0; let mz2=-z; 1/mz2` and for the literal `1/-0`.
- **Rendering half**: `console.log(-0)` → kali `0`, node `-0`. Note `String(-0)` is `"0"` in
  JS, so `console.log("v=" + (-0))` → `v=0` is **correct** in both; only the direct-log
  inspect path differs.
- `Object.is(-0, 0)` is correctly `false` in both, and `0 * -1` is `0` in both.
- **Severity**: silent-wrong-value (value half); rendering-only (log half).
- **Blast radius**: low — matters for numeric/geometry code using the sign of a reciprocal.
  Recorded for completeness of the arithmetic map; would not prioritize.
- **Mechanism hypothesis**: `-0` is folded to the integer `0` (kali's default numeric repr is
  i64), so the sign bit never reaches the f64 division.
- **Confidence**: high on behavior; medium on mechanism.

### R-47: `for..of` over a `let`-declared array binding iterates the characters of the binding's own NAME

- **Folds in**: nothing. Found 2026-07-25 while closing R-11 (§7.10 sightings); promoted to an
  owning ID because nothing in §0.2, §0.3 (R-35..R-46) or §7.9 covers iteration at all.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — all three declarator lanes re-measured on a
  freshly-built binary at merged `main` (`372a3f440`) against node v26.5.0, 2026-07-25.
- **Root-cause group**: unclustered. It has **G3**'s shape — the `for..of` iterable admittance
  test is keyed on binding *form*, so the `let` spelling slips past into the string-iterable lane
  instead of failing closed — with a **G7** flavour, since the discriminator is the declarator's
  binding storage. It is deliberately *not* added to G3's member list, whose "six" and "four of
  the six" counts are stated for the original R-01..R-34 set; R-35..R-46 sit in no §3 cluster
  either.
- **Repro** (module scope): `let a=[1,2,3]; for (const x of a) console.log(x);` → **kali** prints
  one line, the letter `a`; **node** prints `1` `2` `3`. Exit 0, no diagnostic.
- **The name really is the iterand**: `let zz=[1,2,3]; for (const x of zz) …` prints `z` then
  `z` — two iterations for a two-character name, over a three-element array.
  `let a=[10,20]; for (const q of a) …` prints `a` — one iteration, and the *element* values
  never appear. So both the iteration count and every value come from the identifier's text.
- **The three declarator lanes differ, and only `let` is silent**:
  - `let a=[1,2,3]` → **SILENT**, prints `a` (above).
  - `var a=[1,2,3]` → **FAIL-CLOSED**, `error[E5506]: for-of array iteration lowering is
    unavailable unless the iterable is a literal array or supported string iterable with
    literal elements and the loop target is a variable declaration or simple identifier
    binding; use a supported loop form or the later compatibility path` (exit 1).
  - `const a=[1,2,3]` → **CORRECT**, `1` `2` `3`.
- **Scopes affected**: both — `function f(){ let a=[1,2,3]; for (const x of a) console.log(x); }
  f();` prints `a`, identically to the module-scope form.
- **Severity**: **Tier 2** (silently produces a wrong value) **and** **Tier 3** (silently wrong
  control flow) — every value is wrong *and* the loop trip count is wrong, so it straddles
  Tier 2/3 rather than sitting cleanly in either. Filed in Tier 2.
- **Blast radius**: high, and this is among the most *deceptive* shapes in the register. The
  loop body genuinely runs, exit is 0, and the output is plausible-looking data rather than
  `0` or an empty result — the failure mode every other entry's `0` at least makes visible.
  `let xs = [...]; for (const x of xs)` is everyday JS.
- **Mechanism hypothesis**: the iterable operand resolves to the identifier's own text, which
  is then routed to the *string* for-of lane and iterated per character. Consistent with the
  `var` diagnostic, whose text ("literal array or supported string iterable") names exactly
  the two lanes and shows the admittance test is shape-keyed. Not located in source.
- **Confidence**: high on behavior (6 transcripts, 3 declarator lanes, both scopes, and the
  name-length/trip-count correspondence is exact); low on mechanism.
- **Cross-references**: §7.10 sightings (where it was first measured); **R-06-R3** (`let` arrays
  read back as zero/empty through the *indexing* lane — the same `let`-array storage gap seen
  through a different consumer); cluster **G3** for the shape of the mistake (see Root-cause
  group above for why it is not listed as a G3 member).

### R-48: An array stored into an object field typed `I64` reads back `0`

- **Folds in**: nothing. Found 2026-07-25 while closing R-11 (§7.10 sightings); given an owning
  ID because it had none anywhere in this document.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — re-measured on a freshly-built binary at merged
  `main` (`372a3f440`) against node v26.5.0, 2026-07-25.
- **Root-cause group**: unclustered (escape/provenance-loss family, with R-14).
- **Repro**: `let o={a:6}; o.a=[1,2]; console.log(o.a);` → **kali** `0`, **node** `[ 1, 2 ]`
  (exit 0, no diagnostic).
- **Identical through an alias**: `let o={a:6}; let b=[1,2]; o.a=b; console.log(o.a);` → `0`.
- **Not a `let`/`var` lane**: the `const` receiver behaves identically —
  `const o={a:6}; o.a=[1,2]; console.log(o.a);` → `0` (node `[ 1, 2 ]`). The declarator kind is
  not the discriminator here; the field's already-inferred scalar repr is.
- **Scopes affected**: both, measured —
  `function f(){ let o={a:6}; o.a=[1,2]; return o.a; } f()` → `0` (node `[ 1, 2 ]`), identical
  to the module-scope form.
- **Severity**: silent-wrong-value (a dropped store observed as a wrong read).
- **Blast radius**: medium-high. "Initialize a field to a scalar placeholder, then fill it with
  a list" is a common shape, and the `0` read-back is indistinguishable from a legitimately
  empty result.
- **Mechanism hypothesis**: the field's repr is fixed at `Repr::I64` by its numeric-literal
  initializer, and the later array store neither widens the field nor fails closed, so the
  array handle is truncated/lost and the slot reads its zero.
- **Distinct from §7.9's `P5-R-aggregate-array-provenance`**, which is the same *family* but a
  different observable: that one reads a plausible **wrong length** (child-count / holder-length)
  where this one reads **`0`**. Also **≈ R-14** (an array losing provenance across a boundary).
- **Confidence**: high on behavior (3 transcripts incl. the alias and `const` controls); low on
  mechanism.

### R-53: `for (var v of […])` binds every element to `0`; `const` is correct

- **Added**: 2026-07-29, by the **R-35 close-out** (Task 11, branch `r35-switch-lowering`).
  Found while validating `for…of` as a *faithful-loop control* for the R-09 re-derivation —
  i.e. while building the instrument, not while testing the feature.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — measured on a freshly built binary at
  `58234e87c7` against `node v26.5.0`, with the `const` form as a paired control.
- **Root-cause group**: G4 (there is no value distinct from the scalar `0`) by symptom;
  plausibly G7 (binding storage) by mechanism, which is not traced. Recorded as G4.
- **Repro** (per-iteration `console.log`, one argument, literal-rooted):
  ```js
  var t = 0;
  for (var v of [1, 2, 3]) {
    console.log("iter=" + v);
    t = t + v;
  }
  console.log("t=" + t);
  ```
  **node**: `iter=1` / `iter=2` / `iter=3` / `t=6` (exit 0).
  **kali**: `iter=0` / `iter=0` / `iter=0` / `t=0` (exit 0, no diagnostic).
- **The trip count is CORRECT — three iterations — and only the bound value is lost.** That
  is what makes it Tier 2 rather than Tier 1, and it is also what makes it dangerous as an
  instrument: a fixture that asserts *"the loop ran three times"* passes, and a fixture that
  asserts a sum gets a plausible `0`. No `break`, `continue` or `switch` appears anywhere in
  the repro.
- **Control — `const` is correct.** The byte-identical fixture with `for (const v of [1,2,3])`
  gives `iter=1` / `iter=2` / `iter=3` / `t=6` on **both** engines. So this is the loop
  variable's **declarator kind**, not `for…of` and not array literals.
- **WIDENED 2026-07-29 (`64438bf0ef`, fix wave item 1): `let` is affected too.** The original
  entry probed `var` and `const` only and left `let` untested, which made the headline read
  as if `let` were on the correct side. It is not. Measured switch-free at `64438bf0ef`
  against `node v26.5.0`:
  ```js
  var s = 0;
  for (let v of [1, 2, 3, 4]) {
    console.log("iter=" + v);
    s = s + v;
  }
  console.log("s=" + s);
  ```
  **node**: `iter=1` / `iter=2` / `iter=3` / `iter=4` / `s=10` (exit 0).
  **kali**: `iter=0` ×4 / `s=0` (exit 0, no diagnostic).
  The `const` twin of this exact fixture gives `s=10` on both engines. So the correct side is
  **`const` alone**, and the silent side is **`var` *and* `let`**.
- **The silent lane is bounded on the ITERABLE axis.** Over a *binding* rather than an array
  literal — `var a = [1,2,3]; for (const v of a)` — kali fails closed with an honest `E5506`
  ("for-of array iteration lowering is unavailable unless the iterable is a literal array…").
  So R-53's silent surface is precisely **for-of over an array LITERAL with a `var` or `let`
  loop variable**. Everything outside that is either correct (`const` over a literal) or
  fail-closed (any declarator over a binding).
- **Distinct from R-47**, with which it will otherwise be confused — they are near-mirror
  images and both involve `for…of` and `let`/`var`/`const`:
  | | R-47 | R-53 |
  |---|---|---|
  | what is mis-declared | the **array**, `let a = […]` | the **loop variable**, `for (var v of …)` |
  | what is iterated | the characters of the binding's own **name** | the right elements, three times |
  | observed | `a` printed | `0` printed, per element |
  | `var` form | fails closed `E5506` | **SILENT (this entry)** |
  | `const` form | correct | correct |
  Note especially that `var` is the **fail-closed** case in R-47 and the **silent** case
  here, so "R-47 covers the `var` story for `for…of`" is false in both directions.
- **Severity**: Tier 2 — silently produces a wrong value, with a correct trip count.
- **Blast radius**: moderate, and **revised upward 2026-07-29** by the `let` widening above.
  The original reasoning — *"`for (const x of …)` is the idiomatic modern spelling and is
  correct; `for (var x of …)` is the transpiled-output / older-style spelling and is not"* —
  understated it, because `for (let x of …)` is **also** idiomatic modern JS (it is the
  spelling anyone reaches for when the loop body reassigns, and many codebases prefer it
  uniformly) and it is **also** silent. Still short of frontier rank — see §0.1's 2026-07-29
  amendment, point 2, which declines to promote R-53 (or R-51/R-52) into R-35's vacancy and
  explains why the frontier is unranked.
- **PROBE-DESIGN CONSEQUENCE, and this is why the entry is worth its length: `for (var v of
  […])` must NOT be used as a faithful-loop control.** It was under consideration as one
  during this project precisely because `for…of`'s `continue` **is** faithful (see R-09) —
  but the elements it binds are all `0`, so any fixture keyed on the element value measures
  nothing while appearing to run. Use `while`, or a C-style `for` with no update, or
  `for (const v of …)`. Recorded alongside R-52's `for (init; ;)` consequence: this project
  produced **two** independent instrument invalidations, both in loop forms, both silent.
- **Confidence**: high on behavior (paired `const` control, per-iteration logging, both
  engines); **no mechanism traced.**

---

## Tier 3 — silently wrong control flow (value otherwise intact)

### R-29: Assignment to a `const` is silently ignored (node throws)

- **Folds in**: D-C-8.
- **Verification**: `sweep-only` (both scopes).
- **Root-cause group**: G7.
- **Repro**: `const x = 1; x = 2; console.log("r=" + x);` → node
  `TypeError: Assignment to constant variable.` (exit 1); kali `r=1` (exit 0).
- **Severity**: silent-wrong-control-flow, low priority — node exits non-zero, so this is not
  the exit-0-vs-exit-0 class the sweeps primarily targeted. The write is *discarded* rather
  than misapplied, which is the safer of the two failure directions.
- **Blast radius**: low for correct programs; matters only for buggy input, where kali hides a
  bug node would surface.
- **Mechanism hypothesis**: no const-assignment check in the resolver. Note that under R-07
  `const` has no storage at all, so "the write is discarded" is the expected consequence
  rather than an independent decision.
- **Confidence**: high on behavior.

### R-54: A second `default` clause is absorbed into the first — kali accepts a file node rejects

- **Added**: 2026-07-29, by the **R-35 close-out** (Task 11, branch `r35-switch-lowering`).
  Found while completing the acceptance matrix's `default` axis: the matrix requires a
  **denied** cell for Rule 3 ("two or more `default` clauses"), and the denial would not fire.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — traced in source, reproduced on a freshly
  built binary at `58234e87c7`, differentially compared against `node v26.5.0`.
- **Root-cause group**: **G1** (parser fail-open recovery). In the **same function as R-49**
  (`parse_switch_statement`) and **independent of it** — R-49 was a closer that was inspected
  and never consumed; this is an **incomplete stop set**. R-49's fix did not touch it, and
  R-49's "unique non-consuming block-closer" claim is not falsified: that claim was about
  closers, and this is a different shape.
- **Repro**:
  ```js
  var g = 0;
  function s(x) {
    switch (x) {
      case 1: return "one";
      default: g = 5;
      default: return "d2";
    }
  }
  console.log("v=" + s(9));
  console.log("g=" + g);
  ```
  **node**: refuses the whole file — `SyntaxError: More than one default clause in switch
  statement` (exit 1), nothing executes. **kali**: `v=d2` / `g=5` (exit 0).
- **`g=5` is the load-bearing half of the repro.** `v=d2` alone would be consistent with
  "the second `default` replaced the first". `g=5` proves **both bodies ran, merged into a
  single clause** — so this is not a selection difference, it is two clauses becoming one.
- **Mechanism (traced, and it is a two-line asymmetry)**: `parse_switch_statement` has two
  sibling clause arms with two different stop sets for their statement loops.
  | arm | stop set | correct? |
  |---|---|---|
  | `case` (`crates/kali_parser/src/statement.rs:536-541`) | `Case \| Default \| RightBrace` | yes |
  | `default` (`crates/kali_parser/src/statement.rs:561-564`) | `Case \| RightBrace` | **no — `Default` is missing** |
  So a `case` clause correctly stops when it sees `default`, but a `default` clause does not.
  The second `default` token and everything after it is consumed as *statements of the first
  `default`'s consequent* (the bare `default` token itself falls to the loop's
  `else { self.stream.advance(); }` recovery and decays to nothing).
- **Consequence for R-35's allowlist: `switch_plan`'s `"more than one `default` clause"`
  denial (`crates/kali_codegen/src/emit/switch.rs:105`) is UNREACHABLE DEAD CODE.** The AST
  can never carry two `default`s, so the check can never fire. The rule is correctly *stated*
  and is simply enforced at the wrong layer — or rather, not enforced at all. This is worth
  keeping visible because it is a **denial that a reader would reasonably believe is
  tested**, and it is the one Rule-3 cell of the acceptance matrix that has no pin.
- **Severity**: Tier 3. **Only invalid JS is affected** — no valid program can contain two
  `default` clauses, so no correct program is miscompiled. Filed alongside R-29 (assignment
  to a `const` is silently ignored where node throws), which is the same class: kali is
  permissive where node refuses.
- **Blast radius**: low, and it is a *diagnostic* failure rather than a correctness one — but
  it is a real fail-open in a parser this project has already had to fix once, and the
  asymmetry that causes it is the kind that a stop-set audit would catch in bulk. **G1's
  standing recommendation applies**: sweep the sibling stop sets, do not patch this one site.
- **Not fixed in this stage, deliberately.** The R-35 close-out's scope is the probe, the
  matrix, the register and the gate; a parser behaviour change would land after the
  whole-stage adversarial review that has already run, and this defect affects no valid
  program. Recorded rather than patched.
- **Confidence**: high on behavior (both engines, side-effect discriminator, exit statuses
  captured unpiped); **high on mechanism** (the asymmetry is two adjacent literal stop sets
  and it predicts the observed merge exactly).

---

## Tier 4 — rendering-only (the in-memory value is correct)

### R-30: Computed booleans render `1`/`0` in direct `console.log` argument position

- **Folds in**: D-A-9 (boundary map of a known defect).
- **Verification**: `sweep-only` (both scopes).
- **Root-cause group**: G8.
- **Wider than "computed"**: a plain binding to a literal is already affected —
  `var b=true; console.log(b)` prints `1`. The producer set is *every* boolean that is not a
  syntactically inline literal at the log site: comparisons, `!`/`!!`, `&&`/`||` results,
  function returns, **parameters**, ternary results, `const` object fields, plain `var`
  bindings, and `??` **in single-argument `console.log` position only** (added 2026-07-19, third
  addendum round — see R-08 residual 5: a `??` whose statically-selected result is a proven
  boolean hits this exact sink and mechanism, and closes when this entry closes, not when R-08's
  own `Repr::Boolean`/null-axis work lands). **Scope correction, round 4 (2026-07-19): `??` is
  a producer of THIS entry ONLY through the single-argument sink.** The string-concat and
  multi-argument console lanes have their OWN, independent `??`-specific loss of boolean shape
  — see R-08 residual 6 — which does **not** close when this entry (R-30) closes, because the
  value in question (e.g. `"w:" + (Number.isInteger(5) ?? 9)`) never reaches a `console.log`
  argument position, single or multi, at all; unifying the two console formatters (this entry's
  fix) cannot repair a defect in `+`. Only `console.log(true)` with an inline literal is
  correct.
  - **Corrected 2026-07-19 (stale in the over-claim direction)**: the producer list above used
    to also name plain `const` bindings, but `const b = true; console.log(b)` now prints `true`
    correctly (re-verified on a freshly built binary) — the `e4b5f7138` fix's binding-chain
    resolution reaches a plain `const` scalar. Only `var` is still wrong among plain bindings;
    `const` **object fields** (`const o = {f: true}; console.log(o.f)`) are a separate, still-
    broken shape (re-verified: kali `1`, node `true`) and remain correctly listed above.
- **Narrower than "everywhere"**: the concat and template paths are already **FIXED for
  operands `static_equality_class`/`is_string_valued`/`is_float_valued` can prove** —
  `"v=" + (1<2)` → `v=true` ✓, `` `${1<2}` `` → `true` ✓, `"v="+o.f` → `v=true` ✓,
  `"v="+a[0]` → `v=true` ✓. The `e4b5f7138` fix covers string-conversion sites for THOSE
  provable operands; it does not cover the **direct `console.log` argument position** for any
  operand. **Round 4 correction: "the sole remaining hole" overstated this** — the direct-log
  position is the sole hole for operands this entry's producer list covers (comparisons, `!`,
  `&&`/`||`, ternaries, plain bindings, and a literal-selecting `??`), but R-08 residual 6 is a
  SECOND, independent hole in the concat/multi-arg lanes themselves, for a `??` whose left
  operand is an unprovable boolean-returning call — that hole is not owned by this entry and
  does not close with it. **Round 5 correction: "function returns" is also in this entry's own
  producer list (above), and for that producer the direct-log position is NOT the sole hole
  either — no `??` is required.** `function isEven(n){return n%2===0;} console.log("a:"+
  (isEven(4)))` prints kali `a:1`, node `a:true`, in the plain concat lane, no `??` anywhere.
  This is a THIRD, independent hole — not R-08 residual 6 (there is no `??` in the repro) and
  not this entry's own fix (the value never reaches a `console.log` argument). It is tracked as
  its own entry, **R-34** below, because its root cause is a third code path neither this entry
  nor residual 6 touches. So, precisely: the direct-log position is the sole concat/multi-arg
  hole only for the producers whose call/operand site already computes `shape: Boolean`
  (comparisons, `!`, `&&`/`||`, ternaries, plain bindings, a literal-selecting `??`, and — per
  residual 6 — a hand-cased intrinsic call reached through `??`); an unprovable **user function
  return**, `??`-wrapped or not, is a further, uncovered hole (R-34).
- **Truthiness is correct throughout** — this is a rendering defect only, not a value defect.
  `if(o.f)`, `if(a[0])`, `if(b)` and ternaries on `const`-bound booleans all branch correctly.
- **Fix-cost read**: because concat/template already render correctly, the missing piece is
  the direct-log argument path lacking the boolean repr the concat path already has, rather
  than a missing `Repr::Boolean` axis end to end. Narrower than the known-defect note implies.
- **Confounder recorded**: sweep A's first pass used `var o={f:true}` / `var a=[true,false]`
  and saw `if(o.f)` take the **else** branch, which looked like boolean value corruption. It
  was not — it was **R-06**. Re-run with `const`, every one of those shapes is correct.

### R-31: `console.log` of an array prints its length; of an object prints `0`

- **Folds in**: D-A-10.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G8.
- **Repro**: `const a=[1,2]; console.log(a);` → node `[ 1, 2 ]`, kali `2` (exit 0) — the
  length, an especially deceptive answer for a 2-element array of small numbers.
  `const o={f:1}; console.log(o)` → `0`. In concat position both collapse too: `"v="+a` →
  `v=0` (node `v=1,2`), `"v="+o` → `v=0` (node `v=[object Object]`).
- **Blast radius**: moderate-high; logging a whole array or object is a routine debug shape.
- **Confidence**: high on behavior.

### R-32: Numbers never use exponential notation — the `1e21` / `1e-7` thresholds are not implemented

- **Folds in**: D-A-4.
- **Verification**: `sweep-only` (both scopes).
- **Root-cause group**: G8.
- **Repro**: `console.log(1e21);` → node `1e+21`, kali `1000000000000000000000` (exit 0).
- **Both thresholds missing, in both directions**: `1e100` → 101 literal digits;
  `123456789012345678901234.0` → `123456789012345690000000` (node `1.2345678901234569e+23`);
  `1e-7` → `0.0000001` (node `1e-7`). The just-inside cases are correct, pinning the boundary
  exactly: `1e20` → `100000000000000000000` ✓ and `1e-6` → `0.000001` ✓. Magnitude handling is
  right; only the switch to exponent form is absent.
- **Two independent number formatters exist and they disagree.** `console.log(1e-7)` prints
  `0.0000001` but `console.log("v=" + 1e-7)` prints `v=1e-7`, which *matches* node. The concat
  path implements the small-number threshold and the direct-log path does not. Any fix should
  unify them rather than patch one, or they will keep drifting.
- **Blast radius**: moderate. Only bites at extreme magnitudes, but the output is byte-wrong
  while looking entirely reasonable — exactly the failure a golden-output fixture catches late
  and a human reading output never notices.
- **Confidence**: high on behavior; the `1e20`/`1e21` and `1e-6`/`1e-7` pairs bracket the
  boundary from both sides.

### R-33: `console.warn` injects a `[warn] ` prefix node does not emit

- **Folds in**: D-A-13.
- **Verification**: `sweep-only-top-level-only`.
- **Root-cause group**: G8.
- **Repro**: `console.warn("hi");` → node `hi`, kali `[warn] hi` (exit 0). `console.error("hi")`
  is correct (no prefix).
- **Blast radius**: low in logic terms, but it breaks any **byte-for-byte** comparison of a
  program that uses `console.warn` — and byte-for-byte acceptance is this project's primary
  correctness method.
- **Confidence**: high.

### R-34: A boolean-returning user function's result renders `1`/`0` in the string-concat and multi-argument `console.log` lanes — no `??` required

- **Folds in**: none (new, round 5, 2026-07-19) — split out of R-08 residual 6, which round 4
  wrongly folded an `isEven`-style ordinary-function example in as `??`-specific and
  annotated with a baseline it never checked. That baseline is wrong: the divergence is present
  with no `??` in the program at all.
- **Verification**: probed directly on a freshly built binary, this round (05255c2bc). Not yet
  swept.
- **Root-cause group**: not G8 (see below) — currently unclustered.
- **Repro**, verified verbatim on a freshly built binary (2026-07-19):
  ```js
  function isEven(n) { return n % 2 === 0; }
  console.log("a:" + isEven(4));   // kali a:1,  node a:true
  console.log("a:", isEven(4));    // kali a: 1, node a: true

  function f(){return 1<2;}
  console.log("v=" + f());         // kali v=1,  node v=true
  ```
  Truthiness and branch selection are unaffected — `if (isEven(4)) …` takes the correct branch,
  and `isEven(4) === true` evaluates the correct comparison in-memory (though *printing* that
  comparison's own result is separately R-30, since it is a direct-log boolean). This is a
  Tier-4 rendering-only defect: the in-memory value is right, only its string rendering is wrong.
- **Mechanism, traced (not inference)**: an ordinary function call that resolves to a known
  callee goes through the GENERIC resolved-call path in
  `crates/kali_codegen/src/emit/call.rs:3112-3123`:
  ```rust
  if let Some(index) = resolved {
      let shape = if self.repr_table.return_repr(callee_name) == kali_common::Repr::F64 {
          ValueShape::Float
      } else {
          ValueShape::Unknown
      };
      function.instruction(&Instruction::Call(index));
      return EmittedValue { produced: true, shape };
  }
  ```
  This is the ONLY shape this call site ever produces for a user function: `Float` if the
  return repr is `F64`, otherwise unconditionally `Unknown` — there is no `Boolean` arm, for any
  function, anywhere in this path. That is not an oversight local to this one site: it cannot be
  written, because `kali_common::Repr` (`crates/kali_common/src/repr.rs:18-38`) has no `Boolean`
  variant at all (`I64`, `F64`, `Object(ShapeId)`, `String`, `GrowableArrayI64`, `AbortHandle` —
  confirmed by reading the enum in full), and no other table in the codebase tracks "this
  function always returns a boolean" (`grep`-verified: `return_repr`, the only per-function
  return-type fact kept anywhere, is the only such query in `kali_codegen`/`kali_types`). So
  `isEven`'s call result is `ValueShape::Unknown` at the moment it is emitted — **before** it
  ever reaches `emit_as_string` (`operators.rs:1537-1572`, shared by `+` and the multi-argument
  console lane via `emit_console_argument_as_string`, `call.rs:60-69`), whose boolean-formatting
  arm is keyed on exactly `emitted.shape == ValueShape::Boolean` and is therefore skipped,
  falling through to `int_to_string` and printing the raw `1`/`0` bit pattern.
- **Distinct from R-08 residual 6**: residual 6's mechanism is "`??`'s runtime fallback discards
  a shape the operand emission already computed as `Boolean`" (a hand-cased intrinsic like
  `Number.isInteger`/`Object.is` DOES get `shape: Boolean` from its own dedicated call arm,
  `call.rs:1398-1494`/`:1496-1559`). For an ordinary user function there is no such
  already-`Boolean` value to discard — the generic resolved-call path above never produces one —
  so there is nothing for `??`'s fallback to be blamed for, and indeed no `??` appears in this
  entry's repro at all.
- **Distinct from R-30**: R-30's fix target is the single-argument DIRECT `console.log` sink
  (`emit_console_argument`, `call.rs:23-41`), which never inspects `shape` at all. This entry's
  defect is upstream of any sink: `emit_as_string` (used by concat and the multi-argument
  console lane) DOES inspect `shape` correctly — it simply never receives `Boolean` for this
  producer, because the call-emission site above never sets it. Unifying the console formatters
  (R-30's fix) does not touch `call.rs:3112-3123` and would not repair this entry.
- **Does this share a fix with either?** No, verified rather than assumed: all three defects
  live at three different code sites (`call.rs:23-41` for R-30, `operators.rs:2210-2229` for
  residual 6, `call.rs:3112-3123` for this entry). This entry's root blocker is the same
  underlying gap that blocks R-08 residuals 1-4 — no `Repr::Boolean` axis exists anywhere in
  `kali_common::Repr` for a whole-program, cross-function boolean-return proof — but it
  manifests through this third, previously-unregistered code path, so it is filed as its own
  entry rather than folded into either.
- **Not G8** (rendering-divergence cluster: R-30, R-31, R-32, R-33, et al.): G8's signature is
  "the concat path is correct and the direct-`console.log` path is wrong" for a given value
  class. Here concat is ALSO wrong (as is multi-argument console and direct-log) — the failure
  mode is the opposite of what motivates G8's inference, so this entry is not asserted as a G8
  member without further evidence.
- **Blast radius**: potentially large — any boolean-returning helper function (a common pattern:
  `isX`/`hasX`/predicate helpers) silently renders `1`/`0` instead of `true`/`false` wherever its
  result is concatenated or passed as a non-first `console.log` argument, with no diagnostic.
- **Not fixed in this wave** (registration only, per standing instruction). Not yet pinned by a
  dedicated test in `soundness_strict_equality.rs` — this round is documentation-only.
- **Confidence**: high on behavior and on the traced mechanism (source read end-to-end at the
  cited lines); the cluster/root-cause-group placement is deliberately left open rather than
  guessed.

---

## 3. Root-cause clusters

Eight clusters. **Only G1 and G7 are traced in source; the rest are inference from behavioral
signature and are labelled as such.** Grouping errors here are cheap to make and expensive to
act on, so each cluster states plainly what would raise its confidence.

### G1 — Parser fail-open recovery (**traced in source**, high confidence)

- **Members**: R-01 *(traced)*, **R-49** *(traced, CLOSED 2026-07-28)*, **R-50**
  *(traced, inverted — added to this line 2026-07-29)*, **R-54** *(traced, added
  2026-07-29)*, R-43 *(**inferred, untraced** — see the membership-evidence note below)*.
- **Membership evidence, added 2026-07-29 at the R-35 close-out.** This line previously
  listed R-43 flat alongside R-01 and R-49 while the "Traced" bullet below named sites for
  R-01 and R-49 **only**, so a reader could not tell which memberships were measured and
  which were inferred. They are now labelled, and the two corrections are:
  - **R-50 was claiming G1 membership that this line did not record.** R-50's own entry
    (§7) states *"Root-cause group: G1 (parser fail-open recovery) — inverted"*, but G1's
    Members line never listed it. The claim is sound and is now recorded here: R-50 is
    the **inverted** member — the other members fail *open* (the stream desynchronizes and
    the parser runs on or stops early, silently), whereas R-50 fails *closed* on valid
    input (`E2000` on a sequence-expression `switch` discriminant that both node and kali's
    own pre-stage baseline `f1d02e872` accepted). Same underlying cause — *a parser
    position whose contract with the token stream is wrong* — opposite failure direction.
    It is included deliberately: a cluster defined by its **mechanism** must hold members
    that fail in either direction, or the mechanism is being confused with the symptom.
  - **R-43's membership is INFERRED, not traced, and is now labelled as such.** R-43 (array
    destructuring assignment is a no-op) has **no parser site named anywhere in this
    file**, and unlike R-01 and R-49 it has no `### R-43` §2 entry to carry one — it exists
    only as a §0.3 bullet, which asserts *"Cluster G1"* without evidence. The **only**
    supporting datum in the register is §7.9's retained `P5-R-destructuring-assign` bullet,
    which records that *"the AST shows the statement decaying into two unrelated
    `ExpressionStatement`s, no diagnostic"* — an **AST-shape observation**, which is
    consistent with a G1 fail-open recovery but does not identify the recovery site and
    does not exclude the alternative that the parser never had a destructuring-assignment
    production at all (an unimplemented form, not a fail-open one). **Locating that site is
    the open work**; until it is located, R-43's G1 membership should be read as a
    hypothesis. The cheapest discriminator: if a discarded `accept` or a blind `advance()`
    is on the path, it is G1; if the expression parser simply has no target-pattern arm,
    it is not, and R-43 belongs in its own group.
- A failed `accept(...)` whose `Result` is discarded (`let _ = …`) followed by `break` leaves
  the token stream desynchronized and silently drops the remaining statements.
- **Traced**: R-01 at `crates/kali_parser/src/declaration.rs:29-30`; R-49 at
  `crates/kali_parser/src/statement.rs` (`parse_switch_statement`'s clause loop,
  pre-`9db9150c0`); R-50 at the same function post-`5c9bbd051` (the `expect(kind)`
  hardening — the discriminant position accepts no comma); **R-54** at
  `crates/kali_parser/src/statement.rs:561-564` (the `default` arm's statement-loop stop set,
  which omits `Default` where the sibling `case` arm at `:536-541` includes it). **R-43 has
  no traced site**; see the membership-evidence note above.
- **`parse_switch_statement` has now yielded THREE independent G1-class defects** — R-49 (a
  closer inspected but not consumed), R-50 (a required-token position too strict after the
  hardening) and R-54 (an incomplete stop set). None was found by looking for the next two
  after fixing the first. That is the strongest available argument for G1's standing
  recommendation: **sweep the pattern, do not patch the site.**
- **R-49 is the same cluster with the dual failure mode**: not a discarded `accept` result but
  a closer that was *inspected and never consumed*, which desynchronizes the stream in the
  opposite direction — the enclosing parser stops early instead of running on. Confirmed the
  **unique** non-consuming block-closer in the parser; the three sibling sites
  (`parse_block_statement`, `parse_class_body`, `parse_arrow_function_body_expression`) all
  consume theirs.
- **Standing risk**: this is a *pattern*, not one site. Every discarded `accept` result in the
  parser is a candidate for the same class. A sweep of `let _ = self.stream.accept` is cheap
  and should be done as part of any fix. See §4's blind-`advance()` inventory for the
  measured size of the un-swept surface.

### G2 — Call lowering: unresolvable callee folds to constant `0` (inference, medium confidence)

- **Members**: R-02, R-05; possibly R-03.
- **Signature**: the callee body never runs, the call expression evaluates to `0`, exit 0.
  Uniform across function values, aliases, parameters, returned functions, object-literal
  methods and `this`.
- **Inference, not traced**: nobody read the call-lowering code. The competing explanation is
  several independent zero-emitting fallbacks that merely look alike. **Raising confidence:
  instrument the call-lowering path and count the `0`-emitting fallback sites.** If it is one
  site, one allowlist closes the whole cluster; if it is several, this cluster is fictional
  and each needs its own fix.
- The correction in R-02 (direct sibling capture works) shows the Stage C closure lane is a
  genuine admitted lane sitting *inside* this cluster, not an exception to it — which is
  consistent with "allowlisted shapes work, everything else falls to `0`".

### G3 — Guards whose own diagnostic text names the unsoundness that leaks past them (high confidence as a *pattern*, inference as a shared *mechanism*)

- **Members**: ~~R-11 (bitwise compound assign bypasses the `E5506` that `+=` honors)~~ —
  **CLOSED 2026-07-25, and the claim is now INVERTED on the object-field lane**: `o.a &= 3`
  lowers and computes `2` (node `2`), while its arithmetic sibling `o.a += 1` still fails
  closed `E5506` (measured on both `main` and HEAD). The G3 *pattern* stands — that pairing
  was real when written — but R-11 is no longer an instance of it, and the specific "bitwise
  bypasses the `E5506` that `+=` honors" phrasing no longer describes any lane. See §2's R-11
  close note. Remaining members: R-12
  (one alias binding bypasses the literal-array-store `E5506`), R-18 (a string *literal*
  operand bypasses the `&&`/`||` runtime-string `E5506`), R-08's `??` half (`??=` fails closed
  on the exact indistinguishability that `??` fails open on), R-03 (`forEach` absent from the
  array-callback denylist that fires for `map`), R-13 (computed-member admittance keyed on key
  *shape*, so a variable key falls through).
- These six are **not one code path**. What they share is a *shape of mistake*: a guard keyed
  on one syntactic form or one operand kind, with a sibling form slipping past into precisely
  the miscompile the guard's message describes. In four of the six the compiler's own
  diagnostic text is a written admission of the bug that is live one shape away.
- This is the class this repository has closed before only by replacing the denylist with an
  **allowlist at the choke point** — recorded in the Spec-4a for-in-key lesson, the
  throw-fallout Stage 5 lesson, and the Stage D review lesson. The register's strongest
  recommendation is that each of these six be fixed that way and not by adding the missing
  shape to the denylist.
- **Confidence**: high that the pattern is real (six independent instances, four with the
  guard's own text as evidence); the cluster asserts no shared code.

### G4 — There is no value distinct from the scalar `0` (inference, medium-high confidence)

- **Members**: R-08 (`===`, `??`), R-21 (`undefined`/`null`/absent rendering and arithmetic),
  partially R-23 (`typeof`).
- **Signature**: `null`, `undefined`, `false` and `0` all lower to the i64 scalar `0`;
  comparisons are plain `i64.eq` with no tag discrimination; absent reads return the zero of
  whatever type the consumer inferred.
- **Corroboration**: the `??=` lowering's own `E5506` text ("null and 0 are indistinguishable
  for a scalar value") is a direct statement of this cluster's thesis by the compiler itself.
- **Complication that keeps this at medium-high rather than high**: R-21 records that
  `f() === undefined` *does* take the true branch and `if (f())` *does* take the falsy branch,
  so some `undefined` sentinel exists and behaves. Either the sentinel is `0` and the
  comparison succeeds coincidentally, or there are two representations. Resolving that
  question is prerequisite to any fix here.
- **Note**: R-22 (`==` cross-type coercion) is deliberately **not** in this cluster. Its
  same-type and `null==undefined` cases are correct, so its table exists and one rung is
  missing — a different mistake.

### G5 — A string handle reaches a consumer that never proved it was a string (inference, medium-high confidence)

- **Members**: R-16 (per-method repr arms), R-17 (plain string arrays, empty `.join`,
  `Object.keys`), R-18 (string literal in `&&`/`||`), R-15's element half.
- **Signature**: an interned-string handle — a NaN-box-shaped i64, all observed values in the
  `-92233543…` range — is rendered as an integer, at exit 0.
- **Strong corroboration**: the *same value* prints correctly when it reaches `console.log`
  directly and corrupts only in concat position (R-16, R-17). That position-dependence is hard
  to explain except as a consumer-side proof obligation that some sinks discharge and others
  do not.
- **Best-traced member**: R-16 names both halves of the hand-mirrored pair —
  `crates/kali_codegen/src/intrinsics/string.rs` lowers the methods,
  `crates/kali_types/src/static_analysis/string.rs` fails to mark the results `Repr::String`.
  This is the exact hazard recorded in project memory (`kali-substring-runtime-spec2`): codegen
  oracles and `kali_types` predicates are hand-mirrored, so a new expression kind needs arms on
  **both** sides or it fails open.
- **Raising confidence**: enumerate every producer of a string handle and every consumer that
  renders one, and check that each consumer's admittance is an allowlist. If the fix is one
  allowlist at the concat/repr choke point, this cluster is real.

### G6 — Unresolved or unimplemented builtins fold to a default instead of failing closed (inference, medium confidence)

- **Members**: R-19 (`String`/`toString` → `0`), R-20 (`JSON.stringify` → `0`), R-15 (`split`
  → empty array), R-24 (`Object.freeze` → identity), R-25 (array spread → `len=1`).
- **STATUS 2026-07-20 (G6 item 4 shipped)**: R-19/R-20 CLOSED for canonical spellings; R-15
  runtime lane deny-set-closed (static concat leak residual R-A4-4); R-25 PARTIALLY closed
  (fold sites only); R-24 DEFERRED (needs write-barrier/dataflow, not a fold gate). NET
  mechanism = a value-builtin DENY-SET at emit_call's terminal fallback with warn+0 as the
  restored default — NOT the "one choke-point fix makes all unknown builtins fail closed"
  originally hypothesized (measurement proved the terminal is a SHARED choke point also reached
  by ~300 unresolved-import calls + ~50 host fail-soft surfaces → a 361-test blast radius;
  see the SDD ledger G6 section). RESIDUAL denylist leaks R-A4-1..3 (globalThis-qualified /
  aliased receivers) closable only by an allowlist at the resolve choke point (Group 3).
- **Signature**: a builtin that is not implemented produces a type-plausible zero value rather
  than a diagnostic.
- **The discriminating control already exists**: `Number(...)` fails **honestly** with
  `E3100: undefined identifier 'Number'`, and `parseFloat`/`parseInt` fail with a precise
  `E5506`. So the compiler *has* the honest behavior; some builtins are on a path that
  bypasses it.
- **This is the cheapest high-value structural fix in the document** (sweep A's assessment,
  which this register endorses): if unknown-builtin calls fail closed at one choke point, five
  entries convert from silent-wrong to honest errors at once.
- **Raising confidence**: call any other plausible-but-absent builtin and observe whether it
  yields `0` or `E3100`. That is a five-minute experiment and it either confirms or destroys
  the cluster.

### G7 — Binding storage: `const` has no cell, non-`const` composite initializers are lost (partly traced, medium confidence)

- **Members**: R-07 (**traced**: `control_flow.rs:1284-1286` + `:1614-1616`), R-06, R-29,
  R-10, and R-02's `let`/`var`-vs-`const` boundary.
- **The traced half is solid**: a local `const` that gets no slot stores the *initializer node
  id* and re-emits it at each read. That single fact explains R-07 entirely and explains R-29
  as a consequence (there is no cell to write).
- **The inferred half is the interesting one and is explicitly a guess.** Two sweeps
  independently found the **same polarity** on unrelated surfaces:
  - sweep A: `const o={f:7}` correct, `var`/`let o={f:7}` → all fields `0` (R-06)
  - sweep C: `const g = <fn literal>` correct, `let`/`var g = <fn literal>` → call yields `0`
    (R-02's boundary)
  In both, `const` works and the mutable forms lose a *composite/heap* initializer. That is
  suggestive of one storage decision — perhaps that only `const` initializers are inlined at
  use sites (R-07's mechanism) and therefore only `const` composites are ever materialized,
  while `let`/`var` allocate a scalar slot that a composite initializer never writes.
- **This grouping is inference, not traced.** It is also the single most valuable one to
  either confirm or kill, because if true, R-06, R-07 and part of R-02 are one fix, and if
  false, R-06 is an unowned defect nobody has diagnosed. R-10 (block shadowing) is placed here
  only because it is also a binding-table defect; that placement is the weakest in this
  document.

### G8 — Per-sink rendering divergence: the direct-log path and the concat path are separate formatters (inference, medium-high confidence)

- **Members**: R-30 (booleans render in concat, not in direct log), R-32 (the `1e-7` threshold
  is implemented in concat, not in direct log), R-31 (array/object direct log), R-33
  (`console.warn` prefix), R-04 (the console family's argument handling), R-23 (`typeof`),
  R-28's rendering half, R-21's `"v="+undefined` → `v=false` vs `console.log(x)` → `0`.
- **Signature**: for at least three independent value classes (booleans, small floats,
  `undefined`), the concat path is *correct* and the direct-`console.log` path is *wrong* —
  and in R-21's case the two produce two *different* wrong answers for the same value.
- **This is a strong, cheaply-actionable inference**: there is not one renderer with holes,
  there are (at least) two renderers that have drifted. Every fix in this cluster should
  unify them; patching one will simply re-open the drift, which R-30 and R-32 show has
  already happened twice.
- **Raising confidence**: locate both formatting paths and diff their case tables. If they are
  literally two functions, this cluster is proven rather than inferred.

**Unclustered** (isolated mechanisms, no shared-root claim): R-09 (`continue` update),
R-14 (returned array), R-22 (`==` coercion rung), R-26 (unary `+` digit accumulator),
R-27 (comma operator), R-28's value half, R-34 (boolean-returning user function loses shape in
concat/multi-arg console — deliberately not asserted as a G8 member; see R-34's own note).

---

## 4. Evidence integrity — standing warnings

**This repository's diagnosis has been confounded at least seven distinct ways.** Every one
below actually happened, either in these sweeps or in the prior work they build on. Treat this
section as a checklist, not as background.

**The standing rule: verify in the fixture's own scope, and validate the instrument before
trusting it.**

1. **Top-level vs in-function scope are different programs.** Module scope in kali is not
   function scope, and there are live module-scope-only defects (the known
   `const a=[]; a.push(1)` no-op; the module-scope literal-array element store that silently
   no-ops where the in-function form fails closed). The previous revision of
   `pr16-honest-repin-inventory.md` was **wrong in a way that would have written falsehoods
   into `main`** for exactly this reason — it triaged 694 tests with top-level reproducers and
   misattributed the failure reason of six whole families. Anything marked
   `sweep-only-top-level-only` in §2 carries this risk today.

2. **`console.log` silently drops arguments (R-04) — the primary instrument is broken.** Any
   probe written as `console.log(label, value)` reports only the label. Multi-argument probes
   in this repository's history are unreliable by construction. **Rule: exactly one argument
   per call, built by literal-rooted concatenation** (`"x=" + v`). This applies to
   `console.error`, `console.warn` and `console.info` identically — and `console.warn`
   additionally injects a `[warn] ` prefix (R-33) that will corrupt a byte-for-byte diff.

3. **Do not build a side-effect counter out of a growable array.** A growable array that
   escapes (via a function argument, a return, or module scope) fails closed or silently
   no-ops depending on shape, so the counter measures the compiler's array lane rather than
   the effect under test. Use a **module-scope mutable scalar** (`let n = 0; n = n + 1;`), and
   note that reading a mutable module binding from *inside* a function fails closed with
   `E5506` — so in-function side-effect evidence must use `console.log` inside the callee
   instead. Both sweeps B and C hit this and had to change instrument mid-sweep.

4. **`cmd | tail` makes `$?` the exit status of `tail`.** Any harness that pipes kali's output
   before capturing the status reports exit 0 unconditionally, which erases the single most
   important signal distinguishing "fails closed" from "silently miscompiles". Capture the
   status of the *command*, and prefer `PIPESTATUS`/`set -o pipefail` if a pipe is
   unavoidable.

5. **Constant-folding probes can hide the very defect they test.** The `Object.freeze` case is
   the worked example (R-24): `const o={x:1}; Object.freeze(o); o.x=99` diverges from node,
   but the "obvious" one-liner `const o=Object.freeze({x:1}); o.x=99` **agrees** with node
   because the literal folds and the write is dropped for unrelated reasons. A probe that
   folds is not a probe. Bind first, operate second, and prefer values the compiler cannot
   see through.

6. **A default parameter anywhere in a fixture silently deletes the rest of it (R-01).** This
   is the most corrosive item in this document, because it does not produce a wrong answer —
   it produces a *shorter program*, at exit 0, with no diagnostic. Any fixture, probe, or
   minimized reproducer in this repository that contains a default parameter has been
   silently truncated, and any conclusion drawn from it — including "this shape is correct" —
   may be an artifact of the code that never ran. **Grep any evidence base for `(` … `=` …
   `)` parameter defaults before trusting it.**

7. **Near-miss agreements are traps.** Two are documented here: `String(42).length` prints `2`
   and matches node, which looks like proof `String()` works — it is a constant `2` for every
   input (R-19); and sweep A's first boolean pass saw `if(o.f)` take the wrong branch and
   concluded booleans were corrupted, when the actual cause was R-06 dropping the `var`
   initializer. A single agreeing data point is not evidence; vary the input and check that
   the *agreement* varies with it.

8. **Fix reports are unreliable — re-run the reproducer on a freshly built binary.** Recorded
   in project memory from Spec 5, and re-confirmed here: the controller's re-run of sweep C's
   "closures are effectively nonexistent" finding **falsified** it (direct sibling capture is
   correct; only returned closures are broken), and the controller's own `E4201` observation
   for a mixed-shape file did not reproduce on two nearby variants. Both corrections are in
   R-02.

9. **Any statement after a `switch` was reparented to module scope until 2026-07-28 (R-49).**
   `parse_switch_statement` never consumed the switch's closing brace, so the enclosing block
   parser stopped at it and everything after the `switch` — to the end of the enclosing
   function — was hoisted into the module body and executed at module load, even when the
   function was never called. **Every probe in this repository whose fixture contains a
   `switch` was measuring that leak, not the feature under test.** R-35's originally recorded
   boundary is the known casualty and has been re-derived
   (`r35-switch-boundary-rederived.md`); other pre-`9db9150c0` findings with a `switch` in
   the fixture should be re-run before being relied on. CLOSED by `9db9150c0`.

### 4.1 `e2::EXPECTED_TOKEN` (E2000) is now emitted — a standing fact about the evidence base has changed

Until 2026-07-28, **"the parser has never reported a missing required token" was true of this
compiler.** `e2::EXPECTED_TOKEN` (E2000) and `e2::UNEXPECTED_TOKEN` (E2001) were declared in
`crates/kali_error/src/_error_codes.rs` and emitted from **nowhere**; a required token that
was simply absent fell into a recovery arm that skipped it silently. Any past inference of the
form "kali accepted this file, therefore the syntax is supported" is unsound for that period.

`5c9bbd051` added `Parser::expect(kind)` (`crates/kali_parser/src/parser.rs:62`) and routed
all six required-token positions in `parse_switch_statement` through it: `switch`, `(`, `)`,
`{`, each clause's `:`, and the closing `}` at EOF. Verified on the built binary at that
commit:

```
$ kali run <switch missing its '('>    error[E2000]: expected LeftParen but found Identifier   (exit 1)
$ kali run <switch missing its ')'>    error[E2000]: expected RightParen but found LeftBrace   (exit 1)
```

**Consequence for future probing**: a malformed fixture now produces a diagnostic where the
old parser was silent. An `E2000` on a switch fixture is **never** a *lowering* verdict — it is
a statement about the parse. This currently holds for `parse_switch_statement` only; every
other required-token position in the parser still recovers silently.

**CORRECTED 2026-07-28 (fix round 2) — `E2000` does NOT imply the fixture is malformed.** This
paragraph originally read "it is a statement about the fixture", which a reviewer falsified
with **well-formed** JS that node accepts. The actual rule is wider and worse:

> `E2000` fires whenever `parse_expression` cannot fully consume the discriminant, whether or
> not the program is valid JavaScript.

The confirmed trigger is a **sequence-expression discriminant** — `switch (x, x) { … }` is
valid JS, but `parse_expression` stops at the comma, so the `expect(TokenType::RightParen)` at
`crates/kali_parser/src/statement.rs:508` reports `error[E2000]: expected RightParen but found
Comma` and the file is rejected. This is a **fail-closed regression on valid JS introduced by
this stage**, and it is filed as its own entry — see **R-50** in §7. Anyone reading an `E2000`
must check whether the fixture is malformed *or* merely uses a discriminant form the
expression parser stops short on.

### 4.2 Follow-up work: the blind-`advance()` inventory (NOT attempted in this stage)

Measured on `5c9bbd051`, 2026-07-28 (a count without a named baseline is not a measurement):

```
$ grep -rn "let _ = self.stream.advance();" crates/kali_parser/src/ | wc -l
103
```

Per file at that commit:

| count | file |
|---|---|
| 24 | `crates/kali_parser/src/statement.rs` |
| 19 | `crates/kali_parser/src/module.rs` |
| 15 | `crates/kali_parser/src/declaration.rs` |
| 14 | `crates/kali_parser/src/expression/mod.rs` |
| 12 | `crates/kali_parser/src/expression/primary.rs` |
| 12 | `crates/kali_parser/src/expression/call.rs` |
| 4  | `crates/kali_parser/src/parser.rs` |
| 3  | `crates/kali_parser/src/expression/object.rs` |

`statement.rs` line numbers at `5c9bbd051`: 113, 144, 173, 185, 194, 224, 240, 245, 298, 333,
410, 433, 488, 504, 530, 556, 590, 604, 618, 626, 633, 654, 676, 696.

The pre-stage baseline `f1d02e872` held **28** in `statement.rs` and **107** parser-wide;
Tasks 2 and 3 of the R-35 stage replaced four of them with `accept`/`expect`. Every remaining
site advances the stream **without checking what it consumed**, which is the R-49 / G1 failure
mode with the check removed rather than discarded.

**This was deliberately not attempted here.** A parser-wide sweep is its own project: each
converted site can turn a currently-silent acceptance into a new diagnostic, so it carries a
full test-census cost (the R-35 stage's own gate baseline exists precisely because that cost
is not free), and it cannot be validated by the switch fixtures this stage owns. Filed as
follow-up work, sized above.

---

## 5. Impact on the PR #16 merge-readiness effort

**The premise of the 694-test honest re-pin does not hold.**

`docs/superpowers/followups/pr16-honest-repin-inventory.md` classifies 694 honest-red
workspace tests into class A (kali refuses at compile time with an explicit diagnostic) and
class B (kali silently miscompiles), and the wave tasks instantiate one re-pin per row from
those tables. The re-pin text asserts, per test, *why* kali cannot run it.

That effort assumed the compiler's observable behavior could be trusted as evidence for those
assertions. This register shows it cannot, in at least six ways that bear directly on how the
inventory's evidence was collected:

1. **Default parameters (R-01).** Any fixture or minimized reproducer containing a default
   parameter was silently truncated at exit 0. The observed behavior is the behavior of a
   *prefix* of the fixture. Any row whose evidence came from such a file states a conclusion
   about code that never executed.
2. **`const` initializers (R-07).** `const` has no storage; its initializer is re-emitted per
   read. Any fixture using a `const` snapshot of a mutable value — or a `const` bound to a
   side-effecting call — produced values that are wrong for reasons unrelated to the feature
   under test, and the row's "actual limit" would name the wrong construct.
3. **Multi-argument logging (R-04).** Any probe or fixture using `console.log(label, value)`
   observed only the label. Where a row's classification rests on *absence* of output, that
   absence may be R-04 rather than the feature failing.
4. **Aliasing (R-12).** One interposed binding turns a correctly-refused array store into a
   silent no-op. A row classified **A** (refuses) on the direct form may be **B** (silently
   wrong) on the fixture's actual aliased form, and vice versa.
5. **The A/B boundary itself is unstable.** R-11 (**CLOSED 2026-07-25** — see §2; its pair is
   now inverted, `o.a &= 3` lowers while `o.a += 1` fails closed, so it no longer illustrates
   this in the direction written), R-12, R-18, R-03, R-13 and R-08's `??` half
   each show a *pair* of near-identical shapes where one fails closed and the sibling fails
   open. The class-A/class-B distinction is therefore not a property of a *feature*; it is a
   property of the exact syntactic shape the fixture happens to use. Classifying by feature
   name — which is the failure mode the inventory's own §0 methodology correction was written
   to prevent — remains live at the shape level.
6. **The scope confound the inventory already corrected for is not fully retired.** Ten
   entries in §2 are marked `sweep-only-top-level-only`. Where the inventory's own evidence
   was gathered at module scope for a function-scope fixture, the same class of error is
   possible.

**Consequence: pins written over these defects would encode a false correctness picture into
`main`.** A pin comment saying "kali has no X" is a durable, load-bearing claim. If the real
reason the test fails is R-01 truncating the fixture, or R-07 re-evaluating a `const`, or
R-04 eating the assertion's second argument, then the pin is a *falsehood committed to the
main branch* — which is precisely the outcome the inventory's methodology correction was
written to avoid, arrived at by a different route.

**`pr16-honest-repin-inventory.md` is now SUSPECT wherever its evidence could have been
affected** — specifically any row whose reproducer or fixture involves default parameters,
`const` initializers, multi-argument logging, or aliased array/object bindings. It is not
wholesale invalid: its §0 methodology correction is sound and its in-scope census method is
the right one. What is invalidated is the assumption that in-scope execution of a fixture
observes the fixture's own semantics.

**Recommended sequencing for PR #16**: fix the evidence-corrupting defects (§6 group 1) first,
then **re-derive** the affected inventory rows against a binary containing those fixes, before
any further re-pin wave lands. Re-pinning on the current binary buys pins that will have to be
rewritten.

A `SUPERSEDING EVIDENCE` note pointing here has been added at the top of
`pr16-honest-repin-inventory.md`.

---

## 6. Recommended fix ordering

Effort and risk are rough T-shirt estimates from the mechanism evidence in §2-3, not from any
attempt at a fix. "Risk" means risk of the fix itself causing regressions or being larger than
it looks.

### Group 1 — Evidence-corrupting: fix before trusting any further diagnosis

Nothing else in this list, and no further PR #16 re-pin work, should be believed until these
land. Each one silently invalidates probes rather than merely miscompiling programs.

| # | entry | effort | risk | note |
|---|---|---|---|---|
| 1 | **R-01** default param truncates the module | **small** | **low** | Traced to one discarded `accept` at `declaration.rs:29-30`. Make the failed `accept` a hard parse error; defaults then fail closed like the arrow form already does. Sweep the parser for sibling `let _ = …accept` sites in the same change. |
| 2 | **R-04** console family drops arguments | small–medium | low | One choke point, four sinks. Must cover `log`/`error`/`warn`/`info` together (R-33's stray `[warn] ` prefix is in the same code and should go with it). Highest value per line of change in the document: it repairs the instrument every future investigation depends on. |
| 3 | **R-07** `const` is not a binding | **medium** | **medium-high** | Traced to two sites. The obvious fix — promote all `const` declarators to local slots, reusing the `self.locals` arm that already handles arrays — is small to write, but `const` inlining is load-bearing for the module-constant lanes (for-in key tables, `is_pure_module_const_init`), so it will move a lot of generated code. Gate carefully and expect fixture churn. |

### Group 2 — Contained fixes with a known shape

Each is a bounded change against an identified mechanism, and several are the same edit
applied at different sites.

| # | entry | effort | risk | note |
|---|---|---|---|---|
| 4 | **G6 / R-19, R-20, R-15, R-25** unknown builtins fold to `0` | small | low | **Do the cluster experiment first** (§3 G6): call an absent-but-plausible builtin and see whether it yields `0` or `E3100`. If one choke point routes them, a single "unknown builtin ⇒ fail closed" edit converts four entries from silent-wrong to honest errors. Highest structural payoff for the effort in this document. — DONE 2026-07-20 (partial: R-19/R-20 canonical + R-15 + R-25 folds; R-24 deferred to Group-3/own-plan). See SDD ledger G6 section + R-A4-1..5 residuals. |
| 5 | **R-11** bitwise compound assignment | ~~small~~ **medium** | low | ~~Write-back is simply missing.~~ **DONE 2026-07-25** (`0104f5baf`..`9dcdcc3c1`). The "write-back is simply missing" sizing was wrong for the reason §2's R-11 close note records: the operators never tokenized, so the whole lexer→AST→parser→HIR→types path had to be built first (T1.5) before any codegen fix had an input. The rest went as recommended: one shared combiner (`emit_bitwise_i32_op_extend`), four target arms, and everything else routed to `E5506` by a positive-evidence allowlist rather than a denylist of shapes. |
| 6 | **R-09** `continue` skips the `for` update | small | low–medium | Add a dedicated continue target before the update expression. Self-contained; `while`/`do-while`/`for…of` are already correct and give a reference lowering. |
| 7 | **R-16** per-method string repr arms | small | low | Add the missing `Repr::String` arms in `kali_types/src/static_analysis/string.rs` for `slice`/`charAt`/`toUpperCase`/`repeat`, mirroring `substring`. **But this is the hand-mirrored-oracle hazard itself**: prefer a structural change that makes the two tables impossible to desynchronize over adding four arms that the next method will again omit. |
| 8 | **R-24** `Object.freeze` no-op | small | low | Either implement the write barrier or fail closed on `freeze`. Failing closed is defensible and cheaper. Verify with the bind-first probe (R-24's caveat), not the folding one. |
| 9 | **R-33, R-32, R-31, R-30** rendering divergences | small each | low | All in G8. Do **not** patch the direct-log path in isolation — R-30 and R-32 both show the two formatters have already drifted twice. Unify them, then these four are one change plus test churn. |
| 10 | **R-26** unary `+` digit accumulator | small | low | Add the `0..=9` range guard, whitespace trimming, and a `NaN` path. Mechanism is fully understood (predicts six divergent outputs exactly). Note this lane is load-bearing for `+process.argv[2]`. |
| 11 | **R-27** comma operator | small | low | Emit the last operand with `want_value=true`. |
| 12 | **R-28** `-0` | small | low | Low priority; recorded for arithmetic-map completeness. |
| 13 | **R-22** `==` cross-type coercion | small | low | One missing rung in an otherwise-present table. |

### Group 3 — Guard-hole closures (do these as one project, allowlist-first)

R-03, R-12, R-13, R-18 and R-08's `??` half are five instances of G3. Individually each is a
small edit; **doing them individually is the mistake this repository has already made
repeatedly**. Project memory records four separate occasions where a denylist was patched
shape-by-shape and leaked again, and one (Spec 4a for-in keys) where it took six rounds before
a structural default-deny at the single choke point closed the class by construction.

- **Effort**: medium as a project, small per site.
- **Risk**: medium — an allowlist at a choke point will refuse programs that currently compile,
  which will turn currently-green fixtures red. That is the *correct* direction (refusing beats
  lying) but it must be budgeted, and it interacts directly with the PR #16 test census.
- **Recommendation**: for each of the five, find the single read/store/admit site and convert
  the guard to a default-deny allowlist. Do not add the missing shape to the denylist.

### Group 4 — Architectural; needs its own design pass

These are not bounded fixes. Each is a missing model, and each should get a brainstorm before
any code.

| entry | scope of work | note |
|---|---|---|
| **R-08 + R-21 (cluster G4)** — no value distinct from scalar `0` | **large, architectural** | Requires a tag/repr axis that distinguishes `null`, `undefined`, `false` and `0`. Touches `===`, `!==`, `??`, every absent-value read, arithmetic coercion, and every rendering sink. The `??=` diagnostic already states the problem in the compiler's own words. **Prerequisite**: resolve the §3 G4 complication — `f() === undefined` currently works, so establish whether there is already a second representation before designing a third. |
| **R-02 + R-05 (cluster G2)** — first-class function values | **large, architectural** | The honest interim move is far cheaper than the full fix: make the call-lowering choke point **fail closed** for any callee outside the admitted lanes (statically-resolved name, `const`-bound literal, Stage C env-pointer closure). That converts an extreme silent-miscompile into an `E5506` in a small change, and defers real indirect-call support to its own stage. Strongly recommended as a near-term action even though the full capability is architectural. |
| **R-10** block shadowing | medium–large | Requires the resolver to push a scope frame per block. Contained in concept, but it changes binding identity everywhere and interacts with R-07's storage change; sequence it after R-07. |
| **R-06** `var`/`let` composite initializers dropped | unknown | **Diagnose before estimating.** If cluster G7's inference holds, this falls out of the R-07 storage fix. If it does not, this is an undiagnosed defect of very high blast radius with nobody's mechanism attached to it, and it needs its own investigation first. Resolving G7 either way is the single highest-information cheap experiment in this document. |
| **R-14** returned arrays read back as zeros | medium | Suspect the escape/arena analysis (returned objects are promoted, arrays evidently are not). Interacts with the arena reclamation lanes shipped in Specs 6-7; treat as an escape-analysis change, not a codegen patch. |
| **R-23** `typeof` | small–medium, **but check history first** | Project memory records a `typeof` codegen flip that was **reverted** in throw-fallout Stage 5 per the decision rule. Establish whether that revert is what leaves this open before re-doing the work — and whether the decision traded a test regression for a live silent miscompile. |

### Not recommended for fixing yet

**R-29** (`const` reassignment silently ignored) is a consequence of R-07 having no storage
cell. Re-evaluate it after R-07 lands rather than adding a resolver check now.

---

## 7. Fail-loudly-but-wrong defects (not silent — recorded for completeness)

Every entry in §2 is scoped to exit-0, no-diagnostic divergences (see the note under the tier
table in §1). This section is for the opposite shape: kali exits **nonzero** with a
**diagnostic**, so nobody's trust in an exit-0 result is at stake, but the diagnostic is the
wrong *kind* — an internal-error code (`E4201`, "WebAssembly translation error") rather than
the project's honest fail-closed code (`E5506`) that names the actual limitation, the way fix 5
does for calls through a first-class function value in this same commit range. A user hits an
opaque compiler-internals message instead of a clear one. Added by soundness-batch1-pra wave 0.

### FL-01: A const-bound, expression-bodied arrow whose result is a float emits WASM that fails to validate (`E4201`)

- **Verification**: reproduced on a freshly built binary (base `00ff4ecc0`), 2026-07-19. This is
  the deterministic, pre-existing shape a wave-0 brief asked to be re-checked — NOT the
  intermittent `E4201` the controller once chased for a mixed-closure-shape file (see the
  correction inside R-02 above); that sighting did not reproduce on nearby variants, while this
  one reproduces on every variant probed (11 shapes, see below).
- **Repro**:
  ```js
  const half = (x) => x / 2;
  console.log(half(5));
  ```
  kali: `error[E4201]: failed to load WASM module: WebAssembly translation error` (exit 1) —
  node: `2.5` (exit 0).
- **Mechanism — TRACED, not inferred.** `kali build` (unlike `kali run`) succeeds and writes a
  `.wasm` file; the malformed module only surfaces when something loads/validates it. Running
  `wasm-tools validate` on the built module gives the exact cause:
  ```
  error: func 33 failed to validate
  Caused by:
      0: type mismatch: expected i64, found f64 (at offset 0xc1d)
  ```
  `wasm-tools print` shows the function itself:
  ```wat
  (func (;33;) (type 22) (param i64) (result i64)
    (local i64 i64)
    local.get 0
    f64.convert_i64_s
    i64.const 2
    f64.convert_i64_s
    f64.div
    return
    i64.const 0)
  ```
  The function's declared WASM signature is `(result i64)`, but its body computes a genuine
  `f64.div` and `return`s that f64 value directly, with no conversion back to the declared
  type. The arithmetic lowering correctly recognizes this as float computation (both operands
  are converted to f64 before dividing); the function-signature/return-type inference for this
  specific binding shape does not agree, and declares an `i64` result anyway — a repr
  disagreement between the body emitter and the signature emitter for one binding shape.
- **Repr-triggered, not closure-triggered — boundary probed with 11 variants on a freshly built
  binary**: `function half(x) { return x / 2; }` (named function declaration) and
  `const half = (x) => { return x / 2; };` (block-bodied arrow, note the braces) both compile
  and run correctly (`2.5`, matching node). Only **const + arrow + EXPRESSION body (no braces)
  + float-valued result** hits the mismatch. The float-ness, not the division, is the operative
  variable: `const g = (x) => 1.5;`, `const g = () => 1.5;`, `const g = (x) => 3.5 + x;` and
  `const g = (x) => x * 0.5;` all fail identically; `const g = (x) => x + 1;` (integer-valued)
  succeeds. `let half = (x) => x / 2; half(5);` does not reach this bug at all — it hits fix 5's
  honest `E5506` instead (calling through a non-const function value), which is further evidence
  this is specific to the *admitted* const-bound-arrow lane, not the general call path.
- **Severity**: not a silent miscompile — exits 1 with a diagnostic, so no false confidence is
  created. The defect is that the diagnostic is `E4201` (an internal WASM-translation failure)
  rather than a diagnostic naming the actual gap (a repr mismatch in float-returning
  expression-bodied const arrows).
- **No fix in this wave** — inventory only, per the wave-0 brief. A fix would need to make the
  const-arrow return-type inference agree with the arithmetic lowering's float classification
  (or vice versa) for the expression-body shape specifically.

---

### R-50: A sequence-expression `switch` discriminant is rejected `E2000` — valid JS that this stage stopped accepting

- **Added**: 2026-07-28, R-35 switch-lowering stage, fix round 2, from the whole-stage
  adversarial review of `f1d02e872..0b1c48532`.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — reproduced on freshly built binaries at
  **both** the pre-stage baseline and the current branch tip, and differentially compared.
- **Numbering note**: this is a numbered `R-nn` entry filed in **§7**, not §2, because it is
  **not** a silent miscompile — kali exits nonzero with a diagnostic. It is therefore *not*
  counted in §1's tier table, which counts `### R-` headers under §2's tier headings only.
  `grep -c "^### R-"` over the whole file returned **38** while §2 held **37** tier-ranked
  entries; both numbers were correct and they measure different things.
  **Re-counted 2026-07-29** at the R-35 close-out, after R-51/R-52 were added to §2's
  Tier 1 and R-53 to §2's Tier 2: `grep -c "^### R-"` now returns **42** while §2 holds
  **41** tier-ranked entries (8 + 26 + 2 + 5). The difference is still exactly **1**, and it
  is still this entry — R-50 is the sole `### R-` header outside §2's tier headings.
- **Cross-referenced 2026-07-29** from
  `docs/superpowers/followups/r35-switch-boundary-rederived.md` ("What this matrix does NOT
  cover, and the entry that does"). That file had no reference to R-50 at all, which meant a
  reader treating it as *the* R-35 boundary document would have concluded the boundary is
  exactly what `switch_plan` admits. It is not: the true boundary is the **intersection** of
  what the parser accepts and what `switch_plan` admits, and R-50 is the one known shape
  where the parser is the narrower of the two.
- **Root-cause group**: G1 (parser fail-open recovery) — inverted. G1's other members fail
  *open*; this one fails *closed* on valid input, from the same underlying cause: a parser
  position whose contract with the token stream is wrong.
- **Repro** (`docs/superpowers/followups/r35-switch-boundary-fixtures/seq1.js`, also
  `disc/d01_seqexpr.js`):
  ```js
  function s(x) {
    switch (x, x) {
      case 1: return "A";
      default: return "D";
    }
  }
  console.log("v=" + s(1));
  ```
  **node v26.5.0**: `v=A` (exit 0). **kali at `f1d02e872`** (pre-stage): `v=A` (exit 0) —
  agreed with node. **kali at `0b1c48532`** (this stage):
  ```
  error[E2000]: expected RightParen but found Comma
  error[E2000]: expected LeftBrace but found Comma
  ```
  empty stdout, **exit 1**.
- **Mechanism (traced)**: a sequence expression is valid in a discriminant position, but
  `parse_expression` stops at the comma. Task 3 routed the discriminant's closing paren
  through the new `expect(TokenType::RightParen)` at
  `crates/kali_parser/src/statement.rs:508`, which now reports the residual comma. Before
  Task 3 that position discarded its result, so the leftover tokens were skipped silently and
  the program happened to compile.
- **Severity**: **fail-closed, not a miscompile.** No exit-0 wrong answer is created and no
  trust is misplaced — a valid program is refused, loudly. That is a usability and
  compatibility regression, strictly better than the silent acceptance it replaced, and it is
  recorded here so it is not mistaken for a lowering verdict (see §4.1).
- **Blast radius**: low. A comma operator in a `switch` discriminant is rare. But the *class* —
  "a discriminant form `parse_expression` stops short on is now a hard error" — is only as
  narrow as the search that bounded it.
- **Bounding search — empirical, NOT exhaustive.** 13 discriminant forms were measured on both
  binaries plus node (`r35-switch-boundary-fixtures/disc/`, 2026-07-28). **`x, x` is the only
  form of the 13 whose verdict this stage changed**; the other 12 measure identically at
  `f1d02e872` and at the branch tip:

  | form | pre-stage | this stage | node | changed by this stage? |
  |---|---|---|---|---|
  | `x, x` | `v=A` exit 0 | **`E2000` exit 1** | `v=A` exit 0 | **YES — this entry** |
  | `typeof x` / `!x` / `x.length` | `v=D` exit 0 | `v=D` exit 0 | `v=D` exit 0 | no |
  | `a[0]` | `E5506` exit 1 | `E5506` exit 1 | `v=A` exit 0 | no (pre-existing) |
  | `"lit"` / `x === 1` / `-x` | `v=A` exit 0 | `v=A` exit 0 | `v=D` exit 0 | no (pre-existing **R-35**) |
  | `(x)` / `x ? 1 : 2` / `x++` / `g(x)` / `x` | `v=A` exit 0 | `v=A` exit 0 | `v=A` exit 0 | no |

  Other unlisted forms — `yield`, `await`, assignment expressions, `in`/`instanceof`,
  destructuring — were **not** tested. Treat the bound as "no second instance found in 13
  forms", never as "the sequence expression is the only one".
- **Confidence**: high on behavior (differential, both binaries, both freshly built); high on
  mechanism (the `file:line` is named and the error text matches the token exactly).

---

## 7.9 Stage P5 sightings (2026-07-23)

Silent miscompiles observed while building Stage P5 (`String()` coercion +
`TextEncoder`/`TextDecoder`). These are **sightings + cross-references only** — no
fixes were attempted, and existing entries are NOT renumbered. Each is measured on
the freshly built HEAD binary; all pre-existing unless marked NEW. Full context is
in `docs/superpowers/followups/stageD-triage.md` §8.6 (the "Stage P5" SHIPPED
entry inventory), whose item numbers are cross-referenced below.

Maps to an existing register entry:

- **Block-function-declaration shadow of a handle name** (§8.6 #16, F-newD-1) — a
  hoisted `{ function u(){} u.pathname }` returns the OUTER handle's real value
  (URL `/p`, crypto `8`) where node gives `undefined`, exit 0. Bypasses both
  binding chokes structurally (a fn decl is its own plan, introduces its name
  through no declarator/for-of node), so the Stage-P5 `stale_provenance_shadow_lane`
  guard cannot see it. **≈ R-10** (block-scoped shadowing unmodeled) — the same
  root, a different introduction site than the block-`const` redeclaration R-10
  documents, and NOT closed by the P5 guard.
- **Numeric block-scope divergence** (§8.6 #19, P5-R-blockscope-numeric) —
  `let s=7n; function f(){ { let s=0n; s+=1n; } return s; } f()` → 1, node 7n, no
  `String()` involved. **≈ R-10.** Sound w.r.t. the new `numeric_bindings` proof
  today, but the proof is keyed on a function-granular scope model; any R-10 fix
  must revisit both sides together.
- **Array handle stored into an aggregate reads back with the wrong length** (§8.6
  #8, P5-R-aggregate-array-provenance; and the leaking alias/return routes in #9,
  P5-R-newA-residuals I-4) — `const o={buf:rb}; o.buf.length` → 1 (node 4);
  `holder[0]=rb; holder[0].length` → 2 (node 4); `const z=fb; z.length` → 0;
  `function mk(){return fb} mk().length` → 1 (node 4). The emitted values are the
  child-count / holder-length — maximally plausible wrong numbers. **≈ R-14** (an
  array returned from a function reads back as zeros) — same escape/arena
  provenance-loss family, now also seen through object-field and index stores.
- **`class`-method bodies return `0`** (§8.6 #17, P5-R-classmethod-zero) —
  `class Foo{m(x){return 'A';}} new Foo().m('x')` → 0; surfaces now that a
  shadowing `class TextEncoder` correctly takes the user lane. **Corresponds to the
  Stage-5 "class-method bodies return 0" finding** (recorded in
  `kali-throw-fallout-stage5.md`); the `function` spelling of the same shadow fails
  closed.
- **Computed / method string-length and rendering divergences** (§8.6 #10
  P5-R-computed-length → `s["length"]` = 0; #15 P5-R-tostring-length →
  `arr.toString().length` = 1; #11 P5-R-bytelength-undef → `.byteLength` on a
  runtime string = byte count where node gives `undefined`). Same per-sink /
  per-method string-repr family as **R-16** (per-method string-repr gap) and the
  computed-member handling of **R-13**.
- **`String(x)` result leaks a tagged handle once it leaves its choke** (§8.6 #5
  F-newB-1, #6 F-newB-2/3/4) — `function g(y){return String(y)} const s=g(1n);
  'x'+s` → `x-9223354375949254655`, node `x1`. The P5 String coercion is sound at
  its own choke but there is no `Repr::String` return seed, so the value reads as a
  raw handle at `+`-concat/template/`.byteLength`. **Related to R-16/R-17/R-19** —
  a string value reaching a consumer that never proved it was a string (cluster
  G5). Note R-19 ("`String(x)` … silently return `0`") is now PARTLY SUPERSEDED:
  P5 made bare-identifier `String(x)` coerce correctly; the residual leak is the
  return-seed/concat-site gap, not a blanket `0`.

Appears NEW (no clean pre-existing register entry):

- **Module-scope growable `push` is a silent no-op** (§8.6 #7,
  P5-R-modulescope-growable-push, HIGH) — `const g=[]; g.push(7); g.length` → 0,
  `g[0]` undefined, `g.join('-')` empty (node 1/7/7); also dropped when the push is
  inside a function targeting a module-scope growable. Fixed-size module-scope
  arrays are fine. A silent WRITE loss — worse than a read divergence, since every
  downstream reader sees a plausible empty array with `warnings:[]`, exit 0. No
  existing register entry covers the module-scope growable write lane specifically
  (distinct from R-06's `var`/`let` initializer drop, which is const-vs-non-const
  and about the declaration, not `push` on a `const [] ` at module scope).
- **`globalThis.String(1n)` folds to `0`** (§8.6 #4, P5-R-globalthis-string, NEW
  in Task 6) — the member-call form prints `0` (exit 0) where node prints `1`. The
  bare-identifier `String(1n)` now coerces (P5 Task 1); the member-call spelling
  hits the unresolved-member/call-folds-to-`0` path instead. **Closest existing
  entry is R-02** (calling through a first-class function value returns `0`) / the
  G2 unresolvable-callee-folds-to-`0` cluster, but the specific
  `globalThis.<builtin>(...)` member spelling is not separately entried.
- **The for-of / block-`const`-redeclaration shadow family, now PARTIALLY CLOSED**
  by the P5 T-new-D `stale_provenance_shadow_lane` guard — a for-of or block-const
  redeclaration shadowing a name bound to a TextEncoder/TextDecoder marker, a bytes
  handle, a URL/USP handle, an abort handle, an Event marker, or a
  `getRandomValues` result now fails closed (E5506) at BOTH binding chokes rather
  than serving the stale handle. Recorded here so the register reflects that this
  slice of the R-10 shadow hazard is closed for the eight P5/P4/P3 name-keyed lanes;
  the block-fn-decl introduction site (F-newD-1 above) and the general R-10 scope
  model remain open.

Was filed as "Appears NEW", but is NOT — re-categorized 2026-07-25:

- **Parser silently drops destructuring assignment** (§8.6 #18,
  P5-R-destructuring-assign, HIGH) — `let a=0n; [a]=[1n]; console.log(a)` → 0,
  node `1n`; the AST shows the statement decaying into two unrelated
  `ExpressionStatement`s, no diagnostic. A parser fail-open recovery (cluster G1).
  **The owning ID is `R-43`** (§0.3), which the 2026-07-24 re-derivation gave the same defect;
  this bullet's original claim that *"no register entry covers destructuring-assignment drop
  specifically"* is **FALSE**, and it was moved out from under the "Appears NEW" heading above
  because that heading asserts the very thing R-43 refutes. Both repros measured identical on
  merged `main` (`372a3f440`), 2026-07-25: `let a=0n; [a]=[1n]; a` → `0` (node `1n`) and R-43's
  own `let a=1,b=2; [a,b]=[b,a]` → `1,2` (node `2,1`). Retained here only for its AST-decay
  mechanism datum, which R-43 lacks.

---

## 7.10 R-11 sightings, accepted costs and lessons (2026-07-25)

Found while closing **R-11** (bitwise compound assignment, branch
`r11-bitwise-compound-assign`). Everything in the first block is **pre-existing** — each was
re-measured on a `main`-worktree binary (`62d786e74`) with no bitwise operator anywhere in the
program, so none of it is caused by R-11. **Nothing here was fixed**; these are sightings, and
no existing entry was renumbered. Oracle: node v26.5.0.

**Reconciled 2026-07-25** (after the 2026-07-24 register re-derivation landed as `372a3f440`,
which was written from the same `62d786e74` base and merged separately, so the two efforts
documented several of the same defects twice). Every sighting below was re-measured on a
freshly-built binary at merged `main` (`372a3f440`); the four that duplicated an existing entry
are now one-line cross-references to the ID that owns them, and the two that owned nothing were
promoted to **R-47** and **R-48**. No finding was dropped: every datum a bullet carried and its
owning entry lacked was folded into that entry first.

### Sightings (pre-existing, verified by measurement, unfixed)

- **Element stores into, and reads off, a `let` array literal are silently dropped** — owned by
  **R-06-R3** (§2, R-06 residuals), which now records the `let` spelling and the `.length`
  datum. Cross-reference only; nothing here is a separate lane from the `var` spelling.
- **`for..of` over a `let` array binding iterates the characters of the binding's NAME** —
  genuinely new; it now owns **R-47** (§2, Tier 2). Cross-reference only.
- **Whole-object reassignment is a dropped write** — owned by **R-06-R2** (§2, R-06 residuals),
  which now records that the `let` spelling measures identical to the `var` one.
  Cross-reference only.
- **An array stored into an `I64` object field reads back `0`** — it had no owning ID; it now
  owns **R-48** (§2, Tier 2), a distinct observable from §7.9's
  `P5-R-aggregate-array-provenance` (which reads a wrong *length*, not `0`).
  Cross-reference only.
- **There is no block-scoped `let`** — owned by **R-10** (§2) and stated in §0.2's R-10 row;
  all three agree, so this is a cross-reference. Retained in full only because it is R-10's
  **declaration-only** form, which needs no assignment anywhere in the program (R-10's own entry
  now also carries this datum and the direct re-measurement, under Verification).
  `let n = 6;` / `{ let n = 7; console.log(n); }` / `console.log(n);` prints **`7`** then
  **`7`**; node prints `7` then `6`. Exit 0, no diagnostic; re-measured on merged `main`
  (`372a3f440`). There is not one assignment operator in the repro — the inner *declaration*
  alone is the write — which makes R-10 a **binding-storage** defect (**G7**), not an
  assignment defect, and confirms it is unrelated to R-11.
- **`expr_is_provably_not_bigint`'s BigInt-literal check is `text.ends_with('n')`**
  (`crates/kali_codegen/src/lower.rs`). A bare `Value` node's text is either a literal *or an
  identifier*, so any identifier ending in `n` — `n`, `len`, `min`, `fn`, `in`, `train` — is read
  as a BigInt literal. **Over-taint only**: the misread makes the predicate return `false`
  (unproven ⇒ tainted ⇒ denied), and it cannot under-taint, because the arm can only turn a
  would-be `true` into `false`. But it silently disables the interprocedural
  parameter-inflow arm for those names, so a program using the canonical `n` gets a strictly
  weaker proof than one using `k`. Recovery: distinguish literal from identifier at the node
  level instead of by suffix.
- **The imported-module hole — the one place an R-11-unsound program still reaches exit 0
  silently, and it is unpinnable by construction.** Imports are never analyzed, so the R-11
  resolve gate cannot fire inside imported code. With `lib.ts` = `export const s = "hi"; export
  function bump(){ let n = 6; n &= s; console.log(n); }` and `main.ts` = `import { bump } from
  "./lib.ts"; bump();` — kali exits 0 printing **nothing**, with **zero diagnostics**; node
  prints `0`. This is the tracked **"static named imports never link"** bug
  (`kali-throw-fallout-stage5.md`): the call is dropped, so the unsound line never runs *and*
  never gets diagnosed. It cannot be pinned as an R-11 regression test, because the pin would
  assert the import bug's behavior rather than R-11's; when static imports are made to link,
  the R-11 gate will start seeing this code and must be re-audited at that time.

### Accepted costs and follow-ups (deliberate, fail-closed, pinned — recovery work, not defects)

- **The float taint set is name-keyed** — an over-denial, but **not** of correct programs.
  `collect_float_tainted_module_scalars` / `collect_float_tainted_captured_cells` key on the
  binding NAME over module-global slot names, so an unrelated same-named local elsewhere in the
  program over-denies the real target. **Every number here names the binary it was measured on**
  (see the correction note below):

  ```
  let flags = 6;
  function other(){ let flags = 6.5; return flags; }   // unrelated, same name
  other();
  function f(){ flags |= 8; }
  f();
  console.log(flags);
  ```
  | binary | result |
  |---|---|
  | `main` / `e416b22a1` (code-identical — `62d786e74..e416b22a1` is docs-only) | **`6`** — the R-11 silent no-op |
  | HEAD `9dcdcc3c1` | `E5506` |
  | node v26.5.0 | `14` |

  So relative to `main` this is **silently-wrong → fail-closed, i.e. an improvement**, not a
  lost-correct program. The same holds across the whole shadow axis of the 294-cell matrix:
  every `shadow-float-*` and `shadow-bigint-*` row (18 rows) prints the unmodified `22` on
  `main` where node gives `2/23/21/176/2/2`, and HEAD denies all of them.
  **Correction (2026-07-25).** An earlier revision of this bullet said "node `14`, pre-R-11
  `14`, HEAD `E5506`" and reported "**168 rows `MATCH → E5506`**". Both were baselined on a
  **mid-branch** binary, not on `main`. **The baseline binary, named:** both numbers are
  relative to **`d61821a46`** — the Task-6 review round-1 build, i.e. the parent of
  `961726acd`, which is the commit that introduced this scan
  (`collect_float_tainted_module_scalars` and the shared `collect_float_tainted_scalars`; the
  captured-cell half, `collect_float_tainted_captured_cells`, already existed at `d61821a46`).
  So the `14` comes from a build in which the bitwise lowering existed but the module-global
  float scan did not, and the 168 rows are an **intra-branch, round-over-round** delta
  `d61821a46 → 961726acd`, measured by the round-2 reviewer over a 576-program shadow corpus —
  and over that corpus **0 rows moved `ok → wrong`**, the same bound the next bullet states for
  `write_value_is_numeric`: this over-denial is entirely refusals, never a new wrong value.
  Stated against `main`, the honest count from the 294-cell matrix
  is **2 cells move `MATCH → E5506`** — and see the next bullet for what those two are.
  `flags = flags | 8` (the plain-operator spelling) does give `14` on `main`, which is
  presumably how the wrong value was captured.
  Recovery: re-key by
  `(owner, name)` for the module-global **and** captured lanes **at once** — they share
  `collect_float_tainted_scalars`, and re-keying one alone would leave the other blind.
  **Never delete the scan**: it is the only guard that refuses a float on either lane
  (`is_f64` reads the promoted slot's repr, and `write_value_is_numeric`'s literal arm accepts
  `6.5` — a float IS "numeric" by that proof), and without it the lane emits an invalid module
  (`E4201`). Deleting it to "recover" the 168 rows would recover nothing that ever worked.
- **`write_value_is_numeric`'s allowlist is narrower than correctness needs.**
  (`crates/kali_types/src/repr_infer.rs`.) It admits only a numeric/BigInt literal, a
  self-reference, a PARAMETER of the current function, and unary/binary arithmetic over those.
  A target initialized from a non-parameter identifier (another local or a `const`), a CALL, a
  MEMBER read, or an INDEX read therefore gets no positive evidence and is denied.
  **Baselines, because this number has two of them and they say opposite things:**
  - **Relative to mid-branch commit `820e3dd91`** (the round-2 parent, where the bitwise
    lowering existed but `binding_is_proven_numeric` was not yet in the target guard):
    **6 of 32 programs (~19%)** of the local-scalar bitwise lane move `ok → DENY`, none
    `ok → wrong`. This is the number the pin's own comment records, and it is an
    **intra-branch, round-over-round** delta.
  - **Relative to `main`** (`e416b22a1`, code-identical): of those same six pinned rows,
    **four were ALREADY WRONG on `main`**, and the two that matched node did so **only by
    coincidence, because the operator was a mathematical identity on that value** — so the
    R-11 silent no-op happened to equal node's answer:

    | # | program | `main` | node | HEAD |
    |---|---|---|---|---|
    | 1 | `let a=3; let b=3; let n=a*b; n\|=0;` | `9` | `9` | `E5506` — coincidence (`9\|0 == 9`) |
    | 2 | `function f(){return 6;} let n=f(); n<<=2;` | `6` | `24` | `E5506` — already wrong |
    | 3 | `let o={a:3}; let n=o.a; n\|=1;` | `3` | `3` | `E5506` — coincidence (`3\|1 == 3`) |
    | 4 | `const c=6; let n=c; n<<=2;` | `6` | `24` | `E5506` — already wrong |
    | 5 | `let m=6; let n=m; n<<=2;` | `6` | `24` | `E5506` — already wrong |
    | 6 | `function f(){return 7;} let n=0; n=f(); n<<=2;` | `7` | `28` | `E5506` — already wrong |

    **`main` never once computed a bitwise compound assignment correctly.** The 294-cell matrix
    says the same thing independently: its only two pre-R-11 `MATCH` cells are
    `member-of-string` with `&=` and `|=` (`const s="abc"; let n=s.length; n&=3;` → `3`, node
    `3` — because `3&3 == 3` and `3|3 == 3`), and the same target with
    `^= <<= >>= >>>=` was WRONG on `main`. So the honest main-relative figure is **2 of 294
    cells `MATCH → E5506`, both coincidence matches**, and there is **no** program in any
    measured corpus that `main` genuinely got right and HEAD refuses.

  Recovery: teach `write_value_is_numeric` member/call/local-identifier
  inflow — **not** a loosening of the codegen guard, and emphatically not a "recovery" of
  behavior that never existed. Pinned by
  `bitwise_compound_over_denies_write_values_outside_the_numeric_proof`; **do not weaken that
  test** — widening the proof should make it need updating on the *admit* side, not deletion.
- **Three object-field write routes are uncovered by the BigInt/float taint scan and are safe
  ONLY because those writes are currently silently dropped**: computed `o[k] = v`,
  arrow-parameter dot write (`const w=(x)=>{ x.a = 7n; }`), and for-of element dot write
  (`for (const o of os) { o.a = 7n; }`). `collect_bigint_tainted_shape_fields` walks only
  object-literal declarator inits and static dot-field writes. Three tripwire tests pin the
  current dropped-write behavior (`bitwise_compound_tripwire_{computed_key,arrow_parameter,
  forof_element}_write_not_covered_by_bigint_taint_scan`) — pinned as *current behavior*, not as
  certified-correct output (all three diverge from node, which throws). **Do not implement any
  of those write lanes without extending `collect_bigint_tainted_shape_fields` first**: partial
  coverage would be worse than none, because it would look like a proof.
- **`emit_object_field_compound_assign_dynamic` is still unclaimed for static dot fields.** No
  static dot-field *arithmetic* compound assign lowers (`o.a += 1` → `E5506`). If a later task
  opens it, it must reuse the object-field lane's **three-check target proof**
  (`shape_field(..) == Some((_, Repr::I64))` **and** `shape_field_is_proven_numeric` **and**
  `!shape_field_bigint_targets.contains(&(shape, field))`), not the `Repr::I64` default —
  `Repr::I64` is `ReprTable::scalar`'s `#[default]` and proves nothing.
- **DEFERRED — `unstable_provenance_names` omits the six bitwise operators.**
  `crates/kali_codegen/src/lower.rs:2892` lists `= += -= *= /= %= **= ??= &&= ||=` but not
  `&= |= ^= <<= >>= >>>=`, so a bitwise write does not invalidate function-value provenance
  (the guard that refuses to resolve a name through `fn_valued_locals` once a reassignment or
  shadow could have made the recorded mapping stale). **Latent only — no live defect today**,
  and the protection turns out to be double-barrelled: independently confirmed across 13
  shapes, a `let`-bound function value denies the *call*, and the one spelling where provenance
  does resolve a call (`const f = () => 7`) denies the *assignment*
  (`let f=()=>1; f &= 1;` → `E5506` "on a non-integer binding 'f'"; same for the
  function-scoped and called-through spellings). It must be extended **before** any widening of
  bitwise admission — in particular before `write_value_is_numeric` is taught new inflow
  shapes, since that is the change most likely to admit a binding this list does not track.

### Follow-up inventory — structure and coherence debt (whole-branch review, 2026-07-25)

**None of these is a defect and none blocks merge.** They are the structural debt the final
whole-branch review found after eight review rounds; the reviewer explicitly preferred one
recorded inventory over further churn on a branch this deep. Recorded so the work is findable,
in priority order. All line numbers are as of `fc777af54`.

1. **Unify the provenance-scan family (largest, ~350 lines and one whole-program AST walk).**
   `collect_bigint_tainted_module_scalars` (`crates/kali_codegen/src/lower.rs:4454`) and
   `mark_non_i64_tainted_captured_scalars` (`:5198`) are line-for-line identical after comment
   stripping — **93 normalized lines each**, differing only in the function name, the candidate
   container type (`BTreeMap` vs `BTreeSet`, hence `contains_key` vs `contains`) and which
   predicate they call. Their predicates are likewise identical — `expr_is_provably_not_bigint`
   (`:5469`) and `expr_is_provably_i64_literal_or_arith` (`:5319`), **100 normalized lines
   each**, differing only in the recursive self-call and `parse_numeric_literal_value` vs
   `parse_number_literal` at two sites. Because i64-parseable ⇒ f64-parseable,
   `bigint_tainted ⊆ float_tainted`, and both consumers test the two sets in a single
   disjunction (`crates/kali_codegen/src/emit/literal.rs:1454-1455`,
   `crates/kali_codegen/src/emit/closure_access.rs:394-395`) — so **the two BigInt scalar sets
   can never be the sole reason a program is denied**. One predicate-parameterized walk would
   do. **The shape-field BigInt scan at `:4726`
   (`collect_bigint_tainted_shape_fields`) is NOT subsumed and must stay** — it keys on
   `(shape, field)`, walks object-literal inits and dot-field writes, and has no float twin (see
   item 4).
2. **`unstable_provenance_names` should call the complete twin it already has.**
   `lower.rs:2892` is a hand-written assignment-op list that omits the six bitwise operators;
   `is_assignment_operator_text` (`:4231`, same file) is its complete twin and already contains
   all six. **One-line fix: call that function.** Verified inert today — such programs are
   denied twice over — but it is a denylist the language surface has just grown past. Full
   analysis and the ordering constraint are in the DEFERRED bullet above.
3. **A misleading diagnostic (product-side; needs a re-gate).**
   `crates/kali_codegen/src/emit/object.rs:573` reports "*a BigInt value was observed for this
   field elsewhere in the program*" for **every** denial from the object-inflow closure,
   including programs that contain no BigInt at all — e.g. `const o = { a: 6 }; const p = o;
   p.a &= 3;`. The **verdict is correct**; only the stated cause is wrong. Three tests pin the
   substring `"BigInt"` (`crates/kali_cli/tests/soundness_bitwise_compound.rs:1488-1502`), so
   fixing the message means updating those needles — which makes this the one item here that
   touches product code and therefore requires a full re-gate.
4. **Close the object-field float axis.** The object-field lane
   (`crates/kali_codegen/src/emit/object.rs:501`) has BigInt taint but **no float taint**,
   unlike the module-global and captured lanes. That asymmetry is the cause of both pinned
   `E4201` tests. Recorded explicitly as a **confirmed decision, not an inherited oversight**:
   the `E4201` pre-exists and reproduces without the bitwise line at all, but the
   **reachability change is real** — pre-R-11 those programs got a clean `E5506` at resolve, and
   now they reach the invalid-module path. Closing the axis means giving this lane the
   `(shape, field)`-keyed float twin item 1 says the BigInt shape-field scan lacks.
5. **Minor cleanups.**
   - `ReprTable::scalar_entry` (`crates/kali_common/src/repr.rs:279`) is **dead**: public,
     unit-tested (`crates/kali_common/src/repr_tests.rs`), discussed in five doc comments, and
     called by **nothing** — it was superseded in Task 2 round 3 by `binding_is_proven_numeric`
     (an explicit `scalar_entry` record denies 100% of the lane; see the "A default is not a
     proof" lesson). Delete it, or the next reader will assume it is the target-axis proof.
   - `object.rs:584` keeps a redundant `is_float_valued` belt that the other three arms dropped
     once `bitwise_compound_rhs_is_provably_i64` became the positive proof.
   - The deny trio (diagnostic + `I64Const(0)` + `return`) is copy-pasted at four sites.
   - `closure_access.rs:391` holds the only `?` in a bitwise lane (`scalar_capture_owner(name)?`)
     — unreachable, and it wants an `expect` naming the invariant instead of silently
     short-circuiting if that ever changes.
   - `object.rs:596-620` recomputes the store address **around** the RHS evaluation. That is
     safe **only** because the RHS oracle currently admits literals alone, which cannot
     invalidate the base. Wants a note at the site so a future widening of
     `bitwise_compound_rhs_is_provably_i64` does not silently corrupt the reload.

### Lessons this project produced

- **A default is not a proof.** `ReprTable::scalar` is `unwrap_or_default()` with default
  `Repr::I64`, and *nothing in the codebase ever writes `Repr::I64` explicitly* — so
  `scalar_repr(x) == I64` cannot distinguish "proven integer" from "repr_infer recorded nothing
  about this binding at all". Two tasks shipped Criticals built on that reading (a string handle
  truncated by `I32WrapI64` into a wrong-but-plausible integer at exit 0). The fix was not a
  stricter reading of the same accessor — requiring an explicit `scalar_entry` record denies
  100% of the lane — but a *different, affirmatively written* signal,
  `ReprTable::numeric_bindings` / `binding_is_proven_numeric`.
- **A guard keyed on one binding class leaks to sibling classes.** Hit **six times** on this
  project alone (module-global slots → module const inits → module binding names →
  hand-mirrored predicate list → one added `emit_identifier` arm reopened it in a single
  commit). Widening the denylist failed every time. It closed only when the second copy was
  *deleted*: `resolve_identifier_kind` → `IdentifierResolution` is now the single classifier,
  both consumers `match` it exhaustively with no `_` arm, and a new resolution arm is a compile
  error until handled at both sites. Divergence is prevented by the type system, not by
  discipline.
- **State the direction, not the count, unless the axis is proven exhaustive — and name the
  baseline binary, every time.** Three audit rounds each replaced a corpus-bound count with a
  stronger absolute ("all N cells", "the cost is exactly this one shape"), and each time a
  missing corpus axis falsified it in about five lines. This close did the same to its own
  predecessor: the Task-6 "143 cells" figure is a 222-cell-corpus number, and the same
  measurement over the final 294-cell corpus gives 209.
  **And then this document violated the lesson in the very edit that recorded it** — which is
  the most instructive form of it, so it is written down rather than quietly fixed. The first
  revision of §7.10 carried two corpus-bound numbers ("pre-R-11 `14`", "168 rows
  `MATCH → E5506`", "6 of 32 previously-correct programs") that named **no baseline binary**.
  In a document whose stated baseline is `main`, "previously-correct" reads as "correct before
  this project" — and it was false: measured on `main`, those programs were **already silently
  wrong**, and the handful that matched node did so only because the operator was a
  mathematical identity. An unbaselined count is not a weaker claim than a baselined one; it is
  a claim about a binary the reader cannot identify, and here it inverted the sign of the
  finding — turning "we replaced a silent miscompile with a refusal" into "we lost working
  behavior". The concrete hazard is real: it invites future work to loosen
  `write_value_is_numeric`, or delete the float scan this same section warns against deleting,
  in order to recover behavior that never existed. **A number without a named baseline is not
  a measurement.**
- **A fix a task adds must enter that task's own measurement corpus in the same round.** Twice a
  round's blast-radius numbers were computed over a program space that excluded the change the
  round had just made, so the reported cost was of the *previous* build. Re-run the corpus after
  the last edit, not before it.
- **A plan whose examples contradict its own constraints will have the examples followed — and
  that was this branch's single largest avoidable cost.** The plan
  (`docs/superpowers/plans/2026-07-24-r11-bitwise-compound-assign.md`) states Global Constraint
  #2 in one line: *"Allowlist, never denylist. Admit integer targets explicitly; everything else
  fails closed `E5506`. Never add a 'shape to skip' list."* Every one of its code sketches for
  Tasks 2-5 is then a **denylist** — `if <is float> { reject }` at plan lines 296-299, 396, 481
  and 555, with nothing admitted positively. The sketches won: Task 2 needed **four review
  rounds** to convert its guard into a positive proof (`binding_is_proven_numeric` +
  `bitwise_compound_rhs_is_provably_i64`), and Tasks 3, 4 and 5 each re-derived the *same*
  conversion independently on their own lane, each under review pressure, each after shipping a
  first cut built from the sketch. Prose stating a constraint does not compete with code showing
  a shape; an implementer copies the shape. This is the **second instance of one underlying
  issue** — the same issue as the plan's false root cause recorded in §2: *the plan was written
  against code paths never verified to receive input, and its sample code contradicted its own
  stated constraint*. Both are failures of the plan to be checked against anything — the first
  against the running compiler (one token dump would have falsified it), the second against its
  own page. **Rule: if a plan states a constraint, every code sketch in it must be an instance
  of that constraint, or the sketch must be deleted.** A plan reviewer should diff the sketches
  against the constraints before any task starts.

---

## 7.11 R-35 close-out — the switch allowlist, its residual, and what it is coupled to (2026-07-29)

Branch `r35-switch-lowering`, Stage 2, closed at this section's commit. **This section is the
authoritative statement of R-35's boundary.** §0.2's row and §0.3's bullet are summaries and
defer to it; `r35-switch-boundary-rederived.md` is the *pre-fix* measurement and describes a
compiler that no longer exists.

The stage's shape, because it is the reusable part: `SwitchStmt` was allocated with **no
text**, so it reached codegen as a `Branch` with `text: None`, fell into the generic arm — the
`if` lowering — and was emitted as `if (discriminant) { clause-0 } else { clause-1 }`, with
clauses 2+ never emitted at all. The fix was **not** to write a `switch` emitter and then
harden it. It was to add `emit_switch` with an **empty allowlist first**, so that every
intermediate commit on the branch failed closed rather than miscompiling silently, and then
admit one proven shape per task. `switch_plan` returns `Err(reason)` unless it can *prove*
every part of the switch is admitted; there is no denylist of bad shapes anywhere in
`crates/kali_codegen/src/emit/switch.rs`. Extending the admitted set means **adding a proof**,
never removing a rejection.

### What is now CORRECT (matches `node v26.5.0` byte-for-byte)

Pinned by `crates/kali_cli/tests/switch_runtime.rs`. Both module scope and function scope
throughout.

- **Discriminant**: a proven `Repr::I64` scalar, or a proven `Repr::String`. Includes
  parameters, module bindings, function locals, call results, and runtime-built strings
  (`"a" + "b"`, `t.substring(1,2)`) — string comparison is **content** equality via the
  existing `__streq`, not handle identity, so a freshly allocated equal string selects its
  clause.
- **Case tests**: numeric literals including unary `+`/`-`, or string literals — **in the
  discriminant's own domain**. Duplicates are admitted and are first-match-wins by
  construction.
- **Clause terminators**: `return`, unlabeled `break`, unlabeled `continue` under a
  **faithful** enclosing loop, and empty non-`default` clauses that group onto the next
  terminated clause.
- **Grouping**: a run of consecutive empty non-`default` `case` clauses collapses onto the
  following clause with a **non-short-circuiting `i32.or`** fold over N eagerly-evaluated
  comparisons, guarding **one** emission of the body. Unobservable today (case tests are
  literals — no side effects, no non-termination) but it is *not* JS `||` semantics and the
  next reader should not assume it is.
- **`default`**: zero or one, **last position only**.
- **Structure**: `break` binds to the switch and `continue` reaches past it to the enclosing
  loop **by construction** — the switch frame's `continue_index` *is* the enclosing loop's.
  Nesting (switch in switch, switch in loop) works from the same frame stack. The
  discriminant is evaluated **exactly once** into a dedicated per-switch local. A switch opens
  **no arena frame**, verified across 200 allocating iterations.

### RESIDUAL FAIL-CLOSED — the named follow-up list

Every item below is a fail-closed limit with a message naming the actual limit. **Corrected
2026-07-29 (`64438bf0ef`) — the preamble here previously claimed all of them were "routed
through the single `switch_plan` choke point, and pinned in
`crates/kali_cli/tests/switch_fail_closed.rs`". That over-claims on three counts, and the
exceptions are exactly where a future reader would otherwise go looking in the wrong file:**

- **Most rows** (1-4, 7-10, 14) do route through `switch_plan` and are pinned in
  `switch_fail_closed.rs`. For those the original sentence was accurate.
- **Rows 5 and 6** deny at a *different* choke point — `emit_break_or_continue`'s
  `continue_index: None` arm in `crates/kali_codegen/src/emit/control_flow.rs`, not
  `switch_plan`. `switch_plan` *admits* these clauses; the denial happens later, during
  emission. They are pinned (`NO_ENCLOSING_LOOP`, `UNFAITHFUL_CONTINUE`).
- **Row 11** never reaches codegen at all and is not `E5506` — it surfaces as **`E3100`**
  from name resolution. See the row.
- **Row 12** has **no pin**, and cannot have one: it is unreachable dead code (R-54).

**These are named so a later stage can pick one up without re-deriving the boundary.** None is
a defect of this stage; each is work not yet done.

| # | shape | denial constant / note |
|---|---|---|
| 1 | **True fallthrough** — a non-empty clause ending in neither `return`, `break` nor `continue` | Rule 4. The original R-35 hazard; do not widen without re-deriving §7.11's grouping section. |
| 2 | **`let` / `const` in a clause body** | Rule 5. Blocked on **R-10** (block shadowing unmodeled), not on switch. Measured: `let`/`const` in a clause do **not** fail closed on their own — they measured byte-identical to `var` pre-stage — so this denial is load-bearing and must stay on the deny path. |
| 3 | **Non-literal case tests**, and **cross-domain** literal tests (a string case against an i64 discriminant or vice versa) | Rule 2. Cross-domain is denied rather than "silently never matches": node falls to `default` for it and `__streq`'s tag guard happens to agree, but *the two engines agreeing by accident* is not a lowering proof. |
| 4 | **Float, boolean, object, array and unknown discriminants** | Rule 1 — denied by **failing to construct a proof**, not by being listed. See the accepted-regression note on booleans below. |
| 5 | **`continue` with no enclosing loop** | There is no `continue_index` to inherit; `emit_break_or_continue` fails closed. |
| 6 | **`continue` under an UNFAITHFUL enclosing loop** | `UNFAITHFUL_CONTINUE`. **Faithful**: `while`, `for…of`, and a C-style `for` with **no** update clause. **Unfaithful**: a C-style `for` **with** an update, `for…in`, and `do`/`while` — their `continue` skips the update or the test. **This is R-09, not R-35**, and the switch deliberately refuses to widen into it: no switch allowlist can fix a loop-lowering defect. See R-09's corrected "Not affected" line. **THE DENIAL MESSAGE'S SUGGESTED REWRITES ARE MEASURED, AND THE `for…of` ONE IS BINDING-QUALIFIED (corrected 2026-07-29, `64438bf0ef`, fix wave item 3).** This message advised *"use a `while` or `for...of` loop"* — but a bare `for…of` recommendation routes the user straight into **R-53**: `for (let v of […])` and `for (var v of […])` bind every element to `0`, silently, exit 0. Re-measured on this exact fixture at `64438bf0ef`: `while` → kali matches node byte-for-byte (`iter=1..6`, `s=19`); `for (const v of [1,2,3,4])` → matches node (`iter=1..4`, `s=8`); `for (let v of …)` and `for (var v of …)` → kali `iter=0` ×4 and `s=0` where node gives `s=8`. The message now names **`while`**, or **`for…of` whose loop variable is declared `const`**, and says why. Note the two properties are independent and both are needed: `continue`-faithfulness is about *where the index advance is emitted* (all `for…of` lanes are faithful), while R-53 is about *what the loop variable is bound to* (only `const` is right). A future edit to this sentence must re-measure against `node v26.5.0` before naming any construct — routing a user out of an honest denial and into a silent miscompile is strictly worse than the denial. |
| 7 | **A `default` that is not the LAST clause** | `DEFAULT_NOT_LAST`. This lowering emits `default` **unconditionally at its own source position** with an early return from the chain recursion, so any later clause is silently unreachable. See the design-doc correction below — this one **shipped**. |
| 8 | **A `default` grouped with a preceding empty `case`** | `DEFAULT_CANNOT_GROUP`. Grouping would narrow `default`'s "everything else" semantics into a plain equality disjunction. |
| 9 | **A trailing empty clause with no body to group onto** | `TRAILING_EMPTY_GROUP`. No clause remains to carry the accumulated test. |
| 10 | **An empty `default` clause** | Not eligible for `EmptyGroup` (it has no test to hand forward), so it denies via the same Rule-4 "no terminator" message every terminator-less clause gets. |
| 11 | **Labeled `break` / `continue` in a clause** | ~~Binds to an enclosing labeled statement, not to this switch; `emit_break_or_continue` rejects labels globally.~~ **CORRECTED 2026-07-29 (`64438bf0ef`) — that mechanism is wrong.** Labeled STATEMENTS do not survive at all, far upstream of codegen: the label declaration resolves as a bare identifier and raises **`error[E3100]: undefined identifier 'outer'`**. Measured both with and without a switch in the loop — the switch-free `outer: for (var i=0;i<3;i=i+1) { sum = sum + 1; }` raises the *identical* E3100 where node prints `sum=3`, and the switch-bearing fixture emits **E3100 and no E5506 at all** (grep count 0). So no `"break:<label>"` node ever reaches `switch_plan`, `emit_break_or_continue` never gets the chance to reject a label here, and this row is **not an `E5506` row**. `is_unlabeled_break_statement`'s exact-`"break"` match is **defence in depth**, not the operative gate. If labeled statements are ever supported, a clause *ending* in `break outer;` would then be denied by **Rule 4** (that exact match fails), and this row plus `a_labeled_break_in_a_clause_is_fail_closed` must be re-pinned on `RULE_4_TERMINATOR`. `crates/kali_cli/tests/switch_fail_closed.rs`'s `a_labeled_break_in_a_clause_is_fail_closed` already states this correctly and explicitly warns against this row's old phrasing; the row now agrees with its own pin. |
| 12 | **Two or more `default` clauses** | Rule 3, message `"more than one `default` clause"` — **but the check is UNREACHABLE DEAD CODE**, see the note below. |
| 13 | **`throw` as a clause terminator** | **DEFERRED, not denied on principle** (design §5.2). It terminates in principle, but kali's `throw` lowering is its own lane and admitting it needs its own measurement. Pre-stage it measured `E4000` where it fires and **SILENT where it does not**, so it is a real hazard rather than a quiet one. **This is the most valuable single item on this list to pick up next.** |
| 14 | **An EMPTY switch — `switch (x) {}` — with no clauses at all** | **ADDED 2026-07-29 (`64438bf0ef`), fix wave item 2. This is a DENIAL ON VALID JS and was recorded nowhere** — not in this table, not in §0.2's residual list. Node runs `var x = 1; switch (x) {} console.log("done=" + x);` and prints `done=1` at **exit 0**; kali refuses it with `error[E5506]: this `switch` is not in the supported lowering set (a switch with no clauses); rewrite it as `if`/`else if` or use a supported switch shape (fail-closed)`, exit 1 (both measured). **Mechanism:** `switch_plan` folds clauses into `folded`, and after the trailing-empty-group check it returns `Err("a switch with no clauses")` when `folded.is_empty()` (`crates/kali_codegen/src/emit/switch.rs`). The guard is load-bearing, not gratuitous — `emit_clause_chain` has no base case for an empty chain and every downstream invariant in the plan assumes at least one clause — but the shape it refuses is legal and side-effect-free in JS (evaluate the discriminant, match nothing, fall out), so it belongs beside the **two accepted regressions** below rather than among the "work not yet done" rows. Cost is negligible in practice; the point of recording it is that the residual set must be *complete*, since an unrecorded denial on valid input is how a fail-closed compiler's honesty claim erodes. **No pin added in this wave** (documentation-only); a `switch_fail_closed.rs` cell asserting this exact message would be a cheap follow-up. |

**Rule 3's denial cannot fire, and that is R-54.** `switch_plan` checks for a second
`default` at `crates/kali_codegen/src/emit/switch.rs:105`, but the AST can never carry one:
`parse_switch_statement`'s `default` arm omits `Default` from its statement-loop stop set
(`crates/kali_parser/src/statement.rs:561-564`) where the sibling `case` arm includes it
(`:536-541`), so a second `default` is **absorbed into the first** and both bodies run
merged. kali therefore accepts a file node refuses outright with `SyntaxError: More than one
default clause in switch statement`. **Only invalid JS is affected**, so no correct program
is miscompiled and the allowlist's soundness is untouched — but the rule is stated and not
enforced, and it is the one cell of the acceptance matrix that has **no pin** (a `switch_runtime`
test would have to assert a divergence, and a `switch_fail_closed` test would have to assert
an `E5506` that never arrives). Found by this close-out while completing the matrix's
`default` axis; filed as **R-54** (§2, Tier 3, cluster G1) and deliberately **not fixed here**
— a parser behaviour change would land after the whole-stage adversarial review that has
already run.

**Parameter discriminants have an extra, load-bearing rule — read this before extending
anything.** A `switch` on a **parameter** is admitted only when *all* of:

1. the parameter's inflow is proven — numeric-literal, or a proven string — at **every**
   enumerable call site; **and**
2. the parameter is **never written** anywhere in the function body; **and**
3. the enclosing function **does not escape**.

and it is *additionally* denied whenever the function body contains an **array literal, object
literal, or function expression** — **even when the parameter is provably untouched**. That
last clause is not a modelling nicety; it is the conservative boundary of the repr inference
this proof delegates to, and removing it re-opens the aggregate-provenance family (R-06, R-12,
R-14, R-48). Do not "tidy" it away.

### Two ACCEPTED REGRESSIONS — deliberate, recorded as such, not defects

1. **Boolean discriminants (matrix cell 16) now fail closed, and they were previously
   CORRECT.** Cell 16 was one of only **two** cells in the whole 32-cell pre-stage matrix
   where kali agreed with node. It is now `E5506`. This is **deliberate and accepted**:
   `r35-switch-boundary-rederived.md`'s own analysis ("Cell 16 is a coincidence, not a
   capability") shows the agreement was an **ordering artifact** — a boolean is truthy or
   falsy in exactly the way the broken `if`-lowering happened to need, so it was right for a
   reason that had nothing to do with `switch`. **Fail-closed beats accidentally-right**: an
   honest `E5506` tells the user the truth, while a coincidence that survives a refactor
   silently is precisely how this register fills up. Recorded here so a future reader running
   a before/after comparison finds the explanation instead of filing a regression.
2. **`switch (x, x)` — a sequence-expression discriminant — regressed from correct to
   `E2000`.** Filed as its own entry, **R-50** (§7), because it is a *parser* regression on
   valid input, not a lowering decision. Also a fail-closed-on-valid-input cost of this stage,
   and unlike cell 16 it is **not** endorsed — it should be fixed.

### A design-doc claim that was FALSE and caused a shipped miscompile

`docs/superpowers/specs/2026-07-27-r35-switch-lowering-design.md:261-262` asserted:

> *"A `default` in a non-final position is admitted. Once fallthrough is denied, `default`'s
> position carries no semantics. No rule needed."*

**True of JS selection semantics. False of this emission strategy.** `emit_clause_chain`
lowers `default` unconditionally at its own position and returns early from the chain
recursion, so every clause after a non-final `default` is never emitted. It shipped from
Task 7 (when `default` first became reachable in a chain) until Task 10's fix. The claim is
now **struck in place** in the design doc, with the mechanism and the three regression pins,
rather than deleted — because the reasoning error generalizes and is worth keeping visible:

> **"The source language gives this construct no semantics" does not imply "the lowering gives
> it no semantics."** A step from a *language* property to an *emission* property that never
> visits the emitter is unsound. Any future "no rule needed" must be justified against the
> emitter as written.

Noted here as well as in the design doc so that a reader arriving from either direction hits
it. This is the second time in this project a *documented* conclusion was wrong in a way no
test caught — the first was R-09's "Not affected" line.

### Standing couplings — three places a future fix un-masks a leak

Each of these is currently safe **only because a second defect is covering it**. They are
listed together because the pattern is the point: this project produced three instances of
it in one stage.

1. **Optional calls ↔ the switch parameter proof.** `s?.(x)` is an invocation route invisible
   to **both** halves of the parameter proof — the escape walk has no
   `OptionalChainExpression` arm (so no escape mark) and no `CallEdge` is built (so no
   argument evidence). It is latent **only because optional calls are dropped entirely**
   (**R-51**). **If optional-call lowering is ever implemented, the escape gate must be
   extended before or within that change, or the `new`-site leak returns verbatim.**
2. **`for`-clause arity ↔ `continue_is_faithful`.** Three broken `for` arities are flagged
   `continue_is_faithful = true` at `crates/kali_codegen/src/emit/control_flow.rs:348`,
   harmless **only because those loops are already wrong** (**R-52**). **If the arity bug is
   ever fixed, `control_flow.rs:348` must be re-derived in the same change**, or a switch
   `continue` will be admitted into a loop that skips its update — R-09's silent form,
   reached through a construct the allowlist certified.
3. **`for…in` ↔ the faithfulness predicate.** `control_flow.rs:348` reads
   `update.is_none() && kind != "do-while"`. `for…in` is excluded from it only because
   `for-in` is a **different HIR node kind reaching a different emit path**, not because the
   predicate names it. Anyone editing that line must re-check `for…in` explicitly.

### Design note for whoever makes cross-module calls real

Today an exported function **is** admitted as a switch host, and that is correct **only
while a cross-module imported call returns `0` wholesale**. If cross-module calls ever
deliver real arguments, an exported function's parameters can be supplied by a call site the
edge builder cannot see, and the parameter proof becomes vacuous.

**Do NOT fix this by adding an `exported` field to the AST.** That makes every consumer
responsible for remembering a new check — the **denylist-shaped** failure this plan paid for
repeatedly. The real concept is *"an invocation site the edge builder cannot enumerate"*.
**Extend the escape notion once, at the walk**, covering `export`, dynamic `import()` and the
optional call (**R-51**) together.

**And note that the three export spellings differ — matching `ExportNamed` alone silently
misses the common case:**

| spelling | what survives to the AST |
|---|---|
| `export function s(){}` | **Nothing.** `parse_export_declaration` (`crates/kali_parser/src/module.rs:88`) discards the `export` token at `:89` and dispatches straight to `parse_function_declaration()` at `:136-142`. `export function s(){}` and `function s(){}` produce the **identical** AST — no export statement node exists, and nothing downstream can distinguish them. |
| `export { s }` | Survives as `Statement::ExportNamed` (`crates/kali_parser/src/module.rs:176`). |
| `export default …` | Survives as `Statement::ExportDefault` (`crates/kali_parser/src/module.rs:127`). |

So the **first** row — the most common spelling by far — needs a **parser change** before any
downstream escape rule can see it at all. Pinned today by
`an_exported_function_is_admitted_because_export_is_not_an_escape` in `switch_runtime.rs`,
which records the *measurement* rather than the prose; when cross-module calls become real,
that test must flip to fail-closed.

---

## 8. Cross-references

- `docs/superpowers/followups/pr16-honest-repin-inventory.md` — the 694-test adjudication map
  this register calls into question (§5). Carries a `SUPERSEDING EVIDENCE` pointer back here.
- `docs/superpowers/followups/stageD-triage.md` §8.6 — the residual/admittance inventory and
  the ALLOWLIST-1 tripwire; cluster G3 is the same lesson at sweep scale.
- `.superpowers/sdd/sweep-{a,b,c,d}-*.md` — the four source registers, retained for their full
  probe logs, correct-shape inventories (which bound the damage) and fail-closed maps.
- `docs/superpowers/followups/r35-switch-boundary-rederived.md` — the 32-cell, both-scopes
  R-35 boundary matrix measured on `5c9bbd051` (2026-07-28) after the R-49 parser-containment
  fix. **Supersedes the boundary sentence §0.3's R-35 bullet originally carried**, which was
  measured through the R-49 leak. Also carries the traced `switch` lowering mechanism and the
  consequences for the Stage 2 allowlist. **It describes a compiler that no longer exists** —
  for the post-fix boundary read **§7.11**, not this file. Cross-references **R-50** for the
  one shape where the parser is narrower than the allowlist.
- **§7.11 of this document** — the R-35 close-out: the admitted set, the fourteen-item residual
  fail-closed list with its denial constants, the parameter-discriminant rule, the two
  accepted regressions, the three standing couplings, and the design note for cross-module
  calls. **This is the authoritative R-35 boundary**; §0.2's row and §0.3's bullet are
  summaries that defer to it.
- `docs/superpowers/specs/2026-07-27-r35-switch-lowering-design.md` — the Stage 2 design.
  Read with §7.11's correction: its §5.2 note *"a `default` in a non-final position is
  admitted … no rule needed"* is **struck as false** and caused a shipped silent miscompile.
- `docs/superpowers/followups/r35-gate-baseline.txt` — the zero-newly-red gate baseline for
  this project: commit `f1d02e872`, **9466 passed / 0 failed / 27 ignored** across 376 suites.
  The failing list is empty; the file is unsorted, so strip its comment block before feeding
  it to `comm` (the file says how).
- `crates/kali_cli/tests/switch_runtime.rs` and `crates/kali_cli/tests/switch_fail_closed.rs`
  — the admitted-side and denied-side pins for §7.11. Every denied-side assertion checks the
  **specific** denial constant, not merely that some `E5506` occurred, so a cell that
  silently degraded to a different rule still fails.
- `crates/kali_parser/tests/parser_integration.rs`, `mod switch` — R-49's regression pins.
  They moved here from the deleted `crates/kali_cli/tests/switch_parser_containment.rs`; see
  R-49's corrected citation.
