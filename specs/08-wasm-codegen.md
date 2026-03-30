# 08 — WebAssembly Code Generation

## Target

The canonical compilation target for Kali in Phases 1-3 is a **single `wasm32` module using the Kali host ABI**.

- **Required baseline**:
  - WASM MVP
  - 32-bit linear memory (`wasm32`)
  - Kali host ABI support for runtime services plus the subset of host imports selected by the active API surface/runtime profile; emitted modules must not imply that process/network/file imports always exist in every build
- **Optional WASM extensions** used when available and enabled by the selected target/profile:
  - Multi-value returns
  - Bulk memory operations
  - Reference types (for carefully bounded host interop)
  - Tail calls
  - Exception handling (for try/catch)
  - Threads (later compatibility only, for the separate `--wasm-threads` runtime profile used by `SharedArrayBuffer` / `Atomics`)
  - SIMD (for typed array optimizations)

The emitted `.wasm` artifact is portable at the WASM layer, but its full execution contract depends on the Kali host ABI and the feature set required by the chosen profile (API surface + build mode + runtime-profile switches). In practice, Kali-hosted execution in Phases 1-3 is standardized on wasmtime via the native host adapter, while browser-targeted bundle output relies on the browser host adapter (generated JS glue) to adapt the same guest-facing ABI onto the real browser host.

Interface-layer rule:
- core code generation still targets a linked core WASM module first
- WIT descriptions and any later WebAssembly Component Model wrapper are derived from that core artifact and its exported ABI
- these interface-layer artifacts improve embedding/interoperability but do not change the underlying single-linked-payload compilation model

## Artifact Reproducibility Contract

To keep AOT builds auditable and automation-friendly, code generation is reproducible by default:
- the same source graph, lockfile, effective command context, and Kali version/toolchain should produce byte-stable `.wasm` output and companion artifact contents
- artifact bytes must not depend on wall-clock timestamps, randomized symbol names, hash-map iteration order, or host-specific absolute paths unless the user explicitly opts into such metadata
- if debug metadata or source maps need filesystem paths, project-relative paths are the default contract; embedding raw absolute host paths is opt-in only
- custom sections, symbol tables, and emitted artifact lists should use deterministic ordering when the producer owns that order

This rule keeps build artifacts aligned with the JSON determinism rules in [specs/18-schemas.md](18-schemas.md) and the top-level reproducibility goal in [SPEC.md](../SPEC.md).

## Code Generation from LIR

### Function Emission
Each LIR function maps to one WASM function:
```
LIR Function → WASM function with:
  - Local declarations (typed)
  - Body instructions
  - Type signature in the type section
```

### Control Flow
- LIR blocks → WASM `block`/`loop`/`if` structured control flow
- Break/continue → `br`/`br_if` to label indices
- The LIR is already structured (no arbitrary goto), so mapping is direct
- Switch statements → `br_table`

### Memory Access
- Static object fields → `i32.load`/`i64.load`/`f64.load` at known offsets
- Array elements → base + (index × element_size), with bounds check
- Dynamic properties → call to runtime hash map lookup
- Stack allocations → stack pointer manipulation in linear memory

### Function Calls
- Direct calls (known target) → `call` instruction
- Indirect calls (closures, virtual dispatch) → `call_indirect` via function table
- Imported host functions (I/O, APIs) → WASM imports

### String Representation
Strings in linear memory:
```
┌────────┬────────┬──────────────┐
│ len:u32│flags:u8│ data: [u8]   │  (UTF-8 encoded)
└────────┴────────┴──────────────┘
```
- Immutable by default (copy-on-write for `String` operations)
- Small strings (≤ 23 bytes) can be inline (no heap pointer)
- String interning for constants at compile time

### Number Representation
- All JS numbers use `f64` semantics by default
- The compiler may lower hot integer-only regions to `i32` / `i64` internally when it can prove equivalence or insert overflow/semantic guards that preserve JavaScript-visible behavior
- Typed-array element loads/stores may use narrower machine types (`i8`/`i16`/`i32`/`f32`) at the memory boundary while values re-enter normal JS semantics through the appropriate coercions
- BigInt → runtime arbitrary-precision integer (in linear memory)

