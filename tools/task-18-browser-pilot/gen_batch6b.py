#!/usr/bin/env python3
"""Generate the Task 18 batch 6B case files.

ONE source target -- `browser_non_literal_iterator_sources.rs`, a `#[path]`
submodule carrier with 0 top-level `#[test]` fns and 90 across four submodules
-- becomes TWO case files:

    non_literal_iterator_sources_explicit_api.toml       78 fns / 124 trials
    non_literal_iterator_sources_inherited_manifest.toml 12 fns /  96 trials

WHY TWO AND NOT ONE (the whole point of this generator's shape). U10 says
migrate a submodule carrier and its sibling directory into ONE `.toml`. U2
overrides it here: two of the carrier's helpers write a `kali.json` manifest,
and for the 12 `*_under_inherited_browser_config` tests the manifest's PRESENCE
is the case -- the browser API surface is inherited from it instead of being
passed as `--api browser`. `[source]` is FILE-WIDE (`expand.rs` clones the whole
map into every trial), so folding all 90 into one file would make `kali.json`
unconditionally present and the other 78 cases would pass whether or not
`--api browser` did anything. `audit-case-migration.py` cannot see that (no
literal is dropped) and neither can `cargo test` (the trial still passes).

EVERY NUMBER IN THE HEADERS IS DERIVED HERE, NOT TYPED. The invocation
enumeration comes from `enumerate_invocations.py` parsing the four submodules;
the fixture bodies come through `case_emit.fixture_in_fn` (rule 9: never
retype a program under test); every `:N` comes from `batch5_prose.cite_line`
searching for the construct; every exact pin is captured from the real built
`kali` for EVERY case, not a sample (U9).

Run: python3 gen_batch6b.py [--no-live]
     --no-live skips the real-binary capture pass and reuses nothing -- it
     exists only for a syntax check and will refuse to write files.
"""

import json as _json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import emit, fixture_in_fn, fixture_starting, write  # noqa: E402
from enumerate_invocations import (  # noqa: E402
    invocations, strip_block_comments_and_strings, test_fn_bodies,
)
from kali_run import run_kali  # noqa: E402
from math_shapes import rule12_no_comments_prose  # noqa: E402
from submodules import submodule_paths  # noqa: E402
import batch5_prose as P  # noqa: E402

STEM = "non_literal_iterator_sources"
CARRIER = os.path.join(TESTS, f"browser_{STEM}.rs")
EXPLICIT_STEM = f"{STEM}_explicit_api"
INHERITED_STEM = f"{STEM}_inherited_manifest"
EXTS4 = ["js", "ts", "jsx", "tsx"]
HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"
# ^ the value of `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`, read from
# crates/kali_runtime_contract/src/browser/contract.rs rather than assumed --
# this source passes the constant, and the migrated `env` key must resolve to
# the same variable name.
CONTRACT = os.path.join(REPO, "crates/kali_runtime_contract/src/browser/contract.rs")


# --------------------------------------------------------------------------
# 1. The source, its submodules, and its fixtures.
# --------------------------------------------------------------------------

CARRIER_TEXT = open(CARRIER).read()
SUBMODULES = {p.name: p for p in submodule_paths(CARRIER)}
SUB_TEXT = {name: p.read_text() for name, p in SUBMODULES.items()}

# fixture fn -> the `[source]` key stem it is written under. The source writes
# every one of them to the SAME filename (`main.<ext>` / `smoke.test.<ext>`) in
# different tests, and `[source]` is one flat file-wide namespace, so U5
# variant-suffixed stems are mandatory, not cosmetic.
FIXTURE_STEMS = {
    "for_of_source": "for_of",
    "for_await_source": "for_await",
    "object_keys_source": "object_keys",
    "object_values_source": "object_values",
    "object_entries_source": "object_entries",
    "array_callback_iteration_source": "array_callback",
    "set_constructor_call_expression_source": "set_constructor",
    "map_constructor_call_expression_source": "map_constructor",
}
FIXTURES = {fn: fixture_in_fn(CARRIER_TEXT, fn) for fn in FIXTURE_STEMS}

# Ruling 7's MANDATORY mechanical duplicate-identity assertion. The two
# manifest-writing helpers each embed the `kali.json` body as their own literal;
# the ruling declines U13's hoist into `[constants]` for `browser/` but requires
# the duplication be asserted, not eyeballed.
MANIFEST = P.assert_identical(
    "kali.json manifest body, written by both inherited helpers",
    fixture_starting(CARRIER_TEXT,
                     "assert_inherited_browser_iterator_source_rejects", "{\n"),
    fixture_starting(CARRIER_TEXT,
                     "assert_inherited_browser_array_callback_iteration_source_rejects",
                     "{\n"),
)


def newname(fixture_fn, filename):
    """The U5-renamed `[source]` key for one (fixture, source filename) pair."""
    stem = FIXTURE_STEMS[fixture_fn]
    if filename.startswith("smoke.test."):
        return f"{stem}_smoke.test.{filename.split('smoke.test.')[1]}"
    if not filename.startswith("main."):
        raise AssertionError(f"unexpected source filename {filename!r}")
    return f"{stem}.{filename.split('.', 1)[1]}"


# --------------------------------------------------------------------------
# 2. Invocation enumeration, mechanically, from the four submodules.
# --------------------------------------------------------------------------

# The one carrier-level helper that is itself a loop: it fans json_output(2) x
# ext(4) over `assert_browser_iterator_source_rejects` with the map fixture.
# `enumerate_invocations` reports a call to it as ONE invocation (it does not
# follow helper-to-helper calls), so it is expanded here -- and the expansion is
# read out of the carrier rather than assumed.
FANOUT_HELPER = "assert_map_constructor_iteration_from_call_expression_source_rejects"

INHERITED_HELPERS = {
    "assert_inherited_browser_iterator_source_rejects",
    "assert_inherited_browser_array_callback_iteration_source_rejects",
}
# The helpers whose message assertion is an OR (rule 11 resolves it live).
OR_HELPERS = {
    "assert_browser_array_callback_iteration_source_rejects",
    "assert_inherited_browser_array_callback_iteration_source_rejects",
    "assert_browser_requested_array_callback_iteration_source_rejects",
}
# The helpers that set the browser-harness command env var on the child.
ENV_HELPERS = {
    "assert_browser_requested_iterator_source_rejects",
    "assert_browser_requested_array_callback_iteration_source_rejects",
}


