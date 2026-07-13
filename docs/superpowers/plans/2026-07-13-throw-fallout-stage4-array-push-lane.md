# throw-fallout Stage 4 — Growable runtime array Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce a real growable runtime array (`const x = []`, `x.push(v)`, `x.length`, `x[i]`, `for (const v of x)`, `x.join(sep)`) to green the 16 `array_callback_identity_browser_harness` tests (834 → 818) and close the silent push-no-op miscompile by construction — zero flips.

**Architecture:** A growable array is a tagged handle `ARRAY_TAG | (hdr_ptr<<32)` into an arena-allocated header `[len:i64 @+0][cap:i64 @+8][data_ptr:i64 @+16]` plus a `cap`-slot i64 data buffer at `data_ptr`. The header's separate `data_ptr` keeps the handle stable across a geometric (2×) realloc. A binding is promoted to *growable* by a new `kali_types` recognizer (array-literal binding that is a `.push` receiver and does not escape), mirrored by a codegen oracle; `push`/`length`/index/for-of/`join` get runtime-length lowerings gated on that recognizer. Non-escaping arrays live in the function/loop arena and are reclaimed en masse; escaping growable arrays, `.map()`/`.filter()` result materialization, and non-`push` mutators fail closed (E5506).

**Tech Stack:** Rust workspace; `wasm-encoder` codegen; wasmtime host lane + node `.mjs`/Chromium browser harness lanes; `kali_types` (resolve + repr inference) and `kali_mir` (escape flow) analysis crates.

## Global Constraints

- **Branch:** `soundness-batch1-pra`. Stage base commit: `2f3786b2e` (design-doc commit). Denominator entering this stage: **834**.
- **The one hard gate:** `cargo test --workspace --no-fail-fast` on the branch → capture the FAILED set → diff against the persistent main worktree at `/workspace/.worktrees/kali-main` (built at merge-base, 0 failures). A task/stage is green only when its target tests pass **and** the global failing set strictly shrank **and** zero main-green tests turned red. Plain `cargo test --workspace` fail-fasts at the first failing binary — always enumerate with `--no-fail-fast`; exit-code verdicts use the exact CI command. See memory `ci-gate-vs-poisoned-baseline`.
- **Fix, never flip.** `push` is implemented to match node byte-for-byte; no construct the fixture needs is rejected/trapped to pass. No self-check `throw` may be re-silenced (no re-masking — the push must *actually accumulate*).
- **Both-sides hand-mirror discipline (non-negotiable).** Every new recognizer needs an arm in **both** kali_codegen (emit oracle) and kali_types (resolve/repr predicate), or it fails open. Reviewed per recognizer. Repeated program lesson (`kali-substring-runtime-spec2`, `kali-forin-spec4a`).
- **No silent miscompiles.** Any growable-array construct outside the supported surface emits an E5506 diagnostic (never a silent no-op). The current push-no-op is exactly the silent miscompile this stage kills.
- **Parity is defined by node**, same fixture, byte-for-byte.
- **GC-less** stays true — reclamation is arena/escape only; no tracing/copying GC (`kali-gc-less-invariant`).
- **Design doc:** `docs/superpowers/specs/2026-07-13-throw-fallout-stage4-array-push-lane-design.md`.

### Memory layout (authoritative — every task references this)

```
handle : i64  = ARRAY_TAG | (hdr_ptr << 32)         ; tagged, distinct from STRING_HANDLE_TAG (bit 63)
hdr @ hdr_ptr : [ len:i64 @+0 ][ cap:i64 @+8 ][ data_ptr:i64 @+16 ]   ; 24 bytes
data @ data_ptr : [ v0:i64 @+0 ][ v1:i64 @+8 ] … [ v(cap-1) @+(cap-1)*8 ]   ; cap * 8 bytes
```

- Element slots are i64 **tagged values** (numbers → i64; strings → string handles). Supported element reprs this stage: **I64 and String**. F64/Object elements fail closed.
- Empty `[]`: `len=0`, `cap=INITIAL_CAP` (use `4`). Seeded `[a,b]`: `len=seed_len`, `cap=max(seed_len,4)`, seed copied into `data`.
- `push(v)`: `if len==cap { new_cap = cap*2; new_data = __alloc(new_cap*8); memory.copy(new_data, data_ptr, len*8); *(hdr+16)=new_data; *(hdr+8)=new_cap } ; *(data_ptr + len*8)=v ; *(hdr+0)=len+1`.
- Existing plain (non-growable) allocated arrays keep their inline `[len @+0][elem @+8]` layout (`emit_array_allocation_with_len`, `call.rs:2832`) — untouched. Growable arrays are a **separate, tag-distinguished** lane; do not conflate the two layouts.

### Anchor map (from the two source explorations — cite these, verify line numbers before editing)

**kali_common / repr:**
- `ReprTable` struct + `array_bindings: HashSet<(String,String)>` field `crates/kali_common/src/repr.rs:46`; setter `set_array_binding` `:270-273`; getter `is_array_binding` `:277-280` (the verbatim pattern to mirror for `growable_array_bindings`). `add_shape_conflict` `:375-377` (E5506 channel from repr inference).

