# Stage 3.3 Status Update

**Date:** 2026-04-17  
**Status:** 🟡 Package-audit preview plumbing landed, browser bundle output now emits deterministic chunk artifacts for literal dynamic import boundaries, and a curated Phase-3 package corpus covers representative browser, utility, and Node-runner package classes alongside the wrapper/source-map output for the registry-analysis / packaging breadth track

## Summary

Stage 3.3 now has a concrete `package-audit --preview` path instead of an unconditional gating stub, browser bundle output includes a deterministic source-map companion plus deterministic chunk artifacts for literal dynamic import boundaries, and the curated package corpus now exercises representative browser, utility, and Node-runner package classes against the documented support ladder. The command remains unavailable by default, but preview mode now parses registry-package targets, preserves the envelope-only JSON shape, and returns a deterministic summary string in both text and JSON output modes. The browser bundle path now supports the default ESM wrapper plus a CommonJS-flavored `--format cjs` variant, and the JSON output now records the selected wrapper format explicitly. The later public availability row remains unchanged.

## Evidence

- `kali package-audit lodash` still fails on the normal availability gate when `--preview` is absent ✅
- `kali package-audit --preview lodash` now emits a deterministic preview summary ✅
- `kali package-audit --preview --output json lodash` emits a schema-v1 envelope with `payload: null` ✅
- browser bundle builds now emit deterministic source-map companions and basic source-map JSON ✅
- browser bundle builds now also support `--format cjs` with a `.cjs` wrapper and `.cjs.map` source map ✅
- browser bundle builds now emit deterministic chunk artifacts for literal dynamic import boundaries ✅
- package corpus evidence now covers browser-safe packages (`react`, `preact`, `vue`), utility packages (`ramda`, `rxjs`, `immer`, `uuid`, `typescript`, `esbuild`), and Node-runner packages (`vitest`, `jest`) at their documented rungs ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- `package-audit` now has an opt-in preview implementation gate in the CLI parser
- The browser bundle artifact flow now emits a deterministic source-map companion for the generated JS glue, can switch between ESM and CJS wrappers with `--format`, and emits deterministic chunk artifacts for literal dynamic import boundaries
- The browser bundle JSON envelope records the selected wrapper format and includes the source-map artifact entry plus any emitted chunk artifacts
- The preview path reuses the registry-target validation rules and emits a stable envelope-only JSON response
- Text-mode preview output is deterministic and matches the same preview scaffold summary used in JSON mode
- The curated package corpus exercises the documented support ladder for browser-safe, utility, and Node-runner package classes without weakening the later availability row
- Stage-3.3 progress is visible without changing the canonical later-compatibility maturity row

## Next Step

Broaden the Phase-3 breadth work beyond the audit scaffold by deepening tree-shaking and the cross-module inference tasks, then widen the package corpus further where additional real-world package shapes are still missing.
