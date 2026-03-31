# Stage 1.7 — WASM Code Generation

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/08-wasm-codegen.md`](../../specs/08-wasm-codegen.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)  
**Depends on:** [1.6 — HIR & LIR Lowering](06-hir-lir-lowering.md)

## Goal

Implement `kali_codegen` — translate `LirModule` into a valid, linked WebAssembly binary. After
this stage simple TypeScript/JavaScript programs compile to real WASM modules that can be
validated by any standard WASM validator (e.g. `wasmparser`).

## Workable Milestone

- Simple programs (expressions, arithmetic, function calls, closures) compile to valid WASM
  binaries.
- The emitted binary is validated by `wasmparser` / `wabt` in CI.
- The `kali build <file>` subcommand (executable artifact mode) produces a `.wasm` output file.

## Tasks

### 1. WASM binary encoder

Use a pure-Rust WASM encoder library (e.g. `wasm-encoder`) to produce binary output directly
rather than going through a text format. The encoder should:

- Emit the standard WASM module sections in spec order: Type, Import, Function, Table, Memory,
  Global, Export, Start, Element, Code, Data, DataCount.
- Produce one Type section entry per distinct function signature.
- Map `LirFunction` → WASM function (type index + locals + body).
- Map LIR imports → WASM Import section entries.
- Map LIR exports → WASM Export section entries.
- Emit Data sections for static string / numeric constant pools.

### 2. Instruction selection

Map each `LirInstr` to one or more WASM instructions:

| LIR instruction | WASM translation |
|---|---|
| `Add(i64, i64)` | `i64.add` |
| `Sub(i64, i64)` | `i64.sub` |
| `Mul(i64, i64)` | `i64.mul` |
| `Div(i64, i64)` | `i64.div_s` |
| `Eq(i64, i64)` | `i64.eq` |
| `TagCheck(val, kind)` | `i64.and` + `i64.eq` (compare tag bits) |
| `Untag(val, int)` | `i64.shr_s` (arithmetic shift right by tag width) |
| `Tag(val, int)` | `i64.shl` + `i64.or` |
| `Load(ptr, offset)` | `i64.load offset=N` |
| `Store(ptr, offset, val)` | `i64.store offset=N` |
| `CallDirect(id, args)` | `call $func_idx` |
| `CallIndirect(tbl, ty, args)` | `call_indirect (type $ty_idx)` |
| `CallImport(id, args)` | `call $import_idx` |
| `Branch(cond, then, else)` | `if … end` or `br_if` + `block` |
| `Jump(target)` | `br $label` |
| `Return(val)` | `return` |
| `Unreachable` | `unreachable` |
| `Alloc(size)` | call to Kali runtime allocator import |
| `RcIncref(ptr)` | call to Kali runtime RC increment import |
| `RcDecref(ptr)` | call to Kali runtime RC decrement import |

### 3. Memory layout and runtime ABI

Define the Phase-1 runtime ABI constants that the emitted WASM modules assume:

- **Linear memory**: one WASM memory starting at 64 KiB page 0; page 0 is reserved.
- **Stack pointer**: a WASM global `(global $__stack_ptr (mut i32))` for the shadow stack.
- **`TaggedVal` encoding**: 64-bit value; low 3 bits are the tag:
  - `0b000` — integer (i52 shifted left by 3)
  - `0b001` — float64 (stored as a separate `f64` global or encoded via NaN-boxing)
  - `0b010` — boolean (`0` = false, `1` = true in the upper bits)
  - `0b011` — null
  - `0b100` — undefined
  - `0b101` — pointer to heap object (object/array/function/string)
  - `0b110` — string (short-string optimisation for ASCII strings ≤ 6 bytes, else pointer)
  - `0b111` — reserved
- **Heap object header**: `(ref_count: i32, tag: i32, …payload…)` at every heap-allocated object.
- **Function references**: stored as `i32` indices into a WASM Table (funcref table) for
  indirect calls; closures carry a pointer to a heap-allocated closure record.

Document these ABI constants in `specs/08-wasm-codegen.md` under the Phase-1 runtime ABI section.

### 4. Host import table

Define the set of WASM imports that Kali's runtime (`kali_runtime`) provides to every emitted
module. In Phase 1 these are minimal:

- `kali:rt/alloc(size: i32) -> i32` — allocate `size` bytes, return pointer.
- `kali:rt/rc_incref(ptr: i32)` — increment reference count.
- `kali:rt/rc_decref(ptr: i32)` — decrement reference count; free if zero.
- `kali:rt/panic(msg_ptr: i32, msg_len: i32)` — emit diagnostic and trap.
- `kali:rt/console_log(val: i64)` — print a `TaggedVal` (temporary; replaced by proper API
  host functions in Stage 1.8).

### 5. Build mode flags

Wire the three build modes to codegen:

| Build mode | Codegen behaviour |
|---|---|
| `fast` (default) | No optimisation; emit code directly from LIR without any peephole passes |
| `release` | Apply the set of currently implemented optimisation passes from `kali_optimize` (stub in Phase 1; a no-op pass is acceptable for now) |
| `release-advanced` | Same as `release` in Phase 1; richer optimisation families land in Phase 3 |

The mode names are stable from Phase 1 onward; the optimisation content behind `release` and
`release-advanced` grows without renaming the flags.

### 6. Artifact emission

Implement the first real artifact flow in `kali_cli`:

```
kali build <file>                   # default executable artifact
kali build --fast <file>            # explicit fast build mode
kali build --release <file>         # release build mode
```

Output: `<basename>.wasm` adjacent to the source file, or in the directory specified by `--out-dir`.

Validate the emitted binary with an in-process call to `wasmparser::validate()` before writing it
to disk. If validation fails, emit an `E8xxx` internal diagnostic (codegen bug) rather than
writing a corrupt artifact.

### 7. Binary validation in CI

Add a CI step that:

1. Compiles the fixture programs from earlier stages.
2. Runs `wasmparser::validate()` on each output.
3. Runs `wasm-tools validate` (external) as a second opinion.

### 8. Unit and integration tests

- **Binary validation**: every fixture program compiles to a binary that passes `wasmparser`.
- **Instruction mapping**: unit tests asserting that specific LIR snippets produce the expected
  WASM instruction sequence (compare disassembly via `wasmprinter`).
- **ABI tests**: static assertions that `TaggedVal` encoding round-trips correctly for each tag.
- **CLI integration**: `kali build fixtures/hello.ts` produces a `.wasm` file on disk.

## Out of Scope

- Running the emitted WASM (Stage 1.8).
- Browser bundle output (`--bundle`) (Stage 1.11).
- Library artifact output (`--lib`) (Stage 1.11).
- Optimisation passes beyond the stub no-op (Phase 3 depth).
- MIR-backed layout-aware codegen (Phase 2 target).

## Definition of Done

- [ ] `kali build <file>` produces a `.wasm` file for representative fixtures.
- [ ] Every emitted binary passes `wasmparser::validate()`.
- [ ] Instruction mapping tests pass.
- [ ] CI binary validation step passes on the fixture suite.
- [ ] `cargo test -p kali_codegen` passes.
- [ ] No Stage 1.1–1.6 regressions.