**kali_codegen:**
- Array literal materialization: `emit_array_allocation_with_len` `crates/kali_codegen/src/emit/call.rs:2832-2886` (header `I64Store {offset:0}` at `:2874`; `__alloc` call at `:2866`). Value-position aggregate no-op (returns 0): `emit_aggregate_literal` `crates/kali_codegen/src/intrinsics/array.rs:11-43`.
- Allocator API: `alloc_callee_index(&self)->u32` `crates/kali_codegen/src/emitter.rs:345-351` (arena vs global fail-closed); call pattern `call.rs:2854-2867`. `alloc_global_fn_index` `emitter.rs:359-361`.
- `.join` runtime synthetic `emit_join_body` `crates/kali_codegen/src/lower.rs:4092-4267` (runtime len via `I64Load{offset:0}` at `:4093-4101`; elem `I64Load{offset:8}`; **String-only renderer**, `memory.copy` at `:4196-4212`). Dispatch `emit_runtime_join` `call.rs:3097-3141`; recognizer `runtime_join_call_parts` `call.rs:2992-3014`; `emit_call` arm `call.rs:783`. Synthetic registration `lower.rs:39-45`, global wiring `lower.rs:908-917`. Static (int-rendering) join fold `intrinsics/string.rs:1080-1106`.
- for-of lowering: `emit_for_of_array_iteration` `crates/kali_codegen/src/intrinsics/array.rs:1128-1545` — **compile-time static unroll**, length `array.children.len()` at `:1501`, per-element re-emit at `:1518-1527`; E5506 reject at `:1489-1499`. Runtime `.length` read template (the pattern to reuse for runtime len): `crates/kali_codegen/src/emit/control_flow.rs:1266-1308` (base via `emit_array_base_address`, `I64Load{offset:0}`). Runtime element read: `emit_dynamic_array_read` `call.rs:3287-3312`, `emit_array_element_address_node` `call.rs:3339`.
- Element write: `crates/kali_codegen/src/emit/literal.rs:382-447` (gated on `array_bindings.contains(base_name)`; store `offset:8` per `array_elem_repr`).
- Codegen oracle set: `array_bindings: HashSet<String>` `emitter.rs:102` (params `emitter.rs:235-240`; declarators `collect_array_binding_names` `lower.rs:2756-2812`); `dynamic_array_read_base` `control_flow.rs:993-1019`; `array_elem_repr` `emitter.rs:332-334`.
- Tag: `STRING_HANDLE_TAG = 0x8000_0000_0000_0000` `crates/kali_codegen/src/lib.rs:76` (only bit 63 claimed); handle encode `encode_string_handle` `lower.rs:4355-4357`. Arrays/objects currently **untagged** raw base pointers.
- E5506 emit idiom: `self.diagnostics.push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, msg))` + `Unreachable` (stmt) / `I64Const(0)` (value) — example `array.rs:1489-1499`.

**kali_types / kali_mir:**
- Array binding tracking: `scope.rs:44` `array_literal_bindings`, `scope.rs:62` `runtime_array_bindings`; declarator recording `resolve/mod.rs:806-816` (`ArrayExpression` → `array_literal_bindings`), `:823-833` (`runtime_array_bindings`). Element repr inference `repr_infer.rs` — `note_array_init` `:1009+`, `array_elem_node_for` `:433`, emission `set_array_binding` `:2518/:2712`, `set_array_element` `:2535`.
- Both-sides mirror predicates: `is_structural_runtime_array(&self,name)->bool` `crates/kali_types/src/resolve/expression.rs:338-366` (mirrors codegen `array_bindings`); `resolve_array_literal_binding_name` `crates/kali_types/src/static_analysis/array.rs:430-441`; `register_runtime_array_binding` `expression.rs:652-670`. **This is where `is_growable_array_binding` lives.**
- `.push` recognition: **ABSENT** — no `"push"` arm in the `repr_infer.rs` method match (arms only at `:1640 toFixed`, `:1650 substring`, `:1665 join`, `:1702 fill`). Add a `"push" =>` arm at `~:1665` + a receiver-recording predicate near `is_structural_runtime_array`.
- `.join` validation: `resolve_array_join_member_call` `static_analysis/array.rs:808-913` (runtime lane gates `string_element_array_binding` `:873`, `is_structural_runtime_array` `:891`; reject at `:907-910`).
- for-of gate: `resolve/mod.rs:577-608` — `left_is_supported && is_static_array_iteration_target(right)` else `Diagnostic::error(e5::FEATURE_UNAVAILABLE, "…array iteration lowering is unavailable unless the iterable is a literal array…")` at `:589-598`.
- Escape query (kali_mir, NOT kali_types): `binding_escapes(&self, owner, name)->bool` `crates/kali_mir/src/analysis/escape_flow.rs:314-319`; `function_binding_escapes(...)` `crates/kali_mir/src/analysis/mod.rs:72-84`.
- E5506 from repr inference: `table.add_shape_conflict(msg)` (e.g. `repr_infer.rs:2533`); from resolve pass: `self.diagnostics.push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, …))`.

---

## File map

| File | Responsibility | Tasks |
|---|---|---|
| `docs/superpowers/followups/throw-fallout-stage4-triage.md` | Target-set enumeration + empirical current-behavior pins | 1, 7 |
| `crates/kali_common/src/repr.rs` | `growable_array_bindings` axis + set/is accessors | 2 |
| `crates/kali_types/src/resolve/expression.rs` | `is_growable_array_binding` predicate + push-receiver recording | 2 |
| `crates/kali_types/src/repr_infer.rs` | `"push"` method arm (element-node wiring + growable mark) | 2 |
| `crates/kali_types/src/resolve/mod.rs` | growable promotion at declarator; for-of gate widening; escape/reassign reject | 2, 4, 6 |
| `crates/kali_types/src/static_analysis/array.rs` | `.join` runtime-lane admission for growable receiver | 5 |
| `crates/kali_codegen/src/lib.rs` | `ARRAY_HANDLE_TAG` constant | 2 |
| `crates/kali_codegen/src/emitter.rs` | `growable_array_bindings` oracle set + accessor | 2 |
| `crates/kali_codegen/src/emit/call.rs` | growable alloc/push/index emit + `emit_call` `.push` arm | 2, 3 |
| `crates/kali_codegen/src/emit/literal.rs` | growable `const x=[]`/`[seed]` declarator materialization | 2 |
| `crates/kali_codegen/src/intrinsics/array.rs` | for-of over growable (runtime loop) | 4 |
| `crates/kali_codegen/src/lower.rs` | `__join_growable` synthetic + registration/wiring | 5 |
| `crates/kali_cli/tests/array_callback_identity_browser_harness.rs` | the 16 target tests (assert only; do not weaken) | 7 |

