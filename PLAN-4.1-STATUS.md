# Stage 4.1 Status Update

**Date:** 2026-04-17  
**Status:** ✅ Package-audit availability promotion synced; browser-bundle chunk discovery broadened

## Summary

Stage 4.1's package-audit work is now reflected consistently across the CLI, package semantics, maturity matrix, and README. The command remains schema-v1 envelope-only JSON, but it is now documented as the Phase 4 context-free registry-analysis/security-audit command instead of lingering in later-compatibility wording. The source-graph CLI surface has also gained explicit `--compat` plumbing for the shared compatibility-feature vocabulary so `compat.features` requests are parsed and rejected through the canonical availability gate instead of being silently ignored. Separately, browser-bundle chunk discovery now recognizes simple statically-resolvable `import(...)` string-concatenation targets in addition to direct string literals, which helps the emitted chunk graph stay aligned with the already-linked module set.

## Evidence

- `kali package-audit lodash` succeeds on the default command path ✅
- `kali package-audit --output json lodash` emits the schema-v1 envelope with `payload: null` ✅
- `kali package-audit --pretty lodash` remains invalid without `--output json` ✅
- `check --compat eval` now rejects through the canonical E5006 availability path, and inherited `compat.features = ["eval"]` hits the same gate ✅
- browser-bundle chunk discovery now follows simple statically-resolvable `import(...)` concatenations during artifact emission ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- The `package-audit` CLI path no longer advertises `--preview` in help output, while the compatibility shim remains hidden
- The package-audit availability row in `specs/19-feature-maturity.md` now opens in Phase 4
- The CLI and package semantics docs now read the command as the Phase 4 context-free registry-analysis/security-audit workflow
- Browser-bundle chunk discovery now includes simple statically-resolved dynamic-import concatenations so the chunk graph can pick up more obviously linked targets without waiting for later runtime compatibility

## Next Step

Finish the remaining Stage 4.1 dynamic-compatibility work (`eval` / `Function()` execution and truly runtime-resolved non-literal `import()`) separately.
