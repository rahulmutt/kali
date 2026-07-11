# throw-fallout Stage 1 — Runtime String Equality Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make runtime `==`/`!=`/`===`/`!==` on strings compare **contents** (length + bytes) instead of handle identity, via a `__streq` synthetic wasm function, draining the 656-test #2/#3 bucket of the throw-fallout denominator and lifting the E3200 equality taint-reject (maintainer-approved lift + re-pin).

**Architecture:** A new `(i64,i64)→i64` synthetic `__streq` (handle-identity fast path → string-tag guard → length pre-check → byte loop) follows the exact `__substring`/`__join` emission pattern in `crates/kali_codegen/src/lower.rs`. A new arm in `emit_binary` (`crates/kali_codegen/src/emit/operators.rs`) routes every equality whose operands are BOTH provably string-valued through `Call(__streq)`, preempting both the silent handle-identity `i64.eq` miscompile and the E3200 equality reject. The reject lane is retained as the fail-closed backstop for mixed/tainted residues. Spec: `docs/superpowers/specs/2026-07-11-throw-fallout-stage1-string-equality-design.md`.

**Tech Stack:** Rust workspace (kali compiler), `wasm_encoder` instruction emission, `cargo test` integration tests in `crates/kali_cli/tests/` that invoke the built `kali` binary on temp-dir fixtures.

## Global Constraints

