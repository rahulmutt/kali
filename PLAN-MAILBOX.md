# PLAN Mailbox

## 2026-04-17 — Parser import syntax acceptance and package-corpus follow-up

Added parser support for import declarations and literal dynamic `import()` expressions so the AST now follows the already-documented import path instead of dropping those statements. That unblocked the package-corpus tests, which now rely on real import parsing again rather than bypassing the syntax entirely.

Suggested follow-up:
- keep an eye on later browser code-splitting work, since literal dynamic `import()` now reaches the AST path that those stages expect
- if a later stage needs explicit import-boundary lowering, wire the plan notes to that stage instead of reintroducing import-syntax gaps

## 2026-04-17 — Stage 3.3 browser-bundle wrapper/source-map progress note

Updated the Stage 3.3 status note to reflect that browser-bundle output now ships with a deterministic source-map companion and explicit `--format esm|cjs` wrapper selection, and that the JSON output records the selected wrapper format and full artifact list.

Suggested follow-up:
- keep the Stage 3.3 status note in sync with any future browser-bundle output-shape changes
- broaden the remaining Stage 3.3 breadth tasks (code-splitting, tree-shaking, package corpus, and cross-module inference) separately from this wrapper/source-map scaffold

## 2026-04-12 — Stage 1.14 configless-install wording aligned to SPEC

`plan/phase-1/14-evidence-hardening.md` previously described `kali install` on a project with no `kali.json` as a clear error, but `specs/14-packages.md` defines the configless-install split as a clean no-op success that must not create a placeholder manifest. The stage note has been updated to match the spec-correct behavior.

## 2026-04-12 — Stage 1.14 raw-URL install idempotence coverage added

Added a regression test in `crates/kali_npm` that installs the same raw-URL graph twice and asserts the resulting `kali.lock` bytes remain identical across both runs. This closes the most straightforward remaining install-workflow determinism gap in Stage 1.14 without changing the underlying install semantics.

## 2026-04-17 — Stage 3.3 browser-bundle source-map companion output

`kali build --bundle` now emits a deterministic `.js.map` companion and appends a `sourceMappingURL` footer to the generated browser bundle JS. The runtime smoke test now checks for the source-map file and validates its basic JSON shape.

Suggested follow-up:
- update the stage-3.3 status note to mention the browser-bundle debug artifact increment
- if we decide to formalize the new output shape, sync the browser bundle artifact docs/schemas afterward

## 2026-04-17 — Stage 3.3 browser-bundle format targets

`kali build --bundle` now accepts a `--format` selector for the browser bundle wrapper: `esm` stays the default and `cjs` emits a CommonJS-flavored JS companion plus a `.cjs.map` source-map sibling. The runtime smoke suite now covers both output shapes and the CLI also rejects `--format` when `--bundle` is absent.

Suggested follow-up:
- update the Stage 3.3 status note to mention the format selector work
- decide whether the CLI/spec docs should describe the bundle-format selector now or defer until the surrounding bundle vocabulary is finalized

## 2026-04-17 — Stage 3.3 package-audit preview plumbing

Implemented the Phase-3 opt-in `package-audit --preview` gate so the command now has a concrete preview-only execution path instead of failing unconditionally. The preview path currently emits the schema-v1 envelope with `payload: null` and a short summary string in both text and JSON modes while keeping the default, non-preview command gate unavailable in earlier phases.

Suggested follow-up:
- decide whether the preview flag should be documented in the CLI/spec set or remain an implementation-only staging hook until the later public availability row opens
- keep the maturity matrix unchanged until the later compatibility row is actually promoted

## 2026-04-17 — Stage 3.2 Node child-process projection expansion

The Stage 3.2 status note now reflects the expanded `kali_api_node` helper surface (`NodeChildProcess`) and the runtime-linker coverage for the `process_spawn` projection under `kali:node`.

Suggested follow-up:
- continue broadening the Node runtime projection to the remaining documented Phase-3 built-ins that still lack host-import wiring
- keep the Stage 3.2 status text aligned with the actual host-import coverage so it does not imply full Node parity yet

## 2026-04-12 — Stage 2.1 alias-chain precision completed

Stage 2.1's remaining escape-analysis gap was closed by teaching `kali_mir` to resolve function-expression aliases through alias chains, including anonymous function expressions lowered to synthetic function names. That keeps direct-call precision intact for `const alias = identity; const alias2 = alias; alias2(...)` style call targets.

## 2026-04-12 — Stage 2.4 provisional Lean model update

`PLAN.md`'s Stage 2.4 row was updated to reflect the checked-in Lean workspace and the fact that the current progress/preservation work is still represented by theorem statements with documented-sorry placeholders.

No change was made to the Phase 2 completion gate yet; it should continue to read as an open gate until the later proof-backed work closes the remaining obligations.

## 2026-04-12 — Stage 2.1 escape-analysis follow-up

While extending MIR ownership analysis, I confirmed that call-argument escape marking is working, but the current HIR lowering for object-literal / heap-store-shaped values still flattens them into placeholder nodes instead of a stable composite shape. That means precise "stored into heap object" escape tracking is still blocked by frontend/HIR normalization, not by MIR alone.

Plan follow-up: schedule the HIR normalization work before claiming full heap-store classification coverage in Stage 2.1, or explicitly narrow the stage note to call/return/capture escapes until the frontend shape is stabilized.

## 2026-04-12 — Stage 2.1 HIR normalization resolved

The frontend now lowers object-literal properties into a dedicated `ObjectProperty` HIR node and lowers property keys as literals instead of identifiers. MIR ownership analysis now sees object literal values as escape flows without treating property names as bindings.

Suggested plan/status follow-up: update the Stage 2.1 status note to reflect the stabilized composite shape and keep the remaining escape-analysis work focused on other coverage gaps.

## 2026-04-12 — Stage 2.1 escape-analysis broadened

Added targeted MIR ownership tests for array-element and member-assignment heap-store flows so Stage 2.1 coverage now extends beyond call/return/object-literal cases.

Suggested follow-up: keep extending the analyzer with any remaining nested store or closure-shape edge cases before marking the stage complete.

## 2026-04-12 — Stage 3.2 package-host-fit preparation

`kali_npm` now keys install-time host-fit validation off the project `compilerOptions.apiSurface`, so `node`-targeted projects can accept Node-only builtins while the default standalone context still rejects them with `E6005`.

Suggested follow-up:
- keep the Stage 3.2 plan/status notes aligned with this package-host-fit split
- continue wiring the remaining CLI/runtime `--api node` paths so this install-time allowance becomes part of a real Node compatibility command context
- no spec change was needed for this increment
