# Divergences and tripwires the computed-member-static-name project found and did NOT fix

**Filed** 2026-09-09, at `71b5f42f6c`, by the **computed-member-static-name**
project (`docs/superpowers/sdd/2026-09-08-computed-member-static-name/`), on the
convention this repository already uses for
`console-render-unification-discovered-defects.md`: a project that measures more
than it fixes writes down what it left, so a later reader does not read the
silence as absence.

**Oracle:** `node v26.8.1`. **Baseline:** `dc19c3a040`. **Every reading below was
re-measured at `71b5f42f6c`** on `.cache/cargo-target/debug/kali`.

**Two of the six things this project found are large enough to have their own
files and are not repeated here**, only named:

* `member-length-renders-the-child-count.md` — `.length` on a member-expression
  receiver renders a node's arity. This project moved its computed half and left
  its **dot** half silent.
* `release-mode-optimizer-inlines-an-allocating-initializer.md` — the optimizer
  inlines an allocating array initializer at `--release`, so the release tiers
  refuse programs `--fast` compiles correctly **and silently miscompile others**.
  The most consequential of the six. **It hits THREE Benchmarks Game fixtures,
  not two**: `spectral-norm`, `nbody` and — added 2026-09-09 at `6b59ddeef9` —
  `fannkuch-redux`, which the original sweep missed for the reason §6 below
  gives.

The four below are the rest, **and section 5 — the twin gaps — was added by
the branch's final review**: this file enumerated six findings and not one of
them was a twin gap, which is the one class this project exists to close.

**Section 6 was added later still**, 2026-09-09 at `6b59ddeef9`, by the fix
round that unblocked PR #36. It is not a computed-member defect at all; it is
the instrument defect that let one of this project's own measurements be wrong,
and it is filed here because this is where the reader of that measurement will
be standing.

---

## 1. `process.argv` distinguishes `[2]` from `["2"]`, which JavaScript does not

**Silent, exit 0, both readings.** Measured under `--api node` with `-- 5`,
against `node argv.js 5`:

```js
console.log(process.argv[2]);                       // kali 5   node 5
const i = 2; console.log(process.argv[i]);          // kali 0   node 5
console.log(process.argv["2"]);                     // kali 0   node 5
```

**This is NOT a fold defect, and the discriminating reading is the third line.**
The folded spelling `process.argv[i]` agrees **exactly** with the string-literal
spelling `process.argv["2"]` — both print `0` — which is this project's whole
contract: a folded bracket access behaves as the literal bracket access it
denotes. The root defect is upstream of that: codegen's argv element lane
(`crates/kali_codegen/src/intrinsics/host.rs`, `is_process_argv_element`)
requires the **index child's text** to parse as a non-negative integer literal,
so `argv[2]` and `argv["2"]` take different lanes where JavaScript treats them
as the same property key — property names are strings, and `2` and `"2"` denote
one name.

Pre-existing and unchanged from `dc19c3a040`. `kali check` is clean on the
program. **Pinned WRONG ON PURPOSE** as
`the_argv_index_lane_disagrees_with_itself_between_a_number_and_a_string_spelling`
in `crates/kali_cli/tests/cases/object/computed_member_static_name.toml`
(controller ruling R15).

**Suggested home:** §2, silent. Its fix unit is the argv element lane's index
recognizer, which should key on the property NAME (one formatter, as everything
else in this family now does) rather than on the index child's literal form.

## 2. A BigInt rendering divergence on the newly-live bracket-store lane

**Silent, exit 0.** `check` clean.

```js
let o = {a: 1, b: 7n}; o["a"] = 5; console.log(o.b);   // kali 7   node 7n
let o = {a: 1, b: 7n}; o.a  = 5; console.log(o.b);     // kali 7   node 7n   ← control
```

**The value is right; the `n` suffix is missing.** This is a RENDERING
divergence, not a wrong value, and it is **not new**: the dot spelling has always
had it, as the control shows. What this project changed is reach — the bracket
store now lands, so it reaches the same 2.6-before-2.7 materialization hole the
dot spelling reaches, and inherits the existing defect rather than introducing
one.

**Pinned WRONG ON PURPOSE** as
`a_bracket_store_reaches_the_same_bigint_rendering_hole_the_dot_spelling_has` in
the same case file, so a later BigInt rendering fix moves it deliberately.

**Suggested home:** a lane of whatever entry owns BigInt field rendering, not a
new entry — the control proves the bracket spelling is not the subject.

## 3. A `Uint8Array` TRIPWIRE — not a live defect today

