# Stage 4.1 Status Update

**Date:** 2026-04-17  
**Status:** ✅ Package-audit availability promotion synced; browser-bundle chunk discovery broadened; browser bundles now expose a runtime `loadDynamicImport(specifier)` helper over discovered chunk targets; dynamic-import target resolution now normalizes path-equivalent runtime specifiers and parenthesized static fragments before lookup; `--compat eval` plumbing now rewrites simple statically-resolvable eval sources and rejects eval/Function() without the compat gate; simple `Function()` constructor bodies now rewrite through the same compatibility gate

## Summary

Stage 4.1's package-audit work is now reflected consistently across the CLI, package semantics, maturity matrix, and README. The command remains schema-v1 envelope-only JSON, but it is now documented as the Phase 4 context-free registry-analysis/security-audit command instead of lingering in later-compatibility wording. The source-graph CLI surface has also gained explicit `--compat` plumbing for the shared compatibility-feature vocabulary so `compat.features` requests are parsed and rejected through the canonical availability gate instead of being silently ignored. Separately, browser-bundle chunk discovery now recognizes simple statically-resolvable `import(...)` string-concatenation targets in addition to direct string literals, browser bundle JS now carries a generated runtime lookup map plus `loadDynamicImport(specifier)` for discovered chunk targets, the runtime helper now normalizes path-equivalent specifiers before lookup so `./sub/../lazy.ts` can resolve to the same discovered chunk as `./lazy.ts`, and the resolver now distinguishes statically known dynamic-import targets from unresolved expressions — including const-bound and parenthesized static fragments — rather than leaving them to blend into generic import-resolution failures. The browser-bundle chunk graph now also folds const-bound static `import(...)` fragments before artifact emission, so `import((root + name))` can discover the same linked target as the literal concatenation cases. The `eval` compatibility path now accepts `--compat eval` plus inherited `compat.features = ["eval"]`, rewrites simple statically-resolvable eval strings before codegen, and rejects eval/Function() usage when the compat gate is absent. The same compatibility gate now also handles simple `Function()` constructor bodies that reduce to a statically resolvable `return` expression. The type resolver also now folds const-bound dynamic-import fragments such as `const root = "./"; const name = "lazy.ts"; import(root + name);` so statically known linked targets can be distinguished from truly runtime-resolved specifiers.

## Evidence

- `kali package-audit lodash` succeeds on the default command path ✅
- `kali package-audit --output json lodash` emits the schema-v1 envelope with `payload: null` ✅
- `kali package-audit --pretty lodash` remains invalid without `--output json` ✅
- `check --compat eval` now reaches the Phase-4 compatibility path, and inherited `compat.features = ["eval"]` is accepted as the same effective request ✅
- `check` / `run` reject `eval` and `Function()` without the shared `--compat eval` gate ✅
- browser-bundle chunk discovery now follows simple statically-resolvable `import(...)` concatenations during artifact emission ✅
- browser-bundle chunk discovery now also folds const-bound static `import(...)` fragments before emitting the chunk graph ✅
- dynamic-import resolution now accepts static concatenations, parenthesized fragments, const-bound fragments, and rejects unresolved linked targets with the dedicated `E4008` diagnostic ✅
- resolver tests now cover `const`-bound and parenthesized dynamic-import fragments that fold to known linked targets ✅
- type resolution now distinguishes const-foldable dynamic-import fragments from truly runtime-resolved specifiers ✅
- simple `eval` source strings now rewrite before compilation when the compat flag is present ✅
- simple `Function()` constructor bodies now rewrite before compilation when the compat flag is present ✅
- dynamic eval / Function() sources built from constant program-state fragments now rewrite and execute through the compat path ✅
- `cargo test --workspace` passes ✅
- browser bundle smoke coverage now exercises the generated dynamic-import loader against a discovered chunk target ✅
- browser bundle smoke coverage also exercises a normalized runtime specifier (`./sub/../lazy.ts`) against the same discovered chunk target ✅
- browser bundle smoke coverage now also exercises const-bound static dynamic-import fragments so `import((root + name))` discovers the linked chunk target during artifact emission ✅

## Notable Deliverables

- The `package-audit` CLI path no longer advertises `--preview` in help output, while the compatibility shim remains hidden
- The package-audit availability row in `specs/19-feature-maturity.md` now opens in Phase 4
- The CLI and package semantics docs now read the command as the Phase 4 context-free registry-analysis/security-audit workflow
- Browser-bundle chunk discovery now includes simple statically-resolved dynamic-import concatenations so the chunk graph can pick up more obviously linked targets without waiting for later runtime compatibility
- Browser-bundle JS now carries a generated runtime lookup map plus `loadDynamicImport(specifier)` so discovered chunk bundles can be loaded through the host-side bundle wrapper
- The resolver now treats unknown dynamic-import targets as a distinct `E4008` path rather than leaving them to blend into generic import-resolution failures
- The browser bundle runtime helper now canonicalizes path-equivalent dynamic-import specifiers before looking them up in the bundle-local target map
- The `eval` compat path now accepts the documented feature switch and rewrites simple statically-resolved source strings before the normal compiler pipeline runs
- The `Function()` constructor path now shares that same compatibility gate for simple statically-resolved bodies

## Next Step

Continue the remaining Stage 4.1 dynamic-compatibility work elsewhere as needed.
