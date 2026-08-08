#!/usr/bin/env python3
"""Generate the batch 4 case files (all except math_log2_log10.toml, which has
its own script because it shipped in its own commit as the count keys' proof).

One function per source file. Each returns the full spec so a reviewer can read
the mapping decision, the matrix arithmetic and the assertion set in one place.
Every fixture is pulled from the .rs through lexer.py (rule 9); nothing here
retypes a program under test.

Run: python3 gen_batch4.py [name ...]   (no args = all)
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import fixture, fixture_in_fn, fixture_starting, emit, write  # noqa: E402
from math_shapes import (  # noqa: E402
    bundle_steps, harness_step, envelope_build, envelope_harness, META,
)

REGISTRY = {}


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    return open(os.path.join(TESTS, f"browser_{name}.rs")).read()


# Prose fragments reused across files. Each is factual about the shape it
# describes; per U8 every backticked fn name below is grepped against the real
# source's fn list by check_rationale_fn_names.py before shipping.

MIRROR_COUNT = (
    "The source spells this claim as `.matches({needle!r}).count() >= {n}`, which the case "
    "format carries directly as `{key}` with `at_least = {n}`. It is not weakened to a "
    "`*_contains` (one occurrence would satisfy that) and not strengthened to `exact` "
    "(the source states a lower bound, not an equality), per controller ruling 3 -- mirror "
    "the source."
)

NO_MATRIX_HEADER = """\
NO [matrix] (rule 7 / U1). The invocation enumeration below does not form a
uniform cross product, so a file-wide axis would fan at least one case over a
cell the source never runs -- inventing an untested combination, which is both
a rule 7 arithmetic failure and a rule 2 violation. `expand()` fans every
[[case]] by every axis with no per-case opt-out
(crates/kali_case_runner/src/expand.rs), so the axis is dropped for the WHOLE
file and every invocation becomes its own named sibling [[case]]."""

RULE13_NOTE = """\
RULE 13 -- transitive helper docs. Every fn in each call chain was checked for
a `///` doc comment; none of this file's own helpers carries one. The chain
reaches `kali_runtime_contract::browser_bundle_harness_script` and
`::browser_harness_command_parts_for`, which do carry one-line `///` docs, but
in the migrated form this case file never calls them -- the
`browser_bundle_harness` step kind means the case RUNNER does
(crates/kali_case_runner/src/steps.rs:70), so those docs describe shared runner
infrastructure (design spec 5.3), not what this case claims. All 45 case files
shipped before batch 4 do the same (measured: 0 of 45 carry either string).
Raised in the batch 4 report as a standing question rather than decided here."""


def no_rust_comments(name):
    return f"""\
RULE 12 (carry every source comment): `browser_{name}.rs` contains no Rust
comments -- the only `//` in the file is the `// kali-tree-shake:` marker
inside a JS fixture body, which is program text and is carried verbatim into
[source]. Nothing to move into `rationale`; comment_coverage.py is run with
--allow-empty for this pair."""


# --------------------------------------------------------------------------
# browser_math_asinh_acosh_atanh_identities.rs -- 24 fns. THE RESTORED
# RETENTION: batch 3 kept this file whole under a ruling that
# `.matches().count()` was a 5.11 outlier; the human partner reversed that
# ruling and the count keys exist because of this file. Its `//!` header is
# deleted in the same commit that adds this case file, and the case file is
# written from scratch rather than resurrected from git -- the old one
# predates the keys and encoded the claim as a contiguous-substring
# approximation that was ruled a rule 2 violation.
# --------------------------------------------------------------------------
@target("math_asinh_acosh_atanh_identities")
def asinh():
    text = rs("math_asinh_acosh_atanh_identities")
    # Fn-anchored, not line-anchored: this file's 85-line `//!` retention
    # header is DELETED by this migration, which shifts every line below it.
    # A hardcoded line range silently extracted the wrong literals the first
    # time this was written -- see fixture_in_fn's docstring.
    bundle_src = fixture_in_fn(text, "browser_bundle_math_inverse_hyperbolic_source")
    run_src = fixture_in_fn(text, "browser_harness_math_inverse_hyperbolic_run_source")
    test_src = fixture_in_fn(text, "browser_harness_math_inverse_hyperbolic_test_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_math_inverse_hyperbolic", "const mod = await import(")

    needle, bound = "0\n", 3
    count_stdout = [{"needle": needle, "at_least": bound}]
    count_json = [{"path": "stdout", "needle": needle, "at_least": bound}]

    header = f"""\
