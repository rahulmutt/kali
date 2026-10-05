# Captured Bindings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A synchronous closure that captures a binding one env record away
reads and writes that binding, or kali refuses the program. Phase 1 refuses
every capture codegen cannot lower. Phase 2 makes `i64` parameters, F64
bindings and captured boolean `const`s work.

**Architecture:**
- **Phase 1 (fail closed).**
  - The codegen chokes that today fall to the zero placeholder (identifier
    read, plain `=`) refuse a capture the env plan lists but
    `resolve_capture_access` cannot lower.
  - A read of a captured boolean `const` refuses too, because it renders `1`.
  - A `check` pass in `kali_cli` mirrors these refusals from the AST, using
    the block-scope rename walk extended with function plan keys.
- **Phase 2 (real).**
  - An AST pass rewrites a captured parameter `k` into
    `function f(k{p}) { let k = k{p}; … }`, so it takes the working local path.
  - The shared promotion predicate admits a `TaggedVal` cell whose repr is
    `I64` or `F64` and which `binding_is_proven_numeric` proves.
  - F64 cells store `f64` bits through `i64` reinterpretation.
  - A captured boolean `const` reads with `ValueShape::Boolean`.

**Tech Stack:** Rust workspace (`kali_common`, `kali_mir`, `kali_types`,
`kali_codegen`, `kali_cli`), TOML case files, the bash probe runner, and node
v26.10.0 as the oracle.

**Spec:** `docs/superpowers/specs/2026-10-05-captured-bindings-design.md`, read
together with amendments A-1 and A-2 in its §6. **The amendments override §1
to §5:**
- A-1: a captured parameter's cell is `TaggedVal` and unpromoted, not
  "promoted and uninitialized". Phase 2 needs the proof-gated widening as well
  as the rewrite.
- A-2.1: boolean means a captured `const` with a boolean initializer only,
  through the new `boolean_consts` proof.
- A-2.2: there are no unrewritten parameter forms.
- A-2.3: only the capturer is refused.
- A-2.4: the capturer is named by its plan key.
- A-2.5: refs owned by block-scoping are skipped.
- A-2.6: `check` lives in `kali_cli`, with a named residue.
- A-2.7: run node with `FORCE_COLOR` unset.

This plan adds one more narrowing, recorded as amendment A-4 in Task 9 Step 1:
an F64 cell lowers `=`, `+=`, `-=`, `*=` and `/=`. Its `%=`, bitwise compound
assignments and `++` / `--` keep their existing E5506.

## Global Constraints

- **Baseline:** `2ddf18c66`. **Oracle:** `node v26.10.0`, always run as
  `env -u FORCE_COLOR node …` (A-2.7). Every case rationale quotes node's
  output and kali's output at the baseline.
- **Diagnostic code.** Every new refusal is `E5506`
  (`e5::FEATURE_UNAVAILABLE`). There is no new code, flag or schema, and
  `specs/12-cli.md`, `specs/18-schemas.md` and `README.md` stay untouched.
- **Message text.** Every new refusal string comes from
  `kali_common::captured_binding_unavailable_message` (Task 1). No pass formats
  its own text. There are three reasons, verbatim:
  - `its value type has no closure cell`
  - `` `<name>` is two or more closures away``
  - `` `<name>` is a parameter of `<owner>` ``
- **Parameter spelling.** `<name>{p}`. `{` and `}` are not identifier
  characters, so it cannot collide with a source name. `display_names_in`
  strips it.
- **One predicate.** `closure::cell_is_promotable` stays the single promotion
  predicate. Promotion (`lower.rs`) and access (`closure_access.rs`) call it
  with identical arguments.
  - **Iteration-plan call sites** pass `allow_tagged = false`, so block-scoping
    is unchanged (spec §1.1).
  - **The deferred-registration allowlist** (`host.rs:1746`) passes
    `allow_tagged = false` too, so the deferred lane is unchanged.
- **Not widened:** `String` cells, `Object` parameters, `TaggedVal` cells
  without a numeric proof, MIR depth ≥ 2, and iteration-record cells.
- **Byte identity.** A program with no captured parameter, no captured F64
  binding and no captured boolean `const` compiles to the same wasm at HEAD as
  at the baseline. Task 12 checks it.
- **Test layout.** Rust unit tests go in sibling `*_tests.rs` files wired with
  `#[cfg(test)] #[path = "…"] mod …;`. Don't add an inline `mod tests {}`.
  CLI tests are `.toml` cases under `crates/kali_cli/tests/cases/`, and there
  are no new `tests/*.rs` targets.
- **Stop rules.**
  - **Phase gate.** After Task 6, stop and report the phase-1 triage to the
    human partner before starting Task 7.
  - **Large sweep.** In any sweep (Tasks 6 and 12), if more than about 50
    trials move or any capability loss appears, stop and report before
    re-pinning anything.
- **Commit messages.** Use `feat(captured-bindings): …`,
  `test(captured-bindings): …` and `docs(captured-bindings): …`.

## Review Focus

1. **A closure that both reads and writes a captured parameter, with the
   owner reading after.** In
   `function f(k){ const g=()=>{ k=k*2; return k; }; const a=g(); return a+k; } console.log(f(3));`
   node prints `12`. The owner's `return a+k` must read the cell the closure
   wrote, not a stale WASM parameter. Pinned in Task 8 (case
   `owner_reads_what_the_closure_wrote`).
2. **A captured parameter of a function called with a string and with a
   number.** In
   `function f(k){ const g=()=>k; return g(); } console.log(f(1)); console.log(f("a"));`
   monomorphize may or may not split `f`. Either the program prints node's
   `1` and `a`, or it refuses. It must never print `0`. Pinned in Task 8 (case
   `a_parameter_called_with_two_types_never_reads_zero`).
3. **A captured F64 in arithmetic inside the closure.** `g=()=>x*2+0.5` must
   take the f64 instruction path in the capturer, whose own namespace says
   `I64` (`operators.rs:1724-1746`). Otherwise the module is invalid (E4201).
   Pinned in Task 9 (case `captured_float_arithmetic_in_the_closure`).
4. **A nested function declaration as the capturer.** In
   `function f(k){ function g(){ return k; } return g(); }` the capturer is a
   `FunctionDeclaration` under its own name, not `__kali_fn_N`. The rewrite and
   the `check` key must both handle it. Pinned in Task 5 (check unit
   `a_function_declaration_capturer_is_keyed_by_name`) and Task 8 (case
   `a_nested_declaration_reads_the_parameter`).
5. **A program with nothing captured keeps its wasm.** The widened predicate
   and the rewrite must be no-ops for it. Pinned in Task 12 (byte-identity
   step) and Task 7 (unit `an_uncaptured_parameter_is_untouched`).

---

## File Structure

| file | status | responsibility |
|---|---|---|
| `crates/kali_common/src/messages.rs` (+ `messages_tests.rs`) | modify | `CaptureRefusal`, `captured_binding_unavailable_message` |
| `crates/kali_common/src/display_name.rs` (+ `display_name_tests.rs`) | modify | strip `{p}`; `captured_param_spelling` |
| `crates/kali_common/src/repr.rs` | modify | `boolean_consts` set, setter, `binding_is_boolean_const` |
| `crates/kali_common/src/lib.rs` | modify | re-exports |
| `crates/kali_mir/src/env_plan.rs` (+ `env_plan_tests.rs`) | modify | `is_parameter`, `is_tagged` on `EnvCell` / `CapturedRef` |
| `crates/kali_types/src/repr_infer.rs` (+ `repr_infer_tests.rs`) | modify | record `boolean_consts` |
| `crates/kali_codegen/src/closure.rs` (+ `closure_tests.rs`, create) | modify | widened predicate; f64 cell load/store helpers |
| `crates/kali_codegen/src/emit/closure_access.rs` | modify | refusal helper; tagged and F64 access; boolean shape |
| `crates/kali_codegen/src/emit/control_flow.rs` | modify | read choke deny |
| `crates/kali_codegen/src/emit/literal.rs` | modify | `=` choke deny |
| `crates/kali_codegen/src/emit/operators.rs` | modify | owner-keyed float check for captured identifiers |
| `crates/kali_codegen/src/emitter.rs`, `lower.rs`, `iteration.rs`, `env_safety.rs`, `intrinsics/host.rs` | modify | pass `allow_tagged` to the predicate |
| `crates/kali_cli/src/build/block_scope_rename/walk.rs`, `table.rs`, `apply.rs` | modify | `Hooks::enter` takes a function plan key |
| `crates/kali_cli/src/build/capture_refusals/mod.rs` (+ `_tests.rs`) | create | the `check` mirror |
| `crates/kali_cli/src/build/capture_param_rewrite/mod.rs` (+ `_tests.rs`) | create | parameter-to-local rewrite |
| `crates/kali_cli/src/build/mod.rs`, `compile.rs` | modify | wire both passes |
| `crates/kali_cli/tests/cases/closure/captured_bindings.toml` | create | the cases |
| `crates/kali_cli/tests/cases/README.md` | modify | add `closure/` to the family list |
| `tools/array-return-probes/probes/cb_*.js`, `baseline-cb.tsv` | create | probes |
| docs (Tasks 6, 13) | modify | followups file, `specs/15`, `specs/19`, register, block-scoping followups |

---

### Task 0: Probes and the baseline (before any code change)

**Files:**
- Create: `tools/array-return-probes/probes/cb_*.js` (32 files)
- Create: `tools/array-return-probes/baseline-cb.tsv`

**Interfaces:**
- Produces: `baseline-cb.tsv`, with columns `name verdict node_out kali_out check_exit`. Tasks 6 and 12 diff against it.

- [ ] **Step 1: Confirm the branch**

```bash
git branch --show-current          # expect captured-bindings
git log --oneline -4               # expect A-2, A-1, the spec, then 2ddf18c66
```

- [ ] **Step 2: Build the baseline binary in a separate worktree**

```bash
git worktree add "$TMPDIR/kali-cb-baseline" 2ddf18c66
(cd "$TMPDIR/kali-cb-baseline" && cargo build -q -p kali_cli)
export KALI_BASE="$TMPDIR/kali-cb-baseline/target/debug/kali"
"$KALI_BASE" --version
```

- [ ] **Step 3: Write the probes** (one program per file, exactly as below)

