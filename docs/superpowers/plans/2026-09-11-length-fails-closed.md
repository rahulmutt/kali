# `.length` Fails Closed Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every `.length` read in `kali_codegen` compute the real value or refuse with `E5506`, never render a node's child count or bake in `0`, and carry every ledger artifact that must move with that change.

**Architecture:** Three edits in `kali_codegen`. `emit_unary`'s `"length"` floor refuses instead of pushing `I64Const(0)`. `render_length`'s three fallbacks return `None`. `call.rs`'s `String(x.length)` proof is keyed on the member shape. The rest of the plan is the register filing and retirement of R-63, the R-15 lane split, test changes that must land atomically with the fix, and the followup corrections.

**Tech Stack:** Rust workspace (`cargo test`), the `cases` black-box runner (`crates/kali_cli/tests/cases/**.toml`), node (`v26.8.2` locally) as oracle, the blast-radius tool (`tools/blast-radius/*.mjs`, acorn 8.18.0), the Task-18 browser case generator (`tools/task-18-browser-pilot/gen_batch8a.py`, Python 3).

**Spec:** `docs/superpowers/specs/2026-09-11-length-fails-closed-design.md` (committed `34d0a6807f`). Read it before Task 1; this plan argues from it and does not repeat its reasoning.

## Global Constraints

- **Branch:** `length-fails-closed`. **Baseline:** `152fdd5364`. Line references are as of the baseline.
- **Every commit is green.** `bash scripts/test-gate.sh` must print `GATE OK: 0 failing tests` after every task. The baseline prints exactly that. Task 3 is one commit on purpose: the fix turns R-15's and R-63's `silent` oracle cases red, and flipping them without the fix turns them red too.
- **CI also runs clippy and fmt; the gate runs neither.** Before pushing: `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check`. (`--all-targets` fails at the baseline on six unrelated files; see `release-tier-allocation-identity-discovered-defects.md` §10. Use CI's command.)
- **One cargo target directory** (`/workspace/.cache/cargo-target`). No git worktrees, no second `CARGO_TARGET_DIR`: both have exhausted this pod's disk and killed a run.
- **Never hand-edit a generated file.** `cases/browser/promise_all_bundle.toml` and `promise_all_settled_bundle.toml` are written by `tools/task-18-browser-pilot/gen_batch8a.py` from constants in `batch8a_captures.py`. Edit the inputs, regenerate. The generator is a byte-exact fixed point at the baseline (verified during planning).
- **Never hand-edit a generated region of `docs/superpowers/followups/blast-radius-ranking.md`.** Re-splice it with the script in Task 1 Step 16.
- **SDD reports go in `.superpowers/sdd/2026-09-11-length-fails-closed/`**, which is gitignored (`.superpowers/sdd/.gitignore` is `*`). **Never cite a report path from a committed document**: the 2026-09-10 followups cite `docs/superpowers/sdd/…/task-N-report.md` files that were never committed, and Task 4 has to disclose those dead citations.
- **Do not modify `scripts/test-gate.sh`.**
- **Oracle:** `node` on `PATH` (`v26.8.2` here). Record the version actually used in anything written to a document.
- **Commit trailer:** end every commit message with the line
  `Claude-Session: https://claude.ai/code/session_013rmRTZwswauPtUg3zzTg9k`

---

## What was measured while writing this plan (2026-09-11, at `152fdd5364`)

All throwaway: applied, measured, reverted. Recorded so an implementer knows what a correct run looks like.

| measurement | result |
|---|---|
| Spec §4.1 consumer matrix, 16 positions × {`o.a.length`, `arr.length`} × 2 scopes + 3 extras, with this plan's exact Task 3 code | baseline 28 ok / 10 refused / **29 silent**; with the code 26 ok / 41 refused / **0 silent**. No control lost. **The stop gate is cleared.** `Number(R)` and `xs[R]` refuse for both receivers at baseline through unrelated gates, so they are not pinned. |
| Every Task 2/3 pin program (91), baseline vs Task 3 code | all 91 exit 0 at baseline; every must-refuse pin refuses under Task 3; every control keeps its exact stdout. All refusals show the floor's message **except** `bracket_member_array.js`, which shows `reading growable-array field 'a' as a plain value` (the receiver's own refusal, emitted first). |
| Existing tests under Task 3 code with Task 2's edits applied | bundle cases 8+8 pass (their node-harness step still fails as expected); `runtime_smoke` `build_emits_browser_bundle_promise_all_sequencing*` 4 pass; `soundness/textcodec` 227 pass; `object/computed_member_static_name` only `a_dot_member_length_still_renders_the_child_count` fails; `oracle/tier2` only `r15_split_returns_empty_array_{module_scope,in_function}` fail |
| Task 2's edits at the baseline binary | bundle cases 8+8 pass, `runtime_smoke` 4 pass, `crypto_random_uuid_lowers_to_kalirt_import` passes |
| `gen_batch8a.py promise_all_bundle promise_all_settled_bundle` at baseline | rewrote both files; `git status` clean (fixed point) |
| R-63 matcher (Task 1) in a scratch copy of `tools/blast-radius` | 7 on the Task 1 test program; `count.mjs`: `counted 177 programs, 43 countable predicates`, R-63 raw 35 / reachable 0 (anchor 0/0, extension 35/0), zero kind `present-but-unreachable` |
| plain `node count.mjs` at baseline | changes exactly two things in `counts.json`: `nodeVersion` `v26.8.1` → `v26.8.2`, and an added null record for R-61 (R-61's own filing did not regenerate the file) |

---

## File Structure

| file | responsibility | task |
|---|---|---|
| `docs/superpowers/followups/kali-silent-miscompile-register.md` | R-63 §2 entry + §0.2 row + Movement sentence (file, then retire); R-15 §0.2 row + STATUS bullet; §0.2 case-count sentence | 1, 2, 3 |
| `crates/kali_cli/tests/cases/oracle/tier2.toml` | R-63 oracle pair (`r63a`); R-15 element-only lane (`r15e`); verdict flips; header counts and lane table | 1, 2, 3 |
| `tools/blast-radius/predicates.json`, `matchers.mjs`, `matchers.test.mjs`, `count.mjs` | R-63 catalogue record, matcher, test, upper-bound disclosure | 1 |
| `tools/blast-radius/counts.json`, `accepts.json` | regenerated outputs | 1, 3 |
| `tools/blast-radius/clusters.json` | R-63 joins G6 (file), leaves (retire) | 1, 3 |
| `crates/kali_blast_radius/src/{manifest_tests,catalogue_tests,register_tests,oracle_tests}.rs` | SHA re-pins and count constants | 1, 2 |
| `docs/superpowers/followups/blast-radius-ranking.md` | re-splice + §6 EIGHTH and NINTH amendments | 1, 3 |
| `crates/kali_cli/tests/cases/runtime/length_fails_closed.toml` | **create** — receiver pins: controls (Task 2), refusals (Task 3) | 2, 3 |
| `crates/kali_cli/tests/cases/runtime/length_fails_closed_consumers.toml` | **create** — spec §4.1 consumer matrix pins | 2, 3 |
| `tools/task-18-browser-pilot/batch8a_captures.py`, `gen_batch8a.py` | delete the `.length !== 2 ||` guards; per-target rationale amendment | 2 |
| `crates/kali_cli/tests/cases/browser/promise_all{,_settled}_bundle.toml` | regenerated | 2 |
| `crates/kali_cli/tests/runtime_smoke.rs:151` | same guard deleted | 2 |
| `crates/kali_codegen/src/intrinsics/host_tests/crypto.rs:25` | stop reading `u.length` | 2 |
| `crates/kali_codegen/src/emit/operators.rs:427-435` | the floor refuses (spec §3.1) | 3 |
| `crates/kali_codegen/src/intrinsics/host.rs:1336-1338`, `:1369`, `:1373-1377` | fallbacks decline (spec §3.2) | 3 |
| `crates/kali_codegen/src/emit/call.rs:4096-4103` | member-shape proof (spec §3.3) | 3 |
| `crates/kali_cli/tests/cases/object/computed_member_static_name.toml:628-637` | on-purpose pin becomes a refusal pin | 3 |
| `docs/superpowers/followups/member-length-renders-the-child-count.md`, `codegen-array-literal-predicate-is-still-negative-space.md`, `release-tier-allocation-identity-discovered-defects.md` | dated correction notices | 4 |
| `docs/superpowers/followups/length-fails-closed-discovered-defects.md` | **create** | 4 |

Task order: **1** file R-63 (green at baseline) → **2** baseline-safe test preparation → **3** the fix, atomic → **4** documents → **5** final verification.

---

## Task 1: File R-63 as a SILENT register entry

Spec §6.1 items 1 and 4. **No compiler code changes in this task.** Every artifact an entry needs is added at once, because the gates in `crates/kali_blast_radius` fail on any subset.

**Files:**
- Modify: `crates/kali_cli/tests/cases/oracle/tier2.toml` (source keys after `r60a_function.js` at `:1054-1059`; cases appended at end; header `:9-28`, lane table `:114`)
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.2 sentence `:238-250`; §0.2 row after `:307`; Movement bullet ending `:538`; §2 entry before `## Tier 3` at `:4291`)
- Modify: `tools/blast-radius/predicates.json`, `matchers.mjs`, `matchers.test.mjs`, `count.mjs`, `clusters.json`
- Regenerate: `tools/blast-radius/accepts.json`, `tools/blast-radius/counts.json`
- Modify: `crates/kali_blast_radius/src/manifest_tests.rs:338-356`, `catalogue_tests.rs:127-128`, `register_tests.rs:130-131`, `oracle_tests.rs:184,200-201`
- Modify: `docs/superpowers/followups/blast-radius-ranking.md` (both generated regions; §6 EIGHTH amendment before `### 6.1` at `:1164`)

**Interfaces:**
- Produces: register entry id `R-63`; oracle lane prefix `r63a` with programs `r63a_module.js` / `r63a_function.js` and cases `r63a_length_renders_a_child_count_{module_scope,in_function}` (Task 3 flips both); matcher `lengthReadOnUnprovenReceiver`; a `clusters.json` assignment of R-63 to G6 (Task 3 removes it); oracle total 167 (Task 2 makes it 169); register §2 total 48; the splice script `.superpowers/sdd/2026-09-11-length-fails-closed/splice_ranking.py` (reused in Task 3).

- [ ] **Step 1: Add the R-63 oracle programs**

In `crates/kali_cli/tests/cases/oracle/tier2.toml`, directly after the `"r60a_function.js"` value (the line `main();` then `"""` at `:1058-1059`), add:

```toml
"r63a_module.js" = """const o = {a: "xyz"};
console.log(o.a.length);
"""
"r63a_function.js" = """function main() {
  const o = {a: "xyz"};
  console.log(o.a.length);
}
main();
"""
```

- [ ] **Step 2: Append the R-63 oracle pair at the end of the same file**

```toml

[[case]]
name = "r63a_length_renders_a_child_count_module_scope"
kind = "oracle"
register_entry = "R-63"
program = "r63a_module.js"
verdict = "silent"
rationale = '''
R-63, the member-receiver lane, module scope. Section 2's `Repro` verbatim: a `.length` read off a named member whose value is a string.

Expected verdict set from the register's recorded status: section 0.2's R-63 row and section 2's entry both record SILENT in both scopes, so SILENT.

MEASURED 2026-09-11 at `152fdd5364` against node v26.8.2: SILENT. kali prints `1` at exit 0 with empty stderr; node prints `3` at exit 0. The `1` is the object literal's PROPERTY COUNT: `render_length`'s tail fallback (`crates/kali_codegen/src/intrinsics/host.rs:1373-1377` at `152fdd5364`) recursed through the receiver binding into the object literal and rendered its child count.

WHAT A REGRESSION HERE WOULD MEAN. This pair is filed SILENT by the length-fails-closed project and flips to FAIL_CLOSED in the commit that fixes it (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 6.1). After that flip, a return to SILENT means a `.length` fallback fabricates a number again.
'''

[[case]]
name = "r63a_length_renders_a_child_count_in_function"
kind = "oracle"
register_entry = "R-63"
program = "r63a_function.js"
verdict = "silent"
rationale = '''
R-63, the member-receiver lane, in-function scope. The module-scope program verbatim inside `function main() { ... }` with a trailing `main();` -- the wrapper is the only addition.

Expected verdict set from the register's recorded status: SILENT, as for the module scope.

MEASURED 2026-09-11 at `152fdd5364` against node v26.8.2: SILENT. kali prints `1` at exit 0 with empty stderr; node prints `3` at exit 0. Byte-identical to the module scope.
'''
```

- [ ] **Step 3: Run the oracle cases, then the blast-radius gates, and watch the gates fail**

Run: `cargo test -p kali_cli --test cases -- oracle/tier2`
Expected: PASS, `102 passed` (both new cases classify SILENT at the baseline binary).

Run: `cargo test -p kali_blast_radius 2>&1 | grep -E "^test .*FAILED|panicked|Only in"`
Expected: FAIL. `every_zero_two_row_is_the_class_set_its_live_cases_assert` reports `Only in the cases: ["R-63"]`, and `exactly_four_oracle_cases_are_unattributed_and_all_of_them_are_ground_truth` reports `left: 167` / `right: 165`. Those are the two failures the rest of this task clears.

- [ ] **Step 4: Add R-63's §2 entry**

In `docs/superpowers/followups/kali-silent-miscompile-register.md`, replace

```
  follow-on project starts from.

---

## Tier 3 — silently wrong control flow (value otherwise intact)
```

with

````
  follow-on project starts from.

### R-63: `.length` renders a node's child count, or `0`, for a receiver with no length lane

- **Added**: 2026-09-11, by the **length-fails-closed** project
  (`docs/superpowers/specs/2026-09-11-length-fails-closed-design.md`), off
  `152fdd5364`. Filed SILENT in its own commit so the retirement that follows is
  a separate step a reader can audit; the retirement is recorded as a bullet
  below when it lands.
- **Verification**: `CONFIRMED-BY-CONTROLLER` — measured at `152fdd5364` against
  **`node v26.8.2`**, in **both** scopes, on `.cache/cargo-target/debug/kali`
  built from that commit.
- **Root-cause group**: **G6** — *unresolved or unimplemented builtins fold to a
  default instead of failing closed*. A `.length` read on a receiver no codegen
  lane understands is a builtin property read that yields a type-plausible
  integer instead of a diagnostic, which is the signature §3 gives G6. It is not
  G2 (no callee is unresolved) and not G4 (the property is present and has a
  right answer).
- **Repro** (module scope; the in-function form is the same two statements
  inside `function main() { ... } main();`):
  `const o = {a: "xyz"}; console.log(o.a.length);` → node `3`; kali `1` (exit 0,
  empty stderr).
