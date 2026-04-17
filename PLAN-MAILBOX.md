## 2026-04-17 — Stage 4.2 finite-trace lowering bridge follow-up

The HIR lowering-correctness slice now includes a finite-trace preservation theorem (`KaliIR.LoweringCorrectness.lower_preserves_steps`) on top of the existing single-step bridge, so the proof boundary and the Stage 4.2 progress note should describe both levels of preservation explicitly.

Suggested follow-up:
- keep `proofs/BOUNDARY.md`, the Stage 4.2 status note, and the TODO progress tracker aligned with the broader HIR lowering-correctness inventory
- continue widening the proof model in small slices so the semantic-preservation story remains honest as it grows

## 2026-04-17 — Stage 3.2 Node URL host-import projection

Extended the pure-Rust Node helper layer with `NodeUrl::parse` / `NodeUrl::resolve`, then threaded those helpers through the runtime linker as `kali:node` URL host imports and added runtime smoke coverage for the new projection path.

Suggested follow-up:
- keep the Stage 3.2 status note, TODO progress text, and any release-facing Node-compatibility summaries aligned with the actual linked helper surface
- continue broadening the Node runtime helper surface one concrete workflow at a time so the partial compatibility story stays honest

## 2026-04-17 — Stage 4.2 proof-boundary widening follow-up

I’m planning the next small Stage 4.2 proof-model widening pass: extend the current Lean boundary to cover assignment and try/catch in the closed core fragment, while still leaving bare throw, full memory safety, and the broader lowering-preservation target for later.

Suggested follow-up:
- update `proofs/BOUNDARY.md` and the verification chapter wording so the published boundary clearly names the widened fragment
- keep the canonical repository short summary unchanged unless the proof-backed claim itself changes

## 2026-04-17 — Stage 4.2 HIR lowering-correctness widening landed

The HIR lowering bridge now covers assignment and try/catch as well as the existing `let1`, sequencing, and conditional cases. The lowering-correctness theorem now explicitly carries those new step cases through to the core `EAssign` / `ETry` semantics.

Suggested follow-up:
- keep the Stage 4.2 status note and `proofs/BOUNDARY.md` in sync with the widened HIR subset
- continue the memory-safety / lowering-correctness work in small proof-backed slices so the boundary stays honest as it grows

## 2026-04-17 — Stage 3.2 NodeBuffer round-tripping follow-up

Extended `kali_api_node::NodeBuffer` with deterministic hex encode/decode alongside the existing base64 helpers, then updated the Stage 3.2 status note and plan progress text to reflect the broader binary-data workflow slice.

Suggested follow-up:
- keep broadening the Node helper surface one concrete workflow at a time so the runtime linker and pure-Rust helper layer stay aligned
- if the buffer surface widens again, update the Stage 3.2 progress note and any release-facing summaries together so the status text keeps matching the actual helper slice

## 2026-04-17 — Stage 4.2 lowering-correctness bridge added

Added `proofs/KaliIR/LoweringCorrectness.lean` as a small-step preservation bridge for the current HIR subset. The proof boundary now names the extra Lean file and theorem inventory explicitly, but it still stops short of the later full HIR → LIR semantic-preservation target.

Suggested follow-up:
- keep the Stage 4.2 status note and `proofs/BOUNDARY.md` aligned with the currently claimed HIR subset
- widen the proof story only when the semantics target is broad enough to merit a stronger lowering-correctness claim

## 2026-04-17 — Stage 4.2 closed-fragment widening landed

The Lean proof fragment in `proofs/KaliCore/Soundness.lean` now covers the widened closed core calculus: literals, variables, closed functions, application, sequencing, and conditionals. That keeps the proof honest while still leaving assignment/exceptions, memory safety, and lowering-correctness for later Stage 4.2 work.

Suggested follow-up:
- keep `proofs/BOUNDARY.md` and the Stage 4.2 status note aligned with the widened theorem inventory
- continue pushing the boundary toward ownership/memory safety and lowering-correctness without reintroducing the proof-overclaim

# PLAN Mailbox

## 2026-04-17 — Stage 4.2 proof-backed boundary status sync

Synced the Stage 4.2 status note and TODO progress tracker to the proof-backed boundary wording now published in `proofs/BOUNDARY.md`. The status text now quotes the canonical repository summary verbatim, and the progress tracker records the widened closed fragment plus the current theorem inventory.

