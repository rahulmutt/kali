# mandelbrot End-to-End Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Vendor a `mandelbrot` CLBG fixture that runs under `kali run` and writes the byte-for-byte canonical binary PBM image to stdout.

**Architecture:** Two new lanes. Lane 1: lower the bitwise-integer operators (`&`/`|`/`^`/`<<`/`>>`/`>>>`) to real wasm with JS 32-bit semantics, replacing a silent `I64Add` miscompile, and reject float operands with `E5506`. Lane 2: a host-only binary-stdout sink — a `Kali.writeStdoutBytes(arr)` intrinsic lowers to a new conditional `kali:rt` host import that decodes an array handle to raw bytes and appends them to a `Vec<u8>` sink that `kali run` flushes verbatim.

**Tech Stack:** Rust (`kali_codegen`, `kali_runtime`, `kali_cli`), `wasm-encoder`/wasmtime, TypeScript fixtures. Spec: `docs/superpowers/specs/2026-07-03-mandelbrot-end-to-end-design.md`.

## Global Constraints

- **Reject-don't-miscompile:** every unsupported shape must fail to compile (diagnostic + non-zero exit), never emit a wrong answer. Bitwise on `f64` operands → `E5506`.
- **Fold-first byte-identity:** integer-only programs must stay byte-identical to the pre-change path. Bitwise lowering only fires for operands that reach `emit_binary` unfolded; constant folds are unaffected.
- **JS 32-bit bitwise semantics:** operands are `ToInt32`-coerced (`i32.wrap_i64`), the op runs on `i32`, `>>>` zero-extends (uint32), all others sign-extend back to `i64`.
- **Import indices are fixed for 0–20:** never insert an unconditional import that shifts the fixed block (`test_register`=0 … `float_to_string`=20 in `crates/kali_codegen/src/lib.rs:42-62`). The new import is **conditional**, appended after the existing conditional imports (mirror `env_set`), with a dynamically-computed index.
- **Binary stdout is host-only:** the browser harness serializes stdout as a JSON string; if `Kali.writeStdoutBytes` is reached under the browser backend it must diagnose, not corrupt output.
- **Image size:** `n = 200` (divisible by 8, no intra-row PBM padding). Header is the literal ASCII `P4\n200 200\n`. If `n` is retuned, the header bytes and the golden asset re-pin together.
- **Fixture metadata:** `buildModes` is exactly `["--fast", "--release", "--release-advanced"]`; `sourceSha256` must equal the fixture file digest (mirror `crates/kali_cli/tests/clbg_nbody_runtime.rs`).
- **Verification gate (memory `kali-repo-verification-env`):** `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli` and `cargo fmt --all --check` must be green. Push-a-PR-and-merge per memory `kali-integration-convention`.

---

## File Structure

- `crates/kali_codegen/src/emit/operators.rs` — add the bitwise reject + `emit_bitwise` helper and match arms (Task 1).
- `crates/kali_cli/tests/bitwise_operators_runtime.rs` — new; end-to-end bitwise behavior + reject tests (Task 1).
- `crates/kali_codegen/src/intrinsics/host.rs` — recognize `Kali.writeStdoutBytes` (Task 2).
- `crates/kali_codegen/src/emit/call.rs` — emit the array handle + `Call(stdout_write_bytes)` (Task 2).
- `crates/kali_codegen/src/lower.rs` — conditional import + dynamic index threading (Task 2).
- `crates/kali_codegen/src/lib.rs` — `FUNCTION_INDEX_OFFSET`/detection wiring if needed (Task 2).
- `crates/kali_runtime/src/state.rs`, `outcome.rs`, `execute.rs` — `stdout_bytes` sink threading (Task 3).
- `crates/kali_runtime/src/host/imports_default.rs`, `host/memory.rs` — `stdout_write_bytes` host import + array-handle byte decode (Task 3).
- `crates/kali_cli/src/bin/cmd_run.rs` — raw byte flush (Task 3).
- `crates/kali_cli/tests/binary_stdout_runtime.rs` — new; tiny end-to-end byte-write fixture (Task 3).
- `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.{ts,json}`, `mandelbrot-benchmark-v1.expected.pbm` — fixture + golden (Task 4).
- `crates/kali_cli/tests/clbg_mandelbrot_runtime.rs` — new; canonical-output + metadata tests (Task 4).
- `specs/19-feature-maturity.md`, memory files — docs (Task 5).

---

## Task 1: Bitwise-integer operator lowering + f64 reject

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`emit_binary` around lines 700–915)
- Test: `crates/kali_cli/tests/bitwise_operators_runtime.rs` (create)

