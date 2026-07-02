# Float Console Output + Reserved Glue Names Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the two follow-ups from the browser-bundle work: (1) runtime float values reaching `console.*` or string concatenation currently emit **type-invalid wasm** (`call[0] expected type i64, found f64.div`) — `console.log(7 / 2)`, `console.log(1.5 + 2)`, and `console.log("v: " + (7 / 2))` all fail to instantiate; (2) a user export named `start`/`load`/`loadWithImports`/`loadDynamicImport` silently produces an unloadable browser bundle — make it a build-time error.

**Architecture:** Root cause of (1), established by systematic investigation: the repr system correctly keeps floats as raw `f64` in locals/params/returns/arrays and adapts them at comparisons/truthiness/`toFixed` — but the i64 tagged-value domain has **no float encoding and no f64→string bridge**, and two emit seams pass `Float`-shaped operands into i64-typed calls unconverted: the console-import runtime path (`emit/call.rs:117-127`, plus the `console.assert` message path at `:84-90`) and `emit_as_string` (`emit/operators.rs:531-540`, string `+`). Fix: one new unconditional host import `kali:rt float_to_string (f64) -> i64` (string handle, JS `String(number)` semantics) inserted at import index 20 (bumping `COVERAGE_HIT_IMPORT_INDEX`/`FUNCTION_INDEX_OFFSET` 20→21 — all conditional-import bookkeeping is expressed relative to those constants and shifts automatically), plus `Float`-arm adaptations at the two seams. The import must land in the wasmtime host and ALL FOUR hand-mirrored JS import lists in the same commit, or every existing bundle/harness test fails with a LinkError. (2) is a scoped check in `write_browser_bundle_files` where the export list is already in hand.

**Tech Stack:** Rust (`kali_codegen` wasm emission via wasm-encoder, `kali_runtime` wasmtime host, `kali_cli` glue templates), JS glue mirrors, node harness lane, gated Chromium lane.

**Empirically established before planning** (scratch reproductions, all against `target/debug/kali`): every failing/working case below was observed, not assumed. Working today: float locals/params/returns/arrays, `toFixed`, float comparisons/truthiness, statically-rendered `console.log(1.5)`. Failing today (wasm validation): `console.log(7 / 2)`, `console.log(1.5 + 2)`, `const x = 7/2; console.log(x)`, `console.log("v: " + (7 / 2))`, same inside functions. Also discovered, OUT OF SCOPE (record only): template literals print their raw source (`` console.log(`v: ${7 / 2}`) `` → `v: ${7 / 2}`).

## Global Constraints

- The new import's wire name is exactly `"float_to_string"` in the `kali:rt` namespace, type `(f64) -> i64` (returns a guest string handle).
- ATOMICITY: Task 1 must change, in ONE commit: `kali_codegen` (emission), `kali_runtime/src/host/imports_default.rs` (wasmtime host), and all four JS mirror lists (`cmd_build.rs` ESM + CJS `defaultImportObject`, `harness.rs` both `importObject`s). Any split leaves every bundle/harness lane failing with instantiation LinkErrors.
- Import indices are ABI inside the emitter: `FLOAT_TO_STRING_IMPORT_INDEX = 20`; `COVERAGE_HIT_IMPORT_INDEX` and `FUNCTION_INDEX_OFFSET` both become 21. Do not reorder any existing import.
- Host formatting semantics (`format_js_number`): `NaN` → `"NaN"`, `+∞` → `"Infinity"`, `-∞` → `"-Infinity"`, `±0.0` → `"0"`, else Rust `format!("{value}")` (shortest round-trip; matches JS for ordinary magnitudes; known divergence at |x| ≥ 1e21 where JS uses exponent notation — documented, not reproduced). JS mirrors use native `String(value)`.
- Reserved glue export names are exactly: `load`, `loadWithImports`, `loadDynamicImport`, `start`. Collision is a build ERROR with code `e5::INVALID_EXPORT_SURFACE` (E5511).
- The glue templates in `cmd_build.rs` and the two harness lists in `harness.rs` are inside Rust `format!` raw strings — literal JS braces must be doubled `{{ }}`.
- `cargo test --workspace` is NOT a usable gate (pre-existing chromium-sandbox failures). Gates are the named lanes per task.
- Repo hygiene before every commit: `cargo fmt` no diff; `cargo clippy -p kali_codegen -p kali_runtime -p kali_cli --tests -- -D warnings` clean.
- If `cargo test -p kali_codegen` fails ONLY on tests asserting concrete import counts / function indices shifted by the new import, updating those expectations is part of Task 1 — mechanical, but list every such edit in the report.

