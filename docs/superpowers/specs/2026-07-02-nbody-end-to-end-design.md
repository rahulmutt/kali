# Design — n-body End-to-End (First Heap-Object CLBG Slice)

Date: 2026-07-02
Status: Approved design (brainstorming output; user approved section walkthrough)

## 1. Problem and motivation

The fannkuch-redux slice built the integer imperative core; the spectral-norm slice
(`docs/superpowers/specs/2026-07-01-spectral-norm-end-to-end-design.md`, merged) added the f64
representation lane: interprocedural int-vs-float repr inference, f64 arithmetic/arrays/
params/returns, runtime `Math.sqrt`, and `float_to_fixed` (`.toFixed`) output. Two adapted
Computer Language Benchmarks Game (CLBG) fixtures now execute end-to-end.

Every remaining CLBG program needs one new major capability. Ranked by incremental cost
(capability audit, 2026-07-02): **n-body** needs runtime heap objects; mandelbrot needs
byte-oriented stdout + byte buffers; binary-trees needs allocation reclamation; fasta,
k-nucleotide, reverse-complement need runtime strings/stdin/hashmaps; pidigits needs true
bignum; regex-redux needs a regex engine.

**n-body is the next simplest and the natural ladder step.** Its only missing capability is a
runtime object model — and the *minimal* one: five bodies allocated once at startup and
mutated in place, so no reclamation, no dynamic shapes, no polymorphism. Everything else it
needs (f64 arithmetic, `Math.sqrt`, f64 unary negation, `.toFixed(9)`, arrays, loops,
functions) already works.

The blocker is architectural (confirmed by reading codegen): **objects do not exist at
runtime.** `emit_aggregate_literal` walks an object literal's fields, emits each value only
for side-effects, `Drop`s them, and pushes `I64Const(0)` as the object's "value"
(`crates/kali_codegen/src/emit/literal.rs:17-42`). Property reads are resolved at compile time
by structural fold (`crates/kali_codegen/src/emit/operators.rs:269`, `:337` via
`object_literal_field`, `crates/kali_codegen/src/intrinsics/object.rs:21`). **No property-store
path exists at all.** So "make n-body work" is introducing the first genuine runtime
heap-object lane, with one CLBG program as the acceptance test.

Correction to the record: spectral-norm's spec §4.1 refers to "the existing `ObjectLayout`
classification." No such pass exists anywhere in the repo (grep is empty). This slice
introduces the first real object-layout classification (the `Shape` table below).

## 2. Goal and non-goals

**Goal.** Make `kali run` execute an idiomatic TS port of the CLBG n-body benchmark correctly
end-to-end and print the two canonical energy lines (`toFixed(9)`) for pinned `n = 1000`,
byte-matching a reference Node.js run, by adding genuine (not pattern-matched) runtime heap
objects: static shape classification of object literals, bump-allocated fixed-layout structs
in linear memory, typed property load **and store** at static offsets, arrays of object
references, and objects passed as function parameters and returns. Plus two small enabling
pieces: scientific-notation numeric literals in the lexer (n-body's planetary constants are
written `4.84143144246472090e+00` upstream), and module-scope `const` reads from inside
functions (`SOLAR_MASS`, `DAYS_PER_YEAR`), which today **silently miscompile** — codegen
lowers the identifier through a zero placeholder (`const K = 3; function f() { return K + 1; }`
prints `1`). The slice inlines compile-time-pure module-const initializers at function read
sites and upgrades every other module-binding read from a function to a gated reject.

**Non-goals (explicitly deferred):**

- Classes, `new`, `this`, prototypes, methods. The port uses object-literal factory functions
  and free functions — still idiomatic TS, normalized to Kali's pipeline exactly as spectral
  normalized `i++` to `i = i + 1`. These constructs stay on the existing gated path.
- Dynamic shape mutation (adding/deleting fields), polymorphic shapes at one program point,
  nested objects (object-typed fields), objects as array-literal *keys* or reaching
  console/string seams — all **gated reject** via the existing `e5::FEATURE_UNAVAILABLE`
  convention (`crates/kali_codegen/src/emit/literal.rs:347`), never silently miscompiled.
