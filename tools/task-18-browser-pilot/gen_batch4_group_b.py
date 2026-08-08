#!/usr/bin/env python3
"""Generate the batch 4 "group B" case files (5 math targets).

Own module rather than an append to gen_batch4.py: four implementers were
running concurrently against that one file, and a concurrent whole-file write
silently drops another agent's function. The scaffolding (REGISTRY / target /
rs / main) is copied from gen_batch4.py; the shared helpers (case_emit,
math_shapes, lexer, toml_emit) are imported, never edited.

Targets:
  math_hypot_global_this_root        14 fns -> 32 named siblings, no matrix
  math_floor_trunc_ceil_aliases      17 fns -> 16 migrated, 4 cases x ext(4)
  math_floor_trunc_ceil_bundle        9 fns ->  8 migrated, 2 cases x ext(4)
  math_expm1_log1p_identities        12 fns -> 12 named siblings, no matrix
  math_inverse_trig_identities        4 fns ->  8 invocations, 4 cases x ext(2)

Run: python3 gen_batch4_group_b.py [name ...]   (no args = all)
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import fixture_in_fn, fixture_starting, emit, write  # noqa: E402
from math_shapes import (  # noqa: E402
    rule12_no_comments_prose,  # noqa: E402
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


# ---------------------------------------------------------------------------
# RULE 8 / RULE 9 -- the two floor/trunc/ceil files build their fixtures with
# `format!` over `kali_common::math_floor_trunc_ceil_frozen_callable_*`, so
# their resolved program text exists in no string literal anywhere in the .rs
# and CANNOT be pulled out with fixture_in_fn. Rule 8 forbids hand-applying
# Rust's `{}` substitution and `{{`/`}}` brace collapse. The three literals
# below are therefore the byte-exact OUTPUT OF THE REAL CODE, captured by a
# temporary test target that did
#
#     mod aliases { include!("browser_math_floor_trunc_ceil_aliases.rs");
#                   #[test] fn dump() { fs::write(..., run_source()); ... } }
#     mod bundle  { include!("browser_math_floor_trunc_ceil_bundle.rs"); ... }
#
# and was run with `cargo test -p kali_cli --test <tmp> -- zz_dump`. `include!`
# rather than a retyped copy of the builders, so the executed `format!` is
# literally the one in the shipped source. The temp target was deleted after
# capture. To re-derive: recreate that file and re-run it; the constants below
# must come back byte-identical.
#
# They are embedded here (rather than loaded from a dump file) so this module
# runs from a clean checkout with no uncommitted inputs -- the defect that got
# the pilot's per-file generators deleted (see README).
# ---------------------------------------------------------------------------

FIXT_ALIASES_RUN = 'const value = 1.6; const alias = value; console.log(Math.floor(alias)); console.log(Math.trunc(alias)); console.log(Math.ceil(alias)); console.log(Object.freeze(globalThis.Math["floor"])(alias)); console.log(Object.freeze((globalThis.Math["floor"]))(alias)); console.log(Object.freeze(globalThis.Math[\'floor\'])(alias)); console.log(Object.freeze((globalThis.Math[\'floor\']))(alias)); console.log(Object.freeze(globalThis.Math.floor)(alias)); console.log(Object.freeze((globalThis.Math.floor))(alias)); console.log(Object.freeze(globalThis["Math"]["floor"])(alias)); console.log(Object.freeze((globalThis["Math"]["floor"]))(alias)); console.log(Object.freeze((globalThis["Math"]))["floor"](alias)); console.log(Object.freeze((globalThis["Math"]))[\'floor\'](alias)); console.log(Object.freeze((globalThis.Math))["floor"](alias)); console.log(Object.freeze((globalThis.Math))[\'floor\'](alias)); console.log(Object.freeze((globalThis[\'Math\']))["floor"](alias)); console.log(Object.freeze((globalThis[\'Math\']))[\'floor\'](alias)); console.log(Object.freeze(globalThis["Math"][\'floor\'])(alias)); console.log(Object.freeze((globalThis["Math"][\'floor\']))(alias)); console.log(Object.freeze(globalThis["Math"].floor)(alias)); console.log(Object.freeze((globalThis["Math"])["floor"])(alias)); console.log(Object.freeze((globalThis[\'Math\'])[\'floor\'])(alias)); console.log(Object.freeze(globalThis[\'Math\'].floor)(alias)); console.log(Object.freeze((globalThis[\'Math\']).floor)(alias)); console.log(Object.freeze((globalThis["Math"]).floor)(alias)); console.log(Object.freeze((globalThis["Math"].floor))(alias)); console.log(Object.freeze(Math["floor"])(alias)); console.log(Object.freeze((Math["floor"]))(alias)); console.log(Object.freeze(Math[\'floor\'])(alias)); console.log(Object.freeze((Math[\'floor\']))(alias)); console.log(Object.freeze(globalThis.Math["trunc"])(alias)); console.log(Object.freeze((globalThis.Math["trunc"]))(alias)); console.log(Object.freeze(globalThis.Math[\'trunc\'])(alias)); console.log(Object.freeze((globalThis.Math[\'trunc\']))(alias)); console.log(Object.freeze(globalThis.Math.trunc)(alias)); console.log(Object.freeze((globalThis.Math.trunc))(alias)); console.log(Object.freeze(globalThis["Math"]["trunc"])(alias)); console.log(Object.freeze((globalThis["Math"]["trunc"]))(alias)); console.log(Object.freeze((globalThis["Math"]))["trunc"](alias)); console.log(Object.freeze((globalThis["Math"]))[\'trunc\'](alias)); console.log(Object.freeze((globalThis.Math))["trunc"](alias)); console.log(Object.freeze((globalThis.Math))[\'trunc\'](alias)); console.log(Object.freeze((globalThis[\'Math\']))["trunc"](alias)); console.log(Object.freeze((globalThis[\'Math\']))[\'trunc\'](alias)); console.log(Object.freeze(globalThis["Math"][\'trunc\'])(alias)); console.log(Object.freeze((globalThis["Math"][\'trunc\']))(alias)); console.log(Object.freeze(globalThis["Math"].trunc)(alias)); console.log(Object.freeze((globalThis["Math"])["trunc"])(alias)); console.log(Object.freeze((globalThis[\'Math\'])[\'trunc\'])(alias)); console.log(Object.freeze(globalThis[\'Math\'].trunc)(alias)); console.log(Object.freeze((globalThis[\'Math\']).trunc)(alias)); console.log(Object.freeze((globalThis["Math"]).trunc)(alias)); console.log(Object.freeze((globalThis["Math"].trunc))(alias)); console.log(Object.freeze(Math["trunc"])(alias)); console.log(Object.freeze((Math["trunc"]))(alias)); console.log(Object.freeze(Math[\'trunc\'])(alias)); console.log(Object.freeze((Math[\'trunc\']))(alias)); console.log(Object.freeze(globalThis.Math["ceil"])(alias)); console.log(Object.freeze((globalThis.Math["ceil"]))(alias)); console.log(Object.freeze(globalThis.Math[\'ceil\'])(alias)); console.log(Object.freeze((globalThis.Math[\'ceil\']))(alias)); console.log(Object.freeze(globalThis.Math.ceil)(alias)); console.log(Object.freeze((globalThis.Math.ceil))(alias)); console.log(Object.freeze(globalThis["Math"]["ceil"])(alias)); console.log(Object.freeze((globalThis["Math"]["ceil"]))(alias)); console.log(Object.freeze((globalThis["Math"]))["ceil"](alias)); console.log(Object.freeze((globalThis["Math"]))[\'ceil\'](alias)); console.log(Object.freeze((globalThis.Math))["ceil"](alias)); console.log(Object.freeze((globalThis.Math))[\'ceil\'](alias)); console.log(Object.freeze((globalThis[\'Math\']))["ceil"](alias)); console.log(Object.freeze((globalThis[\'Math\']))[\'ceil\'](alias)); console.log(Object.freeze(globalThis["Math"][\'ceil\'])(alias)); console.log(Object.freeze((globalThis["Math"][\'ceil\']))(alias)); console.log(Object.freeze(globalThis["Math"].ceil)(alias)); console.log(Object.freeze((globalThis["Math"])["ceil"])(alias)); console.log(Object.freeze((globalThis[\'Math\'])[\'ceil\'])(alias)); console.log(Object.freeze(globalThis[\'Math\'].ceil)(alias)); console.log(Object.freeze((globalThis[\'Math\']).ceil)(alias)); console.log(Object.freeze((globalThis["Math"]).ceil)(alias)); console.log(Object.freeze((globalThis["Math"].ceil))(alias)); console.log(Object.freeze(Math["ceil"])(alias)); console.log(Object.freeze((Math["ceil"]))(alias)); console.log(Object.freeze(Math[\'ceil\'])(alias)); console.log(Object.freeze((Math[\'ceil\']))(alias));\n'
FIXT_ALIASES_TEST = 'Kali.test(\'math floor trunc ceil identities\', () => {\n  const value = 1.6;\n  const alias = value;\n  console.log(Math.floor(alias));\n  console.log(Math.trunc(alias));\n  console.log(Math.ceil(alias));\n  console.log(Object.freeze(globalThis.Math["floor"])(alias)); console.log(Object.freeze((globalThis.Math["floor"]))(alias)); console.log(Object.freeze(globalThis.Math[\'floor\'])(alias)); console.log(Object.freeze((globalThis.Math[\'floor\']))(alias)); console.log(Object.freeze(globalThis.Math.floor)(alias)); console.log(Object.freeze((globalThis.Math.floor))(alias)); console.log(Object.freeze(globalThis["Math"]["floor"])(alias)); console.log(Object.freeze((globalThis["Math"]["floor"]))(alias)); console.log(Object.freeze((globalThis["Math"]))["floor"](alias)); console.log(Object.freeze((globalThis["Math"]))[\'floor\'](alias)); console.log(Object.freeze((globalThis.Math))["floor"](alias)); console.log(Object.freeze((globalThis.Math))[\'floor\'](alias)); console.log(Object.freeze((globalThis[\'Math\']))["floor"](alias)); console.log(Object.freeze((globalThis[\'Math\']))[\'floor\'](alias)); console.log(Object.freeze(globalThis["Math"][\'floor\'])(alias)); console.log(Object.freeze((globalThis["Math"][\'floor\']))(alias)); console.log(Object.freeze(globalThis["Math"].floor)(alias)); console.log(Object.freeze((globalThis["Math"])["floor"])(alias)); console.log(Object.freeze((globalThis[\'Math\'])[\'floor\'])(alias)); console.log(Object.freeze(globalThis[\'Math\'].floor)(alias)); console.log(Object.freeze((globalThis[\'Math\']).floor)(alias)); console.log(Object.freeze((globalThis["Math"]).floor)(alias)); console.log(Object.freeze((globalThis["Math"].floor))(alias)); console.log(Object.freeze(Math["floor"])(alias)); console.log(Object.freeze((Math["floor"]))(alias)); console.log(Object.freeze(Math[\'floor\'])(alias)); console.log(Object.freeze((Math[\'floor\']))(alias)); console.log(Object.freeze(globalThis.Math["trunc"])(alias)); console.log(Object.freeze((globalThis.Math["trunc"]))(alias)); console.log(Object.freeze(globalThis.Math[\'trunc\'])(alias)); console.log(Object.freeze((globalThis.Math[\'trunc\']))(alias)); console.log(Object.freeze(globalThis.Math.trunc)(alias)); console.log(Object.freeze((globalThis.Math.trunc))(alias)); console.log(Object.freeze(globalThis["Math"]["trunc"])(alias)); console.log(Object.freeze((globalThis["Math"]["trunc"]))(alias)); console.log(Object.freeze((globalThis["Math"]))["trunc"](alias)); console.log(Object.freeze((globalThis["Math"]))[\'trunc\'](alias)); console.log(Object.freeze((globalThis.Math))["trunc"](alias)); console.log(Object.freeze((globalThis.Math))[\'trunc\'](alias)); console.log(Object.freeze((globalThis[\'Math\']))["trunc"](alias)); console.log(Object.freeze((globalThis[\'Math\']))[\'trunc\'](alias)); console.log(Object.freeze(globalThis["Math"][\'trunc\'])(alias)); console.log(Object.freeze((globalThis["Math"][\'trunc\']))(alias)); console.log(Object.freeze(globalThis["Math"].trunc)(alias)); console.log(Object.freeze((globalThis["Math"])["trunc"])(alias)); console.log(Object.freeze((globalThis[\'Math\'])[\'trunc\'])(alias)); console.log(Object.freeze(globalThis[\'Math\'].trunc)(alias)); console.log(Object.freeze((globalThis[\'Math\']).trunc)(alias)); console.log(Object.freeze((globalThis["Math"]).trunc)(alias)); console.log(Object.freeze((globalThis["Math"].trunc))(alias)); console.log(Object.freeze(Math["trunc"])(alias)); console.log(Object.freeze((Math["trunc"]))(alias)); console.log(Object.freeze(Math[\'trunc\'])(alias)); console.log(Object.freeze((Math[\'trunc\']))(alias)); console.log(Object.freeze(globalThis.Math["ceil"])(alias)); console.log(Object.freeze((globalThis.Math["ceil"]))(alias)); console.log(Object.freeze(globalThis.Math[\'ceil\'])(alias)); console.log(Object.freeze((globalThis.Math[\'ceil\']))(alias)); console.log(Object.freeze(globalThis.Math.ceil)(alias)); console.log(Object.freeze((globalThis.Math.ceil))(alias)); console.log(Object.freeze(globalThis["Math"]["ceil"])(alias)); console.log(Object.freeze((globalThis["Math"]["ceil"]))(alias)); console.log(Object.freeze((globalThis["Math"]))["ceil"](alias)); console.log(Object.freeze((globalThis["Math"]))[\'ceil\'](alias)); console.log(Object.freeze((globalThis.Math))["ceil"](alias)); console.log(Object.freeze((globalThis.Math))[\'ceil\'](alias)); console.log(Object.freeze((globalThis[\'Math\']))["ceil"](alias)); console.log(Object.freeze((globalThis[\'Math\']))[\'ceil\'](alias)); console.log(Object.freeze(globalThis["Math"][\'ceil\'])(alias)); console.log(Object.freeze((globalThis["Math"][\'ceil\']))(alias)); console.log(Object.freeze(globalThis["Math"].ceil)(alias)); console.log(Object.freeze((globalThis["Math"])["ceil"])(alias)); console.log(Object.freeze((globalThis[\'Math\'])[\'ceil\'])(alias)); console.log(Object.freeze(globalThis[\'Math\'].ceil)(alias)); console.log(Object.freeze((globalThis[\'Math\']).ceil)(alias)); console.log(Object.freeze((globalThis["Math"]).ceil)(alias)); console.log(Object.freeze((globalThis["Math"].ceil))(alias)); console.log(Object.freeze(Math["ceil"])(alias)); console.log(Object.freeze((Math["ceil"]))(alias)); console.log(Object.freeze(Math[\'ceil\'])(alias)); console.log(Object.freeze((Math[\'ceil\']))(alias));\n});\n'
FIXT_BUNDLE_SRC = '// kali-tree-shake: mathFloorTruncCeilAliasChain\nfunction mathFloorTruncCeilAliasChain() {\n  const value = 1.6;\n  const alias = value;\n  console.log(Math.floor(alias));\n  console.log(Math.trunc(alias));\n  console.log(Math.ceil(alias));\n  console.log(Object.freeze(globalThis.Math["floor"])(alias));\n  console.log(Object.freeze((globalThis.Math["floor"]))(alias));\n  console.log(Object.freeze(globalThis.Math[\'floor\'])(alias));\n  console.log(Object.freeze((globalThis.Math[\'floor\']))(alias));\n  console.log(Object.freeze(globalThis.Math.floor)(alias));\n  console.log(Object.freeze((globalThis.Math.floor))(alias));\n  console.log(Object.freeze(globalThis["Math"]["floor"])(alias));\n  console.log(Object.freeze((globalThis["Math"]["floor"]))(alias));\n  console.log(Object.freeze((globalThis["Math"]))["floor"](alias));\n  console.log(Object.freeze((globalThis["Math"]))[\'floor\'](alias));\n  console.log(Object.freeze((globalThis.Math))["floor"](alias));\n  console.log(Object.freeze((globalThis.Math))[\'floor\'](alias));\n  console.log(Object.freeze((globalThis[\'Math\']))["floor"](alias));\n  console.log(Object.freeze((globalThis[\'Math\']))[\'floor\'](alias));\n  console.log(Object.freeze(globalThis["Math"][\'floor\'])(alias));\n  console.log(Object.freeze((globalThis["Math"][\'floor\']))(alias));\n  console.log(Object.freeze(globalThis["Math"].floor)(alias));\n  console.log(Object.freeze((globalThis["Math"])["floor"])(alias));\n  console.log(Object.freeze((globalThis[\'Math\'])[\'floor\'])(alias));\n  console.log(Object.freeze(globalThis[\'Math\'].floor)(alias));\n  console.log(Object.freeze((globalThis[\'Math\']).floor)(alias));\n  console.log(Object.freeze((globalThis["Math"]).floor)(alias));\n  console.log(Object.freeze((globalThis["Math"].floor))(alias));\n  console.log(Object.freeze(Math["floor"])(alias));\n  console.log(Object.freeze((Math["floor"]))(alias));\n  console.log(Object.freeze(Math[\'floor\'])(alias));\n  console.log(Object.freeze((Math[\'floor\']))(alias));\n  console.log(Object.freeze(globalThis.Math["trunc"])(alias));\n  console.log(Object.freeze((globalThis.Math["trunc"]))(alias));\n  console.log(Object.freeze(globalThis.Math[\'trunc\'])(alias));\n  console.log(Object.freeze((globalThis.Math[\'trunc\']))(alias));\n  console.log(Object.freeze(globalThis.Math.trunc)(alias));\n  console.log(Object.freeze((globalThis.Math.trunc))(alias));\n  console.log(Object.freeze(globalThis["Math"]["trunc"])(alias));\n  console.log(Object.freeze((globalThis["Math"]["trunc"]))(alias));\n  console.log(Object.freeze((globalThis["Math"]))["trunc"](alias));\n  console.log(Object.freeze((globalThis["Math"]))[\'trunc\'](alias));\n  console.log(Object.freeze((globalThis.Math))["trunc"](alias));\n  console.log(Object.freeze((globalThis.Math))[\'trunc\'](alias));\n  console.log(Object.freeze((globalThis[\'Math\']))["trunc"](alias));\n  console.log(Object.freeze((globalThis[\'Math\']))[\'trunc\'](alias));\n  console.log(Object.freeze(globalThis["Math"][\'trunc\'])(alias));\n  console.log(Object.freeze((globalThis["Math"][\'trunc\']))(alias));\n  console.log(Object.freeze(globalThis["Math"].trunc)(alias));\n  console.log(Object.freeze((globalThis["Math"])["trunc"])(alias));\n  console.log(Object.freeze((globalThis[\'Math\'])[\'trunc\'])(alias));\n  console.log(Object.freeze(globalThis[\'Math\'].trunc)(alias));\n  console.log(Object.freeze((globalThis[\'Math\']).trunc)(alias));\n  console.log(Object.freeze((globalThis["Math"]).trunc)(alias));\n  console.log(Object.freeze((globalThis["Math"].trunc))(alias));\n  console.log(Object.freeze(Math["trunc"])(alias));\n  console.log(Object.freeze((Math["trunc"]))(alias));\n  console.log(Object.freeze(Math[\'trunc\'])(alias));\n  console.log(Object.freeze((Math[\'trunc\']))(alias));\n  console.log(Object.freeze(globalThis.Math["ceil"])(alias));\n  console.log(Object.freeze((globalThis.Math["ceil"]))(alias));\n  console.log(Object.freeze(globalThis.Math[\'ceil\'])(alias));\n  console.log(Object.freeze((globalThis.Math[\'ceil\']))(alias));\n  console.log(Object.freeze(globalThis.Math.ceil)(alias));\n  console.log(Object.freeze((globalThis.Math.ceil))(alias));\n  console.log(Object.freeze(globalThis["Math"]["ceil"])(alias));\n  console.log(Object.freeze((globalThis["Math"]["ceil"]))(alias));\n  console.log(Object.freeze((globalThis["Math"]))["ceil"](alias));\n  console.log(Object.freeze((globalThis["Math"]))[\'ceil\'](alias));\n  console.log(Object.freeze((globalThis.Math))["ceil"](alias));\n  console.log(Object.freeze((globalThis.Math))[\'ceil\'](alias));\n  console.log(Object.freeze((globalThis[\'Math\']))["ceil"](alias));\n  console.log(Object.freeze((globalThis[\'Math\']))[\'ceil\'](alias));\n  console.log(Object.freeze(globalThis["Math"][\'ceil\'])(alias));\n  console.log(Object.freeze((globalThis["Math"][\'ceil\']))(alias));\n  console.log(Object.freeze(globalThis["Math"].ceil)(alias));\n  console.log(Object.freeze((globalThis["Math"])["ceil"])(alias));\n  console.log(Object.freeze((globalThis[\'Math\'])[\'ceil\'])(alias));\n  console.log(Object.freeze(globalThis[\'Math\'].ceil)(alias));\n  console.log(Object.freeze((globalThis[\'Math\']).ceil)(alias));\n  console.log(Object.freeze((globalThis["Math"]).ceil)(alias));\n  console.log(Object.freeze((globalThis["Math"].ceil))(alias));\n  console.log(Object.freeze(Math["ceil"])(alias));\n  console.log(Object.freeze((Math["ceil"]))(alias));\n  console.log(Object.freeze(Math[\'ceil\'])(alias));\n  console.log(Object.freeze((Math[\'ceil\']))(alias));\n  return [Math.floor(alias), Math.trunc(alias), Math.ceil(alias), Object.freeze(globalThis.Math["floor"])(alias), Object.freeze((globalThis.Math["floor"]))(alias), Object.freeze(globalThis.Math[\'floor\'])(alias), Object.freeze((globalThis.Math[\'floor\']))(alias), Object.freeze(globalThis.Math.floor)(alias), Object.freeze((globalThis.Math.floor))(alias), Object.freeze(globalThis["Math"]["floor"])(alias), Object.freeze((globalThis["Math"]["floor"]))(alias), Object.freeze((globalThis["Math"]))["floor"](alias), Object.freeze((globalThis["Math"]))[\'floor\'](alias), Object.freeze((globalThis.Math))["floor"](alias), Object.freeze((globalThis.Math))[\'floor\'](alias), Object.freeze((globalThis[\'Math\']))["floor"](alias), Object.freeze((globalThis[\'Math\']))[\'floor\'](alias), Object.freeze(globalThis["Math"][\'floor\'])(alias), Object.freeze((globalThis["Math"][\'floor\']))(alias), Object.freeze(globalThis["Math"].floor)(alias), Object.freeze((globalThis["Math"])["floor"])(alias), Object.freeze((globalThis[\'Math\'])[\'floor\'])(alias), Object.freeze(globalThis[\'Math\'].floor)(alias), Object.freeze((globalThis[\'Math\']).floor)(alias), Object.freeze((globalThis["Math"]).floor)(alias), Object.freeze((globalThis["Math"].floor))(alias), Object.freeze(Math["floor"])(alias), Object.freeze((Math["floor"]))(alias), Object.freeze(Math[\'floor\'])(alias), Object.freeze((Math[\'floor\']))(alias), Object.freeze(globalThis.Math["trunc"])(alias), Object.freeze((globalThis.Math["trunc"]))(alias), Object.freeze(globalThis.Math[\'trunc\'])(alias), Object.freeze((globalThis.Math[\'trunc\']))(alias), Object.freeze(globalThis.Math.trunc)(alias), Object.freeze((globalThis.Math.trunc))(alias), Object.freeze(globalThis["Math"]["trunc"])(alias), Object.freeze((globalThis["Math"]["trunc"]))(alias), Object.freeze((globalThis["Math"]))["trunc"](alias), Object.freeze((globalThis["Math"]))[\'trunc\'](alias), Object.freeze((globalThis.Math))["trunc"](alias), Object.freeze((globalThis.Math))[\'trunc\'](alias), Object.freeze((globalThis[\'Math\']))["trunc"](alias), Object.freeze((globalThis[\'Math\']))[\'trunc\'](alias), Object.freeze(globalThis["Math"][\'trunc\'])(alias), Object.freeze((globalThis["Math"][\'trunc\']))(alias), Object.freeze(globalThis["Math"].trunc)(alias), Object.freeze((globalThis["Math"])["trunc"])(alias), Object.freeze((globalThis[\'Math\'])[\'trunc\'])(alias), Object.freeze(globalThis[\'Math\'].trunc)(alias), Object.freeze((globalThis[\'Math\']).trunc)(alias), Object.freeze((globalThis["Math"]).trunc)(alias), Object.freeze((globalThis["Math"].trunc))(alias), Object.freeze(Math["trunc"])(alias), Object.freeze((Math["trunc"]))(alias), Object.freeze(Math[\'trunc\'])(alias), Object.freeze((Math[\'trunc\']))(alias), Object.freeze(globalThis.Math["ceil"])(alias), Object.freeze((globalThis.Math["ceil"]))(alias), Object.freeze(globalThis.Math[\'ceil\'])(alias), Object.freeze((globalThis.Math[\'ceil\']))(alias), Object.freeze(globalThis.Math.ceil)(alias), Object.freeze((globalThis.Math.ceil))(alias), Object.freeze(globalThis["Math"]["ceil"])(alias), Object.freeze((globalThis["Math"]["ceil"]))(alias), Object.freeze((globalThis["Math"]))["ceil"](alias), Object.freeze((globalThis["Math"]))[\'ceil\'](alias), Object.freeze((globalThis.Math))["ceil"](alias), Object.freeze((globalThis.Math))[\'ceil\'](alias), Object.freeze((globalThis[\'Math\']))["ceil"](alias), Object.freeze((globalThis[\'Math\']))[\'ceil\'](alias), Object.freeze(globalThis["Math"][\'ceil\'])(alias), Object.freeze((globalThis["Math"][\'ceil\']))(alias), Object.freeze(globalThis["Math"].ceil)(alias), Object.freeze((globalThis["Math"])["ceil"])(alias), Object.freeze((globalThis[\'Math\'])[\'ceil\'])(alias), Object.freeze(globalThis[\'Math\'].ceil)(alias), Object.freeze((globalThis[\'Math\']).ceil)(alias), Object.freeze((globalThis["Math"]).ceil)(alias), Object.freeze((globalThis["Math"].ceil))(alias), Object.freeze(Math["ceil"])(alias), Object.freeze((Math["ceil"]))(alias), Object.freeze(Math[\'ceil\'])(alias), Object.freeze((Math[\'ceil\']))(alias)];\n}\n'

# U9 -- live-captured from the real binary at .cache/cargo-target/debug/kali
# with node as the harness backend, via kali_run.py, and confirmed IDENTICAL
# for every extension on the file's matrix axis and for both `run` and `test`.
# Never hand-computed.
# The bundle harness `body` literal, likewise embedded rather than extracted:
# browser_math_floor_trunc_ceil_bundle.rs has since been TRIMMED to its one
# retained test (U4), so `assert_browser_bundle_math_floor_trunc_ceil_alias` no
# longer exists in the working tree and no content-anchored extractor can reach
# it. Captured from the PRE-TRIM source with case_emit's content-anchored
# fixture_starting (prefix "const mod = await import("), not retyped; the
# pre-trim file is in git history.
FIXT_BUNDLE_HARNESS_BODY = 'const mod = await import(bundleJs.href);\nawait mod.mathFloorTruncCeilAliasChain();\n'

PIN_ALIAS_JSON_STDOUT = '1\n1\n2\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n'
PIN_HYPOT_JSON_STDOUT = "5\n5\n5\n5\n"
PIN_EXPM1_JSON_STDOUT = "0\n0\n"
PIN_TRIG_JSON_STDOUT = "0\n0\n0\n"


# ---------------------------------------------------------------------------
# Shared prose fragments. Per U8, no bare lowercase_snake identifier is
# backticked here unless it is a real fn in the .rs being migrated; helper
# names from this tooling and from other crates are written unbacktioked or as
# whole phrases so check_rationale_fn_names.py can adjudicate them.
# ---------------------------------------------------------------------------

class _Rule12:
    """I2 fix round 1: was a fixed template asserting the source's only `//`
    was a `// kali-tree-shake:` marker inside a JS fixture. Two of this group's
    sources contain no `//` at all and declare no bundle fixture, so that
    sentence shipped false. Now derived from the source by the shared verifying
    helper, which also raises rather than emitting a false --allow-empty
    discharge when real Rust comments exist. `.format(stem=...)` kept so the
    call sites below are unchanged."""

    @staticmethod
    def format(stem, rs=None):
        return rule12_no_comments_prose(rs or os.path.join(TESTS, f"browser_{stem}.rs"), stem)


NO_RUST_COMMENTS = _Rule12()

MIRROR_CONTAINS = (
    "The source spells this as a plain `.contains(...)` against raw stdout, so it is carried "
    "as stdout_contains and NOT strengthened to an exact stdout pin -- controller ruling 3, "
    "mirror the source: a plain `.contains` against a field that HAS a substring form keeps "
    "the substring form even though the exact output was observed."
)

MIRROR_JSON_PIN = (
    "On the JSON branch the same claim is taken against the string leaf json[\"stdout\"], "
    "which has NO substring form in the case format (there is no json_contains key), so per "
    "controller ruling 3 it becomes an exact `json.stdout` pin -- and, per U9, only after the "
    "value was captured from the real kali binary rather than hand-computed."
)

RULE13_RUNNER_NOTE = """\
RULE 13 -- transitive helper docs. Every fn in each call chain was checked for
a `///` doc comment. None of this file's own helpers carries one. The chain
reaches kali_runtime_contract's browser_bundle_harness_script and
browser_harness_command_parts_for, which do carry one-line docs, but in the
migrated form this case file never calls them -- the browser_bundle_harness
step kind means the case RUNNER does (design spec 5.3), so those docs describe
shared runner infrastructure, not what this case claims. Every case file
shipped before this batch does the same."""


# ===========================================================================
# 1. browser_math_hypot_global_this_root.rs -- 14 fns, 32 invocations,
#    NO [matrix], 32 named siblings.
# ===========================================================================
@target("math_hypot_global_this_root")
def hypot():
    text = rs("math_hypot_global_this_root")
    bundle_src = fixture_in_fn(text, "browser_bundle_global_this_math_hypot_source")
    run_src = fixture_in_fn(text, "browser_harness_global_this_math_hypot_run_source")
    test_src = fixture_in_fn(text, "browser_harness_global_this_math_hypot_test_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_global_this_math_hypot",
        "const mod = await import(")

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '5\\n5\\n5\\n5\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
EXTRA-OK: 'program.js' -- U5-renamed [source] entry filename; passed on argv only, referenced by no fixture body (checked), so the rename cannot change the program
EXTRA-OK: 'program.ts' -- U5-renamed [source] entry filename; passed on argv only, referenced by no fixture body (checked), so the rename cannot change the program
Migrated from tests/browser_math_hypot_global_this_root.rs -- all 14 #[test]
fns, nothing retained.

{NO_RUST_COMMENTS.format(stem="math_hypot_global_this_root")}

RULE 7 -- INVOCATION ARITHMETIC, derived BY HAND because the mechanical
the enumerator undercounted this file WHEN THIS FILE WAS WRITTEN, and no
longer does. (HISTORICAL NOTE, fix round 1 / I1.) It reported 30 total against
a true figure of 32, because invocations() called CALL.search(body) and so saw
only the FIRST assert_* call in a #[test] body, while
`build_emits_global_this_math_hypot_perfect_square_slice_in_jsx_and_tsx_input`
has TWO calls inside its `for filename in ["app.jsx", "app.tsx"]` loop
(json_output false, then true) -- 4 invocations counted as 2. That bug was
found during this batch and fixed in the same commit that shipped this file;
the helper now uses CALL.finditer and the repaired tool reports TOTAL
INVOCATIONS: 32, agreeing with the hand count below. A separate, still-live
limitation: it does not bind the tuple loop variables in
`run_and_test_supports_global_this_math_hypot_perfect_square_slice_when_browser
_harness_is_configured_in_js_ts_jsx_and_tsx_input` (its TUPLE_ROW regex cannot
match a row containing a `source()` call), though it does get that fn's COUNT
right. Hand-derived, per helper, and now also confirmed by the tool:

  assert_browser_bundle_global_this_math_hypot -- 8 invocations
    4 single-call fns: (app.js,false) (app.ts,false) (app.js,true)
      (app.ts,true)
    1 loop fn x 2 filenames x 2 calls = 4: (app.jsx,false) (app.jsx,true)
      (app.tsx,false) (app.tsx,true)
  assert_browser_harness_global_this_math_hypot -- 24 invocations
    8 single-call fns: run/test x main.js/main.ts x json false/true
    1 loop fn: 8 (command, filename, source) rows x `for json_output in
      [false, true]` = 16
  TOTAL = 8 + 24 = 32.

[matrix] DECLINED for the whole file (rule 7 / U1). No axis is uniform:
  * `ext` -- the bundle helper covers js/ts/jsx/tsx evenly (8 = 4 x 2), and so
    does the loop fn, but the eight single-call harness fns cover only js and
    ts. A file-wide `ext` axis fans EVERY [[case]] with no per-case opt-out
    (crates/kali_case_runner/src/expand.rs), so those cases would be fanned to
    jsx/tsx cells no source fn performs -- rule 2's invented combination.
  * `command` and `json_output` change the assertion SHAPE, not a substituted
    string (JSON envelope vs text stdout; `exitCode` for run vs total/passed/
    failed for test), so neither is a legal axis either.
  32 invocations -> 32 named sibling [[case]] entries, one per real
  invocation, no folding.

RULE 6 -- FOUR DELIBERATE DUPLICATE PAIRS. The loop fn re-runs four
invocations the single-call fns already make: (run, main.js, false),
(run, main.ts, false), (run, main.js, true), (run, main.ts, true). They are
NOT deduplicated. Rule 6 keeps one [[case]] per source #[test] fn even when
two fns' invocations are literally identical, because the case is the only
remaining trace of the fn; the four loop-side siblings carry a
`run_and_test_supports_` prefix and say so in their own rationale.

U5 -- ONE [source] KEY RENAME, and why it was unavoidable. [source] is a flat
file-wide namespace, and this source binds the name `main.js` to TWO different
programs: the run fixture (in `run_supports_..._in_js_input` and in the loop
fn) and the TEST fixture (in `test_supports_global_this_math_hypot_perfect_
square_slice_when_browser_harness_is_configured_in_js_input` and its json
sibling, which run `kali test` against a file named main.js rather than the
smoke.test.js the loop fn uses). Same for main.ts. One key cannot hold two
bodies, so the four test-command-on-a-main-named-entry cases use
`program.js` / `program.ts` instead. This is safe under U5: the entry name
reaches kali only as an argv token, no fixture body references it by string,
and the property those four fns actually exercise -- running `kali test`
against an entry whose name is NOT `*.test.*` -- is preserved. Verified
against the real binary rather than assumed: `kali test --api browser` over
the same test fixture produces byte-identical stdout ("5\\n5\\n5\\n5\\nok 1\\n")
and byte-identical json.stdout for main.js, main.ts, program.js and
program.ts.

U2 -- [source] is file-wide, and that is safe here: all 14 fixtures are
written unconditionally into a fresh temp dir, none is behind an `if`, and no
case's point is a file's presence or absence. Every command names its entry on
argv.

U13 -- the bundle/run/test bodies are each repeated across four extension-
suffixed keys rather than hoisted into [constants]. Deliberate, following the
shipped no-matrix precedent in cases/browser/math_atan2_global_this_root.toml:
hoisting program text into [constants] moves it onto a surface
audit-case-migration.py DOES search (U13 records this counter-hazard itself),
so a future phantom claim could be satisfied by a fixture body, and it would
also defeat the rule-9 fixture gate, which reads [source] values directly.

ASSERTION SHAPE, mirrored and nothing more. The bundle helper's json branch
DOES assert `errors` is empty (:78-81) and so does the harness helper's
(:186); the harness json branch also asserts json["stderr"] == "" (:185). All
three are carried. Raw-stdout claims are plain `.contains("5\\n")` (:125,
:189) and stay stdout_contains. The harness argv passes --max-threads 0 and
--max-spawned-processes 0 (:149-152); the build argv does not."""

    bundle_prose_base = (
        "Migrated from browser_math_hypot_global_this_root.rs, "
        "`{fn}`. `assert_browser_bundle_global_this_math_hypot` builds a browser bundle "
        "(`kali build --bundle --api browser`), asserts the emitted app/app.meta.json "
        "metadata (apiSurface/artifactKind), then runs the bundle glue under the "
        "browser-bundle-harness contract. The bundled program calls "
        "globalThis.Math.hypot(3, 4) through four spellings -- dotted, "
        "bracketed-property, bracketed-object, and both bracketed -- each printing 5. "
        + MIRROR_CONTAINS
    )
    harness_prose_base = (
        "Migrated from browser_math_hypot_global_this_root.rs, "
        "`{fn}`. `assert_browser_harness_global_this_math_hypot` runs "
        "`kali {argv} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        "browser harness backed by node, against a program that calls "
        "globalThis.Math.hypot(3, 4) through four spellings, each printing 5. "
    )

    cases = []

    # -- bundle: the four single-call fns, then the loop fn's four ---------
    bundle_calls = [
        ("js", False, "build_emits_global_this_math_hypot_perfect_square_slice_in_js_input"),
        ("ts", False, "build_emits_global_this_math_hypot_perfect_square_slice_in_ts_input"),
        ("js", True, "json_build_emits_global_this_math_hypot_perfect_square_slice_in_js_input"),
        ("ts", True, "json_build_emits_global_this_math_hypot_perfect_square_slice_in_ts_input"),
    ]
    loop_fn_bundle = ("build_emits_global_this_math_hypot_perfect_square_slice_"
                      "in_jsx_and_tsx_input")
    for ext in ("jsx", "tsx"):
        bundle_calls.append((ext, False, loop_fn_bundle))
        bundle_calls.append((ext, True, loop_fn_bundle))

    for ext, is_json, fn_name in bundle_calls:
        prefix = "json_" if is_json else ""
        looped = fn_name == loop_fn_bundle
        name = f"{prefix}build_emits_global_this_math_hypot_perfect_square_slice_in_{ext}_input"
        rationale = bundle_prose_base.format(fn=fn_name)
        if is_json:
            rationale += (
                " This sibling asserts the JSON build envelope "
                "(schemaVersion/command/success/exitCode, payload artifactKind/bundleFormat) "
                "instead of plain text, and asserts `errors` is the empty array (:78-81)."
            )
        if looped:
            rationale += (
                " Source-side this invocation comes from that fn's "
                "`for filename in [\"app.jsx\", \"app.tsx\"]` loop, whose body makes TWO calls "
                "(json_output false then true); it is split into its own named sibling per "
                "rule 5 rather than folded."
            )
        cases.append({
            "name": name,
            "rationale": rationale,
            "steps": bundle_steps(
                f"app.{ext}", harness_body, {"stdout_contains": ["5\n"]},
                json_output=is_json,
                json_claims=envelope_build(errors=True) if is_json else None,
                meta_fields=META),
        })

    # -- harness: eight single-call fns ------------------------------------
    single = [
        ("run", "js", False, "main.js",
         "run_supports_global_this_math_hypot_perfect_square_slice_when_browser_harness_"
         "is_configured_in_js_input"),
        ("run", "ts", False, "main.ts",
         "run_supports_global_this_math_hypot_perfect_square_slice_when_browser_harness_"
         "is_configured_in_ts_input"),
        ("test", "js", False, "program.js",
         "test_supports_global_this_math_hypot_perfect_square_slice_when_browser_harness_"
         "is_configured_in_js_input"),
        ("test", "ts", False, "program.ts",
         "test_supports_global_this_math_hypot_perfect_square_slice_when_browser_harness_"
         "is_configured_in_ts_input"),
        ("run", "js", True, "main.js",
         "run_supports_global_this_math_hypot_perfect_square_slice_when_browser_harness_"
         "is_configured_in_json_js_input"),
        ("run", "ts", True, "main.ts",
         "run_supports_global_this_math_hypot_perfect_square_slice_when_browser_harness_"
         "is_configured_in_json_ts_input"),
        ("test", "js", True, "program.js",
         "test_supports_global_this_math_hypot_perfect_square_slice_when_browser_harness_"
         "is_configured_in_json_js_input"),
        ("test", "ts", True, "program.ts",
         "test_supports_global_this_math_hypot_perfect_square_slice_when_browser_harness_"
         "is_configured_in_json_ts_input"),
    ]
    for command, ext, is_json, entry, fn_name in single:
        cases.append({
            "name": fn_name,
            "rationale": _harness_rationale(harness_prose_base, fn_name, command, is_json,
                                            entry, renamed=entry.startswith("program."),
                                            looped=False),
            "steps": [harness_step(
                command, entry, json_output=is_json,
                json_claims=_hypot_envelope(command) if is_json else None,
                asserts={} if is_json else {"stdout_contains": ["5\n"]},
                thread_flags=True)],
        })

    # -- harness: the loop fn's sixteen ------------------------------------
    loop_fn_harness = ("run_and_test_supports_global_this_math_hypot_perfect_square_slice_"
                       "when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input")
    loop_rows = [("run", "main.js", "js"), ("test", "smoke.test.js", "js"),
                 ("run", "main.ts", "ts"), ("test", "smoke.test.ts", "ts"),
                 ("run", "main.jsx", "jsx"), ("test", "smoke.test.jsx", "jsx"),
                 ("run", "main.tsx", "tsx"), ("test", "smoke.test.tsx", "tsx")]
    for command, entry, ext in loop_rows:
        for is_json in (False, True):
            prefix = "json_" if is_json else ""
            cases.append({
                "name": f"run_and_test_supports_global_this_math_hypot_perfect_square_slice_"
                        f"when_browser_harness_is_configured_{prefix}{command}_{ext}_input",
                "rationale": _harness_rationale(harness_prose_base, loop_fn_harness, command,
                                                is_json, entry, renamed=False, looped=True),
                "steps": [harness_step(
                    command, entry, json_output=is_json,
                    json_claims=_hypot_envelope(command) if is_json else None,
                    asserts={} if is_json else {"stdout_contains": ["5\n"]},
                    thread_flags=True)],
            })

    source = {}
    for ext in ("js", "ts", "jsx", "tsx"):
        source[f"app.{ext}"] = bundle_src
    for ext in ("js", "ts", "jsx", "tsx"):
        source[f"main.{ext}"] = run_src
    for ext in ("js", "ts", "jsx", "tsx"):
        source[f"smoke.test.{ext}"] = test_src
    for ext in ("js", "ts"):
        source[f"program.{ext}"] = test_src

    return ("math_hypot_global_this_root.toml", header, None, source, cases)


