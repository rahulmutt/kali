# Phase 19 — Optimization and Performance Evidence

## Goal

Turn optimization and PGO work into deterministic, evidence-backed performance claims without changing observable semantics.

## Owning specs

- `specs/07-specialization.md`
- `specs/08-wasm-codegen.md`
- `specs/16-testing.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 19.1 Optimization inventory upkeep

- Keep a concise current-evidence inventory of real `fast`, `release`, and `release-advanced` behavior.
- Update the inventory only when tests prove a mode's behavior.
- Preserve deterministic artifacts and schema-v1 output contracts.

### 19.2 Specialization depth

- Improve layout, representation, and call-shape specialization in claim-aligned slices.
- Treat `--max-specializations` as an exact upper bound.
- Preserve fallback paths and JavaScript-visible semantics.
- Current progress: `Object.freeze`-wrapped literal object shapes now also fold through the release and release-advanced optimizer paths for `Object.keys(...)`, `Object.values(...)`, `Object.entries(...)`, and `Reflect.ownKeys(...)`, keeping the frozen-object specialization path aligned with the existing literal-object folding tests. Those optimizer folds now also recognize bracketed alias spellings such as `globalThis["Object"]["keys"]`, `globalThis["Object"]["values"]`, `globalThis["Object"]["entries"]`, and `globalThis["Object"]["hasOwn"]` once the access chain normalizes back to the same helper family, and the Reflect helper family now also carries bracketed `globalThis["Reflect"]["ownKeys"]` / `globalThis["Reflect"].ownKeys` optimizer regression coverage so the same canonicalization path stays pinned for `Reflect.ownKeys` too. `Object.hasOwn(...)` now also keeps folding when its object operand is a frozen `Object.fromEntries(...)` shape in the optimizer regression suite, extending the frozen-object evidence one step beyond the plain literal-object case. Codegen unit coverage now also pins the bracketed `globalThis["Object"]["hasOwn"]` helper spelling over a frozen `Object.fromEntries(...)` operand, and now also covers the mixed-bracket `globalThis.Object["hasOwn"]` / `globalThis["Object"].hasOwn` spellings so the same canonical helper path stays covered outside the optimizer regression suite.

### 19.3 PGO input hardening

- Keep `--profile` as deterministic build-only additive input.
- Reject malformed, unknown-field, version-mismatched, and nondeterministic profile data.
- Do not create a fourth build-mode vocabulary.

### 19.4 Benchmark promotion

- Use version-pinned workload fixtures with hash validation.
- Promote performance wording only with workload, build mode, baseline, repeatability, and artifact-determinism evidence.
- Keep package anecdotes separate from package-compatibility claims.

## Exit gate

- Optimizations preserve conformance, sandbox, schema, and proof-boundary tests.
- PGO output is deterministic.
- Public performance claims are workload-specific and evidence-backed.
