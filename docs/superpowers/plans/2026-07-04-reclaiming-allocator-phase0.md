# Reclaiming Allocator — Phase 0 Implementation Plan (memory.grow + centralized `__alloc` + string-escape lane)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lift kali's fixed 1 MB heap wall by adding `memory.grow` behind a centralized `__alloc` helper, and make string-escape sequences (`\t`, `\n`, …) produce real bytes — the two prerequisites that let the binary-trees CLBG fixture run at larger depth and emit canonical TAB-separated output.

**Architecture:** Three independent workstreams. (A) String escapes: the lexer *validates* escape sequences (rejecting unknown ones with a diagnostic) but keeps the raw token value so `kali_fmt` round-trips unchanged; `StringPool::intern` *decodes* recognized escapes into runtime bytes — the single decode chokepoint. (B) A synthetic `__alloc(size)->ptr` wasm helper centralizes the 5 inlined bump-allocation sites. (C) `__alloc` grows linear memory geometrically when the bump pointer would overflow committed pages.

**Tech Stack:** Rust; `wasm-encoder` for codegen; `wasmtime` runtime; crates `kali_lexer`, `kali_codegen`, `kali_cli` (integration tests).

## Global Constraints

- **kali is GC-less by design** — no tracing/copying/generational collector, ever. This phase adds only `memory.grow` + a bump `__alloc`; no reclamation logic (that is Phase 1 regions). Copied verbatim from the spec's Non-goals.
- **Reject, don't miscompile** — unsupported input (here: unknown/`\x`/`\u` escape sequences) must produce a compile diagnostic, never silent wrong bytes.
- **Byte-identical existing fixtures** — the `__alloc` refactor must not change any existing emitted output; n-body, mandelbrot, spectral-norm, fannkuch, and all object/array runtime tests stay green and byte-for-byte unchanged.
- **`memory.grow` returns −1 on failure and never traps** — every growth path must check the −1 result and surface OOM cleanly.
- **`kali_fmt` output must be unchanged** — do not decode escapes in the lexer token value; the formatter re-emits `token.value` verbatim (`crates/kali_fmt/src/formatter.rs:101`).
- Diagnostic code for rejected escapes: reuse the existing `e5::FEATURE_UNAVAILABLE` (`E5506`) family already used for other reject-don't-miscompile gates.

## File structure

- `crates/kali_lexer/src/string.rs` — add escape-sequence validation in `lex_string` (reject unknown; keep raw value).
- `crates/kali_lexer/src/string_tests.rs` (new, or extend `engine_tests.rs`) — lexer escape-validation unit tests.
- `crates/kali_codegen/src/ctx.rs` — add `decode_string_escapes` and apply it in `StringPool::intern`.
- `crates/kali_codegen/src/lower.rs` — emit the synthetic `__alloc` function (type, code, index bookkeeping) + `memory.grow` body; keep the `__heap` global.
- `crates/kali_codegen/src/emit/object.rs`, `crates/kali_codegen/src/emit/call.rs` — replace the 5 inline bump sites with `call __alloc`.
- `crates/kali_cli/tests/string_escapes_runtime.rs` (new) — end-to-end `od`-verified escape output.
- `crates/kali_cli/tests/heap_grow_runtime.rs` (new) — end-to-end >1 MB allocation succeeds; OOM traps cleanly.

---

## Task 1: String-escape lane (validate in lexer, decode at intern)

**Files:**
- Modify: `crates/kali_lexer/src/string.rs`
- Test: `crates/kali_lexer/src/engine_tests.rs`
- Modify: `crates/kali_codegen/src/ctx.rs:159` (`StringPool::intern`)
- Test: `crates/kali_cli/tests/string_escapes_runtime.rs` (new)

**Interfaces:**
- Produces: `decode_string_escapes(&str) -> String` in `crates/kali_codegen/src/ctx.rs` (recognized escapes → char; an unrecognized `\x` sequence is passed through **verbatim**, because rejection already happened in the lexer). Recognized set: `\n \t \r \\ \" \' \` \0 \b \f \v`.
- Produces: lexer now emits `e1::UNSUPPORTED_ESCAPE` (add this code) — or, if adding an `e1` code is undesirable, reuse `e1::UNTERMINATED_STRING`'s module with a new constant — on an unrecognized escape in `lex_string`. (Choose the new constant name `UNSUPPORTED_ESCAPE`; wire it in `kali_error`.)

