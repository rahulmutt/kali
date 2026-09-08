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
does not appear below. R-56 was RETIRED 2026-08-16 at `12fd424897` by the
hir-property-key-identity project — see §3.

**Update, 2026-08-16, by the hir-property-key-identity project.** §2.5 and §2.8
are now fixed, and §3's proposed fix has happened; each was re-measured against
`node v26.7.0` on a binary built at `a7ea7b0cf7`, not carried forward from the
original `5aebc5ec3d` baseline above. The rest of this file was re-read, not
re-measured line by line, against the same change; §2.1-§2.4, §2.6-§2.7 and
§2.9-§2.13 were checked and found still true and are unchanged, as is §4.

**Update, 2026-09-08, by the register-property-key-followups branch — and read
this before the "None of them is filed" sentence above.** That sentence is still
true **of the thirteen rows in §2 of this file**, none of which has been filed.
It is no longer true of the property-key family this file's §2.5, §2.8 and §3
belong to: the hir-property-key-identity project measured four FURTHER
divergences that are not rows here, left them unfiled because filing is a
human's decision, and on 2026-09-08 the human asked for them to be filed. Two of
the four are now §2 Tier-2 register entries — **R-57** (a property key spelled
with an escape sequence is stored undecoded) and **R-58** (a legacy-octal
numeric key is read as decimal, in the same nine-line parser function §2.8's
corrected attribution names) — filed at `dde0f083c0` with every reading
re-measured there against `node v26.8.1`. The remaining two are the next task on
that branch. Nothing in §2 of this file moved, and §5's obligation list was
exercised twice by that filing; what it turned out to be missing is recorded at
the foot of §5.

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

### 2.5 `Object.keys` yields a Rust-formatted numeric key — FIXED at `4a69275c63`

```js
for (const k of Object.keys({1e-7: 1})) console.log(k);
for (const k of Object.keys({[-1e999]: 1, [1e21]: 2})) console.log(k);
```

Re-measured at `a7ea7b0cf7` against `node v26.7.0`:

```
$ kali run keys_small_module.js      $ node keys_small_module.js
1e-7                                 1e-7

$ kali run keys_extreme_module.js    $ node keys_extreme_module.js
-Infinity                            -Infinity
1e+21                                1e+21
```

**This row originally recorded only the `1e-7` lane.** This project also
measured the `-1e999` lane, and it is the worse of the two: before the fix,
`Object.keys({[-1e999]: 1, [1e21]: 2})` printed `-inf` for the first key — a
string no JavaScript program can produce, since no numeric literal or
coercion in JS spells negative infinity that way — followed by
`1000000000000000000000` for the second. Both now agree with node.

**Fixed** by `4a69275c63`: `lower_property_name` now stores a key's text as
`format_js_number(key)` — the same formatter `console.log` itself uses —
instead of Rust's `Display for f64`, so `collect_object_enumeration_iteration_items`
reads the JS spelling out of the key node directly rather than the expression-
slot renderer's Rust spelling.

**Why printed output alone could not have caught the string-vs-number half.**
`Object.keys` must return strings, but `console.log("5")` and `console.log(5)`
print the identical line — so a key that silently arrived as a `Repr::Number`
rather than a string would be invisible to every transcript in this row. A
`typeof` probe closes that gap, but `typeof` on a value bound directly to a
for-of loop's key can itself be constant-folded at compile time and print
`string` even when the underlying item is not (confirmed by contrast: routing
the same value through a non-foldable function-parameter boundary instead
makes kali's runtime `typeof` answer `0`, which turned out to be a separate,
pre-existing, general defect — `typeof` on any runtime-string function
parameter answers `0` in kali today, nothing to do with property keys). So the
string-ness claim rests on non-foldable evidence — `.length` and strict
equality both ways, routed through that same function-parameter boundary — not
on `typeof` alone.

**Pinned by** (`crates/kali_cli/tests/cases/object/property_key_identity.toml`):
`object_keys_renders_small_magnitude_key_with_rust_display_module_scope` and
`_in_function` for the `1e-7` lane;
`object_keys_leaks_rust_infinity_spelling_module_scope` and `_in_function` for
the `-1e999`/`1e21` lane; and `object_keys_yields_strings_not_numbers_module_scope`
and `_in_function` for the `typeof` probe (with
`object_keys_yields_strings_for_bigint_keys_module_scope` and `_in_function`
covering the same probe for a BigInt key).

### 2.6 Static `Map`/`Set` lookups fold to a placeholder

```js
console.log(new Map([[5,"a"],["5","b"]]).get(5));   // kali 0, node a
console.log(new Map([[1e21,"a"]]).get(1e21));       // kali 0, node a
```

Silent, exit 0. The static fold reaches a placeholder `0` before key identity is
consulted, so both the number/string collision and the large-magnitude case
return `0` rather than the stored value.

