# Design — fannkuch-redux End-to-End (First Real Imperative-Core Execution Slice)

Date: 2026-06-30
Status: Proposed (brainstorming output; awaiting user review before plan)

## 1. Problem and motivation

Kali's specs name "adapted Computer Language Benchmarks Game (CLBG) workloads" as a
Phase-1 optimization/performance evidence lane (`specs/19-feature-maturity.md:215`,
`specs/16-testing.md:44,58`, `plan/phase-24/README.md` §24.4). That lane is **named but
empty**: the repository contains no CLBG program, only ~124 tiny *optimizer* microbenchmark
fixtures under `crates/kali_cli/tests/fixtures/benchmarks/` whose harness
(`crates/kali_cli/tests/runtime_smoke.rs:5928`) merely **compiles** a fixture in three build
modes and measures wasm size / instruction counts — it never runs a program or checks output.

The deeper reason no CLBG program exists is that **Kali cannot yet execute general dynamic
code**. Empirically, through `kali run`:

| Program | Expected | Kali today |
|---|---|---|
| `console.log(40 + 2)` | `42` | `42` (constant-folded) |
| `const a=[10,20,30]; console.log(a[0]+a[1]+a[2])` | `60` | `60` (static fold) |
| `let x = 5; console.log(x)` | `5` | **`0`** |
| `let x=5; x=x+1; console.log(x)` | `6` | **`0`** |
| `function add(a,b){return a+b} console.log(add(40,2))` | `42` | **`0`** |
| `for(let i=0;i<5;i++) s+=i; console.log(s)` | `10` | **`0`** |
| recursion (`fib`, prefix-sum) | — | **runtime trap (infinite)** |

Architecturally (confirmed by reading codegen/runtime):

- **No loops exist.** There is no `wasm` `loop`/back-edge anywhere; `for`/`while`/`do-while`
  lower to a one-shot `if` (`crates/kali_codegen/src/emit/control_flow.rs:340-404`) and
  silently do not iterate (a latent silent-miscompile, since the checker does not gate them).
- **i64-only value model.** No f64. Binary ops emit `I64*`
  (`crates/kali_codegen/src/emit/operators.rs:397-432`).
- **No heap / allocator.** Array literals emit the constant `0`
  (`crates/kali_codegen/src/emit/literal.rs:22-36`); `push`/`pop` unimplemented; TypedArrays
  are not a language feature.
- **General locals / params / returns do not compute** — user functions return `0` and
  recursion fails to terminate.
- The **only** correct execution path is **compile-time constant folding** (`const` bindings,
  literal arithmetic, static array/string folds). The ~249 "end-to-end run" integration tests
  exercise only constant-foldable programs.

So "make a CLBG benchmark work" is not a few missing intrinsics — it is **building the
imperative-core execution backend**, using one CLBG program as the acceptance test.

## 2. Goal and non-goals

**Goal.** Make `kali run` execute `fannkuch-redux` correctly end-to-end and produce the exact
canonical CLBG output for a pinned input `n`, by implementing genuine (not pattern-matched)
lowering for the imperative core: real loops, working mutable locals, working function
calls/params/returns/recursion, linear-memory integer arrays, and runtime integer→string +
string concatenation for output.

**Why fannkuch-redux.** It is the CLBG program with the smallest feature surface: **integers
only**. It needs no floating point (rules out n-body, mandelbrot, spectral-norm), no heap
objects/GC (binary-trees), no stdin/hashmaps (k-nucleotide, reverse-complement, fasta), no
arbitrary-precision bignum (pidigits), no regex (regex-redux). It needs exactly the imperative
core plus a fixed mutable integer array. Building it first establishes the foundation every
later benchmark reuses.

**Non-goals (explicitly deferred to later slices):**

- Floating-point (f64) arithmetic and `Math.*` on computed values.
- Dynamic object/class model, hash maps, growable arrays beyond what fannkuch needs.
- Generic runtime strings beyond integer→decimal and concatenation (no `substring`, `split`,
  regex, non-ASCII).
- stdin / raw byte stdout; output stays line-oriented `console.log`.
- A general `Array.prototype` (`push`/`pop`/`map`/…); only fixed-size index read/write +
  `.length` + `new Array(n)`/literal init are in scope.
- Performance claims. This slice proves **correctness of execution**, not throughput. Any
  benchmark-promotion wording stays out of scope (per `plan/phase-24/README.md` §24.4).