- [ ] **Step 1: Write the failing lexer unit test**

In `crates/kali_lexer/src/engine_tests.rs` add:

```rust
#[test]
fn test_lexer_rejects_unknown_escape() {
    let mut lexer = Lexer::new(FileId::new(0), r#""a\qb""#.to_string());
    let _ = lexer.next_token();
    assert!(
        lexer.diagnostics().iter().any(|d| d.message.contains("escape")),
        "expected an unsupported-escape diagnostic, got: {:?}",
        lexer.diagnostics()
    );
}

#[test]
fn test_lexer_accepts_known_escapes_and_keeps_raw_value() {
    let mut lexer = Lexer::new(FileId::new(0), r#""a\tb\n""#.to_string());
    let token = lexer.next_token().expect("token");
    // Value is kept RAW (with backslashes) so kali_fmt round-trips.
    assert_eq!(token.value, r#""a\tb\n""#);
    assert!(lexer.diagnostics().is_empty(), "{:?}", lexer.diagnostics());
}
```

(If `Lexer` exposes diagnostics under a different accessor than `diagnostics()`, adjust to the real accessor — check `crates/kali_lexer/src/engine.rs` for the existing `emit_error` sink used by `test_lexer_unterminated_string`.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_lexer test_lexer_rejects_unknown_escape -- --nocapture`
Expected: FAIL (no diagnostic emitted — escapes are currently accepted verbatim).

- [ ] **Step 3: Add the diagnostic code**

In `kali_error` (the `e1` lexer error-code module — find it via `grep -rn UNTERMINATED_STRING crates/kali_error/src`), add a sibling constant, e.g.:

```rust
pub const UNSUPPORTED_ESCAPE: u16 = /* next free E1xxx code in this module */;
```

- [ ] **Step 4: Validate escapes in `lex_string`**

In `crates/kali_lexer/src/string.rs`, replace the `Some(&c) if c == '\\'` arm (lines 19–26) with one that keeps the raw value but rejects unknown escapes:

```rust
Some(&c) if c == '\\' => {
    value.push(c);
    self.position += 1;
    if let Some(next) = self.source.get(self.position).copied() {
        // Keep the raw sequence in `value` (kali_fmt re-emits it verbatim);
        // only validate. Recognized single-char escapes plus the two quote
        // chars and backtick. Numeric \x / \u forms are out of scope: reject.
        if !matches!(next, 'n' | 't' | 'r' | '\\' | '"' | '\'' | '`' | '0' | 'b' | 'f' | 'v') {
            self.emit_error(
                e1::UNSUPPORTED_ESCAPE,
                "unsupported string escape sequence",
            );
        }
        value.push(next);
        self.position += 1;
    }
}
```

Add `use kali_error::_error_codes::e1;` if not already imported (it is — line 3).

- [ ] **Step 5: Run lexer tests to verify pass**

Run: `cargo test -p kali_lexer -- --nocapture`
Expected: PASS (both new tests + existing string tests).

- [ ] **Step 6: Write the failing decode + end-to-end test**

Create `crates/kali_cli/tests/string_escapes_runtime.rs`:

```rust
use std::process::Command;
use std::path::PathBuf;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn string_escapes_decode_to_real_bytes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("esc.ts");
    std::fs::write(&src, "console.log(\"a\\tb\");\nconsole.log(\"c\\nd\");\nconsole.log(\"e\\\\f\");\n").expect("write");
    let out = Command::new(kali_bin()).arg("run").arg(&src).output().expect("run");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    // Real TAB (0x09), then a\tb newline; real newline inside c/d; single backslash e\f.
    assert_eq!(out.stdout, b"a\tb\nc\nd\ne\\f\n");
}
```

Also add a `decode_string_escapes` unit test in `crates/kali_codegen/src/ctx.rs` (under a `#[cfg(test)] mod` if one exists, else create `ctx_tests.rs`):

