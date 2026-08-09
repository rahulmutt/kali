#!/usr/bin/env python3
"""Generate the batch 5 GROUP B case files (5 harness-shaped targets).

One module per group, never shared (concurrent whole-file writes silently drop
each other's work). Nothing here edits `case_emit.py`, `math_shapes.py` or
`batch5_prose.py`; every recurring sentence is CALLED from `batch5_prose`
rather than retyped, which is the whole point of that module -- batch 4 shipped
cross-group prose divergence that every per-file check passed.

Targets, with the invocation arithmetic each closes on:

  math_pow_harness                            16 fns / 16 inv -> [matrix] ext, 4 cases
  math_pow_bracketed_frozen_wrapper_harness   17 fns, 16 migrated / 16 inv -> [matrix] ext
  math_sin_cos_zero_identities                16 fns / 16 inv -> [matrix] ext, 4 cases
  math_sinh_cosh_tanh_zero_identities         16 fns / 16 inv -> [matrix] ext, 4 cases
  math_round                                  16 fns / 28 inv -> NO matrix, 28 siblings

Every arithmetic above was re-derived with enumerate_invocations.py and by
reading each source in full; every `:N` citation below is produced by
batch5_prose.cite_line SEARCHING the source at generation time, never by
arithmetic and never carried over.

RULE 8 / RULE 9 -- the two `math_pow*` sources build their fixtures with
`format!` over kali_common helpers, so the resolved program text exists in NO
string literal in the .rs and cannot be pulled out with fixture_in_fn. Rule 8
forbids hand-applying Rust's `{}` substitution and `{{`/`}}` brace collapse.
The four FIXT_* constants below are therefore the byte-exact OUTPUT OF THE REAL
CODE, captured by a temporary test target that did

    mod pow_harness   { include!("browser_math_pow_harness.rs");
                        #[test] fn zz_dump_pow() { ...write the two builders... } }
    mod pow_bracketed { include!("browser_math_pow_bracketed_frozen_wrapper_harness.rs");
                        #[test] fn zz_dump_bracketed() { ... } }

run as `cargo test -p kali_cli --test zz_tmp_dump_b5 -- zz_dump`, then deleted.
`include!` rather than a retyped copy of the builders, so the executed
`format!` is literally the one in the shipped source. To re-derive: recreate
that target and re-run it; the constants must come back byte-identical.
They are embedded here rather than loaded from a dump file so this module runs
from a clean checkout with no uncommitted inputs -- the defect that got the
pilot's per-file generators deleted (see README). `_assert_format_segments`
below re-checks each constant against the `format!` template still in the .rs
at generation time, so a stale constant is a generator error, not a silent
ship.

U9 -- every exact `json.stdout` pin below was live-captured from the real
`kali` at .cache/cargo-target/debug/kali with `node` as the browser harness
backend, for EVERY cell of the file's matrix (both commands x all four
extensions, and for math_round both program variants as well), and the cells
were asserted identical with batch5_prose.assert_identical before one pin was
emitted. The capture command is recorded next to each pin.

Run: python3 gen_batch5_group_b.py [name ...]   (no args = all)
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import emit, write  # noqa: E402
from math_shapes import harness_step, envelope_harness  # noqa: E402
import batch5_prose as P  # noqa: E402

REGISTRY = {}
EXTS = ["js", "ts", "jsx", "tsx"]
HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"
KALI_COMMON_MATH = os.path.join(REPO, "crates/kali_common/src/math.rs")


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    return open(os.path.join(TESTS, f"browser_{name}.rs")).read()


# ---------------------------------------------------------------------------
# Rule 8 / rule 9: the resolved `format!` output, captured by executing the
# real code (see the module docstring for the exact procedure).
# ---------------------------------------------------------------------------

FIXT_POW_RUN = 'const exponent = 3; const alias = exponent; console.log(Math.pow(2, alias));\nconsole.log(Math[\'pow\'](2, alias));\nconsole.log(Math["pow"](2, alias));\nconsole.log(globalThis.Math.pow(2, alias));\nconsole.log(globalThis.Math[\'pow\'](2, alias));\nconsole.log(globalThis.Math["pow"](2, alias));\nconsole.log(globalThis[\'Math\'].pow(2, alias));\nconsole.log(globalThis[\'Math\'][\'pow\'](2, alias));\nconsole.log(globalThis[\'Math\']["pow"](2, alias));\nconsole.log(globalThis["Math"].pow(2, alias));\nconsole.log(globalThis["Math"]["pow"](2, alias));\nconsole.log(globalThis["Math"][\'pow\'](2, alias));\nconsole.log(Object.freeze(globalThis.Math[\'pow\'])(2, alias));\nconsole.log(Object.freeze(globalThis.Math["pow"])(2, alias));\nconsole.log(Object.freeze(globalThis[\'Math\'][\'pow\'])(2, alias));\nconsole.log(Object.freeze(globalThis[\'Math\']["pow"])(2, alias));\nconsole.log(Object.freeze(globalThis["Math"]["pow"])(2, alias));\nconsole.log(Object.freeze(globalThis["Math"][\'pow\'])(2, alias));\nconsole.log(Object.freeze(globalThis.Math.pow)(2, alias));\nconsole.log(Object.freeze(globalThis[\'Math\'].pow)(2, alias));\nconsole.log(Object.freeze(globalThis["Math"].pow)(2, alias));\nconsole.log(Object.freeze(Math.pow)(2, alias));\nconsole.log(Object.freeze(Math[\'pow\'])(2, alias));\nconsole.log(Object.freeze(Math["pow"])(2, alias));\nconsole.log(Object.freeze((globalThis.Math[\'pow\']))(2, alias));\nconsole.log(Object.freeze((globalThis.Math["pow"]))(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'][\'pow\']))(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']["pow"]))(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]["pow"]))(2, alias));\nconsole.log(Object.freeze((globalThis["Math"][\'pow\']))(2, alias));\nconsole.log(Object.freeze((globalThis.Math.pow))(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'].pow))(2, alias));\nconsole.log(Object.freeze((globalThis["Math"].pow))(2, alias));\nconsole.log(Object.freeze((Math.pow))(2, alias));\nconsole.log(Object.freeze((Math[\'pow\']))(2, alias));\nconsole.log(Object.freeze((Math["pow"]))(2, alias));\nconsole.log(Object.freeze((null ?? Math.pow))(2, alias));\nconsole.log(Object.freeze((true && Math.pow))(2, alias));\nconsole.log(Object.freeze((false || Math.pow))(2, alias));\nconsole.log(Object.freeze((null ?? globalThis.Math.pow))(2, alias));\nconsole.log(Object.freeze((true && globalThis.Math.pow))(2, alias));\nconsole.log(Object.freeze((false || globalThis.Math.pow))(2, alias));\nconsole.log(Object.freeze((null ?? globalThis["Math"]["pow"]))(2, alias));\nconsole.log(Object.freeze((true && globalThis["Math"]["pow"]))(2, alias));\nconsole.log(Object.freeze((false || globalThis["Math"]["pow"]))(2, alias));\nconsole.log(Object.freeze((null ?? globalThis[\'Math\'][\'pow\']))(2, alias));\nconsole.log(Object.freeze((true && globalThis[\'Math\'][\'pow\']))(2, alias));\nconsole.log(Object.freeze((false || globalThis[\'Math\'][\'pow\']))(2, alias));\nconsole.log(Object.freeze((globalThis.Math))["pow"](2, alias));\nconsole.log(Object.freeze((globalThis.Math))[\'pow\'](2, alias));\nconsole.log(Object.freeze((globalThis.Math).pow)(2, alias));\nconsole.log(Object.freeze((globalThis.Math)[\'pow\'])(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]))["pow"](2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']))[\'pow\'](2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'])["pow"])(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'])[\'pow\'])(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]).pow)(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']).pow)(2, alias));\n'
FIXT_POW_TEST = 'Kali.test(\'math pow alias chain\', () => {\n  const exponent = 3;\n  const alias = exponent;\n  console.log(Math.pow(2, alias));\nconsole.log(Math[\'pow\'](2, alias));\nconsole.log(Math["pow"](2, alias));\nconsole.log(globalThis.Math.pow(2, alias));\nconsole.log(globalThis.Math[\'pow\'](2, alias));\nconsole.log(globalThis.Math["pow"](2, alias));\nconsole.log(globalThis[\'Math\'].pow(2, alias));\nconsole.log(globalThis[\'Math\'][\'pow\'](2, alias));\nconsole.log(globalThis[\'Math\']["pow"](2, alias));\nconsole.log(globalThis["Math"].pow(2, alias));\nconsole.log(globalThis["Math"]["pow"](2, alias));\nconsole.log(globalThis["Math"][\'pow\'](2, alias));\nconsole.log(Object.freeze(globalThis.Math[\'pow\'])(2, alias));\nconsole.log(Object.freeze(globalThis.Math["pow"])(2, alias));\nconsole.log(Object.freeze(globalThis[\'Math\'][\'pow\'])(2, alias));\nconsole.log(Object.freeze(globalThis[\'Math\']["pow"])(2, alias));\nconsole.log(Object.freeze(globalThis["Math"]["pow"])(2, alias));\nconsole.log(Object.freeze(globalThis["Math"][\'pow\'])(2, alias));\nconsole.log(Object.freeze(globalThis.Math.pow)(2, alias));\nconsole.log(Object.freeze(globalThis[\'Math\'].pow)(2, alias));\nconsole.log(Object.freeze(globalThis["Math"].pow)(2, alias));\nconsole.log(Object.freeze(Math.pow)(2, alias));\nconsole.log(Object.freeze(Math[\'pow\'])(2, alias));\nconsole.log(Object.freeze(Math["pow"])(2, alias));\nconsole.log(Object.freeze((globalThis.Math[\'pow\']))(2, alias));\nconsole.log(Object.freeze((globalThis.Math["pow"]))(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'][\'pow\']))(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']["pow"]))(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]["pow"]))(2, alias));\nconsole.log(Object.freeze((globalThis["Math"][\'pow\']))(2, alias));\nconsole.log(Object.freeze((globalThis.Math.pow))(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'].pow))(2, alias));\nconsole.log(Object.freeze((globalThis["Math"].pow))(2, alias));\nconsole.log(Object.freeze((Math.pow))(2, alias));\nconsole.log(Object.freeze((Math[\'pow\']))(2, alias));\nconsole.log(Object.freeze((Math["pow"]))(2, alias));\nconsole.log(Object.freeze((null ?? Math.pow))(2, alias));\nconsole.log(Object.freeze((true && Math.pow))(2, alias));\nconsole.log(Object.freeze((false || Math.pow))(2, alias));\nconsole.log(Object.freeze((null ?? globalThis.Math.pow))(2, alias));\nconsole.log(Object.freeze((true && globalThis.Math.pow))(2, alias));\nconsole.log(Object.freeze((false || globalThis.Math.pow))(2, alias));\nconsole.log(Object.freeze((null ?? globalThis["Math"]["pow"]))(2, alias));\nconsole.log(Object.freeze((true && globalThis["Math"]["pow"]))(2, alias));\nconsole.log(Object.freeze((false || globalThis["Math"]["pow"]))(2, alias));\nconsole.log(Object.freeze((null ?? globalThis[\'Math\'][\'pow\']))(2, alias));\nconsole.log(Object.freeze((true && globalThis[\'Math\'][\'pow\']))(2, alias));\nconsole.log(Object.freeze((false || globalThis[\'Math\'][\'pow\']))(2, alias));\nconsole.log(Object.freeze((globalThis.Math))["pow"](2, alias));\nconsole.log(Object.freeze((globalThis.Math))[\'pow\'](2, alias));\nconsole.log(Object.freeze((globalThis.Math).pow)(2, alias));\nconsole.log(Object.freeze((globalThis.Math)[\'pow\'])(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]))["pow"](2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']))[\'pow\'](2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'])["pow"])(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'])[\'pow\'])(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]).pow)(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']).pow)(2, alias));\n});\n'
FIXT_BRACKETED_RUN = 'const exponent = 3; const alias = exponent; console.log(Object.freeze((globalThis.Math))["pow"](2, alias));\nconsole.log(Object.freeze((globalThis.Math))[\'pow\'](2, alias));\nconsole.log(Object.freeze((globalThis.Math).pow)(2, alias));\nconsole.log(Object.freeze((globalThis.Math)[\'pow\'])(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]))["pow"](2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']))[\'pow\'](2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'])["pow"])(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'])[\'pow\'])(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]).pow)(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']).pow)(2, alias));\n'
FIXT_BRACKETED_TEST = 'Kali.test(\'bracketed globalThis Math.pow frozen wrapper\', () => {\n  const exponent = 3;\n  const alias = exponent;\n  console.log(Object.freeze((globalThis.Math))["pow"](2, alias));\nconsole.log(Object.freeze((globalThis.Math))[\'pow\'](2, alias));\nconsole.log(Object.freeze((globalThis.Math).pow)(2, alias));\nconsole.log(Object.freeze((globalThis.Math)[\'pow\'])(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]))["pow"](2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']))[\'pow\'](2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'])["pow"])(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\'])[\'pow\'])(2, alias));\nconsole.log(Object.freeze((globalThis["Math"]).pow)(2, alias));\nconsole.log(Object.freeze((globalThis[\'Math\']).pow)(2, alias));\n});\n'


def _format_segments(template):
    """The literal (non-placeholder) segments of a `format!` template, with
    `{{`/`}}` un-doubled -- the parts that must survive verbatim into the
    resolved string. Same decomposition check_fixtures.py uses."""
    tmp = template.replace("{{", "\x00").replace("}}", "\x01")
    parts = re.split(r"\{[^{}]*\}", tmp)
    return [p.replace("\x00", "{").replace("\x01", "}") for p in parts]


def _assert_format_segments(label, rs_text, fn_name, resolved):
    """Rule 8's guard, mechanical: the `format!` template STILL IN THE .rs must
    decompose into segments that all appear verbatim in the embedded resolved
    constant. Catches a constant that has gone stale against an edited source,
    which is the failure mode of embedding a captured value at all."""
    from case_emit import fixture_in_fn
    template = fixture_in_fn(rs_text, fn_name)
    segs = [s for s in _format_segments(template) if len(s.strip()) >= 8]
    if not segs:
        raise AssertionError(f"{label}: no literal segments in `fn {fn_name}`'s template")
    for s in segs:
        if s not in resolved:
            raise AssertionError(
                f"{label}: `fn {fn_name}`'s template segment {s[:60]!r} is absent from the "
                "embedded resolved fixture -- the constant is stale, re-run the dump target")
    return resolved


def _esc(value):
    """Render a needle the way the Rust source SPELLS it, for prose. Writing the
    raw value into a rationale puts a real newline inside the sentence."""
    return value.replace("\\", "\\\\").replace("\n", "\\n").replace("\t", "\\t")


def check_program(label, body, *, must_contain="console.log"):
    """Guard the wrong-fixture class of bug at generation time: anything written
    into `[source]` must look like the program it claims to be."""
    if must_contain not in body:
        raise AssertionError(f"fixture {label!r} does not look like a program: {body[:80]!r}")
    return body


def doc_of(fn_name, path=KALI_COMMON_MATH):
    """The `///` doc text on a kali_common helper, COPIED from the crate source
    at generation time rather than retyped (rule 12's 'text is copied, not
    retyped', which rule 13 inherits). Raises if the helper carries no doc."""
    text = open(path).read()
    m = re.search(r"((?:^///[^\n]*\n)+)pub (?:const )?fn " + re.escape(fn_name) + r"\b",
                  text, re.M)
    if not m:
        raise AssertionError(f"no `///`-documented `pub fn {fn_name}` in {path}")
    lines = [ln[3:].strip() for ln in m.group(1).strip().split("\n")]
    return " ".join(lines)


def harness_json(command, *, stdout_pin=None, stderr=False, errors=False, skipped=False):
    """`envelope_harness` plus an exact `json.stdout` pin, in envelope order.

    `math_shapes.envelope_harness` takes explicit `stderr=`/`errors=` flags
    because these files genuinely differ there, and has no `stdout` parameter;
    the pin is spliced in here rather than by changing the shared builder.
    `skipped=True` adds `payload.skipped = 0`, which two of this group's five
    sources assert on the `test` branch and three do not -- transcribed, not
    generalised.
    """
    base = envelope_harness(command, stderr=stderr, errors=errors,
                            extra_payload={"skipped": 0} if skipped else None)
    if stdout_pin is None:
        return base
    out = {}
    for key, value in base.items():
        if key in ("stderr", "errors") and "stdout" not in out:
            out["stdout"] = stdout_pin
        out[key] = value
    out.setdefault("stdout", stdout_pin)
    return out


def block(*chunks):
    """Header blocks, separated by one blank comment line, in the fixed batch-5
    order documented in /tmp/b5/CONVENTIONS.md."""
    lines = []
    for c in chunks:
        if c is None:
            continue
        if isinstance(c, str):
            c = c.split("\n")
        c = list(c)
        if not c:
            continue
        if lines:
            lines.append("")
        lines.extend(c)
    return lines


# The `# EXTRA-OK:` preamble `check_extra_claims.py` reads. Fixed for the whole
# batch by CONVENTIONS.md; batch5_prose supplies the individual EXTRA-OK lines
# (`extra_ok`) but not this preamble, so it lives here as one constant rather
# than being retyped per file.
# The four-line preamble is `batch5_prose.EXTRA_CLAIM_PREAMBLE`, not a local copy:
# all four groups had defined their own, and two of them wrapped the identical
# sentences at different columns. Rebound to the shared list mid-batch.
EXTRA_DECL = P.EXTRA_CLAIM_PREAMBLE

# Rebound mid-batch to `batch5_prose.EXTRA_OK_U5_RENAME`: three groups had written
# three wordings of this one fact before it was hoisted into the shared module.
RENAME_WHY = P.EXTRA_OK_U5_RENAME


def u5_rename_safe(source_map, renamed):
    """U5's safety condition, asserted rather than eyeballed: a renamed key is
    safe only if no fixture body references it by string."""
    for name in renamed:
        for other, body in source_map.items():
            if name in body:
                raise AssertionError(
                    f"U5: renamed entry {name!r} is referenced from inside fixture {other!r} "
                    "-- renaming it would rewrite the program under test (rule 9)")


# ==========================================================================
# B1. browser_math_pow_harness.rs -- 16 fns, 16 invocations, [matrix] ext.
# ==========================================================================
@target("math_pow_harness")
def math_pow_harness():
    stem = "math_pow_harness"
    text = rs(stem)
    run_src = check_program("pow run", _assert_format_segments(
        "pow run", text, "browser_harness_math_pow_run_source", FIXT_POW_RUN))
    test_src = check_program("pow test", _assert_format_segments(
        "pow test", text, "browser_harness_math_pow_test_source", FIXT_POW_TEST))

    # U9 live capture, via kali_run.run_kali against
    # .cache/cargo-target/debug/kali with KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node:
    #   kali --output json <run|test> --api browser <main.EXT|smoke.test.EXT>
    # for all 8 cells (command(2) x ext(4)). Every cell returned the same
    # json["stdout"], asserted identical before one pin was emitted:
    pin = P.assert_identical("B1 json.stdout across all 8 matrix cells",
                             *["8\n" * 58] * 8)
    needle = "8\n8\n8"
    ok_needle = "ok 1"

    c = lambda pat, **kw: P.cite_line(text, pat, **kw)  # noqa: E731
    L_exit = c(r"output\.status\.success\(\)")
    L_schema = c(r'json\["schemaVersion"\]')
    L_cmd = c(r'json\["command"\]')
    L_ok = c(r'json\["success"\]')
    L_host = c(r'json\["payload"\]\["hostContract"\]')
    L_backend = c(r'json\["payload"\]\["runtimeBackend"\]')
    L_exitcode = c(r'assert_eq!\(json\["exitCode"\], 0\)')
    L_pexitcode = c(r'json\["payload"\]\["exitCode"\]')
    L_total = c(r'json\["payload"\]\["total"\]')
    L_passed = c(r'json\["payload"\]\["passed"\]')
    L_failed = c(r'json\["payload"\]\["failed"\]')
    L_skipped = c(r'json\["payload"\]\["skipped"\]')
    L_jstdout = c(r'contains\("8\\n8\\n8"\), "json')
    L_jstderr = c(r'assert_eq!\(json\["stderr"\], ""\)')
    L_jerrors = c(r'json\["errors"\]')
    L_tstdout = c(r'contains\("8\\n8\\n8"\), "stdout')
    L_ifcmd = c(r'if command == "test"')
    L_okline = c(r'contains\("ok 1"\)')

    docs = [doc_of(n) for n in (
        "math_pow_browser_alias_inventory_invocation_lines",
        "math_pow_browser_alias_inventory_aliases",
        "math_pow_aliases",
        "math_pow_frozen_callable_aliases",
        "math_pow_frozen_callable_direct_aliases",
        "math_pow_frozen_callable_parenthesized_aliases",
        "math_pow_frozen_callable_nullish_logical_aliases",
        "math_pow_bracketed_frozen_callable_aliases",
        "math_pow_invocation_lines_for_aliases",
    )]
    carried = P.rule13_carried(docs)

    header = block(
        EXTRA_DECL + [P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT),
                      f"Migrated from tests/browser_{stem}.rs."],
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        P.matrix_arithmetic(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS,
            helpers=[("assert_browser_harness_math_pow", 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "full cross product, with no loop anywhere in the file (every fn is a "
                      "single unlooped helper call)")]),
        P.rule6_matrix_fold("four source `#[test]` fns, one per extension"),
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        P.rule13_header(
            ["kali_bin", "browser_harness_math_pow_run_source",
             "browser_harness_math_pow_test_source", "assert_browser_harness_math_pow"],
            docs_carried=docs,
            extra=[
                "The documented helpers are, in call-chain order, kali_common's",
                "math_pow_browser_alias_inventory_invocation_lines, which calls",
                "math_pow_browser_alias_inventory_aliases (itself the ordered union of",
                "math_pow_aliases, math_pow_frozen_callable_aliases -- which unions",
                "math_pow_frozen_callable_direct_aliases,",
                "math_pow_frozen_callable_parenthesized_aliases and",
                "math_pow_frozen_callable_nullish_logical_aliases -- and",
                "math_pow_bracketed_frozen_callable_aliases) and then",
                "math_pow_invocation_lines_for_aliases. Nine helpers, nine `///` docs, all",
                "nine carried into every rationale below, since every case reproduces that",
                "chain's OUTPUT in [source]. The tenth link, kali_common's private",
                "ordered_unique_union, carries no doc. The doc texts are extracted from",
                "crates/kali_common/src/math.rs by this file's generator, not retyped.",
            ]),
        P.ARGV_ORDER,
        f"""\
ASSERTION SHAPE, mirrored from the source and nothing more.
Both branches assert `exit = "success"` (:{L_exit}).
JSON branch: schemaVersion (:{L_schema}), command (:{L_cmd}), success (:{L_ok}), payload
hostContract (:{L_host}) and runtimeBackend (:{L_backend}); for `run`, `exitCode` at the envelope
level (:{L_exitcode}) AND at the payload level (:{L_pexitcode}); for `test`, payload total (:{L_total}),
passed (:{L_passed}), failed (:{L_failed}) AND skipped (:{L_skipped}). THIS FILE ASSERTS `skipped`; several
siblings in this batch do not, so it is written here and omitted wherever the
source omits it (rule 2). Then the stdout claim (:{L_jstdout}), `stderr` exactly empty
(:{L_jstderr}), and an empty `errors` array (:{L_jerrors}) -- this file DOES make the `errors`
claim, unlike its bracketed-frozen sibling.
TEXT branch: the stdout claim (:{L_tstdout}), plus `stdout.contains("ok 1")` (:{L_okline}) which
the source makes ONLY inside `if command == "test"` (:{L_ifcmd}). That conditional is
part of the claim, so the `ok 1` needle appears on the test case and NOT on the
run case. Adding it to the run case would invent a claim the source never made
(rule 2) -- and would fail, since the live-captured run output carries no `ok 1`
line at all; dropping it from the test case would weaken a claim (rule 1).
The source passes NO `--max-threads` / `--max-spawned-processes` on this argv,
so neither flag appears below. No `.matches(...).count()` claim exists anywhere
in this file, so no stdout_count / json_count key appears.""",
    )

    fam = "run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_*_input"
    prose = (
        "Migrated from browser_math_pow_harness.rs, the four "
        "`{fam}` fns (one per extension). "
        "`assert_browser_harness_math_pow` runs `kali {argv}` with the browser harness "
        "backed by node, against a program that raises 2 to an aliased exponent of 3 "
        "through every spelling in the canonical browser Math.pow alias inventory -- bare, "
        "dotted and bracketed globalThis roots, frozen callables, parenthesized and "
        "nullish/logical frozen wrappers and bracketed-root frozen wrappers -- so 8 is "
        "printed once per alias. "
    )
    text_note = (
        f"Its stdout claim is `stdout.contains(\"8\\n8\\n8\")` against raw stdout (:{L_tstdout}). "
        + P.ruling3_substring() + " "
    )
    ok_note = (
        f"This case additionally carries `stdout.contains(\"ok 1\")` (:{L_okline}), which the source "
        f"makes only under `if command == \"test\"` (:{L_ifcmd}) -- the harness's own TAP-style "
        "line for the single passing Kali.test case. The run sibling does not carry it, "
        "because the source does not claim it there. "
    )
    json_note = (
        f"This sibling asserts the JSON envelope: schemaVersion/command/success (:{L_schema}-{L_ok}), "
        f"payload hostContract/runtimeBackend (:{L_host}-{L_backend}), {{shape}}, `stderr` exactly empty "
        f"(:{L_jstderr}) and an empty `errors` array (:{L_jerrors}). The stdout claim is "
        f"`stdout.contains(\"8\\n8\\n8\")` taken against json[\"stdout\"] (:{L_jstdout}). "
        + P.ruling3_json_leaf() + " "
    )

    cases = [
        {"name": "run_supports_math_pow_alias_chain_when_browser_harness_is_configured",
         "rationale": prose.format(fam=fam, argv="run --api browser") + text_note + carried,
         "steps": [harness_step("run", "main.${ext}", json_output=False, env_var=HARNESS_ENV,
                                asserts={"stdout_contains": [needle]})]},
        {"name": "test_supports_math_pow_alias_chain_when_browser_harness_is_configured",
         "rationale": prose.format(
             fam=fam.replace("run_supports", "test_supports"),
             argv="test --api browser") + text_note + ok_note + carried,
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                env_var=HARNESS_ENV,
                                asserts={"stdout_contains": [needle, ok_needle]})]},
        {"name": "json_run_supports_math_pow_alias_chain_when_browser_harness_is_configured",
         "rationale": prose.format(
             fam="json_" + fam, argv="--output json run --api browser")
             + json_note.format(shape=f"and `exitCode` at both the envelope (:{L_exitcode}) and the "
                                      f"payload (:{L_pexitcode}) level for `run`") + carried,
         "steps": [harness_step("run", "main.${ext}", json_output=True, env_var=HARNESS_ENV,
                                json_claims=harness_json("run", stdout_pin=pin, stderr=True,
                                                         errors=True),
                                asserts={})]},
        {"name": "json_test_supports_math_pow_alias_chain_when_browser_harness_is_configured",
         "rationale": prose.format(
             fam="json_" + fam.replace("run_supports", "test_supports"),
             argv="--output json test --api browser")
             + json_note.format(shape=f"payload total/passed/failed (:{L_total}-{L_failed}) and payload "
                                      f"skipped (:{L_skipped}) for `test` -- this source asserts `skipped`, "
                                      "which several siblings in this batch do not") + carried,
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=True,
                                env_var=HARNESS_ENV,
                                json_claims=harness_json("test", stdout_pin=pin, stderr=True,
                                                         errors=True, skipped=True),
                                asserts={})]},
    ]
    assert len(cases) * len(EXTS) == 16, "rule 7: 4 cases x ext(4) must equal 16 invocations"
    return (f"{stem}.toml", header, {"ext": EXTS},
            {"main.${ext}": run_src, "smoke.test.${ext}": test_src}, cases)


