# fasta Output Layer + `process.argv`/N (Spec 5) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compile fasta-node-1's two output functions (`fastaRepeat`, `fastaRandom`) plus `n = +process.argv[2]` to WASM, so the full fasta program runs byte-for-byte vs `node` with N read from the CLI.

**Architecture:** Almost the entire output layer is already assembled from shipped primitives (one/two-arg `substring`, two-substring `+`, `new Array(n)` + reassignment + `line[i]=…` + `.join('')`, per-line `console.log`, `for..in`, mutable-global LCG). Only four small gaps remain: (1) `break` out of a `for..in`, (2) a `for..in` key materialized after the loop into a string-element array, (3) `process.argv[i]` → runtime string handle, (4) unary `+` string→number coercion. Each is a fail-closed relaxation landed with its codegen lane and both-sides oracle arms in the same task.

**Tech Stack:** Rust workspace (kali_codegen, kali_types, kali_cli, kali_mir), `wasm-encoder`, WASM target, `node` v26.4.0 for goldens.

## Global Constraints

Copied verbatim from the spec (`docs/superpowers/specs/2026-07-08-fasta-output-layer-and-argv-spec5-design.md`). Every task's requirements implicitly include these.

- **Fail-closed, never fail-open.** Unprovable receiver/key/target/coercion rejects `E5506` (`e5::FEATURE_UNAVAILABLE = 5506`) or `E3` (`e3::TYPE_MISMATCH`) rather than miscompiling. Reject-don't-miscompile for any unproven variant.
- **Both-sides oracle mirroring.** Every NEW expression shape gets arms on BOTH the codegen recognizers AND the four `kali_types` predicates (`crates/kali_types/src/resolve/expression.rs`: `expression_is_string_typed` :69, `operand_repr_is_string` :758, `expression_is_length_fold_receiver` :971, `expression_is_runtime_string_value` :1035) IN THE SAME CHANGE, or it fails open.
- **Both-walks ordinal safety.** `for..in` takes NO arena and NO loop ordinal. Do NOT add `for-in` to the codegen loop whitelist (`loop_preorder_ordinals`); leave `kali_mir` walk's `ForInStmt` arm skipping `arena_enter_loop`. Task 1 adds a `LoopFrame` (break/continue label bookkeeping) but NOT a loop-arena ordinal — these are independent mechanisms.
- **Strings never dangle.** Runtime string alloc (argv handles, materialized key handles) routes through `StringPool::intern` / `__alloc_global` (globals g4/g5/g6, never reset), NEVER the resettable `__alloc` (g1/g2/g3).
- **Handle encoding.** `STRING_HANDLE_TAG (0x8000_0000_0000_0000) | (offset<<32) | len` (byte count) via `encode_string_handle(offset, len)` (`kali_codegen/src/lower.rs:3259`). Intern via `self.strings.intern(text) -> (offset, len)`.
- **No new host imports beyond `args_get`.** Task 5 adds exactly ONE (`args_get`, already host-registered in `imports_default.rs:685`). The 4 hand-mirrored `kali:rt` JS import lists (`kali_runtime/src/browser/harness.rs:198` and `:530`; `kali_cli/src/bin/cmd_build.rs:1553` and `:1817`) change ONLY to add `args_get`. Task 6 (coercion) adds NO import (inline parse).
- **Base-behavior invariants (guardrails).** All 5 CLBG fixtures byte-identical: nbody, fannkuch-redux, spectral-norm, mandelbrot, binary-trees. binary-trees is the both-walks arena guardrail. Static object-fold + numeric for/while unchanged.
- **GC-less invariant.** No runtime string→value map; the `line` array and argv handles are region/persistent allocations, not GC roots.
- **Per-task gate:** `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir` + `cargo clippy -p <touched crates> -- -D warnings`. Final task (7) adds `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`.
- **Conventions:** conventional-commit messages; commit after every task. Synthetic top-level fn name = `_start`. Node for goldens: v26.4.0. Golden outputs independently re-derived twice (implementer + reviewer).
- **Integration:** HOLD until whole-branch review is clean, then push a PR + self-merge per `kali-integration-convention` (`gh` authed as `rahulmutt`; `gh auth setup-git` if git can't read credentials).

---

## File Structure

- `crates/kali_codegen/src/emit/control_flow.rs` — `emit_for_in` gains a `LoopFrame` push/pop (Task 1). Already ~1560 lines; no split (established large-file pattern).
- `crates/kali_types/src/repr_infer.rs` — array-element-store arm gains a `seed_for_in_key_string_use` call (Task 2).
- `crates/kali_codegen/src/intrinsics/host.rs` + `crates/kali_codegen/src/emit/operators.rs` — `process.argv[i]` element read recognizer + emit (Task 5).
- `crates/kali_codegen/src/lower.rs`, `crates/kali_codegen/src/lib.rs`, `crates/kali_codegen/src/emitter.rs` — `args_get` conditional import + index threading + a reserved scratch local (Task 5).
- `crates/kali_runtime/src/browser/harness.rs`, `crates/kali_cli/src/bin/cmd_build.rs` — add `args_get` to the 4 JS import lists (Task 5).
- `crates/kali_types/src/resolve/expression.rs` + `crates/kali_types/src/static_analysis/` — `process.argv[i]` string classification arms; unary-`+`-coercion acceptance (Tasks 5, 6).
- `crates/kali_codegen/src/emit/operators.rs` — `emit_unary` `+` arm inline string→i64 parse (Task 6).
- `crates/kali_cli/tests/runtime_forin.rs` — Task 1/2 unit e2e.
- `crates/kali_cli/tests/runtime_fasta_output.rs` (new) — Task 3/4 shells.
- `crates/kali_cli/tests/runtime_argv.rs` (new) — Task 5/6 e2e (needs a CLI-arg helper).
- `crates/kali_cli/tests/runtime_fasta_capstone.rs` (new) — Task 7 full capstone.

---

### Task 1: `break` out of a `for..in`

Today `emit_for_in` emits a `Block { Loop { … } }` but never pushes a `LoopFrame`, so a `break` inside a for-in body targets the *enclosing* loop (a miscompile when nested) or errors `E5` "break outside loop" (when the for-in has no enclosing loop). Push a `LoopFrame` so an unlabeled `break`/`continue` inside the for-in targets the for-in itself.

**Files:**
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (`emit_for_in`, ~484–514)
- Test: `crates/kali_cli/tests/runtime_forin.rs`

**Interfaces:**
- Consumes: `push_control_frame(ControlFlowLabelKind)`, `pop_control_frame(...)`, `self.loop_frames: Vec<LoopFrame>`, `LoopFrame { break_index, continue_index }` — all already used by `emit_loop` (control_flow.rs:250–258, 325–328).
- Produces: no new public symbols; a `break`/`continue` inside a `for..in` body now resolves to the for-in loop.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_cli/tests/runtime_forin.rs`:

```rust
#[test]
fn break_targets_the_forin_loop_not_the_enclosing_loop() {
    // Inner `for..in` breaks after ONE field on every outer iteration.
    // Correct: break exits the for-in, outer runs twice -> out == 2.
    // Bug (break targets outer `for`): out == 1.
    let out = run_source(
        "function f(t) {\n  var out = 0;\n  for (var i = 0; i < 2; i = i + 1) {\n    for (var c in t) { out = out + 1; break; }\n  }\n  return out;\n}\nconst t = { a: 1, c: 2 };\nconsole.log(f(t));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

#[test]
fn break_inside_bare_forin_with_no_enclosing_loop() {
    // A `break` in a for-in with no enclosing loop must target the for-in
    // (before this task it errored "break outside loop"). Breaks at n==2.
    let out = run_source(
        "function f(t) {\n  var n = 0;\n  for (var c in t) { n = n + 1; if (n == 2) break; }\n  return n;\n}\nconst t = { a: 1, c: 2, g: 3 };\nconsole.log(f(t));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_forin break_targets_the_forin_loop_not_the_enclosing_loop break_inside_bare_forin_with_no_enclosing_loop`
Expected: FAIL — first asserts `1\n != 2\n` (wrong loop) or a crash; second fails with a compile error ("break … outside the supported static loop lowering path") so `status.success()` is false.

- [ ] **Step 3: Push a `LoopFrame` in `emit_for_in`**

In `crates/kali_codegen/src/emit/control_flow.rs`, replace the bare block/loop opening (the two lines at ~487–488):

```rust
        // block (break target) { loop (continue target) { ... } }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
```

with frame-tracked open (mirrors `emit_loop`):

```rust
        // block (break target) { loop (continue target) { ... } }. Register the
        // labels so a `break`/`continue` inside the body targets THIS for-in
        // (not an enclosing loop). No loop-arena ordinal is involved — this is
        // label bookkeeping only; for..in still takes no arena.
        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.loop_frames.push(LoopFrame {
            break_index,
            continue_index,
        });
```

Then replace the loop/block close (the two `End` lines at ~513–514):

```rust
        function.instruction(&Instruction::End); // end loop
        function.instruction(&Instruction::End); // end block
```

with frame-tracked close:

```rust
        function.instruction(&Instruction::End); // end loop
        self.loop_frames.pop();
        self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::End); // end block
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);
```

The existing `BrIf(1)` (break-when-`ord >= N`) and `Br(0)` (back-edge) are unchanged: they are structural depths inside the emitted `Block/Loop`, independent of the label bookkeeping. The key local (`key_local`) and ordinal local (`ord_local`) already retain their last value after a `break`, so the key stays live for Task 2.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_forin break_targets_the_forin_loop_not_the_enclosing_loop break_inside_bare_forin_with_no_enclosing_loop`
Expected: PASS (both print `2\n`).

- [ ] **Step 5: Run the per-task gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir && cargo clippy -p kali_codegen -- -D warnings`
Expected: all green. (Especially: existing `runtime_forin` and binary-trees fixtures unchanged — the for-in body's own break/normal exit is byte-identical when no `break` is present.)

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/control_flow.rs crates/kali_cli/tests/runtime_forin.rs
git commit -m "feat(codegen): break/continue target the for..in loop (fasta Spec 5 Task 1)"
```

---

### Task 2: `for..in` key materialized after the loop into a string-element array

fasta's `fastaRandom` inner loop is `for (var c in table) if (r < table[c]) break; line[i] = c;` — the key `c` is stored into a string-element array AFTER the for-in exits (via `break`). Today this is REJECTED: the for-in-key provenance persists in the function scope (grow-only registry, no lexical boundary), so the default-deny gate at `resolve/expression.rs:1782` fires on the `c` read (repr not String, not a suppressed safe position). The fix is structural: make "a for-in key stored into an array element" a String-materialization sink in `repr_infer` — exactly like the existing `return c` / `console.log(c)` / `+`/`==` sinks. That lifts `c`'s repr to `String` (so `identifier_repr_is_string("c")` is true and the gate's materialized-key carve-out admits the read) AND, via the existing store edge `rn -> elem`, lifts `line`'s element repr to `String` (so `line.join('')` and `string_element_array_binding("line")` succeed). No codegen change is needed: the store/materialization path is the same one the shipped `collect` test (`runtime_forin.rs:196`, `out[i] = c` inside the loop) already exercises byte-for-byte.

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs` (array-element-store arm, ~1079–1093)
- Test: `crates/kali_cli/tests/runtime_forin.rs`

**Interfaces:**
- Consumes: `seed_for_in_key_string_use(func: &str, expr: &Expression)` (repr_infer.rs:284 — no-ops unless `expr` is an active for-in key); the existing `add_edge(rn, elem)` store edge (1088).
- Produces: for a for-in key `c`, `scalar(func, c) == Repr::String` and `array_element(func, line) == Repr::String` whenever `line[i] = c` appears; downstream `string_element_array_binding` / `.join` gates then accept.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_cli/tests/runtime_forin.rs`:

```rust
#[test]
fn forin_key_stored_into_string_array_after_break_then_joined() {
    // The fasta `fastaRandom` inner shape: select a key via break, store it
    // into a preallocated array by index, join. Keys are a,c,g.
    //   i=0: r=0 -> c=a (break immediately)
    //   i=1: r=1 -> a(skip),c (break)
    //   i=2: r=2 -> a,c(skip),g (break)  => "acg"
    let out = run_source(
        "function build(t) {\n  var line = new Array(3);\n  for (var i = 0; i < 3; i = i + 1) {\n    var r = i;\n    for (var c in t) { if (r < 1) break; r = r - 1; }\n    line[i] = c;\n  }\n  return line.join(\"\");\n}\nconst t = { a: 1.0, c: 2.0, g: 3.0 };\nconsole.log(build(t));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "acg\n");
}

#[test]
fn forin_key_stored_into_non_string_context_still_rejects() {
    // Fail-closed pin: storing a for-in key into a NUMBER-typed slot (used as
    // an ordinal elsewhere, no string sink) must NOT silently succeed as a
    // string. Here `c` flows only to a numeric field -> the string sink never
    // fires; a raw-ordinal leak into a returned number must reject, not print
    // an ordinal. (Guards against the sink over-firing.)
    let out = run_source(
        "function f(t) { for (var c in t) { return c + 1; } return 0; }\nconst t = { a: 1.0 };\nconsole.log(f(t));\n",
    );
    // `c + 1` with a string key is number-context misuse: rejected E5/E3, not "01" or an ordinal.
    assert!(!out.status.success(), "must reject for-in key in a numeric `+`; stdout: {}", String::from_utf8_lossy(&out.stdout));
}
```

- [ ] **Step 2: Run the tests to verify the first fails**

Run: `cargo test -p kali_cli --test runtime_forin forin_key_stored_into_string_array_after_break_then_joined forin_key_stored_into_non_string_context_still_rejects`
Expected: `forin_key_stored_into_string_array_after_break_then_joined` FAILS (compile rejects `line[i] = c` with "a for..in key value ('c') is only usable as …"). `forin_key_stored_into_non_string_context_still_rejects` already PASSES (pre-existing reject) — it is a regression guard.

- [ ] **Step 3: Add the string-sink seed at the array-element store**

In `crates/kali_types/src/repr_infer.rs`, in the array-element-store arm, after `self.element_store_sources.push((elem, rn));` (line ~1089) add the seed call:

```rust
                if let Expression::Identifier(name) = &member.object {
                    let elem = self.array_elem_node_for(func, name);
                    // Store is directed: value -> element (a float value floats
                    // the array; an int value into a float array stays int).
                    self.add_edge(rn, elem);
                    self.element_store_sources.push((elem, rn));
                    // Spec 5: a `for..in` key stored into an array element is a
                    // string-materialization sink, exactly like `return c` /
                    // `console.log(c)` / `+`/`==`. Seed the key's scalar node
                    // String so (a) the element axis lifts to String via the
                    // edge above (enabling `.join('')` and the
                    // `string_element_array_binding` gate) and (b) the resolve
                    // gate's materialized-key carve-out
                    // (`identifier_repr_is_string`) admits the `c` read after
                    // the loop. No-ops unless the RHS is an active for-in key.
                    self.seed_for_in_key_string_use(func, &assign.right);
                } else {
                    self.visit_expr(func, &member.object);
                }
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test runtime_forin forin_key_stored_into_string_array_after_break_then_joined forin_key_stored_into_non_string_context_still_rejects`
Expected: PASS (first prints `acg\n`; second still rejects).

- [ ] **Step 5: Run the per-task gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir && cargo clippy -p kali_types -- -D warnings`
Expected: green. Confirm the shipped `collect` test (`runtime_forin.rs` ~:196) and all `runtime_string_arrays` / `runtime_join` tests still pass (the seed is additive and no-ops for non-for-in-key stores).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_cli/tests/runtime_forin.rs
git commit -m "feat(types): for..in key stored into an array element is a string sink (fasta Spec 5 Task 2)"
```

---

### Task 3: `fastaRandom` shell byte-for-byte

Assemble Tasks 1+2 with the already-shipped LCG mutable globals, `new Array(n)` reassignment (`runtime_string_arrays.rs:104`/`:118`), `makeCumulative`, and per-line `console.log(line.join(''))`. This is a verification task — it should pass with NO new production code once Tasks 1–2 land; if a gap surfaces, close it here and note it.

**Files:**
- Test: `crates/kali_cli/tests/runtime_fasta_output.rs` (new)

**Interfaces:**
- Consumes: everything from Tasks 1–2 plus shipped primitives. Produces: no new symbols.

- [ ] **Step 1: Write the shell + failing golden test**

Create `crates/kali_cli/tests/runtime_fasta_output.rs`. Use the same `kali_bin()`/`run_source` helper block as `runtime_forin.rs` (copy the top 24 lines, renaming the temp-dir slug to `kali-fasta-output`). Then:

```rust
const FASTA_RANDOM_SHELL: &str = "\
var last = 42;
function rand(max) { last = (last * 3877 + 29573) % 139968; return max * last / 139968; }
function makeCumulative(table) {
  var prev = null;
  for (var c in table) {
    if (prev) table[c] = table[c] + table[prev];
    prev = c;
  }
}
function fastaRandom(n, table) {
  var line = new Array(60);
  makeCumulative(table);
  while (n > 0) {
    if (n < line.length) line = new Array(n);
    for (var i = 0; i < line.length; i = i + 1) {
      var r = rand(1);
      for (var c in table) { if (r < table[c]) break; }
      line[i] = c;
    }
    console.log(line.join(\"\"));
    n = n - line.length;
  }
}
var IUB = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02, K: 0.02, M: 0.02, N: 0.02, R: 0.02, S: 0.02, V: 0.02, W: 0.02, Y: 0.02 };
fastaRandom(70, IUB);
";

#[test]
fn fasta_random_shell_matches_node() {
    // GOLDEN: derive by running the SAME source under node v26.4.0 and pasting
    // its stdout below. Re-derive independently a second time (reviewer) before
    // merge. Command:
    //   node --input-type=module -e "$(cat <<'EOF'
    //   <FASTA_RANDOM_SHELL contents, with makeCumulative's += restored as table[c] += table[prev]>
    //   EOF
    //   )"
    // The += is written as `table[c] = table[c] + table[prev]` in the shell to
    // avoid a compound-index dependency; it is numerically identical to node's.
    const GOLDEN: &str = "<PASTE node stdout: two 60-col lines + one 10-col line, each ending \\n>";
    let out = run_source(FASTA_RANDOM_SHELL);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), GOLDEN);
}
```

Derive `GOLDEN` now: write `FASTA_RANDOM_SHELL` (with the `table[c] = table[c] + table[prev]` form) to a temp `.mjs`, run `node <file>`, and paste its exact stdout (three lines: 60 + 60 + 10 chars, `70` total). Keep the `\n`-terminated form.

- [ ] **Step 2: Run to verify it fails on the placeholder golden**

Run: `cargo test -p kali_cli --test runtime_fasta_output fasta_random_shell_matches_node`
Expected: FAIL if `GOLDEN` is still the placeholder, or FAIL if a production gap remains. If it fails with a compile error, that is a real gap — debug via `superpowers:systematic-debugging` and close it here (record what was missing in the commit body).

- [ ] **Step 3: Fill in the derived golden (and close any gap)**

Replace the `GOLDEN` placeholder with node's exact stdout. If Step 2 revealed a codegen/types gap, fix it minimally in the touched crate; otherwise no production change.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p kali_cli --test runtime_fasta_output fasta_random_shell_matches_node`
Expected: PASS.

- [ ] **Step 5: Run the per-task gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir && cargo clippy -p kali_cli -- -D warnings`
Expected: green.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/runtime_fasta_output.rs
git commit -m "test(cli): fastaRandom shell byte-for-byte vs node (fasta Spec 5 Task 3)"
```

---

### Task 4: `fastaRepeat` shell byte-for-byte

Assemble the shipped substring primitives: one-arg `seq.substring(seqi)`, two-arg `seq.substring(a, b)`, two-substring `+` concat at the wrap boundary, `while`, and per-line `console.log`. Verification task — should pass with no new production code; close any gap if found.

**Files:**
- Test: `crates/kali_cli/tests/runtime_fasta_output.rs` (append)

**Interfaces:**
- Consumes: shipped substring/concat/console.log. Produces: none.

- [ ] **Step 1: Write the shell + failing golden test**

Append to `crates/kali_cli/tests/runtime_fasta_output.rs`:

```rust
const FASTA_REPEAT_SHELL: &str = "\
function fastaRepeat(n, seq) {
  var seqi = 0;
  var lenOut = 60;
  while (n > 0) {
    if (n < lenOut) lenOut = n;
    if (seqi + lenOut < seq.length) {
      console.log(seq.substring(seqi, seqi + lenOut));
      seqi = seqi + lenOut;
    } else {
      console.log(seq.substring(seqi) + seq.substring(0, lenOut - (seq.length - seqi)));
      seqi = lenOut - (seq.length - seqi);
    }
    n = n - lenOut;
  }
}
var ALU = \"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG\" + \"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA\";
fastaRepeat(120, ALU);
";

#[test]
fn fasta_repeat_shell_matches_node() {
    // ALU here is 84 chars (two 42-char segments). fastaRepeat(120) emits:
    //   line 1: chars [0,60)               (mid-string branch)
    //   line 2: chars [60,84)+[0,36)       (wrap-boundary else branch)
    // GOLDEN: derive by running FASTA_REPEAT_SHELL under node v26.4.0; paste
    // its exact stdout (two 60-col lines). Re-derive a second time (reviewer).
    const GOLDEN: &str = "<PASTE node stdout: two 60-char lines, each ending \\n>";
    let out = run_source(FASTA_REPEAT_SHELL);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), GOLDEN);
}
```

Derive `GOLDEN` via node now and paste it. (Both branches are exercised: line 1 is the `seqi+lenOut < length` branch; line 2 is the wrap-around `else`.)

- [ ] **Step 2: Run to verify it fails on the placeholder golden**

Run: `cargo test -p kali_cli --test runtime_fasta_output fasta_repeat_shell_matches_node`
Expected: FAIL on placeholder, or reveal a real gap (debug + close here).

- [ ] **Step 3: Fill in the derived golden (and close any gap)**

Paste node's exact stdout; fix any minimal gap in the touched crate if Step 2 surfaced one.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p kali_cli --test runtime_fasta_output fasta_repeat_shell_matches_node`
Expected: PASS.

- [ ] **Step 5: Run the per-task gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir && cargo clippy -p kali_cli -- -D warnings`
Expected: green.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/runtime_fasta_output.rs
git commit -m "test(cli): fastaRepeat shell byte-for-byte vs node (fasta Spec 5 Task 4)"
```

---

### Task 5: `process.argv[i]` element read → runtime string handle (N3)

`process.argv[<static int>]` yields `0` today (placeholder). Add the `args_get` host import to codegen (conditional, following the `stdout_write_bytes` pattern) and the 4 JS `kali:rt` lists, then emit: allocate a persistent buffer via `__alloc_global`, call `args_get(index, buf, cap)`, clamp the returned length to `>= 0`, and encode a `STRING_HANDLE_TAG | buf<<32 | len` handle. Classify `process.argv[<static int>]` as string-valued in the codegen `is_string_valued` mirror and the four `kali_types` predicates.

**Files:**
- Modify: `crates/kali_codegen/src/lib.rs` (import-index constants)
- Modify: `crates/kali_codegen/src/lower.rs` (`program_uses_args_get` probe, conditional import declaration, index threading, reserved scratch local)
- Modify: `crates/kali_codegen/src/emitter.rs` (`args_get_import_index` field + constructor)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (`is_process_argv_element` recognizer)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (element-read emit in the numeric-index arm)
- Modify: `crates/kali_types/src/resolve/expression.rs` (4 predicate arms) + `crates/kali_types/src/repr_infer.rs` (classify argv element as a runtime-string node)
- Modify: `crates/kali_runtime/src/browser/harness.rs` (2 lists), `crates/kali_cli/src/bin/cmd_build.rs` (2 lists)
- Test: `crates/kali_cli/tests/runtime_argv.rs` (new)

**Interfaces:**
- Consumes: `args_get` host fn (`imports_default.rs:685`, ABI `(index: i32, out_ptr: i32, out_cap: i32) -> i32` returns bytes-written or `-1`); `alloc_global` callee index (`__alloc_global`); `encode_string_handle`; the `stdout_write_bytes_import_index` conditional-import template (`lower.rs:159–172`, `389–392`; `emitter.rs:78,177,229`).
- Produces: `ARGS_GET` reachable from wasm; `process.argv[<int>]` emits a valid string handle; `is_process_argv_element(&self, node) -> Option<i64>` (returns the static index) shared by emit + oracles.

- [ ] **Step 1: Write the failing test**

Create `crates/kali_cli/tests/runtime_argv.rs`. Copy the `kali_bin()` helper from `runtime_forin.rs`, and add an args-passing runner:

```rust
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_node_source_with_args(src: &str, args: &[&str]) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-argv-{}-{}-{}",
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
fn process_argv_element_read_yields_the_string() {
    // On the node surface argv == ["node", <src>, ...guestArgs], so argv[2] is
    // the first guest arg. Printing it must echo "hello".
    let out = run_node_source_with_args(
        "console.log(process.argv[2]);\n",
        &["hello"],
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello\n");
}

#[test]
fn process_argv_element_length_is_the_byte_count() {
    let out = run_node_source_with_args(
        "console.log(process.argv[2].length);\n",
        &["abcd"],
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test runtime_argv process_argv_element_read_yields_the_string process_argv_element_length_is_the_byte_count`
Expected: FAIL — first prints `0\n` (placeholder) or a link error; second prints a wrong length.

- [ ] **Step 3: Add the `args_get` recognizer**

In `crates/kali_codegen/src/intrinsics/host.rs`, add next to `is_process_argv` (line 464):

```rust
    /// `process.argv[<int literal>]` (a computed element read on the argv
    /// receiver). Returns the static index. Only a static non-negative integer
    /// literal index is supported; anything else fails closed (falls through to
    /// the placeholder, which the caller must not treat as a string).
    pub(crate) fn is_process_argv_element(&self, node: &LirNode) -> Option<i64> {
        if node.children.len() != 2 {
            return None;
        }
        if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
            return None;
        }
        if !self.is_process_argv(node.children[0]) {
            return None;
        }
        let index = parse_number_literal(self.node(node.children[1]).text.as_deref()?)?;
        (index >= 0).then_some(index)
    }
```

(Confirm the computed-index node shape: a two-child `Value` `[object, index]`. If argv element indices lower as a one-child node with the index in `text` — as static array indices do — mirror `is_process_argv` on `node.children[0]` and read `node.text` for the index; verify against the actual LIR by adding a `dbg!` in Step 5 if the recognizer never matches.)

- [ ] **Step 4: Thread the `args_get` conditional import**

Follow the `stdout_write_bytes` template exactly:

1. `crates/kali_codegen/src/lower.rs` near line 80, add a probe + flag:

```rust
    let uses_args_get = program_uses_args_get(lir);
```

Add `+ if uses_args_get { 1 } else { 0 }` to the `function_index_offset` running sum (alongside the existing `uses_stdout_write_bytes` term), and add an `args_get_import_index` computed as `COVERAGE_HIT_IMPORT_INDEX + Σ(preceding conditional flags)` in the same block that computes `stdout_write_bytes_import_index` (~159–172), preserving declaration order vs the sum order. Conditionally declare it in the import section (near 389–392):

```rust
    if args_get_import_index.is_some() {
        import_section.import("kali:rt", "args_get", EntityType::Function(ARGS_GET_TYPE_INDEX));
    }
```

Add a `program_uses_args_get(lir: &LirProgram) -> bool` walker (mirror `program_uses_stdout_write_bytes`) that returns true iff any node satisfies `is_process_argv_element`-shape (a computed element read whose object `is_process_argv`). Add the function type `(i32, i32, i32) -> i32` to the `type_section` if not already present, capturing its index as `ARGS_GET_TYPE_INDEX`.

2. `crates/kali_codegen/src/lib.rs`: no fixed index constant needed (it is conditional, computed at runtime); leave the fixed block untouched.

3. `crates/kali_codegen/src/emitter.rs`: add `pub(crate) args_get_import_index: Option<u32>` (near line 78), the constructor param (177), and the init assignment (229) — mirroring `stdout_write_bytes_import_index`.

- [ ] **Step 5: Emit the element read**

In `crates/kali_codegen/src/emit/operators.rs`, in the computed-member arm that currently falls through to the placeholder (the numeric-index arm, ~261–304), add BEFORE the placeholder:

```rust
        if let Some(index) = self.is_process_argv_element(node) {
            let Some(args_get) = self.args_get_import_index else {
                // Probe/emit desync — fail closed rather than emit a bad call.
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "process.argv element read requires the args_get import".to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue { produced: true, shape: ValueShape::Scalar };
            };
            const ARGV_BUF_CAP: i32 = 256; // args longer than this trap in args_get; ample for CLI ints/paths.
            let buf_local = self.locals[&crate::lower::argv_buf_local_name()];
            let len_local = self.locals[&crate::lower::argv_len_local_name()];
            // buf = __alloc_global(CAP)
            function.instruction(&Instruction::I32Const(ARGV_BUF_CAP));
            function.instruction(&Instruction::Call(self.alloc_global_fn_index()));
            function.instruction(&Instruction::LocalSet(buf_local));
            // len = args_get(index, buf, CAP)
            function.instruction(&Instruction::I32Const(index as i32));
            function.instruction(&Instruction::LocalGet(buf_local));
            function.instruction(&Instruction::I32Const(ARGV_BUF_CAP));
            function.instruction(&Instruction::Call(args_get));
            // clamp len to >= 0 (out-of-range index returns -1 -> empty string)
            function.instruction(&Instruction::LocalTee(len_local));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::I32LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::LocalSet(len_local));
            function.instruction(&Instruction::End);
            // handle = TAG | (buf << 32) | len
            function.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
            function.instruction(&Instruction::LocalGet(buf_local));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Or);
            return EmittedValue { produced: true, shape: ValueShape::String };
        }
```

Reserve the two scratch locals: in `crates/kali_codegen/src/lower.rs`, add `pub(crate) fn argv_buf_local_name() -> String { "__argv_buf".into() }` and `argv_len_local_name() -> String { "__argv_len".into() }` (both `i32`), and in `collect_function_locals` push them when the function body contains a `process.argv` element read (mirror the `for_in_ord_local_name` reservation at ~1586, guarded by the same `is_process_argv_element`-shape walk). Confirm `alloc_global_fn_index()` exists on the emitter (the `__alloc_global` callee index — see `alloc_callee_index`, emitter.rs:315); if only `alloc_callee_index` exists, add a direct `alloc_global_fn_index()` accessor returning the `__alloc_global` function index unconditionally.

- [ ] **Step 6: Mirror the string classification (both sides + repr)**

- `crates/kali_types/src/resolve/expression.rs`: add a `process.argv[<int>]` arm returning `true` to `expression_is_string_typed` (:69), `operand_repr_is_string` (:758), and `expression_is_runtime_string_value` (:1035); `expression_is_length_fold_receiver` (:971) must return `false` for it (it is a runtime value, not a static fold). Add a shared `fn is_process_argv_element_expr(&self, expr: &Expression) -> bool` (a computed `MemberExpression` with an integer-literal index whose base member-name is `process.argv`) and key all four arms on it.
- `crates/kali_codegen/src/emit/operators.rs` / wherever `is_string_valued` lives: add the same `is_process_argv_element` case so `.length` and `+`-coercion (Task 6) see it as a string.
- `crates/kali_types/src/repr_infer.rs`: register `process.argv[<int>]` as a runtime-string node so its consuming binding solves `Repr::String` (mirror how a substring result is registered, ~1329/1349).

- [ ] **Step 7: Add `args_get` to the 4 JS import lists**

In `crates/kali_runtime/src/browser/harness.rs` (the two `"kali:rt": {` blocks at ~198 and ~530, each already defining `args_len`) and `crates/kali_cli/src/bin/cmd_build.rs` (the two blocks at ~1553 and ~1817), add an `args_get` entry mirroring the existing `args_len` shape (same wasmtime/JS binding surface). Match the ABI `(index, out_ptr, out_cap) -> i32` (writes the arg's UTF-8 bytes into guest memory at `out_ptr`, returns byte count or `-1`). Keep all four lists byte-identical to each other for `args_get`.

- [ ] **Step 8: Run to verify it passes**

Run: `cargo test -p kali_cli --test runtime_argv process_argv_element_read_yields_the_string process_argv_element_length_is_the_byte_count`
Expected: PASS (`hello\n`, `4\n`).

- [ ] **Step 9: Run the per-task gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir && cargo clippy -p kali_codegen -p kali_types -p kali_runtime -p kali_cli -- -D warnings`
Expected: green. Confirm the browser bundle tests still link (the 4 import lists stay consistent). Confirm `git diff` shows `args_get` added to exactly the 4 JS lists and nowhere else spurious.

- [ ] **Step 10: Commit**

```bash
git add crates/kali_codegen crates/kali_types crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs crates/kali_cli/tests/runtime_argv.rs
git commit -m "feat: process.argv[i] runtime string handle via args_get import (fasta Spec 5 Task 5)"
```

---

### Task 6: Unary `+` runtime-string → i64 coercion (N4)

`emit_unary`'s `+` arm currently REJECTS a runtime-string operand (`operators.rs:76–90`). Replace that for `+` only with an inline decimal→i64 parse over the string handle's bytes: `n = +process.argv[2]` must yield the integer `n`. `-`/`~` keep rejecting string operands (fasta needs neither). Result repr is `I64` (keeps `n` integer, consistent with the shells). Non-`+` unary string cases stay fail-closed.

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`emit_unary` guard + `"+"` arm)
- Modify: `crates/kali_types/src/` unary-`+` gate (allow `+` on a runtime string; classify the result numeric)
- Test: `crates/kali_cli/tests/runtime_argv.rs` (append)

**Interfaces:**
- Consumes: the string handle produced by Task 5 (`STRING_HANDLE_TAG | offset<<32 | len`); handle decode is `offset = (h >> 32) & 0x7FFF_FFFF`, `len = h & 0xFFFF_FFFF`.
- Produces: `+<runtime string>` emits an `i64` (JS `Math.trunc(Number(str))` for a decimal-integer arg); the result is numeric-typed for downstream repr inference.

- [ ] **Step 1: Write the failing test**

Append to `crates/kali_cli/tests/runtime_argv.rs`:

```rust
#[test]
fn unary_plus_coerces_argv_to_integer() {
    let out = run_node_source_with_args(
        "var n = +process.argv[2];\nconsole.log(n + 1);\n",
        &["1000"],
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1001\n");
}

#[test]
fn unary_plus_coerced_argv_drives_a_loop_count() {
    let out = run_node_source_with_args(
        "var n = +process.argv[2];\nvar s = 0;\nfor (var i = 0; i < n; i = i + 1) { s = s + i; }\nconsole.log(s);\n",
        &["5"],
    );
    // 0+1+2+3+4 = 10
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test runtime_argv unary_plus_coerces_argv_to_integer unary_plus_coerced_argv_drives_a_loop_count`
Expected: FAIL — `+process.argv[2]` rejects with `TYPE_MISMATCH` (guard at operators.rs:76), so `status.success()` is false.

- [ ] **Step 3: Replace reject-with-coerce for `+` on a runtime string**

In `crates/kali_codegen/src/emit/operators.rs`, narrow the guard at lines 76–90 to exclude `+` (keep `-`/`~`/`!`):

```rust
        if (matches!(op, "-" | "~") && self.is_string_valued(arg))
            || (op == "!" && self.is_runtime_concat_string(arg))
        {
            // (unchanged reject body)
        }
```

Then in the `"+"` arm (currently `"+" => self.emit_node(function, arg, true),` at line 113), branch on a string operand:

```rust
            "+" => {
                if self.is_string_valued(arg) {
                    return self.emit_string_to_i64_parse(function, arg);
                }
                self.emit_node(function, arg, true)
            }
```

Add the inline parser (new method in `operators.rs`), which decodes the handle and accumulates a base-10 i64 over its bytes. It needs three reserved i64 locals (`__coerce_ptr`, `__coerce_end`, `__coerce_acc`) — reserve them in `collect_function_locals` when the body contains a `+`-on-string (mirror the argv-scratch reservation from Task 5):

```rust
    /// Inline `Math.trunc(Number(handle))` for a decimal-integer string:
    /// acc = 0; for p in [offset, offset+len): acc = acc*10 + (byte(p) - '0').
    /// Non-digit bytes are not expected for the argv-integer path; a leading
    /// '-' is not handled (fasta's N is non-negative). Produces i64.
    fn emit_string_to_i64_parse(&mut self, function: &mut Function, arg: LirNodeId) -> EmittedValue {
        let ptr = self.locals[&crate::lower::coerce_ptr_local_name()];
        let end = self.locals[&crate::lower::coerce_end_local_name()];
        let acc = self.locals[&crate::lower::coerce_acc_local_name()];
        // handle on stack -> consume into `acc` (reused as scratch, then zeroed
        // as the accumulator). Using LocalSet (not Tee) so no handle is left
        // dangling on the operand stack.
        let produced = self.emit_node(function, arg, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalSet(acc)); // acc = handle
        // ptr = (acc >> 32) & 0x7FFF_FFFF   (byte offset)
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7FFF_FFFF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(ptr));
        // end = ptr + (acc & 0xFFFF_FFFF)   (offset + len)
        function.instruction(&Instruction::LocalGet(ptr));
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end));
        // acc = 0  (now the running accumulator)
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(acc));
        // while (ptr < end) { acc = acc*10 + (load8_u(ptr) - 48); ptr += 1; }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(ptr));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::BrIf(1)); // break out of block
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(ptr));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load8U(MemArg { offset: 0, align: 0, memory_index: 0 }));
        function.instruction(&Instruction::I64Const(48));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(acc));
        function.instruction(&Instruction::LocalGet(ptr));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(ptr));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End); // loop
        function.instruction(&Instruction::End); // block
        function.instruction(&Instruction::LocalGet(acc));
        EmittedValue { produced: true, shape: ValueShape::Scalar }
    }
