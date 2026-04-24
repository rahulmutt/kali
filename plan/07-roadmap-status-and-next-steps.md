# Roadmap Status and Next Steps

This guide complements [`../PLAN.md`](../PLAN.md) by answering a narrower question:

> **Given the current spec set and workspace layout, what should implementation focus on next, what is safe to parallelize, and what should stay blocked until earlier demos are real?**

Use this file when the broad phase map is clear but day-to-day prioritization still needs a sharper answer.

## Core rule

- [`../PLAN.md`](../PLAN.md) and the phase/stage files own implementation order
- [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) owns public availability
- [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) owns the current proof-backed boundary
- this file is the **execution-priority overlay** for near-term work, not a second maturity matrix

## Current recommended execution order

Treat the roadmap as an active implementation queue with three levels of priority.

Current repository note:
- the phase checklists currently carried in this repository snapshot are all marked complete in their phase documents
- use this page as a prioritization overlay for future spec-led work, not as an open todo list for the closed stage packets

Recent hardening:
- a dedicated determinism smoke lane now runs through `scripts/check-determinism.sh` and a matching CI job, so the repeated-build evidence is exercised separately from the larger runtime smoke suite; the lane also pins the repeated-invocation envelopes for `effects`, `package-effects`, and `package-audit`, not just the build artifacts
- recursive project discovery now has an explicit nested-child regression for no-argument `check`, so the parent workspace walk stays bounded by nested `kali.json` roots instead of drifting into child-project diagnostics
- package-audit findings now use span position as a final deterministic tie-breaker after severity, code, message, notes, and suggestion, so same-message audit findings no longer depend on incidental registry iteration order
- package-effects now also rejects the full inherited-analysis flag family (`--api`, `--compat`, `--wasm-threads`, and `--sandbox`) with the canonical package-analysis-specific `E5508` message, keeping the registry-analysis command honest about its inherited-only context; the matching JSON-output regression is now pinned too so the schema-v1 envelope path stays covered alongside human output
- package-audit and package-effects JSON envelopes are now pinned under inherited browser context and quiet mode, reducing machine-contract drift across analysis presentation flags
- package-audit now also has repeated-invocation determinism regressions in both JSON and human output, plus repeated pretty JSON envelope coverage under quiet mode, so the envelope, summary, and findings order stay pinned across back-to-back runs instead of only under a single invocation
- package-effects now also has repeated-invocation determinism coverage in both native JSON and envelope modes, keeping the registry-analysis sibling command pinned across back-to-back runs as well
- package-effects now also keeps quiet-mode pretty output stable across repeated invocations, closing the last missing determinism lane for the sibling registry-analysis surface
- the source-graph `effects` command now has matching pretty-JSON, envelope, repeated-invocation, and quiet-mode pretty JSON envelope smoke coverage, and it now also keeps that repeated pretty JSON envelope path stable under inherited browser analysis context, so the public effect-report lane stays aligned with the registry-analysis hardening
- the source-graph `effects` command now also pins repeated JSON-envelope output under inherited browser analysis context, closing the last missing browser-context determinism lane for the public effect-report surface
- the reusable effect-report payload now also deduplicates repeated logical roots in first-seen order before serialization, keeping future multi-root callers deterministic without widening the current CLI shape
- package-effects and package-audit now also keep pretty JSON formatting stable when `--quiet` is combined with `--pretty`, so the presentation-control pair stays pinned without reintroducing human-output drift in the registry-analysis lane
- package-audit now also rejects the legacy `--preview` shim with the canonical `E5508` command-shape diagnostic before registry lookup, keeping the envelope-only contract free of an extra compatibility mode; the regression is covered in both plain and JSON output modes so the schema-v1 envelope path stays pinned too
- package-effects and package-audit now reject missing or multi-package targets with the canonical `E5508` registry-analysis command-shape diagnostic instead of Clap's generic required-argument failure, keeping the single-package contract honest at the parser boundary
- package-audit now also stays context-free under inherited `compat.features = ["eval"]` in JSON output, keeping the registry-analysis envelope honest even when project config carries a dynamic-compatibility hint
- package-effects and package-audit now also ignore a top-level `sandbox` config path in JSON output, so the registry-analysis lanes stay decoupled from policy-attachment plumbing even when the manifest names a sandbox file that would matter to runtime-enforced commands
- package-audit now also keeps the inherited-browser-plus-top-level-sandbox JSON envelope path pinned, so the context-free registry-audit lane stays honest even when both unrelated axes show up together in the same project config
- package-effects now also has an explicit combined browser-resolution + top-level-sandbox regression in JSON output, keeping the registry-analysis effect-report orthogonality pinned on the same test case instead of only across separate single-axis checks
- package-effects now also pins quiet-mode JSON output under inherited `eval` compatibility, so the analysis envelope remains stable even when dynamic-effect metadata comes from `kali.json`
- package-effects now preserves inherited compat features in its reported analysis context, so inherited `compat.features` like `eval` stay visible to the JSON payload instead of being silently dropped
- the public `effects` lane now carries the same inherited-context hardening for `compat.features = ["eval"]` and Node API-surface rejection, keeping the effect-report and analysis-gating paths aligned across explicit and inherited configs
- the public `effects` lane now also has explicit `--api browser` JSON-envelope coverage, matching the inherited-browser hardening so explicit and inherited browser analysis stay aligned in the smoke suite
- browser-targeted static policy-validation coverage now exercises inherited browser API surfaces for both `check` and `build --bundle`, including the sandbox-attached variants that keep the browser-targeted command set aligned with inherited config
- the browser-targeted `check` lane now also has explicit `--api browser` + `--sandbox` JSON-envelope coverage, so the static policy-validation surface keeps its machine contract pinned in both human and schema-v1 output modes
- the browser bundle smoke lane now also exercises the explicit `--api browser` path with `--bundle` + `--sandbox`, so the browser-targeted build evidence no longer relies only on inherited config for its executable and JSON snapshots
- the standalone browser runtime negative gates for `run --api browser` and `test --api browser` are now explicitly env-hardened in the CLI smoke suite, so the later browser rejection path stays pinned even if a browser harness helper leaks into the test environment
- the top-level CLI spine now has a dedicated `kali --version` smoke test, keeping the entrypoint contract pinned alongside the other command-shape regressions
- `kali build --validate-ir` now runs structural HIR/MIR/LIR validation on demand, so the build lane can fail early on inconsistent lowered trees without changing the emitted artifact shape
- the component build smoke lane now also exercises `--validate-ir` alongside the existing `--component` JSON artifact coverage, keeping the validation pass honest on the public embedding path as well as the browser bundle path
- the C-ABI build smoke lane now also exercises `--validate-ir` alongside the existing `--capi` JSON artifact coverage, keeping the validation pass honest on the exported-library path as well as the component and browser bundle paths
- the browser bundle smoke lane now also exercises `--validate-ir` together with `--bundle`, `--api browser`, and a valid sandbox policy, so the browser-targeted build evidence covers the validation path instead of only the default browser bundle shape
- the semver package bin now has an explicit default-standalone rejection regression, so the package-execution lane keeps the Node-only CLI split honest before the later Node row opens
- the package corpus now also carries an explicit `@mariozechner/pi-coding-agent` published-bin-entrypoint probe, so the bootstrap-named CLI package is tracked separately from ordinary package-content coverage and stays honest about the Node-only execution path
- `kali install --dev semver` now has a configless-project regression that records the package in `devDependencies` and materializes the lockfile without inventing a placeholder manifest, so the registry-install lane covers the documented dev-dependency path as well
- `kali install --allow-scripts semver` now has a CLI smoke regression with empty lifecycle scripts, so the explicit registry-target path keeps the documented no-op allow-scripts behavior pinned too
- the human diagnostic renderer now emits canonical error-doc links when `--verbose` is enabled, so the richer presentation path stays aligned with the E-code documentation contract instead of only the default concise output
- sandbox-agnostic `init` / `fmt` / `lint` and profile-agnostic `install` now reject `--sandbox` / `--api` through the canonical `E5508` path instead of Clap's generic unexpected-argument failure, keeping the workflow-command surface aligned with `specs/12-cli.md`
- the global `--pretty` gate and the `package-audit --pretty` path now report the canonical `E5508` CLI-usage diagnostic, keeping the shared command-shape code aligned with `specs/15-errors.md`
- the documented Node execution subset is now live for `run` / `test`, and the documented Node analysis/build subset is now live for `check` / `build` on the same source-compatible inputs while `effects` / `package-effects` stay gated explicitly; the package-corpus smoke lane exercises representative Node-package runtime cases across explicit and inherited `apiSurface=node` paths, and the semver-style `process.argv.slice(2)` probe now also strips the CLI `--` separator before constructing Node runtime argv so guest-argument passthrough matches the documented Node path
- the documented Node embedding-build subset now also has explicit and inherited `apiSurface=node` smoke coverage for `kali build --capi` and `kali build --component`, so the later Phase-3 embedding rows stay pinned to real artifact evidence instead of only the broader `check` / `build` lane
- `kali init` smoke coverage now includes the nested-child-project case under an ancestor manifest, so the documented subdirectory scaffold path is pinned as a real CLI demo instead of only as a spec claim
- the optimization/PGO lane now also covers browser-bundle JSON builds with attached profile data, so the `build --profile` provenance path is no longer exercised only by the executable/library smoke case
- Brave stable wrapper spellings (`brave-browser-stable` and `brave browser stable`) now resolve through the shared browser-launcher alias table too, keeping the browser host chooser aligned with another common stable-channel name family without changing the browser-runtime contract
- the browser entrypoint smoke lane now also covers the Google Chrome stable wrapper spelling (`google-chrome-stable`) and the Microsoft Edge stable wrapper spellings (`msedge-stable`, `edge-stable`, and `microsoft-edge-stable`), plus the Firefox/Opera/Vivaldi stable spellings (`firefox-esr`, `opera-stable`, and `vivaldi-stable`), so the alias table stays pinned across more common stable-channel names without changing the browser-runtime contract
- The Brave stable wrapper spellings (`brave-browser-stable` and `brave browser stable`) now have dedicated browser-entrypoint smoke coverage as well, so the alias table stays pinned beyond the existing browser-entrypoint cases.
- the browser/runtime compatibility layer now also exposes a deterministic `Atomics::is_lock_free` query helper alongside the existing shared-buffer bytewise operations, keeping the later threaded-profile baseline honest about the host's byte-atomic capability without widening the public maturity claims
- the host-registered predicate context now carries normalized file, network, process-env, timer, and eval detail keys in addition to the existing process/thread payloads, so programmable-policy callers can inspect the same deterministic capability metadata for path/URL/key-shaped operations without inventing a second policy vocabulary
- the public README command reference now also calls out `kali test --coverage`, keeping the user-facing summary aligned with the stable function-coverage contract already documented in `specs/12-cli.md`

