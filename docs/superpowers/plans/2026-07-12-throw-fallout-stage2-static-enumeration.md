# throw-fallout Stage 2 — Static Object Enumeration Semantics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make static object enumeration honest end-to-end — quoted-key object literals get real repr shapes (F-Stage1-4), ES integer-first key ordering has one shared source of truth, `delete`+reinsert folds against a per-binding static shape timeline instead of stale constants, and the silent `keys.length`/`keys[0]` miscompile over enumeration results is fixed — draining the Stage-2 slice of the 974-test throw-fallout set with zero flips.

**Architecture:** Four lanes over the existing compile-time enumeration fold. Lane C (delete) starts in the **parser**: `TokenType::Delete` is currently swallowed (no `parse_unary_expression` arm — the exact historical `typeof` bug documented at `crates/kali_parser/src/expression/mod.rs:98-102`), so `delete r.b;` reaches LIR as a bare member-read statement and every downstream "delete" arm is dead code. After the parser fix, an order-aware fold pass in `kali_optimize` maintains a per-binding property timeline through straight-line top-level code and erases consumed deletes; any `delete` that survives to codegen hits a new default-deny error arm (allowlist-at-the-choke-point, the Spec 4a lesson). Lane A is AST-side only (`record_object_literal` + `clean_shape` — HIR already strips key quotes, verified by LIR probe). Lane B hoists the ES ordering into `kali_common`. Lane D's sharp hypothesis: the fold clones property-key text verbatim into `Literal` nodes, so identifier-keyed folds produce **unquoted** literal text that downstream doesn't read as a string (the fold's own unit tests hand-build quoted key text the real front end never produces — provenance mismatch, Spec 3 lesson).

**Tech Stack:** Rust workspace (kali compiler: kali_parser → kali_hir → kali_mir → kali_lir → kali_optimize → kali_codegen), `cargo test` integration tests in `crates/kali_cli/tests/` invoking the built `kali` binary on temp-dir fixtures, node v26.x as the parity oracle.

## Global Constraints

- Branch: `soundness-batch1-pra`. Never commit to `main`.
- Gate verdict command: `cargo test --workspace`. Enumerating the failing set REQUIRES `cargo test --workspace --no-fail-fast`. Baseline: the `main` worktree at `/workspace/.worktrees/kali-main` (0 failures; expected HEAD `b48a067d3` — if it differs, STOP and reconcile per the denominator doc).
- Stage-gate pass = the failing set strictly shrank from the 974-name post-Stage-1 set AND no main-green test is red at the checkpoint. Honest-red is allowed mid-stage (Tasks 1–6 change behavior before pins land), never at the checkpoint.
- Fix, never flip. Every new expectation is derived by running the equivalent source under `node` on the same fixture, never from whatever makes a test pass. Rejects may only replace **silent no-ops on surface no target test exercises** (e.g. out-of-lane `delete`), never green a failing target test.
- No re-masking: a fix that silently no-ops a self-check `throw` is a defect even if the test goes green (Task 7 guards this). Re-mask guards must anchor on a **genuine** mismatch (Stage 1 final-review lesson).
- Both-sides discipline: `record_object_literal` (kali_types/repr_infer.rs) and `clean_shape` (kali_types/monomorphize.rs:955) are documented hand-mirrors ("exactly repr_infer's acceptance rule") — they change together in the same task. Any new codegen-reachable node text (e.g. `"delete"`) needs its codegen arm in the same stage.
- Hand-mirrored-list sweep at checkpoint: `SYNTHETIC_FUNCTIONS` test-side copy in `count_tag_boxing_ops` (crates/kali_cli/tests/runtime_smoke.rs) — no new synthetics are expected this stage, but the grep sweep is mandatory (Stage 1's two-test regression).
- Boolean printing: `console.log(<comparison>)` prints `1`/`0` in kali (pre-existing pinned divergence). New tests use throw-based self-checks (`if (...) throw` + `console.log('ok')`) wherever possible.
- Temp fixture dirs in tests MUST use the per-process `AtomicU64` counter slug convention (see the `run_source` helper at crates/kali_cli/tests/runtime_string_value_flow.rs:8).
- Before the final commit of the stage: `cargo fmt --all` and `cargo clippy --workspace --all-targets` clean.
- Scratchpad for machine-local artifacts: `/tmp/claude-1000/-workspace/457cb3a8-934e-4524-9201-1adb6c3f06bb/scratchpad` (referred to as `$SCRATCH` below; expand it literally in commands).

---

### Task 0: Stage-start triage — pin the target set empirically

**Files:**
- Create: `docs/superpowers/followups/throw-fallout-stage2-triage.md`

**Interfaces:**
- Produces: `$SCRATCH/stage2-pre.txt` (one test name per line, sorted, duplicates kept) and the triage doc with a per-family lane attribution table. Task 8 diffs against these. Later tasks consult the attribution table for which families they green.

- [ ] **Step 1: Confirm the main worktree is unchanged**

Run: `git -C /workspace/.worktrees/kali-main rev-parse --short HEAD`
Expected: `b48a067d3`. If it differs, STOP and re-verify the worktree per the denominator doc before proceeding.

- [ ] **Step 2: Enumerate the current branch failing set**

Run (long — full workspace build + test):
```bash
cargo test --workspace --no-fail-fast 2>&1 | tee $SCRATCH/stage2-pre.log
grep -E '^test .+ \.\.\. FAILED$' $SCRATCH/stage2-pre.log | sed 's/^test //; s/ \.\.\. FAILED$//' | sort > $SCRATCH/stage2-pre.txt
wc -l $SCRATCH/stage2-pre.txt
```
Expected: 974 names (the post-Stage-1 set recorded in the denominator doc's "Stage 1 drain" section). Small drift (±a few): note the names in the triage doc. Large drift: STOP and reconcile.

- [ ] **Step 3: Build the branch binary and probe the four canonical reproducers vs node**

Build once: `cargo build -p kali_cli` (binary at `target/debug/kali`).
For each probe below, write the source to `$SCRATCH/probe-N.js`, run `./target/debug/kali run $SCRATCH/probe-N.js` AND `node $SCRATCH/probe-N.js`, and record stdout/stderr/exit-code of BOTH in the triage doc verbatim.

Probe 1 (Lane D — quoted-key enumeration element/length):
```js
const keys = Object.keys({ "b": 1 });
console.log(keys.length);
console.log(keys[0]);
```
node: `1` then `b`. Known branch behavior (F-Stage1-4 addendum): `2` then `0`, exit 0, silent.

Probe 2 (Lane D — UNQUOTED identifier keys, same reads — this splits "quoted-key gap" from "folded-array element reads are broken generally"):
```js
const keys = Object.keys({ b: 1 });
console.log(keys.length);
console.log(keys[0]);
```
node: `1` then `b`. Record what the branch does — this is the sharpest single datum for Task 4.

Probe 3 (Lane C — delete+reinsert, the runtime_smoke.rs:954 core):
```js
const r = { "a": 1, "b": 2, "c": 3 };
delete r.b;
r.b = 4;
const ks = Object.keys(r);
const vs = Object.values(r);
if (ks.length !== 3 || ks[0] !== 'a' || ks[1] !== 'c' || ks[2] !== 'b') throw new Error('keys stale');
if (vs[0] !== 1 || vs[1] !== 3 || vs[2] !== 4) throw new Error('values stale');
console.log('ok');
```
node: `ok`. Branch: expect the stale fold to fire the self-check (trap since Stage 0) — record exactly.

Probe 4 (Lane A — quoted-key for..in):
```js
const o = { "b": 1, "2": 2, "a": 3, "1": 4 };
for (var k in o) { console.log(k); }
```
node: `1`,`2`,`b`,`a`. Branch: expect E5506 reject (F-Stage1-4).

- [ ] **Step 4: Attribute the target superset per-family**

The target superset = the #4 bucket's 46 names (denominator doc §"#4 delete+reinsert / own-keys staleness": `browser_reflect_own_keys` 40, `reflect_own_keys_js_input` 4, `runtime_smoke` direct-iteration 2) PLUS the 44 `frozen_object`-pattern names the Stage 1 triage tagged "#4-adjacent" (throw-fallout-stage1-triage.md §"Expected to REMAIN red"). For each family, read its fixture source (`crates/kali_cli/tests/browser_reflect_own_keys.rs:12-198`, `crates/kali_cli/tests/reflect_own_keys_js_input.rs`, `crates/kali_cli/tests/runtime_smoke.rs:850-985`, and the `frozen_object*` test files located via `grep -rl frozen_object crates/kali_cli/tests --include=*.rs`) and record in a table: family → fixture constructs used → blocking lane(s) (A/B/C/D) or out-of-stage cause (`for await`/async → Stage 7; `[]`+`.push` → Stage 4; host wiring → Stage 3). Use the probe results, not name patterns. State the honest expected drain as a RANGE with the multi-blocked names listed (Stage 1's forecast-falsified lesson: no drain claim rests on a name pattern).

- [ ] **Step 5: Sweep the repo for unary-`delete` usage (Task 1/6 blast radius)**

Run:
```bash
grep -rn "delete " crates/kali_cli/tests crates/kali_cli/src crates/kali_common/src --include=*.rs | grep -v "env\.delete\|\[\"delete\"\]\|'delete'\|\"delete\"" > $SCRATCH/stage2-delete-sweep.txt
```
Classify every hit in the triage doc: (a) `delete process.env.*` / `delete globalThis.process.env.*` unary forms (known: crates/kali_cli/tests/node_api_surface/core.rs:618-627, late_compat_js_input.rs:537-539, late_compat_browser_js_input/misc.rs:727-728, late_compat_browser_tsx_input.rs) — note for each whether the test is a **run** test (executes the binary) or a **source-string assertion** test (late_compat ones assert source text only), and whether it is currently green (absent from stage2-pre.txt); (b) object-delete forms (known: runtime_smoke.rs:955, red); (c) irrelevant (Rust `delete` in strings/comments). Task 1 Step 5 and Task 6 Step 4 re-verify every green run-test family in (a) stays green.

- [ ] **Step 6: Write the triage doc and commit**

`docs/superpowers/followups/throw-fallout-stage2-triage.md` sections: pre-stage count + drift; probe transcripts (all four, kali AND node); the attribution table; the delete-usage sweep classification; "follow-ups opened this stage" (empty, filled by later tasks).

```bash
git add docs/superpowers/followups/throw-fallout-stage2-triage.md
git commit -m "docs(soundness): throw-fallout Stage 2 triage — pin the target set empirically"
```

---

### Task 1: Parser — `delete` becomes a real unary expression

**Files:**
- Modify: `crates/kali_parser/src/expression/mod.rs` (new arm in `parse_unary_expression`, after the `TokenType::Typeof` arm at ~line 103-112)
- Test: `crates/kali_codegen/src/ctx_tests.rs` (LIR-shape pin — kali_codegen is the crate whose test support runs the full parser→HIR→MIR→LIR pipeline via `parse_and_lower_lir`, test_support.rs:36)

**Interfaces:**
- Consumes: `TokenType::Delete` (already lexed, crates/kali_lexer/src/identifier.rs:60).
- Produces: `Expression::UnaryExpression { operator: "delete", argument }` in the AST; LIR node `Value` with `text == Some("delete")` and one member-expression child. Task 5's statement recognizer and Task 6's codegen arm both key on exactly this LIR shape.

- [ ] **Step 1: Write the failing LIR-shape test**

Add to `crates/kali_codegen/src/ctx_tests.rs`:

```rust
#[test]
fn delete_statement_survives_to_lir_as_a_delete_unary() {
    // Stage 2 (throw-fallout): the parser previously had NO
    // `TokenType::Delete` arm — the token was swallowed and `delete r.b;`
    // reached LIR as a bare member-read statement, making every downstream
    // "delete" arm dead code (the same historical bug the `typeof` comment
    // in kali_parser::expression::parse_unary_expression documents).
    let program = crate::test_support::parse_and_lower_lir("const r = { a: 1 };\ndelete r.a;");
    let found = program.nodes.iter().any(|n| {
        n.kind == LirNodeKind::Value
            && n.text.as_deref() == Some("delete")
            && n.children.len() == 1
    });
    assert!(found, "no Value(\"delete\") node in LIR: {:#?}", program.nodes);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_codegen delete_statement_survives_to_lir_as_a_delete_unary`
Expected: FAIL on the assert (no `delete` node exists — verified by the Stage-2 recon probe).

- [ ] **Step 3: Add the parser arm**

In `crates/kali_parser/src/expression/mod.rs`, `parse_unary_expression`, insert directly after the `Some(TokenType::Typeof) => { ... }` arm (mirror its comment convention):

```rust
            // `delete <expr>` was previously NOT parsed as a unary operator
            // (same historical bug as `typeof` above): the token fell through
            // to the primary parser, was swallowed, and `delete r.b` compiled
            // as a bare member read — a silent no-op with no diagnostic.
            // Parse it as a real unary expression; the optimizer's static
            // shape timeline consumes the provable lane and codegen
            // default-denies the rest (throw-fallout Stage 2).
            Some(TokenType::Delete) => {
                let _ = self.stream.advance();
                let argument = self.parse_unary_expression();
                Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "delete".to_string(),
                    argument,
                }))
            }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p kali_codegen delete_statement_survives_to_lir_as_a_delete_unary`
Expected: PASS.

- [ ] **Step 5: Blast-radius check — types side and env-delete run-tests**

(a) Build and run Probe 3's fixture (`$SCRATCH/probe-3.js` from Task 0):
```bash
cargo build -p kali_cli && ./target/debug/kali run $SCRATCH/probe-3.js; echo "exit=$?"
```
Expected: the program still COMPILES (kali_types visits the unary's argument as a plain expression — no delete-specific reject) and codegen's previously-dead `"delete"` arm (crates/kali_codegen/src/emit/operators.rs:200) now fires its non-env fallback: an E8001 UNIMPLEMENTED **warning** + no-op, so behavior is unchanged-stale for now (Task 5 fixes the semantics, Task 6 hardens the arm). If instead kali_types REJECTS the program (new E-code on stderr), record the diagnostic and add a pass-through arm for `"delete"` in the kali_types expression walker that visits the argument without recording a value flow — the reject would otherwise turn main-green env-delete tests red. Re-run the probe after any such fix.

(b) Run every family the Task 0 Step 5 sweep classified as a currently-green **run** test containing unary `delete` (at minimum):
```bash
cargo test -p kali_cli --test node_api_surface 2>&1 | tail -5
cargo test -p kali_cli --test runtime_smoke misc 2>&1 | tail -5
```
Expected: no new failures vs stage2-pre.txt (`delete process.env.X` now routes through the operators.rs env lane — `process_env_property_key` — or the warning fallback; both preserve exit-0 behavior). If a family newly fails, STOP: fix forward within this task (the env lane's key recognizer may need the failing spelling) before committing.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_parser/src/expression/mod.rs crates/kali_codegen/src/ctx_tests.rs
git commit -m "fix(parser): parse 'delete' as a real unary operator (throw-fallout Stage 2 Lane C prerequisite)"
```

---

### Task 2: Lane B — one shared ES property-ordering function

**Files:**
- Modify: `crates/kali_common/src/object.rs` (new public functions at the top, after the imports)
- Modify: `crates/kali_common/src/object_tests.rs` (unit tests; follow the existing module's test layout — object_tests/ dir exists)
- Modify: `crates/kali_optimize/src/helpers.rs:147-155` (`object_property_order_key` delegates to kali_common)
- Modify: `crates/kali_optimize/src/object_fold.rs:264-276` (`ordered_object_literal_properties` sort uses the shared comparator semantics — keep its index tiebreak, it is equivalent to the stable sort)

**Interfaces:**
- Produces: `kali_common::object::property_order_key(key: &str) -> Option<u64>` and `kali_common::object::sort_properties_es_order<T>(properties: &mut [(String, T)])` (stable: array-index-like keys ascending first, remaining keys keep insertion order; strips one level of `"` quoting before classification because LIR literal text may carry source quotes). Task 3 calls `sort_properties_es_order` from kali_types.

- [ ] **Step 1: Write the failing unit tests**

Add to the kali_common object tests (create `crates/kali_common/src/object_tests/ordering.rs` and register `mod ordering;` in `crates/kali_common/src/object_tests.rs`, matching how `reflect.rs` is registered):

```rust
use crate::object::{property_order_key, sort_properties_es_order};

#[test]
fn property_order_key_classifies_array_index_like_keys() {
    assert_eq!(property_order_key("0"), Some(0));
    assert_eq!(property_order_key("1"), Some(1));
    assert_eq!(property_order_key("\"2\""), Some(2)); // LIR text may keep quotes
    assert_eq!(property_order_key("01"), None); // leading zero: not an index
    assert_eq!(property_order_key(""), None);
    assert_eq!(property_order_key("b"), None);
    assert_eq!(property_order_key("4294967295"), None); // == 2^32-1: not an index
}

#[test]
fn sort_properties_es_order_matches_node_enumeration_order() {
    // node: Object.keys({ "b": 1, "2": 2, "a": 3, "1": 4 }) => ['1','2','b','a']
    let mut props = vec![
        ("b".to_string(), 1),
        ("2".to_string(), 2),
        ("a".to_string(), 3),
        ("1".to_string(), 4),
    ];
    sort_properties_es_order(&mut props);
    let keys: Vec<&str> = props.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, vec!["1", "2", "b", "a"]);
    let values: Vec<i32> = props.iter().map(|(_, v)| *v).collect();
    assert_eq!(values, vec![4, 2, 1, 3]);
}

#[test]
fn sort_properties_es_order_is_stable_for_string_keys() {
    let mut props = vec![
        ("z".to_string(), 0),
        ("a".to_string(), 1),
        ("m".to_string(), 2),
    ];
    sort_properties_es_order(&mut props);
    let keys: Vec<&str> = props.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, vec!["z", "a", "m"]); // insertion order, NOT alphabetical
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_common ordering`
Expected: FAIL to compile (`property_order_key` not found in `crate::object`).

- [ ] **Step 3: Implement in kali_common**

Add to `crates/kali_common/src/object.rs` (top of file, after existing `use` lines):

```rust
/// ES own-property enumeration order key: `Some(n)` when `key` is an
/// array-index-like string (canonical base-10, no leading zeros, `< 2^32-1`),
/// `None` otherwise. Strips one level of `"` quoting first — LIR literal
/// text keeps source quoting, while AST/repr key text is unquoted; both
/// layers must classify identically (throw-fallout Stage 2, Lane B).
pub fn property_order_key(key: &str) -> Option<u64> {
    let normalized = key.trim_matches('"');
    if normalized.is_empty() || (normalized.len() > 1 && normalized.starts_with('0')) {
        return None;
    }
    let value = normalized.parse::<u64>().ok()?;
    (value < u32::MAX as u64).then_some(value)
}

/// Stable in-place ES enumeration-order sort: array-index-like keys first in
/// ascending numeric order, then every other key in insertion order. The ONE
/// ordering used by the optimizer's enumeration fold, kali_types shape field
/// lists, and codegen key tables — divergence is impossible by construction.
pub fn sort_properties_es_order<T>(properties: &mut [(String, T)]) {
    properties.sort_by(|(left, _), (right, _)| {
        match (property_order_key(left), property_order_key(right)) {
            (Some(l), Some(r)) => l.cmp(&r),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal, // sort_by is stable
        }
    });
}
```

- [ ] **Step 4: Delegate the optimizer to the shared function**

Replace the body of `object_property_order_key` in `crates/kali_optimize/src/helpers.rs:147-155`:

```rust
    pub(crate) fn object_property_order_key(key: &str) -> Option<u64> {
        // Single source of truth (throw-fallout Stage 2, Lane B).
        kali_common::object::property_order_key(key)
    }
```

Leave `ordered_object_literal_properties`'s sort (object_fold.rs:264-276) as-is — its explicit `(order, source_index)` tiebreak is semantically identical to the stable shared sort, and it now routes through the shared classifier via the delegation above.

- [ ] **Step 5: Run tests to verify everything passes (behavior-neutral check)**

Run: `cargo test -p kali_common ordering && cargo test -p kali_optimize`
Expected: PASS, including all existing object_fold tests unchanged (this task must be behavior-neutral).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_common/src/object.rs crates/kali_common/src/object_tests.rs crates/kali_common/src/object_tests/ordering.rs crates/kali_optimize/src/helpers.rs
git commit -m "refactor(common): shared ES property-order classifier + stable sort (throw-fallout Stage 2 Lane B)"
```

---

### Task 3: Lane A — quoted-string object-literal keys get real repr shapes

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs:470-519` (`record_object_literal`)
- Modify: `crates/kali_types/src/monomorphize.rs:948-970` (`clean_shape` — the documented hand-mirror; both change together)
- Test: `crates/kali_types/src/repr_infer_tests.rs` or the existing repr_infer test module (locate with `grep -rn "mod repr_infer_tests\|record_object_literal" crates/kali_types/src --include=*.rs`; follow the existing pattern), plus a CLI e2e test in Task 7
- Sweep: any existing test pinning the quoted-key E5506 reject (`grep -rn "E5506" crates/kali_cli/tests crates/kali_types/src --include=*.rs | grep -i "quot\|string key\|non-identifier"`)

**Interfaces:**
- Consumes: `kali_common::object::sort_properties_es_order` (Task 2).
- Produces: `Repr::Object(shape)` materializes for `{ "b": 1 }`-style literals; shape field lists are in **ES enumeration order** (safe: identifier keys can never be array-index-like — they cannot start with a digit — so ordering is a no-op for every previously-admitted shape and only bites for the newly admitted quoted keys). The for..in E5506 gate (crates/kali_types/src/resolve/mod.rs:567-572) lifts automatically because it keys on shape presence. Task 7's e2e pins consume this.

- [ ] **Step 1: Write the failing unit test**

In the kali_types repr_infer test module (following its existing test-construction pattern — the tests build small programs and assert on inferred shapes/conflicts; adapt the snippet below to the module's actual helpers after reading two neighboring tests):

```rust
#[test]
fn quoted_string_keys_materialize_the_same_shape_as_identifier_keys() {
    // F-Stage1-4: `{ "b": 1, "a": 2 }` previously recorded a deferred
    // "non-identifier property name" conflict and never materialized a
    // shape; the byte-identical program with unquoted keys worked. Quoted
    // and unquoted keys are the same object in JS.
    // Field order is ES enumeration order: array-index-like keys first,
    // ascending; then insertion order.
    // { "b": 1, "2": 2, "a": 3, "1": 4 } -> fields ["1", "2", "b", "a"]
    // (assert via the module's shape-inspection helper on a program:
    //  `const o = { "b": 1, "2": 2, "a": 3, "1": 4 }; for (var k in o) { console.log(k); }`)
}
```

The assertion target: the inferred shape for `o` exists (no E5506 conflict) and its ordered field list equals `["1", "2", "b", "a"]`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_types quoted_string_keys_materialize`
Expected: FAIL — no shape is recorded (the let-else fires the deferred conflict).

- [ ] **Step 3: Implement — repr_infer side**

In `record_object_literal` (crates/kali_types/src/repr_infer.rs:478-486), replace the Identifier-only let-else:

```rust
            let key = match &prop.key {
                kali_ast::PropertyName::Identifier(key)
                | kali_ast::PropertyName::String(key) => key.clone(),
                kali_ast::PropertyName::Number(_) => {
                    // Honest fail-closed residue: unquoted numeric keys
                    // (`{ 1: x }`) stay off the shape lane until a fixture
                    // needs them (f64 canonicalization is its own problem).
                    // Quoted numeric-LIKE strings ("1") are ordinary string
                    // keys and are admitted above (throw-fallout Stage 2).
                    self.obj_pending_conflicts.insert(
                        slot.clone(),
                        format!(
                            "object literal for {slot:?} uses a numeric property name, which is unavailable in the current phase"
                        ),
                    );
                    return;
                }
            };
            let key = &key;
```

(The remaining body — getter/setter check, nested-object check, `obj_field_node_for(&slot, key)`, `names.push(key.clone())` — is unchanged; `key` keeps the same `&String`-compatible usage.)

Then, immediately before the `match self.obj_literal_fields.entry(slot.clone())` at line ~508, ES-order the names:

```rust
        // ES enumeration order (throw-fallout Stage 2, Lane B): one shared
        // ordering across shape fields, key tables, and the enumeration
        // fold. A no-op for identifier-only shapes (identifiers can't be
        // array-index-like), so pre-existing shapes are byte-identical.
        let mut keyed: Vec<(String, ())> = names.into_iter().map(|n| (n, ())).collect();
        kali_common::object::sort_properties_es_order(&mut keyed);
        let names: Vec<String> = keyed.into_iter().map(|(n, ())| n).collect();
```

Also update the function's doc comment (lines 462-469): "non-identifier key" → "numeric key" (string keys are now admitted).

- [ ] **Step 4: Implement — the monomorphize hand-mirror**

`clean_shape` (crates/kali_types/src/monomorphize.rs:955-970) — same admission rule AND the same ordering, or the mirror lies:

```rust
fn clean_shape(obj: &ObjectExpression) -> Option<ShapeTuple> {
    let mut names = Vec::with_capacity(obj.properties.len());
    for prop in &obj.properties {
        let key = match &prop.key {
            PropertyName::Identifier(key) | PropertyName::String(key) => key.clone(),
            PropertyName::Number(_) => return None,
        };
        if !matches!(prop.kind, ObjectPropertyKind::Init) {
            return None;
        }
        if matches!(prop.value, Expression::ObjectExpression(_)) {
            return None;
        }
        names.push(key);
    }
    let mut keyed: Vec<(String, ())> = names.into_iter().map(|n| (n, ())).collect();
    kali_common::object::sort_properties_es_order(&mut keyed);
    Some(keyed.into_iter().map(|(n, ())| n).collect())
}
```

Update its doc comment: it accepts Identifier + String keys, Number stays out, and the field list is ES-ordered — "exactly repr_infer's acceptance rule" must remain true.

- [ ] **Step 5: Run tests**

Run: `cargo test -p kali_types`
Expected: the new test PASSES; every pre-existing kali_types test still passes (identifier-only shapes are order-stable per the no-op argument above). If a monomorphize test fails, read it — it is either a vacuous pin of the old Identifier-only rule (re-pin node-derived) or a real ordering regression (STOP and diagnose).

- [ ] **Step 6: e2e smoke — Probe 4 goes green**

```bash
cargo build -p kali_cli
./target/debug/kali run $SCRATCH/probe-4.js
node $SCRATCH/probe-4.js
```
Expected: both print `1`,`2`,`b`,`a` (four lines, byte-identical). If kali's ORDER differs, the codegen key-table lane is not reading the shape's field list — find the divergence (`crates/kali_codegen/src/emit/control_flow.rs:395-420` reads `self.repr_table.shape_fields(shape)`) before proceeding.

- [ ] **Step 7: Re-pin any quoted-key E5506 reject pins**

Run the sweep from the Files block. For every test that pinned "quoted keys → E5506" (F-Stage1-4 documented the class), re-pin to the node-derived green behavior (run the fixture under node, pin that). If none exist, note "no reject pins found" in the triage doc.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/monomorphize.rs crates/kali_types/src/<test files>
git commit -m "fix(types): quoted-string object keys materialize real shapes in ES order (throw-fallout Stage 2 Lane A, F-Stage1-4)"
```

---

### Task 4: Lane D — enumeration-result element/length reads tell the truth

**Files:**
- Modify: `crates/kali_optimize/src/object_fold.rs` (key-literal text encoding in `fold_object_enumeration_call`, lines 112-155)
- Modify: `crates/kali_optimize/src/object_fold_tests/reflect_own_keys.rs` (+ siblings under `object_fold_tests/` — expectations move to quoted text AND the hand-built LIR key text moves to the real front-end shape)
- Possibly modify: `crates/kali_codegen/src/emit/call.rs` or `intrinsics/object.rs` (fail-closed backstop — only if Step 1's diagnosis shows the unfolded path is reachable-and-silent)
- Test: new CLI integration test in Task 7's file (the node-parity pins); unit tests here

**Interfaces:**
- Consumes: the Task 0 Probe 1/2 transcripts (the diagnosis inputs).
- Produces: folded enumeration arrays whose elements are **string literals in canonical quoted text** (`format!("{:?}", key)` — the encoding the fold's string-mode branch at object_fold.rs:76-98 already uses); `keys.length`/`keys[N]` reads over them return node-correct values.

- [ ] **Step 1: Diagnose with the probe pair (no code yet)**

Recon facts to verify (record findings in the triage doc):
1. The real front end lowers BOTH `{ "a": 1 }` and `{ a: 1 }` property keys to UNQUOTED `Literal` text (verified by LIR probe during planning: `"a": 1` → `Literal text=Some("a")`).
2. `ordered_object_literal_properties` (object_fold.rs:260) takes `key_node.text` verbatim, and the keys/ownKeys fold arms call `clone_string_literal(program, key)` with that verbatim text — so folded key elements are `Literal` nodes with UNQUOTED text (`b`), which downstream literal parsing (`parse_literal_text`) does NOT classify as strings.
3. The fold's own unit tests (object_fold_tests/reflect_own_keys.rs:24) hand-build key text `"\"1\""` (quoted) — a shape the front end never produces. Their passing is provenance-blind (Spec 3 lesson: mirror provenance, not expression shapes).

Confirm by instrumenting nothing: run Probe 2 (`{ b: 1 }`, unquoted) — if it also prints garbage, the unquoted-element hypothesis stands for identifier keys too and the fix below covers both probes. If Probe 2 is CORRECT while Probe 1 is broken, the divergence is upstream of the fold (quoted-key literals taking a different lane) — STOP, diagnose where Probe 1's `Object.keys` call actually lowers (is the fold even firing? add a temporary `eprintln!` in `fold_object_enumeration_call`, run Probe 1, remove it), and record the corrected root cause in the triage doc before choosing the fix.

- [ ] **Step 2: Write the failing unit test (front-end-shaped, not hand-built)**

Add to `crates/kali_optimize/src/object_fold_tests/reflect_own_keys.rs`:

```rust
#[test]
fn folded_keys_are_canonical_quoted_string_literals() {
    // Front-end provenance: LIR property-key text is UNQUOTED for both
    // `{ a: 1 }` and `{ "a": 1 }` (they are identical by HIR). The folded
    // enumeration array must emit its key elements as CANONICAL QUOTED
    // string-literal text (the same `format!("{:?}", ...)` encoding the
    // string-mode fold branch uses), or downstream length/element reads
    // see non-string literals (throw-fallout Stage 2 Lane D).
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    // Object.keys({ b: 1, "2": 2 }) with UNQUOTED key text, as the real
    // front end produces it:
    let callee_obj = builder.alloc_text(LirNodeKind::Value, "Object");
    let callee = builder.alloc_text(LirNodeKind::Value, "keys");
    builder.node_mut(callee).unwrap().children = vec![callee_obj];
    let k1 = builder.alloc_text(LirNodeKind::Literal, "b");
    let v1 = builder.alloc_text(LirNodeKind::Literal, "1");
    let p1 = builder.alloc_text(LirNodeKind::Value, "init");
    builder.node_mut(p1).unwrap().children = vec![k1, v1];
    let k2 = builder.alloc_text(LirNodeKind::Literal, "2");
    let v2 = builder.alloc_text(LirNodeKind::Literal, "2");
    let p2 = builder.alloc_text(LirNodeKind::Value, "init");
    builder.node_mut(p2).unwrap().children = vec![k2, v2];
    let object = builder.alloc(LirNodeKind::Value);
    builder.node_mut(object).unwrap().children = vec![p1, p2];
    let call = builder.alloc(LirNodeKind::Call);
    builder.node_mut(call).unwrap().children = vec![callee, object];
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram { root, nodes: builder.into_nodes() };
    Optimizer::new(OptimizationLevel::Fast).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let texts: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    // ES order (index-like "2" first), canonical quoted encoding:
    assert_eq!(texts, vec!["\"2\"", "\"b\""]);
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p kali_optimize folded_keys_are_canonical_quoted_string_literals`
Expected: FAIL — current elements are `["2", "b"]` unquoted (or the ES-order assert fires; record which).

- [ ] **Step 4: Implement the canonical encoding**

In `fold_object_enumeration_call` (object_fold.rs), the two key-cloning arms (`Object.keys` at ~line 113-119 and `Reflect.ownKeys` at ~line 120-126) and the entries key at ~line 142:

```rust
                for (key, _) in properties {
                    elements.push(self.clone_string_literal(
                        program,
                        format!("{:?}", key.trim_matches('"')),
                    ));
                }
```

(entries arm: same `format!("{:?}", key.trim_matches('"'))` for `key_id`.) The `trim_matches('"')` keeps the fix idempotent for any already-quoted text (the hand-built legacy test shape) while canonicalizing the real unquoted front-end shape.

- [ ] **Step 5: Update the legacy hand-built expectations**

Existing tests in `object_fold_tests/reflect_own_keys.rs` (and siblings — `grep -rn '\\\\"1\\\\"' crates/kali_optimize/src/object_fold_tests/`) assert mixed text like `vec!["\"1\"", "\"2\"", "b"]`. Update every such expectation to all-quoted (`vec!["\"1\"", "\"2\"", "\"b\""]`). These are encoding pins, not behavior flips — node is the behavior oracle and Step 6 pins it end-to-end.

- [ ] **Step 6: e2e verification — both probes**

```bash
cargo test -p kali_optimize && cargo build -p kali_cli
./target/debug/kali run $SCRATCH/probe-1.js && node $SCRATCH/probe-1.js
./target/debug/kali run $SCRATCH/probe-2.js && node $SCRATCH/probe-2.js
```
Expected: kali and node print identically (`1` then `b`) for BOTH probes. If probe output is STILL wrong, the unfolded/element-read path is implicated — diagnose whether `Object.keys` in a `const` initializer position even reaches the fold on the CLI pipeline (the CLI driver must run the optimizer; check where `kali_optimize::Optimizer` is invoked from the build pipeline) and record + fix at the actual divergence point. Do NOT commit a fix that only satisfies the unit test (fresh-binary rule).

- [ ] **Step 7: Fail-closed backstop decision**

If Step 1/6 diagnosis showed a reachable SILENT unfolded-enumeration lane (enumeration call in a value position that neither folds nor rejects — the class Probe 1 exposed), add an explicit reject where that lane bottoms out (expected site: the generic unresolved-call placeholder fallback in `crates/kali_codegen/src/emit/call.rs` — recognize `Object.keys/values/entries`/`Reflect.ownKeys` callees and emit `Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, "Object enumeration is only supported where the object has a compile-time-known fixed shape")` instead of the silent placeholder). Then re-run the full kali_cli test binary most likely to exercise it (`cargo test -p kali_cli --test runtime_smoke 2>&1 | tail -3`) and require no new failures vs stage2-pre.txt. If the diagnosis showed the fold covers every reachable case, record "backstop not needed — no reachable silent lane" in the triage doc instead.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_optimize/src crates/kali_codegen/src
git commit -m "fix(optimize): folded enumeration keys are canonical quoted string literals (throw-fallout Stage 2 Lane D)"
```

---

### Task 5: Lane C — static shape timeline in the enumeration fold

**Files:**
- Modify: `crates/kali_optimize/src/object_fold.rs` (timeline machinery: mutated-name scan, eligibility, ordered walk; new fns after `collect_constant_bindings_into`)
- Modify: `crates/kali_optimize/src/driver.rs:165-166` and `185-186` (both fold invocations become the ordered pass; the flat `collect_constant_bindings` used for specialization at :185 excludes mutated names)
- Test: `crates/kali_optimize/src/object_fold_tests/` (new file `timeline.rs`, registered in `object_fold_tests.rs`)

**Interfaces:**
- Consumes: LIR `Value("delete")` nodes (Task 1); `Value("=")` member-store nodes (pre-existing; LIR shape verified by probe: statement wrapper `Value(None)` → `Value("=")` → `[member, value]`, where member = `Value(text=key)` → `[Value(text=base)]`).
- Produces: `pub(crate) fn fold_object_enumeration_calls_ordered(&self, program: &mut LirProgram)` — the single fold entry point the driver calls; consumed `delete` statements are erased to empty `Block` nodes so they never reach codegen (Task 6's default-deny arm is the enforcement). `pub(crate) fn collect_mutated_binding_names(&self, program: &LirProgram) -> BTreeSet<String>` — also used at driver.rs:185 to exclude mutated names from the specialization env.

- [ ] **Step 1: Write the failing timeline unit tests**

Create `crates/kali_optimize/src/object_fold_tests/timeline.rs` (register `mod timeline;` in `object_fold_tests.rs`). Build LIR programs matching the **probe-verified front-end shape** (statement wrappers are `Value(None)` with one child; const decl = `Instruction("const")` → `Instruction(name)` → `[Value(name), init]`):

```rust
use super::*;

/// `Object.<method>(r)` call node: Call → [Value(method) → [Value("Object")], Value("r")].
fn build_enum_call(builder: &mut LirBuilder, method: &str) -> LirNodeId {
    let callee_obj = builder.alloc_text(LirNodeKind::Value, "Object");
    let callee = builder.alloc_text(LirNodeKind::Value, method);
    builder.node_mut(callee).unwrap().children = vec![callee_obj];
    let arg = builder.alloc_text(LirNodeKind::Value, "r");
    let call = builder.alloc(LirNodeKind::Call);
    builder.node_mut(call).unwrap().children = vec![callee, arg];
    call
}

/// const r = { a: 1, b: 2, c: 3 }; delete r.b; r.b = 4;
/// Object.keys(r); Object.values(r);
/// — LIR shapes exactly as the front end produces them (probe-verified
/// during planning: statement wrappers are Value(None) with one child;
/// const decl = Instruction("const") → Instruction(name) → [Value(name), init]).
/// Returns (program, del_unary_id, keys_call_id, values_call_id).
fn build_delete_reinsert_program() -> (LirProgram, LirNodeId, LirNodeId, LirNodeId) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    // object literal { a: 1, b: 2, c: 3 }
    let mut props = Vec::new();
    for (k, v) in [("a", "1"), ("b", "2"), ("c", "3")] {
        let key = builder.alloc_text(LirNodeKind::Literal, k);
        let value = builder.alloc_text(LirNodeKind::Literal, v);
        let p = builder.alloc_text(LirNodeKind::Value, "init");
        builder.node_mut(p).unwrap().children = vec![key, value];
        props.push(p);
    }
    let literal = builder.alloc(LirNodeKind::Value);
    builder.node_mut(literal).unwrap().children = props;
    // const r = <literal>
    let name = builder.alloc_text(LirNodeKind::Value, "r");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "r");
    builder.node_mut(declarator).unwrap().children = vec![name, literal];
    let decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    builder.node_mut(decl).unwrap().children = vec![declarator];
    // delete r.b;  => Value(None) -> Value("delete") -> Value("b") -> Value("r")
    let del_base = builder.alloc_text(LirNodeKind::Value, "r");
    let del_member = builder.alloc_text(LirNodeKind::Value, "b");
    builder.node_mut(del_member).unwrap().children = vec![del_base];
    let del_unary = builder.alloc_text(LirNodeKind::Value, "delete");
    builder.node_mut(del_unary).unwrap().children = vec![del_member];
    let del_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(del_stmt).unwrap().children = vec![del_unary];
    // r.b = 4;  => Value(None) -> Value("=") -> [member(b->r), Literal(4)]
    let st_base = builder.alloc_text(LirNodeKind::Value, "r");
    let st_member = builder.alloc_text(LirNodeKind::Value, "b");
    builder.node_mut(st_member).unwrap().children = vec![st_base];
    let four = builder.alloc_text(LirNodeKind::Literal, "4");
    let assign = builder.alloc_text(LirNodeKind::Value, "=");
    builder.node_mut(assign).unwrap().children = vec![st_member, four];
    let st_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(st_stmt).unwrap().children = vec![assign];
    // bare-statement enumeration calls
    let keys_call = build_enum_call(&mut builder, "keys");
    let keys_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(keys_stmt).unwrap().children = vec![keys_call];
    let values_call = build_enum_call(&mut builder, "values");
    let values_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(values_stmt).unwrap().children = vec![values_call];
    builder.node_mut(root).unwrap().children =
        vec![decl, del_stmt, st_stmt, keys_stmt, values_stmt];
    (
        LirProgram { root, nodes: builder.into_nodes() },
        del_unary,
        keys_call,
        values_call,
    )
}
```

(If a builder call above mismatches the real `LirBuilder` API — e.g. `alloc` vs `alloc_text` arity — fix the helper against `crates/kali_lir/src`, keeping the SHAPES identical to the probe transcript in the plan's Architecture note. The three tests:)

```rust
#[test]
fn timeline_folds_delete_then_reinsert_to_node_order() {
    // keys => ["a", "c", "b"] (quoted canonical text after Task 4:
    // "\"a\"", "\"c\"", "\"b\""); values => ["1", "3", "4"].
}

#[test]
fn stale_fold_is_dead_mutated_binding_outside_the_lane_does_not_fold() {
    // Same program but the delete statement is nested inside a Branch node
    // (kind: LirNodeKind::Branch wrapping del_stmt). Assert the Object.keys
    // call is NOT folded (still LirNodeKind::Call) — no stale constants —
    // and the delete node is NOT erased (still Value("delete")).
}

#[test]
fn consumed_delete_statements_are_erased_to_empty_blocks() {
    // After the straight-line fold: the delete unary node's slot in
    // program.nodes is now LirNodeKind::Block with no children and no text.
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_optimize timeline`
Expected: test 1 FAILS with stale `["\"a\"", "\"b\"", "\"c\""]` / `["1","2","3"]` (today's flat fold ignores mutations); test 2 FAILS the "not folded" assert (today it folds stale); test 3 FAILS (no erasure exists).

- [ ] **Step 3: Implement the timeline**

In `object_fold.rs`, add after `collect_constant_bindings_into`:

```rust
    /// Names that are the base of any member store (`x.k = v`) or member
    /// delete (`delete x.k`) anywhere in the program. Name-based and
    /// shadowing-blind BY DESIGN: a shadowed name over-approximates to
    /// "mutated", which only ever DISABLES folding (fail-closed direction).
    pub(crate) fn collect_mutated_binding_names(&self, program: &LirProgram) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for node in &program.nodes {
            let is_store = node.kind == LirNodeKind::Value
                && node.text.as_deref() == Some("=")
                && node.children.len() == 2;
            let is_delete = node.kind == LirNodeKind::Value
                && node.text.as_deref() == Some("delete")
                && node.children.len() == 1;
            if !is_store && !is_delete {
                continue;
            }
            let member = node.children[0];
            if let Some((base, _key)) = self.dot_member_base_and_key(program, member) {
                names.insert(base);
            }
        }
        names
    }

    /// `x.k` dot-member: node text = key, exactly one child = bare
    /// identifier base. Computed access (`x[expr]`, 2 children) returns
    /// None — outside the timeline lane.
    fn dot_member_base_and_key(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<(String, String)> {
        let node = program.nodes.get(id.0 as usize)?;
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return None;
        }
        let key = node.text.as_deref()?.to_string();
        let base_node = program.nodes.get(node.children[0].0 as usize)?;
        if base_node.kind != LirNodeKind::Value || !base_node.children.is_empty() {
            return None;
        }
        Some((base_node.text.as_deref()?.to_string(), key))
    }
```

The ordered pass (same file):

```rust
    /// Order-aware enumeration folding (throw-fallout Stage 2, Lane C).
    ///
    /// Non-mutated bindings: exactly the old flat behavior. Mutated
    /// bindings: eligible for the static shape timeline iff EVERY
    /// occurrence of the name is one of (a) its own const declarator,
    /// (b) a top-level straight-line `delete x.k` statement, (c) a
    /// top-level straight-line dot-member store `x.k = v`, (d) the object
    /// argument of an enumeration call inside a top-level straight-line
    /// statement. Anything else (aliasing decl, call argument, member
    /// read, use inside a function/branch/loop) makes the binding
    /// INELIGIBLE: it is excluded from the fold env entirely (killing the
    /// stale fold) and its deletes are left in place for codegen's
    /// default-deny arm — fail-closed, never fold-stale.
    pub(crate) fn fold_object_enumeration_calls_ordered(&self, program: &mut LirProgram) {
        let mutated = self.collect_mutated_binding_names(program);
        let eligible = self.timeline_eligible_bindings(program, &mutated);
        let mut env = BindingEnv::default();
        let root_children = program.nodes[program.root.0 as usize].children.clone();
        for stmt in root_children {
            // 1. const decl? record it (mutated-but-eligible too: the
            //    timeline starts from the literal; mutated-and-INELIGIBLE
            //    names are skipped so they never fold).
            if let Some((name, init)) = self.extract_const_binding(program, stmt) {
                if !mutated.contains(&name) || eligible.contains(&name) {
                    let resolved = self.resolve_constant_binding(program, init, &env).unwrap_or(init);
                    if self.is_specializable_binding(program, resolved) {
                        env.bindings.insert(name, resolved);
                    }
                }
            }
            // 2. timeline mutation? rewrite the binding's snapshot literal.
            if let Some((kind, name, key, value)) = self.as_timeline_mutation(program, stmt) {
                if eligible.contains(&name) {
                    if let Some(current) = env.bindings.get(&name).copied() {
                        if let Some(next) = self.apply_timeline_mutation(program, current, kind, &key, value) {
                            env.bindings.insert(name.clone(), next);
                            if kind == TimelineMutation::Delete {
                                self.erase_statement(program, stmt);
                                continue; // erased: nothing left to fold in it
                            }
                        }
                    }
                }
            }
            // 3. fold enumeration calls inside this statement against the
            //    env as of THIS program point.
            self.fold_object_enumeration_calls(program, stmt, &env);
        }
    }
```

Supporting pieces (write in full):

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineMutation {
    Delete,
    Store,
}
```

- `as_timeline_mutation(program, stmt) -> Option<(TimelineMutation, String, String, Option<LirNodeId>)>`: unwrap statement wrappers (`Value`, empty text, exactly 1 child — the same deref rule `resolve_constant_binding` uses at object_fold.rs:299-305), then match `Value("delete")`/1-child → `dot_member_base_and_key` → `(Delete, base, key, None)`, or `Value("=")`/2-children with a dot-member first child → `(Store, base, key, Some(children[1]))`.
- `apply_timeline_mutation(program, current, kind, key, value) -> Option<LirNodeId>`: `ordered_object_literal_properties(program, current)` → on Delete remove `key` (compare with `trim_matches('"')` on both sides), on Store update the value in place when the key exists (order unchanged) or push `(key, value_node)` at the END (reinsertion order restarts). NOTE: run the update against the **source-order** property list, not the ES-sorted one — rebuild via `push_object_literal(program, properties)` (helpers.rs:~120) and let the enumeration fold do the ES sort at fold time exactly as it does for source literals. Simplest correct recipe: keep a `Vec<(String, LirNodeId)>` extracted WITHOUT the ES sort (walk the literal's `init` children directly, as `ordered_object_literal_properties` does before its sort) so insertion order is preserved for the append.
- `timeline_eligible_bindings(program, mutated) -> BTreeSet<String>`: for each mutated name, scan all nodes for occurrences (a `Value` node, no children, text == name). Track for each occurrence whether it sits at a permitted site by walking the root's top-level statements and, within each, permitting exactly: the declarator's own name child, timeline-mutation base positions, and enumeration-call object arguments (callee matched by the same names `fold_object_enumeration_call` accepts). Occurrences anywhere else — including anywhere beneath a node of kind `Branch`/`Instruction`(function)/inside non-top-level blocks — disqualify. Count-based cross-check: total occurrences of the name in the whole program must equal the number of permitted occurrences found at top level, else ineligible. (The count-based rule is what makes nesting-blindness safe: a use inside a function body is an occurrence that was never permitted, so the binding drops out.)
- `erase_statement(program, stmt)`: `program.nodes[stmt.0 as usize] = LirNode { kind: LirNodeKind::Block, text: None, children: vec![], function_flavor: None };`

Driver rewiring (driver.rs):
- Line 165-166 → `self.fold_object_enumeration_calls_ordered(program);` (delete the now-unused flat collect at this site).
- Line 185-186 (release path re-collect) → collect flat env, then strip mutated names before use:
```rust
                constant_bindings = self.collect_constant_bindings(program, program.root);
                let mutated = self.collect_mutated_binding_names(program);
                constant_bindings.bindings.retain(|name, _| !mutated.contains(name));
                self.fold_object_enumeration_calls_ordered(program);
```
(`optimize_node`/inline.rs at :35/:49 keep receiving `constant_bindings` — now mutation-free, so the inline-time folds can never fold stale.)

- [ ] **Step 4: Run the new tests + the whole optimizer suite**

Run: `cargo test -p kali_optimize`
Expected: the three timeline tests PASS; all pre-existing tests PASS (non-mutated bindings take the identical code path; if a driver_test regresses, diagnose whether it depended on use-before-decl folding — that ordering nicety was never real JS and the test should be re-pinned node-derived, recorded in the triage doc).

- [ ] **Step 5: e2e — Probe 3 goes green on a fresh binary**

```bash
cargo build -p kali_cli
./target/debug/kali run $SCRATCH/probe-3.js; echo "exit=$?"
node $SCRATCH/probe-3.js
```
Expected: both print `ok`, exit 0. (The delete statement was erased by the fold; codegen never sees it.) If kali still warns E8001 on stderr, the erasure didn't fire — check eligibility (`console.log('ok')` uses no binding, but `ks`/`vs` are const-bound folded arrays: their element reads must be in the Lane-D-fixed lane).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_optimize/src
git commit -m "feat(optimize): static shape timeline — delete+reinsert folds at each program point, stale fold eliminated (throw-fallout Stage 2 Lane C)"
```

---

### Task 6: Codegen — `delete` default-denies outside the timeline lane

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs:200-241` (the `"delete"` arm's non-env fallback: E8001 warning + no-op → E5506 error)
- Test: new CLI integration test file `crates/kali_cli/tests/object_delete_gate.rs` (negative pins)

**Interfaces:**
- Consumes: Task 5's erasure (in-lane deletes never reach codegen) and Task 1's parser fix (out-of-lane deletes DO reach codegen).
- Produces: any `delete` surviving to codegen that is not a recognized `process.env` form is a compile-time `Diagnostic::error(e5::FEATURE_UNAVAILABLE …)` — the allowlist choke point.

- [ ] **Step 1: Write the failing negative pins**

Create `crates/kali_cli/tests/object_delete_gate.rs` (copy the temp-dir + `AtomicU64` slug harness pattern from `crates/kali_cli/tests/runtime_string_value_flow.rs:8-40`):

```rust
//! Out-of-lane `delete` is a fail-closed compile error (throw-fallout
//! Stage 2 Lane C). In-lane deletes (straight-line top-level
//! delete+reinsert over a const-bound literal whose only other uses are
//! folded enumerations) are consumed by the optimizer's timeline and never
//! reach codegen — everything else must reject, never silently no-op:
//! before Stage 2 `delete r.b` compiled as a bare member read (the parser
//! swallowed the token), so ANY silent path here re-opens a miscompile.

// helper: run_expect_reject(source) -> (stderr, exit_code)

#[test]
fn delete_inside_a_branch_rejects_e5506() {
    // node: prints c=2. kali: must NOT run — conditional delete is
    // outside the static timeline.
    let (stderr, code) = run_expect_reject(
        "const r = { a: 1, b: 2 };\nif (r.a) { delete r.b; }\nconsole.log('c=' + Object.keys(r).length);",
    );
    assert_ne!(code, 0);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("delete"), "stderr: {stderr}");
}

#[test]
fn delete_then_member_read_rejects_e5506() {
    // node: prints undefined. kali: a runtime member read of a
    // deleted-not-reinserted key is untested surface — fail closed
    // (the read disqualifies the binding from the timeline, so the
    // delete reaches codegen).
    let (stderr, code) = run_expect_reject(
        "const r = { a: 1, b: 2 };\ndelete r.b;\nconsole.log(r.b);",
    );
    assert_ne!(code, 0);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn delete_of_aliased_object_rejects_e5506() {
    // Aliasing + mutation: both names must see the mutation (node
    // semantics) — outside the timeline, fail closed.
    let (stderr, code) = run_expect_reject(
        "const r = { a: 1, b: 2 };\nconst s = r;\ndelete r.b;\nconsole.log(Object.keys(s).length);",
    );
    assert_ne!(code, 0);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test object_delete_gate`
Expected: FAIL — today these compile with the E8001 warning and run (exit 0).

- [ ] **Step 3: Implement the default-deny fallback**

In `crates/kali_codegen/src/emit/operators.rs`, the `"delete"` arm: keep the `process_env_property_key` branch (lines 201-229) EXACTLY as is; replace the fallthrough (lines 231-241, warning + evaluate + drop + zero) with:

```rust
                // Default-deny (throw-fallout Stage 2, Lane C): every
                // in-lane `delete` was consumed and erased by the
                // optimizer's static shape timeline, so a `delete`
                // reaching codegen is outside the provable lane. Reject —
                // the pre-Stage-2 warning+no-op silently preserved stale
                // shapes (and before the parser fix, `delete` was
                // swallowed entirely).
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "the 'delete' operator is only supported on a const-bound fixed-shape object literal in straight-line top-level code whose enumerations are compile-time known; this 'delete' is outside that lane".to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
```

- [ ] **Step 4: Run tests + env-family regression check**

```bash
cargo test -p kali_cli --test object_delete_gate
cargo test -p kali_cli --test node_api_surface 2>&1 | tail -5
```
Expected: gate tests PASS; node_api_surface has no new failures vs stage2-pre.txt (its `delete process.env.*` spellings — core.rs:618-627 — must all hit the env branch, not the new error; if one spelling rejects, extend `process_env_property_key`'s recognized forms to cover it and note the spelling in the triage doc).

- [ ] **Step 5: Verify Probe 3 still green (in-lane path untouched)**

Run: `cargo build -p kali_cli && ./target/debug/kali run $SCRATCH/probe-3.js`
Expected: `ok`, exit 0.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/object_delete_gate.rs
git commit -m "fix(codegen): out-of-lane 'delete' fail-closes with E5506 instead of a silent no-op (throw-fallout Stage 2 Lane C)"
```

---

### Task 7: End-to-end node-parity pins + re-mask guards

**Files:**
- Create: `crates/kali_cli/tests/static_enumeration_stage2.rs`
- Reference fixtures: `crates/kali_cli/tests/runtime_smoke.rs:954-978` (delete-reinsert), `crates/kali_cli/tests/browser_reflect_own_keys.rs:12-198` (quoted-key/ordering core)

**Interfaces:**
- Consumes: all of Tasks 1-6.
- Produces: the stage's load-bearing behavior pins. Task 8's checkpoint expects these green plus the target-family drain.

- [ ] **Step 1: Write the pins (all expectations below were derived from node during planning; re-derive each with `node` before pinning — never trust this document over a fresh node run)**

`crates/kali_cli/tests/static_enumeration_stage2.rs`, same harness pattern as Task 6's file:

```rust
//! Stage 2 (throw-fallout) node-parity pins: static enumeration over
//! quoted keys, ES integer-first ordering, and delete+reinsert timelines.
//! Every expectation is node-derived (fresh `node` run on the same
//! source), NEVER reverse-engineered from kali's output.

#[test]
fn delete_reinsert_enumeration_matches_node() {
    // The runtime_smoke.rs:954 core shape. node: a,c,b / 1,3,4 / ok.
    let stdout = run_expect_ok(
        "const r = { \"a\": 1, \"b\": 2, \"c\": 3 };\n\
         delete r.b;\n\
         r.b = 4;\n\
         const ks = Object.keys(r);\n\
         const es = Object.entries(r);\n\
         const vs = Object.values(r);\n\
         if (ks.length !== 3 || ks[0] !== 'a' || ks[1] !== 'c' || ks[2] !== 'b') throw new Error('keys');\n\
         if (es.length !== 3 || es[2][0] !== 'b' || es[2][1] !== 4) throw new Error('entries');\n\
         if (vs.length !== 3 || vs[0] !== 1 || vs[1] !== 3 || vs[2] !== 4) throw new Error('values');\n\
         console.log('ok');",
    );
    assert_eq!(stdout, "ok\n");
}

#[test]
fn quoted_and_numeric_like_keys_enumerate_in_es_order() {
    // The browser_reflect_own_keys core object. node: 1,2,b,a via keys,
    // for..in, and Reflect.ownKeys alike.
    let stdout = run_expect_ok(
        "const o = { \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 };\n\
         const keys = Object.keys(o);\n\
         if (keys.length !== 4 || keys[0] !== '1' || keys[1] !== '2' || keys[2] !== 'b' || keys[3] !== 'a') throw new Error('keys order');\n\
         const own = Reflect.ownKeys(o);\n\
         if (own.length !== 4 || own[0] !== '1' || own[3] !== 'a') throw new Error('ownKeys order');\n\
         let seen = '';\n\
         for (var k in o) { seen = seen + k; }\n\
         if (seen !== '12ba') throw new Error('for-in order');\n\
         console.log('ok');",
    );
    assert_eq!(stdout, "ok\n");
}

#[test]
fn store_only_mutation_folds_fresh_values() {
    // No delete at all — the timeline must also kill the stale-VALUES fold.
    // node: 2.
    let stdout = run_expect_ok(
        "const r = { a: 1 };\n\
         r.a = 2;\n\
         const vs = Object.values(r);\n\
         if (vs[0] !== 2) throw new Error('stale value');\n\
         console.log(vs[0]);",
    );
    assert_eq!(stdout, "2\n");
}
```

- [ ] **Step 2: The re-mask guards (Invariant 3)**

Same file. Each guard takes a pin's fixture, makes the self-check GENUINELY wrong (a true mismatch, not a tautology — Stage 1 final-review lesson), and asserts the program FAILS (the `throw` fires and traps):

```rust
#[test]
fn re_mask_guard_delete_reinsert_self_check_still_fires() {
    // Deliberately wrong expectation (ks[2] is 'b', not 'c'): the throw
    // MUST fire and the run MUST fail. If this exits 0, a fix re-masked
    // the self-check throw (program Invariant 3 violation).
    let (stdout, code) = run_capture(
        "const r = { \"a\": 1, \"b\": 2, \"c\": 3 };\n\
         delete r.b;\n\
         r.b = 4;\n\
         const ks = Object.keys(r);\n\
         if (ks[2] !== 'c') throw new Error('expected mismatch');\n\
         console.log('MUST NOT PRINT');",
    );
    assert_ne!(code, 0);
    assert!(!stdout.contains("MUST NOT PRINT"), "stdout: {stdout}");
}

#[test]
fn re_mask_guard_es_order_self_check_still_fires() {
    let (stdout, code) = run_capture(
        "const o = { \"b\": 1, \"1\": 4 };\n\
         const keys = Object.keys(o);\n\
         if (keys[0] !== 'b') throw new Error('expected mismatch: ES order puts 1 first');\n\
         console.log('MUST NOT PRINT');",
    );
    assert_ne!(code, 0);
    assert!(!stdout.contains("MUST NOT PRINT"), "stdout: {stdout}");
}
```

- [ ] **Step 3: Node-verify every fixture, then run**

For EACH fixture string above: write it to a temp file, run `node <file>`, confirm the recorded expectation (`ok`/`2`/nonzero-exit) matches node's actual behavior. Then:

Run: `cargo test -p kali_cli --test static_enumeration_stage2`
Expected: all PASS on the current branch build. Any failure is a real Stage-2 defect — fix in the lane task it belongs to before proceeding (do not weaken the pin).

- [ ] **Step 4: Run the target families**

```bash
cargo test -p kali_cli --test runtime_smoke 2>&1 | tail -5
cargo test -p kali_cli --test reflect_own_keys_js_input 2>&1 | tail -5
cargo test -p kali_cli --test browser_reflect_own_keys 2>&1 | tail -5
```
Record per-family pass/fail counts in the triage doc against Task 0's attribution (families attributed to Stage 4/7 blockers are EXPECTED to stay partially red — record exact names; families attributed to Lanes A-D must be green, else diagnose before the checkpoint).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/static_enumeration_stage2.rs docs/superpowers/followups/throw-fallout-stage2-triage.md
git commit -m "test(cli): Stage 2 node-parity pins + re-mask guards for static enumeration (throw-fallout Stage 2)"
```

---

### Task 8: Stage checkpoint — gate, drain snapshot, sweeps

**Files:**
- Modify: `docs/superpowers/followups/throw-fallout-stage0-denominator.md` (append "Stage 2 drain" section)
- Modify: `docs/superpowers/followups/throw-fallout-stage2-triage.md` (final attribution + follow-ups)

- [ ] **Step 1: Hand-mirror sweep**

```bash
grep -rn "SYNTHETIC_FUNCTIONS" crates --include=*.rs
```
Expected: the lower.rs definition + the count_tag_boxing_ops test-side copy, both unchanged this stage (no new synthetics). Also re-grep for new hand-mirrors this stage may have created: `grep -rn "exactly repr_infer's acceptance rule" crates` — `clean_shape`'s doc must still be true (Task 3 changed both sides together).

- [ ] **Step 2: fmt + clippy**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets 2>&1 | tail -5
```
Expected: no diff from fmt (or commit the formatting), clippy clean.

- [ ] **Step 3: Full enumeration + diff vs main**

```bash
git -C /workspace/.worktrees/kali-main rev-parse --short HEAD   # still b48a067d3
cargo test --workspace --no-fail-fast 2>&1 | tee $SCRATCH/stage2-post.log
grep -E '^test .+ \.\.\. FAILED$' $SCRATCH/stage2-post.log | sed 's/^test //; s/ \.\.\. FAILED$//' | sort > $SCRATCH/stage2-post.txt
wc -l $SCRATCH/stage2-post.txt
comm -13 $SCRATCH/stage2-pre.txt $SCRATCH/stage2-post.txt   # newly red — MUST be empty
comm -23 $SCRATCH/stage2-pre.txt $SCRATCH/stage2-post.txt   # drained
```
Pass criteria: post-count < 974 strictly; the newly-red comm is EMPTY (any name there = stage-introduced regression: fix before closing, exactly like Stage 1's `count_tag_boxing_ops` episode). If a newly-red name appears that is also absent from main's failing set (main = 0 failures, so any newly-red name violates the gate), STOP and fix.

- [ ] **Step 4: Drain snapshot + honest attribution**

Append to the denominator doc a "Stage 2 drain" section: pre 974 → post N; the drained-name list (from `comm -23`); for each target-family name still red, its attributed out-of-stage blocker (Stage 3/4/7) per the Task 0/Task 7 evidence. State plainly if the drain fell short of the triage's expected range and why.

- [ ] **Step 5: Follow-ups + commit**

Record in the triage doc any follow-ups opened this stage (candidates seen during planning: numeric literal keys `{ 1: x }` still fail-closed; env-vs-env and bound-alias env.get carried from Stage 1; computed-member delete `delete r["b"]` out of lane; aliased mutation out of lane). Then:

```bash
git add docs/superpowers/followups/throw-fallout-stage0-denominator.md docs/superpowers/followups/throw-fallout-stage2-triage.md
git commit -m "docs(soundness): throw-fallout Stage 2 drain snapshot — static enumeration lanes vs the 974 set"
```

- [ ] **Step 6: Verdict**

Run `cargo test --workspace` (the exact CI command) and record its exit + first failing binary (if any) in the triage doc for continuity. The stage is DONE when Steps 3-4 passed; the branch stays red overall until later stages drain their buckets (expected mid-program state).

---

## Self-Review (performed at planning time)

- **Spec coverage:** Lane A → Task 3; Lane B → Task 2 (+ Task 3 applies it to shapes); Lane C → Tasks 1, 5, 6; Lane D → Task 4; opening triage → Task 0; error-handling posture → Tasks 4 (backstop), 6 (default-deny); re-mask guards + negative pins → Tasks 6, 7; stage gate/drain snapshot → Task 8. Frozen-object 44 names: covered via Task 0 attribution + Task 7 Step 4 family runs (their fixtures exercise the same lanes; no frozen-specific code change is expected — `resolve_constant_binding` already unwraps `Object.freeze`).
- **Known deliberate contingencies (not placeholders):** Task 1 Step 5's types-side pass-through, Task 4's Step 1/7 diagnosis fork, Task 5 Step 4's driver_test re-pin rule — each has an explicit decision rule and a recording requirement.
- **Type consistency:** `property_order_key`/`sort_properties_es_order` names match across Tasks 2/3/5; `fold_object_enumeration_calls_ordered`/`collect_mutated_binding_names` match across Task 5's interface block and driver rewiring; `TimelineMutation` defined in Task 5 where used.
