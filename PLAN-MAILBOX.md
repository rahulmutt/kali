## 2026-04-19 — Stage 3.1 array-layout specialization widening

I widened the Stage 3.1 specialization path so MIR-backed array bindings now preserve their layout fingerprints during call-site specialization. That lets the optimizer split otherwise identical hot paths when callers supply arrays with different element/length layouts, which is a concrete follow-up slice on top of the existing struct/closure/object-layout coverage.

Planned update:
- sync `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, and `TODO.md` so the Stage 3.1 specialization-depth notes explicitly mention the array-layout widening
- keep the claim narrow: this is still a specialization-depth widening within the existing optimizer model, not a new support-rung claim