---

## Task 1: Stage-4 triage — enumerate the exact target set and pin the current behavior empirically

**No code changes. Deliverable: the triage doc + repro transcripts.** Per the program's twice-learned lesson (Stage-1/2/3 forecasts each falsified), no fix is written against an assumed failure mode — every claim is reproduced on a freshly-built branch binary.

**Files:**
- Create: `docs/superpowers/followups/throw-fallout-stage4-triage.md`
- Scratch: `$SCRATCH/stage4-pre.txt` (branch failing set), `$SCRATCH/stage4-main.txt` (main worktree failing set)

**Interfaces:**
- Produces: the confirmed target-set names, and a "current behavior on the branch binary" table (exact stdout/stderr/exit + diagnostic code) that Tasks 2–6 assert the delta against, plus the escape/browser-lane/host-import facts that bound the design.

- [ ] **Step 1: Verify the main worktree is clean (gate baseline).**

Run: `cd /workspace/.worktrees/kali-main && git log --oneline -1 && cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort > "$SCRATCH/stage4-main.txt" && wc -l "$SCRATCH/stage4-main.txt"`
Expected: 0 lines. If non-zero, STOP — the gate is poisoned (`ci-gate-vs-poisoned-baseline`).

- [ ] **Step 2: Enumerate the branch failing set.**

Run: `cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort > "$SCRATCH/stage4-pre.txt" && wc -l "$SCRATCH/stage4-pre.txt"`
Expected: 834 lines (entering denominator). Record any drift in the triage doc before proceeding.

- [ ] **Step 3: Confirm the target set = the 16 `array_callback_identity_browser_harness` names.**

Run: `grep -E 'array_callback_identity_slices_in_browser_api_surface_with_harness' "$SCRATCH/stage4-pre.txt" | sort` → expect 16 (`{run,test} × {plain,json} × {js,ts,jsx,tsx}`). Also run `grep -E 'array_callback' "$SCRATCH/stage4-pre.txt"` and confirm NO other `array_callback_*` names appear (the rest of the family must stay green). Record both lists.

- [ ] **Step 4: Pin the push-no-op current behavior on a fresh binary.**

Build once: `cargo build -p kali_cli`. Then reproduce each of these with `./target/debug/kali run <file>` and record exact stdout/stderr/exit + diagnostic code:
- `function m(){const o=[];o.push(1);o.push(2);console.log(o.length);}m();` → expect prints `0` (push no-op), exit 0.
- `function m(){const o=[];o.push(1);console.log(o[0]);}m();` → expect `undefined`, exit 0.
- `function m(){const o=[];o.push(1);o.push(2);console.log(o.join(","));}m();` → expect empty line, exit 0.
- The reduced harness (push in a for-of-over-map body + join guard):
  ```js
  function m(){const o=[];for(const x of [1,2].map(v=>v)){o.push(x);}if(o.join(",")!=="1,2")throw new Error("got:"+o.join(","));console.log("ok");}m();
  ```
  → expect the `throw` fires (E4000 unreachable, exit 1) because `o.join(",")` is empty.

Record all four in a "current behavior" table. This is the DELTA Tasks 2–5 must close.

- [ ] **Step 5: Confirm the sub-constructs that already work (must NOT regress).**

Reproduce and record (each expected to already print `1\n2` / correct):
- `for(const x of [1,2].map(v=>v))console.log(x);` — map source.
- `for(const x of [1,2].filter(v=>v))console.log(x);` — filter source.
- `for(const x of Array.from([1,2].filter(v=>v)))console.log(x);` — Array.from source.
- `for(const x of [...[1,2].filter(v=>v)])console.log(x);` — spread source.
- `for(const x of [1,2].flatMap(v=>[v]))console.log(x);` — flatMap source.
- `console.log(\`some:${[0,1].some(v=>v)}\`);console.log(\`every:${[1,0].every(v=>v)}\`);` — some/every on literals.

Record: "all six already green; Stage 4 must not touch the for-of *source* lane, only the loop *body*'s push + read-back."

- [ ] **Step 6: Pin the browser-lane behavior + confirm no new host import is needed.**

Run the reduced harness (Step 4) through the browser lane: `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node ./target/debug/kali --output json run --api browser <file>` and record exact behavior (expect the same push-no-op → guard throws → `success:false`). Then grep the four `kali:rt` browser import lists (`crates/kali_runtime/src/browser/harness.rs` ×2, `crates/kali_cli/src/bin/cmd_build.rs` ×2 — per memory `kali-browser-harness-import-sync`) for any array/`__join` host import: expect NONE (array ops are pure-wasm). Record: "growable-array ops need no new `kali:rt` host import; the 4-list sync hazard is N/A for Stage 4 — RE-CONFIRM at the Task-7 gate if any synthetic ends up needing a host helper."

