# Runtime `Array.prototype.join` over string-element arrays — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compile fasta's `fastaRandom` shell verbatim (minus the Spec 4 `for..in` picker): string handles stored into and read from linear-memory arrays, array binding reassignment (`line = new Array(n)`), and a runtime `join(sep)` via a synthetic guest `__join`, with every adjacent silent-wrong shape fail-closed.

**Architecture:** Extend the `ReprTable` element axis (today `I64|F64`, already driving repr-directed element load/store) with `String` plus element-level non-ASCII/taint bits, inferred by Spec 1's string-seed BFS (element STORE edges already carry the string axis; only the READ edge and the table emission are new). Re-key Spec 2's F1 store gate on the proven axis. Codegen stores/reads are already i64-slot-shaped — the work is oracle arms mirrored on BOTH sides (codegen `is_string_valued` family + kali_types predicates, the Spec 2 two-Critical lesson). `join` lowers to a new hand-emitted synthetic `__join(arr, sep) -> i64` (two-pass: sum lengths, ONE `__alloc_global` allocation, `memory.copy` per element/separator — NO host import, NO browser-glue change). Reassignment routes array-alloc RHS through the existing `emit_array_allocation` at the `"="` assignment path. Escape flow needs only a `join`-call classification arm plus regression pins (stores already fail closed; strings are never arena-allocated).

**Tech Stack:** Rust; crates `kali_common` (ReprTable), `kali_types` (inference + gates), `kali_mir` (escape flow), `kali_codegen` (wasm emit via `wasm-encoder`, incl. bulk-memory `MemoryCopy`), `kali_cli` (end-to-end `kali run` tests).

## Global Constraints

- Design doc: `docs/superpowers/specs/2026-07-06-runtime-join-string-arrays-design.md` (Spec 3 of the 6-spec fasta series, incl. the 2026-07-06 planning amendments). Spec 2 context: `docs/superpowers/plans/2026-07-06-substring-runtime.md`.
- **Fail-closed, never fail-open:** a wrong runtime result is worse than a compile error. Any receiver/element/separator/target the analysis cannot prove safe must reject with a diagnostic (`e5::FEATURE_UNAVAILABLE` = 5506, `crates/kali_error/src/_error_codes.rs:99`).
- **A gate relaxation and its codegen lane land in the SAME task.**
- **Both-sides oracle mirroring:** any new expression shape (element read, join call, `&&`/`||`) gets arms on the codegen oracles (`is_string_valued` family, `emit/operators.rs`) AND the kali_types predicates (`expression_is_string_typed` :69, `operand_repr_is_string` :199, `expression_is_runtime_string_value` :422, `expression_is_length_fold_receiver` :358 — all in `crates/kali_types/src/resolve/expression.rs`) in the SAME change, or it fails open. `expression_is_runtime_string_value`'s ternary arm recurses into ITSELF and alone owns the substring member-call fallthrough — preserve that reach.
- **Strings never dangle (NEW standing invariant):** every runtime string allocation goes through `__alloc_global`, NEVER the resettable `__alloc`. `escape_flow.rs:431-438` structurally relies on this ("runtime strings are global-arena host values and never dangle across a reset"); violating it is memory corruption. `__join` must call `__alloc_global` unconditionally.
- **No new host imports.** The 4 hand-mirrored `kali:rt` JS import lists (`kali_runtime/src/browser/harness.rs:198,530`; `kali_cli/src/bin/cmd_build.rs:1553,1817`) stay byte-identical — verify with `git diff` on those files at the end.
- **Base-behavior invariants:** static-fold join (`const a = ["x","y"]; a.join(",")`) stays compile-time; interned-literal `==` byte-identical; float/int element lanes (fannkuch, spectral-norm) byte-identical — CLBG fixture tests are the guardrail; `kali check`-only programs with never-called functions keep compiling (monotone conflicts only).
- Handle encoding: `STRING_HANDLE_TAG (0x8000_0000_0000_0000) | offset << 32 | len` (`kali_codegen/src/lib.rs:66`); `len` is a BYTE count — hence the ASCII gates. Array layout: `[ length: i64 @ +0 ][ elem0: i64|f64 @ +8 ]...`, base address held in an i64 local (`emit_array_allocation_with_len`, `kali_codegen/src/emit/call.rs:2360-2414`).
- Full local gate per task: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir`; final task adds `cargo test --workspace`, `cargo clippy --workspace -- -D warnings` (CI-exact), `cargo fmt --all -- --check`.
- Conventional-commit messages; commit after every task.
- The synthetic top-level function name is `"_start"` in repr_infer, the resolver, and codegen.

## File Structure

| File | Responsibility in this plan |
|---|---|
| `crates/kali_common/src/repr.rs` (+ `repr_tests.rs`) | Task 1: String on the element axis + element non-ASCII/taint quartets |
| `crates/kali_types/src/repr_infer.rs` (+ `repr_infer_tests.rs`) | Task 2: element string axis emission, read-edge lift, conflicts, reassignment merge |
| `crates/kali_mir/src/analysis/escape_flow.rs` (+ `escape_flow_tests.rs`) | Task 3: join-call classification arm + store-taint pins |
| `crates/kali_types/src/resolve/expression.rs` | Tasks 4, 5, 8: F1 re-key, mirror-predicate arms, reassignment + object-literal + `&&`/`||` gates |
| `crates/kali_types/src/static_analysis/array.rs`, `static_analysis/string.rs` | Tasks 7, 8: join gate rewrite, slice gates |
| `crates/kali_codegen/src/emit/operators.rs`, `emit/literal.rs`, `emit/call.rs`, `emit/control_flow.rs`, `emitter.rs`, `lower.rs` | Tasks 4-7: oracle arms, `"="` array-alloc routing, `__join` synthetic + call lane |
| `crates/kali_cli/tests/runtime_string_arrays.rs` (NEW) | Tasks 4, 5: store/read/reassignment e2e |
| `crates/kali_cli/tests/runtime_join.rs` (NEW) | Tasks 7, 8, 9: join e2e, gate pins, capstone |
| `crates/kali_cli/tests/runtime_smoke.rs` | Task 6: census exclusion list gains `__join` |

---

### Task 1: String element repr + element provenance bits on `ReprTable`

**Files:**
- Modify: `crates/kali_common/src/repr.rs`
- Test: `crates/kali_common/src/repr_tests.rs`

**Interfaces:**
- Consumes: existing `set_array_element(func, binding, repr)` (repr.rs:108-114), `array_element(func, binding) -> Repr` (repr.rs:79-84), the scalar non-ASCII/taint quartets (repr.rs:138-181).
- Produces: `set_array_element` accepting `Repr::String` (flips `any_string`); `ReprTable::mark_array_element_non_ascii(func, binding)`, `is_array_element_non_ascii(func, binding) -> bool`, `mark_array_element_concat_tainted(func, binding)`, `is_array_element_concat_tainted(func, binding) -> bool` — exact mirrors of the scalar quartets, keyed `(func, binding)` where `binding` is the ARRAY's name.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_common/src/repr_tests.rs`:

```rust
#[test]
fn repr_table_records_string_element_axis_and_provenance() {
    let mut t = kali_common::ReprTable::default();
    assert_eq!(t.array_element("_start", "a"), kali_common::Repr::I64);
    t.set_array_element("_start", "a", kali_common::Repr::String);
    assert_eq!(t.array_element("_start", "a"), kali_common::Repr::String);
    assert!(t.any_string());

    assert!(!t.is_array_element_non_ascii("_start", "a"));
    t.mark_array_element_non_ascii("_start", "a");
    assert!(t.is_array_element_non_ascii("_start", "a"));
    assert!(!t.is_array_element_non_ascii("_start", "other"));

    assert!(!t.is_array_element_concat_tainted("_start", "a"));
    t.mark_array_element_concat_tainted("_start", "a");
    assert!(t.is_array_element_concat_tainted("_start", "a"));
}
```

