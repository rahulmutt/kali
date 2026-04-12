# Plan Mailbox — Stage 2.3 Public Embedding Surface Notes

## Observations and Required Plan Updates

### 1. Stage 2.3 public embedding artifacts are now implemented in-tree

The CLI now ships positive Phase-2 embedding coverage:
- `kali build --capi` emits a deterministic C ABI artifact plus header output.
- `kali build --component` emits a valid component artifact.
- The runtime smoke suite now exercises both flows positively instead of treating them as gated placeholders.

### 2. Plan progress update requested

The Phase-2 stage 2.3 progress note and completion marker in `PLAN.md` were updated to reflect the implemented embedding surface and the passing workspace test run.

**Date:** 2026-04-12  
**Status:** Processed — embedding artifact flows landed and plan progress was updated