- Allocation reclamation / GC. n-body allocates five body objects and one array, once. The
  existing `__heap` bump allocator suffices; binary-trees forces reclamation later.
- General object features: computed property names, spread, getters/setters, `Object.*` on
  runtime objects, equality/identity comparison of object references.
- Performance/throughput claims. This slice proves correctness of execution, consistent with
  `plan/phase-24/README.md` §24.4 and the fannkuch/spectral precedent.

## 3. Target program

A vendored, idiomatic TS port of the published Node.js / JavaScript n-body submission,
normalized to Kali's pipeline (per `specs/16-testing.md:44`): retains upstream CLBG
attribution, respects CLBG license terms, no benchmark-specific intrinsic tuning.
Normalization: constructor/prototype style becomes object-literal factories + free functions;
`i++`/compound assignment become explicit forms, matching the two prior ports. Planetary
constants are vendored **verbatim** (digit-for-digit, including e-notation) from the upstream
submission at implementation time. `n` is a compile-time integer literal pinned at `1000`;
expected output (two lines) is captured from a reference Node run of the same port and pinned
in the test.

Sketch (constants elided here, vendored verbatim in the fixture):

```ts
// The Computer Language Benchmarks Game — n-body
// idiomatic TS port of the Node.js / JavaScript submission, normalized to
// Kali's pipeline (no intrinsic tuning). Retains upstream attribution.
const PI = 3.141592653589793;
const SOLAR_MASS = 4 * PI * PI;
const DAYS_PER_YEAR = 365.24;

function Jupiter() {
  return { x: /* verbatim */, y: ..., z: ..., vx: ..., vy: ..., vz: ..., mass: ... };
}
// Saturn(), Uranus(), Neptune(), Sun() likewise

function offsetMomentum(bodies) {
  let px = 0; let py = 0; let pz = 0;
  for (let i = 0; i < bodies.length; i = i + 1) {
    const b = bodies[i];
    px = px + b.vx * b.mass; py = py + b.vy * b.mass; pz = pz + b.vz * b.mass;
  }
  bodies[0].vx = -px / SOLAR_MASS;
  bodies[0].vy = -py / SOLAR_MASS;
  bodies[0].vz = -pz / SOLAR_MASS;
}

function advance(bodies, dt) {
  for (let i = 0; i < bodies.length; i = i + 1) {
    const bi = bodies[i];
    for (let j = i + 1; j < bodies.length; j = j + 1) {
      const bj = bodies[j];
      const dx = bi.x - bj.x; const dy = bi.y - bj.y; const dz = bi.z - bj.z;
      const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
      const mag = dt / (distance * distance * distance);
      bi.vx = bi.vx - dx * bj.mass * mag;
      // ... vy, vz; and the symmetric bj updates with bi.mass
    }
  }
  for (let i = 0; i < bodies.length; i = i + 1) {
    const b = bodies[i];
    b.x = b.x + dt * b.vx; b.y = b.y + dt * b.vy; b.z = b.z + dt * b.vz;
  }
}

function energy(bodies) {
  let e = 0;
  for (let i = 0; i < bodies.length; i = i + 1) {
    const bi = bodies[i];
    e = e + 0.5 * bi.mass * (bi.vx * bi.vx + bi.vy * bi.vy + bi.vz * bi.vz);
    for (let j = i + 1; j < bodies.length; j = j + 1) {
      const bj = bodies[j];
      const dx = bi.x - bj.x; const dy = bi.y - bj.y; const dz = bi.z - bj.z;
      e = e - (bi.mass * bj.mass) / Math.sqrt(dx * dx + dy * dy + dz * dz);
    }
  }
  return e;
}

const bodies = [Sun(), Jupiter(), Saturn(), Uranus(), Neptune()];
offsetMomentum(bodies);
console.log(energy(bodies).toFixed(9));
for (let i = 0; i < 1000; i = i + 1) {
  advance(bodies, 0.01);
}
console.log(energy(bodies).toFixed(9));
```

