#!/usr/bin/env python3
r"""Generate the Task 18 batch 8B case files -- the last 13 browser targets.

THIRTEEN TARGETS, TWENTY CASE FILES, 375 TRIALS. The file count exceeds the
target count because six targets write a `kali.json` manifest behind an `if` and
U2 forces those into two files apiece; the four `#[path]` submodule carriers, by
contrast, each take ONE file, and that is this batch's structural finding.

=========================================================================
THE SECTION-2 DERIVATION: CAN `run` AND `test` SHARE A CASE FILE?
=========================================================================

ANSWER: YES, for all four `runtime_summary_fallback_*` carriers, and it is
measured rather than argued. The measurement is `--derive`, committed here, run
against the real binary.

The hazard is U2's: `[matrix]` and `[source]` are FILE-WIDE, so folding two
halves that assert on the presence or absence of different things silently stops
one of them discriminating -- and a case that asserts nothing still passes, so
neither `audit-case-migration.py` nor `cargo test` reports it.

Derived from what the MIGRATED cases assert (not from what the source family
asserts somewhere -- batch 8A's first answer was wrong for exactly that reason,
and here BOTH halves are migrated, so the derivation covers every claim):

  1. NEITHER HALF WRITES A CONDITIONAL FIXTURE. Every `#[test]` fn in both
     submodules writes exactly one file, unconditionally, with
     `write_<ext>_source`. There is no manifest, no policy file, no lockfile and
     no sibling module anywhere in these four targets -- so there is no fixture
     whose presence or absence could be leaked across the fold. Derived by the
     extractor, which raises if any fn writes other than exactly one `.join(...)`
     filename.

  2. THE TWO HALVES' FILENAMES DO NOT COLLIDE. `run` writes `<case>.<ext>`,
     `test` writes `<case>.test.<ext>`. The union is therefore a disjoint merge,
     not an overwrite, and every body in it is byte-identical (asserted
     mechanically -- ruling 7's mandatory half).

  3. THE OBSERVABLE BEHAVIOUR IS UNCHANGED BY THE MERGE. For each of the 117
     `#[test]` fns, the real binary was run three times on the same argv and
     environment: in a directory holding only that fn's own fixture (what the
     source's `tempdir` held), in one holding its own half's fixtures, and in
     one holding BOTH halves' fixtures (what a shared file-wide `[source]`
     produces). Modulo `payload.runtimeMs`, which nothing pins:

         117 tests probed; 0 whose output differs

     Every command names its entry explicitly on argv and neither `kali run` nor
     `kali test` picks up a sibling by discovery, so the unused fixtures are
     inert.

  4. THE PROBE IS NOT VACUOUSLY GREEN. A zero nobody tried to make non-zero is
     worth nothing, so the same instrument is run against a known positive: add
     a `kali.json` to the trial dir and drop the flag under test, and it reports
     DISARMED for these very cases. The zero in (3) is therefore a statement
     about the actual merge, not about the instrument.

THE FOUR CARRIERS DO NOT ALL LOOK ALIKE AND ARE NOT ASSUMED TO. `ts_input` is
13 + 14 where its three siblings are 14 + 14; `js_input` is 17 + 17 and its
carrier declares two fewer helpers than the other three (its `parse_json_stdout`
asserts `errors` empty, theirs does not). Every count and every helper claim set
below is read out of each carrier separately.

=========================================================================
THE SIX MANIFEST SPLITS, ALSO DERIVED, AND THEY DO *NOT* ALL RESOLVE ALIKE
=========================================================================

`--derive` asks one question per candidate: with the other half's `kali.json`
present (which is what a shared file-wide `[source]` produces) and the flag under
test REMOVED, does the command still produce every value the case pins? If yes,
the flag is unverified and the halves must split.

    runtime_wasm_threads_js_input              DISARMED  -> split
    wasm_threads_max_threads_harness           DISARMED  -> split
    wasm_threads_browser_surface               DISARMED  -> split
    runtime_spawned_process_budget_js_input    DISARMED  -> split
    runtime_sandbox_* (JSON cases)             still discriminates
    runtime_sandbox_* (TEXT cases)             DISARMED  -> split

The sandbox row is the one that would have been got wrong by deriving from the
wrong subset. Its JSON cases pin `errors[0].context.origin = "cli"`, which a
manifest cannot supply -- with the manifest and no flag the binary emits
`"config"` -- so on the JSON cases alone the answer is "no split needed". Its
TEXT cases pin only stderr substrings, all of which the manifest DOES supply, so
they are disarmed. `[source]` is file-wide and has no per-case opt-out (U2), so
one disarmed case in a file disarms that file: all three sandbox targets split.

=========================================================================
WHAT THIS BATCH COULD NOT EXPRESS, STATED PLAINLY
=========================================================================

`#[cfg(unix)]`. Eight `#[test]` fns -- one in each `runtime_summary_fallback_*`
submodule -- are compiled only on unix, because they `chmodSync(0o000)` the
summary file and expect the read to fail. The case format has no platform key
(design spec 5.4 lists twelve; none of them gates compilation), and `ignore =
true` would disable the case on unix too, which weakens it (rule 1). They are
migrated UNCONDITIONALLY, and the report escalates the choice rather than
burying it. The three reasons, in order:

  * nothing is weakened -- every claim the source makes on unix, the case makes
    on unix;
  * the suite has no non-unix lane. `grep -n 'runs-on\|os:' .github/workflows/ci.yml`
    lists `ubuntu-latest` and `macos-latest` and nothing else, so the added
    claim is never evaluated anywhere the suite runs;
  * the alternative is a U4 trim of all four `#[path]` carriers immediately
    before 8C's single irreversible family-wide deletion, to gate a claim that
    no environment the suite runs in can distinguish.

The alternative is fully specified in the report (which 8 cases to drop and
which four retention headers to add) rather than rendered, because rendering it
would double four files for a disposition that is the controller's to make.

`env_remove`. Eight sandbox `#[test]` fns call
`.env_remove(BROWSER_HARNESS_COMMAND_ENV)`. The runner sets a step's `env` on top
of the inherited environment and never clears it, so a case file can SET a
variable but cannot UNSET one. Those cases therefore carry no `env` key. That is
not a weakening of any assertion, and the reason is measured, not assumed: with
the variable unset, set to `node`, and set to `"   "`, the binary emits the same
exit code, the same `E5506`, the same `context.origin` and the same message --
the rejection precedes any use of the harness command. The residual is that in a
shell that exports the variable, those cases stop being distinguishable from
their `when_browser_harness_is_configured` siblings; they still assert
everything the source asserted. Reported.

=========================================================================
HOW THE CLAIMS GET HERE
=========================================================================

Nothing in this file transcribes an assertion. `batch8b_extract` reads every
argv token, `.env(...)` value, `.join("...")` filename and `assert*` claim out of
the `.rs`; `batch8b_claims` resolves the `if json_output` / `if command == "run"`
branches under the binding the call site supplies, and RAISES on a condition it
cannot evaluate rather than letting both branches contribute. `batch8b_capture`
supplies every pinned value by running the real binary, and refuses to serve a
capture whose recorded argv/env/fixtures no longer match what the source
derives.

RULE 11 / RULING 17. `browser_wasm_threads_browser_surface` makes the batch's
one OR-shaped assertion: `stderr.contains("runtime profile") ||
stderr.contains("wasm-threads")`. Resolved against the real binary over all 16
cells: BOTH disjuncts are true on every cell, and the cells agree. Ruling 17
therefore applies -- pin the first in source order (`runtime profile`), disclose
the other, and do NOT pin both, because pinning a disjunct the source never
asserted unconditionally is a rule-2 invention. The generator derives the group
from the source's `||` rather than being told, and raises if the pinned needle
is not the first one the source spells.

RULING 3. A plain `.contains` against raw stdout/stderr keeps `*_contains`. A
plain `.contains` against a `json` string leaf becomes an exact pin, because the
format has no json-substring key -- and the generator asserts the captured value
actually contains the source's needle before pinning it, so the strengthening is
verified rather than asserted.

RULING 7. Duplicate `[source]` bodies are NOT hoisted into `[constants]`; the
identity is asserted mechanically instead.

RULING 16. No case file states a family-wide population count.

Run: python3 gen_batch8b.py [stem ...]      (no args = all 20 files)
     python3 gen_batch8b.py --derive        (the U2 / disarmament measurements)
     python3 gen_batch8b.py --recapture     (re-run every live capture)
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

import batch5_prose as P  # noqa: E402
from case_emit import emit, write, source_text  # noqa: E402
from batch8b_capture import Captures  # noqa: E402
from batch8b_claims import claims_for  # noqa: E402
from batch8b_extract import (  # noqa: E402
    HARNESS_ENV, claims_in, comment_blocks, doc_owner, fn_body, helper_claims,
    literals, prose, summary_fallback_rows, test_fns,
)

EXTS = ["js", "ts", "jsx", "tsx"]
REGISTRY = {}
CAPS = None


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    """The source a case file is generated FROM.

    Batch 8B declares no U4 trim-and-keep retention, so `source_text`'s
    `PRE-TRIM REF:` branch is not reached for any target here. Checked rather
    than assumed, so a later trim cannot silently regenerate a smaller case file
    from a trimmed source.
    """
    text = source_text(name, quiet=True)
    if text.startswith("//!"):
        raise AssertionError(
            f"browser_{name}.rs has grown a `//!` retention header. Batch 8B's targets "
            "are plain migrations; regenerate against the intended blob deliberately.")
    return text


def cited(rs_text, needle, *, expect=1, pick=0, file=None):
    """`` `needle` (:N) `` -- the gated citation form, searched not computed."""
    n = P.cite_line(rs_text, re.escape(needle), label=needle, expect=expect)
    if expect > 1:
        n = n[pick]
    where = f"{file}:{n}" if file else f":{n}"
    return f"`{needle}` ({where})"


def cited_fn(rs_text, name, *, file=None):
    """A citation at `fn <name>`'s declaration that the gate can RE-RESOLVE.

    The obvious snippet, `` `fn write_js_source(` ``, yields NO needle:
    `batch5_crosscheck._distinctive` takes the snippet's leading identifier and
    requires four characters, and the leading identifier here is `fn`. The
    citation is then "matched but yielding NO NEEDLE" -- reported clean whether
    it is right or wrong, which is the exact class ruling 11 exempts `:N` from
    the no-moving-figures rule only because it does NOT have.

    Batch 8A's answer was to raise `NO_NEEDLE_DECLARED`. The better answer is
    available here: drop the `fn ` and cite `` `write_js_source(` ``, which
    yields the fn name as a word-bounded needle and resolves against the
    declaration statement. The LINE is still derived from the unambiguous
    `fn <name>(` form, so the pointer cannot bind to a call site.
    """
    n = P.cite_line(rs_text, re.escape(f"fn {name}("), label=f"fn {name}(")
    where = f"{file}:{n}" if file else f":{n}"
    return f"`{name}(` ({where})"


# --------------------------------------------------------------------------
# claims -> step keys
# --------------------------------------------------------------------------


def _fold_exit(claims):
    codes = [c[1] for c in claims if c[0] == "exit_code"]
    succ = [c[1] for c in claims if c[0] == "exit_success"]
    if len(set(codes)) > 1:
        raise AssertionError(f"two different exit codes asserted: {codes}")
    if codes:
        code = codes[0]
        if succ and succ[0] and code != 0:
            raise AssertionError(f"source asserts success AND code {code}")
        return "success" if code == 0 else code
    if succ:
        return "success" if succ[0] else "failure"
    raise AssertionError("no exit claim at all -- a case that asserts nothing still passes")


def build_step(claims, argv, env, *, caps=None, cap_key=None, files=None,
               or_pins=None):
    """One step, with every §5.4 key derived from the resolved claim list."""
    step = {"args": list(argv)}
    if env:
        step["env"] = dict(env)
    step["exit"] = _fold_exit(claims)

    groups = [c[1] for c in claims if c[0] == "or_group"]
    dropped = []
    for group in groups:
        # Ruling 17: pin the FIRST disjunct in source order, disclose the rest.
        if or_pins is not None and group[0] not in or_pins:
            raise AssertionError(
                f"ruling 17: the pinned needle for {group!r} must be its first member")
        dropped.extend(group[1:])

    sc = [c[1] for c in claims if c[0] == "stdout_contains"]
    if sc:
        step["stdout_contains"] = sc
    se = [c[1] for c in claims if c[0] == "stderr_contains" and c[1] not in dropped]
    if se:
        step["stderr_contains"] = se
    if any(c[0] == "stderr_exact" for c in claims):
        step["stderr"] = ""
    if any(c[0] == "stdout_empty" for c in claims):
        step["stdout"] = ""
        PINNED.append(("empty-stdout", "stdout", ""))

    paths = {}
    for c in claims:
        if c[0] == "json":
            if c[2] == "<json-macro>":
                raise AssertionError(
                    "a `serde_json::json!` value reached emission unparsed; it must be "
                    "parsed, never defaulted")
            paths[c[1]] = c[2]
    if any(c[0] == "errors_empty" for c in claims):
        paths["errors"] = []
    for c in claims:
        if c[0] != "json_contains":
            continue
        if caps is None:
            raise AssertionError("a json-leaf `.contains` needs a live capture")
        observed = caps.json_leaf(cap_key, files, argv, env, c[1])
        PINNED.append(("json-leaf", c[1], observed))
        if not isinstance(observed, str) or c[2] not in observed:
            raise AssertionError(
                f"{cap_key}: the captured json.{c[1]} does not contain the source's "
                f"needle {c[2]!r}; the exact pin would NOT be a strengthening")
        if c[1] in paths and paths[c[1]] != observed:
            raise AssertionError(f"{cap_key}: two different pins for json.{c[1]}")
        paths[c[1]] = observed
    if any(c[0] == "errors_nonempty" for c in claims):
        if not any(p.startswith("errors.0") for p in paths):
            raise AssertionError(
                "source asserts `!errors.is_empty()` but nothing pins an errors[0] path, "
                "so the non-emptiness claim would be dropped")
    if paths:
        step["json_paths"] = paths
    return step, dropped


def _assert_pin_uniform(label, values):
    """A matrix-folded case carries ONE pin, so the pin must be ext-invariant."""
    first = values[0]
    for v in values[1:]:
        if v != first:
            raise AssertionError(
                f"{label}: the live value is not identical across the matrix axis "
                f"({v!r} vs {first!r}); one pin cannot stand for all of them")
    return first


# --------------------------------------------------------------------------
# shared header pieces
# --------------------------------------------------------------------------


def ruling3_json_leaf(paths):
    """Ruling 3's json-leaf sentence, naming the leaves THIS file pins.

    `batch5_prose.ruling3_json_leaf()` hardcodes `json["stdout"]`, which is the
    leaf eleven of this batch's twenty files pin -- and is simply the wrong
    field name on the rejection files, which pin `errors[0].message`. A shared
    sentence naming a field the file does not have is ruling 18's failure class,
    so the leaf list is derived from the emitted claims.
    """
    named = ", ".join(f"`json.{p}`" for p in sorted(set(paths)))
    return (
        f"On the JSON branch the source spells its claim as `.contains(...)` against the "
        f"string leaf {named}, which has NO substring form in this case format (the twelve "
        "keys of design spec 5.4 include no json-substring key), so per controller ruling 3 "
        "it becomes an exact pin -- and, per U9, only after the value was captured from the "
        "real `kali` binary rather than hand-computed. The generator refuses to emit a pin "
        "that does not contain the source's own needle, so the strengthening is verified "
        "rather than asserted.")


def ruling7_block(source):
    """Ruling 7's no-hoist paragraph, or the honest `nothing to hoist` variant.

    Derived from the emitted `[source]` map: a file with one entry has no
    duplicate to decline hoisting, and saying otherwise describes a state the
    file does not have.
    """
    bodies = list(source.values())
    if len(bodies) > 1 and len(set(bodies)) < len(bodies):
        return P.RULING7_NO_HOIST
    if len(bodies) > 1:
        return [
            "RULING 7 -- NOTHING TO HOIST. The `[source]` entries below are all DIFFERENT",
            "programs (checked mechanically by this file's generator, which raises if any two",
            "entries would carry the same key with different bodies), so U13 has no duplicated",
            "body to hoist and controller ruling 7's decline does not arise.",
        ]
    return [
        "RULING 7 -- NOTHING TO HOIST. This file has a single `[source]` entry, so there is no",
        "duplicated body for U13 to hoist and no identity to assert.",
    ]


def extra_ok_block(renames=()):
    """The `# EXTRA-OK:` declarations, DERIVED from what this file just pinned.

    Ruling 18 in miniature: a hand-written declaration list is prose that has to
    be kept in step with the emitted claims, and nothing keeps it there. So the
    list is built from `PINNED`, which is appended to at the exact moment a pin
    is taken from a capture, and from the U5 rename table, which is built by
    detecting a real collision. A file that stops pinning something stops
    declaring it in the same run.
    """
    entries = []
    for kind, path, value in PINNED:
        if kind == "json-leaf":
            entries.append((value, P.EXTRA_OK_JSON_STDOUT if path == "stdout" else (
                f"live-captured exact `json.{path}` pin; the source asserts `.contains` on a "
                "JSON leaf, which has no substring form, so ruling 3 requires an exact pin "
                "captured from the real binary")))
        elif kind == "empty-stdout":
            # `is_empty` is named PLAINLY, not backticked: U8's gate
            # (`check_rationale_fn_names.py`) resolves every backticked
            # lower-case identifier against the source's own fn list, and this
            # one is a std method, not a fn in the `.rs`. Backticking it turned
            # that gate red on both bundle pairs the moment this declaration
            # started rendering.
            entries.append((value,
                            "the exact-empty `stdout` pin on the browser-bundle harness step. "
                            "The source spells it as an is_empty check on the harness "
                            "process's stdout, which carries no literal for the extractor to "
                            "find, and no substring key can express \"nothing at all\" (rule 1)"))
    for _, new, _ in renames:
        for ext in EXT_ORDER:
            entries.append((new.replace("${ext}", ext), P.EXTRA_OK_U5_RENAME))
    if not entries:
        return []
    seen, lines = set(), list(P.EXTRA_CLAIM_PREAMBLE)
    for value, why in entries:
        if value in seen:
            continue
        seen.add(value)
        lines.append(P.extra_ok(value, why))
    return lines


RULING16_NOTE = [
    "No count of the wider `browser/` corpus appears anywhere in this file (ruling 16): a",
    "family-wide population count has no gateable home inside a case file, and the",
    "remaining batches would falsify it by construction.",
]

def argv_order(cases):
    """The ARGV ORDER section, with the `--output json` position DERIVED.

    This was a constant that said `--output json` precedes the subcommand. True
    for eleven of this batch's twenty files and FALSE for the two bundle files,
    whose source appends the flag inside its `if json_output` branch -- after
    the fixed prefix. A shared sentence describing a position the file does not
    have is ruling 18's failure class, so the position is read off the emitted
    argv instead, and a file whose json cases disagree with each other raises.
    """
    positions = set()
    for case in cases:
        for step in case["steps"]:
            args = step.get("args", [])
            if "--output" not in args:
                continue
            i = args.index("--output")
            positions.add("first" if i == 0 else "after the subcommand and its flags")
    lines = [
        "ARGV ORDER. `args` reproduces the source's own argument order exactly, token for",
        "token, and the order is DERIVED from the source's `.arg(...)` chain by this file's",
        "generator rather than written down. The entry filename is last, as the source",
        "appends it last.",
    ]
    if len(positions) > 1:
        raise AssertionError(
            f"this file's json cases disagree about where `--output json` sits: {positions}")
    if positions:
        where = positions.pop()
        lines.append(
            "`--output json` appears " + where + " in the json cases below, because that is "
            "where the source builds it.")
    else:
        lines.append("No case in this file passes `--output json`.")
    return lines


def u2_split_header(*, this_half, other_stem, this_fns, this_invocations,
                    disarmed_shape, measurement):
    return [
        "U2 -- `[source]` is FILE-WIDE, WHICH IS WHY THIS TARGET IS TWO CASE FILES.",
        "The source writes `kali.json` behind an `if with_browser_api_surface_manifest`.",
        "`expand.rs` clones the whole file-level `[source]` map into EVERY trial and a",
        "`[[case]]` cannot opt out of a fixture a sibling case needs, so one shared file would",
        "make the manifest unconditionally present. That is invisible to",
        "`audit-case-migration.py` (no literal is dropped) and invisible to `cargo test` (the",
        "trial still passes), so it is measured instead:",
    ] + measurement + [
        f"This file is the {this_half} half: {this_fns} source `#[test]` fn(s) worth of",
        f"invocations ({this_invocations} of them). The other half is `{other_stem}`.",
        f"The disarmed shape, stated so a reader can check the split is on the right line: "
        f"{disarmed_shape}",
        "Every command below names its entry explicitly on argv, so the unused sibling",
        "fixtures in a trial dir are inert.",
    ]


def rule13_block(docs):
    if not docs:
        return [
            "RULE 13 -- transitive helper docs. This source's helper chain carries NO `///`",
            "doc comment on any helper any case reaches (checked mechanically by this file's",
            "generator, which raises if the count it accounts for is wrong), so there is none",
            "to carry. Controller ruling 6's runner-infrastructure exemption is separate and is",
            "not relied on here.",
        ]
    lines = [
        "RULE 13 -- transitive helper docs, carried verbatim into every rationale whose call",
        "chain reaches the documented helper (bottom-up, U6: never pooled, never",
        "over-attributed):",
    ]
    for owner, text in docs:
        lines.append(f"  * {owner} -- \"{text}\"")
    return lines


# ==========================================================================
# THE FOUR `#[path]` SUBMODULE CARRIERS -- one case file each
# ==========================================================================


SF_STEMS = [f"runtime_summary_fallback_{e}_input" for e in ("js", "jsx", "ts", "tsx")]


def build_summary_fallback(stem):
    PINNED.clear()
    rows, carrier, subs, writer, fixture = summary_fallback_rows(stem)
    ext = stem[len("runtime_summary_fallback_"):-len("_input")]

    # ruling 7: the duplicate `[source]` bodies are asserted identical, not eyeballed.
    P.assert_identical(f"{stem} fixture bodies", *[r["fixture"] for r in rows])
    source = {}
    for r in rows:
        if r["file"] in source and source[r["file"]] != fixture:
            raise AssertionError(f"{stem}: {r['file']} would carry two different programs")
        source[r["file"]] = fixture
    if "${" in fixture:
        raise AssertionError("rule 10: this fixture carries a genuine `${` and needs escaping")

    run_names = sorted({r["file"] for r in rows if r["half"] == "run"})
    test_names = sorted({r["file"] for r in rows if r["half"] == "test"})
    if set(run_names) & set(test_names):
        raise AssertionError(
            f"{stem}: the two halves' fixture filenames collide, so the fold would overwrite "
            "one half's program with the other's")

    # rule 13: `///` docs on the carrier's helpers, attributed to the cases that reach them.
    docs = {}
    for block in comment_blocks(carrier, "///"):
        docs[doc_owner(carrier, block[0])] = prose(block)
    plain = comment_blocks(carrier, "//")
    if plain:
        raise AssertionError(
            f"{stem}: carrier has {len(plain)} bare `//` comment block(s) this generator "
            "does not carry (rule 12)")
    for rel in subs:
        sub_text = open(os.path.join(TESTS, rel)).read()
        if comment_blocks(sub_text, "//") or comment_blocks(sub_text, "///"):
            raise AssertionError(f"{rel}: has Rust comments this generator does not carry")

    cfg_rows = [r for r in rows if any("cfg" in a for a in r["attrs"])]

    cases = []
    for r in rows:
        claims = helper_claims(carrier, r["helper"]) + r["claims"]
        # Keyed on the repr: a parsed `serde_json::json!` value is a dict or a
        # list, so the claim tuple is not hashable.
        seen, merged = set(), []
        for c in claims:
            if repr(c) not in seen:
                seen.add(repr(c))
                merged.append(c)
        env = {HARNESS_ENV: r["harness"]}
        key = f"{stem}::{r['fn']}"
        step, _ = build_step(merged, r["argv"], env, caps=CAPS, cap_key=key,
                             files=source)
        cases.append({
            "name": r["fn"],
            "rationale": _sf_rationale(stem, r, merged, carrier, docs, cfg_rows),
            "steps": [step],
        })

    names = [c["name"] for c in cases]
    if len(set(names)) != len(names):
        raise AssertionError(f"{stem}: duplicate `[[case]]` name(s)")

    header = [
        f"Migrated from tests/browser_{stem}.rs AND its two `#[path]` submodules",
        f"({', '.join('tests/' + s for s in subs)}). U10: the sibling directory is part of",
        "the unit, so all of it lands in this one file.",
        "",
    ]
    header += _sf_rule12_block(stem, docs, subs)
    header += [""]
    header += P.matrix_declined(
        test_fns=len(rows), invocations=len(rows), cases=len(cases),
        reason=[
            "This carrier covers exactly ONE input extension "
            f"(`.{ext}`), so there is no `ext`",
            "dimension to hoist: every fixture in it already ends `." + ext + "`. The two",
            "dimensions that DO vary -- the subcommand (run/test) and the output mode -- change",
            "the ASSERTION SHAPE rather than substituting a string, and the fixture NAME differs",
            "between them as well (`<case>." + ext + "` for run, `<case>.test." + ext + "` for test),",
            "so neither is a substitutable axis. Each case additionally carries its own",
            "`KALI_BROWSER_BUNDLE_HARNESS_COMMAND` program, which is per-case data and not an",
            "axis value.",
        ])
    header += [""]
    header += P.RULE6_ONE_TO_ONE
    header += [""]
    header += _sf_u2_block(stem, rows, run_names, test_names)
    header += [""]
    header += rule13_block(sorted(docs.items()))
    header += [""]
    header += argv_order(cases)
    header += [""]
    header += _sf_assertion_shape(stem, rows, carrier, cfg_rows)
    header += [""]
    header += ruling7_block(source)
    header += [""]
    header += RULING16_NOTE
    block = extra_ok_block()
    if block:
        header += [""] + block
    return header, {}, source, cases


def _sf_rule12_block(stem, docs, subs):
    lines = [
        "RULE 12 / U6 -- SOURCE COMMENT PROSE, LISTED BEFORE ANY TOML WAS WRITTEN.",
        f"tests/browser_{stem}.rs carries {len(docs)} `///` doc comment block(s) and NO bare",
        "`//` comment block; neither submodule carries any comment at all. The generator",
        "raises if either count is wrong, so a source that grows a comment breaks generation",
        "rather than shipping a file that silently drops it.",
    ]
    if docs:
        lines.append("The doc blocks, with the helper each documents:")
        for owner, text in sorted(docs.items()):
            lines.append(f"  * {owner} -- \"{text}\"")
        lines += [
            "Each is carried into the rationale of exactly the cases whose output passes",
            "through that helper (U6 bottom-up; copying both into every case to make",
            "`comment_coverage.py` read clean would be the over-attribution U6 forbids).",
            "",
            "CONSEQUENCE: `comment_coverage.py` IS RED ON THIS PAIR, AND MUST STAY RED.",
            "The checker pools the header and every rationale and asks whether each source",
            "comment line appears in ALL of them; it has no per-helper attribution. Each doc",
            "block above documents ONE assertion helper and is carried only by the cases that",
            "reach it, so the checker reports the other cases as missing it. That report is",
            "correct about the text and wrong about the requirement: U6 states in terms that",
            "the fix is NOT to copy the prose into cases whose helper never runs, and",
            "explicitly forbids doing so to turn the checker green. This is the same",
            "documented limitation the shipped `map_iteration_harness` and",
            "`set_iteration_harness` pairs carry. Every other arm of `verify_pair.sh` is green",
            "on this pair.",
        ]
    return lines


def _sf_u2_block(stem, rows, run_names, test_names):
    return [
        "U2 -- `[source]` is FILE-WIDE, AND THE run/test FOLD IS SAFE HERE. MEASURED.",
        "Both `#[path]` submodules land in this one file, so `expand.rs` clones every fixture",
        "below into every trial. That is the exact shape U2 warns about, so it was measured",
        "against the real binary rather than argued:",
        f"  * no fixture in either submodule is written behind an `if` -- all {len(rows)} `#[test]`",
        "    fns call the same unconditional writer, and this file's generator raises if any fn",
        "    writes other than exactly one filename;",
        f"  * the halves' filenames are disjoint ({len(run_names)} `run` names ending `."
        f"{stem.split('_')[-2]}`, {len(test_names)} `test` names",
        "    ending `.test." + stem.split('_')[-2] + "`), so the union is a merge and not an overwrite, and every",
        "    body in it is byte-identical (asserted mechanically, ruling 7);",
        f"  * for each of the {len(rows)} fns the binary was run on the same argv and environment in a",
        "    directory holding only its own fixture, in one holding its own half's fixtures, and",
        "    in one holding BOTH halves' -- and the output is identical in all three, modulo",
        "    `payload.runtimeMs`, which nothing here pins.",
        "The measurement is `gen_batch8b.py --derive`, and it is checked against a known",
        "positive in the same run: adding a `kali.json` and dropping the flag under test makes",
        "the same instrument report DISARMED, so the zero above is a fact about this fold",
        "rather than about the instrument.",
        "Every command names its entry explicitly on argv, so the unused siblings are inert.",
    ]


def _sf_assertion_shape(stem, rows, carrier, cfg_rows):
    helpers = sorted({r["helper"] for r in rows if r["helper"]})
    lines = [
        "ASSERTION SHAPE. Text cases carry `stdout_contains` and an exact `stderr = \"\"`,",
        "mirroring the source's plain `.contains(...)` against raw stdout and its",
        "`assert_eq!(String::from_utf8_lossy(&output.stderr), \"\")` -- ruling 3, mirror the",
        "source, so the substring claim is NOT strengthened. JSON cases carry the envelope",
        "paths the source asserts and an EXACT `json.stdout` pin, because a JSON string leaf",
        "has no substring form in this format (ruling 3's other limb); every such pin was",
        "captured from the real binary and the generator refuses to emit one that does not",
        "contain the source's own needle, so the strengthening is verified rather than",
        "assumed.",
        "The per-case envelope claim sets are NOT hand-listed: they are the claims of the",
        f"assertion helper the fn routes through ({', '.join('`' + h + '`' for h in helpers)})",
        "merged with the fn's own inline asserts, read out of the `.rs`.",
    ]
    if cfg_rows:
        lines += [
            "",
            "ONE THING THIS FORMAT CANNOT EXPRESS, AND IT IS NOT PAPERED OVER.",
            f"{len(cfg_rows)} of these `#[test]` fn(s) carry `#[cfg(unix)]` in the source:",
        ]
        for r in cfg_rows:
            lines.append(f"  * `{r['fn']}` ({os.path.basename(r['sub'])})")
        lines += [
            "They chmod the summary file to 0o000 and require the read to fail, which is a unix",
            "property. There is no platform key in this format, and `ignore = true` would",
            "disable the case on unix too -- a weakening (rule 1). They are migrated",
            "unconditionally: nothing the source asserts is lost, and the suite has no non-unix",
            "lane for the extra claim to be evaluated on. The batch report escalates the choice",
            "and specifies the alternative (a U4 trim retaining exactly these fns).",
        ]
    return lines


def _sf_rationale(stem, r, claims, carrier, docs, cfg_rows):
    ext = stem[len("runtime_summary_fallback_"):-len("_input")]
    half = r["half"]
    parts = [
        f"Migrated from browser_{stem}.rs, `#[path]` submodule "
        f"`{os.path.basename(r['sub'])}` -- one `[[case]]` per source `#[test]` fn, no "
        f"`[matrix]`, so this case is exactly one fn and one real invocation (rule 6)."]
    parts.append(
        f"The fn writes the shared browser test program to `{r['file']}` in a fresh temp dir "
        f"({cited_fn(carrier, r['writer'])}), points "
        f"`{HARNESS_ENV}` at a `node -e` program that fabricates the browser harness's summary "
        f"file and stdout, and runs `kali "
        f"{' '.join(r['argv'][:-1])} <entry>`.")
    if r["helper"]:
        hp = helper_claims(carrier, r["helper"])
        parts.append(
            f"Its output goes through {cited_fn(carrier, r['helper'])}, whose own claims "
            f"({len(hp)} of them) are merged into this case rather than restated: they are read "
            "out of that helper's body by this file's generator.")
        if r["helper"] in docs:
            parts.append(
                f"RULE 13 -- the `///` doc on {r['helper']}, carried verbatim: "
                f"\"{docs[r['helper']]}\"")
    else:
        parts.append(
            "The fn asserts directly on the process output; there is no assertion helper in "
            "its call chain, so no helper claims and no helper doc are carried here.")
    leaves = [c[1] for c in claims if c[0] == "json_contains"]
    if leaves:
        parts.append(ruling3_json_leaf(leaves))
    if any(c[0] == "stdout_contains" for c in claims):
        parts.append(P.ruling3_substring())
    if any(c[0] == "stderr_exact" for c in claims):
        parts.append(
            "`stderr = \"\"` is an exact pin because the source spells an exact equality "
            "(`assert_eq!(String::from_utf8_lossy(&output.stderr), \"\")`), which rule 1 "
            "requires be carried as an exact pin rather than a substring claim.")
    if any("cfg" in a for a in r["attrs"]):
        parts.append(
            "MIGRATION NOTE: the source fn carries `#[cfg(unix)]`, so it is compiled only on "
            "unix. The case format has no platform key, so this case runs on every platform "
            "the suite runs on. Nothing the source asserts is weakened; the file header and "
            "the batch report record the escalation.")
    parts.append(
        "U2 -- this file merges both `#[path]` submodules. The fold was measured against the "
        "real binary (see the file header): with every fixture of both halves present the "
        "output of this exact command is unchanged, so no claim here is supplied by a "
        "sibling fixture.")
    return " ".join(parts)


for _stem in SF_STEMS:
    REGISTRY[_stem] = (lambda s: (lambda: build_summary_fallback(s)))(_stem)


# ==========================================================================
# THE NINE `flat` TARGETS
# ==========================================================================

from batch8b_claims import claims_of, resolve  # noqa: E402
from batch8b_extract import (  # noqa: E402
    argv_of, entry_of, env_of, fixture_of, fn_params, invocations, json_loop,
    manifest_body, policy_body, writes_manifest,
)

EXT_ORDER = ["js", "ts", "jsx", "tsx"]

# Every claim string a file carries that the source does NOT spell as a literal,
# accumulated as it is emitted rather than listed by hand. `check_extra_claims.py`
# (U14's `extra` direction, rule 2's checkable invariant) reports each of these,
# and a declaration written from memory is a declaration that stops matching the
# moment a pin moves.
PINNED = []



def extify(name):
    for e in ("jsx", "tsx", "js", "ts"):
        if name.endswith("." + e):
            return name[: -len(e)] + "${ext}", e
    raise AssertionError(f"{name!r} does not end in a known extension")


def flat_rows(stem, helpers, *, extra_bind=None):
    """One row per REAL invocation, every loop expanded (rule 7's precondition).

    Covers both flat shapes with one code path: a target whose `#[test]` fns
    call a shared helper (bind the call site's literals to the helper's own
    parameter names, read off its signature) and a target whose fns build the
    `Command` inline (the fn body IS the helper body, with an empty binding).
    """
    text = rs(stem)
    rows = []
    for fn in test_fns(text):
        calls = invocations(text, fn["body"], helpers) if helpers else []
        if not calls:
            if "Command::new(kali_bin())" not in fn["body"]:
                raise AssertionError(f"{stem}::{fn['name']} neither calls a helper nor builds a Command")
            calls = [(None, [])]
        for helper, args in calls:
            body = fn_body(text, helper)[0] if helper else fn["body"]
            bind = dict(zip(fn_params(text, helper), args)) if helper else {}
            modes = [False, True] if json_loop(body) else [bind.get("json_output")]
            if modes == [None]:
                modes = [("--output" in fn["body"] and "json" in fn["body"])]
            for mode in modes:
                b = dict(bind)
                b["json_output"] = mode
                entry = entry_of(text, body, b, resolve)
                rows.append({
                    "stem": stem, "fn": fn["name"], "attrs": fn["attrs"],
                    "helper": helper, "bind": b, "entry": entry,
                    "argv": argv_of(text, body, b, entry=entry, resolve=resolve),
                    "env": env_of(body), "fixture": fixture_of(text, body, b, resolve),
                    "manifest": writes_manifest(text, body, b, resolve),
                    "policy": "write_valid_policy(" in resolve(body, b),
                    "claims": (claims_for(text, helper, b, extra_bind=extra_bind)
                               if helper else
                               claims_of(text, fn["body"], b, extra_bind=extra_bind)),
                    "json_output": bool(mode),
                })
    return text, rows


def group_rows(rows):
    """Fold rows that differ ONLY in the input extension into one case.

    The fold is rule 7's `[matrix]`, and it is only legal when the group covers
    the axis uniformly -- which is CHECKED here, per group, rather than declared
    once for the file. A group covering three of four extensions declines the
    axis for the whole file (U1: there is no per-case opt-out).
    """
    groups = {}
    for r in rows:
        tmpl, ext = extify(r["entry"])
        argv = [tmpl if a == r["entry"] else a for a in r["argv"]]
        key = (tuple(argv), r["env"], r["fixture"], tuple(map(str, r["claims"])),
               r["manifest"], r["policy"], r["json_output"])
        groups.setdefault(key, {"rows": [], "argv": argv, "entry": tmpl})
        groups[key]["rows"].append((ext, r))
    for g in groups.values():
        g["exts"] = [e for e, _ in g["rows"]]
    return list(groups.values())


def group_name(fn_names, *, json_output, suffix=None):
    if len(fn_names) == 1:
        base = fn_names[0]
    else:
        common = os.path.commonprefix(fn_names)
        base = common[: common.rfind("_")] + "_all_inputs"
    tail = suffix or ("json" if json_output else "text")
    return f"{base}__{tail}"


def flat_case(stem, group, *, caps, source, half, or_pins=None):
    fns = sorted({r["fn"] for _, r in group["rows"]})
    name = group_name(fns, json_output=group["rows"][0][1]["json_output"])
    exemplar = group["rows"][0][1]
    env = {} if exemplar["env"] is None or exemplar["env"][1] is None else \
        {exemplar["env"][0]: exemplar["env"][1]}
    # A pinned json leaf must be identical across the axis, or one pin cannot
    # stand for the whole fanned case.
    steps = []
    per_ext = []
    for ext, r in sorted(group["rows"], key=lambda x: EXT_ORDER.index(x[0])):
        files = {k.replace("${ext}", ext): v for k, v in source.items()}
        argv = [a.replace("${ext}", ext) for a in group["argv"]]
        # The command belongs in the key: one source fn can contribute a `run`
        # cell and a `test` cell, and a key built from the fn-derived case name
        # alone is the SAME string for both. The second capture then overwrote
        # the first and the file shipped one cell's observation under the other
        # cell's argv. Caught by the staleness check on the next run, which is
        # the whole reason the inputs are recorded beside the observation.
        step, dropped = build_step(
            r["claims"], argv, env, caps=caps,
            cap_key=f"{stem}::{half}::{name}::{r['bind'].get('command', '-')}::{ext}",
            files=files, or_pins=or_pins)
        per_ext.append((ext, step, dropped))
    base = per_ext[0][1]
    for ext, step, _ in per_ext[1:]:
        for key in base:
            if key == "args":
                continue
            if step.get(key) != base.get(key):
                raise AssertionError(
                    f"{stem} {name}: `{key}` differs between ext={per_ext[0][0]} and "
                    f"ext={ext}; the matrix fold would carry one pin for two values")
    step = dict(base)
    step["args"] = group["argv"]
    return name, fns, step, per_ext[0][2]


DISARM_MEASUREMENT = [
    "  $ gen_batch8b.py --derive",
    "    with the flag under test present and NO manifest (what the source runs): the",
    "      case's pins hold;",
    "    with the manifest present and the flag REMOVED: the case's pins STILL hold.",
    "  So in a shared file the flag would be unverified -- the manifest would supply the",
    "  claim, no literal would be dropped (so the audit cannot see it) and the trial would",
    "  still pass (so `cargo test` cannot either).",
]

SANDBOX_MEASUREMENT = [
    "  $ gen_batch8b.py --derive",
    "    JSON cases: with the manifest present and `--api browser` REMOVED, the binary emits",
    "      `errors[0].context.origin = \"config\"` where these cases pin `\"cli\"`, so on the",
    "      JSON cases ALONE a shared file would still discriminate;",
    "    TEXT cases: they pin only stderr substrings (`E5506`, the contract sentence, the",
    "      selected-host sentence) and the manifest supplies every one of them with the flag",
    "      removed -- so those cases WOULD be disarmed.",
    "  `[source]` is file-wide with no per-case opt-out, so one disarmed case disarms the",
    "  file. Deriving from the JSON half alone would have answered `no split needed`, which",
    "  is the mistake batch 8A made in the other direction and is why the derivation is run",
    "  per case SHAPE rather than per target.",
]



def apply_u5(stem, rows):
    """U5: variant-suffix a `[source]` key two different programs would share.

    `runtime_spawned_process_budget_js_input` writes BOTH a `console.log` program
    and a `Kali.test` program to `main.<ext>` -- its explicit-surface fns hand
    `kali test` a `main.<ext>` rather than a `smoke.test.<ext>`, which is the
    same shape batch 8A hit on `set_iteration_harness`. `[source]` is one flat
    file-wide namespace, so one of the two programs would silently overwrite the
    other. Detected rather than declared: the rename fires only where a real
    collision exists, and `assert_rename_is_argv_only` then checks U5's safety
    condition (the name is argv-only and no fixture body references it).
    """
    by_entry = {}
    for r in rows:
        by_entry.setdefault(r["entry"], set()).add(r["fixture"])
    collisions = {e for e, bodies in by_entry.items() if len(bodies) > 1}
    if not collisions:
        return []
    renames = []
    for r in rows:
        if r["entry"] not in collisions:
            continue
        base, ext = r["entry"].rsplit(".", 1)
        command = r["bind"].get("command")
        if command not in ("run", "test"):
            raise AssertionError(
                f"{stem}: collision on {r['entry']!r} with no `command` binding to name it by")
        new = f"{base}_{command}.{ext}"
        r["argv"] = [new if a == r["entry"] else a for a in r["argv"]]
        renames.append((r["entry"], new, f"the `{command}` leg writes its own program text"))
        r["entry"] = new
    return sorted(set(renames))


def emit_flat(*, stem, out_stem, helpers, keep_manifest, other_stem, half_label,
              program_desc, helper_desc, measurement, disarmed_shape,
              extra_header=None, or_pins=None, extra_bind=None,
              rule12_expect=0, migration_notes=()):
    PINNED.clear()
    text, rows = flat_rows(stem, helpers, extra_bind=extra_bind)
    blocks = comment_blocks(text, "//")
    if len(blocks) != rule12_expect:
        raise AssertionError(
            f"{stem}: {len(blocks)} Rust comment block(s), generator accounts for "
            f"{rule12_expect}")
    docs = {}
    for block in comment_blocks(text, "///"):
        docs[doc_owner(text, block[0])] = prose(block)

    mine = [r for r in rows if r["manifest"] == keep_manifest]
    theirs = [r for r in rows if r["manifest"] != keep_manifest]
    renames = apply_u5(stem, mine)
    if not mine:
        raise AssertionError(f"{stem}: the {half_label} half is empty")

    groups = group_rows(mine)
    uniform = all(sorted(g["exts"]) == sorted(EXT_ORDER) for g in groups)

    source = {}
    cases = []
    if uniform:
        for g in groups:
            key = g["entry"]
            if key in source and source[key] != g["rows"][0][1]["fixture"]:
                raise AssertionError(
                    f"{stem}: `{key}` would carry two different programs -- U5 rename needed")
            source[key] = g["rows"][0][1]["fixture"]
    else:
        for r in mine:
            if r["entry"] in source and source[r["entry"]] != r["fixture"]:
                raise AssertionError(
                    f"{stem}: `{r['entry']}` would carry two different programs")
            source[r["entry"]] = r["fixture"]
    if keep_manifest:
        source["kali.json"] = manifest_body(text)
    policy_leak = []
    if any(r["policy"] for r in mine):
        source["kali.policy.json"] = policy_body(text)
        # U2 again, one level down: `kali.policy.json` is ALSO written behind an
        # `if` in one target -- the two malformed-harness-command fns of
        # `runtime_sandbox_js_input` write no policy at all. A file-wide
        # `[source]` therefore makes it present for them too. Rather than
        # asserting that is harmless, it is MEASURED: the affected cases are run
        # against the real binary with and without the policy in the trial dir,
        # and the generator raises unless the two observations are identical.
        for r in mine:
            if r["policy"]:
                continue
            env = {} if r["env"] is None or r["env"][1] is None else {r["env"][0]: r["env"][1]}
            without = {k: v for k, v in source.items() if k != "kali.policy.json"}
            a = CAPS.get(f"{stem}::{half_label}::{r['fn']}::no-policy", without, r["argv"], env)
            b = CAPS.get(f"{stem}::{half_label}::{r['fn']}::with-policy", source, r["argv"], env)
            if (a["rc"], a["stdout"], a["stderr"]) != (b["rc"], b["stdout"], b["stderr"]):
                raise AssertionError(
                    f"{stem}::{r['fn']}: a leaked `kali.policy.json` CHANGES this case's "
                    "observed output, so the file-wide [source] disarms it and U2 forces a "
                    "further split")
            policy_leak.append(r["fn"])

    matrix = {"ext": EXT_ORDER} if uniform else {}
    dropped_all = []
    if uniform:
        for g in groups:
            name, fns, step, dropped = flat_case(stem, g, caps=CAPS, source=source,
                                                 half=half_label, or_pins=or_pins)
            dropped_all += dropped
            cases.append({"name": name, "fns": fns, "steps": [step],
                          "row": g["rows"][0][1], "exts": g["exts"]})
    else:
        for r in mine:
            env = {} if r["env"] is None or r["env"][1] is None else {r["env"][0]: r["env"][1]}
            step, dropped = build_step(
                r["claims"], r["argv"], env, caps=CAPS,
                cap_key=f"{stem}::{half_label}::{r['fn']}", files=source, or_pins=or_pins)
            dropped_all += dropped
            cases.append({"name": r["fn"], "fns": [r["fn"]], "steps": [step],
                          "row": r, "exts": None})

    invocations_here = len(mine)
    fns_here = sorted({r["fn"] for r in mine})

    header = [f"Migrated from tests/browser_{stem}.rs -- the {half_label} half."]
    header += [""]
    header += _flat_rule12(stem, blocks, docs)
    header += [""]
    if uniform:
        header += P.matrix_arithmetic(
            test_fns=len(fns_here), invocations=invocations_here,
            helpers=[(h, sum(1 for r in mine if r["helper"] == h),
                      "every real call of it in this half, with every `for` loop expanded")
                     for h in sorted({r["helper"] for r in mine if r["helper"]})]
            or [("the `#[test]` fns' own inline `Command` builders",
                 invocations_here, "one per (fn, json_output) pair")],
            cases=len(cases), axis="ext", values=EXT_ORDER)
    else:
        header += P.matrix_declined(
            test_fns=len(fns_here), invocations=invocations_here, cases=len(cases),
            reason=[
                "The entries this half writes do NOT cover one uniform extension axis: "
                f"{sorted({r['entry'] for r in mine})}.",
                "An `ext` axis would have to manufacture combinations the source never ran.",
            ])
    header += [""]
    header += (P.rule6_matrix_fold(
        "one `(command, json_output)` cell of the source's fns, fanned to the four extensions")
        if uniform else P.RULE6_ONE_TO_ONE)
    header += [""]
    header += u2_split_header(
        this_half=half_label, other_stem=other_stem, this_fns=len(fns_here),
        this_invocations=invocations_here, disarmed_shape=disarmed_shape,
        measurement=measurement)
    header += [""]
    header += rule13_block(sorted(docs.items()))
    header += [""]
    header += argv_order([{"steps": c["steps"]} for c in cases])
    header += [""]
    if renames:
        P.assert_rename_is_argv_only(source, [n for _, n, _ in renames], EXT_ORDER)
        header += P.u5_renames(renames) + [""]
    else:
        header += [
            "U5 -- NO `[source]` KEY RENAME IS NEEDED. Every entry filename in this half",
            "carries exactly one program text, so the flat file-wide `[source]` namespace has",
            "no collision. Checked mechanically, not eyeballed: the generator raises if any",
            "filename would carry two different bodies.",
        ] + [""]
    header += _flat_assertion_shape(cases, dropped_all, or_pins)
    if extra_header:
        header += [""] + list(extra_header)
    header += [""]
    if policy_leak:
        header += [""] + [
            "U2, ONE LEVEL DOWN -- `kali.policy.json` IS ALSO CONDITIONAL, AND THAT WAS",
            "MEASURED RATHER THAN WAVED THROUGH. These fns write no sandbox policy at all:",
        ] + [f"  * `{f}`" for f in sorted(set(policy_leak))] + [
            "A file-wide `[source]` puts `kali.policy.json` in their trial dir anyway. They",
            "never pass `--sandbox`, and the real binary was run for each of them with and",
            "without that file present: identical exit code, identical stdout, identical",
            "stderr. The policy is therefore inert for them and no further split is needed.",
            "This is checked on every generator run, not recorded once -- both observations",
            "live in the capture table and the generator raises if they ever diverge.",
        ]
    header += ruling7_block(source)
    header += [""]
    header += RULING16_NOTE
    block = extra_ok_block(renames)
    if block:
        header += [""] + block

    out = []
    for c in cases:
        out.append({
            "name": c["name"],
            "rationale": _flat_rationale(
                stem, c, program_desc, helper_desc, text, docs, uniform,
                other_stem, half_label, dropped_all, or_pins, migration_notes),
            "steps": c["steps"],
        })
    # One source fn can contribute several cells -- `run_and_test_support_...`
    # calls its helper once for `run` and once for `test` -- so a name derived
    # from the fn name alone collides. Disambiguated by the cell's own `command`
    # binding, and only where a collision actually exists, so the other 18 files
    # keep the shorter name.
    from collections import Counter
    counts = Counter(c["name"] for c in out)
    for case, built in zip(out, cases):
        if counts[case["name"]] < 2:
            continue
        command = built["row"]["bind"].get("command")
        if not command:
            raise AssertionError(
                f"{out_stem}: duplicate case name {case['name']!r} with no `command` "
                "binding to disambiguate it by")
        case["name"] = re.sub(r"__(text|json)$", rf"__{command}__\1", case["name"])
    names = [c["name"] for c in out]
    if len(set(names)) != len(names):
        raise AssertionError(f"{out_stem}: duplicate `[[case]]` name(s): "
                             f"{[n for n, k in Counter(names).items() if k > 1]}")
    if uniform and len(cases) * len(EXT_ORDER) != invocations_here:
        raise AssertionError(
            f"{out_stem}: matrix arithmetic does not close: {len(cases)} x 4 != "
            f"{invocations_here}")
    if not uniform and len(cases) != invocations_here:
        raise AssertionError(f"{out_stem}: {len(cases)} cases vs {invocations_here} invocations")
    return header, matrix, source, out


def _flat_rule12(stem, blocks, docs):
    if not blocks and not docs:
        return [
            "RULE 12 / U6 -- SOURCE COMMENT PROSE. tests/browser_" + stem + ".rs carries NO",
            "Rust comment block at all -- neither a bare `//` block nor a `///` doc comment on",
            "any helper (the generator counts both and raises if either count changes), so",
            "there is no prose to move verbatim into a `rationale` here.",
        ]
    lines = [
        "RULE 12 / U6 -- SOURCE COMMENT PROSE, LISTED BEFORE ANY TOML WAS WRITTEN.",
    ]
    for start, body in blocks:
        lines.append(f"  * `// {body[0]}` (:{start}) -- opens a {len(body)}-line block")
    for owner, t in sorted(docs.items()):
        lines.append(f"  * `///` on {owner} -- \"{t}\"")
    lines.append(
        "Each is carried into the rationale of exactly the cases its owner's call path "
        "reaches (U6 bottom-up), COPIED out of the `.rs` by this generator rather than "
        "retyped, so an em-dash cannot become `--`.")
    return lines


def _flat_assertion_shape(cases, dropped, or_pins):
    lines = [
        "ASSERTION SHAPE. Every key below is derived from the source's own asserts, resolved",
        "for this case's `(command, json_output)` binding by this file's generator, which",
        "RAISES on a branch condition it cannot evaluate rather than letting both branches",
        "contribute. Text cases keep `stdout_contains`/`stderr_contains` (ruling 3: a plain",
        "`.contains` against a field that HAS a substring form is not strengthened). JSON",
        "cases pin the envelope paths the source asserts; where the source spells",
        "`.contains(...)` against a JSON string leaf, which has no substring form in this",
        "format, ruling 3 requires an exact pin, and the generator refuses to emit one that",
        "does not contain the source's own needle -- so the strengthening is verified.",
        "`assert!(!errors.is_empty())` is carried by the `errors.0.*` pins, which a missing",
        "path hard-fails on; the generator raises if a file asserts non-emptiness with no",
        "`errors.0` pin to carry it.",
    ]
    if dropped:
        lines += [
            "",
            "RULE 11 / RULING 17 -- ONE OR-SHAPED SOURCE ASSERTION, RESOLVED BY OBSERVATION.",
            "The source accepts either of two stderr needles. Run against the real binary over",
            "every cell, BOTH disjuncts are true and the cells agree, which is the case rule 11",
            "does not cover and ruling 17 does: pin the FIRST in source order",
            f"({', '.join(repr(p) for p in (or_pins or []))}), disclose the other",
            f"({', '.join(repr(d) for d in sorted(set(dropped)))}), and do NOT pin both --",
            "`A` and `A and B` are both stronger than `A or B`, but they are ordered, so pinning",
            "one disjunct is the strictly weaker and more faithful strengthening. Pinning the",
            "other as well would assert unconditionally something the source only ever offered",
            "as an alternative (rule 2).",
        ]
    return lines


def _flat_rationale(stem, case, program_desc, helper_desc, text, docs, uniform,
                    other_stem, half_label, dropped, or_pins, migration_notes):
    r = case["row"]
    parts = [f"Migrated from browser_{stem}.rs."]
    if uniform:
        parts.append(
            f"This `[[case]]` is one `(command, json_output)` cell of the source, matrix-fanned "
            f"by `ext(4)`, so it stands for {', '.join('`' + f + '`' for f in case['fns'])} -- "
            "one trial per real invocation, with the assertion mapping staying one-to-one per "
            "trial (rule 6's sanctioned rule-7 fold, stated here as the rule requires).")
    else:
        parts.append(
            f"With no `[matrix]` there is no fold: this case is exactly the source fn "
            f"`{case['fns'][0]}` and its one real invocation (rule 6).")
    parts.append(program_desc)
    if r["helper"]:
        parts.append(
            f"It routes through {cited_fn(text, r['helper'])}; {helper_desc}")
    if r["helper"] in docs:
        parts.append(f"RULE 13 -- the `///` doc on {r['helper']}, carried verbatim: "
                     f"\"{docs[r['helper']]}\"")
    leaves = [c[1] for c in r["claims"] if c[0] == "json_contains"]
    if leaves:
        parts.append(ruling3_json_leaf(leaves))
    if any(c[0] in ("stdout_contains", "stderr_contains") for c in r["claims"]):
        parts.append(P.ruling3_substring(
            surface="raw stdout/stderr", key="stdout_contains/stderr_contains"))
    if dropped and any(c[0] == "or_group" for c in r["claims"]):
        parts.append(
            "RULE 11 / RULING 17 -- the source's disjunction sentence, carried so the narrowing "
            "is recorded rather than silent: it accepts stderr containing either "
            + " or ".join(repr(x) for x in [p for p in (or_pins or [])] + sorted(set(dropped)))
            + ". Both are true on every cell of the real binary's output, so the first in "
            "source order is pinned and the other is disclosed but not asserted (pinning both "
            "would be a rule-2 invention).")
    if r["env"] is not None and r["env"][1] is None:
        parts.append(
            "The source calls `.env_remove(...)` on the browser-harness command variable. A "
            "case file can set a variable but cannot unset one (the runner layers a step's "
            "`env` on the inherited environment), so this case carries no `env` key. Measured "
            "rather than assumed: with the variable unset, set to `node`, and set to three "
            "spaces, the binary emits the same exit code, the same E5506, the same "
            "context.origin and the same message -- this rejection precedes any use of the "
            "harness command, so no claim here depends on it.")
    parts.append(
        f"U2 -- this target is TWO case files split on `kali.json` presence; this is the "
        f"{half_label} half and `{other_stem}` is the other. The split is measured, not "
        "assumed (see the file header).")
    for note in migration_notes:
        parts.append(note)
    return " ".join(parts)


# --- the six manifest-split targets, twelve files -------------------------

SPLIT_TARGETS = {
    "runtime_wasm_threads_js_input": dict(
        helpers=["assert_browser_wasm_threads_acceptance_for_command"],
        program=("The program under test is a one-line `console.log` for `run` and a "
                 "single `Kali.test` registration for `test`, written to `main.<ext>` / "
                 "`smoke.test.<ext>`; both are copied out of the source, never retyped "
                 "(rule 9)."),
        helper_desc=("that helper loops `for json_output in [false, true]`, so each of its "
                     "call sites is two real invocations, and it asserts a clean exit plus "
                     "-- on the JSON branch -- the envelope's schemaVersion/command/success, "
                     "the browser host-contract pair, an empty thread topology, and the "
                     "run/test-specific payload fields."),
        disarmed="the accepting cases pin `payload.hostContract = \"browser-requested\"`, "
                 "which the manifest supplies on its own",
        measurement=DISARM_MEASUREMENT),
    "wasm_threads_max_threads_harness": dict(
        helpers=["assert_browser_harness_accepts_thread_budget",
                 "assert_browser_harness_rejects_positive_thread_budget"],
        program=("The program under test is a one-line `console.log` for `run` and a single "
                 "`Kali.test` registration for `test`, written to `main.<ext>` / "
                 "`smoke.test.<ext>` (rule 9: copied, not retyped)."),
        helper_desc=("that helper loops `for json_output in [false, true]`, so each call site "
                     "is two real invocations."),
        disarmed="the accepting cases pin the browser host-contract pair, which the "
                 "manifest supplies on its own",
        measurement=DISARM_MEASUREMENT),
    "wasm_threads_browser_surface": dict(
        helpers=["assert_browser_wasm_threads_rejection_for_command"],
        program=("The program under test is the two-line `let value = 1 + 2; value;` written "
                 "to `app.<ext>` (rule 9: copied out of the source)."),
        helper_desc=("that helper loops `for json_output in [false, true]`, so each of its "
                     "call sites is two real invocations, and it asserts exit code 5 plus the "
                     "E5506 runtime-profile rejection on whichever stream the mode selects."),
        disarmed="the rejecting cases pin an E5506 whose *cause* is the requested "
                 "wasm-threads profile, which the manifest requests on its own",
        measurement=DISARM_MEASUREMENT,
        or_pins=["runtime profile"]),
    "runtime_spawned_process_budget_js_input": dict(
        helpers=["assert_browser_requested_accepts_zero_spawned_process_budget",
                 "assert_explicit_browser_api_surface_accepts_zero_spawned_process_budget",
                 "assert_browser_requested_rejects_positive_spawned_process_budget",
                 "assert_explicit_browser_api_surface_rejects_positive_spawned_process_budget"],
        program=("The program under test is a one-line `console.log` for `run` and a single "
                 "`Kali.test` registration for `test` (rule 9: copied out of the source)."),
        helper_desc=("that helper loops `for json_output in [false, true]`, so each call site "
                     "is two real invocations."),
        disarmed="the accepting cases pin `payload.hostContract = \"browser-requested\"`, "
                 "which the manifest supplies on its own",
        measurement=DISARM_MEASUREMENT,
        notes_explicit=[P.migration_note_stale_fn_name(
            "assert_browser_requested_rejects_positive_spawned_process_budget",
            "its name says browser-requested (spelled without backticks here: U8's gate resolves "
            "every backticked lower-case identifier against this source's fn list, and that is a "
            "prose fragment of a fn name, not a fn), but unlike its accepting twin it writes NO "
            "`kali.json` and passes NO `--api browser`, so no browser API surface is requested "
            "at all; the E5506 it asserts is the spawned-process budget rejection, which fires "
            "on the default surface too."),
            P.migration_note_stale_fn_name(
                "run_and_test_support_explicit_browser_api_surface_with_zero_spawned_process_"
                "budget_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_inputs",
                "it hands `kali test` a `main.<ext>` rather than a `smoke.test.<ext>`, unlike "
                "every other `test` leg in this source; the argv is carried faithfully and the "
                "`[source]` key is U5-renamed because the two legs write different programs to "
                "that one filename.")]),
}

SANDBOX_TARGETS = ["runtime_sandbox_js_input", "runtime_sandbox_ts_input",
                   "runtime_sandbox_jsx_tsx"]
SANDBOX_HELPERS = ["assert_browser_runtime_rejection_inherited_without_browser_harness",
                   "assert_browser_runtime_rejection_without_browser_harness",
                   "assert_browser_runtime_rejection_inherited",
                   "assert_browser_runtime_rejection"]
SANDBOX_PROGRAM = (
    "The program under test is a one-line `console.log('browser run');` for `run` and a "
    "single empty `Kali.test('browser', () => {});` registration for `test`, alongside a "
    "`kali.policy.json` sandbox policy every case in this half writes unconditionally "
    "(rule 9: all three are copied out of the source, never retyped).")
SANDBOX_HELPER_DESC = (
    "that helper writes the fixtures, runs the command and routes the output through "
    "`assert_browser_runtime_rejection_text` or `assert_browser_runtime_rejection_json` "
    "depending on the output mode; both are inlined into this case's claim set by the "
    "generator rather than summarised.")


def _register_split(stem, spec):
    for keep, label, suffix in ((False, "explicit-api", "explicit_api"),
                                (True, "inherited-manifest", "inherited_manifest")):
        out_stem = f"{stem}_{suffix}"
        other = f"{stem}_{'inherited_manifest' if not keep else 'explicit_api'}"

        def builder(stem=stem, spec=spec, keep=keep, label=label, other=other):
            notes = spec.get("notes_explicit", []) if not keep else []
            return emit_flat(
                stem=stem, out_stem=f"{stem}_{'explicit_api' if not keep else 'inherited_manifest'}",
                helpers=spec["helpers"], keep_manifest=keep, other_stem=other,
                half_label=label, program_desc=spec["program"],
                helper_desc=spec["helper_desc"], measurement=spec["measurement"],
                disarmed_shape=spec["disarmed"], or_pins=spec.get("or_pins"),
                migration_notes=notes)
        REGISTRY[out_stem] = builder


for _stem, _spec in SPLIT_TARGETS.items():
    _register_split(_stem, _spec)

for _stem in SANDBOX_TARGETS:
    _register_split(_stem, dict(
        helpers=SANDBOX_HELPERS, program=SANDBOX_PROGRAM,
        helper_desc=SANDBOX_HELPER_DESC, measurement=SANDBOX_MEASUREMENT,
        disarmed=("the TEXT cases pin only stderr substrings, every one of which the manifest "
                  "supplies with `--api browser` removed; the JSON cases pin "
                  "`errors[0].context.origin = \"cli\"`, which it does not")))


# --- the two bundle targets ----------------------------------------------

BUNDLE_TARGETS = {
    "wrapped_assignment_bundle": dict(
        helper="assert_browser_bundle_wrapped_assignment",
        builder="browser_bundle_wrapped_assignment_source",
        export="browserWrappedAssignmentTargets",
        program=("a bundle whose exported function exercises the wrapped nullish and logical "
                 "assignment targets `((last)) ??= null`, `((left)) ||= 1` and "
                 "`((right)) &&= 2`, throwing if any of them lands on the wrong value")),
    "wrapped_mutable_compound_assignment_bundle": dict(
        helper="assert_browser_bundle_wrapped_mutable_compound_assignment",
        builder="browser_bundle_wrapped_mutable_compound_assignment_source",
        export="browserWrappedMutableCompoundAssignmentTargets",
        program=("a bundle whose exported function applies every mutable compound assignment "
                 "operator through a wrapped target -- `+=`, `-=`, `*=`, `/=`, `%=`, `**=` -- "
                 "and throws if the result is not 1")),
}


def _bundle_split(stem, body):
    """A bundle helper's body, cut into its `kali` half and its harness half.

    Derived from the source's own second `Command::new`, so a helper that stops
    running a harness -- or runs two -- is a generator error rather than a step
    quietly built from the wrong claims.
    """
    marker = "Command::new(&harness_executable)"
    if body.count(marker) != 1:
        raise AssertionError(
            f"{stem}: {body.count(marker)} occurrence(s) of {marker!r}; the split between "
            "the `kali` step's claims and the harness step's claims is not derivable")
    at = body.index(marker)
    return body[:at], body[at:]


def build_bundle(stem):
    PINNED.clear()
    spec = BUNDLE_TARGETS[stem]
    text = rs(stem)
    fixture = literals(fn_body(text, spec["builder"])[0])
    fixture = [v for v in fixture if v.lstrip().startswith("// kali-tree-shake")]
    if len(fixture) != 1:
        raise AssertionError(f"{stem}: {len(fixture)} candidate fixture bodies")
    fixture = fixture[0]
    body = fn_body(text, spec["helper"])[0]
    harness_body = [v for v in literals(body) if v.startswith("const mod = await import(")]
    if len(harness_body) != 1:
        raise AssertionError(f"{stem}: {len(harness_body)} candidate harness bodies")
    harness_body = harness_body[0]

    blocks = comment_blocks(text, "//")
    docs = {doc_owner(text, b[0]): prose(b) for b in comment_blocks(text, "///")}

    fns = [f["name"] for f in test_fns(text)]
    if len(fns) != 2:
        raise AssertionError(f"{stem}: {len(fns)} `#[test]` fns, expected 2")
    filenames = invocations(text, test_fns(text)[0]["body"], [spec["helper"]])
    exts = []
    for _, args in filenames:
        _, e = extify(args[0])
        exts.append(e)
    if sorted(exts) != sorted(EXT_ORDER):
        raise AssertionError(f"{stem}: the loop covers {exts}, not the four extensions")

    escaped, constants = ({k: v.replace("${", "${dollar}{") for k, v in
                           {"app.${ext}": fixture}.items()},
                          {"dollar": "$"} if "${" in fixture else {})
    for key, value in escaped.items():
        if "${" in value.replace("${dollar}{", ""):
            raise AssertionError("rule 10: an unescaped `${` survives")

    cases = []
    for json_output in (False, True):
        argv = ["build", "--bundle", "--api", "browser"] + \
            (["--output", "json"] if json_output else []) + ["app.${ext}"]
        # The helper runs TWO processes -- `kali build`, then the browser-bundle
        # harness -- and each gets its own step, so each step's claims must come
        # from its own half of the body. Split POSITIONALLY at the harness's own
        # `Command::new`, not by filtering claim kinds: a kind filter is a hand
        # partition, and it is what let the harness's exact-empty `stdout` pin be
        # hardcoded below rather than derived, which in turn meant `PINNED` never
        # saw it and `extra_ok_block()` never declared it. Two shipped files then
        # failed `check_extra_claims.py` on a bare `''` while this generator's
        # docstring claimed the declarations were derived.
        cli_part, harness_part = _bundle_split(stem, resolve(body, {"json_output": json_output}))
        cli_claims = claims_in(cli_part)
        harness_claims = claims_in(harness_part)
        if not any(c[0] == "stdout_empty" for c in harness_claims):
            raise AssertionError(
                f"{stem}: the harness half asserts no `is_empty()` on its stdout -- the "
                "exact-empty pin below would be a rule-2 invention")
        step_cli, _ = build_step(cli_claims, argv, {})
        meta_claims = re.findall(r'assert_eq!\(\s*metadata\["([^"]+)"\]\s*,\s*"([^"]*)"\s*\)', body)
        if not meta_claims:
            raise AssertionError(f"{stem}: no app.meta.json claims found")
        step_harness, _ = build_step(harness_claims, [], {})
        step_harness.pop("args")
        steps = [
            step_cli,
            {"kind": "file_json", "path": "app/app.meta.json",
             "fields": {k: v for k, v in meta_claims}},
            dict(kind="browser_bundle_harness", entry="app", body=harness_body,
                 **step_harness),
        ]
        name = [f for f in fns if f.startswith("json_") == json_output][0]
        cases.append({
            "name": f"{name}__{'json' if json_output else 'text'}",
            "rationale": _bundle_rationale(stem, spec, text, blocks, docs, json_output,
                                           fns, meta_claims),
            "steps": steps,
        })

    header = [f"Migrated from tests/browser_{stem}.rs."]
    header += [""]
    header += _flat_rule12(stem, blocks, docs)
    header += [""]
    header += P.matrix_arithmetic(
        test_fns=2, invocations=8,
        helpers=[(spec["helper"], 8,
                  "2 `#[test]` fns, each looping `for filename in [\"app.js\", \"app.ts\", "
                  "\"app.jsx\", \"app.tsx\"]`, so 2 x 4 = 8")],
        cases=2, axis="ext", values=EXT_ORDER, non_axes=("json_output",))
    header += [""]
    header += P.rule6_matrix_fold("one `json_output` half of the source's 2 fns, fanned to "
                                  "the four extensions")
    header += [""]
    header += P.u2_source_file_wide(["app.${ext}"])
    header += [""]
    header += [
        "U5 -- NO `[source]` KEY RENAME IS NEEDED. This source writes exactly one program",
        "text, to `app.<ext>` in every test, so the file-wide `[source]` namespace has one",
        "entry per extension and nothing collides.",
    ]
    header += [""]
    header += rule13_block(sorted(docs.items()))
    header += [""]
    header += argv_order(cases)
    header += [""]
    header += [
        "ASSERTION SHAPE. Three steps per case, in the source's own order: the `kali build`",
        "invocation, the emitted `app/app.meta.json` (a `file_json` step, because the source",
        "reads that file off disk rather than out of stdout), and the browser-bundle harness",
        "run. THE BUILD SUCCEEDS AND SO DOES THE HARNESS: the source asserts",
        "`output.status.success()` on both and additionally that the harness's stdout is",
        "EXACTLY empty, so the harness step carries `exit = \"success\"` and `stdout = \"\"`.",
        "That exact-empty pin is the source's own emptiness assertion on the harness stdout,",
        "not an invention: a `stdout_absent` list cannot express \"nothing at all\" (rule 1).",
        "Per controller ruling 6, the `browser_bundle_harness` step kind means the RUNNER",
        "builds the harness script, so the `///` docs on",
        "`kali_runtime_contract::browser_bundle_harness_script` and",
        "`::browser_harness_command_parts_for` are shared runner infrastructure and are NOT",
        "carried here -- the exemption is in the rule, not in this file's discretion.",
    ]
    if constants:
        header += [""]
        header += rule10_prose_local(escaped)
    header += [""]
    header += ruling7_block(escaped)
    header += [""]
    header += RULING16_NOTE
    block = extra_ok_block()
    if constants:
        block = (block or list(P.EXTRA_CLAIM_PREAMBLE)) + [
            P.extra_ok("$", "the value of the rule-10 `[constants] dollar` escape, not an "
                            "assertion at all -- the RESOLVED program text is byte-identical "
                            "to the source's, which is the whole point of rule 10")]
    if block:
        header += [""] + block
    return header, {"ext": EXT_ORDER}, escaped, cases, constants


def rule10_prose_local(escaped):
    names = sorted({m.group(1) for body in escaped.values()
                    for m in re.finditer(r"\$\{dollar\}\{(\w+)\}", body)})
    if not names:
        raise AssertionError("rule10 block requested for a file with no escaped literal")
    listed = ", ".join(names)
    count = ("one of its own bindings" if len(names) == 1
             else f"{len(names)} of its own bindings")
    return [
        "RULE 10 -- A GENUINE JS TEMPLATE LITERAL, ESCAPED THROUGH `[constants]`.",
        f"The program under test interpolates {count} ({listed}) with a real JS",
        "template literal in its throw message. `expand.rs`'s `substitute()` hard-fails on any",
        "`${...}` it cannot resolve, and it substitutes `[source]` BODIES as well as step",
        "fields, so this file declares `[constants] dollar = \"$\"` and spells every genuine",
        "`${` as `${dollar}{`. The RESOLVED program text is byte-identical to the source's --",
        "an encoding of rule 9, not an exception to it. DERIVED, not marked: the generator",
        "escapes whatever `${` the extracted fixture actually contains and raises if any",
        "survives, so a file cannot declare a constant it does not need or need one it does",
        "not declare.",
    ]


def _bundle_rationale(stem, spec, text, blocks, docs, json_output, fns, meta_claims):
    parts = [
        f"Migrated from browser_{stem}.rs.",
        f"This `[[case]]` is the {'--output json' if json_output else 'text'} half of the "
        f"source's 2 `#[test]` fns, matrix-fanned by `ext(4)` to the four extensions, so it "
        f"stands for one trial per real invocation of "
        f"`{[f for f in fns if f.startswith('json_') == json_output][0]}`, with the assertion "
        "mapping staying one-to-one per trial (rule 6's sanctioned rule-7 fold, stated here "
        "as the rule requires).",
        f"{cited_fn(text, spec['helper'])} writes "
        f"{spec['program']} to `app.<ext>` in a fresh temp dir, builds it with "
        f"`kali build --bundle --api browser{' --output json' if json_output else ''}`, "
        f"asserts the build succeeds, asserts the "
        f"emitted `app/app.meta.json` carries "
        + ", ".join(f"`{k} = \"{v}\"`" for k, v in meta_claims) +
        f" ({cited(text, 'join(\"app.meta.json\")')}), then writes the browser-bundle harness "
        "and runs it under node.",
        "BOTH PROCESSES SUCCEED here, unlike the fail-closed bundle targets migrated in "
        "earlier batches: the source asserts `output.status.success()` on the harness run and "
        "that its stdout is exactly empty, so the harness step carries `exit = \"success\"` "
        "and `stdout = \"\"`.",
        f"The program under test is {cited_fn(text, spec['builder'])}; its text is copied out of the `.rs` "
        "by this generator, never retyped (rule 9).",
    ]
    if blocks:
        parts.append(
            "RULE 12 -- the Rust comment prose of this source, carried verbatim: \""
            + " ".join(prose(b) for b in blocks) + "\"")
    if docs:
        parts.append("RULE 13 -- " + "; ".join(f"{o}: \"{t}\"" for o, t in sorted(docs.items())))
    return " ".join(parts)


for _stem in BUNDLE_TARGETS:
    REGISTRY[_stem] = (lambda s: (lambda: build_bundle(s)))(_stem)


# --- main -----------------------------------------------------------------


def main(argv):
    global CAPS
    if "--derive" in argv:
        import batch8b_derive
        return batch8b_derive.main()
    recapture = "--recapture" in argv
    wanted = [a for a in argv if not a.startswith("--")]
    CAPS = Captures(recapture=recapture)
    names = wanted or sorted(REGISTRY)
    unknown = [n for n in names if n not in REGISTRY]
    if unknown:
        raise SystemExit(f"unknown target(s): {unknown}\nknown: {sorted(REGISTRY)}")
    total_cases = 0
    total_trials = 0
    for name in names:
        built = REGISTRY[name]()
        if len(built) == 5:
            header, matrix, source, cases, constants = built
        else:
            header, matrix, source, cases = built
            constants = {}
        text = emit(header, matrix, source, cases)
        if constants:
            lines = text.split("\n")
            at = next(i for i, l in enumerate(lines) if l.startswith("[matrix]")
                      or l.startswith("[source]"))
            block = ["[constants]"] + [f'{k} = "{v}"' for k, v in constants.items()] + [""]
            text = "\n".join(lines[:at] + block + lines[at:])
        write(os.path.join(CASES, f"{name}.toml"), text)
        fan = 1
        for values in (matrix or {}).values():
            fan *= len(values)
        total_cases += len(cases)
        total_trials += len(cases) * fan
    CAPS.finish()
    print(f"\n{len(names)} file(s): {total_cases} case(s), {total_trials} trial(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
