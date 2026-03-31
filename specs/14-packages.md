# 14 — Package Management

## Dependency Compatibility

Package loading is compile-time first: Kali resolves and analyzes dependency graphs during the canonical **source-graph commands** (`check` / `effects` / `build` / `run` / `test`), and it links them for artifact-producing/executing flows rather than pretending every command does the same work. Single-package registry-analysis commands (`package-effects`, `package-audit`) are separate later tools rather than hidden variants of that ordinary source-graph workflow. This chapter follows the shared **linked-artifact model** from [SPEC.md](../SPEC.md): for normal builds, application code and its statically resolvable dependencies lower into one linked core guest payload, and companion outputs such as JS glue do not change that rule.

Ownership rule:
- this chapter owns package-resolution order, install mutability, lock/materialization behavior, and registry/raw-URL dependency rules
- [12 — CLI](12-cli.md) owns command-line flag/arity behavior for `install`, `package-effects`, and `package-audit`
- [19 — Feature Maturity](19-feature-maturity.md) owns whether those package-oriented commands are available in a given phase
- [18 — Schemas](18-schemas.md) owns the machine-readable payloads emitted by package-analysis commands

Reading shortcut:
- this chapter uses three package-workflow buckets on purpose, because the bootstrap brief's broad package goal is easy to overread if they blur together
- **source-graph commands** (`check` / `effects` / `build` / `run` / `test`) analyze a local source/import graph plus its resolved dependencies under the selected command context
- **install workflow** (`install`) is the only early command family allowed to mutate manifest/lock/materialized dependency state
- **registry-analysis commands** (`package-effects` / `package-audit`) are later single-package workflows with documented command/schema shapes, but their actual availability still comes from [19 — Feature Maturity](19-feature-maturity.md) (`package-effects` is **Phase 2 target**; `package-audit` is **Later compatibility**)
- follow the shared **`package-effects` dual classification** from [SPEC.md](../SPEC.md): `package-effects` is still a registry-analysis command in this chapter even though, by output contract, it also belongs to the later public effect-report surface
- use that split before reading any sentence that says a package is “supported”, so package-shape support, install behavior, and later registry-analysis tooling do not get conflated

Canonical dependency-source shorthand:

| Source kind | Phase-1 standing | Notes |
|---|---|---|
| Registry packages (`npm`, `jsr:`) | first-class | participate in manifest + lock + materialization, and may be installed/analyzed through the ordinary project workflow when they fit the package-support decision order |
| Raw URL imports | first-class | participate in the same lock/cache discipline, but stay on the ordinary source-graph/install workflow rather than becoming registry-analysis targets |
| Registry-analysis commands (`package-effects`, `package-audit`) | documented later-phase command family only | registry-only by input shape; these answer a different question from whether a project dependency is installable/checkable/buildable/executable |

Phase-1 shorthand answer:
- **Potentially yes**, depending on the exact support rung being claimed, for pure JS/TS npm/JSR packages and raw URL imports that fit the shared **linked-artifact model** and whose host assumptions match either the Deno-oriented standalone surface or the shared **Phase-1 browser-targeted command set**.
- **Not yet** for package support that depends on the broader `node` API surface.
- **No by default** for packages whose normal published install/runtime path falls into the shared **native/binary/bootstrap-heavy package contract**.

Support-rung clarification for that shorthand:
- on the Deno-oriented standalone surface, early package claims may be **installable/materializable**, **checkable**, **buildable**, or **executable** depending on the command being discussed
- on the shared **Phase-1 browser-targeted command set** — including equivalent inherited-config forms whenever the effective `apiSurface` resolves to `browser` — early package claims are normally **checkable** and sometimes **deployable-through-host** via `build --bundle`; they are not standalone-browser **executable** claims in Kali itself
- practical simplification: if a browser-targeted package claim sounds like “browser support,” rewrite it as either **checkable** or **deployable-through-host** before deciding whether it is actually in scope
- when package support wording is ambiguous, prefer naming the exact command/context and rung instead of saying only that a package is “supported in the browser-targeted context”

This shorthand is only a triage aid. The full answer still uses one fixed decision order, always read against the published artifact Kali actually installs rather than against an upstream repo's source tree, build pipeline, or optional install-time side effects:

| Step | Question | Phase-1 shortcut |
|---|---|---|
| 1. package shape | Does the published package stay inside the **pure JS/TS package contract** and the shared **linked-artifact model**? | if **no**, it falls into the **native/binary/bootstrap-heavy package contract** and is rejected by default |
| 2. host/API fit | Do its runtime assumptions fit the Deno-oriented standalone surface or the shared **Phase-1 browser-targeted command set**? | Deno/browser-targeted fits may proceed; broader Node assumptions stay phase-gated |
| 3. command maturity | Is the requested command/context actually shipped for that host/API combination? | for example, browser-targeted `check` / `build --bundle` are in scope, but standalone browser `run` / `test` are not |
| 4. claimed support rung | Are you claiming `installable/materializable`, `checkable`, `buildable`, `executable`, or `deployable-through-host`? | name the rung explicitly instead of saying only “supported” |

Canonical answer template:
- prefer answering package-support questions in one sentence using this order: **`<package>` is `<rung>` for `<command/context>` because the published artifact Kali actually installs `<does/does not>` fit the package-shape and host/API requirements for that command/context.**
- examples:
  - `lodash` is **executable** for `kali run` in the shared **Default standalone context (schema v1)** because its published package stays inside the pure JS/TS contract and does not require broader Node-only host APIs.
  - a browser-only UI helper may be **checkable** and **deployable-through-host** for browser-targeted `check` / `build --bundle`, while still not being standalone-browser **executable** in Kali itself.
  - a package with N-API bindings is **rejected by default** in Phase 1 because it falls outside the pure JS/TS package contract before command maturity is even considered.

Compact workflow comparison:

| Workflow bucket | Primary question | Mutates manifest/lock/materialized state? | Context participation |
|---|---|---|---|
| **source-graph commands** (`check` / `effects` / `build` / `run` / `test`) | “Can Kali analyze/build/run this local project graph in the selected command context?” | No | resulting **availability context** (derived from the full effective command context for the participating axes) |
| **install workflow** (`install`) | “What dependency state should be recorded/materialized for this project?” | Yes | intentionally profile-agnostic in Phase 1 |
| **registry-analysis: `package-effects`** | “What effects would one registry package report under the inherited analysis context?” | No | inherits semantic analysis context once the command exists; version selection still follows the shared **identity-only registry target** + **stable-release selection rule (schema v1)** rather than the current project's installed version |
| **registry-analysis: `package-audit`** | “What context-free registry-analysis/security-audit result is reported for one package?” | No | context-free in schema v1; version selection still follows the shared **identity-only registry target** + **stable-release selection rule (schema v1)** rather than the current project's installed version |