Note this makes the SameValueZero half of this document's own `-0n` rationale
*theoretical* — that lookup never reaches a text comparison today. The
console-rendering half of that rationale stands on its own. (This is this
document's own editorial gloss, not a quote of R-56's register entry: `grep -n
"SameValueZero\|-0n"` on `kali-silent-miscompile-register.md` returns nothing
at any revision -- R-56's entry is entirely about a string key colliding with
a numeric key. R-56 is also RETIRED as of `12fd424897`, so this cross-reference
points at a closed entry regardless.)

**Suggested home:** §2, silent.

### 2.7 Duplicate object-literal keys resolve to the first binding

```js
console.log({a:1, a:2}.a);   // kali 1, node 2
```

Silent, exit 0. Shape-independent — identifier, string and numeric keys behave
the same. JavaScript specifies last-wins.

**Suggested home:** §2, silent.

### 2.8 `Object.hasOwn` on a BigInt key answers `false` — FIXED at `20e2de09f6`, one residual disclosed

```js
const o = {42n: 1};
console.log(Object.hasOwn(o, 0));
console.log(o[0]);
console.log(o[42]);
```

Re-measured at `a7ea7b0cf7` against `node v26.7.0`:

```
$ kali run bigint_module.js   $ node bigint_module.js
false                         false
0                              undefined
1                              1
```

**Corrected attribution.** This row originally named HIR's `lower_property_name`
as what destroyed the key. That named the wrong crate. The key was destroyed
one crate earlier, in the parser: `kali_parser/src/expression/object.rs`'s
numeric-key arm read a key token with `token.value.parse::<f64>().ok().unwrap_or(0.0)`,
which silently substituted the key `0` for any numeric-literal token that
would not parse as an `f64` — every BigInt token, `42n` included — before
`lower_property_name` ever saw it. `{42n:1}` was never a BigInt key that HIR
lost; it was stored under the key `0` from the moment the parser read it, and
`Object.hasOwn(o, 42n)` answered `false` because the object genuinely had no
property named `42`.

**Fixed** by `20e2de09f6`: `PropertyName` gained a `BigInt` variant holding the
literal's exact digits (`"42"`, no `n` suffix — text, because a BigInt past
`f64` precision has no exact double), and the `unwrap_or(0.0)` fallback is
gone; a numeric key the parser genuinely cannot read is now refused through
`push_feature_unavailable` instead of being silently rewritten to `0`.
`Object.hasOwn(o, 0)` now correctly answers `false` and `o[42]` now correctly
reads `1` — both agree with node.

