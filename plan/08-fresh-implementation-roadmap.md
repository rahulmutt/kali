# Fresh Implementation Roadmap

This guide answers a narrower question than [`../PLAN.md`](../PLAN.md):

> **If Kali had to be implemented from the current spec set today, what should the team build first, what should stay strictly sequential, and what should ship together as the smallest workable packets?**

It is intentionally:
- more actionable than the phase summaries
- less detailed than the individual stage files
- still subordinate to [`../PLAN.md`](../PLAN.md), the phase READMEs, and the owning specs

## Core rule

This roadmap is a **fresh-start execution overlay**, not a second maturity model.

It does **not** change:
- the normative contracts in [`../SPEC.md`](../SPEC.md)
- the phase/status owner in [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md)
- the stage ownership already defined in `plan/phase-*/`

It only answers: **what is the most sensible implementation order if we want to move from docs to a workable system with the least backtracking?**

## Recommended execution packets

Implement Kali in these packets, in order.

| Packet | Main stages | Why it is one packet | Must be demoable before moving on |
|---|---|---|---|
| F0 — Contract lock | pre-1.1 planning baseline | freeze shared vocabulary, maturity boundaries, schemas, and proof-boundary discipline before code claims drift | `SPEC.md`, `PLAN.md`, `specs/`, `plan/`, and `proofs/BOUNDARY.md` are internally aligned |
| F1 — CLI/workspace spine | 1.1 | everything else needs one buildable binary, shared diagnostics, config discovery, and canonical developer tasks | `kali --version`, `cargo build`, `cargo test --workspace` |
| F2 — Frontend acceptance | 1.2-1.5 | parser/type-checker correctness is the first semantic backbone; do not begin serious runtime/package work before this is stable | deterministic token/AST output and `kali check` on local inputs |
| F3 — End-to-end local pipeline | 1.6-1.8 | lowering, codegen, and runtime should produce one complete local-file compiler/runtime loop before product-surface breadth expands | `kali build`, `kali run`, and `kali test` on local fixtures |
| F4 — Phase-1 product surface | 1.9-1.13 | sandboxing, packages, build modes, workflow commands, diagnostics, and schemas should land as one coordinated product layer after runtime exists | `kali run --sandbox`, `kali install`, `kali build --bundle`, `kali init`, JSON output |
| F5 — Phase-1 evidence closure | 1.14 | Phase 1 is not ready until browser/package/determinism/proof-ready evidence lanes are passing | canonical Phase-1 CI / `mise` tasks pass |
| F6 — Semantic/public-surface stabilization | 2.1-2.5 | MIR, ownership, effects, embedding, Lean, and coverage all depend on semantics settling beyond MVP form | `kali effects`, stable embedding artifacts, `mise run lean-proofs`, `kali test --coverage` |
| F7 — Breadth with proof burden | 3.1-5.5 | later performance/compatibility work should open one support rung at a time with explicit evidence | each widened surface has its own demo and evidence lane |

## Packet-by-packet execution notes

### F0 — Contract lock

**Primary files**
- `SPEC.md`
- `PLAN.md`
- `specs/`
- `plan/`
- `proofs/BOUNDARY.md`
- `schemas/`

**Do first**
- resolve vocabulary conflicts between specs
- pin the canonical Phase-1 browser-targeted command set
- pin the static-vs-runtime sandbox split
- ensure proof-ready vs proof-backed wording is consistent
- verify the maturity matrix does not overclaim commands just because their shape is documented

**Do not do yet**
- code scaffolding that assumes unresolved CLI/schema details
- package/runtime implementation work
- proof-backed wording unless the published boundary actually allows it

### F1 — CLI/workspace spine

**Primary code areas**
- `crates/kali_cli`
- `crates/kali_common`
- `crates/kali_error`
- workspace config (`Cargo.toml`, `mise.toml`, CI/mise tasks)

**Must include**
- version/help entrypoint
- shared error-envelope scaffolding
- config-discovery skeleton
- canonical command-dispatch placeholders
- proof-ready repository hygiene