- Branch: `soundness-batch1-pra`. Never commit to `main`.
- Gate verdict command: `cargo test --workspace`. Enumerating the failing set REQUIRES `cargo test --workspace --no-fail-fast` (the plain run fail-fasts at the first failing test binary). Baseline: the `main` worktree at `/workspace/.worktrees/kali-main` (0 failures; machine-local path).
- Stage-gate pass = the 977-name denominator (`docs/superpowers/followups/throw-fallout-stage0-denominator.md`) strictly shrank AND no main-green test is red at the checkpoint. Honest-red is allowed mid-stage (between Tasks 2 and 3), never at the checkpoint.
- Fix, never flip. The ONLY tests whose expectations change are the five equality-reject pins (Task 3) whose old pin encodes the now-removed reject; every new expectation is derived by running the equivalent source under `node`, never from whatever makes the test pass.
- No re-masking: a fix that silently no-ops a self-check `throw` is a defect even if the test goes green (Task 6 checks this).
- `__streq`'s body and the `!=`/`!==` negation site must NOT emit `i64.eqz`: `boolean_branches_use_the_layout_fast_path` (crates/kali_codegen/src/emit/control_flow_tests/pipeline_basics.rs:24) asserts the WHOLE printed module contains no `i64.eqz`, and synthetics are emitted into every module (same constraint `__join` documents at lower.rs:3461).
- Boolean printing: `console.log(<comparison>)` prints `1`/`0` in kali (pre-existing pinned divergence from node's `true`/`false` — see `interned_literal_equality_is_preserved`, crates/kali_cli/tests/runtime_string_value_flow.rs:133). New tests use throw-based self-checks (`if (...) throw` + `console.log('ok')`) wherever possible; direct comparison prints pin `1\n`/`0\n`.
- Both-sides oracle discipline: any new operand classification must key on the SAME recognizer emission routes with. The types side (`kali_types`) has NO equality-specific gate (the E3200 equality reject lives only in codegen at operators.rs:1410), so no types-side code change is expected; Task 5/6 fixtures verify no types-side reject fires for admitted forms.
- Before the final commit of the stage: `cargo fmt --all` and `cargo clippy --workspace --all-targets` clean (repo convention from the Stage-0/Task-11 gates).
- Temp fixture dirs in tests MUST use the per-process `AtomicU64` counter slug convention (macOS CI collision flake — see the `run_source` helper at crates/kali_cli/tests/runtime_string_value_flow.rs:8).

---

### Task 0: Stage-start triage — pin the target set

**Files:**
- Create: `docs/superpowers/followups/throw-fallout-stage1-triage.md`

**Interfaces:**
- Produces: the enumerated pre-stage failing set at `/tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-pre.txt` (one `binary::test` name per line) and the triage doc. Task 7 diffs against these.

- [ ] **Step 1: Confirm the main worktree is unchanged**

Run: `git -C /workspace/.worktrees/kali-main rev-parse --short HEAD`
Expected: `b48a067d3` (the commit the denominator snapshot certified at 0 failures). If it differs, STOP and re-verify the worktree per the denominator doc before proceeding.

- [ ] **Step 2: Enumerate the current branch failing set**

Run (long — ~full workspace build + test):
```bash
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-pre.log
grep -E '^test .+ \.\.\. FAILED$' /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-pre.log | sed 's/^test //; s/ \.\.\. FAILED$//' | sort > /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-pre.txt
wc -l /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-pre.txt
```
Expected: 977 names (the Stage-0 denominator). A small drift (±a few) means flaky/env-dependent tests — note them in the triage doc; a large drift means the branch moved since Stage 0 — STOP and reconcile first.

Note: test names in the log are NOT binary-qualified. When the same test name exists in several binaries, count occurrences rather than deduplicating (`sort` without `-u` above is deliberate).

- [ ] **Step 3: Write the triage doc**

Create `docs/superpowers/followups/throw-fallout-stage1-triage.md` with: the pre-stage count; the expected-to-drain subset (the #2/#3 bucket listing from the denominator doc); the expected-to-REMAIN-red overlap entries (names matching `for_await`/`promise`/`async`/`queue_microtask` → Stage 7; `reflect_own_keys`/`frozen_object` delete-reinsert shapes → Stage 2; `performance`/`crypto`/`coverage_hit` → Stage 3) with one line of reasoning each; and a "follow-ups opened this stage" section (filled by later tasks: F-Stage1-1 mixed-type equality from the spec, F-Stage1-2 env-vs-env equality from Task 4).

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/followups/throw-fallout-stage1-triage.md
git commit -m "docs(soundness): throw-fallout Stage 1 triage — pin the pre-stage failing set"
```

---

### Task 1: Emit the `__streq` synthetic in every module

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (4 sites: `SYNTHETIC_FUNCTIONS` ~line 37, `FunctionPlan` push after the `__join_arena` push ~line 312, `local_decls` arm ~line 662, dispatch arm ~line 734, plus the new `emit_streq_body` function after `emit_substring_body` ~line 3439)
- Modify: `crates/kali_codegen/src/emitter.rs` (new `streq_fn_index` accessor after `join_arena_fn_index` ~line 380)
- Test: `crates/kali_codegen/src/emit/control_flow_tests/pipeline_basics.rs`

**Interfaces:**
- Produces: synthetic wasm function `__streq(a: i64, b: i64) -> i64` (1 = equal, 0 = not) present in every emitted module; `FunctionEmitter::streq_fn_index(&self) -> u32`. Task 2 calls `self.streq_fn_index()` from `emit_binary`.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_codegen/src/emit/control_flow_tests/pipeline_basics.rs` (same helpers as `boolean_branches_use_the_layout_fast_path` at line 24):

```rust
#[test]
fn streq_synthetic_is_emitted_without_i64_eqz() {
    // `__streq` (throw-fallout Stage 1) is an unconditional synthetic like
    // `__join`: present in every module. Its byte loop is the module's only
    // `i64.load8_u` consumer, and — like `__join` (see the comment in
    // `emit_join_body`) — it must never emit `i64.eqz`, which
    // `boolean_branches_use_the_layout_fast_path` bans module-wide.
    let program = parse_and_lower_lir("console.log(1);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty());
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.load8_u"));
    assert!(!printed.contains("i64.eqz"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_codegen streq_synthetic_is_emitted_without_i64_eqz`
Expected: FAIL on `printed.contains("i64.load8_u")` (no byte loads exist yet).

- [ ] **Step 3: Implement the synthetic**

(a) `SYNTHETIC_FUNCTIONS` (lower.rs:37) — add the name and extend the doc comment's list ("…the runtime-substring helper (Spec 2), the runtime-join pair (Spec 3 / fasta Spec 7), and the runtime string-equality helper (throw-fallout Stage 1)"):

```rust
pub const SYNTHETIC_FUNCTIONS: &[&str] = &[
    "__alloc",
    "__alloc_global",
    "__page_get",
    "__arena_reset",
    "__substring",
    "__join",
    "__join_arena",
    "__streq",
];
```

(b) `FunctionPlan` push — immediately after the `__join_arena` push's closing `});` in `collect_functions`:

```rust
    // Synthetic runtime string equality `__streq(a: i64, b: i64) -> i64`
    // (throw-fallout Stage 1): content comparison of two tagged string
    // handles — 1 when equal, 0 when not. Handle-identity fast path, then a
    // string-tag guard (a 0/untagged operand — e.g. a missing `Deno.env.get`
    // — is unequal to every real string), then length pre-check, then a
    // byte-compare loop. Same inert-placeholder pattern as the synthetics
    // above; body hand-emitted by `emit_streq_body`.
    all_functions.push(FunctionPlan {
        name: "__streq".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
```

(c) `local_decls` arm — extend the chain at lower.rs:654-663 (and add to the local-count comment block above it: "`__streq` (`emit_streq_body`): 4 i64 — `len`, `i`, `pa`, `pb` (locals 2-5; locals 0-1 are its `a`/`b` params)."):

```rust
        } else if function.name == "__streq" {
            local_decls.push((4, ValType::I64));
        } else if matches!(function.name.as_str(), "__join" | "__join_arena") {
```

(Insert the `__streq` arm anywhere in the synthetic chain before the final `else`; keeping it next to `__substring` is fine too.)

(d) Dispatch arm — in the `match function.name.as_str()` at lower.rs:724-736:

```rust
                "__streq" => emit_streq_body(&mut body),
```

(e) `emit_streq_body` — add after `emit_substring_body` (lower.rs:3439):

```rust
/// `__streq(a, b) -> i64`: content equality of two tagged string handles —
/// 1 when equal, 0 when not (throw-fallout Stage 1). Locals: 0 = a, 1 = b
/// (params), 2 = len, 3 = i, 4 = pa, 5 = pb.
///
/// Order of checks:
///   1. identical handles → 1 (interned-vs-interned and aliased handles);
///   2. string-tag guard: unless BOTH operands carry `STRING_HANDLE_TAG`,
///      they are not two live strings (e.g. a missing `Deno.env.get` is 0)
///      → 0 (the identical case already returned);
///   3. length mismatch (low 32 bits) → 0;
///   4. len == 0 → 1 (two empty strings are equal at ANY offsets);
///   5. byte loop over the two decoded offsets — first mismatch → 0, loop
///      completion → 1.
///
/// Offsets are decoded exactly as the runtime does (`(h >> 32) & 0x7FFF_FFFF`
/// — masked, mirroring `read_guest_string_handle` in
/// kali_runtime/src/host/memory.rs), matching `emit_substring_body`.
///
/// NO `i64.eqz` anywhere in this body: like `__join` (see the comment in
/// `emit_join_body`), `__streq` is present in every module and
/// `boolean_branches_use_the_layout_fast_path` asserts module-wide printed
/// text contains no `i64.eqz`. Zero-tests use `i64.const 0` + `i64.eq`.
fn emit_streq_body(func: &mut Function) {
    // 1. if a == b return 1
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // 2. if (a & b & TAG) == 0 return 0  — not two tagged strings
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // 3. len = a & 0xFFFF_FFFF; if len != (b & 0xFFFF_FFFF) return 0
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Ne);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // 4. if len == 0 return 1
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // pa = (a >> 32) & 0x7FFF_FFFF; pb = (b >> 32) & 0x7FFF_FFFF
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(5));
    // i = 0
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3));
    // 5. loop: if *(pa+i) != *(pb+i) return 0; i += 1; continue while i < len
    func.instruction(&Instruction::Loop(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load8U(MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    }));
    func.instruction(&Instruction::I64Ne);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::BrIf(0));
    func.instruction(&Instruction::End);
    // all len bytes equal
    func.instruction(&Instruction::I64Const(1));
    // NO trailing End — the dispatch loop appends it (same as every synthetic).
}
```

(f) `emitter.rs` accessor — after `join_arena_fn_index` (~line 380):

```rust
    /// Wasm function index of the synthetic runtime string-equality helper
    /// (`__streq(a, b) -> i64`, throw-fallout Stage 1): content comparison of
    /// two tagged string handles (identity fast path, tag guard, length
    /// pre-check, byte loop). Called by `emit_binary`'s both-string equality
    /// arm.
    pub(crate) fn streq_fn_index(&self) -> u32 {
        self.functions["__streq"]
    }
```

Note: `streq_fn_index` is dead code until Task 2 lands. If `cargo clippy` flags the unused method in between, that is expected — Tasks 1 and 2 land in the same stage and the final Task 7 clippy gate is the binding one.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p kali_codegen streq_synthetic_is_emitted_without_i64_eqz`
Expected: PASS.

- [ ] **Step 5: Run the whole codegen crate**

Run: `cargo test -p kali_codegen`
Expected: PASS. If a printed-module or function-count assertion fails because every module now contains one more function, the ONLY acceptable fix is updating that pinned count/text to include `__streq` (with a one-line comment citing this stage); any other failure is a real bug in Step 3 — fix it, do not re-pin.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/lower.rs crates/kali_codegen/src/emitter.rs crates/kali_codegen/src/emit/control_flow_tests/pipeline_basics.rs
git commit -m "feat(codegen): emit __streq content-equality synthetic in every module (throw-fallout Stage 1)"
```

---

### Task 2: Route both-string equality through `__streq`

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (new arm after the string-`+` concat lane ending at line 1372; comment update in the reject block at lines 1374-1389)
- Test: create `crates/kali_cli/tests/runtime_string_equality.rs`

**Interfaces:**
- Consumes: `self.streq_fn_index()` (Task 1), `self.is_string_valued(id)` (operators.rs:808), `self.emit_node(...)`.
- Produces: every `==`/`!=`/`===`/`!==` whose operands are BOTH `is_string_valued` compiles to `Call __streq` (+ negation for `!=`/`!==`) and returns `ValueShape::Boolean`. Tasks 3-6 rely on this lane existing exactly here.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/runtime_string_equality.rs`:

```rust
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    // Per-process AtomicU64 counter slug — repo convention against the macOS
    // CI temp-dir collision flake (see runtime_string_value_flow.rs).
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-streq-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

fn assert_ok(out: &std::process::Output) {
    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok\n");
}

// Every expectation below was derived by running the same source under node
// (self-check `throw` shapes, so the pre-existing 1/0-vs-true/false boolean
// print divergence never matters).

#[test]
fn concat_equality_compares_content() {
    // node: "x" + "y" == "xy" → true (fresh handle vs interned literal).
    let out = run_source(
        "let a = \"x\";\nlet b = a + \"y\";\nif (b !== \"xy\") { throw new Error(\"content equality failed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn concat_inequality_same_length_different_bytes() {
    // node: "x" + "z" === "xy" → false.
    let out = run_source(
        "let a = \"x\";\nlet b = a + \"z\";\nif (b === \"xy\") { throw new Error(\"same-length different bytes compared equal\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn concat_inequality_different_length() {
    // node: "x" + "yz" === "xy" → false (length pre-check path).
    let out = run_source(
        "let a = \"x\";\nlet b = a + \"yz\";\nif (b === \"xy\") { throw new Error(\"different lengths compared equal\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn empty_concat_equals_empty_literal() {
    // node: "" + "" === "" → true (len-0 path: fresh empty handle at a
    // DIFFERENT offset than the interned "" — must still be equal).
    let out = run_source(
        "let a = \"\";\nlet b = a + \"\";\nif (b !== \"\") { throw new Error(\"empty strings compared unequal\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn empty_vs_nonempty_is_unequal() {
    // node: "" + "" === "x" → false.
    let out = run_source(
        "let a = \"\";\nlet b = a + \"\";\nif (b === \"x\") { throw new Error(\"empty equalled nonempty\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn interned_literal_equality_still_true() {
    // node: s = "hi"; s === "hi" → true (the __streq identity fast path —
    // must not regress the interned lane).
    let out = run_source(
        "let s = \"hi\";\nif (s !== \"hi\") { throw new Error(\"interned equality regressed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn substring_equality_compares_content() {
    // node: "GGCC".substring(0, 1) === "G" → true (zero-copy slice handle vs
    // interned literal; previously E3200-rejected).
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet s = a.substring(0, i);\nif (s !== \"G\") { throw new Error(\"substring equality failed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn join_equality_compares_content() {
    // node: ["x"].join("") === "x" → true (fresh __join buffer; previously
    // E3200-rejected).
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nif (a.join(\"\") !== \"x\") { throw new Error(\"join equality failed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

fn run_node_source_with_args(src: &str, args: &[&str]) -> std::process::Output {
    // node-API surface + `--` guest-arg separator, same shape as
    // runtime_argv.rs's helper.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-streq-argv-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    let mut cmd = Command::new(kali_bin());
    cmd.arg("run").arg("--api").arg("node").arg(&path).arg("--");
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("run kali")
}

#[test]
fn argv_element_equality_compares_content() {
    // node: process.argv[2] === "hello" → true when invoked with "hello"
    // (argv elements are fresh args_get buffers; previously handle-compared).
    let out = run_node_source_with_args(
        "if (process.argv[2] !== \"hello\") { throw new Error(\"argv equality failed\"); }\nconsole.log(\"ok\");\n",
        &["hello"],
    );
    assert_ok(&out);
}

#[test]
fn double_negation_lanes_agree() {
    // node: both ("a"+x) == "az" and !(("a"+x) != "az") are true — the ==
    // and != lowerings must be exact complements.
    let out = run_source(
        "let x = \"z\";\nlet b = \"a\" + x;\nif (b == \"az\") { if (b != \"az\") { throw new Error(\"eq and ne disagree\"); } console.log(\"ok\"); } else { throw new Error(\"eq false\"); }\n",
    );
    assert_ok(&out);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_string_equality --no-fail-fast`
Expected: FAIL — the concat/substring/join shapes fail with the E3200 compile reject (`operator '==' on a runtime string value is unavailable…`); the argv shape fails at runtime (handle compare → self-check throws); `interned_literal_equality_still_true` may already PASS (interned lane) — that's fine.

- [ ] **Step 3: Implement the equality arm**

In `crates/kali_codegen/src/emit/operators.rs`, insert directly after the string-`+` concat lane's closing brace (line 1372), BEFORE the `// A string operand in a NON-\`+\` position …` comment block:

```rust
        // Runtime string equality (throw-fallout Stage 1): when BOTH operands
        // are proven string-valued, `==`/`===` (and the negations) are CONTENT
        // equality — `__streq` compares length + bytes with a handle-identity
        // fast path, so fresh runtime handles (enumeration keys, concat,
        // substring, join, argv) compare by VALUE, matching node.
        // Handle-identity `i64.eq` on strings survives only as the fast path
        // INSIDE `__streq`. Anything not both-string (mixed, unproven) falls
        // through to the fail-closed reject below, unchanged.
        if matches!(op, "==" | "!=" | "===" | "!==")
            && self.is_string_valued(left)
            && self.is_string_valued(right)
        {
            for operand in [left, right] {
                let emitted = self.emit_node(function, operand, true);
                if !emitted.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
            }
            function.instruction(&Instruction::Call(self.streq_fn_index()));
            if matches!(op, "!=" | "!==") {
                // Negate WITHOUT `i64.eqz` (module-wide printed-text pin in
                // pipeline_basics::boolean_branches_use_the_layout_fast_path):
                // `__streq` returns exactly 0 or 1, so `== 0` is the complement.
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }
```

Then update the equality bullet of the reject-lane comment (lines 1381-1384). Replace:

```rust
        //   - Equality (`== != === !==`): identity-comparing handles is correct
        //     ONLY for interned literal constants; a fresh runtime concat handle
        //     is not the interned handle of the same text. Reject only a tainted
        //     (runtime-concat-derived) operand, preserving `s == "hi"` etc.
```

with:

```rust
        //   - Equality (`== != === !==`): a BOTH-proven-string equality was
        //     already content-compared via `__streq` above (Stage 1) and never
        //     reaches here. The taint reject below survives as the fail-closed
        //     BACKSTOP for the residue: a tainted string against a NON-string
        //     operand (e.g. `("a"+s) == 5`), where neither identity compare nor
        //     `__streq` is meaningful.
```

The reject `let reject = if is_equality { … }` computation itself is UNCHANGED (it is now unreachable for both-string operands and still guards the mixed-tainted residue, which the spec's error-handling section requires to keep rejecting).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_string_equality`
Expected: PASS (all 10).

- [ ] **Step 5: Run the neighboring string suites and codegen crate**

Run: `cargo test -p kali_codegen && cargo test -p kali_cli --test runtime_string_value_flow --test runtime_substring_length --test runtime_join --test runtime_ternary --test runtime_string_arrays --no-fail-fast`
Expected: `kali_codegen` all PASS (including `boolean_branches_use_the_layout_fast_path`). In the CLI suites exactly FIVE failures — the old reject pins now that the reject is preempted:
- `runtime_string_value_flow::concat_result_equality_is_rejected`
- `runtime_substring_length::substring_result_equality_is_rejected`
- `runtime_join::join_result_equality_is_rejected`
- `runtime_ternary::string_ternary_equality_is_rejected`
- `runtime_string_arrays::tainted_element_equality_is_rejected`

This is the expected honest mid-stage red; Task 3 re-pins them. Any OTHER failure in these suites (e.g. the truthiness/relational/store pins) is a regression — fix before committing.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/runtime_string_equality.rs
git commit -m "fix(codegen): both-string equality compares content via __streq, preempting the E3200 equality reject (throw-fallout Stage 1)"
```

---

### Task 3: Re-pin the five equality-reject tests to node-derived results

**Files:**
- Modify: `crates/kali_cli/tests/runtime_string_value_flow.rs:95-106` (`concat_result_equality_is_rejected`)
- Modify: `crates/kali_cli/tests/runtime_substring_length.rs:97-108` (`substring_result_equality_is_rejected`)
- Modify: `crates/kali_cli/tests/runtime_join.rs:154-163` (`join_result_equality_is_rejected`)
- Modify: `crates/kali_cli/tests/runtime_ternary.rs:93-108` (`string_ternary_equality_is_rejected`)
- Modify: `crates/kali_cli/tests/runtime_string_arrays.rs:65-74` (`tainted_element_equality_is_rejected`)

**Interfaces:**
- Consumes: the Task 2 lane. Each file's existing `run_source` helper is reused unchanged.
- Produces: the five tests green against the new content-equality behavior; names change from `…_is_rejected` to `…_compares_content`.

- [ ] **Step 1: Derive every expectation from node**

Run each and record output (all print `true` or take the then-branch):
```bash
node -e 'let a = "x"; let b = a + "y"; console.log(b == "xy");'                                      # true
node -e 'let a = "GGCC"; let i = 1; let s = a.substring(0, i); if (s == "G") { console.log(1); }'    # 1
node -e 'const a = new Array(1); a[0] = "x"; if (a.join("") == "x") { console.log(1); }'             # 1
node -e 'let c = 1; let x = "z"; if ((c > 0 ? "a" + x : "b" + x) == "az") { console.log(1); } else { console.log(2); }'  # 1
node -e 'function f(s) { const a = new Array(1); a[0] = s + "y"; if (a[0] == "xy") { console.log(1); } } f("x");'        # 1
```
Expected: `true`, `1`, `1`, `1`, `1`.

- [ ] **Step 2: Re-pin all five tests**

Replace each test body IN PLACE (keep file position; rename). The kali sources are IDENTICAL to the old pins — only the assertion flips, per the node-derived result.

`runtime_string_value_flow.rs` — replace `concat_result_equality_is_rejected` with:

```rust
#[test]
fn concat_result_equality_compares_content() {
    // RE-PIN (throw-fallout Stage 1): `b == "xy"` on a FRESH runtime concat
    // handle is now CONTENT equality via `__streq` (node: true). The old pin
    // asserted the E3200 fail-closed reject that Stage 1 lifted. kali prints
    // the comparison as `1` (same pre-existing boolean-print lane
    // `interned_literal_equality_is_preserved` pins).
    let out = run_source("let a = \"x\";\nlet b = a + \"y\";\nconsole.log(b == \"xy\");\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}
```

`runtime_substring_length.rs` — replace `substring_result_equality_is_rejected` with:

```rust
#[test]
fn substring_result_equality_compares_content() {
    // RE-PIN (throw-fallout Stage 1): a slice is a non-interned runtime
    // string; `==` is now content equality via `__streq` (node: prints 1).
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet s = a.substring(0, i);\nif (s == \"G\") { console.log(1); }\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}
```

`runtime_join.rs` — replace `join_result_equality_is_rejected` with:

```rust
#[test]
fn join_result_equality_compares_content() {
    // RE-PIN (throw-fallout Stage 1): a fresh `__join` buffer now compares by
    // content via `__streq` (node: prints 1).
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nif (a.join(\"\") == \"x\") {\n  console.log(1);\n}\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}
```

`runtime_ternary.rs` — replace `string_ternary_equality_is_rejected` with:

```rust
#[test]
fn string_ternary_equality_compares_content() {
    // RE-PIN (throw-fallout Stage 1): `==` on a string-armed ternary is now
    // content equality via `__streq` — the taken arm "a"+"z" equals "az"
    // (node: prints 1). Pre-Stage-1 this was the fail-closed E3200 reject
    // (and before THAT, a silent wrong-branch handle compare).
    let out = run_source(
        "let c = 1;\nlet x = \"z\";\nif ((c > 0 ? \"a\" + x : \"b\" + x) == \"az\") { console.log(1); } else { console.log(2); }\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}
```

`runtime_string_arrays.rs` — replace `tainted_element_equality_is_rejected` with:

```rust
#[test]
fn tainted_element_equality_compares_content() {
    // RE-PIN (throw-fallout Stage 1): a concat-tainted element read now
    // compares by content via `__streq` (node: prints 1).
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = s + \"y\";\n  if (a[0] == \"xy\") {\n    console.log(1);\n  }\n}\nf(\"x\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}
```

- [ ] **Step 3: Run the five suites**

Run: `cargo test -p kali_cli --test runtime_string_value_flow --test runtime_substring_length --test runtime_join --test runtime_ternary --test runtime_string_arrays --no-fail-fast`
Expected: ALL PASS (the truthiness, relational, store, and mixed-element pins in the same files must still pass untouched).

- [ ] **Step 4: Commit**

```bash
git add crates/kali_cli/tests/runtime_string_value_flow.rs crates/kali_cli/tests/runtime_substring_length.rs crates/kali_cli/tests/runtime_join.rs crates/kali_cli/tests/runtime_ternary.rs crates/kali_cli/tests/runtime_string_arrays.rs
git commit -m "test(cli): re-pin 5 equality-reject pins to content-equality results (throw-fallout Stage 1 lift, node-derived)"
```

---

### Task 4: `Deno.env.get` single-operand equality

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (new recognizer helper next to `is_string_valued`; widen the Task 2 arm's condition)
- Test: `crates/kali_cli/tests/runtime_string_equality.rs` (append)

**Interfaces:**
- Consumes: `self.env_get_import_index(&LirNode) -> Option<u32>` (crates/kali_codegen/src/intrinsics/host.rs:93 — the SAME recognizer the call emitter routes with at call.rs:2089); `__streq`'s tag guard (Task 1) which makes the 0-valued "missing env var" result unequal to every real string.
- Produces: equality with EXACTLY ONE `Deno.env.get(...)` operand (other side proven string) routed through `__streq`. Env-vs-env stays on today's path (recorded as follow-up F-Stage1-2).

**Why at-most-one env-get operand:** both `Deno.env.get` results are materialized into the SAME reserved buffer (env lane, call.rs:2108-2116 — `env_buffer_offset = 0`), so by the time `__streq` ran, the second call would have overwritten the first's bytes and two same-length values would compare equal spuriously. That aliasing predates this stage (env-vs-env identity compare is wrong on `main` too); it is out of scope and recorded, not absorbed.

- [ ] **Step 1: Write the failing tests**

Append to `crates/kali_cli/tests/runtime_string_equality.rs`:

```rust
fn run_source_with_env(src: &str, key: &str, value: Option<&str>) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-streq-env-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    let mut cmd = Command::new(kali_bin());
    cmd.arg("run").arg(&path).env_remove(key);
    if let Some(value) = value {
        cmd.env(key, value);
    }
    cmd.output().expect("run kali")
}

// Node analog for derivation: `process.env.K` (node has no Deno global);
// semantics asserted are plain JS string/undefined equality.

#[test]
fn env_get_equality_matches_set_value() {
    // node analog: process.env.K = "y"; process.env.K === "y" → true.
    let out = run_source_with_env(
        "if (Deno.env.get(\"KALI_STREQ_A\") !== \"y\") { throw new Error(\"env equality failed\"); }\nconsole.log(\"ok\");\n",
        "KALI_STREQ_A",
        Some("y"),
    );
    assert_ok(&out);
}

#[test]
fn env_get_equality_rejects_different_value() {
    // node analog: env K = "z"; K === "y" → false.
    let out = run_source_with_env(
        "if (Deno.env.get(\"KALI_STREQ_B\") === \"y\") { throw new Error(\"different env value compared equal\"); }\nconsole.log(\"ok\");\n",
        "KALI_STREQ_B",
        Some("z"),
    );
    assert_ok(&out);
}

#[test]
fn env_get_missing_is_unequal_to_every_string() {
    // node analog: undefined === "y" → false, and undefined === "" → false
    // (the __streq TAG guard: a 0 result is not a string handle).
    let out = run_source_with_env(
        "if (Deno.env.get(\"KALI_STREQ_MISSING\") === \"y\") { throw new Error(\"missing env equalled a string\"); }\nif (Deno.env.get(\"KALI_STREQ_MISSING\") === \"\") { throw new Error(\"missing env equalled empty string\"); }\nconsole.log(\"ok\");\n",
        "KALI_STREQ_MISSING",
        None,
    );
    assert_ok(&out);
}

#[test]
fn env_get_empty_value_equals_empty_literal() {
    // node analog: env K = ""; K === "" → true (present-but-empty is a REAL
    // empty string, distinct from missing/undefined).
    let out = run_source_with_env(
        "if (Deno.env.get(\"KALI_STREQ_EMPTY\") !== \"\") { throw new Error(\"empty env value unequal to empty literal\"); }\nconsole.log(\"ok\");\n",
        "KALI_STREQ_EMPTY",
        Some(""),
    );
    assert_ok(&out);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_string_equality env_get --no-fail-fast`
Expected: FAIL — `is_string_valued` has no env-get arm, so these fall through to the raw `i64.eq` handle compare (fresh env handle vs interned literal → always unequal): the `!==` self-checks throw at runtime.

- [ ] **Step 3: Implement the recognizer + widen the arm**

In `operators.rs`, add next to `is_string_valued` (after its closing brace, line 887):

```rust
    /// True when `id` is a `Deno.env.get(...)` call — the SAME recognizer the
    /// call emitter routes with (`env_get_import_index`, intrinsics/host.rs),
    /// so this lane and the emission agree by construction. Its runtime value
    /// is a tagged string handle OR 0 (missing variable → JS `undefined`);
    /// `__streq`'s tag guard makes the 0 case unequal to every real string,
    /// which matches node (`undefined === s` is false for every string `s`).
    /// Deliberately NOT an `is_string_valued` arm: in `+`/`.length`/store
    /// positions a maybe-0 value must keep failing closed; only the equality
    /// lane (where `__streq` is total over 0) consults this.
    pub(crate) fn is_env_get_string_call(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let node = self.node(id);
        if node.kind != LirNodeKind::Call {
            return false;
        }
        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let callee_node = self.node(self.unwrap_transparent(callee));
        self.env_get_import_index(callee_node).is_some()
    }
```

(If `self.node(...)` returns a reference whose borrow conflicts with the second lookup, bind `let callee = …` first as shown — the emitter's other recognizers use the same shape.)

Then widen the Task 2 arm's condition. Replace:

```rust
        if matches!(op, "==" | "!=" | "===" | "!==")
            && self.is_string_valued(left)
            && self.is_string_valued(right)
        {
```

with:

```rust
        let is_equality_op = matches!(op, "==" | "!=" | "===" | "!==");
        let left_string = is_equality_op && self.is_string_valued(left);
        let right_string = is_equality_op && self.is_string_valued(right);
        let left_env = is_equality_op && !left_string && self.is_env_get_string_call(left);
        let right_env = is_equality_op && !right_string && self.is_env_get_string_call(right);
        // At most ONE env-get operand: both env.get results materialize into
        // the SAME reserved buffer (call.rs env lane), so env-vs-env would
        // read the second call's bytes twice and spuriously equal any two
        // same-length values. Env-vs-env keeps today's path (follow-up
        // F-Stage1-2 in the Stage 1 triage doc).
        if (left_string || left_env)
            && (right_string || right_env)
            && !(left_env && right_env)
        {
```

(The `is_equality_op` guards keep the oracle calls off the hot non-equality path; everything else in the arm body is unchanged.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_string_equality`
Expected: PASS (all 14).

- [ ] **Step 5: Record follow-up F-Stage1-2**

Append to `docs/superpowers/followups/throw-fallout-stage1-triage.md` under "follow-ups opened this stage":

> **F-Stage1-2 — env-vs-env equality is unsound (pre-existing).** `Deno.env.get(a) == Deno.env.get(b)` compares two handles aliasing the SAME reserved buffer (call.rs env lane, buffer offset 0): on `main` the identity compare is wrong for equal-length differing values, and Stage 1 deliberately does NOT route env-vs-env through `__streq` (the second call overwrites the first's bytes pre-compare). Fix requires per-call buffers or copy-out — host-wiring family, candidate for Stage 3.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/runtime_string_equality.rs docs/superpowers/followups/throw-fallout-stage1-triage.md
git commit -m "fix(codegen): Deno.env.get single-operand equality via __streq with tag-guarded missing case (throw-fallout Stage 1)"
```

---

### Task 5: Enumeration-key equality reproducers (the headline class)

**Files:**
- Test: `crates/kali_cli/tests/runtime_string_equality.rs` (append)

**Interfaces:**
- Consumes: the Task 2 lane. These forms should ALREADY be covered: enumeration keys reach the arm via `is_string_valued`'s computed-element arm (operators.rs:826, element repr `String`) and its bare-identifier arm (operators.rs:863, repr-lifted keys).
- Produces: pinned proof that the four bucket shapes (keys array element, for-of key variable, Object.entries pair element, for-in key) compare by content.

- [ ] **Step 1: Write the tests**

Append to `crates/kali_cli/tests/runtime_string_equality.rs`:

```rust
// The headline #2/#3 bucket shapes (throw-fallout denominator): enumeration
// keys are FRESH runtime buffers; `!==` against an interned literal was true
// by handle identity even when the text matched. All node-derived.

#[test]
fn object_keys_element_equality() {
    // The exact browser_object_keys_harness self-check shape.
    let out = run_source(
        "const values = { \"b\": 1, \"a\": 2 };\nconst keys = [];\nfor (const key of Object.keys(values)) {\n  keys.push(key);\n}\nif (keys.length !== 2 || keys[0] !== \"b\" || keys[1] !== \"a\") {\n  throw new Error(\"unexpected Object.keys iteration semantics\");\n}\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn object_keys_loop_variable_equality() {
    // Direct compare of the for-of binding (no array round-trip).
    let out = run_source(
        "const o = { \"b\": 1, \"a\": 2 };\nlet seen = 0;\nfor (const key of Object.keys(o)) {\n  if (seen === 0 && key !== \"b\") { throw new Error(\"first key mismatch\"); }\n  seen = seen + 1;\n}\nif (seen !== 2) { throw new Error(\"key count mismatch\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn for_in_key_equality() {
    // Spec 4a materialized for-in keys are repr-lifted `String`.
    let out = run_source(
        "const o = { \"b\": 1, \"a\": 2 };\nlet matched = 0;\nfor (const k in o) {\n  if (k === \"b\") { matched = matched + 1; }\n  if (k === \"a\") { matched = matched + 1; }\n}\nif (matched !== 2) { throw new Error(\"for-in key equality failed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn object_entries_key_equality() {
    let out = run_source(
        "const o = { \"b\": 1, \"a\": 2 };\nconst names = [];\nfor (const pair of Object.entries(o)) {\n  names.push(pair[0]);\n}\nif (names[0] !== \"b\" || names[1] !== \"a\") {\n  throw new Error(\"entries key equality failed\");\n}\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p kali_cli --test runtime_string_equality --no-fail-fast`
Expected: PASS. If a specific shape FAILS, the oracle arm for that operand form is missing or misrouted in `is_string_valued` (operators.rs:808) — use superpowers:systematic-debugging on that one shape; the fix belongs in the oracle (with its documented both-sides discipline), NEVER in weakening the test. If `object_entries_key_equality`'s `pair[0]` shape turns out to be outside the provable element-repr lane (a pre-existing gap unrelated to equality), record it in the triage doc as expected-to-remain with its root cause and delete that one test — do not pin a wrong result.

- [ ] **Step 3: Commit**

```bash
git add crates/kali_cli/tests/runtime_string_equality.rs
git commit -m "test(cli): pin enumeration-key content equality — keys/entries/for-in shapes (throw-fallout Stage 1)"
```

---

### Task 6: Re-mask check and fail-closed backstops

**Files:**
- Test: `crates/kali_cli/tests/runtime_string_equality.rs` (append)

**Interfaces:**
- Consumes: Tasks 1-4 behavior. No production code changes in this task; if any step's expectation fails, that is a Task 1-4 defect to fix there.

- [ ] **Step 1: Write the tests**

Append to `crates/kali_cli/tests/runtime_string_equality.rs`:

```rust
// ---- Invariant 3 (no re-masking) + fail-closed backstops ----

#[test]
fn wrong_comparison_self_check_still_fails() {
    // Invariant 3: the fix must not re-silence self-check throws. A comparison
    // that is genuinely false must take the throw path and fail the run
    // (print-then-trap → non-zero exit).
    let out = run_source(
        "const keys = Object.keys({ \"b\": 1 });\nif (keys[0] !== \"nope\") {\n  throw new Error(\"honest failure\");\n}\nconsole.log(\"unreachable ok\");\n",
    );
    assert!(
        !out.status.success(),
        "a false comparison's throw must fail the run; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        printed.contains("honest failure"),
        "throw's print-then-trap message missing; combined output: {printed}"
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("unreachable ok"),
        "execution continued past a throw"
    );
}

#[test]
fn mixed_tainted_equality_still_rejects_e3200() {
    // Fail-closed backstop: a tainted string against a NON-string operand
    // still hits the E3200 reject (the Task 2 arm requires BOTH sides
    // string-proven; the reject lane below it is retained for this residue).
    let out = run_source(
        "function f(s) {\n  if ((s + \"y\") == 5) {\n    console.log(1);\n  }\n}\nf(\"x\");\n",
    );
    assert!(
        !out.status.success(),
        "mixed tainted-string == number must stay rejected; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E3200"), "expected E3200, stderr: {stderr}");
}

#[test]
fn proven_string_vs_number_strict_equality_unchanged() {
    // Mixed lane pin (spec: out of scope, unchanged): an UNTAINTED proven
    // string against a number keeps today's handle-vs-number compare, which
    // agrees with node for `===` (false). node: "hi" === 5 → false. The `==`
    // coercion divergence ("5" == 5) is follow-up F-Stage1-1, NOT fixed here.
    let out = run_source(
        "let s = \"hi\";\nif (s === 5) { throw new Error(\"string equalled number\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}
```

Note on the third test: if the types-side gate turns out to reject `s === 5` at compile time (E3200 family) rather than compiling the accidental-correct compare, that is ALSO acceptable unchanged behavior — but then re-shape the pin to assert the rejection (`!success` + `E3200`), matching whatever `main` does for the same source. Verify against `main` behavior first: `git -C /workspace/.worktrees/kali-main` hosts the same test harness — or simply run the source through the branch binary and read which of the two shapes it takes today (this expression is NOT in the Task 2 lane either way, so branch == main for it).

- [ ] **Step 2: Run the tests**

Run: `cargo test -p kali_cli --test runtime_string_equality --no-fail-fast`
Expected: PASS (all 21, or 20 + the reshaped mixed pin).

- [ ] **Step 3: Run the untouched-reject regression sweep**

Run: `cargo test -p kali_cli --test runtime_string_value_flow --test runtime_argv --no-fail-fast`
Expected: ALL PASS — in particular `concat_result_truthiness_is_rejected`, `concat_result_relational_is_rejected`, `unary_minus_on_runtime_argv_string_still_fails_closed`, `unary_bitnot_on_runtime_argv_string_still_fails_closed` (the non-equality rejects the spec keeps).

- [ ] **Step 4: Commit**

```bash
git add crates/kali_cli/tests/runtime_string_equality.rs
git commit -m "test(cli): re-mask check + mixed-operand fail-closed backstops for string equality (throw-fallout Stage 1)"
```

---

### Task 7: Stage gate — enumeration diff, drain snapshot, hygiene

**Files:**
- Modify: `docs/superpowers/followups/throw-fallout-stage0-denominator.md` (append a "Stage 1 drain" section)
- Modify: `docs/superpowers/followups/throw-fallout-stage1-triage.md` (final counts)

**Interfaces:**
- Consumes: `/tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-pre.txt` (Task 0).
- Produces: the stage checkpoint verdict + the updated drain snapshot the next stage starts from.

- [ ] **Step 1: Browser-lane spot check**

Run: `cargo test -p kali_cli --test browser_object_keys_harness --test browser_object_entries_harness --no-fail-fast`
Expected: ALL PASS (these were 41 + 32 red in the denominator listing). Requires the browser harness runtime (node) available — same environment the denominator enumeration used.

- [ ] **Step 2: Hygiene**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets`
Expected: fmt makes no complaints on re-run (`git diff --stat` empty after the second `cargo fmt --all`); clippy clean. Fix any warning introduced by this stage (e.g. an unused import); do not touch pre-existing warnings outside the stage's files.

- [ ] **Step 3: Full post-stage enumeration**

Run:
```bash
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-post.log
grep -E '^test .+ \.\.\. FAILED$' /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-post.log | sed 's/^test //; s/ \.\.\. FAILED$//' | sort > /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-post.txt
comm -13 /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-pre.txt /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-post.txt
wc -l /tmp/claude-1000/-workspace/8bded042-0c7d-404e-a4aa-97c47ef7e39b/scratchpad/stage1-post.txt
```
Expected:
- The `comm -13` (names red NOW that were not red pre-stage) output is **EMPTY**. Any name there is a regression this stage introduced — fix it before proceeding (the five Task 3 re-pins were red only BETWEEN Tasks 2 and 3 and are green again, so they must not appear).
- The post count is STRICTLY below the pre count, with the drain dominated by the #2/#3 bucket. The remaining red set should consist of the other buckets (async 169, delete-reinsert 46, host wiring ~45, dynamic import 32, short-circuit 13, array/for-of 16) plus any #2/#3 entries the triage doc predicted would remain (overlap shapes needing Stages 2/3/7).

- [ ] **Step 4: Snapshot the drain**

Append to `docs/superpowers/followups/throw-fallout-stage0-denominator.md`:

```markdown
## Stage 1 drain (runtime string equality)

**Date:** <fill: today> · **Commits:** <fill: Task 1..7 short SHAs>

| measure | count |
|---|---|
| pre-stage failing set (Task 0 enumeration) | <fill> |
| post-stage failing set | <fill> |
| drained by Stage 1 | <fill> |
| #2/#3 entries remaining (overlap → later stages) | <fill> |
| tests re-pinned (equality-reject pins, node-derived) | 5 |

Newly-red vs pre-stage: none (gate requirement).
Remaining red by bucket: <fill: one line per bucket with counts>.
Follow-ups opened: F-Stage1-1 (mixed `==` coercion, spec), F-Stage1-2 (env-vs-env
equality, triage doc).
```

Fill every `<fill>` with the measured numbers (no placeholders may survive the commit), and update the triage doc's counts to match.

- [ ] **Step 5: Verdict command**

Run: `cargo test --workspace ; echo "exit: $?"`
Expected: still non-zero exit (Stages 2-7 buckets remain red — the PROGRAM finishes at zero, this stage's gate is strict shrink + zero regressions, both established in Step 3). Record the first-failing-binary name in the triage doc for continuity.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/followups/throw-fallout-stage0-denominator.md docs/superpowers/followups/throw-fallout-stage1-triage.md
git commit -m "docs(soundness): throw-fallout Stage 1 drain snapshot — string-equality bucket vs the 977 denominator"
```

---

## Out of scope (recorded, do not implement)

- Relational `< <= > >=` on runtime strings (static-ASCII fold + reject unchanged).
- String truthiness / logical-operand rejects (unchanged).
- Mixed-type `==` coercion (`"5" == 5`) — follow-up F-Stage1-1 (spec §Scope).
- Env-vs-env equality — follow-up F-Stage1-2 (Task 4).
- `console.log` boolean formatting (`1`/`0` vs node's `true`/`false`) — pre-existing pinned lane.
- Word-width compare-loop optimization in `__streq` — only if a benchmark ever cares.
