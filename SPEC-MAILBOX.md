# SPEC Mailbox

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
