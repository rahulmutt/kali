# RALPH

- Iteration: 1 of 1
- Prompt file: /workspace/prompts/iterate-plan.md
- Updated: 2026-03-31T08:49:10.072Z

## Last emitted message

```text
All 22 links resolve. Here's a summary of what was created:

---

## What was generated

### `PLAN.md` (top-level)
A one-page navigator with a table for each phase mapping stage numbers → plan file → workable milestone. Closes with cross-cutting rules (hard invariants, evidence-before-promotion, proof-ready-from-day-one).

### `plan/phase-1/` — 14 stages (Core Compiler MVP)

Each stage leaves the project in a usable state, ordered from simplest to most complex:

| # | Stage | Workable milestone |
|---|---|---|
| 1.1 | Workspace scaffold | `kali --version` works; CI runs; `proofs/BOUNDARY.md` exists |
| 1.2 | Lexer | Tokenises TS/JS; all `E1xxx` lex errors |
| 1.3 | Parser & AST | Full ECMA-262 + TypeScript + JSX grammar; snapshot tests |
| 1.4 | Name resolution | `kali check` reports unresolved names |
| 1.5 | Type checker | `kali check` reports type errors under bounded inference |
| 1.6 | HIR & LIR lowering | Full pipeline exists; LIR dump for inspection |
| 1.7 | WASM codegen | `kali build` produces valid, validated `.wasm` files |
| 1.8 | Runtime & execution | `kali run` and `kali test` work in the Default standalone context |
| 1.9 | Sandbox & policy | `--sandbox` enforced at runtime; policy schema validated statically |
| 1.10 | Package management | `kali install` with deterministic lock file; npm/JSR/raw-URL |
| 1.11 | Build artifacts | `--bundle` (browser) and `--lib` (base library) artifact modes |
| 1.12 | Developer workflow | `kali init`, `kali fmt`, `kali lint` |
| 1.13 | Diagnostics & schemas | `--output json` schema-v1 envelopes; stable error codes |
| 1.14 | Evidence hardening | test262 conformance, checker baselines, package corpus, browser smoke tests, determinism |

### `plan/phase-2/` — 3 stages (Ownership, Effects & Embedding)
MIR + escape analysis → public effect reporting (`kali effects`, `kali package-effects`, inferred-effect-vs-policy on `check/build --sandbox`) → stable public embedding surface (WIT `--lib`, `--capi`, `--component`).

### `plan/phase-3/` — 3 stages (Specialisation & Ecosystem)
Monomorphisation + optimisation passes → `--api node` + Node built-ins → package corpus expansion + browser packaging improvements + cross-module constraint solving.

### `plan/phase-4/` — 2 stages (Dynamic Compatibility & Deep Verification)
`eval`/`Function()` AOT-interpreter gate + non-literal `import()` + `kali package-audit` → Lean 4 proof tree, type soundness + memory safety proofs, proof-backed repository status.
```