**Interfaces:**
- Consumes: existing `emit_binary` / `emit_float_operand` / `is_float_valued` / `self.diagnostics` / `e5::FEATURE_UNAVAILABLE`.
- Produces: correct wasm for `&`,`|`,`^`,`<<`,`>>`,`>>>` on `i64` operands (JS-32-bit semantics); `E5506` rejection when either operand is `f64`.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/bitwise_operators_runtime.rs`. Operands are loop-derived so the constant folder cannot collapse them (this mirrors mandelbrot's real `byte`/`bit` usage):

```rust
use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_js(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn run_js_expect_failure(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(
        !output.status.success(),
        "expected rejection but it ran\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    combined
}

// Builds a runtime byte = 255 via loop-carried shift+or (mandelbrot's packing),
// defeating constant folding, then exercises each operator against it.
const PACK: &str = "let byte = 0;\nfor (let i = 0; i < 8; i = i + 1) { byte = (byte << 1) | 1; }\n";

#[test]
fn shift_left_and_or_pack_bits() {
    assert_eq!(run_js(&format!("{PACK}console.log(\"\" + byte);")), "255\n");
}
#[test]
fn bitwise_and() {
    assert_eq!(run_js(&format!("{PACK}console.log(\"\" + (byte & 15));")), "15\n");
}
#[test]
fn bitwise_or() {
    assert_eq!(run_js(&format!("{PACK}console.log(\"\" + (byte | 256));")), "511\n");
}
#[test]
fn bitwise_xor() {
    assert_eq!(run_js(&format!("{PACK}console.log(\"\" + (byte ^ 255));")), "0\n");
}
#[test]
fn shift_right_arithmetic() {
    assert_eq!(run_js(&format!("{PACK}console.log(\"\" + (byte >> 4));")), "15\n");
}
#[test]
fn shift_right_arithmetic_negative() {
    // neg = -255 (runtime), -255 >> 1 = -128 (sign-preserving)
    let src = format!("{PACK}let neg = 0 - byte;\nconsole.log(\"\" + (neg >> 1));");
    assert_eq!(run_js(&src), "-128\n");
}
#[test]
fn unsigned_shift_zero_extends() {
    // -255 >>> 0 = 4294967041 (uint32)
    let src = format!("{PACK}let neg = 0 - byte;\nconsole.log(\"\" + (neg >>> 0));");
    assert_eq!(run_js(&src), "4294967041\n");
}
#[test]
fn bitwise_on_float_operand_is_rejected() {
    // x is f64 (seeded by 1.5), loop-derived so not folded; `x & 1` must reject.
    let src = "let x = 0.0;\nfor (let i = 0; i < 3; i = i + 1) { x = x + 1.5; }\nconsole.log(\"\" + (x & 1));";
    let out = run_js_expect_failure(src);
    assert!(out.contains("5506") || out.to_lowercase().contains("bitwise"),
        "expected E5506 bitwise-on-float diagnostic, got: {out}");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --test bitwise_operators_runtime`
Expected: FAIL — the `<<`/`|` packing currently miscompiles to `I64Add` (wrong number), and the float case currently emits a warning + `I64Add` instead of rejecting.

- [ ] **Step 3: Add the reject + operand-emit exclusion in `emit_binary`**

In `crates/kali_codegen/src/emit/operators.rs`, right after `let operand_float = ...` (currently line 701) and before the `float_op` match, insert:

```rust
        let is_bitwise = matches!(op, "&" | "|" | "^" | "<<" | ">>" | ">>>");
        if is_bitwise && operand_float {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "bitwise operator '{op}' on a floating-point operand is unavailable in the current phase; use integer operands or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }
```

Then change the auto operand-push guard (currently `if op != "??" && op != "**" {`) to also skip bitwise, since `emit_bitwise` emits and wraps its own operands:

```rust
        if op != "??" && op != "**" && !is_bitwise {
            self.emit_float_operand(function, left, float_op);
            self.emit_float_operand(function, right, float_op);
        }
```

- [ ] **Step 4: Add the bitwise match arm + helper**

In the `match op { … }` inside `emit_binary`, add a single arm just before the catch-all `_ =>`:

```rust
            "&" | "|" | "^" | "<<" | ">>" | ">>>" => {
                self.emit_bitwise(function, op, left, right)
            }
```

Add the helper method to the same `impl` block (near `emit_float_operand`):

```rust
    /// Lowers a bitwise-integer operator with JS 32-bit semantics: both operands
    /// are `ToInt32`-coerced (`i32.wrap_i64`), the op runs on `i32` (wasm masks
    /// shift counts mod 32, matching JS `& 31`), and the result extends back to
    /// `i64` — sign-extended for every op except `>>>`, which zero-extends
    /// (uint32). Float operands are rejected before this point (see `emit_binary`).
    fn emit_bitwise(
        &mut self,
        function: &mut Function,
        op: &str,
        left: LirNodeId,
        right: LirNodeId,
    ) -> EmittedValue {
        self.emit_float_operand(function, left, false);
        function.instruction(&Instruction::I32WrapI64);
        self.emit_float_operand(function, right, false);
        function.instruction(&Instruction::I32WrapI64);
        match op {
            "&" => function.instruction(&Instruction::I32And),
            "|" => function.instruction(&Instruction::I32Or),
            "^" => function.instruction(&Instruction::I32Xor),
            "<<" => function.instruction(&Instruction::I32Shl),
            ">>" => function.instruction(&Instruction::I32ShrS),
            ">>>" => function.instruction(&Instruction::I32ShrU),
            _ => unreachable!("emit_bitwise called with non-bitwise op"),
        };
        if op == ">>>" {
            function.instruction(&Instruction::I64ExtendI32U);
        } else {
            function.instruction(&Instruction::I64ExtendI32S);
        }
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }
```

(Confirm `Function`, `LirNodeId`, `Instruction`, `EmittedValue`, `ValueShape` are already imported in this file — they are used throughout `operators.rs`.)

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test bitwise_operators_runtime`
Expected: PASS (all 8 tests).

- [ ] **Step 6: Guard fold-first byte-identity**

Run: `cargo test -p kali_codegen -p kali_cli`
Expected: PASS with no regressions (integer fixtures — fannkuch etc. — unchanged; bitwise only fires on unfolded operands).

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/bitwise_operators_runtime.rs
git commit -m "feat(codegen): lower bitwise-integer operators (JS 32-bit); reject f64 operands with E5506"
```

---

## Task 2: Recognize and emit `Kali.writeStdoutBytes(arr)` as a conditional host import

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (near `is_kali_test_call`, line 570)
- Modify: `crates/kali_codegen/src/emit/call.rs` (near the `Kali.test` emit path, line ~60)
- Modify: `crates/kali_codegen/src/lower.rs` (conditional-import block, lines 52–245)
- Test: `crates/kali_codegen/src/emit/call_tests/` (new inline test module) OR assert via wasmprinter

**Interfaces:**
- Consumes: `is_kali_test_call` pattern; the `env_set` conditional-import mechanism (`uses_env_set`, `function_index_offset`, `env_set_import_index`).
- Produces: a `program_uses_stdout_write_bytes(lir) -> bool` detector; an `stdout_write_bytes_import_index: Option<u32>` threaded into the emit context; a recognizer `is_kali_write_stdout_bytes_call(&LirNode) -> bool`; emission of `<array handle i64>; Call(index)` for the call, producing no value.

- [ ] **Step 1: Write the failing test**

Add a codegen test that compiles a `Kali.writeStdoutBytes(arr)` program and asserts the wasm imports `stdout_write_bytes` and emits a `call` to it. Create `crates/kali_codegen/src/emit/call_tests/write_stdout_bytes.rs` (register it in the `call_tests` mod file alongside the existing submodules):

```rust
use crate::test_support::compile_source_to_wasm_text; // mirror the helper other call_tests use

#[test]
fn write_stdout_bytes_imports_and_calls_host() {
    let src = "const out = new Array(2); out[0] = 80; out[1] = 52; Kali.writeStdoutBytes(out);";
    let text = compile_source_to_wasm_text(src);
    assert!(text.contains("stdout_write_bytes"), "missing import:\n{text}");
    assert!(text.contains("call"), "missing call:\n{text}");
}
```

(If `call_tests` uses a different compile-to-wat helper name, use that one — grep `crates/kali_codegen/src/emit/call_tests/` for the existing pattern and match it. The assertion content stays the same.)

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_codegen write_stdout_bytes_imports_and_calls_host`
Expected: FAIL — `stdout_write_bytes` not imported; the call is unrecognized.

- [ ] **Step 3: Add the recognizer in `intrinsics/host.rs`**

Below `is_kali_test_call` (line 579), add:

```rust
    pub(crate) fn is_kali_write_stdout_bytes_call(&self, callee_node: &LirNode) -> bool {
        if callee_node.text.as_deref() != Some("writeStdoutBytes") {
            return false;
        }
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.node(object).text.as_deref() == Some("Kali")
    }
```

- [ ] **Step 4: Add the detector + conditional import in `lower.rs`**

Mirror the `env_set` wiring exactly. (a) Near line 52 add:

```rust
    let uses_stdout_write_bytes = program_uses_stdout_write_bytes(lir);
```

(b) Add its contribution to `function_index_offset` (line 57 block) and to every subsequent conditional index computation that sums the earlier `uses_*` flags — add `+ if uses_stdout_write_bytes { 1 } else { 0 }` in the same positions `uses_cwd_set` appears, so the new import lands **after** all existing conditional imports. (c) Compute its index:

```rust
    let stdout_write_bytes_import_index = if uses_stdout_write_bytes {
        Some(
            crate::FUNCTION_INDEX_OFFSET
                + if uses_env_set { 1 } else { 0 }
                + if uses_env_delete { 1 } else { 0 }
                + if uses_env_get { 1 } else { 0 }
                + if uses_env_has { 1 } else { 0 }
                + if uses_cwd_set { 1 } else { 0 }
                + if program_uses_process_exit(lir) { 1 } else { 0 }, // include EVERY conditional import declared before this one; verify against the final import order below
        )
    } else {
        None
    };
```

(Read lines 52–260 first and place this AFTER the last existing conditional index so the offsets match the actual `import_section.import(...)` order.) (d) Append the import at the very end of the conditional import block (after `process_exit`, line ~245):

```rust
    if uses_stdout_write_bytes {
        import_section.import("kali:rt", "stdout_write_bytes", EntityType::Function(1));
    }
```

Type `Function(1)` is `(i64) -> ()` — already declared (`type_section` line 149).

(e) Add the detector fn near the other `program_uses_*` helpers in `lower.rs`:

```rust
fn program_uses_stdout_write_bytes(lir: &LirModule) -> bool {
    lir.nodes.iter().any(|node| {
        node.kind == LirNodeKind::Call
            && node
                .children
                .first()
                .and_then(|callee| lir.nodes.get(callee.0 as usize))
                .map(|callee| {
                    callee.text.as_deref() == Some("writeStdoutBytes")
                        && callee
                            .children
                            .first()
                            .and_then(|obj| lir.nodes.get(obj.0 as usize))
                            .and_then(|obj| obj.text.as_deref())
                            == Some("Kali")
                })
                .unwrap_or(false)
    })
}
```

(Match the exact `LirModule`/`LirNodeKind`/node-shape types used by the neighboring `program_uses_env_set` — copy its traversal shape rather than the sketch above if it differs.)

- [ ] **Step 5: Thread the index into the emit context and emit the call**

Add `stdout_write_bytes_import_index: Option<u32>` to the emit-context struct that already carries `env_set_import_index` (grep `env_set_import_index` to find the struct + its construction site in `lower.rs`; add the field and pass the value through identically).

In `crates/kali_codegen/src/emit/call.rs`, in the call-emission dispatch (near the `Kali.test` handling, line ~60), add before generic call handling:

```rust
        if self.is_kali_write_stdout_bytes_call(callee_node) {
            let Some(index) = self.stdout_write_bytes_import_index else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Kali.writeStdoutBytes is unavailable under this backend".to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue { produced: false, shape: ValueShape::Unknown };
            };
            // first arg is the byte array; emit its handle (i64) and call the host import
            let arg = node.children[1];
            let _ = self.emit_node(function, arg, true);
            function.instruction(&Instruction::Call(index));
            return EmittedValue { produced: false, shape: ValueShape::Unknown };
        }
```

(Confirm the argument child index: `Kali.writeStdoutBytes(out)` lowers as a Call node whose `children[0]` is the callee member and `children[1]` is `out`. Verify against how `Kali.test`/other calls index args in this file and adjust if the arg offset differs.)

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test -p kali_codegen write_stdout_bytes_imports_and_calls_host`
Expected: PASS.

- [ ] **Step 7: Guard no-regression on the fixed import block**

Run: `cargo test -p kali_codegen -p kali_cli`
Expected: PASS — programs that do NOT call `writeStdoutBytes` emit no new import (index space unchanged), so all existing fixtures stay byte-identical.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add crates/kali_codegen/src
git commit -m "feat(codegen): recognize Kali.writeStdoutBytes and emit conditional stdout_write_bytes import"
```

---

## Task 3: Runtime dual-sink + `stdout_write_bytes` host import + CLI byte flush

**Files:**
- Modify: `crates/kali_runtime/src/state.rs:29` (add `stdout_bytes`)
- Modify: `crates/kali_runtime/src/outcome.rs:15` (add `stdout_bytes`)
- Modify: `crates/kali_runtime/src/execute.rs` (thread `stdout_bytes` through every site that clones `stdout` — lines ~110, 183, 208, 226, 247, 270, 293, 311, 357)
- Modify: `crates/kali_runtime/src/host/imports_default.rs` (register `stdout_write_bytes`)
- Modify: `crates/kali_runtime/src/host/memory.rs` (add `decode_array_low_bytes`)
- Modify: `crates/kali_cli/src/bin/cmd_run.rs:257-258` (flush bytes)
- Test: `crates/kali_cli/tests/binary_stdout_runtime.rs` (create)

**Interfaces:**
- Consumes: `Kali.writeStdoutBytes` emission (Task 2); `guest_memory`/`__heap`/array layout (`host/memory.rs`).
- Produces: `KaliHostState.stdout_bytes: Vec<u8>`; `RunOutcome.stdout_bytes: Vec<u8>`; host import `stdout_write_bytes(handle: i64)` appending each array element's low byte to the sink; `kali run` flushing raw bytes after text stdout.

- [ ] **Step 1: Write the failing end-to-end test**

Create `crates/kali_cli/tests/binary_stdout_runtime.rs`:

```rust
use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn write_stdout_bytes_emits_raw_bytes() {
    // Emits the 5 ASCII bytes for "P4\n4 " — includes 0x0A and 0x20 to prove
    // arbitrary (non-alphanumeric) bytes survive the sink verbatim.
    let src = "const out = new Array(5);\n\
        out[0] = 80; out[1] = 52; out[2] = 10; out[3] = 52; out[4] = 32;\n\
        Kali.writeStdoutBytes(out);";
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.ts");
    fs::write(&path, src).expect("write");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, vec![80u8, 52, 10, 52, 32]);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_cli --test binary_stdout_runtime`
Expected: FAIL — no `stdout_write_bytes` host import registered (link error) and no byte flush.

- [ ] **Step 3: Add the `stdout_bytes` sink field**

In `crates/kali_runtime/src/state.rs`, beside `pub stdout: String,` (line 29) add `pub stdout_bytes: Vec<u8>,`; in its initializer (line 88) add `stdout_bytes: Vec::new(),`.
In `crates/kali_runtime/src/outcome.rs`, beside `pub stdout: String,` (line 15) add `pub stdout_bytes: Vec<u8>,`.
In `crates/kali_runtime/src/execute.rs`, every struct literal that sets `stdout: state.stdout.clone()` (and the `stdout: String::new()` at line 110) must also set `stdout_bytes: state.stdout_bytes.clone()` (or `Vec::new()` for the empty case). Update all ~9 sites. Fix any other `RunOutcome { .. }` constructors the compiler flags.

- [ ] **Step 4: Add the array-handle byte decoder in `host/memory.rs`**

Model on `decode_string_handle_bytes` (line 45) and the `__heap`/`guest_memory` helpers. Array layout is `[len@+0 as i64][elem_i@+8+i*8 as i64]`; each element contributes its low byte:

```rust
/// Decodes an array handle (an `i64` pointer into `__heap`) into its element
/// low-byte stream: reads the `i64` length at offset 0, then `len` `i64`
/// elements at offsets 8, 16, …, masking each to its low 8 bits. Used by
/// `stdout_write_bytes` to emit a byte buffer built in the array lane.
pub(crate) fn decode_array_low_bytes(
    caller: &mut Caller<'_, KaliHostState>,
    handle: i64,
) -> wasmtime::Result<Vec<u8>> {
    if handle <= 0 {
        return Ok(Vec::new());
    }
    let memory = guest_memory(caller)?;
    let base = handle as usize;
    let mut len_buf = [0u8; 8];
    memory.read(&mut *caller, base, &mut len_buf)
        .map_err(|e| wasmtime::Error::msg(format!("array length read failed: {e}")))?;
    let len = i64::from_le_bytes(len_buf).max(0) as usize;
    let mut out = Vec::with_capacity(len);
    let mut elem = [0u8; 8];
    for i in 0..len {
        let off = base + 8 + i * 8;
        memory.read(&mut *caller, off, &mut elem)
            .map_err(|e| wasmtime::Error::msg(format!("array element read failed: {e}")))?;
        out.push((i64::from_le_bytes(elem) & 0xFF) as u8);
    }
    Ok(out)
}
```

(Verify `memory.read`'s exact signature against the existing reads in `host/memory.rs` — match how `decode_string_handle_bytes` reads guest memory, including the `Caller` borrow form.)

- [ ] **Step 5: Register the `stdout_write_bytes` host import**

In `crates/kali_runtime/src/host/imports_default.rs`, add a registration block (mirror `console_log`, lines 8–18; gate on the same `HostOperation::Console`):

```rust
    linker
        .func_wrap(
            "kali:rt",
            "stdout_write_bytes",
            |mut caller: Caller<'_, KaliHostState>, handle: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let bytes = decode_array_low_bytes(&mut caller, handle)?;
                caller.data_mut().stdout_bytes.extend_from_slice(&bytes);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("stdout_write_bytes", error))?;
```

(Ensure `decode_array_low_bytes` is in scope — add a `use`/`pub(crate)` re-export matching how `decode_string_handle_bytes` is imported here.)

- [ ] **Step 6: Flush raw bytes in `cmd_run.rs`**

In `crates/kali_cli/src/bin/cmd_run.rs`, after the text flush at line 257–258, add:

```rust
                    if !outcome.stdout_bytes.is_empty() {
                        use std::io::Write;
                        let mut out = std::io::stdout();
                        let _ = out.write_all(&outcome.stdout_bytes);
                        let _ = out.flush();
                    }
```

(Place it inside the same branch that prints `outcome.stdout`, immediately after the `print!`. Confirm the surrounding control flow; the text `print!` and the byte `write_all` both target stdout so ordering is text-then-bytes.)

- [ ] **Step 7: Run the test to verify it passes**

Run: `cargo test -p kali_cli --test binary_stdout_runtime`
Expected: PASS — stdout is exactly `[80,52,10,52,32]`.

- [ ] **Step 8: Full sink-threading + no-regression check**

Run: `cargo test -p kali_runtime -p kali_cli`
Expected: PASS. If any `RunOutcome`/state constructor was missed, the compiler flags it — add `stdout_bytes` there.

- [ ] **Step 9: Commit**

```bash
cargo fmt --all
git add crates/kali_runtime/src crates/kali_cli/src/bin/cmd_run.rs crates/kali_cli/tests/binary_stdout_runtime.rs
git commit -m "feat(runtime): host-only binary stdout sink + stdout_write_bytes import; kali run flushes raw bytes"
```

---

## Task 4: mandelbrot fixture, golden PBM, and canonical-output test

**Files:**
- Create: `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts`
- Create: `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.expected.pbm`
- Create: `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.json`
- Create: `crates/kali_cli/tests/clbg_mandelbrot_runtime.rs`

**Interfaces:**
- Consumes: bitwise lane (Task 1), `Kali.writeStdoutBytes` + byte sink (Tasks 2–3).
- Produces: a fixture whose `kali run` stdout equals the canonical PBM bytes.

- [ ] **Step 1: Generate the golden PBM with an independent reference**

Write the reference generator to the scratchpad (NOT into the repo) and run it to produce the golden asset. It implements the canonical algorithm independently of kali:

```bash
cat > /tmp/claude-1000/-workspace/*/scratchpad/mandel_ref.js <<'EOF'
const n = 200;
const bytes = [];
const header = `P4\n${n} ${n}\n`;
for (const ch of header) bytes.push(ch.charCodeAt(0));
for (let y = 0; y < n; y++) {
  const Ci = 2.0 * y / n - 1.0;
  let byte = 0, bits = 0;
  for (let x = 0; x < n; x++) {
    const Cr = 2.0 * x / n - 1.5;
    let Zr = 0, Zi = 0, Tr = 0, Ti = 0;
    for (let i = 0; i < 50; i++) {
      Zi = 2 * Zr * Zi + Ci;
      Zr = Tr - Ti + Cr;
      Tr = Zr * Zr;
      Ti = Zi * Zi;
      if (Tr + Ti > 4.0) break;
    }
    byte = (byte << 1) | (Tr + Ti <= 4.0 ? 1 : 0);
    if (++bits === 8) { bytes.push(byte); byte = 0; bits = 0; }
  }
}
require('fs').writeFileSync(process.argv[1], Buffer.from(bytes));
EOF
node /tmp/claude-1000/-workspace/*/scratchpad/mandel_ref.js crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.expected.pbm
```

(Use the actual scratchpad path from the session. This produces an 11 + 5000 = 5011-byte file.)

- [ ] **Step 2: Write the fixture `.ts`**

Create `crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts` exactly matching the reference algorithm and header, using only supported surface (no `?:` — an explicit `if`):

```ts
// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// mandelbrot — TS port normalized to Kali (no intrinsic tuning). Retains upstream attribution.
function mandelbrot(n) {
  const out = new Array(11 + n * n / 8);
  out[0] = 80; out[1] = 52; out[2] = 10;                 // "P4\n"
  out[3] = 50; out[4] = 48; out[5] = 48; out[6] = 32;    // "200 "
  out[7] = 50; out[8] = 48; out[9] = 48; out[10] = 10;   // "200\n"
  let p = 11;
  for (let y = 0; y < n; y = y + 1) {
    const Ci = 2.0 * y / n - 1.0;
    let byte = 0;
    let bits = 0;
    for (let x = 0; x < n; x = x + 1) {
      const Cr = 2.0 * x / n - 1.5;
      let Zr = 0.0; let Zi = 0.0; let Tr = 0.0; let Ti = 0.0;
      for (let i = 0; i < 50; i = i + 1) {
        Zi = 2.0 * Zr * Zi + Ci;
        Zr = Tr - Ti + Cr;
        Tr = Zr * Zr;
        Ti = Zi * Zi;
        if (Tr + Ti > 4.0) { break; }
      }
      let bit = 0;
      if (Tr + Ti <= 4.0) { bit = 1; }
      byte = (byte << 1) | bit;
      bits = bits + 1;
      if (bits === 8) {
        out[p] = byte;
        p = p + 1;
        byte = 0;
        bits = 0;
      }
    }
  }
  Kali.writeStdoutBytes(out);
}
mandelbrot(200);
```

- [ ] **Step 3: Write the runtime test (expected to fail until validated)**

Create `crates/kali_cli/tests/clbg_mandelbrot_runtime.rs` (mirror `clbg_nbody_runtime.rs`):

```rust
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

#[test]
fn mandelbrot_runs_and_matches_canonical_output() {
    let source = fixture("mandelbrot-benchmark-v1.ts");
    let expected = fs::read(fixture("mandelbrot-benchmark-v1.expected.pbm")).expect("read golden");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), expected.len(), "PBM byte length mismatch");
    assert_eq!(output.stdout, expected, "PBM bytes differ from canonical");
}

#[test]
fn mandelbrot_metadata_is_consistent() {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(fixture("mandelbrot-benchmark-v1.json")).expect("read metadata"),
    )
    .expect("parse metadata");
    assert_eq!(meta["benchmark"], "mandelbrot");
    assert_eq!(meta["version"], 1);
    assert_eq!(meta["sourceFile"], "mandelbrot-benchmark-v1.ts");
    assert_eq!(
        meta["buildModes"],
        serde_json::json!(["--fast", "--release", "--release-advanced"])
    );
    let src = fs::read(fixture("mandelbrot-benchmark-v1.ts")).expect("read source");
    let digest = format!("sha256-{:x}", Sha256::digest(&src));
    assert_eq!(meta["sourceSha256"], digest, "metadata sha256 must match source");
}
```

- [ ] **Step 4: Run the fixture directly to validate compile + fuel (Risks 1–2)**

Run: `cargo build -p kali_cli && ./target/debug/kali run crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts > /tmp/claude-1000/-workspace/scratch_mandel.pbm; echo "exit=$?"; wc -c /tmp/claude-1000/-workspace/scratch_mandel.pbm`
Expected: `exit=0` and byte count `5011`. If it fails to compile because an accumulator landed on the i64 path (Risk 2), initialize `Zr/Zi/Tr/Ti` so inference marks them float (they are seeded by `2.0*...` and `/`); if it traps on fuel (Risk 1), reduce `n` to `128` (or `64`), regenerate the golden with the same `n`, and update the header bytes in the `.ts`.

- [ ] **Step 5: Create the metadata JSON**

Compute the source sha and write `mandelbrot-benchmark-v1.json`:

```bash
SHA=$(sha256sum crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts | cut -d' ' -f1)
cat > crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.json <<EOF
{
  "benchmark": "mandelbrot",
  "version": 1,
  "sourceFile": "mandelbrot-benchmark-v1.ts",
  "sourceSha256": "sha256-${SHA}",
  "buildModes": ["--fast", "--release", "--release-advanced"]
}
EOF
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test clbg_mandelbrot_runtime`
Expected: PASS (both tests).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.ts \
        crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.expected.pbm \
        crates/kali_cli/tests/fixtures/benchmarks/mandelbrot-benchmark-v1.json \
        crates/kali_cli/tests/clbg_mandelbrot_runtime.rs
git commit -m "test(cli): vendored mandelbrot fixture with canonical PBM golden output"
```

---

## Task 5: Documentation, maturity rows, memory, and integration

**Files:**
- Modify: `specs/19-feature-maturity.md`
- Modify: `/home/dev/.claude/projects/-workspace/memory/kali-heap-object-lane.md` (+ `MEMORY.md` pointer if a new memory is added)

- [ ] **Step 1: Add maturity rows**

In `specs/19-feature-maturity.md`, add two `Phase 1 MVP` rows near the existing CLBG rows (208–235):
- **Bitwise-integer operators** — `& | ^ << >> >>>` lower to real wasm with JS 32-bit `ToInt32`/`ToUint32` semantics (`>>>` yields uint32); operands the inference marks `f64` are rejected with `E5506` rather than the former silent `I64Add`. Evidence: `bitwise_operators_runtime.rs` + the mandelbrot fixture's bit-packing.
- **Host-only binary stdout (`Kali.writeStdoutBytes`)** — an array of byte values is written verbatim to stdout via the `stdout_write_bytes` host import into a `Vec<u8>` sink that `kali run` flushes; browser backend gates the intrinsic (host-only). Evidence: `binary_stdout_runtime.rs` + `clbg_mandelbrot_runtime.rs`.
- Extend the optimization-evidence-lane row (235) to name **mandelbrot** as the fixture exercising the bitwise + binary-stdout lanes (execution-correctness coverage, not a throughput claim).

- [ ] **Step 2: Update memory**

Append to `/home/dev/.claude/projects/-workspace/memory/kali-heap-object-lane.md` (or add a sibling memory `kali-bitwise-and-binary-stdout-lane.md` with a `MEMORY.md` pointer): mandelbrot shipped as the 4th CLBG fixture; the bitwise-`I64Add` miscompile is closed; the dual-sink host-only binary-stdout surface (`Kali.writeStdoutBytes` → conditional `stdout_write_bytes` import) exists; browser binary stdout + compound bitwise-assign (`<<=` etc.) remain deferred follow-ups.

- [ ] **Step 3: Run the full verification gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli && cargo fmt --all --check`
Expected: green, fmt clean.

- [ ] **Step 4: Commit**

```bash
git add specs/19-feature-maturity.md
git commit -m "docs(spec): maturity rows for bitwise-integer + host-only binary-stdout lanes (mandelbrot)"
```

- [ ] **Step 5: Push a PR and merge** (per memory `kali-integration-convention`)

```bash
git push -u origin mandelbrot-bitwise-lane
gh pr create --fill --title "mandelbrot CLBG fixture: bitwise-integer lane + faithful binary PBM stdout"
```

Then merge once CI/review is green.

---

## Self-Review

**Spec coverage:**
- Bitwise lane + JS-32-bit + f64 reject → Task 1. ✓
- `Kali.writeStdoutBytes` recognition + conditional import → Task 2. ✓
- Dual-sink runtime + host import + array decode + CLI flush + browser gate → Task 3 (+ gate diagnostic in Task 2 Step 5). ✓
- Fixture + golden + reference-generation + metadata + tests → Task 4. ✓
- Maturity rows + memory + verification + integration → Task 5. ✓
- Risks 1–3 (fuel, f64 inference, reject path) → Task 4 Step 4 + Task 1. Risk 4 (byte-sink threading) → Task 3 Step 8. Risk 5 (byte masking) → Task 3 Step 4 (`& 0xFF`). ✓

**Placeholder scan:** No TBD/TODO; every code step carries real code. The golden PBM sha/bytes are generated by an exact script (Task 4 Step 1), not hand-authored — this is derived golden data, not a placeholder.

**Type consistency:** `stdout_bytes: Vec<u8>` used identically across `state.rs`/`outcome.rs`/`execute.rs`/host import. `stdout_write_bytes_import_index: Option<u32>` and `is_kali_write_stdout_bytes_call`/`program_uses_stdout_write_bytes`/`decode_array_low_bytes` names are consistent across Tasks 2–3. `emit_bitwise(function, op, left, right)` signature consistent within Task 1.

**Known implementation-time verifications (flagged inline, not placeholders):** exact `LirModule`/node traversal shape in `program_uses_stdout_write_bytes` (mirror `program_uses_env_set`); the emit-context struct name/construction site for threading the index (grep `env_set_import_index`); the `call.rs` argument child offset; `memory.read` borrow form. Each step names the existing symbol to mirror.
