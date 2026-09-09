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
  The most consequential of the six.

The four below are the rest.

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
