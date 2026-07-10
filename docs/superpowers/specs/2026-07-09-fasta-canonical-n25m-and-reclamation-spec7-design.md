# fasta Spec 7 — Canonical N=25M via per-line reclamation, + soundness closures

**Status:** design (awaiting review)
**Series:** CLBG fasta (Specs 1–6 shipped; PRs #9–#14). This is the terminal
milestone of the fasta series — the canonical CLBG input size.
**Predecessor:** Spec 6 (`docs/superpowers/specs/2026-07-09-fasta-verbatim-vendor-spec6-design.md`,
shipped PR #14, main `71b3503ab`).
**Node oracle:** v26.4.0.
**Integration:** push a PR + self-merge per the `kali-integration-convention`
memory (`gh` authed as rahulmutt).

## 1. Goal

Run the verbatim upstream `fasta-node-1` fixture at its **canonical input
`N = 25,000,000`** byte-for-byte against node v26.4.0, and close three
soundness gaps that the reclamation work either depends on or sits adjacent to.

The headline is a memory result, not a new language surface: the fixture
already runs correctly at N=2,000,000. What blocks N=25M is that every per-line
output string temporary that allocates (`join` and string `+`; `substring` is
zero-copy and allocates nothing) is allocated in the
**never-reset global arena** and leaks, so cumulative allocation crosses the
sandbox policy's memory budget (E4000) somewhere around N≥4M. The fix is
**per-line reclamation**: route those temporaries into the per-iteration arena
that already exists (binary-trees Phase 1), so peak memory becomes bounded and
N-independent.

### Deliverable / acceptance

1. **Canonical pin.** `fasta-benchmark-v1.ts` at N=25,000,000, run under
   `--api node --sandbox <policy>`, produces output whose SHA-256 equals the
   recorded node v26.4.0 reference
   `6a26f1c843bebd234692ff1bd98ad517dd7df732fe93d2095845a2ddafc9ecee`.
   This tier **replaces** the interim N=2,000,000 SHA-256 tier in
   `clbg_fasta_runtime.rs`. The N=8 byte-golden tier is retained.
2. **Bounded, N-independent peak.** After reclamation, nothing on the fasta
   output path leaks O(N). Proven by a dedicated small-policy unit test (§4.2),
   independent of the slow canonical pin.
3. **No regression.** All five prior CLBG fixtures (nbody, fannkuch,
   spectral-norm, mandelbrot n=200, binary-trees N=21) remain byte-identical;
   `param_compound_assign` stays 16/16; full 5-crate gate + `cargo fmt --check`
   + `cargo clippy --workspace -- -D warnings` green.
4. **Three soundness closures** land in the same branch (§3.2–§3.4), each with
   its own reject/behaviour pins.

### Gate command (corrected, per Spec 6 ledger)

```
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

The CLI package is `kali_cli` (binary `kali`; `CARGO_BIN_EXE_kali` in tests).
Run `cargo clippy --workspace -- -D warnings` locally before every push — CI
(`ci.yml`) runs it and the plan-level per-task gate historically missed lints.

## 2. Background: where the leak is

Tracing the allocators on the fasta output path:

| Temporary            | Producer                        | Allocator today        | Volume @ N=25M (approx)                |
|----------------------|---------------------------------|------------------------|----------------------------------------|
| `line.join("")`      | `__join` (synthetic wasm)       | `__alloc_global`       | ~3.3M lines × ~64B ≈ **213 MB** (dominant) |
| `a + b` (strings)    | `string_concat` (**host import**, `kali:rt`) | host calls exported `__alloc_global` | ~10 MB (fastaRepeat wraparound only)   |
| `seq.substring(...)` | `__substring` (synthetic wasm)  | **allocates nothing** — zero-copy re-tag | **not a leak source** |

**Verified on a fresh binary (2026-07-09):** `__substring` is a pure-ALU
zero-copy handle re-tag (`emit_substring_body` calls no allocator); a substring
result aliases its parent string's bytes (for fasta, the module-constant ALU
string), so it leaks nothing and is out of scope for reclamation. Only `__join`
and `string_concat` allocate per-line and must be reclaimed. Both allocate via
`__alloc_global` — the never-reset g4/g5/g6 arena trio (`lower.rs:3324` for
`__join`; `host/memory.rs:137-179` for the host import). The
per-iteration arena machinery (`emit_loop` open/reset/release; `__alloc` routing
via `alloc_callee_index`) **exists** and is load-bearing for binary-trees, but
runtime strings bypass it by hardcoding the global allocator. That bypass was
the deliberate fail-closed default when strings shipped ("host runtime strings
route through the never-reset global arena"); Spec 7 makes it precise.

The wall is the **policy memory budget** (E4000 = budget exceeded), driven by
*cumulative* global growth — not the wasm32 4 GB ceiling and not fuel (E4003).
Reclamation bounds *peak*, so a modest policy budget then admits any N.

## 3. Design

Four workstreams on one branch. §3.1 is the headline; §3.2 is a hard
precondition-adjacent soundness fix flagged by the Spec 6 final review; §3.3–§3.4
are pre-existing fail-opens in the same bug family, pulled in by explicit
request.

### 3.1 Per-line reclamation — escape-routed arena variants

**Mechanism.** Emit arena-variant twins of the leaking string producers that
allocate via the current-arena allocator, and select **per call site**:

- **`__join_arena`** (synthetic wasm): byte-for-byte identical to `__join`
  (`emit_join_body`) except it calls **`__alloc`** (g1/g2/g3 current-arena trio)
  where `__join` calls `__alloc_global`. Structurally, `emit_join_body` already
  takes an `alloc_index: u32` argument — the twin passes the `__alloc` index.
  Emitted only when at least one call site selects it (no dead function in
  modules that never arena-route a join). `__substring` needs **no twin** — it
  allocates nothing.
- **`string_concat` (host import):** `__alloc` is *already exported* from every
  module (the export loop at `lower.rs:516-521` exports every synthetic by
  name; verified). Add a sibling host import `string_concat_arena` that is
  identical to `string_concat` except its allocator helper looks up
  `caller.get_export("__alloc")` instead of `"__alloc_global"`. The
  current-arena globals (g1/g2/g3) are live and correct while a host import runs
  mid-iteration, so the exported `__alloc` bumps the right arena. The codegen
  emits `Call(string_concat_arena_import_index)` vs
  `Call(STRING_CONCAT_IMPORT_INDEX)` per site. The four hand-mirrored `kali:rt`
  JS import lists (`kali-browser-harness-import-sync` memory: `browser/harness.rs`
  ×2 + `bin/cmd_build.rs` ESM+CJS) each gain a `string_concat_arena` entry and an
  `allocGuestStringCurrent` variant, or browser tests fail with a LinkError.

**Per-call-site routing — new `ArenaTable` per-site channel.** The escape
analysis found `ArenaTable` is purely per-function/per-loop and runtime strings
are invisible to it (classified `Scalar` in `classify_value`, so they produce no
allocation site). Per-call-site routing therefore requires **new machinery**
(the chosen approach, over the lighter per-function verdict):

1. A new per-site key in `ArenaTable` (e.g. `arena_string_site: BTreeSet<(String
   func, u32 site_ordinal)>`), populated by the analysis, queried at the join/
   concat call site. It must survive the name-collision poisoning the
   `FunctionArenaFacts` pipeline applies (`arena_gate.rs:143-147, 250-256`).
2. `classify_value` (escape_flow.rs) learns to model each string-producing call
   site's **locality**: the result is arena-routable when it flows only into
   same-iteration consumers that drop or copy it — the `console.log` sink
   (whitelisted via `is_whitelisted_host_method`, already treated as
   consuming/dropping its argument) and `string_concat`/`string_concat_arena`
   (copies its inputs, so a value feeding a concat is local regardless of the
   concat result's own routing). Any result bound beyond the iteration, stored
   into a field/element/global, or returned → **not routable** (veto).

**Fail-closed default = global.** Apply the binary-trees discipline:
**enumerate every out-flow shape and fail-closed on each**; shape-specific
detection will miss a shape. Both-sides oracle mirroring is mandatory — the
analysis predicate that records the site as arena-routable and the codegen
recognizer that selects the twin must agree, or the site fails open.

The enclosing function must additionally be arena-context-bearing (inside an
open loop/function arena) for the twin to have somewhere to allocate; this reuses
the existing `arena_eligible`/`loop_arena`/`opens_arena` state that
`emit_loop`/`emit_function_arena_prologue` already establish.

**Why this reaches N=25M.** With join/concat arena-routed inside
their `while`-loop iteration arenas, each output line's temporaries reset at the
top of the next iteration. Peak collapses from ~243 MB cumulative to the working
set of a single line plus the persistent `line`/`table` structures — a few KB —
independent of N. (The `line` array, allocated before the `while` and
conditionally reassigned inside it, is used within its own iteration and is not
a per-line temporary; it stays in the function arena / its own iteration, not
reset out from under a live use.)

### 3.2 Existential-laundering closure (∃ ⇒ ∃ ∧ ∀)

Spec 6's param compound/update gate admits a param when it has *positively
proven scalar inflow* — `scalar_inflow_params`, populated when **some** call
edge supplies a syntactically-scalar argument (an existential ∃ proof;
`repr_infer.rs` Step 1b). The final review found the hole (deferred, masked): a
scalar edge at one site admits an **indirect** edge at another. `f(5); f(g())`
proves `f`'s param scalar via `f(5)`, then admits a compound assign even though
`f(g())` could deliver a heap handle; a self-recursive `h(p+1)` seeds its own
chain. Harmless today only because indirect array *delivery* is non-functional
(the param receives 0, never a heap handle) — but §3.1 does not make indirect
delivery functional, so this stays inert; we close it now because it is the same
positive-proof family and the reproducers are already written.

**Fix.** Pair the existing ∃ proof with an **∀-no-unproven-edge** condition: a
param is proven-scalar iff **some** edge is syntactically scalar **and no** edge
passes it a non-scalar-or-unproven argument. `f(g())`'s call-result argument is
not syntactically scalar and is unproven → it vetoes `f`'s param, so the
compound assign rejects fail-closed (E5506). A self-recursive `h(p+1)` passes a
syntactically-scalar (arithmetic) argument, so it does not self-veto. Remove the
`repr_infer.rs` tripwire note; the two reviewer-written reproducers become
permanent reject pins.

**No regression:** fasta's call sites all pass syntactic expressions (`2*n`,
`3*n`, `5*n`), so their param proofs are preserved; `param_compound_assign`
stays green.

### 3.3 Module-scope var-object compound fail-open (`o+=1`)

Pre-existing on main. **Verified on a fresh binary (2026-07-09):**
`var o = {x:1}; o += 1; console.log(o)` prints `1` (node: `[object Object]1`),
and `var o = {x:1}; console.log(o.x)` prints `0`. So the object initializer is
lost and `o` is a plain I64 binding holding 0. The earlier characterization
("routes through `emit_module_global_assignment`") was **wrong**: `o` is *not*
promoted to a module global (`collect_module_scalar_globals` only promotes names
referenced *inside a function*, and this `o` is module-only). It stays a
`_start` local with default repr `I64` (its object shape never proven), so
`o += 1` takes the generic **local** compound path (`literal.rs` `+=` arm) and
does `0 + 1`.

The gate that should reject it is the types-side
`compound_update_target_is_scalar` (`resolve/expression.rs:1697`), which admits
when `scalar(func, name) ∈ {I64, F64, String}`. `o`'s repr is default I64, so it
is admitted. Var locals are exempted from the param positive-proof lane
(`param_lacks_scalar_inflow` records only params), so an object-initialized var
falls through. Same positive-proof family as §3.2, now for a var local.

**Fix (diagnose-then-fix).** A binding whose declarator initializer is an
object/array literal (or otherwise heap) must not be admitted as a scalar
compound/update target. Route the fix through the existing single choke point
`target_repr_is_one_of` (`resolve/expression.rs:1927`): make an
object/array-initialized binding fail the scalar allowlist — either by giving it
a non-I64 (heap) repr in inference, or by an object-initializer taint set
mirroring `non_scalar_params`. An object/array binding **rejects** (E5506) —
node string-coerces the object, which kali cannot reproduce, so
reject-don't-miscompile is correct. The exact locus needs a short debugging pass
(Task 3, Step 2) to confirm why the object repr defaults to I64. Pins:
`var o={x:1}; o+=1` → E5506 exit 1; `var o={x:1}; console.log(o.x)` behaviour is
tracked but not in scope to *fix* here (only the compound miscompile is).

### 3.4 `??=` nullish-vs-falsy lowering bug — reject scalar `??=` fail-closed

Pre-existing on var locals; **newly reachable on params** after Spec 6 (params
were immutable before). **Verified on a fresh binary (2026-07-09):**
`let x = 0; x ??= 1; console.log(x)` prints `1` (node: `0`), and a numeric param
`f(0)` with `p ??= 1` prints `1` (node: `0`). Root cause: `??=` lowers with
`I64Eqz` (`literal.rs:571-588`) — a **falsy** test (value == 0), identical to
`||=` — and **`null`/`undefined` both lower to i64 `0`** for a scalar
(`control_flow.rs:1131-1137`). So kali *cannot* distinguish `null` from numeric
`0` for a scalar binding: a correct nullish test is unrepresentable without a
new nullable-scalar representation, which is out of scope.

**Fix (decided): reject scalar `??=` fail-closed.** Emit E5506 for `??=` on any
scalar local/param (the unrepresentable-nullish case), keeping only the
**for-in-key-alias** path, which has a real null sentinel (`-1`,
`literal.rs:495-505`) and already lowers `null`/`0` distinctly. This eliminates
the miscompile and is series-consistent (reject-don't-miscompile). It changes
the accidentally-"working" `let value = null; value ??= 1` case (currently → 1)
to a clean reject — acceptable, since that only "worked" because null happened to
equal the `0` sentinel.

Implementation: the reject belongs in the resolve phase (mirroring the compound
gate) so compilation stops before codegen. The existing
`assert_nullish_assignment_lowers` codegen test (`test_support.rs:47`) and its
users (`literal_tests.rs:100`) must be updated to the for-in-key-alias case or
removed for the now-rejected scalar case. Pins: `let x=0; x??=1` → E5506;
numeric param `p??=1` → E5506; a for-in-key-alias `??=` still lowers.

## 4. Testing

### 4.1 Canonical pin (`clbg_fasta_runtime.rs`)

- Retain the N=8 byte-golden tier unchanged.
- Replace the N=2M SHA-256 tier with **N=25,000,000**, asserting SHA-256 =
  `6a26f1c8…`, run under `--api node --sandbox <policy>`.
- The N=25M sandbox policy must raise the fuel and memory budgets to cover the
  bounded peak (~KB working set + a safety margin) and the (now eliminated)
  residuals; re-derive concrete budget numbers during implementation.

### 4.2 Bounded-peak unit test (the real reclamation proof)

A minimal `join`/`concat`-in-a-`while`-loop fixture run under a
**small fixed memory policy** at two very different N values. Both must pass ⇒
peak is O(1) in N ⇒ reclamation works. This is the fast, deterministic proof of
the headline, decoupled from the slow canonical pin.

### 4.3 Soundness pins

- §3.2: the two laundering reproducers (`f(5); f(g())`; self-recursive
  `h(p+1)`) reject fail-closed; a positive test that fasta's expression-arg call
  sites still admit compound/update.
- §3.3: `var o={x:1}; o+=1` → E5506 exit 1; a genuine numeric var local
  (`var k=0; k+=1`) still compiles and runs.
- §3.4: `let x=0; x??=1` → E5506; numeric param `p??=1` → E5506; a for-in-key
  alias `??=` still lowers (updated `assert_nullish_assignment_lowers` user).

### 4.4 Regression

All five prior CLBG fixtures byte-identical; full 5-crate gate + fmt + clippy.

## 5. Risks & mitigations

- **N=25M wall-clock in-gate.** If kali runs N=25M in more than ~30 s, downgrade
  that tier to opt-in (`#[ignore]` + a dedicated CI job) while keeping §4.2 as
  the in-gate reclamation proof. Measure early during implementation and decide.
- **Host-allocator boundary (concat).** Exporting `__alloc` and adding a
  host-side arena allocation path touches a load-bearing seam
  (`host/memory.rs`, the four `kali:rt` import lists that must stay in sync per
  the `kali-browser-harness-import-sync` memory — browser harness ×2 + build
  bundle glue ×2). Keep the change minimal and mirror all import lists, or
  browser tests fail with a LinkError.
- **Fail-open in string escape analysis.** The single highest-risk item.
  Enforce veto-every-out-flow-shape + both-sides oracle mirroring; prefer
  vetoing to global over granting the arena path on any unproven shape.
- **Twin emission / DCE.** Emit arena twins only when referenced.
- **Process discipline (5+ prior validations).** Implementer/fix reports are
  unreliable: the controller re-runs every reproducer and the canonical pin on a
  freshly-built binary; reviewers execute a fix's claimed mechanism example, not
  just its reject pins.

## 6. Out of scope

- Making indirect array call-return / pass-through *delivery* functional (the
  masked non-functionality §3.2 relies on for its "inert today" status). §3.2's
  ∀-condition is a prerequisite for ever doing that, but that work is a separate
  spec.
- Any fasta input size beyond the canonical N=25M.
- The `crates/kali_cli/tests/fixtures/kali.json` nested-project-boundary latency
  (inert; watch item carried in the Spec 6 memory).

---

## 7. Revision 2026-07-10 — §3.1's arena-context assumption falsified; string-site-triggered loop arenas (Task 4f)

**What broke.** §3.1 assumed the twins "reuse the existing
`arena_eligible`/`loop_arena`/`opens_arena` state that
`emit_loop`/`emit_function_arena_prologue` already establish" and that fasta's
lines reset "inside their `while`-loop iteration arenas." Task 5 (canonical
N=25M pin) proved this false on a fresh release binary: **neither fasta loop
ever opens an arena**, so granted string sites route through `__alloc` into the
never-reset boot arena (g1=g2=g3 from module start) and leak — E4000 at N≈4-6M,
byte-identical to the pre-Task-4 wall (`.superpowers/sdd/task-5-report.md`).
Two independent causes (arena_gate.rs:797-826):
- `fastaRepeat`'s loop has no `ArrayExpr`/`ObjectExpr`, so `reaches_alloc`
  never fires (`NewExpr` doesn't count; walk.rs:232-243).
- `fastaRandom`'s loop trips the loop-level `has_outflow` veto via
  `line = new Array(n)` — correctly, for the OBJECT arena (the buffer outlives
  iterations), but that array routes to `__alloc_global` and a string-only
  arena would never capture it.

**Mechanism facts the fix rests on** (verified 2026-07-10 by census over the
emitted fasta wasm; full audit in `.superpowers/sdd/task-4f-investigation.md`):
1. **Routing and open/reset are decoupled.** `__alloc` vs `__alloc_global` is
   chosen per-site (`alloc_callee_index` per-function `arena_eligible`;
   `arena_string_site` per string site). `loop_arena` drives ONLY the
   open/reset/release emission in `emit_loop` (control_flow.rs:174-183,
   235-277, 338-342). A new, independent reason to emit open/reset changes no
   allocation's allocator.
2. **Capture is unconditional.** Open/reset rebinds g1/g2/g3 for the loop's
   whole dynamic extent: every `__alloc` executed during an iteration — direct,
   callee, or synthetic — lands in the loop arena and dies at the reset.
3. In both fasta loops the ONLY `__alloc` in the dynamic extent is the granted
   string site itself (concat in `fastaRepeat`, join in `fastaRandom`);
   every `new Array` is `__alloc_global` (NewExpr never sets `allocates`).

**Fix (Task 4f): a `string_arena_loop` channel.** A loop emits open/reset
(without setting `loop_arena` or touching any routing fact) iff ALL hold:
- **(T)** its body (excluding nested function-like subtrees) contains ≥1
  GRANTED `arena_string_site`, correlated analysis-side by threading the open
  loop-ordinal stack through `string_site_walk` (single-source; no new
  codegen-side mirror — codegen keys the channel by the EXISTING two-sided
  loop-ordinal stream, exactly like `loop_arena`);
- **(V1)** the enclosing function is NOT `arena_eligible` (else object/array
  sites route to `__alloc` and an unproven object could be captured; loops in
  arena-eligible functions keep the existing `loop_arena` machinery as their
  only arena path);
- **(V2)** no `has_unknown_call` in the loop body (unknown callee could
  allocate-and-retain via `__alloc`) — same veto as `loop_arena_qualifies`;
- **(V3)** no known callee reachable from the loop body may allocate
  (`reaches_alloc_transitively`, unknown ⇒ may-allocate, fail closed).
`has_outflow` is deliberately NOT consulted: outflow of `__alloc_global`-routed
values is invisible to a string-only arena, and granted string sites can never
outflow by 4b's default-deny grant. Soundness: by V1-V3 the only `__alloc`
users in the dynamic extent are granted string sites of this function (dead
within the iteration by 4b) and callees' granted string sites (dead by callee
return, since a returned/stored string is never granted). A callee that
`opens_arena` is additionally safe by LIFO save/restore.
**Accepted slack:** 4b's `+`-site grants are type-blind, so a numeric
`console.log(a+b)` loop in a non-allocating function may open a pointless —
but empty and sound — arena; prior fixtures' STDOUT must stay byte-identical
(the gate checks output, not wasm bytes).

**Codegen surface:** OR the new getter into the two existing `loop_arena`
gates only: save-local reservation (lower.rs:1846-1853) and `emit_loop`'s
`is_arena_loop` (control_flow.rs:173-183). Early-exit handling (break/
continue/return) is inherited from the existing frame machinery.

**Acceptance:** §4.1's canonical N=25M pin becomes reachable; a second
bounded-peak fixture (no object-literal trigger — the fasta shape) pins the
new channel at two N under one small budget, alongside 4e's object-triggered
fixture.

## 8. Revision 2026-07-10 (b) — for-in key-table rebuild leak (Task 4g)

**Second N=25M blocker** (found by 4f's acceptance run; mechanism audit in
`.superpowers/sdd/task-4g-investigation.md`): `emit_for_in` builds the
per-shape key-handle table (N_keys × 8 bytes) via `alloc_callee_index()` →
`__alloc_global` in the for-in **preheader** (control_flow.rs:487 →
object.rs:150-153). The table is a compile-time constant — each slot is a
data-segment string-handle constant — yet it is rebuilt on every for-in
EXECUTION. `fastaRandom` nests `for (c in table)` inside the per-character
loop: 120 B × 60 chars/line × ~316k lines ≈ 2.27 GB of identical tables at
the trap point (verified against VmHWM 2.19 GB). The global arena is immune
to every reclamation channel by design, so this must stop allocating, not
get reclaimed.

**Fix (Task 4g): emit the key table as module-constant data.** Since every
slot is a compile-time constant handle, the whole table belongs in the
module's constant-data layout (same discipline as data-segment strings),
referenced by a constant base — zero runtime allocation, O(1) in both N and
call count. Fallback if the constant-data layout cannot host non-string
words cleanly: replace the read-site load (control_flow.rs:1042-1057,
`table_base + ord*8`) with a constant dispatch (`br_table`/select chain)
over the statically-known keys — also zero-alloc. Hoisting the build to a
per-call prologue is REJECTED: it leaves a per-call residue (unbounded for
hot-called functions), against the branch's structural-bounds theme.

**Soundness:** the stored/read values are unchanged constants; `line[i]=c`
stores and `join` reads are independent of the table's storage (verified —
the element store loads the handle value, it never points into the table).
Outputs of every for-in consumer must stay byte-identical (pinned by
runtime_forin.rs — incl. the fasta-shaped
`forin_key_stored_into_string_array_after_break_then_joined` — plus
runtime_fasta_output.rs and the for-in ??= pin in nullish_assign_reject.rs).
