# The fourteen `trim_matches('"')` key-text sites, classified

Project: `docs/superpowers/specs/2026-08-16-hir-property-key-identity-design.md`
(Task 5). Sites enumerated at `f563a0ecf4`, the commit after Task 3 made an HIR
key-slot node's text the property name itself (`String(key)`, no quoting
marker).

## 1. The question each row answers

After Task 3 a key-slot text never carries quotes, so un-quoting one is either
dead or actively harmful: it conflates the string key `'"5"'` with the numeric
key `5`, which is R-56's collision at a second address. The design does not
promise that all fourteen fall, so each site was classified by reading its
CALLERS and asking one question: **does the text arriving here come from a key
slot, from a probe/rendered expression, or from both?**

**Read §6 before relying on the invariant.** "A key slot's text IS the property
name" is true for every shape of key EXCEPT one whose source spelling contains
an escape sequence, which is stored undecoded. That exception is stated once,
in full, in §6, and every claim elsewhere in this document is subject to it.

The census is the `grep` the task brief specifies, minus `kali_fmt`:

```
grep -rn "trim_matches('\"')" --include="*.rs" crates/ | grep -v "_tests\|/tests"
```

15 lines. `kali_fmt/src/formatter.rs:433` is out of scope — it trims a source
literal's own text in the formatter, a different currency entirely. The other
fourteen are below.

## 2. What made eight of them look load-bearing

Eight sites (all of `object_fold.rs`) read LIR object-literal key text, and LIR
key text has one producer for source literals — HIR, verbatim through MIR and
LIR, both of which copy `node.text` unchanged — but a SECOND producer for
synthesized literals: `Optimizer::push_object_literal`. Its two callers were:

* `fold_object_from_entries_call`, which built key texts from
  `constant_property_key` → `literal_text`, the literal SPELLING: a string key
  came back re-quoted and re-escaped (`"b"`, and `"\"x\""` for `'"x"'`);
* `apply_timeline_mutation`, which reuses existing key texts and appends the
  mutation's member text — already property names, both.

So the first producer was a genuine second currency reaching those eight sites,
and deleting their trims alone would have broken `Object.fromEntries`
enumeration. Rather than keep eight un-quoting sites to serve one producer —
and rather than invent a second marker convention, which the design's §8 risk 1
forbids — the producer was moved onto the key-slot currency:
`constant_property_key` now returns `String(key)` (`constant_value_property_name`
in `constant_fold.rs`), spelled with the same `format_js_number` that
`lower_property_name` and `expression_to_property_name` use. The key's TYPE is
still available there (it is a `ConstantValue`), so this spends the
discriminator where the type is known, exactly as the design says to.

That producer change also fixed a live Release-mode defect that Task 3 had
opened and nothing measured: `fold_object_has_own_call` compared the quoted
probe text against a key slot's text, so after Task 3 an `Object.hasOwn` with a
**string-literal probe** over a **source** object literal, folded at
`Release`/`ReleaseAdvanced`, would have answered `false`. Two neighbouring
shapes were unaffected and it is worth being precise about which: a NUMERIC
probe was fine (`literal_text(Number(1))` is `"1"`, which already matched the
slot text `format_js_number` stores), and a `fromEntries`-SYNTHESIZED object was
self-consistent (both its stored keys and the probe came through
`constant_property_key`, so they agreed with each other) — which is exactly why
the four `from_entries` tests in this area are untouched and were green
throughout. The optimizer's own tests did not catch the string-probe case
because their fixtures still spelled key slots the pre-Task-3 way (`"1"`, `"2"`);
those fixtures are corrected by this task and now model what the front end
actually produces, and a negative assertion was added on that lane (§7).

## 3. The table

All fourteen fell. `input currency` names what reaches the site; the `why`
column names the other side of the comparison (the Ruling-5 enumeration).

