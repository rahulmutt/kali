# Stage 5.3 — Programmable Policy & Algebraic Effects

**Phase:** 5 — Later Compatibility & Platform Expansion  
**Spec refs:** [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/13-embedding.md`](../../specs/13-embedding.md), [`specs/04-type-system.md`](../../specs/04-type-system.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [2.2 — Public Effect Reporting](../phase-2/02-public-effect-reporting.md) and [2.3 — Public Embedding Surface](../phase-2/03-public-embedding-surface.md)

## Goal

Introduce the spec's later programmable effect/policy features without breaking the earlier,
smaller contracts:

- host-registered sandbox predicates as an embedding-only narrowing layer
- any algebraic-effect surface as an explicit later language/runtime feature

This stage must preserve two guardrails:
1. project policy files remain declarative data
2. programmable narrowing must never widen declarative denies or maturity gates

## Workable Milestone

- Trusted embedding hosts can register deterministic narrowing predicates over the canonical
  `effects.*` / `resources.*` vocabulary.
- Predicate denials appear through the normal Kali diagnostic/runtime error surfaces.
- If algebraic effects are enabled, they integrate with the existing effect-report and lowering
  pipeline instead of creating a disconnected second effect system.

## Progress

- `kali_sandbox` now exposes a canonical host-predicate context plus a deterministic registration
  registry for embedding-side narrowing predicates, and the public embedding crate re-exports that
  context so embedding callers can inspect the same capability/subject payload that sandbox
  enforcement uses. The canonical context now also carries host-specific detail for the threaded
  budget path (`resources.maxThreads` / `activeThreads`), and the embedding tests pin that detail
  through the re-exported context and through a direct `ThreadSpawn` narrowing regression.
- Declarative policy remains primary: the sandbox check helper applies the declarative decision
  first and only then evaluates host-registered predicates, with deterministic registration order and
  explicit rejection when the predicate registry is disabled.
- The dynamic-effect analyzer now treats both `new Proxy(...)` and `Proxy.revocable(...)` / `globalThis.Proxy.revocable(...)` as `proxy-traps`, so the proxy semantics slice stays aligned with the later reflective-compatibility boundary in both the sandbox model and the CLI smoke coverage.
- The public embedding layer now exposes an explicit host-predicate registration façade (`OperationContext`, `PredicateDecision`, and `EmbeddingCtx::register_sandbox_predicate` / `check_operation_with_policy`) so embedding callers can narrow declarative policy with the canonical capability vocabulary instead of wiring a second ad hoc policy language.
- Host-predicate registration now has an explicit availability gate in the embedding wrapper: callers can opt the façade out deterministically and receive the canonical `E5006` availability diagnostic instead of silently registering inert predicates when the feature is unavailable.

## Tasks

### 1. Host-registered predicate API

Add the embedding-side programmable-policy registration surface:

- registration API in the public embedding layer
- canonical operation-context payload
- narrowing-only semantics
- deterministic, synchronous evaluation rules
- rejection if the feature is not enabled or not available for the selected host/profile

### 2. Declarative-policy interaction rules

Keep the declarative sandbox contract primary:

- predicate checks run after the declarative allow/deny decision is known
- predicates may reject additional operations but never authorize a denied one
- CLI/config/schema docs stay explicit that `kali.policy.json` is still non-executable

### 3. Diagnostic and schema alignment

Ensure programmable-policy denials fit the existing machine contracts:

- canonical error codes and JSON envelopes
- optional host-specific context attached as extra detail, not a replacement schema
- stable naming for predicate-enabled capability families

### 4. Algebraic-effect design and integration

If Kali opens algebraic effects, introduce them as one coherent feature set:

- parser / AST / type-system additions
- lowering/runtime model for handlers
- interaction with built-in sandbox-relevant effects
- clear boundaries between user-defined handlers and the stable built-in reporting vocabulary

The built-in public effect-report surface must remain understandable even if algebraic effects add
language-level abstraction features.

### 5. Verification and soundness handoff

Because programmable policy and algebraic effects change semantic reasoning, produce the handoff
work needed for later proof/evidence expansion:

- new invariants for policy-narrowing correctness
- effect-system soundness notes for handlers
- explicit proof-boundary exclusions until mechanized work actually exists

### 6. Tests

- embedding tests for allowed vs denied predicate outcomes
- negative tests proving predicates cannot widen declarative policy
- JSON/diagnostic tests for predicate-triggered failures
- algebraic-effect parser/type/lowering/runtime tests if that surface opens
- regression tests ensuring ordinary declarative policy files remain data-only

## Out of Scope

- executable project-local sandbox policy code
- hidden `effects --sandbox` or `package-effects --sandbox` modes
- blanket user-defined effect kinds in stable public reports unless the owning schemas and
  maturity rows are updated explicitly

## Status

In progress.