def _hypot_envelope(command):
    j = envelope_harness(command, stderr=True, errors=True)
    j["stdout"] = PIN_HYPOT_JSON_STDOUT
    return j


def _harness_rationale(base, fn_name, command, is_json, entry, *, renamed, looped):
    argv = f"{command} --output json" if is_json else command
    out = base.format(fn=fn_name, argv=argv)
    if is_json:
        out += (
            "This sibling asserts the JSON envelope: schemaVersion/command/success, payload "
            "hostContract/runtimeBackend, and "
            + ("`exitCode` at both the envelope and the payload level (run)"
               if command == "run" else
               "payload total/passed/failed (test)")
            + ". It also asserts stderr is exactly empty (:185) and `errors` is the empty "
              "array (:186). " + MIRROR_JSON_PIN
        )
    else:
        out += MIRROR_CONTAINS
    if renamed:
        out += (
            " NOTE (U5): the source writes this entry as " + entry.replace("program", "main")
            + ", but [source] is one flat file-wide namespace and that name is already bound "
            "to the run fixture by the sibling run_supports_ cases, so the key is renamed to "
            + entry + " here. The name reaches kali only as an argv token, no fixture body "
            "references it, the not-a-*.test.*-name property the source exercised is kept, "
            "and the rename was confirmed output-identical against the real binary."
        )
    if looped:
        out += (
            " Source-side this invocation is one cell of that fn's nested loops: eight "
            "(command, filename, source) rows x `for json_output in [false, true]`. It is "
            "split into its own named sibling per rule 5. Four of the sixteen cells duplicate "
            "invocations the single-call run_supports_ fns already make; per rule 6 they are "
            "kept as separate cases anyway, because a [[case]] is the only remaining trace of "
            "the source fn that produced it."
        )
    return out