| file:line (at `f563a0ecf4`) | input currency | action | why |
|---|---|---|---|
| `kali_optimize/src/object_fold.rs:156` | key-slot text (source literals and `push_object_literal` synthesized ones) | deleted | `__proto__` guard in the enumeration fold. Other side is the constant `"__proto__"`. Every spelling of the prototype setter — `__proto__`, `"__proto__"`, `'__proto__'`, and a `fromEntries` entry keyed by the string — reaches this as the same nine-character name, so the guard still matches all of them. See §4. |
| `kali_optimize/src/object_fold.rs:165` | key-slot text | deleted | `Object.keys` element rendering. The trim fed `format!("{:?}", …)`, so the OTHER side is the array element a downstream string read decodes. With the trim, `{'"2"': 'a'}` enumerated as `2`; without it the element encodes the real name and prints `"2"`, matching node. |
| `kali_optimize/src/object_fold.rs:174` | key-slot text | deleted | `Reflect.ownKeys`, identical to `:165`. |
| `kali_optimize/src/object_fold.rs:195` | key-slot text | deleted | `Object.entries` key element, identical to `:165`. |
| `kali_optimize/src/object_fold.rs:748` | member-access text (`dot_member_base_and_key`) | deleted | Timeline `__proto__` guard. Other side is the constant `"__proto__"`. `kali_parser`'s `expression_to_property_name` already reduced `x.__proto__` / `x["__proto__"]` / `x['__proto__']` to the same name. |
| `kali_optimize/src/object_fold.rs:756` | member-access text vs key-slot text | deleted | `delete x.k` retain. Both sides are property names by construction (member text is `String(index)`; the stored key IS the name). With the trims, `delete x.a` erased the unrelated own property `'"a"'`. |
| `kali_optimize/src/object_fold.rs:762` | member-access text vs key-slot text | deleted | `x.k = v` in-place update; same two sides as `:756`, same collision. |
| `kali_optimize/src/object_fold.rs:920` | member-access text (`member_mutation_base_node`) | deleted | Timeline-eligibility `__proto__` disqualifier. Other side is the constant `"__proto__"`. See §4. |
| `kali_codegen/src/intrinsics/object.rs:86` | probe/rendered names only | deleted | The `field` parameter of `object_literal_field`. Enumerated callers: `emit/operators.rs:664` and `:865` (member node text = `expression_to_property_name`), `emit/operators.rs:526` (a normalized static index, decimal), `emit/object.rs:104` (a `kali_types` shape field name, which `repr_infer.rs::record_object_literal` takes from `PropertyName::Identifier|String` — numeric and BigInt keys never reach the shape lane), `emit/call.rs:4072` (member text). All are `String(key)`. |
| `kali_codegen/src/intrinsics/object.rs:96` | key-slot text | deleted | The stored side of the same comparison. Together with `:86` this is the measured `p['a']` divergence: two quote-stripped sides made `{'"a"': 1}` answer a probe for `a`, inventing a property in a program whose `Object.hasOwn` already said `false`. |
| `kali_common/src/object.rs:9` | key-slot text and `kali_types` shape field names | deleted | `property_order_key`'s ES-order classification. Other side is nothing — it classifies a single text — but its callers (`sort_properties_es_order` from `kali_types::monomorphize`/`repr_infer`, and `kali_optimize`'s `object_property_order_key`) all pass property names. Doc comment rewritten (brief Step 5): the old contract it asserted, "LIR literal text keeps source quoting, while AST/repr key text is unquoted", has not been true since Task 3. |
| `kali_mir/src/analysis/infer.rs:134` | HIR key-slot text (via `layout_field_name`) | deleted | The layout lane's private copy of the same classifier; `layout_field_name` reads the ObjectProperty's key child's text. Same single-text classification, same rewrite. |
| `kali_codegen/src/emit/call.rs:4893` | key-slot text | deleted | Builds `program_fn_valued_property_names`. Other side: `emit/call.rs:4579` tests a member callee's `text` against this set — a property name. Both sides unquoted. |
| `kali_codegen/src/lower.rs:4895` | key-slot text | deleted | `resolve_object_literal_properties`, feeding the BigInt shape-field taint set. Other side: `taint_shape_fields_from_object_inflow` inserts into a set keyed by `repr_table.shape_fields(shape)` names — `kali_types` property names. Both sides unquoted. |

