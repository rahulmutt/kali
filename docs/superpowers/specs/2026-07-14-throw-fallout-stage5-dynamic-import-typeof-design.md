# Throw-fallout Stage 5 — dynamic-import member `typeof` (bucket #7)

Date: 2026-07-14
Status: approved (brainstorm complete)
Parent: `2026-07-11-throw-fallout-design.md` (Stage 5)
Branch: `soundness-batch1-pra` (stage entry: 783 failing, Stage 4 certified)

## Problem

All 32 bucket-#7 tests (26 in `browser_template_literal_dynamic_import_harness`,
6 `dynamic_import_file_specifier`/`directory_index` cases in `runtime_smoke`)
fail at the same guard:

```js
const chunk = await import(`./${name}`);
if (typeof chunk.lazyValue !== 'function') {
  throw new Error('missing lazyValue export');   // <- always thrown today
}
```

Ground truth established by pipeline trace + fresh-binary reproducers:

- **Specifier pipeline is green.** Both fold lanes (bundler
  `parse_static_dynamic_import_specifier` in `crates/kali_cli/src/build/eval.rs`,
  type-checker `resolve_static_string_expression` in
  `crates/kali_types/src/static_analysis/string.rs`) already fold template
  literals, sequence exprs, `Object.freeze` wrappers, `??`/`&&`/`||`, and
  const-bound identifiers. Chunk artifacts are emitted per target; the JS glue
  exposes a working `loadDynamicImport` (`cmd_build.rs:1862`).
- **`import(...)` has no codegen.** `ImportExpr` lowers to a textless
  single-child wrapper that `unwrap_transparent_value`
  (`kali_codegen/src/lower.rs:3302`) collapses to its child, so `chunk` is
  bound to the **specifier string**, not a namespace.
- **`typeof` fails open.** For any operand outside the small provable lane,
  the `"typeof"` arm (`kali_codegen/src/emit/operators.rs:152-227`) emits
  `I64Const(0)` plus an `e8::UNIMPLEMENTED` *warning* and keeps compiling.
  `0 !== <interned 'function' handle>` is always true → guard throws. This is
  the silent-miscompile class the program exists to kill.
- **The member call is a MIRAGE (falsified during planning).** An initial
  probe with a `0n`-returning export looked green, but a distinguishable
  probe (`return 7n` + `console.log("inside lazyValue")` in the body) shows
  the body NEVER runs and the call produces constant `0` — for the dynamic
  form, static `import * as ns; ns.f()`, AND static named
  `import { f } from './x.js'; f()` alike. Cross-module calls do not exist
  in the wasm lane; the linked graph is diagnostics-only. The 32 fixtures'
  guards compare against `0n`, which the fail-open `0` fakes — so a
  typeof-only fix would drain the bucket fraudulently (umbrella Invariant 3
  forbids it). Confirmed soundness holes: `ns.notAnExport()` silently
  returns 0 (node: TypeError); `console.log(chunk)` prints the specifier
  string `./lazy.js` (node: the namespace object).

So the honest drain needs TWO lanes: the `typeof` fold AND a real
provenance-routed member-call lane, plus soundness closures around both.

## Approach (chosen: A-expanded — provenance typeof fold + AST-level call linking)

User re-approved the expanded scope 2026-07-14 after the call-lane mirage was
discovered. Rejected alternatives: (B) full namespace value semantics through
HIR/MIR/codegen — a new repr axis, far larger; (C) real runtime dynamic
linking (new host import, cross-instance calls, two memories) — Stage-7-scale
machinery; (D) typeof-only — fake green, disqualified by Invariant 3.

### Components

1. **Namespace provenance (kali_types resolver).** A binding is a proven
   module-namespace iff it is `import * as x from <spec>` or
   `const x = await import(<spec>)` where `<spec>` folds via the existing
   `resolve_static_string_expression` lane AND resolves in the linked graph.
   Provenance carries the resolved module id (not a bare flag). This is
   binding PROVENANCE, not expression-shape matching (Spec-3 lesson).
2. **`typeof` fold on namespace members — BOTH mirrored sides.** New arm in
   codegen `typeof_static_text` AND the kali_types predicate, both consulting
   the same provenance oracle (twin-desync lesson: one-sided arms fail open).
   - function export → `'function'`
   - name not exported → `'undefined'` (namespaces are sealed → provable)
   - any other export kind → fail-closed E-code this stage (fixtures need
     only `'function'`; extend kinds only if collateral demands).
3. **Positional allowlist for namespace bindings (default-deny).** Today the
   dynamic binding physically holds the specifier string, so
   `console.log(chunk)` or `chunk + ''` silently emits the wrong value. Per
   the Spec-4a structural lesson, allowlist safe positions at the
   resolve-identifier choke point — member-call receiver, member-typeof
   operand, being the operand of the binding's own `await`, and
   statement-discard of the whole result — and E-code every other use. The
   allowlist applies to BOTH provenance sources (static `import * as` and
   dynamic), since neither has audited value semantics outside those
   positions.