# ===========================================================================
# 2. browser_math_floor_trunc_ceil_aliases.rs -- 17 fns, ONE RETAINED.
# ===========================================================================
@target("math_floor_trunc_ceil_aliases")
def floor_trunc_ceil_aliases():
    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '1\\n1\\n2\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n1\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_floor_trunc_ceil_aliases.rs.

PARTIAL MIGRATION (U4 trim-and-keep) -- 16 of the file's 17 #[test] fns are
migrated here. The 17th,
`browser_harness_math_floor_trunc_ceil_source_includes_full_frozen_callable_
inventory`, is a FIXTURE SELF-INSPECTION test: it takes
`browser_harness_math_floor_trunc_ceil_run_source()` and asserts
`source.contains(expected)` for every `expected` in a runtime-computed
inventory returned by kali_common's math_floor_trunc_ceil_frozen_callable_
aliases. It builds no command, runs no binary, and asserts nothing about
behaviour -- it checks the fixture's own text against a list that exists only
at runtime. That claim has no expressible form in the case format (there is no
step kind that asserts about [source] text; [source] is program text by
construction, not a claim), and it is invisible to
audit-case-migration.py, whose `.contains()` extractor cannot tell a
fixture-text read from an output assertion and which excludes everything under
[source] from its search by design. Migrating it would produce a false green.
It is escalated per rule 3/4 and RETAINED hand-written; controller ruling 4 is
explicit that the audit script is NOT extended for this shape. No other fn in
this file reaches that construct -- the other 16 all route through
`assert_browser_harness_math_floor_trunc_ceil`, which never reads fixture text
-- so U4's trim-and-keep applies and exactly one test is retained.