Feature surface distilled — **new**: object literals with seven named f64 fields; heap
allocation with a fixed layout; property reads through local aliases (`bi.x`); property
writes including read-modify-write (`b.x = b.x + dt * b.vx`) and writes through an array
element (`bodies[0].vx = …`); an array literal of five object references; object references
flowing array-element → local binding → function parameter; e-notation float literals;
module-scope consts (`PI`, `SOLAR_MASS`, `DAYS_PER_YEAR`) read from inside the factories and
`offsetMomentum`.
**Already present**: f64 `+ - * /`, f64 unary negation (`-px`, `F64Neg` at
`crates/kali_codegen/src/emit/operators.rs:77`), runtime `Math.sqrt` (inline `F64Sqrt`),
`.length`, integer loop counters, `.toFixed(9)` via `float_to_fixed`, `console.log` of string
handles.

## 4. Architecture of the slice

All work flows through the existing pipeline AST → HIR → MIR → LIR → wasm on the wasmtime
host. The value model is untouched for integers, floats, and strings; **an object reference
slots in as a third statically-chosen representation: an `i64` pointer into linear memory**,
exactly the type-directed philosophy of the spectral repr lane. Likely zero `kali_runtime`
changes: output reuses `float_to_fixed` + `console.log`, allocation reuses the guest-side
bump allocator.

### 4.1 The crux — shape classification, riding the existing repr inference

Spectral built an interprocedural union-find representation inference in `kali_types`
(shared `Repr`/`ReprTable` types in `kali_common`, threaded to codegen as a side table on
`ResolutionResult` → `AnalyzedSource` → `CodegenCtx`). This slice **extends that same
inference** rather than adding a parallel one:

1. **Shapes.** A `Shape` is an interned, ordered list of `(field name, field repr)`. Each
   object-literal site derives its shape from its syntactic fields; field reprs come from the
   same inference (all seven n-body fields solve to f64). Identical field lists intern to the
   same `ShapeId`.
2. **Repr extension.** `Repr` gains an `Object(ShapeId)` variant alongside int/float. An
   object-literal expression seeds its node `Object(shape)`.
3. **Propagation.** The existing equality edges (assignment, argument ↔ parameter, return ↔
   call result, array-element read/write ↔ element repr) propagate `Object(ShapeId)`
   unchanged — `const bi = bodies[i]` gives `bi` the element's shape; `advance(bodies, dt)`
   gives the parameter the array's repr.
4. **Unification rules.** `Object(s) ∪ Object(s)` = itself; `Object(s₁) ∪ Object(s₂)` with
   `s₁ ≠ s₂`, or `Object ∪ float-seed`, or an `Object` node used where a scalar is required
   (arithmetic operand, array index, console/string seam) ⇒ **gated reject**
   (`FEATURE_UNAVAILABLE`), not a miscompile.
5. **Additivity.** A program with no object literals produces no `Object` nodes; fannkuch and
   spectral lower byte-identically. Programs whose object literals are fully handled by the
   existing compile-time fold (see §4.3) are also unchanged.

Member access resolves against the expression's shape: unknown field name ⇒ reject; known
field ⇒ static byte offset = field index × 8 (all n-body fields are 8-byte f64; i64 fields
get the same slot size, so offset arithmetic is uniform). No header word: an object is just
its fields (arrays keep their length header; objects need none because shape is static).

### 4.2 Codegen lowering (type-directed off the extended repr table)

Seven pieces, each independently micro-testable before the full benchmark (fannkuch/spectral
discipline):

**0. e-notation numeric literals.** `lex_number` (`crates/kali_lexer/src/number.rs:5`)
currently accepts only digits, one decimal point, and the bigint `n` suffix. Extend it to
accept `[eE][+-]?digits` after the integer/fraction part (rejecting a following bigint
suffix), and make literal→f64 value parsing accept the same. E-notation forces a float seed
in repr inference. Micro-acceptance: `console.log((1.5e+01).toFixed(1))` → `15.0`.

**1. Object literal materialization.** For a literal marked runtime-materialized (§4.3):
bump-allocate `nfields × 8` bytes via the existing `__heap` path
(`crates/kali_codegen/src/emit/call.rs:2265`), emit each field value with its repr-directed
store (`F64Store`/`I64Store` at `base + index*8`), push the base pointer as the expression's
`i64` value. Replaces the drop-and-`I64Const(0)` path for these sites. Note: under the
fold-first rule (§4.3) a write-free, fully-foldable literal never materializes, so pieces
1–3 share the first independently-runnable acceptance test (piece 3's program, whose member
store forces materialization); pieces 1–2 are unit-covered until then.

