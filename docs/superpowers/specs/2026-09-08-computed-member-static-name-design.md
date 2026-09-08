# Computed member access: decline upstream, fold at the twins, refuse at one gateway

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `dc19c3a040` (main, clean tree, the merge of PR #35) |
| kali binary | `kali 0.1.0`, `.cache/cargo-target/debug/kali`, rebuilt by `cargo build -p kali_cli --bin kali` at that commit before every transcript below was taken |
| oracle | `node v26.8.1` |
| measured on | 2026-09-08 |

Every transcript in §2 was taken at that baseline for this document. Nothing is
carried over from the register's entries; where a reading here agrees with one
there, both were measured, and where this document contradicts an existing one,
§2 is the reading and the other document is the memory.

**One such contradiction, stated up front.** The register's R-13 entry describes
its write lane (`o[k] = 8` vanishes) as the variable-key half of one defect. §2.3
below shows the store vanishes for a *literal* key too (`o["b"] = 8`, `a[1] = 9`),
so R-13's write lane belongs to the bracket-store family the console-render
follow-up recorded as its item 2.3 and left out of scope, not to the fabricated
name at all. This project closes both because they share a fix, not because
they share a cause.

## 1. What this project is

The parser turns a computed member index into the property name it denotes, but
only for the shapes it can read. For every other shape it does not decline; it
**fabricates** a name: the identifier's own text for `o[i]`, the literal string
`index` for anything else. Register entry **R-59** traces that name end to end
into the static lanes that read it, and shows that **R-13**'s eleven-week
mechanism hypothesis ("an admit-list falls through to a default `0`") was wrong:
there is no admit-list, and the `0` is the fabricated name *missing*. Add a
property named `k` to R-13's repro object and the `0` becomes that property's
value.

This project makes an index the compiler cannot read into a value it **refuses**,
rather than a name it invents. It does that in three moves:

1. **Decline upstream.** The parser returns `Option<String>`, and the AST's
   member node carries `Option<String>`, so absence is a type every consumer
   must decide about, not a string that happens to collide.
2. **Fold at the twins.** The checker and codegen, the two passes that own
   bindings, fold a compile-time-constant `const` index to its name through the
   resolvers they already have, and from that point treat the access exactly as
   its dot or literal spelling.
3. **Refuse at one gateway per side.** Anything neither a runtime lane nor the
   fold admits is an `E5506`, with one message, in both `kali check` and
   `kali run`.

**In scope.** R-59 (whole entry). R-13 (both lanes). The bracket-store family,
follow-up item 2.3, which R-13's write lane turns out to be a member of.

**Out of scope, and not blocked by this work.** The absent-property read after a
successful fold (R-21's lane). A runtime string-keyed lookup. A runtime index
lane for array *literals*. Folding boolean, `null`, or BigInt indices. The unary
catch-all's placeholder tail (R-21's and R-60's site). Each is named in §5 with
the measurement that puts it outside.

## 2. The defect family, measured

Every program below was run at the baseline in module scope; each was also run
inside `function main() { … }` with a trailing `main();` and read byte-identically
unless a row says otherwise. Exit codes are `0` on both engines unless shown.

### 2.1 R-59 — the fabricated name hits a real property

```js
const o = {index: 9, i: 7}; let i = 1;
console.log(o[i]);        // kali 7   node undefined
console.log(o[i + 0]);    // kali 9   node undefined
```

`o[i]` reaches the parser's `Identifier` arm and probes for the property named
`i`; `o[i + 0]` reaches the catch-all and probes for `index`. Both properties
exist only so the two arms can be told apart. `kali check` accepts the file.

### 2.2 R-13 — the fabricated name misses, and the `0` is the miss

```js
const o = {a:1, b:2}; const k = "b";
console.log(o[k]);        // kali 0   node 2
console.log(o[k] + 1);    // kali 1   node 3
```

Identical with `let k` and `var k`. The literal spelling `o["b"]` prints `2` on
both engines, and `o["c"]` prints `0` against node's `undefined`, which is the
same absent-read lane `o.zz` reaches (`0` against `undefined`). So after this
project folds `k` to `"b"`, a hit is correct and a miss is R-21's `0`, exactly
as the literal spelling behaves today.

### 2.3 The write lane is the bracket-store family, not the fabricated name

```js
const o = {a:1, b:2}; const k = "b"; o[k] = 8;   console.log(o.b);   // kali 2  node 8
const o = {a:1, b:2};                o["b"] = 8; console.log(o.b);   // kali 2  node 8
const o = {a:1, b:2};                o.b = 8;    console.log(o.b);   // kali 8  node 8  ← control
const a = [5, 6];                    a[1] = 9;   console.log(a[1]);  // kali 6  node 9
```

The dot spelling is the control that localizes it: the *bracket* store falls out
of every store arm in codegen and the caller turns the unmatched assignment into
a bare read of the target. `kali check` accepts every line. Folding `k` to `"b"`
would therefore land on a lane that is itself broken; §4.4 routes every
static-name bracket store through the dot-store arm, which closes the literal
spelling as a side effect.

### 2.4 Array and string receivers

```js
const a = [5, 6];   let i = 1;   console.log(a[i]);   // kali 0  node 6
const a = [5, 6];   const i = 1; console.log(a[i]);   // kali 0  node 6
const a = [5,6,7];  for (let j = 0; j < 3; j++) console.log(a[j]);  // kali 0 0 0  node 5 6 7
const s = "abc";    const k = 1; console.log(s[k]);   // kali 0  node b
const s = "abc";                 console.log(s[1]);   // kali 0  node b
```

An array can hold no property named `i`, so the fabricated name always misses
and the read is the placeholder `0`. A `const` index folds to the literal
spelling, which the static element fold already reads correctly (`a[1]` → `6`).
A mutable index has no admitting lane over an array literal and refuses (§5).
The string receiver's *literal* spelling is already a silent `0`; §4.4 refuses
both spellings rather than fold onto a lane that is wrong.

### 2.5 The controls that must not move

- `o.b` reads and `o.b = 8` writes on a `const` object literal (§2.3's control).
- `o["b"]`, `o[1]` on `{1: "one"}`, `o[(1)]`, `o[(0, 1)]`, `o[+1]`: the parser's
  readable set, all correct today.
- `const a = new Array(3); for (let i = 0; i < 3; i++) a[i] = i * 2;` then
  `a[i]`: the linear-memory runtime lane, which reads the index child and agrees
  with node. Its checker twin is the runtime-array binding registry.
- `Object.hasOwn(o, k)` with `const k = "b"` prints `true`: the key-argument lane
  already folds a `const` string binding, through the same resolver §4.4 uses.

### 2.6 Check and run do not agree today

`kali check` runs the checker only; codegen never runs. Every program in §2.1–2.4
passes `kali check`, and a program codegen refuses at `run` (a function reading a
module `const` object, `E5506`) passes `check` too. A refusal that lives only in
codegen is therefore invisible to the canonical static surface. That is why the
checker and codegen halves of §4.4 are twins.

## 3. The root cause, traced

1. **The parser fabricates.** `expression_to_property_name`
   (`crates/kali_parser/src/literal.rs`) returns `String` and has no way to say
   "I cannot read this". Its `Identifier` arm returns the identifier's text; its
   sequence-empty, unary-parse-failed and catch-all arms return `"index"`.
2. **Both bracket parse sites store it as the name.**
   `crates/kali_parser/src/expression/call.rs` at the plain `o[e]` site and the
   optional-chain `o?.[e]` site build a `MemberExpression` whose `property` is
   that string, keeping the real index in `computed_index`.
3. **HIR carries both.** `crates/kali_hir/src/lowering/expression.rs` allocates
   the member node with the name as text and pushes the index as a second child.
   MIR erases the node kind and preserves the text; LIR carries both onward.
4. **The static consumers read the text.** Codegen's two-child member arm
   (`crates/kali_codegen/src/emit/control_flow.rs`) tries the runtime lanes and
   then hands the node to `emit_unary`, whose catch-all resolves the receiver
   aggregate and looks the text up with `object_literal_field`
   (`crates/kali_codegen/src/intrinsics/object.rs`). A hit emits the wrong
   property's value; a miss falls to the placeholder `0`. The static array fold
   (`static_member_index` in `crates/kali_codegen/src/emit/call.rs`) requires
   all-digit text, so a fabricated name always misses there.
5. **Passes without bindings read it too.** MIR's arena and escape analyses
   recognize `join`/`substring`/host methods and look up struct-field layouts by
   the text. The optimizer's `member_access_name` builds `object.name` strings
   for the `Object.hasOwn` and `Object.keys` folds without checking the child
   count. The checker's name-string lanes (`member_access_name`, the late-host
   classifiers) build the same strings, so `Object[k]` with a fabricated `"k"`
   is classified by name.
6. **The checker never reads the name for a shape check.** Every shape/field
   consumer in `kali_types` is guarded by "not computed". The computed lane is
   handled structurally by `reject_nonuniform_forin_key_object_access`, which
   only fires for an identifier index over a *proven* object shape; a fold-lane
   object literal is not one, so §2.1–2.4 pass.
7. **The store has no lane.** `emit_assignment`
   (`crates/kali_codegen/src/emit/literal.rs`) has arms for the for-in ordinal
   store and the runtime-array element store; a bracket store on a fold-lane
   aggregate matches neither and reaches the `assignment_target_name`
   fallthrough, which returns `false` for `=`, and the caller emits a bare read.
   The checker's assignment visitor treats every computed target as an array
   element store, so no object write access is recorded and nothing
   materializes.

## 4. The design

### 4.1 `kali_parser`

`expression_to_property_name` returns `Option<String>`. Its readable set is
**exactly today's**: a string literal (normalized), a number literal rendered by
`format_js_number` (the same formatter HIR stores a numeric key with, so probe
and key stay one currency), and the parenthesized, sequence-last and `+`/`-`
unary forms that recurse into one of those. Every other arm returns `None`: a
bare identifier, any binary expression, an empty sequence, a unary whose
recursive result does not parse as a number, and the boolean, `null`, BigInt and
regex literals. There is no fallback string in the function.

Both bracket parse sites store the option directly. `computed_index` is
unchanged.

The function's doc comment is rewritten to state the contract and drops its
"recommendation, not done here" paragraph.

### 4.2 `kali_ast`

```rust
pub struct MemberExpression {
    pub object: Expression,
    /// `Some(name)`: the property name JavaScript will read is statically
    /// known — always for dot access, and for a computed access only when the
    /// parser could read the index. `None`: computed access whose index must
    /// be evaluated; `computed_index` is `Some`.
    #[serde(default)]
    pub property: Option<String>,
    #[serde(default)]
    pub computed_index: Option<Box<Expression>>,
}
```

`property` gains `#[serde(default)]` so an absent field deserializes as
`None`. Two accessors:

- `dot_name(&self) -> Option<&str>`: the name only when `computed_index` is
  `None`. For the sites already guarded by "not computed".
- `static_name(&self) -> Option<&str>`: the name for dot access or a readable
  computed access. For the name-string lanes that build dotted names.

A consumer with neither accessor matches the option itself. That is the point:
there is no way to read a name that is not there.

The three consumer doc comments that cite the parser function (codegen's object
intrinsics, the optimizer's object fold and its helpers) are corrected: the
"one currency" claim now holds for every computed access that *has* a name,
because a computed access without a readable index has none.

### 4.3 `kali_hir`, `kali_mir`, `kali_lir`, and the passes without bindings

HIR allocates the member node with text only when the AST has a name, otherwise
without text; the index is still pushed as the second child. MIR and LIR already
carry text as `Option<String>`. Downstream, a computed member is still "a
two-child `Value` whose text is not a binary operator", and a textless two-child
node satisfies that test, so it routes to the computed-member arms. Every
transparent-wrapper recognizer inspected requires exactly one child; §6 carries
a unit test that pins that rather than relying on inspection.

Passes with no binding knowledge **decline** on a nameless member, which is
always sound:

- MIR arena and escape analysis: no text → the existing fail-closed
  classification (heap, not scalar; not a whitelisted host call). A folded
  `const` key gets less analysis than its literal spelling would. That is a
  conservative loss, recorded, not a defect.
- The optimizer's `member_access_name` and `normalized_member_access_name`
  return `None` on a nameless member, so `Object[k]` can no longer fold as any
  static method.
- The CLI's module linker already refuses computed namespace access; its
  `freeze` recognizer and the export-signature namer go through `static_name`.

None of these folds a `const` key. The fold lives only in the twins.

### 4.4 The twins

Both twins apply one rule to a computed member with **no name**, in the read
path and the assignment path, in this order:

1. **Runtime lanes first, unchanged.** A for-in key over the base's own shape is
   decided by the existing for-in gate (checker) and for-in ordinal lane
   (codegen). A runtime array receiver — a linear-memory array binding, a
   growable array, a string-element array — is decided by the existing
   dynamic-index lanes. Existing denies (crypto and search-params results) fire
   where they fire today.
2. **Fold.** The index is resolved to a name: a string through the existing
   static string resolver, a number through the existing static numeric
   resolver rendered with `format_js_number`. **An identifier folds only when
   it is a `const` declarator.** Codegen's `bindings` map is `const`-only by
   construction; the checker's static-value tables also admit `let`, so the
   checker must additionally check the declarator kind. A `let` key, even one
   never reassigned, does not fold, because the two twins must admit the same
   set.
3. **Dot semantics from here.** With a name, the access is the dot spelling.
4. **Refuse.** Otherwise `E5506`, one canonical message used verbatim by both
   twins:

   > computed member access `o[k]` is unavailable in the current phase unless
   > the index is a literal or a compile-time-constant `const` binding, or the
   > receiver is a runtime array or a `for..in` key over the same object

**Checker (`kali_types`).** In `resolve_member_expression`, after the existing
for-in gate. A folded read records the same deferred object access a dot read
records; a folded store records the same write access, so the object
materializes on the evidence `o.b = 8` uses today. The assignment visitor's
computed-target branch, which currently treats every `o[expr] = v` as an array
element store, is narrowed to receivers that are arrays; an object receiver
follows steps 2–4. Compound stores (`o[k] += 1`) follow the same path as `=` and then inherit
the dot spelling's behaviour, which today is an `E5506` refusal for a `const`
object-literal target (`o.b += 5` refuses at the baseline; measured). The
update form `o[k]++` keeps its existing refusal.

**Codegen (`kali_codegen`).** One resolver, `static_member_name(node)`: the
node's text if present, else the fold of the index child through the `const`
binding resolvers (`resolve_static_numeric_value` and a string counterpart over
`resolve_bound_node`). It has exactly two callers.

- *Read gateway.* In the two-child member arm, after every runtime lane has
  declined and before `emit_unary`. With a name, the gateway records it on the
  node **in place**, so every later by-id consumer sees the same name, and
  re-dispatches through the same arm, which now behaves as if the parser had
  read the index: the object-literal field fold, the static element fold, and
  the existing miss behaviour all apply unchanged. Without one, `E5506` and an
  `unreachable`; `emit_unary` never sees a nameless member. The catch-all's
  placeholder tail is untouched.
- *Store choke point.* In `emit_assignment`, before the `assignment_target_name`
  fallthrough. A two-child target on a non-array receiver resolves its static
  name; with one, the store is emitted by the dot-store arm as if spelled
  `o.b = v`; without one, `E5506`. An array-literal receiver has no store arm
  and refuses. Runtime arrays keep their element-store lane.
- *String receivers.* `s[1]` is a silent `0` today, and a folded `s[k]` would
  land on the same lane. The gateway treats a statically-known string receiver
  as a receiver with no admitting lane: `E5506` for both spellings. No character
  fold is added; that widening belongs to the string helper slices.

### 4.5 Edge cases

- **Optional chain.** `o?.[i]` fabricates through the second parse site today
  and is fixed by the same change; `o?.[k]` with a `const` key folds.
- **Absent property after a fold.** `const k = "c"; o[k]` prints `0` against
  node's `undefined`, exactly as `o.c` does. R-21's lane; R-13's oracle asserts
  a present property, so the verdict is unaffected.
- **`o[""]`.** A string literal, readable, `Some("")`. There is no sentinel
  value; that is why the field is an `Option`.
- **`delete o[k]`.** No member-text lane; unchanged, already fail-closed.
- **A fold that hits the for-in gate's shape.** Step 1 runs first, so a
  `const` key over a proven object shape is decided by the fold only if the
  for-in gate did not claim the identifier as a for-in key.

## 5. Non-goals, each with the measurement that puts it outside

| lane | today | after | why not here |
|---|---|---|---|
| absent property after a fold, `o[k]` with `k = "c"` | `0` | `0` | R-21: `o.c` prints the same `0`; the fold makes the two spellings agree |
| `let k = "b"; o[k]` | `0` | `E5506` | checker could fold, codegen cannot; refusing is the only shared answer |
| `for (let j…) a[j]` over an array literal | `0 0 0` | `E5506` | the linear-memory lane exists for `new Array(n)` receivers; extending it to literals is its own design |
| `s[k]`, `s[1]` on a string | `0` | `E5506` | a character fold is a string-helper widening, not a member-access fix |
| `o[true]`, `o[null]`, `o[1n]` | `5` (fabricated `index`) | `E5506` | the parser's readable set does not grow; a decline closes R-59's lane honestly |
| the unary catch-all's placeholder `0` | `0` | `0` | R-21's and R-60's site; this project stops nameless members from reaching it, and changes nothing about what reaches it with a name |
| a runtime string-keyed object lookup | none | none | needs a key table per shape at runtime; the for-in ordinal lane is the only precedent and it is deliberately narrow |

## 6. Verification

### 6.1 Pin before touching, in both scopes

Every program in §2 is pinned as a case before any source changes, asserting
today's output, so the fix wave reads as verdict flips rather than new tests.

### 6.2 Oracle flips, re-measured before edited

- `r59a_*` (both scopes): `silent` → `fail_closed`. The rationale's own
  regression note names this as one of the two outcomes that close R-59.
- `r13r_*` and `r13w_*` (both scopes): `silent` → `fixed`. The write case's
  `.b` read-back is the dot lane, so it proves the store landed.
- `classifier_ground_truth.toml`'s SILENT row: R-13's `var`-key read becomes
  `fail_closed`, so the classifier loses its SILENT fixture. It is replaced by
  **R-10**'s repro (`var x = 1; { let x = 2; } console.log(x)`), measured SILENT
  first, with the header table and the rationale saying why R-13 left the class.
  R-10 is chosen because its fix is architectural and no project is near it.

### 6.3 Wrong-on-purpose pins flip

The `computed_member_index_is_fabricated_from_the_index_expression_*` pair in
`crates/kali_cli/tests/cases/object/property_key_identity.toml` becomes a
refusal assertion with its rationale rewritten. Follow-up item 2.3's three lines
get regression cases asserting node's value.

### 6.4 New case file, `crates/kali_cli/tests/cases/object/computed_member_static_name.toml`

The admitted matrix, for read and for store, both scopes: dot; string literal;
number literal; `const` string key; `const` number key; parenthesized, sequence
and unary forms of each; optional-chain spelling. The refused matrix, each with a
`check` step and a `run` step asserting the same `E5506` and the same message,
and the JSON form of each through `--output json`: `let` key; `var` key;
parameter key; binary index; `true`, `null`, BigInt; string receiver (both
spellings); array-literal store; array-literal read at a mutable index. The
controls of §2.5 are asserted unchanged.

### 6.5 Unit tests, in sibling `*tests.rs` files

- Parser: `expression_to_property_name` returns `None` for every unreadable
  shape in §4.1 and the recorded value for every readable one.
- AST: the two accessors.
- Codegen: `static_member_name` folds a `const` string and a `const` number,
  declines a `let` and a parameter; every transparent-wrapper recognizer
  declines a two-child node.
- Checker: the fold declines a `let` declarator; a folded store records a write
  access.

### 6.6 Re-pin discipline

`cargo test --workspace` after the AST change. The mechanical edits to test
struct literals (`property: X` → `property: Some(X)`, roughly 450 sites, almost
all in `kali_types`'s test modules) land in **their own commit** so the behaviour
diff is readable. Fixtures that pinned a zero from a computed read or a dropped
bracket store go red; each is re-pinned to the refusal or the correct value,
and any that pinned a wrong value on purpose gets its rationale rewritten. None
is deleted, none is re-pinned silently. Budget a dedicated re-pin commit, as
R-35 and the console-render project both needed.

## 7. Ledger obligations

**R-13 and R-59 close by re-derivation, not by editing a verdict.** Fix,
re-measure, regenerate §0.2 of the register from the oracle cases, then record
each closure in its own §2 entry against the closing commit. R-13's mechanism
bullet, struck and "disproved, not replaced", is replaced with the traced
mechanism from §3 and the note that its write lane was item 2.3's family.

**The ranking regenerates.** Both entries leave the SILENT filter, so
`cargo run -p kali_blast_radius --example rank` is re-run and the generated
region re-spliced; `spliced_document_matches_the_generator` fails until it is.
Expected, not asserted: G3 keeps R-12 alone and drops from 45 to R-12's own
count; R-59's singleton cluster is removed; R-13's contested-assignment row
leaves §2.4. §6 of that document gets a dated amendment recording what the
generator printed. §6.6 item 4 of the ranking: re-run, do not re-read.

**`clusters.json` and the instrument.** Both entries leave `clusters.json`,
following R-56's retirement at `12fd424897`. `predicates.json`, `matchers.mjs`
and `counts.json` are not touched; both matchers stay so a regression re-lights
the count. No §4.3 apparatus commit is expected. If one becomes necessary, it
lands alone, ahead of the change that motivates it.

**The console-render follow-up** marks item 2.3 fixed at the closing commit and
names this project.

**The plan.** `plan/phase-21/README.md` §21.3 gains one progress line naming the
admitted and refused sets. No maturity row changes: computed member access was
never documented as available.

## 8. Risks, and what would falsify this design

- **The twins disagree.** The checker admits a receiver codegen refuses, or the
  reverse. The check-versus-run parity step on every refusal case is the
  detector. If they cannot be made to agree on the `const`-only rule, the
  design is wrong about the admitted set, not about the shape.
- **A textless two-child node is misread as a wrapper.** The recognizer unit
  test is the detector. If it fails, the node needs a kind marker that survives
  MIR's erasure, which is a larger change than this design prices.
- **The dot-store arm needs materialization evidence the folded store does not
  produce**, so `o[k] = 8` refuses instead of working. R-13's write lane then
  closes `FAIL_CLOSED` rather than `FIXED`, which is honest and must be recorded
  as such in the entry and the oracle rationale, not smoothed over.
- **Fixture churn exceeds the budget.** The console-render ladder change turned
  164 tests red at once. The re-pin commit is budgeted; if the red set is
  dominated by one lane, that lane may be a pre-existing defect to file rather
  than a fixture to re-pin.
- **A pass that declines on `None` was load-bearing for a green lane.** A
  `const` key that used to fold through the optimizer's name lane by accident
  (a fabricated name that happened to be right) stops folding. That is the
  design working; the case that goes red is re-pinned to the twins' answer.
