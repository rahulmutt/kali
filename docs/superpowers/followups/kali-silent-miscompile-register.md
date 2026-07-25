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

## 1. Executive summary

**42 raw defects → 33 after deduplication.** Nine entries were folded into siblings that
share a demonstrated or strongly-inferred root cause (noted per entry).

Severity split (each entry ranked at the most severe class it carries):

| tier | class | count |
|---|---|---|
| 1 | **silently drops code or output** — statements never run, calls never fire, output vanishes | 5 |
| 2 | **silently produces a wrong value** | 23 |
| 3 | **silently wrong control flow only** (value otherwise intact) | 1 |
| 4 | **rendering-only** (in-memory value is correct) | 4 |

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
    - **R-06-R2 — whole-object reassignment.** `var o={f:1}; o={f:2}; o.f` → silent-`0`; the object-literal-RHS assignment store is a distinct mechanism from the declarator init. Unchanged.
    - **R-06-R3 — arrays.** `var a=[7,9]` / `var a=[1,2]; a[0]=9` read back `0` — var-array runtime storage largely unimplemented. Own later stage (entangled with R-12/R-13/arena lanes).
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
- **Scopes affected**: both.
- **Not affected**: `continue` in `while`, `do/while` and `for…of` are all correct; `break` in
  `for`/`for…of`/nested loops is correct.
- **Severity**: silent-wrong-value (the `p28b` form) degrading to a hang when the body does
  not otherwise advance the loop variable.
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
- **Verification**: `sweep-only` (both scopes).
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
  **The 2 `MATCH→E5506` cells are the total main-relative cost of this project, and both are
  coincidences**, not working programs: they are `member-of-string` with `&=` and `|=`
  (`const s="abc"; let n=s.length; n&=3;` → `main` `3`, node `3`) — the R-11 silent no-op
  matched node only because `3&3 == 3` and `3|3 == 3`. The identical target with
  `^= <<= >>= >>>=` was WRONG on `main`. Those two are also the ONLY `MATCH` cells `main` scored
  in the whole 294-cell matrix, so **`main` never once computed a bitwise compound assignment
  correctly**. Any later claim that this project "lost working behavior" should be checked
  against that fact first — see §7.10, where an earlier revision of this very entry made
  exactly that error.
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
- So **interposing a single binding (`const b=a`) converts a correctly-refused program into a
  silently-wrong one.** Aliasing an array into a shorter local name is ubiquitous.
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

- **Members**: R-01.
- A failed `accept(...)` whose `Result` is discarded (`let _ = …`) followed by `break` leaves
  the token stream desynchronized and silently drops the remaining statements.
- **Traced**: `crates/kali_parser/src/declaration.rs:29-30`.
- **Standing risk**: this is a *pattern*, not one site. Every discarded `accept` result in the
  parser is a candidate for the same class. A sweep of `let _ = self.stream.accept` is cheap
  and should be done as part of any fix.

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
- **Parser silently drops destructuring assignment** (§8.6 #18,
  P5-R-destructuring-assign, NEW, HIGH) — `let a=0n; [a]=[1n]; console.log(a)` → 0,
  node `1n`; the AST shows the statement decaying into two unrelated
  `ExpressionStatement`s, no diagnostic. A parser fail-open recovery (cluster G1),
  but no register entry covers destructuring-assignment drop specifically.
- **The for-of / block-`const`-redeclaration shadow family, now PARTIALLY CLOSED**
  by the P5 T-new-D `stale_provenance_shadow_lane` guard — a for-of or block-const
  redeclaration shadowing a name bound to a TextEncoder/TextDecoder marker, a bytes
  handle, a URL/USP handle, an abort handle, an Event marker, or a
  `getRandomValues` result now fails closed (E5506) at BOTH binding chokes rather
  than serving the stale handle. Recorded here so the register reflects that this
  slice of the R-10 shadow hazard is closed for the eight P5/P4/P3 name-keyed lanes;
  the block-fn-decl introduction site (F-newD-1 above) and the general R-10 scope
  model remain open.

---

## 7.10 R-11 sightings, accepted costs and lessons (2026-07-25)

Found while closing **R-11** (bitwise compound assignment, branch
`r11-bitwise-compound-assign`). Everything in the first block is **pre-existing** — each was
re-measured on a `main`-worktree binary (`62d786e74`) with no bitwise operator anywhere in the
program, so none of it is caused by R-11. **Nothing here was fixed**; these are sightings, and
existing entries are NOT renumbered. Oracle: node v26.5.0.

### Sightings (pre-existing, verified by measurement, unfixed)

- **An element store into a `let` array literal is silently dropped.**
  `let a=[1,2,3]; a[1] = 5; console.log(a[1]);` → `0`, node `5`, exit 0, no diagnostic. Same
  family as **R-12**/**R-06-R3**, but this is the un-aliased `let` spelling, which R-12's entry
  records as *correctly fail-closed* for `const`.
- **Reads off a `let` array literal return `0` too.** `let a=[1,2,3]; console.log(a.length)` →
  `0` (node `3`); `console.log(a[0])` → `0` (node `1`). So the store above is not merely lost —
  the whole binding reads back as an empty/zero array. **≈ R-06-R3.**
- **`for..of` over a `let` array binding iterates the characters of the binding's NAME.**
  `let a=[1,2,3]; for (const x of a) console.log(x);` prints `a` — one line, the letter `a` —
  where node prints `1 2 3`. `let zz=[1,2,3]; …` prints `z` then `z`; `let a=[10,20]; for
  (const q of a) …` prints `a`. Exit 0, no diagnostic. The `const` spelling
  (`const a=[1,2,3]`) is CORRECT (`1 2 3`) and the `var` spelling fails **closed** (`E5506`
  "for-of array iteration lowering is unavailable…"), so this is the `let` lane only: the
  iterable is being resolved as the identifier's own text and iterated as a string. Maximally
  deceptive — the output is plausible-looking data, and the loop body does run.
- **Whole-object reassignment is a dropped write.** `let o={a:6}; o={a:9}; console.log(o.a);` →
  `0`, node `9`, exit 0. This is **R-06-R2** re-confirmed on current `main` (R-06's objects-half
  close did not touch the assignment-store mechanism, only the declarator init).
- **An array stored into an `I64` object field reads back `0`.** `let o={a:6}; o.a=[1,2];
  console.log(o.a);` → `0` (node `[ 1, 2 ]`); identical through an alias (`let b=[1,2]; o.a=b`).
  The field keeps its `Repr::I64` and the handle is lost. **≈ R-14 / the P5
  aggregate-array-provenance family in §7.9.**
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
  **mid-branch** binary, not on `main`: the `14` comes from a mid-branch build in which the
  bitwise lowering already existed but the float scan did not, and the 168 rows are an
  **intra-branch, round-over-round** delta measured over a 576-program shadow corpus against
  that same mid-branch parent. Stated against `main`, the honest count from the 294-cell matrix
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

---

## 8. Cross-references

- `docs/superpowers/followups/pr16-honest-repin-inventory.md` — the 694-test adjudication map
  this register calls into question (§5). Carries a `SUPERSEDING EVIDENCE` pointer back here.
- `docs/superpowers/followups/stageD-triage.md` §8.6 — the residual/admittance inventory and
  the ALLOWLIST-1 tripwire; cluster G3 is the same lesson at sweep scale.
- `.superpowers/sdd/sweep-{a,b,c,d}-*.md` — the four source registers, retained for their full
  probe logs, correct-shape inventories (which bound the damage) and fail-closed maps.