Current progress note:
- the top-level `kali --version` entrypoint is now pinned by a dedicated smoke test, so the F1 command spine stays regression-covered even as later product work widens the CLI surface
- the browser entrypoint smoke lane now also covers the Google Chrome stable wrapper spelling (`google-chrome-stable`) and the Microsoft Edge stable wrapper spellings (`msedge-stable`, `edge-stable`, and `microsoft-edge-stable`), plus the Firefox/Opera/Vivaldi stable spellings (`firefox-esr`, `opera-stable`, and `vivaldi-stable`), so the browser-launcher alias coverage stays aligned with another common stable-channel name family as the browser runtime surface widens

### F2 — Frontend acceptance

**Primary code areas**
- `crates/kali_lexer`
- `crates/kali_parser`
- `crates/kali_ast`
- `crates/kali_types`
- frontend fixtures/tests

**Strict order inside the packet**
1. lexer
2. parser + AST
3. name resolution
4. type checker

**Definition of readiness for F3**
- `kali check` runs on explicit local files
- import/name/type diagnostics are deterministic
- `.js` inputs participate under the bounded inference contract
- the checker-baseline lane exists

### F3 — End-to-end local pipeline

**Primary code areas**
- `crates/kali_hir`
- `crates/kali_lir`
- `crates/kali_codegen`
- `crates/kali_runtime`
- `crates/kali_api_deno`

**Strict order inside the packet**
1. typed lowering
2. deterministic WASM emission
3. Kali-hosted execution and test runner

**Guardrail**
Do not widen host-API, browser, or package claims here. This packet is about local-file closure, not ecosystem breadth.

Current progress note:
- the semver package bin now has an explicit default-standalone rejection regression, and the Node-path smoke now pins the exact help-path output shape plus the package-json require and guest-argument counting slices, so the F3 handoff keeps the Node-only CLI split honest while the later Node row remains gated
- recursive project discovery now has a nested-child regression for no-argument `check`, so the F2 discovery path stays bounded by nested `kali.json` roots instead of accidentally widening into child-project diagnostics
- the package corpus now also carries an explicit `@mariozechner/pi-coding-agent` published-bin-entrypoint probe, so the fresh-start breadth lane keeps the bootstrap-named CLI package separate from ordinary package-content coverage and honest about its Node-only execution path
- `kali install --dev semver` now has a configless-project regression that records the package in `devDependencies` and materializes the lockfile, so the F4b package/install lane now has corpus-style evidence for the documented dev-dependency path too
- `kali install --allow-scripts semver` now has a CLI smoke regression with empty lifecycle scripts, so the F4b package/install lane now also pins the explicit registry-target no-op allow-scripts path

### F4 — Phase-1 product surface

**Primary code areas**
- `crates/kali_sandbox`
- `crates/kali_npm`
- `crates/kali_embed`
- `crates/kali_fmt`
- `crates/kali_lint`
- `schemas/`
- `types/`
- browser/package/CLI fixtures

**Parallel window**
This is the main parallel zone, but only after F3 is complete.

Recommended substreams:
- **F4a sandbox/policy** — runtime enforcement and static policy validation
- **F4b package/install** — deterministic install, lock, cache, no hidden repair
- **F4c artifact modes** — executable build, browser bundle, base library artifact
- **F4d workflow commands** — `init`, `fmt`, `lint`
- **F4e machine contracts** — diagnostics, JSON envelopes, schemas, snapshot coverage