- [ ] **Step 7: Pin the escape facts for the target fixture.**

Confirm (by reading the fixture + a quick reasoning note, and optionally a `kali_mir` unit probe) that `observed` in `array_callback_identity_browser_harness.rs` is a function-local that is never returned, never stored into an object/array field, and never assigned to an outer binding → `binding_escapes("browserArrayCallbackIdentitySlices","observed")` must be **false**. Record this as the justification that `observed` is arena-eligible (Task 2 relies on it).

- [ ] **Step 8: Write the triage doc and commit.**

Write `docs/superpowers/followups/throw-fallout-stage4-triage.md` capturing: the 16-name target set (Step 3), the "current behavior" delta table (Step 4), the already-green sub-constructs (Step 5), the browser-lane + no-host-import finding (Step 6), and the escape fact (Step 7). Structure it like `throw-fallout-stage3-triage.md`.

```bash
git add docs/superpowers/followups/throw-fallout-stage4-triage.md
git commit -m "docs(soundness): throw-fallout Stage 4 triage — target set + push-no-op current-behavior pins"
```

---

## Task 2: Growable array — recognition, materialization, `push`, `length`, index (the core)

The largest task: it stands up the whole growable primitive end-to-end for scalar (i64) elements — recognition (both sides), header allocation for `const x=[]`/`[seed]`, `push` with geometric growth, `.length`, and `x[i]` read. Its deliverable is a runtime test proving push accumulates and length/index read it back. for-of and join are Tasks 4–5.

**Files:**
- Modify: `crates/kali_common/src/repr.rs:46` (field), `:270-280` (accessors)
- Modify: `crates/kali_codegen/src/lib.rs:76` (new `ARRAY_HANDLE_TAG`)
- Modify: `crates/kali_codegen/src/emitter.rs:102` (oracle set), `:332-351` (accessors)
- Modify: `crates/kali_codegen/src/emit/literal.rs` (declarator materialization), `crates/kali_codegen/src/emit/call.rs:43` (`emit_call` `.push` arm) + new `emit_growable_*` helpers
- Modify: `crates/kali_types/src/resolve/expression.rs:338-366` (`is_growable_array_binding` + push-receiver recording), `crates/kali_types/src/repr_infer.rs:~1665` (`"push"` arm), `crates/kali_types/src/resolve/mod.rs:806-833` (promotion at declarator)
- Test: `crates/kali_cli/tests/growable_array_core.rs` (new)

**Interfaces:**
- Produces (consumed by Tasks 3–6):
  - `ReprTable::set_growable_array_binding(&mut self, func:&str, binding:&str)` / `is_growable_array_binding(&self, func:&str, binding:&str)->bool` (`repr.rs`, verbatim on the `set_array_binding`/`is_array_binding` pattern).
  - codegen `FunctionEmitter::is_growable_array(&self, name:&str)->bool` (mirrors `repr_table.is_growable_array_binding`).
  - kali_types `Resolver::is_growable_array_binding(&self, name:&str)->bool` (`expression.rs`, scope-walk like `is_structural_runtime_array`).
  - `const ARRAY_HANDLE_TAG: u64 = 0x4000_0000_0000_0000;` (`lib.rs`, bit 62 — distinct from `STRING_HANDLE_TAG` bit 63).
  - Emit helpers on `FunctionEmitter`: `emit_growable_alloc(&mut self, function, seed_len, cap)`, `emit_growable_push(&mut self, function, handle_local, value)`, `emit_growable_length(&mut self, function, handle)`, `emit_growable_index_read(&mut self, function, handle, index)` — all operating on the header layout in Global Constraints.
- Consumes: `alloc_callee_index()` (`emitter.rs:345`), `binding_escapes` (`kali_mir escape_flow.rs:314`).

- [ ] **Step 1: Write the failing test (core push/length/index).**