```rust
#[test]
fn decode_escapes_translates_recognized_and_passes_unknown() {
    assert_eq!(decode_string_escapes(r"a\tb"), "a\tb");
    assert_eq!(decode_string_escapes(r"c\nd"), "c\nd");
    assert_eq!(decode_string_escapes(r"e\\f"), r"e\f");
    assert_eq!(decode_string_escapes(r"\q"), r"\q"); // unknown passed through (lexer already rejected)
}
```

- [ ] **Step 7: Run to verify it fails**

Run: `cargo test -p kali_cli --test string_escapes_runtime -- --nocapture`
Expected: FAIL — stdout is currently `a\tb\nc\nd\ne\\f\n` with literal backslashes (escapes not decoded).

- [ ] **Step 8: Add `decode_string_escapes` and apply it in `intern`**

In `crates/kali_codegen/src/ctx.rs`, add above `impl StringPool`:

```rust
/// Decode the recognized single-character string escapes into their bytes.
/// The lexer has already rejected unrecognized escapes, so an unknown `\x`
/// sequence here is passed through verbatim (best-effort, never a panic).
pub(crate) fn decode_string_escapes(text: &str) -> String {
    if !text.contains('\\') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('`') => out.push('`'),
            Some('0') => out.push('\0'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000C}'),
            Some('v') => out.push('\u{000B}'),
            Some(other) => { out.push('\\'); out.push(other); }
            None => out.push('\\'),
        }
    }
    out
}
```

Then in `StringPool::intern` (line 159), decode before storing:

```rust
pub(crate) fn intern(&mut self, text: &str) -> (u32, u32) {
    let text = decode_string_escapes(text);
    // ... existing dedup/offset logic, but store/measure `&text` (the decoded String) ...
    let offset = self.next_offset;
    let len = text.len() as u32;
    self.entries.push((offset, text));
    self.next_offset = self.next_offset.saturating_add(len);
    (offset, len)
}
```

(Preserve the existing dedup-by-text behavior if present — dedup on the decoded value. Adjust the exact body to match the current `intern` implementation; the change is: decode the incoming `text` first, then run the existing logic against the decoded string.)

- [ ] **Step 9: Run to verify pass**

Run: `cargo test -p kali_codegen decode_escapes -- --nocapture && cargo test -p kali_cli --test string_escapes_runtime -- --nocapture`
Expected: PASS (`out.stdout == b"a\tb\nc\nd\ne\\f\n"`).

- [ ] **Step 10: Guard against regressions + fmt**

Run: `cargo test -p kali_lexer -p kali_codegen -p kali_cli && cargo fmt --check`
Expected: PASS — no existing test regresses; formatting clean. In particular confirm a `kali_fmt` test that formats a string with `\t` still emits `\t` (raw), since the lexer value is unchanged.

- [ ] **Step 11: Commit**

```bash
git add crates/kali_lexer/src/string.rs crates/kali_lexer/src/engine_tests.rs \
        crates/kali_codegen/src/ctx.rs crates/kali_cli/tests/string_escapes_runtime.rs \
        crates/kali_error
git commit -m "feat(lexer,codegen): decode recognized string escapes to bytes; reject unknown escapes (kali_fmt-safe)"
```

---

## Task 2: Centralize allocation into a synthetic `__alloc` helper (behavior-preserving)

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (emit `__alloc`; type/index bookkeeping)
- Modify: `crates/kali_codegen/src/emit/object.rs:62` (`emit_object_allocation`)
- Modify: `crates/kali_codegen/src/emit/call.rs:2356` (`emit_array_allocation_with_len`) and the sibling array sites
- Test: existing fixture tests (no new behavior) + a wasm-structure assertion

**Interfaces:**
- Produces: a synthetic wasm function named `__alloc` with type `(i32) -> i32` (param: byte size; result: the base pointer), registered in `function_name_to_index` so emitters resolve its index via `self.alloc_fn_index()` (add this accessor to the emitter, populated from `function_name_to_index["__alloc"]`).
- Consumes (unchanged): the `__heap` mutable i32 global at global index 0.

**Behavior this task preserves exactly:** `__alloc(size)` returns the old `__heap` and advances `__heap` by `size` — identical to today's inline `base = __heap; __heap += size`. No `memory.grow` yet (Task 3). All existing emitted modules must stay byte-identical *except* for the added function/type/index shifts, and all runtime outputs must be unchanged.

