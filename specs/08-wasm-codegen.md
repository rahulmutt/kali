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
- plain public `--lib` + WIT is the canonical stable interface-layer contract once the Phase-2 public embedding surface lands
- `--capi` headers/metadata and any later WebAssembly Component Model wrapper are projections/packaging flows over that same exported interface rather than separate guest semantics
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
- `kali build --lib` follows the shared **library-oriented instantiation rule** from [SPEC.md](../SPEC.md): no synthetic executable entry invocation is added, normal ECMAScript module-instantiation semantics still apply, and top-level module initialization still runs when the host instantiates the artifact
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
- `eval` handler hook (Phase 4 compatibility path; calls back to a host-mediated generic execution path when enabled, without implying a second runtime compilation/JIT pipeline)

## WASM Binary Emission

Direct binary emission without intermediate text format:
- Custom WASM encoder in `kali_codegen` for maximum control over output
- Emit sections in order: type, import, function, table, memory, global, export, start, element, data count, code, data
- LEB128 encoding for all integers
- Validate output with `wasmparser` crate (pure Rust) in debug builds and via `--validate-ir` flag

## Output Artifacts

Use the canonical artifact kinds from [specs/18-schemas.md](18-schemas.md) in CLI JSON output and embedding metadata.

Artifact-mode selection itself is owned by the canonical matrix in [SPEC.md](../SPEC.md) and the command-shape rules in [12 — CLI](12-cli.md). This chapter stays narrower: it describes the emitted artifact **shapes** once a build mode is already valid, instead of restating a second near-duplicate command matrix.

Shared artifact-shape rules:
- omitting `--bundle`, `--lib`, `--capi`, and `--component` yields the default executable artifact: one `wasm-module` with role `primary-executable`
- `--bundle` under an effective `apiSurface` of `browser` keeps executable compile intent and adds browser JS glue (`kind: js-glue`, `role: browser-glue`) beside that same executable core module
- that browser-bundle artifact shape is selected by the fully merged effective context, so explicit `--api browser` and equivalent inherited-config browser forms emit the same artifact set
- `--lib`, `--capi`, and `--component` are the library-oriented artifact modes: they all reuse the same **statically known export surface** and the shared **library-oriented instantiation rule** from [SPEC.md](../SPEC.md)
- plain Phase-1 `--lib` emits only the **base library artifact** (`kind: wasm-module`, `role: primary-library`); Phase 2 promotes that same selector into the stable public library/WIT contract and adds `kind: wit`, `role: interface-wit` by default
- `--capi` and `--component` are later **public embedding artifact flows** layered on top of that same linked core library payload rather than separate export semantics
- `--component` adds an outer `wasm-component` wrapper around the same linked core payload; it does not authorize a second independently linked guest graph
- if Kali cannot determine the required export surface statically for a library-oriented build, fail with `E5011`

Illustrative artifact sets by valid build mode *(reading aid only; filenames are basename-derived examples, while the normative machine contract is the emitted artifact list's `kind` + `role` metadata plus the availability/gating rules in [12 — CLI](12-cli.md), [18 — Schemas](18-schemas.md), and [19 — Feature Maturity](19-feature-maturity.md))*:

| Valid build mode | Illustrative emitted artifacts |
|---|---|
| default executable build (`kali build foo.ts`) | `foo.wasm` (`kind: wasm-module`, `role: primary-executable`) |
| browser bundle (`kali build --bundle foo.ts` when the effective `apiSurface` is `browser`) | `foo.wasm` (`kind: wasm-module`, `role: primary-executable`) + `foo.js` (`kind: js-glue`, `role: browser-glue`) |
| base library build (`kali build --lib lib.ts`) | Phase 1: basename-derived `lib.wasm` (`kind: wasm-module`, `role: primary-library`). From the Phase 2 target onward, add basename-derived `lib.wit` (`kind: wit`, `role: interface-wit`) by default once the public library/WIT contract is stable. |
| C-ABI embedding build (`kali build --capi lib.ts`) | Basename-derived `lib.wasm` (`kind: wasm-module`, `role: primary-library`) + `lib.wit` (`kind: wit`, `role: interface-wit`) + generated `lib.exports.h` (`kind: c-header`, `role: embedding-header`) + generated `lib.cabi.json` (`kind: cabi-metadata`, `role: embedding-metadata`). The generated exports header is distinct from the stable host ABI header `kali.h`. |
| Component Model build (`kali build --component lib.ts`) | Basename-derived `lib.wasm` (`kind: wasm-module`, `role: primary-library`) + `lib.wit` (`kind: wit`, `role: interface-wit`) + `lib.component.wasm` (`kind: wasm-component`, `role: primary-component`) |

For invalid or unavailable combinations such as `--bundle` without browser mode, browser + library-oriented modes, or early `--api node`, follow the canonical validation/gating rules in [SPEC.md](../SPEC.md), [12 — CLI](12-cli.md), and [19 — Feature Maturity](19-feature-maturity.md) instead of reading this table as a second normative artifact-mode matrix.

## Source Maps

When debug/source-map output is requested, Kali may emit WASM source maps (DWARF-based) mapping WASM offsets back to TypeScript/JavaScript source positions for debugging. Artifact metadata exposed through the CLI JSON envelope should describe source maps using the shared artifact schema in [specs/18-schemas.md](18-schemas.md).

Clarification:
- source maps are optional companion debug artifacts, not part of the minimal Phase 1 default artifact contract
- when emitted, they use artifact kind `source-map` and should normally carry role `debug-source-map`
