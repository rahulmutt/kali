# Defects discovered by the console-render-unification project, and not filed by it

## 1. What this is

The console-render-unification project (spec
`docs/superpowers/specs/2026-08-15-console-render-unification-design.md`, plan
`docs/superpowers/plans/2026-08-15-console-render-unification.md`) closed the
rendering half of the register's G8 cluster. While scoping and implementing it,
it measured **twelve divergences from node that are outside its scope**, plus one
gap in the register's own measuring instrument.

None of them is filed as a register entry. This file exists so that they are not
lost, and so that a future project filing any of them starts from a measurement
rather than a recollection.

**What this file is not.** It is not a register section. Nothing here carries a
tier, a §0.2 row, an oracle case, or a verdict class, and nothing here is counted
by `kali_blast_radius`. Filing any entry below is a deliberate act with its own
obligations — see §4.

**Provenance.** Every row was measured by the controller of that project against
`/workspace/.cache/cargo-target/debug/kali` built at `5aebc5ec3d` (the branch's
head after its final review), compared against `node v26.7.0`, on 2026-08-16.
Each was re-measured at that commit specifically for this document rather than
copied from a task report — the project's own standard is that a recorded claim
is a reading, not a memory.

**One exclusion, stated so its absence is not read as an oversight.** R-56 (the
quoted-numeric-string-key collision in `Object.hasOwn`) was *also* discovered by
this project and **is** filed, in the register's §2 as a Tier 2 entry with a §0.2
row and an oracle pair, because one direction of it regressed on that branch. It
does not appear below.

## 2. The defects

Ordered by severity as this project would score them: silent wrong values first,
then loud failures, then divergences that are arguably not defects.

### 2.1 A raw string handle is printed as a value — `+` lane, large integer binding

**The most serious thing in this file.** A binding holding a large integer,
concatenated, prints the raw tagged handle bits.

```js
var y = 1e19;
console.log("v=" + y);
```

| | output | exit |
|---|---|---|
| kali | `v=-9223354444668731372` | 0 |
| node | `v=10000000000000000000` | 0 |

Silent, at exit 0, and the printed value is not a wrong number — it is guest
memory addressing leaking into program output. This is the same damage shape the
register records for R-08's `String()`-result lane (measured
`x-9223354375949254655` there), reached by a different route.

**Not this project's to fix.** Spec §5.2 forbade touching the `+` and
template-literal path, terminal arm and taint alike, and that constraint held for
the whole project.

**Suggested home:** §2, silent. It is a wrong value, not a rendering choice.

### 2.2 `return_is_monomorphic` mis-seeds `Repr::String` on a conditionally-assigned return

```js
function pick(n) { let s = 0n; if (n > 1n) { s = String(42n); } return s; }
console.log("a", pick(0n));
```

| | output | exit |
|---|---|---|
| kali | `a ` (empty second field) | 0 |
| node | `a 0n` | 0 |

**Mechanism**, established by reading: `return_is_monomorphic`
(`crates/kali_types/src/repr_infer.rs`, around `:1653-1666`) checks that a
return's sources are *tainted*, not that they are *monomorphic*. So `pick` is
wrongly seeded `Repr::String`, and the `0n` passes through as if it were a string
handle. Multi-argument console lane only.

**Suggested home:** §2, silent.

### 2.3 Bracket assignment to an existing key does not take

```js
const o = {5:1};   o[5]   = 7; console.log(o[5]);    // kali 1, node 7
const p = {a:1};   p["a"] = 7; console.log(p["a"]);  // kali 1, node 7
const q = {a:1};   q.a    = 7; console.log(q.a);     // kali 7, node 7  ← control
```

Silent, exit 0. **Bracket-form assignment specifically** — the dot form is
correct, which is the control that localises it. Not sign- or type-specific:
numeric and string keys both fail.

**Suggested home:** §1 or §2 depending on whether the write is dropped or the
read is stale — the two are distinguishable and this project did not distinguish
them.

### 2.4 A boolean returned from a function renders as `1`

```js
function f() { return true; }
console.log(f());
```

| | output |
|---|---|
| kali | `1` |
| node | `true` |

**Related to R-30 but not the same lane.** R-30's open half is a plain *binding*
read (`var b = true; console.log(b)`); this is a *return* value. Both are blocked
on the same root cause — kali has **no `Repr::Boolean` axis** (`Repr` in
`crates/kali_common/src/repr.rs` has no `Boolean` variant, and no
`is_boolean_valued` function is defined anywhere) — which the register records
under R-34 and which the console-render-unification project corrected R-30's
fix-cost read to acknowledge.

