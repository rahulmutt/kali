# Stage 3.3 Status Update

**Date:** 2026-04-17  
**Status:** 🟡 Package-audit preview plumbing landed and browser-bundle format scaffolding is now in place for the Phase-3 registry-analysis / packaging breadth track

## Summary

Stage 3.3 now has a concrete `package-audit --preview` path instead of an unconditional gating stub, and browser bundle output now includes both a deterministic `.js.map` companion and an explicit bundle-format selector. The command remains unavailable by default, but preview mode now parses registry-package targets, preserves the envelope-only JSON shape, and returns a deterministic summary string in both text and JSON output modes. The browser bundle path now supports the default ESM wrapper plus a CommonJS-flavored `--format cjs` variant. The later public availability row remains unchanged.

## Evidence

- `kali package-audit lodash` still fails on the normal availability gate when `--preview` is absent ✅
- `kali package-audit --preview lodash` now emits a deterministic preview summary ✅
- `kali package-audit --preview --output json lodash` emits a schema-v1 envelope with `payload: null` ✅
- browser bundle builds now emit `.js.map` companions and basic source-map JSON ✅
- browser bundle builds now also support `--format cjs` with a `.cjs` wrapper and `.cjs.map` source map ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- `package-audit` now has an opt-in preview implementation gate in the CLI parser
- The browser bundle artifact flow now emits a deterministic source-map companion for the generated JS glue and can switch between ESM and CJS wrappers with `--format`
- The preview path reuses the registry-target validation rules and emits a stable envelope-only JSON response
- Text-mode preview output is deterministic and matches the same preview scaffold summary used in JSON mode
- Stage-3.3 progress is visible without changing the canonical later-compatibility maturity row

## Next Step

Broaden the Phase-3 breadth work beyond the audit scaffold by tackling code-splitting/tree-shaking, the remaining package-corpus work, and the cross-module inference tasks.