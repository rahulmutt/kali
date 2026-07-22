# Stage P5 — `String()` coercion + TextEncoder/TextDecoder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land runtime `String(x)` coercion and a sound `TextEncoder`/`TextDecoder` roundtrip so the `webBaselineSmoke` fixture runs verbatim byte-for-byte across `kali run` + browser + bundle, retiring the web-baseline parity series.

**Architecture:** Two lanes. (1) `String(<coercible scalar/string>)` routes into the *existing* `emit_as_string` coercion ladder; the `String` deny-set entry is removed and non-coercible/0-arg/multi-arg forms fail closed. (2) A byte-array provenance handle following the project's proven opaque-handle pattern (mirror `AbortHandle`/`Url`/`UrlSearchParams`): a new `Repr::Bytes` axis + per-emitter same-function side-tables + an `admit_bytes_handle_read` flag + one identifier-read choke that denies any escape. `encode` is retyped `Repr::String`→`Repr::Bytes` (closing a confirmed live latent hazard) and its sole existing consumer `crypto.subtle.digest` is migrated to admit `Bytes`; net-new `decode` relabels `Bytes`→`String`.

**Tech Stack:** Rust workspace (`kali_common`, `kali_types`, `kali_codegen`, `kali_cli`); wasm codegen via `wasm-encoder`; integration tests are standalone `crates/kali_cli/tests/*.rs` binaries run through the built `kali` CLI.

## Global Constraints