- **Every lane, measured at `152fdd5364` against node v26.8.2, both engines at
  exit 0, kali with empty stderr:**

  | program | kali | node |
  |---|---|---|
  | `const o = {a: "xyzwv"}; console.log(o.a.length);` | `1` | `5` |
  | `const o = {a: "xyz", z: 1}; console.log(o.a.length);` | `2` | `3` |
  | `const o = {a: "xyz"}; console.log(o["a"].length);` | `2` | `3` |
  | `let o = {a: "xyz"}; console.log(o.a.length);` | `0` | `3` |
  | `console.log(["abc"][0].length);` | `2` | `3` |
  | `const o = {length: "abc"}; console.log(o.length);` | `1` | `abc` |
  | `function main() { let a = [1, 2]; console.log(a.length); } main();` | `0` | `2` |
  | `(await Promise.all([p1, p2, p3])).length` in an async function | `2` | `3` |
  | `(await Promise.all([p1])).length` | `2` | `1` |
  | `(await Promise.allSettled([p1, p2, p3])).length` | `2` | `3` |

  The two-promise `Promise.all` form prints `2` on both engines by coincidence,
  and so do `{a: "xyz", z: 1, y: 2}`'s `o.a.length` (`3`) and
  `Object.keys({ab: 1, c: 2})[0].length` (`2`). The value tracks a node's arity,
  never the string or the array.
- **Mechanism, traced.** A `.length` read passes three rungs (spec §2.3). Rung 1,
  `render_length` (`crates/kali_codegen/src/intrinsics/host.rs:1194`), ends in
  three fallbacks: a text-less node renders `children.len()` (`:1336-1338`), an
  unbound identifier renders `Some("0")` (`:1369`), and the tail recurses into a
  one-child node or renders its child count (`:1373-1377`). Rung 3,
  `emit_unary`'s `"length"` arm (`crates/kali_codegen/src/emit/operators.rs:368`),
  ends in a floor that emits the receiver, drops it and pushes `I64Const(0)` with
  no diagnostic (`:427-435`). `emit/call.rs:4096-4103` also accepts rung 1's
  `Some("0")` as proof that `String(x.length)` is an integer.
- **Severity**: **Tier 2** — silently produces a wrong value.
- **Blast radius**: countable, matcher `lengthReadOnUnprovenReceiver`: raw 35 /
  reachable 0, every site in the extension stratum, zero kind
  `present-but-unreachable`. An upper bound for the reasons `count.mjs`'s
  `UPPER_BOUNDS` records.
- **Fix direction, and what NOT to do.** Make the floor refuse, and make the
  fallbacks decline onto it (spec §3). **Do not start by narrowing
  `kali_codegen`'s `is_array_literal`**: spec §2.2 measured that move turning
  three suite tests and five probe programs silently wrong, because every
  "not an array literal" branch lands on one of these fallbacks.
- **Pinned by**: two oracle cases (`r63a`, both scopes, `tier2.toml`) asserting
  the SILENT class.
- **Related**: `docs/superpowers/followups/member-length-renders-the-child-count.md`
  described the member lane first and suggested this home for it (its §6).
- **Confidence**: high on behaviour (both scopes, ten lanes, a sixteen-position
  consumer matrix in spec §4.1); high on mechanism (three reverted spikes, spec
  §2.2-§2.5).

---

## Tier 3 — silently wrong control flow (value otherwise intact)
````

- [ ] **Step 5: Add R-63's §0.2 row**

In the same file, replace

```

**Two entries a reader may look for and not find.**
```

with

```
| R-63 `.length` renders a node's child count, or `0`, for a receiver with no length lane | **SILENT** (both scopes) | **added 2026-09-11 by the length-fails-closed project**, off `152fdd5364`; **measured at `152fdd5364` against `node v26.8.2`, both scopes**: `const o = {a: "xyz"}; console.log(o.a.length)` prints `1` in kali at exit 0 with empty stderr where node prints `3` at exit 0. Lane `r63a`. §2's entry carries nine more lanes measured by hand. |

**Two entries a reader may look for and not find.**
```

(The first line of the old text is the blank line that ends the table. The new row goes directly under the R-60 row, with no blank line between them.)

- [ ] **Step 6: Update §0.2's case-count sentence**

Replace

```
**161 cases back the 46
rows.** ~~157 cases back the 44 rows
```

with

```
**163 cases back the 47
rows.** ~~161 cases back the 46 rows … holds 165~~ (superseded 2026-09-11, when the length-fails-closed project added **R-63** with its own two-case scope pair), ~~157 cases back the 44 rows
```

and replace

```
The oracle directory holds
165 (~~161~~, ~~157~~);
```

with

```
The oracle directory holds
167 (~~165~~, ~~161~~, ~~157~~);
```

- [ ] **Step 7: Append R-63's arrival to the Movement bullet**

Replace

```
  ever removed from the ranking. See the ranking's §6 SEVENTH amendment, which is
  also the first regeneration in which the ACCEPT SET moved.
```

with

```
  ever removed from the ranking. See the ranking's §6 SEVENTH amendment, which is
  also the first regeneration in which the ACCEPT SET moved.
  **2026-09-11: R-63 arrives, taking 28 to 29** — the length-fails-closed
  project files a class it measured and is about to fix, in a commit of its own,
  so its retirement is a separate step with its own intermediate figure (the
  R-56 precedent of 2026-08-16). See the ranking's §6 EIGHTH amendment.
```

### Task 1, continued: the catalogue, matcher, counts and cluster

- [ ] **Step 8: Add R-63's catalogue record**

R-61 is the last record in `tools/blast-radius/predicates.json`. Replace the file's final lines

```
have no concept of build tier" }
  ]
}
```

with

```
have no concept of build tier" },
    { "id": "R-63", "kind": "countable",
      "matcher": "lengthReadOnUnprovenReceiver",
      "description": "a `.length` READ (not a store: assignment and update targets are excluded, as R-60's matcher excludes them) spelled `x.length` or `x[\"length\"]`, whose receiver is a member or element expression, an awaited `Promise.all(...)` / `Promise.allSettled(...)` result written inline or through a binding whose initializer is one (bare or `globalThis.`-qualified, dotted or string-bracket), or an identifier bound by `let`; upper bound, because a member whose value is a real array or string field and a `let` binding a runtime lane proves both read `.length` correctly, and an acorn AST cannot see which lane codegen proves" }
  ]
}
```

- [ ] **Step 9: Add the matcher and its helpers**

In `tools/blast-radius/matchers.mjs`, replace the single line `export const MATCHERS = {` (`:434`) with:

```js
/** `x.length` or `x["length"]` -- the property a `.length` read names. */
function isLengthProperty(member) {
  return member.computed
    ? staticPropertyNameOf(member.property) === "length"
    : member.property.type === "Identifier" && member.property.name === "length";
}

/** `Promise.all(...)` / `Promise.allSettled(...)`, dotted or string-bracket, bare or `globalThis.`-qualified. */
function isPromiseCombinatorCall(node) {
  if (!node || node.type !== "CallExpression" || node.callee.type !== "MemberExpression") return false;
  const callee = node.callee;
  const method = callee.computed ? staticPropertyNameOf(callee.property) : callee.property.name;
  if (method !== "all" && method !== "allSettled") return false;
  const root = callee.object;
  if (root.type === "Identifier") return root.name === "Promise";
  if (root.type !== "MemberExpression" || root.object.type !== "Identifier" || root.object.name !== "globalThis") return false;
  return (root.computed ? staticPropertyNameOf(root.property) : root.property.name) === "Promise";
}

/** `await Promise.all(...)` / `await Promise.allSettled(...)`. */
function isAwaitedPromiseCombinator(node) {
  return Boolean(node) && node.type === "AwaitExpression" && isPromiseCombinatorCall(node.argument);
}

/**
 * The receivers R-63 measured reaching a `.length` fallback: a member or element
 * receiver, an awaited `Promise.all`/`allSettled` result (inline or through a
 * binding), and a `let`-bound identifier.
 */
function isUnprovenLengthReceiver(receiver, analysis) {
  if (receiver.type === "MemberExpression") return true;
  if (isAwaitedPromiseCombinator(receiver)) return true;
  if (receiver.type !== "Identifier") return false;
  const binding = analysis.binding(receiver);
  if (!binding) return false;
  if (binding.kind === "let") return true;
  return isAwaitedPromiseCombinator(binding.init);
}

export const MATCHERS = {
```

Then replace the end of the `MATCHERS` object

```js
      .filter((node) => !stores.has(node) && resolvesToObjectFromEntriesCall(node.object, analysis))
      .length;
  },
};
```

with

```js
      .filter((node) => !stores.has(node) && resolvesToObjectFromEntriesCall(node.object, analysis))
      .length;
  },

  // R-63: a `.length` READ whose receiver no codegen length lane proves, so
  // `render_length`'s fallbacks render a node's child count or `0`
  // (`crates/kali_codegen/src/intrinsics/host.rs:1336-1338`, `:1369`,
  // `:1373-1377` at `152fdd5364`) or `emit_unary`'s floor pushes `0`
  // (`crates/kali_codegen/src/emit/operators.rs:427-435`). The four receiver
  // shapes are the ones the entry measured: a member or element receiver
  // (`o.a.length`, `o["a"].length`, `["abc"][0].length`), an awaited
  // `Promise.all`/`allSettled` result (always `2`: the call node's callee plus
  // one argument), and a `let`-bound identifier.
  //
  // Reads only, as R-60's matcher: a store to `.length` is a different site class
  // and was not measured.
  //
  // Upper bound, disclosed in `count.mjs`'s UPPER_BOUNDS: a member receiver whose
  // value is a real array field (`{a: [1, 2, 3]}.a.length` prints `3` correctly),
  // a nested array element (`m[1].length` prints `3`), and a `let` binding a
  // runtime lane proves all read `.length` correctly and are counted, because an
  // acorn AST cannot see which lane codegen proves.
  lengthReadOnUnprovenReceiver(ast) {
    const analysis = analysisOf(ast);
    const stores = new Set();
    for (const node of analysis.of("AssignmentExpression")) stores.add(node.left);
    for (const node of analysis.of("UpdateExpression")) stores.add(node.argument);
    return analysis
      .of("MemberExpression")
      .filter((node) => !stores.has(node) && isLengthProperty(node) && isUnprovenLengthReceiver(node.object, analysis))
      .length;
  },
};
```

- [ ] **Step 10: Add the matcher test and bump the catalogue count**

In `tools/blast-radius/matchers.test.mjs`, change `assert.equal(catalogueNames.length, 42);` to `assert.equal(catalogueNames.length, 43);`, then replace

```js
  assert.equal(count("memberReadOnObjectFromEntriesResult", src), 5);
});
```

with

```js
  assert.equal(count("memberReadOnObjectFromEntriesResult", src), 5);
});

test("lengthReadOnUnprovenReceiver counts the measured receivers, and reads only", () => {
  // Positives: the four receiver shapes R-63 measured -- member, element, `let`,
  // and an awaited Promise.all/allSettled result inline, through a binding and
  // `globalThis`-qualified. Negatives: `const` array and string bindings and a
  // string literal (lanes that read `.length` correctly), a call receiver, a
  // `var` receiver (not measured), a store target, and a non-`.length` property.
  const src = `
    const o = {a: "xyz"};
    console.log(o.a.length);          // member receiver, counts
    console.log(o["a"].length);       // element receiver, counts
    console.log(["abc"][0].length);   // element of a literal, counts
    let s = [1, 2];
    console.log(s.length);            // let receiver, counts
    async function main() {
      const d = await Promise.all([Promise.resolve(1)]);
      console.log(d.length);          // awaited Promise.all through a binding, counts
      console.log((await Promise.allSettled([Promise.resolve(1)])).length); // inline, counts
      console.log((await globalThis.Promise["all"]([Promise.resolve(1)])).length); // qualified, counts
    }
    const a = [1, 2, 3];
    console.log(a.length);            // const array literal, does not count
    console.log("abc".length);        // string literal, does not count
    const t = "xyz";
    console.log(t.length);            // const string, does not count
    function f() { return [1]; }
    console.log(f().length);          // call receiver, does not count
    var v = [1];
    console.log(v.length);            // var receiver, does not count
    o.a.length = 3;                   // store target, does not count
    console.log(o.a.size);            // not .length, does not count
  `;
  assert.equal(count("lengthReadOnUnprovenReceiver", src), 7);
});
```

- [ ] **Step 11: Disclose the upper bound in `count.mjs`**

In `tools/blast-radius/count.mjs`, R-60's `UPPER_BOUNDS` entry is the last one. Replace

```js
      "receiver.",
  },
};
```

with

```js
      "receiver.",
  },
  "R-63": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: the matcher counts every `.length` READ on a member or " +
      "element receiver, an awaited `Promise.all`/`allSettled` result, or a `let`-bound " +
      "identifier, and an acorn AST cannot see which codegen lane proves the receiver. Several " +
      "counted shapes read correctly, measured at `152fdd5364` against node v26.8.2: " +
      "`const o = {a: [1, 2, 3]}; o.a.length` prints `3` and `const m = [[1, 2], [3, 4, 5]]; " +
      "m[1].length` prints `3`, both on both engines. The entry's own " +
      "lanes are the ones that diverge: a string-valued member (`o.a.length` over " +
      "`{a: \"xyz\"}` prints `1`), a `let` array read in a function (`0`), and an awaited " +
      "combinator result (always `2`).",
  },
};
```

- [ ] **Step 12: Run the matcher tests, re-measure accepts at the baseline binary, and regenerate counts**

```bash
cd /workspace && cargo build -p kali_cli
cd /workspace/tools/blast-radius
node --test
node accepts.mjs
node count.mjs
cd /workspace
git diff --stat tools/blast-radius/accepts.json tools/blast-radius/counts.json
node -e '
const c = require("./tools/blast-radius/counts.json");
const r = c.entries.find((e) => e.id === "R-63");
console.log("nodeVersion", c.nodeVersion, "| R-63", r.raw, "/", r.reachable, r.zero, "| upperBound disclosed:", r.upperBound && r.upperBound.disclosedInRecord);
console.log("R-61 record present:", Boolean(c.entries.find((e) => e.id === "R-61")));'
git diff tools/blast-radius/counts.json | grep -E '^[-+]\s+"reachable"' | head -40
```

Expected:
- `node --test`: every test passes, including `lengthReadOnUnprovenReceiver counts the measured receivers, and reads only`.
- `node count.mjs` prints `counted 177 programs, 43 countable predicates`.
- The `node -e` line prints `nodeVersion v26.8.2 | R-63 35 / 0 present-but-unreachable | upperBound disclosed: true` and `R-61 record present: true`.
- `accepts.mjs` runs `kali check` over the corpus with the **baseline** binary. `accepts.json` was last measured at `f9347ead96`, and PR #42 changed the compiler since. If `git diff --stat` shows `accepts.json` changed, the last command lists the `reachable` figures that moved. **Copy that list into Step 18's amendment.** It is baseline drift, not this project's movement, and measuring it here is what keeps Task 3's re-run attributable.

- [ ] **Step 13: Assign R-63 to G6 in `clusters.json`**

In `tools/blast-radius/clusters.json`, R-60's assignment is the last element of `assignments` and closes the file. Replace the file's final lines

```json
      "alternateCluster": "G2 — call lowering: unresolvable callee folds to constant `0`"
    }
  ]
}
```

with

```json
      "alternateCluster": "G2 — call lowering: unresolvable callee folds to constant `0`"
    },
    {
      "id": "R-63",
      "cluster": "G6 — unresolved or unimplemented builtins fold to a default instead of failing closed",
      "registerSource": "G6 — unresolved or unimplemented builtins fold to a default instead of failing closed. A `.length` read on a receiver no codegen lane understands is a builtin property read that yields a type-plausible integer instead of a diagnostic ... It is not G2 (no callee is unresolved) and not G4 (the property is present and has a right answer).",
      "alternate": "G2 by the shared zero-emitting floor, declined because no call is unresolved; G4 declined because the property is present and node has a right answer for it.",
      "alternateCluster": "G2 — call lowering: unresolvable callee folds to constant `0`"
    }
  ]
}
```

and replace

```json
    "differ) is a RAW comparison and is unaffected."
  ],