```bash
P=tools/array-return-probes/probes
w(){ printf '%s\n' "$2" > "$P/cb_$1.js"; }
w p1 'function f(k){ const g=()=>k; return g(); } console.log(f(5));'
w p2 'function f(k){ let n=k; const g=()=>n; return g(); } console.log(f(5));'
w v2 'function f(k){ setTimeout(()=>console.log(k),0); } f(5);'
w v4 'function f(k){ const g=()=>k+1; console.log(g(), k); } f(5);'
w ok_v6 'function f(k){ let n=k+0; const g=()=>n; return g(); } console.log(f(5));'
w v7 'function f(s){ const g=()=>s; return g(); } console.log(f("hi"));'
w v8 'function f(k){ function g(){ return k; } return g(); } console.log(f(5));'
w v9 'const f=(k)=>{ const g=()=>k; return g(); }; console.log(f(5));'
w a1 'function f(){ let s="hi"; const g=()=>s; return g(); } console.log(f());'
w a2 'function f(){ let x=1.5; const g=()=>x; return g(); } console.log(f());'
w a3 'function f(k){ const g=()=>k; k=k+1; return g(); } console.log(f(5));'
w a4 'function f(k){ const g=()=>{ k=k+1; }; g(); return k; } console.log(f(5));'
w a6 'function f(x){ const g=()=>x; return g(); } console.log(f(1.5));'
w a7 'function f(b){ const g=()=>b; return g(); } console.log(f(true));'
w w1 'function f(){ let s="a"; const g=()=>{ s="b"; }; g(); return s; } console.log(f());'
w w2 'function f(){ let x=1.5; const g=()=>{ x=2.5; }; g(); return x; } console.log(f());'
w w3 'function f(){ let x=1.5; const g=()=>{ x+=1; }; g(); return x; } console.log(f());'
w w4 'function f(k){ const g=()=>{ k++; }; g(); return k; } console.log(f(5));'
w w5 'function f(){ let b=true; const g=()=>b; return g(); } console.log(f());'
w d02 'function m() { let a = 5; setTimeout(() => { let z = 10; const h = () => z + a; console.log(h()); }, 0); } m();'
w w6 'function m() { let a = 5; const o = () => { let z = 10; const h = () => z + a; return h(); }; return o(); } console.log(m());'
w e1 'function f(){ let k=5; let n=k; const g=()=>n; return g(); } console.log(f());'
w e4 'function f(k){ let n=0; n=k; const g=()=>n; return g(); } console.log(f(5));'
w e5 'function f(k){ let n=k; const g=()=>n; console.log(n); return g(); } console.log(f(5));'
w kb1 'function f(){ const b=true; console.log(b); } f();'
w kb2 'function f(){ const b=true; const g=()=>{ console.log(b); }; g(); } f();'
w kb3 'function f(){ const b=1<2; const g=()=>{ console.log(b); }; g(); } f();'
w lb1 'function f(){ let b=true; const g=()=>{ console.log(b); }; g(); } f();'
w rw1 'function f(k){ const g=()=>{ k=k*2; return k; }; const a=g(); return a+k; } console.log(f(3));'
w two 'function f(k){ const g=()=>k; return g(); } console.log(f(1)); console.log(f("a"));'
w fa 'function f(){ let x=1.5; const g=()=>x*2+0.5; return g(); } console.log(f());'
w fw 'function f(){ let x=1.5; const g=()=>{ x=x+1.25; }; g(); return x; } console.log(f());'
ls "$P"/cb_*.js | wc -l   # expect 32
```

- [ ] **Step 4: Record the baseline**

```bash
env -u FORCE_COLOR KALI="$KALI_BASE" tools/array-return-probes/run.sh "$TMPDIR/cb-all.tsv"
grep '^cb_' "$TMPDIR/cb-all.tsv" > tools/array-return-probes/baseline-cb.tsv
cut -f1,2,5 tools/array-return-probes/baseline-cb.tsv
```

Expected verdicts, from spec §2 and A-1 / A-2:
- **SILENT:** `p1 p2 v4 v7 v8 v9 a1 a2 a3 a4 a6 a7 w1 w5 d02 w6 e5 kb2 lb1`
- **REFUSES:** `v2 w3 w4`
- **OTHER (E4201):** `w2`
- **CORRECT:** `ok_v6 e1 e4 kb1`
- **Not measured before the plan:** `kb3 rw1 two fa fw`. Record them as
  found.

`check_exit` is `0` for every row. **Do not edit a probe to fit an expectation.**
If a row differs, write the difference into the commit message and stop to
report it: spec §5.1 makes a differing row an amendment before phase 1.

- [ ] **Step 5: Commit**

```bash
git add tools/array-return-probes/probes/cb_*.js tools/array-return-probes/baseline-cb.tsv
git commit -m "test(captured-bindings): add cb_* probes and their baseline at 2ddf18c66"
```

---

### Task 1: Message and spelling helpers in `kali_common`

**Files:**
- Modify: `crates/kali_common/src/messages.rs`, `crates/kali_common/src/messages_tests.rs`
- Modify: `crates/kali_common/src/display_name.rs`, `crates/kali_common/src/display_name_tests.rs`
- Modify: `crates/kali_common/src/lib.rs` (re-export the new items next to the existing `display_names_in` / message re-exports)

**Interfaces:**
- Produces:
  - `pub enum CaptureRefusal<'a> { ValueType, Depth, Parameter { owner: &'a str } }`
  - `pub fn captured_binding_unavailable_message(capturer: &str, name: &str, reason: CaptureRefusal<'_>) -> String`
  - `pub fn captured_param_spelling(name: &str) -> String` (returns `"<name>{p}"`)

- [ ] **Step 1: Write the failing tests**

In `messages_tests.rs`:

```rust
#[test]
fn captured_binding_message_names_capturer_binding_and_reason() {
    assert_eq!(
        captured_binding_unavailable_message("g", "s", CaptureRefusal::ValueType),
        "a closure `g` that captures `s` is unavailable in the current phase: its value type has no closure cell"
    );
    assert_eq!(
        captured_binding_unavailable_message("h", "a", CaptureRefusal::Depth),
        "a closure `h` that captures `a` is unavailable in the current phase: `a` is two or more closures away"
    );
    assert_eq!(
        captured_binding_unavailable_message("__kali_fn_0", "k", CaptureRefusal::Parameter { owner: "f" }),
        "a closure `__kali_fn_0` that captures `k` is unavailable in the current phase: `k` is a parameter of `f`"
    );
}

#[test]
fn captured_binding_message_shows_written_names() {
    assert_eq!(
        captured_binding_unavailable_message("g{b2}", "k{p}", CaptureRefusal::Parameter { owner: "f{b1}" }),
        "a closure `g` that captures `k` is unavailable in the current phase: `k` is a parameter of `f`"
    );
}
```

In `display_name_tests.rs`:

```rust
#[test]
fn the_parameter_suffix_is_stripped() {
    assert_eq!(display_names_in("k{p} + n{b3}"), "k + n");
    assert_eq!(display_names_in("{p}"), "{p}"); // no identifier before it
    assert_eq!(display_names_in("k{pp}"), "k{pp}");
}

#[test]
fn captured_param_spelling_appends_the_suffix() {
    assert_eq!(captured_param_spelling("k"), "k{p}");
    assert_eq!(display_names_in(&captured_param_spelling("k")), "k");
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_common captured_ -- --nocapture` and `cargo test -p kali_common the_parameter_suffix`
Expected: compile errors, because `captured_binding_unavailable_message`, `CaptureRefusal` and `captured_param_spelling` don't exist yet.

- [ ] **Step 3: Implement**

In `messages.rs`, after `iteration_unrolled_for_of_message`:

```rust
/// Why a closure's capture is refused (captured-bindings spec §3.1, A-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureRefusal<'a> {
    /// The binding's repr has no env-cell lane (string, an unproven
    /// `TaggedVal`, a boolean `const`, …).
    ValueType,
    /// The binding is two or more env records away (MIR depth >= 2).
    Depth,
    /// The binding is a parameter of `owner` (phase 1 only; phase 2 rewrites
    /// every captured parameter into a local).
    Parameter { owner: &'a str },
}

/// Captured-bindings spec §3.1. `capturer` and `owner` are plan keys; the
/// written spelling is shown (`display_names_in`).
pub fn captured_binding_unavailable_message(
    capturer: &str,
    name: &str,
    reason: CaptureRefusal<'_>,
) -> String {
    let why = match reason {
        CaptureRefusal::ValueType => "its value type has no closure cell".to_string(),
        CaptureRefusal::Depth => format!("`{name}` is two or more closures away"),
        CaptureRefusal::Parameter { owner } => format!("`{name}` is a parameter of `{owner}`"),
    };
    let text = format!(
        "a closure `{capturer}` that captures `{name}` is unavailable in the current phase: {why}"
    );
    crate::display_names_in(&text).into_owned()
}
```

In `display_name.rs`, update the module doc's first line to mention both suffixes. Then:
- change the fast-path guard to `if !text.contains("{b") && !text.contains("{p}") {`;
- add a branch to `suffix_len_at` before the `{b` check;
- add the spelling function.

```rust
fn suffix_len_at(bytes: &[u8], i: usize) -> Option<usize> {
    if i == 0 || bytes.get(i) != Some(&b'{') {
        return None;
    }
    let prev = bytes[i - 1];
    if !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$') {
        return None;
    }
    // Captured-bindings spec §3.2: a rewritten parameter is `<name>{p}`.
    if bytes.get(i + 1) == Some(&b'p') && bytes.get(i + 2) == Some(&b'}') {
        return Some(3);
    }
    if bytes.get(i + 1) != Some(&b'b') {
        return None;
    }
    let digits = bytes[i + 2..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    if digits == 0 || bytes.get(i + 2 + digits) != Some(&b'}') {
        return None;
    }
    Some(3 + digits)
}

/// The spelling the captured-parameter rewrite gives parameter `name`
/// (captured-bindings spec §3.2).
pub fn captured_param_spelling(name: &str) -> String {
    format!("{name}{{p}}")
}
```

Re-export `CaptureRefusal`, `captured_binding_unavailable_message` and
`captured_param_spelling` from `lib.rs`, the same way the neighbouring
items are re-exported.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_common`
Expected: PASS, including every existing `display_name_tests` case.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common
git commit -m "feat(captured-bindings): refusal message family and the {p} parameter spelling"
```

---

### Task 2: `is_parameter` and `is_tagged` on env plans