4. **Generic typeof fallback: measure, then close if cheap** (user decision).
   First stage task flips `I64Const(0)`+warning → fail-closed E-code and
   censuses newly-red across the full workspace.
   - Small blast radius → close it in-stage (fix surfaced lanes).
   - Large → revert the generic flip, close fail-open for namespace-member
     operands only, and file the closure as a follow-up with the measured
     census attached.
5. **Namespace member-call lane via AST-level module linking** (the Spec-5
   monomorphization playbook: AST-level cloning before the resolver, zero
   codegen/repr edits, everything keys on function name). Before resolution,
   for each provenance-proven namespace binding whose module qualifies, pull
   the resolved module's exported function declarations into the entry AST
   under mangled names keyed by (module, export) — e.g. `__mod<N>_lazyValue`
   — deduped per module, and rewrite proven `ns.member(...)` call sites to
   direct calls of the mangled name. Purity gate: only modules whose
   top level consists purely of export-function declarations enter the lane;
   any other top-level statement, nested import, or non-function export →
   fail-closed E5506. `ns.notAnExport()` → fail-closed E-code (today:
   silent 0). Chunk artifact emission is unchanged — the linked copy serves
   the wasm execution path; the emitted chunk still serves real browsers.
6. **Static named-import calls (`import { f }; f()`) are the same fail-open
   class** (discovered by probe; body never runs, call yields 0). In-stage
   fix ONLY if the census shows cheap collateral (the same linking pass
   naturally covers it); otherwise documented follow-up with the census
   attached — but any lane this stage touches must leave no silent-0 path
   behind (fix or E-code, never the status quo) on the surfaces it claims.

## Data flow & error handling

Compile-time only. No new runtime machinery, no new synthetics expected, and
**no new `kali:rt` host imports** — the four hand-mirrored browser import
lists are untouched (re-confirm at the gate). Chunk artifact emission, the JS
glue loader, and statement-form `await import(...)` (result discarded — 39
green tests depend on it) keep current behavior; the new lane engages only
when the namespace value is consumed.

Fixture-shape flow: fold specifier (existing) → resolve module in linked
graph (existing) → binding gets namespace provenance (new) → qualifying
module's exported functions linked into the entry AST under mangled names,
proven call sites rewritten to direct calls (new) → `typeof x.member` folds
from the module's export table on both mirrored sides (new) → returned value
flows through the existing single-module function-call lanes (params, reprs,
arenas — all unchanged by construction since linked functions are ordinary
AST functions).

Reject-don't-miscompile uniformly:

- non-foldable specifier → existing `FEATURE_UNAVAILABLE` reject (unchanged)
- unresolvable specifier → existing `DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH`
  (E4008; `FEATURE_UNAVAILABLE` is E5506 — both in
  `crates/kali_error/src/_error_codes.rs:86,102`)
- namespace binding outside allowlisted positions → compile-time E-code
- member call on a name the module does not export → E-code (today silent 0)
- target module fails the purity gate (top-level statements, nested imports,
  non-function exports) → E5506
- `typeof` on non-function export kinds → E-code
- (pending measurement) generic unproved `typeof` → E-code, never silent `0`

Out of scope: lazy chunk *execution* in the wasm lane (chunk top-level side
effects still don't run — pre-existing divergence, documented follow-up; this
stage adds no execution machinery) and Stage 7 promise/microtask semantics.

## Success criteria

- All 32 bucket-#7 tests green in isolation, **zero test edits** (no re-pins).
- Full-workspace enumeration (`cargo test --workspace --no-fail-fast`) diffed
  against BOTH the 783 stage-entry set and a main worktree: PRIMARY GATE
  (`comm -13 pre post`) EMPTY.
- Denominator drops ~32 + collateral; every drained name reconciled by
  isolation run (output-interleaving lesson).
- Static `import * as ns` typeof gap closes collaterally (same fold).
- GC-less invariant untouched.

## Testing

- **Distinguishable-value probes are MANDATORY acceptance evidence.** The
  fixtures' `0n` returns cannot distinguish a real call from the fail-open
  `0` (that coincidence hid the mirage). Every call-lane test must use a
  return value ≠ 0 (e.g. `7n`) AND a body side effect
  (`console.log("inside …")`) asserted against node output.
- **TDD per lane, red probes first:** call lane (distinguishable probe, both
  static-namespace and dynamic forms); typeof fold; missing member →
  `'undefined'`; each allowlist reject; `chunk.notAnExport()` fail-closed;
  same-name-two-chunks routing to respective targets; purity-gate rejects.
- **Adversarial re-mask probes:** (a) sabotage the fold (function member
  reports `'undefined'`) → fixture guard's throw must fire; (b) sabotage the
  link (call routes to a stub returning 0) → distinguishable probe must go
  red. Proves real answers, not re-silenced guards.
- **Isolation runs:** all 32 targets; guard families that must stay green
  (39 statement-form/literal dynamic-import tests, JS-side loader tests,
  static-import suites).
- **Census sync:** `SYNTHETIC_FUNCTIONS` allowlist check stays on the
  checklist though no new synthetic is expected (bitten twice before).
- **Whole-stage adversarial review** at the end (Stages 3-4 each caught a
  miscompile per-task review missed).

## Execution

`writing-plans` cycle → subagent-driven execution, gated per program policy
(PR #16 stays draft; nothing pushed until the umbrella completes).