Registry-analysis independence reminder:
- later `package-effects` / `package-audit` are intentionally **not** alternate views over the current project's installed dependency graph
- that remains true even under the shared **`package-effects` dual classification**: its effect-report role does not change its one-package registry target, stable-release selection rule, or project independence
- they analyze one explicit registry package identity using the shared schema-v1 stable-release selection rule unless a later revision adds an explicit version coordinate
- practical simplification: answer registry-analysis questions and project-command support questions separately, even when they mention the same package name
- this keeps project support questions (`check` / `build` / `run` / `test`) separate from single-package registry questions and prevents the current `kali.lock` or `node_modules/` state from silently changing what a registry-analysis command means

Bootstrap-reading shortcut:
- the bootstrap's package goal should not be read as one yes/no answer to “does Kali support npm?”
- package claims should name the full four-step reading: package shape → host/API fit → command maturity → claimed support rung
- registry-analysis commands stay a separate question from project-command support: they are single-package workflows, and even `package-effects` does **not** use the current project's installed dependency state to pick a different package version
- this keeps Phase-1 ecosystem claims honest: many pure JS/TS packages are already in scope early, while Node-host-heavy or native/binary/bootstrap-heavy packages remain clearly outside the same promise

### Supported Packages
Kali can support registry packages (npm/JSR) that stay inside the shared **pure JS/TS package contract** from [SPEC.md](../SPEC.md), but every concrete support claim still needs the full package reading order: package shape → host/API fit → command maturity → claimed support rung.

Phase-1 source-kind clarification:
- supported **raw URL imports** are also first-class dependency inputs in Phase 1
- they follow the same determinism goals (pinning, lockfile tracking, materialized cache state, and no hidden auto-repair by non-install commands) even though they are not registry packages
- registry-package compatibility and raw-URL compatibility should therefore be read as two dependency-source lanes under one shared install/lock discipline, not as “packages are supported but raw URLs are merely ad hoc”
- this matters for bootstrap alignment: “easy access to millions of existing JavaScript packages” is not registry-only wording in Kali's normalized package story; the same deterministic dependency model also covers direct raw URL inputs where that workflow is the natural fit

Phase simplification:
- **Phase 1 MVP**: pure JS/TS packages that fit the shared **linked-artifact model** and whose host/API assumptions fit either the Deno-oriented standalone surface or the exact **Phase-1 browser-targeted command set**.
- **Phase 3 target**: the remaining pure JS/TS packages whose normal host assumptions still depend on the `node` API surface and additional Node built-ins.

This keeps the early ecosystem promise realistic: many utility libraries and browser-targeted packages are in scope early, while Node-host-heavy packages and the excluded **native/binary/bootstrap-heavy package contract** follow later compatibility work.

Install-time clarification:
- read package support through the shared **published-artifact-first package reading** from [SPEC.md](../SPEC.md): judge the package by the published version/tarball Kali actually installs plus the selected entry files for the active context, not by whatever build pipeline the upstream repository used before publishing
- packages whose normal install/runtime path falls into the **native/binary/bootstrap-heavy package contract** stay outside the Phase 1 compatibility promise even if most of their published sources are JS/TS
- conversely, a package is not excluded merely because its repository used bundling/codegen/native tooling before publish if the published artifact Kali installs already contains the ordinary JS/TS files it needs
- `--allow-scripts` may permit the hook to run for installation workflows, but it must not be misread as a promise that Kali supports that excluded package contract end-to-end

Bootstrap-alignment rule:
- the bootstrap brief's “support non node-gyp packages from npm” goal is normalized through the shared **pure JS/TS package contract**, not through a narrower “anything without `node-gyp` must work” reading
- in practice, packages that depend on N-API/native bindings, prebuilt native modules, postinstall-downloaded executables, or other binary/bootstrap-heavy installation paths remain outside the early supported set even when `node-gyp` itself is absent
- this keeps package compatibility defined by the package's normal source/install contract rather than by one specific native-addon tool name

## Canonical Phase-1 Package-Compatibility Interpretation

Early registry-package compatibility follows the shared **package-support decision order** from [SPEC.md](../SPEC.md), so package shape, host/API fit, and command maturity do not get conflated:
- **Phase 1 package compatibility is broader than "only Deno-authored packages"**.
- **Phase 1 package compatibility is narrower than "Node mode works"**.
- **Phase 1 package compatibility is also not synonymous with "Deno standalone only"**: the shared **Phase-1 browser-targeted command set** is already part of the early package story for packages whose host/API assumptions fit that context.

Concretely, a package can be supported in Phase 1 when:
- its code can be resolved statically into the shared **linked-artifact model**,
- its module format can be handled by Kali's ESM/CJS pipeline,
- and its host/API assumptions are satisfied by either the Deno-oriented standalone surface *(which already includes the shared **Web baseline**)*, or the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md).

A package is **not** automatically in scope just because it lives in npm or JSR. If it depends on broader Node globals/core modules or falls into the **native/binary/bootstrap-heavy package contract**, it stays phase-gated or rejected with the rest of that compatibility work.

Compact bootstrap-normalization table:

| Package shape | Early handling | Why |
|---|---|---|
| Pure JS/TS package whose host/API assumptions fit the Deno-oriented standalone surface *(including the shared **Web baseline**)* | **Phase 1 MVP in scope** | This is the core standalone half of the **pure JS/TS package contract** target |
| Pure JS/TS package whose host/API assumptions fit the shared **Phase-1 browser-targeted command set** | **Phase 1 MVP in scope for those browser-targeted commands** | Package shape is acceptable and the selected Phase-1 browser-targeted command context can analyze/build it without implying standalone browser execution |
| Pure JS/TS package that still expects broader Node globals/core modules | **Phase 3 target** | Package shape is acceptable, but host/API requirements exceed the Phase 1 surface |
| Package that needs native addons, N-API bindings, prebuilt binaries, or postinstall-downloaded executables in the published package Kali installs | **Rejected by default** | Falls into the **native/binary/bootstrap-heavy package contract** under the shared **published-artifact-first package reading** |
| Package whose repository used heavy prepublish tooling, but whose published package already contains ordinary JS/TS artifacts that fit Kali's selected resolution path | **Triaged by the normal pure JS/TS rows above** | Upstream build tooling alone is not a support veto if the installed package artifact is already ordinary JS/TS |
| Package whose install path uses npm lifecycle scripts but whose shipped runtime code still stays inside the pure JS/TS contract | **Phase 1 MVP (opt-in only)** for the install-hook path; runtime/build support still depends on the other rows | `--allow-scripts` is only an installer escape hatch, not a blanket compatibility promotion |

This table is intentionally about **package-shape triage**. The active `apiSurface`, runtime profile, and feature-maturity gates still determine whether a given project command can actually analyze, build, or run that package in the selected context.

Support-decision order simplification:
- use the shared **package-support decision order** from [SPEC.md](../SPEC.md): package shape, then host/API fit, then command/profile maturity, all under the shared **published-artifact-first package reading**.
- use the shared **package-support ladder** from [SPEC.md](../SPEC.md) whenever a section needs to say whether a package is merely installable/materializable, checkable, buildable, executable, or deployable-through-host.
- `--allow-scripts` can affect installation of npm packages, but it does not skip that decision order and never upgrades an unsupported package into a supported project-command/runtime contract.