**Kept: none.** Every input was a property name once the one non-key-slot
producer (§2) was moved onto the same currency.

## 4. The two `__proto__` guards

`object_fold.rs:166` (enumeration fold, `fold_object_enumeration_call`) and
`:943` (timeline eligibility, `collect_permitted_occurrences`) are
prototype-pollution defences: `__proto__` is JavaScript's prototype setter, not
an own property, so folding an enumeration over a literal that carries one would
emit a phantom own key. Removing a trim changes what a guard matches, so both
were reasoned about explicitly rather than swept.

**These two guards are one of two doors, and the other stands open today.**
The enumeration-fold guard fires by DECLINING — it returns `None` so the call
is left unfolded and falls through to the codegen backstop — and that backstop
enumerates object properties with no equivalent guard of its own. So the guard
does its job and the phantom key appears anyway. Measured at this commit, under
`kali run`: `{"__proto__": 1, a: 2}` prints `__proto__` then `a`, where node
prints only `a`. (Note the guard is NOT Release-gated: `optimize_program_
internal` calls `fold_object_enumeration_calls_ordered` before the
`OptimizationLevel` match, `crates/kali_optimize/src/driver.rs:185`, so it runs
under Fast too — unlike `fold_object_has_own_call` and the `fromEntries`
binding path, which are the Release-only folds §7 is about.) That phantom key
is a separate, pre-existing codegen-backstop defect — unowned by this project
and unchanged by it — not something the argument below closes. It matters
where it is stated: this section is about the two `object_fold.rs` guards
specifically, and readers should not come away thinking the
phantom-`__proto__`-key class is closed end to end.

**Both `object_fold.rs` guards still catch everything they caught** —
including against the escape exception of §6, which is checked rather than
assumed below. The guard compares against the constant `"__proto__"`, and
after Task 3 every real spelling of the setter arrives as that exact name:

* `{__proto__: 1}` → `PropertyName::Identifier("__proto__")` → slot text
  `__proto__`;
* `{"__proto__": 1}` / `{'__proto__': 1}` → `PropertyName::String` via
  `unquote_string_literal` → slot text `__proto__`;
* `Object.fromEntries([["__proto__", 1]])` → `constant_property_key` → the
  string's content, `__proto__`;
* `delete x.__proto__`, `x["__proto__"] = v`, `x['__proto__'] = v` → member text
  `__proto__` via `expression_to_property_name`.

The un-quoting could only ever ADD matches, never remove one, so no real
prototype-setter spelling is lost. The one match it added was wrong:
`{'"__proto__"': 1}` declares an ORDINARY own property whose name is the
eleven-character text `"__proto__"`, and the guard was swallowing it — node's
`Object.keys({'"__proto__"': 1})` is `['"__proto__"']`, while kali at
`f563a0ecf4` printed `__proto__`. A security carve-out was silently corrupting
a program it was never meant to cover.

### 4.1 Why §6's escape exception cannot reach these guards

The list above says a key slot's text is the property name, and §6 says that is
false for keys spelled with an escape sequence. A guard that compares a
possibly-undecoded text against `"__proto__"` could in principle fail in two
directions, so both were tested rather than argued.

**The allowlist has ELEVEN members, not four.** kali's lexer accepts
`n`, `t`, `r`, `\`, `"`, `'`, `` ` ``, `0`, `b`, `f`, `v`
(`crates/kali_lexer/src/string.rs:28`) and hard-errors on anything else.
Measured: `const o = {"_\_proto__": 1};` gives
`error[E1004]: unsupported string escape sequence at position 14`, exit 1 — an
escape outside the eleven does not compile. But `const o = {"a\nb": 1};`
compiles at exit 0, with no diagnostic — a supported escape is not rejected —
so the closing argument below has to hold for real, not merely because
"escapes are refused."

