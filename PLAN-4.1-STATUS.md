# Stage 4.1 Status Update

**Date:** 2026-04-17  
**Status:** ✅ Package-audit availability promotion synced

## Summary

Stage 4.1's package-audit work is now reflected consistently across the CLI, package semantics, maturity matrix, and README. The command remains schema-v1 envelope-only JSON, but it is now documented as the Phase 4 context-free registry-analysis/security-audit command instead of lingering in later-compatibility wording.

## Evidence

- `kali package-audit lodash` succeeds on the default command path ✅
- `kali package-audit --output json lodash` emits the schema-v1 envelope with `payload: null` ✅
- `kali package-audit --pretty lodash` remains invalid without `--output json` ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- The `package-audit` CLI path no longer advertises `--preview` in help output, while the compatibility shim remains hidden
- The package-audit availability row in `specs/19-feature-maturity.md` now opens in Phase 4
- The CLI and package semantics docs now read the command as the Phase 4 context-free registry-analysis/security-audit workflow

## Next Step

Finish the remaining Stage 4.1 dynamic-compatibility work (`eval` / `Function()` and non-literal `import()`) separately.