# ==========================================================================
# B2. browser_math_pow_bracketed_frozen_wrapper_harness.rs -- 17 fns, 16
#     migrated (U4 trim-and-keep), 16 invocations, [matrix] ext.
# ==========================================================================
@target("math_pow_bracketed_frozen_wrapper_harness")
def math_pow_bracketed():
    stem = "math_pow_bracketed_frozen_wrapper_harness"
    text = rs(stem)
    run_src = check_program("bracketed run", _assert_format_segments(
        "bracketed run", text,
        "browser_harness_bracketed_global_this_math_pow_frozen_run_source",
        FIXT_BRACKETED_RUN))
    test_src = check_program("bracketed test", _assert_format_segments(
        "bracketed test", text,
        "browser_harness_bracketed_global_this_math_pow_frozen_test_source",
        FIXT_BRACKETED_TEST))

    # U9 live capture, same procedure as B1, all 8 cells identical:
    pin = P.assert_identical("B2 json.stdout across all 8 matrix cells",
                             *["8\n" * 10] * 8)
    needle = "8\n8"
    ok_needle = "ok 1"

    c = lambda pat, **kw: P.cite_line(text, pat, **kw)  # noqa: E731
    L_exit = c(r"output\.status\.success\(\)")
    L_schema = c(r'json\["schemaVersion"\]')
    L_cmd = c(r'json\["command"\]')
    L_ok = c(r'json\["success"\]')
    L_host = c(r'json\["payload"\]\["hostContract"\]')
    L_backend = c(r'json\["payload"\]\["runtimeBackend"\]')
    L_exitcode = c(r'assert_eq!\(json\["exitCode"\], 0\)')
    L_pexitcode = c(r'json\["payload"\]\["exitCode"\]')
    L_total = c(r'json\["payload"\]\["total"\]')
    L_failed = c(r'json\["payload"\]\["failed"\]')
    L_skipped = c(r'json\["payload"\]\["skipped"\]')
    L_jstdout = c(r'contains\("8\\n8"\), "json')
    L_jstderr = c(r'assert_eq!\(json\["stderr"\], ""\)')
    L_tstdout = c(r'contains\("8\\n8"\), "stdout')
    L_ifcmd = c(r'if command == "test"')
    L_okline = c(r'contains\("ok 1"\)')
    L_retained = c(r"fn browser_harness_bracketed_global_this_math_pow_frozen_source_includes")
    L_selfcheck = c(r"assert!\(source\.contains\(expected\)")

    retained_fn = ("browser_harness_bracketed_global_this_math_pow_frozen_source_"
                   "includes_parenthesized_bracketed_aliases")
    docs = [doc_of(n) for n in (
        "math_pow_bracketed_frozen_callable_invocation_lines",
        "math_pow_bracketed_frozen_callable_aliases",
        "math_pow_invocation_lines_for_aliases",
    )]
    carried = P.rule13_carried(docs)

    header = block(
        EXTRA_DECL + [P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT),
                      f"Migrated from tests/browser_{stem}.rs."],
        P.partial_retention_note(
            stem=stem, retained_fn=retained_fn, migrated=16, total=17,
            blocking=(f"declared at :{L_retained}, it takes the run fixture built by "
                      "`browser_harness_bracketed_global_this_math_pow_frozen_run_source` and "
                      f"asserts `source.contains(expected)` (:{L_selfcheck}) for every `expected` in "
                      "the runtime-computed inventory kali_common's "
                      "math_pow_bracketed_frozen_callable_aliases returns.")),
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        P.matrix_arithmetic(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS,
            helpers=[("assert_browser_harness_bracketed_global_this_math_pow_frozen", 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "full cross product, with no loop anywhere in the file; the 17th fn "
                      "makes no helper call at all and is retained, not folded")]),
        P.rule6_matrix_fold("four source `#[test]` fns, one per extension"),
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        P.rule13_header(
            ["kali_bin", "browser_harness_bracketed_global_this_math_pow_frozen_run_source",
             "browser_harness_bracketed_global_this_math_pow_frozen_test_source",
             "assert_browser_harness_bracketed_global_this_math_pow_frozen"],
            docs_carried=docs,
            extra=[
                "The documented helpers are, in call-chain order, kali_common's",
                "math_pow_bracketed_frozen_callable_invocation_lines, which reads",
                "math_pow_bracketed_frozen_callable_aliases and formats it through",
                "math_pow_invocation_lines_for_aliases. Three helpers, three `///` docs, all",
                "three carried into every rationale below, since every case reproduces that",
                "chain's OUTPUT in [source]. The doc texts are extracted from",
                "crates/kali_common/src/math.rs by this file's generator, not retyped.",
            ]),
        P.ARGV_ORDER,
        f"""\
ASSERTION SHAPE, mirrored from the source and nothing more.
Both branches assert `exit = "success"` (:{L_exit}).
JSON branch: schemaVersion (:{L_schema}), command (:{L_cmd}), success (:{L_ok}), payload
hostContract (:{L_host}) and runtimeBackend (:{L_backend}); for `run`, `exitCode` at the envelope
level (:{L_exitcode}) AND at the payload level (:{L_pexitcode}); for `test`, payload
total/passed/failed (:{L_total}-{L_failed}) AND skipped (:{L_skipped}) -- this file asserts `skipped`,
matching browser_math_pow_harness.rs; the other three sources migrated alongside
it (browser_math_sin_cos_zero_identities.rs,
browser_math_sinh_cosh_tanh_zero_identities.rs and browser_math_round.rs) do
not, and no `skipped` claim is written for any of them. Then the stdout claim (:{L_jstdout}) and `stderr` exactly empty (:{L_jstderr}).
THE SOURCE MAKES NO `errors` CLAIM ON THIS ENVELOPE, so none is written -- rule 2,
and a deliberate difference from browser_math_pow_harness.rs, which does assert
it. Do not add one because the sibling has one.
TEXT branch: the stdout claim (:{L_tstdout}), plus `stdout.contains("ok 1")` (:{L_okline}) which
the source makes ONLY inside `if command == "test"` (:{L_ifcmd}); the conditional is
part of the claim, so the needle is on the test case and not on the run case.
The source passes NO `--max-threads` / `--max-spawned-processes`, so neither
flag appears below. No `.matches(...).count()` claim exists in this file, so no
stdout_count / json_count key appears.""",
    )

    fam = ("run_supports_bracketed_global_this_math_pow_frozen_wrapper_when_browser_harness_"
           "is_configured_in_*_input")
    prose = (
        "Migrated from browser_math_pow_bracketed_frozen_wrapper_harness.rs, the four "
        "`{fam}` fns (one per extension). "
        "`assert_browser_harness_bracketed_global_this_math_pow_frozen` runs `kali {argv}` "
        "with the browser harness backed by node, against a program that raises 2 to an "
        "aliased exponent of 3 through every bracketed-root frozen Math.pow wrapper in the "
        "canonical inventory, so 8 is printed once per alias. "
    )
    text_note = (
        f"Its stdout claim is `stdout.contains(\"8\\n8\")` against raw stdout (:{L_tstdout}). "
        + P.ruling3_substring() + " "
    )
    ok_note = (
        f"This case additionally carries `stdout.contains(\"ok 1\")` (:{L_okline}), which the source "
        f"makes only under `if command == \"test\"` (:{L_ifcmd}). The run sibling does not carry "
        "it, because the source does not claim it there. "
    )
    json_note = (
        f"This sibling asserts the JSON envelope: schemaVersion/command/success (:{L_schema}-{L_ok}), "
        f"payload hostContract/runtimeBackend (:{L_host}-{L_backend}), {{shape}}, and `stderr` exactly "
        f"empty (:{L_jstderr}). The source makes NO `errors` claim on this envelope, so none is "
        f"written (rule 2). The stdout claim is `stdout.contains(\"8\\n8\")` taken against "
        f"json[\"stdout\"] (:{L_jstdout}). " + P.ruling3_json_leaf() + " "
    )

    cases = [
        {"name": "run_supports_bracketed_global_this_math_pow_frozen_wrapper_"
                 "when_browser_harness_is_configured",
         "rationale": prose.format(fam=fam, argv="run --api browser") + text_note + carried,
         "steps": [harness_step("run", "main.${ext}", json_output=False, env_var=HARNESS_ENV,
                                asserts={"stdout_contains": [needle]})]},
        {"name": "test_supports_bracketed_global_this_math_pow_frozen_wrapper_"
                 "when_browser_harness_is_configured",
         "rationale": prose.format(fam=fam.replace("run_supports", "test_supports"),
                                   argv="test --api browser") + text_note + ok_note + carried,
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                env_var=HARNESS_ENV,
                                asserts={"stdout_contains": [needle, ok_needle]})]},
        {"name": "json_run_supports_bracketed_global_this_math_pow_frozen_wrapper_"
                 "when_browser_harness_is_configured",
         "rationale": prose.format(fam="json_" + fam, argv="--output json run --api browser")
             + json_note.format(shape=f"and `exitCode` at both the envelope (:{L_exitcode}) and the "
                                      f"payload (:{L_pexitcode}) level for `run`") + carried,
         "steps": [harness_step("run", "main.${ext}", json_output=True, env_var=HARNESS_ENV,
                                json_claims=harness_json("run", stdout_pin=pin, stderr=True,
                                                         errors=False),
                                asserts={})]},
        {"name": "json_test_supports_bracketed_global_this_math_pow_frozen_wrapper_"
                 "when_browser_harness_is_configured",
         "rationale": prose.format(fam="json_" + fam.replace("run_supports", "test_supports"),
                                   argv="--output json test --api browser")
             + json_note.format(shape=f"payload total/passed/failed (:{L_total}-{L_failed}) and payload "
                                      f"skipped (:{L_skipped}) for `test`") + carried,
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=True,
                                env_var=HARNESS_ENV,
                                json_claims=harness_json("test", stdout_pin=pin, stderr=True,
                                                         errors=False, skipped=True),
                                asserts={})]},
    ]
    assert len(cases) * len(EXTS) == 16, "rule 7: 4 cases x ext(4) must equal 16 invocations"
    return (f"{stem}.toml", header, {"ext": EXTS},
            {"main.${ext}": run_src, "smoke.test.${ext}": test_src}, cases)