THE .rs HAS SINCE BEEN TRIMMED to exactly that one test plus the two fixture
builders it reads, carrying a `//!` retention header, on the same pattern as
cases/browser/math_atan2_global_this_root.toml's source. TWO CONSEQUENCES A
LATER READER MUST NOT MISREAD:
  * EVERY `:N` LINE CITATION IN THIS FILE IS A PRE-TRIM LINE NUMBER. Audit and
    diff this pair against the pre-trim source (git history), not against the
    working tree.
  * Running audit-case-migration.py on the POST-trim pair reports the retained
    fn's own needles as absent -- they live in the retained test and by ruling
    4 have no home here. Measured, not assumed. That is the documented
    escalation, not a drop; against the PRE-TRIM source the audit exits 0.

{NO_RUST_COMMENTS.format(stem="math_floor_trunc_ceil_aliases")}
(The `//!` retention header added to the .rs is migration bookkeeping about the
retained test, not prose about behaviour under test, so it is not carried into
any rationale; comment_coverage will list its lines as missing for exactly
that reason.)

RULE 8 / RULE 9 -- both fixtures are built by `format!` over
kali_common::math_floor_trunc_ceil_frozen_callable_invocation_lines, so their
resolved text appears in NO string literal in the .rs. The bodies in [source]
below are the byte-exact output of executing the real builders (a temporary
test target that `include!`d the .rs and dumped them), never a hand-applied
`{{}}` substitution. See this generator's header for the exact procedure.

