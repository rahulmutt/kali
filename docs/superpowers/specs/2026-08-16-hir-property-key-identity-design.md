# Property-key identity: restore the discriminator upstream

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `5c80f081d2` (main, clean tree) |
| kali binary | `kali 0.1.0`, `.cache/cargo-target/debug/kali`, `cargo build -p kali_cli --bin kali` reporting nothing to rebuild at that commit |
| oracle | `node v26.7.0` |
| measured on | 2026-08-16 |

Every transcript in §2 was taken at that baseline for this document. Nothing here
is copied from an earlier report, and where this document contradicts an existing
one, §2 is the reading and the other document is the memory.

**One such contradiction, stated up front.**
`docs/superpowers/followups/console-render-unification-discovered-defects.md`
§2.8 attributes the BigInt-key loss to `kali_hir`'s `lower_property_name`. The
measurement in §2.2 below puts it one crate earlier, in the parser, and shows
that the key is not merely lost but replaced with `0`. That correction moves
part of the fix out of HIR and into `kali_parser` and `kali_ast`.

## 1. What this project is

The console-render-unification project's follow-up file (§3, *"The upstream fix
that retires several of these at once"*) names one root cause behind three
divergences: **`kali_hir` stores a property key's text without storing what kind
of key it was**, so every consumer downstream re-derives the type from quoting
conventions that invert between slots.

That file also records the evidence that consumer-side repair does not converge:
`canonical_property_key_text` took five successive fix rounds during that
project, each closing one spelling and revealing another, and **R-56** remains
open because the one spelling it cannot close is the one where HIR's marker and
the key's own content are the same character.

This project restores the discriminator where the type is still known, and
retires the machinery that guesses it.

**In scope.** R-56, and the two unfiled divergences that share the root cause
(§2.5 and §2.8 of the follow-up file). BigInt property keys become representable
rather than fail-closed.

**Out of scope, and not blocked by this work.** §2.7 (duplicate object-literal
keys resolve first-wins), §2.3 (bracket-form assignment to an existing key does
not take), §2.6 (static `Map`/`Set` lookups fold to a placeholder). None shares
this root cause; each stays measured in the follow-up file.

## 2. The defect family, measured

All five transcripts at the §0 baseline. `|` separates output lines.

### 2.1 R-56 — a quoted-numeric string key is indistinguishable from the number

```js
const o={'"5"':1}; console.log(o['"5"'], Object.hasOwn(o,'"5"'), Object.hasOwn(o,5));
```

| | output | exit |
|---|---|---|
| kali | `1 false true` | 0 |
| node | `1 true false` | 0 |

Both `hasOwn` answers are wrong, at exit 0, in a program whose member read of the
same key still returns `1`. This is register entry **R-56** (§2, Tier 2), with
oracle cases `r56a_..._module_scope` and `r56a_..._in_function` in
`crates/kali_cli/tests/cases/oracle/tier2.toml`.

### 2.2 A BigInt key is replaced by the key `0`

```js
const o={42n:1}; console.log(Object.hasOwn(o,42n), Object.hasOwn(o,0), o[0], o[42]);
```

| | output | exit |
|---|---|---|
| kali | `false true 1 0` | 0 |
| node | `true false undefined 1` | 0 |

The follow-up file's §2.8 records only the first field. The second and third are
the finding: the property is not dropped, it is **stored under the key `0`**, so
a program can read a value out of a key it never wrote.

**Mechanism, read in source.** `kali_lexer/src/number.rs:54-58` consumes a
trailing `n` into the `NumericLiteral` token, so the parser receives the text
`42n`. `kali_parser/src/expression/object.rs:52` is

```rust
.and_then(|token| token.value.parse::<f64>().ok())
.unwrap_or(0.0)
```

`"42n".parse::<f64>()` fails, and the fallback fabricates the key. `PropertyName`
(`kali_ast/src/literal.rs:37`) has no variant that could hold the digits.

### 2.3 Object enumeration leaks Rust's `Display for f64`

```js
for (const k of Object.keys({1e-7:1})) console.log(k);
for (const k of Object.keys({[-1e999]:1, [1e21]:2})) console.log(k);
```

| | output | exit |
|---|---|---|
| kali | `0.0000001` — then `-inf` \| `1000000000000000000000` | 0 |
| node | `1e-7` — then `-Infinity` \| `1e+21` | 0 |

