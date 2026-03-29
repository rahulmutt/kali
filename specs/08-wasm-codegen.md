# 08 — WebAssembly Code Generation

## Target

- **WASM MVP** as baseline (maximum compatibility)
- **WASM extensions** used when available:
  - Multi-value returns
  - Bulk memory operations
  - Reference types (for GC-free externref passing)
  - Tail calls
  - Exception handling (for try/catch)
  - Threads (for SharedArrayBuffer, Atomics)
  - SIMD (for typed array optimizations)

Emit a `.wasm` binary module that can run in any conformant runtime.

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
- All JS numbers → WASM `f64` by default
- When provably integer (through type analysis) → WASM `i32` or `i64`
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
  Start:     Module initialization function
```

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
- `eval` handler (calls back to host for compilation)

## WASM Binary Emission

Direct binary emission without intermediate text format:
- Custom WASM encoder in `kali_codegen` for maximum control over output
- Emit sections in order: type, import, function, table, memory, global, export, start, element, data count, code, data
- LEB128 encoding for all integers
- Validate output with `wasmparser` crate (pure Rust) in debug builds and via `--validate-ir` flag

## Output Artifacts

| Command | Output |
|---------|--------|
| `kali build foo.ts` | `foo.wasm` — standalone WASM module |
| `kali build --bundle foo.ts` | `foo.wasm` + `foo.js` — WASM + JS glue for browsers |
| `kali build --lib foo.ts` | `foo.wasm` — library module (exports, no start) |

## Source Maps

Generate WASM source maps (DWARF-based) mapping WASM offsets back to TypeScript/JavaScript source positions for debugging.