Worth knowing: this is why `Object.hasOwn(...)` printed from inside a function
renders `1` rather than `true`, which can look like a `hasOwn` defect and is not.

**Suggested home:** an existing entry's lane, most likely R-30's, rather than a
new entry — but only after deciding whether R-30 is "the direct-log boolean
lane" or "the missing boolean repr", which the register currently answers both
ways.

### 2.5 `Object.keys` yields a Rust-formatted numeric key

```js
for (const k of Object.keys({1e-7:1})) console.log(k);
```

| | output |
|---|---|
| kali | `0.0000001` |
| node | `1e-7` |

**Mechanism:** `collect_object_enumeration_iteration_items`
(`crates/kali_codegen/src/intrinsics/object.rs`) pushes the key *node* into the
iteration items, and it is rendered downstream with the expression-slot renderer
rather than through `format_js_number`.

**This is a value defect, not a rendering one** — `Object.keys` returns strings,
so a wrong key text propagates into comparisons, lookups and JSON round-trips.
That distinction is why the console-render-unification project declined it: its
spec confines it to rendering, and the verification surface (for-of, spread,
object round-trips) is much wider than the one-line fix suggests.

**Suggested home:** §2, silent.

### 2.6 Static `Map`/`Set` lookups fold to a placeholder

```js
console.log(new Map([[5,"a"],["5","b"]]).get(5));   // kali 0, node a
console.log(new Map([[1e21,"a"]]).get(1e21));       // kali 0, node a
```

Silent, exit 0. The static fold reaches a placeholder `0` before key identity is
consulted, so both the number/string collision and the large-magnitude case
return `0` rather than the stored value.

Note this makes the SameValueZero half of R-56's `-0n` rationale *theoretical* —
that lookup never reaches a text comparison today. The console-rendering half of
that rationale stands on its own.

**Suggested home:** §2, silent.

### 2.7 Duplicate object-literal keys resolve to the first binding

```js
console.log({a:1, a:2}.a);   // kali 1, node 2
```

Silent, exit 0. Shape-independent — identifier, string and numeric keys behave
the same. JavaScript specifies last-wins.

**Suggested home:** §2, silent.

### 2.8 `Object.hasOwn` on a BigInt key answers `false`

```js
console.log(Object.hasOwn({42n:1}, 42n));   // kali false, node true
```

Silent, exit 0. **Mechanism:** HIR's `lower_property_name`
(`crates/kali_hir/src/lowering/object.rs`) destroys the key — a BigInt property
name does not survive lowering in a form the probe can match.

**Related to R-56** and fixed by the same upstream change: preserving whether a
`PropertyName` was `Number`, `String` or a BigInt through lowering. See §3.

### 2.9 The concat lane fails to compile a full order of magnitude before the ECMAScript threshold

```js
console.log("v=" + 1e20);
```

| | output | exit |
|---|---|---|
| kali | `error[E4201]: failed to load WASM module: failed to compile` | nonzero |
| node | `v=100000000000000000000` | 0 |

**Loud, not silent.** Recorded here because it corrects a natural assumption: the
break is *not* keyed on ECMAScript's `1e21` exponential threshold. R-55's entry
bisects it to two consecutive doubles straddling `i64::MAX`
(`9223372036854775000` compiles, `9223372036854776000` does not), which is a
different constant with a different cause.

**Already covered** by R-55's concat lane; listed for completeness so a reader
measuring `1e21` does not conclude the boundary is the JS one.

### 2.10 `var x = 1e21; console.log(x)` still prints expanded digits

```js
var x = 1e21; console.log(x);   // kali 1000000000000000000000, node 1e+21
```

Silent, exit 0. This is the **binding** lane of R-55. The console-render-
unification project fixed R-55's *direct-log* lane (`console.log(1e21)` →
`1e+21`) and left this one open.