**A SECOND, STILL-OPEN DIVERGENCE LIVES IN THE SAME NINE-LINE FUNCTION, and is
now filed as R-58.** The corrected attribution above names
`kali_parser/src/expression/object.rs`'s numeric-key arm as where the BigInt key
was destroyed. `numeric_property_name`, the function that arm calls, has two
branches: the BigInt one, which `20e2de09f6` guarded against a leading zero
(`042n` is refused, matching JavaScript's real SyntaxError), and an `f64` one
three lines below it that parses a numeric key's digits with Rust's
`str::parse::<f64>` and therefore has no legacy-octal grammar at all. `{042: 1}`
is the property `42` in kali and `34` in node, at exit 0. Measured at
`dde0f083c0` against `node v26.8.1` in both scopes and **filed 2026-09-08 as
R-58** (§2, Tier 2). It is the sibling of the divergence this row records —
the same function, the other branch — and the fix `20e2de09f6` shipped did not
and could not reach it.

**Residual, not closed by this fix, and not property-key identity.** `o[0]`
still reads `0` where node reads `undefined`. The reason has changed: `{42n:1}`
now genuinely has no property named `0`, so this is the pre-existing
fabricated-`0` static-member-read defect (the same defect this project met
again on the `quoted_key_member_probe_*` cases below). It is not chased here.

**Do not call that defect an "absent-property read" — it is wider than that.**
Corrected by the hir-property-key-identity branch's final whole-branch review
and measured at that branch's HEAD, both scopes: `const o =
Object.fromEntries([["a", 1]]); console.log(o.a)` prints `0` in kali and `1` in
node, at exit 0. `o` HAS an own property named `a`; the read still fabricates
`0`, because under `kali run`'s Fast mode the `fromEntries` fold never runs and
the member read has no statically known shape to resolve against. So the `0` is
what an unresolvable static member read emits, present property or not — a
wrong VALUE, not only an `undefined`-rendered-as-`0`. Pinned as
`a_present_property_on_a_from_entries_object_also_reads_the_fabricated_zero_*`
in `crates/kali_cli/tests/cases/object/property_key_identity.toml`.

**Pinned by** `bigint_key_is_stored_under_zero_module_scope` and
`bigint_key_is_stored_under_zero_in_function` in
`crates/kali_cli/tests/cases/object/property_key_identity.toml`.

**Related to R-56**, RETIRED at `12fd424897`, and fixed by the same upstream
project. See §3.

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

## 3. The upstream fix that retires several of these at once — DONE

§2.5, §2.8 and R-56 shared one root cause: **`kali_hir`'s `lower_property_name`
discarded whether a `PropertyName` was `Number`, `String` or a BigInt**, storing
only text. Everything downstream then reasoned about a type distinction that no
longer existed, using textual conventions that inverted between the
object-literal key slot and the expression slot.

The evidence that patching the consumer would not converge is on the record:
`canonical_property_key_text`
(`crates/kali_codegen/src/intrinsics/object.rs`) took **five successive fix
rounds** during the console-render-unification project, each closing one spelling
and revealing another, before converging on a round-trip invariant — and R-56
stayed open through all five rounds because the one spelling none of them could
close was the one where HIR's marker and the key's own content were the same
character.

**The fix landed 2026-08-16, at `4a69275c63`.** `lower_property_name` now
stores `String(key)` — `format_js_number` for numbers, digits for BigInts, the
name verbatim otherwise — with no quoting marker, so a key-slot node's text is
the property name for every shape of key it can express a type for. Two
supporting fixes went with it: `20e2de09f6` (earlier the same day) removed the
parser-side `unwrap_or(0.0)` fallback that had been fabricating the key `0` for
any numeric-literal token it could not parse as an `f64` — the actual cause of
§2.8, one crate below where this section originally looked — and gave
`PropertyName` a `BigInt` variant so a BigInt key's digits survive parsing at
all; `28a28b88e1` moved the computed-member-index probe side onto the same
`format_js_number` formatter the key-slot side now uses, closing a lane the
first commit had newly opened (`{1e21:1}` stored the key `1e+21` while
`o[1e21]` probed for `1000000000000000000000`).

**What it retired:** the `KeyTextSlot` enum, the `is_hir_numeric_key_spelling`
predicate, its NaN guard, the guard's fragile coupling to the parser, and the
parser's fabricated-key fallback (`unwrap_or(0.0)`) that produced §2.8's `0`.
`c4245eac62` and `12fd424897` then deleted the fourteen `trim_matches('"')`
un-marking sites that existed only to undo the marker `lower_property_name` no
longer writes.

**`canonical_property_key_text` survives, narrowed to one meaning.** It is no
longer one of two functions reasoning about key text from opposite ends; it is
the PROBE side alone — the property key an expression evaluates to, computed
the way JS does. Its own doc comment now states this: "Only one currency
exists now: a key-slot node's text is already the property name
(`kali_hir`'s `lower_property_name`), so this function is for the PROBE side
alone." It has exactly **one non-test call site**
(`FunctionEmitter::static_probe_key_text`, `crates/kali_codegen/src/intrinsics/object.rs`).

**Do not add callers to `canonical_property_key_text`.** The warning does not
expire with the fix — it changes shape. Before the fix, a second caller risked
re-deriving a type guess this function's *convention* had to invert correctly.
After the fix, a second caller risks reintroducing exactly the bug this
project fixed twice (§2.5, §2.8, and the member-read regression `28a28b88e1`
caught before it shipped): a PROBE computed through this function and a STORED
key-slot text computed through a *different* formatter, agreeing only by
coincidence until one of the two changes. The stored side never needs this
function — it reads a key-slot node's text directly, unmodified — and any code
that finds itself wanting to call this function on a key-slot node instead of
reading its text is the sign the invariant is about to be re-broken.

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

**What this list turned out to be missing, found by using it — 2026-09-08, at
`dde0f083c0`, filing R-57 and R-58.** The list above is accurate and was followed
step by step; it is also incomplete, and every item below cost a red gate or a
stale sentence to discover. A future filer should treat the list above as the
floor, not the ceiling.

- **`crates/kali_blast_radius/src/catalogue_tests.rs` asserts a record COUNT**
  (42 -> 44 here), beside the `check_completeness` call the list already names.
  Two constants, not one.
- **A singleton cluster needs a `clusters.json` cluster DEFINITION as well as an
  assignment.** The list says "a `clusters.json` membership", which reads as one
  edit; `ranking.rs`'s "an empty cluster ranks nothing" assertion has its mirror
  image — an assignment naming an undeclared cluster — and a singleton needs
  both halves. R-56's RETIREMENT already recorded this asymmetry in the removal
  direction; this is the addition direction.
- **A countable record needs an `UPPER_BOUNDS` entry in `count.mjs`** wherever
  the matcher is wider (or narrower) than the defect, and a test in
  `matchers.test.mjs` — including the catalogue-name-count assertion inside it,
  which is a bare number (38 -> 40 here) and fails with no explanation of which
  file to edit.
- **The ranking needs a §6 AMENDMENT, not only a re-splice.** §2-§5 are
  generated and the gate holds them; §6 is authored, nothing checks it, and it is
  where what the generator PRINTED gets recorded. A re-splice with no amendment
  passes every gate and loses the reading.
- **`oracle_tests.rs` carries the case total TWICE** — once in the assertion and
  once in a doc comment above it — and only one of them is what the compiler
  checks.