```

Add `coerce_ptr_local_name()`/`coerce_end_local_name()`/`coerce_acc_local_name()` (`i64`) to `lower.rs` and reserve them in `collect_function_locals` under a `body contains +-on-string` walk.

- [ ] **Step 4: Allow `+` on a runtime string in `kali_types`**

Find the `kali_types` gate that rejects unary `+` on a non-numeric/string operand (search `resolve/` and `static_analysis/` for the unary `+` / `UnaryExpression` handling and the string-operand reject that mirrors codegen's guard). Narrow it so unary `+` over a runtime-string operand is ACCEPTED and the result is classified numeric (so `var n = +process.argv[2]` infers `n: I64` and `n + 1`, `i < n` typecheck). Keep `-`/`~` on a string rejecting. Add a matching `kali_types` unit test if that crate has unary-operator resolve tests (search `resolve/` for existing `unary` tests and mirror).

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p kali_cli --test runtime_argv unary_plus_coerces_argv_to_integer unary_plus_coerced_argv_drives_a_loop_count`
Expected: PASS (`1001\n`, `10\n`).

- [ ] **Step 6: Run the per-task gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir && cargo clippy -p kali_codegen -p kali_types -- -D warnings`
Expected: green. Confirm `-`/`~` on a runtime string still reject (the existing operators.rs guard tests, if any, stay green).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen crates/kali_types crates/kali_cli/tests/runtime_argv.rs
git commit -m "feat: unary + coerces a runtime string to i64 (fasta Spec 5 Task 6)"
```