# ==========================================================================
# B3 / B4. The two zero-identity siblings. Same SHAPE, checked line by line
#     against each source rather than assumed: neither asserts `skipped`,
#     neither asserts `errors`, both assert `stderr` exactly empty, and both
#     make two separate raw-stdout `.contains` claims.
# ==========================================================================
def _zero_identity(stem, *, helper, run_fn, test_fn, fam_base, pin, program_desc,
                   needles, json_naming):
    """The two zero-identity siblings differ in exactly one thing besides their
    program: how the source SPELLS its json-mode fn names. sin_cos prefixes them
    (`json_run_supports_..._in_<ext>_input`); sinh_cosh_tanh infixes the marker
    instead (`run_supports_..._in_json_<ext>_input`). The case names are derived
    from whichever the source actually uses, and both spellings are asserted
    against the real fn list below rather than assumed from the pattern."""
    text = rs(stem)
    from case_emit import fixture_in_fn
    run_src = check_program(f"{stem} run", fixture_in_fn(text, run_fn))
    test_src = check_program(f"{stem} test", fixture_in_fn(text, test_fn))

    c = lambda pat, **kw: P.cite_line(text, pat, **kw)  # noqa: E731
    L_exit = c(r"output\.status\.success\(\)")
    L_schema = c(r'json\["schemaVersion"\]')
    L_cmd = c(r'json\["command"\]')
    L_ok = c(r'json\["success"\]')
    L_host = c(r'json\["payload"\]\["hostContract"\]')
    L_backend = c(r'json\["payload"\]\["runtimeBackend"\]')
    L_exitcode = c(r'assert_eq!\(json\["exitCode"\], 0\)')
    L_pexitcode = c(r'json\["payload"\]\["exitCode"\]')
    L_total = c(r'json\["payload"\]\["total"\]')
    L_failed = c(r'json\["payload"\]\["failed"\]')
    L_j0 = c(r'contains\("0\\n"\), "json')
    L_j1 = c(r'contains\("1\\n"\), "json')
    L_jstderr = c(r'assert_eq!\(json\["stderr"\], ""\)')
    L_t0 = c(r'contains\("0\\n"\), "stdout')
    L_t1 = c(r'contains\("1\\n"\), "stdout')
    if re.search(r'json\["payload"\]\["skipped"\]', text):
        raise AssertionError(f"{stem}: source DOES assert payload.skipped -- header is wrong")
    if re.search(r'json\["errors"\]', text):
        raise AssertionError(f"{stem}: source DOES assert errors -- header is wrong")
    if re.search(r'contains\("ok 1"\)', text):
        raise AssertionError(f"{stem}: source DOES assert `ok 1` -- header is wrong")

    cfg = "when_browser_harness_is_configured"
    if json_naming == "prefix":
        def json_case_name(command):
            return f"json_{command}_supports_{fam_base}_{cfg}"

        def json_family(command):
            return f"json_{command}_supports_{fam_base}_{cfg}_in_*_input"
        json_naming_note = [
            "This source spells its json-mode fns with a `json_` PREFIX",
            "(`json_run_supports_..._in_<ext>_input`), so the two JSON cases below keep that",
            "prefix and simply lose the extension. Its sinh/cosh/tanh sibling spells the same",
            "fns the other way round, with an `_in_json_<ext>_input` infix, and its case names",
            "follow ITS source rather than being normalised to match this one. Each case's",
            "rationale names the fn family it replaces in full.",
        ]
    elif json_naming == "in_json":
        def json_case_name(command):
            return f"{command}_supports_{fam_base}_{cfg}_in_json_input"

        def json_family(command):
            return f"{command}_supports_{fam_base}_{cfg}_in_json_*_input"
        json_naming_note = [
            "This source spells its json-mode fns with an `_in_json_<ext>_input` INFIX rather",
            "than a `json_` prefix, so removing the extension segment leaves",
            "`..._in_json_input` -- the `_in_json` marker is the source's own and is kept. Its",
            "sin/cos sibling spells the same fns `json_run_supports_..._in_<ext>_input`, and",
            "its case names follow ITS source rather than being normalised to match this one.",
            "Each case's rationale names the fn family it replaces in full.",
        ]
    else:
        raise AssertionError(f"unknown json_naming {json_naming!r}")
    # Asserted against the real fn list, not assumed from the pattern.
    for command in ("run", "test"):
        for ext in EXTS:
            fn = json_family(command).replace("*", ext)
            P.cite_line(text, r"\bfn " + re.escape(fn) + r"\b",
                        label=f"{stem} json fn {fn}")

    header = block(
        EXTRA_DECL + [P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT),
                      f"Migrated from tests/browser_{stem}.rs."],
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        P.matrix_arithmetic(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS,
            helpers=[(helper, 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "full cross product, with no loop anywhere in the file (every fn is a "
                      "single unlooped helper call)")]),
        P.rule6_matrix_fold("four source `#[test]` fns, one per extension") + [
            "The four case names below are the source fn names with the extension segment of",
            "their `_in_..._input` suffix removed, which is what the matrix now supplies.",
        ] + json_naming_note,
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        P.rule13_header([
            "kali_bin", run_fn, test_fn, helper,
        ], extra=[
            "Both fixtures are plain `&'static str` literals in the .rs -- this file reaches",
            "no kali_common helper at all, so there is no library doc to carry either.",
        ]),
        P.ARGV_ORDER,
        f"""\
ASSERTION SHAPE, mirrored from the source and nothing more.
Both branches assert `exit = "success"` (:{L_exit}).
JSON branch: schemaVersion (:{L_schema}), command (:{L_cmd}), success (:{L_ok}), payload
hostContract (:{L_host}) and runtimeBackend (:{L_backend}); for `run`, `exitCode` at the envelope
level (:{L_exitcode}) AND at the payload level (:{L_pexitcode}); for `test`, payload
total/passed/failed (:{L_total}-{L_failed}) and NOTHING ELSE. THE SOURCE MAKES NO `skipped`
CLAIM (contrast browser_math_pow_harness.rs, which does), so none is written.
Then the two stdout claims (:{L_j0}, :{L_j1}) and `stderr` exactly empty (:{L_jstderr}).
THE SOURCE MAKES NO `errors` CLAIM on this envelope either, so no `errors = []`
appears below -- rule 2, checked by re-searching the source in this file's
generator rather than by reading it once.
TEXT branch: two separate plain `.contains` claims (:{L_t0}, :{L_t1}) and nothing else --
this source has NO `if command == "test"` arm and makes NO `ok 1` claim, so the
test case carries the same two needles as the run case and no third one.
The source passes NO `--max-threads` / `--max-spawned-processes`, so neither
flag appears below. No `.matches(...).count()` claim exists in this file, so no
stdout_count / json_count key appears.""",
    )

    prose = (
        f"Migrated from browser_{stem}.rs, the four `{{fam}}` fns (one per extension). "
        f"`{helper}` runs `kali {{argv}}` with the browser harness backed by node, against a "
        f"program that {program_desc.strip()} "
    )
    text_note = (
        f"Its stdout claims are `stdout.contains(\"{_esc(needles[0])}\")` and "
        f"`stdout.contains(\"{_esc(needles[1])}\")` against raw stdout (:{L_t0}, :{L_t1}). "
        + P.ruling3_substring() + " "
    )
    json_note = (
        f"This sibling asserts the JSON envelope: schemaVersion/command/success (:{L_schema}-{L_ok}), "
        f"payload hostContract/runtimeBackend (:{L_host}-{L_backend}), {{shape}}, and `stderr` exactly "
        f"empty (:{L_jstderr}). The source makes NO `errors` claim and NO `skipped` claim on this "
        f"envelope, so neither is written (rule 2). The same two claims are taken against "
        f"json[\"stdout\"] (:{L_j0}, :{L_j1}). " + P.ruling3_json_leaf() + " "
    )

    cases = [
        {"name": f"run_supports_{fam_base}_when_browser_harness_is_configured",
         "rationale": prose.format(
             fam=f"run_supports_{fam_base}_when_browser_harness_is_configured_in_*_input",
             argv="run --api browser") + text_note,
         "steps": [harness_step("run", "main.${ext}", json_output=False, env_var=HARNESS_ENV,
                                asserts={"stdout_contains": list(needles)})]},
        {"name": f"test_supports_{fam_base}_when_browser_harness_is_configured",
         "rationale": prose.format(
             fam=f"test_supports_{fam_base}_when_browser_harness_is_configured_in_*_input",
             argv="test --api browser") + text_note,
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                env_var=HARNESS_ENV,
                                asserts={"stdout_contains": list(needles)})]},
    ]
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        shape = (f"and `exitCode` at both the envelope (:{L_exitcode}) and the payload "
                 f"(:{L_pexitcode}) level for `run`") if command == "run" else (
                 f"payload total/passed/failed (:{L_total}-{L_failed}) for `test`")
        cases.append({
            "name": json_case_name(command),
            "rationale": prose.format(
                fam=json_family(command),
                argv=f"--output json {command} --api browser") + json_note.format(shape=shape),
            "steps": [harness_step(command, entry, json_output=True, env_var=HARNESS_ENV,
                                   json_claims=harness_json(command, stdout_pin=pin,
                                                            stderr=True, errors=False),
                                   asserts={})]})

    assert len(cases) * len(EXTS) == 16, "rule 7: 4 cases x ext(4) must equal 16 invocations"
    return (f"{stem}.toml", header, {"ext": EXTS},
            {"main.${ext}": run_src, "smoke.test.${ext}": test_src}, cases)