**Recorded in R-55's entry**, but flagged here because of a structural oddity the
register discloses in §0.2: R-55 lives in §7 ("fail-loudly-but-wrong defects —
not silent"), carries no §0.2 row and therefore **no oracle case**, so this
silent lane is measured by nothing and cannot go red if it regresses. R-55's own
entry says so. Splitting R-55 into a §2 silent entry and a §7 loud one is the
filed follow-up.

### 2.11 `+42n` does not throw

```js
console.log(+42n);   // kali prints 42n; node throws TypeError
```

kali exits 0 having printed a value; node raises
`TypeError: Cannot convert a BigInt value to a number`. A missing-throw defect,
not a rendering one.

### 2.12 Unary minus on a BigInt-looking *string* renders as a BigInt

```js
console.log(-"42n");   // kali -42n, node NaN
```

Silent, exit 0. **Both the old and new answers are wrong** — before the
console-render-unification project this printed `-42`. The static fold's unary
arm is type-blind for strings: its BigInt guard runs on the *rendered* text,
which for a string literal has already had its delimiters stripped, so it cannot
distinguish `42n` from `"42n"`. The limitation is documented in place at
`crates/kali_codegen/src/intrinsics/host.rs`; narrowing the guard would restore a
differently-wrong answer rather than a right one.

### 2.13 `console.log(-0)` prints `0` — probably **not** a defect

```js
console.log(-0);   // kali 0, node -0
```

Recorded to stop it being re-filed. `String(-0)` genuinely **is** `"0"` in
JavaScript; node's `-0` comes from `util.inspect`, not from string coercion.
Matching it means adopting inspect semantics for the console sink, which is
R-31's territory and an explicit non-goal of the console-render-unification spec
(§2).

## 3. The upstream fix that retires several of these at once

§2.5, §2.8 and R-56 share one root cause: **`kali_hir`'s `lower_property_name`
discards whether a `PropertyName` was `Number`, `String` or a BigInt**, storing
only text. Everything downstream then reasons about a type distinction that no
longer exists, using textual conventions that invert between the object-literal
key slot and the expression slot.

The evidence that patching the consumer does not converge is on the record:
`canonical_property_key_text`
(`crates/kali_codegen/src/intrinsics/object.rs`) took **five successive fix
rounds** during the console-render-unification project, each closing one spelling
and revealing another, before converging on a round-trip invariant — and R-56
remains open because the one spelling it cannot close is the one where HIR's
marker and the key's own content are the same character.

Preserving that one bit upstream would retire the `KeyTextSlot` enum, the
`is_hir_numeric_key_spelling` predicate, its NaN guard and the guard's fragile
coupling to the parser, and make the whole collision family decidable.

**Do not add callers to `canonical_property_key_text` before that happens.** Its
two current call sites are safe only because they are adjacent and were written
together; a third elsewhere in the compiler is round six.

## 4. A gap in the register's own measuring instrument

Not a compiler defect. Found by filing R-56 honestly, and worth more attention
than any single entry above.

The blast-radius catalogue lets an entry declare itself `uncountable` with a
free-text reason. Three facts, each verified in source:

- `catalogue.rs` checks only that the reason string is **non-blank**. Nothing
  checks whether it is *true*.
- `score.rs`'s aggregation `try_fold`s over `Option`, so **one** uncountable
  member makes its whole cluster uncountable.
- `score.rs`'s `dominates` returns `false` whenever either side is `None`, so an
  uncountable cluster **can never be dominated** — and therefore lands in band 1
  by construction, regardless of its actual frequency.

So filing an entry `uncountable` is free, undetectable, and promotes it to the
top band. Filing it `countable` — as R-56 was — costs a matcher in
`matchers.mjs`, a re-freeze of two SHA-pinned files, and a regeneration of
`counts.json`.

The incentive points the wrong way, and no gate opposes it. The reasoning is
recorded permanently in R-56's instrument commit message and in
`crates/kali_blast_radius/src/manifest_tests.rs`'s frozen-SHA comment; this is
the pointer to it from outside that commit.

**Suggested fix:** a gate that requires an `uncountable` reason to name a
mechanism the corpus cannot express, or — cheaper and probably better — treat
uncountable clusters as unranked rather than undominated, so declaring one costs
visibility instead of granting it.

## 5. What filing any of these obliges

Recorded because the console-render-unification project discovered it the
expensive way while filing R-56, and the next person should not have to.

A new **§2** entry is not one edit. It requires, at minimum: the entry body; a
§0.2 row; **oracle cases in both scopes** backing that row, or
`every_zero_two_row_is_the_class_set_its_live_cases_assert` fails; a
`predicates.json` catalogue record, or `check_completeness` fails; a matcher in
`matchers.mjs` if the record is countable; **re-pinning both SHA-frozen
constants** and regenerating `counts.json` with its own tool; a `clusters.json`
membership; the hard-coded totals in `crates/kali_blast_radius/src/oracle_tests.rs`
and `register_tests.rs`; the register's §1 severity table and its numbering-note
re-count series; and a regeneration and re-splice of `blast-radius-ranking.md`.

A **§7** entry is much cheaper — no §0.2 row, no oracle case — which is exactly
why §2.10 above ended up unmeasured. Cheapness is not a reason to choose §7.

Per the repo's spec §4.3, the instrument half of that work belongs in its own
commit.
