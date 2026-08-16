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

`object_fold.rs:156` (enumeration fold) and `:920` (timeline eligibility) are
prototype-pollution defences: `__proto__` is JavaScript's prototype setter, not
an own property, so folding an enumeration over a literal that carries one would
emit a phantom own key. Removing a trim changes what a guard matches, so both
were reasoned about explicitly rather than swept.

**Both still catch everything they caught** — including against the escape
exception of §6, which is checked rather than assumed below. The guard compares
against the constant `"__proto__"`, and after Task 3 every real spelling of the
setter arrives as that exact name:

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
directions, so both were tested rather than argued:

**Direction A — can a real `__proto__` key hide from the guard?** It would have
to be spelled with an escape, so that the stored text differs from the decoded
name `__proto__`. It cannot:

* kali's lexer HARD-ERRORS on any escape outside `\\`, `\"`, `\'` and `` \` ``.
  Measured: `const o = {"_\_proto__": 1};` gives
  `error[E1004]: unsupported string escape sequence at position 14`, exit 1 —
  the program does not compile, so no key reaches any guard.
* Each of the four supported escapes decodes to `\`, `"`, `'` or `` ` ``, and
  the name `__proto__` contains none of those characters. So a decoded name of
  exactly `__proto__` cannot be produced by a spelling that contains an escape:
  any such name carries at least one residual `\`, `"`, `'` or `` ` ``.

Therefore a key whose property name is `__proto__` is always spelled without
escapes, its stored text equals its name, and the guard fires.

**Direction B — can the guard fire on a key that is NOT the setter?** That needs
a stored text of exactly `__proto__` whose real name is something else. A stored
text differs from its name only when the source spelling contained a backslash,
and the text `__proto__` contains no backslash — so there was no escape, and
text and name are the same. No false positive.

Both guards therefore hold **in fact**, not merely by the unqualified invariant:
the escape exception is real, and it is unreachable from this comparison in
either direction.

**A TRIPWIRE ON THE LEXER, recorded so it is not rediscovered as a
prototype-pollution bug.** Direction A rests on the lexer's escape allowlist,
which is checked, not assumed:

```
const o = {"__proto__": 1, a: 2};
for (const k of Object.keys(o)) console.log(k);

  kali:  error[E1004]: unsupported string escape sequence at position 13
         error[E1004]: unsupported string escape sequence at position 19   exit 1
  node:  a                                                                exit 0
```

`\x5f` is refused the same way. Note what node's answer says: node decodes the
key to `__proto__`, treats it as the PROTOTYPE SETTER, and enumerates only `a`.
So if the lexer ever grows `\u`/`\x`, that program starts compiling, its stored
key text is the undecoded `__proto__`, the guards do not fire on it,
and the enumeration folds a phantom `__proto__` own key — a prototype-setter
spelling smuggled past a security carve-out. **A change that adds unicode or hex
escapes to the lexer must move these guards onto the decoded name.** Today they
are sound because that spelling does not compile at all.

Tests: `does_not_fold_object_keys_over_a_proto_keyed_literal` (unchanged
assertion; its fixture was corrected to spell the key slot the way the front end
now does) and a new sibling,
`folds_object_keys_over_a_quoted_proto_named_key`, which pins the over-match as
gone. The pre-existing phantom-`__proto__`-key behaviour on the *codegen*
backstop lane (`Object.keys({__proto__: 1, a: 2})` prints `__proto__` then `a`
where node prints `a`) is unchanged by this task and was verified identical at
`f563a0ecf4`; it is a separate, pre-existing defect on a different lane.

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
absent-property-read defect that fabricates `0` where JavaScript has
`undefined` — the same residual already recorded in that file for `o[0]` on the
BigInt case. It is pinned WRONG ON PURPOSE with what would close it.

## 6. The one exception to "a key slot's text is the property name"

**ESCAPE SEQUENCES.** `kali_parser`'s `unquote_string_literal`
(`crates/kali_parser/src/literal.rs:7-24`) strips a string key's delimiters
WITHOUT decoding its escapes, so a key whose source spelling contains one is
stored undecoded:

```
const o = {"a\"b": 1};
for (const k of Object.keys(o)) console.log(k, k.length);
console.log(o['a"b']);

  kali:  a\"b  6   then  0      exit 0
  node:  a"b   3   then  1      exit 0
```

The stored key slot holds the six characters `a\"b`; the property name is the
three characters `a"b`. All three wrong answers follow from that one fact — the
enumerated key prints a stray backslash, its `.length` is 6, and a probe written
with the real name misses and falls into the absent-property lane's fabricated
`0`.

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

**There IS a producer disagreement here, and it is currently unobservable.**
`constant_property_key` reads through `literal_value` →
`parse_string_literal` (`crates/kali_optimize/src/constant_fold.rs`), which DOES
decode `\\`, `\"`, `\'` and `` \` ``. So a `fromEntries` entry spelled
`["a\"b", 1]` yields the correct three-character name while a source literal
with the same spelling yields the six-character undecoded text. It does not show
up today because the enumeration fold re-encodes with `format!("{:?}", …)` and
the downstream string reader strips delimiters without decoding, so both lanes
print the same wrong `a\"b` — measured, before and after. Note the direction:
the `fromEntries` side is the CORRECT one and the source side is the defect.

**Not fixed here, deliberately.** Decoding a key's escapes is a parser change
whose blast radius is every string literal in the language, not a comparison
site. It is pinned instead, in both scopes, as
`escaped_quote_in_a_key_is_stored_undecoded_*` in
`crates/kali_cli/tests/cases/object/property_key_identity.toml`, WRONG ON
PURPOSE, with node's answers and what would close it. **No task in this plan
owns closing it.**

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
