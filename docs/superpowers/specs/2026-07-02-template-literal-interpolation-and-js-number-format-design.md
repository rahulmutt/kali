# Template-literal interpolation + exact JS number formatting (2026-07-02)

Closes both followups recorded by the float-console work
(`2026-07-02-float-console-and-reserved-names.md`): template literals printing
their raw source at runtime, and the host/browser `String(number)` format
divergence.

Approval note: the scope answer ("address both followups") came from the user;
the two design choices below (parser desugar; ryu-js) follow the recommended
options after the user went idle mid-review, and are subject to the written
spec review before any implementation starts.

## 1. Runtime template-literal interpolation

**Root cause (traced, not assumed).** The lexer scans an entire backtick span
— including `${...}` — into a single `TokenType::Template` token
(`kali_lexer/src/template.rs:6-41`), and the parser collapses it into a plain
`Expression::Literal` string carrying the raw source text including backticks
(`kali_parser/src/expression/primary.rs:67-71`). `Expression::TemplateLiteral`
is never constructed anywhere in the parser, so the structured HIR lowering
(`kali_hir/src/lowering/expression.rs:96-113`) is dead code. Codegen receives
the raw text, strips the delimiters (`strip_string_delimiters`), interns it,
and prints it verbatim — hence `` console.log(`v: ${7 / 2}`) `` printing
`v: ${7 / 2}`. The two static resolvers
(`kali_cli/src/build/eval.rs:648-670`, token-stage constant eval;
`kali_types/src/static_analysis/string.rs:45-52`, identifier/string-`+`-only)
serve other pipeline stages and never write an interpolated value back into
the literal codegen emits.

**Design: desugar in the parser to a string-`+` chain.** In
`primary.rs:67-71`, when a `Template` token's text contains `${`, split it
into quasi/expression segments (same delimiter-and-nesting-aware scanning
contract as `find_template_expression_end` in `kali_common/src/template.rs` —
reuse or share, do not re-invent) and sub-parse each `${...}` segment as an
expression. Build a left-associated `+` chain alternating plain string
literals (the quasis) and the parsed expressions. The leading quasi is always
emitted as a string literal — even when empty — so the chain is string-valued
from its first operand:

- `` `v: ${7 / 2}` `` → `"v: " + (7 / 2)`
- `` `${a}${b}` `` → `"" + a + b`
- `` `x` `` (no `${`) → unchanged plain literal path, byte-for-byte today's
  behavior.

Everything downstream already works: the codegen string-`+` path
(`kali_codegen/src/emit/operators.rs:596-604`) stringifies operands via
`emit_as_string`, which routes float-shaped values through the
`float_to_string` bridge and everything else through `int_to_string`, then
concatenates via `string_concat`. This delivers the agreed scope: `${expr}`
supports exactly what `"a" + expr` supports, with identical stringification
semantics. No new host imports, so the four hand-mirrored JS import lists and
`FUNCTION_INDEX_OFFSET` are untouched.

**Why not the alternatives.** A structured `TemplateLiteral` pipeline
(parser → HIR → new MIR/LIR kinds → new codegen arm) touches five crates and
breaks every consumer that pattern-matches raw backtick literal text, for no
additional observable behavior; codegen-local re-parsing of `${` segments at
emit time would put a parser inside the emitter with manual scope wiring.
Both rejected.

**Compatibility constraints (regression gates, not assumptions):**

- `kali_fmt` formats from tokens (`formatter.rs:102` emits `Template` tokens
  raw) — unaffected; pin with a formatter idempotence test on an interpolated
  template.
- CLI constant eval (`eval.rs`) matches `TokenType::Template` on the token
  stream — unaffected by an AST-level desugar.
- `kali_types` static string analysis resolves `` `${prefix}${suffix}` `` for
  for-of iteration and import specifiers by matching backtick literal text.
  After desugaring, those sites see `"" + prefix + suffix`; the resolver
  already handles static-string `+` concatenation, so they should still
  resolve — the existing tests
  (`for_of_template_literal_string_iteration_*`, browser variants,
  `runtime_smoke` dynamic-import template-specifier tests) are the gate. If
  any of them regress, the desugar must be fixed, not the tests.
- A `${` with no matching `}` inside a template is a parse error emitted
  through the parser's existing diagnostic machinery (planning picks the
  concrete code alongside the parser's current conventions); today it
  silently prints raw text.

