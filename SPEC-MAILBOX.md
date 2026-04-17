## 2026-04-17 — Stage 4.2 proof-boundary widening follow-up

I’m planning the next small Stage 4.2 proof-model widening pass: extend the current Lean boundary to cover assignment and try/catch in the closed core fragment, while still leaving bare throw, full memory safety, and the broader lowering-preservation target for later.

Suggested follow-up:
- update `proofs/BOUNDARY.md` and the verification chapter wording so the published boundary clearly names the widened fragment
- keep the canonical repository short summary unchanged unless the proof-backed claim itself changes

# SPEC Mailbox

## 2026-04-17 — Dynamic-import static-resolution follow-up

I’m planning a small implementation step for the Phase-4 dynamic-compatibility work: teach the resolver to statically evaluate simple `import(...)` source expressions (string literals, concatenation, and parenthesized forms) so statically known targets resolve through the existing import graph while unknown dynamic targets can emit the documented `E4008` diagnostic.

Suggested follow-up:
- if this lands, sync `specs/15-errors.md` so `E4008` is explicitly registered in the runtime error namespace
- keep this distinct from the later true runtime `eval` / interpreter-backed compatibility path; this step is only about source-level dynamic-import target resolution

## 2026-04-17 — Stage 4.2 HIR lowering-correctness widening

I widened the provisional HIR lowering model to cover assignment and try/catch alongside the existing `let1`, sequencing, and conditional bridge. The new HIR constructors lower directly to `EAssign` / `ETry`, and the lowering-preservation theorem now carries the assignment and try/catch step cases through to the core semantics.

Suggested follow-up:
- keep `proofs/BOUNDARY.md`, `specs/17-verification.md`, and the Stage 4.2 status note aligned with the widened HIR lowering subset
- continue widening the lowering/memory proof story only when the next claim is broad enough to justify another published boundary update

## 2026-04-17 — Browser-bundle dynamic-import concatenation follow-up

Browser-bundle chunk discovery now recognizes simple statically-resolved `import(...)` string-concatenation targets in addition to direct string literals. That broadens the build-time chunk graph a little further while still staying within the already-linked module set.

Suggested follow-up:
- decide whether `specs/08-wasm-codegen.md` / `specs/12-cli.md` / `specs/18-schemas.md` should explicitly mention concatenation-based static resolution, or whether this stays an implementation detail for now
- keep the later runtime `eval` / non-literal import compatibility wording separate from this build-time chunk-discovery refinement

## 2026-04-17 — Stage 3.3 browser-bundle source-map / format docs sync

Updated the browser-bundle contract docs to match the implemented wrapper/source-map output: `kali build --bundle` now documents the `--format esm|cjs` selector, the browser-bundle artifact tables now include the `source-map` companion, and the schema notes call out the `bundleFormat` field in JSON output.

Suggested follow-up:
- keep the browser-bundle artifact examples, CLI flag tables, and schema notes aligned if the wrapper/output shape changes again
- do not widen the Phase-1 browser-targeted availability row unless the support rung itself changes

## 2026-04-17 — Stage 3.3 browser-bundle source-map companion output

Implemented browser-bundle source-map companion emission so `kali build --bundle` now writes a deterministic `.js.map` file alongside the generated JS glue and adds a `sourceMappingURL` footer. The browser bundle JSON artifact list now also includes a `source-map` artifact entry.

Suggested follow-up:
- sync the browser bundle artifact tables and schema text in `specs/08-wasm-codegen.md`, `specs/12-cli.md`, `specs/18-schemas.md`, and any availability summaries that enumerate the bundle outputs
- keep the Phase-1/Phase-3 availability rows unchanged unless the bundle support rung itself changes

## 2026-04-17 — Stage 3.3 browser-bundle format targets

Implemented a browser-bundle output format selector so `kali build --bundle --format esm` remains the default ESM wrapper while `kali build --bundle --format cjs` now emits a CommonJS-flavored JS companion and `.cjs.map` source map. The JSON command output now includes a `bundleFormat` field for the emitted bundle result.

Suggested follow-up:
- sync the browser-bundle command/docs/schema wording in `specs/12-cli.md`, `specs/18-schemas.md`, `specs/08-wasm-codegen.md`, `specs/19-feature-maturity.md`, and `README.md` if we want the new format selector to be described publicly
- keep the browser-bundle availability row unchanged unless the support rung itself changes

