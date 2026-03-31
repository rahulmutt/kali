# Stage 1.6 — HIR & LIR Lowering

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/05-ir.md`](../../specs/05-ir.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)

## Goal

Implement `kali_hir` (High-level IR) and `kali_lir` (Low-level IR). In Phase 1 the pipeline lowers
`TypedAST → HIR → LIR` directly; MIR (mid-level IR, memory layout + ownership analysis) is a
Phase 2 target. After this stage the complete compile pipeline exists end-to-end, even though the
WASM emitter is still a stub.

## Workable Milestone

- A TypeScript/JavaScript source file can be lowered to LIR without panicking.
- LIR can be printed in a human-readable debug form for pipeline inspection.
- The pipeline is wired into the CLI so future stages only need to plug in the WASM emitter.

## Tasks

### 1. High-level IR (`kali_hir`)

HIR is a *desugared* but still high-level representation that sits between the typed AST and the
WASM-oriented LIR. Its purpose is to eliminate syntactic sugar and make control-flow explicit
while preserving type information.

Key desugaring transformations from TypedAST → HIR:

| Source construct | HIR form |
|---|---|
| `for...of` / `for...in` | Explicit iterator protocol calls |
| `async`/`await` | State-machine transform (coroutine lowering) |
| `function*` / `yield` | Generator state-machine |
| Destructuring patterns | Sequential property accesses + bindings |
| Default parameter values | Guard `if (param === undefined)` |
| Rest parameters | Slice of arguments array |
| Optional chaining `a?.b` | Null-guard + access |
| Nullish coalescing `a ?? b` | `a !== null && a !== undefined ? a : b` |
| Template literals | Concatenation sequence |
| Class body | Constructor function + prototype method assignments |
| `import` / `export` | Module linking instructions (handled by the module linker) |
| `try` / `catch` / `finally` | Explicit landing pad blocks |
| `switch` | Jump table or chain of comparisons |
| Logical `&&` / `||` | Short-circuit branches |

HIR node families:

- **Items**: `HirFunction`, `HirClass`, `HirGlobal` (module-level `let`/`const`).
- **Blocks and control flow**: `HirBlock`, `HirIf`, `HirLoop`, `HirBreak`, `HirContinue`,
  `HirReturn`, `HirThrow`, `HirLandingPad` (try/catch).
- **Expressions**: `HirCall`, `HirMethodCall`, `HirAccess`, `HirIndex`, `HirBinary`,
  `HirUnary`, `HirAssign`, `HirConst` (literal), `HirClosure`, `HirAwait`, `HirYield`.
- **Types**: each HIR node carries a resolved `Ty` from the type checker.

HIR is arena-allocated per-function. Each function gets its own `HirArena` so functions can be
lowered in parallel.

### 2. Low-level IR (`kali_lir`)

LIR is a WASM-oriented, three-address-code-like representation. Its purpose is to be close enough
to WASM that codegen (Stage 1.7) is a straightforward instruction-by-instruction translation.

LIR concepts:

- **Values**: `LirValue` — a typed virtual register. Types at this level are WASM primitives:
  `i32`, `i64`, `f64`, `funcref`, `externref`, and Kali's tagged-value type (`TaggedVal`).
- **`TaggedVal`**: Phase 1 uses a uniform tagged-value representation for JavaScript values that
  can hold any type. The tag (low bits of an `i64`) encodes the runtime type. Specialisation
  that eliminates tags is a Phase 3 optimisation; Phase 1 boxes everything into `TaggedVal` for
  correctness.
- **Instructions**: `LirInstr` — a flat three-address instruction set:
  - Arithmetic / logic: `Add`, `Sub`, `Mul`, `Div`, `Rem`, `BitAnd`, `BitOr`, `BitXor`, `Shl`,
    `Shr`, `Rotl`, `Rotr`.
  - Comparison: `Eq`, `Ne`, `Lt`, `Gt`, `Le`, `Ge`.
  - Memory: `Load(addr, offset)`, `Store(addr, offset, val)`.
  - Control flow: `Branch(cond, then_block, else_block)`, `Jump(block)`, `Return(val)`,
    `Unreachable`.
  - Calls: `CallDirect(func_id, args)`, `CallIndirect(table_idx, type_idx, args)`,
    `CallImport(import_id, args)`.
  - Tag ops: `TagCheck(val, kind)`, `Untag(val, kind)`, `Tag(val, kind)`.
  - GC ops (reference-counted path): `RcIncref(ptr)`, `RcDecref(ptr)`.
  - Allocation: `Alloc(size) -> ptr`, `AllocArray(len) -> ptr`.
- **Basic blocks**: each `LirFunction` is a list of `LirBlock`s in SSA-like form; each block
  ends with a terminator (`Branch`, `Jump`, `Return`, `Unreachable`).
- **Module**: `LirModule` holds all `LirFunction`s plus import/export tables, data segments,
  and memory declarations.

### 3. HIR → LIR lowering

Walk each `HirFunction` and emit LIR:

- Allocate a virtual register for each HIR value.
- Lower HIR control-flow nodes to basic blocks with explicit terminators.
- Lower `HirCall` to `CallDirect` (for statically known callees) or `CallIndirect` (for
  function-typed values).
- Lower property accesses to `Load` / `Store` with computed offsets into object layout.
  In Phase 1 all object layouts are uniform (all fields are `TaggedVal`-sized slots);
  layout-aware lowering is a Phase 2/3 target.
- Emit `Tag` / `Untag` / `TagCheck` instructions at type-dispatch boundaries.
- Emit `RcIncref` / `RcDecref` at ownership transfer points where the escape analysis (Phase 2)
  would later compute lifetimes; in Phase 1 use conservative reference counting.

### 4. LIR pretty-printer

Implement a human-readable text format for LIR (`kali lir-dump <file>` as a hidden dev subcommand):

```
function main():
  block 0:
    %0 = Tag(42i64, int)
    %1 = CallDirect(console_log, [%0])
    Return(%1)