Migrated from tests/browser_math_asinh_acosh_atanh_identities.rs.

HISTORY -- this file is a RESTORED RETENTION. Batch 3 retained the whole .rs
under a controller ruling that `.matches(...).count()` was a design-spec 5.11
outlier and that no assertion key would be added for it, and deleted the case
file it had shipped in `50061950a4`. The human partner REVERSED that ruling
during the batch 4 interlude: `stdout_count`/`json_count` were added to the
format (design spec 5.4, now twelve keys) specifically because this file's 24
tests, and 12 other files like it, would otherwise have been retained
hand-written against a 5.11 budget of ~8 targets for the entire crate. This
case file is written FROM SCRATCH, not recovered from git: the deleted one
predates the keys and encoded `count() >= 3` as a contiguous `"0\\n0\\n0\\n"`
substring, which asserts ADJACENCY the source never claimed (rule 2). The
retention `//!` header is deleted from the .rs in the same commit -- retention
prose in a fully migrated file is worse than none.

{no_rust_comments("math_asinh_acosh_atanh_identities")}
(The deleted `//!` header was migration bookkeeping about the retention, not
prose about the behaviour under test; it is superseded by this file's
existence and is preserved in git history and in the batch 3/4 reports.)

RULE 7 / U1 -- MATRIX ARITHMETIC, closes exactly. Enumerated mechanically with
tools/task-18-browser-pilot/enumerate_invocations.py: 24 #[test] fns, 24
invocations, no loops.
  * `assert_browser_bundle_math_inverse_hyperbolic(filename, json_output)`
    -- 8 = ext(js/ts/jsx/tsx) x json_output(false/true), full cross product.
  * `assert_browser_harness_math_inverse_hyperbolic(command, filename,
    source, json_output)` -- 16 = command(run/test) x ext(4) x
    json_output(false/true), full cross product.
`ext` is the one axis both helpers vary over uniformly and completely, so:
6 [[case]] x ext(4) = 24 trials = 24 #[test] fns. Per rule 6 the fold is
stated here -- each [[case]] corresponds to 4 source fns, one per cell, and
the assertion mapping stays 1:1 per trial. `command` and `json_output` are NOT
axes: each changes the assertion SHAPE (json envelope vs text stdout;
`exitCode` for run vs `total`/`passed`/`failed` for test), which design spec
5.6's closing note excludes from a matrix.

U2 -- [source] is file-wide and that is safe here: all three fixtures are
written unconditionally into a fresh temp dir, none is behind an `if`, and no
case's point is a file's presence or absence. Each command names its entry on
argv.

THE COUNT CLAIM. Three sites, all the same bound: `:202` (bundle harness raw
stdout), `:253` (`json["stdout"].as_str()`, inside `if json_output`), `:257`
(raw stdout, else branch). Carried as `stdout_count` on the two raw surfaces
and `json_count` on the JSON leaf. Live output is `0\\n0\\n0\\n` -- exactly
three occurrences, since Math.asinh(0), Math.acosh(1) and Math.atanh(0) all
print 0 -- so the claim sits exactly on its `>= 3` boundary. It is still
written `at_least = 3` and not `exact = 3`: the source states a lower bound,
and pinning equality would assert something it never did (rule 2).

{RULE13_NOTE}