---

### Task 7: Full fasta shell capstone + workspace gate + guardrails

Wire all three sections (`fastaRepeat(2n, ALU)`, `fastaRandom(3n, IUB)`, `fastaRandom(5n, HomoSap)`) with headers and `n = +process.argv[2]`, byte-for-byte vs node at a small N passed on the CLI. Then run the full workspace gate and the CLBG-fixture guardrails.

**Files:**
- Test: `crates/kali_cli/tests/runtime_fasta_capstone.rs` (new)

**Interfaces:**
- Consumes: Tasks 1–6. Produces: the success-criterion e2e.

- [ ] **Step 1: Write the capstone shell + failing golden test**

Create `crates/kali_cli/tests/runtime_fasta_capstone.rs`. Reuse the `run_node_source_with_args` helper (copy from `runtime_argv.rs`). Embed the full shell (LCG + `makeCumulative` + `fastaRepeat` + `fastaRandom` + the real IUB & HomoSapiens tables + the full ALU string + the three headers + `var n = +process.argv[2]`), using the supported statement forms (`x = x + y` for `+=`, explicit `i = i + 1`). Full ALU string (verbatim, as a chain of `+`-concatenated 42-char literals):

```
"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG" +
"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA" +
"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT" +
"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA" +
"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG" +
"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC" +
"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAAA"
```

