# G6 Unimplemented-Builtin Fail-Closed Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert five recognized-but-unimplemented JS builtin operations from silent wrong-value miscompiles (`0`/empty at exit 0) into honest `E5506` fail-closed errors, without touching the working static-fold lanes they sit beside.

**Architecture:** Three independent streams found by the source trace (the register's "one choke point" was disproved). **B** (array spread) and **C** (`Object.freeze`) are contained leaf edits at their own sites. **A** (`String`/`toString`/`JSON.stringify`/runtime-`split`) is an allowlist-invert of `emit_call`'s terminal warning+0 placeholder, made fail-closed-by-default with a positive keep-warn+0 allowlist discovered against the full-workspace gate. Order: **B → C → A**, whole-stream adversarial review last.

**Tech Stack:** Rust workspace (`kali_codegen`, `kali_types`, `kali_cli`); WASM codegen via `wasm-encoder`; integration tests in `crates/kali_cli/tests/soundness_*.rs` that shell out to the built `kali` binary and diff stdout/stderr against node-captured expectations.

## Global Constraints

- **Direction is REFUSE, not implement.** Every fix emits `E5506` (`e5::FEATURE_UNAVAILABLE`). Never `E3100` (reserved for genuinely-unrecognized identifiers, which already fail closed at resolve). Never add a runtime lowering for these builtins (that is Group 4 / architectural).
- **Fail-closed, not fail-open.** A shape that cannot be proven safe must be denied (E5506), never lowered to `0`. Prefer a positive allowlist at the choke point over a denylist of shapes (repo law: denylists leak — ~8 prior stages).
- **Gate command:** `cargo test --workspace`, diffed against a **`main` worktree** — never `.worktrees/kali-main` (fake-green). Honest-red base ≈ **712**; re-measure at this branch's stage base before starting Stream A's convergence.
- **Must stay byte-for-byte throughout:** 6/6 CLBG goldens and `acceptance_web_baseline_prefix_matches_node_byte_for_byte`.
- **fmt + clippy clean:** `cargo fmt --all` and `cargo clippy --workspace --all-targets` produce zero output/warnings on every commit.
- **Test-harness idiom (copy verbatim into each new test file):** a per-file `kali_bin()` reading `CARGO_BIN_EXE_kali`, a `run_source(src)` that writes to a temp dir keyed by `process::id()` + an `AtomicU64` counter + `src.len()` (the uniqueness triple that fixed the macOS concurrency flake), and `assert_stdout` / `assert_fails_closed(src, needle)` helpers. Copy these from `crates/kali_cli/tests/soundness_first_class_calls.rs:47-90`.
- **Every expected value is captured from node** and noted in a comment (`// node: …`).
- **Build before every probe.** Fix reports are unreliable — re-run each reproducer on a freshly built binary (`cargo build -p kali_cli --bin kali`), never trust a claim against a stale binary.

---

## Test-harness helper block (referenced by every task)

Each new test file begins with this exact block (adjust the temp-dir slug per file):

```rust
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-unimpl-builtins-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.ts");
    std::fs::write(&path, src).unwrap();
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

fn assert_stdout(src: &str, expected: &str) {
    let out = run_source(src);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected, "{out:?}");
}

fn assert_fails_closed(src: &str, needle: &str) {
    let out = run_source(src);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "expected a fail-closed diagnostic, got success with stdout {stdout:?}: {out:?}"
    );
    assert!(stderr.contains("E5506"), "expected E5506 in stderr, got: {stderr}");
    assert!(stderr.contains(needle), "expected {needle:?} in stderr, got: {stderr}");
}
```

---

## Stream B — array spread `[...a]` fails closed

### Task B1: Pin the spread miscompile as fail-closed, and pin the preserve cases

**Files:**
- Create: `crates/kali_cli/tests/soundness_array_spread.rs`
- Modify: `crates/kali_codegen/src/intrinsics/array.rs:5-7` (`is_array_literal`)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (spread-length fold, ~line 332-341) and the array element-index fold path (`emit/operators.rs:436-443`, `emit/call.rs:4867-4875`)

**Interfaces:**
- Consumes: `is_array_literal(&LirNode) -> bool` (existing), `LirNodeKind::Value`.
- Produces: a helper `array_literal_contains_spread(&self, node: &LirNode) -> bool` used by the length/index folds to fail closed. Signature: `fn array_literal_contains_spread(&self, node: &LirNode) -> bool`.

- [ ] **Step 1: Confirm the spread-node representation on a fresh binary**

```bash
cargo build -p kali_cli --bin kali
printf 'const a=[1,2]; const b=[...a]; console.log("len="+b.length+" e0="+b[0]);\n' > /tmp/spread.ts
./target/debug/kali run /tmp/spread.ts   # observe: len=1 e0=0  (node: len=2 e0=1)
```
Expected: `len=1 e0=0`. This confirms the live miscompile the test will pin. (The trace identified the spread child as a textless `Value` node with `text == Some("spread")`; if a debug dump is needed, add a temporary `eprintln!` in `is_array_literal` and remove it before committing.)

- [ ] **Step 2: Write the failing tests**

```rust
// crates/kali_cli/tests/soundness_array_spread.rs
//! Soundness pins for R-25: array spread `[...a]` was mis-classified as a
//! 1-element array literal (`is_array_literal` accepted a textless Value whose
//! child is a `spread` node), so `.length` folded to 1 and `[0]` to 0 at exit
//! 0. There is no spread-expansion lowering; the honest target is
//! REJECT-DON'T-MISCOMPILE (E5506), mirroring object spread `{...o}` which
//! already fails closed. Static array literals without a spread child keep
//! folding. Every expected value captured from node v26.

// <paste the harness helper block here, slug "unimpl-builtins-spread">

#[test]
fn array_spread_of_binding_fails_closed() {
    // node: len=2 e0=1. Pre-fix kali: len=1 e0=0 at exit 0.
    assert_fails_closed(
        r#"const a=[1,2]; const b=[...a]; console.log("len="+b.length);"#,
        "spread",
    );
}

#[test]
fn array_spread_of_literal_fails_closed() {
    // node: 2. Pre-fix kali: 1 at exit 0.
    assert_fails_closed(r#"const b=[...[1,2]]; console.log(b.length);"#, "spread");
}

#[test]
fn plain_array_literal_still_folds() {
    // Preserve pin: no spread child, must keep working.
    assert_stdout(r#"const a=[1,2,3]; console.log("len="+a.length);"#, "len=3\n");
}
```

- [ ] **Step 3: Run tests to verify they fail as expected**

Run: `cargo test -p kali_cli --test soundness_array_spread`
Expected: `array_spread_*` FAIL (currently exit 0 with `len=1`/`1`, not E5506); `plain_array_literal_still_folds` PASS.

- [ ] **Step 4: Implement the spread detector + fail-closed at the folds**

Add to `crates/kali_codegen/src/intrinsics/array.rs` (near `is_array_literal`):

```rust
/// True when an array-literal node has a spread element (`[...a]`). The
/// spread child is a textless `Value` node whose text is `Some("spread")`
/// (confirmed in Task B1 step 1). kali has no spread-expansion lowering, so
/// callers must fail closed rather than treat the spread as one element.
pub(crate) fn array_literal_contains_spread(&self, node: &LirNode) -> bool {
    self.is_array_literal(node)
        && node.children.iter().any(|&child| {
            let c = self.node(child);
            c.kind == LirNodeKind::Value && c.text.as_deref() == Some("spread")
        })
}
```

At the spread-length fold in `emit/operators.rs` (the `if self.is_array_literal(&aggregate)` block near line 334), guard it:

```rust
if self.is_array_literal(&aggregate) {
    if self.array_literal_contains_spread(&aggregate) {
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "array spread `[...x]` is unavailable in the current phase: kali has no \
             spread-expansion lowering, so the spread would be silently counted as one \
             element; use explicit elements or the later compatibility path".to_string(),
        ));
        function.instruction(&Instruction::Unreachable);
        return EmittedValue { produced: false, shape: ValueShape::Unknown };
    }
    function.instruction(&Instruction::I64Const(aggregate.children.len() as i64));
    return EmittedValue { produced: true, shape: ValueShape::Scalar };
}
```

Apply the identical guard at the element-index fold sites (`emit/operators.rs:436-443` and `emit/call.rs:4867-4875`) wherever `is_array_literal(source_node)` gates an index read, so `b[0]` on a spread literal also fails closed. Confirm the exact `e5`/`Diagnostic` import path already used in each file (grep the file for `e5::FEATURE_UNAVAILABLE`).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo build -p kali_cli --bin kali && cargo test -p kali_cli --test soundness_array_spread`
Expected: all three PASS.

- [ ] **Step 6: Confirm object-spread still fails closed and goldens hold**

```bash
printf 'const o={a:1}; const b={...o}; console.log(b.a);\n' > /tmp/os.ts
./target/debug/kali run /tmp/os.ts   # still E5506 (unchanged)
cargo fmt --all && cargo clippy --workspace --all-targets
```
Expected: object spread E5506; fmt/clippy clean.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/tests/soundness_array_spread.rs crates/kali_codegen/src/intrinsics/array.rs crates/kali_codegen/src/emit/operators.rs crates/kali_codegen/src/emit/call.rs
git commit -m "fix(codegen): array spread [...x] fails closed instead of folding to len=1 (R-25)"
```

---

## Stream C — `Object.freeze` / `Object.isFrozen` fail closed, narrowly

### Task C1: Fail closed on freeze/isFrozen of a program-bound object; preserve intrinsic-hardening

**Files:**
- Create: `crates/kali_cli/tests/soundness_object_freeze.rs`
- Modify: `crates/kali_codegen/src/emit/call.rs` (the `is_object_freeze_call(node)` handler at ~line 704; and the `isFrozen` handling — grep `isFrozen`)
- Reference (do not blindly edit): the ~15 `is_object_freeze_call` consumers (`intrinsics/object.rs`, `collections.rs`, `host.rs`, `literal.rs`, `math.rs`) — these are identity-passthrough lanes that must keep working.

**Interfaces:**
- Consumes: `is_object_freeze_call(&LirNode) -> bool` (existing, `intrinsics/object.rs:93`), `denotes_program_function` / `name_is_program_bound` (existing predicates in `emit/call.rs`), `resolve_static_object_identity_value`.
- Produces: a receiver-classification helper `freeze_receiver_needs_write_barrier(&self, node: &LirNode) -> bool` — true when the freeze/isFrozen receiver is a program-bound *object* binding (not an intrinsic namespace, not an inline literal that folds, not a callable being hardened).

- [ ] **Step 1: Confirm the bind-first miscompile and the preserve case on a fresh binary**

```bash
cargo build -p kali_cli --bin kali
printf 'const o={x:1}; Object.freeze(o); o.x=99; console.log("x="+o.x); console.log("f="+Object.isFrozen(o));\n' > /tmp/fz.ts
./target/debug/kali run /tmp/fz.ts   # kali: x=99 / f=0   (node: x=1 / f=true)
printf 'const r=Object.freeze(Math.round); console.log(r(4.6));\n' > /tmp/fh.ts
./target/debug/kali run /tmp/fh.ts   # kali: 5 (MUST stay 5 — intrinsic-hardening preserve)
```
Expected: `x=99`/`f=0` (the defect); `5` (the preserve case). Note the register's caveat: use this **bind-first** probe, never `const o=Object.freeze({x:1})` (that folds and hides the defect).

- [ ] **Step 2: Write the failing tests**

```rust
// crates/kali_cli/tests/soundness_object_freeze.rs
//! Soundness pins for R-24: `Object.freeze` was modeled as an identity
//! passthrough with no write barrier, so a bind-first
//! `Object.freeze(o); o.x=99` let the write through (kali x=99 vs node x=1)
//! and `Object.isFrozen(o)` returned 0. No write-barrier model exists; per the
//! register the honest target is fail closed (E5506) on a program-bound object
//! receiver, PRESERVING the intrinsic-hardening identity lane
//! (`Object.freeze(Math.round)`). Verified bind-first (not the folding probe,
//! which hides the defect). Every expected value captured from node v26.

// <paste the harness helper block here, slug "unimpl-builtins-freeze">

#[test]
fn freeze_of_bound_object_fails_closed() {
    // node: x=1. Pre-fix kali: x=99 at exit 0 (write went through).
    assert_fails_closed(
        r#"const o={x:1}; Object.freeze(o); o.x=99; console.log("x="+o.x);"#,
        "freeze",
    );
}

#[test]
fn is_frozen_of_bound_object_fails_closed() {
    // node: true. Pre-fix kali: 0 at exit 0.
    assert_fails_closed(
        r#"const o={x:1}; console.log(Object.isFrozen(o));"#,
        "freeze",
    );
}

#[test]
fn freeze_of_intrinsic_is_preserved() {
    // Preserve pin: Object.freeze(Math.round) hardening still returns identity.
    // node: 5.
    assert_stdout(r#"const r=Object.freeze(Math.round); console.log(r(4.6));"#, "5\n");
}
```

- [ ] **Step 3: Run tests to verify they fail as expected**

Run: `cargo test -p kali_cli --test soundness_object_freeze`
Expected: `freeze_of_bound_object_fails_closed` and `is_frozen_of_bound_object_fails_closed` FAIL (exit 0, wrong value); `freeze_of_intrinsic_is_preserved` PASS.

- [ ] **Step 4: Implement the narrow receiver classifier + fail-closed**

Add a receiver classifier in `emit/call.rs` (near the freeze handler at line 704). The rule: fail closed only when the freeze/isFrozen argument is a program-bound *object* — an identifier that the program binds to an object literal/value — and NOT an intrinsic namespace or a callable being hardened.

```rust
/// True when a freeze/isFrozen receiver is a program-bound object binding that
/// would need a write barrier kali does not have. Intrinsic namespaces
/// (`Math.round`, `Object.*`) and inline object literals that fold are NOT in
/// this class — they keep the existing identity-passthrough lowering.
fn freeze_receiver_needs_write_barrier(&self, node: &LirNode) -> bool {
    let Some(&arg) = node.children.iter().nth(1) else { return false };
    let arg = self.unwrap_transparent(arg);
    let arg_node = self.node(arg);
    // A callable being hardened (Math.round, a compiled fn) is NOT an object.
    if self.denotes_program_function(arg) { return false; }
    // A bare identifier bound by the program to an object value needs a barrier.
    if arg_node.kind == LirNodeKind::Ident {
        if let Some(text) = arg_node.text.as_deref() {
            return self.name_is_program_bound(text)
                && !self.is_intrinsic_namespace_name(text); // reuse the INTRINSIC_NAMESPACES check
        }
    }
    false
}
```

> Implementer note: `is_intrinsic_namespace_name` is the receiver-allowlist check added for CRITICAL-1 (commit `00ff4ecc0`, `INTRINSIC_NAMESPACES`). Grep `INTRINSIC_NAMESPACES` for the exact predicate name and reuse it; do not reintroduce a property-name denylist. If the exact node kind for an identifier is not `Ident`, confirm via a temporary dump in step 1's probe.

Guard the freeze handler (`emit/call.rs:704`) and the `isFrozen` handler:

```rust
if self.is_object_freeze_call(node) {
    if self.freeze_receiver_needs_write_barrier(node) {
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "Object.freeze of a bound object is unavailable in the current phase: kali has \
             no write-barrier model, so subsequent writes would silently go through; use \
             the later compatibility path".to_string(),
        ));
        function.instruction(&Instruction::Unreachable);
        return EmittedValue { produced: false, shape: ValueShape::Unknown };
    }
    // ... existing identity-passthrough lowering unchanged ...
}
```

Apply the equivalent guard where `Object.isFrozen` is lowered (message: `"Object.isFrozen of a bound object is unavailable ..."`, same `E5506`).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo build -p kali_cli --bin kali && cargo test -p kali_cli --test soundness_object_freeze`
Expected: all three PASS.

- [ ] **Step 6: Regression-check the freeze passthrough lanes + goldens**

Run: `cargo test -p kali_cli --test object_freeze_callable_helpers && cargo fmt --all && cargo clippy --workspace --all-targets`
Expected: existing freeze/callable-helper tests PASS; fmt/clippy clean. If any passthrough test regressed, the classifier is too broad — narrow it (it should touch only program-bound *object* identifiers).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/tests/soundness_object_freeze.rs crates/kali_codegen/src/emit/call.rs
git commit -m "fix(codegen): Object.freeze/isFrozen of a bound object fails closed, preserving intrinsic-hardening (R-24)"
```

> **Escape hatch (from the spec §3.3):** if step 6 shows the passthrough lanes cannot be cleanly separated from the barrier gap without touching many of the ~15 consumer sites, STOP and split Stream C into its own plan (it is ledger item 8, independent of item 4). Report the finding rather than broadening the edit.

---

## Stream A — allowlist-invert the terminal call fallback

### Task A1: Pin the three members fail-closed; pin the split static-lane preserve cases

**Files:**
- Create: `crates/kali_cli/tests/soundness_unimplemented_builtins.rs`

**Interfaces:**
- Consumes: the harness helper block.
- Produces: the acceptance pins Task A2 must turn green (fail-closed) while keeping the preserve pins green.

- [ ] **Step 1: Confirm all four behaviors on a fresh binary**

```bash
cargo build -p kali_cli --bin kali
printf 'const x=42; console.log("r="+String(x));\n'      > /tmp/s.ts && ./target/debug/kali run /tmp/s.ts   # r=0
printf 'const n=42; console.log("r="+n.toString());\n'   > /tmp/t.ts && ./target/debug/kali run /tmp/t.ts   # r=0
printf 'const o={f:1}; console.log("r="+JSON.stringify(o));\n' > /tmp/j.ts && ./target/debug/kali run /tmp/j.ts # r=0
printf 'console.log("abc".split("")[0]);\n'              > /tmp/sp.ts && ./target/debug/kali run /tmp/sp.ts   # a (PRESERVE)
```
Expected: `r=0` for the first three (defects); `a` for split-static (preserve).

- [ ] **Step 2: Write the failing + preserve tests**

```rust
// crates/kali_cli/tests/soundness_unimplemented_builtins.rs
//! Soundness pins for R-19/R-20/R-15 (Stream A): String()/x.toString()/
//! JSON.stringify()/runtime-split slipped past emit_call's E5506 deny and
//! landed on the terminal warning+`i64.const 0` placeholder (call.rs:3263),
//! returning 0 at exit 0. The fix inverts that fallback to fail-closed-by-
//! default (E5506) with a positive keep-warn+0 allowlist for genuinely
//! fail-soft surfaces. The static-fold lanes upstream of the fallback (split
//! static-ASCII, static toString) are preserved. Every expected value from node v26.

// <paste the harness helper block here, slug "unimpl-builtins-call">

#[test]
fn string_call_fails_closed() {
    // node: r=42. Pre-fix kali: r=0 at exit 0.
    assert_fails_closed(r#"const x=42; console.log("r="+String(x));"#, "unavailable");
}

#[test]
fn to_string_method_fails_closed() {
    // node: r=42. Pre-fix kali: r=0 at exit 0.
    assert_fails_closed(r#"const n=42; console.log("r="+n.toString());"#, "unavailable");
}

#[test]
fn json_stringify_fails_closed() {
    // node: r={"f":1}. Pre-fix kali: r=0 at exit 0.
    assert_fails_closed(r#"const o={f:1}; console.log("r="+JSON.stringify(o));"#, "unavailable");
}

#[test]
fn split_static_ascii_lane_is_preserved() {
    // Preserve pin: static-receiver split indexing still folds. node: a.
    assert_stdout(r#"console.log("abc".split("")[0]);"#, "a\n");
}
```

- [ ] **Step 3: Run tests to verify they fail as expected**

Run: `cargo test -p kali_cli --test soundness_unimplemented_builtins`
Expected: the three `*_fails_closed` FAIL (exit 0, `r=0`); `split_static_ascii_lane_is_preserved` PASS.

- [ ] **Step 4: Commit the pins (RED)**

```bash
git add crates/kali_cli/tests/soundness_unimplemented_builtins.rs
git commit -m "test(soundness): pin String/toString/JSON.stringify fail-closed + split static-lane preserve (R-19/R-20/R-15)"
```

### Task A2: Invert the terminal fallback to fail-closed-by-default

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs:3263-3272` (the warning+0 placeholder) and `:3314-3398` (`call_target_keeps_placeholder_lowering`)

**Interfaces:**
- Consumes: `call_target_keeps_placeholder_lowering(node, callee_node, callee_name) -> bool` (existing), `is_deferred_registration_surface`, `scheduling_call_args_provably_safe` (existing carve-out at `call.rs:3336`).
- Produces: a split of the current `true`-population into keep-warn+0 (fail-soft allowlist) vs. E5506. No new public signature; the change is internal to `emit_call`'s terminal.

- [ ] **Step 1: Replace the warning+0 placeholder with fail-closed**

At `call.rs:3263-3271`, replace the `push_placeholder_fallback_diagnostic(...) + I64Const(0)` block with a fail-closed E5506 whose message names the unimplemented-builtin class:

```rust
// Terminal fallback: a callee that reached here is neither a compiled
// function (denied above) nor an admitted fail-soft surface. Historically
// this warned and returned `i64.const 0`, which silently miscompiled
// String()/toString()/JSON.stringify()/runtime-split. Fail closed instead;
// the fail-soft surfaces are admitted by keep_warn_placeholder_lowering below.
self.diagnostics.push(Diagnostic::error(
    e5::FEATURE_UNAVAILABLE as u32,
    format!(
        "calling '{callee_name}' is unavailable in the current phase: it is a recognized \
         builtin with no implemented lowering, so evaluating it would silently return 0; \
         use explicit literals or the later compatibility path — failing closed instead"
    ),
));
for _ in node.children.iter().skip(1) {
    function.instruction(&Instruction::Drop);
}
function.instruction(&Instruction::I64Const(0));
return EmittedValue { produced: true, shape: ValueShape::Unknown };
```

> Note: this is reached only when `call_target_keeps_placeholder_lowering` returned `true`. Step 2 narrows that predicate so genuinely fail-soft surfaces bypass this fallback entirely (keep warn+0), and only unimplemented-value-builtins reach the new E5506.

- [ ] **Step 2: Add the keep-warn+0 admission and route fail-soft surfaces to it**

Introduce an explicit fail-soft allowlist. The deferred-registration carve-out (`call.rs:3336`) already identifies scheduling/event callbacks that must be dropped (warn+0); those must NOT hit the new E5506. Add a `keep_warn_placeholder_lowering(node, callee_node, callee_name) -> bool` that returns true ONLY for the deferred-registration surfaces plus an explicit, commented set of host fail-soft no-op surfaces (initially empty; populated by the Task A3 convergence loop). Emit warn+0 for those before the E5506 fallback:

```rust
if self.keep_warn_placeholder_lowering(node, &callee_node, callee_name) {
    self.push_placeholder_fallback_diagnostic("call target", callee_name);
    for _ in node.children.iter().skip(1) {
        function.instruction(&Instruction::Drop);
    }
    function.instruction(&Instruction::I64Const(0));
    return EmittedValue { produced: true, shape: ValueShape::Unknown };
}
// ... then the E5506 fail-closed block from Step 1 ...
```

```rust
/// The fail-soft allowlist: callee shapes that KEEP the historical warn+0
/// lowering because dropping the call is the ratified behavior (deferred
/// scheduling/event registration) or an acceptance-proven host no-op. This is
/// a POSITIVE allowlist — an unrecognized callee is NOT admitted here; it
/// reaches the E5506 fallback. Populated by the gate-convergence loop (each
/// entry names the fixture that requires it).
fn keep_warn_placeholder_lowering(
    &self,
    node: &LirNode,
    callee_node: &LirNode,
    callee_name: &str,
) -> bool {
    // Deferred-registration surfaces whose non-capturing callback is provably
    // safe to drop (setTimeout(cb,0), addEventListener, ...). Same proof used
    // at the arg-scan carve-out (call.rs:3336).
    if is_deferred_registration_surface(callee_name)
        && self.scheduling_call_args_provably_safe(node)
    {
        return true;
    }
    let _ = callee_node;
    // Host fail-soft no-op surfaces required by acceptance/golden fixtures.
    // (Populated in Task A3. Each addition MUST name the fixture in a comment.)
    false
}
```

- [ ] **Step 3: Build and verify the three pins pass and the preserve pin holds**

Run: `cargo build -p kali_cli --bin kali && cargo test -p kali_cli --test soundness_unimplemented_builtins`
Expected: all four PASS (`String`/`toString`/`JSON.stringify` now E5506; split-static still `a`).

- [ ] **Step 4: Commit**

```bash
git add crates/kali_codegen/src/emit/call.rs
git commit -m "fix(codegen): terminal call fallback fails closed by default; fail-soft surfaces on a positive allowlist (R-19/R-20/R-15)"
```

### Task A3: Gate-driven convergence of the fail-soft allowlist

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (`keep_warn_placeholder_lowering` body) — additive only
- Possibly modify: existing `soundness_*.rs` pins that encoded the old silent-0 (flip to fail-closed)

**Interfaces:**
- Consumes: the full-workspace gate diffed against a `main` worktree.
- Produces: a fully-attributed newly-red set (0 unattributed) and any acceptance-required fail-soft surfaces added to `keep_warn_placeholder_lowering`.

- [ ] **Step 1: Establish the honest-red baseline in a `main` worktree**

```bash
git worktree add /tmp/kali-main-gate main 2>/dev/null || true
( cd /tmp/kali-main-gate && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/gate-main.txt | tail -3 )
```
Record the main FAILED-test set (the honest-red base, ≈712). `--no-fail-fast` is required for full enumeration.

- [ ] **Step 2: Run the branch gate and diff**

```bash
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/gate-branch.txt | tail -3
# Extract FAILED test names from each and diff:
grep -oE '^test [^ ]+ \.\.\. FAILED' /tmp/gate-main.txt | sort -u > /tmp/red-main.txt
grep -oE '^test [^ ]+ \.\.\. FAILED' /tmp/gate-branch.txt | sort -u > /tmp/red-branch.txt
comm -13 /tmp/red-main.txt /tmp/red-branch.txt   # newly-red = on branch, not on main
```
Expected: the newly-red list to triage. (Parallel `cargo test` can interleave output and drop FAILED lines — re-run a suspicious suite single-threaded with `--test <name> -- --test-threads=1` before concluding.)

- [ ] **Step 3: Triage each newly-red (repeat until empty)**

For each newly-red test, run its source on a fresh binary and classify:
- **Silent-0 value-consumer** (the call's `0` was consumed as data → now honest E5506): correct. Update that test's expectation to fail-closed, or add a pin. Leave the callee denied.
- **Legitimate fail-soft** (a host side-effect no-op the acceptance/golden path must continue past): add its surface to `keep_warn_placeholder_lowering` with a comment naming the fixture, rebuild, re-run.

Re-run steps 2-3 until `comm -13` yields only tests already attributed (0 unaccounted-for newly-red). Confirm the 6/6 CLBG goldens and `acceptance_web_baseline_prefix_matches_node_byte_for_byte` are green in `/tmp/gate-branch.txt`.

- [ ] **Step 4: Final fmt/clippy + commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets
git add -A
git commit -m "fix(soundness): converge Stream-A fail-soft allowlist; flip silent-0 pins to fail-closed"
```

### Task A4: Whole-stream adversarial review

**Files:** none (review + any follow-up fix commits)

- [ ] **Step 1: Dispatch a fresh adversarial reviewer (opus)**

Dispatch a subagent (opus) to adversarially probe, on a freshly built binary, for surviving fail-opens across all three streams. Required probes (the repo's ~8× lesson — the gate passes while siblings silently `0`):
- Stream A sibling spellings: `globalThis["String"](x)`, `String(x) + ""`, `let s = JSON.stringify(o); s`, `arr.map(x => x)`-style callbacks (must still be denied), a genuinely fail-soft host call the allowlist admits (must still warn+0), and any free-name host call NOT on the allowlist (must E5506).
- Stream B: `[a, ...b]` (spread not first), `[...a, ...b]`, spread in a nested literal.
- Stream C: `let o = {x:1}; Object.freeze(o)` (`let` not `const`), `Object.freeze(o); Object.freeze(o)`, `Object.isFrozen(intrinsic)`; and confirm `Object.freeze(Math.round)(4.6)` still returns `5`.
- Confirm no probe exits 0 with a wrong value; every unsupported shape is E5506; every preserve case is byte-for-byte node.

- [ ] **Step 2: Fix any confirmed fail-open and re-verify on a fresh binary**

For each confirmed finding, add the fail-closed arm (or allowlist admission) at the choke point, rebuild, re-run the reviewer's probe, and re-run the full gate. Commit each fix. Do not accept a fix report without re-running its reproducer on a freshly built binary.

- [ ] **Step 3: Update the register and memory**

Mark R-19/R-20/R-15/R-25/R-24 closed in `docs/superpowers/followups/kali-silent-miscompile-register.md` (§2 entries + §6 item 4 row), record the final gate numbers and the Stream-A allowlist contents in `.superpowers/sdd/progress.md`, and add/update the project memory per the repo convention.

---

## Self-Review

**Spec coverage:**
- R-19 `String`/`toString` → Task A1/A2 ✓
- R-20 `JSON.stringify` → Task A1/A2 ✓
- R-15 runtime-`split` → covered by the same terminal-fallback invert (A2); static lane preserved (A1 pin) ✓
- R-25 array spread → Task B1 ✓
- R-24 `Object.freeze`/`isFrozen` → Task C1 (with split-to-own-plan escape hatch) ✓
- Allowlist-invert (not denylist) → A2 `keep_warn_placeholder_lowering` positive allowlist ✓
- Gate-driven convergence + `main`-worktree diff + 0 unattributed newly-red → A3 ✓
- Preserve: split static lane (A1), `Object.freeze(Math.round)` (C1), object-spread E5506 (B1 step 6), plain array fold (B1) ✓
- E5506 contract, not E3100 → Global Constraints + every fail-closed block ✓
- Whole-stream adversarial review on fresh binary → A4 ✓

**Placeholder scan:** No "TBD"/"add error handling"/"similar to Task N". The one deliberately-empty structure — `keep_warn_placeholder_lowering`'s host-surface list — is empty *by design* (default-deny) and populated by the A3 convergence loop; that is the allowlist-invert, not a placeholder.

**Type consistency:** `array_literal_contains_spread`, `freeze_receiver_needs_write_barrier`, `keep_warn_placeholder_lowering`, `call_target_keeps_placeholder_lowering` used consistently; `assert_fails_closed(src, needle)` / `assert_stdout(src, expected)` signatures match the copied harness block. `is_intrinsic_namespace_name` flagged as "confirm exact name via `INTRINSIC_NAMESPACES` grep" rather than assumed.