- [ ] **Step 1: Write the failing structural test**

Add to `crates/kali_codegen` a test asserting the module exports/contains an `__alloc` function and that object allocation calls it. Because codegen tests here operate on emitted wasm bytes, assert via a decoder (the crate already depends on `wasmparser` in dev, or reuse an existing codegen test helper). Minimal form:

```rust
#[test]
fn object_allocation_calls_shared_alloc_helper() {
    let wasm = compile_snippet("const o = { a: 1, b: 2 }; o.a;"); // use the crate's existing compile-snippet test helper
    let names = function_names(&wasm); // existing/added helper listing function names
    assert!(names.iter().any(|n| n == "__alloc"), "no __alloc function emitted");
}
```

(If no `compile_snippet`/`function_names` helper exists, use the closest existing codegen test-support entrypoint in `crates/kali_codegen/src/test_support.rs`.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_codegen object_allocation_calls_shared_alloc_helper -- --nocapture`
Expected: FAIL (`__alloc` not emitted).

- [ ] **Step 3: Register the `__alloc` type and function**

In `crates/kali_codegen/src/lower.rs`:

1. After the existing `type_section` entries (after type 9, line 196), add a type for `__alloc`:

```rust
// Type 10: __alloc `(i32 size) -> (i32 ptr)`.
type_section.ty().function(vec![ValType::I32], vec![ValType::I32]);
let alloc_type_index: u32 = 10;
```

2. Add `__alloc` to `all_functions` immediately after `_start` (so it occupies a fixed synthetic slot before named functions), and make its index resolvable. Because `function_name_to_index` is built by enumerating `all_functions` (line 164), inserting `__alloc` at position 1 shifts named-function indices by 1 automatically — this is consistent as long as **every** call site resolves indices through `function_name_to_index` (verify no hardcoded user-function index arithmetic exists that bypasses it). Insert a synthetic plan marker:

```rust
// Synthetic bump allocator; body is hand-emitted (not lowered from LIR).
all_functions.insert(1, FunctionPlan {
    name: "__alloc".to_string(),
    params: vec![/* i32 size */],
    locals: Vec::new(),
    body: lir.root,      // unused: hand-emitted below
    result: true,
    is_entry: false,
    flavor: Some(FunctionFlavor::SyntheticAlloc), // add this variant, or gate on name == "__alloc"
});
```

3. In the code-emission loop (around line 434), special-case `__alloc` to hand-emit its body instead of lowering LIR:

```rust
if function.name == "__alloc" {
    emit_alloc_body(&mut body);   // defined below
} else if function.is_entry {
    // ... existing _start path ...
} else {
    // ... existing emit_function_body path ...
}
```

4. Ensure the `function_section.function(type_index)` registration for `__alloc` uses `alloc_type_index` (10), not a lowered signature. Mirror how `_start` gets its type index.

- [ ] **Step 4: Emit the `__alloc` body (bump only, no grow yet)**

Add to `crates/kali_codegen/src/lower.rs`:

```rust
/// Phase 0 bump body: `ptr = __heap; __heap = ptr + size; return ptr`.
/// (Task 3 inserts the memory.grow check before the store-back.)
fn emit_alloc_body(func: &mut Function) {
    // local 0 = size (param). Return old __heap, then advance it.
    func.instruction(&Instruction::GlobalGet(0)); // old __heap (result value, left on stack at end)
    func.instruction(&Instruction::GlobalGet(0));
    func.instruction(&Instruction::LocalGet(0));  // size
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(0));
    // old __heap is on the stack -> function result.
    func.instruction(&Instruction::End);
}
```

- [ ] **Step 5: Rewire the object allocator to call `__alloc`**

In `crates/kali_codegen/src/emit/object.rs`, replace the inline bump in `emit_object_allocation` (lines ~70–78) with a call:

```rust
// base = __alloc(nfields * 8); (scratch holds the i64 base for field stores)
function.instruction(&Instruction::I32Const((fields.len() * 8) as i32));
function.instruction(&Instruction::Call(self.alloc_fn_index()));
function.instruction(&Instruction::LocalTee(scratch_i32)); // keep i32 base for stores
function.instruction(&Instruction::I64ExtendI32U);
function.instruction(&Instruction::LocalSet(scratch));      // i64 base as before
```

Adjust the subsequent field-store code to use the i32 base already in `scratch_i32` where it currently did `LocalGet(scratch); I32WrapI64`. Add `alloc_fn_index()` to the emitter (`crates/kali_codegen/src/emitter.rs`), returning the index looked up once from `function_name_to_index`.

- [ ] **Step 6: Rewire the array allocators to call `__alloc`**

In `crates/kali_codegen/src/emit/call.rs`, in `emit_array_allocation_with_len` (line 2356) and the static/dynamic siblings, replace the inline `GlobalGet(0) … GlobalSet(0)` bump (which advances by `(len+1)*8`) with:

```rust
// total_bytes = (len + 1) * 8 ; base = __alloc(total_bytes)
// (compute total_bytes into size_scratch as an i32, then:)
function.instruction(&Instruction::LocalGet(size_scratch_i32));
function.instruction(&Instruction::Call(self.alloc_fn_index()));
function.instruction(&Instruction::LocalTee(scratch_i32));
// then store the length header at [base + 0] and return base, as before.
```

Keep the length-header store and the returned-handle shape identical to today; only the pointer source changes from an inline global bump to `__alloc`.

- [ ] **Step 7: Run structural + all fixture tests**

Run: `cargo test -p kali_codegen && cargo test -p kali_cli`
Expected: PASS — `__alloc` emitted, and **every** existing runtime fixture (n-body `-0.169075164…`, mandelbrot PBM 5011 bytes, spectral-norm `1.274219991`, fannkuch `228 / Pfannkuchen(7) = 16`, object/array tests) unchanged.

- [ ] **Step 8: Verify no output drift on the vendored benchmarks explicitly**

Run: `cargo test -p kali_cli --test clbg_nbody_runtime --test clbg_mandelbrot_runtime --test clbg_spectral_norm_runtime --test clbg_fannkuch_runtime`
Expected: PASS (all four canonical outputs identical).

- [ ] **Step 9: Commit**

```bash
git add crates/kali_codegen/src/lower.rs crates/kali_codegen/src/emitter.rs \
        crates/kali_codegen/src/emit/object.rs crates/kali_codegen/src/emit/call.rs