HomoSapiens table: `{ a: 0.3029549426680, c: 0.1979883004921, g: 0.1975473066391, t: 0.3015094502008 }`. IUB table: as in Task 3.

```rust
#[test]
fn full_fasta_shell_matches_node_at_small_n() {
    const N: &str = "8";
    let src = "<the full shell described above, ending with the three sections>";
    // GOLDEN: run the SAME `src` under node v26.4.0 with argv[2] = N:
    //   node <file> 8
    // (In the shell, restore makeCumulative's `+=` and `n -= line.length` for
    // node; the kali source uses the `= a + b` / `= a - b` forms — numerically
    // identical.) Paste node's exact stdout. Re-derive independently (reviewer).
    const GOLDEN: &str = "<PASTE node stdout for argv[2]=8>";
    let out = run_node_source_with_args(src, &[N]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), GOLDEN);
}
```

Derive `GOLDEN` via `node <file> 8` and paste it (headers `>ONE…`/`>TWO…`/`>THREE…` plus the wrapped sequence lines). N=8 keeps output small while exercising all three sections and both wrap branches.

- [ ] **Step 2: Run to verify it fails on the placeholder**

Run: `cargo test -p kali_cli --test runtime_fasta_capstone full_fasta_shell_matches_node_at_small_n`
Expected: FAIL on placeholder golden (or reveal an integration gap between the pieces — debug + close minimally).

