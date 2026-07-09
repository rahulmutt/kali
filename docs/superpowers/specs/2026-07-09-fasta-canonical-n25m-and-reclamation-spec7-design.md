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
output string temporary (`join`, `substring`, string `+`) is allocated in the
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
| `seq.substring(...)` | `__substring` (synthetic wasm, zero-copy header) | `__alloc_global` | ~tens of MB of small descriptors       |
| `a + b` (strings)    | `string_concat` (**host import**, `kali:rt`) | host calls exported `__alloc_global` | ~10 MB (fastaRepeat wraparound only)   |

All three ultimately allocate via `__alloc_global` — the never-reset g4/g5/g6
arena trio (`lower.rs`, `host/memory.rs:143` for the host import). The
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

**Mechanism.** Emit arena-variant twins of the string builtins that allocate via
the current-arena allocator instead of the global one, and select per call site:

- **`__join_arena` / `__substring_arena`** (synthetic wasm): byte-for-byte
  identical to `__join` / `__substring` except they call **`__alloc`** (current
  arena) where the originals call `__alloc_global`. Emitted only when at least
  one call site selects them (avoid dead functions in every module).
- **`string_concat` (host import):** export `__alloc` to the host and add a
  concat arena path so the host allocates the concat result into the *current*
  arena. When a concat call site is arena-routed, the wasm passes the host a
  signal (an extra arg, or a distinct `string_concat_arena` import — decided in
  the plan) telling it to call the exported `__alloc` rather than the exported
  `__alloc_global`. The current-arena globals (g1/g2/g3) are live and correct
  while a host import runs mid-iteration, so an exported `__alloc` bumps the
  right arena.

**Per-call-site routing.** At each string-producing call site, codegen selects
the arena path **iff**:
1. the enclosing function is `arena_eligible`, **and**
2. the call is lexically inside an open iteration/function arena, **and**
3. the escape analysis proves the result is **iteration-local** (does not escape
   the innermost open arena's dynamic extent).

Otherwise it emits the global path. **Fail-closed default = global** (mirrors
`alloc_callee_index`).

**Escape analysis extension.** Teach the existing escape/arena machinery
(`classify_value` / `escape_flow` / `arena_gate`) to model **string-temporary
value flow**. A string-producing call is `ScopeLocal` when its result flows only
into same-iteration consumers that drop or copy it:
- the `console.log` sink (writes bytes to stdout, retains nothing), and
- `string_concat` (copies its inputs' bytes into a fresh result — so a
  `substring` feeding a concat is local even when that concat's *own* result is
  arena- or global-allocated).

Any result bound to a binding that outlives the iteration, stored into a
field/element/global, or returned → **`Global`** (veto). Apply the binary-trees
discipline: **enumerate every out-flow shape and fail-closed on each**;
shape-specific detection will miss a shape. Both-sides oracle mirroring is
mandatory — the kali_types/analysis predicate that grants the arena path and the
codegen recognizer that selects the twin must agree, or the site fails open.

This extension is also what makes `fastaRepeat` / `fastaRandom` `arena_eligible`
in the first place: today their string temporaries read as escaping (global), so
the functions are not arena-eligible.

**Why this reaches N=25M.** With join/substring/concat all arena-routed inside
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

Pre-existing on main (found during Spec 6 T1 verification): module-scope
`var o = {x:1}; o += 1; console.log(o)` prints `1` (node: `[object Object]1`).
`o` carries a default I64 module-scope repr and the compound routes through
`emit_module_global_assignment` (`literal.rs:466-469`) as a scalar add over the
heap handle — a fail-open. Same positive-proof family as §3.2: a module binding
holding a heap value must not admit scalar compound/update.

**Fix.** Extend the compound/update admission gate to **module-scope bindings**:
a module binding needs positively-proven scalar repr ({I64, F64, String}) exactly
as a param does. An object/array module binding **rejects** (E5506) — node
string-coerces the object, which kali cannot generally reproduce, so
reject-don't-miscompile is the correct kali behaviour. Pin: `o+=1` on a module
object binding → E5506 exit 1.

### 3.4 `??=` nullish-vs-falsy lowering bug

Pre-existing on var locals; **newly reachable on params** after Spec 6 (params
were immutable before, so `p ??= x` could not occur). `??=` lowers with
`I64Eqz` (`literal.rs:571-588`) — a **falsy** test (value == 0), identical to
`||=`. So `let x = 0; x ??= 1` wrongly assigns 1 (0 is not nullish), and a
numeric param `p ??= 1` wrongly assigns whenever `p` is 0.

**Fix — implement nullish (not falsy) semantics**, fail-closed on ambiguity:
- **Provably-numeric / never-nullish target** (numeric param or numeric local):
  `??=` is a **no-op** — emit the LHS unchanged. (A number is never
  null/undefined; this covers the newly-reachable param case.)
- **Provably-nullable target with a real null sentinel** (e.g. a for-in-key
  alias, sentinel −1): test against the sentinel, not `I64Eqz(0)`.
- **Otherwise** (nullability not provable): reject fail-closed (E5506) rather
  than the current falsy miscompile.

Pins: `let x=0; x??=1` keeps 0; numeric param `p??=1` is a no-op; the existing
`let value=null; value??=1` → 1 test still passes.

## 4. Testing

### 4.1 Canonical pin (`clbg_fasta_runtime.rs`)

- Retain the N=8 byte-golden tier unchanged.
- Replace the N=2M SHA-256 tier with **N=25,000,000**, asserting SHA-256 =
  `6a26f1c8…`, run under `--api node --sandbox <policy>`.
- The N=25M sandbox policy must raise the fuel and memory budgets to cover the
  bounded peak (~KB working set + a safety margin) and the (now eliminated)
  residuals; re-derive concrete budget numbers during implementation.

### 4.2 Bounded-peak unit test (the real reclamation proof)

A minimal `join`/`substring`/`concat`-in-a-`while`-loop fixture run under a
**small fixed memory policy** at two very different N values. Both must pass ⇒
peak is O(1) in N ⇒ reclamation works. This is the fast, deterministic proof of
the headline, decoupled from the slow canonical pin.

### 4.3 Soundness pins

- §3.2: the two laundering reproducers (`f(5); f(g())`; self-recursive
  `h(p+1)`) reject fail-closed; a positive test that fasta's expression-arg call
  sites still admit compound/update.
- §3.3: `o+=1` on a module object binding → E5506 exit 1.
- §3.4: `x=0; x??=1` keeps 0; numeric param `p??=1` no-op; `value=null;
  value??=1` → 1 preserved.

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