Codegen's `is_array_like_constructor`
(`crates/kali_codegen/src/emit/call.rs:5509`) recognizes **`Array` and
`Uint8Array`**, in bare and `globalThis["Uint8Array"]` spellings. The checker's
allocation recognizer `expression_is_array_allocation`
(`crates/kali_types/src/resolve/expression.rs:807`) recognizes **`Array` only**.
That is exactly the asymmetry spec §8 forbids — a checker that admits less than
codegen is safe, but a checker that admits MORE is a defect, and the shape here
is one wrong step away from the second.

**It is unreachable today, and that is measured, not assumed.** `Uint8Array` is
an undefined identifier in this phase, so **both twins refuse identically**, all
three spellings, exit 1 on `run` and on `check`:

| program | kali `run` | kali `check` | node |
|---|---|---|---|
| `const u = new Uint8Array(3); console.log(u[0]);` | `E3100`, exit 1 | `E3100`, exit 1 | `0` |
| `const u = new Uint8Array(3); let i=0; console.log(u[i]);` | `E3100`, exit 1 | `E3100`, exit 1 | `0` |
| `const u = new Uint8Array(3); u[0]=1; console.log(u[0]);` | `E3100`, exit 1 | `E3100`, exit 1 | `1` |

Message, both twins: `error[E3100]: undefined identifier 'Uint8Array'`.

> **THE TRIPWIRE.** If `Uint8Array` is ever admitted as an identifier, the
> checker's `expression_is_array_allocation` must gain that shape **in the same
> commit**, or a check-refuses/run-admits gap opens on every
> `new Uint8Array(n)` binding indexed by a non-literal — `kali check` would
> refuse with the computed-member `E5506` while `kali run` compiled it through
> codegen's array-like lane. Whoever admits the identifier owns this.

Nothing pins it, because there is nothing to pin: a case asserting today's
`E3100` would go red the moment the identifier is admitted, which is the right
moment for a human to read this section, but it would go red for a reason that
looks like an unrelated feature landing. Recorded here instead, deliberately.

**Suggested home:** not §2 — it is not a divergence from node. A note on
whichever plan item admits typed arrays.

## 4. The batch-3 case generator can silently REVERT two measured re-pins

`crates/kali_cli/tests/cases/runtime/join.toml` and
`crates/kali_cli/tests/cases/runtime/string_value_flow.toml` carry
`# GENERATED by tools/migration/gen_task19_batch3.py -- do not edit this file by
hand`. Both were hand-edited by this project's Task 7 to re-pin expectations the
binary no longer produces.

**The generator has no re-pin channel.** Its expectations are *derived* from the
assertions of deleted `.rs` sources read at the Task 19 deletion ref
(`t19b3_extract.claims_of`, ref `cc76f5a918`), so running it with `--write`
would **re-render the old expectation** — the one the binary no longer
produces — and the measured re-pin would be gone with no diagnostic. The two are
irreconcilable once behaviour moves, and the hand edit is the only
representation of the measurement.

**Measured consequence**, `python3 tools/migration/gen_task19_batch3.py` at
`71b5f42f6c` (exit 120):

```
GENERATOR FAILED
  DRIFT: runtime/join.toml is not what this spec renders (run --write to see the diff)
  DRIFT: runtime/string_value_flow.toml is not what this spec renders (run --write to see the diff)
  runtime/join: audit-case-migration.py exits 1 and the header declares no EXPECTED-RED for it (ruling 9: name EVERY gate that is expected-red)
  runtime/join: check_extra_claims.py exits 1 and the header declares no EXPECTED-RED for it (ruling 9: name EVERY gate that is expected-red)
  runtime/string_value_flow: audit-case-migration.py exits 1 and the header declares no EXPECTED-RED for it (ruling 9: name EVERY gate that is expected-red)
  runtime/string_value_flow: check_extra_claims.py exits 1 and the header declares no EXPECTED-RED for it (ruling 9: name EVERY gate that is expected-red)
```

Those gates run only under `bash scripts/test-gate.sh --gates-only`, which no CI
job invokes and which the plain gate does not run — so **nothing in CI notices
this drift**, and nothing in CI would notice a `--write` reverting the re-pin
either. What WOULD notice is `cargo test -p kali_cli --test cases`, which would
go red immediately on the reverted expectations.

**Controller ruling R24: the re-pin channel was deliberately NOT built.** A
revert surfaces at once as red cases rather than shipping a wrong expectation, so
the exposure is bounded, and building the channel inside a project scoped to
computed member access would have been inventing scope. **The exact three edits
the channel needs are named**, in both files' own headers and in Task 7's report:

1. a declared `(stem, fn)` table that overrides the derived `exit` / `stdout` /
   `stderr_contains` and carries the dated measurement — the shape
   `t19b3_extract.COMPUTED` / `LCG_STDOUT` already has, generalized;
2. an arm in `gen_task19_batch3`'s claim-sentence renderer, so the rationale
   narrates the override instead of claiming the pin came from the source;
3. `EXPECTED-RED (rc=1)` header declarations for `audit-case-migration.py` and
   `check_extra_claims.py`, because a re-pin necessarily drops a source literal
   that no longer appears in the binary's output and both of those are
   literal-coverage checks.

A dated warning header was added to both files recording the hand edit, why the
generator could not carry it, and that the generator check is red.

**Suggested home:** not the register at all — this is a tooling defect, and it
belongs wherever the case-migration machinery is owned. **It is a PLAN defect
too, and worth saying once plainly:** the plan did not anticipate that a re-pin
target would be a byte-pinned generated file, and any future project that
re-pins behaviour will meet the same wall.

## 5. THE TWIN GAPS — `check`-clean / `run`-refuses, three of them, all shipping

**Added 2026-09-09 by the branch's final whole-branch review (finding 2), and
re-measured at `826861b32d` against node v26.8.1 before being written down
(that is the last commit on this branch that changed behaviour; everything after
it is comment and prose).**
Every reading below is from `.cache/cargo-target/debug/kali`; all three
reproduce.

Spec §8 names twin disagreement as **this design's own falsifier**: `kali check`
and `kali run` must refuse the same programs with the same message, and codegen
may refuse a strict subset of what the checker admits only in the direction
where the checker's refusal wins. The reverse — the checker admitting what
codegen refuses — is the defect. Three programs ship in that shape. Each is
**fail-closed** (`run` refuses; nothing is silently miscompiled) and each is
pinned, so none is an unrecorded risk; but a reader who comes to this file for
"what this project left behind" would have found the falsifier's own class
missing from it, which is why it is written down here.

| program | `kali check` | `kali run` | node | pinned by |
|---|---|---|---|---|
| `const o = {a: "xyz"}; const k = "a"; console.log(o[k].length);` | exit 0, `Checked 1 file(s)` | `E5506` computed-member, exit 1 | `3` | `check_still_admits_the_chained_access_off_a_folded_member` |
| `const o = {a:1,b:2,c:3,d:4}; const k = "keys"; console.log(Object[k](o).length);` | exit 0, `Checked 1 file(s)` | `E5506` computed-member, exit 1 | `4` | `check_still_admits_the_computed_callee` |
| `const a = [1,2]; const b = a; b[0] = 7; console.log(b[0]); console.log(a[0]);` | exit 0, `Checked 1 file(s)` | `E5506` computed-member, exit 1 | `7`, `7` | nothing — recorded only in **R-12**'s §0.2 row |

The first two pins live in
`crates/kali_cli/tests/cases/object/computed_member_static_name.toml`, each
beside its `run`-refuses twin, and each rationale already says the disagreement
is deliberate and unowned. **The third is pinned nowhere as a case**: R-12's
re-derived §0.2 row in `kali-silent-miscompile-register.md` records "`kali check`
still exits 0 on the aliased program while `run` refuses", and the oracle harness
cannot see it because the harness observes `run` only.

**Why the first two happen, mechanically.** Both are the **by-id decline** of
spec §4.3. Codegen's fold does not write the folded name back onto the LIR node:
`named_twin` (`crates/kali_codegen/src/emit/computed_member.rs`) returns a
*clone* carrying the name, because the emitter borrows the program immutably and
cannot mutate it — the function's own doc comment says so. So the folded access
is re-dispatched through the literal spelling's lanes, but any consumer that
reaches the node **by id** — the outer `.length` member in `o[k].length`, the
callee lookup in `Object[k](o)` — still sees `LirNodeKind::ComputedMember` and
declines by construction, exactly as §4.3 designs every by-id consumer to.
`kali check` has no such by-id step for these two compositions, so it admits
them. Note that refusing is still the RIGHT outcome for both under `run`: at the
baseline `dc19c3a040` each printed a silent wrong `2` from `render_length`'s
text-less arm, so this is a silent wrong answer replaced by an honest refusal,
not a working lane regressed.

The third has a different mechanism — the aliased literal-array store is refused
by the store choke point rather than by a by-id decline — but the same shape:
`run` refuses, `check` does not.