**Direction A — can a real `__proto__` key hide from the guard?** It would have
to be spelled with an escape, so that the STORED text differs from the decoded
name `__proto__`. It cannot, for a reason simpler than which characters the
eleven escapes decode to: per §6, `kali_parser`'s `unquote_string_literal`
never decodes ANY escape in a key slot — the delimiters are stripped and
nothing else — so a key spelled with an escape ALWAYS keeps a literal
backslash in its stored text, for all eleven, not just the previously-checked
four. `__proto__` contains no backslash, so no escaped spelling can ever equal
it. (The eleven also do not help an attacker even under real JavaScript
decode semantics: none of `\n \t \r \\ \" \' \` \0 \b \f \v` decodes to a
letter, digit, or underscore — the only characters `__proto__` is made of — so
no spelling containing one of these escapes can be the setter's name at all,
independent of whether kali decodes it.)

Therefore a key whose property name is `__proto__` is always spelled without
escapes, its stored text equals its name, and the guard fires.

**Direction B — can the guard fire on a key that is NOT the setter?** That needs
a stored text of exactly `__proto__` whose real name is something else. A stored
text differs from its name only when the source spelling contained a backslash,
and the text `__proto__` contains no backslash — so there was no escape, and
text and name are the same. No false positive.

Both guards therefore hold **in fact**, not merely by the unqualified invariant:
the escape exception is real (**and is now filed as R-57**, whose **Fix
direction** bullet carries this section's tripwire forward: Direction A's
argument below holds only while key slots are never decoded, and R-57's fix is to
decode them — which does not by itself falsify the guards, since the second
observation in that paragraph still holds, but does move them off a structural
fact and onto the allowlist's contents, so the commit that closes R-57 owes them
either a move onto the DECODED name or an explicit re-derivation), and it is unreachable from this comparison in
either direction — and the argument now rests on §6's "never decoded" finding
rather than on an escape count, so it does not need re-deriving if the
allowlist grows within the current single-character shape.

**A TRIPWIRE ON THE LEXER, recorded so it is not rediscovered as a
prototype-pollution bug, and now ENFORCED by a test, not only by prose.**
Direction A rests on two facts about the lexer that a future change could
silently break: that key slots are never decoded (§6), and that none of the
accepted escapes decode to a letter, digit, or underscore. Both are pinned by
`test_lexer_string_escape_allowlist_is_pinned_for_the_proto_guard` in
`crates/kali_lexer/src/engine_tests.rs`, whose failure message names both
`object_fold.rs` guards above and this section. What actually defeats Direction
A is a lexer change that (a) starts DECODING key-slot escapes, or (b) adds an
escape that decodes to a letter, digit, or underscore (`\u`, `\x`, or similar) —
either one lets an escaped spelling's real name coincide with `__proto__`
while the guard still compares undecoded, or pre-existing, text. Demonstrated
by construction — not by a lexer change, since neither exists today — with the
spelling a future `\u` escape would let through, respelling `__proto__`'s
leading underscore as `\u005f` (the Unicode escape for `_`):

```
const o = {"\u005f_proto__": 1, a: 2};
for (const k of Object.keys(o)) console.log(k);

  kali (today):  error[E1004]: unsupported string escape sequence at position 13   exit 1
  node:          a                                                                 exit 0
```

`\u005f` decodes to `_`; node reads the whole key as `__proto__`, treats it
as the PROTOTYPE SETTER, and enumerates only `a`. kali refuses this program
outright today — `u` is not in the eleven — so the guard is never reached,
sound but only because the spelling does not compile. **A change that adds
unicode or hex escapes to the lexer, or that starts decoding key-slot escapes
at all, must move these guards onto the DECODED name before it ships.** Today
they are sound because neither precondition holds, and the lexer test above is
what catches either one moving.

Separately, and NOT a hypothetical: the codegen backstop noted at this
section's opening — the lane the guard's own `return None` hands the call to —
already has no guard at all, for the unescaped setter spelling
`{"__proto__": 1, a: 2}`, no `\u` required. Measured, this commit, under
`kali run`:

```
const o = {"__proto__": 1, a: 2};
for (const k of Object.keys(o)) console.log(k);

  kali:  __proto__
         a                exit 0
  node:  a                exit 0
```