**2. Property read.** Member access on an `Object(shape)`-repr expression lowers to
`F64Load`/`I64Load` at `ptr + offset(field)`. Micro-acceptance: piece 3's program (read side
of the read-modify-write), plus a read through a second binding alias.

**3. Property write.** New assignment-target lowering for member expressions:
`e.f = v` evaluates `e` (pointer), `v`, then `F64Store`/`I64Store` at the static offset.
Covers read-modify-write since the read side is piece 2. Micro-acceptance:
`const p = { x: 1.0 }; p.x = p.x + 1.5; console.log(p.x.toFixed(1))` → `2.5`.

**4. Arrays of object references.** An array literal whose elements have shape `s` stores
element pointers as plain i64 values in the existing `[len@+0][elems@+8…]` layout; element
repr `Object(s)` makes `bodies[i]` yield a shaped pointer, so `bodies[i].vx` and
`bodies[0].vx = …` chain pieces 2–3. Micro-acceptance:
`const a = [{ x: 1.0 }, { x: 2.0 }]; a[1].x = 5.0; console.log(a[1].x.toFixed(1))` → `5.0`.

**5. Objects across function boundaries.** Parameters, returns, and locals with
`Object(shape)` repr are wasm `i64` (pointers) — signatures need no change beyond what the
repr table already drives. Factory functions (`Jupiter()` returning a literal) and consumers
(`advance(bodies, dt)`) fall out. Micro-acceptance:
`function mk(v) { return { x: v }; } function getx(p) { return p.x; }
console.log(getx(mk(3.5)).toFixed(1))` → `3.5`.

**6. Gates.** Reject (never miscompile) with `FEATURE_UNAVAILABLE`: unknown field on a shaped
expression; shape mismatch on unification; object reaching arithmetic, an array index, a
console/string seam, or a context requiring a scalar; classes/`new`/`this`; dynamic field
add/delete (any member write to a field not in the shape). Micro-acceptance: a compile test
asserting the diagnostic for `p.z = 1.0` on a `{x, y}` literal, and for `console.log(p)`.

**7. Module-const reads from functions.** Discovered while planning: an identifier read inside
a function that resolves to a module-scope binding reaches codegen's zero-placeholder fallback
(`crates/kali_codegen/src/emit/control_flow.rs:452`) — a warning plus `I64Const(0)`, i.e. a
silent wrong answer. Because all functions share one `LirProgram` node space, the fix is a
compile-time inline: lowering collects `module const name → init node` for top-level `const`
declarators plus the set of all top-level binding names; a function-body identifier that
misses locals and fold bindings then (a) inlines the const's initializer when it is
compile-time pure (literal, module-const identifier, unary/binary arithmetic over pure
operands — recursively, cycle-bounded), or (b) is rejected with `FEATURE_UNAVAILABLE` when it
names any other module binding (mutable `let`, impure init). Locals shadow correctly for free
(the locals lookup runs first). `is_float_valued` gets the same fallback so inlined float
consts pick f64 instructions. Micro-acceptance:
`const K = 3; function f() { return K + 1; } console.log(f())` → `4`, and a reject test for a
module `let` read from a function.

### 4.3 Fold-first: preserving the existing compile-time object lane

Existing fixtures (`object-enumeration-*`, `const-object-property-access-*`,
`reflect-own-keys-*`, `object-literal-property-order-canonicalization-*`) rely on the current
compile-time object fold, and its `I64Const(0)` placeholder is only ever reachable when no
runtime read exists. Rule: **a literal site is runtime-materialized only if it needs to be** —
any member store through any alias, any flow across a function/array boundary, or any member
read the existing fold cannot resolve. Otherwise the current fold path is kept verbatim.
Consequence: every existing fixture lowers byte-identically (their literals are fully folded);
the new lane activates only where the old lane was already semantically wrong-or-gated. If an
optimizer fixture's wasm-size evidence note shifts anyway, it is updated as an evidence note
per `plan/phase-24/README.md` §24.1, not silently re-baselined.

### 4.4 Output