```

with

```json
    "differ) is a RAW comparison and is unaffected.",
    "",
    "ADDED 2026-09-11 by the length-fails-closed project: R-63 (`.length` renders a node's child",
    "count, or `0`, for a receiver with no length lane) joins G6 on the register's own §2",
    "Root-cause line. It is filed SILENT in its own commit and is expected to leave G6 when the",
    "same project's fix retires it (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md",
    "section 6.1)."
  ],
```

### Task 1, continued: re-pins, header, ranking and commit

- [ ] **Step 14: Re-freeze the catalogue and matcher SHAs**

In `crates/kali_blast_radius/src/manifest_tests.rs`, replace

```rust
/// this series that touches `predicates.json` without touching
/// `counts.json`.
const FROZEN_PREDICATES_SHA256: &str =
```

with

```rust
/// this series that touches `predicates.json` without touching
/// `counts.json`.
///
/// **Re-frozen 2026-09-11**, the seventh movement of these constants, by the
/// length-fails-closed project filing **R-63** (§2, Tier 2 — `.length` renders
/// a node's child count, or `0`, for a receiver with no length lane). One more
/// §2 entry, one more catalogue record, and **`matchers.mjs` moved**: the record
/// is countable, `lengthReadOnUnprovenReceiver`, raw 35 / reachable 0, an upper
/// bound disclosed in `count.mjs`'s `UPPER_BOUNDS`. No other matcher body
/// changed.
///
/// `counts.json` **was** regenerated, and the regeneration moved two things that
/// are not R-63: `nodeVersion` went from `v26.8.1` to `v26.8.2` (the node this
/// machine runs), and R-61 gained the null record `count.mjs` writes for every
/// uncountable entry, which R-61's own filing had deliberately not generated.
const FROZEN_PREDICATES_SHA256: &str =
```

Then run `sha256sum tools/blast-radius/predicates.json tools/blast-radius/matchers.mjs` and replace the two hex strings on `:354` and `:356` (the lines after `const FROZEN_PREDICATES_SHA256: &str =` and `const FROZEN_MATCHERS_SHA256: &str =`) with the two digests it prints, in that order.

- [ ] **Step 15: Bump the three count constants**

- `crates/kali_blast_radius/src/catalogue_tests.rs`: replace `        47,\n        "expected exactly 47 catalogue records` with `        48,\n        "expected exactly 48 catalogue records`.
- `crates/kali_blast_radius/src/register_tests.rs`: replace `        47,\n        "expected exactly 47 tier-ranked entries` with `        48,\n        "expected exactly 48 tier-ranked entries`.
- `crates/kali_blast_radius/src/oracle_tests.rs`: replace `out of exactly 165.` with `out of exactly 167.`; replace `        165,` (the line after `cases.len(),`) with `        167,`; replace `"§0.2 states that 165 oracle cases exist and that 161 of them back its 46 rows; a \` with `"§0.2 states that 167 oracle cases exist and that 163 of them back its 47 rows; a \`.

- [ ] **Step 16: Update `tier2.toml`'s header**

In `crates/kali_cli/tests/cases/oracle/tier2.toml`, replace

```
# Tier 2 holds THIRTY-ONE entries: R-06, R-07, R-08, R-09, R-10, R-11, R-12,
# R-13, R-14, R-15, R-16, R-17, R-18, R-19, R-20, R-21, R-22, R-23, R-24, R-25,
# R-26, R-27, R-28, R-47, R-48, R-53, R-56, R-57, R-58, R-59, R-60.
# ~~TWENTY-NINE ... R-56, R-57, R-58.~~
# ~~TWENTY-SEVEN ... R-53, R-56.~~
#
# ALL THIRTY-ONE ARE COVERED HERE, in ONE HUNDRED cases (~~twenty-nine /
# ninety-six~~; ~~twenty-seven / ninety-two~~). Nothing in Tier 2 is left
# unauthored. The file was written in
# five passes -- R-06..R-17
```

with

```
# Tier 2 holds THIRTY-TWO entries with oracle cases: R-06, R-07, R-08, R-09,
# R-10, R-11, R-12, R-13, R-14, R-15, R-16, R-17, R-18, R-19, R-20, R-21, R-22,
# R-23, R-24, R-25, R-26, R-27, R-28, R-47, R-48, R-53, R-56, R-57, R-58, R-59,
# R-60, R-63. (R-61 is also Tier 2, but it is a release-tier condition and this
# harness measures the default tier only; see its section 2 entry.)
# ~~THIRTY-ONE ... R-59, R-60.~~
# ~~TWENTY-NINE ... R-56, R-57, R-58.~~
# ~~TWENTY-SEVEN ... R-53, R-56.~~
#
# ALL THIRTY-TWO ARE COVERED HERE, in ONE HUNDRED AND TWO cases (~~thirty-one /
# one hundred~~; ~~twenty-nine / ninety-six~~; ~~twenty-seven / ninety-two~~).
# Nothing in Tier 2 is left unauthored. The file was written in
# six passes -- R-06..R-17
```

then replace

```
# register-property-key-followups branch, off `dde0f083c0`) and R-59/R-60
# (four cases, 2026-09-08, at `02297ca6c2`, the same branch's second pair) -- and
```

with

```
# register-property-key-followups branch, off `dde0f083c0`), R-59/R-60
# (four cases, 2026-09-08, at `02297ca6c2`, the same branch's second pair) and
# R-63 (two cases, 2026-09-11, off `152fdd5364`, by the length-fails-closed
# project) -- and
```

then, in the lane table, replace

```
#   R-60   --                         r60a_..._module_scope  r60a_..._in_function
```

with

```
#   R-60   --                         r60a_..._module_scope  r60a_..._in_function
#   R-63   member receiver            r63a_..._module_scope  r63a_..._in_function
```

Check: `grep -c '^\[\[case\]\]' crates/kali_cli/tests/cases/oracle/tier2.toml` prints `102`.

- [ ] **Step 17: Re-splice the ranking**

Create `.superpowers/sdd/2026-09-11-length-fails-closed/splice_ranking.py` (the directory is gitignored; Task 3 reuses the script):

```python
"""Splice `cargo run -p kali_blast_radius --example rank` output into the ranking.

The generator prints the provenance table, then `## 2. The bands` onward. The
test `ranking_tests::spliced_document_matches_the_generator` compares each half
with the text between its markers, trimmed of newlines.
"""
import pathlib
import sys

DOC = pathlib.Path("docs/superpowers/followups/blast-radius-ranking.md")


def splice(text: str, marker: str, content: str) -> str:
    begin = text.index(f"<!-- {marker}:BEGIN")
    begin = text.index("-->", begin) + len("-->")
    end = text.index(f"<!-- {marker}:END -->")
    return text[:begin] + "\n" + content.strip("\n") + "\n" + text[end:]


generated = pathlib.Path(sys.argv[1]).read_text()
provenance, body = generated.split("## 2. The bands", 1)
text = DOC.read_text()
text = splice(text, "GENERATED-PROVENANCE", provenance)
text = splice(text, "GENERATED", "## 2. The bands" + body)
DOC.write_text(text)
print(f"spliced {DOC}")
```

Run:

```bash
cd /workspace
cargo run -q -p kali_blast_radius --example rank > .superpowers/sdd/2026-09-11-length-fails-closed/rank-task1.md
python3 .superpowers/sdd/2026-09-11-length-fails-closed/splice_ranking.py .superpowers/sdd/2026-09-11-length-fails-closed/rank-task1.md
git diff --stat docs/superpowers/followups/blast-radius-ranking.md
git diff docs/superpowers/followups/blast-radius-ranking.md | grep -E '^[-+]\|' | head -40
```

Expected: the diff touches only lines inside the two generated regions: R-63's new per-entry row, G6's cluster row, the provenance table (node version, HEAD) and any bands that moved.

- [ ] **Step 18: Write the ranking's EIGHTH amendment**

In `docs/superpowers/followups/blast-radius-ranking.md`, replace `### 6.1 The most important thing here is not a rank` with the paragraph below followed by a blank line and that same heading. Fill the two bracketed transcriptions from Step 12's and Step 17's command output, **copied, not predicted**:

```
**AMENDMENT 2026-09-11 — an EIGHTH regeneration, filing R-63.** The
length-fails-closed project (`docs/superpowers/specs/2026-09-11-length-fails-closed-design.md`)
filed **R-63** — `.length` renders a node's child count, or `0`, for a receiver
with no length lane — as a SILENT Tier 2 entry in **G6**, in a commit of its own,
before the fix that retires it. Its record is countable
(`lengthReadOnUnprovenReceiver`): raw 35 / reachable 0, every site in the
extension stratum, so it enters §3 as `present-but-unreachable` and adds nothing
to G6's reachable figure. G6's §2 row is now [the `+| G6` line from Step 17's
diff] (was [the `-| G6` line]). `counts.json` was regenerated, which also moved
`nodeVersion` from `v26.8.1` to `v26.8.2` and added R-61's null record; no other
matcher body changed. **The accept set was re-measured at the baseline binary
`152fdd5364`**, so that the next regeneration's movement belongs to this project
alone: [either "it did not move." if Step 12 showed `accepts.json` unchanged, or
"it moved, carrying compiler drift between `f9347ead96` and `152fdd5364`:" followed
by each Step 12 line as `R-NN reachable a → b`]. Every figure above is read out of
the regenerated §2–§5 and the `counts.json` diff.
```

(Square brackets mark transcriptions from command output; the committed paragraph contains no brackets.)

- [ ] **Step 19: Run the gates**

```bash
cd /workspace
cargo test -p kali_blast_radius
cargo test -p kali_cli --test cases -- oracle/tier2
bash scripts/test-gate.sh
```

Expected: `kali_blast_radius` all pass (including `every_zero_two_row_is_the_class_set_its_live_cases_assert`, `exactly_four_oracle_cases_are_unattributed_and_all_of_them_are_ground_truth`, `the_frozen_catalogue_and_its_matchers_are_the_ones_the_counts_were_taken_with`, `spliced_document_matches_the_generator`); `oracle/tier2` `102 passed`; `GATE OK: 0 failing tests`.

- [ ] **Step 20: Commit**

```bash
git add crates/kali_cli/tests/cases/oracle/tier2.toml \
  docs/superpowers/followups/kali-silent-miscompile-register.md \
  docs/superpowers/followups/blast-radius-ranking.md \
  tools/blast-radius/predicates.json tools/blast-radius/matchers.mjs \
  tools/blast-radius/matchers.test.mjs tools/blast-radius/count.mjs \
  tools/blast-radius/counts.json tools/blast-radius/accepts.json \
  tools/blast-radius/clusters.json \
  crates/kali_blast_radius/src/manifest_tests.rs \
  crates/kali_blast_radius/src/catalogue_tests.rs \
  crates/kali_blast_radius/src/register_tests.rs \
  crates/kali_blast_radius/src/oracle_tests.rs
git commit -F - <<'EOF'
docs(register): file R-63 -- .length renders a child count, or 0, for a receiver with no length lane

Filed SILENT in G6 with its oracle pair, catalogue record, countable matcher
(raw 35 / reachable 0, an upper bound), cluster assignment and a re-spliced
ranking, measured against node v26.8.2 at 152fdd5364. The accept set is
re-measured at the baseline binary so the fix's regeneration is attributable.

Claude-Session: https://claude.ai/code/session_013rmRTZwswauPtUg3zzTg9k
EOF
```

---

## Task 2: Test preparation that is green at the baseline

Spec §4.2 (controls), §4.3 (bundle, `runtime_smoke`, crypto rows) and §4.4 (R-15's element-only lane). **Every change here passes against the baseline compiler**, which is what lets Task 3 be only the change that must be atomic. No compiler code changes.

**Files:**
- Create: `crates/kali_cli/tests/cases/runtime/length_fails_closed.toml`
- Create: `crates/kali_cli/tests/cases/runtime/length_fails_closed_consumers.toml`
- Modify: `crates/kali_cli/tests/cases/oracle/tier2.toml` (R-15 programs `:715-720`, R-15 cases ending `:1512`, header, lane table)
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.2 sentence, R-15 row `:276`)
- Modify: `crates/kali_blast_radius/src/oracle_tests.rs`
- Modify: `tools/task-18-browser-pilot/batch8a_captures.py`, `tools/task-18-browser-pilot/gen_batch8a.py`
- Regenerate: `crates/kali_cli/tests/cases/browser/promise_all_bundle.toml`, `promise_all_settled_bundle.toml`
- Modify: `crates/kali_cli/tests/runtime_smoke.rs:151`, `crates/kali_codegen/src/intrinsics/host_tests/crypto.rs:25`

**Interfaces:**
- Consumes: Task 1's oracle total (167) and its §0.2 sentence text.
- Produces: the two case files, whose `[source]` tables already hold every Task 3 refusal program (Task 3 adds only `[constants]` entries and `[[case]]` blocks); R-15 lane `r15e` with cases `r15e_split_element_leaks_a_handle_{module_scope,in_function}`; oracle total 169.

- [ ] **Step 1: Create the receiver case file**

Create `crates/kali_cli/tests/cases/runtime/length_fails_closed.toml` with exactly this content. Every expected stdout is node's output, and every control was measured equal at `152fdd5364`:

```toml
# Cases for the length-fails-closed project (spec
# docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2).
#
# A `.length` read either computes the real value or refuses with E5506. The
# `_is_correct` cases read `.length` through a lane that computes it and must keep
# node's exact output. The `_refuses` cases read it on a receiver no lane proves:
# each printed a wrong number at exit 0 at `152fdd5364` -- a node's child count, or
# a baked `0` -- or the right number only by coincidence, and each rationale records
# kali's and node's output there. Every expectation was measured against node
# v26.8.2 at `152fdd5364` (controls and refusals) and against the fix (refusals).
#
# `[source]` keys are one file per program, named for the shape, because
# `[source]` is file-wide (see this directory's README).

[source]
"string_binding.js" = '''
const s = "xyz"; console.log(s.length);
'''
"array_literal_binding.js" = '''
const a = [1, 2, 3]; console.log(a.length);
'''
"member_array_dot.js" = '''
const o = {a: [1, 2, 3]}; console.log(o.a.length);
'''
"nested_element_length.js" = '''
const m = [[1, 2], [3, 4, 5]]; console.log(m[1].length);
'''
"empty_array.js" = '''
const a = []; console.log(a.length);
'''
"object_keys_length.js" = '''
const o = {a: 1, b: 2, c: 3, d: 4}; console.log(Object.keys(o).length);
'''
"allocation_binding.js" = '''
const a = new Array(5); console.log(a.length);
'''
"fill_binding.js" = '''
const a = new Array(4).fill(7); console.log(a.length);
'''
"split_length.js" = '''
console.log("a,b,c".split(",").length);
'''
"one_call_element.js" = '''
function f() { return 3; } const a = [f()]; console.log(a.length, a[0]);
'''
"concat_length.js" = '''
const a = [1, 2, 3]; console.log("n=" + a.length);
'''
"string_parameter.js" = '''
function f(s) { return s.length; } console.log(f("hello"));
'''
"random_uuid_length.js" = '''
const u = crypto.randomUUID(); console.log(u.length);
'''
"string_of_bigint_array_length_two.js" = '''
const a = [1n, 2n]; console.log(String(a.length));
'''
"string_of_bigint_array_length_three.js" = '''
const a = [1n, 2n, 3n]; console.log(String(a.length));
'''
"member_string_one_prop.js" = '''
const o = {a: "xyz"}; console.log(o.a.length);
'''
"member_string_five_chars.js" = '''
const o = {a: "xyzwv"}; console.log(o.a.length);
'''
"member_string_two_props.js" = '''
const o = {a: "xyz", z: 1}; console.log(o.a.length);
'''
"member_string_in_function.js" = '''
function main() { const o = {a: "xyz"}; console.log(o.a.length); } main();
'''
"bracket_member_string.js" = '''
const o = {a: "xyz"}; console.log(o["a"].length);
'''
"bracket_member_array.js" = '''
const o = {a: [1, 2, 3]}; console.log(o["a"].length);
'''
"let_receiver.js" = '''
let o = {a: "xyz"}; console.log(o.a.length);
'''
"element_string.js" = '''
console.log(["abc"][0].length);
'''
"let_array_in_function.js" = '''
function main() { let a = [1, 2]; console.log(a.length); } main();
'''
"length_field_object.js" = '''
const o = {length: "abc"}; console.log(o.length);
'''
"string_of_member_length.js" = '''
const o = {a: "xyz"}; console.log(String(o.a.length));
'''
"string_of_length_field.js" = '''
const o = {length: "abc"}; console.log(String(o.length));
'''
"promise_all_three.js" = '''
async function main() { const d = await Promise.all([Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)]); console.log(d.length); } main();
'''
"promise_all_one.js" = '''
async function main() { const d = await Promise.all([Promise.resolve(1)]); console.log(d.length); } main();
'''
"promise_all_settled_three.js" = '''
async function main() { const d = await Promise.allSettled([Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)]); console.log(d.length); } main();
'''
"member_string_three_props.js" = '''
const o = {a: "xyz", z: 1, y: 2}; console.log(o.a.length);
'''
"keys_element_length.js" = '''
const o = {ab: 1, c: 2}; console.log(Object.keys(o)[0].length);
'''
"promise_all_two.js" = '''
async function main() { const d = await Promise.all([Promise.resolve(1), Promise.resolve(2)]); console.log(d.length); } main();
'''

[[case]]
name = "a_const_string_binding_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "string_binding.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "a_const_array_literal_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "array_literal_binding.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "an_array_valued_member_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "member_array_dot.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "a_nested_array_element_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "nested_element_length.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "an_empty_array_literal_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `0` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "empty_array.js"]
exit = "success"
stdout = "0\n"

[[case]]
name = "an_object_keys_result_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `4` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "object_keys_length.js"]
exit = "success"
stdout = "4\n"

[[case]]
name = "a_new_array_binding_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `5` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "allocation_binding.js"]
exit = "success"
stdout = "5\n"

[[case]]
name = "a_filled_new_array_binding_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `4` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "fill_binding.js"]
exit = "success"
stdout = "4\n"

[[case]]
name = "a_static_split_result_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "split_length.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "a_one_call_element_array_literal_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `1 3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "one_call_element.js"]
exit = "success"
stdout = "1 3\n"

[[case]]
name = "a_length_in_string_concatenation_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `n=3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "concat_length.js"]
exit = "success"
stdout = "n=3\n"

[[case]]
name = "a_string_parameter_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `5` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "string_parameter.js"]
exit = "success"
stdout = "5\n"

[[case]]
name = "a_random_uuid_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `36` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "random_uuid_length.js"]
exit = "success"
stdout = "36\n"

[[case]]
name = "string_of_a_two_element_array_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `2` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "string_of_bigint_array_length_two.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "string_of_a_three_element_array_length_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "string_of_bigint_array_length_three.js"]
exit = "success"
stdout = "3\n"
```

- [ ] **Step 2: Create the consumer-matrix case file**

Create `crates/kali_cli/tests/cases/runtime/length_fails_closed_consumers.toml` with exactly this content:

```toml
# Cases for the length-fails-closed project (spec
# docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.1):
# the consumer matrix.
#
# `render_static_value` has 56 call sites. When `render_length` declines, each
# consumer either routes the read to emission, which ends at the refusing floor,
# or folds a number of its own. These cases pin the answer for each consumer
# position, with a receiver no lane proves (`o.a.length` over `{a: "xyz"}`, which
# must refuse) and one a lane computes (`arr.length` over `[1, 2, 3]`, which must
# stay correct), at module scope and inside a function. Measured against node
# v26.8.2 at `152fdd5364` and against the fix. `Number(...)` and an array index
# position are not pinned: both refuse for BOTH receivers at `152fdd5364`, through
# gates unrelated to `.length`, so they cannot tell the receivers apart.
#
# `dollar` escapes the template-literal program's `${`, per this directory's README.

[constants]
dollar = "$"