Tests: `does_not_fold_object_keys_over_a_proto_keyed_literal` (unchanged
assertion; its fixture was corrected to spell the key slot the way the front end
now does) and a new sibling,
`folds_object_keys_over_a_quoted_proto_named_key`, which pins the over-match as
gone. The pre-existing phantom-`__proto__`-key behaviour on the *codegen*
backstop lane is unchanged by this task and was verified identical at
`f563a0ecf4`; it is a separate, pre-existing defect on a different lane, stated
here at this section's opening rather than only in a closing footnote.

## 5. What moved, measured

Both programs were measured at `f563a0ecf4` and after, in module and function
scope, against node v26.7.0. Both are pinned in
`crates/kali_cli/tests/cases/object/property_key_identity.toml`.

```
const o = {foo:'b','"2"':'a'};
for (const k of Object.keys(o)) console.log(k);
  before  kali: 2, foo          node: foo, "2"     wrong TEXT and wrong ORDER
  after   kali: foo, "2"        node: foo, "2"     agrees

const p = {'"a"': 1};
console.log(p['"a"']); console.log(p['a']); console.log(Object.hasOwn(p,'a'));
  before  kali: 1, 1, false     node: 1, undefined, false
  after   kali: 1, 0, false     node: 1, undefined, false
```

The second one is partly fixed: the INVENTED property is gone (line 2 no longer
finds a property that does not exist), and what remains is the pre-existing
static-member-read defect that fabricates `0` — the same residual already
recorded in that file for `o[0]` on the BigInt case. It is pinned WRONG ON
PURPOSE with what would close it.

**That defect is wider than the name "absent-property read" this document and
the corpus previously gave it**, corrected by the final whole-branch review and
measured at this commit. The fabricated `0` is what a static member read emits
whenever it cannot resolve the object's shape, and that happens for properties
that are PRESENT too:

```
const o = Object.fromEntries([["a", 1]]);
console.log(o.a);

  kali:  0    exit 0
  node:  1    exit 0
```

`o` has an own property `a` worth `1`. The reason the read still yields `0` is
§7: `fold_object_from_entries_call`'s binding path is Release-only and `kali
run` is Fast, so no shape is ever materialized for the read to consult. On that
shape it is not an `undefined`-versus-`0` rendering divergence at all — it is a
wrong VALUE for a property that exists, at exit 0. Pinned in both scopes as
`a_present_property_on_a_from_entries_object_also_reads_the_fabricated_zero_*`
in `crates/kali_cli/tests/cases/object/property_key_identity.toml`. Unowned by
this project and unchanged by it; recorded so the residual is not restated more
narrowly than it is.

**FILED 2026-09-08 as R-60**, at `dde0f083c0`, on the human's instruction. This
passage stays as written — it is the history of how the divergence was found —
and the entry re-measured all of it rather than citing it. One sentence above is
narrowed there rather than repeated: *"no shape is ever materialized for the read
to consult"* is true of the READ and not of the object. On the same `o`, in the
same Fast-mode run, `Object.hasOwn(o, "a")` folds to `true` and
`Object.hasOwn(o, "b")` to `false` — both agreeing with node — because
`static_object_has_own` recognizes a `fromEntries` call operand directly, while
`Object.keys(o)` / `Object.values(o)` / `Object.entries(o)` refuse the program
(`error[E5506]`, exit 1, measured, directly and through the binding). The §7
Release-gating explanation therefore accounts for the ENUMERATION fold's absence
and not for the member read's answer, and the entry says which of the two it
measured: the Fast-mode lane, in both scopes, with no runner available on this
machine for a `--release` artifact.

## 6. The one exception to "a key slot's text is the property name"

**FILED 2026-09-08 as R-57, at `dde0f083c0`.** This section is where the
divergence below was first characterised, and it stays as written — its
measurements are the history of how it was found, and the two-mechanism reading
it works out (undecoded storage, then a second escaping pass) is the reading the
register entry adopts. What has changed is that it is no longer unfiled: the
human asked for it, and it is now **R-57** in §2 of
`docs/superpowers/followups/kali-silent-miscompile-register.md`, Tier 2, with a
§0.2 row, an `r57a` oracle pair in both scopes, a predicate record and a
`clusters.json` singleton. **The entry re-measured everything below at
`dde0f083c0` against `node v26.8.1` rather than citing it**, which is how one
number here was found to be describing a different lane: `k.length` is `6` in the
ITERATION lane, as this section says, but `Object.keys(o)[0].length` is `2` — the
array-element `.length` divergence, which is not this defect and not a
property-key phenomenon at all. Read the two closing paragraphs of this section
("Not fixed here, deliberately") with the entry, which now owns the fix
direction.

**ESCAPE SEQUENCES.** `kali_parser`'s `unquote_string_literal`
(`crates/kali_parser/src/literal.rs:7-24`) strips a string key's delimiters
WITHOUT decoding its escapes, so a key whose source spelling contains one is
stored undecoded:

```
const o = {"a\"b": 1};
for (const k of Object.keys(o)) { console.log(k, k.length); console.log(k === "a\"b"); }
console.log(o['a"b']);

  kali:  a\"b  6 / false   then  0      exit 0
  node:  a"b   3 / true    then  1      exit 0