The follow-up file's §2.5 records the `1e-7` lane only. The `-inf` lane is
measured here for the first time and is worse: `-inf` is not a string any
JavaScript program can produce, and `Object.keys` returns strings, so the wrong
text propagates into comparisons, lookups and JSON round-trips.

`Object.hasOwn` repairs both of these today (`Object.hasOwn({[-1e999]:1},
"-Infinity")` is `true`, measured), which is a fair measure of how narrow the
existing repair is: one consumer of the key text, out of several.

### 2.4 The control that must not move

```js
const o={5:1}, p={"5":1}; console.log(Object.hasOwn(o,5), Object.hasOwn(p,5), Object.hasOwn(o,"5"));
```

Both agree on `true true true`. JavaScript gives the numeric key `5` and the
string key `"5"` the same property name, and a discriminator that separated them
would fix §2.1 by breaking this. It is pinned in §6.

## 3. The root cause

`kali_hir/src/lowering/object.rs:20`:

```rust
PropertyName::Identifier(value) => alloc_text(Literal, value.clone()),
PropertyName::Number(value)     => alloc_text(Literal, format!("\"{}\"", key)),  // key = Rust Display
PropertyName::String(value)     => alloc_text(Literal, value.clone()),
```

Two decisions here produce the whole family:

1. **The type is encoded as a quote character.** In a key slot, `"` means *this
   was a number, already stringified*. But a string key's own content may begin
   and end with `"`, and then the two are the same text. No predicate at any
   consumer can recover the difference, which is R-56.
2. **The number is rendered with Rust's `Display`**, not with JavaScript's
   `String(number)`. That is where `0.0000001`, `-inf` and the expanded `1e21`
   come from.

`kali_codegen/src/intrinsics/object.rs` then spends `KeyTextSlot`,
`is_hir_numeric_key_spelling`, a NaN guard, and roughly sixty lines of comment
reconstructing what those two decisions destroyed — and the comment at `:127-139`
states in terms that the remaining hole cannot be closed at that level.

## 4. The design

**The invariant.** *An HIR key-slot node's text is the property name itself —
`String(key)` — for every shape of key, with no quoting marker.*

Today that slot means three different things depending on a quote character.
After this change it means one, so both sides of a key comparison are in one
currency by construction rather than by agreement.

### 4.1 `kali_ast`

`PropertyName` gains a BigInt variant holding the literal's digits **as text**:

```rust
pub enum PropertyName {
    Identifier(String),
    Number(f64),
    String(String),
    BigInt(String),
}
```

Text, not a parsed value: a BigInt key can exceed `f64`'s exact range, and the
point of the variant is that the key survives. No external schema exposes this
type (`schemas/` covers artifacts, diagnostics, envelopes and policies), so the
variant is an internal change whose consumers the compiler enumerates.

### 4.2 `kali_parser`

The numeric-key arm (`expression/object.rs:49-56`) gains a BigInt branch on the
token's trailing `n`, and **the `unwrap_or(0.0)` fallback becomes a fail-closed
diagnostic** — the same `push_feature_unavailable` path the computed-key arm
already takes when it cannot fold a key, so the refusal is an existing surface
rather than a new error code. Fabricating a key for a literal the compiler could not read is the
same defect class this project exists to close; leaving it in place while fixing
its most visible instance would trade one silent wrong answer for a quieter one.

In `computed_object_property_name` (`:136`, unary arm at `:172-191`), unary `-` over a BigInt yields
the negated digits. Unary `+` over a BigInt **declines** — `+42n` is a
`TypeError` in JavaScript, so refusing the program is the honest answer and
inventing a key is not.

### 4.3 `kali_hir`

`lower_property_name` computes the name once, where the type is still in hand:

| shape | text stored |
|---|---|
| `Identifier` / `String` | the name, verbatim |
| `Number` | `format_js_number(value)` |
| `BigInt` | the digits |

`kali_common::js_number::format_js_number` is documented as rendering a value
"the way JavaScript's `String(number)` does" — `±0` to `0`, `1e21` to `1e+21`,
`1e-7` to `1e-7`, infinities to `Infinity`/`-Infinity`. It is the function the
console-render-unification project introduced to end this species of divergence,
and `kali_hir` already depends on `kali_common`. Using it here is what makes
drift between the key's name and the number's rendering structurally impossible
rather than merely tested.