Suggested follow-up:
- keep the remaining Stage 4.2 widening work focused on ownership/memory safety and lowering correctness
- if the proof boundary widens again, update the status note and any release-facing summaries together so the canonical summary stays in lockstep


## 2026-04-17 — Stage 4.2 RC snapshot safety follow-up

The proof tree now models ownership / RC state with `RcSnapshot`, live-reference tracking, and released-reference liveness checks instead of the earlier reference-free placeholder story. `noDanglingReference` and `releasedNotLive` are both mechanized, and the boundary/status docs now describe the smaller RC snapshot slice explicitly.

Suggested follow-up:
- keep widening the memory model toward the full ownership / reference-counting target when the next proof step is ready
- if the boundary widens again, sync `proofs/BOUNDARY.md`, the Stage 4.2 status note, and any release-facing summaries together

## 2026-04-17 — Dynamic-import static-resolution follow-up

I’m planning to implement the remaining Phase 4.1 dynamic-import slice by statically evaluating simple `import(...)` target expressions during name resolution. That should let linked targets resolve normally while unknown expressions fail with the canonical `E4008` path instead of silently passing through the resolver.

Suggested follow-up:
- update the Stage 4.1 status note once the resolver can distinguish statically known vs. unresolved dynamic-import targets
- keep this separate from the later eval/runtime-interpreter work and from the already-implemented browser-bundle chunk-discovery refinement

## 2026-04-17 — Stage 4.1 eval compat plumbing

The CLI/runtime pipeline now accepts `--compat eval` / inherited `compat.features = ["eval"]` instead of rejecting the feature outright, and the build path rewrites simple statically-resolvable eval strings before lowering so the runtime can exercise the Phase 4 compat gate without a second compilation tier.

Suggested follow-up:
- keep expanding the evaluator only within the documented Phase 4 `eval` compatibility path; `Function()` still needs its own execution handling
- keep the rewrite pass narrow so it stays a precompiled-stub bridge, not a hidden general-purpose interpreter

## 2026-04-17 — Node runtime stdout/stderr projection follow-up

Added `kali:node` host-import coverage for `process_stdout_write` and `process_stderr_write` in the runtime linker, so the Node compatibility path can now bridge guest writes into the captured runtime output stream as well as the existing argv/env helpers.

Suggested follow-up:
- keep broadening the remaining Phase-3 Node helper surface in the same runtime-linker projection pattern
- if future Node imports need richer stdout/stderr semantics, decide whether they should append raw text or newline-terminated records before widening the contract

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

## 2026-04-17 — Stage 3.2 Node buffer helper progress

Added base64 round-tripping to `NodeBuffer` so the helper layer now covers a more realistic slice of Node-style binary data handling instead of only raw bytes and UTF-8 text.

Suggested follow-up:
- keep broadening the helper surface one concrete Node-style data path at a time
- if later runtime wiring needs more Buffer-specific behavior, decide whether the extra shape belongs in the pure helper layer or in the wasmtime import projection

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

## 2026-04-17 — Stage 4.1 package-audit availability progress

`kali package-audit` now runs from the default command path instead of requiring `--preview`, while preserving the existing scaffold output and keeping the preview flag as a compatibility shim.

Suggested follow-up:
- keep the Stage 4.1 plan note in sync with any eventual spec/maturity promotion for package-audit
- finish the remaining Stage 4.1 dynamic-compatibility work (`eval` / `Function()` and non-literal `import()`) separately

## 2026-04-17 — Stage 4.1 package-audit promotion sync

Recorded the package-audit availability promotion in the spec/maturity docs so the plan notes no longer trail the implemented default command path.

Suggested follow-up:
- keep the Stage 4.1 status file and phase-4 completion gate aligned with any future audit-payload expansion
- leave the preview shim as an internal compatibility detail unless we decide to document it explicitly

## 2026-04-17 — Stage 4.1 dynamic-import static-fragment folding progress

The resolver now folds const-bound dynamic-import fragments during name resolution, so `import(root + name)`-style targets can resolve when the fragments are statically known. That keeps the compile-time linked-graph check in place for known targets while leaving the true runtime-mediated graph lookup as the remaining Phase 4.1 follow-up.

