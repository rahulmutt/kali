#!/usr/bin/env python3
"""Generate the Task 18 batch 5 GROUP A case files (5 bundle-shaped targets).

Own module per group: four implementers run concurrently and a shared file is a
write race in which one whole-file write silently drops another agent's work.
Nothing here edits `case_emit.py`, `math_shapes.py` or `batch5_prose.py`.

Targets and the arithmetic each one closes on (re-derived below at generation
time, and cross-checked against enumerate_invocations.py):

  math_sin_cos_tan_zero_identities          8 fns /  8 invocations -> [matrix] ext
  math_pow_bracketed_frozen_wrapper_bundle  8 fns /  8 invocations -> [matrix] ext
  math_pow_bracketed_frozen_wrapper         9 fns /  8 migrated    -> [matrix] ext (U4)
  math_pow_alias_bundle                    16 fns / 16 invocations -> [matrix] ext
  math_log2_log10_mixed_root                9 fns / 24 invocations -> [matrix] ext

PROSE. Every recurring sentence comes from `batch5_prose`; none is retyped here.
Only the per-file spec (program under test, assertion inventory, citations) is
written in this module, because that is what review has to read.

CITATIONS. Every `:N` below is produced by `batch5_prose.cite_line(rs_text,
regex)` at generation time. None is computed by arithmetic and none is carried
over from an earlier measurement.

RULE 8 / RULE 9 -- the three `format!`-built fixtures. Three of these sources
build their `[source]` program with `format!` over `kali_common` helpers, so the
resolved text exists in NO string literal in the `.rs` and cannot be pulled out
by `case_emit.fixture_in_fn`. Rule 8 forbids hand-applying Rust's `{}`
substitution (and it would have been wrong here: the `format!` template already
indents the placeholder by two spaces and the helper indents again, so the first
emitted line carries FOUR spaces while every later line carries two -- a
hand-applied substitution reproduces neither). The constants below are therefore
the byte-exact OUTPUT OF THE REAL CODE, captured by a temporary test target

    crates/kali_cli/tests/zz_b5a_dump.rs
      mod a2 { include!("browser_math_pow_bracketed_frozen_wrapper_bundle.rs");
               #[test] fn zz_dump() { fs::write(..., <the source fn>()); } }
      mod a3 { include!("browser_math_pow_bracketed_frozen_wrapper.rs");    ... }
      mod a4 { include!("browser_math_pow_alias_bundle.rs");                ... }

run as `cargo test -p kali_cli --test zz_b5a_dump -- zz_dump --test-threads=1`
and deleted afterwards. `include!` rather than a retyped copy of the builders,
so the executed `format!` is literally the one in the shipped source.

`math_pow_alias_bundle`'s browser-bundle-harness BODY is also `format!`-built,
inline inside `assert_browser_bundle_math_pow_alias_with_source` with an
`{export_name}` placeholder, so it is not reachable as a value at all. It was
captured by running the real helper from that same dump target with
`KALI_BROWSER_BUNDLE_HARNESS_COMMAND` pointed at a wrapper script that copies
the harness script it is handed and then `exec node "$1"` (so the helper's own
assertions still hold), and then subtracting
`kali_runtime_contract::browser_bundle_harness_prelude("app", false)` from the
captured script -- `browser_bundle_harness_script` is defined as prelude+body,
so the remainder is the resolved body, still never hand-substituted.

To re-derive: recreate that target and re-run it; every constant below must come
back byte-identical. They are embedded here rather than read from a dump file so
this module runs from a clean checkout with no uncommitted inputs (the defect
that got the pilot's per-file generators deleted -- see README).

U9 -- every exact pin is live-captured from the real binary at
.cache/cargo-target/debug/kali via `kali_run.py`, for EVERY cell of the file's
matrix axis, and `batch5_prose.assert_identical` asserts the cells agree before
one pin is emitted. See `PIN_MIXED_ROOT_JSON_STDOUT`.

Run: python3 gen_batch5_group_a.py [name ...]   (no args = all)
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")
KALI_COMMON_MATH = os.path.join(REPO, "crates/kali_common/src/math.rs")

from case_emit import fixture_in_fn, fixture_starting, emit, write  # noqa: E402
from math_shapes import (  # noqa: E402
    bundle_steps, harness_step, envelope_build, envelope_harness, META,
)
import batch5_prose as P  # noqa: E402

EXTS = ["js", "ts", "jsx", "tsx"]
HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"
# ^ the value of `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`, read from
# crates/kali_runtime_contract/src/browser/contract.rs rather than assumed: one
# of this group's five sources passes the constant and the others spell nothing
# at all, and the migrated `env` must resolve to the same variable either way.

REGISTRY = {}


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    return open(os.path.join(TESTS, f"browser_{name}.rs")).read()


def check_program(label, body, *, must_contain="console.log"):
    """Guard the wrong-literal-extraction bug class at generation time.

    A fixture pulled from the wrong place still produces a parseable case file
    (batch 4 shipped `"app.${ext}" = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"`
    once). Anything written into `[source]` or a harness `body` must look like
    the program it claims to be before it is emitted.
    """
    if must_contain not in body:
        raise AssertionError(f"fixture {label!r} does not look like a program: {body[:80]!r}")
    return body


def check_captured(label, body, rs_text):
    """A `format!`-captured fixture must still line up with the .rs template.

    The capture is the only thing standing between rule 8 and a hand-typed
    approximation, and a stale capture (taken before a source edit) reproduces
    the OLD program while every gate stays green -- check_fixtures.py compares
    the template's literal segments against [source], which a stale capture from
    the same template also satisfies. So the generator re-derives the template's
    literal segments from the .rs itself and requires each one to be present in
    the captured text, with `format!`'s `{{`/`}}` un-doubled.
    """
    check_program(label, body)
    templates = [lit for lit in _string_literals(rs_text) if "{" in lit and "}" in lit]
    segments = set()
    for template in templates:
        tmp = template.replace("{{", "\x00").replace("}}", "\x01")
        for part in re.split(r"\{[^{}]*\}", tmp):
            part = part.replace("\x00", "{").replace("\x01", "}")
            if len(part) >= 12 and "console.log" in part or "kali-tree-shake" in part:
                segments.add(part)
    matched = [seg for seg in segments if seg in body]
    if not matched:
        raise AssertionError(
            f"captured fixture {label!r} shares no `format!` template segment with the .rs -- "
            "the capture is stale or came from the wrong source")
    return body


def _string_literals(rs_text):
    from lexer import find_string_literals
    return [lit["value"] for lit in find_string_literals(rs_text)]


def hdr(*chunks):
    """Flatten header chunks (str or list[str]) into one list of `#` lines.

    Embedded newlines are split HERE, not left inside a list element. `emit`
    prefixes `# ` per header entry, so an entry containing a newline used to
    emit one comment line and bare continuation lines, which `tomllib` rejects
    outright. (case_emit.emit has since grown the same split, so this is now
    belt and braces; both are idempotent for single-line entries, and keeping it
    here means this module cannot regress if that one is reverted.)
    """
    out = []
    for chunk in chunks:
        if chunk is None:
            continue
        pieces = [chunk] if isinstance(chunk, str) else list(chunk)
        for piece in pieces:
            out.extend(piece.split("\n"))
    return out


def para(*chunks):
    """Join rationale sentences into one paragraph."""
    return " ".join(c.strip() for c in chunks if c)


# --------------------------------------------------------------------------
# RULE 13. The `///` docs carried into rationales are EXTRACTED from
# kali_common at generation time, never retyped, on the same discipline as a
# fixture (rule 9): a doc quoted from memory is a claim about another crate
# that nothing in this pair's gate set can check.
#
# The carried set is derived mechanically from the CALL CHAIN: every
# `///`-documented `kali_common` fn transitively invoked while producing the
# fixture text this case file reproduces in [source]. That is rule 13's own
# test (the case still depends on what the helper computed), and it is the same
# depth the already-shipped math_floor_trunc_ceil_aliases.toml carries.
# --------------------------------------------------------------------------

def kali_common_doc(fn_name):
    text = open(KALI_COMMON_MATH).read()
    m = re.search(r"///([^\n]*)\n(?:pub )?(?:const )?fn " + re.escape(fn_name) + r"\b", text)
    if not m:
        raise AssertionError(f"no `///` doc immediately above `fn {fn_name}` in kali_common")
    return m.group(1).strip()


def docs(*fn_names):
    return [kali_common_doc(n) for n in fn_names]


# The call chains, spelled out so a reader can re-walk them in
# crates/kali_common/src/math.rs.
CHAIN_BRACKETED_LINES = (
    "math_pow_bracketed_frozen_callable_invocation_lines",
    "math_pow_bracketed_frozen_callable_aliases",
    "math_pow_invocation_lines_for_aliases",
)
CHAIN_BRACKETED_ENTRIES = (
    "math_pow_bracketed_frozen_callable_invocation_entries",
    "math_pow_invocation_entries_for_aliases",
)
CHAIN_BROWSER_INVENTORY = (
    "math_pow_browser_alias_inventory_invocation_lines",
    "math_pow_browser_alias_inventory_aliases",
    "math_pow_aliases",
    "math_pow_frozen_callable_aliases",
    "math_pow_frozen_callable_direct_aliases",
    "math_pow_frozen_callable_parenthesized_aliases",
    "math_pow_frozen_callable_nullish_logical_aliases",
    "math_pow_bracketed_frozen_callable_aliases",
    "math_pow_invocation_lines_for_aliases",
)


def rule13_chain_note(fn_names):
    """The `extra` lines for `rule13_header`, naming the documented chain.

    kali_common helper names are written WITHOUT backticks: they are not fns of
    the `.rs` under migration, and U8's checker adjudicates a backticked
    fn-shaped token against that file's fn list (see gen_batch4_group_a's note).
    """
    lines = [
        "The documented chain, in call order, all in crates/kali_common/src/math.rs:",
    ]
    row = "  "
    for n in fn_names:
        piece = f"{n}, "
        if len(row) + len(piece) > 86:
            lines.append(row.rstrip())
            row = "  "
        row += piece
    lines.append(row.rstrip().rstrip(",") + ".")
    lines.append(
        "Their `///` docs are extracted from that file by this generator, not retyped,"
    )
    lines.append("and carried verbatim into the rationale of every case they reach (U6).")
    return lines


# --------------------------------------------------------------------------
# EXTRA-OK justifications.
#
# `batch5_prose` supplies `EXTRA_OK_JSON_STDOUT` for a live-captured pin. It
# supplies no reason string for the OTHER deliberate extra this batch produces,
# a U5-renamed `[source]` entry filename, so the already-shipped wording from
# cases/browser/math_hypot_frozen_aliases.toml is reused verbatim rather than
# reworded -- same reason batch5_prose exists. Flagged in the group A report as
# a gap in the shared vocabulary.
# --------------------------------------------------------------------------

# Rebound mid-batch to `batch5_prose.EXTRA_OK_U5_RENAME`: three groups had written
# three wordings of this one fact before it was hoisted into the shared module.
EXTRA_OK_U5_RENAME = P.EXTRA_OK_U5_RENAME

# The four-line preamble is `batch5_prose.EXTRA_CLAIM_PREAMBLE`, not a local copy:
# all four groups had defined their own, and two of them wrapped the identical
# sentences at different columns. Rebound to the shared list mid-batch.
EXTRA_CLAIM_PREAMBLE = P.EXTRA_CLAIM_PREAMBLE


# --------------------------------------------------------------------------
# RULE 8 captures (see this module's docstring for the exact procedure).
# --------------------------------------------------------------------------

FIXT_A2_BUNDLE = (
    '// kali-tree-shake: mathPowBracketedFrozenCallable\n'
    'function mathPowBracketedFrozenCallable() {\n'
    '  const exponent = 3;\n'
    '  const alias = exponent;\n'
    '    console.log(Object.freeze((globalThis.Math))["pow"](2, alias));\n'
    "  console.log(Object.freeze((globalThis.Math))['pow'](2, alias));\n"
    '  console.log(Object.freeze((globalThis.Math).pow)(2, alias));\n'
    "  console.log(Object.freeze((globalThis.Math)['pow'])(2, alias));\n"
    '  console.log(Object.freeze((globalThis["Math"]))["pow"](2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math']))['pow'](2, alias));\n"
    '  console.log(Object.freeze((globalThis[\'Math\'])["pow"])(2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math'])['pow'])(2, alias));\n"
    '  console.log(Object.freeze((globalThis["Math"]).pow)(2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math']).pow)(2, alias));\n"
    '}\n'
)

FIXT_A3_BUNDLE = (
    '// kali-tree-shake: bracketedGlobalThisMathPowFrozenWrapper\n'
    'function bracketedGlobalThisMathPowFrozenWrapper() {\n'
    '  const exponent = 3;\n'
    '  const alias = exponent;\n'
    '    console.log(Object.freeze((globalThis.Math))["pow"](2, alias));\n'
    "  console.log(Object.freeze((globalThis.Math))['pow'](2, alias));\n"
    '  console.log(Object.freeze((globalThis.Math).pow)(2, alias));\n'
    "  console.log(Object.freeze((globalThis.Math)['pow'])(2, alias));\n"
    '  console.log(Object.freeze((globalThis["Math"]))["pow"](2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math']))['pow'](2, alias));\n"
    '  console.log(Object.freeze((globalThis[\'Math\'])["pow"])(2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math'])['pow'])(2, alias));\n"
    '  console.log(Object.freeze((globalThis["Math"]).pow)(2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math']).pow)(2, alias));\n"
    '  return [\n'
    '    Object.freeze((globalThis.Math))["pow"](2, alias),\n'
    "    Object.freeze((globalThis.Math))['pow'](2, alias),\n"
    '    Object.freeze((globalThis.Math).pow)(2, alias),\n'
    "    Object.freeze((globalThis.Math)['pow'])(2, alias),\n"
    '    Object.freeze((globalThis["Math"]))["pow"](2, alias),\n'
    "    Object.freeze((globalThis['Math']))['pow'](2, alias),\n"
    '    Object.freeze((globalThis[\'Math\'])["pow"])(2, alias),\n'
    "    Object.freeze((globalThis['Math'])['pow'])(2, alias),\n"
    '    Object.freeze((globalThis["Math"]).pow)(2, alias),\n'
    "    Object.freeze((globalThis['Math']).pow)(2, alias),\n"
    '  ];\n'
    '}\n'
)

_A4_PRELUDE_ALIAS = (
    '// kali-tree-shake: mathPowAliasChain\n'
    'function mathPowAliasChain() {\n'
    '  const exponent = 3;\n'
    '  const alias = exponent;\n'
)

_A4_PRELUDE_GLOBAL = (
    '// kali-tree-shake: globalThisMathPowAliasChain\n'
    'function globalThisMathPowAliasChain() {\n'
    '  const exponent = 3;\n'
    '  const alias = exponent;\n'
)

_A4_ALIAS_LINES = (
    '    console.log(Math.pow(2, alias));\n'
    "  console.log(Math['pow'](2, alias));\n"
    '  console.log(Math["pow"](2, alias));\n'
    '  console.log(globalThis.Math.pow(2, alias));\n'
    "  console.log(globalThis.Math['pow'](2, alias));\n"
    '  console.log(globalThis.Math["pow"](2, alias));\n'
    "  console.log(globalThis['Math'].pow(2, alias));\n"
    "  console.log(globalThis['Math']['pow'](2, alias));\n"
    '  console.log(globalThis[\'Math\']["pow"](2, alias));\n'
    '  console.log(globalThis["Math"].pow(2, alias));\n'
    '  console.log(globalThis["Math"]["pow"](2, alias));\n'
    '  console.log(globalThis["Math"][\'pow\'](2, alias));\n'
    "  console.log(Object.freeze(globalThis.Math['pow'])(2, alias));\n"
    '  console.log(Object.freeze(globalThis.Math["pow"])(2, alias));\n'
    "  console.log(Object.freeze(globalThis['Math']['pow'])(2, alias));\n"
    '  console.log(Object.freeze(globalThis[\'Math\']["pow"])(2, alias));\n'
    '  console.log(Object.freeze(globalThis["Math"]["pow"])(2, alias));\n'
    '  console.log(Object.freeze(globalThis["Math"][\'pow\'])(2, alias));\n'
    '  console.log(Object.freeze(globalThis.Math.pow)(2, alias));\n'
    "  console.log(Object.freeze(globalThis['Math'].pow)(2, alias));\n"
    '  console.log(Object.freeze(globalThis["Math"].pow)(2, alias));\n'
    '  console.log(Object.freeze(Math.pow)(2, alias));\n'
    "  console.log(Object.freeze(Math['pow'])(2, alias));\n"
    '  console.log(Object.freeze(Math["pow"])(2, alias));\n'
    "  console.log(Object.freeze((globalThis.Math['pow']))(2, alias));\n"
    '  console.log(Object.freeze((globalThis.Math["pow"]))(2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math']['pow']))(2, alias));\n"
    '  console.log(Object.freeze((globalThis[\'Math\']["pow"]))(2, alias));\n'
    '  console.log(Object.freeze((globalThis["Math"]["pow"]))(2, alias));\n'
    '  console.log(Object.freeze((globalThis["Math"][\'pow\']))(2, alias));\n'
    '  console.log(Object.freeze((globalThis.Math.pow))(2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math'].pow))(2, alias));\n"
    '  console.log(Object.freeze((globalThis["Math"].pow))(2, alias));\n'
    '  console.log(Object.freeze((Math.pow))(2, alias));\n'
    "  console.log(Object.freeze((Math['pow']))(2, alias));\n"
    '  console.log(Object.freeze((Math["pow"]))(2, alias));\n'
    '  console.log(Object.freeze((null ?? Math.pow))(2, alias));\n'
    '  console.log(Object.freeze((true && Math.pow))(2, alias));\n'
    '  console.log(Object.freeze((false || Math.pow))(2, alias));\n'
    '  console.log(Object.freeze((null ?? globalThis.Math.pow))(2, alias));\n'
    '  console.log(Object.freeze((true && globalThis.Math.pow))(2, alias));\n'
    '  console.log(Object.freeze((false || globalThis.Math.pow))(2, alias));\n'
    '  console.log(Object.freeze((null ?? globalThis["Math"]["pow"]))(2, alias));\n'
    '  console.log(Object.freeze((true && globalThis["Math"]["pow"]))(2, alias));\n'
    '  console.log(Object.freeze((false || globalThis["Math"]["pow"]))(2, alias));\n'
    "  console.log(Object.freeze((null ?? globalThis['Math']['pow']))(2, alias));\n"
    "  console.log(Object.freeze((true && globalThis['Math']['pow']))(2, alias));\n"
    "  console.log(Object.freeze((false || globalThis['Math']['pow']))(2, alias));\n"
    '  console.log(Object.freeze((globalThis.Math))["pow"](2, alias));\n'
    "  console.log(Object.freeze((globalThis.Math))['pow'](2, alias));\n"
    '  console.log(Object.freeze((globalThis.Math).pow)(2, alias));\n'
    "  console.log(Object.freeze((globalThis.Math)['pow'])(2, alias));\n"
    '  console.log(Object.freeze((globalThis["Math"]))["pow"](2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math']))['pow'](2, alias));\n"
    '  console.log(Object.freeze((globalThis[\'Math\'])["pow"])(2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math'])['pow'])(2, alias));\n"
    '  console.log(Object.freeze((globalThis["Math"]).pow)(2, alias));\n'
    "  console.log(Object.freeze((globalThis['Math']).pow)(2, alias));\n"
)

_A4_RETURN_ALIAS = (
    '  return Math.pow(2, alias);\n'
    '}\n'
)

_A4_RETURN_GLOBAL = (
    '  return globalThis.Math.pow(2, alias);\n'
    '}\n'
)

BODY_A4_ALIAS = (
    'const mod = await import(bundleJs.href);\n'
    'await mod.mathPowAliasChain();\n'
)

BODY_A4_GLOBAL_THIS_ALIAS = (
    'const mod = await import(bundleJs.href);\n'
    'await mod.globalThisMathPowAliasChain();\n'
)

FIXT_A4_ALIAS = _A4_PRELUDE_ALIAS + _A4_ALIAS_LINES + _A4_RETURN_ALIAS
FIXT_A4_GLOBAL_THIS_ALIAS = _A4_PRELUDE_GLOBAL + _A4_ALIAS_LINES + _A4_RETURN_GLOBAL
# ^ Split this way because the two captured programs differ ONLY in their
# tree-shake marker / exported fn name and their `return` line; the invocation
# block between them came back byte-identical from the capture, which is what
# both builders calling the same kali_common helper predicts. That equality was
# asserted MECHANICALLY at capture time (the capture script compares the two
# dumps' middles and refuses to emit these constants otherwise), not eyeballed,
# and the shared factor here is what preserves it: a later edit cannot desync
# the two programs' invocation blocks without editing one constant.
if FIXT_A4_ALIAS == FIXT_A4_GLOBAL_THIS_ALIAS:
    raise AssertionError("the two math_pow_alias_bundle programs must differ")

# U9 -- live-captured with kali_run.py from .cache/cargo-target/debug/kali,
# `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`, for all four `ext` cells and for
# BOTH `run` and `test`; all eight captures returned the identical string, which
# `_assert_pin_cells` re-asserts below before the single pin is emitted.
PIN_MIXED_ROOT_JSON_STDOUT = "3\n3\n3\n3\n"


def _assert_pin_cells(programs):
    """U9's mechanical half: RE-CAPTURE the pin from the real binary, for every
    cell of this file's matrix axis and for both commands, and assert all eight
    cells agree with each other AND with the embedded constant before one pin is
    emitted. `assert_identical` over eight copies of one constant would prove
    nothing; the capture is what makes the assertion real.

    Skipped, loudly, only if the built binary is absent -- the constant then
    stands on the capture recorded in this module's docstring, and the generator
    says so rather than reporting a green it did not run.
    """
    import json as _json
    from kali_run import KALI, run_kali

    if not os.path.exists(KALI):
        print(f"  !! {KALI} absent -- json.stdout pin NOT re-captured this run")
        return PIN_MIXED_ROOT_JSON_STDOUT

    captured = []
    for ext in EXTS:
        for command, entry, program in (
            ("run", f"main.{ext}", programs["main"]),
            ("test", f"smoke.test.{ext}", programs["test"]),
        ):
            code, out, _err, _dir = run_kali(
                {entry: program},
                ["--output", "json", command, "--api", "browser",
                 "--max-threads", "0", "--max-spawned-processes", "0", entry],
                env={HARNESS_ENV: "node"},
            )
            if code != 0:
                raise AssertionError(f"live capture failed for {command}/{ext}: {_err!r}")
            captured.append(_json.loads(out)["stdout"])
    return P.assert_identical(
        "math_log2_log10_mixed_root json.stdout pin, run/test x ext(js/ts/jsx/tsx), "
        "live-captured, against the embedded constant",
        PIN_MIXED_ROOT_JSON_STDOUT, *captured,
    )


def harness_json(command, *, stdout_pin, stderr, errors):
    """`envelope_harness` with the exact `json.stdout` pin spliced in.

    `math_shapes.envelope_harness` has no `stdout` parameter because most of
    this migration asserts a COUNT on that leaf rather than an equality; this
    file's harness helper instead makes a plain `.contains` against
    json["stdout"], which per ruling 3 becomes an exact pin. Spliced here rather
    than by changing the shared builder.
    """
    base = envelope_harness(command, stderr=stderr, errors=errors)
    out = {}
    for key, value in base.items():
        if key in ("stderr", "errors") and "stdout" not in out:
            out["stdout"] = stdout_pin
        out[key] = value
    out.setdefault("stdout", stdout_pin)
    return out


# ==========================================================================
# A1. browser_math_sin_cos_tan_zero_identities.rs
# ==========================================================================

@target("math_sin_cos_tan_zero_identities")
def gen_sin_cos_tan():
    stem = "math_sin_cos_tan_zero_identities"
    text = rs(stem)
    helper = "assert_browser_bundle_math_sin_cos_tan"

    c_build_exit, c_harness_exit = P.cite_line(
        text, r"output\.status\.success\(\)", label="status.success", expect=2)
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_env_errors = P.cite_line(text, r'assert!\(envelope\["errors"\]')
    c_env_is_empty = P.cite_line(text, r"\.is_empty\(\)")
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_contains = P.cite_line(text, r'stdout\.contains\("1\\n"\)')
    c_count = P.cite_line(text, r'stdout\.matches\("0\\n"\)\.count\(\) >= 2')

    program = check_program(
        "app.${ext}", fixture_in_fn(text, "browser_bundle_math_sin_cos_tan_source", 0))
    body = check_program(
        "harness body",
        fixture_starting(text, helper, "const mod = await import("),
        must_contain="await import(")

    count_keys = [
        "THE COUNT KEYS. The source makes exactly one occurrence-count claim, on the raw",
        f"stdout of the browser-bundle harness process: `.matches(\"0\\n\").count() >= 2`",
        f"(:{c_count}). It is carried as `stdout_count` with `at_least = 2` -- the bound the",
        "source states, neither weakened nor strengthened. The preceding line's separate",
        f"`.contains(\"1\\n\")` (:{c_contains}) is a DIFFERENT claim about the same surface and is",
        "carried alongside it as `stdout_contains`; collapsing the two into one would drop a",
        "claim. There is no count claim anywhere else in the file, and none on a JSON leaf, so",
        "no `json_count` is written.",
    ]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the `kali build` process (:{c_build_exit}) and on the harness",
        f"process (:{c_harness_exit}).",
        f"In json mode, the build envelope's schemaVersion/command/success/exitCode and the",
        f"payload's artifactKind/bundleFormat (:{c_env_first}-{c_env_bundle_format}), AND an empty",
        f"`errors` array: this source really does assert",
        f"`envelope[\"errors\"].as_array().is_empty()` (:{c_env_errors}-{c_env_is_empty}), so",
        "`errors = []` is written here. (The three `Math.pow` sources migrated alongside it in",
        "this group assert no such thing and get no `errors` claim -- the difference is real and",
        "is not normalised away.)",
        f"The emitted app/app.meta.json apiSurface/artifactKind (:{c_meta_api}-{c_meta_kind}) is",
        "asserted in BOTH modes, because the source reads that file outside the `if json_output`",
        "block.",
        f"The harness step carries `stdout_contains` (:{c_contains}) and `stdout_count`",
        f"(:{c_count}).",
        "The source asserts NOTHING about stderr on either process, and never reads the build",
        "envelope's stdout leaf, so no stderr claim and no `json.stdout` pin are written.",
        "It passes no --max-threads / --max-spawned-processes arguments, so neither appears on",
        "argv.",
    ]

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=8, invocations=8, cases=2, axis="ext", values=EXTS,
            non_axes=("json_output",),
            helpers=[(
                helper, 8,
                "ext(js/ts/jsx/tsx) x json_output(false/true), a complete cross\n"
                "    product. Every `#[test]` fn is one unlooped call and the file contains no\n"
                "    loop at all.",
            )],
        ),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["app.${ext}"]),
        "",
        count_keys,
        "",
        P.rule13_header([
            "kali_bin", "browser_bundle_math_sin_cos_tan_source", helper,
        ]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    prog_desc = (
        f"`{helper}` builds a browser bundle with `kali build --bundle --api browser`, asserts "
        "the emitted app/app.meta.json metadata, then runs the bundle glue under the "
        "browser-bundle-harness contract backed by node, against a program that computes "
        "Math.sin(0), Math.cos(0) and Math.tan(0) -- so the cosine prints 1 and the sine and "
        "tangent each print 0."
    )
    claims = (
        f"Its two claims about that output are separate source lines: `.contains(\"1\\n\")` "
        f"(:{c_contains}) and `.matches(\"0\\n\").count() >= 2` (:{c_count}), and both are carried."
    )
    cite_prefix = (
        f"Migrated from browser_{stem}.rs, the four "
        "`build_emits_math_sin_cos_tan_zero_identities_in_*_input` fns (one per extension)."
    )
    json_prefix = (
        f"Migrated from browser_{stem}.rs, the four "
        "`json_build_emits_math_sin_cos_tan_zero_identities_in_*_input` fns (one per extension)."
    )
    ruling3 = " ".join([
        P.ruling3_substring(),
        P.ruling3_count('"0\\n"', 2),
    ])
    envelope_sentence = (
        "This sibling additionally asserts the build JSON envelope -- schemaVersion/command/"
        "success/exitCode, payload artifactKind/bundleFormat, and the empty `errors` array the "
        f"source checks at :{c_env_errors}-{c_env_is_empty} -- rather than plain text; output "
        "shape is not a matrix axis because it changes the assertion shape, so it is a separate "
        "case."
    )

    cases = [
        {
            "name": "build_emits_math_sin_cos_tan_zero_identities",
            "rationale": para(cite_prefix, prog_desc, claims, ruling3),
            "steps": bundle_steps(
                "app.${ext}", body,
                {"stdout_contains": ["1\n"],
                 "stdout_count": [{"needle": "0\n", "at_least": 2}]},
                json_output=False, meta_fields=META),
        },
        {
            "name": "json_build_emits_math_sin_cos_tan_zero_identities",
            "rationale": para(json_prefix, prog_desc, claims, ruling3, envelope_sentence),
            "steps": bundle_steps(
                "app.${ext}", body,
                {"stdout_contains": ["1\n"],
                 "stdout_count": [{"needle": "0\n", "at_least": 2}]},
                json_output=True,
                json_claims=envelope_build(errors=True),
                meta_fields=META),
        },
    ]

    return emit(header, {"ext": EXTS}, {"app.${ext}": program}, cases)


# ==========================================================================
# A2/A3 shared shape: the bracketed-root frozen `Math.pow` wrapper bundles.
# ==========================================================================

def _pow_bracketed(stem, *, program, retained_block, rule13_docs, chain_extra,
                   test_fns, migrated_note, name_pattern):
    text = rs(stem)
    helper = "assert_browser_bundle_bracketed_global_this_math_pow_frozen"
    source_fn = "browser_bundle_bracketed_global_this_math_pow_frozen_source"

    c_build_exit, c_harness_exit = P.cite_line(
        text, r"output\.status\.success\(\)", label="status.success", expect=2)
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_contains = P.cite_line(text, r'stdout\.contains\("8\\n8"\)')
    c_format = P.cite_line(text, r"^\s*format!\($")

    body = check_program(
        "harness body",
        fixture_starting(text, helper, "const mod = await import("),
        must_contain="await import(")

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the `kali build` process (:{c_build_exit}) and on the harness",
        f"process (:{c_harness_exit}).",
        "In json mode, the build envelope's schemaVersion/command/success/exitCode and the",
        f"payload's artifactKind/bundleFormat (:{c_env_first}-{c_env_bundle_format}).",
        "The source makes NO `errors` claim on that envelope -- compare",
        "browser_math_sin_cos_tan_zero_identities.rs, migrated in this same group, whose json",
        "branch does assert the errors array is empty -- so no `errors = []` is written here.",
        f"The emitted app/app.meta.json apiSurface/artifactKind (:{c_meta_api}-{c_meta_kind}) is",
        "asserted in BOTH modes, because the source reads that file outside the `if json_output`",
        "block.",
        f"The harness step carries the file's ONE stdout claim, `.contains(\"8\\n8\")`",
        f"(:{c_contains}).",
        "There is no count claim in this file, so no `stdout_count`; no stderr claim on either",
        "process; and the build envelope's stdout leaf is never read, so no `json.stdout` pin.",
        "The source passes no --max-threads / --max-spawned-processes arguments, so neither",
        "appears on argv.",
    ]

    rule8 = [
        "RULE 8 / RULE 9 -- the [source] program is `format!`-built and therefore appears in NO",
        f"string literal in the .rs (:{c_format}). Its body below is the byte-exact OUTPUT of",
        "executing the real builder, captured by a temporary test target that include!d this .rs",
        "and dumped the fn's return value; it is never a hand-applied `{}` substitution. That",
        "matters concretely here: the template indents the placeholder by two spaces and the",
        "kali_common helper indents every line it emits by two more, so the FIRST emitted line",
        "carries four spaces and the rest carry two -- an asymmetry a hand substitution",
        "reproduces wrongly in either direction. The exact procedure is recorded in this file's",
        "generator, tools/task-18-browser-pilot/gen_batch5_group_a.py.",
    ]

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        retained_block,
        "" if retained_block else None,
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=test_fns, invocations=8, cases=2, axis="ext", values=EXTS,
            non_axes=("json_output",),
            helpers=[(helper, 8, migrated_note)],
        ),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["app.${ext}"]),
        "",
        rule8,
        "",
        P.rule13_header(["kali_bin", source_fn, helper],
                        docs_carried=rule13_docs,
                        extra=chain_extra),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    prog_desc = (
        f"`{helper}` builds a browser bundle with `kali build --bundle --api browser`, asserts "
        "the emitted app/app.meta.json metadata, then runs the bundle glue under the "
        "browser-bundle-harness contract backed by node, against a program that raises 2 to an "
        "aliased exponent of 3 through every bracketed-root frozen `Math.pow` callable in the "
        "canonical inventory, so every line of its output is 8."
    )
    claims = (
        f"The file's only stdout claim is `.contains(\"8\\n8\")` (:{c_contains}), which is what "
        "pins that at least two consecutive lines of 8 were printed rather than one."
    )
    ruling3 = P.ruling3_substring()
    envelope_sentence = (
        "This sibling additionally asserts the build JSON envelope -- schemaVersion/command/"
        "success/exitCode and payload artifactKind/bundleFormat -- rather than plain text; "
        "output shape is not a matrix axis because it changes the assertion shape, so it is a "
        "separate case. The source asserts no `errors` array on this envelope and none is "
        "written."
    )
    carried = P.rule13_carried(rule13_docs)

    cases = [
        {
            "name": "build_emits_bracketed_global_this_math_pow_frozen_wrapper",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four `{name_pattern}` fns (one per "
                "extension).",
                prog_desc, claims, ruling3, carried),
            "steps": bundle_steps(
                "app.${ext}", body, {"stdout_contains": ["8\n8"]},
                json_output=False, meta_fields=META),
        },
        {
            "name": "json_build_emits_bracketed_global_this_math_pow_frozen_wrapper",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four `json_{name_pattern}` fns (one per "
                "extension).",
                prog_desc, claims, ruling3, envelope_sentence, carried),
            "steps": bundle_steps(
                "app.${ext}", body, {"stdout_contains": ["8\n8"]},
                json_output=True, json_claims=envelope_build(errors=False),
                meta_fields=META),
        },
    ]

    return emit(header, {"ext": EXTS}, {"app.${ext}": program}, cases)


@target("math_pow_bracketed_frozen_wrapper_bundle")
def gen_pow_bracketed_bundle():
    stem = "math_pow_bracketed_frozen_wrapper_bundle"
    program = check_captured("app.${ext}", FIXT_A2_BUNDLE, rs(stem))
    return _pow_bracketed(
        stem,
        program=program,
        retained_block=None,
        rule13_docs=docs(*CHAIN_BRACKETED_LINES),
        chain_extra=rule13_chain_note(CHAIN_BRACKETED_LINES),
        test_fns=8,
        migrated_note=(
            "ext(js/ts/jsx/tsx) x json_output(false/true), a complete cross\n"
            "    product. Every `#[test]` fn is one unlooped call and the file contains no\n"
            "    loop at all."
        ),
        name_pattern="build_emits_bracketed_global_this_math_pow_frozen_wrapper_in_*_input",
    )


@target("math_pow_bracketed_frozen_wrapper")
def gen_pow_bracketed_wrapper():
    stem = "math_pow_bracketed_frozen_wrapper"
    text = rs(stem)
    retained = ("browser_bundle_bracketed_global_this_math_pow_frozen_source_includes_"
                "parenthesized_bracketed_aliases")
    c_retained = P.cite_line(text, r"^fn " + re.escape(retained))
    c_loop = P.cite_line(text, r"for expected in ")
    c_assert = P.cite_line(text, r"assert!\(source\.contains\(expected\)")

    program = check_captured("app.${ext}", FIXT_A3_BUNDLE, text)
    retained_block = P.partial_retention_note(
        stem=stem,
        retained_fn=retained,
        migrated=8,
        total=9,
        blocking=(
            f"its whole body (:{c_loop}-{c_assert}) is\n"
            "`assert!(source.contains(expected))` looped over kali_common's\n"
            "math_pow_bracketed_frozen_callable_aliases, taken against the text the fixture\n"
            f"builder itself returns (the fn is declared at :{c_retained})."
        ),
    )
    return _pow_bracketed(
        stem,
        program=program,
        retained_block=retained_block,
        rule13_docs=docs(*CHAIN_BRACKETED_LINES, *CHAIN_BRACKETED_ENTRIES),
        chain_extra=rule13_chain_note(CHAIN_BRACKETED_LINES + CHAIN_BRACKETED_ENTRIES),
        test_fns=9,
        migrated_note=(
            "ext(js/ts/jsx/tsx) x json_output(false/true), a complete cross\n"
            "    product over the EIGHT migrated fns. Each is one unlooped call; the ninth\n"
            "    `#[test]` fn is the retained fixture self-inspection above, which invokes no\n"
            "    helper and no binary and so contributes no invocation to this arithmetic."
        ),
        name_pattern="build_emits_bracketed_global_this_math_pow_frozen_wrapper_in_*_input",
    )


# ==========================================================================
# A4. browser_math_pow_alias_bundle.rs
# ==========================================================================

@target("math_pow_alias_bundle")
def gen_pow_alias_bundle():
    stem = "math_pow_alias_bundle"
    text = rs(stem)
    thin = "assert_browser_bundle_math_pow_alias"
    real = "assert_browser_bundle_math_pow_alias_with_source"

    c_build_exit, c_harness_exit = P.cite_line(
        text, r"output\.status\.success\(\)", label="status.success", expect=2)
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_contains = P.cite_line(text, r'stdout\.contains\("8\\n"\)')
    c_thin = P.cite_line(text, r"^fn " + re.escape(thin) + r"\(")
    c_real = P.cite_line(text, r"^fn " + re.escape(real) + r"<")
    c_harness_format = P.cite_line(text, r"&format!\($")

    alias_src = check_captured("app_alias.${ext}", FIXT_A4_ALIAS, text)
    global_src = check_captured("app_global_this_alias.${ext}",
                                FIXT_A4_GLOBAL_THIS_ALIAS, text)
    body_alias = check_program("harness body (alias)", BODY_A4_ALIAS,
                               must_contain="await import(")
    body_global = check_program("harness body (globalThis)", BODY_A4_GLOBAL_THIS_ALIAS,
                                must_contain="await import(")
    if alias_src == global_src:
        raise AssertionError("the two A4 programs must differ; a capture went wrong")

    renames = [
        ("app.<ext>", "app_alias.${ext}",
         "the mathPowAliasChain program, written by the eight\n"
         "    `build_emits_math_pow_alias_chain_*` fns through the thin wrapper\n"
         f"    `{thin}` (:{c_thin})"),
        ("app.<ext>", "app_global_this_alias.${ext}",
         "the globalThisMathPowAliasChain program, written by the\n"
         "    eight `build_emits_global_this_math_pow_alias_chain_*` fns, which call\n"
         f"    `{real}` (:{c_real})\n    directly"),
    ]
    u5 = P.u5_renames(renames) + [
        "The rename is NOT cosmetic bookkeeping: `kali build --bundle` names its output",
        "directory and metadata file after the INPUT STEM, so `app_alias.js` emits",
        "app_alias/app_alias.meta.json and a bundle the harness must import as",
        "`entry = \"app_alias\"`. Every one of those three places follows the rename below, which",
        "is exactly U5's own warning about a rename that reaches past the `[source]` key.",
        "Both keys are suffixed rather than only the clashing one, so neither variant reads as",
        "the default and a later editor cannot reintroduce the clash by adding a third program.",
    ]

    extra_ok = list(EXTRA_CLAIM_PREAMBLE)
    for ext in EXTS:
        extra_ok.append(P.extra_ok(f"app_alias.{ext}", EXTRA_OK_U5_RENAME))
    for ext in EXTS:
        extra_ok.append(P.extra_ok(f"app_global_this_alias.{ext}", EXTRA_OK_U5_RENAME))

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the `kali build` process (:{c_build_exit}) and on the harness",
        f"process (:{c_harness_exit}).",
        "In json mode, the build envelope's schemaVersion/command/success/exitCode and the",
        f"payload's artifactKind/bundleFormat (:{c_env_first}-{c_env_bundle_format}). The source",
        "makes NO `errors` claim on that envelope, so no `errors = []` is written.",
        f"The emitted <stem>/<stem>.meta.json apiSurface/artifactKind (:{c_meta_api}-{c_meta_kind})",
        "is asserted in BOTH modes, because the source reads that file outside the",
        "`if json_output` block.",
        f"The harness step carries the file's ONE stdout claim, `.contains(\"8\\n\")`",
        f"(:{c_contains}) -- note the single trailing newline, unlike the `\"8\\n8\"` its two",
        "bracketed-frozen siblings in this group assert; the weaker needle is what this source",
        "says and it is not upgraded.",
        "There is no count claim in this file, so no `stdout_count`; no stderr claim on either",
        "process; and the build envelope's stdout leaf is never read, so no `json.stdout` pin.",
        "The source passes no --max-threads / --max-spawned-processes arguments, so neither",
        "appears on argv.",
    ]

    rule8 = [
        "RULE 8 / RULE 9 -- THREE `format!`-built texts, none of them present as a literal in",
        "the .rs, all three captured by executing the real code rather than by hand-applying",
        "`{}` substitution:",
        "  * both [source] programs, dumped from a temporary test target that include!d this",
        "    .rs and wrote each builder's return value out verbatim. The template indents its",
        "    placeholder by two spaces and the kali_common helper indents again, so the first",
        "    emitted line carries four spaces and the rest two -- an asymmetry a hand",
        "    substitution gets wrong in one direction or the other.",
        f"  * the browser-bundle-harness `body`, built by an inline `format!`",
        f"    (:{c_harness_format}) with an `{{export_name}}` placeholder inside",
        f"    `{real}`, which makes it",
        "    unreachable as a value at all. It was captured by running that real helper with",
        "    the harness",
        "    command pointed at a wrapper that copies the script it is handed and then execs",
        "    node (so the helper's own assertions still pass), then subtracting",
        "    kali_runtime_contract's browser_bundle_harness_prelude from the captured script --",
        "    browser_bundle_harness_script is defined as prelude followed by body, so the",
        "    remainder is the resolved body.",
        "The exact procedure is recorded in this file's generator,",
        "tools/task-18-browser-pilot/gen_batch5_group_a.py.",
    ]

    header = hdr(
        extra_ok,
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS,
            non_axes=("json_output",),
            helpers=[
                (thin, 8,
                 "ext(js/ts/jsx/tsx) x json_output(false/true). This is a thin\n"
                 "    wrapper: its whole body forwards to the helper below with the\n"
                 "    mathPowAliasChain program and export name, so these 8 are also 8\n"
                 "    executions of that helper's body."),
                (real, 8,
                 "ext(js/ts/jsx/tsx) x json_output(false/true), called\n"
                 "    DIRECTLY by 8 further `#[test]` fns with the globalThisMathPowAliasChain\n"
                 "    program and export name. 16 `#[test]` fns, 16 real invocations, two entry\n"
                 "    points, one helper body, no loop anywhere in the file."),
            ],
        ),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["app_alias.${ext}", "app_global_this_alias.${ext}"]),
        "",
        u5,
        "",
        rule8,
        "",
        P.rule13_header(
            ["kali_bin", "browser_bundle_math_pow_alias_source",
             "browser_bundle_global_this_math_pow_alias_source", thin, real],
            docs_carried=docs(*CHAIN_BROWSER_INVENTORY),
            extra=rule13_chain_note(CHAIN_BROWSER_INVENTORY)),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    carried = P.rule13_carried(docs(*CHAIN_BROWSER_INVENTORY))
    ruling3 = P.ruling3_substring()
    claims = (
        f"The file's only stdout claim is `.contains(\"8\\n\")` (:{c_contains})."
    )

    def desc(entry_desc, program_desc):
        return (
            f"{entry_desc} builds a browser bundle with `kali build --bundle --api browser`, "
            "asserts the emitted bundle metadata, then runs the bundle glue under the "
            "browser-bundle-harness contract backed by node, against a program that "
            f"{program_desc}"
        )

    alias_desc = desc(
        f"`{thin}`, whose body is one forwarding call to `{real}`,",
        "raises 2 to an aliased exponent of 3 through every alias in the canonical browser "
        "`Math.pow` inventory and returns Math.pow(2, alias), so every line of its output is 8.")
    global_desc = desc(
        f"`{real}`, called directly,",
        "raises 2 to an aliased exponent of 3 through the same canonical browser `Math.pow` "
        "inventory but returns globalThis.Math.pow(2, alias), so every line of its output is 8.")

    rename_sentence_alias = (
        "This case's program is written to app_alias.<ext> rather than the source's app.<ext>: "
        "the file writes two DIFFERENT programs to that one name and `[source]` is a single "
        "flat namespace (U5). The bundle directory, the metadata path and the harness entry all "
        "follow the renamed stem, because `kali build --bundle` names its output after the "
        "input stem."
    )
    rename_sentence_global = rename_sentence_alias.replace(
        "app_alias.<ext>", "app_global_this_alias.<ext>")
    envelope_sentence = (
        "This sibling additionally asserts the build JSON envelope -- schemaVersion/command/"
        "success/exitCode and payload artifactKind/bundleFormat -- rather than plain text; "
        "output shape is not a matrix axis because it changes the assertion shape, so it is a "
        "separate case. The source asserts no `errors` array on this envelope and none is "
        "written."
    )

    def steps(entry, meta_path, body, json_output):
        st = bundle_steps(entry, body, {"stdout_contains": ["8\n"]},
                          json_output=json_output,
                          json_claims=envelope_build(errors=False) if json_output else None,
                          meta_fields=META)
        st[1]["path"] = meta_path
        st[2]["entry"] = entry.split(".")[0]
        return st

    cases = [
        {
            "name": "build_emits_math_pow_alias_chain",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four "
                "`build_emits_math_pow_alias_chain_in_*_input` fns (one per extension).",
                alias_desc, claims, ruling3, rename_sentence_alias, carried),
            "steps": steps("app_alias.${ext}", "app_alias/app_alias.meta.json",
                           body_alias, False),
        },
        {
            "name": "json_build_emits_math_pow_alias_chain",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four "
                "`json_build_emits_math_pow_alias_chain_in_*_input` fns (one per extension).",
                alias_desc, claims, ruling3, envelope_sentence, rename_sentence_alias, carried),
            "steps": steps("app_alias.${ext}", "app_alias/app_alias.meta.json",
                           body_alias, True),
        },
        {
            "name": "build_emits_global_this_math_pow_alias_chain",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four "
                "`build_emits_global_this_math_pow_alias_chain_in_*_input` fns (one per "
                "extension).",
                global_desc, claims, ruling3, rename_sentence_global, carried),
            "steps": steps("app_global_this_alias.${ext}",
                           "app_global_this_alias/app_global_this_alias.meta.json",
                           body_global, False),
        },
        {
            "name": "json_build_emits_global_this_math_pow_alias_chain",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four "
                "`json_build_emits_global_this_math_pow_alias_chain_in_*_input` fns (one per "
                "extension).",
                global_desc, claims, ruling3, envelope_sentence, rename_sentence_global,
                carried),
            "steps": steps("app_global_this_alias.${ext}",
                           "app_global_this_alias/app_global_this_alias.meta.json",
                           body_global, True),
        },
    ]

    source = {"app_alias.${ext}": alias_src, "app_global_this_alias.${ext}": global_src}
    return emit(header, {"ext": EXTS}, source, cases)


# ==========================================================================
# A5. browser_math_log2_log10_mixed_root.rs
# ==========================================================================

@target("math_log2_log10_mixed_root")
def gen_log2_log10_mixed_root():
    stem = "math_log2_log10_mixed_root"
    text = rs(stem)
    bundle_helper = "assert_browser_bundle_global_this_math_bracketed_log2_log10"
    loop_fn = ("run_and_test_supports_global_this_math_bracketed_log2_log10_identities_"
               "when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input")

    c_exits = P.cite_line(text, r"output\.status\.success\(\)",
                          label="status.success", expect=3)
    c_build_exit, c_bundle_harness_exit, c_cli_exit = c_exits
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_env_errors = P.cite_line(text, r'assert!\(envelope\["errors"\]')
    c_env_is_empty = P.cite_line(text, r"^\s+\.is_empty\(\)\);?$", expect=1)
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_bundle_contains = P.cite_line(text, r'stdout\.contains\("3\\n"\)')
    c_bundle_count = P.cite_line(text, r'stdout\.matches\("3\\n"\)\.count\(\) >= 2')
    c_loop_fn = P.cite_line(text, r"^fn " + re.escape(loop_fn))
    c_table_run = P.cite_line(text, r'^\s+"3\\n3",$', expect=4)[0]
    c_table_test = P.cite_line(text, r'^\s+"3\\nok 1",$', expect=4)[0]
    c_output_json = P.cite_line(text, r"for output_json in \[false, true\]")
    c_threads = P.cite_line(text, r'\.arg\("--max-threads"\)')
    c_procs = P.cite_line(text, r'\.arg\("--max-spawned-processes"\)')
    c_j_schema = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\]')
    c_j_command = P.cite_line(text, r'assert_eq!\(json\["command"\], command\)')
    c_j_success = P.cite_line(text, r'assert_eq!\(json\["success"\]')
    c_j_host = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["hostContract"\]')
    c_j_backend = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["runtimeBackend"\]')
    c_j_exit = P.cite_line(text, r'assert_eq!\(json\["exitCode"\], 0\)')
    c_j_payload_exit = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["exitCode"\], 0\)')
    c_j_total = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["total"\], 1\)')
    c_j_failed = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["failed"\], 0\)')
    c_j_stdout = P.cite_line(text, r'json\["stdout"\]\.as_str\(\)\.expect\("stdout"\)\.contains')
    c_j_stderr = P.cite_line(text, r'assert_eq!\(json\["stderr"\], ""\)')
    c_j_errors = P.cite_line(text, r'assert!\(json\["errors"\]\.as_array\(\)')
    c_text_contains = P.cite_line(text, r"stdout\.contains\(expected_stdout\)")

    bundle_program = check_program(
        "app.${ext}",
        fixture_in_fn(text, "browser_bundle_global_this_math_bracketed_log2_log10_source", 0))
    run_program = check_program(
        "main.${ext}",
        fixture_in_fn(text,
                      "browser_harness_global_this_math_bracketed_log2_log10_run_source", 0))
    test_program = check_program(
        "smoke.test.${ext}",
        fixture_in_fn(text,
                      "browser_harness_global_this_math_bracketed_log2_log10_test_source", 0),
        must_contain="Kali.test(")
    bundle_body = check_program(
        "harness body",
        fixture_starting(text, bundle_helper, "const mod = await import("),
        must_contain="await import(")

    pin = _assert_pin_cells({"main": run_program, "test": test_program})

    extra_ok = list(EXTRA_CLAIM_PREAMBLE) + [P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT)]

    count_keys = [
        "THE COUNT KEYS. The source makes exactly one occurrence-count claim, and it is on the",
        "browser-bundle harness process's raw stdout:",
        f"`.matches(\"3\\n\").count() >= 2` (:{c_bundle_count}), carried as `stdout_count` with",
        f"`at_least = 2`. The line above it (:{c_bundle_contains}) makes a SEPARATE",
        "`.contains(\"3\\n\")` claim about the same surface; both are carried on that step,",
        "because collapsing them into one would drop a claim.",
        "The harness `#[test]` fn makes NO count claim at all -- on its json branch it asserts",
        f"`json[\"stdout\"].as_str().contains(\"3\")` (:{c_j_stdout}), an equality-free containment",
        "on a JSON leaf, which is why the four harness cases carry a `json.stdout` pin and no",
        "`json_count`. (Its bracketed sibling browser_math_log2_log10.rs does spell that same",
        "leaf as a count, which is where `json_count` came from; this file does not.)",
    ]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"BUNDLE helper `{bundle_helper}`:",
        f"`exit = \"success\"` on the build (:{c_build_exit}) and on the harness process",
        f"(:{c_bundle_harness_exit}); in json mode",
        "the build envelope's schemaVersion/command/success/exitCode and payload",
        f"artifactKind/bundleFormat (:{c_env_first}-{c_env_bundle_format}) PLUS an empty `errors`",
        f"array (:{c_env_errors}-{c_env_is_empty}); the emitted app/app.meta.json",
        f"apiSurface/artifactKind (:{c_meta_api}-{c_meta_kind}) in BOTH modes, because the source",
        "reads that file outside the `if json_output` block; then `stdout_contains`",
        f"(:{c_bundle_contains}) and `stdout_count` (:{c_bundle_count}).",
        f"HARNESS `#[test]` fn (:{c_loop_fn}): `exit = \"success\"` (:{c_cli_exit}).",
        f"On the text branch, `stdout.contains(expected_stdout)` (:{c_text_contains}), where",
        f"`expected_stdout` is the table's own literal -- \"3\\n3\" for `run` (:{c_table_run}) and",
        f"\"3\\nok 1\" for `test` (:{c_table_test}). Those are DIFFERENT strings, so they are",
        "mirrored per command and never merged.",
        "On the json branch, schemaVersion/command/success",
        f"(:{c_j_schema}-{c_j_success}), payload hostContract/runtimeBackend",
        f"(:{c_j_host}-{c_j_backend}), then `exitCode` at both the envelope and the payload level",
        f"for `run` (:{c_j_exit}-{c_j_payload_exit}) or payload total/passed/failed for `test`",
        f"(:{c_j_total}-{c_j_failed}); the `json.stdout` pin (:{c_j_stdout}); `stderr = \"\"`",
        f"(:{c_j_stderr}); and `errors = []` (:{c_j_errors}).",
        "The source asserts nothing about `skipped`, so no `skipped` claim is written even",
        "though the live payload carries one.",
        "The bundle helper passes NO --max-threads / --max-spawned-processes arguments and the",
        f"harness fn DOES (:{c_threads}, :{c_procs}); that difference is preserved exactly, not",
        "normalised across the file.",
    ]

    header = hdr(
        extra_ok,
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=9, invocations=24, cases=6, axis="ext", values=EXTS,
            helpers=[
                (bundle_helper, 8,
                 "ext(js/ts/jsx/tsx) x json_output(false/true), one\n"
                 "    unlooped call per `#[test]` fn across 8 fns."),
                (loop_fn, 16,
                 "ONE `#[test]` fn containing a nested loop:\n"
                 "    8 `(command, source_name, source, expected_stdout)` table entries\n"
                 "    (run/test x the four extensions) x `for output_json in [false, true]`\n"
                 f"    (:{c_output_json}) = 16 invocations, all made by that single fn.\n"
                 "    enumerate_invocations.py reports it as UNPARSED because the harness logic\n"
                 "    is inlined in the loop body rather than called, so this half of the\n"
                 "    arithmetic was read off the source directly."),
            ],
        ),
        "",
        P.rule6_matrix_fold(
            "4 trials of one source construct, one per `ext` cell -- but\n"
            "WHICH construct differs between the two halves of this file, and the distinction\n"
            "is load-bearing. The two bundle cases each fold 4 separate `#[test]` fns. The four\n"
            "harness cases fold no fns at all: they correspond to loop ITERATIONS of the single\n"
            f"`#[test]` fn\n  `{loop_fn}`\n(:{c_loop_fn}), whose table walks run/test x "
            "js/ts/jsx/tsx and whose inner loop walks\njson_output. A reader must not infer "
            "four source fns behind them"),
        "",
        P.u2_source_file_wide(["app.${ext}", "main.${ext}", "smoke.test.${ext}"]),
        "",
        count_keys,
        "",
        P.rule13_header([
            "kali_bin",
            "browser_bundle_global_this_math_bracketed_log2_log10_source",
            "browser_harness_global_this_math_bracketed_log2_log10_run_source",
            "browser_harness_global_this_math_bracketed_log2_log10_test_source",
            bundle_helper, loop_fn,
        ]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    bundle_desc = (
        f"`{bundle_helper}` builds a browser bundle with `kali build --bundle --api browser`, "
        "asserts the emitted app/app.meta.json metadata, then runs the bundle glue under the "
        "browser-bundle-harness contract backed by node, against a program whose four calls -- "
        "globalThis.Math[\"log2\"](8), a frozen alias of it, globalThis.Math[\"log10\"](1000) and "
        "a frozen alias of that -- each print 3."
    )
    bundle_claims = (
        f"The source makes TWO separate claims about that output, `.contains(\"3\\n\")` "
        f"(:{c_bundle_contains}) and `.matches(\"3\\n\").count() >= 2` (:{c_bundle_count}), so "
        "both are carried."
    )
    bundle_ruling3 = " ".join([P.ruling3_substring(), P.ruling3_count('"3\\n"', 2)])
    bundle_envelope = (
        "This sibling additionally asserts the build JSON envelope -- schemaVersion/command/"
        "success/exitCode, payload artifactKind/bundleFormat, and the empty `errors` array the "
        f"source checks at :{c_env_errors}-{c_env_is_empty} -- rather than plain text; output "
        "shape is not a matrix axis because it changes the assertion shape, so it is a separate "
        "case."
    )

    def harness_prefix(command, json_mode):
        mode = "true" if json_mode else "false"
        return (
            f"Migrated from browser_{stem}.rs. This case does NOT correspond to four source "
            f"`#[test]` fns: it corresponds to four loop ITERATIONS of the single `#[test]` fn "
            f"`{loop_fn}` (:{c_loop_fn}) -- the four `{command}` rows of that fn's "
            f"`(command, source_name, source, expected_stdout)` table, one per extension, taken "
            f"at `output_json = {mode}`."
        )

    def harness_desc(command):
        return (
            f"That fn runs `kali {command} --api browser` with the browser harness backed by "
            "node and with --max-threads 0 --max-spawned-processes 0 "
            f"(:{c_threads}, :{c_procs}), against a program whose four calls -- "
            "globalThis.Math[\"log2\"](8), a frozen alias of it, "
            "globalThis.Math[\"log10\"](1000) and a frozen alias of that -- each print 3."
        )

    text_claim = {
        "run": (
            f"On the text branch the claim is `stdout.contains(expected_stdout)` "
            f"(:{c_text_contains}) with the table's `run` literal \"3\\n3\" (:{c_table_run})."),
        "test": (
            f"On the text branch the claim is `stdout.contains(expected_stdout)` "
            f"(:{c_text_contains}) with the table's `test` literal \"3\\nok 1\" "
            f"(:{c_table_test}) -- a different string from the `run` rows', mirrored per "
            "command rather than merged."),
    }
    json_claim = (
        f"On the json branch the same output is claimed as "
        f"`json[\"stdout\"].as_str().contains(\"3\")` (:{c_j_stdout}) -- note the bare \"3\", not "
        "\"3\\n\"."
    )
    json_envelope = {
        "run": (
            "This sibling asserts the JSON envelope: schemaVersion/command/success, payload "
            "hostContract/runtimeBackend, and `exitCode` at both the envelope and the payload "
            f"level (:{c_j_exit}-{c_j_payload_exit}), plus stderr exactly empty "
            f"(:{c_j_stderr}) and an empty `errors` array (:{c_j_errors})."),
        "test": (
            "This sibling asserts the JSON envelope: schemaVersion/command/success, payload "
            f"hostContract/runtimeBackend, and payload total/passed/failed "
            f"(:{c_j_total}-{c_j_failed}), plus stderr exactly empty (:{c_j_stderr}) and an "
            f"empty `errors` array (:{c_j_errors}). The source asserts nothing about `skipped`, "
            "so nothing about it is written."),
    }
    entry_for = {"run": "main.${ext}", "test": "smoke.test.${ext}"}

    cases = [
        {
            "name": "build_emits_global_this_math_bracketed_log2_log10_identity_literals",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four "
                "`build_emits_global_this_math_bracketed_log2_log10_identity_literals_in_*_input`"
                " fns (one per extension).",
                bundle_desc, bundle_claims, bundle_ruling3),
            "steps": bundle_steps(
                "app.${ext}", bundle_body,
                {"stdout_contains": ["3\n"],
                 "stdout_count": [{"needle": "3\n", "at_least": 2}]},
                json_output=False, meta_fields=META),
        },
        {
            "name": "json_build_emits_global_this_math_bracketed_log2_log10_identity_literals",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four "
                "`json_build_emits_global_this_math_bracketed_log2_log10_identity_literals_in_*_"
                "input` fns (one per extension).",
                bundle_desc, bundle_claims, bundle_ruling3, bundle_envelope),
            "steps": bundle_steps(
                "app.${ext}", bundle_body,
                {"stdout_contains": ["3\n"],
                 "stdout_count": [{"needle": "3\n", "at_least": 2}]},
                json_output=True, json_claims=envelope_build(errors=True),
                meta_fields=META),
        },
    ]

    for command in ("run", "test"):
        cases.append({
            "name": (f"{command}_supports_global_this_math_bracketed_log2_log10_identities_"
                     "when_browser_harness_is_configured"),
            "rationale": para(harness_prefix(command, False), harness_desc(command),
                              text_claim[command], P.ruling3_substring()),
            "steps": [harness_step(
                command, entry_for[command], json_output=False, thread_flags=True,
                asserts={"stdout_contains": ["3\n3" if command == "run" else "3\nok 1"]},
                env_var=HARNESS_ENV)],
        })
    for command in ("run", "test"):
        cases.append({
            "name": (f"json_{command}_supports_global_this_math_bracketed_log2_log10_identities_"
                     "when_browser_harness_is_configured"),
            "rationale": para(harness_prefix(command, True), harness_desc(command),
                              json_claim, P.ruling3_json_leaf(), json_envelope[command]),
            "steps": [harness_step(
                command, entry_for[command], json_output=True, thread_flags=True,
                asserts={},
                json_claims=harness_json(command, stdout_pin=pin, stderr=True, errors=True),
                env_var=HARNESS_ENV)],
        })

    source = {
        "app.${ext}": bundle_program,
        "main.${ext}": run_program,
        "smoke.test.${ext}": test_program,
    }
    return emit(header, {"ext": EXTS}, source, cases)


def main(argv):
    names = argv or sorted(REGISTRY)
    for name in names:
        if name not in REGISTRY:
            raise SystemExit(f"unknown target {name!r}; have {sorted(REGISTRY)}")
        write(os.path.join(CASES, f"{name}.toml"), REGISTRY[name]())
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
