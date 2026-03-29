# 08 — WebAssembly Code Generation

## Target

The canonical compilation target for Phases 1-3 is a **single `wasm32` module using the Kali host ABI**.

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

The emitted `.wasm` artifact is portable at the WASM layer, but its full execution contract depends on the Kali host ABI and the feature set required by the chosen profile (API surface + build mode + runtime-profile switches). In practice, Phase 1-3 Kali-hosted execution is standardized on wasmtime, while browser-targeted bundle output relies on generated JS glue to adapt the guest-facing ABI onto the real browser host.

Interface-layer rule:
- core code generation still targets a linked core WASM module first
- WIT descriptions and any later WebAssembly Component Model wrapper are derived from that core artifact and its exported ABI
- these interface-layer artifacts improve embedding/interoperability but do not change the underlying single-linked-payload compilation model

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
- `kali build --lib` omits automatic program start so the host controls instantiation and exported entry calls
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
- in Phase 1, `--bundle` is reserved for browser-targeted output and therefore requires the **effective** `apiSurface` to be `browser` (from CLI or config)
- in early phases, `--lib`, `--capi`, and `--component` are non-browser artifact modes; pairing them with `--api browser` is rejected until a separate browser-library/browser-embedding contract is specified
- `--lib` is the base exported-library mode; `--capi` and `--component` are later packaging layers over that same exported-library contract
- because `--capi` and `--component` already choose exported-library semantics, users should not combine them with `--lib` in early phases; these are separate artifact-mode selectors, not additive modifiers
- WIT sidecars are not a separate artifact mode: Phase 1 plain `--lib` emits the core library `wasm-module`, and relevant library/embedding outputs emit WIT by default once the public interface surface stabilizes in Phase 2+
- unsupported combinations must fail explicitly instead of guessing whether the user wanted an executable bundle, a library artifact, a public embedding artifact set, or a component wrapper

| Command | Output |
|---------|--------|
| `kali build foo.ts` | `foo.wasm` — Kali-hosted WASM module (`kind: wasm-module`, `role: primary-executable`) |
| `kali build --bundle --api browser foo.ts` | `foo.wasm` + `foo.js` — WASM + JS glue for browsers, where the JS file acts as the browser host adapter for the guest ABI (`foo.wasm`: `kind: wasm-module`, `role: primary-executable`; `foo.js`: `kind: js-glue`, `role: browser-glue`) |
| `kali build --bundle foo.ts` | Rejected under the default config; `--bundle` is reserved for browser-targeted output and requires the effective `apiSurface` to be `browser` |
| `kali build --lib foo.ts` | Phase 1: `foo.wasm` — library module (exports, no automatic start; `kind: wasm-module`, `role: primary-library`). Phase 2+: the same base library artifact also emits `foo.wit` (`kind: wit`, `role: interface-wit`) by default once the public interface contract is stabilized. |
| `kali build --lib --api browser foo.ts` | Rejected in early phases; browser mode is an analysis/build context tied to `check` and `build --bundle`, not a library-artifact profile |
| `kali build --capi foo.ts` | Phase 2 target: `foo.wasm` + `foo.wit` + generated embedding header/metadata for use with the host-side `kali_capi` library (`foo.wasm`: `kind: wasm-module`, `role: primary-library`; WIT: `kind: wit`, `role: interface-wit`; header: `kind: c-header`, `role: embedding-header`; metadata: `kind: cabi-metadata`, `role: embedding-metadata`) |
| `kali build --component foo.ts` | Phase 2 target: `foo.wasm` + `foo.wit` + `foo.component.wasm` for a Component Model packaging path (`foo.wasm`: `kind: wasm-module`, `role: primary-library`; `foo.wit`: `kind: wit`, `role: interface-wit`; `foo.component.wasm`: `kind: wasm-component`, `role: primary-component`) |

## Source Maps

When debug/source-map output is requested, Kali may emit WASM source maps (DWARF-based) mapping WASM offsets back to TypeScript/JavaScript source positions for debugging. Artifact metadata exposed through the CLI JSON envelope should describe source maps using the shared artifact schema in [specs/18-schemas.md](18-schemas.md).

Clarification:
- source maps are optional companion debug artifacts, not part of the minimal Phase 1 default artifact contract
- when emitted, they use artifact kind `source-map` and should normally carry role `debug-source-map`