```

The stored key slot holds the FOUR characters `a\"b` (the delimiters stripped,
the escape left alone); the property name is the three characters `a"b`. The
printed `6` above does not describe that stored text at all — it comes from a
second, independent escaping pass, below. What the four-character stored text
directly explains is two of the three wrong answers: a probe written with the
real name (`o['a"b']`) misses the undecoded four-character slot and falls into
the fabricated-`0` member-read lane described at the end of §5, and the
enumerated key prints with a stray backslash because the slot's text has one.

**The `6` is `fold_object_enumeration_call` re-escaping an already-undecoded
key.** Enumerating through `Object.keys` re-encodes the stored text with
`format!("{key:?}", …)` (`crates/kali_optimize/src/object_fold.rs:173,180,199`)
before handing it to the string reader — a SECOND escaping pass stacked on the
parser's already-undecoded text. That is what stretches the four-character
slot's `.length` to `6` for a key read out through enumeration; it is not a
property of the stored text itself, and it is a distinct divergence from the
undecoded-storage one, not a restatement of it.

**This falsifies the invariant as this project has been stating it.** "An HIR
key-slot node's text IS the property name (`String(key)`)" holds for identifier
keys, quoted keys without escapes, numeric keys, and BigInt keys — but not for
escaped ones. Every claim in this document, and the doc comments at the sites it
classifies, are subject to this exception; the load-bearing ones now say so
(`kali_common/src/object.rs`, `kali_codegen/src/intrinsics/object.rs` on both
`object_literal_field` and `static_object_has_own`, and
`kali_optimize/src/helpers.rs` on `constant_property_key`).

**Pre-existing, and measured so.** Identical at `f563a0ecf4` and after Task 5,
verified by building both commits in separate worktrees rather than by
inspection. Task 5's fourteen deletions and its one producer move do not touch
escape decoding.

**"Self-consistent" describes one comparison, not the key's behaviour.**
`static_object_has_own` folds `Object.hasOwn(o, "a\"b")` to `true` for this `o`
because its probe and the stored key are the SAME undecoded four-character
text — that comparison genuinely does not lie, and only because the two
spellings are byte-identical in source. It does not generalize: the same
object's enumerated key fails a strict-equality probe against the identical
literal. Measured in the same run: `k === "a\"b"` is `false` for `k` read out
of `Object.keys(o)`, even though `console.log(k)` shows text that reads as the
same spelling. One program giving `hasOwn` and `===` opposite answers about the
same nominal property, in a single run at exit 0, is R-56's own signature —
surviving in the lane this document previously called self-consistent.

**There IS a producer disagreement here, and it is currently unobservable.**
`constant_property_key` reads through `literal_value` →
`parse_string_literal` (`crates/kali_optimize/src/constant_fold.rs`), which DOES
decode `\\`, `\"`, `\'` and `` \` ``. So a `fromEntries` entry spelled
`["a\"b", 1]` yields the correct three-character name while a source literal
with the same spelling yields the four-character undecoded text. Re-measured
under `kali run` (`BuildMode::Fast`, what the corpus exercises):

```
for (const k of Object.keys(Object.fromEntries([["a\"b", 1]]))) console.log(k, k.length);
for (const k of Object.keys({"a\"b": 1}))                     console.log(k, k.length);

  kali:  a\"b  6   /  a\"b  6      exit 0
  node:  a"b   3   /  a"b   3      exit 0
