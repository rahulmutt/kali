# Phase 6 Conformance Dashboard

This dashboard is a deterministic implementation snapshot for the language features called out in the phase-6 roadmap.

- Normative availability still comes from [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
- Evidence links point to tests or canonical diagnostic gates.
- Rows are grouped by the current maturity bucket so the supported / gated / rejected split stays obvious.

## Supported today

| Feature | Status | Evidence |
|---|---|---|
| Latest published ECMA-262 lexical grammar (tokenization) | Phase 1 MVP | `crates/kali_parser/src/tests.rs` |
| Current-edition non-Annex-B semantics for features Kali marks as supported in a given command/profile | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_runtime/src/tests.rs` |
| Static ESM `import` / `export` | Phase 1 MVP | `crates/kali_parser/src/tests.rs`, `crates/kali_types/src/tests.rs` |
| First-class JavaScript compilation with bounded inference | Phase 1 MVP | `crates/kali_types/src/tests.rs` |
| Budgeted local/intra-module constraint solving inside the shared bounded inference contract | Phase 1 MVP | `crates/kali_types/src/tests.rs` |
| Basic arithmetic precedence and array length semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Basic try/catch exception handling | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| CommonJS module lowering | Phase 1 MVP | `crates/kali_codegen/src/tests.rs` |
| `require("literal")` | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_codegen/src/tests.rs` |
| Literal-string `import()` over the linked graph, including directory-index targets | Phase 3 target | `crates/kali_types/src/tests.rs`, `crates/kali_cli/src/build_tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` |

## Gated for later phases

| Feature | Status | Evidence |
|---|---|---|
| Open-ended or unstable cross-module/public-API constraint solving | Phase 3 target | Canonical `E5506`/annotation-required boundary in `specs/04-type-system.md` and `specs/19-feature-maturity.md` |
| Non-literal `import(expr)` | Rejected by default | Canonical non-literal dynamic-loading gate (`E5506`) |
| `eval` | Phase 4 compatibility | Canonical `--compat eval` gate and `E5506` until enabled |
| `Function()` constructor | Phase 4 compatibility | Canonical `--compat eval` gate and `E5506` until enabled |
| `Proxy` | Later compatibility | Canonical availability gate (`E5506`) |
| `WeakMap` / `WeakSet` | Later compatibility | Canonical availability gate (`E5506`) |
| `FinalizationRegistry` | Later compatibility | Canonical availability gate (`E5506`) |

## Rejected by default

| Feature | Status | Evidence |
|---|---|---|
| Stage-3+/draft TC39 proposals beyond the latest published ECMA-262 edition | Rejected by default | Canonical rejected-by-default row in `specs/19-feature-maturity.md` |
| Dynamic `require()` | Rejected by default | Canonical rejected-by-default row and `E5506`/unsupported-dynamic-loading diagnostics |
| Non-literal `import(expr)` | Rejected by default | Canonical non-literal dynamic-loading gate (`E5506`) |

## Reading note

This dashboard is intentionally smaller than the full maturity matrix. It only tracks the ECMA/TS slice that the Phase 6 roadmap cares about today, and it stays sorted by maturity bucket so new rows can be compared without guessing at the intended status.