ASSERTION SHAPE, mirrored and nothing more. This source asserts NO `errors`
array anywhere -- neither on the build envelope (:150-159) nor on the harness
envelope (:237-254) -- so no `errors = []` is written, unlike several siblings
in this batch that do assert it. The harness json branch DOES assert
`json["stderr"] == ""` (:254). The bundle harness step carries only the count
claim (:202); there is no accompanying `.contains`. The source passes no
`--max-threads`/`--max-spawned-processes`, so neither appears on argv."""

    bundle_prose = (
        "Migrated from browser_math_asinh_acosh_atanh_identities.rs. "
        "`assert_browser_bundle_math_inverse_hyperbolic` builds a browser bundle "
        "(`kali build --bundle --api browser`), asserts the emitted `app/app.meta.json` "
        "metadata, then runs the bundle glue under the browser-bundle-harness contract. "
        "The bundled program calls Math.asinh(0), Math.acosh(1) and Math.atanh(0), each of "
        "which prints `0`. " + MIRROR_COUNT.format(needle="0\n", n=3, key="stdout_count") +
        " This file is the reason the count keys exist: batch 3 retained all 24 of its tests "
        "hand-written because the claim had no expressible form, and the human partner "
        "reversed that by adding the keys."
    )
    harness_prose = (
        "Migrated from browser_math_asinh_acosh_atanh_identities.rs. "
        "`assert_browser_harness_math_inverse_hyperbolic` runs `kali {cmd} --api browser` "
        "with the browser harness backed by `node`, against a program calling Math.asinh(0), "
        "Math.acosh(1) and Math.atanh(0) -- each printing `0`. "
    )

    cases = [
        {"name": "build_emits_math_inverse_hyperbolic_identity_literals",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_count": count_stdout},
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_math_inverse_hyperbolic_identity_literals",
         "rationale": bundle_prose + " This sibling asserts the JSON build envelope "
                      "(schemaVersion/command/success/exitCode and payload "
                      "artifactKind/bundleFormat) instead of plain text. The source makes no "
                      "`errors` claim on this envelope, so none is written.",
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_count": count_stdout},
                               json_output=True,
                               json_claims=envelope_build(errors=False),
                               meta_fields=META)},
    ]

    for command, fname in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"{command}_supports_math_inverse_hyperbolic_identity_literals_"
                    "when_browser_harness_is_configured",
            "rationale": harness_prose.format(cmd=command) +
                         MIRROR_COUNT.format(needle="0\n", n=3, key="stdout_count"),
            "steps": [harness_step(command, fname, json_output=False,
                                   asserts={"stdout_count": count_stdout})],
        })
    for command, fname in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"json_{command}_supports_math_inverse_hyperbolic_identity_literals_"
                    "when_browser_harness_is_configured",
            "rationale": harness_prose.format(cmd=command + " --output json") +
                         "The same count claim is taken here against the JSON string leaf "
                         "`json[\"stdout\"]` rather than raw stdout (:253), which is why "
                         "`json_count` exists alongside `stdout_count`. " +
                         MIRROR_COUNT.format(needle="0\n", n=3, key="json_count") +
                         " No equality pin is written for `json.stdout`: the source asserts "
                         "only how many times `0\\n` occurs in it. `stderr` is asserted "
                         "exactly empty (:254); the source makes no `errors` claim on this "
                         "envelope, so none is written.",
            "steps": [harness_step(command, fname, json_output=True,
                                   json_claims=envelope_harness(command, stderr=True,
                                                                errors=False),
                                   asserts={"json_count": count_json})],
        })

    return ("math_asinh_acosh_atanh_identities.toml", header,
            {"ext": ["js", "ts", "jsx", "tsx"]},
            {"app.${ext}": bundle_src, "main.${ext}": run_src,
             "smoke.test.${ext}": test_src},
            cases)


def main(argv):
    names = argv or sorted(REGISTRY)
    for name in names:
        if name not in REGISTRY:
            raise SystemExit(f"unknown target {name!r}; known: {sorted(REGISTRY)}")
        out, header, matrix, source, cases = REGISTRY[name]()
        write(os.path.join(CASES, out), emit(header.split("\n"), matrix, source, cases))


if __name__ == "__main__":
    main(sys.argv[1:])