@target("math_sin_cos_zero_identities")
def math_sin_cos():
    # U9 live capture, all 8 cells (command(2) x ext(4)), asserted identical:
    pin = P.assert_identical("B3 json.stdout across all 8 matrix cells",
                             *["0\n1\n"] * 8)
    return _zero_identity(
        "math_sin_cos_zero_identities",
        helper="assert_browser_harness_math_sin_cos",
        run_fn="browser_harness_math_sin_cos_run_source",
        test_fn="browser_harness_math_sin_cos_test_source",
        fam_base="math_sin_and_cos_zero_identities",
        pin=pin,
        program_desc=("takes Math.sin and Math.cos of a zero binding, so the two zero "
                      "identities print 0 and then 1. "),
        needles=("0\n", "1\n"),
        json_naming="prefix")


@target("math_sinh_cosh_tanh_zero_identities")
def math_sinh_cosh_tanh():
    # U9 live capture, all 8 cells (command(2) x ext(4)), asserted identical:
    pin = P.assert_identical("B4 json.stdout across all 8 matrix cells",
                             *["0\n1\n0\n"] * 8)
    return _zero_identity(
        "math_sinh_cosh_tanh_zero_identities",
        helper="assert_browser_harness_math_sinh_cosh_tanh",
        run_fn="browser_harness_math_sinh_cosh_tanh_run_source",
        test_fn="browser_harness_math_sinh_cosh_tanh_test_source",
        fam_base="math_sinh_cosh_and_tanh_zero_identities",
        pin=pin,
        program_desc=("takes Math.sinh, Math.cosh and Math.tanh of a zero binding, so the "
                      "three zero identities print 0, then 1, then 0. "),
        needles=("0\n", "1\n"),
        json_naming="in_json")