```

This is essential for pipeline debugging throughout the rest of Phase 1.

### 5. Module linker skeleton

Introduce `LirModule::link(modules: Vec<LirModule>) -> LirModule` which merges multiple per-file
`LirModule`s into one linked payload. In Phase 1 this is a simple concatenation + symbol
resolution pass. The full single-linked-WASM-payload guarantee from `specs/01-architecture.md`
is enforced here: all statically imported modules are merged before codegen.

### 6. Parallel lowering

Because HIR lowering is per-function and LIR is per-module, wire `rayon::par_iter()` over the
function list so lowering is parallel. The `LirModule` collection step is sequential.

### 7. Tests

- **Snapshot tests**: lower representative HIR fixtures; assert the LIR dump matches the golden
  snapshot.
- **Round-trip tests**: lower HIR → LIR → pretty-print → parse pretty-print → assert structural
  equality (ensures the printer is faithful).
- **Control-flow tests**: one fixture per complex HIR desugaring case (async/await state machine,
  generator, try/catch landing pad, optional chain).

## Out of Scope

- WASM binary emission (Stage 1.7).
- MIR / ownership / escape analysis (Phase 2 target); Phase 1 uses conservative reference
  counting as a placeholder.
- Optimisation passes (`kali_optimize` stub only; optimisation is Phase 3 depth).
- `TaggedVal` specialisation / unboxing (Phase 3 target).

## Definition of Done

- [ ] TypedAST → HIR → LIR pipeline completes on representative TS/JS fixture programs.
- [ ] LIR pretty-printer produces readable output for every fixture.
- [ ] Module linker merges multi-file programs into one `LirModule`.
- [ ] Snapshot and round-trip tests pass under `cargo test -p kali_hir -p kali_lir`.
- [ ] Parallel lowering is wired and produces identical results to sequential lowering.
- [ ] No Stage 1.1–1.5 regressions.
