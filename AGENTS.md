# Developer Practices
- Write clean, maintainable code.
- Reuse existing functions if possible or refactor when needed.
- Follow best software engineering practices and methodologies.

# Development Methodology

## Phased Implementation
Kali follows a strict phased implementation strategy. Each phase must leave the project in a **workable state**:  
1. **Compiles** — `cargo build` succeeds with no warnings that would block merge
2. **Passes tests** — Existing test suite passes (`cargo test --workspace`)
3. **Provides end-user value** — At least one new capability is demonstrable via CLI
4. **Maintains invariants** — AOT-only, pure Rust, no tracing GC, sandbox-first, deterministic

**Critical path awareness**: Within Phase 1, stages 1.1–1.8 form the critical path that must complete before 1.9–1.14 begin. Parallel development (stages 1.9–1.14) requires coordination on shared schemas and diagnostics.

## Hard Invariants - Never Compromise
These invariants must hold across all phases. They may deepen but never bend:
- **AOT-only compilation** — No language-level JIT path; complete TS/JS → WASM before execution
- **Pure Rust implementation** — No embedded C/C++ implementation dependencies
- **No tracing/background GC** — Ownership/reference-counted strategies only where chapters permit
- **Sandbox-first honesty** — Policy/enforcement claims match what Kali can actually mediate
- **Deterministic machine contracts** — JSON output, artifact structures, command behavior stay explicit and tool-friendly

## Testing Strategy

### Conformance Suite
Phase 1 evidence hardening (stage 1.14) requires:
- Unit/integration coverage
- TypeScript/JavaScript checker baselines
- Package-corpus checks under the **linked-artifact model**
- **Phase-1 browser-targeted smoke tests** for browser-targeted command set
- Determinism checks for all CLI outputs and generated artifacts
- Proof-ready CI pipeline

### Evidence Discipline
- A stage completing implementation work **does not** automatically promote a feature's maturity level
- Public availability always reads from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)
- Phase-N maturity rows open only when backed by passing evidence tracks from `specs/16-testing.md`

## Schema Coordination

### Diagnostic Registry
All parallel streams must use the canonical diagnostic code registry:
- **E5xx**: Type-checking diagnostics
- **E6xx**: Package management diagnostics  
- **E9xx**: Sandbox/policy diagnostics
- Refer to [`specs/15-errors.md`](./specs/15-errors.md) before assigning new error codes

### JSON Schema Envelopes
All machine-readable outputs must conform to schema-v1 envelopes defined in [`specs/18-schemas.md`](./specs/18-schemas.md):
- `--output json` flag semantics must be consistent
- Stable codes and canonical structures across all parallel streams
- Changes require cross-review for breaking impact

### Schema Update Packet for CLI Changes
When updating CLI commands, update this minimum set:
1. [`specs/12-cli.md`](./specs/12-cli.md) — Command definitions, flags, arity
2. [`specs/15-errors.md`](./specs/15-errors.md) — Diagnostic codes for new errors
3. [`specs/18-schemas.md`](./specs/18-schemas.md) — JSON envelope shapes
4. [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) — Availability maturity row
5. [`README.md`](./README.md) — Overview bullets/examples if CLI surface changes

## CLI Development Discipline

### Command Availability Tracking
Distinguish between:
- **Internally implemented** — Feature exists in code but is not publicly advertised
- **Publicly available** — Feature can be documented as available per the maturity matrix

**Rule**: Commands documented as "defined early" before shipping must **not** claim phase availability in `specs/19-feature-maturity.md`. Always read actual availability from the maturity matrix, not from the mere fact that a command/flag/artifact has a documented shape.

### Canonical Commands
Ensure all CLI streams reference the shared canonical command definitions:
- **Phase-1 browser-targeted command set** — `kali check` + `kali build --bundle` when `apiSurface` is `browser`
- **Phase-1 static policy-validation surface** — `check/build --sandbox` in executable, library, and bundle modes
- Attaching `--sandbox` never rescues an otherwise-invalid command shape

## Parallel Development Coordination

### When Parallel Development Is Safe
Parallel streams may proceed once the critical path (1.1–1.8) completes:
- Static validation (1.9), package management (1.10), build artifacts (1.11), developer workflow (1.12), diagnostics (1.13), evidence (1.14)