def _fanout_body():
    """The `for` nest inside the carrier's own looping helper, as source text."""
    masked = strip_block_comments_and_strings(CARRIER_TEXT)
    m = re.search(r"\bfn\s+" + re.escape(FANOUT_HELPER) + r"\s*\(", masked)
    if not m:
        raise AssertionError(f"no `fn {FANOUT_HELPER}` in the carrier")
    brace = masked.find("{", m.end() - 1)
    depth, i = 0, brace
    while i < len(masked):
        if masked[i] == "{":
            depth += 1
        elif masked[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    return CARRIER_TEXT[brace + 1:i]


def enumerate_all():
    """[(submodule, fn_name, helper, args)] -- every real invocation, loops expanded."""
    fanout, unparsed = invocations(_fanout_body())
    if unparsed:
        raise AssertionError(f"could not parse {FANOUT_HELPER}: {unparsed}")
    if len({h for h, _ in fanout}) != 1:
        raise AssertionError(f"{FANOUT_HELPER} calls more than one helper")

    rows = []
    for sub in sorted(SUBMODULES):
        text = SUB_TEXT[sub]
        masked = strip_block_comments_and_strings(text)
        for fn_name, body, _line in test_fn_bodies(text, masked):
            invs, bad = invocations(body)
            if bad or not invs:
                raise AssertionError(f"{sub}:{fn_name}: {bad or 'no helper call found'}")
            for helper, args in invs:
                if helper == FANOUT_HELPER:
                    command, bundle = args
                    for inner_helper, inner_args in fanout:
                        resolved = [command if a == "command" else
                                    bundle if a == "bundle" else a
                                    for a in inner_args]
                        rows.append((sub, fn_name, inner_helper, resolved))
                else:
                    rows.append((sub, fn_name, helper, args))
    return rows


def unq(tok):
    """A Rust string literal token -> its text; a bool token -> a bool."""
    if tok in ("true", "false"):
        return tok == "true"
    if tok.startswith('"') and tok.endswith('"'):
        return tok[1:-1]
    if tok.endswith("()"):
        return tok[:-2]
    raise AssertionError(f"unexpected argument token {tok!r}")


def parse_row(sub, fn_name, helper, args):
    """One invocation as a dict of the five things that decide its case."""
    if helper in ENV_HELPERS:
        fixture_fn, filename, json_output, command = args
        bundle = False
    else:
        fixture_fn, filename, json_output, command, bundle = args
        bundle = unq(bundle)
    return {
        "sub": sub,
        "fn": fn_name,
        "helper": helper,
        "fixture": unq(fixture_fn),
        "filename": unq(filename),
        "json": unq(json_output),
        "command": unq(command),
        "bundle": bundle,
        "inherited": helper in INHERITED_HELPERS,
        "env": helper in ENV_HELPERS,
        "or_message": helper in OR_HELPERS,
    }


ROWS = [parse_row(*r) for r in enumerate_all()]

# The collision U5 exists for, DERIVED: how many distinct programs the source
# writes, and how few distinct filenames it writes them to.
_SOURCE_FILENAMES = sorted({r["filename"] for r in ROWS})
_SOURCE_STEMS = sorted({f.rsplit(".", 1)[0] for f in _SOURCE_FILENAMES})
SOURCE_FILENAME_PHRASE = (
    f"just {len(_SOURCE_STEMS)} filename stem(s) ("
    + " and ".join(f"`{s}.<ext>`" for s in _SOURCE_STEMS) + ")")

TESTS_TOTAL = sum(len([f for f, _, _ in test_fn_bodies(
    SUB_TEXT[s], strip_block_comments_and_strings(SUB_TEXT[s]))]) for s in SUBMODULES)
assert TESTS_TOTAL == len({(r["sub"], r["fn"]) for r in ROWS}), (
    "some #[test] fn produced no invocation")

INHERITED_ROWS = [r for r in ROWS if r["inherited"]]
EXPLICIT_ROWS = [r for r in ROWS if not r["inherited"]]


def argv(row, entry):
    """The argv the source's `Command` builder appends, in its own order."""
    out = ["--output", "json"] if row["json"] else []
    if row["env"]:
        # `assert_browser_requested_*`: env first, then --output json, then the
        # subcommand, then --api browser. No --bundle in this shape.
        out += [row["command"], "--api", "browser", entry]
        return out
    out += [row["command"]]
    if row["bundle"]:
        out += ["--bundle"]
    if not row["inherited"]:
        out += ["--api", "browser"]
    out += [entry]
    return out


def envelope_command(row):
    """What the source asserts `json["command"]` is."""
    if row["env"]:
        return row["command"]          # assert_eq!(json["command"], command)
    return "build" if row["bundle"] else "check"


# --------------------------------------------------------------------------
# 3. Live capture against the real binary (U9), per case, never a sample.
# --------------------------------------------------------------------------

def live(row, entry, sources):
    """Run the real `kali` in a dir holding the WHOLE `[source]` map.

    The trial dir the case runner builds holds every `[source]` entry, not just
    this case's, so the capture must too -- otherwise a fixture-interference bug
    (a `check` that walked the directory, a `test` that discovered every
    `*.test.*` sibling) would be invisible until the case file shipped.
    """
    env = {HARNESS_ENV: "node"} if row["env"] else None
    rc, out, err, _ = run_kali(sources, argv(row, entry), env=env)
    if rc != 1:
        raise AssertionError(f"{row['fn']} / {entry}: exit {rc}, source asserts 1")
    err = err.decode()
    if row["json"]:
        doc = _json.loads(out)
        if doc["schemaVersion"] != 1 or doc["success"] is not False or doc["exitCode"] != 1:
            raise AssertionError(f"{row['fn']} / {entry}: envelope {doc!r}")
        if doc["command"] != envelope_command(row):
            raise AssertionError(
                f"{row['fn']} / {entry}: json.command {doc['command']!r} != "
                f"{envelope_command(row)!r}")
        errors = doc["errors"]
        if not errors:
            raise AssertionError(f"{row['fn']} / {entry}: empty errors array")
        if errors[0]["code"] != "E5506":
            raise AssertionError(f"{row['fn']} / {entry}: code {errors[0]['code']!r}")
        message = errors[0]["message"]
        _check_message(row, entry, message, "json errors[0].message")
        return message
    if "E5506" not in err:
        raise AssertionError(f"{row['fn']} / {entry}: stderr lacks E5506: {err!r}")
    _check_message(row, entry, err, "stderr")
    return None


def _check_message(row, entry, text, where):
    """The source's own message predicate, re-run against the live output."""
    if row["or_message"]:
        # rule 11: the source accepts either branch; the live run decides which.
        if not ("array callback-produced iterables" in text or "literal array" in text):
            raise AssertionError(f"{row['fn']} / {entry}: {where} satisfies neither "
                                 f"disjunct: {text!r}")
        if "array callback-produced iterables" in text:
            raise AssertionError(
                f"{row['fn']} / {entry}: the OR resolves to the "
                "'array callback-produced iterables' branch, which this generator's "
                "pins do not encode -- rule 11 requires the pin follow the observed "
                "branch, so regenerate with that branch handled")
    if "literal array" not in text:
        raise AssertionError(f"{row['fn']} / {entry}: {where} lacks 'literal array': "
                             f"{text!r}")




# --------------------------------------------------------------------------
# 4. Citations -- every one derived by SEARCHING for the construct, and scoped
#    to the fn that actually contains it.
# --------------------------------------------------------------------------

def _fn_span(text, fn):
    """(first_line, last_line), 1-based inclusive, of `fn <fn>`'s definition."""
    masked = strip_block_comments_and_strings(text)
    m = re.search(r"\bfn\s+" + re.escape(fn) + r"\s*[(<]", masked)
    if not m:
        raise AssertionError(f"no `fn {fn}` in that source")
    first = text[:m.start()].count("\n") + 1
    brace = masked.find("{", m.end() - 1)
    depth, i = 0, brace
    while i < len(masked):
        if masked[i] == "{":
            depth += 1
        elif masked[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    return first, text[:i].count("\n") + 1


def cite_in(text, fn, pattern, *, label=None):
    """The single line matching `pattern` INSIDE `fn`, as an absolute line no.

    Scoping matters here and is not a convenience: the carrier defines six
    near-identical assert helpers, so `assert_eq!(output.status.code(), Some(1))`
    matches six times file-wide. A citation that resolved to "the first one"
    would point into a different helper than the case it annotates, and
    `batch5_crosscheck.py` would report it as CORRECT -- the construct really is
    on that line. Only scoping makes the citation mean what the prose says.
    """
    lo, hi = _fn_span(text, fn)
    hits = [i for i, line in enumerate(text.split("\n"), 1)
            if lo <= i <= hi and re.search(pattern, line)]
    if len(hits) != 1:
        raise AssertionError(
            f"citation anchor {label or pattern!r} in `fn {fn}`: {len(hits)} match(es) "
            f"{hits}, wanted 1")
    return hits[0]


def src_line(n):
    """The carrier's line `n`, stripped -- quoted into prose as its own citation
    anchor, so the snippet a rationale shows is literally the text at `:n`."""
    return CARRIER_TEXT.split("\n")[n - 1].strip().rstrip(";")


def carrier_cite(pattern, *, label=None, expect=1):
    return P.cite_line(CARRIER_TEXT, pattern, label=label, expect=expect)


def fn_cite(sub, fn_name):
    """`<submodule>.rs:N` for a `#[test]` fn's own `fn` line.

    `batch5_crosscheck.py` resolves this qualified form against that submodule;
    a bare `:N` would be resolved against the CARRIER, where the fn does not
    exist. (rustfmt splits four of these signatures across two lines; the anchor
    is the `fn <name>(` prefix and the gate's enclosing-statement expansion
    covers the `) {` continuation.)
    """
    line = P.cite_line(SUB_TEXT[sub], r"^fn " + re.escape(fn_name) + r"\(",
                       label=f"{sub}:{fn_name}")
    return f"{sub}:{line}"


# Per-helper citations. Every construct the rationales cite, resolved inside the
# helper that contains it, so six near-identical helpers cannot be conflated.
HELPER_CITE = {}
for _h in sorted({r["helper"] for r in ROWS}):
    _c = {
        "fn": carrier_cite(r"^fn " + re.escape(_h) + r"\(", label=_h),
        "abs_path": cite_in(CARRIER_TEXT, _h, r"dir\.path\(\)\.join\(filename\)"),
        "exit_code": cite_in(CARRIER_TEXT, _h,
                             r"assert_eq!\(output\.status\.code\(\), Some\(1\)\)"),
        "success": cite_in(CARRIER_TEXT, _h, r"assert!\(!output\.status\.success\(\)"),
        "schema": cite_in(CARRIER_TEXT, _h, r'assert_eq!\(json\["schemaVersion"\], 1\)'),
        "command": cite_in(CARRIER_TEXT, _h, r'assert_eq!\(json\["command"\]'),
        "success_json": cite_in(CARRIER_TEXT, _h, r'assert_eq!\(json\["success"\], false\)'),
        "exit_json": cite_in(CARRIER_TEXT, _h, r'assert_eq!\(json\["exitCode"\], 1\)'),
        "errors_nonempty": cite_in(CARRIER_TEXT, _h, r"!errors\.is_empty\(\)"),
        "code": cite_in(CARRIER_TEXT, _h, r'assert_eq!\(errors\[0\]\["code"\], "E5506"\)'),
        "message": cite_in(CARRIER_TEXT, _h, r'message\.contains\("literal array"\)'),
        "stderr_code": cite_in(CARRIER_TEXT, _h, r'stderr\.contains\("E5506"\)'),
        "stderr_message": cite_in(CARRIER_TEXT, _h, r'stderr\.contains\("literal array"\)'),
        "output_json": cite_in(CARRIER_TEXT, _h, r'cmd\.arg\("--output"\)\.arg\("json"\)'),
        "run_kali": cite_in(CARRIER_TEXT, _h, r'\.expect\("run kali"\)'),
    }
    if _h not in ENV_HELPERS:
        _c["bundle_flag"] = cite_in(CARRIER_TEXT, _h, r'cmd\.arg\("--bundle"\)')
    if _h in OR_HELPERS:
        _c["message_or"] = cite_in(
            CARRIER_TEXT, _h, r'message\.contains\("array callback-produced iterables"\)')
        _c["stderr_or"] = cite_in(
            CARRIER_TEXT, _h, r'stderr\.contains\("array callback-produced iterables"\)')
    if _h in INHERITED_HELPERS:
        _c["manifest"] = cite_in(CARRIER_TEXT, _h, r'dir\.path\(\)\.join\("kali\.json"\)')
        _c["manifest_expect"] = cite_in(CARRIER_TEXT, _h, r'\.expect\("write manifest"\)')
    if _h in ENV_HELPERS:
        _c["env"] = cite_in(CARRIER_TEXT, _h,
                            r'cmd\.env\(BROWSER_HARNESS_COMMAND_ENV, "node"\)')
    HELPER_CITE[_h] = _c

FIXTURE_CITE = {fn: carrier_cite(r"^fn " + re.escape(fn) + r"\(") for fn in FIXTURE_STEMS}
FANOUT_CITE = carrier_cite(r"^fn " + re.escape(FANOUT_HELPER) + r"\(")


# --------------------------------------------------------------------------
# 5. Rule 13 -- every helper in every call chain, checked mechanically.
# --------------------------------------------------------------------------

CHAIN_FNS = (["kali_bin"] + sorted(FIXTURE_STEMS)
             + sorted({r["helper"] for r in ROWS}) + [FANOUT_HELPER])


def assert_no_doc_comments(fns):
    """Rule 13: raise if any fn in the chain carries a `///` doc comment.

    The rule-13 header block says "none of them carries a `///` doc comment".
    That sentence is only true because this ran -- batch 6A's fix round 1 (I4)
    is exactly a claimed mechanical check that no mechanism performed.
    """
    lines = CARRIER_TEXT.split("\n")
    documented = []
    for fn in fns:
        first, _ = _fn_span(CARRIER_TEXT, fn)
        i = first - 2                       # the line above the `fn` line
        while i >= 0 and lines[i].lstrip().startswith("///"):
            documented.append(fn)
            break
    if documented:
        raise AssertionError(
            f"rule 13: `///` docs found on {sorted(set(documented))}; the header's "
            "'none carries a doc comment' sentence would be false")
    return len(fns)


CHAIN_CHECKED = assert_no_doc_comments(CHAIN_FNS)


def contract_doc_line():
    """The `///` doc on `BROWSER_HARNESS_COMMAND_ENV`, read from its own crate."""
    text = open(CONTRACT).read()
    m = re.search(
        r"^(///[^\n]*)\npub const BROWSER_HARNESS_COMMAND_ENV: &str = \"([^\"]+)\";",
        text, re.M)
    if not m:
        raise AssertionError("BROWSER_HARNESS_COMMAND_ENV moved or lost its doc")
    if m.group(2) != HARNESS_ENV:
        raise AssertionError(
            f"BROWSER_HARNESS_COMMAND_ENV is {m.group(2)!r}, generator has {HARNESS_ENV!r}")
    return m.group(1).lstrip("/ ").strip()


CONTRACT_DOC = contract_doc_line()


def contract_doc_precedent():
    """How many shipped case files set that env, and how many carry its doc.

    Ruling 13: a sentence quantifying over a set of files runs its enumerating
    command BEFORE the sentence is written. This IS that command, and its two
    numbers go into the header beside the claim.
    """
    setters = documented = 0
    root = os.path.join(TESTS, "cases")
    for dirpath, _dirs, files in os.walk(root):
        for name in sorted(files):
            # THIS BATCH'S OWN TWO FILES ARE EXCLUDED, and the exclusion is
            # printed in the header beside the numbers. Both strings occur in
            # the header that reports the count -- the env var because the
            # run/test cases set it, the doc line because the header quotes it
            # -- so an unfiltered grep would count the sentence's own file and
            # the figure would move by being written. Ruling 11, third disguise.
            if not name.endswith(".toml") or STEM in name:
                continue
            body = open(os.path.join(dirpath, name)).read()
            if HARNESS_ENV in body:
                setters += 1
                if CONTRACT_DOC in body:
                    documented += 1
    return setters, documented


# --------------------------------------------------------------------------
# 6. Case construction.
# --------------------------------------------------------------------------

def group_by_fn(rows):
    """[(sub, fn, [row, ...])] in source order."""
    out, index = [], {}
    for row in rows:
        key = (row["sub"], row["fn"])
        if key not in index:
            index[key] = len(out)
            out.append((row["sub"], row["fn"], []))
        out[index[key]][2].append(row)
    return out


def suffix_for(row, varies):
    """The loop coordinate that tells one invocation of a looped fn from another.

    Only the dimensions that ACTUALLY vary inside that fn contribute, so a
    1:1 fn gets no suffix at all and keeps the source fn name verbatim as its
    case name (rule 6: the case is the only remaining trace of the fn).
    """
    parts = []
    if "fixture" in varies:
        parts.append(FIXTURE_STEMS[row["fixture"]])
    if "filename" in varies:
        parts.append(row["filename"].rsplit(".", 1)[1])
    if "json" in varies:
        parts.append("json" if row["json"] else "text")
    return ("__" + "_".join(parts)) if parts else ""


def varying(rows):
    return {dim for dim in ("fixture", "filename", "json")
            if len({r[dim] for r in rows}) > 1}


# --- rationale -----------------------------------------------------------

SPLIT_EXPLICIT = (
    "TWO-FILE SPLIT (U2). This file is the EXPLICIT-`--api browser` half: no "
    "`kali.json` appears in its `[source]` table at all, which is exactly what keeps "
    "this case able to FAIL if `--api browser` regressed. The 12 "
    "manifest-inheriting `#[test]` fns are in "
    f"{INHERITED_STEM}.toml. They cannot be `[[case]]` entries in this file: "
    "`expand.rs`'s `expand()` clones the whole file-level `[source]` map into EVERY "
    "trial regardless of which case references which key, so a shared `kali.json` "
    "would supply `apiSurface` here too and this case would pass whether or not the "
    "flag did anything -- with no literal dropped, so `audit-case-migration.py` "
    "cannot see it, and with the trial still green, so `cargo test` cannot either."
)
SPLIT_INHERITED = (
    "TWO-FILE SPLIT (U2). This file is the MANIFEST-INHERITED half: `kali.json` is in "
    "its `[source]` table and every argv below omits `--api browser`, so the browser "
    "API surface can only come from the manifest. The other 78 `#[test]` fns pass the "
    f"flag explicitly and are in {EXPLICIT_STEM}.toml, whose `[source]` "
    "deliberately holds no manifest at all -- see that file's header for why one "
    "shared `[source]` table would silently disarm them."
)


def rationale(row, entry, message, *, matrix_fold):
    c = HELPER_CITE[row["helper"]]
    fixture_line = FIXTURE_CITE[row["fixture"]]
    parts = []

    origin = (f"Migrated from browser_{STEM}.rs, the `{row['fn']}` `#[test]` fn in its "
              f"`{row['sub']}` `#[path]` submodule "
              f"(`{row['fn']}(` ({fn_cite(row['sub'], row['fn'])})). ")
    if matrix_fold:
        origin += (
            "That fn loops `for json_output in [false, true]` over "
            "`for filename in [\"main.js\", \"main.ts\", \"main.jsx\", \"main.tsx\"]`, 8 real "
            f"invocations; this `[[case]]` is its {'JSON' if row['json'] else 'text'}-output "
            "half, matrix-fanned by `ext(4)` to the 4 filenames, so the 12 fns become 24 "
            "cases x 4 = 96 trials = 96 invocations.")
    elif row["looped"]:
        origin += (f"That fn makes {row['n_siblings']} real invocations with its loops "
                   f"expanded; this `[[case]]` is exactly one of them (`{entry}`, "
                   f"{'--output json' if row['json'] else 'text output'}), split into named "
                   "siblings per rule 5 rather than folded.")
    else:
        origin += ("That fn is a single unlooped helper call, so it maps 1:1 to this one "
                   "`[[case]]` (rule 6).")
    parts.append(origin)

    if row["fanout"]:
        parts.append(
            f"The call goes through `{FANOUT_HELPER}(` (:{FANOUT_CITE}), a carrier-level "
            "helper that is itself the `json_output(2) x filename(4)` loop and forwards to "
            f"`{row['helper']}(` (:{c['fn']}).")

    helper = (f"`{row['helper']}(` (:{c['fn']}) writes the program to a fresh temp dir as "
              f"`dir.path().join(filename)` (:{c['abs_path']}) and runs the real `kali` with "
              f"that dir as cwd.")
    if row["inherited"]:
        helper += (" It ALSO writes a `kali.json` manifest -- "
                   f"`dir.path().join(\"kali.json\")` (:{c['manifest']}), "
                   f"`.expect(\"write manifest\")` (:{c['manifest_expect']}) -- whose "
                   "`compilerOptions.apiSurface` is `browser`, and it passes NO `--api` "
                   "flag: the API surface is inherited from the manifest. That conditional "
                   "fixture is why this is a separate case FILE.")
    if row["env"]:
        helper += (" It sets the browser-harness command variable on the child -- "
                   f"`cmd.env(BROWSER_HARNESS_COMMAND_ENV, \"node\")` (:{c['env']}) -- "
                   f"carried as `env = {{ {HARNESS_ENV} = \"node\" }}`.")
    parts.append(helper)

    if matrix_fold:
        wrote = ('`main.js`, `main.ts`, `main.jsx` and `main.tsx`, one per loop iteration, '
                 'which is the `ext` axis this case is fanned by')
    else:
        wrote = f"`{row['filename']}`"
    parts.append(
        f"The program under test is `{row['fixture']}()` (:{fixture_line}), written to "
        f"`{entry}`. The source writes it to {wrote}; the `[source]` key is U5-renamed "
        f"because `[source]` is one flat file-wide namespace and this source writes "
        f"{len(FIXTURE_STEMS)} different programs to {SOURCE_FILENAME_PHRASE}. The rename "
        "is argv-only -- no fixture body in this file names any of these files by string "
        "-- so it does not rewrite the program under test (rule 9).")

    parts.append(
        "The source asserts the command FAILED and pins the exact code: "
        f"`assert!(!output.status.success()` (:{c['success']}) and "
        f"`assert_eq!(output.status.code(), Some(1))` (:{c['exit_code']}). "
        "`assert_eq!` is already exact, so rule 1's non-negotiable direction gives "
        "`exit = 1` rather than the weaker `exit = \"failure\"` status class.")

    if row["json"]:
        parts.append(
            "In `--output json` mode the source parses stdout as the envelope and asserts "
            f"`assert_eq!(json[\"schemaVersion\"], 1)` (:{c['schema']}), "
            f"`{src_line(c['command'])}` (:{c['command']}), "
            f"`assert_eq!(json[\"success\"], false)` (:{c['success_json']}) and "
            f"`assert_eq!(json[\"exitCode\"], 1)` (:{c['exit_json']}); all four are exact "
            "source assertions and become exact `json.*` pins.")
        parts.append(
            f"It then asserts `!errors.is_empty()` (:{c['errors_nonempty']}) and "
            f"`assert_eq!(errors[0][\"code\"], \"E5506\")` (:{c['code']}). The "
            "non-emptiness claim needs no key of its own and is NOT dropped: design spec "
            "5.4 makes an out-of-range numeric segment a HARD failure, never a silent "
            "skip, so `json.errors.0.code` cannot pass against an empty array. Pinning it "
            "implies the non-emptiness claim and is strictly stronger.")
        if row["or_message"]:
            parts.append(
                "The source's message claim here is a DISJUNCTION, carried verbatim as "
                "rule 11 requires: `message.contains(\"array callback-produced iterables\")` "
                f"(:{c['message_or']}) `|| message.contains(\"literal array\")` "
                f"(:{c['message']}). The case format has no disjunction, so the real binary "
                "was run for this exact case and the `literal array` branch is the one it "
                "emits; that branch is what is pinned. Every run satisfying the pin "
                "satisfies the source's OR, so this is a verified strengthening, not a "
                "narrowing of an absence claim.")
        else:
            parts.append(
                f"The message claim is `message.contains(\"literal array\")` (:{c['message']}) "
                "against `errors[0][\"message\"]`.")
        parts.append(
            "That message is a JSON string leaf, and the case format has NO substring form "
            "for one (there is no json_contains key), so controller ruling 3 makes it an "
            "exact `json.errors.0.message` pin -- and, per U9, only after the value was "
            "captured from the real built `kali` for this case rather than hand-computed.")
    else:
        parts.append(
            f"In text mode the source asserts `stderr.contains(\"E5506\")` "
            f"(:{c['stderr_code']}) and "
            + (f"`stderr.contains(\"array callback-produced iterables\")` "
               f"(:{c['stderr_or']}) `|| stderr.contains(\"literal array\")` "
               f"(:{c['stderr_message']}), a DISJUNCTION resolved against the real binary "
               "per rule 11 and carried verbatim here: the built `kali` emits the "
               "`literal array` branch for this case, so that is the needle pinned, and "
               "every run satisfying it satisfies the source's OR."
               if row["or_message"] else
               f"`stderr.contains(\"literal array\")` (:{c['stderr_message']}).")
            + " " + P.ruling3_substring(surface="the captured `stderr`",
                                        key="stderr_contains"))
        parts.append(
            "The source reads no JSON in this mode, so this case pins no `json` field: "
            "adding one would be a rule 2 invention.")

    parts.append(SPLIT_INHERITED if row["inherited"] else SPLIT_EXPLICIT)
    return " ".join(parts)


def build_cases(rows, *, matrix_fold, sources):
    """[(case dict, [(row, entry), ...])] plus the live-captured pins."""
    cases, seen = [], set()
    for sub, fn, fn_rows in group_by_fn(rows):
        varies = varying(fn_rows)
        if matrix_fold:
            # 12 fns x json(2), each fanned by ext(4). Assert the fan is exactly
            # the 4 extensions once each before folding (rule 7).
            for json_output in (False, True):
                cell = [r for r in fn_rows if r["json"] is json_output]
                exts = sorted(r["filename"].rsplit(".", 1)[1] for r in cell)
                if exts != sorted(EXTS4):
                    raise AssertionError(
                        f"{sub}:{fn} json={json_output} covers {exts}, not {EXTS4}")
                row = dict(cell[0], looped=True, n_siblings=len(fn_rows), fanout=False)
                entry = f"{FIXTURE_STEMS[row['fixture']]}.${{ext}}"
                name = f"{fn}__{'json' if json_output else 'text'}"
                cases.append((name, row, entry, cell))
        else:
            for r in fn_rows:
                row = dict(r, looped=len(fn_rows) > 1, n_siblings=len(fn_rows),
                           fanout=(r["fixture"] == "map_constructor_call_expression_source"
                                   and r["helper"] == "assert_browser_iterator_source_rejects"
                                   and len(fn_rows) == 8))
                entry = newname(r["fixture"], r["filename"])
                cases.append((fn + suffix_for(r, varies), row, entry, [r]))
    for name, _row, _entry, _cell in cases:
        if name in seen:
            raise AssertionError(f"duplicate case name {name!r}")
        seen.add(name)
    return cases


def step_for(row, entry, message):
    step = {"args": argv(row, entry), "exit": 1}
    if row["env"]:
        step["env"] = {HARNESS_ENV: "node"}
    if row["json"]:
        step["json_paths"] = {
            "schemaVersion": 1,
            "command": envelope_command(row),
            "success": False,
            "exitCode": 1,
            "errors.0.code": "E5506",
            "errors.0.message": message,
        }
    else:
        step["stderr_contains"] = ["E5506", "literal array"]
    return step


# --------------------------------------------------------------------------
# 7. Headers.
# --------------------------------------------------------------------------

PLAIN = "assert_browser_iterator_source_rejects"
INHERITED = "assert_inherited_browser_iterator_source_rejects"
REQUESTED = "assert_browser_requested_iterator_source_rejects"
ARRCB = "assert_browser_array_callback_iteration_source_rejects"
ARRCB_INH = "assert_inherited_browser_array_callback_iteration_source_rejects"
ARRCB_REQ = "assert_browser_requested_array_callback_iteration_source_rejects"


def u10_block(half, fns_here, invocations_here):
    subs = ", ".join(f"`{n}`" for n in sorted(SUBMODULES))
    return [
        "U10 -- SUBMODULE CARRIER, and the trap it exists for.",
        f"`grep -c '#\\[test\\]'` on tests/browser_{STEM}.rs returns 0 and would",
        f"silently drop all {TESTS_TOTAL} tests: every one of them lives behind a",
        f"`#[path = \"...\"] mod` declaration, in {len(SUBMODULES)} sibling files ({subs}).",
        "The carrier holds only the fixture builders and the six assert helpers.",
        f"This file carries {fns_here} of those {TESTS_TOTAL} `#[test]` fns "
        f"({invocations_here} real invocations);",
        "the rest are the other half of the U2 split named below, and the two files",
        f"together account for all {TESTS_TOTAL}.",
        "Citations into a submodule are written `<file>.rs:N` and are resolved against",
        "THAT file by `batch5_crosscheck.py`; a bare `:N` means the carrier.",
        f"The carrier and its sibling directory are RETAINED here -- they are deleted",
        "together, by the family-wide operation after batch 8, not by this commit.",
    ]


def u2_block(half):
    common = [
        "U2 -- `[source]` is FILE-WIDE, WHICH IS WHY THIS TARGET IS TWO CASE FILES AND",
        "NOT ONE.",
        f"U10 says migrate a submodule carrier and its sibling directory into ONE `.toml`.",
        f"That is wrong here and U2 takes precedence. Two of the carrier's helpers write a",
        f"`kali.json` manifest -- `dir.path().join(\"kali.json\")` "
        f"(:{HELPER_CITE[INHERITED]['manifest']}), and again at",
        f"`dir.path().join(\"kali.json\")` (:{HELPER_CITE[ARRCB_INH]['manifest']}) -- and "
        "for the 12",
        "`*_under_inherited_browser_config` `#[test]` fns the manifest's PRESENCE is the",
        "whole case: those invocations pass no `--api` flag at all and the browser API",
        "surface is resolved from the manifest instead. The other 78 pass `--api browser`",
        "explicitly and run against a tree with NO manifest.",
        "`crates/kali_case_runner/src/expand.rs`'s `expand()` clones the whole file-level",
        "`[source]` map into EVERY trial regardless of which case references which key, so",
        "one shared table would make `kali.json` unconditionally present and the 78",
        "explicit cases would pass whether or not `--api browser` did anything. No literal",
        "is dropped by that leak, so `audit-case-migration.py` cannot see it; the trial",
        "still passes, so `cargo test` cannot either. That invisibility is exactly why U2",
        "exists.",
    ]
    if half == "explicit":
        return common + [
            f"THIS FILE is the explicit half: `kali.json` appears in its `[source]` table",
            "NOWHERE. Its sibling is",
            f"{INHERITED_STEM}.toml.",
            "Within this file `[source]` is safe in the ordinary way: every fixture is",
            "written unconditionally by the source into a fresh temp dir, no fixture is",
            "written behind an `if`, and every command names its entry explicitly on argv,",
            "so the unused siblings in a trial dir are inert.",
        ]
    return common + [
        f"THIS FILE is the manifest-inherited half: `kali.json` IS in its `[source]` table",
        "and no argv below carries `--api`. Its sibling is",
        f"{EXPLICIT_STEM}.toml.",
        "Within this file `[source]` is safe in the ordinary way: the manifest is written",
        "for every one of these 12 fns unconditionally (the source's `if` is between the",
        "two HELPERS, not inside one), and every command names its entry on argv, so the",
        "unused fixture siblings in a trial dir are inert.",
    ]


def argv_order_block(half):
    lines = [
        "ARGV ORDER is transcribed in the exact order the source's `Command` builder",
        "appends it, and is not normalised:",
        f"  * build/check: `[--output json] <build|check> [--bundle] "
        f"{'' if half == 'inherited' else '--api browser '}<entry>`. The",
        f"             `--output json` pair is appended FIRST, before the subcommand",
        f"             (`cmd.arg(\"--output\").arg(\"json\")` (:{HELPER_CITE[PLAIN]['output_json']})),",
        f"             then the subcommand, then `--bundle` "
        f"(`cmd.arg(\"--bundle\")` (:{HELPER_CITE[PLAIN]['bundle_flag']}))",
        "             for `build` only.",
    ]
    if half == "explicit":
        lines += [
            "  * run/test: `[--output json] <run|test> --api browser <entry>`, with the",
            f"             harness command variable set on the child first "
            f"(`cmd.env(BROWSER_HARNESS_COMMAND_ENV, \"node\")` "
            f"(:{HELPER_CITE[REQUESTED]['env']})).",
            "             This shape never passes `--bundle`.",
            "  * `--api browser` is the last pair before the entry, on the statement that",
            f"             ends `.expect(\"run kali\")` (:{HELPER_CITE[PLAIN]['run_kali']}).",
        ]
    else:
        lines += [
            "  * NO `--api` pair at all: the inherited helpers append the entry directly,",
            f"             on the statement that ends `.expect(\"run kali\")` "
            f"(:{HELPER_CITE[INHERITED]['run_kali']}).",
            "             That absence is the case, not an omission.",
        ]
    lines += [
        "The source passes an ABSOLUTE path as the entry -- "
        f"`dir.path().join(filename)` (:{HELPER_CITE[PLAIN]['abs_path']}), and once per",
        "helper besides -- while the case runner passes the bare filename relative to the",
        "trial dir, matching every previously shipped `browser/` case file.",
    ]
    return lines


def assertion_shape_block(half):
    c = HELPER_CITE[PLAIN]
    return [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        "`exit = 1`, not the weaker `exit = \"failure\"` status class: the source pins",
        "the code exactly with",
        f"`assert_eq!(output.status.code(), Some(1))` (:{c['exit_code']}, and once per",
        "helper besides), which rule 1 makes an exact pin rather than a choice. The",
        f"companion `assert!(!output.status.success()` (:{c['success']}) is implied by it.",
        "In `--output json` mode the envelope's four fields are four exact `assert_eq!`",
        f"claims -- `assert_eq!(json[\"schemaVersion\"], 1)` (:{c['schema']}) through",
        f"`assert_eq!(json[\"exitCode\"], 1)` (:{c['exit_json']}) -- and become four exact",
        f"`json.*` pins; `assert_eq!(errors[0][\"code\"], \"E5506\")` (:{c['code']}) becomes",
        "`json.errors.0.code`.",
        f"The source's `!errors.is_empty()` (:{c['errors_nonempty']}) gets NO key of its",
        "own and is not dropped: design spec 5.4 makes an out-of-range numeric segment a",
        "HARD failure, never a silent skip, so `json.errors.0.code` cannot pass against an",
        "empty array and implies the claim.",
        "`errors[0][\"message\"]` is a JSON string leaf with no substring form in the format",
        "(there is no json_contains key), so its `.contains` claim becomes an exact",
        "`json.errors.0.message` pin per controller ruling 3 -- live-captured, per case,",
        "from the real built `kali` (see U9 below).",
        "In text mode: `stderr` HAS a substring form, so the two claims --",
        f"`stderr.contains(\"E5506\")` (:{c['stderr_code']}) and",
        f"`stderr.contains(\"literal array\")` (:{c['stderr_message']}) -- stay",
        "`stderr_contains` and are NOT strengthened to an exact `stderr` pin even though",
        "the exact text was observed: controller ruling 3, mirror the source.",
        "RULE 11 -- the three array-callback helpers accept EITHER message text",
        f"(`message.contains(\"array callback-produced iterables\")` "
        f"(:{HELPER_CITE[ARRCB]['message_or']}) `|| message.contains(\"literal array\")`",
        f"(:{HELPER_CITE[ARRCB]['message']}), and the same disjunction on `stderr`). The",
        "format has no disjunction, so the real binary decides: it emits the `literal",
        "array` branch for every one of these programs, and that branch is what is pinned.",
        "Every run satisfying the pin satisfies the source's OR, so this is a verified",
        "strengthening; the disjunction sentence is carried into every affected case's",
        "rationale rather than dropped.",
        "NOTHING ELSE IS ASSERTED. The source reads no stdout in text mode, no `payload`",
        "in either mode, no emitted file, and runs no harness process -- so there is no",
        "`stdout`/`stdout_contains`, no `file_json` step, no `browser_bundle_harness` step",
        "and no count key anywhere in this file. Adding any would be a rule 2 invention.",
    ]


def escalation_block():
    """Rule 3's escalation, recorded in the artifact and not only in the report."""
    or_rows = [r for r in ROWS if r["or_message"]]
    or_fns = {(r["sub"], r["fn"]) for r in or_rows}
    helpers = sorted({r["helper"] for r in or_rows})
    return [
        "ESCALATION (rule 3), RATIFICATION PENDING -- the audit's claim model was",
        "CONJUNCTIVE, and this file's rule-11 narrowing made it RED.",
        "`audit-case-migration.py` required EVERY `.contains` literal in the source to",
        "appear somewhere in the case files. This source asserts two DIFFERENT literals",
        "DISJUNCTIVELY (the array-callback message, above), and rule 11 resolves an OR",
        "against the real binary and pins the branch that actually occurs -- so the",
        "unpinned branch can never appear, and the audit failed a migration that is",
        "strictly STRONGER than its predecessor. The tool was asserting a conjunction the",
        "source never made.",
        "Rule 3 says escalate a claim the tool cannot see rather than shipping around it,",
        "and the ledger's own wording for the remedy is a TOOL FIX ('Never ship a file",
        "whose audit exits non-zero. Escalate for a tool fix instead.'). A per-file audit",
        "exception is permanently REJECTED (rule 4's evidence), so that was not an option.",
        "`disjunctive_contains_groups` was therefore added to `audit-case-migration.py`: a",
        "pure top-level `||` of `.contains` literals inside one `assert!` needs ONE member",
        "present, not all. It FAILS CLOSED -- a disjunct carrying a top-level `&&` forms no",
        "group at all, a single-distinct-literal OR (rule 11's own two-streams shape) forms",
        "no group, and if NO member appears every member is still reported missing -- and",
        "the resolution is PRINTED as a `DISJUNCTION` line on every run, so which branch",
        "was pinned is never silent. Seven tests in `scripts/audit-case-migration_test.py`",
        "pin both directions.",
        "THE CONTROLLER MAY PREFER THE OTHER DISPOSITION, and this is the sentence that",
        f"makes that choice visible from the artifact. Reverting the arm makes the "
        f"{len(or_fns)} `#[test]`",
        f"fns that reach {' / '.join('`' + h + '`' for h in helpers)}",
        f"({len(or_rows)} of the source's {len(ROWS)} invocations) a U4 trim-and-keep",
        "retention instead; nothing else in either case file changes.",
    ]


def u9_block(n_trials):
    return [
        "U9 -- LIVE VERIFICATION, per case and not a sample.",
        f"Every one of this file's {n_trials} trials was run against the real built `kali`",
        "by this file's generator before it was written: the generator writes the WHOLE",
        "`[source]` map into a fresh temp dir (the same set the case runner writes, so",
        "fixture interference between siblings would show up), runs the exact argv and env",
        "below, and re-checks the source's own predicates -- exit code 1, the four envelope",
        "fields, a non-empty `errors` array, `code == E5506`, and the message/stderr",
        "predicate INCLUDING the disjunction -- before any pin is emitted. The",
        "`json.errors.0.message` pins are the values that run produced; where a case is",
        "matrix-fanned, all cells were captured and asserted byte-identical to each other",
        "before one pin was written.",
        "A live run proves REALISM, not FIDELITY (U9): the source-vs-TOML direction is",
        "`audit-case-migration.py`, `check_fixtures.py` and `check_extra_claims.py`.",
    ]


def rule13_block():
    docs = (
        "RULE 13 -- transitive helper docs. Checked every fn in each call chain "
        f"({CHAIN_CHECKED} of them):"
    )
    setters, documented = contract_doc_precedent()
    return [docs] + P._wrap_list(
        CHAIN_FNS, "-- none carries a `///` doc comment, asserted mechanically in "
                   "gen_batch6b.py's doc-comment check (it walks back from each fn's "
                   "signature line and raises on a `///` above it), not read by eye."
    ) + [
        "The chain reaches no `kali_common` helper and no `browser_bundle_harness` step,",
        "so ruling 6's runner-infrastructure exemption is not in play here at all -- this",
        "file's cases never build a harness script or a harness command.",
        "One `///`-documented item IS touched: the constant",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`, whose doc reads",
        f"\"{CONTRACT_DOC}\" and whose value is the",
        f"`{HARNESS_ENV}` key the run/test cases set as `env`.",
        "It is a `pub const &str`, not a helper in a call chain, and rule 13 is about doc",
        "comments on HELPERS; the case reproduces its value, not anything it computed.",
        "Shipped precedent agrees and was ENUMERATED before this sentence was written",
        "(ruling 13), not recalled:",
        f"  grep -rl \"{HARNESS_ENV}\" crates/kali_cli/tests/cases/ \\",
        f"    | grep -vc {STEM}          -> {setters}",
        f"  grep -rl \"{CONTRACT_DOC}\" \\",
        f"    crates/kali_cli/tests/cases/ | grep -vc {STEM}   -> {documented}",
        f"{documented} of the {setters} case files that set that variable carry the doc line;",
        "this file follows them. The `grep -v` is not decoration: both strings occur in",
        "THIS header, so an unfiltered count would count the sentence's own file and the",
        "figure would move by being written (ruling 11).",
    ]


def extra_ok_block(entries, messages):
    lines = list(P.EXTRA_CLAIM_PREAMBLE)
    for value in sorted(messages):
        lines.append(P.extra_ok(value, P.EXTRA_OK_JSON_STDOUT.replace(
            "`json.stdout`", "`json.errors.0.message`").replace(
            'json["stdout"]', 'errors[0]["message"]')))
    for value in sorted(entries):
        lines.append(P.extra_ok(value, P.EXTRA_OK_U5_RENAME))
    return lines


# --------------------------------------------------------------------------
# 8. Build both files.
# --------------------------------------------------------------------------

def source_map(rows, *, templated):
    """The `[source]` table, in fixture-declaration order then js/ts/jsx/tsx."""
    out = {}
    order = list(FIXTURE_STEMS)
    for fixture in order:
        used = [r for r in rows if r["fixture"] == fixture]
        if not used:
            continue
        if templated:
            out[f"{FIXTURE_STEMS[fixture]}.${{ext}}"] = FIXTURES[fixture]
            continue
        names = sorted({newname(fixture, r["filename"]) for r in used},
                       key=lambda n: (".test." in n, EXTS4.index(n.rsplit(".", 1)[1])))
        for name in names:
            out[name] = FIXTURES[fixture]
    return out


def build_file(half):
    rows = INHERITED_ROWS if half == "inherited" else EXPLICIT_ROWS
    matrix_fold = half == "inherited"
    sources = source_map(rows, templated=matrix_fold)
    if matrix_fold:
        sources["kali.json"] = MANIFEST

    # U5's safety condition, CHECKED (not asserted in prose): no fixture body
    # names any `[source]` key by string, so the rename is argv-only.
    P.assert_rename_is_argv_only(sources, [k for k in sources if k != "kali.json"], EXTS4)

    built = build_cases(rows, matrix_fold=matrix_fold, sources=sources)
    cases, entries, messages = [], set(), set()
    for name, row, entry, cell in built:
        pins = []
        for member in cell:
            concrete = entry
            if matrix_fold:
                concrete = entry.replace("${ext}", member["filename"].rsplit(".", 1)[1])
            concrete_sources = {
                k.replace("${ext}", member["filename"].rsplit(".", 1)[1]): v
                for k, v in sources.items()}
            entries.add(concrete)
            pins.append(live(member, concrete, concrete_sources))
        message = None
        if row["json"]:
            message = P.assert_identical(f"{name}: json.errors.0.message across cells",
                                         *pins)
            messages.add(message)
        cases.append({
            "name": name,
            "rationale": rationale(row, entry, message, matrix_fold=matrix_fold),
            "steps": [step_for(row, entry, message)],
        })

    n_fns = len({(r["sub"], r["fn"]) for r in rows})
    n_inv = len(rows)
    if matrix_fold:
        arithmetic = P.matrix_arithmetic(
            test_fns=n_fns, invocations=n_inv, cases=len(cases), axis="ext",
            values=EXTS4, non_axes=("json_output",),
            helpers=[
                (INHERITED,
                 len([r for r in rows if r["helper"] == INHERITED]),
                 "5 `#[test]` fns per command x json_output(false/true) x "
                 "ext(js/ts/jsx/tsx), a complete cross product"),
                (ARRCB_INH,
                 len([r for r in rows if r["helper"] == ARRCB_INH]),
                 "1 `#[test]` fn per command x filename(js/ts/jsx/tsx) x "
                 "json_output(false/true), a complete cross product"),
            ])
        mapping = P.rule6_matrix_fold(
            "exactly ONE source `#[test]` fn's `json_output` half, fanned over that fn's "
            "own `for filename in [...]` loop")
    else:
        arithmetic = P.matrix_declined(
            test_fns=n_fns, invocations=n_inv, cases=len(cases),
            reason=[
                "68 of those invocations are individual, unlooped helper calls whose "
                "extension",
                "coverage is DELIBERATELY PARTIAL and differs per program: `for_of_source` is "
                "tested",
                "on js and jsx only, `for_await_source` on ts and tsx only, "
                "`object_values_source` on",
                "js only, while `object_keys_source` and `object_entries_source` cover all "
                "four.",
                "The remaining 56 do vary uniformly over ext(4), but `[matrix]` is FILE-WIDE.",
            ])
        mapping = P.RULE6_ONE_TO_ONE + [
            "Where one source fn expands into several invocations through its own `for` "
            "loops,",
            "each invocation is its own named sibling (rule 5: N independent programs -> N",
            "sibling cases), and the case name carries the loop coordinate that tells them "
            "apart.",
        ]

    header = []
    header += extra_ok_block(entries, messages)
    header += [
        f"Migrated from tests/browser_{STEM}.rs -- the "
        + ("MANIFEST-INHERITED" if matrix_fold else "EXPLICIT `--api browser`") + " half.",
        "",
    ]
    header += u10_block(half, n_fns, n_inv)
    header += [""]
    header += rule12_no_comments_prose(CARRIER, STEM).split("\n")
    header += [""]
    header += arithmetic
    header += [""]
    header += mapping
    header += [""]
    header += u2_block(half)
    header += [""]
    renames = []
    for f in FIXTURE_STEMS:
        used = [r for r in rows if r["fixture"] == f]
        if not used:
            continue
        originals = sorted({r["filename"] for r in used})
        if matrix_fold:
            new_keys = [f"{FIXTURE_STEMS[f]}.${{ext}}"]
            originals = [f"main.${{ext}}"]
        else:
            new_keys = sorted({newname(f, r["filename"]) for r in used})
        renames.append((
            "` / `".join(originals),
            "` / `".join(new_keys),
            f"the program `{f}()` (:{FIXTURE_CITE[f]}) writes"))
    header += P.u5_renames(
        renames,
        collision=(f"{len(FIXTURE_STEMS)} different program texts to "
                   f"{SOURCE_FILENAME_PHRASE}"))
    header += [""]
    dupe_groups = sum(
        1 for f in FIXTURE_STEMS
        if len([k for k, v in sources.items() if v is FIXTURES[f]]) > 1)
    if dupe_groups:
        header += P.RULING7_NO_HOIST
        header += [
            "In this file that check runs on the RENDERED TOML, re-parsed: each of the",
            f"{dupe_groups} duplicate-bodied `[source]` group(s) below must be "
            "byte-identical,",
            "and every `[source]` value must match exactly one `fn *_source()` literal in",
            "the carrier -- so an emitter or escaping bug that wrote the wrong literal fails",
            "the generator rather than shipping. Asserting it over the generator's in-memory",
            "dict instead would be vacuous: every key there is filled from the same object,",
            "so equality would hold by construction and the check could never fail.",
        ]
    else:
        header += [
            "RULING 7 -- NO DUPLICATE `[source]` BODIES IN THIS FILE, so U13's hoist question",
            "does not arise: the `ext` axis collapses what would be four keys per program",
            "into one `${ext}`-templated key, and every `[source]` value below is distinct.",
            "The identity half of the check still runs on the RENDERED TOML, re-parsed --",
            "every `[source]` value must match exactly one `fn *_source()` literal in the",
            "carrier, and `kali.json` must match the manifest literal the two inherited",
            "helpers embed -- so an emitter or escaping bug fails the generator rather than",
            "shipping.",
        ]
    header += [""]
    header += rule13_block()
    header += [""]
    header += argv_order_block(half)
    header += [""]
    header += assertion_shape_block(half)
    header += [""]
    header += escalation_block()
    header += [""]
    header += u9_block(len(rows))

    matrix = {"ext": EXTS4} if matrix_fold else None
    text = emit(header, matrix, sources, cases)
    dupes = assert_ruling7_identity(text)
    if dupes != dupe_groups:
        raise AssertionError(
            f"header states {dupe_groups} duplicate-bodied [source] group(s), the "
            f"rendered file has {dupes}")
    return text, len(cases), n_fns, n_inv, dupes