Create `crates/kali_cli/tests/growable_array_core.rs` modeled on `array_callback_identity_map.rs` (kali_bin() helper, tempdir, `run`):
```rust
// fixture source:
//   function main(){const o=[];o.push(1);o.push(2);o.push(3);
//     console.log(o.length); console.log(o[0]); console.log(o[2]);
//     let s=0; for (let i=0;i<o.length;i++){s+=o[i];} console.log(s);}
//   main();
// assert `run` stdout == "3\n1\n3\n6\n"
```
Add a second fixture asserting **growth across the realloc boundary** (INITIAL_CAP=4): push 10 elements in a `for (let i=0;i<10;i++) o.push(i*2);`, then assert `o.length`==10 and `o[9]`==18 and `o[4]`==8 (proves the copy on realloc preserved earlier slots). Include both `js` and `ts` extensions.

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p kali_cli --test growable_array_core 2>&1 | tail -20`
Expected: FAIL — current binary prints `0` for length / `undefined` for index (push no-op), or E5506.

- [ ] **Step 3: Add the `growable_array_bindings` repr axis.**

In `crates/kali_common/src/repr.rs`: add field `growable_array_bindings: HashSet<(String, String)>` next to `array_bindings` (`:46`), init in `Default`, and add `set_growable_array_binding`/`is_growable_array_binding` copied verbatim from `set_array_binding`/`is_array_binding` (`:270-280`). Add a `repr_tests.rs` unit test that a set binding reads back true and an unset one false.

- [ ] **Step 4: Add the kali_types recognizer + push-receiver recording + promotion.**

- In `crates/kali_types/src/resolve/expression.rs` (next to `is_structural_runtime_array` `:338`): add `is_growable_array_binding(&self, name:&str)->bool` scope-walking a new `growable_array_bindings` scope set (add the set to `scope.rs` mirroring `runtime_array_bindings` `:62`), and `register_growable_array_binding(&mut self, name)` (mirror `register_runtime_array_binding` `:652`).
- In `crates/kali_types/src/repr_infer.rs` add a `"push" =>` arm in the method-call match (~`:1665`, beside `"join"`): wire the pushed argument's repr node into the receiver's element node (`array_elem_node_for`), and record the receiver identifier as a push receiver.
- In `crates/kali_types/src/resolve/mod.rs` at the `ArrayExpression` declarator (`:806-816`): after recording `array_literal_bindings`, if the binding is a push receiver in this function AND `!binding_escapes(func, name)` (call into kali_mir; thread the escape solution the same way existing resolve code consults it) AND the binding is not reassigned to a non-array → `register_growable_array_binding(name)` and `repr_table.set_growable_array_binding(func, name)`. If it IS a push receiver but escapes or is reassigned → `add_shape_conflict(format!("growable array `{name}` escapes or is reassigned; unsupported"))` (fail-closed E5506).
- Mirror `set_growable_array_binding` into the repr emission pass so the ReprTable carries it to codegen.

- [ ] **Step 5: Add the codegen oracle + tag + emit helpers.**

- `crates/kali_codegen/src/lib.rs:76`: add `pub(crate) const ARRAY_HANDLE_TAG: u64 = 0x4000_0000_0000_0000;`.
- `crates/kali_codegen/src/emitter.rs`: add `growable_array_bindings: HashSet<String>` field (`:102` beside `array_bindings`), populate it in the emitter ctor from `repr_table.is_growable_array_binding(function_name, name)` for every declarator/param name (mirror `:235-240`), and add `is_growable_array(&self, name)->bool`.
- Add emit helpers (new module `crates/kali_codegen/src/emit/growable.rs`, or in `call.rs`) implementing the Global-Constraints layout with the existing idioms: allocate via `alloc_callee_index()` (pattern `call.rs:2854-2867`); header/data stores/loads via `I64Store`/`I64Load` with `MemArg{offset, align:3, memory_index:0}`; growth via `memory.copy` (pattern from `emit_join_body` `lower.rs:4196-4212`). Handle = `(hdr_ptr as u64 zero-extended) | ARRAY_HANDLE_TAG` via `I64Or`; decode hdr_ptr = `(handle & !ARRAY_HANDLE_TAG)` then `I32WrapI64` (mirror the string decode idiom `lower.rs:3945`).

- [ ] **Step 6: Materialize the declarator + wire push/length/index dispatch.**

- Declarator: in `crates/kali_codegen/src/emit/literal.rs` (or `collect_array_binding_names` `lower.rs:2756-2812`), when a binding `is_growable_array(name)`, lower `const x = []`/`[seed…]` to `emit_growable_alloc` (store the resulting tagged handle into the binding's local) instead of the aggregate no-op (`array.rs:11-43`).
- `.push`: add an `emit_call` arm (`call.rs:43`, symmetric to the `.join` arm `:783`) that, when the receiver `is_growable_array(base)`, calls `emit_growable_push(function, handle_local, value)` — appends with growth, updates the binding local on realloc, returns undefined. Guard: only bare-identifier receivers; anything else → E5506.
- `.length`: in the `.length` member read, branch on `is_growable_array(base)` to load `hdr.len` (`emit_growable_length`) instead of the plain-array length lane (`control_flow.rs:1266-1308`).
- `x[i]` read: in the index-read path (`emit_dynamic_array_read` `call.rs:3287` / `dynamic_array_read_base` `control_flow.rs:993`), branch on `is_growable_array(base)` to read `*( *(hdr+16) + i*8 )` (`emit_growable_index_read`). OOB (`i>=len`) → the runtime `undefined` sentinel (match how the codebase represents `undefined` in value position; if none is cheap here, keep in-range correct and file OOB as a follow-up — no target test indexes OOB).

- [ ] **Step 7: Run the core test; iterate to green.**

Run: `cargo test -p kali_cli --test growable_array_core 2>&1 | tail -20`
Expected: PASS (both the basic and the realloc-boundary fixtures, js + ts).

- [ ] **Step 8: Local regression check + commit.**

Run: `cargo test -p kali_cli --test array_callback_identity_map --test array_callback_identity_filter --test array_callback_reduce --test runtime_string_arrays 2>&1 | grep -E '^test result'` (the adjacent array lanes must stay green). Then:
```bash
git add crates/kali_common/src/repr.rs crates/kali_codegen crates/kali_types crates/kali_cli/tests/growable_array_core.rs
git commit -m "feat(codegen+types): growable runtime array core — recognize/materialize/push/length/index [stage4]"
```

---

## Task 3: `push` of string elements (element-repr String slot)

The fixture pushes integers only, but `push` + `join` of strings is the same lane and the design supports String elements; wiring it now keeps the element-repr axis honest and de-risks Task 5's string join. Small delta on Task 2.

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs` (`"push"` arm element-repr union), `crates/kali_codegen/src/emit/*` growable push/index (repr-directed slot)
- Test: `crates/kali_cli/tests/growable_array_core.rs` (extend)

**Interfaces:**
- Consumes: Task 2's `emit_growable_push`/`emit_growable_index_read`, `array_elem_repr(name)` (`emitter.rs:332`).

- [ ] **Step 1: Write the failing test.**