- **Green baseline gate:** `cargo test --workspace` must stay **0-failed** — measured before AND after the stage, diffed against a clean `main` worktree. No newly-red tests.
- **fmt/clippy clean:** `cargo fmt --all --check` and `cargo clippy --workspace --all-targets` clean.
- **No host imports added:** both lanes are pure linear-memory work. Do NOT touch the 4 hand-mirrored `kali:rt` import lists.
- **Fail closed, never miscompile:** any shape/position outside the proven path emits `E5506` (`e5::FEATURE_UNAVAILABLE`), never a silent `0` or a divergent value.
- **Allowlist at the choke, not denylist of sinks:** byte-array escape is closed at the single identifier-read choke, admitted only by named consumers.
- **No new `ValueShape` variant; no tracing GC.** Provenance survives bindings via the `Repr` axis, per project precedent.
- **Branch:** `soundness-stage-p5` (already created off `main` `694607bb2`; spec committed).
- **Integration convention:** when the stage is complete + whole-stage-reviewed, push a PR and self-merge (`gh` authed as `rahulmutt`; run `gh auth setup-git` if git can't read credentials).

**Confirmed current behavior (probed 2026-07-22, clean branch):**
- `String(40n+2n)` → `E5506`.
- `const e=new TextEncoder(); const b=e.encode("hi"); new TextDecoder().decode(b)` → prints `0` (bound encode + decode both silently miscompile to 0).
- `console.log(new TextEncoder().encode("hi"))` → prints `hi` (inline encode result is a `Repr::String` — the live latent hazard; JS prints `104,105`).

---

## File Structure

- `crates/kali_common/src/repr.rs` — add `Repr::Bytes` variant + arms in the classifier chains (`repr.rs:212-280`).
- `crates/kali_types/src/repr_infer.rs` — add `bytes_bindings` set; retype the `encode` seed from `Repr::String` to `Repr::Bytes`; recognize a bound-receiver `enc.encode(...)`; recognize `dec.decode(...)`; seed `Repr::Bytes` for bound `encode` bindings.
- `crates/kali_types/src/late_host.rs` — `is_new_text_decoder` mirroring `is_new_text_encoder` (`late_host.rs:210`).
- `crates/kali_codegen/src/emitter.rs` — `bytes_locals`, `text_encoder_locals`, `text_decoder_locals: BTreeSet<String>`; `admit_bytes_handle_read: bool`; recognizers `is_bytes_handle`, `is_text_encoder_marker`, `is_text_decoder_marker`.
- `crates/kali_codegen/src/emit/control_flow.rs` — declarator lane: insert into the three side-tables; identifier-read choke (~1744): deny bytes/marker reads unless admitted.
- `crates/kali_codegen/src/emit/call.rs` — String() coercion arm; bound-receiver encode; `decode` dispatch; `digest` + `decode` operand `admit_bytes_handle_read`.
- `crates/kali_codegen/src/intrinsics/host.rs` — extend `is_text_encoder_encode` (`host.rs:327`) for a bound receiver; add `is_text_decoder_decode`.
- `crates/kali_codegen/src/intrinsics/string.rs` — (String() lives in `emit/call.rs`; no change unless a helper lands here).
- `crates/kali_cli/tests/soundness_textcodec.rs` — NEW standalone test binary (mirror `soundness_url.rs` helpers).
- `crates/kali_cli/tests/runtime_smoke/build.rs` — flip the `build_emits_browser_bundle_web_baseline_primitives*` assertions.
- `docs/superpowers/followups/stageD-triage.md` — §8.6 SHIPPED entry + P5 residuals.

---

## Task 1: `String(x)` runtime coercion lane

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (add arm before `deny_placeholder_lowering` call at `~3573`; remove `"String"` from `deny_placeholder_lowering` at `~3635`)
- Test: `crates/kali_cli/tests/soundness_textcodec.rs` (create)

**Interfaces:**
- Consumes: `self.emit_as_string(function, id)` (`emit/operators.rs:1591`) — the coercion ladder (string passthrough / boolean / `float_to_string` / `int_to_string`); `self.deny_e5506(function, msg) -> EmittedValue` (stack-polymorphic `Unreachable` E5506); `self.object_shape_of_node(id) -> Option<..>` (object detector, `emit/call.rs:70` usage).
- Produces: `String(<coercible>)` → `EmittedValue { produced: true, shape: ValueShape::String }`; all other `String(...)` forms → `E5506`.

- [ ] **Step 1: Write the failing tests** — create `crates/kali_cli/tests/soundness_textcodec.rs`:

```rust
// Stage P5 — String() coercion + TextEncoder/TextDecoder soundness pins.
use std::fs;
use std::process::{Command, Output};
use tempfile::tempdir;

fn kali_bin() -> String {
    env!("CARGO_BIN_EXE_kali").to_string()
}

fn run(source: &str) -> Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// Compile+run, assert success, return trimmed stdout.
fn run_ok(source: &str) -> String {
    let out = run(source);
    assert!(
        out.status.success(),
        "expected success\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Compile+run, assert fail-closed, return stderr.
fn run_e5506(source: &str) -> String {
    let out = run(source);
    assert!(
        !out.status.success(),
        "expected fail-closed E5506\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("E5506"), "expected E5506, got: {stderr}");
    stderr
}

#[test]
fn string_of_i64_renders_decimal() {
    assert_eq!(run_ok("console.log(String(40n + 2n));"), "42");
}

#[test]
fn string_of_negative_i64_renders_sign() {
    assert_eq!(run_ok("console.log(String(0n - 7n));"), "-7");
}

#[test]
fn string_of_float_renders() {
    assert_eq!(run_ok("console.log(String(3.5));"), "3.5");
}

#[test]
fn string_of_boolean_renders_word() {
    assert_eq!(run_ok("console.log(String(1n === 1n));"), "true");
}

#[test]
fn string_of_string_is_identity() {
    assert_eq!(run_ok("console.log(String('hi'));"), "hi");
}

#[test]
fn string_of_object_fails_closed() {
    run_e5506("const o = { a: 1n }; console.log(String(o));");
}

#[test]
fn string_zero_arg_fails_closed() {
    run_e5506("console.log(String());");
}

#[test]
fn string_multi_arg_fails_closed() {
    run_e5506("console.log(String(1n, 2n));");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test soundness_textcodec string_ -- --nocapture`
Expected: the `string_of_*` success tests FAIL (currently `String` is E5506-denied); the `*_fails_closed` tests may already pass (String is denied). This confirms the harness compiles and the coercion is the missing piece.

- [ ] **Step 3: Add the `String(x)` coercion arm** in `crates/kali_codegen/src/emit/call.rs`, immediately BEFORE the `deny_placeholder_lowering` block (before the comment `// Positive DENY-SET:` at `~3565`):

```rust
// Stage P5: `String(<coercible>)` runtime coercion. A bare-identifier
// callee (gate-1 already failed closed if the program binds `String`),
// exactly one argument, whose repr the `emit_as_string` ladder renders
// soundly (string / boolean / float / i64). Everything else — objects,
// arrays, unproven values, 0-arg, multi-arg — fails closed E5506 rather
// than miscompiling (`String(obj)` cannot render `[object Object]`).
if callee_name == "String" && callee_node.children.is_empty() {
    let arg_ids: Vec<LirNodeId> = node.children.iter().skip(1).copied().collect();
    if arg_ids.len() != 1 {
        return self.deny_e5506(
            function,
            "String(...) is supported only with a single scalar/string argument \
             in the current phase (fail-closed)",
        );
    }
    let arg = arg_ids[0];
    if self.object_shape_of_node(arg).is_some() {
        return self.deny_e5506(
            function,
            "String(<object/array>) is unavailable in the current phase: kali \
             cannot render an object's default string form (fail-closed)",
        );
    }
    self.emit_as_string(function, arg);
    return EmittedValue { produced: true, shape: ValueShape::String };
}
```

Note: `emit_as_string` internally emits the arg and applies the ladder; do not `emit_node` the arg first. `object_shape_of_node` guards the array/object case (arrays carry an object shape); if a not-yet-classified exotic value slips past, `emit_as_string`'s own object-rejection (`emit_console_argument_as_string` pattern) is the backstop, but the explicit guard here keeps the diagnostic specific.

- [ ] **Step 4: Remove `"String"` from the deny-set** in `deny_placeholder_lowering` (`emit/call.rs:~3635`):

```rust
// Free-name coercion calls: `Boolean(x)`. `String(x)` is handled by the
// Stage P5 coercion arm above (removed from this deny-set); a `String`
// form that falls through the arm (already E5506'd there) never reaches
// here. `Boolean` remains silent-0 when consumed, so it stays denied.
"Boolean" => callee_node.children.is_empty(),
```

(Delete `String` from the `"String" | "Boolean"` match arm, leaving `"Boolean"`.)

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test soundness_textcodec string_ -- --nocapture`
Expected: all 8 `string_*` tests PASS.

- [ ] **Step 6: Gate + fmt, then commit**

Run: `cargo fmt --all && cargo clippy -p kali_codegen -p kali_cli --all-targets 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/kali_codegen/src/emit/call.rs crates/kali_cli/tests/soundness_textcodec.rs
git commit -m "feat(codegen): P5 Task 1 — String() runtime coercion via emit_as_string; drop String deny-set entry [stageP5]"
```

---

## Task 2: `Repr::Bytes` axis scaffolding

**Files:**
- Modify: `crates/kali_common/src/repr.rs` (add variant at `:18`; add arms in classifier chains `:212-280`)

**Interfaces:**
- Produces: `kali_common::Repr::Bytes` — a new opaque-handle repr variant, inert until Task 3 seeds it. Later tasks compare `repr == Repr::Bytes`.

- [ ] **Step 1: Add the variant** in `crates/kali_common/src/repr.rs` (after `UrlSearchParams` at `~:57`):

```rust
    /// A `TextEncoder().encode(...)` byte-array handle. The i64 is the
    /// argument's `(buf,len)` string handle reinterpreted as contiguous
    /// UTF-8 bytes (zero-copy). Provenance-only: it may be read solely as a
    /// `TextDecoder.decode` / `crypto.subtle.digest` operand; any other read
    /// fails closed at the codegen identifier choke (Stage P5).
    Bytes,
```

- [ ] **Step 2: Add classifier arms** — for each `if repr == Repr::String { ... } ... if repr == Repr::UrlSearchParams { ... }` chain in `repr.rs:212-280` (e.g. `is_scalar`/`is_reference`/display helpers), add a `Repr::Bytes` arm that classifies it exactly as the other opaque handles (`AbortHandle`/`Url`/`UrlSearchParams`) are classified — a reference-like scalar handle, NOT a string. Concretely, wherever `UrlSearchParams` appears in such a chain, add `| Repr::Bytes` to the same arm (opaque i64 handle, not string, not float).

- [ ] **Step 3: Build to verify exhaustiveness** — the compiler forces every `match Repr` to gain a `Bytes` arm:

Run: `cargo build -p kali_common 2>&1 | tail -20`
Expected: either clean, or `non-exhaustive patterns: Bytes not covered` errors pointing at each `match repr` site. Add a `Bytes` arm mirroring the `UrlSearchParams` arm at each until clean.

- [ ] **Step 4: Build the dependents**

Run: `cargo build -p kali_types -p kali_codegen 2>&1 | tail -20`
Expected: clean (no consumer scrutinizes `Bytes` yet).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/repr.rs
git commit -m "feat(common): P5 Task 2 — add inert Repr::Bytes opaque-handle variant [stageP5]"
```

---

## Task 3: `encode` migration — bound receiver, `Repr::Bytes`, digest admit, escape choke

This is the coupled soundness core: retype `encode`→`Repr::Bytes`, recognize the bound receiver, migrate the existing `digest` consumer to admit `Bytes`, and close every `encoded` escape at the identifier choke. It lands atomically so the gate never goes red (digest depends on the old `Repr::String`).

**Files:**
- Modify: `crates/kali_codegen/src/emitter.rs` (side-tables + flag + recognizers)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (`is_text_encoder_encode` bound receiver)
- Modify: `crates/kali_codegen/src/emit/call.rs` (encode result shape context; digest operand admit)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (declarator side-table insert; identifier choke deny)
- Modify: `crates/kali_types/src/repr_infer.rs` (`bytes_bindings`; encode seed `Repr::String`→`Repr::Bytes`; bound-receiver recognition)
- Test: `crates/kali_cli/tests/soundness_textcodec.rs`

**Interfaces:**
- Consumes: `admit_url_handle_read`/`emit_url_receiver_handle` pattern (`emit/url.rs`), `usp_locals`/`is_url_search_params` pattern (`emitter.rs:133/625`), the identifier choke (`emit/control_flow.rs:1744`), `is_text_encoder_ctor`/`text_encoder_encode_new` (`repr_infer.rs:4648/4665`), `crypto_subtle_digest_import_index` operand emit (`emit/call.rs:3203`).
- Produces:
  - `emitter.rs`: `pub(crate) bytes_locals: BTreeSet<String>`, `pub(crate) text_encoder_locals: BTreeSet<String>`, `pub(crate) admit_bytes_handle_read: bool`; `fn is_bytes_handle(&self, name: &str) -> bool` (`self.bytes_locals.contains(name)`), `fn is_text_encoder_marker(&self, name: &str) -> bool`.
  - `encode` (bound or inline) → an i64 byte handle bound as `Repr::Bytes`; `console.log(encode(..))`, `return`, arg, store, `.length`, etc. → `E5506`.
  - `digest(algo, <bytes binding>)` still works (admits the `bytes_locals` read).

- [ ] **Step 1: Write failing tests** — append to `soundness_textcodec.rs`:

```rust
// --- encode provenance (Task 3) ---

#[test]
fn digest_consumes_bound_encode_bytes() {
    // digest over a bound encode result must still succeed (migrated consumer).
    let out = run_ok(
        "const e = new TextEncoder(); const b = e.encode('hi'); \
         const h = crypto.subtle.digest('SHA-256', b); console.log('ok');",
    );
    assert_eq!(out, "ok");
}

#[test]
fn encode_result_cannot_print() {
    // Was: silent `hi` (Repr::String hazard). Now: fail closed.
    run_e5506("const b = new TextEncoder().encode('hi'); console.log(b);");
}

#[test]
fn encode_bound_result_cannot_print() {
    run_e5506("const e = new TextEncoder(); const b = e.encode('hi'); console.log(b);");
}

#[test]
fn encode_result_cannot_return() {
    run_e5506("function f() { const b = new TextEncoder().encode('hi'); return b; } console.log(f());");
}

#[test]
fn encode_result_cannot_concat() {
    run_e5506("const b = new TextEncoder().encode('hi'); console.log('' + b);");
}

#[test]
fn encode_result_cannot_length() {
    run_e5506("const b = new TextEncoder().encode('hi'); console.log(b.length);");
}

#[test]
fn encode_non_string_arg_fails_closed() {
    run_e5506("const b = new TextEncoder().encode(42n); console.log('x');");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p kali_cli --test soundness_textcodec encode -- --nocapture`
Expected: `encode_result_cannot_print` etc. FAIL (currently print `hi`/`0`); `digest_consumes_bound_encode_bytes` may FAIL (bound encode currently yields silent 0 the digest hashes, printing `ok` — verify actual). This confirms the hazard and the bound gap.

- [ ] **Step 3: Add side-tables + recognizers** in `crates/kali_codegen/src/emitter.rs` (mirror `usp_locals` at `:133`, `abort_handle_locals` at `:118`, `admit_url_handle_read` at `:147`, `is_url_search_params` at `:625`):

```rust
// (fields, near usp_locals / abort_handle_locals)
/// Bindings proven to hold a TextEncoder().encode(...) byte handle in THIS
/// emitter's scope (Repr::Bytes). Provenance-only; escape denied at the
/// identifier choke unless `admit_bytes_handle_read`.
pub(crate) bytes_locals: std::collections::BTreeSet<String>,
/// Bindings proven to hold a stateless `new TextEncoder()` marker.
pub(crate) text_encoder_locals: std::collections::BTreeSet<String>,
// (flag, near admit_url_handle_read)
pub(crate) admit_bytes_handle_read: bool,
```

Initialize all three empty in `FunctionEmitter::new` (alongside `usp_locals: BTreeSet::new()`). Add recognizers (near `is_url_search_params`):

```rust
pub(crate) fn is_bytes_handle(&self, name: &str) -> bool {
    self.bytes_locals.contains(name)
}
pub(crate) fn is_text_encoder_marker(&self, name: &str) -> bool {
    self.text_encoder_locals.contains(name)
}
```

- [ ] **Step 4: Recognize a bound-receiver `enc.encode`** in `crates/kali_codegen/src/intrinsics/host.rs` `is_text_encoder_encode` (`:327`). It currently requires `object_node.kind == Call` with ctor `TextEncoder`. Extend to also accept a bare-identifier object that is a `text_encoder_locals` marker:

```rust
pub(crate) fn is_text_encoder_encode(&self, callee_node: &LirNode) -> bool {
    if callee_node.text.as_deref() != Some("encode") {
        return false;
    }
    let Some(&object) = callee_node.children.first() else {
        return false;
    };
    let object_node = self.node(object);
    // inline `new TextEncoder().encode(...)`
    if object_node.kind == LirNodeKind::Call {
        if let Some(&ctor) = object_node.children.first() {
            if self.node(ctor).text.as_deref() == Some("TextEncoder") {
                return true;
            }
        }
    }
    // bound `const e = new TextEncoder(); e.encode(...)`
    object_node.children.is_empty()
        && object_node
            .text
            .as_deref()
            .is_some_and(|name| self.is_text_encoder_marker(name))
}
```

- [ ] **Step 5: Populate the marker + bytes side-tables at the declarator lane** in `crates/kali_codegen/src/emit/control_flow.rs` (in the `new URL`/`new URLSearchParams` declarator intercept neighborhood `:963-1067`, mirroring `self.usp_locals.insert(name)` at `:1067`). Add: when a declarator RHS is `new TextEncoder()`, emit the marker (an `i64.const 0` placeholder is fine — the marker is never read as a value) and `self.text_encoder_locals.insert(name);`. When a declarator RHS is a recognized `is_text_encoder_encode(...)` call, emit the encode (Step 6 path) and `self.bytes_locals.insert(name);`. Follow the exact structure of the URL/USP intercept (recognize RHS shape → emit → store into promoted local → record name).

- [ ] **Step 6: Retype the encode emit result context.** The encode emit arm (`emit/call.rs:3236`) keeps its zero-copy passthrough body but its result is now byte provenance. Because the shape stays a raw i64 handle, the *value shape* remains `ValueShape::String` at the emit site (transient), but the *binding* is recorded `bytes_locals` (Step 5) and `Repr::Bytes` (Step 8). Leave the emit body as-is (it already returns the arg handle). Verify the arm still recognizes bound receivers via the updated `is_text_encoder_encode`.

- [ ] **Step 7: Add the identifier-read choke deny** in `crates/kali_codegen/src/emit/control_flow.rs` (~`:1744`, immediately after the URL/USP deny block):

```rust
// Stage P5 byte-array escape choke: a bare read of a TextEncoder().encode
// byte handle (or a stateless encoder marker) is E5506 unless an
// allowlisted consumer set `admit_bytes_handle_read` (TextDecoder.decode
// receiver-arg, crypto.subtle.digest operand). The raw i64 handle must never
// escape as an observable value (`console.log(b)`, `return b`, `'' + b`,
// `b.length`, store). Allowlist the safe position at the single read site.
if (self.is_bytes_handle(text) || self.is_text_encoder_marker(text))
    && !self.admit_bytes_handle_read
{
    return self.deny_e5506(
        function,
        "a TextEncoder byte buffer cannot be read in this position: kali admits \
         it only as a TextDecoder().decode or crypto.subtle.digest operand \
         (fail-closed)",
    );
}
```

- [ ] **Step 8: Retype the repr seed + bound recognition** in `crates/kali_types/src/repr_infer.rs`. (a) Change the `"encode" if is_text_encoder_ctor(&member.object)` arm (`:2751`) so the result is seeded `Repr::Bytes`, not a string seed: replace `self.add_string_seed(result); self.runtime_string_nodes.push(result);` with a bytes-binding record. Add a `bytes_bindings: BTreeSet<(String, String)>` field (mirror `usp_bindings` at `:450`), record `(func, binding-name)` when the declarator initializer is an encode call, and seed `table.set_scalar(func, name, Repr::Bytes)` in the seeding pass (mirror `usp_bindings` seeding at `:4304`, gated on the binding still being default `Repr::I64` and the ctor unshadowed). (b) Extend `is_text_encoder_ctor`/`text_encoder_encode_new` recognition so a bound `enc.encode` (where `enc` is a `new TextEncoder()` binding) is recognized the same way (mirror how a bound USP binding is recognized in `repr_infer`).

- [ ] **Step 9: Migrate the `digest` operand to admit bytes** in `crates/kali_codegen/src/emit/call.rs` (`crypto.subtle.digest` lane, operand emit at `:3203`). Wrap the operand `emit_node` in the admit flag (mirror `emit_url_receiver_handle`'s set/restore):

```rust
let saved = self.admit_bytes_handle_read;
self.admit_bytes_handle_read = true;
let produced = self.emit_node(function, input_expr, true);
self.admit_bytes_handle_read = saved;
```

- [ ] **Step 10: Run the Task 3 tests**

Run: `cargo test -p kali_cli --test soundness_textcodec encode -- --nocapture`
Expected: all `encode_*` + `digest_consumes_bound_encode_bytes` PASS.

- [ ] **Step 11: Full-workspace gate (this task changes shared codegen/types)**

Run: `cargo test --workspace 2>&1 | tail -15`
Expected: 0 failed. If any digest/encode corpus test regressed, the migration missed an admit site — fix before committing.

- [ ] **Step 12: fmt/clippy + commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets 2>&1 | tail -3`

```bash
git add crates/kali_codegen/src crates/kali_types/src crates/kali_cli/tests/soundness_textcodec.rs
git commit -m "feat: P5 Task 3 — encode returns Repr::Bytes (bound+inline), digest admits it, escape choke closes the hazard [stageP5]"
```

---

## Task 4: `TextDecoder` marker + `decode` dispatch

**Files:**
- Modify: `crates/kali_codegen/src/emitter.rs` (`text_decoder_locals` + `is_text_decoder_marker`)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (`is_text_decoder_decode`)
- Modify: `crates/kali_codegen/src/emit/call.rs` (decode dispatch arm)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (declarator marker insert; extend choke to `is_text_decoder_marker`)
- Modify: `crates/kali_types/src/late_host.rs` (`is_new_text_decoder`)
- Modify: `crates/kali_types/src/repr_infer.rs` (decode result seeds `Repr::String`)
- Test: `crates/kali_cli/tests/soundness_textcodec.rs`

**Interfaces:**
- Consumes: `is_bytes_handle`, `admit_bytes_handle_read`, `emit_string_handle_ptr`/`emit_string_handle_len` (`emit/call.rs:4488+`), the `text_encoder_locals` declarator pattern from Task 3.
- Produces: `dec.decode(<bytes-provenance arg>)` → `EmittedValue { produced: true, shape: ValueShape::String }` (the same `(buf,len)` relabeled); `is_text_decoder_marker`.

- [ ] **Step 1: Write failing tests** — append:

```rust
// --- decode roundtrip (Task 4) ---

#[test]
fn encode_decode_roundtrip_ascii() {
    assert_eq!(
        run_ok("const e=new TextEncoder(); const d=new TextDecoder(); \
                const b=e.encode('hi'); console.log(d.decode(b));"),
        "hi"
    );
}

#[test]
fn encode_decode_roundtrip_non_ascii() {
    assert_eq!(
        run_ok("const e=new TextEncoder(); const d=new TextDecoder(); \
                const b=e.encode('héllo'); console.log(d.decode(b));"),
        "héllo"
    );
}

#[test]
fn decode_result_is_a_real_string() {
    // decode output is a normal string: comparison + concat work.
    assert_eq!(
        run_ok("const e=new TextEncoder(); const d=new TextDecoder(); \
                const b=e.encode('42'); console.log(d.decode(b) === '42');"),
        "true"
    );
}

#[test]
fn decode_of_string_literal_fails_closed() {
    run_e5506("const d = new TextDecoder(); console.log(d.decode('hi'));");
}

#[test]
fn decode_of_i64_fails_closed() {
    run_e5506("const d = new TextDecoder(); console.log(d.decode(42n));");
}

#[test]
fn decode_marker_cannot_print() {
    run_e5506("const d = new TextDecoder(); console.log(d);");
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p kali_cli --test soundness_textcodec decode -- --nocapture`
Expected: roundtrip tests FAIL (decode currently silent 0).

- [ ] **Step 3: Add `text_decoder_locals` + recognizer** in `emitter.rs` (mirror Task 3's `text_encoder_locals`):

```rust
pub(crate) text_decoder_locals: std::collections::BTreeSet<String>,
// ...
pub(crate) fn is_text_decoder_marker(&self, name: &str) -> bool {
    self.text_decoder_locals.contains(name)
}
```

Initialize empty in `FunctionEmitter::new`.

- [ ] **Step 4: Recognize `new TextDecoder()` + `dec.decode`.** In `crates/kali_types/src/late_host.rs` add `is_new_text_decoder` mirroring `is_new_text_encoder` (`:210`). In `crates/kali_codegen/src/intrinsics/host.rs` add:

```rust
pub(crate) fn is_text_decoder_decode(&self, callee_node: &LirNode) -> bool {
    if callee_node.text.as_deref() != Some("decode") {
        return false;
    }
    let Some(&object) = callee_node.children.first() else {
        return false;
    };
    let object_node = self.node(object);
    // inline `new TextDecoder().decode(...)`
    if object_node.kind == LirNodeKind::Call {
        if let Some(&ctor) = object_node.children.first() {
            if self.node(ctor).text.as_deref() == Some("TextDecoder") {
                return true;
            }
        }
    }
    // bound `const d = new TextDecoder(); d.decode(...)`
    object_node.children.is_empty()
        && object_node
            .text
            .as_deref()
            .is_some_and(|name| self.is_text_decoder_marker(name))
}
```

- [ ] **Step 5: Insert the decoder marker at the declarator lane** in `control_flow.rs` (mirror Task 3 Step 5): a `new TextDecoder()` RHS emits the marker placeholder and `self.text_decoder_locals.insert(name);`.

- [ ] **Step 6: Add the `decode` dispatch arm** in `emit/call.rs` (beside the encode arm at `~3236`):

```rust
if self.is_text_decoder_decode(&callee_node) {
    // `TextDecoder().decode(<bytes>)`: the byte handle IS a contiguous UTF-8
    // (buf,len) — relabel it back to a string result. Arity 1 only; the arg
    // must be a proven byte handle (admitted via the flag while emitted).
    let arg_ids: Vec<LirNodeId> = node.children.iter().skip(1).copied().collect();
    if arg_ids.len() != 1 {
        return self.deny_e5506(
            function,
            "TextDecoder().decode requires exactly one byte-buffer argument \
             in the current phase (fail-closed)",
        );
    }
    let arg = arg_ids[0];
    // Only a bytes-provenance identifier (or an inline encode) is admissible.
    let is_bytes = self.node(self.unwrap_transparent(arg)).text.as_deref()
        .is_some_and(|n| self.is_bytes_handle(n))
        || self.is_text_encoder_encode(&self.node(self.unwrap_transparent(arg)).clone());
    if !is_bytes {
        return self.deny_e5506(
            function,
            "TextDecoder().decode only accepts a TextEncoder().encode byte buffer \
             in the current phase (fail-closed)",
        );
    }
    let saved = self.admit_bytes_handle_read;
    self.admit_bytes_handle_read = true;
    let produced = self.emit_node(function, arg, true);
    self.admit_bytes_handle_read = saved;
    if !produced.produced {
        function.instruction(&Instruction::I64Const(0));
    }
    // The i64 is already STRING_HANDLE_TAG | (buf<<32) | len — return as String.
    return EmittedValue { produced: true, shape: ValueShape::String };
}
```

(If `unwrap_transparent`/`is_text_encoder_encode(&…clone())` ergonomics are awkward, factor an `fn arg_is_bytes_provenance(&self, arg: LirNodeId) -> bool` helper. The intent: admit a bound bytes handle OR an inline `encode(...)` argument; deny everything else.)

- [ ] **Step 7: Extend the identifier choke** (`control_flow.rs:~1744`, Task 3 Step 7 block) to also cover the decoder marker: change the condition to `(self.is_bytes_handle(text) || self.is_text_encoder_marker(text) || self.is_text_decoder_marker(text)) && !self.admit_bytes_handle_read`.

- [ ] **Step 8: Seed `decode` result `Repr::String`** in `repr_infer.rs` — add a `"decode" if is_text_decoder_ctor(&member.object)`-style arm (mirror the encode arm at `:2751`) that recognizes inline+bound decode and seeds the result `Repr::String` (`add_string_seed`), so `d.decode(b) === '42'` types as a string comparison.

- [ ] **Step 9: Run the Task 4 tests**

Run: `cargo test -p kali_cli --test soundness_textcodec decode -- --nocapture`
Expected: all `decode`/`roundtrip` tests PASS.

- [ ] **Step 10: Full-workspace gate + fmt/clippy**

Run: `cargo test --workspace 2>&1 | tail -15 && cargo fmt --all && cargo clippy --workspace --all-targets 2>&1 | tail -3`
Expected: 0 failed, clean.

- [ ] **Step 11: Commit**

```bash
git add crates/kali_codegen/src crates/kali_types/src crates/kali_cli/tests/soundness_textcodec.rs
git commit -m "feat: P5 Task 4 — TextDecoder marker + decode(bytes)->string roundtrip, escape choke extended [stageP5]"
```

---

## Task 5: `webBaselineSmoke` acceptance flip (three surfaces)

**Files:**
- Modify: `crates/kali_cli/tests/runtime_smoke/build.rs` (`build_emits_browser_bundle_web_baseline_primitives*` — flip assertions)
- Test: `crates/kali_cli/tests/soundness_textcodec.rs` (add a `kali run` acceptance test of the verbatim fixture body)

**Interfaces:**
- Consumes: `browser_bundle_web_baseline_source()` (`runtime_smoke.rs:4442`) — the verbatim fixture.
- Produces: green acceptance across `kali run` + bundle build (+ browser harness where configured).

- [ ] **Step 1: Add a `kali run` acceptance test** — append to `soundness_textcodec.rs`. This exercises the full fixture tail verbatim:

```rust
#[test]
fn web_baseline_smoke_string_and_codec_tail_runs() {
    // The String() + TextEncoder/TextDecoder tail of webBaselineSmoke, verbatim.
    let src = "\
function tail(left, right) {\n\
  const query = { get(k) { return k === 'x' ? '1' : '0'; } };\n\
  const encoder = new TextEncoder();\n\
  const decoder = new TextDecoder();\n\
  const encoded = encoder.encode(String(left + right));\n\
  if (decoder.decode(encoded) !== String(left + right)) {\n\
    throw new Error('bad');\n\
  }\n\
  console.log(String(left));\n\
  return left - left;\n\
}\n\
tail(40n, 2n);\n";
    assert_eq!(run_ok(src), "40");
}
```

- [ ] **Step 2: Run — expect PASS** (Tasks 1/3/4 make this work)

Run: `cargo test -p kali_cli --test soundness_textcodec web_baseline_smoke_string_and_codec_tail_runs -- --nocapture`
Expected: PASS, stdout `40`.

- [ ] **Step 3: Flip the build assertions** in `crates/kali_cli/tests/runtime_smoke/build.rs`. For `build_emits_browser_bundle_web_baseline_primitives` (`:3708`) and every sibling (`*_in_js_input`, `json_*`), replace the fail-closed assertion:

```rust
    // was: assert!(!output.status.success(), "expected fail-closed on String/...");
    //      assert!(stderr.contains("E5506") && stderr.contains("String"), ...);
    assert!(
        output.status.success(),
        "web-baseline bundle must build now that String()/TextEncoder/TextDecoder land\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
```

(Keep the rest of each test — the metadata/bundle-shape assertions — intact.)

- [ ] **Step 4: Run the flipped build tests**

Run: `cargo test -p kali_cli --test runtime_smoke web_baseline -- --nocapture 2>&1 | tail -20`
Expected: the `build_emits_browser_bundle_web_baseline_primitives*` family PASS (builds succeed). If a build still errors, read the stderr — a residual `String`/codec position in the fixture is not yet covered; fix in the owning task before proceeding.

- [ ] **Step 5: Full-workspace gate**

Run: `cargo test --workspace 2>&1 | tail -15`
Expected: 0 failed.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/runtime_smoke/build.rs crates/kali_cli/tests/soundness_textcodec.rs
git commit -m "test: P5 Task 5 — flip web-baseline build pins to expect success; kali run acceptance of String()+codec tail [stageP5]"
```

---

## Task 6: Residual tripwires, follow-up inventory, memory

**Files:**
- Test: `crates/kali_cli/tests/soundness_textcodec.rs` (P5-R residual tripwires)
- Modify: `docs/superpowers/followups/stageD-triage.md` (§8.6 SHIPPED entry + P5-R1..R4)

**Interfaces:** none produced; documentation + guard tests.

- [ ] **Step 1: Add residual/over-deny tripwire tests** — append:

```rust
// --- P5 residuals / deliberate boundaries ---

#[test]
fn p5_r1_global_this_string_stays_denied() {
    // Documented over-deny: globalThis.String(x) is not the bare-identifier arm.
    run_e5506("console.log(globalThis.String(1n));");
}

#[test]
fn p5_r3_bytes_in_nested_position_denies() {
    // Tripwire: a bytes handle in an exotic nested position must deny, not coerce.
    run_e5506("const b = new TextEncoder().encode('hi'); const a = [b]; console.log(a[0]);");
}

#[test]
fn p5_r4_text_decoder_option_arg_denies() {
    // Constructor options are unsupported; a decode with a label arg is not the lane.
    run_e5506("const d = new TextDecoder('utf-8'); const b = new TextEncoder().encode('x'); console.log(d.decode(b, {}));");
}
```

- [ ] **Step 2: Run the tripwires**

Run: `cargo test -p kali_cli --test soundness_textcodec p5_r -- --nocapture`
Expected: PASS. If `p5_r3` prints a value instead of E5506, the array-store escape leaked — the choke or the store site needs the deny (this is the exact class the review hammers). Fix before proceeding.

- [ ] **Step 3: Write the §8.6 SHIPPED entry + residuals** in `docs/superpowers/followups/stageD-triage.md`, mirroring the P4 SHIPPED entry format: what shipped (String() coercion; Repr::Bytes handle; bound encode/decode; digest migration; verbatim acceptance), final gate numbers, and P5-R1 (globalThis.String over-deny), P5-R2 (0/multi-arg String deny), P5-R3 (bytes-in-nested tripwire), P5-R4 (TextEncoder/Decoder options unsupported).

- [ ] **Step 4: Commit**

```bash
git add crates/kali_cli/tests/soundness_textcodec.rs docs/superpowers/followups/stageD-triage.md
git commit -m "docs+test: P5 Task 6 — residual tripwires + §8.6 SHIPPED entry [stageP5]"
```

---

## Task 7: Whole-stage adversarial review, gate certification, PR + merge

The last seven stages each had a whole-stage review catch a CRITICAL no per-task review saw (composition mutation, store-site/value-sink fail-opens, loop-capture). Do NOT skip it.

- [ ] **Step 1: Double green-baseline gate.** Create/refresh a clean `main` worktree, run `cargo test --workspace` there and on `soundness-stage-p5`, diff FAILED sets. Requirement: branch introduces **0 newly-red**. Record baseline-red / branch-red / newly-red / drain counts.

- [ ] **Step 2: fmt/clippy + CLBG goldens.** `cargo fmt --all --check`; `cargo clippy --workspace --all-targets`; run the 6 CLBG fixture goldens + web-baseline byte-for-byte.

- [ ] **Step 3: Whole-stage adversarial review.** Use `superpowers:requesting-code-review` over the full stage diff with the most capable model, focused on the §5 soundness core: enumerate EVERY store site (`a[i]=`, `o.f=`, element/field init) and EVERY generic value sink (print, concat, template, `+=`, return, arg, `.length`, index, `for..of`, `.push`, `.join`, `===`) and prove a `bytes_locals`/marker read at each denies. Also probe composition (encode→bind→mutate-source→decode), the repr-vs-codegen coherence guard (spec §5), and the `globalThis.String` over-deny boundary.

- [ ] **Step 4: Resolve findings.** For each CRITICAL/major, add a failing test, fix, re-gate. Re-run Step 1 after any code change.

- [ ] **Step 5: Update the P5 memory** at `/home/dev/.claude/projects/-workspace/memory/` — a `kali-textcodec-string-p5.md` project memory (what shipped, gate numbers, the confirmed-live-hazard lesson, whether the review caught an 8th CRITICAL) + a one-line pointer in `MEMORY.md`. Link `[[kali-url-usp-p4]]`, `[[kali-g6-unimplemented-builtin-failclosed]]`.

- [ ] **Step 6: PR + self-merge** (per `[[kali-integration-convention]]`):

```bash
gh auth setup-git
git push -u origin soundness-stage-p5
gh pr create --title "Stage P5: String() coercion + TextEncoder/TextDecoder (web-baseline finale)" \
  --body "String() runtime coercion + sound TextEncoder/TextDecoder roundtrip via a Repr::Bytes provenance handle; webBaselineSmoke runs verbatim byte-for-byte. Closes the confirmed live encode-as-string latent hazard. Gate: 0 newly-red."
# after CI green:
gh pr merge --squash --delete-branch
```

---

## Self-Review (completed during authoring)

- **Spec coverage:** §3 String → Task 1; §4.0/§4.1/§4.2 encode+Repr::Bytes+digest → Tasks 2-3; decode → Task 4; §5 escape choke → Tasks 3/4 + Task 7 review; §6.1 acceptance flip → Task 5; §6.2 test module → Tasks 1/3/4/6; §6.3 gate → every task + Task 7; §7 residuals → Task 6. All covered.
- **Placeholder scan:** every code step shows concrete Rust or an exact mirror-site + the specific new lines; no "TBD"/"add error handling"/"similar to Task N".
- **Type consistency:** `bytes_locals`/`text_encoder_locals`/`text_decoder_locals`/`admit_bytes_handle_read`/`is_bytes_handle`/`is_text_encoder_marker`/`is_text_decoder_marker`/`is_text_decoder_decode` are named identically across Tasks 3-4; `Repr::Bytes` consistent; test helpers `run_ok`/`run_e5506` defined once in Task 1 and reused.
- **Known soft spots for the implementer to verify (not placeholders):** (a) the exact declarator-lane insertion point in `control_flow.rs:963-1067` — follow the URL/USP intercept structure; (b) `unwrap_transparent`/clone ergonomics in Task 4 Step 6 — factor the `arg_is_bytes_provenance` helper if needed; (c) `repr_infer` bound-receiver recognition (Task 3 Step 8b) mirrors the USP bound-binding path.