(If `any_string` is not a public accessor, check how existing repr tests read the `any_string` bit — there is a Spec 1-era accessor; mirror whatever `repr_tests.rs` already uses. If none exists, assert only the element-axis and quartet behavior and extend `set_array_element` per Step 3 regardless.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p kali_common repr_table_records_string_element_axis_and_provenance`
Expected: FAIL to compile — no method `mark_array_element_non_ascii`.

- [ ] **Step 3: Write minimal implementation**

In `crates/kali_common/src/repr.rs`:

1. Extend `set_array_element` (lines 108-114) so `Repr::String` flips `any_string` exactly the way `Repr::F64` flips `any_float` today:

```rust
    pub fn set_array_element(&mut self, func: &str, binding: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        if repr == Repr::String {
            self.any_string = true;
        }
        self.array_elements
            .insert((func.to_string(), binding.to_string()), repr);
    }
```

(Copy the real current body and add only the `String` branch — do not restructure.)

2. Add two fields next to `string_non_ascii` / `string_concat_tainted` (struct at lines 35-69):

```rust
    /// Arrays whose ELEMENTS may contain non-ASCII string text: `(function, array binding)`.
    array_element_non_ascii: HashSet<(String, String)>,
    /// Arrays whose ELEMENTS may hold runtime-concat-derived strings.
    array_element_concat_tainted: HashSet<(String, String)>,
```

3. Add four methods, exact mirrors of the scalar quartets at repr.rs:138-181 (copy and rename):

```rust
    pub fn mark_array_element_non_ascii(&mut self, func: &str, binding: &str) {
        self.array_element_non_ascii
            .insert((func.to_string(), binding.to_string()));
    }

    pub fn is_array_element_non_ascii(&self, func: &str, binding: &str) -> bool {
        self.array_element_non_ascii
            .contains(&(func.to_string(), binding.to_string()))
    }

    pub fn mark_array_element_concat_tainted(&mut self, func: &str, binding: &str) {
        self.array_element_concat_tainted
            .insert((func.to_string(), binding.to_string()));
    }

    pub fn is_array_element_concat_tainted(&self, func: &str, binding: &str) -> bool {
        self.array_element_concat_tainted
            .contains(&(func.to_string(), binding.to_string()))
    }
```

Do NOT touch `is_empty` (if it enumerates fields, mirror how the Spec 2 non-ASCII fields were handled there — read the current body first).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p kali_common repr`
Expected: PASS (new test plus all existing repr tests).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/repr.rs crates/kali_common/src/repr_tests.rs
git commit -m "feat(repr): String on the array-element axis + element non-ASCII/taint provenance"
```

---

### Task 2: repr_infer — element string axis, read-edge lift, conflicts, reassignment merge

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs`
- Test: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Consumes: Task 1's `set_array_element(.., Repr::String)`, `mark_array_element_non_ascii`, `mark_array_element_concat_tainted`.
- Produces: `infer_reprs` marks `set_array_element(func, name, Repr::String)` for arrays whose every store is string-valued; element non-ASCII/taint bits; a fail-closed shape conflict `"elements of \`{name}\` ... are used as both strings and numbers"` for mixed arrays; element READS flow the string axis (a binding fed by `a[i]` on a string-element array becomes `Repr::String`); `a = new Array(n)` / `a = [..]` / `a = b` assignments merge element nodes. Programs without string-element arrays are byte-identical.

**Background (verified against merged main):**
- Element STORE edges already carry the string axis: `visit_assignment` `a[i] = v` adds `self.add_edge(rn, elem)` at repr_infer.rs:958; `.fill(v)` at :1159; array-literal/`new Array` declarator init at :694-712 (`init_is_array` :728-739 matches BOTH `ArrayExpression` and `new Array`). So the element node is already string-REACHABLE when a string is stored; nothing consumes that fact yet.
- Element READ edges are deliberately float-only: `visit_member` computed read adds `self.add_edge_float_only(elem, result)` at :1024 with the Spec 1 exclusion comment at :1020-1023. Object-FIELD reads are added float-only in `resolve_objects` (:1425-1431) — those stay excluded (fields remain gated).
- `emit_table` registers array bindings and F64 element reprs at :1772-1782; the string/non-ASCII/taint solves are at :1613-1633; scalar conflicts at :1747-1751 via `add_shape_conflict(scope_conflict_message(..))` (:1960-1966); conflicts surface as E5506 through `kali_cli/src/build/compile.rs:650-655`.

- [ ] **Step 1: Write the failing tests**

Add to `crates/kali_types/src/repr_infer_tests.rs` (helper `reprs(src) -> ReprTable` already exists):

```rust
#[test]
fn string_stores_prove_string_element_axis() {
    let t = reprs("function f(s) { const a = new Array(2); a[0] = s.substring(0, 1); a[1] = \"x\"; }\nf(\"hey\");\n");
    assert_eq!(t.array_element("f", "a"), Repr::String);
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn mixed_string_and_number_element_stores_conflict() {
    let t = reprs("const a = new Array(2);\na[0] = \"x\";\na[1] = 1;\n");
    assert!(
        t.shape_conflicts()
            .iter()
            .any(|m| m.contains("elements of `a`")),
        "conflicts: {:?}",
        t.shape_conflicts()
    );
}

#[test]
fn element_read_of_string_element_array_is_string() {
    let t = reprs("const a = new Array(1);\na[0] = \"x\";\nlet s = a[0];\n");
    assert_eq!(t.scalar("_start", "s"), Repr::String);
}

#[test]
fn non_ascii_element_store_marks_element_non_ascii() {
    let t = reprs("const a = new Array(1);\na[0] = \"héllo\";\n");
    assert!(t.is_array_element_non_ascii("_start", "a"));
}

#[test]
fn concat_store_marks_element_tainted_but_literal_store_does_not() {
    let t = reprs("function f(s) { const a = new Array(1); a[0] = s + \"y\"; }\nf(\"x\");\nconst b = new Array(1);\nb[0] = \"z\";\n");
    assert!(t.is_array_element_concat_tainted("f", "a"));
    assert!(!t.is_array_element_concat_tainted("_start", "b"));
}

#[test]
fn array_alloc_reassignment_merges_element_axes() {
    let t = reprs("function f(n) { let a = new Array(60); if (n < 60) { a = new Array(n); } a[0] = \"x\"; }\nf(3);\n");
    assert_eq!(t.array_element("f", "a"), Repr::String);
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn string_element_array_flows_through_param() {
    let t = reprs("function g(q) { q[0] = \"x\"; }\nfunction f() { const a = new Array(1); g(a); let s = a[0]; }\nf();\n");
    assert_eq!(t.array_element("f", "a"), Repr::String);
}
```

(Adjust `Repr` import path to match the file's existing `use` lines. If the param-flow test's expectation doesn't match the existing interprocedural array-param union at :1268-1278 — e.g. the String lands on `g`'s param entry instead — assert what the union actually produces for the CALLER binding and note it; the load-bearing requirement is that a store through a param alias cannot leave the caller's element axis silently I64 while reads treat it as string. If the mechanism cannot prove it, the correct fail-closed outcome is a conflict or the store gate rejecting in Task 4 — pin whichever holds, and record the choice in the task report.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_types repr_infer`
Expected: the seven new tests FAIL (`array_element` returns `I64`, no conflict message, `s` is `I64`); all existing tests still pass.

- [ ] **Step 3: Implement — element store source tracking + table emission**

In `crates/kali_types/src/repr_infer.rs`:

1. Add a field next to `runtime_string_nodes` (~line 92):

```rust
    /// Element-store edges `(element node, stored-value node)` — one entry per
    /// `a[i] = v` / `.fill(v)` / array-literal-init element. Consulted at
    /// emit_table time to fail-close arrays mixing string and non-string
    /// stores (the element node itself unions both axes, so reachability
    /// alone cannot see the mix).
    element_store_sources: Vec<(usize, usize)>,
```

2. Populate it at the three store sites (each already has the element node and value node in scope):
   - `visit_assignment` `a[i] = v` (:949-963): after `self.add_edge(rn, elem);` push `self.element_store_sources.push((elem, rn));`
   - `.fill(v)` (:1147-1165): after `self.add_edge(vnode, elem);` push `(elem, vnode)`.
   - `visit_declarator_init` array-literal elements (:694-712): push `(elem, <element value node>)` per element, next to the existing per-element `add_edge`.

3. In `emit_table`, the string/non-ASCII/taint solves already exist (:1613-1633: `string_adj`, `string_reach = solve_reach(..string_seeds..)`, `tainted`, `non_ascii`). Extend the array-binding emission loop at :1772-1782. It currently does (approximately) "for each `(func, name)` with an element node: `set_array_binding`; if float-reachable `set_array_element(F64)`". Add the string decision, keeping F64 priority questions OUT of it (a float+string mix is a conflict, not a priority choice):

```rust
    let elem_root = self.uf.find(elem_node);
    let elem_string = string_reach[elem_root];
    if elem_string {
        let mixed_store = self
            .element_store_sources
            .iter()
            .any(|(e, s)| self.uf.find(*e) == elem_root && !string_reach[self.uf.find(*s)]);
        if mixed_store || float_reach[elem_root] {
            table.add_shape_conflict(element_conflict_message(&func, &name));
        } else {
            table.set_array_element(&func, &name, Repr::String);
            if non_ascii[elem_root] {
                table.mark_array_element_non_ascii(&func, &name);
            }
            if tainted[elem_root] {
                table.mark_array_element_concat_tainted(&func, &name);
            }
        }
    }
```

Fit this INTO the existing loop's variable names (read the real loop first: the float decision reads some `float_reach`-equivalent — reuse exactly what it reads; do not build a second solve). Add the message builder next to `scope_conflict_message` (:1960-1966), same shape:

```rust
fn element_conflict_message(func: &str, name: &str) -> String {
    if func == "_start" {
        format!("elements of `{name}` at module scope are used as both strings and numbers")
    } else {
        format!("elements of `{name}` in `{func}` are used as both strings and numbers")
    }
}
```

(Mirror `scope_conflict_message`'s actual phrasing conventions — copy its real body and adjust the words.)

- [ ] **Step 4: Implement — read-edge lift + reassignment merge**

1. **Read edge** — `visit_member` computed array read (:1013-1029): replace `self.add_edge_float_only(elem, result)` at :1024 with `self.add_edge(elem, result)`, and REWRITE the exclusion comment at :1020-1023 to say: element reads now carry the string axis because element STORES are gated (Spec 2 F1, re-keyed in Spec 3) and mixed arrays conflict at emit_table — a string can no longer launder through an element unseen; object-FIELD reads (resolve_objects) remain float-only and gated. Do NOT touch `resolve_objects`.

2. **Reassignment merge** — in `visit_assignment`, the plain identifier-target arm (`a = <expr>`, the non-subscript case). Add, alongside whatever scalar edge it already draws:

```rust
    // `a = new Array(n)` / `a = [..]`: route the RHS through the same
    // element-node path as a declarator init, so reassignment unions the
    // element axes instead of silently dropping the array-ness.
    if self.init_is_array(&assign.right) {
        self.note_array_init(&func, &name, &assign.right);
    } else if let Expression::Identifier(rhs) = &assign.right {
        // `a = b` between arrays: elements of b flow into elements of a.
        if self.binding_has_element_node(&func, rhs.name.as_str()) {
            let src = self.array_elem_node_for(&func, rhs.name.as_str());
            let dst = self.array_elem_node_for(&func, &name);
            self.add_edge(src, dst);
            self.element_store_sources.push((dst, src));
        }
    }
```

To get `note_array_init`, EXTRACT the array-init body of `visit_declarator_init` (:694-712 — the part that creates the element node and wires literal elements / `new Array` size) into a helper `fn note_array_init(&mut self, func: &str, name: &str, init: &Expression)` and call it from BOTH the declarator and the assignment arm (verbatim code motion — behavior of the declarator path must not change). `binding_has_element_node` is a lookup into whatever map `array_elem_node_for` inserts into — add a non-inserting `contains` twin (read `array_elem_node_for` :252-260 first). Adjust names/signatures to the file's actual conventions (`func` may be implicit state — mirror how :949-963 obtains it).

- [ ] **Step 5: Adapt the superseded Spec 1 pin**

`element_read_captor_is_not_string` (repr_infer_tests.rs:360) pins `let a = [1]; let s = a[0]; a[0] = "x";` with `s == I64` — that pin ENCODED Spec 1's read-edge exclusion, which this task deliberately lifts; the shape is now a mixed-store conflict (int literal element + string store). Rewrite the test:

```rust
#[test]
fn mixed_literal_int_and_string_store_is_element_conflict() {
    // Spec 1 pinned s == I64 here via the element-read string-axis exclusion.
    // Spec 3 lifts that exclusion (stores are gated + mixed arrays conflict),
    // so this launder shape now fails closed instead of reading back an int.
    let t = reprs("let a = [1];\nlet s = a[0];\na[0] = \"x\";\n");
    assert!(t
        .shape_conflicts()
        .iter()
        .any(|m| m.contains("elements of `a`")));
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p kali_types`
Expected: PASS — 7 new tests + rewritten pin + all 420-ish existing kali_types tests. Any existing test newly failing means the read-edge lift leaked string-ness into an int/float lane: STOP and investigate before adapting anything (adaptations require the same justification discipline as Step 5).

- [ ] **Step 7: Run the full local gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir`
Expected: exit 0. This task must be behavior-neutral end-to-end: nothing consumes the String element repr yet (the F1 gate still rejects string stores syntactically — re-keying happens in Task 4), so e2e suites cannot change. If a kali_cli test fails, the read-edge lift regressed a live lane — fix, don't adapt.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs
git commit -m "feat(types): String element axis — store-source conflicts, read-edge string lane, reassignment merge"
```

---

### Task 3: escape_flow — join-call classification + store-taint pins

**Files:**
- Modify: `crates/kali_mir/src/analysis/escape_flow.rs`
- Test: `crates/kali_mir/src/analysis/escape_flow_tests.rs`

**Interfaces:**
- Consumes: `classify_value`'s CallExpr arm (escape_flow.rs:477-504, the Spec 2 substring template), `ValueClass::Scalar`, test harness `solution_for(source)` (escape_flow_tests.rs:159-167).
- Produces: `.join(..)` member-call results classify as `ValueClass::Scalar` (global-arena string, mirroring the concat arm at :431-438); pinned regressions for the existing unconditional store-taint (arena_gate.rs:493-517) and for substring aliasing.

**Why Scalar and why this is safe:** `__join` (Task 6) copies every element byte into a fresh `__alloc_global` allocation — the result aliases nothing and can never dangle across an arena reset, exactly like `string_concat` output, which `classify_value` already classifies `Scalar` with that rationale (escape_flow.rs:431-438). Without an explicit arm, `line.join('')` falls into the unknown-callee fail-closed path (heap/poison), needlessly vetoing arenas for every joining function. Types-side gating (Task 7) guarantees only string-element array receivers reach codegen, so a user method named `join` on an object never compiles — same property the substring arm relies on.

- [ ] **Step 1: Write the failing test + the two pins**

Add to `crates/kali_mir/src/analysis/escape_flow_tests.rs`, next to `substring_result_aliases_receiver_for_taint` (:261-269):

```rust
#[test]
fn join_result_does_not_carry_receiver_identity() {
    // let out = q.join("") copies bytes into a fresh global-arena string:
    // returning it must NOT taint the receiver array param — unlike
    // substring, which zero-copy ALIASES its receiver and must taint it
    // (see substring_result_aliases_receiver_for_taint above).
    let solution = solution_for("function f(q) { return q.join(\"\"); }");
    assert!(!solution.param_escapes("f", 0));
}

#[test]
fn string_stored_into_array_element_taints_source() {
    // arr[0] = p publishes p outward through the container: the member-store
    // arm (arena_gate.rs arena_note_assignment) taints the stored value's
    // sources unconditionally. PIN — Spec 3's string-element store lane
    // (Task 4) relies on this staying fail-closed.
    let solution = solution_for("function f(p) { const a = new Array(1); a[0] = p; }");
    assert!(solution.param_escapes("f", 0));
}
```

- [ ] **Step 2: Run tests — verify the join test fails, the pin's status**

Run: `cargo test -p kali_mir escape_flow`
Expected: `join_result_does_not_carry_receiver_identity` FAILS (unknown-call fallback taints `q`). `string_stored_into_array_element_taints_source` is expected to already PASS (it is a pin, not new behavior) — if it FAILS, STOP: the store path is not fail-closed for this shape and the controller must be told before Task 4 opens the store lane.

- [ ] **Step 3: Add the classification arm**

In `crates/kali_codegen`— no. In `crates/kali_mir/src/analysis/escape_flow.rs`, inside `classify_value`'s `HirNodeKind::CallExpr` arm, directly AFTER the substring block (:477-504), mirroring its callee-matching shape exactly:

```rust
                // Runtime `join` COPIES element bytes into a fresh
                // `__alloc_global` string: like `+` concat (BinaryExpr arm
                // above), the result is a global-arena host value that never
                // dangles across an arena reset — effectively scalar here.
                // Types-side gating admits only string-element array
                // receivers, so a user method named `join` never reaches
                // this arm compiled. Contrast substring above, which
                // zero-copy ALIASES its receiver and must carry its nodes.
                if node
                    .children
                    .first()
                    .map(|id| &self.nodes[id.0 as usize])
                    .is_some_and(|callee| {
                        callee.kind == HirNodeKind::MemberExpr
                            && callee.text.as_deref() == Some("join")
                    })
                {
                    return ValueClass::Scalar;
                }
```

- [ ] **Step 4: Check the unknown-call arena veto**

`loop_arena_qualifies` (arena_gate.rs:622-651) vetoes loops containing unknown calls. Find how a compiled `p.substring(0,1)` call avoids (or doesn't avoid) `has_unknown_call` — grep `has_unknown_call` in `crates/kali_mir/src/analysis/` and read the recording site. If member-intrinsic calls like `substring` are exempted by a name list, add `"join"` identically; if substring is NOT exempted (arenas just tolerate the veto), do nothing — a vetoed arena is fail-closed (allocations go global), only a precision loss. Record which case held in the task report.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p kali_mir`
Expected: PASS — both new tests, plus all existing escape/arena tests (especially `substring_result_aliases_receiver_for_taint` and `member_read_carries_base_identity_for_taint` — the new arm must not shadow them).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_mir/src/analysis/escape_flow.rs crates/kali_mir/src/analysis/escape_flow_tests.rs
git commit -m "feat(mir): classify join results as global-arena scalars; pin store-taint fail-closed behavior"
```

### Task 4: String element store + read lanes end-to-end — F1 re-key + oracle arms

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs` (F1 gate :452-463, mirror predicates :69/:199/:358/:422, `reject_unprovable_string_length` :385)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`is_string_valued` :553-611, `is_runtime_concat_string` :623-668)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (extract the dynamic-read recognizer, :879-891 and :913-924)
- Test: `crates/kali_cli/tests/runtime_string_arrays.rs` (NEW), plus predicate-level tests where the crate has them

**Interfaces:**
- Consumes: Task 2's proven element axis (`repr_table.array_element(func, name) == Repr::String`, `is_array_element_non_ascii`, `is_array_element_concat_tainted`, `is_array_binding`).
- Produces: resolver helper `string_element_array_binding(&self, name: &str) -> bool` (true iff the identifier resolves to an array binding with proven String element repr, using the SAME scope-chain/function-key rule as `identifier_repr_is_string` — read that predicate first and reuse its mechanism, do not invent a new scope rule); codegen helper `dynamic_array_read_base(&self, node: &LirNode) -> Option<String>` (base name iff this node is a dynamic array-element read the emitter would route to `emit_dynamic_array_read{_node}`). Tasks 5-9 rely on both.

**Both-sides constraint for this task:** the store gate relaxation, the read-side types predicates, and the codegen oracle arms land here TOGETHER. Relaxing the gate without the oracle arms miscompiles `a[0] + "x"` (handle-as-int); oracle arms without the gate relaxation are dead code.

- [ ] **Step 1: Write the failing e2e tests**

Create `crates/kali_cli/tests/runtime_string_arrays.rs`. Copy the `kali_bin`/`run_source` helpers VERBATIM from `crates/kali_cli/tests/runtime_substring_length.rs:4-27`, changing the temp-dir slug to `kali-strarr-`:

```rust
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-strarr-{}-{}-{}",
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

#[test]
fn string_element_store_and_read_roundtrip() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(2);\n  a[0] = s.substring(0, 2);\n  a[1] = \"!\";\n  console.log(a[0]);\n  console.log(a[0] + a[1]);\n}\nf(\"hey\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "he\nhe!\n");
}

#[test]
fn string_element_read_feeds_length_and_substring() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = s.substring(0, 2);\n  console.log(a[0].length);\n  console.log(a[0].substring(1, 2));\n}\nf(\"hey\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\ne\n");
}

#[test]
fn interned_literal_element_identity_equality_stays_green() {
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nif (a[0] == \"x\") {\n  console.log(7);\n}\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}

#[test]
fn tainted_element_equality_is_rejected() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = s + \"y\";\n  if (a[0] == \"xy\") {\n    console.log(1);\n  }\n}\nf(\"x\");\n",
    );
    assert!(!out.status.success(), "concat-tainted element == must reject");
}

#[test]
fn mixed_element_array_is_rejected() {
    let out = run_source("const a = new Array(2);\na[0] = \"x\";\na[1] = 1;\nconsole.log(a[0]);\n");
    assert!(!out.status.success(), "mixed string/number elements must reject");
}

#[test]
fn object_field_string_store_still_rejected() {
    let out = run_source(
        "function f(s) {\n  const o = { v: 0 };\n  o.v = s;\n}\nf(\"x\");\n",
    );
    assert!(!out.status.success(), "field stores stay gated (arrays only in Spec 3)");
}

#[test]
fn non_ascii_element_length_is_rejected() {
    let out = run_source(
        "const a = new Array(1);\na[0] = \"héllo\";\nconsole.log(a[0].length);\n",
    );
    assert!(!out.status.success(), "byte-len .length on non-ASCII element must reject");
}
```

- [ ] **Step 2: Run tests to verify current state**

Run: `cargo test -p kali_cli --test runtime_string_arrays`
Expected: the three green-lane tests FAIL (E5506 from the F1 gate); the reject tests may PASS already (they assert failure) — that is fine, they are pins.

- [ ] **Step 3: Types side — helper + F1 re-key + mirror arms**

In `crates/kali_types/src/resolve/expression.rs`:

1. Add the helper next to `operand_repr_is_string` (:199), reusing the exact function-key/scope-chain mechanism `identifier_repr_is_string` uses (read it first — Spec 1's locals-first walk with `_start` only for true free refs):

```rust
    /// True iff `name` resolves to a linear-memory array binding whose
    /// element repr is proven String (Spec 3 store/read/join lane).
    pub(crate) fn string_element_array_binding(&self, name: &str) -> bool {
        let func = /* same key resolution as identifier_repr_is_string */;
        self.repr_table.is_array_binding(func, name)
            && self.repr_table.array_element(func, name) == kali_common::Repr::String
    }
```

Add siblings `array_element_non_ascii(&self, name) -> bool` and `array_element_concat_tainted(&self, name) -> bool` delegating to the Task 1 quartets with the same key resolution.

2. Re-key the F1 gate `reject_runtime_string_store` (:452-463). Keep its current body; insert the accept path before the diagnostic push. The subscript-target shape test must MIRROR the one `repr_infer::visit_assignment` uses at repr_infer.rs:949-963 to recognize `a[i] = v` (read it; the AST shape for a computed element store is whatever that site matches — reuse the same pattern, e.g. via a small shared-shape helper if one exists):

```rust
        // Spec 3 lane: element stores into arrays with proven String
        // elements are supported — the read side, oracle arms, and mixed
        // arrays' conflicts make this sound. Fields and everything
        // unproven keep rejecting below.
        if let Some(base_name) = /* computed-subscript base ident of assign.left,
                                    same shape repr_infer visit_assignment matches */ {
            if self.string_element_array_binding(base_name) {
                return;
            }
        }
```

Update the message tail (it currently claims "element and field reads have no string lane yet"): `"storing a runtime string value into this element or field is unavailable in the current direct-runtime path unless the target is an array whose elements are all proven strings; use the later compatibility path"`. The array-literal element gate (:507-516) and `.fill` gate (`static_analysis/string.rs:957-973`) stay UNCHANGED (still reject — no fasta need).

3. Mirror-predicate arms — element reads. In each of `expression_is_string_typed` (:69), `operand_repr_is_string` (:199), and `expression_is_runtime_string_value` (:422): add an arm recognizing a computed member READ `a[i]` whose base identifier satisfies `string_element_array_binding` → `true`. In `expression_is_runtime_string_value`, place the arm so the ternary recursion and substring fallthrough are preserved (add before the final fallthrough, do not restructure existing arms).

4. `.length` gating for element reads — `reject_unprovable_string_length` (:385): its ASCII proof must accept `a[i].length` iff `string_element_array_binding(a) && !array_element_non_ascii(a)`, and reject otherwise (this is what makes `non_ascii_element_length_is_rejected` pass). Read the existing receiver-shape dispatch and add the subscript-receiver case alongside the identifier case.

5. `==`/`!=` taint gating: find where Spec 1 keys equality rejection on `is_string_concat_tainted*` (grep `is_string_concat_tainted` in kali_types) and add the element case: a computed read of `a[i]` is treated as tainted iff `array_element_concat_tainted(a)`.

- [ ] **Step 4: Codegen side — recognizer extraction + oracle arms**

In `crates/kali_codegen`:

1. Extract the dynamic-read recognizer. The dispatch conditions at `emit/control_flow.rs:879-891` (1-child form) and `:913-924` (2-child computed form) decide when a node is routed to `emit_dynamic_array_read{_node}`. Extract those conditions into ONE helper on `FunctionEmitter` and call it from BOTH dispatch sites (so oracle and emitter cannot drift):

```rust
    /// Base binding name iff `node` is a dynamic array-element read this
    /// emitter routes to emit_dynamic_array_read{_node}. Single source of
    /// truth shared by the dispatch sites and the string oracles.
    pub(crate) fn dynamic_array_read_base(&self, node: &LirNode) -> Option<String>
```

The extraction is verbatim code motion: the dispatch sites' behavior must be byte-identical (existing codegen tests are the guardrail).

2. `is_string_valued` (`emit/operators.rs:553-611`): after the existing prelude (unwrap_transparent + resolve_bound_node), before the `match`, add:

```rust
        if let Some(base) = self.dynamic_array_read_base(node) {
            return self.array_elem_repr(&base) == kali_common::Repr::String;
        }
```

(`array_elem_repr` is the existing per-binding element-repr lookup the element load/store paths already use — `emit/literal.rs:344`, `emit/call.rs:2707-2730`. Reuse it, mirroring its function-key convention.)

3. `is_runtime_concat_string` (:623-668): same placement, keying on the element taint bit:

```rust
        if let Some(base) = self.dynamic_array_read_base(node) {
            return self
                .repr_table
                .is_array_element_concat_tainted(&self.function_name, &base);
        }
```

(If bindings can be module-scope, mirror the `_start` fallback `is_string_valued`'s identifier arm uses — same convention, same order.)

4. The element STORE and LOAD emission need NO change: `emit/literal.rs:376-391` already routes `Repr::String` through `I64Store`, and `emit/call.rs:2707-2730` already routes it through `I64Load`. Verify by reading; add a one-line comment at each match arm noting Spec 3 activates the String case.

5. `.length` of an element read: `render_length` (`intrinsics/host.rs:543-551`) already defers when `is_string_valued` — the new early-return in (2) makes `a[0].length` defer to the dynamic handle-mask arm (`control_flow.rs:849-862`) automatically. No change; verify with the e2e test.

- [ ] **Step 5: Run the e2e tests**

Run: `cargo test -p kali_cli --test runtime_string_arrays`
Expected: ALL PASS (3 green + 4 reject).

- [ ] **Step 6: Run the full local gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir`
Expected: exit 0. Existing F1 pins in `runtime_substring_length.rs` assert failure status (not message text) — if any pin asserted the OLD message text verbatim, update the assertion to the new text and say so in the report.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_types/src/resolve/expression.rs crates/kali_codegen/src/emit/operators.rs crates/kali_codegen/src/emit/control_flow.rs crates/kali_cli/tests/runtime_string_arrays.rs
git commit -m "feat(types,codegen): string element store/read lanes — F1 re-keyed on proven element axis, oracle arms both sides"
```

---

### Task 5: Array binding reassignment

**Files:**
- Modify: `crates/kali_codegen/src/emit/literal.rs` (the `"="` local-binding arm, :437-453)
- Modify: `crates/kali_types/src/resolve/expression.rs` (scalar-into-array conflict gate)
- Test: `crates/kali_cli/tests/runtime_string_arrays.rs`

**Interfaces:**
- Consumes: `resolve_array_alloc_call` (`emit/call.rs:2316-2328`), `emit_array_allocation` (:2333-2339), `array_bindings` (`emitter.rs:97`), Task 2's reassignment element-axis merge, Task 4's `string_element_array_binding` key mechanism.
- Produces: `a = new Array(n)` and `a = b` (array-to-array) compile correctly for int, float, and string element arrays; `a = 5` on an array binding rejects E5506.

**Ground truth:** today the `"="` arm at literal.rs:437-453 emits the RHS generically — an array-alloc RHS never reaches `emit_array_allocation`, so `let a = new Array(60); a = new Array(n); a[0]` silently prints 0 (probed on main 745a3ecea). The declarator path (control_flow.rs:596-610) is the model to mirror.

- [ ] **Step 1: Write the failing e2e tests**

Add to `crates/kali_cli/tests/runtime_string_arrays.rs`:

```rust
#[test]
fn array_alloc_reassignment_int_elements() {
    // fastaRandom's partial-last-line shape; silent-wrong 0 on main 745a3ecea.
    let out = run_source(
        "function g(n) {\n  let a = new Array(60);\n  if (n < 60) {\n    a = new Array(n);\n  }\n  for (let i = 0; i < a.length; i = i + 1) {\n    a[i] = i * 10;\n  }\n  console.log(a[1]);\n  console.log(a.length);\n}\ng(3);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n3\n");
}

#[test]
fn array_alloc_reassignment_string_elements() {
    let out = run_source(
        "function g(n, s) {\n  let a = new Array(4);\n  if (n < 4) {\n    a = new Array(n);\n  }\n  for (let i = 0; i < a.length; i = i + 1) {\n    a[i] = s.substring(0, 1);\n  }\n  console.log(a[0] + a[1]);\n  console.log(a.length);\n}\ng(2, \"xy\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "xx\n2\n");
}

#[test]
fn array_to_array_binding_copy() {
    let out = run_source(
        "function g() {\n  const b = new Array(2);\n  b[0] = 5;\n  b[1] = 6;\n  let a = new Array(1);\n  a = b;\n  console.log(a[1]);\n  console.log(a.length);\n}\ng();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "6\n2\n");
}

#[test]
fn scalar_reassignment_of_array_binding_is_rejected() {
    let out = run_source(
        "let a = new Array(2);\na[0] = 1;\na = 5;\nconsole.log(a[0]);\n",
    );
    assert!(!out.status.success(), "scalar into array binding must reject, not clobber the handle");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_cli --test runtime_string_arrays`
Expected: the three green tests FAIL (stdout `0\n0\n`-shaped); the reject test FAILS too (compiles silently today).

- [ ] **Step 3: Codegen — route array RHS through the allocation/copy path**

In `crates/kali_codegen/src/emit/literal.rs`, at the TOP of the `"="` arm (:437-453), before the generic `self.emit_node(function, right, true)`:

```rust
                // `a = new Array(n)`: same routing as the declarator path
                // (control_flow.rs:596-610) — the allocation needs a stable
                // handle in the local, and the binding (re)registers as an
                // array so element/length lanes stay routed.
                if let Some(size_arg) = self.resolve_array_alloc_call(right) {
                    let allocated = self.emit_array_allocation(function, size_arg);
                    if !allocated.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                    function.instruction(&Instruction::LocalTee(index));
                    self.array_bindings.insert(name.clone());
                    return true;
                }
                // `a = b` where b is an array binding: the local already
                // holds b's base handle; copy it and register a as array.
                if let Some(rhs_name) = self.bare_identifier_name(right) {
                    if self.array_bindings.contains(&rhs_name) {
                        let rhs = self.emit_node(function, right, true);
                        if !rhs.produced {
                            function.instruction(&Instruction::I64Const(0));
                        }
                        function.instruction(&Instruction::LocalTee(index));
                        self.array_bindings.insert(name.clone());
                        return true;
                    }
                }
```

Notes for fitting: the arm's real variable names (`index`, `name`) and its tee-vs-set / produced-value convention must match the surrounding code — read :437-453 first; if the arm uses `LocalSet` + separate produced handling, mirror that instead of `LocalTee`. `bare_identifier_name` = whatever existing helper resolves a LIR node to a bare identifier's text after `unwrap_transparent` (grep for how `is_string_valued`'s identifier arm gets `node.text`; add a small helper if none exists).

- [ ] **Step 4: Types — scalar-into-array conflict gate**

In `crates/kali_types/src/resolve/expression.rs`, next to `reject_runtime_string_store` and called from the same `AssignmentExpression` dispatch (:543):

```rust
    /// `a = 5` where `a` is an array binding would clobber the base handle
    /// with an integer — later element reads would dereference address 5.
    /// Fail closed. Array-alloc and array-identifier RHS are the supported
    /// reassignment shapes.
    pub(crate) fn reject_array_binding_scalar_reassignment(
        &mut self,
        assign: &AssignmentExpression,
    ) {
        let Expression::Identifier(target) = &assign.left else {
            return;
        };
        let func = /* same key resolution as string_element_array_binding */;
        if !self.repr_table.is_array_binding(func, target.name.as_str()) {
            return;
        }
        let rhs_is_array = matches!(&assign.right, Expression::NewExpression(_))
            /* plus the `Array(n)` CallExpression form and bare identifiers
               that are themselves array bindings — mirror the shapes
               repr_infer::init_is_array (repr_infer.rs:728-739) accepts */;
        if rhs_is_array {
            return;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "reassigning an array binding to a non-array value is unavailable in the current direct-runtime path".to_string(),
        ));
    }
```

Implement `rhs_is_array` by REUSING `init_is_array`'s shape list (repr_infer.rs:728-739) — if that fn is private to repr_infer, replicate its match arms here with a comment cross-referencing it, plus the identifier-that-is-array-binding case. Compound ops (`a += ...` etc.) on array bindings must also reject — extend the guard to non-`"="` operators on array-binding targets.

**Known residuals (adjudicated at design time, record in the report):** (1) `let a; a = new Array(n)` — reads BEFORE the first assignment fall to the old lane; the program is JS-error-shaped (`a[0]` on `undefined` throws in node), not a valid-program miscompile. (2) `let a = 5; a = new Array(2)` then reads after the reassign are correct; reads between init and reassign are likewise JS-error-shaped. Neither blocks; both are strictly-no-worse than main.

- [ ] **Step 5: Run the e2e tests + full local gate**

Run: `cargo test -p kali_cli --test runtime_string_arrays`
Expected: ALL PASS.

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir`
Expected: exit 0 (the CLBG fixtures — fannkuch in particular — prove the declarator lane didn't regress).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/literal.rs crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/runtime_string_arrays.rs
git commit -m "feat(codegen,types): array binding reassignment — alloc/copy routing at '=', scalar-clobber gate"
```

---

### Task 6: Synthetic `__join` guest function

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (SYNTHETIC_FUNCTIONS :37-43, FunctionPlan block after :247-255, signature chain :373-423, local_decls :561-562, dispatch match :611-632, new `emit_join_body` near :2206)
- Modify: `crates/kali_codegen/src/emitter.rs` (accessor next to `substring_fn_index` :278-280)
- Modify: `crates/kali_cli/tests/runtime_smoke.rs` (census exclusion list :799-805)
- Test: existing suites (every compiled module now contains and validates `__join`); Task 7's e2e tests exercise behavior.

**Interfaces:**
- Produces: a guest function `__join(arr: i64, sep: i64) -> i64` in every module, resolvable via `FunctionEmitter::join_fn_index()`. Semantics: `arr` is an array base address (`[len@+0][elem0@+8]...`, each element an i64 string handle), `sep` a string handle. Returns a fresh string handle whose bytes are `elem0 sep elem1 sep ... elemN-1`. Empty array returns `TAG` (offset 0, len 0 — never dereferenced). ALWAYS copies for n≥1 (spec §4: a runtime zero-copy branch would break the fresh-allocation escape invariant). Allocates ONCE via `__alloc_global` — NEVER `__alloc` (Global Constraints: strings never dangle).

**`memory.copy` note:** no repo precedent (verified); wasm bulk-memory is supported by wasmtime, node, and Chromium. `wasm_encoder::Instruction::MemoryCopy { src_mem: 0, dst_mem: 0 }` pops `(dst: i32, src: i32, len: i32)` — push dst first, then src, then len. A zero-length copy is legal (separator `''`). If validation rejects it anywhere in Step 4, fall back to an inline `I32Load8U`/`I32Store8` byte loop and say so in the report.

- [ ] **Step 1: Registry, plan, signature, locals, accessor**

In `crates/kali_codegen/src/lower.rs`:

1. `SYNTHETIC_FUNCTIONS` (:37-43): append `"__join"`.
2. After the `__substring` FunctionPlan push (:247-255), BEFORE `all_functions.extend(function_plans)` (:256):

```rust
    // Synthetic runtime-join `__join(arr: i64, sep: i64) -> i64` (Spec 3):
    // two-pass copy of an all-string-element array into ONE fresh
    // __alloc_global string — sum lengths, allocate, memory.copy each
    // element and separator. NEVER __alloc: runtime strings must not
    // dangle across an arena reset (escape_flow relies on it).
    all_functions.push(FunctionPlan {
        name: "__join".to_string(),
        params: vec!["arr".to_string(), "sep".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
```

3. Signature chain (:373-423), next to the `__substring` arm:

```rust
        } else if function.name == "__join" {
            (
                vec![ValType::I64, ValType::I64],
                vec![ValType::I64],
            )
        }
```

4. `local_decls` (:561-562), next to the `__substring` arm — six i64 temps (locals 2-7):

```rust
        } else if function.name == "__join" {
            local_decls.push((6, ValType::I64));
        }
```

5. Dispatch match (:611-632): add `"__join" => emit_join_body(&mut body, alloc_global_index),` where `alloc_global_index` is fetched like `page_get_index` is today: `let alloc_global_index = function_name_to_index["__alloc_global"];` (add it next to the `page_get_index` line). Do NOT emit a trailing `End` inside the body — the loop appends it at :631.

In `crates/kali_codegen/src/emitter.rs`, next to `substring_fn_index` (:278-280):

```rust
    /// Function index of the synthetic runtime-join helper.
    pub(crate) fn join_fn_index(&self) -> u32 {
        self.functions["__join"]
    }
```

(Mirror the real map name `substring_fn_index` uses.)

In `crates/kali_cli/tests/runtime_smoke.rs` (:799-805): append `"__join"` to the local `SYNTHETIC_FUNCTIONS` census-exclusion list.

- [ ] **Step 2: Hand-emit the body**

In `lower.rs`, next to `emit_substring_body` (:2206). Locals: 0=`arr`, 1=`sep` (params), 2=`n`, 3=`i`, 4=`total`, 5=`out`, 6=`cur`, 7=`h`. Element address = `arr + (i << 3)` with memarg `offset: 8`. Every `I64Load`/`MemoryCopy` address operand is `I32WrapI64`'d (wasm32).

```rust
/// `__join(arr, sep) -> i64`: copy every element string (i64 handles in the
/// array's slots) plus `sep` between them into ONE fresh __alloc_global
/// buffer; return `TAG | out<<32 | total`. Empty array returns bare TAG
/// (offset 0, len 0 — a zero-length handle is never dereferenced).
/// Locals: 0=arr 1=sep (params), 2=n 3=i 4=total 5=out 6=cur 7=h.
fn emit_join_body(func: &mut Function, alloc_global_index: u32) {
    // n = *(arr + 0)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg { offset: 0, align: 3, memory_index: 0 }));
    func.instruction(&Instruction::LocalSet(2));
    // if n == 0 return TAG
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Eqz);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // total = 0; i = 0
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3));
    // pass 1: total += len(elem_i) for each i
    func.instruction(&Instruction::Loop(BlockType::Empty));
    //   h = *(arr + (i<<3) + 8)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg { offset: 8, align: 3, memory_index: 0 }));
    func.instruction(&Instruction::LocalSet(7));
    //   total = total + (h & 0xFFFF_FFFF)
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    //   i += 1; continue while i < n
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::BrIf(0));
    func.instruction(&Instruction::End);
    // total += (sep & 0xFFFF_FFFF) * (n - 1)
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Sub);
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    // out = zext(__alloc_global(wrap((total + 7) & !7)))
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(7));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I64Const(-8)); // !7 as two's-complement
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::Call(alloc_global_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(5));
    // cur = out; i = 0
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalSet(6));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(3));
    // pass 2: copy elements, separator between them
    func.instruction(&Instruction::Loop(BlockType::Empty));
    //   h = *(arr + (i<<3) + 8)
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg { offset: 8, align: 3, memory_index: 0 }));
    func.instruction(&Instruction::LocalSet(7));
    //   memory.copy(dst=cur, src=(h>>32)&0x7FFF_FFFF, len=h&0xFFFF_FFFF)
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::MemoryCopy { src_mem: 0, dst_mem: 0 });
    //   cur += len(h)
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(6));
    //   i += 1
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(3));
    //   if i < n { copy separator; continue }
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Empty));
    //     memory.copy(dst=cur, src=sep off, len=sep len) — zero-len is a legal no-op
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64ShrU);
    func.instruction(&Instruction::I64Const(0x7FFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::MemoryCopy { src_mem: 0, dst_mem: 0 });
    //     cur += sep_len
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(0xFFFF_FFFF));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(6));
    //     continue the loop (br 1: label 0 = this If, label 1 = the Loop)
    func.instruction(&Instruction::Br(1));
    func.instruction(&Instruction::End); // If
    func.instruction(&Instruction::End); // Loop — falls through when i == n
    // TAG | out << 32 | total
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::I64Const(32));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Const(crate::STRING_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64Or);
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Or);
    // NO trailing End — the dispatch loop appends it (lower.rs:631).
}
```

Fitting notes: match the file's real imports (`MemArg`, `BlockType` — mirror `emit_substring_body`'s use). If element ADDRESS loads elsewhere in the crate wrap-then-load with the offset in the memarg the same way, keep the shape identical. Re-derive every stack effect during review — the Spec 2 review stack-walked `emit_substring_body` instruction by instruction and that discipline caught real bugs. Check whether `__substring`'s address computation wraps to i32 before `I64Load` — if the crate convention differs (e.g. addresses kept i64 with a different load form), MIRROR THE CONVENTION, not this listing.

- [ ] **Step 3: Wire nothing else yet**

`__join` is dead code until Task 7 adds the call lane — that is intentional (same staging as Spec 2's Task 3). The synthetic must still VALIDATE in every module.

- [ ] **Step 4: Run the codegen + cli suites (validation check)**

Run: `cargo test -p kali_codegen -p kali_cli`
Expected: PASS — every existing test now compiles a module containing `__join`; wasmtime validates it (including `memory.copy`) on every `kali run` test. A validation error fails loudly here. Also run the browser smoke if configured locally (`mise run browser-smoke`) or note it for the final gate — node executes the same module bytes.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/lower.rs crates/kali_codegen/src/emitter.rs crates/kali_cli/tests/runtime_smoke.rs
git commit -m "feat(codegen): synthetic __join(arr,sep) guest function — two-pass memory.copy into one __alloc_global string"
```

### Task 7: Runtime join end-to-end — types gate rewrite + codegen call lane

**Files:**
- Modify: `crates/kali_types/src/static_analysis/array.rs` (`resolve_array_join_member_call` :791-828)
- Modify: `crates/kali_types/src/resolve/expression.rs` (mirror-predicate join arms)
- Modify: `crates/kali_types/src/repr_infer.rs` (join arm in `visit_call`, mirroring the substring arm :1144) + `repr_infer_tests.rs`
- Modify: `crates/kali_codegen/src/emit/call.rs` (recognizer + emitter + dispatch near :450-452)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`is_string_valued`, `is_runtime_concat_string` join arms)
- Test: `crates/kali_cli/tests/runtime_join.rs` (NEW)

**Interfaces:**
- Consumes: Task 6's `join_fn_index()`; Task 4's `string_element_array_binding` / `array_element_non_ascii` / `expression_repr_is_ascii_string` (resolve/expression.rs:238); `encode_string_handle` (lower.rs:2363-2365); `self.strings.intern(sep)` (`StringPool`, ctx.rs:185-213); `runtime_substring_call_parts` as the recognizer template (call.rs:2485-2510).
- Produces: `runtime_join_call_parts(&self, node: &LirNode) -> Option<(LirNodeId, Option<LirNodeId>)>` and `emit_runtime_join`; the types gate accepting exactly: linear-memory array receiver with proven ASCII String elements + 0/1 args where the separator is a proven-ASCII string (static or runtime). EVERY other join shape rejects E5506 — the silent fall-through dies here.

**Gate/lane same-task discipline:** the resolver stops rejecting runtime receivers and codegen starts emitting the `__join` call in ONE commit.

- [ ] **Step 1: Write the failing e2e tests**

Create `crates/kali_cli/tests/runtime_join.rs` — copy the `kali_bin`/`run_source` helpers from `runtime_string_arrays.rs` verbatim, slug `kali-join-`:

```rust
#[test]
fn runtime_join_empty_default_and_static_separators() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(3);\n  for (let i = 0; i < 3; i = i + 1) {\n    a[i] = s.substring(i, i + 1);\n  }\n  console.log(a.join(\"\"));\n  console.log(a.join(\"-\"));\n  console.log(a.join());\n}\nf(\"xyz\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "xyz\nx-y-z\nx,y,z\n");
}

#[test]
fn runtime_join_runtime_separator_and_concat_consumer() {
    let out = run_source(
        "function g(s, sep) {\n  const a = new Array(2);\n  a[0] = s.substring(0, 1);\n  a[1] = s.substring(1, 2);\n  console.log(a.join(sep));\n  console.log(\"[\" + a.join(\"\") + \"]\");\n}\ng(\"ab\", \"::\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a::b\n[ab]\n");
}

#[test]
fn runtime_join_literal_string_elements() {
    // probe_a from the design investigation: silent 0 on main 745a3ecea.
    let out = run_source(
        "var line = new Array(3);\nfor (var i = 0; i < line.length; i = i + 1) {\n  line[i] = \"x\";\n}\nconsole.log(line.join(\"\"));\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "xxx\n");
}

#[test]
fn runtime_join_zero_length_array_prints_empty_line() {
    let out = run_source(
        "function f() {\n  const a = new Array(0);\n  console.log(a.join(\"-\"));\n}\nf();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "\n");
}

#[test]
fn runtime_join_single_element_array_copies() {
    // The always-copy rule (spec §4): a 1-element join returns a FRESH
    // buffer, never the element handle itself.
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = s.substring(0, 2);\n  console.log(a.join(\"-\"));\n}\nf(\"hey\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "he\n");
}

#[test]
fn join_result_feeds_length_and_substring() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(2);\n  a[0] = s.substring(0, 1);\n  a[1] = s.substring(1, 2);\n  const j = a.join(\"\");\n  console.log(j.length);\n  console.log(j.substring(1, 2));\n}\nf(\"ab\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\nb\n");
}

#[test]
fn static_fold_join_stays_green() {
    let out = run_source("const q = [\"x\", \"y\"];\nconsole.log(q.join(\",\"));\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x,y\n");
}

#[test]
fn join_of_int_element_array_is_rejected() {
    let out = run_source(
        "const a = new Array(2);\na[0] = 1;\na[1] = 2;\nconsole.log(a.join(\",\"));\n",
    );
    assert!(!out.status.success(), "runtime join over number elements must reject");
}

#[test]
fn join_with_non_ascii_element_is_rejected() {
    let out = run_source(
        "const a = new Array(1);\na[0] = \"é\";\nconsole.log(a.join(\"\"));\n",
    );
    assert!(!out.status.success(), "byte-length join over non-ASCII elements must reject");
}

#[test]
fn join_with_unproven_separator_is_rejected() {
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nlet f = 1 / 2;\nconsole.log(a.join(f));\n",
    );
    assert!(!out.status.success(), "non-string separator must reject");
}

#[test]
fn static_receiver_with_variable_separator_is_rejected_not_silent() {
    // probe_b from the design investigation: printed 0 silently on main.
    let out = run_source(
        "var line = [\"a\", \"b\", \"c\"];\nvar sep = \"-\";\nconsole.log(line.join(sep));\n",
    );
    assert!(!out.status.success(), "was silent-wrong 0; must reject now");
}

#[test]
fn join_result_equality_is_rejected() {
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nif (a.join(\"\") == \"x\") {\n  console.log(1);\n}\n",
    );
    assert!(!out.status.success(), "join results are runtime concat — identity == must reject");
}

#[test]
fn ternary_wrapped_join_receiver_is_rejected() {
    let out = run_source(
        "function f(c) {\n  const a = new Array(1);\n  a[0] = \"x\";\n  const b = new Array(1);\n  b[0] = \"y\";\n  console.log((c > 0 ? a : b).join(\"\"));\n}\nf(1);\n",
    );
    assert!(!out.status.success(), "non-identifier receivers hit the fail-closed default");
}
```

- [ ] **Step 2: Run tests to verify the green ones fail**

Run: `cargo test -p kali_cli --test runtime_join`
Expected: green-lane tests FAIL (today: silent `0` output or E5506 depending on shape); `static_fold_join_stays_green` PASSES (pin); reject tests that already reject PASS.

- [ ] **Step 3: Types — rewrite `resolve_array_join_member_call`**

Replace the body at `static_analysis/array.rs:791-828`. Keep the static-fold lane EXACTLY as-is; add the runtime lane; kill the silent early-return:

```rust
    pub(crate) fn resolve_array_join_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };

        if member.property.as_str() != "join" {
            return;
        }

        let supported_arg_count = matches!(expr.args.len(), 0 | 1);

        // Static fold lane (unchanged): literal receiver + static separator.
        if self.is_static_array_iteration_target(&member.object) {
            let has_static_separator = expr
                .args
                .first()
                .is_none_or(|argument| self.resolve_static_string_expression(argument).is_some());
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            if supported_arg_count && has_static_separator {
                return;
            }
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Array.prototype.join is unavailable for static literal-array receivers unless the optional separator is a statically-known string in the current direct-runtime path; use explicit literals or the later compatibility path".to_string(),
            ));
            return;
        }

        // Runtime lane (Spec 3): linear-memory array receiver with proven
        // ASCII String elements; separator absent (default ","), statically
        // known, or a proven-ASCII runtime string.
        self.resolve_expression(&member.object);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }
        if supported_arg_count {
            if let Expression::Identifier(base) = &member.object {
                let name = base.name.as_str();
                if self.string_element_array_binding(name)
                    && !self.array_element_non_ascii(name)
                {
                    let separator_ok = expr.args.first().is_none_or(|argument| {
                        self.resolve_static_string_expression(argument)
                            .map(|s| s.is_ascii())
                            .unwrap_or_else(|| self.expression_repr_is_ascii_string(argument))
                    });
                    if separator_ok {
                        return;
                    }
                }
            }
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "Array.prototype.join is unavailable unless the receiver is a statically-known array literal with a statically-known separator, or a runtime array whose elements are all proven ASCII strings with a proven-ASCII string separator, in the current direct-runtime path".to_string(),
        ));
    }
```

Fitting notes: `expression_repr_is_ascii_string`'s real signature/receiver may need `&mut self` or a different arg shape — read :238 and adapt the call, not the semantics. If `Expression::Identifier`'s field is not `.name`, mirror how `resolve_array_join_member_call`'s neighbors destructure identifiers. Static separators must ALSO be ASCII-checked in the runtime lane (`s.is_ascii()`) because `__join` counts bytes — the static FOLD lane is exempt (it folds in Rust, byte-exactly).

**repr_infer arm (same commit):** a BOUND join result (`const j = a.join('')`) must flow `Repr::String` to `j`. In `crates/kali_types/src/repr_infer.rs`, `visit_call`'s member-method match (:1087-1137) — where the `substring` arm (:1144) seeds the result node as a string source and pushes it into `runtime_string_nodes` (taint) — add a `"join"` arm doing exactly the same: string-seed the fresh result node and push it to `runtime_string_nodes` (a join result is a fresh runtime buffer — identity `==` must reject, same rationale as substring). Also union the receiver's element non-ASCII into the result the way substring propagates receiver non-ASCII — read how the substring arm does it and mirror; if the element bits are only applied at emit_table (not node-level), the Task 7 join GATE's `!array_element_non_ascii` check already makes a non-ASCII-element join unreachable, and no node-level propagation is needed — verify and note which holds. Add a `repr_infer_tests.rs` pin:

```rust
#[test]
fn bound_join_result_is_string_and_tainted() {
    let t = reprs("const a = new Array(1);\na[0] = \"x\";\nconst j = a.join(\"\");\n");
    assert_eq!(t.scalar("_start", "j"), Repr::String);
    assert!(t.is_string_concat_tainted("_start", "j"));
}
```

Mirror predicates (`resolve/expression.rs`): add a join-call arm to `operand_repr_is_string` and `expression_is_runtime_string_value` — a `CallExpression` whose callee is a `MemberExpression` with property `join` and a receiver identifier passing `string_element_array_binding` → `true`. In `expression_is_runtime_string_value`, keep the arm alongside the substring fallthrough (both are "runtime string producers"); do not disturb the ternary self-recursion. Join results count as concat-tainted wherever the equality gate asks (the result is a fresh buffer — interned identity never holds): find the equality-gate taint predicate extended in Task 4 Step 3.5 and make join calls return tainted there too.

- [ ] **Step 4: Codegen — recognizer, emitter, dispatch, oracle arms**

In `crates/kali_codegen/src/emit/call.rs`:

1. Recognizer, next to `runtime_substring_call_parts` (:2485-2510), same template:

```rust
    /// `(receiver, separator)` iff this is a runtime join over a
    /// linear-memory array binding with proven String elements. Literal
    /// receivers are never in `array_bindings`, so the static fold lane
    /// (intrinsics/array.rs resolve_static_array_join_call) stays disjoint.
    pub(crate) fn runtime_join_call_parts(
        &self,
        node: &LirNode,
    ) -> Option<(LirNodeId, Option<LirNodeId>)> {
        if node.kind != LirNodeKind::Call || !(1..=2).contains(&node.children.len()) {
            return None;
        }
        let callee = self.resolve_transparent_callable_node(node.children[0])?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("join") {
            return None;
        }
        let receiver = callee_node.children.first().copied()?;
        let receiver_node = self.node(self.unwrap_transparent(receiver));
        let base = receiver_node.text.as_deref()?;
        if !self.array_bindings.contains(base) {
            return None;
        }
        if self.array_elem_repr(base) != kali_common::Repr::String {
            return None;
        }
        Some((receiver, node.children.get(1).copied()))
    }
```

(Mirror how `runtime_substring_call_parts` unwraps and how `array_elem_repr` is keyed — copy the conventions, keep the checks.)

2. Emitter, next to `emit_runtime_substring` (:2543-2561):

```rust
    fn emit_runtime_join(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        separator: Option<LirNodeId>,
    ) -> EmittedValue {
        let base = self.emit_node(function, receiver, true);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        match separator {
            Some(sep) => {
                let emitted = self.emit_node(function, sep, true);
                if !emitted.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
            }
            None => {
                // JS default separator is ",".
                let (offset, len) = self.strings.intern(",");
                function.instruction(&Instruction::I64Const(
                    crate::lower::encode_string_handle(offset, len),
                ));
            }
        }
        function.instruction(&Instruction::Call(self.join_fn_index()));
        EmittedValue {
            produced: true,
            shape: ValueShape::String,
        }
    }
```

(`encode_string_handle`'s path/visibility: it lives in lower.rs:2363-2365 and is `pub(crate)` — import it the way `emit/call.rs` already references lower items, e.g. how :189 encodes rendered console strings; mirror that call form exactly.)

3. Dispatch inside `emit_call`, next to the substring dispatch (:450-452):

```rust
        if let Some((receiver, separator)) = self.runtime_join_call_parts(node) {
            return self.emit_runtime_join(function, receiver, separator);
        }
```

Place it AFTER whatever static-fold join handling runs in this path (search `resolve_static_array_join_call` consumers) so folds keep winning for literal receivers — though the recognizer's `array_bindings` check already makes them disjoint.

4. Oracle arms in `emit/operators.rs` — same early-return style as Task 4:
   - `is_string_valued`: `if self.runtime_join_call_parts(node).is_some() { return true; }` next to the Task 4 element-read early-return (before the match).
   - `is_runtime_concat_string`: same check → `return true;` (a join result is a fresh runtime buffer; interned identity `==` can never hold — reject).

- [ ] **Step 5: Run the e2e tests**

Run: `cargo test -p kali_cli --test runtime_join`
Expected: ALL PASS (5 green + 6 reject + fold pin).

- [ ] **Step 6: Run the full local gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir`
Expected: exit 0. Watch specifically for kali_types tests that pinned the OLD join-gate message (grep the message string in test files first; update text-pins with justification in the report).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_types/src/static_analysis/array.rs crates/kali_types/src/resolve/expression.rs crates/kali_codegen/src/emit/call.rs crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/runtime_join.rs
git commit -m "feat(types,codegen): runtime Array.prototype.join — gate rewrite kills the silent fall-through, __join call lane"
```

---

### Task 8: Fail-closed gates batch — object literals, `&&`/`||`, slice, literal-array mutation

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs` (`resolve_object_property` :875-878, logical-operator gating, `expression_is_runtime_string_value`)
- Modify: `crates/kali_types/src/static_analysis/string.rs` (`resolve_string_slice_member_call` :360-398), `static_analysis/array.rs` (`resolve_array_slice_member_call` :650-689) as probed
- Test: `crates/kali_cli/tests/runtime_join.rs` (gate pins section)

**Interfaces:**
- Consumes: `expression_is_runtime_string_value` (Task 4-extended), `operand_repr_is_string`, the Spec 1 proven-string operand rejection family.
- Produces: E5506 rejects for the four silent-wrong families from the spec matrix. No new green lanes.

- [ ] **Step 1: Probe the current silent-wrong shapes**

Run each through `cargo run -q -p kali_cli -- run <file>` and record stdout vs node (write them to your scratch dir, not the repo):

1. `function f(s) { const o = { v: s }; } f("x");` — object-literal construction with runtime string.
2. `function f(s) { const a = new Array(1); a[0] = 1 && s; console.log(a[0]); } f("x");` — logical launder into a store (types may now conflict via Task 2 — record what actually happens).
3. `function f(s) { console.log(s.slice(1)); } f("abc");` — string slice on runtime receiver (node: `bc`).
4. `const a = new Array(2); a[0] = 7; const b = a.slice(0); console.log(b[0]);` — array slice on runtime receiver.
5. `function g(k) { const a = [1, 2, 3]; a[k] = 42; console.log(a[k]); } g(1);` — literal-array runtime-index mutation (node: `42`).
6. `function h() { const a = [1, 2, 3]; a[1] = 42; console.log(a[1]); } h();` — literal-array static-index mutation in a function (node: `42`; printed `0` on main).
7. `var a = [1, 2, 3]; a[1] = 42; console.log(a[1]);` — top-level static-index mutation (record whether the fold lane gets this RIGHT today; if it prints `42`, it must KEEP working).

- [ ] **Step 2: Write the failing reject tests**

Add to `crates/kali_cli/tests/runtime_join.rs` (one per silent-wrong probe — adjust expectations to Step 1 findings; every silent-WRONG shape gets a reject pin, every already-correct shape gets a green pin):

```rust
#[test]
fn object_literal_runtime_string_value_is_rejected() {
    let out = run_source("function f(s) {\n  const o = { v: s };\n}\nf(\"x\");\n");
    assert!(!out.status.success(), "object-literal construction store must reject");
}

#[test]
fn logical_launder_into_element_store_is_rejected() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = 1 && s;\n  console.log(a[0]);\n}\nf(\"x\");\n",
    );
    assert!(!out.status.success(), "&&/|| must not launder runtime strings into stores");
}

#[test]
fn runtime_string_slice_is_rejected() {
    let out = run_source("function f(s) {\n  console.log(s.slice(1));\n}\nf(\"abc\");\n");
    assert!(!out.status.success(), "slice on a runtime string receiver must reject (was silent 0)");
}

#[test]
fn runtime_array_slice_is_rejected() {
    let out = run_source(
        "const a = new Array(2);\na[0] = 7;\nconst b = a.slice(0);\nconsole.log(b[0]);\n",
    );
    assert!(!out.status.success(), "slice on a runtime array receiver must reject");
}

#[test]
fn literal_array_runtime_index_mutation_is_rejected() {
    let out = run_source(
        "function g(k) {\n  const a = [1, 2, 3];\n  a[k] = 42;\n  console.log(a[k]);\n}\ng(1);\n",
    );
    assert!(!out.status.success(), "was silent-wrong 0; must reject");
}

#[test]
fn literal_array_function_scope_mutation_is_rejected() {
    let out = run_source(
        "function h() {\n  const a = [1, 2, 3];\n  a[1] = 42;\n  console.log(a[1]);\n}\nh();\n",
    );
    assert!(!out.status.success(), "was silent-wrong 0; must reject");
}
```

- [ ] **Step 3: Implement the gates**

1. **Object literal** — `resolve_object_property` (resolve/expression.rs:875-878), mirroring the array-literal element gate at :507-516 (read it, reuse its phrasing conventions):

```rust
    pub(crate) fn resolve_object_property(&mut self, property: &ObjectProperty) {
        self.resolve_property_name(&property.key);
        self.resolve_expression(&property.value);
        if self.expression_is_runtime_string_value(&property.value) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "a runtime string value is unavailable as an object-literal property value in the current direct-runtime path; use a statically-known string or the later compatibility path".to_string(),
            ));
        }
    }
```

If `ObjectProperty` distinguishes `init`/`get`/`set` kinds, gate only `init` values (getter/setter bodies are function expressions, not stored values — check the AST definition in `kali_ast`). Static string literals pass automatically (`expression_is_runtime_string_value` returns false for fold-receivers — the same reason F1 never fired on `{v: "x"}`-adjacent array shapes).

2. **`&&`/`||`** — two layers, both fail-closed:
   - `expression_is_runtime_string_value` (:422): add a logical-expression arm recursing into BOTH operands (`&&`/`||` yield one operand's value) so store/object gates catch `1 && s`. Match whatever AST node the parser produces for logical ops (the Spec 2 inventory says they lower to `BinaryExpression` — verify in `kali_ast` and match that).
   - Operand-position rejection: locate the Spec 1 proven-string operand rejection for relational/non-`+`/bitwise/unary positions (grep `operand_repr_is_string` call sites in kali_types) and extend its operator set with `"&&"` and `"||"` — a proven-string operand of a logical op rejects (no correct runtime case exists: truthiness of runtime strings is itself gated per Spec 1). If an existing green test uses `s && ...` in a `kali check`-only fixture, prefer the narrower fix (only the `expression_is_runtime_string_value` recursion) and record the narrowing.

3. **Slice** — per Step 1 findings:
   - String receiver: in `resolve_string_slice_member_call` (static_analysis/string.rs:360-398), the non-static receiver currently early-returns silently. Add before that early-return: if `self.operand_repr_is_string(&member.object)` (a PROVEN runtime string receiver) → push E5506 `"String.prototype.slice is unavailable on runtime string receivers in the current direct-runtime path; use substring"`. Unproven receivers keep the early return (they reject elsewhere or are not strings at all).
   - Array receiver: same treatment in `resolve_array_slice_member_call` (static_analysis/array.rs:650-689) keyed on `is_array_binding` via the Task 4 key mechanism — but PRESERVE the `process.argv`/`Deno.args` exception (`is_runtime_args_slice_member`, resolve/member.rs:50-60).

4. **Literal-array unfoldable mutation.** Narrow syntactic gate at the resolver's assignment path (same dispatch as `reject_runtime_string_store`, :543): a computed subscript STORE whose base identifier resolves to a static literal-array binding (the registry behind `is_static_array_iteration_target`, static_analysis/array.rs:107-146) rejects E5506 when (a) the index is not statically foldable, OR (b) the store is inside a non-`_start` function scope. Message: `"mutating a literal array is unavailable in the current direct-runtime path unless the whole access folds statically; use new Array(n) for runtime mutation"`. Probe 7's top-level static-index fold behavior must stay byte-identical — if the 5-crate gate shows over-reject collateral (existing fixtures mutate literal arrays in folded ways), NARROW to runtime-index-only, keep probe 6 as a documented known-silent residual, and record it in the report for the Spec 4 inventory.

- [ ] **Step 4: Run the reject tests + full local gate**

Run: `cargo test -p kali_cli --test runtime_join`
Expected: ALL PASS.

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli -p kali_parser -p kali_mir -p kali_hir`
Expected: exit 0. Object-literal and literal-array gates have the widest blast radius — any existing test tripping them is collateral to adjudicate: static shapes must keep passing (fix the gate), genuinely silent-wrong shapes get their pins updated with justification.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_types/src/resolve/expression.rs crates/kali_types/src/static_analysis/string.rs crates/kali_types/src/static_analysis/array.rs crates/kali_cli/tests/runtime_join.rs
git commit -m "feat(types): fail-closed gates — object-literal string values, logical launder, runtime slice, literal-array mutation"
```

---

### Task 9: fastaRandom-shell capstone + full verification gate

**Files:**
- Test: `crates/kali_cli/tests/runtime_join.rs`

**Interfaces:**
- Consumes: every lane from Tasks 1-8.
- Produces: the Spec 3 success criterion #1 — the fastaRandom shell (stub picker) byte-for-byte vs node.

- [ ] **Step 1: Write the capstone test**

Add to `crates/kali_cli/tests/runtime_join.rs`. The source is the upstream `fastaRandom` shell with ONLY the `for (c in table)` picker swapped for a substring pick (Spec 4 replaces it with the verbatim table walk). The golden below was captured with `node capstone.js` (node 26.4.0) on 2026-07-06 — 4 lines: 60, 60, 60, 20 chars (204 bytes); `n=200` forces the `line = new Array(n)` partial-last-line reassignment.

```rust
#[test]
fn fasta_random_shell_matches_node_byte_for_byte() {
    // Spec 3 capstone: fastaRandom's shell — new Array(60), reassignment to
    // new Array(n), string element stores in a loop, join(''), n -= length —
    // with the Spec 4 for..in picker stubbed by a substring pick. Golden
    // captured from `node` running these exact bytes.
    let src = r#"const ALU = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGACCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAATACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCAGCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGGAGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCCAGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA";
function fastaRandom(n, seed) {
  let line = new Array(60);
  while (n > 0) {
    if (n < line.length) {
      line = new Array(n);
    }
    for (let i = 0; i < line.length; i = i + 1) {
      let k = (i * 7) % seed.length;
      line[i] = seed.substring(k, k + 1);
    }
    console.log(line.join(''));
    n = n - line.length;
  }
}
fastaRandom(200, ALU);
"#;
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let expected = "GCGCTAGACCTTCGAATTATGTCAGGTGCAAGGGCGAGACTGCGCTAGACCTTCGAATTA\nGCGCTAGACCTTCGAATTATGTCAGGTGCAAGGGCGAGACTGCGCTAGACCTTCGAATTA\nGCGCTAGACCTTCGAATTATGTCAGGTGCAAGGGCGAGACTGCGCTAGACCTTCGAATTA\nGCGCTAGACCTTCGAATTAT\n";
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected);
}
```

- [ ] **Step 2: Independently re-verify the golden**

Write the `let src` bytes (between the `r#"` delimiters, exactly) to a scratch file and run `node <file>`; confirm the output equals `expected` byte-for-byte (`diff <(node file.js) golden.txt`). Do NOT trust the plan's golden without this check — if node disagrees, node wins; fix the plan's constant and note it.

- [ ] **Step 3: Run the capstone**

Run: `cargo test -p kali_cli --test runtime_join fasta_random_shell`
Expected: PASS. If it fails, this is the integration point Specs 1-2 kept finding fail-opens at — debug with the systematic-debugging discipline (isolate which lane composes wrong: drop the reassignment, then the join, then the stores, comparing against node each time).

- [ ] **Step 4: Full verification gate**

Run, in order, expecting all to succeed:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
git diff --exit-code crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs
```

The `git diff` proves the 4 hand-mirrored JS import lists were never touched (no-new-host-imports invariant). `cargo test --workspace` covers the CLBG fixture goldens (fannkuch/spectral-norm/n-body/binary-trees/mandelbrot byte-identical) and the census test with `__join` excluded.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/runtime_join.rs
git commit -m "test(cli): fastaRandom-shell capstone — string arrays + reassignment + join('') byte-for-byte vs node"
```

---

## Post-plan notes for the controller

- **Deviation discipline:** as in Specs 1-2, plan snippets yield to the branch's #1 constraint (reject > silent-wrong) and to real file shapes — implementers must flag deviations, controllers adjudicate, and wrong-premise plan text gets fixed in-branch and recorded in the roll-up.
- **Review emphasis:** (1) `emit_join_body` instruction-by-instruction stack walk (Spec 2 discipline); (2) the strings-never-dangle invariant — grep the final diff for any `__alloc`-vs-`__alloc_global` string path; (3) both-sides mirror completeness — every new predicate arm (element read, join call, logical ops) present on codegen AND types sides; (4) A/B probes vs base d-commit for the read-edge lift (int/float lanes byte-identical).
- **Follow-up inventory seeds (for the Spec 4 opener):** object FIELD string stores/reads (kept gated here), `.fill(<string>)` (kept gated), array-literal materialization with runtime strings (kept gated), possible literal-array-mutation gate narrowing residual (Task 8 Step 3.4).