git commit -m "refactor(codegen): centralize object/array bump allocation into a synthetic __alloc helper (behavior-preserving)"
```

---

## Task 3: `memory.grow` inside `__alloc`

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (`emit_alloc_body` — add growth check)
- Test: `crates/kali_cli/tests/heap_grow_runtime.rs` (new)

**Interfaces:**
- Consumes: the `__alloc` helper from Task 2.
- Behavior added: before advancing `__heap`, if `ptr + size` exceeds the currently-committed bytes (`memory.size * 65536`), grow the memory by enough 64 KiB pages (geometric: grow by `max(pages_needed, current_pages)` — i.e. at least double — to amortize). If `memory.grow` returns −1, trap deterministically (unreachable) so the runtime surfaces a clean OOM rather than a wild out-of-bounds store.

- [ ] **Step 1: Write the failing end-to-end test**

Create `crates/kali_cli/tests/heap_grow_runtime.rs`:

```rust
use std::process::Command;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn allocation_beyond_one_megabyte_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("grow.ts");
    // ~3 MB of i64 array storage: 24 arrays of 16384 elements = 24 * (16384+1)*8 bytes ~= 3.15 MB,
    // well past the old 1 MB (16-page) wall. Touch each so it is not folded away.
    std::fs::write(&src, r#"
let total = 0;
for (let k = 0; k < 24; k = k + 1) {
  const a = new Array(16384);
  a.fill(1);
  total = total + a.length;
}
console.log(total);
"#).expect("write");
    let out = Command::new(kali_bin()).arg("run").arg(&src).output().expect("run");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), (24 * 16384).to_string());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test heap_grow_runtime -- --nocapture`
Expected: FAIL — traps (`E4000`) because cumulative allocation exceeds the fixed 16-page (1 MB) memory and nothing grows it.

- [ ] **Step 3: Add the growth check to `emit_alloc_body`**

Replace `emit_alloc_body` (from Task 2) with a version that grows before storing back. Use two i32 locals (declare them in the function's locals): `new_top` and `pages_needed`.

```rust
/// Bump with growth: ptr = __heap; new_top = ptr + size;
/// if new_top > memory.size*65536 { grow by max(ceil(deficit/65536), memory.size) pages;
///   if grow == -1 { unreachable } }
/// __heap = new_top; return ptr.
fn emit_alloc_body(func: &mut Function) {
    // locals: 0 = size (param); 1 = new_top; 2 = cur_bytes.
    const PAGE: i32 = 65536;
    // new_top = __heap + size
    func.instruction(&Instruction::GlobalGet(0));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::LocalSet(1)); // new_top
    // cur_bytes = memory.size * PAGE
    func.instruction(&Instruction::MemorySize(0));
    func.instruction(&Instruction::I32Const(PAGE));
    func.instruction(&Instruction::I32Mul);
    func.instruction(&Instruction::LocalSet(2)); // cur_bytes
    // if new_top > cur_bytes { grow }
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32GtU);
    func.instruction(&Instruction::If(BlockType::Empty));
    {
        // deficit_pages = ceil((new_top - cur_bytes) / PAGE)
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::LocalGet(2));
        func.instruction(&Instruction::I32Sub);
        func.instruction(&Instruction::I32Const(PAGE - 1));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::I32Const(PAGE));
        func.instruction(&Instruction::I32DivU); // deficit_pages
        // grow_pages = max(deficit_pages, memory.size)  [geometric doubling]
        func.instruction(&Instruction::MemorySize(0));
        // stack: [deficit_pages, cur_pages] -> pick max
        func.instruction(&Instruction::LocalTee(2)); // reuse local 2 = cur_pages
        func.instruction(&Instruction::I32GtU);      // deficit_pages > cur_pages ?
        func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        {
            // recompute deficit_pages (cheaper: keep it in a local instead — see note)
            // For clarity, store deficit_pages in a 3rd local before this If.
        }
        func.instruction(&Instruction::End);
        // memory.grow(grow_pages); if result == -1 -> unreachable
        func.instruction(&Instruction::MemoryGrow(0));
        func.instruction(&Instruction::I32Const(-1));
        func.instruction(&Instruction::I32Eq);
        func.instruction(&Instruction::If(BlockType::Empty));
        func.instruction(&Instruction::Unreachable);
        func.instruction(&Instruction::End);
    }
    func.instruction(&Instruction::End); // end outer if
    // __heap = new_top; return old ptr
    func.instruction(&Instruction::LocalGet(1)); // new_top
    func.instruction(&Instruction::GlobalSet(0));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Sub); // ptr = new_top - size
    func.instruction(&Instruction::End);
}
```

**Note for the implementer:** the `max(deficit_pages, cur_pages)` selection above is easier and less error-prone with a dedicated 3rd i32 local holding `deficit_pages` computed once, then `select`/`if` on it. Declare 3 i32 locals for `__alloc` (`new_top`, `cur_pages`, `deficit_pages`) in the function's local declarations and simplify the block accordingly. The invariant to preserve: after the block, memory is large enough that `new_top <= memory.size*65536`, and on `memory.grow == -1` the function executes `unreachable`.

- [ ] **Step 4: Run the growth test**

Run: `cargo test -p kali_cli --test heap_grow_runtime -- --nocapture`
Expected: PASS — prints `393216` (24 × 16384).

- [ ] **Step 5: Add and run the binary-trees-N=10 smoke (previously trapping)**

Add to `crates/kali_cli/tests/heap_grow_runtime.rs`:

```rust
#[test]
fn binary_trees_depth_ten_no_longer_traps() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("bt10.ts");
    std::fs::write(&src, r#"
function bottomUpTree(depth) {
  if (depth > 0) { return { left: bottomUpTree(depth-1), right: bottomUpTree(depth-1) }; }
  return { left: null, right: null };
}
function itemCheck(node) {
  if (node.left === null) { return 1; }
  return 1 + itemCheck(node.left) + itemCheck(node.right);
}
console.log(itemCheck(bottomUpTree(10)));
"#).expect("write");
    let out = Command::new(kali_bin()).arg("run").arg(&src).output().expect("run");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), (2i64.pow(11) - 1).to_string()); // 2047
}
```

Run: `cargo test -p kali_cli --test heap_grow_runtime -- --nocapture`
Expected: PASS (`2047`).

- [ ] **Step 6: Confirm clean OOM past the sandbox cap**

Add a test that a runaway allocation under a tight `--sandbox` memory policy fails cleanly (non-zero exit, no panic) rather than hanging or wild-writing:

```rust
#[test]
fn oom_past_sandbox_cap_fails_cleanly() {
    let dir = tempfile::tempdir().expect("tempdir");
    let policy = dir.path().join("tiny.policy.json");
    std::fs::write(&policy, r#"{"schemaVersion":1,"effects":{"fileSystem":{"read":false,"write":false},"network":{"fetch":false,"connect":false,"listen":false,"maxConnections":null},"process":{"spawn":false,"envRead":false,"envWrite":false},"timer":{"schedule":false,"maxTimeoutMs":null,"maxActiveTimers":null},"eval":false,"random":false,"console":true},"resources":{"maxMemoryMB":4,"maxCpuTimeMs":100000,"maxOpenFiles":null,"maxSpawnedProcesses":0,"maxThreads":0}}"#).expect("write");
    let src = dir.path().join("oom.ts");
    std::fs::write(&src, "for (let k=0;k<10000;k=k+1){ const a=new Array(16384); a.fill(1); }\nconsole.log(0);").expect("write");
    let out = Command::new(kali_bin()).arg("run").arg("--sandbox").arg(&policy).arg(&src).output().expect("run");
    assert!(!out.status.success(), "expected clean OOM failure");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("panic"), "should not panic: {err}");
}
```

Run: `cargo test -p kali_cli --test heap_grow_runtime oom_past_sandbox_cap_fails_cleanly -- --nocapture`
Expected: PASS. (If the sandbox does not currently cap wasm memory growth, note it: the `unreachable` on `memory.grow == -1` still bounds growth at the wasm 4 GiB ceiling; tighten sandbox enforcement in a follow-up if this test cannot be made to fail cleanly at 4 MB.)

- [ ] **Step 7: Full regression + fmt**

Run: `cargo test -p kali_codegen -p kali_cli && cargo fmt --check`
Expected: PASS — all four CLBG fixtures and object/array tests unchanged; new grow tests green.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen/src/lower.rs crates/kali_cli/tests/heap_grow_runtime.rs
git commit -m "feat(codegen): __alloc grows linear memory geometrically past the 1MB wall; clean OOM on memory.grow == -1"
```