### Priority A — finish the Phase-1 critical path

Do these first, in order, and keep them sequential unless a stage file explicitly says otherwise:

1. `1.1` workspace and CLI spine
2. `1.2` lexer
3. `1.3` parser and AST
4. `1.4` name resolution
5. `1.5` type checker
6. `1.6` HIR/LIR lowering
7. `1.7` WASM code generation
8. `1.8` runtime and execution

Why this is first:
- it is the shortest route to a believable local-file compiler/runtime loop
- it creates the semantic foundation every later package, sandbox, and browser claim depends on
- it keeps the repo in a continuously demoable state

### Priority B — use the post-1.8 parallel window carefully

Only after `1.8` is solid, open parallel work in:
- `1.9` sandbox and policy
- `1.10` package management
- `1.11` build artifacts
- `1.12` developer workflow
- `1.13` diagnostics and schemas
- `1.14` evidence hardening

These streams must stay synchronized on:
- [`../specs/12-cli.md`](../specs/12-cli.md)
- [`../specs/15-errors.md`](../specs/15-errors.md)
- [`../specs/18-schemas.md`](../specs/18-schemas.md)
- [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md)

### Priority C — only start post-MVP depth after evidence closure

After Phase 1 is complete, move in this order:
1. `2.1` MIR and ownership
2. `2.2`, `2.3`, `2.4`, `2.5`
3. `3.1`
4. `3.2` and `3.4` in parallel where safe
5. `3.3`
6. `4.1`
7. `4.2`
8. `5.x` one surface at a time