The `format!("\"{}\"", key)` quoting and the
`if *value == 0.0 { "0" } else { value.to_string() }` `Display` call both go.

### 4.4 `kali_codegen` — the collapse

`static_property_key_text(id, KeyTextSlot::ObjectLiteralKey)` becomes "read the
literal's text", because the text is already the name. That takes with it:

- `KeyTextSlot` — one remaining variant is not a slot;
- `is_hir_numeric_key_spelling`, its NaN guard, its `-0` branch and its
  round-trip-through-HIR's-writer invariant;
- the comment block at `:107-153` defending a convention that no longer exists.

`canonical_property_key_text` survives as the **expression-side** function only:
quoted means string content, an `n` suffix means BigInt digits, bare means a
number to render. That side was never the defect and its convention is
invertible.

### 4.5 `Object.keys` needs its own fix, not just the new text

`collect_object_enumeration_iteration_items`
(`intrinsics/object.rs:759`, key push at `:803`) pushes the key **node** into the iteration
items, so the item is later rendered by the expression renderer, which reads a
bare text as a number. The fold must instead push a string-carrying node built
from the key text.

Canonical text alone would make the printed output right by coincidence —
`console.log("5")` and `console.log(5)` are indistinguishable at the sink — while
`typeof k` stayed wrong. `Object.keys` returns strings; the fold must produce
strings.

### 4.6 The `trim_matches('"')` classification rule

`key.trim_matches('"')` appears at **14 non-test sites** across six files:
`kali_optimize/src/object_fold.rs` (8), `kali_codegen/src/intrinsics/object.rs`
(2), `kali_codegen/src/lower.rs`, `kali_codegen/src/emit/call.rs`,
`kali_mir/src/analysis/infer.rs`, and `kali_common/src/object.rs`.
(`kali_fmt/src/formatter.rs` also trims quotes, on source literal text; it is not
in this currency and is not in scope.)

After this change a key-slot text never carries quotes, so each trim is either
dead or actively harmful — it conflates the string key `'"5"'` with the numeric
key `5`, which is R-56's collision surviving at a second address.

**The rule is classify, not sweep.** Each site's input currency is established
first: `object_literal_field` and `apply_timeline_mutation` are fed raw HIR text
by some callers and rendered probe text by others, and
`intrinsics/object.rs:559` already documents one such asymmetry as deliberate.
Delete the trim where the input is provably key-slot text; leave it and record
why where it is not. **This design does not promise that all fourteen fall.**

`kali_common::object::property_order_key`'s doc comment asserts the cross-layer
contract in terms — *"LIR literal text keeps source quoting, while AST/repr key
text is unquoted; both layers must classify identically"* — and must be rewritten
to state what is true afterwards, whichever way its own trim is classified.

### 4.7 Edge cases

| key | text after | why |
|---|---|---|
| `{[-1e999]: 1}` | `-Infinity` | `format_js_number`, not `Display`; closes §2.3's second lane |
| `{[1e21]: 1}` | `1e+21` | the ECMAScript exponential threshold, from `ryu-js` |
| `{[-0]: 1}` | `0` | `format_js_number` folds both zeros, matching `String(-0)` |
| `{NaN: 1}` | `NaN` | an identifier key, stored verbatim; the guard that existed to reject it deletes with the round trip |
| `{[0/0]: 1}` | unchanged | arithmetic never folds to a property name; stays fail-closed |
| `{123456789012345678901234567890n: 1}` | exact digits | text, never an `f64` |
| `{'"5"': 1}` | `"5"` (three chars) | a string key's content, verbatim — distinct from `5` for the first time |
| `{0x10: 1}` | unchanged | lexes as `0` then identifier `x10`; already fails loudly (`E3100`), a separate defect |

## 5. Non-goals

- **No new key normalization elsewhere.** The 14 trim sites are classified, not
  unified behind a new shared helper. A workspace-wide key-normalization pass
  moves `object_fold`, escape/repr inference and ES enumeration ordering at once,
  which is a wider verification surface than this root cause needs.
- **No `Repr::Boolean` work.** `Object.hasOwn(...)` printed from inside a
  function renders `1` rather than `true`; that is the missing boolean repr axis
  (follow-up file §2.4, register R-34), and it will make some fixed cases here
  still look wrong at a console sink. Cases are written to avoid depending on it.
