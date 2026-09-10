# The mutation scan that gates layout-binding specialization is scope-blind and name-based, so a const array mutated only through a differently-named parameter is invisible to it

**Filed** 2026-09-10, by the **release-tier-allocation-identity** project
(`docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md`
§5, first non-goal bullet; `docs/superpowers/sdd/2026-09-10-release-tier-allocation-identity/`,
Task 8). Found while tracing why `spectral-norm-benchmark-v1`'s `const w` is
safe from the layout-binding specialization defect this project fixed (Stage 1,
`is_array_literal`'s positive element check) — it turns out `w` was *also*
protected by a second, independent gap this document is about, and Stage 1's
fix happens to close the only avenue that currently reaches it.

**Oracle:** `node v26.8.2`. **Measured on:** this branch's HEAD at the time of
measurement (`d67d820f98`, the commit Task 8 started from — Stage 1 already
landed). **This is a §5 non-goal of the release-tier-allocation-identity
project, filed here rather than fixed there, per that project's own scoping
rule**: "a distinct root cause with a distinct fix, and folding it in would
make this project two claims."

## 1. The mechanism, traced in source

`collect_mutated_binding_names` (`crates/kali_optimize/src/object_fold.rs:519`)
is the **one, shared** scan every binding-eligibility check in `kali_optimize`
uses to decide "is this name safe to treat as a constant". Its own doc comment
says what it is on purpose:

> Names that are the base of any member store (`x.k = v`) or member delete
> (`delete x.k`) anywhere in the program. **Name-based and shadowing-blind BY
> DESIGN**: a shadowed name over-approximates to "mutated", which only ever
> DISABLES folding (fail-closed direction).

It walks every `LirNode` in the **whole program** — not per-scope, not
per-function — and for every store or delete node, resolves the member
expression's base to a bare identifier (`dot_member_base_and_key` for
`x.k = v`, `member_chain_base_identifier` for `x[expr] = v`) and inserts that
identifier's **text** into a `BTreeSet<String>`. There is no binding
resolution here at all: two identifiers spelled the same way, in two
unrelated scopes, are the same entry in this set. The doc comment is explicit
that over-marking (two unrelated `x`s, one mutated) is the safe direction and
is accepted. **The dangerous direction is under-marking — a name that IS
mutated, but not under the spelling this scan is looking for — and that is
exactly what a function parameter does.**

`specialize_layout_bindings` (`crates/kali_optimize/src/specialize.rs:82-86`,
`:108-112`) is the consumer that matters for this project — the same pass
Task 2 patched:

```rust
if !mutated.contains(&name) && self.is_specializable_binding(program, init) {
    local_env.bindings.insert(name, init);
}
```

A binding enters the specialization environment — and every later read of its
name gets overwritten with a **clone of its initializer node**
(`specialize.rs:120-127`, the same substitution this whole project is about) —
only if its name is *not* in `mutated` AND its initializer passes
`is_specializable_binding`. If a `const` array is genuinely mutated, but only
ever through a **different name** — a function parameter the array was passed
under — its own name never becomes a store base, `mutated` never contains it,
and the second half of that `&&` is the only thing standing between it and
being silently treated as an immutable literal.

## 2. spectral-norm's `w` is the real-world instance of this shape

`crates/kali_cli/tests/fixtures/benchmarks/spectral-norm-benchmark-v1.ts`
(read, not edited — this project never touches fixture files):

```js
function Au(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) {
      t = t + A(i, j) * u[j];
    }
    v[i] = t;          // <-- the only store to this array, anywhere
  }
}
...
function spectralnorm(n) {
  const u = new Array(n).fill(1);
  const v = new Array(n);
  const w = new Array(n);   // <-- the const binding's own name
  for (let i = 0; i < 10; i = i + 1) {
    AtAu(u, v, w);
    AtAu(v, u, w);
  }
  ...
}
```

`w` (and, symmetrically, `u` and `v` at different call sites) is written
**only** as `v[i] = t` inside `Au`'s body — under the parameter name `v`,
which `Au` happens to share with `spectralnorm`'s own local `v`, but `w` is
never called `w` inside `Au`. `collect_mutated_binding_names`'s program-wide
scan records `"v"` (and, from `Atu`, also `"v"`) as mutated. It never records
`"w"`, because the text `"w"` never appears as a store base anywhere in the
program. If `w`'s initializer (`new Array(n)`) ever passed
`is_specializable_binding`, `w` would be silently admitted to the spec env
and duplicated at every later read — which would be wrong, because `w` really
is mutated, once per outer loop iteration, through the alias.

## 3. What Stage 1 changed, and why the hole is not live today