Practical shorthand:
- a package may be **installable/materializable** without being **checkable**, **buildable**, **executable**, or **deployable-through-host** for the selected command/context.
- package discussions should therefore name the rung they mean instead of using one broad word such as “supported”.
- when browser-targeted package support is the topic, default to the narrower browser words first: **checkable** or **deployable-through-host**. Use **executable** only for Kali-hosted runtime/test contracts.
- if the question is about `package-effects` or `package-audit`, answer it separately from those project-command rungs: those commands are registry-analysis workflows, not alternate ways to ask whether the current project graph is runnable.

Common support-claim examples:

| Claim wording | What it should mean |
|---|---|
| “this npm package works in Phase 1” | Name the rung: for example **checkable/buildable/executable** on the Deno-oriented standalone surface, or **checkable/deployable-through-host** in the shared **Phase-1 browser-targeted command set** |
| “this browser package is supported” | Usually **checkable** and potentially **deployable-through-host** via `build --bundle`, including equivalent inherited-config forms when the effective `apiSurface` is `browser`; not standalone browser-runtime **executable** support in Kali itself |
| “this package installs” | Only **installable/materializable**; this does not by itself promise that `check`, `build`, `run`, or browser deployment will succeed |
| “this package needs Node” | Package shape may still be fine, but the claim is blocked on host/API fit and therefore stays on the Phase 3 Node path rather than becoming an early package-compatibility success |
| “this package can be audited/analyzed” | Answer the registry-analysis question separately: that refers to later `package-effects` / `package-audit` workflows, not to ordinary project-command support |

This keeps five often-confused questions separate: “can Kali materialize this package?”, “can Kali understand its source shape?”, “can the selected command/context actually support the host APIs it expects?”, “can Kali deploy it through a non-Kali host such as the browser bundle path?”, and “what would a later single-package registry-analysis workflow report about it?”.

## Dependency Source Kinds

To keep install, lock, and materialization rules simple, Kali distinguishes only these early source kinds:
- **Registry packages** — npm and JSR packages declared in `kali.json` under `dependencies` / `devDependencies`, resolved by package identity plus the manifest's exact pinned version, and materialized into `node_modules/`
- **Raw URL imports** — exact `https://...` dependencies declared in source code or `kali.json#imports`, cached under `.kali/cache/urls/`

Clarification:
- path/local alias rewrites in `kali.json#imports` are not a third dependency source kind; they are source-organization rewrites that do not create separate external lock/materialization state.
- schema v1 `kali.json#imports` stays in the URL/path-rewrite lane: it may target raw URLs or path/local rewrites, but it must not alias registry packages or canonical registry identifiers such as `jsr:@std/path`.
- raw URL dependencies are first-class project dependency inputs, but they are **not** registry-analysis targets: schema-v1 `package-effects` / `package-audit` accept only canonical registry package identifiers, while raw-URL questions stay on the ordinary source-graph commands and the shared install/lock workflow.

### Canonical Registry Package Identifiers

Kali uses one shared registry-package identifier grammar across `kali.json`, `kali install`, package-analysis commands, and lockfile provenance:
- **npm packages** use the normal bare package name, for example `lodash` or `@types/node`
- **JSR packages** use an explicit `jsr:` prefix, for example `jsr:@std/path`

Interpretation rules:
- follow the shared **registry package identifier vs package coordinate** split from [SPEC.md](../SPEC.md): CLI/manifests/diagnostics use the registry package identifier spelling (`lodash`, `jsr:@std/path`), while structured JSON package metadata may decompose the same package into `{ registry, name, version }`
- bare package names default to the npm registry in CLI/package-manifest contexts
- the `jsr:` prefix is required for JSR so package identity stays unambiguous in `kali.json`, lockfiles, diagnostics, and install commands
- this prefix is a **registry identity marker**, not a request to invent a second installation layout; both npm and JSR registry packages still materialize into `node_modules/` in early phases
- the canonical on-disk materialization path is `node_modules/<package-name>` using the registry-native package name without the `jsr:` identity marker; for example npm `lodash` materializes at `node_modules/lodash`, and `jsr:@std/path` materializes at `node_modules/@std/path`
- because early phases use one shared `node_modules/` tree, Kali must reject a project that would require two distinct registry identities to occupy the same on-disk package path (for example npm `@scope/name` and `jsr:@scope/name`) rather than inventing shadow package trees or ambiguous resolution precedence
- docs and examples should prefer this canonical form instead of relying on context to guess whether `@scope/name` came from npm or JSR

Declaration-model rule:
- registry dependencies belong in the project manifest
- raw URL dependencies belong in source/import maps, not in a second manifest dependency table
- explicit raw-URL installs follow the shared **raw-URL install staging/pin workflow** from [SPEC.md](../SPEC.md): they pin/materialize shared lock/cache state without creating a new top-level manifest section, and durable raw-URL ownership still belongs in source imports or `kali.json#imports`
- because raw URL state is owned by the current source/import-map graph instead of a manifest dependency table, plain `kali install` may prune raw URL lock/cache entries that are no longer referenced

Lockfile rule:
- `kali.lock` is the canonical reproducibility record for **both** source kinds
- registry packages and raw URL imports may use different on-disk materialization locations, but they share one lock discipline
- non-install commands must check the required materialized state for the dependency kinds actually used by the project instead of assuming `node_modules/` alone is always the full dependency state

This removes an ambiguity from the earlier wording: a URL-only project may have no `node_modules/` tree at all and still be fully installed.

Registry-collision simplification rule:
- if two manifest entries would collapse to the same `node_modules` package path after stripping the optional `jsr:` registry marker, `kali install` must fail explicitly before materialization
- the failure should name both conflicting package identities so the user can choose one source of truth
- early phases prefer this explicit rejection over a more complex multi-registry shadow layout

### Canonical stable-release selection rule (schema v1)

Several early schema-v1 workflows intentionally accept the shared **identity-only registry target** form from [SPEC.md](../SPEC.md) instead of an inline version/range selector. To keep those workflows deterministic, they share one resolution rule:
- **latest non-yanked stable published version** means the highest published SemVer version for that targeted **registry package identifier** that has **no prerelease identifier** and is not yanked
- those identity-only workflows must fail explicitly rather than silently selecting a prerelease when no non-yanked stable version exists
- the canonical failure path for that case is `E5001`: the package identity resolved, but no acceptable stable release existed for the schema-v1 identity-only workflow

Schema-v1 uses this rule for:
- registry-analysis commands such as `kali package-effects <pkg>` and `kali package-audit <pkg>`
- explicit registry-package adds via `kali install <pkg>` and `kali install --dev <pkg>`

Install simplification:
- when `kali install <pkg>` or `kali install --dev <pkg>` adds a new manifest entry from that **identity-only registry target** form, it follows the shared **stable-release selection rule (schema v1)** and **exact-version-first registry manifest rule (schema v1)** from [SPEC.md](../SPEC.md)
- schema-v1 registry dependency values in `kali.json` are therefore exact resolved version strings; broad SemVer ranges are invalid config (`E5009`) rather than an alternate supported manifest mode