**Files:**
- Modify: `crates/kali_mir/src/env_plan.rs`, `crates/kali_mir/src/env_plan_tests.rs`
- Modify: `crates/kali_codegen/src/iteration_tests.rs` (one `EnvCell` literal)

**Interfaces:**
- Produces: `EnvCell { name, offset, is_scalar, is_parameter: bool, is_tagged: bool }` and `CapturedRef { …, is_parameter: bool, is_tagged: bool }`.
  - `is_parameter`: the owner's MIR binding kind is `MirBindingKind::Parameter`.
  - `is_tagged`: its layout is `LayoutDescriptor::TaggedVal`.

- [ ] **Step 1: Write the failing tests** (append to `env_plan_tests.rs`)

```rust
/// Captured-bindings A-1: a parameter's layout is TaggedVal, so its cell is
/// a heap cell; the plan says so, and that it is a parameter.
#[test]
fn a_captured_parameter_cell_is_tagged_and_a_parameter() {
    let analysis = crate::test_support::analyze("function f(k){ const g=()=>k; return g(); }");
    let plans = derive_env_plans(&analysis);
    assert_eq!(
        plans["f"].cells,
        vec![EnvCell {
            name: "k".into(),
            offset: 0,
            is_scalar: false,
            is_parameter: true,
            is_tagged: true,
        }]
    );
    let reference = plans["__kali_fn_0"].captured_for("k").expect("g captures k");
    assert_eq!(reference.depth, 1);
    assert!(reference.is_parameter);
    assert!(reference.is_tagged);
}

#[test]
fn a_local_copied_from_a_parameter_is_tagged_but_not_a_parameter() {
    let analysis =
        crate::test_support::analyze("function f(k){ let n=k; const g=()=>n; return g(); }");
    let cell = derive_env_plans(&analysis)["f"].cell_for("n").cloned().expect("n cell");
    assert!(cell.is_tagged);
    assert!(!cell.is_parameter);
    assert!(!cell.is_scalar);
}

#[test]
fn a_local_from_arithmetic_is_scalar_and_not_tagged() {
    let analysis =
        crate::test_support::analyze("function f(k){ let n=k+0; const g=()=>n; return g(); }");
    let cell = derive_env_plans(&analysis)["f"].cell_for("n").cloned().expect("n cell");
    assert!(cell.is_scalar);
    assert!(!cell.is_tagged);
    assert!(!cell.is_parameter);
}
```