## 3. Target program

A vendored, idiomatic TS port of the published Node.js / JavaScript fannkuch-redux submission,
normalized to Kali's pipeline (per `specs/16-testing.md:44`): retains upstream CLBG
attribution, respects CLBG license terms, no benchmark-specific intrinsic tuning. The port uses
only in-scope features: `let`/`const`, integer arithmetic, `<`/`>`/`===`/`!==`, `if`, `while`
(or `for`), `function` declarations + calls, a fixed-size integer array (`new Array(n)` filled
0..n-1) with indexed read/write and prefix reversal, and final output:

```
<checksum>
Pfannkuchen(<n>) = <maxFlipsCount>
```

`n` is a compile-time integer literal in the fixture (proposed `n = 7`). Expected output is
captured from a reference Node run and pinned in the test.

## 4. Architecture of the slice

All work flows through the existing pipeline AST → HIR → MIR → LIR → wasm
(`crates/kali_{ast,hir,mir,lir,codegen}`), executed by the wasmtime host
(`crates/kali_runtime`). The guiding principle: **replace the "constant-fold-or-emit-0"
fallback with genuine type-directed lowering for the in-scope constructs.** Constant folding
remains available as an optimization where it already applies (observable outputs of constant
programs must not change), but correctness no longer depends on a value being statically known.

### 4.1 Value representation (unchanged tag scheme, extended by static types)

- A JS value is one `i64`.
- **Integer:** raw two's-complement `i64`, low 63 bits (bit 63 = 0).
- **String handle:** `STRING_HANDLE_TAG (bit 63) | (offset << 32) | len`, pointing into the
  exported `memory` (`crates/kali_codegen/src/lib.rs:62`, decoded by
  `crates/kali_runtime/src/host/memory.rs`).
- **Array handle (new):** a raw byte offset into `memory` (a small positive integer, bit 63 = 0).
  It is **not** separately tagged; lowering is **type-directed** — the type system
  (`kali_types`) already knows whether a value is `number`, `string`, or array at each use
  site, so codegen never needs to inspect an array value at runtime. fannkuch never prints an
  array, so no array formatting in the host is required.

This means **no value-model rewrite** — integers and strings keep their current encoding, and
arrays slot in as untagged memory offsets disambiguated statically.

### 4.2 Linear-memory layout and the bump allocator

Current memory: `[0, 4096)` reserved scratch (`ENV_GET_BUFFER_RESERVED`), then the interned
static string pool from `4096` upward (`crates/kali_codegen/src/ctx.rs`). We add a **runtime
heap** region above all static string data:

- A reserved global (or a fixed compile-time base just past `StringPool::next_offset`, rounded
  up) holds the **bump pointer**.
- `kali_alloc(nbytes) -> offset`: returns the current bump pointer and advances it (8-byte
  aligned). **No free** — allocations live for the whole program. This is consistent with the
  no-tracing-GC rule (`specs/19-feature-maturity.md:205`): bump allocation is not GC.
- Initial `memory` size is set large enough for the pinned workload (fannkuch n=7 arrays and
  the tiny output strings need only a few KiB); `memory.grow` is **out of scope** for this
  slice (fixed initial pages, with a generous default).

### 4.3 The five build pieces

Each piece is independently testable with a micro-program before the full benchmark.

**A. Real loops.** Lower `while` (and `for`, desugared to init + `while` + update) to a `wasm`
`block { loop { <cond br_if out>; <body>; <update>; br loop } }`. Replace the one-shot-`if`
lowering in `control_flow.rs`. Honor `break`/`continue` via the surrounding `block`/`loop`
labels. Micro-acceptance: `let s=0; for(let i=0;i<5;i=i+1){s=s+i} console.log(s)` → `10`.

**B. Mutable locals.** General `let` declaration → a wasm local; reads → `local.get`; assignments
and compound assignments (`+=` etc.) → evaluate RHS then `local.set`. This must work on the
ordinary (non-folded) path, not only the existing narrow "supported mutable-local slice."
Micro-acceptance: `let x=5; x=x+1; console.log(x)` → `6`.