### Package Resolution
Follow the common package.json / `exports` / CommonJS-vs-ESM mechanics used by the Node ecosystem, but keep the early-phase Kali rules explicit so browser-targeted, Deno-oriented, and later Node-specific behavior do not drift.

Terminology simplification:
- use the cross-spec term **package-resolution context** from [SPEC.md](../SPEC.md) for the normalized package-selection inputs: `apiSurface` plus module edge kind (`import` vs `require`)
- supported browser-targeted commands should therefore reuse one browser package-resolution context instead of describing near-duplicate browser condition ladders per command

Package-resolution quick summary:

| Step | Canonical rule |
|---|---|
| import-map rewrite | Apply `kali.json#imports` first |
| package branch selection | Prefer `package.json#exports`; evaluate it with the current **package-resolution context** |
| legacy fallback | Only if `exports` does not resolve the entry; still respect edge kind |
| browser refinement | In the shared browser-targeted context, apply `package.json#browser` rewrites **after** branch/entry selection, not as a second parallel condition-order system |
| file classification | Classify the resolved file once and share that ESM/CJS decision across resolution, checking, and lowering |

Simplification rule:
- `exports` picks the package-published branch
- browser replacement maps refine that chosen browser-targeted path
- legacy fallback happens only when `exports` did not resolve the edge at all
- Kali must not bounce between those stages heuristically looking for "something that works"

Canonical early-phase code-resolution ladder:
1. Apply import-map rewrites from `kali.json#imports` before package resolution.
2. Preserve any explicit registry qualifier on the package specifier (for example `jsr:@std/path`) so later resolution, lockfile lookup, and diagnostics keep the same package identity.
3. Resolve package self-references (`"name": "pkg"` imported as `pkg/...`) using the package's own `exports` map before walking upward into `node_modules`.
4. If the specifier names a package or package subpath, consult `package.json#exports` first.
5. Evaluate `exports` against the current API surface and edge kind:
   - distinguish **ESM import edges** from **CJS require edges**
   - resolve the exact requested subpath; do not flatten subpath exports into one package-wide entry
   - use the canonical condition order table below
   - unsupported or unmatched conditional branches are skipped; Kali should not guess a fallback branch that the package did not publish
6. If `exports` does not resolve the entry, fall back to legacy entry fields using the same API-surface intent **and still respecting edge kind**. In schema v1, keep one shared fallback table instead of near-duplicate prose ladders:

   | Context | ESM `import` edge | CJS `require` edge |
   |---|---|---|
   | Deno-oriented standalone (`--api deno`, Phase 1 default) | `module`, then `main` | `main`, then `module` |
   | browser-targeted context | `module`, then `main` | `main`, then `module` |
   | later Node API surface | later Node-specific rule when explicitly documented; until then, do not guess | later Node-specific rule when explicitly documented; until then, do not guess |

7. In the shared **browser-targeted context**, after `exports` or that legacy fallback table picks a package-published target, apply any `package.json#browser` replacement-map rewrite that covers that selected package-local path:
   - this rewrite layer belongs to the one shared browser **package-resolution context** reused by the exact **Phase-1 browser-targeted command set** and by later browser-context analysis commands once their own maturity rows open
   - that later reuse is intentionally about one shared browser package-resolution rule, not about widening the exact **Phase-1 browser-targeted command set** before those later commands are phase-enabled
   - if the browser map rewrites the selected path to another package-local file, continue resolution from that rewritten target
   - if the browser map marks the selected path as unavailable (`false`), reject that edge instead of probing alternate non-browser files heuristically
   - this browser-map stage refines the already chosen browser-targeted package edge; it does not restart package resolution under a second ad hoc condition-order algorithm