```

Both lanes print the same thing, agreeing with the original claim — but the
explanation an earlier draft of this paragraph gave for it, that
`constant_property_key`'s fold is Release-only and "never runs under `kali
run`", is FALSE and is corrected here. `fold_object_enumeration_calls_ordered`
is called BEFORE the `OptimizationLevel` match
(`crates/kali_optimize/src/driver.rs:185`), and `fold_object_enumeration_call`
materializes a `fromEntries` operand by calling `fold_object_from_entries_call`
inline (`crates/kali_optimize/src/object_fold.rs:127`), which is what reaches
`constant_property_key`. That lane is reachable under Fast; §7's Release-only
claim covers `fold_object_has_own_call` and the BINDING path of
`fold_object_from_entries_call`, not this nested-call one.

What IS established by measurement is narrower: the decoded producer's output
does not reach the enumeration's output. A differential probe separates the two
possibilities, because a decoded key and an undecoded one would re-escape to
different lengths. With a lone backslash escape, a decoded `fromEntries` key
(`a\b`, three characters) would re-encode through `format!("{key:?}")` to a
four-character payload, while the undecoded source-literal key (`a\\b`, four
characters) re-encodes to a six-character one:

```
for (const k of Object.keys(Object.fromEntries([["a\\b", 1]]))) console.log(k, k.length);
for (const k of Object.keys({"a\\b": 1}))                     console.log(k, k.length);

  kali:  a\\b  6   /  a\\b  6      exit 0
  node:  a\b   3   /  a\b   3      exit 0
```

Six in BOTH lanes, not four and six — so the decoded three-character name never
arrives at the printer, and the two lanes really are landing on one undecoded
text rather than converging by a round trip. WHICH step drops the decoded key
(the `fromEntries` materialization declining on this operand, or something
downstream of it) is NOT established here, and this document should not be read
as claiming it. Nor is the `--release` behaviour re-verified — the case runner
does not execute the `.wasm` that build emits (§7). Note the direction either
way: the `fromEntries` side is the CORRECT one and the source side is the
defect.

**Not fixed here, deliberately.** Decoding a key's escapes is a parser change
whose blast radius is every string literal in the language, not a comparison
site. It is pinned instead, in both scopes, as
`escaped_quote_in_a_key_is_stored_undecoded_*` in
`crates/kali_cli/tests/cases/object/property_key_identity.toml`, WRONG ON
PURPOSE, with node's answers and what would close it. **No task in this plan
owns closing it.** — **still true of this plan, and superseded as a statement
about the register: FILED 2026-09-08 as R-57**, which carries the fix direction,
the eleven-escape sweep, the two same-spelling probes that AGREE with node, and
the standing warning that the `__proto__` guards of §4.1 must move onto the
decoded name in the same commit as any decoding fix. Still not fixed; now
tracked.

## 7. The lane the corpus cannot reach

`fold_object_has_own_call` and the binding path of
`fold_object_from_entries_call` run only at `OptimizationLevel::Release` and
`ReleaseAdvanced` (`kali_optimize/src/driver.rs:187-189`). `kali run` is
`BuildMode::Fast` (`kali_cli/src/lib.rs:440`) and `--release` exists only on
`kali build`, which emits a `.wasm` the case runner does not execute. **No
corpus case can reach that lane**, which is why Task 3's regression there
survived a green corpus.

Every assertion in `object_fold_tests/object_has_own.rs` expected `Some("true")`,
so a regression flipping the fold to always-`true` — the mirror of the
always-`false` one this task fixed — would have passed the whole tree. Two tests
were added: `release_folds_object_has_own_to_false_when_the_key_is_absent`
(`Object.hasOwn({'"a"': 1}, 'a')` must fold to `false`) and its positive control
`release_folds_object_has_own_to_true_for_the_quoted_name_itself`. The negative
one was mutation-checked: reverting `constant_property_key` to `literal_text`
turns it red.