[source]
"p01_eq_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(arr.length === 3);
'''
"p01_eq_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(o.a.length === 3);
'''
"p01_eq_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(arr.length === 3); } main();
'''
"p01_eq_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(o.a.length === 3); } main();
'''
"p02_neq_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(arr.length !== 3);
'''
"p02_neq_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(o.a.length !== 3);
'''
"p02_neq_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(arr.length !== 3); } main();
'''
"p02_neq_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(o.a.length !== 3); } main();
'''
"p03_lt_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(arr.length < 5);
'''
"p03_lt_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(o.a.length < 5);
'''
"p03_lt_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(arr.length < 5); } main();
'''
"p03_lt_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(o.a.length < 5); } main();
'''
"p04_arith_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(arr.length + 1);
'''
"p04_arith_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(o.a.length + 1);
'''
"p04_arith_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(arr.length + 1); } main();
'''
"p04_arith_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(o.a.length + 1); } main();
'''
"p05_template_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(`n=${dollar}{arr.length}`);
'''
"p05_template_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(`n=${dollar}{o.a.length}`);
'''
"p05_template_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(`n=${dollar}{arr.length}`); } main();
'''
"p05_template_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(`n=${dollar}{o.a.length}`); } main();
'''
"p06_concat_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log("n=" + arr.length);
'''
"p06_concat_bad_mod.js" = '''
const o = {a: "xyz"}; console.log("n=" + o.a.length);
'''
"p06_concat_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log("n=" + arr.length); } main();
'''
"p06_concat_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log("n=" + o.a.length); } main();
'''
"p07_ternary_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(arr.length > 2 ? "big" : "small");
'''
"p07_ternary_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(o.a.length > 2 ? "big" : "small");
'''
"p07_ternary_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(arr.length > 2 ? "big" : "small"); } main();
'''
"p07_ternary_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(o.a.length > 2 ? "big" : "small"); } main();
'''
"p08_if_ok_mod.js" = '''
const arr = [1, 2, 3]; if (arr.length === 3) { console.log("three"); } else { console.log("other"); }
'''
"p08_if_bad_mod.js" = '''
const o = {a: "xyz"}; if (o.a.length === 3) { console.log("three"); } else { console.log("other"); }
'''
"p08_if_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; if (arr.length === 3) { console.log("three"); } else { console.log("other"); } } main();
'''
"p08_if_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; if (o.a.length === 3) { console.log("three"); } else { console.log("other"); } } main();
'''
"p09_math_max_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(Math.max(arr.length, 1));
'''
"p09_math_max_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(Math.max(o.a.length, 1));
'''
"p09_math_max_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(Math.max(arr.length, 1)); } main();
'''
"p09_math_max_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(Math.max(o.a.length, 1)); } main();
'''
"p10_string_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(String(arr.length));
'''
"p10_string_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(String(o.a.length));
'''
"p10_string_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(String(arr.length)); } main();
'''
"p10_string_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(String(o.a.length)); } main();
'''
"p13_logical_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log(arr.length && "yes");
'''
"p13_logical_bad_mod.js" = '''
const o = {a: "xyz"}; console.log(o.a.length && "yes");
'''
"p13_logical_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log(arr.length && "yes"); } main();
'''
"p13_logical_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log(o.a.length && "yes"); } main();
'''
"p14_let_ok_mod.js" = '''
const arr = [1, 2, 3]; let n = arr.length; console.log(n);
'''
"p14_let_bad_mod.js" = '''
const o = {a: "xyz"}; let n = o.a.length; console.log(n);
'''
"p14_let_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; let n = arr.length; console.log(n); } main();
'''
"p14_let_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; let n = o.a.length; console.log(n); } main();
'''
"p16_multiarg_ok_mod.js" = '''
const arr = [1, 2, 3]; console.log("len", arr.length);
'''
"p16_multiarg_bad_mod.js" = '''
const o = {a: "xyz"}; console.log("len", o.a.length);
'''
"p16_multiarg_ok_fn.js" = '''
function main() { const arr = [1, 2, 3]; console.log("len", arr.length); } main();
'''
"p16_multiarg_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; console.log("len", o.a.length); } main();
'''
"p15_return_bad_fn.js" = '''
function main() { const o = {a: "xyz"}; function g() { return o.a.length; } console.log(g()); } main();
'''

[[case]]
name = "strict_equality_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `true` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p01_eq_ok_mod.js"]
exit = "success"
stdout = "true\n"

[[case]]
name = "strict_equality_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `true` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p01_eq_ok_fn.js"]
exit = "success"
stdout = "true\n"

[[case]]
name = "strict_inequality_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `false` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p02_neq_ok_mod.js"]
exit = "success"
stdout = "false\n"

[[case]]
name = "strict_inequality_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `false` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p02_neq_ok_fn.js"]
exit = "success"
stdout = "false\n"

[[case]]
name = "less_than_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `true` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p03_lt_ok_mod.js"]
exit = "success"
stdout = "true\n"

[[case]]
name = "less_than_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `true` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p03_lt_ok_fn.js"]
exit = "success"
stdout = "true\n"

[[case]]
name = "arithmetic_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `4` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p04_arith_ok_mod.js"]
exit = "success"
stdout = "4\n"

[[case]]
name = "arithmetic_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `4` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p04_arith_ok_fn.js"]
exit = "success"
stdout = "4\n"

[[case]]
name = "template_literal_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `n=3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p05_template_ok_mod.js"]
exit = "success"
stdout = "n=3\n"

[[case]]
name = "template_literal_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `n=3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p05_template_ok_fn.js"]
exit = "success"
stdout = "n=3\n"

[[case]]
name = "string_concatenation_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `n=3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p06_concat_ok_mod.js"]
exit = "success"
stdout = "n=3\n"

[[case]]
name = "string_concatenation_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `n=3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p06_concat_ok_fn.js"]
exit = "success"
stdout = "n=3\n"

[[case]]
name = "ternary_test_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `big` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p07_ternary_ok_mod.js"]
exit = "success"
stdout = "big\n"

[[case]]
name = "ternary_test_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `big` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p07_ternary_ok_fn.js"]
exit = "success"
stdout = "big\n"

[[case]]
name = "if_condition_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `three` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p08_if_ok_mod.js"]
exit = "success"
stdout = "three\n"

[[case]]
name = "if_condition_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `three` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p08_if_ok_fn.js"]
exit = "success"
stdout = "three\n"

[[case]]
name = "math_max_argument_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p09_math_max_ok_mod.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "math_max_argument_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p09_math_max_ok_fn.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "string_call_argument_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p10_string_ok_mod.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "string_call_argument_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p10_string_ok_fn.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "logical_and_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `yes` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p13_logical_ok_mod.js"]
exit = "success"
stdout = "yes\n"

[[case]]
name = "logical_and_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `yes` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p13_logical_ok_fn.js"]
exit = "success"
stdout = "yes\n"

[[case]]
name = "let_initializer_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p14_let_ok_mod.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "let_initializer_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p14_let_ok_fn.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "second_console_log_argument_on_an_array_length_module_scope_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `len 3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p16_multiarg_ok_mod.js"]
exit = "success"
stdout = "len 3\n"

[[case]]
name = "second_console_log_argument_on_an_array_length_in_function_is_correct"
rationale = """A `.length` read through a lane that computes it. Measured at `152fdd5364` against node v26.8.2: kali and node both print `len 3` at exit 0. It must keep doing so once the `.length` floor refuses (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.2)."""
args = ["run", "p16_multiarg_ok_fn.js"]
exit = "success"
stdout = "len 3\n"
```

- [ ] **Step 3: Run the new cases**

Run: `cargo test -p kali_cli --test cases -- runtime/length_fails_closed`
Expected: PASS, `41 passed` (15 receiver controls and 26 consumer controls; one filter matches both files).

- [ ] **Step 4: Add R-15's element-only programs**

In `crates/kali_cli/tests/cases/oracle/tier2.toml`, replace the line

```
"r16_module.js" = """const s="hello world"; console.log("c=" + s.slice(0,3));
```

with

```
"r15e_module.js" = """const s="a,b,c"; const p=s.split(","); console.log("1="+p[1]);
"""
"r15e_function.js" = """function main() {
  const s="a,b,c"; const p=s.split(","); console.log("1="+p[1]);
}
main();
"""
"r16_module.js" = """const s="hello world"; console.log("c=" + s.slice(0,3));
```

- [ ] **Step 5: Add R-15's element-only cases**

In the same file, replace

```
[[case]]
name = "r16_string_method_handle_leak_in_concat_module_scope"
```

with

```
[[case]]
name = "r15e_split_element_leaks_a_handle_module_scope"
kind = "oracle"
register_entry = "R-15"
program = "r15e_module.js"
verdict = "silent"
rationale = """R-15, the element-read lane on its own, module scope. The register's repro without the `.length` consumer: `const s="a,b,c"; const p=s.split(","); console.log("1="+p[1]);`.

WHY THIS LANE EXISTS. Added 2026-09-11 by the length-fails-closed project (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.4). That project makes a `.length` read with no length lane refuse, so the `r15` pair's program, which prints `len=` before the element, stops at compile time and can no longer observe the element half. This lane keeps the leaking half measured by itself.

MEASURED 2026-09-11 at `152fdd5364` against node v26.8.2: SILENT. kali prints `1=-9223354418898927615` at exit 0 with empty stderr; node prints `1=b` at exit 0. The handle's bit pattern is the one the register recorded. The same reading holds with that project's fix applied: this program reads no `.length`."""

[[case]]
name = "r15e_split_element_leaks_a_handle_in_function"
kind = "oracle"
register_entry = "R-15"
program = "r15e_function.js"
verdict = "silent"
rationale = """R-15, the element-read lane on its own, in-function scope. The module-scope program verbatim inside `function main() { ... }` with a trailing `main();`.

MEASURED 2026-09-11 at `152fdd5364` against node v26.8.2: SILENT, byte-identical to the module scope: kali `1=-9223354418898927615` at exit 0, node `1=b`."""

[[case]]
name = "r16_string_method_handle_leak_in_concat_module_scope"
```

- [ ] **Step 6: Run the oracle cases and watch the count gate fail**

Run: `cargo test -p kali_cli --test cases -- oracle/tier2`
Expected: PASS, `104 passed`.

Run: `cargo test -p kali_blast_radius 2>&1 | grep -E "FAILED|left|right"`
Expected: FAIL in `exactly_four_oracle_cases_are_unattributed_and_all_of_them_are_ground_truth` only (`left: 169`, `right: 167`). R-15's class set is still `{SILENT}`, so `every_zero_two_row_is_the_class_set_its_live_cases_assert` passes.

- [ ] **Step 7: Update the counts and the prose that states them**

- `crates/kali_blast_radius/src/oracle_tests.rs`: `out of exactly 167.` → `out of exactly 169.`; `        167,` (after `cases.len(),`) → `        169,`; `that 167 oracle cases exist and that 163 of them back its 47 rows` → `that 169 oracle cases exist and that 165 of them back its 47 rows`.
- Register §0.2: replace `**163 cases back the 47\nrows.**` with `**165 cases back the 47\nrows.** ~~163 cases back the 47 rows … holds 167~~ (superseded 2026-09-11, when the same project added R-15's element-only lane \`r15e\`),` — keep everything after `rows.**` as it is. Replace `holds\n167 (~~165~~, ~~161~~, ~~157~~);` with `holds\n169 (~~167~~, ~~165~~, ~~161~~, ~~157~~);`.
- Register §0.2 R-15 row: replace `Partial closure, live defect. |` with `Partial closure, live defect. Lane \`r15e\` (added 2026-09-11 by the length-fails-closed project) reads the element alone, so the leaking half stays measured by itself when the \`r15\` program's \`.length\` read refuses. |`.
- `tier2.toml` header: replace `ALL THIRTY-TWO ARE COVERED HERE, in ONE HUNDRED AND TWO cases (~~thirty-one /` with `ALL THIRTY-TWO ARE COVERED HERE, in ONE HUNDRED AND FOUR cases (~~one hundred and two~~; ~~thirty-one /`; replace `# project) -- and` (the line Task 1 Step 16 wrote) with `# project), plus R-15's element-only lane r15e (two cases, same date, same\n# project) -- and`; in the lane table replace `#   R-15   --                         r15_..._module_scope   r15_..._in_function` with the two lines `#   R-15   .length + element read     r15_..._module_scope   r15_..._in_function` and `#   R-15   element read alone         r15e_..._module_scope  r15e_..._in_function`.

Check: `grep -c '^\[\[case\]\]' crates/kali_cli/tests/cases/oracle/tier2.toml` prints `104`.

- [ ] **Step 8: Re-run the blast-radius gates**

Run: `cargo test -p kali_blast_radius`
Expected: PASS. If `spliced_document_matches_the_generator` fails, re-splice exactly as Task 1 Step 17 (writing `rank-task2.md`) and check that `git diff` moves nothing outside the provenance table; anything else moving is a finding to stop on.

- [ ] **Step 9: Delete the bundle guards at the generator's input and regenerate**

Save this script as `.superpowers/sdd/2026-09-11-length-fails-closed/task2_generator_edit.py` and run it from the repository root. It deletes exactly 24 and 28 guard lines, scoped to the two constants by name (the same line shape appears 156 times across `batch8a_captures.py`, so a file-wide substitution would corrupt other captures), amends the capture docstring, and gives the two bundle targets a rationale sentence saying so:

```python
import pathlib, re
root = pathlib.Path("tools/task-18-browser-pilot")

# 1. Delete the guard lines from the two captured constants, scoped by constant name.
cap = root / "batch8a_captures.py"
text = cap.read_text()
guard = re.compile(r"^    '    [A-Za-z]+\.length !== 2 \|\|\\n'\n", re.M)
for name, expected in (("CAP_PROMISE_ALL_BUNDLE", 24), ("CAP_PROMISE_ALL_SETTLED_BUNDLE", 28)):
    start = text.index(f"\n{name} = ") + 1
    end = text.index("\nCAP_", start + 1) + 1
    block, n = guard.subn("", text[start:end])
    assert n == expected, (name, n)
    text = text[:start] + block + text[end:]

# 2. The docstring's "none was edited afterwards" is no longer true for two constants.
old_doc = ("source. Every constant below came from that one run; none was edited\nafterwards.\n")
assert text.count(old_doc) == 1
new_doc = ("source. Every constant below came from that one run; none was edited\nafterwards, "
           "EXCEPT two. AMENDED 2026-09-11 by the length-fails-closed project\n"
           "(docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section\n"
           "4.3): CAP_PROMISE_ALL_BUNDLE and CAP_PROMISE_ALL_SETTLED_BUNDLE each had every\n"
           "`X.length !== 2 ||` guard line deleted by hand (24 and 28 lines). The rationale\n"
           "gen_batch8a.py renders for those two targets says so.\n")
text = text.replace(old_doc, new_doc)
cap.write_text(text)

# 3. A per-target amendment sentence in the generated rationale.
gen = root / "gen_batch8a.py"
g = gen.read_text()
anchor = "BUNDLE_TARGETS = {\n"
assert g.count(anchor) == 1
constant = (
    "# 2026-09-11, length-fails-closed: the Promise.all and Promise.allSettled bundle\n"
    "# captures had their `.length !== 2 ||` guard lines deleted by hand, so their\n"
    "# rendered rationale must not go on claiming byte-exact builder output unqualified.\n"
    "LENGTH_GUARDS_AMENDMENT = (\n"
    "    \"AMENDED 2026-09-11 by the length-fails-closed project (spec \"\n"
    "    \"docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, section 4.3): every \"\n"
    "    \"`X.length !== 2 ||` clause was deleted from this capture by hand, so the program is no \"\n"
    "    \"longer byte-exact builder output. Those guards passed only because kali rendered an \"\n"
    "    \"awaited Promise.all or Promise.allSettled result's `.length` as the call node's child \"\n"
    "    \"count, which is 2 for any number of promises. The element checks beside every deleted \"\n"
    "    \"guard are unchanged, and runtime/length_fails_closed.toml pins that `.length` read \"\n"
    "    \"refusing.\"\n"
    ")\n\n"
)
g = g.replace(anchor, constant + anchor)
for what in ('        what="the browser `Promise.all` smoke body",\n',
             '        what="the browser `Promise.allSettled` smoke body",\n'):
    assert g.count(what) == 1, what
    g = g.replace(what, what + "        amended=LENGTH_GUARDS_AMENDMENT,\n")
insert_anchor = ('        f"RULE 12 -- the Rust comment prose of browser_{name}.rs, carried verbatim: "\n'
                 '        f"\\"{repin}\\"",\n    ]\n'
                 '    if docs:\n'
                 '        parts.append(P.rule13_carried(docs) + " That doc belongs to "\n')
assert g.count(insert_anchor) == 1, g.count(insert_anchor)
head = insert_anchor.split('    if docs:')[0]
g = g.replace(insert_anchor, head + '    if spec.get("amended"):\n        parts.insert(4, spec["amended"])\n' + insert_anchor[len(head):])
gen.write_text(g)
print("edits applied")
```

```bash
cd /workspace
python3 .superpowers/sdd/2026-09-11-length-fails-closed/task2_generator_edit.py
(cd tools/task-18-browser-pilot && python3 gen_batch8a.py promise_all_bundle promise_all_settled_bundle)
git diff --stat
grep -c '^    [A-Za-z]*\.length !== 2 ||$' crates/kali_cli/tests/cases/browser/promise_all_bundle.toml crates/kali_cli/tests/cases/browser/promise_all_settled_bundle.toml
grep -c 'AMENDED 2026-09-11 by the length-fails-closed' crates/kali_cli/tests/cases/browser/promise_all_bundle.toml crates/kali_cli/tests/cases/browser/promise_all_settled_bundle.toml
(cd tools/task-18-browser-pilot && python3 gen_batch8a.py promise_race_bundle) && git diff --stat -- crates/kali_cli/tests/cases/browser/promise_race_bundle.toml
```

Expected (all measured during planning): the script prints `edits applied`; `git diff --stat` adds exactly `batch8a_captures.py`, `gen_batch8a.py` and the two bundle tomls to Task 2's earlier files; the guard-line counts are `0` and `0` (the files still contain `.length` twice each, inside the amendment sentence); the amendment appears `2` times per file (the text and JSON cases); regenerating `promise_race_bundle`, which shares `_bundle_rationale`, produces no diff. **Never hand-edit the two tomls**: the generator owns them and is a fixed point.

- [ ] **Step 10: Delete the same guard in `runtime_smoke.rs`, and stop the crypto unit test reading `.length`**

- `crates/kali_cli/tests/runtime_smoke.rs:151`: replace `  if (values.length !== 2 || values[0] !== left || values[1] !== right) {` with `  if (values[0] !== left || values[1] !== right) {`.
- `crates/kali_codegen/src/intrinsics/host_tests/crypto.rs:25`: replace `"const u = crypto.randomUUID(); console.log(u.length);"` with `"const u = crypto.randomUUID(); console.log(u);"`. The test asserts the `crypto_random_uuid` import; `parse_and_lower_lir` runs no type inference, so a `.length` read there reaches the floor. Task 2 Step 1's `a_random_uuid_length_is_correct` pins the real pipeline's `36`.

- [ ] **Step 11: Run what Steps 9-10 touched**

```bash
cargo test -p kali_cli --test cases -- browser/promise_all_bundle
cargo test -p kali_cli --test cases -- browser/promise_all_settled_bundle
cargo test -p kali_cli --test runtime_smoke -- build_emits_browser_bundle_promise_all_sequencing
cargo test -p kali_codegen --lib crypto_random_uuid_lowers_to_kalirt_import
```

Expected: `8 passed`, `8 passed`, `4 passed`, `1 passed`. (Each bundle case's `browser_bundle_harness` step still expects `exit = "failure"`, and still gets it: measured during planning, at the baseline and with Task 3's code.)

- [ ] **Step 12: Gate and commit**

Run: `bash scripts/test-gate.sh` — expected `GATE OK: 0 failing tests`.

```bash
git add crates/kali_cli/tests/cases/runtime/length_fails_closed.toml \
  crates/kali_cli/tests/cases/runtime/length_fails_closed_consumers.toml \
  crates/kali_cli/tests/cases/oracle/tier2.toml \
  docs/superpowers/followups/kali-silent-miscompile-register.md \
  crates/kali_blast_radius/src/oracle_tests.rs \
  tools/task-18-browser-pilot/batch8a_captures.py tools/task-18-browser-pilot/gen_batch8a.py \
  crates/kali_cli/tests/cases/browser/promise_all_bundle.toml \
  crates/kali_cli/tests/cases/browser/promise_all_settled_bundle.toml \
  crates/kali_cli/tests/runtime_smoke.rs \
  crates/kali_codegen/src/intrinsics/host_tests/crypto.rs
git add docs/superpowers/followups/blast-radius-ranking.md 2>/dev/null || true
git commit -F - <<'EOF'
test(length): pin the lanes that compute .length, and stop tests leaning on the ones that invent it

Adds the receiver and consumer-matrix control cases, measured equal to node
v26.8.2 at 152fdd5364; splits R-15's element read into its own oracle lane so
it stays measured once `.length` refuses; deletes the Promise.all bundle guards
that passed only because an awaited result's `.length` rendered the call node's
child count (at the generator's input, with the rationale amended); and stops
the randomUUID lowering test reading `.length` without type inference. Green at
the baseline compiler.

Claude-Session: https://claude.ai/code/session_013rmRTZwswauPtUg3zzTg9k
EOF
```

---

## Task 3: The fix, in one commit

Spec §3 (the three code changes), §4.2-§4.4 (refusal pins, the on-purpose pin, oracle flips) and §6.1 items 2-3 (R-63 retired, R-15's row). **One commit.** The code change alone turns the R-15 and R-63 oracle cases and the on-purpose pin red; flipping those without the code turns them red the other way; retiring R-63 without removing its cluster assignment fails `ranking.rs:313`. Everything below lands together, and the gate runs once, at the end.

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs:427-435`, `crates/kali_codegen/src/intrinsics/host.rs:1336-1338,1369,1373-1377`, `crates/kali_codegen/src/emit/call.rs:4096-4103`
- Modify: `crates/kali_cli/tests/cases/runtime/length_fails_closed.toml`, `length_fails_closed_consumers.toml` (Task 2's files)
- Modify: `crates/kali_cli/tests/cases/object/computed_member_static_name.toml` (the last case block)
- Modify: `crates/kali_cli/tests/cases/oracle/tier2.toml` (r15 and r63a verdicts and rationales)
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (R-15 row and §2 STATUS; R-63 row, §2 title and retirement bullet; Movement sentence)
- Modify: `tools/blast-radius/clusters.json`; regenerate `tools/blast-radius/accepts.json`, `counts.json`
- Modify: `docs/superpowers/followups/blast-radius-ranking.md` (re-splice + NINTH amendment)

**Interfaces:**
- Consumes: Task 2's case files (their `[source]` tables hold every refusal program already); Task 1's `r63a` cases and clusters assignment; Task 2's `r15e` lane; `.superpowers/sdd/2026-09-11-length-fails-closed/splice_ranking.py`.
- Produces: the floor's diagnostic text `` `.length` is unavailable in the current phase for this receiver: no lane proves its length, so kali refuses rather than emit a placeholder 0 ``, pinned by the needle `no lane proves its length`.

- [ ] **Step 1: Add the floor's needle as a constant in both case files**

In `crates/kali_cli/tests/cases/runtime/length_fails_closed.toml`, replace

```
(see this directory's README).

[source]
```

with

```
(see this directory's README).

[constants]
FLOOR = "no lane proves its length"

[source]
```

In `crates/kali_cli/tests/cases/runtime/length_fails_closed_consumers.toml`, replace `dollar = "$"` with the two lines `dollar = "$"` and `FLOOR = "no lane proves its length"`.

- [ ] **Step 2: Append the refusal pins**

Append this to the end of `length_fails_closed.toml` (after a blank line):

```toml
[[case]]
name = "a_string_member_length_refuses"
rationale = """At `152fdd5364` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "member_string_one_prop.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "a_five_character_string_member_length_refuses"
rationale = """At `152fdd5364` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `5`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "member_string_five_chars.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "a_string_member_of_a_two_property_object_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "member_string_two_props.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "a_string_member_length_in_a_function_refuses"
rationale = """At `152fdd5364` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "member_string_in_function.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "a_string_bracket_member_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "bracket_member_string.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "an_array_bracket_member_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2). The floor emits the receiver first, so this receiver's own refusal is the one reported and the needle names it."""
args = ["run", "bracket_member_array.js"]
exit = "failure"
stderr_contains = ["E5506", "reading growable-array field 'a' as a plain value"]

[[case]]
name = "a_let_object_member_length_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "let_receiver.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "a_string_element_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "element_string.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "a_let_array_length_in_a_function_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `2`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "let_array_in_function.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "an_object_length_field_refuses"
rationale = """At `152fdd5364` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `abc`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "length_field_object.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "string_of_a_string_member_length_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "string_of_member_length.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "string_of_an_object_length_field_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `abc`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "string_of_length_field.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "an_awaited_promise_all_of_three_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "promise_all_three.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "an_awaited_promise_all_of_one_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `1`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "promise_all_one.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "an_awaited_promise_all_settled_of_three_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "promise_all_settled_three.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "a_string_member_of_a_three_property_object_length_refuses"
rationale = """At `152fdd5364` kali printed `3` at exit 0 with no diagnostic where node v26.8.2 prints `3`: right only by coincidence: the number kali rendered was a node's child count that happened to equal the real length. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "member_string_three_props.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "an_object_keys_element_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `2`: right only by coincidence: the number kali rendered was a node's child count that happened to equal the real length. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "keys_element_length.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "an_awaited_promise_all_of_two_length_refuses"
rationale = """At `152fdd5364` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `2`: right only by coincidence: the number kali rendered was a node's child count that happened to equal the real length. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "promise_all_two.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]
```

Append this to the end of `length_fails_closed_consumers.toml` (after a blank line):

```toml
[[case]]
name = "strict_equality_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `false` at exit 0 with no diagnostic where node v26.8.2 prints `true`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p01_eq_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "strict_equality_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `false` at exit 0 with no diagnostic where node v26.8.2 prints `true`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p01_eq_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "strict_inequality_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `true` at exit 0 with no diagnostic where node v26.8.2 prints `false`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p02_neq_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "strict_inequality_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `true` at exit 0 with no diagnostic where node v26.8.2 prints `false`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p02_neq_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "less_than_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `true` at exit 0 with no diagnostic where node v26.8.2 prints `true`: right only by coincidence: the number kali rendered was a node's child count that happened to equal the real length. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p03_lt_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "less_than_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `true` at exit 0 with no diagnostic where node v26.8.2 prints `true`: right only by coincidence: the number kali rendered was a node's child count that happened to equal the real length. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p03_lt_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "arithmetic_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `4`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p04_arith_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "arithmetic_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `4`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p04_arith_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "template_literal_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `n=0` at exit 0 with no diagnostic where node v26.8.2 prints `n=3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p05_template_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "template_literal_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `n=0` at exit 0 with no diagnostic where node v26.8.2 prints `n=3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p05_template_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "string_concatenation_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `n=0` at exit 0 with no diagnostic where node v26.8.2 prints `n=3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p06_concat_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "string_concatenation_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `n=0` at exit 0 with no diagnostic where node v26.8.2 prints `n=3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p06_concat_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "ternary_test_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `small` at exit 0 with no diagnostic where node v26.8.2 prints `big`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p07_ternary_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "ternary_test_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `small` at exit 0 with no diagnostic where node v26.8.2 prints `big`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p07_ternary_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "if_condition_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `other` at exit 0 with no diagnostic where node v26.8.2 prints `three`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p08_if_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "if_condition_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `other` at exit 0 with no diagnostic where node v26.8.2 prints `three`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p08_if_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "math_max_argument_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p09_math_max_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "math_max_argument_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p09_math_max_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "string_call_argument_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p10_string_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "string_call_argument_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p10_string_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "logical_and_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `yes`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p13_logical_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "logical_and_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `yes`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p13_logical_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "let_initializer_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p14_let_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "let_initializer_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p14_let_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "second_console_log_argument_on_a_string_member_length_module_scope_refuses"
rationale = """At `152fdd5364` kali printed `len 1` at exit 0 with no diagnostic where node v26.8.2 prints `len 3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p16_multiarg_bad_mod.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "second_console_log_argument_on_a_string_member_length_in_function_refuses"
rationale = """At `152fdd5364` kali printed `len 1` at exit 0 with no diagnostic where node v26.8.2 prints `len 3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p16_multiarg_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]

[[case]]
name = "return_from_a_nested_function_of_a_string_member_length_refuses"
rationale = """At `152fdd5364` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: silently wrong. No lane proves this receiver's length, so it must refuse (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2)."""
args = ["run", "p15_return_bad_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${FLOOR}"]
```

- [ ] **Step 3: Run them and watch every refusal fail**

Run: `cargo test -p kali_cli --test cases -- runtime/length_fails_closed`
Expected: FAIL, `41 passed; 45 failed`. Each of the 45 fails because kali exits 0 (it prints a child count or `0`); that is the silent behaviour the next step removes.

- [ ] **Step 4: Change the compiler**

Save this diff as `.superpowers/sdd/2026-09-11-length-fails-closed/task3.patch` and apply it with `git apply --check .superpowers/sdd/2026-09-11-length-fails-closed/task3.patch && git apply .superpowers/sdd/2026-09-11-length-fails-closed/task3.patch`. It was measured during planning exactly as written. (If `git apply` refuses, make the same four hunks by hand; the context lines are the baseline's.)

```diff
diff --git a/crates/kali_codegen/src/emit/call.rs b/crates/kali_codegen/src/emit/call.rs
index ba015da5b3..7bdacc6e0e 100644
--- a/crates/kali_codegen/src/emit/call.rs
+++ b/crates/kali_codegen/src/emit/call.rs
@@ -4093,12 +4093,13 @@ impl<'a> FunctionEmitter<'a> {
                 return true;
             }
         }
-        // A statically-known `.length` (`String(a.length)`): the fold renders the
-        // count as a plain integer.
-        if member_node.text.as_deref() == Some("length")
-            && member_node.children.len() == 1
-            && self.render_length(&member).is_some()
-        {
+        // A `.length` read (`String(a.length)`) is an integer or it does not
+        // compile: every `.length` lane yields a count, and the one that cannot
+        // prove a length refuses (`emit_unary`'s `"length"` floor). So the member
+        // shape is the proof. This used to require `render_length(..).is_some()`,
+        // which a baked `Some("0")` satisfied for receivers with no length at all.
+        // Spec: docs/superpowers/specs/2026-09-11-length-fails-closed-design.md §3.3
+        if member_node.text.as_deref() == Some("length") && member_node.children.len() == 1 {
             return true;
         }
         // Bare identifier, checked BEFORE `resolve_bound_node`: a fold-lane
diff --git a/crates/kali_codegen/src/emit/operators.rs b/crates/kali_codegen/src/emit/operators.rs
index 9b14fc3af3..240ecafb16 100644
--- a/crates/kali_codegen/src/emit/operators.rs
+++ b/crates/kali_codegen/src/emit/operators.rs
@@ -424,15 +424,29 @@ impl<'a> FunctionEmitter<'a> {
                     };
                 }
 
+                // The floor. Every lane above declined, so nothing proves a length
+                // for this receiver. It used to push `I64Const(0)` here: a silent
+                // wrong `.length` for every receiver that reached it. Emit the
+                // receiver first, so a refusal specific to it (a computed member, a
+                // module-binding read) is the one reported, and add this arm's own
+                // refusal only when the receiver raised none.
+                // Spec: docs/superpowers/specs/2026-09-11-length-fails-closed-design.md §3.1
+                let errors_before = self.diagnostics.iter().filter(|d| d.is_error()).count();
                 let produced = self.emit_node(function, arg, true);
                 if produced.produced {
                     function.instruction(&Instruction::Drop);
                 }
-                function.instruction(&Instruction::I64Const(0));
-                EmittedValue {
-                    produced: true,
-                    shape: ValueShape::Scalar,
+                if self.diagnostics.iter().filter(|d| d.is_error()).count() > errors_before {
+                    function.instruction(&Instruction::Unreachable);
+                    return EmittedValue {
+                        produced: false,
+                        shape: ValueShape::Unknown,
+                    };
                 }
+                self.deny_e5506(
+                    function,
+                    "`.length` is unavailable in the current phase for this receiver: no lane proves its length, so kali refuses rather than emit a placeholder 0",
+                )
             }
             op if op.parse::<usize>().is_ok() || op.parse::<isize>().is_ok() => {
                 // `process.argv[<int literal>]` (Spec 5 Task 5): read the arg's
diff --git a/crates/kali_codegen/src/intrinsics/host.rs b/crates/kali_codegen/src/intrinsics/host.rs
index 0d8e29063e..5c7a100bcc 100644
--- a/crates/kali_codegen/src/intrinsics/host.rs
+++ b/crates/kali_codegen/src/intrinsics/host.rs
@@ -1333,8 +1333,14 @@ impl<'a> FunctionEmitter<'a> {
         {
             return None;
         }
+        // A text-less node that no arm above resolved has no static length. This
+        // arm used to render its CHILD COUNT (a member node's 2, a call's callee
+        // plus arguments, an object literal's property count) as the length,
+        // silently. Declining sends the read to the runtime lanes, whose floor
+        // (`emit_unary`'s `"length"` arm) computes it or refuses.
+        // Spec: docs/superpowers/specs/2026-09-11-length-fails-closed-design.md §3.2
         if node.text.is_none() {
-            return Some(node.children.len().to_string());
+            return None;
         }
 
         if node.children.is_empty() {
@@ -1366,15 +1372,17 @@ impl<'a> FunctionEmitter<'a> {
                     // baking in a wrong constant.
                     return None;
                 }
-                return Some("0".to_string());
+                // No binding, no array or growable lane: this identifier's length
+                // is not statically known. This used to bake in `0`.
+                return None;
             }
         }
 
-        if node.children.len() == 1 {
-            self.render_length(&node.children[0])
-        } else {
-            Some(node.children.len().to_string())
-        }
+        // What is left (a named member such as `o.a`, whose one child is the
+        // receiver) has no static length either. This tail used to recurse into
+        // that child and render ITS child count, so `o.a.length` printed the
+        // object's property count; it declines instead.
+        None
     }
 
     /// Recognize `new EventTarget()` (Stage D event lane) with the `EventTarget`
```

What each hunk is, so a reviewer can check it against the spec:

- `call.rs` (spec §3.3): `String(x.length)` is admitted on the member shape (`text == "length"`, one child). The floor makes every `.length` read an integer or a compile error, so a rendered value is no longer needed as evidence.
- `operators.rs` (spec §3.1): the floor emits the receiver first, then refuses. If emitting the receiver already pushed an error (a computed member, a module-binding read, a growable field), the floor adds no second diagnostic and only closes the stack with `Unreachable`. `run_refuses_a_computed_callee` depends on that ordering.
- `host.rs`, first hunk (spec §3.2): the text-less child-count fallback declines.
- `host.rs`, second hunk: the baked `Some("0")` and the tail child-count fallback decline.

Run: `cargo build -p kali_cli`
Expected: builds with no warnings from `kali_codegen`.

- [ ] **Step 5: Run the pins again**

Run: `cargo test -p kali_cli --test cases -- runtime/length_fails_closed`
Expected: PASS, `86 passed`.

- [ ] **Step 6: See exactly which existing tests the change moves**

```bash
cargo test -p kali_cli --test cases -- object/computed_member_static_name 2>&1 | grep -E "^test result|^    [a-z]"
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | grep -E "^test result|^    [a-z]"
cargo test -p kali_cli --test cases -- soundness/textcodec 2>&1 | grep -E "^test result"
cargo test -p kali_codegen --lib 2>&1 | grep -E "^test result"
```

Expected, and nothing else: `computed_member_static_name` fails only `a_dot_member_length_still_renders_the_child_count`; `oracle/tier2` fails only `r15_split_returns_empty_array_module_scope`, `r15_split_returns_empty_array_in_function`, `r63a_length_renders_a_child_count_module_scope` and `r63a_length_renders_a_child_count_in_function` (each a `verdict mismatch … expected silent, measured fail_closed`); `soundness/textcodec` passes (including `string_call_proof_admits_the_scalar_shapes_the_parent_build_rendered_static_length`, which the `call.rs` hunk keeps green); `kali_codegen --lib` passes. **Any other failure is spec §7's "a rung nobody mapped" or "more losses than measured": stop and report it instead of adjusting a test.**

- [ ] **Step 7: Turn the on-purpose pin into a refusal pin**

In `crates/kali_cli/tests/cases/object/computed_member_static_name.toml`, the last block of the file is exactly:

```toml
[[case]]
name = "a_dot_member_length_still_renders_the_child_count"
rationale = """WRONG ON PURPOSE, and NO TASK IN THIS PLAN OWNS CLOSING IT. node prints 3; kali prints 1. The value is NOT the string's length and does not track it: measured at `71b5f42f6c` against node v26.8.1, `{a: "xyz"}` and `{a: "xyzwv"}` both print 1, `{a: "xyz", z: 1}` prints 2, and `{a: "xyz", z: 1, y: 2}` prints 3 -- which agrees with node by coincidence. See docs/superpowers/followups/member-length-renders-the-child-count.md.

THE MECHANISM, MEASURED RATHER THAN ASSUMED. The plan's Task 8 brief described this as `the member node's child count`. That is the BRACKET spelling: `o["a"].length` returns 2 for every object and every string, because `render_length` stops at the two-child member node. The DOT spelling recurses one level through the receiver binding into the OBJECT LITERAL and returns ITS property count, which is why the number tracks the object's width and not the string. Both arms are `crates/kali_codegen/src/intrinsics/host.rs`'s `render_length` fallbacks (`:1336-1338` and `:1373-1377`), reached from `render_static_value`'s `"length"` member arm.

WHY IT IS HERE. The computed half of this defect (`o[k].length`) refuses as of this project, and `run_refuses_a_chained_access_off_a_folded_member` above pins that. Without this case the dot half would be silently unpinned, and a reader would reasonably infer from the refusal that the whole family was handled. It was not: the dot spelling and the string-literal bracket spelling are both still silent, and the bracket-literal one is not pinned anywhere at all."""
args = ["run", "dot_member_length.js"]
exit = "success"
stdout = "1\n"
```

Replace that block with:

```toml
[[case]]
name = "a_dot_member_length_refuses_instead_of_rendering_the_child_count"
rationale = """Was `a_dot_member_length_still_renders_the_child_count`, WRONG ON PURPOSE: at `152fdd5364` kali printed `1` -- the object literal's PROPERTY COUNT, through `render_length`'s tail fallback -- where node prints `3`, and no task in the plan that wrote it owned closing it. The length-fails-closed project owns it (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md, sections 3.1-3.2): `render_length`'s fallbacks decline and `emit_unary`'s `.length` floor refuses, so the dot spelling now fails closed with E5506 instead of printing a child count. The bracket-literal spelling `o["a"].length`, which this file noted was pinned nowhere, is pinned refusing in runtime/length_fails_closed.toml. See docs/superpowers/followups/member-length-renders-the-child-count.md, which that project closes."""
args = ["run", "dot_member_length.js"]
exit = "failure"
stderr_contains = ["E5506", "no lane proves its length"]
```

Run: `cargo test -p kali_cli --test cases -- object/computed_member_static_name`
Expected: PASS, `61 passed`.

### Task 3, continued: the ledger that must move with the fix

- [ ] **Step 8: Flip R-15's `r15` pair to `fail_closed`**

In `crates/kali_cli/tests/cases/oracle/tier2.toml`:

- replace `program = "r15_module.js"\nverdict = "silent"` with `program = "r15_module.js"\nverdict = "fail_closed"`;
- replace `program = "r15_function.js"\nverdict = "silent"` with `program = "r15_function.js"\nverdict = "fail_closed"`;
- replace `would be a different program and would likely classify FAIL_CLOSED; this case does not measure it and does not claim it."""` with the same text minus its closing `"""`, followed by:

```

RE-MEASURED 2026-09-11 against node v26.8.2, with the length-fails-closed project's fix (spec docs/superpowers/specs/2026-09-11-length-fails-closed-design.md): FAIL_CLOSED (was SILENT). kali exits 1 with empty stdout and `error[E5506]: `.length` is unavailable in the current phase for this receiver: no lane proves its length, so kali refuses rather than emit a placeholder 0` on stderr; node prints `len=3` then `1=b` at exit 0. The `len=0` was `render_length`'s arity fallback, and the fix refuses the `.length` read, so the whole program stops at compile time. That also means this program no longer observes the ELEMENT half, which is still wrong: the `r15e` pair measures it alone and still asserts SILENT. R-15 is not fixed, and is not retired. The verdict was flipped only after this reading."""
```

- replace `for this repro."""\n\n[[case]]\nname = "r15e_split_element_leaks_a_handle_module_scope"` with:

```
for this repro.

RE-MEASURED 2026-09-11 against node v26.8.2, with the length-fails-closed project's fix: FAIL_CLOSED (was SILENT), byte-identical to the module scope -- exit 1, empty stdout, the same `.length` floor diagnostic; node `len=3` / `1=b`. The element half is measured alone by `r15e`. The verdict was flipped only after this reading."""

[[case]]
name = "r15e_split_element_leaks_a_handle_module_scope"
```

- [ ] **Step 9: Flip R-63's pair to `fail_closed`**

In the same file:

- replace `program = "r63a_module.js"\nverdict = "silent"` with `program = "r63a_module.js"\nverdict = "fail_closed"`, and `program = "r63a_function.js"\nverdict = "silent"` with `program = "r63a_function.js"\nverdict = "fail_closed"`;
- replace `After that flip, a return to SILENT means a `.length` fallback fabricates a number again.\n'''` with:

```
After that flip, a return to SILENT means a `.length` fallback fabricates a number again.