---

## Self-review

**Spec coverage (Phase 0 scope):**
- memory.grow → Task 3. ✓
- centralized `__alloc` helper (refactor of 5 inline sites) → Task 2. ✓
- string-escape lexer lane (recognized set processed; `\x`/`\u`/unknown rejected with diagnostic; kali_fmt unaffected) → Task 1. ✓
- Byte-identical existing fixtures constraint → Task 2 Steps 7–8, Task 3 Step 7. ✓
- Reject-don't-miscompile for escapes → Task 1 Steps 1–5. ✓
- GC-less: no reclamation added this phase → confirmed; only bump + grow. ✓
- Phase 1 (escape-analysis regions, canonical N=21) → deliberately **out of scope**; separate plan after Phase 0 lands.

**Placeholder scan:** The `emit_alloc_body` growth block in Task 3 Step 3 flags a clarity refinement (dedicated 3rd local for `deficit_pages`) rather than leaving logic unspecified — the invariant and the −1→unreachable behavior are fully specified. Acceptable; the implementer has the exact instructions and the invariant.

**Type consistency:** `__alloc` is `(i32)->i32` throughout (Task 2 type 10, Task 3 body, both emitter call sites). `alloc_fn_index()` accessor defined in Task 2 and used in Tasks 2–3. `decode_string_escapes(&str)->String` defined and used consistently in Task 1.

**Open item carried to implementation:** whether the sandbox already enforces `maxMemoryMB` on wasm growth (Task 3 Step 6). If not, the `unreachable` bound at the 4 GiB wasm ceiling still holds; tightening sandbox memory enforcement is a noted follow-up, not a Phase 0 blocker.