def assert_ruling7_identity(text):
    """Ruling 7's MANDATORY duplicate-identity assertion, on the ARTIFACT.

    Ruling 7 declines U13's `[constants]` hoist for `browser/` but makes the
    identity assertion mandatory: "duplication without a check is just
    duplication". Asserting it over the generator's own in-memory dict would be
    vacuous -- every key is filled from the same Python object, so equality
    holds by construction and the check could never fail. So it runs against the
    RENDERED TOML, re-parsed: every `[source]` value must be byte-identical to
    the `fn *_source()` literal it claims to be, and duplicate-bodied keys must
    therefore agree with each other. That can fail -- an emitter or escaping bug
    is exactly what it catches, and it is the same defect class
    `check_fixtures.py` was written for after batch 4 shipped a fixture that was
    an unrelated `.expect()` message.
    """
    import tomllib
    doc = tomllib.loads(text)
    groups = {}
    for key, body in (doc.get("source") or {}).items():
        if key == "kali.json":
            if body != MANIFEST:
                raise AssertionError("emitted kali.json is not the carrier's literal")
            continue
        matches = [fn for fn, lit in FIXTURES.items() if lit == body]
        if len(matches) != 1:
            raise AssertionError(
                f"[source] {key!r} matches {len(matches)} fixture literal(s) in the "
                "carrier, wanted exactly 1")
        groups.setdefault(matches[0], []).append(key)
    for fn, keys in groups.items():
        P.assert_identical(f"{fn} across {sorted(keys)}",
                           *[doc["source"][k] for k in keys])
    return sum(1 for keys in groups.values() if len(keys) > 1)


def main(argv):
    total_cases = total_fns = total_inv = 0
    for half, stem in (("explicit", EXPLICIT_STEM), ("inherited", INHERITED_STEM)):
        text, n_cases, n_fns, n_inv, dupes = build_file(half)
        write(os.path.join(CASES, f"{stem}.toml"), text)
        print(f"  {half}: {n_fns} #[test] fns, {n_inv} invocations, {n_cases} cases, "
              f"{dupes} duplicate-bodied [source] group(s) checked")
        total_cases += n_cases
        total_fns += n_fns
        total_inv += n_inv
    if total_fns != TESTS_TOTAL or total_inv != len(ROWS):
        raise AssertionError(
            f"the two files account for {total_fns} fns / {total_inv} invocations, "
            f"but the source has {TESTS_TOTAL} / {len(ROWS)}")
    print(f"TOTAL: {total_fns} #[test] fns, {total_inv} invocations, "
          f"{total_cases} cases -- arithmetic closes")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