**Stage 1 removed the exposure for allocations, and that is the whole reason
this is a followup and not a bug report.** Before Stage 1,
`is_array_literal`'s negative-space definition admitted the `new Array(n)`
wrapper unconditionally (any text-less `Value` that isn't an object literal),
so `w`'s initializer passed `is_specializable_binding` and the only thing
protecting it from this project's own headline defect and from this
scope-blind mutation scan was luck of naming. Stage 1's positive element check
(`is_materializable_element`, `crates/kali_optimize/src/layout.rs`) requires
every child to be itself materializable; `new Array(n)` has a `Call` child, so
it fails the check and never reaches the `mutated.contains(&name)` test at
all — the whole binding is excluded one step earlier, for an unrelated reason.

**The hole remains open for genuine array literals** (`const w = [1, 2, 3]`,
no allocation, no `Call` child) — those pass Stage 1's positive check and
*would* reach the `mutated` test. Measuring whether that is exploitable today
turned up a second, independent, pre-existing guard that happens to block it:
kali refuses to compile any program that mutates a literal array at all,
under any name:

```
$ cat alias3.js
function main() {
  const a = [1, 2, 3];
  a[0] = 99;
  console.log(a[0]);
}
main();
$ kali build --fast alias3.js
error[E5506]: mutating a literal array is unavailable in the current direct-runtime path; use new Array(n) for runtime mutation
```

(`crates/kali_types/src/resolve/expression.rs:1685`, pre-existing, unrelated
to this project.) So today, every avenue this scan's own gap would need to be
reached through is independently blocked: allocations by Stage 1's positive
element check, and literal arrays by the direct-runtime mutation refusal.
**This is not a proof the scan is safe** — it is a statement about which
guards happen to cover for it today. The gap in `collect_mutated_binding_names`
itself is unchanged: it is still a scope-blind, name-based scan, traced in
source, exactly as spectral-norm's `w` demonstrates. It becomes live again the
moment either covering guard moves — most plausibly, the moment a future
project represents array-literal-ness positively in the LIR (Stage 2, sized
and stopped in `task-7-report.md`) and, in doing so, also teaches
`is_specializable_binding` that an allocating initializer *can* be
specialized under some narrower condition, or the moment literal-array
mutation is ever supported.

## 4. Runnable repro, measured at every tier

The exact shape (`Au`/`v`/`w`, reduced to the load-bearing three lines),
measured in-process (`kali run` has no tier flag, so the release tiers were
measured via the same in-process harness Task 1 built —
`crates/kali_cli/tests/inprocess/release_allocation_identity.rs`'s
`compile_at`/`run_at` — as a throwaway, non-committed test, reverted
immediately after measuring):

```js
function Au(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    v[i] = u[i] + 1;
  }
}
function main() {
  const u = new Array(3).fill(1);
  const w = new Array(3);
  Au(u, w);
  console.log(w[0]);
}
main();
```

| tier | result |
|---|---|
| node `v26.8.2` | `2` |
| `--fast` (`kali run`) | `2` |
| `--release` (in-process) | `2` |
| `--release-advanced` (in-process) | `2` |

**All three tiers agree with node today.** `w`'s initializer (`new Array(3)`)
is excluded from the spec env by Stage 1's positive element check before the
`mutated` set is ever consulted for it — confirmed by construction (§3), not
merely by this one passing measurement. The repro is recorded here as the
shape a later change must re-measure against, not as a currently-failing
case.

## 5. Fix direction

`collect_mutated_binding_names` needs the SAME thing every name-based scan in
this codebase has needed once it stopped being an approximation and started
gating a substitution: it must resolve a store's base through the CALL SITE'S
argument bindings, not just the local text, or the specialization walk must
never admit a binding whose value escapes into a call argument at all
(conservatively excluding any `const` passed to a user function as an
argument from the spec env, regardless of what that function does with it).
The second is cheaper and is the direction `is_specializable_binding`'s
existing conservative-by-design posture (Stage 1: decline rather than guess)
already points toward. **Do not fix this by widening what the scan
considers a "mutation"** — the doc comment's "over-marking is safe" is a
real invariant elsewhere in the codebase (the ordered object-enumeration fold
at `object_fold.rs:646` also consumes this same scan) and a change here
affects both consumers at once.

## 6. Suggested home

**Not the silent-miscompile register.** Every register entry (§2) requires a
currently-measurable exit-0, no-diagnostic wrong value; §3's guards mean there
is none to pin today, and adding a register row for a defect with `raw 0 /
reachable 0` and no reachable oracle case would be exactly the "publish a
zero-frequency entry with no gate objecting" trap
`crates/kali_blast_radius/src/manifest_tests.rs`'s own comment on R-56 warns
against. This document is the record instead, matching the treatment given to
§2.5's dynamic-index-on-a-literal gap by the design spec that produced this
project. If a later change removes either covering guard named in §3, this
document is the mechanism trace and repro shape that change must re-run
before shipping.
