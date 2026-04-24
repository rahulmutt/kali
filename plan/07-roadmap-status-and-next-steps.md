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
- the MIR crate now also publishes an ownership-sensitive representation fingerprint helper alongside the canonical layout fingerprint helper, so later MIR-aware lowering and analysis passes can reuse one deterministic layout/ownership summary instead of reconstructing it ad hoc
- a dedicated determinism smoke lane now runs through `scripts/check-determinism.sh` and a matching CI job, so the repeated-build evidence is exercised separately from the larger runtime smoke suite; the lane also pins the repeated-invocation envelopes for `effects`, `package-effects`, and `package-audit`, including the quiet-mode pretty JSON variants that keep the browser-context effects path and the package-analysis reporting lanes deterministic, not just the build artifacts
- package-audit now also carries a repeated JSON-envelope regression under inherited browser analysis context, so the context-free registry-audit lane stays deterministically pinned even when the browser-context inheritance path is present in the workspace config
- the public effect-report surface now also honors the documented Node analysis context, so Node-aware source-graph and package-analysis effect reports can stay aligned with the broader Node compatibility row instead of stopping at `check` / `build` / `run` / `test` only
- the public effect-report surface now also marks computed bracketed host access on Deno compatibility paths with the canonical `computed-host-access` dynamic reason, so bracketed `Deno[...]` / `globalThis["Deno"]` access remains visible in the same machine-readable effect lane instead of collapsing into a plain host call; the same reason is now pinned across both the source-graph `effects` command and the package-analysis `package-effects` wrapper
- the query-only `Deno.permissions.query(...)` facade stays effect-free in that same lane, so observation-only permission checks do not create a synthetic effect or dynamic-reason row
- computed bracketed `Deno.permissions.query(...)` access now keeps the same effect-free payload while still surfacing the shared `computed-host-access` dynamic reason, so bracketed permissions queries stay visible without inventing a new effect kind
- the Deno permission facade now also rejects the recognized-but-unavailable `Deno.permissions.request()` / `revoke()` members through the canonical `E5506` path, including the `globalThis.Deno.permissions.*` forms and the statically-known string-literal bracketed equivalents, so the observable permission surface stays query-only instead of silently exposing interactive escalation members in the current phase.
- recursive project discovery now has an explicit nested-child regression for no-argument `check`, so the parent workspace walk stays bounded by nested `kali.json` roots instead of drifting into child-project diagnostics
- package-audit findings now use span position as a final deterministic tie-breaker after severity, code, message, notes, and suggestion, so same-message audit findings no longer depend on incidental registry iteration order
- package-audit now also rejects prerelease-only registries with the canonical no-stable-version `E6001` path, so the stable-release selection rule stays explicit instead of silently drifting onto a prerelease-only target
- package-effects now also rejects the full inherited-analysis flag family (`--api`, `--compat`, `--wasm-threads`, and `--sandbox`) with the canonical package-analysis-specific `E5508` message, keeping the registry-analysis command honest about its inherited-only context; the matching JSON-output regression is now pinned too so the schema-v1 envelope path stays covered alongside human output, and the sibling missing-dependency-state regression now reports the canonical `E6004` package-management diagnostic when the package has not been materialized in the current project
- package-audit and package-effects JSON envelopes are now pinned under inherited browser context and quiet mode, reducing machine-contract drift across analysis presentation flags
- package-audit now also has repeated-invocation determinism regressions in both JSON and human output, plus repeated pretty JSON envelope coverage under quiet mode, so the envelope, summary, and findings order stay pinned across back-to-back runs instead of only under a single invocation
- package-audit now also keeps quiet pretty JSON deterministic under inherited browser analysis, so the browser-context registry-audit lane stays pinned across the pretty-envelope path as well as the plain JSON-envelope path.
- package-audit now also carries a quiet pretty JSON regression with a top-level sandbox config path, keeping the browser-context registry-audit lane deterministic even when unrelated policy plumbing is present in the manifest.
- package-audit now also keeps inherited Node analysis context aligned with a top-level sandbox config in JSON output, so the registry-audit lane stays orthogonal across the documented node-and-policy axes instead of only proving each one in isolation
- package-effects now also has repeated-invocation determinism coverage in both native JSON and envelope modes, keeping the registry-analysis sibling command pinned across back-to-back runs as well
- package-effects now also keeps quiet-mode pretty output stable across repeated invocations, closing the last missing determinism lane for the sibling registry-analysis surface
- the source-graph `effects` command now has matching pretty-JSON, envelope, repeated-invocation, and quiet-mode pretty JSON envelope smoke coverage, and it now also keeps that repeated pretty JSON envelope path stable under inherited browser analysis context, so the public effect-report lane stays aligned with the registry-analysis hardening
- the source-graph `effects` command now also pins repeated JSON-envelope output under inherited browser analysis context, closing the last missing browser-context determinism lane for the public effect-report surface
- the reusable effect-report payload now also deduplicates repeated logical roots in first-seen order before serialization, keeping future multi-root callers deterministic without widening the current CLI shape
- the reusable effect-report payload now also keeps grouped effect kinds and per-kind locations in deterministic sort order, so multi-effect reports stay diff-friendly even when the same analysis graph is observed through mixed call sites
- the reusable effect-report payload now also trims and deduplicates semantic analysis axes before serialization, so incidental whitespace in inherited runtime-profile or compat-feature config does not perturb the machine contract
- package-effects and package-audit now also keep pretty JSON formatting stable when `--quiet` is combined with `--pretty`, so the presentation-control pair stays pinned without reintroducing human-output drift in the registry-analysis lane
- package-audit now also rejects the legacy `--preview` shim with the canonical `E5508` command-shape diagnostic before registry lookup, keeping the envelope-only contract free of an extra compatibility mode; the regression is covered in both plain and JSON output modes so the schema-v1 envelope path stays pinned too
- package-effects and package-audit now reject missing or multi-package targets with the canonical `E5508` registry-analysis command-shape diagnostic instead of Clap's generic required-argument failure, keeping the single-package contract honest at the parser boundary
- package-audit now also stays context-free under inherited `compat.features = ["eval"]` in JSON output, keeping the registry-analysis envelope honest even when project config carries a dynamic-compatibility hint
- package-effects and package-audit now also ignore a top-level `sandbox` config path in JSON output, so the registry-analysis lanes stay decoupled from policy-attachment plumbing even when the manifest names a sandbox file that would matter to runtime-enforced commands
- package-audit now also keeps the inherited-browser-plus-top-level-sandbox JSON envelope path pinned, so the context-free registry-audit lane stays honest even when both unrelated axes show up together in the same project config
- package-effects now also has an explicit combined browser-resolution + inherited-eval + top-level-sandbox regression in JSON output, keeping the registry-analysis effect-report orthogonality pinned when the browser and compat axes overlap instead of only across separate single-axis checks
- package-effects now also pins quiet-mode JSON output under inherited `eval` compatibility, so the analysis envelope remains stable even when dynamic-effect metadata comes from `kali.json`
- package-effects now preserves inherited compat features in its reported analysis context, so inherited `compat.features` like `eval` stay visible to the JSON payload instead of being silently dropped
- package-effects now also keeps inherited Node analysis context aligned with a top-level sandbox config in JSON output, so the registry-analysis effect-report lane stays orthogonal across the documented node-and-policy axes instead of only proving each one in isolation
- the public `effects` lane now carries the same inherited-context hardening for `compat.features = ["eval"]` and Node API-surface rejection, keeping the effect-report and analysis-gating paths aligned across explicit and inherited configs
- the public `effects` lane now also has explicit `--api browser` JSON-envelope coverage, matching the inherited-browser hardening so explicit and inherited browser analysis stay aligned in the smoke suite
- the public `effects` lane now also ignores a top-level `sandbox` config path in JSON output for the explicit browser-analysis path, so source-graph analysis stays decoupled from policy-attachment plumbing just like the registry-analysis hardening already does
- the public `effects` lane now also keeps inherited browser analysis context aligned with a top-level sandbox path in JSON output, so the browser-resolution and sandbox-attachment axes stay pinned together in the source-graph effect-report lane as well
- the public `effects` lane now pins the `new Function(...)` compatibility path with the distinct `function-constructor` dynamic reason while still surfacing the shared `Eval` effect under both explicit and inherited `--compat eval` inputs, keeping the `eval`/`Function()` gate honest in effect-report coverage rather than only in runtime smoke
- browser-targeted static policy-validation coverage now exercises inherited browser API surfaces for both `check` and `build --bundle`, including the sandbox-attached variants that keep the browser-targeted command set aligned with inherited config
- the browser-targeted `check` lane now also has explicit `--api browser` + `--sandbox` JSON-envelope coverage, so the static policy-validation surface keeps its machine contract pinned in both human and schema-v1 output modes
- the browser bundle smoke lane now also exercises the explicit `--api browser` path with `--bundle` + `--sandbox`, so the browser-targeted build evidence no longer relies only on inherited config for its executable and JSON snapshots
- the standalone browser runtime negative gates for `run --api browser` and `test --api browser` are now explicitly env-hardened in the CLI smoke suite, so the later browser rejection path stays pinned even if a browser harness helper leaks into the test environment
- those same browser runtime rejections now also stay pinned under inherited `compilerOptions.apiSurface = browser` plus attached `--sandbox` policies for `run` and `test`, so the explicit/inherited config split remains honest even when sandbox plumbing is present
- the top-level CLI spine now has a dedicated `kali --version` smoke test, keeping the entrypoint contract pinned alongside the other command-shape regressions
- `kali build --validate-ir` now runs structural HIR/MIR/LIR validation on demand, so the build lane can fail early on inconsistent lowered trees without changing the emitted artifact shape
- the component build smoke lane now also exercises `--validate-ir` alongside the existing `--component` JSON artifact coverage, keeping the validation pass honest on the public embedding path as well as the browser bundle path
- the C-ABI build smoke lane now also exercises `--validate-ir` alongside the existing `--capi` JSON artifact coverage, keeping the validation pass honest on the exported-library path as well as the component and browser bundle paths
- the browser bundle smoke lane now also exercises `--validate-ir` together with `--bundle`, `--api browser`, and a valid sandbox policy, so the browser-targeted build evidence covers the validation path instead of only the default browser bundle shape
- the inherited-browser browser-bundle smoke lane now also combines `--bundle`, `--validate-ir`, and `--sandbox`, so the build-evidence overlay now pins the inherited config path with the same validation/policy pairing as the explicit browser-flag path
- the semver package bin now has an explicit default-standalone rejection regression, and the Node-path smoke now pins the exact help-path output shape plus the package-json require and guest-argument counting slices, so the package-execution lane keeps the Node-only CLI split honest before the later Node row opens
- the package corpus now also carries an explicit `@mariozechner/pi-coding-agent` published-bin-entrypoint probe, so the bootstrap-named CLI package is tracked separately from ordinary package-content coverage and stays honest about the Node-only execution path
- the package corpus now also includes a `date-fns`-style named-export plus subpath-export-map regression on the default standalone surface, so the curated pure JS/TS package set keeps broadening across representative real-world module shapes instead of only the already-covered semver/zod patterns
- that same package-corpus probe now also asserts Node argv passthrough on the package bin entrypoint, so the published-CLI evidence lane covers both execution and argument delivery on the documented Node surface
- `kali install` now rejects explicit registry version/range selectors on identity-only targets with canonical `E5508`, keeping the install path aligned with the schema-v1 package-identity contract instead of silently accepting versioned CLI input
- `kali install --dev semver` now has a configless-project regression that records the package in `devDependencies` and materializes the lockfile without inventing a placeholder manifest, so the registry-install lane covers the documented dev-dependency path as well
- `kali install --allow-scripts semver` now has a CLI smoke regression with empty lifecycle scripts, so the explicit registry-target path keeps the documented no-op allow-scripts behavior pinned too
- `kali install` on an empty workspace now has a CLI JSON-envelope regression that proves the command stays a clean no-op without scaffolding `kali.json` or `kali.lock`, keeping the configless install path honest at the operator boundary
- the human diagnostic renderer now emits canonical error-doc links when `--verbose` is enabled, so the richer presentation path stays aligned with the E-code documentation contract instead of only the default concise output
- sandbox-agnostic `init` / `fmt` / `lint` and profile-agnostic `install` now reject `--sandbox` / `--api` through the canonical `E5508` path instead of Clap's generic unexpected-argument failure, keeping the workflow-command surface aligned with `specs/12-cli.md`
- the workflow-owner commands now pin both `--api` and `--sandbox` rejection in the smoke suite, so `init` / `fmt` / `lint` keep their early-phase command-shape guardrails honest instead of relying on parser-only failures
- the global `--pretty` gate and the `package-audit --pretty` path now report the canonical `E5508` CLI-usage diagnostic, keeping the shared command-shape code aligned with `specs/15-errors.md`
- the documented Node execution subset is now live for `run` / `test`, and the documented Node analysis/build subset is now live for `check` / `build` on the same source-compatible inputs while `effects` / `package-effects` stay gated explicitly; the package-corpus smoke lane exercises representative Node-package runtime cases across explicit and inherited `apiSurface=node` paths, and the semver-style `process.argv.slice(2)` probe now also strips the CLI `--` separator before constructing Node runtime argv so guest-argument passthrough matches the documented Node path
- the documented Node embedding-build subset now also has explicit and inherited `apiSurface=node` smoke coverage for `kali build --capi` and `kali build --component`, so the later Phase-3 embedding rows stay pinned to real artifact evidence instead of only the broader `check` / `build` lane
- `kali init` smoke coverage now includes the nested-child-project case under an ancestor manifest, so the documented subdirectory scaffold path is pinned as a real CLI demo instead of only as a spec claim
- the optimization/PGO lane now also covers browser-bundle JSON builds with attached profile data, so the `build --profile` provenance path is no longer exercised only by the executable/library smoke case
- Brave stable wrapper spellings (`brave-browser-stable` and `brave browser stable`) now resolve through the shared browser-launcher alias table too, keeping the browser host chooser aligned with another common stable-channel name family without changing the browser-runtime contract
- the browser entrypoint smoke lane now also covers the Google Chrome stable wrapper spelling (`google-chrome-stable`) and the Microsoft Edge stable wrapper spellings (`msedge-stable`, `edge-stable`, and `microsoft-edge-stable`), plus the Firefox/Opera/Vivaldi stable spellings (`firefox-esr`, `opera-stable`, and `vivaldi-stable`), so the alias table stays pinned across more common stable-channel names without changing the browser-runtime contract
- the browser entrypoint smoke lane now also covers the base Chrome and Google Chrome wrapper families (`chrome`, `chrome-beta`, `chrome-canary`, `chrome-dev`, `chrome-unstable`, `google-chrome`, `google-chrome-beta`, `google-chrome-canary`, `google-chrome-dev`, and `google-chrome-unstable`), keeping the Chromium-family browser alias coverage aligned with the runtime harness normalization table without widening the browser contract
- the same browser-entrypoint lane now also covers the spaced Google Chrome stable spelling (`google chrome stable`), keeping the launcher alias coverage aligned with the runtime harness normalization table without widening the browser contract
- the browser entrypoint smoke lane now also covers additional Firefox wrapper spellings (`firefox`, `firefox-beta`, `firefox-nightly`, `firefox-developer-edition`, `firefox developer edition`, and `firefox beta`), so the alias table stays aligned with the broader Firefox family without changing the browser-runtime contract
- the browser entrypoint smoke lane now also covers the Chrome-for-Testing family (`chrome-for-testing`, `chromium-for-testing`, and `google-chrome-for-testing`) plus the additional privacy/community aliases (`librewolf`, `waterfox`, `zen-browser`, `zen browser`, `thorium-browser`, and `thorium browser`), keeping the CLI-level browser evidence aligned with the broader launcher alias table without changing the browser-runtime contract
- the browser entrypoint smoke lane now also covers the plain `opera`, `vivaldi`, and `mullvad browser` spellings alongside the broader browser family set, keeping the browser-launcher coverage aligned with the same alias table without changing the browser-runtime contract
- the browser entrypoint smoke lane now also covers the remaining `chromium-browser`, `chromium-headless-shell`, `google chrome`, `brave`, `brave-browser`, `brave browser`, `vivaldi snapshot`, `microsoft-edge`, and `microsoft edge` spellings, so the CLI smoke lane stays aligned with the runtime alias table without widening the browser contract
- The Brave stable wrapper spellings (`brave-browser-stable` and `brave browser stable`) now have dedicated browser-entrypoint smoke coverage as well, so the alias table stays pinned beyond the existing browser-entrypoint cases.
- the browser entrypoint smoke lane now also covers the additional Edge, Brave beta/dev/nightly, and Opera beta/developer/unstable aliases, keeping the launcher alias table aligned with the broader wrapper families without changing the browser-runtime contract
- the browser/runtime compatibility layer now also exposes a deterministic `Atomics::is_lock_free` query helper alongside the existing shared-buffer bytewise operations, keeping the later threaded-profile baseline honest about the host's byte-atomic capability without widening the public maturity claims; the probe now lives behind a shared `kali_common` helper so browser-facing compatibility layers can reuse one capability check instead of re-encoding the same atomic target predicate in each crate, and the regression suite pins that helper directly so the capability probe stays explicit rather than only being covered through indirect buffer mutations
- the host-registered predicate context now carries normalized file, network, process-env, timer, and eval detail keys in addition to the existing process/thread payloads, so programmable-policy callers can inspect the same deterministic capability metadata for path/URL/key-shaped operations without inventing a second policy vocabulary
- the public README command reference now also calls out `kali test --coverage`, keeping the user-facing summary aligned with the stable function-coverage contract already documented in `specs/12-cli.md`
- the public README command reference now also names `kali build --validate-ir`, keeping the user-facing build summary aligned with the documented structural-validation aid in `specs/12-cli.md`
- the base library build smoke lane now also exercises `--validate-ir`, so the library artifact path shares the same early IR-validation coverage as the component, C-ABI, and browser-bundle lanes
- raw-URL `install` now has a configless-project smoke regression that materializes `kali.lock` and the raw cache without scaffolding a placeholder `kali.json`, so the configless install split stays honest for explicit URL targets too
- the repeated-build determinism lane now also replays the `--capi` and `--component` artifact modes, keeping the public embedding outputs pinned alongside the executable, library, and browser bundle cases instead of only the lower-footprint build shapes
- the browser package-corpus baseline now also exercises `Blob.text()` / `File.text()` in addition to the shared stream/blob/web-API surface, so the later browser breadth lane keeps the text-decoding helpers evidence-backed alongside the existing transform-stream and crypto coverage
- the browser runtime coverage lane now also has a browser-harness JSON smoke regression, so `kali test --coverage` stays pinned on the browser-requested execution contract instead of only the native standalone runner
- the browser runtime coverage lane now also proves the inherited `compilerOptions.apiSurface = browser` path in JSON output, so the coverage envelope stays aligned across both explicit and inherited browser-requested forms instead of only the direct flag path
- the late host-control `check` lane now also pins `Deno.pid` / `Deno.chdir` / `Deno.exit` and their `process.*` / `globalThis.*` variants in CLI smoke coverage, keeping the process-control gate honest in both text and JSON output alongside the resolver tests; `run` / `test` now also mirror that same gate in JSON output so the unsupported process-control and working-directory paths stay honest across the execution surface too
- `kali init` now scaffolds missing target directories before the empty-directory check and carries a nested-child current-directory regression, so the workflow-command lane stays aligned with the documented current-directory-scoped init contract

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
