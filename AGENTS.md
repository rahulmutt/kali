# AGENTS.md — Working Guide for AI Coding Agents

This repository uses a spec-driven workflow. Read the right document before changing code, and keep claims aligned with the canonical maturity and verification docs.

## 1) Operating principles

- Write clean, maintainable code.
- Reuse existing functions and types when possible; refactor before duplicating logic.
- Follow the project’s established architecture and naming.
- Keep outputs deterministic and machine-friendly.
- Do not overclaim feature availability, policy strength, or proof coverage.

## 2) Canonical documents and what they control

### `SPEC.md` — the normative spec set
Use this to answer: **What is Kali? What does each phase promise?**

`SPEC.md` owns:
- cross-spec terminology and normalization rules
- hard invariants and phase contracts
- feature maturity model
- subsystem ownership across specs 01–19
- verification boundary discipline

Read `SPEC.md` when:
- resolving terminology or cross-spec conflicts
- determining whether something is a hard invariant, phase contract, or phase-gated target
- checking whether a feature is actually supported
- introducing new shared terms, command families, or capability claims

### `PLAN.md` — the implementation playbook
Use this to answer: **How should the work be sequenced?**

`PLAN.md` owns:
- stage ordering and dependencies
- workable milestones
- parallel work guidance
- completion gates for each phase

Read `PLAN.md` when:
- starting a stage
- planning parallel implementation work
- checking stage dependencies or milestone scope

### Relationship between docs

```
prompts/bootstrap.md (raw goals)
        ↓
SPEC.md (normalization → phase contracts)
        ↓
PLAN.md (implementation stages → workable milestones)
```

Rules:
- `SPEC.md` defines the authoritative shape of the system.
- `PLAN.md` defines implementation order only.
- `specs/19-feature-maturity.md` is the source of truth for public availability.
- Early documentation of a command or flag does **not** mean it is publicly available.

## 3) Hard invariants — never violate

These must hold in every phase:
- **AOT-only compilation** — no language-level JIT path; compile TS/JS to WASM before execution.
- **Pure Rust implementation** — no embedded C/C++ dependencies.
- **No tracing/background GC** — only ownership/reference-counting strategies where permitted.
- **Sandbox-first honesty** — policy/enforcement claims must match actual mediation.
- **Deterministic machine contracts** — JSON output, artifact structure, and command behavior must stay explicit.

## 4) Default workflow for changes

1. Identify the owning spec chapter or stage document.
2. Check whether the change affects availability, verification, schemas, diagnostics, or CLI surface.
3. Update the spec set first when the change alters public behavior or claims.
4. Implement the code change.
5. Add or update tests and proof checks.
6. Re-read the affected docs for consistency before finishing.

If a request touches more than one surface, update the relevant docs together instead of leaving them inconsistent.

## 5) Testing and verification

### Lean / proof workflow
- The Lean environment is defined by `devenv.nix` and `devenv.yaml`.
- Use `mise run lean-proofs` for Lean proof builds.
- Do **not** invoke `nix shell` directly for proof builds.
- The proof project uses the toolchain pinned in `proofs/lean-toolchain`.
- When Lean files change, rerun `mise run lean-proofs` so Lake rebuilds the affected modules.

### Conformance expectations
Phase 1 evidence hardening (`stage 1.14`) requires:
- unit and integration coverage
- TypeScript / JavaScript checker baselines
- package-corpus checks under the linked-artifact model
- browser-targeted smoke tests for browser-targeted commands
- determinism checks for CLI outputs and generated artifacts
- proof-ready CI pipeline
- Rust unit tests in sibling `*tests.rs` files, not inline `#[cfg(test)]` modules

### Verification discipline
- A completed implementation stage does **not** automatically change maturity.
- Public availability always comes from `specs/19-feature-maturity.md`.
- Proof-backed status requires a non-empty published boundary in `proofs/BOUNDARY.md`.
- `proofs/BOUNDARY.md` is the source of truth for the proof-backed boundary, theorem/property inventory, covered-path list, proof-CI trigger scope, and canonical short verification summary.
- Do not duplicate or independently evolve the proof-boundary inventory in `plan/phase-4/02-formal-verification-depth.md`; keep that file as a reference back to `proofs/BOUNDARY.md`.
- Do not claim proof-backed status until the boundary is non-empty.