RULE 7 / U1 -- MATRIX ARITHMETIC, closes exactly, over the migrated 16.
Enumerated with enumerate_invocations.py: 16 fns, 16 single invocations, no
loops anywhere (the helper has no internal loop either -- checked by reading
it). Coverage is a complete cross product:
  command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true) = 16.
`ext` is the axis every case varies over uniformly and completely, so:
4 [[case]] x ext(4) = 16 trials = the 16 migrated #[test] fns. Per rule 6 the
fold is stated here -- each [[case]] corresponds to 4 source fns, one per
cell, and the assertion mapping stays 1:1 per trial. `command` and
`json_output` are NOT axes: each changes the assertion SHAPE (JSON envelope vs
text stdout; `exitCode` for run vs total/passed/failed for test), which is
excluded from a matrix.

U2 -- [source] is file-wide and that is safe here: both fixtures are written
unconditionally into a fresh temp dir, neither is behind an `if`, and no
case's point is a file's presence or absence.

{RULE13_RUNNER_NOTE}
The chain DOES reach documented library helpers in another crate --
kali_common's math_floor_trunc_ceil_frozen_callable_invocation_lines and
math_floor_trunc_ceil_frozen_callable_aliases -- which produce the fixture
text. Rule 13 states no "purely descriptive" exemption and explicitly includes
kali_common, so both `///` docs are carried verbatim into every rationale.