**Out of scope:** tagged templates (the parser never built them; their
behavior does not change), nested-template edge cases beyond what the shared
segment scanner already supports (it handles nested backticks/quotes/braces),
and `toFixed`.

## 2. Exact JS `String(number)` on the wasmtime host

**Root cause.** `format_js_number`
(`kali_runtime/src/host/imports_default.rs:827-838`) falls back to Rust's
`format!("{value}")`, which never uses JS's exponent-notation thresholds. The
divergence is reachable: `console.log(1 / 10000000)` prints `1e-7` in a
browser (JS mirrors use native `String(value)`) but `0.0000001` on the host.
The `|x| ≥ 1e21` side is currently unreachable from source but falls out of
the same fix.

**Design: delegate the finite case to `ryu-js`.** Add `ryu-js` (v1.0.2,
Boa project — implements the ECMA-262 Number→String algorithm exactly,
including the `n > 21` / `n ≤ -6` exponent thresholds and the `1e+21` plus
sign) to `[workspace.dependencies]` and reference it from `kali_runtime` with
`{ workspace = true }`. `format_js_number` keeps its explicit
NaN/Infinity/±0 guards and delegates finite values to a `ryu_js::Buffer`.
Availability verified: `cargo add ryu-js --dry-run` resolves v1.0.2 in this
environment. Fallback (only if the dependency cannot land): hand-roll the
spec algorithm from shortest-round-trip digits with the same test battery.
Remove the "known divergence" caveats from the `format_js_number` rustdoc and
the float-console spec once the divergence no longer exists.

**Semantics after the change:** host output is byte-identical to JS
`String(value)` for all doubles. `int_to_string`/`float_to_fixed` are
untouched.

## Verification

- New runtime test: `` console.log(`v: ${7 / 2}`) `` prints `v: 3.5` on the
  wasmtime host lane and in a browser bundle (node harness lane).
- Interpolation semantics tests mirroring the string-`+` suite: identifier,
  float expression, int expression, adjacent `${a}${b}`, string-valued
  segment, template without interpolation unchanged, unterminated `${`
  diagnostic.
- Existing template pins stay green: for-of template-literal iteration
  (host + browser), dynamic-import template specifiers, formatter tests.
- `format_js_number` unit tests on spec boundaries: `1e-7` vs `0.000001`
  (largest non-exponent small magnitude), `1e+21` vs
  `100000000000000000000`, negatives, `NaN`/`±Infinity`/`±0` unchanged.
- Differential test: host `format_js_number` output equals
  `node -e 'String(v)'` over a curated value table (node is already the
  default harness lane).
- End-to-end divergence-closure test: `console.log(1 / 10000000)` prints
  `1e-7` on BOTH the host lane and a browser bundle.
- Gates are the named non-browser lanes plus the node harness lane
  (`cargo test --workspace` remains unusable: pre-existing chromium-sandbox
  failures). Repo hygiene: `cargo fmt` no diff; clippy `-D warnings` on the
  touched crates.

## Implementation notes (2026-07-02)

Task 2 added a full-consumption check beyond the plan's verbatim code: leftover
tokens inside a `${...}` interpolation (e.g. `` `${1 2}` ``) now report E2004
instead of being silently dropped (review finding, fixed in the Task 2 commit).

Task 3's plan-mandated fixture `` `hi ${name}!` `` (string variable) was
replaced with the inline-literal `` `hi ${"kali"}!` ``: a string-typed *variable*
operand in `+` is a pre-existing direct-runtime-path limitation (E3200,
`reject_unsupported_string_variable_addition`, pinned by
`string_typed_variable_plus_operands_are_rejected` in imperative_core_runtime.rs)
that plain `"hi " + name` hits identically on main; the desugared template
correctly behaves like its equivalent `+` chain, and the new
`run_rejects_string_variable_interpolation_with_e3200` test pins that clean
rejection. Interpolating string *variables* therefore remains unsupported until
the string-local repr limitation is lifted (follow-up).

The plan's Step 5 gate `cargo test -p kali_cli --test runtime_smoke -- template`
has 7 pre-existing failures on main (`build::[json_]build_emits_browser_bundle_chunks_for_template_literal_dynamic_imports*`,
"unexpected chunk result 7") unrelated to this plan; the gate for this branch is
"identical to the main baseline: 4 pass, those exact 7 fail".
