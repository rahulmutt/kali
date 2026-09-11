# `.length` on a member-expression receiver renders a NODE'S CHILD COUNT, not a length

> **CLOSED 2026-09-11** by the **length-fails-closed** project
> (`docs/superpowers/specs/2026-09-11-length-fails-closed-design.md`), filed and
> retired in the register as **R-63**, the home §6 below suggested. Every silent
> row of §1's table now refuses with `E5506` at exit 1: `render_length`'s arity
> fallbacks (§2's `:1336-1338` and `:1373-1377`) and its baked `0` decline, and
> `emit_unary`'s `.length` floor refuses instead of pushing `0`. The three-property
> row §1 marks "agrees by coincidence" refuses too, which is the honest outcome:
> its `3` was never computed. The `{a: [1,2,3]}` dot row still prints `3`, now
> through a lane that computes it. §4's pin
> `a_dot_member_length_still_renders_the_child_count` is now
> `a_dot_member_length_refuses_instead_of_rendering_the_child_count`, and the
> bracket-literal spelling and the `let` receiver that §4 says nothing pinned are
> pinned refusing in `crates/kali_cli/tests/cases/runtime/length_fails_closed.toml`,
> as is §5's `["abc"][0].length`. **Not closed here:** §2's sibling arm in
> `render_static_value` (`host.rs:928-942`), which is R-31's mechanism. The
> `docs/superpowers/sdd/2026-09-08-computed-member-static-name/` directory cited
> below was never committed.

**Filed** 2026-09-09, at `71b5f42f6c`, by the **computed-member-static-name**
project (`docs/superpowers/sdd/2026-09-08-computed-member-static-name/`), which
met it while measuring the chained-access lane. It is **not R-13**, **not
R-59**, and **not caused by computed access at all** — the dot spelling, which
this project never touched, is the half that is still silent.

**Oracle:** `node v26.8.1`. **Baseline:** `dc19c3a040`. **Every reading below was
re-measured at `71b5f42f6c`** on `.cache/cargo-target/debug/kali`, in module
scope and inside `function main() { … }`, and the two scopes agree except where
noted. Exit 0, empty stderr, no diagnostic, on every silent row.

## 1. What it does

| program | kali @ `71b5f42f6c` | node | verdict |
|---|---|---|---|
| `const o = {a: "xyz"}; o.a.length` | `1` | `3` | **SILENT** |
| `const o = {a: "xyzwv"}; o.a.length` | `1` | `5` | **SILENT** |
| `const o = {a: "xyz", z: 1}; o.a.length` | `2` | `3` | **SILENT** |
| `const o = {a: "xyz", z: 1, y: 2}; o.a.length` | `3` | `3` | agrees **by coincidence** |
| `const o = {a: "xyz"}; o["a"].length` | `2` | `3` | **SILENT** |
| `const o = {a: "xyzwv"}; o["a"].length` | `2` | `5` | **SILENT** |
| `const o = {a: [1,2,3]}; o.a.length` | `3` | `3` | agrees, by the same mechanism |
| `const o = {a: [1,2,3]}; o["a"].length` | `2` | `3` | **SILENT** |
| `let o = {a: "xyz"}; o.a.length` | `0` | `3` | **SILENT** (a third arm) |
| `const o = {a: "xyzwv"}; const k = "a"; o[k].length` | `E5506`, exit 1 | `5` | fail-closed as of this project |
| `const o = {a:1,b:2,c:3,d:4}; const k = "keys"; Object[k](o).length` | `E5506`, exit 1 | `4` | fail-closed as of this project |
| `const s = "xyz"; s.length` | `3` | `3` | **control** — the static string lane is correct |
| `const a = [1,2,3]; a.length` | `3` | `3` | **control** — the array-literal lane is correct |
| `const o = {a:1,b:2,c:3,d:4}; Object.keys(o).length` | `4` | `4` | **control** — a CALL receiver is correct |

**The value never tracks the string.** It tracks a node's arity, and *which*
node depends on the spelling:

* **The dot spelling** (`o.a.length`) recurses through the receiver binding into
  the OBJECT LITERAL and reports **the object's property count** — 1, 2 and 3 for
  one-, two- and three-property objects holding the same `"xyz"`. The plan that
  filed this described it as "prints `1` for every string length"; that is true
  only for a one-property receiver, and the three-property row above is the
  measurement that sharpens it.
* **The bracket spelling** (`o["a"].length`) stops at the MEMBER NODE and reports
  its own two children (base + index), so it is **always `2`**, whatever the
  object and whatever the string.
* **A `let` receiver** takes neither: the binding is not in codegen's `const`-only
  `bindings` map, so the identifier arm's terminal fallback bakes in **`0`**.

Two rows agree with node and both are coincidences of arity, not correctness:
a three-property object holding a three-character string, and an object field
holding an array literal whose element count IS its `.length`.

## 2. Mechanism

`crates/kali_codegen/src/intrinsics/host.rs`:

* `render_length` (`:1194`) is reached from `render_static_value`'s `"length"`
  member arm (`:916-921`). It has a long list of structural bails — a
  `URLSearchParams` result, a `TextDecoder.decode` result, a `String(...)`
  coercion, an inline `TextEncoder.encode`, a `crypto.getRandomValues` result —
  each added because that shape had **fallen through to the same arity fallback**
  and rendered a call node's `callee + args` count as a "length". Their doc
  comments say so in as many words.
* The fallback is two lines: `if node.text.is_none() { return
  Some(node.children.len().to_string()); }` (`:1336-1338`) and the tail
  `if node.children.len() == 1 { recurse } else { Some(node.children.len()) }`
  (`:1373-1377`). A member node has no text, so a bracket member returns 2 and a
  dot member recurses one level and returns whatever the receiver's arity is.
* The sibling arm in `render_static_value` (`:926-942`) renders a text-less node
  as its child count by the same rule, which is why the defect is a *pair* of
  arms and not one.

**This is a fallback that returns a plausible small integer for anything it does
not understand**, which is exactly the shape the register's §3 group **G3** is
named for; each of the six bails above is one site that was found the expensive
way. It is not filed as a G3 member here because filing is a §2 decision, not a
follow-up's.

## 3. This project moves only half of it

> **This project moves only half of it.** The computed-member half
> (`o[k].length` for a folded `const` key, and `Object[k](...)`) now refuses,
> because a nameless computed member declines both renderers and a member read on
> a nameless base is denied. **The DOT half is untouched and still silently
> prints `1`.** Nothing in the computed-member-static-name project's test set
> would notice if it got worse, and no register entry covers it: R-16/R-17
> (cluster G5) are about a string HANDLE leaking as an integer, and this is a
> renderer returning an arity. Filing it as a section 2 entry is a dozen
> coordinated edits (a predicate, a matcher, a counts re-freeze, an oracle pair,
> a clusters row), which is a task of its own.

**One correction to that paragraph, measured rather than assumed.** The plan that
wrote it listed `o["a"].length` among the spellings that "now refuse". **It does
not.** A string-literal index is inside the parser's readable set, so
`o["a"]` is a NAMED member, never a `LirNodeKind::ComputedMember`, and it reaches
the arity fallback exactly as it did at `dc19c3a040` — printing `2` where node
prints `3`. What refuses is the half whose index the parser cannot read: a
folded `const` key (`o[k].length`) and a computed callee (`Object[k](o)`). So
this project moved **less than half**: one of the three silent spellings, and
the two that a reader is least likely to write.

Also worth stating, because a refusal can look like a closure: the two spellings
that now refuse do so **under `kali run` only**. `kali check` exits 0 on both,
which is a spec §8 twin disagreement recorded by the live cases
`check_still_admits_the_chained_access_off_a_folded_member` and
`check_still_admits_the_computed_callee` in
`crates/kali_cli/tests/cases/object/computed_member_static_name.toml`.

## 4. What pins it today

* `a_dot_member_length_still_renders_the_child_count` in
  `crates/kali_cli/tests/cases/object/computed_member_static_name.toml` — **WRONG
  ON PURPOSE**, pinning `1` against node's `3`, so the dot half is not silently
  unpinned while the computed half's refusal is.
* `run_refuses_a_chained_access_off_a_folded_member` and
  `run_refuses_a_computed_callee` in the same file pin the half that moved, with
  their `check` twins pinning the disagreement.
* Nothing pins the bracket-literal spelling `o["a"].length`, the `let`-receiver
  `0`, or the two coincidental agreements. Those are measured here and nowhere
  else.

## 5. Neighbours it is probably related to, and one it is not

* **`["abc"][0].length` → `2` against node's `3`** is the same arity fallback
  reached from an array element, and it is already recorded — in the R-56
  retirement bullet of `kali-silent-miscompile-register.md` §2, and in §6 of
  `blast-radius-ranking.md`, both of which say it is "very likely R-17's family"
  (**G5**) and decline to decide. This document does not decide either, but it
  supplies the discriminator those two lacked: the value is an **arity**, and R-17
  is about a string handle read as an integer. Those are different mechanisms
  that happen to produce a small wrong number.
* **`Object.keys(o)[0].length` → `2`** is the same thing one spelling further out
  and, for a two-character key, agrees with node by coincidence.
* **NOT related**: `s.length`, `s["length"]` and `const k="length"; s[k]` all
  print `3` for `const s = "abc"` and agree with node, pinned by
  `the_three_string_length_spellings_agree`. The string lane is correct; it is
  the MEMBER lane that is not.

## 6. Suggested home

**§2 of the register, silent**, as a new entry — not a lane of an existing one.
It is not R-16/R-17 (a string handle read as an integer), not R-13 or R-59 (both
about a computed index's name), and not R-21 (an absent property reading `0`).
Its fix unit is `render_length`'s and `render_static_value`'s arity fallbacks,
which nothing else in the register names.