- **No change to the expression-slot convention.** It is invertible and correct.

## 6. Verification

### 6.1 Pin before touching, in both scopes

R-56's two oracle cases already exist and assert today's SILENT class, so they go
red on the fix — that is the signal, not a failure.

§2.2 and §2.3 have nothing, so they get cases in the `object/` family of
`crates/kali_cli/tests/cases/` **before** any source change, recording today's
answers exactly as §2 measured them: `hasOwn(o,0) === true`, `o[0] === 1`,
`o[42] === 0`, `-inf`, `0.0000001`. A fix that flips a test nobody wrote is a fix
nobody measured.

Both scopes — module and in-function — per the corpus norm, because top-level and
in-function are different programs in kali.

### 6.2 The two controls

- §2.4's `{5:1}` vs `{"5":1}` — must stay **identical**. An over-eager
  discriminator separates them and breaks conforming JavaScript.
- §2.1's `{'"5"':1}` vs `{5:1}` — must become **distinct**, all three fields
  matching node.

### 6.3 Re-pin discipline

The case corpus is 287 files expanding to 5,587 trials, and this change moves a
text that `kali_optimize`, `kali_mir`, `kali_types` and `kali_codegen` tests
encode in expectations — anything spelling a numeric key as `"5"`, or an infinite
key as `-inf`.

Every re-pin is classified as **was wrong, now right** or **was right, now
spelled differently**, in the commit that makes it. An unclassified re-pin is how
a regression enters wearing a fix's clothes; the console-render-unification
project's ~160 re-pins set the precedent.

## 7. Ledger obligations

**R-56 closes by re-derivation, not by editing a verdict.** §0.2 of the register
is generated from the oracle cases. The sequence is: fix, re-measure, regenerate
§0.2, then record the closure in R-56's §2 body against the commit that closed
it.

**The ranking regenerates.** R-56 leaves the SILENT filter, so
`cargo run -p kali_blast_radius --example rank` is re-run and
`blast-radius-ranking.md`'s generated region re-spliced;
`spliced_document_matches_the_generator` fails until it is. Expected: ranked
entries 28 → 27, §2 clusters 17 → 16 as R-56's singleton leaves, and **no band
moves**, since R-56 measures 0 reachable / 0 raw. Expected, not asserted — §6.6
item 4 of that document says *re-run, do not re-read*, and the amendment records
what the generator actually printed.

Catalogue and `clusters.json` handling for a retiring entry follows the precedent
set by R-33's retirement at `3a636f62fb`, not a new convention. Per the ranking
spec's §4.3, any instrument change lands as **its own commit**, ahead of the
change that motivates it.

**§2.5 and §2.8 do not become register entries.** They are fixed on the same
branch that would file them, and §5 of the follow-up file prices a new §2 entry
at roughly a dozen coordinated edits. They get regression cases (§6.1) and a
fixed-at-commit record in the follow-up file instead.

**The follow-up file is corrected**, not just annotated: §2.5 gains the `-inf`
and `1e21` lanes measured in §2.3; §2.8's attribution moves from
`lower_property_name` to the parser, with the key-`0` transcript; §3 records that
the upstream fix happened and what it retired.

## 8. Risks, and what would falsify this design

1. **A consumer depends on the quoting to mean something other than "number".**
   The design's answer is §4.6's classification pass. If a site turns out to need
   the type at a point where only text is available, that is evidence for
   approach B (carry the type as data through the IRs), and the design should be
   revisited rather than patched with a second convention.
2. **The re-pin surface is larger than §6.3 anticipates.** Mitigated by pinning
   first: the corpus is run before any source change, so the moved set is
   measured rather than discovered.
3. **BigInt keys reach code paths that assume `f64`-shaped keys.** The new
   variant lands in exhaustive matches — `kali_types` already declines
   `PropertyName::Number` at `monomorphize.rs:964`, `late_host.rs:290` and
   `repr_infer.rs:1716` — so the compiler enumerates the sites, but each needs a
   decision rather than a copied arm.
4. **`Object.keys` order.** ES enumeration puts array-index-like keys first;
   `property_order_key` classifies them from text. `{5:1}` reaches it as `5`
   instead of `"5"` after this change, which its own trim already normalized to
   the same answer — but the ordering is pinned by cases before the change so any
   movement is visible rather than inferred.