Two `console.log(energy(bodies).toFixed(9))` lines (before and after 1000 advance steps),
via the existing `float_to_fixed` host helper. Expected values are captured from a reference
Node run of the identical port and pinned; for `n = 1000` the canonical CLBG values are
`-0.169075164` / `-0.169087605` (the capture must confirm).

## 5. Acceptance criteria

1. `kali run <nbody fixture>` prints exactly the two canonical lines for `n = 1000`,
   byte-matching the reference Node.js run captured and asserted in the test.
2. Each of pieces 0–7 has a passing micro-acceptance test (run-tests for 0–5 and 7, with
   pieces 1–3 sharing the write-driven program per §4.2; diagnostic compile-tests for 6 and
   for piece 7's gated module-binding reads).
3. The fixture ships schema-v1 benchmark metadata (`schemas/benchmark/v1.json`): `benchmark`,
   `version`, `sourceFile`, validated `sourceSha256`, `buildModes`
   `["--fast", "--release", "--release-advanced"]`, CLBG attribution in the source header —
   mirroring `spectral-norm-benchmark-v1.{ts,json}` and the two-test structure of
   `crates/kali_cli/tests/clbg_spectral_norm_runtime.rs`.
4. `cargo test --workspace` non-browser gate is green; fannkuch, spectral-norm, and all
   existing object-fold fixtures are unchanged (no `Object` nodes / fully-folded literals ⇒
   byte-identical lowering).

## 6. Risks, constraints, and interactions

- **`.toFixed` rounding divergence (documented, inherited).** Rust half-to-even vs JS
  half-up differ only on exact ties at the 9th decimal. Verify the two pinned outputs are not
  ties (spectral §6 precedent); the host lane uses Rust formatting, the browser bundle lane
  real JS `toFixed`.
- **Float determinism.** All operations are IEEE-754 f64 (`+ - * / sqrt neg`), each exactly
  specified, and evaluation order is fixed by the source — wasmtime and Node must agree
  bit-for-bit, so byte-matching Node is sound. No fma, no libm-approximated functions.
- **Blast radius.** Additivity argument in §4.1(5) and §4.3: no object literals ⇒ no change;
  folded literals ⇒ no change. Any observable-output test that changes is a regression to
  investigate, not re-baseline.
- **Verbatim constants.** Planetary constants and the port structure are transcribed from the
  upstream submission at implementation time with the digest-pinned fixture; the design
  intentionally does not restate them (transcription in a design doc invites drift).
- **Spec governance.** Add narrow rows to `specs/19-feature-maturity.md`: runtime
  fixed-shape object literals (alloc, typed field load/store), object references in arrays
  and across calls, e-notation literals — scoped to exactly this monomorphic, statically
  shaped surface. Keep `proofs/BOUNDARY.md` untouched.
- **Shape inference remains bespoke.** Like the repr inference it extends, it is not a
  general MIR layout pass; promoting it stays an explicit future follow-up.
- **Heap headroom.** Five 56-byte objects + one 5-element array in fixed 16-page memory —
  no `memory.grow` needed (fannkuch/spectral precedent).
- **Module-binding gate blast radius (piece 7).** Today a module-binding read from a function
  is a *warning* plus a zero placeholder; the slice upgrades it to a hard reject for
  non-inlinable bindings. Any existing test that pinned the silent-zero behavior is pinning a
  miscompile: if one fails, flag it explicitly and correct its expectation as a behavior fix —
  do not weaken the gate. Truly-undefined identifiers keep the existing warning fallback
  (their names are not module bindings).

## 7. Suggested implementation sequencing

Piece 0 (e-notation lexing) first — independent and unblocks writing any float-constant
tests. Then the shape extension to repr inference (§4.1) — the load-bearing analysis. Then
pieces 1 (literal materialization) → 2 (read) → 3 (write) — landing together behind piece
3's write-driven acceptance test — then 4 (arrays of refs) → 5 (across functions) →
6 (gates) → 7 (module-const reads, independent of the object lane), each subsequent piece
landing with its micro-acceptance test before the next (TDD).
Then the vendored port + reference-output capture + fixture metadata + end-to-end test +
`specs/19-feature-maturity.md` rows.