## 6) Schema, diagnostics, and CLI rules

### Diagnostic registry
Use the canonical registry in `specs/15-errors.md`:
- **E5xx** — type-checking diagnostics
- **E6xx** — package management diagnostics
- **E9xx** — sandbox / policy diagnostics

### JSON schemas
All machine-readable outputs must conform to the schema-v1 envelopes in `specs/18-schemas.md`.
Keep `--output json` behavior stable and explicit.

### CLI availability and canonical commands
- Distinguish **implemented internally** from **publicly available**.
- Do not document a command as available unless the maturity matrix says so.
- Canonical Phase-1 browser-targeted set: `kali check` and `kali build --bundle` when `apiSurface` is `browser`.
- Canonical Phase-1 static policy-validation surface: `check/build --sandbox` in executable, library, and bundle modes.
- Adding `--sandbox` does not make an otherwise-invalid command shape valid.

### CLI change packet
When changing CLI behavior or surface area, update at minimum:
1. `specs/12-cli.md`
2. `specs/15-errors.md`
3. `specs/18-schemas.md`
4. `specs/19-feature-maturity.md`
5. `README.md` if user-facing usage changes

## 7) Parallel development guidance

Parallel work is safe only when stage dependencies allow it.

- Within Phase 1, stages 1.9–1.14 can proceed in parallel after 1.8 is complete.
- Phase 3 work has its own ordering constraints; follow `PLAN.md` precisely.
- Before merging or handing off, validate against existing tests, spec updates, and maturity claims.

## 8) Commit / completion checklist

Before marking work complete, ensure:
- implementation matches the owning spec or stage document
- relevant tests pass
- `cargo test --workspace` passes when the change could affect Rust code paths
- docs and maturity rows match actual behavior
- no regressions to previously working commands
- any verification or schema claim changes were updated in the right files

Use descriptive commit messages that reference the stage when relevant, e.g. `feat: implement lexer [stage 1.2]`.

## 9) Repository structure

### Crates
- `kali` — main CLI binary and dispatch
- `core` — compiler core: lexer, parser, AST, type checking, lowering, codegen
- `runtime` — WASM runtime execution and sandbox enforcement
- `packages` — package management and dependency resolution
- `cli` — command definitions, help text, argument parsing

### File layout
- `specs/` — normative spec chapters (`01`–`19`)
- `plan/` — stage documents by phase and step
- `proofs/` — proof boundary and related verification artifacts
- root config files — keep dependencies and workflow definitions current

## 10) Tooling expectations

- Use `bash` for listing, searching, and command execution.
- Use `read` to inspect files instead of `cat` or `sed`.
- Use `edit` for precise replacements.
- Use `write` for new files or full rewrites.
- Keep tool and language dependencies in `mise.toml`.
- Prefer `mise` tasks for recurring workflows such as build, test, lint, format, and proof builds.

## 11) README guidance

Keep `README.md` user-facing and practical:
- explain how to build, run, and use the CLI
- include a compact command reference
- keep maturity mention brief
- avoid planning, bootstrap, or document-structure detail

## 12) Ambiguous claims to avoid

When writing docs or code comments, be precise about what is supported:
- “supports browser APIs” → specify typing, bundle/deploy path, runtime execution, or sandbox subset
- “supports npm packages” → specify shape, host fit, command maturity, or exact rung
- “supports non node-gyp packages” → shape-only, not full compatibility
- “sandbox policy passed in when running” → distinguish runtime enforcement, static validation, and effects reporting

## 13) If uncertain

When in doubt:
- read `SPEC.md` first
- then read the relevant chapter or stage doc
- use `specs/19-feature-maturity.md` for availability questions
- ask for clarification rather than guessing at unsupported behavior