- [ ] **Step 3: Fill in the derived golden**

Paste node's exact stdout. Fix any minimal integration gap discovered.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p kali_cli --test runtime_fasta_capstone full_fasta_shell_matches_node_at_small_n`
Expected: PASS.

- [ ] **Step 5: Guardrail — 5 CLBG fixtures byte-identical**

Run the existing benchmark fixtures and confirm no diff:
Run: `cargo test -p kali_cli nbody fannkuch spectral mandelbrot binary_trees`
Expected: PASS (all byte-identical — Tasks 1–6 are additive and gated).

- [ ] **Step 6: Guardrail — import lists diff clean**

Run: `git diff --stat main -- crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs`
Expected: the ONLY changes are the 4 `args_get` additions from Task 5.

- [ ] **Step 7: Full workspace gate**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
Expected: all green, no format drift.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_cli/tests/runtime_fasta_capstone.rs
git commit -m "test(cli): full fasta shell byte-for-byte with n=+process.argv[2] (fasta Spec 5 Task 7)"
```

- [ ] **Step 9: Whole-branch review + integration**

Generate the whole-branch diff vs the merge-base and run `superpowers:requesting-code-review`. Address findings (the escape-invariant and both-sides-mirror lessons are the highest-risk classes — verify the argv-element string classification and the for-in-key string sink did not fail-open any adjacent position). Once review is clean, push a PR + self-merge per `kali-integration-convention`. Update the memory ledger (a new `kali-fasta-output-argv-spec5` memory + a `progress-spec5-shipped.md` roll-up), leaving Spec 6 (verbatim vendoring + N=25M SHA-256) as the only remaining series item.

---

## Notes for the implementer

- **Golden derivation is authoritative.** Every `<PASTE node stdout …>` must be produced by running the exact same source under `node` v26.4.0 and re-derived a second time by the reviewer before merge (series convention). Do not hand-compute LCG output.
- **`+=` / `-=` in shells.** The shells use `x = x + y` / `x = x - y` instead of `+=`/`-=` to avoid an incidental compound-assignment dependency; this is numerically identical to node. `table[c] += table[prev]` (compound computed index) is already shipped (Spec 4a) and MAY be used directly if a test needs it, but the shells avoid it for isolation.
- **If a recognizer never matches** (Task 5 Step 3, or the string-predicate arms), the LIR node shape for a computed member may differ from the assumed two-child `[object, index]`; add a temporary `dbg!(node)` to confirm the shape, then key the recognizer on the real shape. The static-index member fold (`resolve_static_index_member`) shows the canonical computed-index shape.
- **Systematic debugging.** For any unexpected failure, use `superpowers:systematic-debugging` (write a failing test that isolates the symptom before fixing) rather than speculative edits.