ASSERTION SHAPE, mirrored and nothing more. This source asserts NO `errors`
array on the harness envelope (:78-96 -- compare its sibling
browser_math_hypot_global_this_root.rs, which does), so no `errors` claim is
written. The json branch DOES assert json["stderr"] == "" (:96). Raw-stdout
claims are two plain `.contains` calls (:99, :100) and stay stdout_contains.
The source passes NO --max-threads / --max-spawned-processes, so neither
appears on argv -- again unlike the hypot sibling."""

    doc_prose = (
        "RULE 13 -- doc comments on the library helpers that produce this fixture, carried "
        "verbatim: \"Canonical `console.log(...)` invocation lines for the supported "
        "`Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.\" and \"Canonical "
        "frozen callable aliases for the supported `Math.floor` / `Math.trunc` / `Math.ceil` "
        "helper slice.\""
    )
    base = (
        "Migrated from browser_math_floor_trunc_ceil_aliases.rs, the four "
        "`{fnfamily}` fns (one per extension). "
        "`assert_browser_harness_math_floor_trunc_ceil` runs `kali {argv} --api browser` with "
        "the browser harness backed by node, against a program that computes "
        "Math.floor(1.6), Math.trunc(1.6) and Math.ceil(1.6) and then re-invokes every frozen "
        "callable alias in the canonical floor/trunc/ceil inventory, so 1 and 2 are both "
        "printed many times. "
    )

    def cse(name, fnfamily, command, entry, is_json):
        prose = base.format(fnfamily=fnfamily, argv=(command + " --output json") if is_json
                            else command)
        if is_json:
            prose += (
                "This sibling asserts the JSON envelope: schemaVersion/command/success, "
                "payload hostContract/runtimeBackend, and "
                + ("`exitCode` at both the envelope and the payload level (run)"
                   if command == "run" else "payload total/passed/failed (test)")
                + ", plus stderr exactly empty (:96). The source makes NO `errors` claim on "
                  "this envelope, so none is written. Its two stdout claims here are "
                  "`stdout.contains(\"1\\n\")` and `stdout.contains(\"2\\n\")` taken against "
                  "json[\"stdout\"] (:93-95). " + MIRROR_JSON_PIN + " "
            )
        else:
            prose += (
                "Its two stdout claims here are `stdout.contains(\"1\\n\")` and "
                "`stdout.contains(\"2\\n\")` against raw stdout (:99-100). "
                + MIRROR_CONTAINS + " "
            )
        prose += doc_prose
        return {"name": name, "rationale": prose,
                "steps": [harness_step(
                    command, entry, json_output=is_json,
                    json_claims=_alias_envelope(command) if is_json else None,
                    asserts={} if is_json else {"stdout_contains": ["1\n", "2\n"]},
                    thread_flags=False)]}

    cases = [
        cse("run_supports_math_floor_trunc_ceil_alias_chain_when_browser_harness_is_configured",
            "run_supports_math_floor_trunc_ceil_alias_chain_when_browser_harness_is_"
            "configured_in_*_input", "run", "main.${ext}", False),
        cse("test_supports_math_floor_trunc_ceil_alias_chain_when_browser_harness_is_configured",
            "test_supports_math_floor_trunc_ceil_alias_chain_when_browser_harness_is_"
            "configured_in_*_input", "test", "smoke.test.${ext}", False),
        cse("json_run_supports_math_floor_trunc_ceil_alias_chain_when_browser_harness_is_"
            "configured",
            "json_run_supports_math_floor_trunc_ceil_alias_chain_when_browser_harness_is_"
            "configured_in_*_input", "run", "main.${ext}", True),
        cse("json_test_supports_math_floor_trunc_ceil_alias_chain_when_browser_harness_is_"
            "configured",
            "json_test_supports_math_floor_trunc_ceil_alias_chain_when_browser_harness_is_"
            "configured_in_*_input", "test", "smoke.test.${ext}", True),
    ]

    return ("math_floor_trunc_ceil_aliases.toml", header,
            {"ext": ["js", "ts", "jsx", "tsx"]},
            {"main.${ext}": FIXT_ALIASES_RUN, "smoke.test.${ext}": FIXT_ALIASES_TEST},
            cases)


def _alias_envelope(command):
    j = envelope_harness(command, stderr=True, errors=False)
    j["stdout"] = PIN_ALIAS_JSON_STDOUT
    return j


# ===========================================================================
# 3. browser_math_floor_trunc_ceil_bundle.rs -- 9 fns, ONE RETAINED.
# ===========================================================================
@target("math_floor_trunc_ceil_bundle")
def floor_trunc_ceil_bundle():
    harness_body = FIXT_BUNDLE_HARNESS_BODY

    header = f"""\
Migrated from tests/browser_math_floor_trunc_ceil_bundle.rs.

PARTIAL MIGRATION (U4 trim-and-keep) -- 8 of the file's 9 #[test] fns are
migrated here. The 9th,
`browser_bundle_math_floor_trunc_ceil_source_includes_full_frozen_callable_
inventory`, is a FIXTURE SELF-INSPECTION test: it takes
`browser_bundle_math_floor_trunc_ceil_alias_source()` and asserts
`source.contains(expected)` for every `expected` in the runtime-computed
inventory returned by kali_common's math_floor_trunc_ceil_frozen_callable_
aliases. It builds no command, runs no binary, and asserts nothing about
behaviour. That claim has no expressible form in the case format, and it is
invisible to audit-case-migration.py, whose `.contains()` extractor cannot
tell a fixture-text read from an output assertion and which excludes
everything under [source] from its search by design -- migrating it would
produce a false green. Escalated per rule 3/4 and RETAINED hand-written;
controller ruling 4 is explicit that the audit script is NOT extended for this
shape. No other fn in this file reaches that construct -- the other 8 all
route through `assert_browser_bundle_math_floor_trunc_ceil_alias`, which never
reads fixture text -- so U4's trim-and-keep applies and exactly one test is
retained.

