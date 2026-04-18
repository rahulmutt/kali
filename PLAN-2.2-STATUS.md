# Stage 2.2 Status Update

**Date:** 2026-04-18  
**Status:** ✅ Public effect-reporting stage complete

## Summary

Stage 2.2 is now fully in place. The public effect-report surface is live in both of its explicit halves: `kali effects` and `kali package-effects` emit stable native-JSON effect reports, and `check/build --sandbox` now extends the Phase-1 policy-validation path with inferred-effect-vs-policy rejection.

## Evidence

- `kali effects` emits a native JSON effect report ✅
- `kali package-effects` emits a native JSON package effect report ✅
- `check/build --sandbox` reject inferred effects that exceed the active policy ✅
- Positive CLI/runtime smoke coverage replaces the old unavailable-command gates ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- `kali effects <file>` now analyses a source graph and emits a stable schema-v1 effect report
- `kali package-effects <pkg>` now analyses a single registry package and emits a stable schema-v1 effect report
- `kali check --sandbox` / `kali build --sandbox` now compare inferred effects against the active declarative policy
- The Phase-1 effect-surface gating tests were updated from negative unavailability checks to positive coverage

## Current Limits

- The public effect-report surface is intentionally limited to the documented reporting half plus the sandbox-policy comparison half
- Later registry-analysis breadth and dynamic compatibility remain phase-gated
- The command families remain distinct from `kali package-audit`, which is a later context-free registry-analysis workflow

## Next Step

Continue with the remaining phase workstreams and keep the Phase 2 effect-report surface wording aligned with the published maturity matrix.