Current progress note:
- sandbox-agnostic `init`, `fmt`, and `lint` plus profile-agnostic `install` now reject `--sandbox` / `--api` through the canonical `E5508` path instead of Clap's generic unexpected-argument failure, so the workflow-command packet stays aligned with the documented CLI contract
- the browser-targeted `check` lane now also has explicit `--api browser` + `--sandbox` JSON-envelope coverage, keeping the F4a/F4e handoff honest in schema-v1 output as well as human output
- the public `effects` lane now also ignores a top-level `sandbox` config path in JSON output, so the source-graph analysis path stays decoupled from policy attachment the same way the package-analysis hardening does
- the query-only `Deno.permissions.query(...)` facade now stays effect-free in the same source-graph effect lane, so observation-only permission checks do not manufacture a synthetic effect or dynamic-reason row
- the public `effects` lane now also pins the `new Function(...)` compatibility path with the distinct `function-constructor` dynamic reason while still surfacing the shared `Eval` effect under both explicit and inherited `--compat eval` inputs, keeping the `eval`/`Function()` gate honest in effect-report coverage rather than only in runtime smoke
- the browser entrypoint smoke lane now also covers the Brave stable wrapper spellings (`brave-browser-stable` and `brave browser stable`), so the browser-launcher alias table stays pinned beyond the existing browser-entrypoint cases
- the browser entrypoint smoke lane now also covers the Chrome-for-Testing family (`chrome-for-testing`, `chromium-for-testing`, and `google-chrome-for-testing`) plus the additional privacy/community aliases (`librewolf`, `waterfox`, `zen-browser`, `zen browser`, `thorium-browser`, and `thorium browser`), keeping the CLI-level browser evidence aligned with the broader launcher alias table without changing the browser-runtime contract
- `kali install` now rejects explicit registry version/range selectors on identity-only targets with canonical `E5508`, so the fresh-start package-install lane keeps the schema-v1 identity-only contract honest at the CLI boundary
- the documented Node embedding-build subset now also has explicit and inherited `apiSurface=node` smoke coverage for `kali build --capi` and `kali build --component`, so the F4c artifact-mode packet keeps its later embedding rows tied to real artifact evidence instead of only the broader Node check/build lane
- package-audit now also keeps its schema-v1 envelope stable under inherited `compat.features = ["eval"]` in JSON output, reinforcing the F4e machine-contract boundary for the context-free registry-audit command
- package-effects now also has repeated-invocation determinism coverage in both native JSON and envelope modes, keeping the inherited-context registry-analysis sibling pinned across back-to-back runs. The shared determinism lane now also exercises the repeated-invocation envelopes for `effects` and `package-audit`, so the stable reporting surfaces stay pinned alongside the artifact outputs.
- package-audit now also has a repeated JSON-envelope regression under inherited browser analysis context, so the context-free registry-audit lane stays deterministically pinned even when the workspace inherits browser analysis settings.
- package-audit now also keeps quiet pretty JSON deterministic under inherited browser analysis, so the fresh-start registry-audit packet stays pinned across the pretty-envelope path instead of only the plain JSON-envelope path.
- the source-graph `effects` command now has matching pretty-JSON, envelope, repeated-invocation, and quiet-mode pretty JSON envelope smoke coverage, and it now also keeps that repeated pretty JSON envelope path stable under inherited browser analysis context, so the F4e machine-contract packet keeps pace with the registry-analysis hardening
- the source-graph `effects` command now also pins repeated JSON-envelope output under inherited browser analysis context, so the F6 handoff keeps the browser-context machine contract deterministic in the same way the registry-analysis lane already is
- the reusable effect-report payload now also deduplicates repeated logical roots in first-seen order before serialization, keeping future multi-root callers deterministic without changing the current single-root CLI demos
- package-effects and package-audit now also ignore a top-level `sandbox` config path in JSON output, so the fresh-start registry-analysis packet stays decoupled from runtime policy plumbing as documented
- package-audit now also keeps the inherited-browser-plus-top-level-sandbox JSON envelope path pinned, so the fresh-start registry-audit packet stays context-free even when both unrelated axes appear in the same manifest
- package-effects and package-audit now keep pretty JSON formatting stable when `--quiet` is combined with `--pretty`, so the registry-analysis packet preserves its machine-readable presentation contract across the supported control flags
- package-effects and package-audit now reject missing or multi-package targets with canonical `E5508` command-shape diagnostics, so the fresh-start registry-analysis packet stays single-package honest before the later availability gates even enter the picture
- `kali build --validate-ir` now runs structural HIR/MIR/LIR validation on demand, so the F4 build lane can catch inconsistent lowered trees before codegen without changing the resulting artifact shape
- the component build smoke lane now also exercises `--validate-ir` alongside the existing `--component` JSON artifact coverage, keeping the validation pass honest on the public embedding path as well as the browser bundle path
- the C-ABI build smoke lane now also exercises `--validate-ir` alongside the existing `--capi` JSON artifact coverage, keeping the validation pass honest on the exported-library path as well as the component and browser bundle paths
- the browser bundle smoke lane now also exercises `--validate-ir` together with `--bundle`, `--api browser`, and a valid sandbox policy, so the F4 artifact-mode packet covers the validation path instead of only the default browser bundle shape
- the public README command reference now also names `kali build --validate-ir`, keeping the user-facing build summary aligned with the documented structural-validation aid in `specs/12-cli.md`
- the base library build smoke lane now also exercises `--validate-ir`, so the library artifact path shares the same early IR-validation coverage as the component, C-ABI, and browser-bundle lanes
- raw-URL `install` now has a configless-project smoke regression that materializes `kali.lock` and the raw cache without scaffolding a placeholder `kali.json`, so the fresh-start package-install lane now covers the explicit URL branch of the configless install split too