## 2026-04-17 — Stage 3.3 package-audit preview plumbing

Implemented an opt-in `kali package-audit --preview` path in the CLI so the later registry-audit surface now has a concrete preview-only execution mode instead of an unconditional gate. The canonical availability matrix still keeps `package-audit` at **Later compatibility**, so the preview flag is intentionally not being advertised as public support.

Suggested follow-up:
- if we decide to document the preview flag, sync `specs/12-cli.md`, `specs/15-errors.md`, `specs/18-schemas.md`, and any summary text that would otherwise imply a broader release claim
- keep `specs/19-feature-maturity.md` unchanged until the command's actual public availability opens

## 2026-04-12 — Lean proof boundary wording

The repository now has a checked-in provisional Lean proof tree under `proofs/` and `proofs/BOUNDARY.md` is no longer a pure placeholder manifest. The current wording in `specs/17-verification.md` and the proof-related rows in `specs/19-feature-maturity.md` still talk about the shared **placeholder proof-boundary manifest** as the only pre-proof state.

Suggested follow-up:
- revise the verification chapter and maturity row wording to distinguish a **provisional non-empty proof boundary** from the original empty placeholder state
- keep the canonical short summary in `proofs/BOUNDARY.md` unchanged until the proofs become genuinely proof-backed
- ensure any future proof-backed wording still points at the published boundary rather than the staging history

## 2026-04-17 — Stage 4.1 package-audit availability follow-up

The CLI now allows `kali package-audit <pkg>` without the `--preview` gate and keeps `--preview` as a compatibility shim. The code path still emits the same schema-v1 envelope scaffold, but the public availability status in the spec set still says `package-audit` is later compatibility.

Suggested follow-up:
- sync `specs/12-cli.md`, `specs/14-packages.md`, `specs/19-feature-maturity.md`, and any README summary text if the public availability claim should move with the implementation
- keep the JSON envelope-only contract aligned with the eventual audit payload shape if/when the command is formally promoted

## 2026-04-17 — Stage 4.1 package-audit promotion sync

Promoted the public availability claim for `kali package-audit <pkg>` to Phase 4 compatibility in the spec and summary docs so the matrix, CLI chapter, package semantics chapter, and README all read the shipped command surface consistently.

Suggested follow-up:
- keep the envelope-only JSON contract wording aligned with `specs/18-schemas.md` if the audit payload ever grows beyond the current scaffold
- keep `--preview` hidden/compatibility-only if the shim remains accepted by the CLI

## 2026-04-17 — Proof-boundary wording promotion follow-up

The Lean tree now compiles cleanly and the current published proof boundary is mechanized for the widened closed fragment, but the manifest and summary docs still describe the repo as only proof-ready.

Suggested follow-up:
- promote `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, and the proof-related maturity wording in `specs/19-feature-maturity.md` to the proof-backed wording that matches the published boundary
- keep the Stage 4.2 plan text honest about the remaining memory-safety and lowering-correctness widening work

## 2026-04-17 — Stage 4.2 RC live-reference projection follow-up

I added a small proof-slice refinement to the Stage 4.2 memory model: well-formed RC snapshots now explicitly project each live reference back to ownership and allocation (`liveRefsAreOwnedAndAllocated`). The published boundary and verification chapter now mention that extra theorem alongside the existing no-dangling and release-liveness claims.

Suggested follow-up:
- keep the proof boundary and verification wording aligned if the RC model widens again
- treat the current claim as still narrower than the eventual full ownership / RC target

## 2026-04-17 — Stage 4.2 release-update preservation follow-up

I widened the current RC snapshot slice with a simple live-to-released transition theorem: `releaseRef` now records a released reference while preserving the remaining well-formed live set. The proof boundary, verification chapter, maturity summary, README verification bullet, and Stage 4.2 status note now need to mention that release-update preservation slice explicitly.

Suggested follow-up:
- keep the proof boundary and release-facing summaries aligned with the new release-update preservation theorem inventory
- continue widening the ownership / RC model toward a fuller decrement/freeing story when the next proof slice is ready