RE-MEASURED 2026-09-11 against node v26.8.2, with the fix: FAIL_CLOSED (was SILENT). kali exits 1 with empty stdout and `error[E5506]` naming "no lane proves its length" on stderr; node prints `3` at exit 0. Closed by the length-fails-closed project: `render_length`'s three fallbacks decline and `emit_unary`'s `.length` floor refuses. FAIL_CLOSED, not FIXED, on purpose: kali still has no lane that computes this length, and says so. The verdict was flipped only after this reading.
'''
```

- replace `MEASURED 2026-09-11 at `152fdd5364` against node v26.8.2: SILENT. kali prints `1` at exit 0 with empty stderr; node prints `3` at exit 0. Byte-identical to the module scope.\n'''` with:

```
MEASURED 2026-09-11 at `152fdd5364` against node v26.8.2: SILENT. kali prints `1` at exit 0 with empty stderr; node prints `3` at exit 0. Byte-identical to the module scope.

RE-MEASURED 2026-09-11 against node v26.8.2, with the fix: FAIL_CLOSED (was SILENT), byte-identical to the module scope. The verdict was flipped only after this reading.
'''
```

Run: `cargo test -p kali_cli --test cases -- oracle/tier2`
Expected: PASS, `104 passed`.

- [ ] **Step 10: Move R-15's and R-63's rows in the register**

In `docs/superpowers/followups/kali-silent-miscompile-register.md`:

1. R-15's §0.2 row: replace `| R-15 `.split()` result | **SILENT** (both scopes) | ` with `| R-15 `.split()` result | **FAIL_CLOSED** (`.length` + element lane `r15`, both scopes) / **SILENT** (element-only lane `r15e`, both scopes) | **Moved 2026-09-11 by the length-fails-closed project, NOT fixed and NOT retired:** the `r15` program's `.length` read now refuses (`E5506`, the `.length` floor), which stops the whole program before its element read; the element read alone still leaks the handle and `r15e` pins that. Before 2026-09-11: `.
2. R-15's §2 entry: replace `  is upstream (`String.prototype.split` receiver guard).\n\n### R-16` with:

```
  is upstream (`String.prototype.split` receiver guard).
- **STATUS 2026-09-11 (length-fails-closed)**: the repro's `.length` read now
  refuses with `E5506` (the `.length` floor), so the repro as written fails closed
  at compile time. **The entry is not fixed**: `p[1]` alone still prints the leaked
  handle at exit 0, measured by the `r15e` oracle lane. §0.2's row carries both
  classes.

### R-16
```

3. R-63's §0.2 row: replace `| R-63 `.length` renders a node's child count, or `0`, for a receiver with no length lane | **SILENT** (both scopes) | **added 2026-09-11 by the length-fails-closed project**,` with `| R-63 `.length` renders a node's child count, or `0`, for a receiver with no length lane | **FAIL_CLOSED** (both scopes) | **RETIRED 2026-09-11 by the length-fails-closed project — its one lane moved.** Re-derived from the two `r63a` cases, which now assert `fail_closed`: kali exits 1 with empty stdout and the `.length` floor's `E5506`; node prints `3`. FAIL_CLOSED, not FIXED: no lane computes this length yet, and kali now says so. Originally **added 2026-09-11 by the same project**,`.
4. R-63's §2 title: replace `### R-63: `.length` renders a node's child count, or `0`, for a receiver with no length lane` with `### R-63: `.length` renders a node's child count, or `0`, for a receiver with no length lane — **CLOSED 2026-09-11 (FAIL_CLOSED)**`.
5. R-63's §2 entry: replace `  §2.2-§2.5).\n\n---\n\n## Tier 3 — silently wrong control flow (value otherwise intact)` with:

```
  §2.2-§2.5).
- **RETIRED 2026-09-11, by the length-fails-closed project, in the commit that
  fixes it.**
  - `emit_unary`'s `.length` floor (`crates/kali_codegen/src/emit/operators.rs`)
    emits the receiver and then refuses with `E5506` instead of pushing `0`;
    `render_length`'s three fallbacks return `None`; and `emit/call.rs`'s
    `String(x.length)` proof is keyed on the member shape, so `String(a.length)`
    over `[1n, 2n]` still prints `2`.
  - Every lane in the table above now refuses (pinned in
    `crates/kali_cli/tests/cases/runtime/length_fails_closed.toml`), and so does
    the same receiver in thirteen consumer positions, both scopes (pinned in
    `length_fails_closed_consumers.toml`). The consumer matrix went from 29 silent
    readings to 0 with no control lost.
  - **What this does NOT close**: `(new Array(3)).length` still prints `1`, through
    the array-literal arms of the same two rungs (`host.rs:1267-1285`,
    `operators.rs:389-413`), which accept a `new` wrapper as a one-element array
    literal. Filed in
    `docs/superpowers/followups/length-fails-closed-discovered-defects.md`.

---

## Tier 3 — silently wrong control flow (value otherwise intact)
```

6. The Movement bullet: replace the sentence Task 1 Step 7 added, ending `See the ranking's §6 EIGHTH amendment.`, by that same sentence followed by:

```
  **2026-09-11, next commit: R-63 leaves, RETIRED, taking 29 back to 28**, and
  R-15 gains a FAIL_CLOSED lane without leaving the silent set (its `r15e` element
  lane is still SILENT). See the ranking's §6 NINTH amendment.
```

- [ ] **Step 11: Remove R-63 from G6**

In `tools/blast-radius/clusters.json`, delete the R-63 assignment object Task 1 Step 13 added (and the comma after the R-60 object that preceded it, so R-60's object again ends with `}` and the array closes). Then replace the final note string `    "section 6.1)."` with:

```json
    "section 6.1).",
    "",
    "REMOVED 2026-09-11, next commit: R-63 leaves G6. Its one lane moved to FAIL_CLOSED when the",
    "length-fails-closed project's fix landed, so it left the SILENT filter, and an entry outside",
    "that filter carries no assignment. G6 keeps its other members, so its definition stays."
```

Check: `node -e 'const c=require("./tools/blast-radius/clusters.json"); console.log(c.assignments.some(a=>a.id==="R-63"), c.assignments.at(-1).id)'` prints `false R-60`.

- [ ] **Step 12: Re-measure accepts and counts with the fixed binary**

```bash
cd /workspace && cargo build -p kali_cli
cd tools/blast-radius && node accepts.mjs && node count.mjs && cd /workspace
git diff --stat tools/blast-radius/accepts.json tools/blast-radius/counts.json
git diff tools/blast-radius/counts.json | grep -E '^[-+]\s+"(id|reachable)"' | head -60
git diff tools/blast-radius/accepts.json | grep -E '^[-+]' | grep -v '^[-+]{3}' | head -40
```

Expected: `count.mjs` still prints `counted 177 programs, 43 countable predicates` (no matcher changed, so `manifest_tests.rs`'s SHAs stay as Task 1 left them). The fix makes `kali check` refuse programs that read a `.length` no lane proves, so `accepts.json` may lose programs and several entries' `reachable` figures may fall. **Copy every changed `reachable` figure and every program that left the accept set into Step 14's amendment.** Task 1 re-measured at the baseline, so all of this movement is this project's.

- [ ] **Step 13: Re-splice the ranking**

```bash
cargo run -q -p kali_blast_radius --example rank > .superpowers/sdd/2026-09-11-length-fails-closed/rank-task3.md
python3 .superpowers/sdd/2026-09-11-length-fails-closed/splice_ranking.py .superpowers/sdd/2026-09-11-length-fails-closed/rank-task3.md
git diff docs/superpowers/followups/blast-radius-ranking.md | grep -E '^[-+]\|' | head -60
```

Expected: R-63 moves out of the SILENT rows (§3's per-entry table shows it excluded by the SILENT filter), R-15's lanes cell becomes `FAIL_CLOSED / SILENT`, G6's row loses R-63, and any `reachable` moves from Step 12 appear.

- [ ] **Step 14: Write the NINTH amendment**

In `docs/superpowers/followups/blast-radius-ranking.md`, insert this paragraph, and a blank line, directly before `### 6.1 The most important thing here is not a rank` (so it follows Task 1's EIGHTH amendment). Fill the brackets from Steps 12-13's output, **copied, not predicted**; the committed text has no brackets:

```
**AMENDMENT 2026-09-11 — a NINTH regeneration, retiring R-63.** The
length-fails-closed project's fix landed: `.length`'s floor refuses and
`render_length`'s fallbacks decline. **R-63** moved to FAIL_CLOSED in both scopes
and left the SILENT filter, and with it G6's assignment. **R-15** gained a
FAIL_CLOSED lane (`r15`, whose repro reads `.length` first) and kept a SILENT one
(`r15e`, the element read alone), so it stays in G6 and in the ranking.
[Either "The accept set did not move." or "The accept set moved: [each program
that left it]. Reachable figures moved on [N] entries: [each as `R-NN a → b`]."]
Nothing moved in `predicates.json`, `matchers.mjs` or the frozen SHAs; a
regression re-lights the same numbers. G6's §2 row is now [the `+| G6` line from
Step 13] (was [the `-| G6` line]). Every figure above is read out of the
regenerated §2–§5 and the `counts.json` and `accepts.json` diffs.
```

- [ ] **Step 15: The gate, then one commit**

```bash
cd /workspace
cargo test -p kali_blast_radius
bash scripts/test-gate.sh
```

Expected: `kali_blast_radius` all pass; `GATE OK: 0 failing tests`.

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_codegen/src/intrinsics/host.rs \
  crates/kali_codegen/src/emit/call.rs \
  crates/kali_cli/tests/cases/runtime/length_fails_closed.toml \
  crates/kali_cli/tests/cases/runtime/length_fails_closed_consumers.toml \
  crates/kali_cli/tests/cases/object/computed_member_static_name.toml \
  crates/kali_cli/tests/cases/oracle/tier2.toml \
  docs/superpowers/followups/kali-silent-miscompile-register.md \
  docs/superpowers/followups/blast-radius-ranking.md \
  tools/blast-radius/clusters.json tools/blast-radius/accepts.json tools/blast-radius/counts.json
git commit -F - <<'EOF'
fix(codegen): .length never invents a number -- the floor refuses and the fallbacks decline

emit_unary's `.length` floor emitted the receiver and pushed I64Const(0) with no
diagnostic; render_length's fallbacks rendered a node's child count or a baked
0. A `.length` read on a receiver no lane proves now refuses with E5506, after
the receiver's own refusal if it has one, and String(x.length) is proven by the
member shape instead of by a rendered value.

Retires R-63 (FAIL_CLOSED, both scopes) and moves R-15's `.length` lane to
FAIL_CLOSED while its element lane stays SILENT. The consumer matrix goes from
29 silent readings to 0 with no control lost. Spec:
docs/superpowers/specs/2026-09-11-length-fails-closed-design.md.

Claude-Session: https://claude.ai/code/session_013rmRTZwswauPtUg3zzTg9k
EOF
```

---

## Task 4: Correct the followups and file what this project did not fix

Spec §6.2 and §6.3. Documents only. Every figure below was measured at `152fdd5364` against node v26.8.2 while the spec and this plan were written; do not add a figure that was not.

**Files:**
- Modify: `docs/superpowers/followups/member-length-renders-the-child-count.md` (top)
- Modify: `docs/superpowers/followups/codegen-array-literal-predicate-is-still-negative-space.md` (top)
- Modify: `docs/superpowers/followups/release-tier-allocation-identity-discovered-defects.md` (ranking note item 1)
- Create: `docs/superpowers/followups/length-fails-closed-discovered-defects.md`

**Interfaces:**
- Consumes: Task 3's pin name `a_dot_member_length_refuses_instead_of_rendering_the_child_count` and R-63's retirement.

- [ ] **Step 1: Close the member-length followup**

In `docs/superpowers/followups/member-length-renders-the-child-count.md`, replace

```
# `.length` on a member-expression receiver renders a NODE'S CHILD COUNT, not a length

**Filed** 2026-09-09
```

with

```
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

**Filed** 2026-09-09
```

- [ ] **Step 2: Correct the predicate followup**

In `docs/superpowers/followups/codegen-array-literal-predicate-is-still-negative-space.md`, replace

```
# The negative-space array-literal predicate is a TRIPLET, not a single mistake — two copies are still live

**Filed** 2026-09-10
```

with

```
# The negative-space array-literal predicate is a TRIPLET, not a single mistake — two copies are still live

> **CORRECTED 2026-09-11** by the **length-fails-closed** project
> (`docs/superpowers/specs/2026-09-11-length-fails-closed-design.md`). **§4's
> ranking is withdrawn, and the prescription in §3 and §4 is falsified by
> measurement.** That project picked this document as its item and spiked exactly
> the move §4 calls "small and well-scoped": both codegen copies narrowed to
> decline the one-child shape a `new` wrapper shares. The full workspace suite
> went from 0 to 32 failures, **three of them silent** (`const arr = [mk(0)]`
> printed `0` where its pin expects `201`, and two `E5506` fail-closed pins started
> exiting 0), and 5 of 12 one-element probe programs went from correct to silently
> wrong, with none improving (spec §2.2). In `kali_codegen`, "not an array literal"
> falls through to fallbacks that fabricate a number, so **narrowing this predicate
> is fail-OPEN until the consumer's floor refuses.** Two facts this document did
> not have: `new C(x)` and `[C(x)]` lower to the **same** LIR node (spec §2.1), so
> no shape check can tell them apart; and the predicate does produce one measured
> wrong value, `(new Array(3)).length` → `1` (node `3`), through the array-literal
> arms of `render_length` and `emit_unary`. That value is still open, and is filed
> in `length-fails-closed-discovered-defects.md` §3. The
> `docs/superpowers/sdd/2026-09-10-release-tier-allocation-identity/` directory and
> the `task-5-report.md`, `task-7-report.md` and `task-8-report.md` files cited
> below were never committed and do not exist in this repository.

**Filed** 2026-09-10
```

- [ ] **Step 3: Point the release-tier defects document at the correction**

In `docs/superpowers/followups/release-tier-allocation-identity-discovered-defects.md`, replace

```
   defect. **This is the single most important thing to read out of Task 8's
   filing**, and it says so itself, with the reasoning.
```

with

```
   defect. **This is the single most important thing to read out of Task 8's
   filing**, and it says so itself, with the reasoning.
   **Withdrawn 2026-09-11:** the length-fails-closed project measured that
   document's prescription fail-open; read the correction at its top first. The
   `task-N-report.md` files this document cites were never committed.
```

- [ ] **Step 4: File what this project measured and did not fix**

Create `docs/superpowers/followups/length-fails-closed-discovered-defects.md`:

````markdown
# Defects the length-fails-closed project measured and did NOT fix

**Filed** 2026-09-11 by the **length-fails-closed** project
(`docs/superpowers/specs/2026-09-11-length-fails-closed-design.md`), on the
convention `release-tier-allocation-identity-discovered-defects.md` and its
predecessors use: a project that measures more than it fixes writes down what it
left, so the silence is not read as absence.

**Oracle:** `node v26.8.2`. **Measured at:** `152fdd5364` (the project's
baseline), with `kali run`, unless a row says otherwise. None of these is in the
register; §1 and §2 are silent wrong values and are candidates for it.

**Ranked, most consequential first.** §1 and §2 are silent wrong values on
ordinary code; §3 is silent but narrow; §4 is a cross-reference; §5 is a lead;
§6 is cleanup.

---

## §1. An inline allocation passed as an argument reaches the callee as zeros

| program | kali | node |
|---|---|---|
| `function f(x) { return x.length; } console.log(f(new Array(6)));` | `0` | `6` |
| `function f(x) { return x[0]; } console.log(f(new Array(3).fill(2)));` | `0` | `2` |
| `function f(x) { return x.length; } console.log(f(new Array(4).fill(1)));` | `0` | `4` |
| the first row inside `function main() { ... } main();` | `0` | `6` |
| `function f(x) { return x.length; } const a = new Array(6); console.log(f(a));` | `6` | `6` (control: a BOUND allocation is correct) |
| `function f(x) { return x.length; } console.log(f([1, 2, 3]));` | `E5506`, exit 1 | `3` (control: an inline LITERAL is refused) |

All silent rows exit 0 with no diagnostic. **Mechanism, partly traced:**
`crates/kali_codegen/src/emit/call.rs:3591-3614` refuses an inline array
**literal** argument because "the callee would read zero placeholders", and its
comment at `:3579-3582` assumes `new Array(n)` / `.fill()` arguments "live in
locals … and pass a real handle, so they are untouched here". That holds for a
bound allocation and fails for an inline one. It is a guard keyed on one form
with a sibling hole. The length-fails-closed fix does not touch it: inside `f`,
`x` is an array binding and its `.length` reads a header, not a fallback.

## §2. `new` swallows the member chain after its arguments

`new` parses its callee with `parse_call_expression()`
(`crates/kali_parser/src/expression/primary.rs:241`), so the chain after the
arguments becomes part of the callee:

| program | parses as | kali | node |
|---|---|---|---|
| `console.log(new Array(3).length);` | `new (Array(3).length)` | `2` | `3` |
| `console.log(new Error("m").message);` | `new (Error("m").message)` | `0` | `m` |
| `const a = new Array(4).fill(7);` | `new (Array(4).fill(7))` | works | — |

The third row is why this has hidden: every `new Array(n).fill(v)` in the
benchmark fixtures parses wrongly and still works, because the wrapping `new`
node is transparent to every consumer. Established by a throwaway LIR dump (spec
§2.1). **A fix changes the LIR shape of every `new X(...).m()` in the corpus**,
including every recognizer tuned to the misparsed shape
(`declarator_init_is_array_fill` and its siblings), so size it by the
recognizers that could see the new shape, not by compiler errors
(`release-tier-allocation-identity-discovered-defects.md` §9).

## §3. `(new Array(3)).length` still prints `1`

`console.log((new Array(3)).length);` prints `1` at exit 0; node prints `3`. The
same value before and after the length-fails-closed fix. `new Array(3)` lowers to
a text-less `Value` with one `Call` child, which is also the shape of `[f()]`, and
two `.length` arms accept it as a one-element array literal and return its child
count: `render_length`'s array-literal arm (`intrinsics/host.rs:1267-1285`) and
`emit_unary`'s (`emit/operators.rs:389-413`).

**Now that the floor refuses, narrowing those two arms is fail-closed for
`.length`** (spec §3.5), but it is unmeasured, and it would refuse
`const a = [f()]; a.length`, which prints `1` correctly today. The other ~36
consumers of the same predicate still have silent floors
(`codegen-array-literal-predicate-is-still-negative-space.md`, corrected
2026-09-11), so they must not be narrowed first.

## §4. `render_static_value` renders an aggregate's child count as a value

`crates/kali_codegen/src/intrinsics/host.rs:928-942` renders a text-less node with
two or more children as `children.len()`. It is the value-position sibling of the
`.length` fallbacks the length-fails-closed project closed, and it is the
mechanism of the register's **R-31** ("`console.log` of an array prints its
length"). Not re-filed; recorded here because a reader of R-63's retirement will
look for it.

## §5. Lead: a nested function reading a captured binding gets a warned zero

`function main() { const arr = [1, 2, 3]; function g() { return arr.length; } console.log(g()); } main();`
printed `0` at exit 0 at `152fdd5364` (node `3`). Its stderr carried only
`warning[E3100]: undefined identifier 'arr' reached codegen and was lowered through
a zero placeholder compatibility fallback`. After the fix it refuses, but only
because the read is `.length`. **Other reads of a captured binding through a
nested function were not measured.** Check the register's R-02 and the Stage C
closure notes before filing: this may be a known closure lane rather than a new
entry.

## §6. The per-hazard `.length` bails are now redundant

`render_length` (`intrinsics/host.rs:1195-1250`) and the runtime member lane
(`emit/control_flow.rs:2490` onward) each carry bails for one shape found "the
expensive way": `URLSearchParams`, `TextDecoder.decode`, `String(...)`, inline
`TextEncoder.encode`, `crypto.getRandomValues`. Each exists because that shape fell
through to a fabricated count. With the floor refusing, each bail now declines
onto a refusal the floor would give anyway. Removing them is a behaviour-neutral
cleanup **only if** each bail's own case keeps its current message, because
several of those cases pin a specific needle. The length-fails-closed project left
them alone (spec §3.4).
````

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/followups/member-length-renders-the-child-count.md \
  docs/superpowers/followups/codegen-array-literal-predicate-is-still-negative-space.md \
  docs/superpowers/followups/release-tier-allocation-identity-discovered-defects.md \
  docs/superpowers/followups/length-fails-closed-discovered-defects.md
git commit -F - <<'EOF'
docs(followups): close member-length, withdraw the predicate prescription, file what was left

member-length-renders-the-child-count closes with R-63. The negative-space
predicate followup's ranking and prescription are withdrawn: narrowing it was
measured fail-open. New: an inline allocation argument reads zeros, `new`
swallows the member chain after its arguments, `(new Array(3)).length` still
prints 1, and three smaller items.

Claude-Session: https://claude.ai/code/session_013rmRTZwswauPtUg3zzTg9k
EOF
```

---

## Task 5: Final verification against the baseline

Spec §4.5. **No planned file changes.** A failure here is a finding to report, not something to paper over.

**Files:** none, unless `cargo fmt` rewrites something (Step 3).

- [ ] **Step 1: The full gate**

Run: `bash scripts/test-gate.sh`
Expected: `GATE OK: 0 failing tests`.

- [ ] **Step 2: The test count moved by exactly what this project added**

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | grep '^test result:' | awk '{p += $4; f += $6} END {print "passed=" p, "failed=" f}'
```

Expected: `passed=12019 failed=0`. The baseline ran 11,929 (measured during planning: `passed=11903 failed=26` under a spike of the same suite). This project adds 86 runtime cases, the `r63a` pair and the `r15e` pair: 11,929 + 86 + 2 + 2 = 12,019. The rename in Task 3 Step 7 adds nothing. If the figure differs, account for every test of the difference in the report before continuing.

- [ ] **Step 3: What CI runs that the gate does not**

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

Expected: both exit 0. If `cargo fmt --all -- --check` reports a diff, run `cargo fmt --all`, check that `git diff` touches only files this branch changed, then commit it with the message `style: cargo fmt` and the session trailer. Fix any clippy finding in code; never silence one with `#[allow]`.

- [ ] **Step 4: Nothing outside the spec's list changed**

```bash
git diff --name-only 152fdd5364 -- ':!docs/superpowers/specs' ':!docs/superpowers/plans' | sort
```

Expected: exactly these 30 paths and no others, or 29 if neither Task 1 nor Task 3 moved `accepts.json`.

```
crates/kali_blast_radius/src/catalogue_tests.rs
crates/kali_blast_radius/src/manifest_tests.rs
crates/kali_blast_radius/src/oracle_tests.rs
crates/kali_blast_radius/src/register_tests.rs
crates/kali_cli/tests/cases/browser/promise_all_bundle.toml
crates/kali_cli/tests/cases/browser/promise_all_settled_bundle.toml
crates/kali_cli/tests/cases/object/computed_member_static_name.toml
crates/kali_cli/tests/cases/oracle/tier2.toml
crates/kali_cli/tests/cases/runtime/length_fails_closed.toml
crates/kali_cli/tests/cases/runtime/length_fails_closed_consumers.toml
crates/kali_cli/tests/runtime_smoke.rs
crates/kali_codegen/src/emit/call.rs
crates/kali_codegen/src/emit/operators.rs
crates/kali_codegen/src/intrinsics/host.rs
crates/kali_codegen/src/intrinsics/host_tests/crypto.rs
docs/superpowers/followups/blast-radius-ranking.md
docs/superpowers/followups/codegen-array-literal-predicate-is-still-negative-space.md
docs/superpowers/followups/kali-silent-miscompile-register.md
docs/superpowers/followups/length-fails-closed-discovered-defects.md
docs/superpowers/followups/member-length-renders-the-child-count.md
docs/superpowers/followups/release-tier-allocation-identity-discovered-defects.md
tools/blast-radius/accepts.json
tools/blast-radius/clusters.json
tools/blast-radius/count.mjs
tools/blast-radius/counts.json
tools/blast-radius/matchers.mjs
tools/blast-radius/matchers.test.mjs
tools/blast-radius/predicates.json
tools/task-18-browser-pilot/batch8a_captures.py
tools/task-18-browser-pilot/gen_batch8a.py
```

(That list is 30 paths; `accepts.json` is absent from it if neither Task 1 nor Task 3 moved the accept set, which makes 29. Report the count you get and why.)

- [ ] **Step 5: The out-of-scope values did not move by accident**

```bash
cd /workspace
mkdir -p .superpowers/sdd/2026-09-11-length-fails-closed/final
cd .superpowers/sdd/2026-09-11-length-fails-closed/final
printf 'console.log((new Array(3)).length);\n' > new_array_paren.js
printf 'function f(x) { return x.length; } console.log(f(new Array(6)));\n' > inline_allocation_argument.js
printf 'console.log(new Array(3).length);\n' > new_precedence.js
for f in new_array_paren.js inline_allocation_argument.js new_precedence.js; do
  printf '%-32s kali=%s node=%s\n' "$f" "$(/workspace/.cache/cargo-target/debug/kali run $f 2>&1 | head -1)" "$(node $f)"
done
```

Expected, unchanged from the baseline and recorded in Task 4's discovered-defects document: `new_array_paren.js kali=1 node=3`, `inline_allocation_argument.js kali=0 node=6`, `new_precedence.js kali=2 node=3`. If any of them changed, the change is unexplained by the spec: stop and report it.

- [ ] **Step 6: Report and stop**

Write `.superpowers/sdd/2026-09-11-length-fails-closed/task-5-report.md` (untracked) with each step's command output. Do not push, and do not open a pull request: hand the branch back to the human partner.