**Shared coordination files**
- `specs/12-cli.md`
- `specs/15-errors.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

### F5 — Phase-1 evidence closure

**Primary areas**
- `tests/integration`
- `tests/conformance`
- `tests/browser-smoke`
- `tests/determinism`
- `tests/package-corpus`
- `proofs/`
- CI/mise task wiring

At minimum, Phase 1 should have:
- checker baselines
- integration coverage
- browser-targeted smoke tests
- determinism checks
- package-corpus checks for the claimed rung
- proof-ready manifest/CI discipline

### F6 — Semantic/public-surface stabilization

**Primary areas**
- `crates/kali_mir`
- `crates/kali_sandbox`
- `crates/kali_embed`
- `crates/kali_capi`
- `proofs/`
- coverage schemas/tests

This packet exists because:
- MIR ownership decides memory strategy
- effect reporting depends on canonical semantics
- embedding depends on stable export shape
- Lean depends on semantics worth proving
- coverage depends on a stable test/runtime path

### F7 — Breadth with proof burden

Treat all later work as **surface-by-surface widening**, not one giant compatibility wave.

Preferred order:
1. optimization depth
2. Node path
3. host-capability expansion
4. ecosystem breadth
5. dynamic compatibility
6. proof-boundary widening
7. deferred runtime/platform features

## Best implementation boundaries for a fresh push

```text
crates/
├── kali_cli
├── kali_common
├── kali_error
├── kali_lexer
├── kali_parser
├── kali_ast
├── kali_types
├── kali_hir
├── kali_lir
├── kali_codegen
├── kali_runtime
├── kali_sandbox
├── kali_npm
├── kali_embed
├── kali_capi
├── kali_optimize
├── kali_api_deno
├── kali_api_web
└── kali_api_node
```

Supporting top-level trees:

```text
tests/
├── integration/
├── conformance/
├── browser-smoke/
├── determinism/
└── package-corpus/

fixtures/
├── compiler/
├── runtime/
├── browser/
├── packages/
└── cli/

schemas/
proofs/
types/
bindings/
scripts/
```

## Fresh-start anti-patterns

Avoid these mistakes:
- starting package breadth before local execution works
- building browser runtime support before browser-targeted build support is solid
- landing JSON output before the envelope/schema owner is pinned
- treating proof-ready process work as proof-backed product proof
- widening Node/package claims because one narrow package happened to run
- doing large internal rewrites that leave no stable demo after the packet closes

## Suggested team split after F3

Once F3 is complete, a sensible parallel split is:

| Stream | Main packet/stages | Shared documents that must stay synchronized |
|---|---|---|
| Runtime + sandbox | F4a / 1.9 | `specs/09`, `specs/12`, `specs/15`, `specs/18`, `specs/19` |
| Packages + build artifacts | F4b-F4c / 1.10-1.11 | `specs/08`, `specs/11`, `specs/12`, `specs/14`, `specs/18`, `specs/19` |
| Workflow + machine contracts | F4d-F4e / 1.12-1.13 | `specs/12`, `specs/15`, `specs/18`, `specs/19` |
| Evidence hardening | F5 / 1.14 | `specs/16`, `specs/17`, `specs/19`, `proofs/BOUNDARY.md` |

## Maintenance rule

Update this guide when the **recommended fresh-start execution order** changes.

That usually means updating it together with:
- [`../PLAN.md`](../PLAN.md)
- [`README.md`](./README.md)
- [`05-delivery-increments.md`](./05-delivery-increments.md)
- [`07-roadmap-status-and-next-steps.md`](./07-roadmap-status-and-next-steps.md)