---

### Task 1: `float_to_string` end-to-end (emitter + host + all four JS mirrors) + runtime integration test

**Files:**
- Modify: `crates/kali_codegen/src/lib.rs` (index constants)
- Modify: `crates/kali_codegen/src/lower.rs` (type + import registration + comment)
- Modify: `crates/kali_codegen/src/emit/call.rs` (console argument helper + two call sites + drop guards)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`emit_as_string` float arm)
- Modify: `crates/kali_runtime/src/host/imports_default.rs` (host import + `format_js_number`)
- Modify: `crates/kali_cli/src/bin/cmd_build.rs` (ESM + CJS mirror entries)
- Modify: `crates/kali_runtime/src/browser/harness.rs` (both mirror entries)
- Create: `crates/kali_cli/tests/float_console_runtime.rs`

**Interfaces:**
- Consumes: existing `EmittedValue { produced, shape }`, `ValueShape::Float`, `alloc_guest_string`, `host_import_error`, the four mirror lists' `allocGuestString` helper.
- Produces: `FLOAT_TO_STRING_IMPORT_INDEX: u32 = 20`; host + 4 mirrors provide `kali:rt float_to_string`; `console.log/error/warn/info/debug`, `console.assert` messages, and string `+` accept Float-shaped operands.

- [ ] **Step 1: Write the failing integration test**

Create `crates/kali_cli/tests/float_console_runtime.rs` (modeled on `array_at.rs`):

```rust
//! Runtime float values print through console with JS `String(number)`
//! semantics. Regression for the emitter passing raw f64 into the i64-typed
//! console imports (wasm validation failure: "expected type i64, found
//! f64.div").
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_fixture(source: &str) -> (bool, String, String) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn run_prints_runtime_float_division_results() {
    let (ok, stdout, stderr) = run_fixture(
        "console.log(7 / 2);\nconsole.log(6 / 2);\nconsole.log(1.5 + 2);\nconst x = 7 / 2;\nconsole.log(x);\n",
    );
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "3.5\n3\n3.5\n3.5\n");
}

#[test]
fn run_prints_js_special_float_values() {
    let (ok, stdout, stderr) = run_fixture(
        "console.log(7 / 0);\nconsole.log(-7 / 0);\nconsole.log(0 / 0);\nconsole.log(0 / -1);\n",
    );
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "Infinity\n-Infinity\nNaN\n0\n");
}

#[test]
fn run_concatenates_runtime_floats_into_strings() {
    let (ok, stdout, stderr) = run_fixture("console.log(\"v: \" + (7 / 2));\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "v: 3.5\n");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --test float_console_runtime`
Expected: all 3 FAIL — stderr contains `error[E4201]: failed to load WASM module` (the emitted wasm is type-invalid today).

- [ ] **Step 3: Emitter — index constants, type, import**

3a. `crates/kali_codegen/src/lib.rs`: change

```rust
const FLOAT_TO_FIXED_IMPORT_INDEX: u32 = 19;
const COVERAGE_HIT_IMPORT_INDEX: u32 = 20;
const FUNCTION_INDEX_OFFSET: u32 = 20;
```

to

```rust
const FLOAT_TO_FIXED_IMPORT_INDEX: u32 = 19;
const FLOAT_TO_STRING_IMPORT_INDEX: u32 = 20;
const COVERAGE_HIT_IMPORT_INDEX: u32 = 21;
const FUNCTION_INDEX_OFFSET: u32 = 21;
```

3b. `crates/kali_codegen/src/lower.rs`: after the Type 8 registration (`// Type 8: float_to_fixed ...` block ending ~line 170), add:

```rust
    // Type 9: float_to_string `(f64) -> i64` (value -> string handle, JS
    // `String(number)` semantics).
    type_section
        .ty()
        .function(vec![ValType::F64], vec![ValType::I64]);
```

After `import_section.import("kali:rt", "float_to_fixed", EntityType::Function(8));` add:

```rust
    import_section.import("kali:rt", "float_to_string", EntityType::Function(9));
```

And update the bookkeeping comment above `int_to_string` (currently "Three unconditional runtime helpers occupy fixed import indices 17, 18 and 19 … all expressed against COVERAGE_HIT_IMPORT_INDEX = 20 …") to read "Four unconditional runtime helpers occupy fixed import indices 17 through 20 … all expressed against COVERAGE_HIT_IMPORT_INDEX = 21 …" and append to its trailing list: `float_to_string is (f64) -> i64 (type 9).`

3c. `crates/kali_codegen/src/lower.rs` ~line 262: the repr-directed function-type dedup assigns indices with a hardcoded base equal to the number of fixed types — change

```rust
            let idx = function_types.len() as u32 + 9;
```

to

```rust
            let idx = function_types.len() as u32 + 10;
```

(Type 9 is now `float_to_string`'s; function signature types start at 10.)

- [ ] **Step 4: Emitter — console seam**

In `crates/kali_codegen/src/emit/call.rs` (bring `FLOAT_TO_STRING_IMPORT_INDEX` into scope the same way the file imports its other `*_IMPORT_INDEX` constants):

4a. Add this private helper to the same `impl` block that contains the console handling:

```rust
    /// Emit `id` as a console-import argument: always leaves exactly one i64
    /// (tagged scalar or string handle) on the stack. Float-shaped values are
    /// stringified via the `float_to_string` host import — the i64 value
    /// domain has no float encoding, so passing a raw f64 would emit
    /// type-invalid wasm.
    fn emit_console_argument(&mut self, function: &mut Function, id: LirNodeId) {
        let emitted = self.emit_node(function, id, true);
        if !emitted.produced {
            function.instruction(&Instruction::I64Const(0));
            return;
        }
        if matches!(emitted.shape, ValueShape::Float) {
            function.instruction(&Instruction::Call(FLOAT_TO_STRING_IMPORT_INDEX));
        }
    }
```

4b. Replace the console runtime-argument block (currently lines ~117-127):

```rust
            let mut args = node.children.iter().skip(1);
            if let Some(first_arg) = args.next() {
                let _ = self.emit_node(function, *first_arg, true);
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
```

with:

```rust
            let mut args = node.children.iter().skip(1);
            if let Some(first_arg) = args.next() {
                self.emit_console_argument(function, *first_arg);
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                let produced = self.emit_node(function, *arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
```

(The drop-guard also fixes a latent invalid-wasm path when an extra argument produces no value.)

4c. In the `console.assert` handling, replace the runtime message-argument block (currently lines ~84-90):

```rust
                } else if let Some(first_arg) = message_args.first().copied() {
                    let _ = self.emit_node(function, first_arg, true);
                    function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                    for arg in message_args.iter().skip(1) {
                        let _ = self.emit_node(function, *arg, true);
                        function.instruction(&Instruction::Drop);
                    }
                }
```

with:

```rust
                } else if let Some(first_arg) = message_args.first().copied() {
                    self.emit_console_argument(function, first_arg);
                    function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                    for arg in message_args.iter().skip(1) {
                        let produced = self.emit_node(function, *arg, true);
                        if produced.produced {
                            function.instruction(&Instruction::Drop);
                        }
                    }
                }
```

- [ ] **Step 5: Emitter — string seam**

In `crates/kali_codegen/src/emit/operators.rs`, replace `emit_as_string` (currently lines ~528-540) with:

```rust
    /// Emits `id` as a string handle: if it is already string-valued the emitted
    /// value is a handle; float-shaped values are stringified via
    /// `float_to_string` (JS `String(number)` semantics); otherwise the produced
    /// i64 is coerced to a decimal-string handle via `int_to_string`.
    pub(crate) fn emit_as_string(&mut self, function: &mut Function, id: LirNodeId) {
        let is_string = self.is_string_valued(id);
        let emitted = self.emit_node(function, id, true);
        if !emitted.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        if is_string {
            return;
        }
        if emitted.produced && matches!(emitted.shape, ValueShape::Float) {
            function.instruction(&Instruction::Call(FLOAT_TO_STRING_IMPORT_INDEX));
        } else {
            function.instruction(&Instruction::Call(INT_TO_STRING_IMPORT_INDEX));
        }
    }
```

(Bring `FLOAT_TO_STRING_IMPORT_INDEX` into scope alongside the file's existing `INT_TO_STRING_IMPORT_INDEX` import.)

- [ ] **Step 6: Host import**

In `crates/kali_runtime/src/host/imports_default.rs`, immediately after the `float_to_fixed` registration block, add:

```rust
    linker
        .func_wrap(
            "kali:rt",
            "float_to_string",
            |mut caller: Caller<'_, KaliHostState>, value: f64| -> i64 {
                let text = format_js_number(value);
                alloc_guest_string(&mut caller, text.as_bytes()).unwrap_or(0)
            },
        )
        .map_err(|error| host_import_error("float_to_string", error))?;
```

and at the bottom of the file (after `register_default_host_imports`) add:

```rust
/// Format an f64 with JavaScript `String(number)` semantics for the common
/// cases: `NaN`, `Infinity`, `-Infinity`, and ±0 render exactly as JS does;
/// other finite values use Rust's shortest round-trip formatting, which
/// matches JS for ordinary magnitudes. Known divergence: JS switches to
/// exponent notation for |x| >= 1e21 and very small magnitudes, which this
/// phase does not reproduce.
fn format_js_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    if value == 0.0 {
        return "0".to_owned();
    }
    format!("{value}")
}
```

- [ ] **Step 7: All four JS mirrors**

Insert this entry immediately after the `float_to_fixed(value, digits) {{ ... }},` entry in EACH of the four lists — `crates/kali_cli/src/bin/cmd_build.rs` ESM (~line 1547) and CJS (~line 1791), `crates/kali_runtime/src/browser/harness.rs` first list (~line 198) and second list (~line 511) — matching each list's indentation:

```js
    float_to_string(value) {{
      return allocGuestString(new TextEncoder().encode(String(value)));
    }},
```

- [ ] **Step 8: Run the new tests to verify they pass**

Run: `cargo test -p kali_cli --test float_console_runtime`
Expected: PASS — 3/3, exact stdout matches.

- [ ] **Step 9: Regression lanes**

Run each; all green (adjust only index-shift-sensitive kali_codegen test expectations, if any, per Global Constraints):

```bash
cargo test -p kali_codegen
cargo test -p kali_runtime
cargo test -p kali_cli --test imperative_core_runtime
cargo test -p kali_cli --test browser_bundle_toplevel_start
cargo test -p kali_cli --test browser_number_predicates_bundle
cargo test -p kali_cli --test browser_bundle_cjs_source_classes
cargo test -p kali_cli --test browser_cdp_smoke
```

- [ ] **Step 10: Lint, format, commit**

```bash
cargo clippy -p kali_codegen -p kali_runtime -p kali_cli --tests -- -D warnings
cargo fmt
git add -A crates/kali_codegen/src crates/kali_runtime/src crates/kali_cli/src crates/kali_cli/tests/float_console_runtime.rs
git commit -m "feat(codegen,runtime,cli): float_to_string bridge so runtime floats print and concatenate correctly"
```

---

### Task 2: Browser-lane float coverage (bundle glue mirror, end-to-end)

**Files:**
- Create: `crates/kali_cli/tests/browser_bundle_float_console.rs`

**Interfaces:**
- Consumes: `kali_runtime::browser_bundle_harness_script`, `kali_runtime::browser_harness_command_parts_for`, the glue `start()` helper, Task 1's `float_to_string` mirror in the ESM glue.

- [ ] **Step 1: Write the test**

Create `crates/kali_cli/tests/browser_bundle_float_console.rs`:

```rust
//! Runtime floats route through the browser bundle glue's `float_to_string`
//! mirror: a top-level program's division results print with JS semantics
//! when executed via the glue's `start()` helper under the node harness.
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn browser_bundle_prints_runtime_floats_via_start() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        "console.log(7 / 2);\nconsole.log(\"v: \" + (0 / 0));\nconsole.log(7 / 0);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let harness_path = dir.path().join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.start();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3.5\n"), "stdout: {stdout:?}");
    assert!(stdout.contains("v: NaN\n"), "stdout: {stdout:?}");
    assert!(stdout.contains("Infinity\n"), "stdout: {stdout:?}");
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p kali_cli --test browser_bundle_float_console`
Expected: PASS immediately (Task 1 already landed the mirror; this pins the browser/glue path so it cannot regress independently of the wasmtime path — the JS mirror and the Rust host are separate implementations of the same contract). If it FAILS, stop: that is a real divergence between the glue mirror and the host — report it, do not adjust assertions.

- [ ] **Step 3: Gated browser lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
Expected: 5/5 (real Chromium; bundles now carry the extra import + mirror).

- [ ] **Step 4: Lint, format, commit**

```bash
cargo clippy -p kali_codegen -p kali_runtime -p kali_cli --tests -- -D warnings
cargo fmt
git add crates/kali_cli/tests/browser_bundle_float_console.rs
git commit -m "test(cli): pin JS-side float_to_string mirror via a real browser bundle run"
```

---

### Task 3: Reserved glue-name build diagnostic

**Files:**
- Modify: `crates/kali_cli/src/bin/cmd_build.rs` (const + check in `write_browser_bundle_files`)
- Create: `crates/kali_cli/tests/browser_bundle_reserved_export_names.rs`

**Interfaces:**
- Consumes: `build::collect_browser_bundle_exports` result (already bound as `exports` in `write_browser_bundle_files`, ~line 1187), `Diagnostic::error`, `e5::INVALID_EXPORT_SURFACE`.
- Produces: `kali build --bundle` fails with E5511 when a user export collides with a glue-reserved name, for both ESM and CJS.

- [ ] **Step 1: Write the failing test**

Create `crates/kali_cli/tests/browser_bundle_reserved_export_names.rs`:

```rust
//! The browser bundle glue reserves `load`, `loadWithImports`,
//! `loadDynamicImport`, and `start` — a user export with one of those names
//! previously built green but emitted an unloadable module (duplicate
//! declaration at import time). It must be a build-time error.
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn build_bundle_with_export(name: &str) -> (bool, String) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        format!(
            "// kali-tree-shake: {name}\nexport async function {name}(left, right) {{\n  return left - left + right - right;\n}}\n"
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn build_rejects_exports_that_collide_with_reserved_glue_names() {
    for name in ["load", "loadWithImports", "loadDynamicImport", "start"] {
        let (ok, stderr) = build_bundle_with_export(name);
        assert!(!ok, "[{name}] build unexpectedly succeeded");
        assert!(stderr.contains("E5511"), "[{name}] stderr: {stderr}");
        assert!(stderr.contains(name), "[{name}] stderr: {stderr}");
    }
}

#[test]
fn build_accepts_non_reserved_export_names() {
    let (ok, stderr) = build_bundle_with_export("startup");
    assert!(ok, "stderr: {stderr}");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_cli --test browser_bundle_reserved_export_names`
Expected: `build_rejects_exports_that_collide_with_reserved_glue_names` FAILS on `load` ("build unexpectedly succeeded") — the collision currently builds green. `build_accepts_non_reserved_export_names` passes.

- [ ] **Step 3: Implement the check**

In `crates/kali_cli/src/bin/cmd_build.rs`:

3a. Add near `generate_browser_bundle_js` (module level, adjacent to the templates that own these names):

```rust
/// Export names the emitted bundle glue defines itself; a user export with one
/// of these names would produce a duplicate declaration (ESM: SyntaxError at
/// import time) or silently shadow the helper (CJS).
const RESERVED_GLUE_EXPORT_NAMES: [&str; 4] = ["load", "loadWithImports", "loadDynamicImport", "start"];
```

3b. In `write_browser_bundle_files`, immediately after the `let exports = build::collect_browser_bundle_exports(...)` binding (~line 1187), add:

```rust
    if let Some(reserved) = exports
        .iter()
        .find(|export| RESERVED_GLUE_EXPORT_NAMES.contains(&export.name.as_str()))
    {
        return Err(vec![Diagnostic::error(
            e5::INVALID_EXPORT_SURFACE as u32,
            format!(
                "export `{}` collides with a name the browser bundle glue reserves ({}); rename the export",
                reserved.name,
                RESERVED_GLUE_EXPORT_NAMES.join(", ")
            ),
        )]);
    }
```

(The function already returns `Result<_, Vec<Diagnostic>>` and its callers propagate compile diagnostics with `?`, so the error flows through the standard envelope for both text and `--output json`.)

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test browser_bundle_reserved_export_names`
Expected: PASS — 2/2.

- [ ] **Step 5: Regression lanes**

```bash
cargo test -p kali_cli --test browser_bundle_toplevel_start
cargo test -p kali_cli --test browser_number_predicates_bundle
cargo test -p kali_cli --test browser_bundle_cjs_source_classes
```

Expected: all green (no existing fixture exports a reserved name).

- [ ] **Step 6: Lint, format, commit**

```bash
cargo clippy -p kali_codegen -p kali_runtime -p kali_cli --tests -- -D warnings
cargo fmt
git add crates/kali_cli/src/bin/cmd_build.rs crates/kali_cli/tests/browser_bundle_reserved_export_names.rs
git commit -m "feat(cli): reject browser-bundle exports that collide with reserved glue names (E5511)"
```

---

### Task 4: Spec note + whole-plan verification

**Files:**
- Create: `docs/superpowers/specs/2026-07-02-float-console-and-reserved-names.md`

- [ ] **Step 1: Write the spec note**

Create the file with exactly this content:

```markdown
# Float console output + reserved glue names (2026-07-02)

Two production fixes landed together (see the same-named plan for task detail).

**Float console/string bridge.** Root cause: the i64 tagged-value domain has no
float encoding and had no f64→string bridge; the console-import runtime path
and `emit_as_string` (string `+`) passed `Float`-shaped operands into i64-typed
calls unconverted, emitting type-invalid wasm ("expected type i64, found
f64.div"). Division was only the most common float seed — float locals,
params, returns, arrays, comparisons, and `toFixed` were already correct. Fix:
unconditional `kali:rt float_to_string (f64) -> i64` host import (index 20;
`COVERAGE_HIT_IMPORT_INDEX`/`FUNCTION_INDEX_OFFSET` bumped to 21), mirrored in
all four hand-mirrored JS import lists, with `Float`-arm adaptations at both
emit seams. Semantics: JS `String(number)` — `NaN`/`Infinity`/`-Infinity`/`0`
(for ±0) special-cased on the Rust host, shortest round-trip otherwise; JS
mirrors use native `String(value)`. Known divergence (documented on
`format_js_number`): JS exponent notation for |x| ≥ 1e21 is not reproduced.

**Reserved glue export names.** `kali build --bundle` now fails with E5511
when a user export is named `load`, `loadWithImports`, `loadDynamicImport`, or
`start` — previously a green build emitted an unloadable ESM module (duplicate
declaration) or silently shadowed the CJS helper.

**Recorded, out of scope:** template literals currently print their raw source
(`` console.log(`v: ${7 / 2}`) `` prints `v: ${7 / 2}`) — discovered during
this investigation; separate issue.
```

- [ ] **Step 2: Whole-plan verification**

```bash
cargo test -p kali_cli --test float_console_runtime                   # 3/3
cargo test -p kali_cli --test browser_bundle_float_console            # 1/1
cargo test -p kali_cli --test browser_bundle_reserved_export_names    # 2/2
cargo test -p kali_codegen                                            # green
cargo test -p kali_runtime                                            # green
cargo test -p kali_cli --test imperative_core_runtime                 # green
cargo test -p kali_cli --test browser_bundle_toplevel_start           # 2/2
cargo test -p kali_cli --test browser_number_predicates_bundle        # 8/8
cargo test -p kali_cli --test browser_bundle_cjs_source_classes       # 8/8
cargo test -p kali_cli --test browser_cdp_smoke                       # 10 passed / 5 ignored
cargo test -p kali_cli --test browser_cdp_smoke -- --ignored          # 5/5
cargo clippy -p kali_codegen -p kali_runtime -p kali_cli --tests -- -D warnings
cargo fmt                                                             # no diff
```

If ANY gate is red, STOP and report — do not commit.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-07-02-float-console-and-reserved-names.md
git commit -m "docs(spec): record float_to_string bridge and reserved glue-name diagnostic"
```

---

## Verification (whole plan)

- All Task 4 Step 2 gates green; gated lane run at least twice across the branch.
- `git status` clean; four commits, each formatted.