## Recommended next-read documents by planning question

| If the question is... | Read first | Then read |
|---|---|---|
| what should the team build immediately next? | [`08-fresh-implementation-roadmap.md`](./08-fresh-implementation-roadmap.md) | `../PLAN.md` + relevant phase README |
| what crates/directories should absorb that work? | [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md) | [`01-repository-layout.md`](./01-repository-layout.md) |
| can two streams proceed in parallel? | [`02-workstreams-and-handoffs.md`](./02-workstreams-and-handoffs.md) | `../PLAN.md` + relevant phase README |
| which stage owns a spec chapter or maturity row? | [`03-spec-to-stage-traceability.md`](./03-spec-to-stage-traceability.md) | exact stage file |
| what exact prerequisites and demo should one stage satisfy? | [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md) | exact stage file |
| what checklist should gate stage closure? | [`09-stage-acceptance-checklists.md`](./09-stage-acceptance-checklists.md) | [`00-planning-conventions.md`](./00-planning-conventions.md) + exact stage file |
| which cross-spec risks need extra hardening? | [`10-risk-register.md`](./10-risk-register.md) | relevant phase README + exact stage file |
| whether something is publicly shipped | [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) | owning spec chapter |
| what is proof-backed today | [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) | [`../specs/17-verification.md`](../specs/17-verification.md) |

## Near-term decision rules

Use these rules when picking work:

1. **Prefer the earliest missing demo over later breadth.**
   If `kali check` is not yet deterministic, do not prioritize Node or package breadth.

2. **Prefer closing command-shape owners before adding more commands.**
   If CLI/error/schema wording is drifting, fix that before opening more product surface.

3. **Prefer evidence closure before maturity promotion.**
   A working demo is not enough to widen public claims.

4. **Prefer explicit gates over partial emulation.**
   When a feature is phase-gated, add the honest gate path before adding half-support.

5. **Prefer stage packets that leave the repo usable.**
   Avoid large hidden internal rewrites that leave no stable demo behind.

## When to create a new planning document

Create a new plan doc instead of only editing an existing one when all of the following are true:

1. the work is larger than a hardening pass
2. it has its own dependency order or completion gate
3. it touches more than one subsystem or evidence lane
4. future contributors will benefit from a dedicated checklist

Otherwise, prefer updating the relevant phase README, stage file, or this prioritization guide.

## Maintenance rule

Keep this file compact and action-oriented.

When the plan is in a fully closed state, keep the prioritization notes short and point readers at the owning specs, maturity matrix, and evidence tracks instead of re-expanding the historical stage sequence.

- Do not duplicate spec contracts here
- Do not duplicate the proof-boundary inventory here
- Do not let this file become a second top-level plan
- Update it whenever the recommended near-term execution order changes
