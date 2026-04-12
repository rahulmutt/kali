# Stage 1.13 Status Update

**Date:** 2026-04-12  
**Status:** ✅ Diagnostics and schemas stage complete

## Summary

Stage 1.13 is now complete. The CLI's machine-readable output surface is stable across the shipped Phase-1 commands, the canonical error presentation is in place, and the schema-v1 documents for envelopes, diagnostics, manifests, locks, policies, artifacts, and results are committed in the repository.

## Evidence

- `cargo test -p kali_cli --test runtime_smoke` ✅
- `cargo test --workspace` ✅
- `schemas/` documents parse and match the current CLI contracts ✅

## Notable Deliverables

- `--output json` now emits a single schema-v1 envelope across the shipped command surface
- Program stdout/stderr are captured into the envelope for execution commands
- Command/result schema documents exist for the shipped surfaces plus reserved later-phase shapes
- Diagnostic presentation follows the concise, AI-friendly formatting contract

## Next Step

Move on to Stage 1.14 — Evidence Hardening.