## Module Structure

```
WASM Module:
  Types:     Function signatures
  Imports:   Host functions (I/O, APIs), memory (optional)
  Functions: Compiled user code + runtime support
  Table:     Function table for indirect calls
  Memory:    Linear memory (initial + max pages)
  Globals:   Module-level state, stack pointer
  Exports:   Entry point, public API functions
  Data:      Static data segments (strings, constants)
  Start:     Optional module initialization function
```

`Start` is artifact-mode dependent:
- the default executable artifact path may emit a start/init path for module setup before invoking the entrypoint
- `kali build --lib` omits any **synthetic executable entry invocation** so the host controls instantiation and exported entry calls; this does **not** suspend ordinary ECMAScript module-instantiation semantics, so top-level module initialization still runs when the host instantiates the library artifact
- browser bundles may route initialization through generated JS glue instead of relying solely on the raw WASM start section

## Runtime Support Functions

Compiled into the WASM module from `kali_runtime`:
- Memory allocator (malloc/free equivalent)
- Reference counting (rc_inc, rc_dec, rc_free)
- String operations (concat, slice, compare, regex)
- Dynamic property access (hash map get/set/delete)
- Type coercion functions (ToNumber, ToString, ToBoolean per spec)
- Error creation and stack trace capture
- Iterator protocol support
- Promise/async state machine support
- `eval` handler hook (Phase 4 compatibility path; calls back to host for compilation when enabled)

## WASM Binary Emission

Direct binary emission without intermediate text format:
- Custom WASM encoder in `kali_codegen` for maximum control over output
- Emit sections in order: type, import, function, table, memory, global, export, start, element, data count, code, data
- LEB128 encoding for all integers
- Validate output with `wasmparser` crate (pure Rust) in debug builds and via `--validate-ir` flag

## Output Artifacts

Use the canonical artifact kinds from [specs/18-schemas.md](18-schemas.md) in CLI JSON output and embedding metadata.

Artifact-mode selection follows the canonical matrix in [SPEC.md](../SPEC.md). This chapter focuses on the emitted artifact shapes, not on redefining a second build-mode taxonomy.

Early-phase artifact-mode rule:
- `--bundle`, `--lib`, `--capi`, and `--component` are mutually exclusive build artifact-mode selectors unless a later spec explicitly defines one as implying another
- omitting all four selects the default executable artifact mode (`kali build foo.ts` → one executable-style `wasm-module` artifact)
- in Phase 1, `--bundle` is reserved for browser-targeted output and therefore requires the **effective API surface** to be `browser`; using `--bundle` under an effective API surface of `deno` or `node` is invalid command usage (`E5008`), not a separate feature-maturity rejection
- in early phases, `--lib`, `--capi`, and `--component` are **library-oriented artifact modes**: non-browser, export-oriented modes derived from a **statically known export surface** as defined in [SPEC.md](../SPEC.md)
- those library-oriented modes still obey the ordinary build-command API-surface gates: pairing them with `--api browser` is an `E5008` contradiction because browser mode is only defined for `--bundle`, while pairing them with `--api node` remains on the same Phase 3 `E5006` path as other early Node-targeted builds
- `--lib` is the base exported-library mode, and `--capi` / `--component` are later packaging layers over that same exported-library contract
- if Kali cannot prove that statically known export surface, the library-oriented build must fail with `E5006`
- this Phase-1 `--lib` artifact is intentionally narrower than the later stable public embedding surface: it defines the exported-library shape early without implying that WIT, C headers, or the long-term embedding ABI are already frozen
- `--component` adds a wrapper around the same linked core library payload; it does not authorize a second independently linked guest-program graph and therefore does not weaken the single-linked-core-payload rule
- because `--capi` and `--component` already choose exported-library semantics, users should not combine them with `--lib` in early phases; these are separate artifact-mode selectors, not additive modifiers
- WIT sidecars are not a separate artifact mode: Phase 1 plain `--lib` emits the core library `wasm-module`, and relevant library/embedding outputs emit WIT by default once the public interface surface stabilizes in Phase 2+
- unsupported combinations must fail explicitly instead of guessing whether the user wanted an executable bundle, a library artifact, a public embedding artifact set, or a component wrapper

