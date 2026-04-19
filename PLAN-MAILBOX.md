## 2026-04-19 — Stage 3.1 direct array-literal specialization widening

I widened the Stage 3.1 specialization path one step further so direct array-literal call-site arguments now carry explicit array-shape signatures (`Value:array:len=...`) during MIR-aware specialization, which lets the optimizer split inline arrays with different lengths even when the callee only sees a tagged parameter.

Planned update:
- sync `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, and `TODO.md` so the Stage 3.1 specialization-depth notes explicitly mention the direct array-literal shape widening alongside the existing MIR-backed array-binding coverage
- keep the claim narrow: this is another specialization-depth widening within the existing optimizer model, not a new support-rung claim

## 2026-04-19 — Stage 3.1 array-layout specialization widening

I widened the Stage 3.1 specialization path so MIR-backed array bindings now preserve their layout fingerprints during call-site specialization. That lets the optimizer split otherwise identical hot paths when callers supply arrays with different element/length layouts, which is a concrete follow-up slice on top of the existing struct/closure/object-layout coverage.

Planned update:
- sync `plan/phase-3/01-optimization-and-specialization.md`, `PLAN.md`, and `TODO.md` so the Stage 3.1 specialization-depth notes explicitly mention the array-layout widening
- keep the claim narrow: this is still a specialization-depth widening within the existing optimizer model, not a new support-rung claim