Suggested follow-up:
- keep the TODO entry open for the runtime-mediated fallback path
- if we later widen the supported constant-expression forms, keep the compile-time diagnostic split between statically known linked targets and genuinely runtime-resolved specifiers

## 2026-04-17 — Stage 4.2 soundness proof scope narrowed to a closed fragment

I got `proofs/KaliCore/Soundness.lean` compiling again, but only by narrowing the current proof model to literals, variables, and closed functions. That lets the progress/preservation shape stay mechanised while leaving application/control-flow soundness, context-shifting substitution, and the rest of the full Stage 4.2 proof backlog for a later pass.

Suggested follow-up:
- update the Stage 4.2 status note / boundary text to match the actually modelled closed fragment
- decide whether the remaining application/flow proofs should live in a future proof-boundary widening step or under a separate, more explicit subfragment

## 2026-04-17 — Stage 4.1 parenthesized dynamic-import follow-up

The dynamic-import static-resolution slice now also covers parenthesized static fragments in both the resolver tests and the browser-bundle smoke path. That keeps the already-documented Phase 4.1 behavior aligned with the parser's ability to see grouped constant expressions instead of treating them as unresolved dynamic targets.

Suggested follow-up:
- keep the Stage 4.1 status note in sync if we widen the static-resolution fragment further
- do not broaden the Phase 4.1 compatibility wording unless we intentionally add a new dynamic-import shape beyond the already-linked graph contract

## 2026-04-17 — Proof-boundary wording promotion follow-up

The proof tree compiles cleanly and the currently published boundary is mechanized for the widened closed fragment, so the plan/status docs should stop describing the repository as only proof-ready.

Suggested follow-up:
- update the Stage 4.2 status note and any summary files that quote the verification state so they reflect the proof-backed boundary wording
- keep the Stage 4.2 definition of done open until the remaining ownership/memory-safety and lowering-correctness targets are actually mechanized

## 2026-04-17 — Stage 3.2 path-relative helper broadening

Added `NodePath::relative` / `relative_path` to the pure-Rust Node compatibility helper layer and wired the runtime linker to expose `path.relative` to guest modules. The Stage 3.2 status note now reflects the lexical relative-path helper, the linked host-import projection, and the still-partial broader Node coverage.

Suggested follow-up:
- keep broadening the Node helper surface one concrete API at a time so the runtime linker and pure helper layer stay aligned
- if later Node compatibility work needs more path semantics, widen the documented helper surface together with the runtime-linker projection and the stage status note

## 2026-04-17 — Stage 4.2 RC disjointness follow-up

I widened the Stage 4.2 RC snapshot safety slice by adding `releasedNotLiveRef`, which makes the live/reference and released/reference sets explicitly disjoint for well-formed snapshots. `proofs/BOUNDARY.md`, the Stage 4.2 status note, and the TODO progress tracker now mention that extra theorem alongside the existing `noDanglingReference` and `releasedNotLive` story.

Suggested follow-up:
- keep widening the memory model toward the full ownership / RC target when the next proof step is ready
- if the boundary widens again, sync `proofs/BOUNDARY.md`, the Stage 4.2 status note, and any release-facing summaries together

## 2026-04-17 — Stage 4.2 RC live-reference projection widening

The Stage 4.2 memory-safety slice now includes `liveRefsAreOwnedAndAllocated`, which projects well-formed live references back to ownership and allocation in the RC snapshot model. `proofs/BOUNDARY.md`, the Stage 4.2 status note, and the verification chapter now mention that extra theorem explicitly.

Suggested follow-up:
- keep widening the memory model toward the full ownership / RC target in similarly small proof-backed slices
- if the boundary widens again, sync the status note and boundary manifest together so the canonical proof summary stays honest

## 2026-04-17 — Stage 2.1 closure-capture layout follow-up

`kali_mir` now records the concrete closure capture list in each function binding's layout descriptor, so the MIR ownership model keeps the environment shape explicit in addition to the existing `captured_by` tracking. The Stage 2.1 plan note was updated to reflect that the closure-layout slice is now fully covered.

Suggested follow-up:
- keep the MIR ownership and layout summaries aligned whenever the closure environment model changes again
- if later stages start projecting MIR capture lists into LIR or codegen metadata, update the stage note and any release-facing summaries together so the closure-shape story stays consistent