8. Resolve relative/file entries with extension probing (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`).
9. Classify the resolved file as ESM or CJS using the canonical early-phase rule set:
   - `.mts` / `.mjs` → always ESM
   - `.cts` / `.cjs` → always CommonJS
   - `.ts` / `.tsx` / `.js` / `.jsx` inside a package boundary follow the nearest applicable `package.json#type`
   - when those ambiguous extensions appear outside an applicable package boundary, default to ESM unless the documented resolver/classifier rules require a specific CommonJS interpretation
   - the chosen module kind for a resolved file is shared by resolution, checking, and lowering; Kali must not let one subsystem treat the same file as ESM while another treats it as CJS

Canonical `exports` condition order:

| Analysis/runtime context | Condition order |
|---|---|
| Deno-oriented standalone surface (`--api deno`, Phase 1 default) | `deno`, then edge kind (`import` or `require`), then `default` |
| browser-targeted context | `browser`, then edge kind, then `default` |
| later Node API surface | `node`, then edge kind, then `default` |

Phase-1 simplification:
- only the canonical conditions above plus `default` are part of the early stable resolution contract
- if a package's `exports` tree requires additional environment conditions to choose a branch faithfully, Kali should reject that edge with the canonical availability path instead of guessing bundler-specific precedence

Important separation rules:
- runtime/code resolution must not treat `types` as a normal execution condition
- the Deno-oriented standalone surface should honor a package's explicit `deno` condition when present instead of behaving like an unspecified generic bundler
- `--api node` package resolution is part of the same Phase 3 Node-compatibility gate as the rest of the Node API surface; early phases should not resolve packages as though Node mode were already implemented for `check` or `build`
- the shared **Phase-1 browser-targeted command set** — and any later browser-context analysis command that explicitly reuses that same package-resolution context — should honor a package's explicit `browser` condition and any applicable `package.json#browser` replacement-map rewrite consistently so analysis and emitted artifacts do not resolve different files by accident
- `package.json#module` is treated only as a legacy bundler-compatibility fallback when `exports` is absent; it must not override an explicit `exports` map, and it should not outrank `main` on a legacy CJS `require` edge
- when a package explicitly marks a path as unavailable for the active profile (for example `browser: false`), Kali must respect that instead of probing alternate files heuristically
- declaration/type lookup follows the separate ladder in [Type Resolution](#type-resolution)

To keep configuration simple, `kali.json#imports` is the canonical aliasing mechanism in early phases. A separate TypeScript-style `paths`/`baseUrl` compatibility layer may be added later if ecosystem pressure justifies it, but it is not part of the MVP contract.

Import-map boundary rule:
- `kali.json#imports` may rewrite to raw URLs or path/local targets only.
- It must not be used to alias one registry package to another bare specifier or to a canonical registry identifier such as `jsr:@std/path`.
- Registry ownership stays in `dependencies` / `devDependencies` so install, lockfile provenance, diagnostics, and package-analysis commands all have one source of truth.

Canonical `kali.json#imports` matching rules (schema v1):
- keys without a trailing `/` are **exact-match** rewrites for the full module specifier
- keys with a trailing `/` are **prefix-match** rewrites and apply only when the imported specifier starts with that full prefix
- when multiple keys could match, the **longest matching key wins**
- a prefix key ending with `/` must rewrite to a target that also ends with `/` so the unmatched suffix can be appended without inventing path-join heuristics
- local path targets (`./...`, `../...`, or absolute path-like targets when supported by the host platform) are resolved relative to the directory containing the owning `kali.json`
- raw-URL targets stay absolute after rewrite and then participate in the normal lock/cache materialization flow
- import-map rewrites happen before package resolution; if no import-map entry matches, the original specifier continues into the normal relative/package resolution ladder
- schema v1 does **not** support wildcard/glob/regex import-map keys or targets; exact and prefix rewrites are the whole stable contract

Simplification rule: for any package-resolution edge case not yet modeled faithfully, prefer an explicit `E5006`/availability failure over bundler-style guesswork. This keeps package behavior deterministic and auditable for sandboxed builds.

Practical classifier note:
- package resolution owns the final module-kind decision for a resolved file edge
- parser/checker/codegen must consume that same decision rather than rerunning slightly different heuristics later
- this avoids a common cross-tool drift where `package.json#type`, extension-based classification, and TS/JS frontend assumptions disagree about the same dependency file

### Installation
```bash
kali install lodash                         # Add/install single registry package from npm
kali install jsr:@std/path                  # Add/install single registry package from JSR
kali install                                # Materialize all declared dependencies for the project
kali install --allow-scripts lodash         # Opt into lifecycle hooks for one npm-targeted install; plain `kali install --allow-scripts` is valid only when the invocation actually has effective npm-scriptable install work
kali install --dev vitest                   # Add/install dev dependency
kali install https://deno.land/std/path/mod.ts  # Pin/materialize raw URL dependency
```

Argument semantics are intentionally simple:
- `kali install` takes zero or one explicit **install target** in schema v1
- registry install targets use the canonical registry-package identifier grammar from this chapter (`lodash`, `@types/node`, `jsr:@std/path`)
- in schema v1, explicit registry install targets are **package identities only**, not inline version/range selectors
- adding a registry package through that identity-only CLI form uses the shared **stable-release selection rule (schema v1)** plus the **exact-version-first registry manifest rule (schema v1)** from [SPEC.md](../SPEC.md): resolve the latest non-yanked stable published version, refresh `kali.lock` using that concrete version, and record the manifest dependency as that same exact version string
- registry install targets therefore mutate `kali.json` (`dependencies` or `devDependencies`) and then refresh lock/materialized state
- in the canonical **configless install split** from [SPEC.md](../SPEC.md), an explicit registry-package add (`kali install <pkg>` or `kali install --dev <pkg>`) first creates the minimal canonical manifest `{ "schemaVersion": 1 }` at the effective project root, then records the dependency there
- `--dev` applies only to registry install targets; `kali install --dev https://...` is rejected with `E5008` instead of inventing a raw-URL dev-dependency table
- explicit raw-URL installs follow the shared **raw-URL install staging/pin workflow** from [SPEC.md](../SPEC.md): they update shared lock/cache state only, do not invent a second manifest section, and should not rewrite source/import-map declarations implicitly
- in that same **configless install split**, an explicit raw-URL install may still create `kali.lock` and `.kali/cache/urls/` state at the effective project root, but it must not create a placeholder manifest by itself
- under that same **raw-URL install staging/pin workflow**, an unreferenced staged URL may disappear on the next plain `kali install`
- plain `kali install` reconciles the current manifest + import graph with `kali.lock`, `node_modules/`, and `.kali/cache/urls/`, and may prune raw URL entries that are no longer reachable from that graph
- in that same **configless install split**, plain `kali install` is a no-op success when the effective project root contributes no manifest/import/source dependency inputs, and it must not create a placeholder manifest as a side effect
- because install is intentionally profile-agnostic in early phases, `kali install` does **not** take `--api`; passing `--api ...` is invalid command usage (`E5008`), not a request for a second install graph

Install-graph discovery rule:
- because `kali install` usually runs without an explicit primary source input, source-level raw URL imports are discovered from the canonical project-discovery result rather than from one ad hoc command-local source root; together with manifest/import-map declarations, this forms the project's **install-time declaration graph** from [SPEC.md](../SPEC.md)
- the effective project config/root for that scan is the nearest `kali.json` found by searching the current working directory and then its ancestors; if none exists, install uses the current working directory as the project root
- that install-time scan set is filtered by `kali.json` `include` / `exclude` when present, or by the default project-discovery rules from [SPEC.md](../SPEC.md) when those fields are omitted
- recursive install-time discovery must stop at nested child directories that contain their own `kali.json`; those child roots are separate projects in schema v1
- discovery may use a cheap lexical/module-specifier scan of those files plus `kali.json#imports`; it does not require a full check/build just to decide which raw URLs belong in the lock/cache state
- the install-time scan may include declaration-only files too, because they can own type-only imports that still belong to the project's declared dependency graph
- pruning of raw URL lock/cache entries is judged against this install-time declaration graph, not against arbitrary unrelated files elsewhere in the repository

Installation is **fetch-and-link by default**, not "execute package scripts" by default.

Canonical terms:
- follow the shared **workflow-owner split** from [SPEC.md](../SPEC.md): `kali install --allow-scripts` stays an install-time hook workflow only and does not become a second effect/policy/runtime compatibility path
- follow **effective npm-scriptable install work** from [SPEC.md](../SPEC.md): the invocation-scoped npm package work the current `kali install` actually reconciles in a lifecycle-hook-relevant way
- follow **install-time npm-package hook path** from [SPEC.md](../SPEC.md): the schema-v1 boundary for what `--allow-scripts` does and does not mean

To preserve sandbox-first behavior:
- npm lifecycle scripts (`preinstall`, `install`, `postinstall`) are not executed unless the user explicitly opts in with `kali install --allow-scripts`
- `--allow-scripts` applies only to that install invocation; it is not an ambient project default
- package metadata and tarballs can still be analyzed before linking
- follow the shared **install-time npm-package hook path** boundary: this opt-in does **not** imply `--api node`, broader Node package/runtime compatibility, or coverage by the normal `kali effects` / `kali.policy.json` contract
- top-level project sandbox config is ignored by `kali install`, so lifecycle-script execution is intentionally outside the schema-v1 project-policy model rather than being half-governed by it
- package compatibility claims for normal `check` / `build` / `run` / `test` should therefore not be inflated by the existence of this opt-in installer escape hatch

Canonical `--allow-scripts` triage:

| Invocation shape | Result | Why |
|---|---|---|
| `kali install --allow-scripts lodash` (or another explicit npm target) | Valid opt-in path | The invocation has non-empty **effective npm-scriptable install work** if it reaches the normal npm install path |
| plain `kali install --allow-scripts` with non-empty **effective npm-scriptable install work** in the shared **install-time declaration graph** | Valid opt-in path | Hooks may run only for the npm subset the current install actually reconciles |
| plain `kali install --allow-scripts` when that effective npm install work is empty | Invalid usage (`E5008`) | The flag must not silently degenerate into plain `install` when the current invocation has no hook-relevant npm work |
| `kali install --allow-scripts jsr:@std/path` | Invalid usage (`E5008`) | JSR packages do not participate in npm lifecycle-script execution in schema v1 |
| `kali install --allow-scripts https://...` | Invalid usage (`E5008`) | Raw URLs do not expose an npm lifecycle-script surface |
| mixed install work (npm + JSR and/or raw URLs) | Valid, but npm-only hook execution | Lifecycle scripts may run only for the npm install-work subset while the rest stays on the normal script-free path |
| package in the excluded **native/binary/bootstrap-heavy package contract** | Still unsupported | `--allow-scripts` is an installer escape hatch, not a package-shape promotion mechanism |

Additional rules:
- raw URL installs stay outside this escape hatch entirely because they have no registry lifecycle-script surface
- enabling lifecycle scripts does **not** make the excluded **native/binary/bootstrap-heavy package contract** installable/materializable/executable through the normal support ladder; it only permits hook execution for otherwise eligible npm install work

Uses standard `node_modules/` layout by default for maximum ecosystem compatibility. Kali-specific caches live under `.kali/` instead of inventing a second package tree:
```
node_modules/
├── lodash/
│   ├── package.json
│   └── ...
└── zod/
    └── ...

.kali/
└── cache/
    └── urls/            — Cached URL imports and metadata
```
This simplifies interoperability with existing tools, package metadata, and source layouts.

### Lock File
`kali.lock` — deterministic lockfile stored at the effective project root (that is, beside the effective discovered `kali.json` when one exists, otherwise in the current working directory) and committed to version control. Uses a line-oriented TOML-based format for clean diffs and carries its own format version in the file header rather than a JSON `schemaVersion` field.

Canonical simplification for v1:
- registry packages and raw URL imports share **one** lockfile
- they use separate top-level entry kinds so tools do not have to infer source kind from ad hoc fields
- the lockfile records reproducibility data only; materialization location still follows the documented split (`node_modules/` for registry packages, `.kali/cache/urls/` for raw URLs)

Example:
```toml
# kali.lock v1 — do not edit manually

[[package]]
name = "lodash"
version = "4.17.21"
registry = "npm"
resolved = "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz"
integrity = "sha256-..."
dependencies = []

[[package]]
name = "@std/path"
version = "1.0.8"
registry = "jsr"
resolved = "https://jsr.io/@std/path/1.0.8.tgz"
integrity = "sha256-..."
dependencies = []

[[url]]
specifier = "https://deno.land/std@0.220.0/path/mod.ts"
resolved = "https://deno.land/std@0.220.0/path/mod.ts"
integrity = "sha256-..."
```

Interpretation rules:
- `[[package]]` entries are for registry dependencies only and include the originating registry kind (`npm` or `jsr`)
- lockfile package records intentionally keep `registry` and `name` as separate fields for compact diff-friendly storage; the canonical registry identifier used for ordering, diagnostics, and cross-spec references is the derived pair (`lodash` for npm, `jsr:@std/path` for the example JSR entry above)
- `[[url]]` entries are for exact raw URL imports after import-map expansion/pinning
- future lockfile revisions may add optional metadata fields, but they should preserve this top-level split instead of collapsing both source kinds into one ambiguous record shape
- to keep lockfile diffs deterministic, producers should emit `[[package]]` entries sorted by canonical registry identifier, then version; emit `[[url]]` entries sorted by canonical pinned specifier; and sort per-entry dependency lists lexically by canonical dependency identifier
- the canonical registry identifier used for lockfile ordering and diagnostics is the same one used elsewhere in the spec set: npm packages keep their bare name, while JSR packages keep the explicit `jsr:` prefix

## Install-Time vs Command-Time Resolution Boundary

Because package resolution can vary by analysis/runtime context (`--api deno`, the shared **Phase-1 browser-targeted command set** plus later browser-context analysis commands that explicitly reuse it, and later `--api node`), Kali needs one explicit boundary so `install`, lockfiles, and ordinary commands do not drift.

This is the canonical package-management simplification for early phases: Kali keeps one shared installed package state, then performs the final context-sensitive package-edge choice at command time.

Scope note:
- this boundary is about **project commands** that consume project-managed dependency state (`check`, `effects`, `build`, `run`, `test`)
- single-package registry-analysis commands such as later `package-effects` / `package-audit` stay project-independent for version selection and do not consult the current project's installed dependency state
- those registry-analysis commands instead follow the shared schema-v1 **identity-only registry target** + **stable-release selection** rule unless a later revision adds an explicit version coordinate

- `kali install` is **context-agnostic** in Phases 1-3. It locks package versions, fetches/materializes package contents, and records reproducibility data, but it does **not** pre-resolve one permanent `exports`/`browser`/`deno` branch for every future command.
- `check`, `effects`, `build`, `run`, and `test` perform the final **command-time package edge selection** from the already-installed package metadata using the active analysis/runtime context.
- therefore one `kali.lock` and one materialized package tree can serve both the Deno-oriented standalone surface and the shared **Phase-1 browser-targeted command set** without requiring separate per-context installs.
- this is possible because early-phase context differences choose between files that are already present inside the installed package contents; they do not require separate version solves for each supported context.
- if a later feature truly requires context-specific solving or materially different dependency graphs, that complexity must be introduced explicitly in a future lockfile/versioning revision rather than being implied accidentally by Phase 1 package wording.

Practical consequence:
- `kali install` does not take `--api` in early phases, and `compilerOptions.apiSurface` does not cause `install` to write a different lockfile for the same manifest/import graph.
- changing `--api` between `deno` and the shared **Phase-1 browser-targeted command set** affects which already-installed package entry files are chosen at command time, not whether the project is considered installed.
- lockfile/cache state belongs to the effective discovered project root; invoking commands from a subdirectory of the same project should still use that one shared `kali.lock`, `node_modules/`, and `.kali/` state rather than inventing nested installs.
- if a later file-accepting non-install command (`check`, `effects`, `build`, `run`, or `test`) points at explicit files outside the current **install-time declaration graph** from [SPEC.md](../SPEC.md) and those files reach additional raw URL imports, the command should fail with `E5004` and tell the user to rerun `kali install` after updating the project's discoverable sources or import map.
- this is intentional: explicit file targets bypass discovery filtering for command input selection, but they do not retroactively redefine that **install-time declaration graph**, which owns raw URL lock/cache state.

## Deterministic Install & Resolution Contract

This chapter follows the top-level [canonical dependency-management mutability rule](../SPEC.md): in early phases, `kali install` is the only command that mutates project-managed dependency state.

To keep package behavior predictable across `install`, `check`, `effects`, `build`, `run`, and `test`, Kali uses one simple rule set:
- `kali install` is the command that updates dependency-owning manifest fields in `kali.json` when needed, resolves dependency versions, writes `kali.lock`, and refreshes materialized dependency stores.
- `kali check`, `effects`, `build`, `run`, and `test` consume the existing declaration + lock + materialized dependency state; they must not silently re-resolve packages or mutate project-managed dependency state as a side effect.
- If the project's current **install-time declaration graph** from [SPEC.md](../SPEC.md) requires materialized state that is missing or stale, non-install commands fail with `E5004` and tell the user to run `kali install`.
- Here, "stale" means the current declared dependency graph, the corresponding `kali.lock` entries, and the required materialized artifacts no longer agree. Non-install commands should not try to infer staleness from arbitrary mtimes or repair it opportunistically.
- `node_modules/` is the materialized tree for registry packages (npm/JSR), while `.kali/cache/urls/` is the materialized cache for raw URL imports; `kali.lock` is the canonical reproducibility record for both.
- When declaration inputs, `kali.lock`, and the required materialized dependency state disagree, `kali install` is responsible for reconciling them. Other commands should fail clearly rather than guessing which source of truth to trust.
- `--allow-scripts` affects install-time behavior only; it does not change later `check`/`build`/`run` semantics for an already-installed package graph.
- lifecycle scripts executed during install are outside the normal source-program effect-report/sandbox-policy contract and therefore are not evidence that the installed package graph itself requires those same effects at runtime.

This is an intentional simplification: one command mutates project-managed dependency state, all other commands consume it deterministically. For raw URL imports, the source/import-map graph is the declaration source of truth and the lock/cache are the materialized state derived from it.

Diff-friendliness rule:
- lockfile writers should preserve canonical ordering when rewriting existing `kali.lock`
- equivalent dependency graphs should therefore converge on byte-stable lockfile ordering rather than reflecting fetch order or hash-map iteration order

## Import Styles

### ESM (Primary)
```typescript
import { groupBy } from "lodash";
import { z } from "zod";
```

### URL Imports (Deno-style)
```typescript
import { join } from "https://deno.land/std@0.220.0/path/mod.ts";
```
URL imports are cached in `.kali/cache/urls/`. Integrity is verified against the lock file.

Early-phase simplification:
- a URL import used by source code participates in the same lockfile discipline as registry packages
- URL-only projects may therefore have an empty or absent `node_modules/` tree without being considered uninstalled
- non-install commands do **not** repair or repopulate missing URL materialization on the fly; a missing `.kali/cache/urls/` entry is treated as missing dependency state and should fail with `E5004`
- refreshing or first-time pinning of URL dependencies belongs to `kali install` or another explicit dependency-management workflow, not to ordinary compilation

### Import Maps
Support import maps in `kali.json`:
```json
{
    "schemaVersion": 1,
    "imports": {
        "std/": "https://deno.land/std@0.220.0/",
        "~/": "./src/"
    }
}
```

Interpretation rule:
- `imports` is part of the canonical dependency declaration path for URL-based and path/local alias resolution
- raw URL dependencies discovered through source code or expanded import-map entries participate in the same `kali.lock` + `.kali/cache/urls/` discipline as direct URL specifiers
- registry dependencies still belong under `dependencies` / `devDependencies`; `imports` is not a second registry manifest
- schema v1 import-map targets are therefore limited to relative/absolute path-like rewrites and raw URLs; rewrites to bare package specifiers or canonical registry identifiers such as `jsr:@std/path` are rejected explicitly instead of creating a shadow registry-declaration path

## CommonJS Compatibility

Baseline CommonJS support is part of the Phase 1 package story, but it is intentionally narrow and compile-time-oriented:
- CJS modules (`require`, `module.exports`) are transformed to ESM at compile time
- `require()` calls with static string arguments → ESM import
- Dynamic `require()` is **not** part of the linked-artifact model for Phases 1-3; it is rejected by default, and any later compatibility path must be documented in [specs/19-feature-maturity.md](19-feature-maturity.md) rather than invented ad hoc here
- `__dirname`, `__filename` → transformed to `import.meta.dirname`, `import.meta.filename`

## Dynamic Imports

To keep the module system aligned with the single-artifact architecture and the canonical dynamic-loading boundary in [SPEC.md](../SPEC.md):
- static `import` is the primary and fully supported path
- literal-string `import("pkg")` is a **Phase 3 target** feature that may be rewritten against the already-linked graph rather than introducing runtime WASM module linking
- non-literal `import(expr)` is a **later compatibility** path, treated as a dynamic effect boundary and rejected by default in early phases unless the documented maturity path says otherwise

## Type Resolution

For registry packages, Kali should prefer the strongest sound information available without inventing fresh `any` merely to suppress analysis.

Canonical rule: declaration lookup must follow the **same exact package/subpath edge** chosen by code resolution. Type resolution may consult declaration-specific metadata (`exports` `types`, `types` / `typings`, bundled declarations), but it must not silently type-check one package subpath while runtime resolution executes another.

Type-resolution ladder for a resolved package edge:
1. If the resolved package/subpath publishes declaration-specific `exports` metadata for that exact edge (for example a `types` condition or declaration target associated with the chosen subpath), use it first.
2. Otherwise, for a package-root entry, check the package's own top-level `types` / `typings` field in `package.json`.
3. Apply `typesVersions` if present and relevant to the chosen declaration target.
4. Check for bundled declaration files (`.d.ts`, `.d.mts`, `.d.cts`) alongside the resolved source/entry files for that same package/subpath.
5. Check for `@types/<package>` in dependencies as a fallback when the package does not ship authoritative declarations.
6. If package source is available as JS/TS, run the normal Kali checker/inference pipeline on that package and synthesize module-boundary types from the result.
7. If Kali still cannot justify a precise exported type, fall back to `unknown` at the package boundary with a warning.

Canonical declaration-condition simplification:
- declaration lookup follows the **already chosen code edge**, then refines only within that same subpath/branch for declaration metadata
- if an `exports` object for that exact edge contains a declaration-specific branch, Kali should prefer `types` first, then the active edge-kind branch (`import` or `require`) when it points directly at declarations, then `default`
- API-surface conditions such as `deno`, `browser`, or later `node` are resolved during **code-edge selection** first; declaration lookup should not restart a second independent condition walk that might land on a different subpath
- package-root `types` / `typings` metadata is a fallback only when the resolved edge did not already publish a more specific declaration target

Interpretation rules:
- declaration lookup is **subpath-aware**: package-root metadata must not override a more specific declaration target published for the requested subpath
- bundled package types win over `@types` because they are the package author's authoritative declarations
- `typesVersions` refines selection within the package's own declaration ownership; it does not outrank an exact subpath declaration target
- runtime/code resolution must still ignore declaration-only metadata; the separation is that runtime picks the code edge first, then type resolution finds the matching declaration edge
- explicit `any` from upstream declarations is preserved as authored
- synthesized package-boundary `unknown` follows the same conservative fallback philosophy described in [specs/04-type-system.md](04-type-system.md)
- a later loose-compatibility mode may offer broader `any`-style interop, but that must be an explicit opt-in rather than the default package contract

## Registry

To keep schema v1 small and avoid undocumented config surface area, early-phase registry configuration is intentionally narrow:

- Default npm registry: `https://registry.npmjs.org`
- Early override path for the npm registry: `KALI_REGISTRY` environment variable
- `KALI_REGISTRY` is a transport-endpoint override for **npm** package fetch/metadata workflows only; it may affect `install` and registry-analysis commands that talk to npm, but it does **not** change package identity spelling, the shared stable-release selection rule, lockfile identifier ordering, or the meaning of bare package names as npm identities
- The `jsr:` package namespace keeps using the JSR service; `KALI_REGISTRY` does **not** rewrite `jsr:` package identity into a second configurable registry family
- Per-project registry override fields in `kali.json` are **not** part of schema v1; specs must not imply a config key that the schema does not define
- Private-registry auth/config workflows are a later tooling extension unless/until a schema/CLI revision documents the exact contract
- JSR remains an alternative registry source selected by explicit `jsr:` package identifiers, following the same lock/materialization model as npm packages unless a later phase documents a stronger divergence

## Package Analysis

Independently of project install state for **package identity/version selection**, Kali can analyze a registry package through the **registry-analysis commands**.

Clarification:
- follow the shared **registry-analysis independence split** from [SPEC.md](../SPEC.md)
- this keeps one small two-part rule instead of repeating a longer paragraph in every chapter:

| Question | Early schema-v1 answer |
|---|---|
| Which package/version is analyzed? | The explicit registry target plus the shared **stable-release selection rule (schema v1)**; current project manifest/lock/install state does not pick a different version and the command does not mutate project-managed dependency state |
| Which semantic analysis context is used for `package-effects`? | Built-in defaults plus discovered config through the shared **inherited analysis context**; this may change analysis semantics, but not package target/version selection |

Simplification rule:
- follow the shared **registry-analysis command split** and **workflow-owner split** from [SPEC.md](../SPEC.md) instead of growing a fuzzy “package inspection” surface that mixes effect reporting, policy validation, install behavior, and security audit semantics
- command spelling/examples stay owned by [12 — CLI](12-cli.md); this section focuses on package semantics, version selection, and context behavior

Registry-analysis summary:

| Command | Availability | Context model | JSON success shape |
|---|---|---|---|
| `package-effects` | Phase 2 target | The analysis-context-aware half of the shared **registry-analysis command split**: inherits the shared **inherited analysis context** | Schema-v1 **native-JSON command**; standard command envelope with `--output json` |
| `package-audit` | Later compatibility | The context-free half of the shared **registry-analysis command split**: follows **context-free registry analysis (schema v1)** | Schema-v1 **envelope-only JSON command**; audit findings flow through ordinary diagnostics and the envelope keeps canonical `payload: null` rather than a dedicated success payload. `--pretty` remains meaningful only together with `--output json`; see [specs/18-schemas.md](18-schemas.md)'s **Package Audit JSON Output (schema v1)** section |

Shared target-selection rule:
- both commands follow the shared **registry-analysis target contract (schema v1)** from [SPEC.md](../SPEC.md)
- practical expansion of that shared term here: use one explicit canonical registry package identifier (`lodash`, `@scope/name`, `jsr:@std/path`), resolve versionless CLI targets through the shared **stable-release selection rule (schema v1)**, and keep the analysis project-independent from current manifest/lock/install state
- if that identity-only package lookup finds the package but no acceptable non-yanked stable release exists, the canonical failure path is `E5001`
- promoting a package from "analyzed" to "installed dependency" remains the responsibility of `kali install`
- later `package-effects` may still inherit the shared **inherited analysis context**, but that inherited context is semantic-analysis input only and must not blur the shared target-selection/project-independence rule above
- raw URL dependencies are analyzed through the ordinary project workflow (`kali install` + `kali effects` / `check` / `build`) because their durable declaration source is the source/import-map graph, not a registry package coordinate

Registry-analysis cache rule:
- `package-effects` and `package-audit` may use the shared **registry-analysis cache** from [SPEC.md](../SPEC.md) for fetched metadata/tarballs
- that cache is outside project-managed dependency state and must not mutate `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/`
- cache identity is keyed by at least the canonical registry identifier plus the resolved concrete version
- `package-effects` also keys on the **inherited analysis context** so distinct inherited analysis modes cannot collide accidentally, while early context-free `package-audit` does not add those inherited axes to the cache key

`kali package-effects` remains clearly unavailable until the shared effect-report pipeline lands; it should not return a partial bespoke format before then.

Package-effects rule:
- `kali package-effects <pkg>` should reuse the same effect vocabulary, conservative-upper-bound interpretation, and `dynamicReasons` contract as `kali effects`
- the native payload adds only package-specific metadata (see [specs/18-schemas.md](18-schemas.md)) instead of inventing a second unrelated effect schema
- the nested `report.entryPoints` field should identify the package-analysis logical root with the same canonical registry identifier spelling the user targeted rather than an opaque tarball URL, extracted cache path, or internal package ID
- the nested shared effect report includes `analysisContext` so the chosen `apiSurface`, `runtimeProfiles`, and emitted JSON field `compatFeatures` travel with the report instead of living only in ambient CLI/config state
- in configless project mode, that inherited context is just the **default inherited analysis context (schema v1)** from [SPEC.md](../SPEC.md)
- if that inherited context resolves to `apiSurface = browser`, `package-effects` reuses the same browser-targeted analysis/package-resolution context as other browser analysis commands (including the shared browser `exports` / `package.json#browser` handling) without widening the exact **Phase-1 browser-targeted command set** into an early standalone browser runtime promise
- inherited-context availability follows the shared **axis-aligned inherited analysis gating** rule from [SPEC.md](../SPEC.md); if the inherited context is unavailable, the command fails with `E5006` rather than silently falling back to a smaller one
- representative inherited browser / Node / threaded-profile / compatibility rows stay centralized in [19 — Feature Maturity](19-feature-maturity.md) rather than being re-listed here
- as a schema-v1 **native-JSON command**, `package-effects --pretty` reformats the native payload and `package-effects --output json` wraps that same payload in the standard CLI command envelope; those formatting switches change presentation only and do not create a second availability path or a third package-effects-only outer format

Package-audit rule:
- keep `kali package-audit` **single-package** in early phases so it does not overlap with a future whole-project dependency-health workflow
- following the shared **workflow-owner split** and the context-free half of the shared **registry-analysis command split** from [SPEC.md](../SPEC.md), it is a registry-analysis/security-audit workflow rather than a second host-context-sensitive execution, effect-reporting, or policy-validation command
- early `package-audit` therefore does **not** take package-analysis-specific `--api`, runtime-profile, `--compat`, or `--sandbox` flags
- early `package-audit` follows **context-free registry analysis (schema v1)** from [SPEC.md](../SPEC.md): whether the command runs under discovered config or in configless project mode, inherited host-analysis/runtime config does not gate or rewrite its semantics, and browser-oriented package-resolution context from source-graph commands does not participate here either
- if unimplemented, Kali should say so explicitly instead of implying a partial audit guarantee
- schema v1 intentionally keeps `package-audit` on the simpler **envelope-only JSON command** path; `package-audit --output json` follows the schema-owned **Package Audit JSON Output (schema v1)** rule in [specs/18-schemas.md](18-schemas.md), and plain `package-audit --pretty <pkg>` remains invalid usage because `--pretty` does not enable JSON mode by itself
- under that rule, later audit findings are reported via standard diagnostics (`errors` / `warnings`) rather than a second audit-result payload shape, which keeps registry audit aligned with the normal CLI machine contract
- if a later phase adds richer machine-readable audit details, they should still remain inside that same standard command-envelope path rather than creating a second native bare-JSON audit mode

This integrates with the effect system — know what a dependency does before you use it.