Extend `growable_array_core.rs`: `function main(){const o=[];o.push("a");o.push("b");console.log(o[0]);console.log(o.length);}main();` → assert stdout `a\n2\n`. (join of strings is Task 5.)

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p kali_cli --test growable_array_core string 2>&1 | tail -20`
Expected: FAIL (string push not yet wired / prints wrong).

- [ ] **Step 3: Wire String element repr through push/index.**

- `repr_infer.rs` `"push"` arm: union the pushed value's repr into the receiver element node so `array_element(func,name)` becomes `Repr::String` when a string is pushed (mirror the `"join"`/`init_is_array` element-node wiring). A mixed I64+String push set → `add_shape_conflict` (fail-closed), mirroring the existing mixed-store rejection `repr_infer.rs:2520-2535`.
- codegen: since slots are i64 tagged values and string handles already fit an i64 slot, `emit_growable_push`/`emit_growable_index_read` store/load the raw i64 regardless of repr; the only repr-sensitivity is at the *use* site (index read feeding `console.log`/`join`), which already dispatches on `array_elem_repr`. Confirm `array_elem_repr(name)` returns `String` for the growable binding and that the index-read result is treated as a string handle by the existing string-value oracle (`is_string_valued` via `dynamic_array_read_base` `control_flow.rs:993-1019` — extend it to also recognize a growable base).

- [ ] **Step 4: Run to verify it passes.**

Run: `cargo test -p kali_cli --test growable_array_core string 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_codegen crates/kali_types crates/kali_cli/tests/growable_array_core.rs
git commit -m "feat(codegen+types): growable array String-element slots (push/index) [stage4]"
```

---

## Task 4: `for (const v of x)` over a growable array — runtime counted loop

Replace the compile-time static unroll with a real runtime loop when the iterable is a growable binding. The for-of *source* lane (map/filter/etc. over literals) is untouched.

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/array.rs:1128-1545` (`emit_for_of_array_iteration`) + the resolve entry `control_flow.rs:958-959`
- Modify: `crates/kali_types/src/resolve/mod.rs:577-608` (for-of gate: admit a growable iterable)
- Test: `crates/kali_cli/tests/growable_array_core.rs` (extend)

**Interfaces:**
- Consumes: Task 2 `is_growable_array` oracle + `emit_growable_length`/`emit_growable_index_read`; runtime `.length`/element-read idioms (`control_flow.rs:1266-1308`, `call.rs:3287`).

- [ ] **Step 1: Write the failing test.**

