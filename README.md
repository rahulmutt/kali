# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

Early-phase headline assumptions:
- standalone execution is **Deno-first**
- browser support is **analysis/build first** (`check --api browser`, `build --bundle --api browser`)
- broader Node compatibility is a **later ecosystem phase**, not an implied MVP promise
- latest ECMA-262 grammar tracking does **not** imply blanket same-phase runtime support for every accepted feature
- “latest ECMA-262” means the **latest published edition**; draft / Stage-3+ proposal support is explicit and experimental rather than implied
- dynamic compatibility paths such as `eval` and `Function()` are part of the long-term contract, but remain explicitly phase-gated behind the single schema-v1 compatibility switch `eval`
- runtime/embedding behavior is standardized on **wasmtime first**; alternative WASM engines are a later extension, not an equal Phase-1 contract
- build artifact modes follow one canonical matrix: default executable, browser bundle, a Phase-1 **base library** artifact, and later stable public C embedding / Component Model packages layered on that library contract

## Specification
- Top-level overview, canonical terminology, artifact-mode matrix, and bootstrap-resolution notes: [SPEC.md](./SPEC.md)
- Detailed specs: [`specs/`](./specs)
- Phase/status matrix for gated features and command profiles: [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

## Users
- [Kai](https://github.com/rahulmutt/kai) an AI-based coding assistant.
