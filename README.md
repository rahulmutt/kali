# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

Early-phase headline assumptions:
- standalone execution is **Deno-first**
- browser support is **analysis/build first** (`check --api browser`, `build --bundle --api browser`)
- broader Node compatibility is a **later ecosystem phase**, not an implied MVP promise
- build artifact modes follow one canonical matrix: default executable, browser bundle, library, later C embedding package, and later Component Model package

## Specification
- Top-level overview, canonical terminology, and artifact-mode matrix: [SPEC.md](./SPEC.md)
- Detailed specs: [`specs/`](./specs)
- Phase/status matrix for gated features and command profiles: [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

## Users
- [Kai](https://github.com/rahulmutt/kai) an AI-based coding assistant.