### Required Coordination Before Committing
1. **Validate against existing tests**: Run `cargo test --workspace` to ensure no regressions
2. **Update canonical definitions**: Ensure CLI flags, error codes, and schema types match specification
3. **Check maturity tracking**: Document commands with correct phase availability status
4. **Cross-feature impact**: Verify changes don't break features from other parallel streams

### Parallel Development Patterns
- Within Phase 1: 1.9–1.14 can proceed in parallel after 1.8 (runtime execution) completes
- Phase 3: 3.1 (optimization) must complete before 3.2 (Node compatibility) or 3.3 (ecosystem breadth) can begin in parallel
- Always respect stage dependency statements in PLAN.md documents

## Verification System Discipline

### Proof-Ready State
- **Phase 1 proof-ready**: `proofs/BOUNDARY.md` and proof-CI trigger policy must exist from Stage 1.1
- **Proof-backed state**: Requires a non-empty published boundary in `proofs/BOUNDARY.md`
- Repository **may not** claim proof-backed status until boundary is non-empty
- Changing verification claims requires coordinating: `specs/17-verification.md`, `proofs/BOUNDARY.md`, `specs/16-testing.md`, `specs/19-feature-maturity.md`

### Verification Update Packet for Claims Changes
When changing verification claims, update: `
1. [`specs/17-verification.md`](./specs/17-verification.md) — Verification program structure
2. [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) — Current proof boundary and state
3. [`specs/16-testing.md`](./specs/16-testing.md) — Evidence lane requirements
4. [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) — Proof status maturity rows
5. [`README.md`](./README.md) — Root-level status summaries

## Spec Editing Workflow

### When Translating Bootstrap Requests
1. **Classify the ask** with Bootstrap Triage Rule (hard invariant, phase contract, or phase-gated breadth target)
2. **Find the owning chapter** and update phase availability in `specs/19-feature-maturity.md`
3. **Normalize shared vocabulary** first when introducing cross-spec terminology
4. **Prefer one canonical rule** over repetition across chapters
5. **Check release-claim surfaces**: README summaries, phase summaries, examples must read availability from maturity matrix

### Avoiding Overclaims
Be especially careful with these ambiguous phrases - clarify which specific surface/rung/context is being claimed:
- "supports browser APIs" — clarify: ambient typing, bundle/deploy path, execution, or sandbox subset?
- "supports npm packages" — clarify: shape, host fit, command maturity, or exact rung (installable/ checkable/buildable/executable/deployable)?
- "supports non node-gyp packages" — clarify: this is shape-only, not full compatibility
- "sandbox policy passed in when running" — clarify: `run/test --sandbox` enforce at runtime, `check/build --sandbox` validate statically, `effects` report only

## Git and Version Discipline

### Commit Standards
- Each stage document should have a clear workable milestone achievable at that commit
- Use descriptive commit messages that reference stage numbers (e.g., "feat: implement lexer [stage 1.2]")
- No stage may break existing functionality or make previously-functional CLI commands regress
- Parallel streams must pass `cargo test --workspace` before committing work

### Stage Gate Discipline
Before marking a stage complete, verify:
- All tasks in the stage document are completed
- Workable milestone is demonstrable via CLI
- All tests pass
- Specification is updated (19-feature-maturity.md, schemas, diagnostics)
- Documentation reflects the actual availability state

## Code Organization

### Crate Boundaries
- `kali` — Main CLI binary and command dispatch
- `core` — Compiler core (lexer, parser, AST, type checking, lowering, codegen)
- `runtime` — WASM runtime execution and sandbox enforcement
- `packages` — Package management and dependency resolution
- `cli` — CLI command definitions, help text, argument parsing

### File Organization
- All spec chapter files in `specs/` with explicit ownership (numbered 01-19)
- Plan stage files in `plan/<phase>/<step>.md` following the same ownership
- Verification artifacts in `proofs/` with clear boundary management
- Configuration files (mise.toml, Cargo.toml, etc.) updated with all dependencies

## Tools

### Mise Dependency Management
- Add all tool and language dependencies to `mise.toml`
- Keep dependencies aligned with the phases they enable
- Document any version requirements in stage documents

## Reading the Spec Set

To find authority on different questions:
- **Whether something is supported yet?** → SPEC.md → 19-feature-maturity → owning chapter
- **How a supported thing works?** → Owning chapter first, then SPEC.md for cross-spec terminology
- **What proof coverage is claimed today?** → `proofs/BOUNDARY.md` (not roadmap prose)
- **Which document is normative?** → SPEC.md and chapter in specs/ (not BOOTSTRAP.md)