**C. Function calls / params / returns / recursion.** Bind each parameter to a function-body
local; lower `return <expr>` to evaluate then `wasm` `return`; lower call sites to evaluate args
left-to-right and `call` the callee, leaving the i64 result on the stack so it reaches the
caller / `console.log`. Fix whatever currently makes results read as `0` and recursion
non-terminating (likely param-binding / return-value propagation; pinned precisely during
implementation via systematic debugging). Micro-acceptance: `add(40,2)` → `42`; bounded
recursion prefix-sum `s(5)` → `15`.

**D. Linear-memory integer arrays.** `new Array(n)` / integer array literal →
`kali_alloc((1+n)*8)`, store `n` at the base (length header), elements after it; `a[i]` →
load `base + 8 + i*8`; `a[i] = v` → store; `a.length` → load base. Bounds checks may trap (host
surfaces `E4000`); fannkuch indices are in range. Micro-acceptance:
`const a=new Array(3); a[0]=10; a[1]=20; a[2]=a[0]+a[1]; console.log(a[2])` → `30`.

**E. Runtime integer→string + concatenation.** `kali_itoa(i64) -> string handle`: format the
integer's decimal digits into a freshly `kali_alloc`-ed buffer and return a tagged string
handle. String `+` where either side is a string: allocate a buffer, copy both operands' bytes
(operands may be static-interned or runtime-built), return a new handle. Template-literal
interpolation of an integer lowers through the same path. This is the minimum needed for the
exact `Pfannkuchen(n) = <maxflips>` line. Micro-acceptance:
`let n=7; console.log("Pfannkuchen(" + n + ") = " + 16)` → `Pfannkuchen(7) = 16`.

### 4.4 Output

`console.log` already prints either a decimal integer or a memory-resident string (host
`format_console_value`, `crates/kali_runtime/src/host/io.rs:22`). Piece E lets the program build
the exact canonical second line; the checksum prints as a plain integer.

## 5. Acceptance criteria

1. `kali run <fannkuch fixture>` prints exactly the two canonical lines for the pinned `n`,
   byte-matching a reference Node.js run (captured and asserted in the test).
2. Each of pieces A–E has its own passing micro-acceptance run-test (above).
3. The benchmark fixture ships with schema-v1 metadata (`schemas/benchmark/v1.json`):
   `sourceFile`, `sourceSha256` (validated), `buildModes`, plus CLBG attribution, and the
   existing `assert_optimization_benchmark_fixture` compile-in-three-modes path passes for it.
4. `cargo test --workspace` is green; the full existing suite still passes.

## 6. Risks, constraints, and interactions

- **Existing constant-folded tests.** Observable outputs of constant programs must be
  unchanged (real execution of `console.log(40+2)` still prints `42`). Optimizer fixtures that
  assert on wasm **size / instruction counts** may shift when real lowering is added; those
  assertions are updated to the new evidence values (they are evidence notes, not contracts —
  `plan/phase-24/README.md` §24.1/§24.4). No observable-output test should need its expected
  string changed; if one does, that is a regression to investigate, not to paper over.
- **Spec governance / claim drift.** This materially advances real runtime semantics. The
  implementation must add companion rows/updates to `specs/19-feature-maturity.md` (loops,
  mutable locals, function calls, memory arrays, runtime int→string) describing exactly the
  supported slice — narrowly and honestly, no over-claiming beyond the integer fannkuch surface
  — and keep `proofs/BOUNDARY.md` untouched (no new proof-backed claims here).
- **No-GC / pure-Rust invariants.** Bump allocation (no free) respects the no-tracing-GC rule;
  no new non-Rust dependencies are introduced.
- **Scope creep into a general backend.** Pieces are implemented generally enough to compile
  fannkuch's idioms, but the milestone is bounded by the acceptance test. Constructs outside
  A–E stay on their current gated/fallback paths; we do not attempt the whole language.
- **`break`/`continue`, `for` desugaring, signed vs unsigned compares, integer overflow.**
  fannkuch stays within JS safe-integer range and bit 63 = 0; comparisons use signed `i64`
  ops. Edge cases beyond fannkuch's needs are deferred.

## 7. Suggested implementation sequencing

A (loops) → B (mutable locals) — together unlock the simplest dynamic programs and are the
highest-leverage fix. Then C (functions/recursion). Then D (memory arrays). Then E
(int→string). Finally the vendored fannkuch port + end-to-end test + fixture metadata, plus the
honest `specs/19-feature-maturity.md` updates. Each phase lands with its micro-acceptance test
before the next begins (TDD).