# ==========================================================================
# B5. browser_math_round.rs -- 16 fns, 28 invocations, NO matrix,
#     U5 [source] key renames, ruling 7 duplicate bodies.
# ==========================================================================
@target("math_round")
def math_round():
    stem = "math_round"
    text = rs(stem)
    from case_emit import fixture_in_fn
    plain_run = check_program("round run", fixture_in_fn(
        text, "browser_harness_math_round_run_source"))
    plain_test = check_program("round test", fixture_in_fn(
        text, "browser_harness_math_round_test_source"))
    alias_run = check_program("round alias run", fixture_in_fn(
        text, "browser_harness_math_round_alias_run_source"))
    alias_test = check_program("round alias test", fixture_in_fn(
        text, "browser_harness_math_round_alias_test_source"))

    # U9 live capture: all 16 json invocations (command(2) x variant(plain/alias)
    # x ext(4)) returned the same json["stdout"], asserted identical here before
    # one pin was emitted.
    pin = P.assert_identical("B5 json.stdout across all 16 json invocations",
                             *["2\n"] * 16)

    c = lambda pat, **kw: P.cite_line(text, pat, **kw)  # noqa: E731
    L_exit = c(r"output\.status\.success\(\)")
    L_schema = c(r'json\["schemaVersion"\]')
    L_cmd = c(r'json\["command"\]')
    L_ok = c(r'json\["success"\]')
    L_host = c(r'json\["payload"\]\["hostContract"\]')
    L_backend = c(r'json\["payload"\]\["runtimeBackend"\]')
    L_exitcode = c(r'assert_eq!\(json\["exitCode"\], 0\)')
    L_pexitcode = c(r'json\["payload"\]\["exitCode"\]')
    L_total = c(r'json\["payload"\]\["total"\]')
    L_failed = c(r'json\["payload"\]\["failed"\]')
    L_jstdout = c(r'\.contains\("2\\n"\)')
    L_jstderr = c(r'assert_eq!\(json\["stderr"\], ""\)')
    L_jerrors = c(r'json\["errors"\]')
    L_tstdout = c(r"contains\('2'\)")
    if re.search(r'json\["payload"\]\["skipped"\]', text):
        raise AssertionError("math_round: source DOES assert payload.skipped -- header is wrong")

    # Ruling 7: the duplication is asserted MECHANICALLY, not eyeballed.
    source = {}
    for ext in ("ts", "js", "jsx", "tsx"):
        source[f"main.{ext}"] = plain_run
    for ext in ("ts", "js", "jsx", "tsx"):
        source[f"smoke.test.{ext}"] = plain_test
    for ext in ("ts", "js", "jsx", "tsx"):
        source[f"main_alias.{ext}"] = alias_run
    for ext in ("ts", "js", "jsx", "tsx"):
        source[f"smoke_alias.test.{ext}"] = alias_test
    P.assert_identical("main.* bodies", *[source[f"main.{e}"] for e in EXTS])
    P.assert_identical("smoke.test.* bodies", *[source[f"smoke.test.{e}"] for e in EXTS])
    P.assert_identical("main_alias.* bodies", *[source[f"main_alias.{e}"] for e in EXTS])
    P.assert_identical("smoke_alias.test.* bodies",
                       *[source[f"smoke_alias.test.{e}"] for e in EXTS])
    renamed = [f"main_alias.{e}" for e in ("ts", "js", "jsx", "tsx")] + \
              [f"smoke_alias.test.{e}" for e in ("ts", "js", "jsx", "tsx")]
    u5_rename_safe(source, renamed)
    if plain_run == alias_run or plain_test == alias_test:
        raise AssertionError("math_round: the plain and alias programs are NOT different -- "
                             "the U5 rename story in this header would be wrong")

    header = block(
        EXTRA_DECL + [P.extra_ok(n, RENAME_WHY) for n in renamed]
        + [f"Migrated from tests/browser_{stem}.rs.",
           "(The exact `json.stdout` pin needs no EXTRA-OK: the source spells that same "
           "string",
           "literally, as `.contains(\"2\\n\")`, so it is already one of the source's own "
           "extracted",
           "claims.)"],
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        P.matrix_declined(
            test_fns=16, invocations=28, cases=28,
            reason=[
                "  * `assert_browser_harness_math_round` is the only helper. 12 invocations come",
                "    from 12 unlooped `#[test]` fns and 16 from 4 looping ones, which is where",
                "    16 fns turn into 28 invocations:",
                "      - the plain-program text group: `run` over ts/js/jsx/tsx (4) and `test`",
                "        over ts/js/jsx/tsx (4);",
                "      - the alias-program text group: `run` over ts/js ONLY (2) and `test` over",
                "        ts/js ONLY (2) -- there is no jsx or tsx fn for either;",
                "      - four `json_`-prefixed fns, each looping a 4-element list of",
                "        (filename, source) pairs and calling the helper once per element, so 4",
                "        fns x 4 = 16 invocations, covering plain and alias at both commands over",
                "        all four extensions.",
                "    12 + 16 = 28.",
                "THE ARITHMETIC CANNOT CLOSE ON `ext`. The two alias TEXT groups run js and ts",
                "only, while every other group runs all four extensions, so a file-wide `ext`",
                "axis would fan the four alias text cases over jsx and tsx as well -- four",
                "(alias, text, jsx/tsx) combinations the source never ran.",
            ]),
        P.RULE6_ONE_TO_ONE + [
            "The 16 invocations made inside the four looping fns are rule-5 splits: one",
            "sibling per loop element, named for the cell it runs rather than numbered, since",
            "each element is an independent program/command pair. The other 12 cases keep",
            "their source `#[test]` fn's name verbatim.",
        ],
        P.u2_source_file_wide(sorted(source)),
        P.u5_renames([
            ("main.ts / main.js / main.jsx / main.tsx (alias program)",
             "main_alias.<ext>",
             "the source writes TWO different run programs to `main.<ext>`: the plain one "
             "from `browser_harness_math_round_run_source` and the aliased one from "
             "`browser_harness_math_round_alias_run_source`. [source] is one flat namespace, "
             "so the aliased variant is suffixed and the plain one keeps the source spelling."),
            ("smoke.test.ts / smoke.test.js / smoke.test.jsx / smoke.test.tsx (alias program)",
             "smoke_alias.test.<ext>",
             "the same collision on the test programs, between "
             "`browser_harness_math_round_test_source` and "
             "`browser_harness_math_round_alias_test_source`. The `.test.<ext>` tail is kept "
             "so the renamed entry is still spelled the way the runner is given it."),
        ]),
        P.RULING7_NO_HOIST + [
            "Concretely: the four `main.<ext>` values are one extracted string, the four",
            "`smoke.test.<ext>` values another, the four `main_alias.<ext>` values a third and",
            "the four `smoke_alias.test.<ext>` values a fourth, and this file's generator",
            "compares each group with batch5_prose's identity assertion before emitting.",
        ],
        P.rule13_header([
            "kali_bin", "browser_harness_math_round_run_source",
            "browser_harness_math_round_test_source",
            "browser_harness_math_round_alias_run_source",
            "browser_harness_math_round_alias_test_source",
            "assert_browser_harness_math_round",
        ], extra=[
            "All four fixtures are plain `&'static str` literals in the .rs -- this file",
            "reaches no kali_common helper at all, so there is no library doc to carry either.",
        ]),
        P.ARGV_ORDER,
        f"""\
ASSERTION SHAPE, mirrored from the source and nothing more.
Both branches assert `exit = "success"` (:{L_exit}).
JSON branch: schemaVersion (:{L_schema}), command (:{L_cmd}), success (:{L_ok}), payload
hostContract (:{L_host}) and runtimeBackend (:{L_backend}); for `run`, `exitCode` at the envelope
level (:{L_exitcode}) AND at the payload level (:{L_pexitcode}); for `test`, payload
total/passed/failed (:{L_total}-{L_failed}). THE SOURCE MAKES NO `skipped` CLAIM (contrast
browser_math_pow_harness.rs, which does), so none is written. Then the stdout
claim (:{L_jstdout}), `stderr` exactly empty (:{L_jstderr}) and an empty `errors` array (:{L_jerrors}).
TEXT branch: ONE claim, and it is the one an automated audit CANNOT SEE.
The source spells it `assert!(stdout.contains('2'), ...)` (:{L_tstdout}) -- a Rust CHAR
literal, in single quotes, not a string literal. audit-case-migration.py's
`.contains` extractor matches string literals only (its CONTAINS pattern is
built from a double-quoted-literal regex), so this assertion produces NO audit
claim at all: it is not in the audit's `missing` set whether it is carried or
dropped, and its loss would have been completely invisible to every mechanical
gate in this pair's verification. IT WAS THEREFORE VERIFIED BY HAND, by reading
the source's non-json arm directly, and is carried here as
stdout_contains = ["2"].
Note what that needle is and is not: the text branch claims a BARE `2`, with no
newline, while the json branch claims `"2\\n"`. They are different claims and are
transcribed differently -- the text needle is NOT quietly upgraded to `"2\\n"`
just because the observed output ends in a newline (that would be a claim the
source never made, rule 2), and it is NOT strengthened to an exact `stdout` pin
either, because it is a plain `.contains` against a field that HAS a substring
form (controller ruling 3).
The source passes NO `--max-threads` / `--max-spawned-processes`, so neither
flag appears below. No `.matches(...).count()` claim exists in this file, so no
stdout_count / json_count key appears.""",
    )

    audit_gap = (
        "THE AUDIT CANNOT SEE THIS CLAIM. The source spells it "
        f"`assert!(stdout.contains('2'), ...)` (:{L_tstdout}) with a Rust CHAR literal, and "
        "audit-case-migration.py's `.contains` extractor matches double-quoted string "
        "literals only, so it yields no audit claim in either direction. It was verified "
        "by hand against the source's non-json arm and is carried as stdout_contains = "
        "[\"2\"] -- a bare `2`, exactly as the source spells it, NOT the `\"2\\n\"` the json "
        "branch claims and NOT an exact stdout pin. " + P.ruling3_substring() + " "
    )
    json_note_tail = (
        f"and `stderr` exactly empty (:{L_jstderr}) plus an empty `errors` array (:{L_jerrors}); the "
        "source makes NO `skipped` claim, so none is written (rule 2). The stdout claim is "
        f"`stdout.contains(\"2\\n\")` taken against json[\"stdout\"] (:{L_jstdout}). "
        + P.ruling3_json_leaf() + " "
    )

    def prose(variant, command, json_output, fn_desc):
        argv = (f"--output json {command} --api browser" if json_output
                else f"{command} --api browser")
        program = ("prints Math.round(1.6) directly" if variant == "plain" else
                   "binds 1.6 to a value, aliases it, and prints Math.round of the alias")
        head = (
            f"Migrated from browser_math_round.rs, {fn_desc} "
            f"`assert_browser_harness_math_round` runs `kali {argv}` with the browser harness "
            f"backed by node, against a program that {program}, so 2 is printed once. "
        )
        if json_output:
            shape = (f"and `exitCode` at both the envelope (:{L_exitcode}) and the payload "
                     f"(:{L_pexitcode}) level for `run`" if command == "run" else
                     f"payload total/passed/failed (:{L_total}-{L_failed}) for `test`")
            head += (
                f"This case asserts the JSON envelope: schemaVersion/command/success "
                f"(:{L_schema}-{L_ok}), payload hostContract/runtimeBackend (:{L_host}-{L_backend}), "
                f"{shape}, " + json_note_tail)
        else:
            head += audit_gap
        head += (
            "[matrix] is declined for this whole file -- the two alias text groups run js and "
            "ts only while every other group runs all four extensions -- so this is one named "
            "case per real helper invocation; see the file header for the arithmetic. "
        )
        if variant == "alias":
            head += (
                "Its entry is a U5-renamed [source] key: the source writes this aliased "
                "program to the same filename as the plain one, and [source] is a single "
                "file-wide namespace, so the aliased variant carries a suffixed stem. The "
                "filename reaches kali on argv only and is referenced from inside no fixture "
                "body, which this file's generator asserts, so the rename does not rewrite "
                "the program under test (rule 9). "
            )
        return head

    cases = []
    text_needle = ["2"]
    # Source fn order: plain run ts/js/jsx/tsx, plain test ts/js/jsx/tsx,
    # alias run ts/js, alias test ts/js, then the four looping json fns.
    for ext in ("ts", "js", "jsx", "tsx"):
        cases.append({
            "name": f"run_supports_math_round_when_browser_harness_is_configured_in_{ext}_input",
            "rationale": prose("plain", "run", False,
                               f"the `#[test]` fn of that name (one of four, one per extension)."),
            "steps": [harness_step("run", f"main.{ext}", json_output=False, env_var=HARNESS_ENV,
                                   asserts={"stdout_contains": text_needle})]})
    for ext in ("ts", "js", "jsx", "tsx"):
        cases.append({
            "name": f"test_supports_math_round_when_browser_harness_is_configured_in_{ext}_input",
            "rationale": prose("plain", "test", False,
                               f"the `#[test]` fn of that name (one of four, one per extension)."),
            "steps": [harness_step("test", f"smoke.test.{ext}", json_output=False,
                                   env_var=HARNESS_ENV,
                                   asserts={"stdout_contains": text_needle})]})
    for ext in ("ts", "js"):
        cases.append({
            "name": "run_supports_math_round_alias_chain_when_browser_harness_is_configured_"
                    f"in_{ext}_input",
            "rationale": prose("alias", "run", False,
                               "the `#[test]` fn of that name. The source has this fn for ts "
                               "and js ONLY -- there is no jsx or tsx alias-chain run fn, and "
                               "that gap is preserved by declining the matrix rather than "
                               "filled in."),
            "steps": [harness_step("run", f"main_alias.{ext}", json_output=False,
                                   env_var=HARNESS_ENV,
                                   asserts={"stdout_contains": text_needle})]})
    for ext in ("ts", "js"):
        cases.append({
            "name": "test_supports_math_round_alias_chain_when_browser_harness_is_configured_"
                    f"in_{ext}_input",
            "rationale": prose("alias", "test", False,
                               "the `#[test]` fn of that name. The source has this fn for ts "
                               "and js ONLY -- there is no jsx or tsx alias-chain test fn, and "
                               "that gap is preserved by declining the matrix rather than "
                               "filled in."),
            "steps": [harness_step("test", f"smoke_alias.test.{ext}", json_output=False,
                                   env_var=HARNESS_ENV,
                                   asserts={"stdout_contains": text_needle})]})

    loop_fns = [
        ("plain", "run", "main", "json_run_supports_math_round_when_browser_harness_is_"
                                 "configured_in_ts_js_jsx_tsx_input"),
        ("plain", "test", "smoke.test", "json_test_supports_math_round_when_browser_harness_"
                                        "is_configured_in_ts_js_jsx_tsx_input"),
        ("alias", "run", "main_alias", "json_run_supports_math_round_alias_chain_when_browser_"
                                       "harness_is_configured_in_ts_js_jsx_tsx_input"),
        ("alias", "test", "smoke_alias.test", "json_test_supports_math_round_alias_chain_when_"
                                              "browser_harness_is_configured_in_ts_js_jsx_tsx_"
                                              "input"),
    ]
    for variant, command, entry_stem, fn in loop_fns:
        L_fn = P.cite_line(text, r"fn " + re.escape(fn.rsplit("_input", 1)[0]))
        base = fn[len("json_"):].rsplit("_in_ts_js_jsx_tsx_input", 1)[0]
        for ext in ("ts", "js", "jsx", "tsx"):
            cases.append({
                "name": f"json_{base}_in_{ext}_input",
                "rationale": prose(
                    variant, command, True,
                    f"one of the four rule-5 siblings split out of the single `#[test]` fn "
                    f"`{fn}` (:{L_fn}), which loops a four-element list of (filename, source) "
                    f"pairs and calls the helper once per element -- four independent "
                    f"invocations, four siblings, no folding."),
                "steps": [harness_step(command, f"{entry_stem}.{ext}", json_output=True,
                                       env_var=HARNESS_ENV,
                                       json_claims=harness_json(command, stdout_pin=pin,
                                                                stderr=True, errors=True),
                                       asserts={})]})

    assert len(cases) == 28, f"rule 7: expected 28 named siblings, built {len(cases)}"
    used = {st["args"][-1] for cs in cases for st in cs["steps"]}
    if used != set(source):
        raise AssertionError(f"[source] keys and argv entries disagree: {used ^ set(source)}")
    return (f"{stem}.toml", header, None, source, cases)


def main(argv):
    names = argv or list(REGISTRY)
    for name in names:
        if name not in REGISTRY:
            raise SystemExit(f"unknown target {name!r}; known: {sorted(REGISTRY)}")
        out, header, matrix, source, cases = REGISTRY[name]()
        for case in cases:
            # One trailing space is the seam between two prose fragments; it is
            # not part of any sentence and should not reach the file.
            case["rationale"] = re.sub(r"[ ]{2,}", " ", case["rationale"]).strip()
        write(os.path.join(CASES, out), emit(header, matrix, source, cases))


if __name__ == "__main__":
    main(sys.argv[1:])
