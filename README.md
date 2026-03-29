# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

Early-phase headline assumptions:
- standalone execution is **Deno-first**
- browser support is **analysis/build first** (`check --api browser`, `build --bundle --api browser`)
- broader Node compatibility is a **later ecosystem phase**, not an implied MVP promise
- latest ECMA-262 grammar tracking does **not** imply blanket same-phase runtime support for every accepted feature
- dynamic compatibility paths such as `eval` are part of the long-term contract, but remain explicitly phase-gated
- build artifact modes follow one canonical matrix: default executable, browser bundle, library, later C embedding package, and later Component Model package

## Specification
- Top-level overview, canonical terminology, artifact-mode matrix, and bootstrap-resolution notes: [SPEC.md](./SPEC.md)
- Detailed specs: [`specs/`](./specs)
- Phase/status matrix for gated features and command profiles: [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

## Users
- [Kai](https://github.com/rahulmutt/kai) an AI-based coding assistant.