| Command | Output |
|---------|--------|
| `kali build foo.ts` | `foo.wasm` — Kali-hosted WASM module (`kind: wasm-module`, `role: primary-executable`) |
| `kali build --bundle --api browser foo.ts` | `foo.wasm` + `foo.js` — WASM + JS glue for browsers, where the JS file acts as the browser host adapter for the guest ABI (`foo.wasm`: `kind: wasm-module`, `role: primary-executable`; `foo.js`: `kind: js-glue`, `role: browser-glue`) |
| `kali build --bundle foo.ts` | Invalid command usage (`E5008`) under the default config; `--bundle` is reserved for browser-targeted output and requires the effective API surface to be `browser` |
| `kali build --bundle --api node foo.ts` | Invalid command usage (`E5008`); browser bundle mode exists, but pairing it with a non-browser API surface is a contradictory command shape |
| `kali build --lib foo.ts` | Phase 1: `foo.wasm` — export-oriented **base library** module (no synthetic executable entry invocation; ordinary top-level module initialization still occurs when instantiated; `kind: wasm-module`, `role: primary-library`). This is the early exported-library artifact shape, not yet the full stable public embedding contract. Phase 2+: the same base library artifact also emits `foo.wit` (`kind: wit`, `role: interface-wit`) by default once the public interface contract is stabilized. |
| `kali build --lib --api node foo.ts` | Phase 3 target: library-oriented builds still obey the ordinary Node build gate rather than inventing a separate library-only Node surface |
| `kali build --lib --api browser foo.ts` | Invalid command usage (`E5008`) in early phases; browser mode is a browser-targeted context tied to `check` and `build --bundle`, not a library-artifact mode |
| `kali build --capi foo.ts` | Phase 2 target: `foo.wasm` + `foo.wit` + generated embedding header/metadata for use with the host-side `kali_capi` library (`foo.wasm`: `kind: wasm-module`, `role: primary-library`; WIT: `kind: wit`, `role: interface-wit`; header: `kind: c-header`, `role: embedding-header`; metadata: `kind: cabi-metadata`, `role: embedding-metadata`) |
| `kali build --capi --api node foo.ts` | Phase 3 target: still gated by the Node build surface even after public embedding artifacts exist |
| `kali build --capi --api browser foo.ts` | Invalid command usage (`E5008`) in early phases; browser mode is a browser-targeted context tied to `check` and `build --bundle`, not a browser-embedding artifact mode |
| `kali build --component foo.ts` | Phase 2 target: `foo.wasm` + `foo.wit` + `foo.component.wasm` for a Component Model packaging path (`foo.wasm`: `kind: wasm-module`, `role: primary-library`; `foo.wit`: `kind: wit`, `role: interface-wit`; `foo.component.wasm`: `kind: wasm-component`, `role: primary-component`). The outer `.component.wasm` is a packaging wrapper around the same linked core payload, not a separately linked second program. |
| `kali build --component --api node foo.ts` | Phase 3 target: still gated by the Node build surface even after component packaging exists |
| `kali build --component --api browser foo.ts` | Invalid command usage (`E5008`) in early phases; browser mode is a browser-targeted context tied to `check` and `build --bundle`, not a browser-component artifact mode |

## Source Maps

When debug/source-map output is requested, Kali may emit WASM source maps (DWARF-based) mapping WASM offsets back to TypeScript/JavaScript source positions for debugging. Artifact metadata exposed through the CLI JSON envelope should describe source maps using the shared artifact schema in [specs/18-schemas.md](18-schemas.md).

Clarification:
- source maps are optional companion debug artifacts, not part of the minimal Phase 1 default artifact contract
- when emitted, they use artifact kind `source-map` and should normally carry role `debug-source-map`