THE .rs HAS SINCE BEEN TRIMMED to exactly that one test plus the three fixture
builders it reads, carrying a `//!` retention header, on the same pattern as
cases/browser/math_atan2_global_this_root.toml's source. TWO CONSEQUENCES A
LATER READER MUST NOT MISREAD:
  * EVERY `:N` LINE CITATION IN THIS FILE IS A PRE-TRIM LINE NUMBER. Audit and
    diff this pair against the pre-trim source (git history), not against the
    working tree.
  * Running audit-case-migration.py on the POST-trim pair reports the retained
    fn's own needles as absent -- they live in the retained test and by ruling
    4 have no home here. Measured, not assumed. That is the documented
    escalation, not a drop; against the PRE-TRIM source the audit exits 0.

{NO_RUST_COMMENTS.format(stem="math_floor_trunc_ceil_bundle")}
(The `//!` retention header added to the .rs is migration bookkeeping about the
retained test, not prose about behaviour under test, so it is not carried into
any rationale; comment_coverage will list its lines as missing for exactly
that reason.)

RULE 8 / RULE 9 -- the bundle fixture is built by `format!` over
kali_common::math_floor_trunc_ceil_frozen_callable_invocation_lines and
kali_common::math_floor_trunc_ceil_frozen_callable_entries_source, so its
resolved text appears in NO string literal in the .rs, and the template
carries `{{{{`/`}}}}` brace escapes that rule 8 forbids collapsing by hand. The
body in [source] below is the byte-exact output of executing the real builder
(a temporary test target that `include!`d the .rs and dumped it). The harness
`body` is a plain literal and was pulled from the PRE-TRIM .rs
content-anchored on its "const mod = await import(" prefix rather than by line
number (it lived inside `assert_browser_bundle_math_floor_trunc_ceil_alias`,
which the trim removed), so no line shift can corrupt it.

RULE 7 / U1 -- MATRIX ARITHMETIC, closes exactly, over the migrated 8.
Enumerated with enumerate_invocations.py: 8 fns, 8 single invocations, no
loops (the helper has no internal loop either -- checked by reading it).
Coverage is a complete cross product: ext(js/ts/jsx/tsx) x
json_output(false/true) = 8. `ext` is the axis both cases vary over uniformly
and completely, so: 2 [[case]] x ext(4) = 8 trials = the 8 migrated #[test]
fns. Per rule 6 the fold is stated here -- each [[case]] corresponds to 4
source fns, one per cell, and the assertion mapping stays 1:1 per trial.
`json_output` is NOT an axis: it changes the assertion SHAPE (a JSON build
envelope vs no stdout claim at all), not a substituted string.

U2 -- [source] is file-wide and safe here: the single fixture is written
unconditionally into a fresh temp dir, is not behind an `if`, and no case's
point is a file's presence or absence.

{RULE13_RUNNER_NOTE}
The chain DOES reach documented library helpers in another crate --
kali_common's math_floor_trunc_ceil_frozen_callable_invocation_lines,
math_floor_trunc_ceil_frozen_callable_entries_source /
math_floor_trunc_ceil_frozen_callable_entries, and
math_floor_trunc_ceil_frozen_callable_aliases -- which produce the fixture
text. Rule 13 states no "purely descriptive" exemption and explicitly includes
kali_common, so all three distinct `///` docs are carried verbatim into every
rationale.

ASSERTION SHAPE, mirrored and nothing more. The build envelope assertions stop
at schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat (:69-78): this source makes NO `errors` claim there,
unlike browser_math_hypot_global_this_root.rs, so none is written. The
metadata file claims (apiSurface/artifactKind, :85-86) are carried as a
file_json step. The only stdout claims in the whole helper are the two plain
`.contains` calls on the bundle harness output (:121-122), so both cases carry
stdout_contains on the harness step and the non-json build step asserts only
its exit status."""

    doc_prose = (
        "RULE 13 -- doc comments on the library helpers that produce this fixture, carried "
        "verbatim: \"Canonical `console.log(...)` invocation lines for the supported "
        "`Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.\", \"Canonical "
        "`return [...]` entry text for the supported `Math.floor` / `Math.trunc` / "
        "`Math.ceil` frozen callable aliases.\" and \"Canonical frozen callable aliases for "
        "the supported `Math.floor` / `Math.trunc` / `Math.ceil` helper slice.\""
    )
    base = (
        "Migrated from browser_math_floor_trunc_ceil_bundle.rs, the four "
        "`{fnfamily}` fns (one per extension). "
        "`assert_browser_bundle_math_floor_trunc_ceil_alias` builds a browser bundle "
        "(`kali build --bundle --api browser`), asserts the emitted app/app.meta.json "
        "metadata (apiSurface/artifactKind), then runs the bundle glue under the "
        "browser-bundle-harness contract. The bundled program computes Math.floor(1.6), "
        "Math.trunc(1.6) and Math.ceil(1.6) and re-invokes every frozen callable alias in the "
        "canonical floor/trunc/ceil inventory, so 1 and 2 are both printed many times. "
        "The harness stdout claims are two plain `.contains` calls (:121-122). "
        + MIRROR_CONTAINS + " "
    )

    cases = [
        {"name": "build_emits_math_floor_trunc_ceil_alias_chain",
         "rationale": base.format(
             fnfamily="build_emits_math_floor_trunc_ceil_alias_chain_in_*_input")
         + doc_prose,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": ["1\n", "2\n"]},
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_math_floor_trunc_ceil_alias_chain",
         "rationale": base.format(
             fnfamily="json_build_emits_math_floor_trunc_ceil_alias_chain_in_*_input")
         + "This sibling asserts the JSON build envelope (schemaVersion/command/success/"
           "exitCode and payload artifactKind/bundleFormat) instead of plain text. The source "
           "makes no `errors` claim on this envelope, so none is written. " + doc_prose,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": ["1\n", "2\n"]},
                               json_output=True,
                               json_claims=envelope_build(errors=False),
                               meta_fields=META)},
    ]

    return ("math_floor_trunc_ceil_bundle.toml", header,
            {"ext": ["js", "ts", "jsx", "tsx"]},
            {"app.${ext}": FIXT_BUNDLE_SRC},
            cases)


# ===========================================================================
# 4. browser_math_expm1_log1p_identities.rs -- 12 fns, 12 invocations,
#    NO [matrix] (ext coverage is NOT uniform), 12 named siblings.
# ===========================================================================
@target("math_expm1_log1p_identities")
def expm1_log1p():
    text = rs("math_expm1_log1p_identities")
    run_src = fixture_in_fn(text, "browser_harness_math_expm1_log1p_run_source")
    test_src = fixture_in_fn(text, "browser_harness_math_expm1_log1p_test_source")

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '0\\n0\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_expm1_log1p_identities.rs -- all 12 #[test]
fns, nothing retained.

{NO_RUST_COMMENTS.format(stem="math_expm1_log1p_identities")}

RULE 7 -- INVOCATION ARITHMETIC. enumerate_invocations.py: 12 #[test] fns, 12
invocations of `assert_browser_harness_math_expm1_log1p`, no loops -- and the
helper itself has NO internal loop either (checked by reading its body, which
is the one thing the enumerator cannot see). 12 fns -> 12 invocations -> 12
named sibling [[case]] entries, one per fn (rule 6, 1:1).

[matrix] DECLINED (rule 7 / U1). The `ext` coverage is NOT uniform, which is
the specific thing that had to be checked rather than assumed:
  json_output = false -- 4 invocations, js and ts ONLY
    (run main.ts, run main.js, test smoke.test.ts, test smoke.test.js)
  json_output = true  -- 8 invocations, js/ts/jsx/tsx x run/test
An `ext` axis of four values would need 12 / 4 = 3 [[case]] entries, but the
non-json half only ever runs on 2 of the 4 extensions, so fanning it would
manufacture (run, jsx, non-json) and three more cells no source fn performs --
rule 2's invented combination, and rule 7's arithmetic does not close. An
`ext` axis of two values fails symmetrically against the json half. [matrix]
is file-wide with no per-case opt-out (crates/kali_case_runner/src/expand.rs),
so it is dropped for the WHOLE file and every invocation is a named sibling.

U2 -- [source] is file-wide and that is safe here: both fixtures are written
unconditionally into a fresh temp dir, neither is behind an `if`, and no
case's point is a file's presence or absence. Every command names its entry on
argv.

U13 -- the run and test bodies are each repeated across four extension-
suffixed keys rather than hoisted into [constants], following the shipped
no-matrix precedent in cases/browser/math_atan2_global_this_root.toml.
Hoisting program text into [constants] moves it onto a surface
audit-case-migration.py DOES search (U13 records this counter-hazard itself),
so a future phantom claim could be satisfied by a fixture body.

{RULE13_RUNNER_NOTE}

ASSERTION SHAPE, mirrored and nothing more. This source asserts NO `errors`
array on the harness envelope (:55-77), so no `errors` claim is written; it
DOES assert json["stderr"] == "" (:77). The raw-stdout claim is a single plain
`.contains("0\\n")` (:80) and stays stdout_contains. The source passes NO
--max-threads / --max-spawned-processes, so neither appears on argv -- unlike
its sibling browser_math_inverse_trig_identities.rs, which does."""

    base = (
        "Migrated from browser_math_expm1_log1p_identities.rs, `{fn}`. "
        "`assert_browser_harness_math_expm1_log1p` runs `kali {argv} --api browser` with the "
        "browser harness backed by node, against a program that calls Math.expm1(0) and "
        "Math.log1p(0), each of which prints 0. "
    )

    rows = [
        ("run", "main.ts", False,
         "run_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_ts_input"),
        ("run", "main.js", False,
         "run_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_js_input"),
        ("test", "smoke.test.ts", False,
         "test_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_ts_input"),
        ("test", "smoke.test.js", False,
         "test_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_js_input"),
        ("run", "main.js", True,
         "run_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_json_js_input"),
        ("test", "smoke.test.js", True,
         "test_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_json_js_input"),
        ("run", "main.ts", True,
         "run_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_json_ts_input"),
        ("test", "smoke.test.ts", True,
         "test_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_json_ts_input"),
        ("run", "main.jsx", True,
         "json_run_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_jsx_input"),
        ("run", "main.tsx", True,
         "json_run_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_tsx_input"),
        ("test", "smoke.test.jsx", True,
         "json_test_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_jsx_input"),
        ("test", "smoke.test.tsx", True,
         "json_test_supports_math_expm1_and_log1p_identity_literals_when_browser_harness_is_"
         "configured_in_tsx_input"),
    ]

    cases = []
    for command, entry, is_json, fn_name in rows:
        prose = base.format(fn=fn_name,
                            argv=(command + " --output json") if is_json else command)
        if is_json:
            j = envelope_harness(command, stderr=True, errors=False)
            j["stdout"] = PIN_EXPM1_JSON_STDOUT
            prose += (
                "This sibling asserts the JSON envelope: schemaVersion/command/success, "
                "payload hostContract/runtimeBackend, and "
                + ("`exitCode` at both the envelope and the payload level (run)"
                   if command == "run" else "payload total/passed/failed (test)")
                + ", plus stderr exactly empty (:77). The source makes NO `errors` claim on "
                  "this envelope, so none is written. Its stdout claim here is "
                  "`.contains(\"0\\n\")` taken against json[\"stdout\"] (:70-76). "
                + MIRROR_JSON_PIN
            )
            asserts, claims = {}, j
        else:
            prose += ("Its stdout claim here is a single `.contains(\"0\\n\")` against raw "
                      "stdout (:80). " + MIRROR_CONTAINS)
            asserts, claims = {"stdout_contains": ["0\n"]}, None
        cases.append({
            "name": fn_name, "rationale": prose,
            "steps": [harness_step(command, entry, json_output=is_json,
                                   json_claims=claims, asserts=asserts,
                                   thread_flags=False)],
        })

    source = {}
    for ext in ("js", "ts", "jsx", "tsx"):
        source[f"main.{ext}"] = run_src
    for ext in ("js", "ts", "jsx", "tsx"):
        source[f"smoke.test.{ext}"] = test_src

    return ("math_expm1_log1p_identities.toml", header, None, source, cases)