If `plans["__kali_fn_0"]` is not `g`'s key, print `plans.keys()` and use the
key the analysis gives the arrow. Don't change the source.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_mir env_plan`
Expected: compile errors (the fields don't exist yet).

- [ ] **Step 3: Implement**

In `env_plan.rs`:
- Add both fields to `EnvCell` and `CapturedRef`, each with a one-line doc
  pointing to captured-bindings A-1.
- Replace the `(u32, bool)` offset tuple with a small private struct:

```rust
#[derive(Debug, Clone, Copy)]
struct CellFacts {
    offset: u32,
    is_scalar: bool,
    is_parameter: bool,
    is_tagged: bool,
}
```

- **Pass 1:** build each `EnvCell` with
  `is_parameter: binding.kind == MirBindingKind::Parameter` and
  `is_tagged: matches!(binding.layout, LayoutDescriptor::TaggedVal)`. Import
  `MirBindingKind` from `crate`.
- **The module-root iteration branch** (`EnvCell { name, offset: 0, is_scalar, … }`):
  set the two fields from `b` the same way.
- **`renumber`:** keep the fields, since it only rewrites `offset`.
- **The "rebuild the offset tables" step:** map each `EnvCell` to `CellFacts`.
- **Pass 2:** read `CellFacts` and copy `is_parameter` and `is_tagged` into
  every `CapturedRef`.

Add `is_parameter: false, is_tagged: false` to every existing `EnvCell { … }` /
`CapturedRef { … }` literal in `env_plan_tests.rs` and
`kali_codegen/src/iteration_tests.rs`. Exception: a literal whose source binding
is a parameter or a `TaggedVal` gets the true value. The test failure says
which.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_mir && cargo test -p kali_codegen --lib`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_mir crates/kali_codegen/src/iteration_tests.rs
git commit -m "feat(captured-bindings): env cells and captured refs record parameter-ness and TaggedVal layout"
```

---

### Task 3: The `boolean_consts` proof

**Files:**
- Modify: `crates/kali_common/src/repr.rs`
- Modify: `crates/kali_types/src/repr_infer.rs:3200-3210` (the declarator walk that calls `note_fn_alias`) and `:7398-7427` (`emit_table`)
- Test: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Produces:
  - `ReprTable::set_boolean_consts(&mut self, HashSet<(String, String)>)`
  - `ReprTable::binding_is_boolean_const(&self, scope: &str, binding: &str) -> bool`

  Keys are `(binding_scope(func, name), name)`, the same keying as
  `numeric_bindings`.

- [ ] **Step 1: Write the failing tests** (append to `repr_infer_tests.rs`)

```rust
#[test]
fn a_const_with_a_boolean_initializer_is_a_boolean_const() {
    let t = reprs(
        "function f(){ const a=true; const b=1<2; const c=!a; const d=(a===b); return 0; }",
    );
    for name in ["a", "b", "c", "d"] {
        assert!(t.binding_is_boolean_const("f", name), "{name}");
    }
}

#[test]
fn let_and_non_boolean_consts_are_not_boolean_consts() {
    let t = reprs("function f(){ let a=true; const n=1; const s=\"x\"; const c=a&&a; return 0; }");
    for name in ["a", "n", "s", "c"] {
        assert!(!t.binding_is_boolean_const("f", name), "{name}");
    }
}
```

`a&&a` is deliberately not admitted: `&&` selects an operand, and the spec
admits only literals, comparisons and `!`.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_types boolean_const`
Expected: compile error (`binding_is_boolean_const` doesn't exist).

- [ ] **Step 3: Implement**

In `repr.rs`, next to `numeric_bindings`:

```rust
    /// `(scope, binding)` pairs of a `const` whose initializer is a boolean
    /// literal, a comparison or `!` (captured-bindings A-2.1). Positive
    /// evidence only: `scalar(..)` is `I64` for a boolean, so the repr cannot
    /// say this.
    boolean_consts: HashSet<(String, String)>,
```

```rust
    pub fn set_boolean_consts(&mut self, consts: HashSet<(String, String)>) {
        self.boolean_consts = consts;
    }

    pub fn binding_is_boolean_const(&self, scope: &str, binding: &str) -> bool {
        self.boolean_consts
            .contains(&(scope.to_string(), binding.to_string()))
    }
```

Initialize the field wherever `ReprTable`'s constructor or `Default` lists
`numeric_bindings`.

In `repr_infer.rs`:
- Add `boolean_consts: HashSet<(String, String)>` to the inferrer struct.
- At the declarator loop that calls `self.note_fn_alias(func, &decl.kind, &d.id, d.init.as_ref())`, add:

```rust
                    if decl.kind == "const"
                        && d.init.as_ref().is_some_and(is_boolean_valued_init)
                    {
                        let scope = self.binding_scope(func, &d.id);
                        self.boolean_consts.insert((scope, d.id.clone()));
                    }
```

- Add the free function near `strip_parenthesized`:

```rust
/// Captured-bindings A-2.1: a boolean literal, a comparison, or `!`.
fn is_boolean_valued_init(expr: &Expression) -> bool {
    match strip_parenthesized(expr) {
        Expression::Literal(LiteralValue::Boolean(_)) => true,
        Expression::BinaryExpression(binary) => matches!(
            binary.operator.as_str(),
            "==" | "===" | "!=" | "!==" | "<" | ">" | "<=" | ">="
        ),
        Expression::UnaryExpression(unary) => unary.operator == "!",
        _ => false,
    }
}
```

- In `emit_table`, next to `table.set_numeric_bindings(numeric_bindings);`:

```rust
        table.set_boolean_consts(std::mem::take(&mut self.boolean_consts));
```

If `emit_table` takes `&self`, clone the set instead of taking it.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_types`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/repr.rs crates/kali_types
git commit -m "feat(captured-bindings): boolean_consts proof for consts with a boolean initializer"
```

---

### Task 4: Phase 1 in codegen — refuse what the capture lane cannot lower

**Files:**
- Modify: `crates/kali_codegen/src/emit/closure_access.rs`
- Modify: `crates/kali_codegen/src/emit/control_flow.rs:2623-2640` (`CapturedCellOrPlaceholder`)
- Modify: `crates/kali_codegen/src/emit/literal.rs:985-996` (the assignment choke)
- Create: `crates/kali_cli/tests/cases/closure/captured_bindings.toml`
- Modify: `crates/kali_cli/tests/cases/README.md` (family list: add `closure/`)

**Interfaces:**
- Consumes:
  - `CaptureRefusal` and `captured_binding_unavailable_message` (Task 1)
  - `CapturedRef::is_parameter` (Task 2)
  - `ReprTable::binding_is_boolean_const` (Task 3)
- Produces: `FunctionEmitter::unlowered_capture_refusal(&self, name: &str) -> Option<String>` and `FunctionEmitter::captured_boolean_const_owner(&self, name: &str) -> Option<String>`. Task 10 reuses both.

- [ ] **Step 1: Write the failing cases**

Create `closure/captured_bindings.toml`. Its header names the spec and the
baseline. The `[source]` table holds every `cb_*` probe from Task 0
**verbatim**, keyed `"<id>.js"` (`"p1.js"`, …), in the same format as
`scope/per_iteration.toml`. Then add these cases (phase-1 expectations):

```toml
[[case]]
name = "a_closure_over_a_parameter_is_refused"
rationale = """Spec §3.1, A-1: a parameter's cell is TaggedVal and unpromoted, so the closure read fell to the zero placeholder. node prints `5`; baseline printed `0` at exit 0."""
args = ["run", "p1.js"]
exit = "failure"
stderr_contains = ["E5506", "that captures `k`", "`k` is a parameter of `f`"]

[[case]]
name = "a_closure_over_a_local_copied_from_a_parameter_is_refused"
rationale = """A-1: `let n=k` copies the TaggedVal layout. node prints `5`; baseline printed `0` at exit 0."""
args = ["run", "p2.js"]
exit = "failure"
stderr_contains = ["E5506", "that captures `n`", "its value type has no closure cell"]

[[case]]
name = "a_local_from_arithmetic_still_works"
rationale = """Control: `let n=k+0` is a Scalar cell and was lowered at baseline. node and baseline print `5`."""
args = ["run", "ok_v6.js"]
exit = "success"
stdout = "5\n"
```

Add one case of the same form for each of these, with these expectations:
- **`v4`, `v8`, `v9`, `a3`, `a6`, `a7`, `v7`:** refused, "is a parameter of".
- **`a4`:** refused, "is a parameter of". The closure's `k=k+1` reaches the
  read choke or the `=` choke; both refuse with the same reason.
- **`a1`, `a2`, `w1`, `w2`:** refused, "its value type has no closure cell".
- **`d02`, `w6`:** refused, "is two or more closures away".
- **`kb2`, `kb3`:** refused, "its value type has no closure cell". This is the
  boolean `const` (A-2.1).
- **`w3`:** still refused with `compound assignment lowering is unavailable`.
- **`w4`:** still refused with `update expression lowering is unavailable`.
- **`v2`:** still refused with `captured param binding without closure lowering`.
- **`e1`, `e4`, `kb1`:** `exit = "success"` with node's stdout.
- **`w5`, `lb1`:** stay as at baseline, exit 0 with stdout `1`. Their rationale
  says: "A captured `let` boolean renders `1` as the uncaptured one does
  (A-2.1); not this project's lane".
- **`e5`:** refused, value type (the closure read of `n`).
- **`rw1`, `two`:** refused, "is a parameter of".
- **`fa`, `fw`:** refused, value type.

Each rationale quotes node's output and the baseline row from
`baseline-cb.tsv`.

Add `closure/` to the README's "Today's families" sentence.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- closure/`
Expected: the refusal cases FAIL (kali exits 0 with `0`); the controls pass.

- [ ] **Step 3: Implement the helper** (in `closure_access.rs`, after `resolve_capture_access_inner`)

```rust
    /// Captured-bindings spec §3.1 (A-2.3, A-2.5): the refusal for a name
    /// this function captures but `resolve_capture_access` did not lower.
    /// `None` when `name` is this function's own local, is not one of its
    /// captures, or is a capture block-scoping's iteration refusals own.
    pub(crate) fn unlowered_capture_refusal(&self, name: &str) -> Option<String> {
        if self.locals.contains_key(name) {
            return None;
        }
        let reference = self.env_plan.captured_for(name)?;
        let owned_by_iteration = self
            .env_plans
            .get(&reference.owner)
            .is_some_and(|plan| plan.iteration_of.is_some());
        if reference.through_iteration || owned_by_iteration {
            return None;
        }
        let reason = if reference.depth >= 2 {
            kali_common::CaptureRefusal::Depth
        } else if reference.is_parameter {
            kali_common::CaptureRefusal::Parameter {
                owner: &reference.owner,
            }
        } else {
            kali_common::CaptureRefusal::ValueType
        };
        Some(kali_common::captured_binding_unavailable_message(
            &self.function_name,
            name,
            reason,
        ))
    }

    /// Captured-bindings A-2.1: the owner namespace of a lowered capture of a
    /// boolean `const`, or `None`. Phase 1 refuses the read; phase 2 (Task 10)
    /// gives it `ValueShape::Boolean`.
    pub(crate) fn captured_boolean_const_owner(&self, name: &str) -> Option<String> {
        if self.locals.contains_key(name) || self.env_plan.cell_for(name).is_some() {
            return None;
        }
        self.resolve_capture_access(name)?;
        let owner = self.scalar_capture_owner(name)?;
        self.repr_table
            .binding_is_boolean_const(&owner, name)
            .then_some(owner)
    }
```

`env_plans`, `env_plan`, `locals`, `repr_table` and `function_name` already
exist on `FunctionEmitter`. Check their exact field names in `emitter.rs` if
the compiler disagrees.

At the top of `try_emit_captured_read`, add the phase-1 boolean refusal:

```rust
        if self.captured_boolean_const_owner(name).is_some() {
            return Some(self.deny_e5506(
                function,
                &kali_common::captured_binding_unavailable_message(
                    &self.function_name,
                    name,
                    kali_common::CaptureRefusal::ValueType,
                ),
            ));
        }
```

- [ ] **Step 4: Wire the read choke** (`control_flow.rs`, `CapturedCellOrPlaceholder`)

```rust
                            if let Some(value) = self.try_emit_captured_read(function, text) {
                                return value;
                            }
                            // Captured-bindings spec §3.1: a capture the lane
                            // could not lower never reads the placeholder.
                            if let Some(message) = self.unlowered_capture_refusal(text) {
                                return self.deny_e5506(function, &message);
                            }
                            self.push_placeholder_fallback_diagnostic("identifier", text);
```

- [ ] **Step 5: Wire the `=` choke** (`literal.rs`, directly after the `try_emit_captured_assign` block, before `let Some(index) = self.locals.get(&name)`)

```rust
        // Captured-bindings spec §3.1: a plain `=` to a capture the lane could
        // not lower used to drop the store (or emit invalid wasm). Compound
        // and update keep their existing messages below.
        if op == "=" {
            if let Some(message) = self.unlowered_capture_refusal(&name) {
                self.diagnostics
                    .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
                function.instruction(&Instruction::I64Const(0));
                return true;
            }
        }
```

Read the surrounding function first. If this code sits inside a block that
only runs for some names, put the check where every non-local `=` target
passes through.

- [ ] **Step 6: Run the cases**

Run: `cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- closure/`
Expected: PASS. If `d02` reports a different message because the deferred lane
fires first, keep the `stderr_contains` on the depth reason anyway. Both
diagnostics are printed; confirm with `target/debug/kali run`.

- [ ] **Step 7: Run the codegen and MIR unit tests**

Run: `cargo test -p kali_codegen --lib && cargo test -p kali_mir`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases
git commit -m "feat(captured-bindings): phase 1 — a capture the lane cannot lower is refused, never read as zero"
```

---

### Task 5: Phase 1 in `check` — the `capture_refusals` pass

**Files:**
- Modify: `crates/kali_cli/src/build/block_scope_rename/walk.rs`, `table.rs`, `apply.rs` (the `Hooks::enter` signature)
- Create: `crates/kali_cli/src/build/capture_refusals/mod.rs`, `crates/kali_cli/src/build/capture_refusals/capture_refusals_tests.rs`
- Modify: `crates/kali_cli/src/build/mod.rs`, `crates/kali_cli/src/build/compile.rs` (just before `Ok(AnalyzedSource { … })`)
- Modify: `crates/kali_cli/tests/cases/closure/captured_bindings.toml`

**Interfaces:**
- Consumes:
  - `CaptureRefusal` and `captured_binding_unavailable_message` (Task 1)
  - `ReprTable::binding_is_boolean_const` (Task 3)
  - `ReprTable::scalar`
- Produces:
  - `Hooks::enter(&mut self, kind: ScopeKind, label: Option<&str>)`, where `label` is the function's plan key: the declaration name, `FunctionExpression.id`, `ArrowFunctionExpression.id`, `"<Class>__<method>"`, or `None` when unknown.
  - `pub(crate) fn capture_refusals(statements: &mut [Statement], repr_table: &ReprTable, phase: Phase) -> Vec<Diagnostic>`, with `pub(crate) enum Phase { One, Two }`. Task 9 and Task 10 flip to `Two`.

- [ ] **Step 1: Thread the label through the walk**

- **The trait:** change `Hooks::enter` to take `label: Option<&str>`.
- **`walk_function_parts`:** gains a `label: Option<&str>` parameter and passes
  it to `enter(ScopeKind::Function, label)`. The call sites pass:
  - `FunctionDeclaration`: `Some(&f.name)`. Clone the name first, since `f` is
    borrowed mutably: `let label = f.name.clone();`.
  - `FunctionExpression`: `let label = f.id.clone();`, then pass
    `label.as_deref()`.
  - `ArrowFunctionExpression`: the same, with `let label = a.id.clone();`.
  - Class methods: `walk_class_body` gains `class: Option<&str>`, and each
    method computes `let label = class.map(|c| format!("{c}__{}", method.name));`
    and passes `label.as_deref()`.
    `ClassDeclaration` passes `Some(&c.name)`, `ClassExpression` passes
    `c.id.as_deref()`, and `ExportDefault` passes the same.
- **Every other `enter`** passes `None`.
- **`table::Collector` and `apply::Renamer`** take and ignore the label. The
  rename pass behaves exactly as before.

Run: `cargo test -p kali_cli --lib block_scope_rename`
Expected: PASS (no behaviour change).

- [ ] **Step 2: Write the failing unit tests** (`capture_refusals_tests.rs`)

Each test parses with `block_scope_rename::test_support::parse`, then runs
`kali_cli::build::name_anon_functions::name_anonymous_functions` (so arrows have
ids, as in `analyze_source_file`), then `kali_types::infer_reprs`, then
`capture_refusals`:

```rust
use super::*;
use crate::build::block_scope_rename::test_support::parse;

fn refusals(source: &str, phase: Phase) -> Vec<String> {
    let mut statements = parse(source);
    crate::build::name_anon_functions::name_anonymous_functions(&mut statements);
    let table = kali_types::infer_reprs(&statements);
    capture_refusals(&mut statements, &table, phase)
        .into_iter()
        .map(|d| d.message)
        .collect()
}

#[test]
fn a_captured_parameter_is_refused_in_phase_one() {
    let found = refusals("function f(k){ const g=()=>k; return g(); } f(5);", Phase::One);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("`k` is a parameter of `f`"), "{found:?}");
}

#[test]
fn a_function_declaration_capturer_is_keyed_by_name() {
    let found = refusals("function f(k){ function g(){ return k; } return g(); } f(5);", Phase::One);
    assert!(found[0].contains("a closure `g` that captures `k`"), "{found:?}");
}

#[test]
fn a_string_local_capture_is_refused() {
    let found = refusals("function f(){ let s=\"hi\"; const g=()=>s; return g(); } f();", Phase::One);
    assert!(found[0].contains("its value type has no closure cell"), "{found:?}");
}

#[test]
fn a_depth_two_capture_is_refused() {
    let found = refusals(
        "function m(){ let a=5; const o=()=>{ let z=10; const h=()=>z+a; return h(); }; return o(); } m();",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("`a` is two or more closures away"), "{found:?}");
}