**What closing them would require.** A **checker rule** of the form *"a member
whose object is a nameless computed member refuses"*, in
`crates/kali_types/src/resolve/expression.rs`, so `check` reaches the same
verdict `run` does without either twin learning to fold more. That single rule
covers gaps 1 and 2 (the chained member and the computed callee are both
"something applied to a nameless computed member"); gap 3 needs the checker's
array-store lane to see the alias the way codegen's store choke point does, which
is a separate rule and probably belongs with R-12 rather than here.

**Suggested home:** not §2 of the register — none of the three is a silent
divergence from node; all three are fail-closed. They belong to whatever plan
item closes the checker's admit list, and the two pinned cases are the tripwire
that will go red on the day it is closed, which is the day a human should read
this section.

## 6. The incremental cache key carries no compiler-build identity, so a stale artifact can mask a semantics change

**This is a TOOLING defect, and it already caused a false measurement on this
very branch.** Filed 2026-09-09 at `6b59ddeef9`.

**The mechanism.** `compile_source_file`
(`crates/kali_cli/src/build/compile.rs`) short-circuits on an on-disk wasm cache:
a hit returns the cached bytes at lines 237-244 and never runs
resolver / HIR / MIR / LIR / optimizer / **codegen**. The key is built at lines
567-578 and is composed **entirely of inputs** — source hash, build mode, API
surface, specialization budget, runtime profiles, profile data, `compat_eval`,
`coverage` — and then ends in `env!("CARGO_PKG_VERSION")`, which is frozen at
`"0.1.0"` and has never moved. **Nothing in the key identifies the compiler
build.** An artifact therefore survives arbitrary compiler-semantics changes and
is served to a compiler that would no longer produce it.

**The consequence, measured rather than hypothesised.**
`crates/kali_cli/tests/fixtures/kali.json` makes the fixtures directory a project
root, so `crates/kali_cli/tests/fixtures/.kali-cache/incremental/` is a live,
gitignored, machine-local cache that every fixture-driven test reads. It held two
release-tier artifacts for `fannkuch-redux-benchmark-v1.ts` written **2026-07-16
by the pre-project compiler**:

```
sha256-18aab710…-release-deno-16-profiles:-profile:none-false-false-0.1.0.wasm
sha256-18aab710…-release-advanced-deno-16-profiles:-profile:none-false-false-0.1.0.wasm
```

`E5506` is emitted at **codegen**, so a cache hit cannot produce it. With those
two files present, `fannkuch_redux_builds_in_all_release_modes`
(`crates/kali_cli/tests/inprocess/release_constant_condition_loop.rs`) passed in
`0.00s` — three `fs::read`s, not three release compiles. With them moved aside it
failed immediately, identically to CI. **So a local `bash scripts/test-gate.sh`
reporting GATE OK was meaningless for this branch**, and the sweep that re-pinned
`spectral-norm` and `nbody` for the optimizer defect skipped fannkuch not by
oversight but because the machine doing the sweeping was being told fannkuch was
fine. CI caches only `~/.cargo` and `target`, so every runner is cold, compiles
for real, and went red on both ubuntu and macOS.

**The shape of the fix.** Add a compiler-build discriminator to the cache key —
a build fingerprint, a `git describe`, or any value that changes when the
compiler changes — so an artifact produced by a different compiler misses
instead of hitting. Adding it invalidates every existing entry, **by design**.

**Why it was NOT fixed in the same commit as this entry.** This repository has
no such value today and nothing computes one: there is no build script under
`crates/*/src`, no `vergen`, no `GIT_HASH`, no build-info crate — the only
version-shaped constant anywhere is the same frozen `CARGO_PKG_VERSION` (it is
also what `crates/kali_cli/src/build/metadata.rs:90` stamps into build metadata).
Producing one therefore means **new build machinery** — a build script plus, in
practice, a dependency — which is a change to how every crate in the workspace
is built and does not belong inside a PR scoped to computed member access. The
alternative single-site hack, deriving a discriminator at run time from
`std::env::current_exe()` metadata, was considered and rejected: it silently
re-namespaces the cache per **calling binary** (the `kali` binary and each
in-process test binary would get separate entries) and cold-starts the fixture
cache on every rebuild — a real behavioural change to a caching layer, made
sideways, in a PR about something else.

**Until it is fixed, the operational rule is:** a green run of any
fixture-driven test on a machine with a warm `.kali-cache` proves nothing about
a compiler-semantics change. Clear the relevant entries, or read the timing — a
release-mode compile that finishes in ~0.00s is a cache hit, not a pass.

**Suggested home:** not the register — this is not a divergence from node. It
belongs wherever the build/caching machinery is owned, next to §4's generator
defect, which is the same class: an instrument that can quietly report the wrong
thing.