# ===========================================================================
# 5. browser_math_inverse_trig_identities.rs -- 4 fns but 8 invocations
#    (the HELPER loops), 4 cases x ext(2).
# ===========================================================================
@target("math_inverse_trig_identities")
def inverse_trig():
    text = rs("math_inverse_trig_identities")
    run_src = fixture_in_fn(text, "browser_harness_math_inverse_trig_run_source")
    test_src = fixture_in_fn(text, "browser_harness_math_inverse_trig_test_source")

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '0\\n0\\n0\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_inverse_trig_identities.rs -- all 4 #[test]
fns, nothing retained.

{NO_RUST_COMMENTS.format(stem="math_inverse_trig_identities")}

RULE 7 -- INVOCATION ARITHMETIC. THE COUNT IS 8, NOT 4, and the mechanical
enumerator cannot see why: `assert_browser_harness_math_inverse_trig(command,
filename, source)` takes no json_output parameter, because the HELPER carries
its own `for output_json in [false, true]` loop wrapping its entire body
(:26-87). enumerate_invocations.py only expands loops inside #[test] fn
bodies, so it reports 4; the real figure is 4 #[test] fns x 2 helper-internal
iterations = 8 invocations. Derived by reading the helper, as rule 7 requires.

[matrix] closes exactly. Coverage is a complete cross product:
  command(run/test) x ext(js/ts) x output_json(false/true) = 8.
`ext` is the axis every case varies over uniformly and completely -- run uses
main.<ext> and test uses smoke.test.<ext>, both for js and ts and neither for
jsx/tsx -- so: 4 [[case]] x ext(2) = 8 trials = 8 real invocations. Per rule 6
the fold is stated here: each [[case]] corresponds to one half (one
output_json iteration) of two source #[test] fns, and the assertion mapping
stays 1:1 per trial. `command` and `output_json` are NOT axes: each changes
the assertion SHAPE (JSON envelope vs text stdout; `exitCode` for run vs
total/passed/failed for test; and see the "ok 1" note below), not a
substituted string.

THE CONDITIONAL CLAIM. The non-json branch makes an EXTRA assertion the json
branch does not, and it is guarded: `if command == "test"` it also asserts
stdout contains "ok 1" (:83-85). That claim therefore belongs to exactly ONE
of the four cases -- the non-json `test` sibling -- and is written only there.
Attaching it to the `run` siblings would invent a claim the source never made
(rule 2); dropping it would weaken the migration (rule 1). It is also a second
reason `command` cannot be a matrix axis here.

U2 -- [source] is file-wide and that is safe here: both fixtures are written
unconditionally into a fresh temp dir, neither is behind an `if`, and no
case's point is a file's presence or absence.

{RULE13_RUNNER_NOTE}

ASSERTION SHAPE, mirrored and nothing more. This source asserts NO `errors`
array on the harness envelope (:57-79), so no `errors` claim is written; it
DOES assert json["stderr"] == "" (:72). Note the needle is "0\\n0\\n0" with NO
trailing newline -- three values printed by Math.asin(0), Math.acos(1) and
Math.atan(0) -- and it is carried exactly as written, not tidied. The source
passes --max-threads 0 and --max-spawned-processes 0 (:42-45), so both appear
on argv."""

    base = (
        "Migrated from browser_math_inverse_trig_identities.rs, the "
        "`{fnfamily}` fns (one per extension) -- specifically the "
        "`output_json == {oj}` iteration of the helper's own "
        "`for output_json in [false, true]` loop, which is why 4 source fns produce 8 "
        "trials. `assert_browser_harness_math_inverse_trig` runs `kali {argv} --api browser "
        "--max-threads 0 --max-spawned-processes 0` with the browser harness backed by node, "
        "against a program calling Math.asin(0), Math.acos(1) and Math.atan(0) -- each "
        "printing 0. "
    )

    def cse(name, fnfamily, command, entry, is_json, extra=None):
        prose = base.format(fnfamily=fnfamily, oj=str(is_json).lower(),
                            argv=(command + " --output json") if is_json else command)
        if is_json:
            j = envelope_harness(command, stderr=True, errors=False)
            j["stdout"] = PIN_TRIG_JSON_STDOUT
            prose += (
                "This sibling asserts the JSON envelope: schemaVersion/command/success, "
                "payload hostContract/runtimeBackend, and "
                + ("`exitCode` at both the envelope and the payload level (run)"
                   if command == "run" else "payload total/passed/failed (test)")
                + ", plus stderr exactly empty (:72). The source makes NO `errors` claim on "
                  "this envelope, so none is written. Its stdout claim here is "
                  "`.contains(\"0\\n0\\n0\")` taken against json[\"stdout\"] (:73-79). "
                + MIRROR_JSON_PIN
            )
            asserts, claims = {}, j
        else:
            needles = ["0\n0\n0"]
            prose += ("Its stdout claim here is `.contains(\"0\\n0\\n0\")` against raw stdout "
                      "(:82). " + MIRROR_CONTAINS)
            if extra:
                needles.append(extra)
                prose += (
                    " This case additionally asserts stdout contains \"ok 1\": the source "
                    "guards that claim with `if command == \"test\"` (:83-85), so it holds "
                    "for this sibling and for no other. It is not copied onto the `run` "
                    "siblings (that would invent a claim, rule 2) and not dropped (that "
                    "would weaken the migration, rule 1)."
                )
            asserts, claims = {"stdout_contains": needles}, None
        return {"name": name, "rationale": prose,
                "steps": [harness_step(command, entry, json_output=is_json,
                                       json_claims=claims, asserts=asserts,
                                       thread_flags=True)]}

    cases = [
        cse("run_supports_math_inverse_trig_identity_literals_when_browser_harness_is_"
            "configured",
            "run_supports_math_inverse_trig_identity_literals_when_browser_harness_is_"
            "configured_in_*_input", "run", "main.${ext}", False),
        cse("json_run_supports_math_inverse_trig_identity_literals_when_browser_harness_is_"
            "configured",
            "run_supports_math_inverse_trig_identity_literals_when_browser_harness_is_"
            "configured_in_*_input", "run", "main.${ext}", True),
        cse("test_supports_math_inverse_trig_identity_literals_when_browser_harness_is_"
            "configured",
            "test_supports_math_inverse_trig_identity_literals_when_browser_harness_is_"
            "configured_in_*_input", "test", "smoke.test.${ext}", False, extra="ok 1"),
        cse("json_test_supports_math_inverse_trig_identity_literals_when_browser_harness_is_"
            "configured",
            "test_supports_math_inverse_trig_identity_literals_when_browser_harness_is_"
            "configured_in_*_input", "test", "smoke.test.${ext}", True),
    ]

    return ("math_inverse_trig_identities.toml", header,
            {"ext": ["js", "ts"]},
            {"main.${ext}": run_src, "smoke.test.${ext}": test_src},
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