Extend `growable_array_core.rs`: `function main(){const o=[];o.push(10);o.push(20);o.push(30);const out=[];for(const v of o){out.push(v);}console.log(out.length);for(const v of out){console.log(v);}}main();` → assert stdout `3\n10\n20\n30\n`. (Exercises for-of over a growable array both as source and with a growable sink — the fixture's exact shape.)

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p kali_cli --test growable_array_core for_of 2>&1 | tail -20`
Expected: FAIL — `emit_for_of_array_iteration` rejects the non-literal iterable (E5506 `array.rs:1489-1499`) or unrolls wrong.

- [ ] **Step 3: kali_types — admit a growable iterable in the for-of gate.**

`resolve/mod.rs:589-598`: widen the gate so `is_static_array_iteration_target(right) || is_growable_array_binding(<right identifier>)` passes (bare-identifier growable receiver only). Keep every other iterable rejecting exactly as today.

- [ ] **Step 4: codegen — emit a runtime counted loop for a growable iterable.**

In `emit_for_of_array_iteration` (`array.rs:1128`): before the `resolve_literal_aggregate` static path, branch — if the iterable is a bare identifier with `is_growable_array(name)`, emit a real wasm loop: `i=0; n=emit_growable_length(handle); loop { if i>=n break; v = emit_growable_index_read(handle,i); bind loop var to v (a fresh local, not a compile-time LIR node); emit_node(body); i+=1 }`. Reuse the control-frame/`Block`/`Loop`/`BrIf` scaffolding already in this function for the unrolled path; the loop-var binding must be a **local**, so the body's reads of the loop var resolve to that local (adjust the binding insert at `array.rs:1519` accordingly for the runtime path).

- [ ] **Step 5: Run to verify it passes.**

Run: `cargo test -p kali_cli --test growable_array_core for_of 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 6: Regression check + commit.**

Run: `cargo test -p kali_cli --test array_callback_identity_map --test for_of_array_iteration_spread --test for_of_object_keys_iteration 2>&1 | grep -E '^test result'` (static for-of lanes unchanged).
```bash
git add crates/kali_codegen crates/kali_types crates/kali_cli/tests/growable_array_core.rs
git commit -m "feat(codegen+types): for-of over growable array via runtime counted loop [stage4]"
```

---

## Task 5: `x.join(sep)` over a growable array — runtime-length join (int + string rendering)

Generalize the join to a growable receiver. `emit_join_body` already reads a runtime length header and renders **string** elements; a growable array needs (a) indirection through `data_ptr`, and (b) **int→decimal** rendering for i64-element arrays (the fixture joins integers). Add a `__join_growable` synthetic rather than perturbing the existing inline-layout `__join`.

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs:4092-4267` (new `emit_join_growable_body` beside `emit_join_body`), `:39-45` (register `__join_growable`), `:908-917` (global wiring)
- Modify: `crates/kali_codegen/src/emit/call.rs:2992-3141` (`runtime_join_call_parts` + `emit_runtime_join`: growable receiver → `__join_growable`)
- Modify: `crates/kali_types/src/static_analysis/array.rs:808-913` (`resolve_array_join_member_call`: admit growable receiver, i64 or string element)
- Test: `crates/kali_cli/tests/growable_array_core.rs` (extend)

**Interfaces:**
- Consumes: Task 2 growable layout + `is_growable_array`; the int-rendering reference `static_array_join_element_to_string` (`intrinsics/string.rs:1080-1106`) for the digit-emit sequence; `memory.copy` sep idiom (`lower.rs:4196-4253`).

- [ ] **Step 1: Write the failing test.**

Extend `growable_array_core.rs`: two fixtures —
- int join: `function m(){const o=[];o.push(1);o.push(2);o.push(3);console.log(o.join(","));console.log(o.join("\n"));}m();` → assert `1,2,3\n1\n2\n3\n`.
- string join: `function m(){const o=[];o.push("a");o.push("b");console.log(o.join(","));}m();` → assert `a,b\n`.

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p kali_cli --test growable_array_core join 2>&1 | tail -20`
Expected: FAIL — `runtime_join_call_parts` requires `array_elem_repr==String` and a non-growable structural array (`call.rs:3007-3011`); an i64 growable receiver is rejected.

- [ ] **Step 3: Add `emit_join_growable_body`.**

Copy `emit_join_body` (`lower.rs:4092-4267`) to `emit_join_growable_body`. Changes: (a) `n = *(hdr+0)`, `data = *(hdr+16)`, element `h = *(data + (i<<3))` (indirect, no `+8` inline offset); (b) render each slot by the array's element repr — for `Repr::String` reuse the existing `memory.copy` byte path; for `Repr::I64` emit the int→decimal digit sequence adapted from `static_array_join_element_to_string` (`string.rs:1098`) into the output buffer. Pass the element repr into the synthetic (either two synthetics `__join_growable_i64`/`__join_growable_str`, or a repr flag arg — prefer two synthetics for signature simplicity, registered at `lower.rs:39-45`, wired at `:908-917`).

- [ ] **Step 4: Dispatch growable receivers to the new synthetic.**

`call.rs`: extend `runtime_join_call_parts` (`:2992`) to accept a growable receiver (`is_growable_array(base)`), and `emit_runtime_join` (`:3097`) to pick `join_growable_i64_fn_index()`/`join_growable_str_fn_index()` by `array_elem_repr(base)`. Add the corresponding `*_fn_index` accessors on the emitter (mirror `join_arena_fn_index` `emitter.rs:390`).

- [ ] **Step 5: kali_types — admit the growable join receiver.**

`static_analysis/array.rs:808-913` runtime lane: add `is_growable_array_binding(name)` as an accepted receiver alongside `is_structural_runtime_array` (`:891`), for both I64 and String element reprs (relax the `string_element_array_binding` gate `:873` for growable i64 arrays). Keep the non-ASCII and literal-binding rejects.

- [ ] **Step 6: Run to verify it passes.**

Run: `cargo test -p kali_cli --test growable_array_core join 2>&1 | tail -20`
Expected: PASS (int and string).

- [ ] **Step 7: Regression check + commit.**

Run: `cargo test -p kali_cli --test runtime_string_arrays --test runtime_join 2>&1 | grep -E '^test result'` (existing join lanes unchanged; adjust the second `--test` name to the actual join test binary found via `ls crates/kali_cli/tests | grep join`).
```bash
git add crates/kali_codegen crates/kali_types crates/kali_cli/tests/growable_array_core.rs
git commit -m "feat(codegen+types): __join_growable — runtime-length join over growable array (int+string) [stage4]"
```

---

## Task 6: Fail-closed the out-of-scope shapes (no silent miscompile)

Pin and, where needed, close the honest E5506 rejects for constructs the stage does NOT support, so none of them silently miscompiles. Most already reject; this task proves it on a fresh binary and closes any silent hole.

**Files:**
- Modify (only if a silent hole is found): `crates/kali_types/src/resolve/mod.rs`, `crates/kali_types/src/repr_infer.rs`
- Test: `crates/kali_cli/tests/growable_array_fail_closed.rs` (new)

**Interfaces:**
- Consumes: Task 2 recognition + the two E5506 channels (`add_shape_conflict`; `Diagnostic::error(e5::FEATURE_UNAVAILABLE)`).

- [ ] **Step 1: Write the failing/asserting test.**

Create `crates/kali_cli/tests/growable_array_fail_closed.rs` asserting each of these `run`s **fails** (non-zero exit) with an E5506 diagnostic, NOT a silent wrong answer:
- Materialization bind: `function m(){const out=[1,2,3].map(v=>v*2);console.log(out.length);}m();` (repro D).
- Escaping growable: `function make(){const o=[];o.push(1);return o;}function m(){const a=make();console.log(a.length);}m();` (escapes via return).
- Non-push mutator: `function m(){const o=[];o.push(1);o.pop();console.log(o.length);}m();` (pop unsupported).
Each assertion: `!status.success()` and stderr contains `error[E5506]`.

- [ ] **Step 2: Run to see current behavior.**

Run: `cargo test -p kali_cli --test growable_array_fail_closed 2>&1 | tail -30`
Expected: the map-materialization case already E5506s (design repro D). The escaping + pop cases: verify they E5506 and do not silently return a wrong length. If any silently succeeds with a wrong answer, that is a silent miscompile to close in Step 3.

- [ ] **Step 3: Close any silent hole.**

- Escaping growable: Task 2 Step 4 already routes a push-receiver-that-escapes to `add_shape_conflict`. Confirm the `return o` case is seen as an escape (`binding_escapes` true) → E5506. If instead it silently falls to the aggregate-0 lane, add the reject.
- `pop`/other mutators: since only `"push"` has a recognizer arm, a `.pop()` on a growable binding must hit the generic member-call E5506 reject, not a silent no-op. Confirm; if it silently returns 0, add an explicit reject arm.

- [ ] **Step 4: Run to verify all fail-closed.**

Run: `cargo test -p kali_cli --test growable_array_fail_closed 2>&1 | tail -20`
Expected: PASS (all three reject with E5506).

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_types crates/kali_cli/tests/growable_array_fail_closed.rs
git commit -m "test(types): growable array out-of-scope shapes fail closed (E5506), no silent miscompile [stage4]"
```

---

## Task 7: Full-workspace gate checkpoint + adversarial whole-stage review

The only sufficient gate (Stage-2/3 lesson: per-task "green" is necessary but not sufficient). Runs the exact CI command, diffs against the main worktree, drives the fresh binary vs node adversarially, and closes the stage.

**Files:**
- Modify: `docs/superpowers/followups/throw-fallout-stage4-triage.md` (drain snapshot), memory `kali-throw-fallout-stage3.md` follow-up / new `kali-throw-fallout-stage4.md`
- Scratch: `$SCRATCH/stage4-post.txt`

- [ ] **Step 1: Build + full enumeration.**

Run: `cd /workspace && cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' | sort > "$SCRATCH/stage4-post.txt" && wc -l "$SCRATCH/stage4-post.txt"`
Expected: 818 (834 − 16).

- [ ] **Step 2: PRIMARY GATE — zero newly-red vs stage entry.**

Run: `comm -13 "$SCRATCH/stage4-pre.txt" "$SCRATCH/stage4-post.txt"`
Expected: EMPTY. Any name here is a Stage-4 regression — bisect (parent green → commit red on a freshly-built worktree binary) and fix before closing. Do not proceed while non-empty.

- [ ] **Step 3: Confirm the drain = exactly the 16 targets.**

Run: `comm -23 "$SCRATCH/stage4-pre.txt" "$SCRATCH/stage4-post.txt"`
Expected: exactly the 16 `array_callback_identity_slices_in_browser_api_surface_with_harness` names from Task 1 Step 3. Any other drained name (or any missing target) is reconciled and explained in the triage doc.

- [ ] **Step 4: Adversarial whole-stage review (fresh binary vs node).**

On the freshly-built binary, probe each and compare byte-for-byte to `node`:
- push accumulation + both join separators (`,` and `\n`).
- **growth across a realloc boundary**: push > INITIAL_CAP (e.g. 100 pushes) then join/length/`o[99]` — verify no slot corruption on the copy.
- int AND string element joins.
- `x.length`/`x[i]` after pushes; for-of over a growable both as source and sink.
- **re-masking check:** confirm the fixture's `throw` path is *reachable* — temporarily break one push (locally, not committed) and confirm the guard throws → proves the green is from real accumulation, not a re-silenced self-check.
- **browser lane:** run the reduced harness via `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node ./target/debug/kali --output json run --api browser` and confirm `success:true`. RE-CONFIRM no `kali:rt` host import was needed (Task 1 Step 6); if `__join_growable` needed a host helper, all four import lists must carry it (`kali-browser-harness-import-sync`).
Record the transcript in the triage doc.

- [ ] **Step 5: fmt/clippy + exact CI command.**

Run: `cargo fmt --all -- --check && cargo test --workspace 2>&1 | tail -5` (the exact CI fail-fast command; expect it to pass now that the failing set is drained). Fix any fmt drift.

- [ ] **Step 6: Snapshot the drain, write the stage memory, commit.**

Update the triage doc with the 834→818 drain snapshot and the adversarial-review transcript. Write memory `kali-throw-fallout-stage4.md` (denominator 834→818, growable-array primitive, lessons) + a one-line `MEMORY.md` pointer. File follow-ups: OOB-index `undefined` (if deferred in Task 2), map/filter materialization (repro D, still fail-closed), extra mutators (pop/shift/splice), cross-arena/escaping growable arrays, F64/Object element reprs.
```bash
git add docs/superpowers/followups/throw-fallout-stage4-triage.md
git commit -m "docs(soundness): throw-fallout Stage 4 checkpoint — 834->818, 0 regressions, growable-array primitive"
```

- [ ] **Step 7: Report the gate verdict.**

State explicitly: denominator 834 → 818; PRIMARY GATE (newly-red vs entry) = 0; drain = exactly the 16 targets; adversarial review clean; branch stays UNMERGED (PR #16 draft/held per program policy). Stage 4 CERTIFIED.

---

## Self-review notes (spec coverage)

- Spec §1 scope (16 targets, 834→818) → Tasks 1, 7. §2 layout → Global Constraints + Task 2. §3 growable recognition (both sides) → Task 2 Steps 3–5. §4 op lowering: alloc/push/length/index → Task 2; string elems → Task 3; for-of → Task 4; join → Task 5. §5 reclamation/escape → Task 2 Step 4 (arena via `alloc_callee_index`; escape reject) + Task 1 Step 7. §6 error handling (fail-closed, no silent) → Task 6. §7 gate + adversarial review + browser-lane + no-host-import → Task 1 Step 6, Task 7. Both-sides mirror → every codegen task pairs a kali_types arm. GC-less → arena allocator only, no GC introduced.
- OOB-index behavior is the one spec-flagged plan-time decision (Task 2 Step 6): node-faithful `undefined` if cheap, else in-range-correct + follow-up; no target test indexes OOB so either satisfies the gate.