#[test]
fn an_intermediate_function_without_captured_bindings_is_transparent() {
    // `mid` owns nothing captured, so `a` is one env record away (MIR §3.4).
    let found = refusals(
        "function m(){ let a=5; function mid(){ const h=()=>a; return h(); } return mid(); } m();",
        Phase::One,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn an_i64_local_capture_is_admitted() {
    assert!(refusals("function f(){ let c=0; const inc=()=>{ c+=1; }; inc(); return c; } f();", Phase::One).is_empty());
}

#[test]
fn a_boolean_const_capture_is_refused_in_phase_one() {
    let found = refusals("function f(){ const b=true; const g=()=>b; return g(); } f();", Phase::One);
    assert!(found[0].contains("that captures `b`"), "{found:?}");
}

#[test]
fn module_bindings_are_not_captures() {
    assert!(refusals("let s=\"x\"; function f(){ return s; } f();", Phase::One).is_empty());
}
```

- [ ] **Step 3: Run them and watch them fail**

Run: `cargo test -p kali_cli --lib capture_refusals`
Expected: compile error (the module doesn't exist yet).

- [ ] **Step 4: Implement `capture_refusals/mod.rs`**

```rust
//! The `check` mirror of captured-bindings spec §3.1 (A-2.6). It walks the
//! analysed AST with the block-scope rename walk, finds every reference that
//! crosses into an enclosing function, and refuses the shapes codegen's
//! capture lane cannot lower. It computes MIR's structural depth: the number
//! of enclosing functions, up to and including the owner, that own at least
//! one captured binding (`kali_mir::env_plan::env_owning_hops`).
//!
//! Residue (A-2.6): a TaggedVal local codegen does not promote is admitted
//! here, because MIR layouts are not visible before MIR.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::Statement;
use kali_common::{CaptureRefusal, Diagnostic, Repr, ReprTable};

use super::block_scope_rename::walk::{self, BindKind, Hooks, ScopeKind};

#[cfg(test)]
#[path = "capture_refusals_tests.rs"]
mod capture_refusals_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Before the parameter rewrite and the widening: parameters, F64 and
    /// boolean consts are refused.
    One,
    /// After Tasks 7-10: F64 and boolean consts are admitted; parameters no
    /// longer occur.
    Two,
}

#[derive(Debug)]
struct Frame {
    /// `None` for the module root.
    label: Option<String>,
    parent: Option<usize>,
    is_module: bool,
}

#[derive(Debug)]
struct ScopeEntry {
    frame: usize,
    parent: Option<usize>,
    bindings: BTreeMap<String, BindKind>,
}

#[derive(Debug)]
struct Use {
    scope: usize,
    name: String,
}

#[derive(Default)]
struct Recorder {
    frames: Vec<Frame>,
    scopes: Vec<ScopeEntry>,
    stack: Vec<usize>,
    uses: Vec<Use>,
}

impl Hooks for Recorder {
    fn enter(&mut self, kind: ScopeKind, label: Option<&str>) {
        let parent = self.stack.last().copied();
        let frame = match kind {
            ScopeKind::Module | ScopeKind::Function => {
                let id = self.frames.len();
                self.frames.push(Frame {
                    label: label.map(str::to_string),
                    parent: parent.map(|p| self.scopes[p].frame),
                    is_module: kind == ScopeKind::Module,
                });
                id
            }
            ScopeKind::Block => parent.map_or(0, |p| self.scopes[p].frame),
        };
        self.scopes.push(ScopeEntry {
            frame,
            parent,
            bindings: BTreeMap::new(),
        });
        self.stack.push(self.scopes.len() - 1);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    fn bind(&mut self, name: &mut String, kind: BindKind) {
        let current = *self.stack.last().expect("bind inside a scope");
        // `var` lands in the frame's own scope (the first scope of that frame).
        let target = if kind == BindKind::Var {
            self.frame_scope(self.scopes[current].frame)
        } else {
            current
        };
        self.scopes[target].bindings.entry(name.clone()).or_insert(kind);
    }

    fn reference(&mut self, name: &mut String) {
        let scope = *self.stack.last().expect("reference inside a scope");
        self.uses.push(Use {
            scope,
            name: name.clone(),
        });
    }
}

impl Recorder {
    fn frame_scope(&self, frame: usize) -> usize {
        self.scopes
            .iter()
            .position(|s| s.frame == frame)
            .expect("a frame has a scope")
    }

    fn resolve(&self, from: usize, name: &str) -> Option<(usize, BindKind)> {
        let mut cursor = Some(from);
        while let Some(id) = cursor {
            if let Some(kind) = self.scopes[id].bindings.get(name) {
                return Some((id, *kind));
            }
            cursor = self.scopes[id].parent;
        }
        None
    }
}

struct Capture {
    capturer: usize,
    owner: usize,
    name: String,
    kind: BindKind,
}

pub(crate) fn capture_refusals(
    statements: &mut [Statement],
    repr_table: &ReprTable,
    phase: Phase,
) -> Vec<Diagnostic> {
    let mut recorder = Recorder::default();
    walk::walk_program(statements, &mut recorder);

    let mut captures = Vec::new();
    for u in &recorder.uses {
        let Some((scope, kind)) = recorder.resolve(u.scope, &u.name) else {
            continue;
        };
        let owner = recorder.scopes[scope].frame;
        let capturer = recorder.scopes[u.scope].frame;
        if owner == capturer || recorder.frames[owner].is_module {
            continue;
        }
        // Function and class names are program-wide downstream (not cells).
        if kind.is_program_wide() {
            continue;
        }
        captures.push(Capture {
            capturer,
            owner,
            name: u.name.clone(),
            kind,
        });
    }
    let env_owners: BTreeSet<usize> = captures.iter().map(|c| c.owner).collect();

    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for c in &captures {
        let (Some(capturer_key), Some(owner_key)) = (
            recorder.frames[c.capturer].label.as_deref(),
            recorder.frames[c.owner].label.as_deref(),
        ) else {
            continue; // A-2.6: no plan key, admitted and recorded in followups
        };
        if !seen.insert((c.capturer, c.name.clone())) {
            continue;
        }
        let reason = if structural_depth(&recorder.frames, c.capturer, c.owner, &env_owners) >= 2 {
            Some(CaptureRefusal::Depth)
        } else if phase == Phase::One && c.kind == BindKind::Param {
            Some(CaptureRefusal::Parameter { owner: owner_key })
        } else if !repr_has_a_cell(repr_table, owner_key, &c.name, phase) {
            Some(CaptureRefusal::ValueType)
        } else {
            None
        };
        if let Some(reason) = reason {
            out.push(Diagnostic::error(
                kali_common::e5::FEATURE_UNAVAILABLE as u32,
                kali_common::captured_binding_unavailable_message(capturer_key, &c.name, reason),
            ));
        }
    }
    out
}

/// `env_owning_hops`: count each step into an ancestor that owns a captured
/// binding, from the capturer up to and including the owner.
fn structural_depth(frames: &[Frame], from: usize, to: usize, owners: &BTreeSet<usize>) -> u32 {
    let mut depth = 0;
    let mut current = from;
    while current != to {
        let Some(parent) = frames[current].parent else {
            return 0;
        };
        if owners.contains(&parent) {
            depth += 1;
        }
        current = parent;
    }
    depth
}

fn repr_has_a_cell(table: &ReprTable, owner: &str, name: &str, phase: Phase) -> bool {
    if phase == Phase::One && table.binding_is_boolean_const(owner, name) {
        return false;
    }
    match table.scalar(owner, name) {
        Repr::I64 | Repr::Object(_) | Repr::AbortHandle => true,
        Repr::F64 => phase == Phase::Two,
        _ => false,
    }
}
```

Adjust the import paths for `e5` and `Diagnostic` to the ones `compile.rs`
already uses (`grep -n "^use" crates/kali_cli/src/build/compile.rs`). Make
`walk` and its items `pub(crate)` visible to the sibling module if they aren't
already.

Add `mod capture_refusals;` to `build/mod.rs`. In `compile.rs`, just before the
final `Ok(AnalyzedSource { … })`:

```rust
    // Captured-bindings spec §3.4 / A-2.6: the `check` mirror of the codegen
    // capture refusals. `run` reaches it too; codegen's refusal is the backstop.
    diagnostics.extend(crate::build::capture_refusals::capture_refusals(
        &mut parsed.statements,
        &repr_table,
        crate::build::capture_refusals::Phase::One,
    ));
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }
```

If `repr_table` is not in scope on every path there, place the call inside
the branch where it is. If the resolver is skipped, skip the pass.

- [ ] **Step 5: Run the unit tests**

Run: `cargo test -p kali_cli --lib capture_refusals`
Expected: PASS.

- [ ] **Step 6: Add the `check` cases**

In `captured_bindings.toml`, add one `check` case per refused probe from Task
4, `exit = "failure"` with the same `stderr_contains`, except for the residue
`p2` and `e5`. Those get:

```toml
[[case]]
name = "check_admits_a_local_copied_from_a_parameter_the_known_residue"
rationale = """A-2.6 residue: `n` is a TaggedVal local; `run` refuses it, `check` cannot see MIR layouts and exits 0. Recorded in the followups gap table."""
args = ["check", "p2.js"]
exit = "success"
```

`check` cases for `w3` and `w4` (compound / update on an unpromoted capture):
record what `check` does after this task. `w3` is refused as value type (F64).
`w4` is refused as a parameter.

- [ ] **Step 7: Run all cases and the rename tests**

Run: `cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- closure/ scope/ && cargo test -p kali_cli --lib`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_cli
git commit -m "feat(captured-bindings): phase 1 check mirror over the rename walk, with plan keys on function scopes"
```

---

### Task 6: Phase 1 blast radius, triage and the followups file

**Files:**
- Create: `docs/superpowers/followups/captured-bindings-discovered-defects.md`
- Modify: any case file the sweep moves

- [ ] **Step 1: Run everything**

```bash
cargo test --workspace --no-fail-fast 2>&1 | tee "$TMPDIR/cb1-workspace.txt" | tail -40
env -u FORCE_COLOR tools/array-return-probes/run.sh "$TMPDIR/cb1-probes.tsv"
diff <(cut -f1,2 "$TMPDIR/cb-all.tsv") <(cut -f1,2 "$TMPDIR/cb1-probes.tsv")
```

List every failing trial and every probe (any prefix, not only `cb_`) whose
verdict moved.

- [ ] **Step 2: Classify every move**

Re-run each moved program with `$KALI_BASE` and `env -u FORCE_COLOR node`, then
classify it:
- **wanted:** silent became refused;
- **capability loss:** correct at baseline, refused now;
- **other:** anything else, with a sentence each.

Apply the stop rule from Global Constraints.

- [ ] **Step 3: Re-pin and record**

Re-pin each moved trial according to its class. Every rationale gets a dated
line naming this project, node's output and kali's output.

Create the followups file. Its header follows `block-scoping-discovered-defects.md`
(filed by, oracle, measured at, what ships). Its sections:
- §1 the phase-1 triage table (name, baseline, phase 1, class);
- §2 capability loss;
- §3 the `check` / `run` gap: `p2`, `e5`, and any unlabelled frame;
- §4 left for later, empty for now; Task 13 fills it.

- [ ] **Step 4: Green**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
Expected: PASS.

- [ ] **Step 5: Commit, then STOP**

```bash
git add -A crates docs/superpowers/followups/captured-bindings-discovered-defects.md tools/array-return-probes
git commit -m "test(captured-bindings): phase 1 triage and re-pins"
```

Report to the human partner:
- the phase-1 triage table;
- any capability loss;
- the `check` / `run` gap.

Wait for a go-ahead before Task 7.

---

### Task 7: The captured-parameter rewrite

**Files:**
- Create: `crates/kali_cli/src/build/capture_param_rewrite/mod.rs`, `capture_param_rewrite_tests.rs`
- Modify: `crates/kali_cli/src/build/mod.rs`, `crates/kali_cli/src/build/compile.rs:817-827`

**Interfaces:**
- Consumes:
  - `kali_common::captured_param_spelling` (Task 1)
  - the rename walk with labels (Task 5)
- Produces: `pub fn rewrite_captured_params(statements: &mut [Statement]) -> usize` (the number of rewritten parameters).

- [ ] **Step 1: Write the failing unit tests**

```rust
use super::*;
use crate::build::block_scope_rename::test_support::parse;

fn rewritten(source: &str) -> (usize, Vec<Statement>) {
    let mut statements = parse(source);
    let n = rewrite_captured_params(&mut statements);
    (n, statements)
}

#[test]
fn a_captured_parameter_becomes_a_let_from_the_renamed_parameter() {
    let (n, got) = rewritten("function f(k){ const g=()=>k; return g(); }");
    assert_eq!(n, 1);
    let Statement::FunctionDeclaration(f) = &got[0] else { panic!("{got:?}") };
    assert_eq!(f.params, vec!["k{p}".to_string()]);
    let Statement::VariableDeclaration(d) = &f.body.body[0] else { panic!("{:?}", f.body.body) };
    assert_eq!(d.kind, "let");
    assert_eq!(d.declarations[0].id, "k");
    assert_eq!(d.declarations[0].init, Some(Expression::Identifier("k{p}".into())));
}

#[test]
fn an_uncaptured_parameter_is_untouched() {
    let source = "function f(k){ return k + 1; }";
    let (n, got) = rewritten(source);
    assert_eq!(n, 0);
    assert_eq!(got, parse(source));
}

#[test]
fn an_arrow_with_an_expression_body_gets_a_block() {
    let (n, got) = rewritten("const f=(k)=>()=>k;");
    assert_eq!(n, 1);
    let Statement::VariableDeclaration(d) = &got[0] else { panic!() };
    let Some(Expression::FunctionExpression(f)) = &d.declarations[0].init else {
        panic!("{:?}", d.declarations[0].init)
    };
    assert!(f.is_arrow);
    assert_eq!(f.params[0].name, "k{p}");
    let body = &f.body.as_ref().expect("block").body;
    assert!(matches!(&body[0], Statement::VariableDeclaration(_)));
    assert!(matches!(&body[1], Statement::ReturnStatement(_)));
}

#[test]
fn methods_and_function_expressions_are_rewritten() {
    let (n, _) = rewritten(
        "class C { m(k){ const g=()=>k; return g(); } } const h=function(j){ return ()=>j; };",
    );
    assert_eq!(n, 2);
}

#[test]
fn a_parameter_shadowed_in_the_closure_is_not_captured() {
    let (n, _) = rewritten("function f(k){ const g=(k)=>k; return g(1); }");
    assert_eq!(n, 0);
}

#[test]
fn the_rewrite_is_idempotent() {
    let mut statements = parse("function f(k){ const g=()=>k; return g(); }");
    assert_eq!(rewrite_captured_params(&mut statements), 1);
    assert_eq!(rewrite_captured_params(&mut statements), 0);
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_cli --lib capture_param_rewrite`
Expected: compile error (the module doesn't exist yet).

- [ ] **Step 3: Implement**

There are two passes, the same shape as the rename pass.
- **Pass A.** Reuse `capture_refusals`' `Recorder` (make it and `resolve`
  `pub(super)`) to find the set of `(frame, name)` for `BindKind::Param`
  bindings that some reference in another frame resolves to. Key the frames by
  enter order, which both walks share.
- **Pass B.** A second walk over the statements, with the frame counter
  advanced in the same order. For every function-like node whose frame has
  captured parameters:
  1. Replace each such parameter `k` with `captured_param_spelling("k")`.
  2. Prepend `let k = k{p};` to the body:
     `Statement::VariableDeclaration(VariableDeclaration { kind: "let".into(), declarations: vec![VariableDeclarator { id: k, init: Some(Expression::Identifier(spelled)) }] })`,
     one declarator per parameter, in parameter order.
  3. For an `ArrowFunctionExpression` (expression body), replace the
     expression with `Expression::FunctionExpression(Box::new(FunctionExpression { id: a.id.take(), params, body: Some(Box::new(BlockStatement { body: vec![decls…, Statement::ReturnStatement(ReturnStatement { argument: Some(body) })] })), is_async: a.is_async, generator: false, is_arrow: true }))`.

Pass B can't use `walk::walk_program`, because it must replace whole
expressions. Write a minimal recursive traversal in the module that matches
the variants `walk.rs` enumerates. Every `walk_expression` arm needs a
counterpart that recurses the same fields, with no `_ =>` arm, so a new AST
variant fails to compile. The frame counter must advance at exactly the nodes
where `walk.rs` calls `enter(ScopeKind::Module | ScopeKind::Function, …)`.
Assert in a debug build that the two passes counted the same number of frames.

The pass is idempotent because a `{p}` parameter is never captured: the body
reads `k`, not `k{p}`.

Add `pub mod capture_param_rewrite;` to `build/mod.rs`. In `compile.rs`,
directly after the block-scope rename and its `--compat eval` refusal:

```rust
    // Captured-bindings spec §3.2 / A-1: a captured parameter becomes a `let`
    // initialized from the renamed parameter, so it takes the local cell path.
    let rewritten_params =
        crate::build::capture_param_rewrite::rewrite_captured_params(&mut parsed.statements);
    if compat_eval && rewritten_params > 0 {
        return Err(vec![Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::block_scope_eval_refused_message(),
        )]);
    }
```

The `eval` refusal reuses block-scoping's message: `eval` could name the
original parameter, which no longer exists.

Then flip `capture_refusals`' call in `compile.rs` to keep `Phase::One` for
now. Parameters no longer reach it, and Task 9 flips it to `Phase::Two`.

- [ ] **Step 4: Measure the numeric-proof question (A-1 point 4)**

Add to `kali_types/src/repr_infer_tests.rs` (as rewritten source, with `kp`
standing in for `k{p}`, which repr_infer treats as an ordinary name):

```rust
#[test]
fn a_let_from_a_parameter_with_numeric_call_sites_is_proven_numeric() {
    let t = reprs("function f(kp){ let k = kp; const g=()=>k; return g(); } f(5); f(7);");
    assert!(t.binding_is_proven_numeric("f", "k"));
}

#[test]
fn a_let_from_a_parameter_with_a_string_call_site_is_not_proven_numeric() {
    let t = reprs("function f(kp){ let k = kp; const g=()=>k; return g(); } f(\"a\");");
    assert!(!t.binding_is_proven_numeric("f", "k"));
}
```

Run: `cargo test -p kali_types proven_numeric`

- **If both pass:** go on to Step 5.
- **If the first fails:** stop and report to the human partner. A-1 point 4
  says extending the proof is in scope, but how it's extended is a design
  decision. Bring the failing case and the relevant lines of
  `write_value_is_numeric` / `emit_table`'s numeric section.

- [ ] **Step 5: Run the tests and the cases**

Run: `cargo test -p kali_cli --lib && cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- closure/`
Expected:
- The unit tests pass.
- The parameter cases now refuse with "its value type has no closure cell"
  instead of "is a parameter of", because the parameter is now a `TaggedVal`
  local. Update their `stderr_contains` to the value-type reason, and add a
  dated rationale line: "Task 7: the rewrite makes `k` a local; it is refused
  as a TaggedVal until Task 8."

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli crates/kali_types/src/repr_infer_tests.rs
git commit -m "feat(captured-bindings): rewrite a captured parameter into a let initialized from the renamed parameter"
```

---

### Task 8: Promote proven-numeric `TaggedVal` cells

**Files:**
- Modify: `crates/kali_codegen/src/closure.rs` (`cell_is_promotable`)
- Create: `crates/kali_codegen/src/closure_tests.rs` (wire with `#[cfg(test)] #[path = "closure_tests.rs"] mod closure_tests;`)
- Modify: `crates/kali_codegen/src/emitter.rs:1253` (`promotable_scalar_cell_in`), `emit/closure_access.rs:195-240`, `lower.rs:1436,1471`, `iteration.rs:209,251,313`, `env_safety.rs:222`, `intrinsics/host.rs:1746`
- Modify: `crates/kali_cli/tests/cases/closure/captured_bindings.toml`

**Interfaces:**
- Produces:
  - `cell_is_promotable(repr_table, owner, name, is_scalar, allow_tagged: bool) -> bool`
  - `promotable_scalar_cell_in(&self, owner, name, is_scalar, allow_tagged) -> bool`

- [ ] **Step 1: Write the failing unit tests** (`closure_tests.rs`)

```rust
use super::*;

fn table(src: &str) -> kali_common::ReprTable {
    // kali_types is a dev-dependency of kali_codegen if this fails to resolve;
    // otherwise build the table by hand with `ReprTable::default()` +
    // `set_numeric_bindings`.
    kali_types::infer_reprs(&kali_types::test_support::parse_statements(src))
}

#[test]
fn a_tagged_cell_with_a_numeric_proof_promotes() {
    let t = table("function f(kp){ let k = kp; return k; } f(5);");
    assert!(cell_is_promotable(&t, "f", "k", false, true));
}

#[test]
fn a_tagged_cell_without_a_proof_does_not_promote() {
    let t = table("function f(kp){ let k = kp; return k; } f(\"a\");");
    assert!(!cell_is_promotable(&t, "f", "k", false, true));
}

#[test]
fn allow_tagged_false_keeps_the_baseline_verdict() {
    let t = table("function f(kp){ let k = kp; return k; } f(5);");
    assert!(!cell_is_promotable(&t, "f", "k", false, false));
}
```

If `kali_types` isn't reachable from `kali_codegen` tests, build the
`ReprTable` directly: `let mut t = ReprTable::default(); t.set_numeric_bindings([("f".into(), "k".into())].into());`.
Don't add a dependency.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_codegen --lib closure_tests`
Expected: compile error (wrong arity).

- [ ] **Step 3: Implement the predicate**

```rust
pub(crate) fn cell_is_promotable(
    repr_table: &kali_common::ReprTable,
    owner: &str,
    name: &str,
    is_scalar: bool,
    allow_tagged: bool,
) -> bool {
    let repr = repr_table.scalar(owner, name);
    if repr == kali_common::Repr::AbortHandle {
        return true;
    }
    if is_scalar {
        return repr == kali_common::Repr::I64;
    }
    if matches!(repr, kali_common::Repr::Object(_)) {
        return true;
    }
    // Captured-bindings A-1: a TaggedVal cell (a parameter's layout, and a
    // local copied from one) promotes only on positive numeric evidence.
    allow_tagged
        && repr == kali_common::Repr::I64
        && repr_table.binding_is_proven_numeric(owner, name)
}
```

Extend the doc comment with a third shape: "**A-1 tagged-i64**: a
`TaggedVal` cell whose repr is `I64` and whose binding is proven numeric".

`promotable_scalar_cell_in` becomes:

```rust
    pub(crate) fn promotable_scalar_cell_in(
        &self,
        owner: &str,
        name: &str,
        is_scalar: bool,
        allow_tagged: bool,
    ) -> bool {
        self.repr_table.scalar(owner, name) == kali_common::Repr::I64
            && (is_scalar
                || (allow_tagged && self.repr_table.binding_is_proven_numeric(owner, name)))
    }
```

Thread `is_tagged` through the call sites:
- **`closure_access.rs` `resolve_capture_access_inner`:** the `promotable`
  closure takes `(owner, is_scalar, allow_tagged)`. The active-iteration
  branch passes `false`. The own-cell branch passes `cell.is_tagged`. The
  captured branch passes `reference.is_tagged`.
- **`lower.rs:1436`** (function plans): `cell.is_tagged`.
- **`lower.rs:1471`** (iteration plans): `false`, with a comment:
  `// iteration cells are not widened (captured-bindings §1.1)`.
- **`iteration.rs:209`, `251`, `313`:** `false`, with the same comment.
- **`env_safety.rs:222`:** `cell.is_tagged` if it iterates a function plan's
  cells; `false` if it's an iteration plan. Read the loop to decide.
- **`intrinsics/host.rs:1746`:** `false`, with the comment
  `// the deferred lane is unchanged (captured-bindings §1.1)`.

- [ ] **Step 4: Run the unit tests**

Run: `cargo test -p kali_codegen --lib`
Expected: PASS.

- [ ] **Step 5: Update the cases to node's output**

These now print node's output: `p1`, `p2`, `v4`, `v8`, `v9`, `a3`, `a4`,
`rw1`, `e5`, `w4`. `w4` is the update expression; if it's still refused,
`try_emit_captured_assign`'s update lane gates on `promotable_scalar_cell_in`
and should now admit it. Read the update helper, and if it uses another gate,
route it through `resolve_scalar_capture_access`. Flip each to
`exit = "success"` with node's stdout, plus a dated rationale line.

Add the two Review Focus cases:

```toml
[[case]]
name = "owner_reads_what_the_closure_wrote"
rationale = """Review Focus 1: the owner's `return a+k` reads the cell the closure wrote. node prints `12`; baseline: quote the `cb_rw1` row of baseline-cb.tsv."""
args = ["run", "rw1.js"]
exit = "success"
stdout = "12\n"

[[case]]
name = "a_parameter_called_with_two_types_never_reads_zero"
rationale = """Review Focus 2: `f(1)` and `f("a")`. Either node's output or a refusal; never `0`. Pin whichever HEAD does, with its reason."""
args = ["run", "two.js"]
```

Complete `two`'s expectation from the measured HEAD behaviour: either
`exit = "success"` with `stdout = "1\na\n"`, or `exit = "failure"` with
`stderr_contains = ["E5506"]`. If it prints anything else at exit 0, stop and
report.

Also add `a_nested_declaration_reads_the_parameter` (`v8.js`, `5\n`).

These stay refused as value type: `v7` (string parameter), `a6` (F64, until
Task 9), `a7` (boolean parameter, unproven).

- [ ] **Step 6: Run the cases**

Run: `cargo build -q -p kali_cli && cargo test -p kali_cli --test cases`
Expected: PASS for `closure/`. Any other family that moved goes to Task 12;
note it now.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases
git commit -m "feat(captured-bindings): promote TaggedVal i64 cells with a numeric proof (A-1)"
```

---

### Task 9: F64 cells

**Files:**
- Modify: `docs/superpowers/specs/2026-10-05-captured-bindings-design.md` (amendment A-4)
- Modify: `crates/kali_codegen/src/closure.rs`, `closure_tests.rs`
- Modify: `crates/kali_codegen/src/emit/closure_access.rs` (read, declaration, `=` and compound arms)
- Modify: `crates/kali_codegen/src/emit/operators.rs:1724-1746` (`is_float_valued`'s identifier arm)
- Modify: `crates/kali_cli/src/build/compile.rs` (`Phase::Two`)
- Modify: `crates/kali_cli/tests/cases/closure/captured_bindings.toml`

**Interfaces:**
- Produces: `FunctionEmitter::captured_cell_repr(&self, name: &str) -> Option<kali_common::Repr>`, the owner repr of a name `resolve_capture_access` lowers.

- [ ] **Step 1: Record amendment A-4**

Append to the spec's §6:

```markdown
### A-4 (2026-10-05, plan Task 9): F64 writes are `=`, `+=`, `-=`, `*=`, `/=`

An F64 cell stores the double's bits through `i64.reinterpret_f64` /
`f64.reinterpret_i64`, so the 8-byte slot and the store/load helpers stay
untyped. `=` and `+= -= *= /=` lower with f64 arithmetic. `%=` (wasm has no
f64 remainder), the bitwise compound operators and `++` / `--` on an F64 cell
keep their existing E5506. §3.3's "compound assignment and update" sentence is
narrowed to that set, and the followups record the rest.
```

Commit it with the code at Step 7.

- [ ] **Step 2: Write the failing cases and a unit test**

Flip these cases to node's output:
- `a2`: `1.5`
- `a6`: `1.5` (F64 parameter, proven numeric by `f(1.5)`)
- `w2`: `2.5`
- `fa`: `3.5`
- `fw`: `2.75`

`w3` (`x+=1`) becomes `2.5`.

Add `captured_float_arithmetic_in_the_closure` on `fa.js` (Review Focus 3).
Its rationale quotes the E4201 risk.

Unit test in `closure_tests.rs`:

```rust
#[test]
fn an_f64_scalar_cell_promotes_and_a_tagged_f64_needs_a_proof() {
    let mut t = kali_common::ReprTable::default();
    t.set_scalar("f", "x", kali_common::Repr::F64);
    assert!(cell_is_promotable(&t, "f", "x", true, false));
    assert!(!cell_is_promotable(&t, "f", "x", false, true));
    t.set_numeric_bindings([("f".to_string(), "x".to_string())].into());
    assert!(cell_is_promotable(&t, "f", "x", false, true));
}
```

If `ReprTable` has no `set_scalar`, use whatever setter `repr_infer` uses to
record a binding's scalar repr (`grep -n "pub fn set_" crates/kali_common/src/repr.rs`).

- [ ] **Step 3: Run them and watch them fail**

Run: `cargo test -p kali_codegen --lib closure_tests && cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- closure/`
Expected: FAIL.

- [ ] **Step 4: Implement**

**The predicate:** `is_scalar` admits `I64 | F64`, and the tagged branch admits
`I64 | F64` with the proof. `promotable_scalar_cell_in` stays `I64`-only: it
is the arithmetic gate, and F64 arithmetic gets its own arms below.

**The helper**, in `closure_access.rs`:

```rust
    /// Captured-bindings Task 9: the owner repr of a name the capture lane
    /// lowers, so a read can carry its float shape.
    pub(crate) fn captured_cell_repr(&self, name: &str) -> Option<kali_common::Repr> {
        self.resolve_capture_access(name)?;
        let owner = self.scalar_capture_owner(name)?;
        Some(self.repr_table.scalar(&owner, name))
    }
```

**Read** (`try_emit_captured_read`): after `emit_cell_load`:

```rust
        if self.captured_cell_repr(name) == Some(kali_common::Repr::F64) {
            function.instruction(&Instruction::F64ReinterpretI64);
            return Some(EmittedValue { produced: true, shape: ValueShape::Float });
        }
```

**Declaration** (`try_emit_captured_decl`) and the `"="` arm of
`try_emit_captured_assign`: emit the value, convert it to f64 when the cell is
F64, then store the bits. The `"="` arm today resolves through
`resolve_scalar_capture_access`, which is `I64`-only. Change it so `"="`
resolves through `resolve_capture_access` when `captured_cell_repr(name)` is
`F64`:

```rust
        let is_f64 = self.captured_cell_repr(name) == Some(kali_common::Repr::F64);
        // … after emitting the value (`produced` / not):
        if is_f64 {
            if !self.is_float_valued(value_node) {
                function.instruction(&Instruction::F64ConvertI64S);
            }
            function.instruction(&Instruction::I64ReinterpretF64);
        }
        crate::closure::emit_cell_store(function, env_global, depth, offset, scratch);
```

When the value was not produced, push `F64Const(0.0)` instead of
`I64Const(0)` for an F64 cell, then reinterpret. An assignment expression must
leave its value on the stack. Check how the existing `"="` arm does it: if
the stored value is re-read after the store, re-read it with
`emit_cell_load` + `F64ReinterpretI64` for an F64 cell.

**Compound `+= -= *= /=` on an F64 cell:** a new arm before the `I64` arms,
taken when `is_f64`:
1. load the cell, then `F64ReinterpretI64`;
2. emit the right-hand side, converting it to f64 when it isn't float-valued;
3. `F64Add` / `F64Sub` / `F64Mul` / `F64Div`;
4. `I64ReinterpretF64`, then store.

`%=` and the bitwise operators on an F64 cell return `None`, so they keep
their existing E5506 (A-4).

**`is_float_valued`** (`operators.rs`, the `0 =>` identifier arm): before
`self.scalar_repr(name) == F64`, and only when `name` is not in
`self.locals`, add:

```rust
                    if !self.locals.contains_key(name) {
                        if let Some(repr) = self.captured_cell_repr(name) {
                            return repr == kali_common::Repr::F64;
                        }
                    }
```

**`check`:** flip the `compile.rs` call to `Phase::Two`. Boolean consts are
handled in Task 10. Until then, guard them with an explicit clause in
`repr_has_a_cell` that keeps refusing a boolean const in both phases, and add
a `// Task 10` note.

- [ ] **Step 5: Run the tests**

Run: `cargo test -p kali_codegen --lib && cargo test -p kali_cli --lib && cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- closure/`
Expected: PASS.

Also run every F64 probe with `--output json` and confirm no E4201:

```bash
for p in a2 a6 w2 w3 fa fw; do target/debug/kali run tools/array-return-probes/probes/cb_$p.js; echo "exit $?"; done
```

- [ ] **Step 6: Run the workspace**

Run: `cargo test --workspace`
Expected: PASS, apart from movers you note for Task 12.

- [ ] **Step 7: Commit**

```bash
git add crates docs/superpowers/specs/2026-10-05-captured-bindings-design.md
git commit -m "feat(captured-bindings): F64 cells through i64 reinterpretation; check phase two (A-4)"
```

---

### Task 10: A captured boolean `const` reads as a boolean

**Files:**
- Modify: `crates/kali_codegen/src/emit/closure_access.rs` (`try_emit_captured_read`)
- Modify: `crates/kali_cli/src/build/capture_refusals/mod.rs` (`repr_has_a_cell`)
- Modify: `crates/kali_cli/tests/cases/closure/captured_bindings.toml`

- [ ] **Step 1: Flip the cases**

`kb2` and `kb3` now expect `exit = "success"`, `stdout = "true\n"`. Their
`check` cases expect `exit = "success"`. Add:

```toml
[[case]]
name = "a_captured_boolean_const_returned_still_renders_one"
rationale = """A-2.1: the closure read is a boolean, but `console.log(f())` renders a user function's return through R-34's lane, so it stays `1`; not this project's. node prints `true`."""
args = ["run", "kb_ret.js"]
exit = "success"
stdout = "1\n"
```

`kb_ret.js` is
`function f(){ const b=true; const g=()=>b; return g(); } console.log(f());`.
Add it to `[source]` and as probe `cb_kb_ret`. If HEAD prints `true` instead,
pin `true` and note that R-34 was not reached.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- closure/`
Expected: `kb2` and `kb3` FAIL (still refused).

- [ ] **Step 3: Implement**

In `try_emit_captured_read`, replace the phase-1 refusal block with:

```rust
        let boolean = self.captured_boolean_const_owner(name).is_some();
        let (depth, offset) = self.resolve_capture_access(name)?;
        crate::closure::emit_cell_load(function, self.current_env_global(), depth, offset);
        Some(EmittedValue {
            produced: true,
            shape: if boolean { ValueShape::Boolean } else { ValueShape::Scalar },
        })
```

Keep the F64 branch from Task 9 ahead of the `Scalar` return.

In `repr_has_a_cell`, drop the boolean-const refusal.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_cli --lib && cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- closure/`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates tools/array-return-probes/probes/cb_kb_ret.js
git commit -m "feat(captured-bindings): a captured boolean const reads as a boolean (A-2.1)"
```

---

### Task 11: The `check` / `run` differential case

**Files:**
- Modify: `crates/kali_cli/tests/cases/closure/captured_bindings.toml`

- [ ] **Step 1: Measure both commands over every probe**

```bash
for p in tools/array-return-probes/probes/cb_*.js; do
  target/debug/kali run "$p" >/dev/null 2>"$TMPDIR/r.err"; r=$?
  target/debug/kali check "$p" >/dev/null 2>&1; c=$?
  grep -q "that captures" "$TMPDIR/r.err" && echo "$(basename $p) run=$r check=$c"
done
```

- [ ] **Step 2: Pin the property**

For every probe where `run` refuses with "that captures", there must be a
`check` case:
- `exit = "failure"`, for a probe not on the residue list;
- `exit = "success"`, for one on it, with its rationale naming A-2.6.

After phase 2 the residue list is exactly the `TaggedVal` locals with no
numeric proof that still refuse under `run`. Write the list into the case
file header and into the followups' §3. A probe refused by `run` but admitted
by `check` that is not on the list is a bug: fix the `check` pass.

- [ ] **Step 3: Run and commit**

Run: `cargo test -p kali_cli --test cases -- closure/`
Expected: PASS.

```bash
git add crates/kali_cli/tests/cases
git commit -m "test(captured-bindings): check/run differential over every cb probe, residue named"
```

---

### Task 12: Phase 2 blast radius, byte identity and triage

**Files:**
- Modify: `docs/superpowers/followups/captured-bindings-discovered-defects.md`
- Modify: any case file the sweep moves

- [ ] **Step 1: Run everything**

```bash
cargo test --workspace --no-fail-fast 2>&1 | tee "$TMPDIR/cb2-workspace.txt" | tail -40
env -u FORCE_COLOR tools/array-return-probes/run.sh "$TMPDIR/cb2-probes.tsv"
diff <(cut -f1,2 "$TMPDIR/cb1-probes.tsv") <(cut -f1,2 "$TMPDIR/cb2-probes.tsv")
```

- [ ] **Step 2: Byte identity**

Take three programs with nothing captured, compile each with both binaries,
and compare the wasm:
- the corpus program `bs_ok_no_shadow`;
- `function f(a){ return a+1; } console.log(f(2));`
- `function o(){ let c=0; const i=()=>{c+=1;}; i(); return c; } console.log(o());` (an i64 capture that already worked).

```bash
for p in tools/array-return-probes/probes/bs_ok_no_shadow.js "$TMPDIR/ni1.js" "$TMPDIR/ni2.js"; do
  rm -rf "$TMPDIR/b" "$TMPDIR/h"
  "$KALI_BASE" build "$p" --out-dir "$TMPDIR/b" && target/debug/kali build "$p" --out-dir "$TMPDIR/h"
  for w in "$TMPDIR"/b/*.wasm; do cmp "$w" "$TMPDIR/h/$(basename "$w")" && echo "identical: $p"; done
done
```

A difference is a finding. Stop and report it, with a `wasm-tools print` /
`wasm2wat` diff if either is installed via mise.

- [ ] **Step 3: Classify and re-pin**

Every move against phase 1 is one of:
- **wanted:** refused → node-equal;
- **capability loss:**
- **other.**

The stop rule applies. Re-pin with dated rationale lines. Add the phase-2
triage table as the followups' §1.2.

- [ ] **Step 4: Green**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add -A crates docs/superpowers/followups tools/array-return-probes
git commit -m "test(captured-bindings): phase 2 triage, byte identity, re-pins"
```

---

### Task 13: Docs

**Files:**
- Modify: `specs/15-errors.md`, `specs/19-feature-maturity.md`
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md`
- Modify: `docs/superpowers/followups/block-scoping-discovered-defects.md` (§7.11)
- Modify: `docs/superpowers/followups/captured-bindings-discovered-defects.md`

- [ ] **Step 1: `specs/15-errors.md`**

Under E5506's registered texts, add
`captured_binding_unavailable_message` with its three reasons. Each reason
gets one line naming the spec section (§3.1, A-2) and "raised by `run`
(codegen) and `check` (the `capture_refusals` pass), except the A-2.6
residue, which only `run` raises".

- [ ] **Step 2: `specs/19-feature-maturity.md`**

Add one row after the block-scoping row:

```markdown
| Closures over parameters and number bindings | Phase 1 MVP | A synchronous closure reads and writes a binding of its directly enclosing function: an integer or float parameter whose every call site passes a number, an integer or float local, and reads a boolean `const` (`const b = x < y`) as a boolean. Float writes are `=`, `+=`, `-=`, `*=`, `/=`. Refused (E5506, `run` and `check`): a captured string, a captured binding two or more closures away, a float `%=`/bitwise/`++`/`--`. Refused by `run` only: a captured binding copied from a parameter without numeric call sites. Not claimed: deferred callbacks over a float (unchanged), a captured `let` boolean (renders `1`, as uncaptured), object parameters |
```

- [ ] **Step 3: The register**

Add an entry after the highest R-id (find it with
`grep -n "^### R-" docs/superpowers/followups/kali-silent-miscompile-register.md | tail -1`)
for §7.11 item 1, "a captured binding without a cell reads `0`":
- repros `p1`, `a1`, `a2`, `kb2`, `w1`, with node output and baseline output
  from `baseline-cb.tsv`;
- status FIXED for i64/F64 parameters and locals and boolean consts at the
  Task 8 / 9 / 10 commits;
- FAIL_CLOSED for strings, depth ≥ 2 and unproven parameters at the Task 4
  commit;
- the cases.

Add a second entry for §7.11 item 2 (depth 2), FAIL_CLOSED, with cases
`d02` and `w6`. Update the status table rows to match.

- [ ] **Step 4: Block-scoping followups §7.11**

Items 1 and 2 each get a dated "Status:" line naming the register entry and
the commit.

- [ ] **Step 5: Complete the captured-bindings followups**

In §4, "left for later":
- string captures;
- Object parameters;
- the depth-2 lowering (with block-scoping §4);
- deferred F64;
- F64 `%=` / bitwise / `++` / `--` (A-4);
- a captured `let` boolean;
- frames with no plan key in `check`;
- the A-2.6 residue.

- [ ] **Step 6: Consistency re-read**

Re-read `specs/12-cli.md`, `specs/18-schemas.md` and `README.md`, and confirm
none needs a change. Re-read the maturity row against
`closure/captured_bindings.toml`: every claim has a case, and every refusal
has a case.

- [ ] **Step 7: Commit**

```bash
git add specs docs
git commit -m "docs(captured-bindings): errors, maturity row, register entries, followups"
```
