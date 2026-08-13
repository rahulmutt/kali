#!/usr/bin/env python3
"""Generator for Task 19 batch 2 -- the 17 pure-`cli` non-browser targets.

WHAT THIS IS FOR (U12). Every case file this batch ships is emitted from here,
so the mapping a reviewer has to read -- which source `#[test]` became which
`[[case]]`, which axis was declared and which declined, which comment went into
which rationale -- lives in one place instead of being spread across 17
hand-written TOML files. It renders through `case_emit.emit`, the same emitter
the sixteen browser generators use.

IT DECIDES NOTHING ABOUT FIXTURE TEXT, AND THAT IS THE POINT (rules 8/9).
Fixture program text reaches a case file by exactly two routes, never by being
typed here:

  * `lit(stem, anchor)` pulls the literal out of the `.rs` through
    `lexer.find_string_literals` and REFUSES unless exactly one literal in the
    file contains the anchor (ruling 18 #3 -- a non-match is an error, not a
    fallback). A drifting line number cannot silently select the wrong literal
    because nothing here selects by position.
  * `t19b2_captures` holds the five sources whose fixture text does not exist as
    a literal anywhere -- built by a `format!` or by a `kali_common::` helper.
    Those were captured by EXECUTING the real code; that module's docstring
    records the command. `check_captured` re-checks each one against its own
    `.rs` before it is emitted, so a capture taken before a source edit fails
    the generator instead of shipping a program that is no longer the program
    under test.

Source comment prose reaches a rationale the same way: `para(stem, anchor)`
reads it out of the `.rs` with `comment_coverage.extract_comment_paragraphs`,
so rule 12's "text is copied, not retyped" holds by construction and an em-dash
cannot become `--` in transit.

  Usage:
    gen_task19_batch2.py            # CHECK: regenerate and diff; rc=1 on drift
    gen_task19_batch2.py --write    # emit the 17 case files
    gen_task19_batch2.py --list     # the file list and per-file trial counts

The default is the CHECK direction on purpose. A generator that only writes is
a fixed point nobody re-tests; run with no arguments and it asserts that every
shipped file is byte-for-byte what this spec produces.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "tools/task-18-browser-pilot"))
sys.path.insert(0, HERE)

from case_emit import emit, source_text_at  # noqa: E402
from comment_coverage import (extract_comment_paragraphs,  # noqa: E402
                              extract_trailing_comments, is_divider)
from lexer import find_string_literals  # noqa: E402
import t19b2_captures as CAP  # noqa: E402

TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases")

# The commit this batch's `SOURCE REF:` declarations name. Every one of the 17
# sources is still in the tree, so `citation_sweep.sh` content-validates each
# declaration against the working-tree file on every run -- which is the whole
# reason to declare it NOW rather than after the sources are deleted, when
# nothing could check it (pilot §2.4).
SOURCE_REF = "b7f2ed5d5644fff14b03b5266fcc7f26b13ac925"

_TEXT = {}


def rs(stem):
    """The source this case file was migrated FROM -- not always the working tree.

    Ruling 9: once a U4 trim-and-keep retention lands, the working tree holds
    only the RETAINED half, so a fixture the migrated half owned is gone and a
    generator that reads the working tree either crashes or, worse, emits a case
    file built from what is left. The pre-trim blob is the right side, and the
    ref comes from the retained file's OWN `PRE-TRIM REF:` line -- a ref carried
    anywhere else is the moving figure ruling 11 forbids.

    Delegated to `case_emit.source_text_at` rather than reimplemented: batch
    8-inst-1 established seven live readers of that header line, and an eighth
    that disagreed with the gate about which blob a pair means would look
    locally correct in both places. That function also raises when a `//!`
    header and a `PRE-TRIM REF:` disagree, which is the check this generator
    wants and would not have written for itself.

    Proven load-bearing rather than assumed: with `rs()` reading the working
    tree, trimming `object_has_own_frozen_js_input.rs` made `check_captured`
    raise `capture CAP_OBJECT_HAS_OWN__TEST is stale` -- the fixture builder the
    migrated half used had been trimmed away.
    """
    if stem not in _TEXT:
        _TEXT[stem] = source_text_at(os.path.join(TESTS, stem + ".rs"), quiet=True)
    return _TEXT[stem]


def lit(stem, anchor, *, want=1, index=0, exact=False):
    """The unique string literal of `<stem>.rs` containing `anchor`.

    `want` is spelled out by the caller rather than inferred, so a source that
    grows a second copy of a fixture fails here instead of silently selecting
    one. The call sites that pass `want=2` are the files that genuinely write
    byte-identical text under two names; they also assert the identity (U13's
    "assert the identity, don't eyeball it").

    `exact=True` selects by EQUALITY instead of containment, for the short
    expected-output literals where a substring anchor is ambiguous by nature
    ("2\\n" is inside "42\\nafter\\n"). It is still a copy check, not a
    transcription: the value must exist in the `.rs` as a whole literal or this
    raises, so a mistyped pin cannot reach a case file.
    """
    values = [x["value"] for x in find_string_literals(rs(stem))]
    hits = [v for v in values if (v == anchor if exact else anchor in v)]
    if len(hits) != want:
        raise AssertionError(
            f"{stem}.rs: {len(hits)} string literal(s) contain {anchor!r}, wanted {want}")
    return hits[index]


def para(stem, anchor):
    """The unique comment paragraph of `<stem>.rs` containing `anchor`, verbatim."""
    hits = ["\n".join(p) for _, p in extract_comment_paragraphs(rs(stem))
            if not is_divider(p) and anchor in "\n".join(p)]
    if len(hits) != 1:
        raise AssertionError(
            f"{stem}.rs: {len(hits)} comment paragraph(s) contain {anchor!r}, wanted 1")
    return hits[0].strip("\n")


def trailing(stem, anchor):
    """The unique TRAILING comment of `<stem>.rs` containing `anchor`, verbatim.

    Rule 12 does not distinguish a comment that owns its line from one that
    shares a line with code, and neither does this: the text is read out of the
    `.rs` like every other carry. Added after review found
    `heap_grow_runtime.rs:199` uncarried and the rule-12 gate blind to the whole
    shape -- `comment_coverage.py` matched `^\\s*//` and could not see it.
    """
    hits = [c for _, c in extract_trailing_comments(rs(stem)) if anchor in c]
    if len(hits) != 1:
        raise AssertionError(
            f"{stem}.rs: {len(hits)} trailing comment(s) contain {anchor!r}, wanted 1")
    return hits[0]


_CRATE_ITEMS = {}


def _crate_items(crate):
    """`{fn name: (doc, body)}` over every non-test module of a crate.

    Whole-crate, not per-file, because a documented helper is routinely in a
    different module from its caller (`join_semicolon_terminated_segments` lives
    in `helpers.rs`, its callers in `object.rs`).
    """
    if crate not in _CRATE_ITEMS:
        items = {}
        root = os.path.join(REPO, crate)
        for name in sorted(os.listdir(root)):
            if not name.endswith(".rs") or name.endswith("_tests.rs"):
                continue
            src = open(os.path.join(root, name)).read()
            for m in re.finditer(
                    r"^((?:[ \t]*///[^\n]*\n)*)[ \t]*(?:pub(?:\([^)]*\))?\s+)?"
                    r"(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)", src, re.M):
                doc = " ".join(l.strip()[3:].strip()
                               for l in m.group(1).splitlines() if l.strip())
                brace = src.find("{", m.end())
                if brace == -1:
                    continue
                depth, i = 0, brace
                while i < len(src):
                    if src[i] == "{":
                        depth += 1
                    elif src[i] == "}":
                        depth -= 1
                        if depth == 0:
                            break
                    i += 1
                items[m.group(2)] = (doc, src[brace:i + 1])
        _CRATE_ITEMS[crate] = items
    return _CRATE_ITEMS[crate]


def doc_chain(crate, entry):
    """Every `///` doc on every fn in `entry`'s call chain, in call order.

    THE GATE RULE 13 HAS NEVER HAD. `doc(path, fn)` -- what this batch shipped
    first -- reads the block above ONE NAMED function, so the chain was
    author-enumerated and nothing checked it for completeness. Three of this
    batch's seventeen files carried the resulting defect (two uncarried
    documented helpers each) and no gate would have caught it: rule 13 says a
    case produced through a helper call chain carries EVERY doc on EVERY helper
    in that chain, and until now the only thing enforcing "every" was the
    author's attention.

    This walks the chain instead: from `entry`, every call to a fn the crate
    itself defines, transitively, breadth-first in call order. A documented fn
    reached anywhere in that closure MUST be carried, and `carry_docs` raises
    when one is not -- so the failure mode is a loud generator rather than a
    quiet under-carry.

    Undocumented helpers in the chain are walked THROUGH but contribute
    nothing: rule 13's subject is doc comments, and a helper with no doc has no
    prose to carry. That is not an exemption -- there is nothing to exempt.
    """
    items = _crate_items(crate)
    if entry not in items:
        raise AssertionError(f"{crate}: no fn {entry}")
    order, seen, queue = [], {entry}, [entry]
    while queue:
        name = queue.pop(0)
        doc, body = items[name]
        if doc:
            order.append((name, doc))
        for call in re.findall(r"\b([a-z_][A-Za-z0-9_]*)\s*\(", body):
            if call in items and call not in seen:
                seen.add(call)
                queue.append(call)
    if not order:
        raise AssertionError(f"{crate}::{entry}'s chain carries no `///` doc at all")
    return order


def carry_docs(crate, entries, *, lead):
    """The rule-13 sentence for a rationale, with completeness DERIVED.

    `entries` are the chain roots the fixture actually calls. Every documented
    fn reachable from any of them is carried, in call order, deduplicated across
    roots -- so a header cannot claim a complete carry that is not one, and
    adding a documented helper upstream in `kali_common` breaks the generator
    instead of silently shipping an under-carried rationale.
    """
    carried, seen = [], set()
    for entry in entries:
        for name, doc in doc_chain(crate, entry):
            if name in seen:
                continue
            seen.add(name)
            carried.append((name, doc))
    body = " ".join(f'{name}: "{doc}"' for name, doc in carried)
    return f"{lead} {body}"


def check_captured(name, stem, needles):
    """A capture is only usable while the `.rs` still builds it the same way.

    Every needle must still be present in the source that produced the capture.
    These are the `format!` template's own literal segments and the
    `kali_common::` call spellings -- if a source edit removes one, the shipped
    capture is stale and this raises rather than emitting it.
    """
    text = rs(stem)
    missing = [n for n in needles if n not in text]
    if missing:
        raise AssertionError(
            f"capture {name} is stale: {stem}.rs no longer contains {missing!r}")
    return getattr(CAP, name)


# --------------------------------------------------------------------------
# Header boilerplate, DERIVED per file rather than restated 17 times.
# --------------------------------------------------------------------------

CK = "crates/kali_common/src"

RULE13_CHAIN = (
    "RULE 13 -- CROSS-CRATE HELPER DOCS, CARRIED, AND THE CHAIN IS DERIVED "
    "RATHER THAN ENUMERATED. The fixtures are built one level removed inside "
    "`kali_common` and the case reproduces what those helpers computed, so "
    "ruling 6's test comes out the CARRY way, not the "
    "runner-infrastructure-exemption way. Which helpers that is was not decided "
    "here: `gen_task19_batch2.doc_chain` walks the call graph from each fixture "
    "builder and carries every documented fn it reaches, transitively, so a "
    "documented helper added upstream breaks the generator instead of quietly "
    "going uncarried. The first version of this batch enumerated the chain by "
    "hand and under-carried it -- two documented helpers, on this file and on "
    "both of the Number-predicate files -- which is exactly the failure a derived "
    "chain cannot have.")


def head(stem, subject):
    return [f"Migrated from tests/{stem}.rs.",
            f"  SOURCE REF: {SOURCE_REF}",
            "",
            subject,
            ""]


def arithmetic(decl, invocations, cases, axes=None):
    """Rule 7 requires the matrix arithmetic to be recorded, INCLUDING a
    declined matrix and why. `decl` is the prose reason; the sum is computed
    here from the numbers the caller passes and the identity is asserted, so a
    header cannot state arithmetic that does not close."""
    product = 1
    for values in (axes or {}).values():
        product *= len(values)
    if invocations != cases * product:
        raise AssertionError(
            f"matrix arithmetic does not close: {invocations} invocation(s) != "
            f"{cases} case(s) x {product}")
    if axes:
        shape = " x ".join(f"{a}({len(v)})" for a, v in axes.items())
        line = (f"MATRIX DECLARED. {invocations} source helper invocation(s) == "
                f"{cases} case(s) x {shape} == {cases} x {product}. {decl}")
    else:
        line = (f"MATRIX DECLINED. {invocations} source helper invocation(s) == "
                f"{cases} case(s) x 1 -- no axis every case varies over uniformly "
                f"(rule 7, U1). {decl}")
    return [line, ""]


U2_INERT = (
    "U2 CHECK, RUN RATHER THAN ASSUMED. `[source]` is one flat FILE-WIDE map that "
    "`expand()` clones into every trial, so a fixture a sibling case needs is "
    "present in this case's trial too. Measured, "
    "with a known positive first, because a green that was never made red is not "
    "evidence:\n"
    "\n"
    "  * a sandbox policy file present in the working directory but NOT named on "
    "argv is inert. Control: the same deny-everything policy PASSED as "
    "`--sandbox` turns `console.log(1)` into `error[E9007]: inferred effect "
    "'Console.Write' is not permitted by the active policy`, rc=5; the same three "
    "files present as `policy.json` / `tiny.policy.json` / `kali.policy.json` and "
    "not passed leave `kali run m.ts` printing `1`, rc=0.\n"
    "  * a sibling `*.test.*` fixture is inert BECAUSE every case names its "
    "program explicitly on argv, exactly as the source did. Control: a bare "
    "`kali test` in a directory holding `main.test.js` and `other.test.js` "
    "discovers both and prints `ok 2`; `kali test main.test.js` in that same "
    "directory prints `ok 1`.\n"
    "  * `kali.json` IS auto-discovered as a manifest and would not be inert. No "
    "fixture in this batch is named `kali.json`; recorded because the next batch's "
    "author cannot tell from the tree that the name is special.\n"
    "\n"
    "No fixture here is one whose mere PRESENCE or ABSENCE is a case's point, "
    "which is the shape U2 exists to stop.")


def u5(keys):
    return ("`[source]` KEYS ARE VARIANT-SUFFIXED (U5). The source wrote each of "
            "these programs to the same filename under a fresh directory per "
            f"`#[test]`; `[source]` here is one file-wide namespace, so {keys} "
            "could not share one key -- the last body written would win and the "
            "other cases would silently run the wrong program. Each case owns a "
            "key named after it and passes that name as argv.")


def extra_ok(pairs):
    return [f"EXTRA-OK: {v!r} -- {why}" for v, why in pairs]


CC_RED_PER_CASE = (
    "PER-CASE ATTRIBUTION (`from N/M cases` lines). A comment attached to one "
    "`#[test]` fn belongs in the rationale of the case that fn produced and "
    "nowhere else. U6 says so and calls copying all of a file's comments into "
    "all of its cases \"over-attribution ... forbidden, even though it turns "
    "the checker green\".")

CC_RED_FILE_WIDE = (
    "FILE-WIDE PROSE IN THE HEADER (`from ALL N cases` lines). The source's "
    "`//!` module doc, and any helper doc carried under ruling 6's exemption, "
    "describe the whole file rather than any one case, so rule 12 puts them in "
    "this file's `#` header -- which `comment_coverage.py` deliberately does "
    "not read as coverage. Every such line is prose that IS carried, in the "
    "one place the rule says it belongs.")

CC_RED = (
    "CONSEQUENCE FOR THE GATES -- `comment_coverage.py` IS EXPECTED-RED (rc=1) "
    "ON THIS PAIR [classes: {classes}]. The checker asks whether every source "
    "comment line appears in EVERY case's rationale, and it reports two "
    "different things that way:\n"
    "\n"
    "  * {per_case}\n"
    "  * {file_wide}\n"
    "\n"
    "U6 anticipates the first exactly: \"on such a file the checker's false "
    "`missing` must be documented in the header instead.\" That is what this "
    "paragraph is. THE CLASS LIST ABOVE IS GATED, not asserted: "
    "`gen_task19_batch2.check_gate_declarations` re-runs the checker and "
    "requires the classes its output actually contains to match the ones named "
    "here -- the first version of this paragraph claimed every `MISSING` was "
    "per-test attribution, which was false on five of seven files, and nothing "
    "would have caught it. Reproduce with:\n"
    "\n"
    "  python3 tools/task-18-browser-pilot/comment_coverage.py \\\n"
    "    crates/kali_cli/tests/{stem}.rs \\\n"
    "    crates/kali_cli/tests/cases/{family}/{toml}.toml")

CC_EMPTY = (
    "CONSEQUENCE FOR THE GATES -- `comment_coverage.py` IS EXPECTED-RED (rc=2) ON "
    "THIS PAIR, AND THE RED IS THE GATE WORKING. The source carries no comment "
    "lines at all, so the checker's ruling-5 zero-line floor fires: without it a "
    "pair with nothing to check reports a vacuous green, which is the dangerous "
    "direction. Stated explicitly because *no prose* and *prose missed* are "
    "otherwise indistinguishable to a later reader. Reproduce with:\n"
    "\n"
    "  python3 tools/task-18-browser-pilot/comment_coverage.py \\\n"
    "    crates/kali_cli/tests/{stem}.rs \\\n"
    "    crates/kali_cli/tests/cases/{family}/{toml}.toml")

CF_VACUOUS = (
    "CONSEQUENCE FOR THE GATES -- `check_fixtures.py` IS EXPECTED-RED (rc=2) ON "
    "THIS PAIR, AND THE RED IS THE GATE WORKING. It looks for fixture-shaped "
    "string LITERALS in the `.rs` and finds none, because both programs are built "
    "one level removed inside `kali_common` and exist as no literal in this "
    "source at all. rc=2 is its vacuity floor -- \"found no fixtures to check\", "
    "the dangerous direction -- and it is the correct answer: this pair's rule-9 "
    "fidelity is carried by the CAPTURE discipline instead (the fixture text is "
    "the byte-exact output of executing the real code, and "
    "`gen_task19_batch2.check_captured` re-checks each capture against this "
    "source before emitting it). Reproduce with:\n"
    "\n"
    "  python3 tools/task-18-browser-pilot/check_fixtures.py \\\n"
    "    crates/kali_cli/tests/{stem}.rs \\\n"
    "    crates/kali_cli/tests/cases/{family}/{toml}.toml")


U8_MARKER = object()

U8_RED = (
    "CONSEQUENCE FOR THE GATES -- `check_rationale_fn_names.py` (U8) IS "
    "EXPECTED-RED (rc=1) ON THIS PAIR, AND IT CANNOT BE MADE GREEN WITHOUT "
    "BREAKING RULE 12. The gate requires every backticked fn-shaped identifier "
    "in this file to resolve against the source `.rs`'s own fn list. The ones "
    "it cannot resolve here are carried VERBATIM out of the source's own "
    "comments -- rule 12 requires the carry, U7 forbids rewording it -- and "
    "they are: {names}\n"
    "\n"
    "{remedy}\n"
    "\n"
    "THE NAMES AND THEIR CLASSIFICATION ARE DERIVED, not asserted: "
    "`gen_task19_batch2.u8_reason` re-runs the gate, reads back the identifiers "
    "it could not explain, and decides for each whether the source DEFINES it "
    "inside a fixture program. The first version of this paragraph said all of "
    "them named other-crate or standard-library items, which was false here and "
    "would have sent a reader to a remedy that could not work. Reproduce with:\n"
    "\n"
    "  python3 tools/task-18-browser-pilot/check_rationale_fn_names.py \\\n"
    "    crates/kali_cli/tests/{stem}.rs \\\n"
    "    crates/kali_cli/tests/cases/{family}/{toml}.toml")

U8_REMEDY_ALLOWLIST = (
    "REMEDY: a CLI-family entry in the gate's own known-identifier allowlist. "
    "These name items in other crates or in the Rust standard library, which "
    "this file's source never declares, so no amount of correct prose resolves "
    "them. The pilot flagged the same gap on `nullish/assign_reject.toml` and "
    "`misc/growable_array_fail_closed.toml`.")

U8_REMEDY_FIXTURE = (
    "REMEDY: NOT an allowlist entry -- and this is why the classification "
    "matters. These are JAVASCRIPT functions the source DEFINES inside its own "
    "fixture program text, so an allowlist would have to grow a name per "
    "fixture and would still be listing things that are not Rust items at all. "
    "The gate's premise is that a case file's prose only names Rust fns in its "
    "own test source; a CLI target whose comments quote the program under test "
    "breaks that premise structurally. The fix is for the gate to read fixture "
    "literals as a source of defined names, or to stop treating a backticked "
    "token inside carried prose as a citation -- an instrument change either "
    "way, and not this batch's to make.")


def u8_reason(stem, toml_path):
    """The U8 paragraph for this pair, with its names and remedy DERIVED.

    Runs the gate, reads the identifiers it could not explain, and asks of each
    whether the SOURCE defines a function of that name inside a fixture string
    literal. A name that is defined in the program under test needs a different
    remedy from one that lives in another crate, and the first version of this
    paragraph asserted the second for a file where every name was the first.
    """
    import subprocess
    out = subprocess.run(
        [sys.executable,
         os.path.join(REPO, "tools/task-18-browser-pilot/check_rationale_fn_names.py"),
         _gate_source(stem, "check_rationale_fn_names.py"), toml_path],
        cwd=REPO, capture_output=True, text=True)
    names = re.findall(r"UNEXPLAINED: `([^`]+)`", out.stdout)
    if not names:
        return None
    fixtures = "\n".join(x["value"] for x in find_string_literals(rs(stem)))
    in_fixture = [n for n in names
                  if re.search(r"\bfunction\s+" + re.escape(n) + r"\s*\(", fixtures)]
    rendered = ", ".join(f"`{n}`" for n in sorted(names))
    if in_fixture and len(in_fixture) == len(names):
        return U8_RED.replace("{names}", rendered + " -- every one of them a function "
                              "DEFINED IN THIS SOURCE'S OWN FIXTURE PROGRAM.") \
                     .replace("{remedy}", U8_REMEDY_FIXTURE)
    if in_fixture:
        return U8_RED.replace(
            "{names}", rendered + " -- of which "
            + ", ".join(f"`{n}`" for n in sorted(in_fixture))
            + " are functions DEFINED IN THIS SOURCE'S OWN FIXTURE PROGRAM and the rest "
              "name items in other crates or in the standard library.") \
            .replace("{remedy}", U8_REMEDY_FIXTURE + " " + U8_REMEDY_ALLOWLIST)
    return U8_RED.replace("{names}", rendered + ".").replace("{remedy}", U8_REMEDY_ALLOWLIST)


U4_TRIM = (
    "U4 TRIM-AND-KEEP: THE FIFTH `#[test]` FN IS RETAINED HAND-WRITTEN, AND THE "
    "SPLIT IS DERIVED RATHER THAN CHOSEN. `assert_frozen_object_has_own` "
    "(`:79-131`, numbered against the pre-trim blob the retained file's own "
    "`PRE-TRIM REF:` names) asserts nothing beyond `output.status.success()` "
    "for any test in the file: its "
    "`if json_output` block and its `else if command == \"run\"` block are both "
    "UNREACHABLE, because its only caller passes `json_output = false` and "
    "`command = \"check\"`. Eleven literals the audit extracts from those blocks "
    "are therefore DEAD -- a value computed but never asserted, which rule 2 "
    "forbids turning into a claim and rule 3 forbids shipping a red audit around. "
    "Controller ruling R1 sends an unreachable-code claim to a §5.11 retention.\n"
    "\n"
    "WHICH TESTS REACH IT -- the enumerating command, run before this sentence "
    "(ruling 13):\n"
    "\n"
    "  cd crates/kali_cli/tests && python3 - <<'EOF'\n"
    "  import re\n"
    "  t = open(\"object_has_own_frozen_js_input.rs\").read()\n"
    "  for m in re.finditer(r\"#\\[test\\]\\s*\\nfn ([a-z_0-9]+)\\s*\\([^)]*\\)\\s*\\{(.*?)\\n\\}\", t, re.S):\n"
    "      print(m.group(1), sorted(set(re.findall(\n"
    "          r\"\\b(assert_frozen_object_has_own(?:_fails_closed)?)\\s*\\(\", m.group(2)))))\n"
    "  EOF\n"
    "\n"
    "EXACTLY ONE of the five reaches it -- `check_accepts_frozen_object_has_own_"
    "in_js_ts_jsx_tsx_input`. The other four route through "
    "`assert_frozen_object_has_own_fails_closed`, a disjoint helper that shares "
    "no assertion with it. So U4's trim applies and its whole-file clause does "
    "NOT: whole-file retention is legitimate only when EVERY test reaches the "
    "construct, and here four of five do not. One test retained, four migrated, "
    "16 trials.")


RULING6 = (
    "RULING 6 EXEMPTION, APPLIED TO A LOCAL HELPER DOC RATHER THAN A CROSS-CRATE "
    "ONE. The doc above describes how the Rust harness kept its own temp "
    "directories from colliding. The migrated case depends on nothing that helper "
    "computed: the case runner gives every trial its own `tempfile::tempdir()`, so "
    "the helper's job is now the runner's job -- which is ruling 6's own test for "
    "the exemption. Carried "
    "verbatim here so nothing is dropped (rule 12), and deliberately NOT "
    "replicated into every rationale, because attributing harness mechanics to "
    "every case is the over-attribution U6 forbids.")


def hoist_note(const_spelling, stem, family, toml):
    """CONSTANTS_HOIST with its paths filled in, then the `${...}` spelling
    inserted -- in that order, because the spelling carries braces that
    `str.format` would read as fields of its own."""
    text = CONSTANTS_HOIST.format(stem=stem, family=family, toml=toml)
    return [text.replace("<CONST>", const_spelling), ""]


_CC_CLASSES = {}


def cc_classes_of(stem, toml_path):
    """The classes `comment_coverage.py` actually reports for a rendered pair.

    DERIVED FROM THE GATE'S OWN OUTPUT (ruling 18 #1), so a header cannot name a
    class the checker does not report, or omit one it does.

    NOT CIRCULAR, and the reason is a property of the checker rather than an
    assumption: `comment_coverage.py` reads only `rationale` fields and never
    the `#` header, so the classes depend on the half of the file this arm does
    not write. `rendered()` therefore renders once with the classes unknown,
    measures them against that rendering, and renders again -- and asserts the
    second rendering is a fixed point.
    """
    import subprocess
    p = subprocess.run(
        [sys.executable, os.path.join(REPO, "tools/task-18-browser-pilot/comment_coverage.py"),
         _gate_source(stem, "comment_coverage.py"), toml_path],
        cwd=REPO, capture_output=True, text=True)
    found = []
    if re.search(r"from \d+/\d+ cases", p.stdout):
        found.append("per-case")
    if "from ALL " in p.stdout:
        found.append("file-wide")
    return found


def cc_classes(stem, toml_path):
    return _CC_CLASSES.get(toml_path, ["per-case", "file-wide"])


def gates(entries, stem, family, toml):
    out = []
    toml_path = os.path.join(CASES, family, toml + ".toml")
    for e in entries:
        text = e
        if e is U8_MARKER:
            text = u8_reason(stem, toml_path)
            if text is None:
                continue
        elif "{classes}" in e:
            found = cc_classes(stem, toml_path)
            text = e.replace("{classes}", ", ".join(found) or "none")
            text = text.replace("{per_case}",
                                CC_RED_PER_CASE if "per-case" in found
                                else "(no per-case lines on this pair)")
            text = text.replace("{file_wide}",
                                CC_RED_FILE_WIDE if "file-wide" in found
                                else "(no file-wide lines on this pair)")
        out.append(text.format(stem=stem, family=family, toml=toml))
        out.append("")
    return out


# --------------------------------------------------------------------------
# The 17 files.
# --------------------------------------------------------------------------

FILES = []


def add(family, toml, stem, header, source, cases, matrix=None, constants=None):
    FILES.append(dict(family=family, toml=toml, stem=stem, header=header,
                      source=source, cases=cases, matrix=matrix,
                      constants=constants or {}))


CONSTANTS_HOIST = (
    "`[constants]` HOIST, AND WHY IT IS MANDATORY HERE RATHER THAN DECLINED. The "
    "program under test is a declared source constant -- a Rust const item -- which "
    "`audit-case-migration.py` extracts as a `rule constants` claim that must "
    "appear on the surface `assertion_strings()` searches. That surface includes "
    "`[constants]` values and deliberately EXCLUDES `[source]` bodies, so with the "
    "body inlined the audit reports the constant absent and exits 1 -- and rule 3 "
    "makes the audit absolute. Both placements were run against both gates rather "
    "than either instruction being inherited:\n"
    "\n"
    "  placement                      check_fixtures.py   audit-case-migration.py\n"
    "  hoisted into `[constants]`     RED (UNMATCHED)     GREEN\n"
    "  inlined into `[source]`        GREEN               RED (rule constants absent)\n"
    "\n"
    "This is the pilot's bitwise_operators_runtime deadlock reproduced on a "
    "second family, and its derived discriminator is what decides it: HOIST when "
    "the shared text is a declared source constant (the audit requires it), "
    "DECLINE when it is an inline literal (ruling 7). There is no third placement "
    "that is not the one the instrument dispatch closed -- inlining the body AND "
    "leaving an unreferenced `[constants]` entry is now itself an `AUDIT FAILED`.\n"
    "\n"
    "CONSEQUENCE FOR THE GATES -- `check_fixtures.py` IS EXPECTED-RED (rc=1) ON "
    "THIS PAIR. It searches `[source]` values and step `body` verbatim, and this "
    "file's `[source]` value is <CONST>, resolved only at expansion "
    "time. The red is a fixable tool limitation, not a property of the format: "
    "`check_fixtures.py` already carries `_substitute`, an exact port of "
    "`expand.rs::substitute`, and applies it only in the `--argv-correspondence` "
    "arm. Reproduce with:\n"
    "\n"
    "  python3 tools/task-18-browser-pilot/check_fixtures.py \\\n"
    "    crates/kali_cli/tests/{stem}.rs \\\n"
    "    crates/kali_cli/tests/cases/{family}/{toml}.toml")


def R(stem, text):
    return f"Migrated from {stem}.rs. {text}"


# ---- 1. exponentiation_operator -------------------------------------------
def f_exponentiation_operator():
    stem = "exponentiation_operator"
    body = lit(stem, "2 ** 3")
    header = (
        head(stem, "`**` lowers to exponentiation at runtime: `console.log(2 ** 3)` prints 8.")
        + arithmetic("The single `#[test]` fn is a single `kali run` invocation.", 1, 1)
        + ["ASSERTION SHAPE. The source asserts `output.status.success()` and "
           "`stdout.contains(\"8\")`. `stdout` has a substring form, so the "
           "`.contains` stays a `stdout_contains` rather than being strengthened to "
           "the exact `\"8\\n\"` this trial was observed to emit -- ruling 3, "
           "clause 3: mirror the source, and do not strengthen because you observed "
           "the exact output.",
           ""]
        + gates([CC_EMPTY], stem, "misc", "exponentiation_operator"))
    cases = [dict(name="run_supports_exponentiation_operator_in_js_input",
                  rationale=R(stem, "`2 ** 3` must evaluate to 8 at runtime. The source "
                                    "carries no comments, so rules 12 and 13 have nothing "
                                    "to carry into this rationale -- stated rather than "
                                    "left to be inferred from silence."),
                  steps=[dict(args=["run", "main.js"], exit="success",
                              stdout_contains=["8"])])]
    add("misc", "exponentiation_operator", stem, header, {"main.js": body}, cases)


# ---- 2. runtime_fasta_capstone --------------------------------------------
def f_fasta_capstone():
    stem = "runtime_fasta_capstone"
    body = check_captured("CAP_FASTA_CAPSTONE__SHELL", stem,
                          ["const FASTA_CAPSTONE_SHELL: &str", "var n = +process.argv[2];",
                           "fastaRandom(5 * n, HomoSap);"])
    golden = lit(stem, "GGCCGGGCGCGGTGGC\n>TWO IUB ambiguity codes")
    header = (
        head(stem, "The fasta capstone shell run under the node API surface with "
                   "`N` supplied through `process.argv`, pinned against the node golden.")
        + arithmetic("The single `#[test]` fn is a single `kali run --api node ... -- 8` "
                     "invocation.", 1, 1)
        + ["FIXTURE PROVENANCE (rules 8/9). `FASTA_CAPSTONE_SHELL` is a `const &str`, "
           "so its text is reproduced here byte-for-byte from the real declaration; "
           "the generator pulls it through a dump target that `include!`s the source "
           "rather than retyping it.",
           "",
           "ARGV. `--` separates kali's own arguments from the program's, and `8` is "
           "the `N` the source passes; `n = +process.argv[2]` reads it.",
           ""]
        + gates([], stem, "runtime", "fasta_capstone")
        + hoist_note("`${FASTA_CAPSTONE_SHELL}`", stem, "runtime", "fasta_capstone"))
    cases = [dict(
        name="full_fasta_shell_matches_node_at_small_n",
        rationale=R(stem, "The source's assertion is an exact `assert_eq!` on the whole "
                          "stdout against a node-derived golden, so it becomes an exact "
                          "`stdout` pin (ruling 3, clause 1).\n\n"
                          + para(stem, "The full fasta-node-1 shell") + "\n\n"
                          + para(stem, "GOLDEN: derived by running FASTA_CAPSTONE_SHELL")),
        steps=[dict(args=["run", "--api", "node", "main.ts", "--", "8"], exit="success",
                    stdout=golden)])]
    add("runtime", "fasta_capstone", stem, header,
        {"main.ts": "${FASTA_CAPSTONE_SHELL}"}, cases,
        constants={"FASTA_CAPSTONE_SHELL": body})


# ---- 3. runtime_fasta_output ----------------------------------------------
def f_fasta_output():
    stem = "runtime_fasta_output"
    rand = check_captured("CAP_FASTA_OUTPUT__RANDOM", stem,
                          ["const FASTA_RANDOM_SHELL: &str", "fastaRandom(70, IUB);"])
    rep = check_captured("CAP_FASTA_OUTPUT__REPEAT", stem,
                         ["const FASTA_REPEAT_SHELL: &str", "fastaRepeat(120, ALU);"])
    g_rand = lit(stem, "cttBtatcatatgctaKggNcataaaSatgtaaaDcDRtBggDtctttataattcBgtcg")
    g_rep = lit(stem, "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGA")
    header = (
        head(stem, "The two fasta output-layer shells -- `fastaRandom` over `IUB` and "
                   "`fastaRepeat` over `ALU` -- pinned against their node goldens.")
        + arithmetic("Each `#[test]` fn runs a different program and pins a different "
                     "stdout.", 2, 2)
        + [u5("the two shells"), ""]
        + [U2_INERT, ""]
        + extra_ok([("fasta_random_shell.ts", "U5 variant-suffixed `[source]` key surfaced as an "
                     "argv token; it is a fixture FILENAME, not a claim about behaviour"),
                    ("fasta_repeat_shell.ts", "U5 variant-suffixed `[source]` key surfaced as an "
                     "argv token; it is a fixture FILENAME, not a claim about behaviour")])
        + [""]
        + gates([CC_RED], stem, "runtime", "fasta_output")
        + hoist_note("`${FASTA_RANDOM_SHELL}` / `${FASTA_REPEAT_SHELL}`",
                     stem, "runtime", "fasta_output"))
    cases = [
        dict(name="fasta_random_shell_matches_node",
             rationale=R(stem, "Exact `assert_eq!` on stdout against a node golden becomes an "
                               "exact `stdout` pin (ruling 3, clause 1).\n\n"
                               + para(stem, "GOLDEN: derived by running FASTA_RANDOM_SHELL")),
             steps=[dict(args=["run", "fasta_random_shell.ts"], exit="success", stdout=g_rand)]),
        dict(name="fasta_repeat_shell_matches_node",
             rationale=R(stem, "Exact `assert_eq!` on stdout against a node golden becomes an "
                               "exact `stdout` pin (ruling 3, clause 1).\n\n"
                               + para(stem, "ALU here is 84 chars")),
             steps=[dict(args=["run", "fasta_repeat_shell.ts"], exit="success", stdout=g_rep)]),
    ]
    add("runtime", "fasta_output", stem, header,
        {"fasta_random_shell.ts": "${FASTA_RANDOM_SHELL}",
         "fasta_repeat_shell.ts": "${FASTA_REPEAT_SHELL}"}, cases,
        constants={"FASTA_RANDOM_SHELL": rand, "FASTA_REPEAT_SHELL": rep})


# ---- 4. standalone_non_literal_iterator_sources ---------------------------
def f_standalone_iter():
    stem = "standalone_non_literal_iterator_sources"
    setsrc = check_captured("CAP_STANDALONE_ITER__SET", stem,
                            ["fn non_literal_set_source", "new Set(values.filter(Boolean))"])
    mapsrc = check_captured("CAP_STANDALONE_ITER__MAP", stem,
                            ["fn non_literal_map_source", "new Map(values.filter(Boolean))"])
    msg = ("for-of array iteration lowering is unavailable unless the iterable is a literal "
           "array or supported string iterable with literal elements and the loop target is a "
           "variable declaration or simple identifier binding; use a supported loop form or "
           "the later compatibility path")
    header = (
        head(stem, "A `Set`/`Map` built from a non-literal iterator source is rejected "
                   "with E5506, in `run` and in `test`, in text and in `--output json` mode.")
        + arithmetic("Both `#[test]` fns loop `[(set, set-main.js), (map, map-main.js)]` and "
                     "call the helper twice per element, once with `json_output = false` and "
                     "once with `true`, so 2 fns x 2 sources x 2 output modes == 8. A "
                     "`[matrix]` over the output mode was DECLINED and not merely unused: "
                     "`--output json` changes the ASSERTION SHAPE (a json envelope replaces a "
                     "stderr substring claim), not just an argv token, so no axis fans these "
                     "cases uniformly.", 8, 8)
        + ["`[source]` KEYS ARE THE SOURCE'S OWN. `set-main.js` and `map-main.js` are "
           "the filenames the source itself writes, so no U5 rename was needed and no "
           "`# EXTRA-OK:` declaration is required for them.",
           ""]
        + [U2_INERT, ""]
        + ["THE `errors` NON-EMPTINESS CLAIM. The source asserts "
           "`!errors.is_empty()` and then indexes `errors[0]`. Pinning "
           "`json.errors.0.code` is strictly stronger than the non-emptiness claim "
           "and subsumes it -- a path that resolves proves the array has an element "
           "-- so no separate claim is written for it (rule 2: nothing invented, "
           "nothing weakened).",
           "",
           "THE `literal array` SUBSTRING CLAIM. The source's `message.contains(\"literal "
           "array\")` is a plain `.contains` against a `json` STRING LEAF, which is "
           "ruling 3 clause 4's amended case: it becomes `json_count` with "
           "`at_least = 1`, not an exact `json.errors.0.message` pin, because the "
           "source's own assertion is not exact. The non-json branch's two "
           "`stderr.contains` claims stay `stderr_contains`.",
           ""]
        + ["THE E5506 MESSAGE THIS FILE DOES NOT PIN, recorded so a reader can see "
           "where the `literal array` needle comes from. The trial emits: \"" + msg
           + "\". Only the substring the source asserts is claimed.",
           ""]
        + extra_ok([("errors.0.message", "a `json_count` PATH, not an assertion needle. "
                     "`check_extra_claims.py` treats every claim-bearing string as a claim, "
                     "and a dotted jsonpath exists as no contiguous literal in the `.rs`; the "
                     "source spells the same path as `errors[0][\"message\"]`. The path is "
                     "gated elsewhere and more precisely: `audit-case-migration.py` requires "
                     "every segment of a `json_count` path to be a JSON key the source "
                     "actually indexed, and it is green on this pair")])
        + [""]
        + gates([CC_EMPTY], stem, "misc", "standalone_non_literal_iterator_sources"))

    cases = []
    for src, key in (("set", "set-main.js"), ("map", "map-main.js")):
        for cmd in ("run", "test"):
            for js in (False, True):
                pre = ["--output", "json"] if js else []
                step = dict(args=pre + [cmd, key], exit=1)
                if js:
                    step["json_paths"] = {
                        "schemaVersion": 1, "command": cmd, "success": False, "exitCode": 1,
                        "errors.0.code": "E5506",
                    }
                    step["json_count"] = [dict(path="errors.0.message",
                                               needle="literal array", at_least=1)]
                else:
                    step["stderr_contains"] = ["E5506", "literal array"]
                mode = "json" if js else "text"
                cases.append(dict(
                    name=f"{src}_source_rejects_in_{cmd}_{mode}_output",
                    rationale=R(stem, "Source `#[test]` fn: "
                                      f"standalone_non_literal_set_and_map_sources_reject_in_{cmd}"
                                      " -- it loops both sources and both output modes, so it "
                                      "splits into four named siblings under rule 5; the fn name "
                                      "is recorded here so it survives the source's deletion. "
                                      f"A `new {src.capitalize()}(...)` whose argument is a "
                                      "`.filter(Boolean)` call rather than a literal array is "
                                      "not a supported iterator source, so `kali "
                                      f"{cmd}` must reject it. The source asserts "
                                      "`!status.success()` AND `status.code() == Some(1)`; the "
                                      "exact code is the stronger of the two and is pinned "
                                      "(ruling 3, clause 1). "
                                      + ("The `--output json` branch asserts the envelope's "
                                         "`schemaVersion`, `command`, `success` and `exitCode`, "
                                         "the first error's `code`, and that its `message` "
                                         "mentions `literal array`."
                                         if js else
                                         "The text branch asserts only that stderr carries "
                                         "`E5506` and `literal array`.")
                                      + " The source carries no comments, so rules 12 and 13 "
                                        "have nothing to carry into this rationale."),
                    steps=[step]))
    add("misc", "standalone_non_literal_iterator_sources", stem, header,
        {"set-main.js": setsrc, "map-main.js": mapsrc}, cases)


# ---- 5. module_var_object_compound ----------------------------------------
def f_module_var_object_compound():
    stem = "module_var_object_compound"
    keys = {
        "object_initialized_binding_compound.ts": lit(stem, "var o = {x:1};"),
        "array_initialized_binding_compound.ts": lit(stem, "var a = [1, 2];"),
        "numeric_var_local_compound.ts": lit(stem, "var k = 0;"),
    }
    header = (
        head(stem, "A compound assignment on an object- or array-initialized binding has "
                   "no scalar lowering and must reject fail-closed; a genuine numeric "
                   "local must still compile and run.")
        + arithmetic("Each `#[test]` fn runs a different program.", 3, 3)
        + [u5("the three programs"), ""]
        + [U2_INERT, ""]
        + ["RULE 11 / RULING 17 -- AN OR-SHAPED ASSERTION, RESOLVED BY OBSERVATION. "
           "Both reject tests assert "
           "`stderr.contains(\"E5506\") || stderr.contains(\"not a provably scalar\")`. "
           "The format has no disjunction, so the OR is resolved against the real "
           "binary. Observed: on BOTH cells the stream is stderr and BOTH disjuncts "
           "hold -- the object case emits `error[E5506]: compound assignment on "
           "binding 'o' is unavailable: it is not a provably scalar number or string "
           "...`, and the array case emits two diagnostics, the second of which is the "
           "same shape for binding 'a'. Ruling 17 therefore applies: pin the FIRST "
           "disjunct in source order, `E5506`, and disclose the other rather than "
           "pinning both. Pinning all true disjuncts would be a rule-2 invention -- "
           "the source never asserted `not a provably scalar` unconditionally, and "
           "pinning it would fail the case on a benign rewording the source "
           "explicitly tolerates.",
           ""]
        + extra_ok([(v, "U5 variant-suffixed `[source]` key surfaced as an argv token; it is a "
                     "fixture FILENAME, not a claim about behaviour") for v in keys])
        + [""]
        + gates([CC_RED], stem, "misc", "module_var_object_compound"))
    disj = ("Rule 11 / ruling 17: the source accepts `E5506` OR `not a provably scalar`; "
            "both hold on this stream, so the first in source order is pinned and the "
            "second is disclosed here rather than pinned.")
    cases = [
        dict(name="object_initialized_binding_compound_rejects",
             rationale=R(stem, para(stem, "A compound assign on an object-initialized binding")
                         + "\n\n" + disj),
             steps=[dict(args=["run", "object_initialized_binding_compound.ts"], exit="failure",
                         stderr_contains=["E5506"])]),
        dict(name="array_initialized_binding_compound_rejects",
             rationale=R(stem, para(stem, "The array-literal declarator lane")
                         + "\n\n" + disj),
             steps=[dict(args=["run", "array_initialized_binding_compound.ts"], exit="failure",
                         stderr_contains=["E5506"])]),
        dict(name="numeric_var_local_compound_still_runs",
             rationale=R(stem, para(stem, "A genuine numeric var local still compiles")
                         + "\n\nThe source's `assert_eq!` on the whole stdout is exact, so it "
                           "becomes an exact `stdout` pin (ruling 3, clause 1)."),
             steps=[dict(args=["run", "numeric_var_local_compound.ts"], exit="success",
                         stdout=lit(stem, "42\n", exact=True))]),
    ]
    add("misc", "module_var_object_compound", stem, header, keys, cases)


# ---- 6. closure_return_isolation ------------------------------------------
def f_closure_return_isolation():
    stem = "closure_return_isolation"
    noargs = [x["value"] for x in find_string_literals(rs(stem))
              if "consumeArray" in x["value"] and "arrayLiteralFirst" not in x["value"]]
    if len(noargs) != 1:
        raise AssertionError(f"{stem}.rs: {len(noargs)} argument-free enum-preamble programs")
    keys = {
        "decl-only.js": lit(stem, "const f = (x) => x;"),
        "call-arrow.js": lit(stem, "const h = (x) => x + 1;"),
        "block-arrow.js": lit(stem, "const bump = () =>"),
        "enum-preamble.js": lit(stem, "arrayLiteralFirst"),
        "enum-preamble-no-args.js": noargs[0],
    }
    header = (
        head(stem, "Const-bound arrows must compile as standalone wasm functions, so an "
                   "arrow declaration cannot emit a `return` into the enclosing function "
                   "and truncate it.")
        + [para(stem, "Regression tests for the const-bound-arrow return-escape miscompile"), ""]
        + ["FILE-WIDE PROSE (rule 12). The paragraph above is the source's `//!` module "
           "doc: it describes the whole file rather than any one helper, so it lives in "
           "this header. Per-test prose lives in the rationale of the case it belongs "
           "to, per U6.",
           ""]
        + arithmetic("FOUR source `#[test]` fns become FIVE `[[case]]` siblings, and the "
                     "extra one is rule 5, not a fold: "
                     "`run_object_enumeration_survives_const_arrow_preamble` runs TWO "
                     "independent programs and makes independent assertions about each "
                     "(one rejected, one required to run to completion), so it splits into "
                     "two named siblings. Rule 6's 1:1 direction -- never FOLD two fns into "
                     "one case -- is untouched.", 5, 5)
        + ["`[source]` KEYS ARE THE SOURCE'S OWN. This source already gives every "
           "program a distinct filename, so no U5 rename was needed and no "
           "`# EXTRA-OK:` declaration is required for them.",
           ""]
        + [U2_INERT, ""]
        + gates([CC_RED, U8_MARKER], stem, "misc", "closure_return_isolation"))
    cases = [
        dict(name="run_executes_statements_after_const_expression_bodied_arrow_declaration",
             rationale=R(stem, "Declaring an expression-bodied arrow must not truncate the "
                               "statements after it. Exact `assert_eq!` on stdout, so an exact "
                               "`stdout` pin (ruling 3, clause 1)."),
             steps=[dict(args=["run", "decl-only.js"], exit="success",
                         stdout=lit(stem, "A\nB\n", exact=True))]),
        dict(name="run_calls_const_expression_bodied_arrow_via_binding",
             rationale=R(stem, "The arrow must be callable through its binding and must not "
                               "truncate what follows. Exact `assert_eq!` on stdout, so an "
                               "exact `stdout` pin."),
             steps=[dict(args=["run", "call-arrow.js"], exit="success",
                         stdout=lit(stem, "42\nafter\n", exact=True))]),
        dict(name="run_executes_block_bodied_arrow_body_at_call_time_not_declaration",
             rationale=R(stem, para(stem, "Class-2 shape")
                         + "\n\nExact `assert_eq!` on stdout, so an exact `stdout` pin."),
             steps=[dict(args=["run", "block-arrow.js"], exit="success",
                         stdout=lit(stem, "bump\n0\nafter\n", exact=True))]),
        dict(name="array_literal_argument_to_a_user_function_is_rejected",
             rationale=R(stem, para(stem, "Class-4/5 shape")
                         + "\n\nRULE 5 SPLIT: this is the FIRST of the two independent programs "
                           "`run_object_enumeration_survives_const_arrow_preamble` runs. Its "
                           "own claims are `!status.success()` and `stderr.contains(\"E5506\")` "
                           "-- no stdout claim, so none is written (rule 2)."),
             steps=[dict(args=["run", "enum-preamble.js"], exit="failure",
                         stderr_contains=["E5506"])]),
        dict(name="object_enumeration_survives_const_arrow_preamble_without_arguments",
             rationale=R(stem, para(stem, "Return-escape coverage without the rejected argument")
                         + "\n\nRULE 5 SPLIT: the SECOND of the two independent programs "
                           "`run_object_enumeration_survives_const_arrow_preamble` runs. Exact "
                           "`assert_eq!` on stdout, so an exact `stdout` pin."),
             steps=[dict(args=["run", "enum-preamble-no-args.js"], exit="success",
                         stdout=lit(stem, "2\n", exact=True))]),
    ]
    add("misc", "closure_return_isolation", stem, header, keys, cases)


# ---- 7. heap_grow_runtime -------------------------------------------------
def f_heap_grow():
    stem = "heap_grow_runtime"
    keys = {
        "grow.ts": lit(stem, "total = total + a.length"),
        "recurse_grow.ts": lit(stem, "function recurse"),
        "tiny.policy.json": lit(stem, "\"maxMemoryMB\":4"),
        "oom.ts": lit(stem, "k<10000"),
        "span_arrays.ts": lit(stem, "a[19999]"),
    }
    header = (
        head(stem, "Allocation past the old 1 MB wall succeeds -- loop-driven, "
                   "recursion-driven and through the multi-page span path -- while an "
                   "allocation past a sandbox memory cap fails cleanly rather than "
                   "panicking.")
        + arithmetic("Each `#[test]` fn runs a different program.", 4, 4)
        + [u5("the fourth program, which the source wrote to `main.ts`,"), ""]
        + [U2_INERT, ""]
        + ["HELPER DOC CARRIED FROM THE SOURCE (rule 12). `unique_fixture_slug` "
           "(`heap_grow_runtime.rs:13-21`) carries:", ""]
        + ["  " + l for l in para(stem, "Build a process-wide-unique directory name").split("\n")]
        + ["", RULING6, ""]
        + ["ASSERTION SHAPES. Two tests assert `assert_eq!(stdout.trim(), N.to_string())` "
           "-- an EXACT assertion modulo trailing whitespace -- so each becomes an exact "
           "`stdout` pin whose value was captured from the real binary, never computed "
           "(ruling 3 clause 1; U9). The OOM test asserts only `!status.success()` and "
           "`!stderr.contains(\"panic\")`, which is an ABSENCE claim and becomes "
           "`stderr_absent`; it makes no positive stderr claim, so none is written "
           "(rule 2).",
           ""]
        + extra_ok([("span_arrays.ts", "U5 variant-suffixed `[source]` key surfaced as an argv "
                     "token; it is a fixture FILENAME, not a claim about behaviour"),
                    ("393216\n", "a deliberate live-captured exact pin. The source asserts "
                     "`stdout.trim() == (24 * 16384).to_string()` -- the expected value is "
                     "COMPUTED in Rust, so it exists as no literal anywhere in the `.rs` and "
                     "`check_extra_claims.py` cannot find it there. Captured from the real "
                     "binary per U9, never arithmetic done here"),
                    ("256064\n", "a deliberate live-captured exact pin. The source asserts "
                     "`stdout.trim() == (4001 * 64).to_string()` -- the expected value is "
                     "COMPUTED in Rust, so it exists as no literal anywhere in the `.rs`. "
                     "Captured from the real binary per U9, never arithmetic done here")])
        + [""]
        + gates([CC_RED, U8_MARKER], stem, "misc", "heap_grow_runtime"))
    cases = [
        dict(name="allocation_beyond_one_megabyte_succeeds",
             rationale=R(stem, para(stem, "~3 MB of i64 array storage")
                         + "\n\nThe source pins `stdout.trim() == (24 * 16384).to_string()`; "
                           "the exact stdout was captured from the real binary."),
             steps=[dict(args=["run", "grow.ts"], exit="success", stdout="393216\n")]),
        dict(name="recursive_allocation_beyond_wall_no_longer_traps",
             rationale=R(stem, para(stem, "NOTE: the task brief's Step 5")
                         + "\n\n" + para(stem, "4001 calls * (64+1)*8 bytes/array")
                         + "\n\n" + trailing(stem, "256064")
                         + "\n\nThe source pins `stdout.trim() == (4001 * 64).to_string()`; the "
                           "exact stdout was live-captured, never arithmetic done here."),
             steps=[dict(args=["run", "recurse_grow.ts"], exit="success", stdout="256064\n")]),
        dict(name="oom_past_sandbox_cap_fails_cleanly",
             rationale=R(stem, "An allocation loop under a 4 MB `resources.maxMemoryMB` policy "
                               "must fail cleanly. The source's only claims are "
                               "`!status.success()` and that stderr does NOT contain `panic`; "
                               "the second is an absence claim, carried as `stderr_absent`. "
                               "Nothing positive about the diagnostic is asserted by the "
                               "source, so nothing positive is pinned here (rule 2)."),
             steps=[dict(args=["run", "--sandbox", "tiny.policy.json", "oom.ts"], exit="failure",
                         stderr_absent=["panic"])]),
        dict(name="multi_page_array_allocations_are_correct",
             rationale=R(stem, para(stem, "Task 5 (page-pool allocator)")
                         + "\n\n" + trailing(stem, "4*19999")
                         + "\n\nExact `assert_eq!` on the whole stdout, so an exact `stdout` "
                           "pin (ruling 3, clause 1)."),
             steps=[dict(args=["run", "span_arrays.ts"], exit="success",
                         stdout=lit(stem, "80002\n", exact=True))]),
    ]
    add("misc", "heap_grow_runtime", stem, header, keys, cases)


# ---- 8. trap_diagnostics_runtime ------------------------------------------
def f_trap_diagnostics():
    stem = "trap_diagnostics_runtime"
    # U13's identity assertion, mechanical rather than eyeballed: the two
    # runaway-loop programs are DERIVED (every literal the source opens with
    # `let i = 0;`), their count is pinned, and their byte-identity is asserted.
    # A source edit that made them differ, or that added a third, raises here.
    fuels = [x["value"] for x in find_string_literals(rs(stem))
             if x["value"].startswith("let i = 0;")]
    if len(fuels) != 2:
        raise AssertionError(f"{stem}.rs: {len(fuels)} runaway-loop program(s), wanted 2")
    if fuels[0] != fuels[1]:
        raise AssertionError("the two runaway-loop programs are not byte-identical")
    fuel = fuel2 = fuels[0]
    keys = {
        "fuel_runaway.ts": fuel,
        "trees_64mb.ts": lit(stem, "bottomUpTree"),
        "policy.json": lit(stem, "\"maxCpuTimeMs\": 600000"),
        "stdout_before_trap.ts": lit(stem, "console.log(777)"),
        "quiet_trap.ts": fuel2,
    }
    header = (
        head(stem, "A fuel-exhaustion trap reports E4003 with an actionable message, "
                   "keeps pre-trap stdout, survives `--quiet`, and does not present as a "
                   "bare E4000 runtime trap; a raised fuel budget lets a deep object "
                   "workload finish correctly.")
        + arithmetic("Each `#[test]` fn runs a different command line.", 4, 4)
        + [u5("the four programs, which the source all wrote to `main.ts`,"), ""]
        + [U2_INERT, ""]
        + ["U13 -- A DUPLICATED FIXTURE BODY, AND WHY IT IS NOT HOISTED. "
           "`fuel_runaway.ts` and `quiet_trap.ts` carry BYTE-IDENTICAL program text: "
           "the source writes the same runaway loop under two labels. U13 says hoist a "
           "byte-identical shared body into `[constants]`; ruling 7 says decline when "
           "the shared text is an INLINE LITERAL rather than a declared source "
           "constant, because `check_fixtures.py` searches only `[source]` values and "
           "step `body`, so a hoisted body turns that gate red on a correct file. This "
           "is the inline-literal case, so the hoist is declined -- and U13's other "
           "half is honoured: the identity is asserted MECHANICALLY in "
           "`tools/migration/gen_task19_batch2.py`, which raises if the two literals "
           "ever stop being byte-identical, rather than being eyeballed.",
           ""]
        + ["HELPER DOC CARRIED FROM THE SOURCE (rule 12). `unique_fixture_slug` "
           "(`trap_diagnostics_runtime.rs:12-19`) carries:", ""]
        + ["  " + l for l in para(stem, "Build a process-wide-unique directory name").split("\n")]
        + ["", RULING6, ""]
        + ["ASSERTION SHAPES. `stderr.contains(...)` stays `stderr_contains` (ruling 3, "
           "clause 3); `!stderr.contains(\"E4000\")` is an absence claim and becomes "
           "`stderr_absent`; the one exact `assert_eq!` on stdout becomes an exact "
           "`stdout` pin.",
           ""]
        + extra_ok([(v, "U5 variant-suffixed `[source]` key surfaced as an argv token; it is a "
                     "fixture FILENAME, not a claim about behaviour")
                    for v in ("fuel_runaway.ts", "quiet_trap.ts", "stdout_before_trap.ts",
                              "trees_64mb.ts")])
        + [""]
        + gates([CC_RED, U8_MARKER], stem, "misc", "trap_diagnostics_runtime"))
    cases = [
        dict(name="fuel_exhaustion_reports_e4003_with_actionable_message",
             rationale=R(stem, para(stem, "Runs forever; exhausts the 60M default fuel budget")
                         + "\n\nThe diagnostic must name the code, say what happened, and point "
                           "at the knob that grants more compute; and it must NOT present as a "
                           "bare runtime trap, which is the `stderr_absent` claim."),
             steps=[dict(args=["run", "fuel_runaway.ts"], exit="failure",
                         stderr_contains=["E4003", "CPU fuel budget exhausted",
                                          "resources.maxCpuTimeMs"],
                         stderr_absent=["E4000"])]),
        dict(name="deep_object_workload_is_correct_to_64mb_under_raised_fuel_policy",
             rationale=R(stem, para(stem, "8000 iterations x itemCheck(depth-8 tree)")
                         + "\n\nWith `resources.maxCpuTimeMs` raised by a `--sandbox` policy the "
                           "workload finishes; the source's exact `assert_eq!` on stdout becomes "
                           "an exact `stdout` pin."),
             steps=[dict(args=["run", "--sandbox", "policy.json", "trees_64mb.ts"],
                         exit="success", stdout=lit(stem, "4088000\n", exact=True))]),
        dict(name="stdout_emitted_before_a_trap_is_not_lost",
             rationale=R(stem, "Output written before the trap must still be flushed to stdout, "
                               "and the trap must still be reported on stderr. Both source "
                               "claims are `.contains` against a field with a substring form, "
                               "so both stay `*_contains` (ruling 3, clause 3)."),
             steps=[dict(args=["run", "stdout_before_trap.ts"], exit="failure",
                         stdout_contains=["777"], stderr_contains=["E4003"])]),
        dict(name="quiet_run_still_reports_the_trap_diagnostic_on_stderr",
             rationale=R(stem, para(stem, "`--quiet` suppresses status text")),
             steps=[dict(args=["run", "--quiet", "quiet_trap.ts"], exit="failure",
                         stderr_contains=["E4003"])]),
    ]
    add("misc", "trap_diagnostics_runtime", stem, header, keys, cases)


# ---- 9. float_console_runtime ---------------------------------------------
def f_float_console():
    stem = "float_console_runtime"
    keys = {
        "float_division_results.ts": lit(stem, "console.log(6 / 2)"),
        "js_special_float_values.ts": lit(stem, "console.log(0 / -1)"),
        "concatenates_runtime_floats.ts": lit(stem, "console.log(\"v: \" + (7 / 2));\n"),
        "floats_from_mutable_locals_and_params.ts": lit(stem, "function show(v)"),
        "small_magnitudes_exponent_notation.ts": lit(stem, "1 / 10000000"),
    }
    header = (
        head(stem, "Runtime float values print with JS `String(number)` semantics -- "
                   "including `Infinity`, `-Infinity`, `NaN`, negative zero and "
                   "exponent notation.")
        + [para(stem, "Runtime float values print through console"), ""]
        + ["FILE-WIDE PROSE (rule 12). The paragraph above is the source's `//!` module "
           "doc: it describes the whole file rather than any one helper, so it lives in "
           "this header. Per-test prose lives in the rationale of the case it belongs "
           "to, per U6.",
           ""]
        + arithmetic("Each `#[test]` fn runs a different program.", 5, 5)
        + [u5("the five programs, which the source all wrote to `app.ts`,"), ""]
        + [U2_INERT, ""]
        + ["ASSERTION SHAPE, EVERY CASE. Every source assertion is an exact "
           "`assert_eq!` on the whole stdout, so every case pins `stdout` exactly "
           "(ruling 3, clause 1). Each value was captured from the real binary, never "
           "computed (U9).",
           ""]
        + extra_ok([(v, "U5 variant-suffixed `[source]` key surfaced as an argv token; it is a "
                     "fixture FILENAME, not a claim about behaviour") for v in keys])
        + [""]
        + gates([CC_RED], stem, "misc", "float_console_runtime"))
    cases = [
        dict(name="run_prints_runtime_float_division_results",
             rationale=R(stem, "`7 / 2` is 3.5 and `6 / 2` is 3, printed as JS prints them -- "
                               "no trailing `.0` on the integral result."),
             steps=[dict(args=["run", "float_division_results.ts"], exit="success",
                         stdout=lit(stem, "3.5\n3\n3.5\n3.5\n", exact=True))]),
        dict(name="run_prints_js_special_float_values",
             rationale=R(stem, "Division by zero prints `Infinity` / `-Infinity`, `0 / 0` prints "
                               "`NaN`, and negative zero prints `0`."),
             steps=[dict(args=["run", "js_special_float_values.ts"], exit="success",
                         stdout=lit(stem, "Infinity\n-Infinity\nNaN\n0\n", exact=True))]),
        dict(name="run_concatenates_runtime_floats_into_strings",
             rationale=R(stem, "A runtime float concatenated into a string uses the same "
                               "`String(number)` spelling."),
             steps=[dict(args=["run", "concatenates_runtime_floats.ts"], exit="success",
                         stdout=lit(stem, "v: 3.5\n", exact=True))]),
        dict(name="run_prints_floats_read_from_mutable_locals_and_params",
             rationale=R(stem, "The same spelling holds for a float read back out of a mutable "
                               "local and for one passed as a function parameter."),
             steps=[dict(args=["run", "floats_from_mutable_locals_and_params.ts"],
                         exit="success", stdout=lit(stem, "3.5\nv: 3.5\n1.5\n4.5\np: 4.5\n", exact=True))]),
        dict(name="run_prints_small_magnitudes_with_js_exponent_notation",
             rationale=R(stem, para(stem, "Was the recorded reachable divergence")),
             steps=[dict(args=["run", "small_magnitudes_exponent_notation.ts"], exit="success",
                         stdout=lit(stem, "1e-7\n", exact=True))]),
    ]
    add("misc", "float_console_runtime", stem, header, keys, cases)


# ---- 10/11. number predicates ---------------------------------------------
NP_STDOUT = ("1\n1\n1\n0\n0\n0\n1\n0\n0\n1\n1\n1\n0\n1\n1\n1\n1\n1\n1\n0\n1\n1\n0\n1\n1\n1\n1\n"
             "0\n1\n1\n1\n1\n0\n1\n1\n1\n1\n0\n1\n1\n1\n1\n1")

NP_RULE13 = (
    RULE13_CHAIN + "\n"
    "\n"
    "ATTRIBUTION IS STILL PER HELPER (U6): the run-source CHAIN's docs go into "
    "the two `run` cases and the test-source CHAIN's into the two `test` cases, "
    "neither into the other pair. The two chains share a documented helper, "
    "which is why the same sentence appears on both sides -- that is the shared "
    "helper being carried twice, not one chain over-attributed.")


def f_number_predicates():
    stem = "number_predicates_runtime"
    run_src = check_captured("CAP_NUMBER_PREDICATES__RUN", stem,
                             ["kali_common::number_predicates_runtime_source()"])
    test_src = check_captured("CAP_NUMBER_PREDICATES__TEST", stem,
                              ["kali_common::number_predicates_test_source()"])
    d_run = carry_docs(CK, ["number_predicates_runtime_source"],
                       lead="Carried per rule 13 from every documented helper in this "
                            "fixture's kali_common chain, derived by walking it:")
    d_test = carry_docs(CK, ["number_predicates_test_source"],
                        lead="Carried per rule 13 from every documented helper in this "
                             "fixture's kali_common chain, derived by walking it:")
    header = (
        head(stem, "The supported `Number` predicate slice runs and tests identically in "
                   "text and `--output json` mode.")
        + arithmetic("Two helpers x {plain, json} == four `#[test]` fns, each a distinct "
                     "command line. A `[matrix]` over the output mode was DECLINED: "
                     "`--output json` changes the ASSERTION SHAPE, not just an argv token.",
                     4, 4)
        + ["`[source]` KEYS ARE THE SOURCE'S OWN. `main.js` and `smoke.test.js` are the "
           "filenames the source writes and they do not collide, so no U5 rename was "
           "needed.",
           ""]
        + [U2_INERT, ""]
        + [NP_RULE13, ""]
        + ["ASSERTION SHAPES. The text `run` branch asserts "
           "`assert_eq!(stdout.trim(), <exact>)`, an exact assertion modulo trailing "
           "whitespace, so it becomes an exact `stdout` pin. The json `run` branch's "
           "`assert_eq!(json[\"stdout\"], <exact>)` is likewise exact and becomes an "
           "exact `json.stdout` pin. The text `test` branch's two `.contains` claims "
           "stay `stdout_contains` (ruling 3, clause 3). The json `test` branch pins "
           "the payload counters the source pins and no others.",
           ""]
        + ["`json.errors = []`. The source asserts "
           "`json[\"errors\"].as_array().is_empty()`; an empty TOML array compares "
           "equal to an empty JSON array element-for-element "
           "(the case runner's jsonpath value comparison), so this is the "
           "same claim, not a strengthening.",
           ""]
        + gates([CC_EMPTY, CF_VACUOUS], stem, "misc", "number_predicates_runtime"))
    cases = [
        dict(name="run_supports_number_predicates_in_js_input",
             rationale=R(stem, "Text-mode `run` over the predicate slice. " + d_run),
             steps=[dict(args=["run", "main.js"], exit="success", stdout=NP_STDOUT + "\n")]),
        dict(name="json_run_supports_number_predicates_in_js_input",
             rationale=R(stem, "The same `run`, asserting the JSON envelope. " + d_run),
             steps=[dict(args=["--output", "json", "run", "main.js"], exit="success",
                         json_paths={"schemaVersion": 1, "command": "run", "success": True,
                                     "stdout": NP_STDOUT + "\n", "errors": []})]),
        dict(name="test_supports_number_predicates_in_js_input",
             rationale=R(stem, "Text-mode `test` over the predicate slice; the two source "
                               "claims are `.contains` against stdout and stay "
                               "`stdout_contains`. " + d_test),
             steps=[dict(args=["test", "smoke.test.js"], exit="success",
                         stdout_contains=[NP_STDOUT, "ok 1"])]),
        dict(name="json_test_supports_number_predicates_in_js_input",
             rationale=R(stem, "The same `test`, asserting the JSON envelope and the payload "
                               "counters. " + d_test),
             steps=[dict(args=["--output", "json", "test", "smoke.test.js"], exit="success",
                         json_paths={"schemaVersion": 1, "command": "test", "success": True,
                                     "payload.passed": 1, "payload.failed": 0, "errors": []})]),
    ]
    add("misc", "number_predicates_runtime", stem, header,
        {"main.js": run_src, "smoke.test.js": test_src}, cases)


def f_number_predicates_freeze():
    stem = "number_predicates_freeze_runtime"
    run_src = check_captured("CAP_NUMBER_PREDICATES_FREEZE__RUN", stem,
                             ["kali_common::number_predicates_runtime_source()"])
    test_src = check_captured("CAP_NUMBER_PREDICATES_FREEZE__TEST", stem,
                              ["kali_common::number_predicates_test_source()"])
    expected = check_captured("CAP_NUMBER_PREDICATES_FREEZE__EXPECTED_STDOUT", stem,
                              ["fn freeze_number_predicates_expected_stdout"])
    if expected != NP_STDOUT:
        raise AssertionError("the freeze target's expected stdout no longer matches")
    d_run = carry_docs(CK, ["number_predicates_runtime_source"],
                       lead="Carried per rule 13 from every documented helper in this "
                            "fixture's kali_common chain, derived by walking it:")
    d_test = carry_docs(CK, ["number_predicates_test_source"],
                        lead="Carried per rule 13 from every documented helper in this "
                             "fixture's kali_common chain, derived by walking it:")
    header = (
        head(stem, "The frozen-alias spelling of the `Number` predicate slice: the same "
                   "two programs, run and tested in text and `--output json` mode.")
        + arithmetic("Four `#[test]` fns, each a distinct command line. A `[matrix]` over "
                     "the output mode was DECLINED: `--output json` changes the ASSERTION "
                     "SHAPE, not just an argv token.", 4, 4)
        + ["`[source]` KEYS ARE THE SOURCE'S OWN. `main.js` and `smoke.test.js` are the "
           "filenames the source writes and they do not collide, so no U5 rename was "
           "needed.",
           ""]
        + [U2_INERT, ""]
        + [NP_RULE13, ""]
        + ["ASSERTION SHAPES. The two `test` fns assert ONLY `output.status.success()`; "
           "no stdout claim is written for them, because the source makes none "
           "(rule 2) -- the json one additionally pins the payload counters, which it "
           "does assert. The json `run` fn's `assert_eq!(json[\"stdout\"], "
           "format!(\"{}\\n\", expected))` is exact and becomes an exact `json.stdout` "
           "pin. Its newline-less half is the byte-exact output of EXECUTING the real "
           "`kali_common` helper, which the generator asserts; the trailing newline "
           "the `format!` adds is appended by the generator, and the trial pins the "
           "assembled string against the real binary on every run.",
           ""]
        + extra_ok([(NP_STDOUT + "\n", "a deliberate live-captured exact pin. The source "
                     "asserts `json[\"stdout\"] == format!(\"{}\\n\", expected)`; the trailing "
                     "newline is added by the `format!` at runtime, so the assembled string "
                     "exists as no literal in the `.rs` -- only the newline-less constant does. "
                     "Provenance stated exactly rather than waved at: the newline-less half "
                     "is the byte-exact output of executing the real helper (asserted by the "
                     "generator), and the one appended newline is the only hand-applied "
                     "character -- live-verified, because the trial pins this exact string "
                     "against the real binary")])
        + [""]
        + gates([CC_EMPTY, CF_VACUOUS], stem, "misc", "number_predicates_freeze_runtime"))
    cases = [
        dict(name="run_supports_frozen_number_predicates_in_js_input",
             rationale=R(stem, "Text-mode `run`; the source's `assert_eq!(stdout.trim(), ...)` "
                               "is exact modulo trailing whitespace, so this pins the exact "
                               "stdout, live-captured. " + d_run),
             steps=[dict(args=["run", "main.js"], exit="success", stdout=NP_STDOUT + "\n")]),
        dict(name="json_run_supports_frozen_number_predicates_in_js_input",
             rationale=R(stem, "The same `run` in JSON mode, pinning the envelope and the exact "
                               "embedded `stdout`. " + d_run),
             steps=[dict(args=["--output", "json", "run", "main.js"], exit="success",
                         json_paths={"schemaVersion": 1, "command": "run", "success": True,
                                     "exitCode": 0, "stdout": NP_STDOUT + "\n"})]),
        dict(name="test_supports_frozen_number_predicates_in_js_input",
             rationale=R(stem, "Text-mode `test`. The source asserts ONLY that the command "
                               "succeeded, so this case claims only that -- pinning the stdout "
                               "it was observed to emit would be a rule-2 invention. " + d_test),
             steps=[dict(args=["test", "smoke.test.js"], exit="success")]),
        dict(name="json_test_supports_frozen_number_predicates_in_js_input",
             rationale=R(stem, "The same `test` in JSON mode; the source pins the envelope and "
                               "the three payload counters. " + d_test),
             steps=[dict(args=["--output", "json", "test", "smoke.test.js"], exit="success",
                         json_paths={"schemaVersion": 1, "command": "test", "success": True,
                                     "payload.total": 1, "payload.passed": 1,
                                     "payload.failed": 0})]),
    ]
    add("misc", "number_predicates_freeze_runtime", stem, header,
        {"main.js": run_src, "smoke.test.js": test_src}, cases)


# ---- 12/13. promise sequencing --------------------------------------------
def f_promise(stem, toml, what, cap_run, cap_test, e4000):
    run_src = check_captured(cap_run, stem, [f"Promise.{what}([", "async function main()"])
    test_src = check_captured(cap_test, stem, [f"Kali.test('promise {what}'"])
    header = (
        head(stem, f"`Promise.{what}` sequencing fails closed and loud: the fixture's own "
                   "self-check throws and the run exits nonzero.")
        + arithmetic("Four `#[test]` fns == two helper shapes (`run`, `test`) x two input "
                     "extensions, and every case varies over the extension uniformly, so "
                     "`ext` is a real axis (rule 7, U1). The `[source]` keys carry the axis "
                     "too, so a `js` trial materialises only the `js` fixture.",
                     4, 2, {"ext": ["js", "ts"]})
        + ["U13 -- WHAT THE MATRIX BOUGHT. The source writes the same program text under "
           "`main.js` and `main.ts` (and the same test program under `smoke.test.js` and "
           "`smoke.test.ts`). Written as four named siblings that would be a duplicated "
           "`[source]` body and a U13 hoist question; declaring `ext` removes the "
           "duplication instead of hoisting around it, which is strictly better than "
           "either horn of ruling 7's dilemma.",
           ""]
        + [U2_INERT, ""]
        + (["RULE 11 -- AN OR-SHAPED ASSERTION, RESOLVED BY OBSERVATION. The source "
            "accepts `stderr.contains(\"E4000\") || stdout.contains(\"E4000\")`. The "
            "format has no disjunction, so the OR was resolved against the real binary: "
            "on every one of these four trials the code lands on STDERR and stdout does "
            "not carry it, so exactly one disjunct is true and there is no ruling-17 tie "
            "to break. The source's disjunction sentence is carried into every affected "
            "rationale, so the narrowing is recorded rather than silent.",
           ""] if e4000 else
           ["ASSERTION SHAPE. The helper asserts ONLY `!output.status.success()`. No "
            "stdout or stderr claim is written, because the source makes none -- the "
            "trap text these trials do emit is true and unasserted, and pinning it "
            "would be a rule-2 invention.",
            ""])
        + gates([], stem, "misc", toml))
    note = para(stem, "Honest re-pin (PR #16 rev2)")
    cases = []
    for kind, key, cmd in (("run", "main.${ext}", "run"), ("test", "smoke.test.${ext}", "test")):
        step = dict(args=[cmd, key], exit="failure")
        if e4000:
            step["stderr_contains"] = ["E4000"]
        cases.append(dict(
            name=f"{kind}_supports_promise_{what}_in_js_and_ts_input",
            rationale=R(stem, note + "\n\n"
                        + f"Source `#[test]` fns: {kind}_supports_promise_{what}_in_js_input, "
                        + f"{kind}_supports_promise_{what}_in_ts_input -- folded into one "
                        + "`[[case]]` by the file-wide `ext` axis under rule 7, named here so "
                        + "the fn names survive the sources' deletion (rule 6's reason, not "
                        + "just its letter). "
                        + f"`kali {cmd}` over the `Promise.{what}` fixture must exit nonzero. "
                        + ("Rule 11: the source accepts E4000 on EITHER stream; resolved "
                           "against the real binary, this stream is stderr, and that branch is "
                           "what is pinned. "
                           if e4000 else
                           "The source asserts nothing about either stream, so nothing about "
                           "either stream is pinned (rule 2). ")
                        + "This case is fanned over `ext` by the file-wide `[matrix]`, "
                          "reproducing the source's `js` and `ts` invocations 1:1."),
            steps=[step]))
    add("misc", toml, stem, header,
        {"main.${ext}": run_src, "smoke.test.${ext}": test_src}, cases,
        matrix={"ext": ["js", "ts"]})


# ---- 14. runtime_monomorphize ---------------------------------------------
def f_monomorphize():
    stem = "runtime_monomorphize"
    keys = {
        "dump_two_distinct_shapes.ts": lit(stem, "console.log(dump(A)); console.log(dump(B));"),
        "transitive_outer_inner_two_shapes.ts": lit(stem, "function inner(t)"),
        "nested_fn_decl_caller.ts": lit(stem, "function helper()"),
        "ambiguous_conditional_merge.ts": lit(stem, "var o = cond ? A : B;"),
    }
    header = (
        head(stem, "Object-shape monomorphization: a function reached by two distinct "
                   "object-param shapes is specialized per shape and compiles, while a "
                   "genuinely ambiguous merge still fails closed.")
        + [para(stem, "End-to-end acceptance for object-shape monomorphization"), ""]
        + ["FILE-WIDE PROSE (rule 12). The paragraph above is the source's `//!` module "
           "doc: it describes the whole file rather than any one helper, so it lives in "
           "this header. Per-test prose lives in the rationale of the case it belongs "
           "to, per U6.",
           ""]
        + ["THIS TARGET WAS NOT A RETENTION, AND THE TREE COULD NOT SAY SO. Before this "
           "migration `runtime_monomorphize.rs` was the one non-browser source that "
           "`citation_sweep.sh`'s whole-file-retention arm adopted as a RETENTION: it "
           "carries an ordinary `//!` module doc and had no case file, and nothing in "
           "the tree distinguishes a module doc from a U3 retention header. It "
           "contributed 0 problems and passed vacuously. Migrating it removes the one "
           "instance; the general hole is closed separately by "
           "`tools/migration/screen_candidates.py --retention-crosscheck`, which fails "
           "when the sweep's retention population contains a target the screen calls "
           "migratable.",
           ""]
        + arithmetic("Each `#[test]` fn runs a different program.", 4, 4)
        + [u5("the four programs, which the source all wrote to `main.ts`,"), ""]
        + [U2_INERT, ""]
        + extra_ok([(v, "U5 variant-suffixed `[source]` key surfaced as an argv token; it is a "
                     "fixture FILENAME, not a claim about behaviour") for v in keys])
        + [""]
        + gates([CC_RED, U8_MARKER], stem, "runtime", "monomorphize"))
    cases = [
        dict(name="dump_two_distinct_shapes_prints_three_then_two",
             rationale=R(stem, para(stem, "The design-doc repro")
                         + "\n\nExact `assert_eq!` on the whole stdout, so an exact `stdout` "
                           "pin (ruling 3, clause 1)."),
             steps=[dict(args=["run", "dump_two_distinct_shapes.ts"], exit="success",
                         stdout=lit(stem, "3\n2\n", want=2, exact=True))]),
        dict(name="transitive_outer_inner_two_shapes_prints_three_then_two",
             rationale=R(stem, para(stem, "Probe P4")
                         + "\n\nExact `assert_eq!` on the whole stdout, so an exact `stdout` "
                           "pin."),
             steps=[dict(args=["run", "transitive_outer_inner_two_shapes.ts"], exit="success",
                         stdout=lit(stem, "3\n2\n", want=2, index=1, exact=True))]),
        dict(name="nested_fn_decl_caller_still_rejects_cleanly",
             rationale=R(stem, para(stem, "Task 7a-2 follow-up (fail-closed guard)")
                         + "\n\nThe source asserts `!status.success()` and "
                           "`stderr.contains(\"E5506\")`; the `.contains` stays "
                           "`stderr_contains` (ruling 3, clause 3)."),
             steps=[dict(args=["run", "nested_fn_decl_caller.ts"], exit="failure",
                         stderr_contains=["E5506"])]),
        dict(name="ambiguous_conditional_merge_still_rejects",
             rationale=R(stem, para(stem, "Fail-closed pin (design")
                         + "\n\nThe source asserts ONLY `!status.success()`. The trial does emit "
                           "an E5506 diagnostic, but this fn never asserts it -- unlike its "
                           "sibling above -- so pinning it would be a rule-2 invention and "
                           "nothing about either stream is claimed here."),
             steps=[dict(args=["run", "ambiguous_conditional_merge.ts"], exit="failure")]),
    ]
    add("runtime", "monomorphize", stem, header, keys, cases)


# ---- 15. object_has_own_frozen_js_input -----------------------------------
def f_object_has_own():
    stem = "object_has_own_frozen_js_input"
    run_src = check_captured("CAP_OBJECT_HAS_OWN__RUN", stem,
                             ["fn frozen_object_has_own_source",
                              "object_has_own_frozen_callable_condition_source(\"wrapped\"",
                              "object_has_own_property_call_frozen_callable_source()"])
    test_src = check_captured("CAP_OBJECT_HAS_OWN__TEST", stem,
                              ["fn frozen_object_has_own_test_source",
                               "Kali.test('frozen object hasOwn'"])
    carried = carry_docs(CK, ["object_has_own_frozen_callable_source",
                              "object_has_own_property_call_frozen_callable_source",
                              "object_has_own_frozen_callable_condition_source",
                              "object_has_own_property_call_frozen_callable_condition_source"],
                         lead="Carried per rule 13 from every documented helper in this "
                              "fixture's kali_common chain, derived by walking it:")
    header = (
        head(stem, "The frozen `Object.hasOwn` alias surface: `check` accepts it in all "
                   "four input extensions, while `run` and `test` fail closed and loud "
                   "because the fixture's own self-check throws.")
        + arithmetic("FOUR of the source's five `#[test]` fns are migrated here; the fifth "
                     "is a U4 trim-and-keep retention (see the paragraph below). Each of the "
                     "four loops the same four input extensions, so 4 fns x 4 extensions == "
                     "16 invocations and every case varies over the extension uniformly "
                     "(rule 7, U1). The `[source]` keys carry the axis, so a `js` trial "
                     "materialises only `main.js` and `main.test.js`.",
                     16, 4, {"ext": ["js", "ts", "jsx", "tsx"]})
        + [U4_TRIM, ""]
        + ["FIXTURE PROVENANCE (rules 8/9). Both programs are built by a `format!` whose "
           "placeholders are filled from FOUR `kali_common::` helpers. Neither text "
           "exists as a literal anywhere, so both were captured by EXECUTING the real "
           "code (`tools/migration/t19b2_captures.py` records the command); nothing was "
           "hand-substituted.",
           ""]
        + [RULE13_CHAIN, ""]
        + [U2_INERT, ""]
        + ["RULE 11 -- AN OR-SHAPED ASSERTION WHOSE STREAM DEPENDS ON THE OUTPUT MODE. "
           "The fail-closed helper accepts `stderr.contains(\"E4000\") || "
           "stdout.contains(\"E4000\")`. Resolved against the real binary, the answer "
           "is NOT the same for every case, and that is the finding: in text mode the "
           "code lands on STDERR and stdout carries none of it; under `--output json` "
           "the process writes NOTHING to stderr and the code appears inside the JSON "
           "envelope on STDOUT. Exactly one disjunct is true per case -- no ruling-17 "
           "tie -- and each case pins the stream its own trial actually uses, verified "
           "uniform across all four extensions. The source's disjunction sentence is "
           "carried into every affected rationale.",
           ""]
        + gates([], stem, "misc", "object_has_own_frozen_js_input"))
    note = para(stem, "Batch-local variant (PR #16 rev2, batch 7)")
    repin = para(stem, "Honest re-pin (PR #16 rev2)")
    rule13 = carried
    cases = [
        dict(name="run_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input",
             rationale=R(stem, note + "\n\n" + repin
                         + "\n\nText-mode `run`. Rule 11: the source accepts E4000 on EITHER "
                           "stream; resolved against the real binary, text mode puts it on "
                           "stderr. " + rule13),
             steps=[dict(args=["run", "main.${ext}"], exit="failure",
                         stderr_contains=["E4000"])]),
        dict(name="json_run_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input",
             rationale=R(stem, note + "\n\n" + repin
                         + "\n\nJSON-mode `run`. Rule 11: the source accepts E4000 on EITHER "
                           "stream; resolved against the real binary, JSON mode leaves stderr "
                           "EMPTY and carries the code inside the envelope on stdout, so this "
                           "case pins the stdout branch. " + rule13),
             steps=[dict(args=["--output", "json", "run", "main.${ext}"], exit="failure",
                         stdout_contains=["E4000"])]),
        dict(name="test_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input",
             rationale=R(stem, note + "\n\n" + repin
                         + "\n\nText-mode `test`. Rule 11: resolved against the real binary, "
                           "text mode puts the code on stderr. " + rule13),
             steps=[dict(args=["test", "main.test.${ext}"], exit="failure",
                         stderr_contains=["E4000"])]),
        dict(name="json_test_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input",
             rationale=R(stem, note + "\n\n" + repin
                         + "\n\nJSON-mode `test`. Rule 11: resolved against the real binary, "
                           "JSON mode leaves stderr empty and carries the code inside the "
                           "envelope's own `stderr` field on stdout. " + rule13),
             steps=[dict(args=["--output", "json", "test", "main.test.${ext}"], exit="failure",
                         stdout_contains=["E4000"])]),
    ]
    add("misc", "object_has_own_frozen_js_input", stem, header,
        {"main.${ext}": run_src, "main.test.${ext}": test_src}, cases,
        matrix={"ext": ["js", "ts", "jsx", "tsx"]})


# ---- 16. parse_int_static_ascii -------------------------------------------
def f_parse_int():
    stem = "parse_int_static_ascii"
    supported = check_captured("CAP_PARSE_INT__SUPPORTED", stem,
                               ["fn supported_source", "frozenNumberParseInt('10'"])
    keys = {
        "main.js": supported,
        "main.ts": supported,
        "parse_int_dynamic_input.js": lit(stem, "function parse(value)"),
        "parse_int_invalid_radix.js": lit(stem, "'10', 1"),
        "parse_int_nan_result.js": lit(stem, "'nope'"),
    }
    header = (
        head(stem, "`parseInt` over a statically-known ASCII string runs and checks "
                   "clean; a dynamic input, an invalid radix and a NaN result are each "
                   "gated at `check` time with E5506.")
        + arithmetic("Five `#[test]` fns, each a distinct command line.", 5, 5)
        + [u5("the three gated programs, which the source all wrote to `main.js`,"), ""]
        + ["U13 -- A DUPLICATED FIXTURE BODY, AND WHY IT IS NOT HOISTED. `main.js` and "
           "`main.ts` carry BYTE-IDENTICAL text: the source calls `supported_source()` "
           "for both. A `[matrix]` over the extension cannot fold them, because the two "
           "cases run DIFFERENT commands (`run` versus `--output json check`) and make "
           "different claims, so the axis would not fan uniformly (U1). U13's hoist is "
           "declined for ruling 7's reason -- the shared text is an inline literal, and "
           "`check_fixtures.py` searches only `[source]` values and step `body`, so a "
           "hoisted body would turn that gate red on a correct file. U13's other half "
           "is honoured: the identity is asserted MECHANICALLY in "
           "`tools/migration/gen_task19_batch2.py` (one call site, used twice) rather "
           "than eyeballed.",
           ""]
        + [U2_INERT, ""]
        + ["ASSERTION SHAPES. The `run` case's `assert_eq!` on the whole stdout is exact "
           "and becomes an exact `stdout` pin. The four `--output json check` cases pin "
           "the envelope fields the source pins; `json.errors = []` is the same claim as "
           "`errors.as_array().is_empty()`, and `json.errors.0.code` is what the three "
           "gated cases assert.",
           ""]
        + extra_ok([(v, "U5 variant-suffixed `[source]` key surfaced as an argv token; it is a "
                     "fixture FILENAME, not a claim about behaviour")
                    for v in ("parse_int_dynamic_input.js", "parse_int_invalid_radix.js",
                              "parse_int_nan_result.js")])
        + [""]
        + gates([CC_EMPTY], stem, "misc", "parse_int_static_ascii"))
    gated = ("The source asserts `!status.success()`, `json.success == false` and "
             "`json.errors[0].code == \"E5506\"`, and nothing else -- notably no message "
             "claim, so none is written (rule 2).")
    cases = [
        dict(name="run_supports_static_ascii_parse_int",
             rationale=R(stem, "Every supported `parseInt` spelling -- bare, `globalThis.`, "
                               "`Number.parseInt`, fully bracketed, and frozen callables -- "
                               "evaluates at compile time. Exact `assert_eq!` on the whole "
                               "stdout, so an exact `stdout` pin (ruling 3, clause 1). The "
                               "source carries no comments, so rules 12 and 13 have nothing to "
                               "carry into this rationale."),
             steps=[dict(args=["run", "main.js"], exit="success",
                         stdout=lit(stem, "42\n-16\n255\n5\n63\n2\n", exact=True))]),
        dict(name="json_check_accepts_static_ascii_parse_int",
             rationale=R(stem, "The same program under `--output json check` in a `.ts` file: "
                               "the envelope reports success and an empty `errors` array."),
             steps=[dict(args=["--output", "json", "check", "main.ts"], exit="success",
                         json_paths={"schemaVersion": 1, "command": "check", "success": True,
                                     "errors": []})]),
        dict(name="check_gates_parse_int_dynamic_input",
             rationale=R(stem, "A `parseInt` whose input is a function parameter is not "
                               "statically known, so `check` gates it. " + gated),
             steps=[dict(args=["--output", "json", "check", "parse_int_dynamic_input.js"],
                         exit="failure",
                         json_paths={"success": False, "errors.0.code": "E5506"})]),
        dict(name="check_gates_parse_int_invalid_radix",
             rationale=R(stem, "Radix 1 is outside the supported range, so `check` gates it. "
                         + gated),
             steps=[dict(args=["--output", "json", "check", "parse_int_invalid_radix.js"],
                         exit="failure",
                         json_paths={"success": False, "errors.0.code": "E5506"})]),
        dict(name="check_gates_parse_int_nan_result",
             rationale=R(stem, "An input that would yield NaN has no integer result to fold, "
                               "so `check` gates it. " + gated),
             steps=[dict(args=["--output", "json", "check", "parse_int_nan_result.js"],
                         exit="failure",
                         json_paths={"success": False, "errors.0.code": "E5506"})]),
    ]
    add("misc", "parse_int_static_ascii", stem, header, keys, cases)


# ---- 17. reflect_own_keys_js_input ----------------------------------------
def f_reflect_own_keys():
    stem = "reflect_own_keys_js_input"
    run_src = check_captured("CAP_REFLECT_OWN_KEYS__RUN", stem,
                             ["fn reflect_own_keys_source",
                              "reflect_own_keys_frozen_callable_source(\"obj\")"])
    test_src = check_captured("CAP_REFLECT_OWN_KEYS__TEST", stem,
                              ["fn reflect_own_keys_test_source",
                               "Kali.test('reflect ownKeys'"])
    d = carry_docs(CK, ["reflect_own_keys_frozen_callable_source"],
                   lead="Carried per rule 13 from every documented helper in this fixture's "
                        "kali_common chain, derived by walking it:")
    header = (
        head(stem, "`Reflect.ownKeys` ordering holds across every supported spelling of "
                   "the callable -- dotted, bracketed, single-quoted, parenthesized and "
                   "frozen -- under `check`, `run` and `test`.")
        + arithmetic("Each `#[test]` fn is a distinct command line over one of the two "
                     "programs.", 5, 5)
        + ["`[source]` KEYS ARE THE SOURCE'S OWN. `main.js` and `main.test.js` are the "
           "filenames the source writes and they do not collide, so no U5 rename was "
           "needed.",
           ""]
        + ["FIXTURE PROVENANCE (rules 8/9). The `run` program is built by a `format!` "
           "whose placeholder is filled from kali_common's "
           "reflect_own_keys_frozen_callable_source, so its text was captured by "
           "EXECUTING the real code rather than hand-substituted. The `test` program is "
           "a plain literal and was pulled from the `.rs` unchanged.",
           ""]
        + ["RULE 13 -- ATTRIBUTION IS PER HELPER, NOT PER FILE (U6). The kali_common doc "
           "below is carried into the rationale of exactly the THREE cases whose fixture "
           "reaches that helper -- the ones that run `main.js`. The two `main.test.js` "
           "cases use a plain-literal fixture that never calls it, so the doc is "
           "deliberately absent from them; copying it into all five would be the "
           "over-attribution U6 forbids.",
           ""]
        + [U2_INERT, ""]
        + ["ASSERTION SHAPES. Every fn asserts `assert_eq!(status.code(), Some(0))`, "
           "which is an exact exit-code claim and becomes `exit = 0` rather than "
           "`exit = \"success\"`. `stdout.contains(\"reflect ownKeys ok\")` stays "
           "`stdout_contains`; the same `.contains` taken against the JSON string leaf "
           "`json[\"stdout\"]` becomes `json_count` with `at_least = 1`, per ruling 3's "
           "amended clause 4. The two exact `assert_eq!(json[\"stdout\"], \"\")` / "
           "`(json[\"stderr\"], \"\")` claims are exact and are pinned exactly.",
           ""]
        + gates([CC_EMPTY], stem, "misc", "reflect_own_keys_js_input"))
    rule13 = d
    cases = [
        dict(name="check_accepts_reflect_own_keys_in_js_input",
             rationale=R(stem, "`kali check` accepts every supported `Reflect.ownKeys` "
                               "spelling. The source asserts success and an exact exit code "
                               "of 0, and nothing about either stream. " + rule13),
             steps=[dict(args=["check", "main.js"], exit=0)]),
        dict(name="run_accepts_reflect_own_keys_in_js_input",
             rationale=R(stem, "Running the program, its own ordering self-check passes and it "
                               "prints its marker. " + rule13),
             steps=[dict(args=["run", "main.js"], exit=0,
                         stdout_contains=["reflect ownKeys ok"])]),
        dict(name="json_run_accepts_reflect_own_keys_in_js_input",
             rationale=R(stem, "The same `run` under `--output json`; the marker claim is a "
                               "plain `.contains` against the JSON string leaf `stdout`, so it "
                               "is `json_count` with `at_least = 1` rather than an exact pin "
                               "(ruling 3, amended clause 4). " + rule13),
             steps=[dict(args=["--output", "json", "run", "main.js"], exit=0,
                         json_paths={"schemaVersion": 1, "command": "run", "success": True,
                                     "exitCode": 0},
                         json_count=[dict(path="stdout", needle="reflect ownKeys ok",
                                          at_least=1)])]),
        dict(name="test_accepts_reflect_own_keys_in_js_input",
             rationale=R(stem, "The `Kali.test` form of the same ordering check passes. The "
                               "source asserts success and an exact exit code of 0, and "
                               "nothing about either stream, so nothing else is pinned "
                               "(rule 2)."),
             steps=[dict(args=["test", "main.test.js"], exit=0)]),
        dict(name="json_test_accepts_reflect_own_keys_in_js_input",
             rationale=R(stem, "The same `test` under `--output json`. The source pins the "
                               "envelope, all four payload counters, and that the embedded "
                               "`stdout` and `stderr` are EXACTLY empty strings -- exact "
                               "assertions, pinned exactly."),
             steps=[dict(args=["--output", "json", "test", "main.test.js"], exit=0,
                         json_paths={"schemaVersion": 1, "command": "test", "success": True,
                                     "exitCode": 0, "payload.total": 1, "payload.passed": 1,
                                     "payload.failed": 0, "payload.skipped": 0,
                                     "stdout": "", "stderr": ""})]),
    ]
    add("misc", "reflect_own_keys_js_input", stem, header,
        {"main.js": run_src, "main.test.js": test_src}, cases)


def build():
    FILES.clear()
    f_exponentiation_operator()
    f_fasta_capstone()
    f_fasta_output()
    f_standalone_iter()
    # WITHDRAWN ON MEASUREMENT, NOT SKIPPED -- see the report's §4/§16.
    # `f_object_has_own()` is a U4 TRIM-AND-KEEP pair (controller ruling, batch 2
    # fix round 1): the four migrated fns are emitted by `f_object_has_own()`
    # below; the fifth stays hand-written because every assertion its helper
    # makes beyond `status.success()` is unreachable, which is R1's
    # unreachable-code shape.
    f_module_var_object_compound()
    f_closure_return_isolation()
    f_heap_grow()
    f_trap_diagnostics()
    f_float_console()
    f_number_predicates()
    f_number_predicates_freeze()
    f_promise("promise_any_sequencing", "promise_any_sequencing", "any",
              "CAP_PROMISE_ANY__RUN", "CAP_PROMISE_ANY__TEST", True)
    f_promise("promise_race_sequencing", "promise_race_sequencing", "race",
              "CAP_PROMISE_RACE__RUN", "CAP_PROMISE_RACE__TEST", False)
    f_monomorphize()
    f_object_has_own()
    f_parse_int()
    f_reflect_own_keys()
    return FILES


def _with_constants(text, constants):
    """Insert a `[constants]` block ahead of `[matrix]`/`[source]`.

    `case_emit.emit` has no `[constants]` parameter -- the sixteen browser
    generators never needed one, because ruling 7 DECLINED the hoist for that
    family. Rendering the block here rather than growing `emit` keeps this
    batch's need out of the emitter that writes 161 irreplaceable browser case
    files: nothing about their output can move because of a key they do not use.
    Placement matches `misc/bitwise_operators_runtime.toml`, the one shipped file
    that already carries the hoist.
    """
    if not constants:
        return text
    from toml_emit import toml_string
    block = ["[constants]"]
    for name, value in constants.items():
        block.append(f"{name} = {toml_string(value)}")
    block.append("")
    lines = text.split("\n")
    at = next((i for i, l in enumerate(lines)
               if l.startswith("[matrix]") or l.startswith("[source]")
               or l.startswith("[[case]]")), len(lines))
    return "\n".join(lines[:at] + block + lines[at:])


def _wrap(entries, width=88):
    """Hard-wrap header prose so a `#` block is readable at review width.

    Two line shapes are passed through UNWRAPPED, and both matter:
      * a line starting with `EXTRA-OK:` -- `check_extra_claims.py` parses one
        declaration PER LINE (`#\\s*EXTRA-OK:\\s*(.+?)\\s+--\\s+(.*)$`), so a
        wrapped declaration is a declaration it cannot read;
      * an already-indented line -- reproduction commands and bullet items carry
        their own layout, and re-flowing a shell command breaks it.
    """
    import textwrap
    out = []
    for entry in entries:
        for piece in str(entry).split("\n"):
            if not piece.strip() or piece.startswith((" ", "\t")) or \
                    piece.startswith("EXTRA-OK:") or len(piece) <= width:
                out.append(piece)
            else:
                out.extend(textwrap.wrap(piece, width=width,
                                         break_long_words=False,
                                         break_on_hyphens=False))
    return out


def _render_all():
    out = {}
    for f in build():
        path = os.path.join(CASES, f["family"], f["toml"] + ".toml")
        out[path] = _with_constants(
            emit(_wrap(f["header"]), f["matrix"], f["source"], f["cases"]),
            f["constants"])
    return out


def rendered():
    """Render, measure the comment-coverage classes against that rendering,
    render again, and require the second to be a fixed point."""
    import tempfile
    first = _render_all()
    d = tempfile.mkdtemp(prefix="gen-cc-classes-")
    changed = False
    for path, text in first.items():
        stem = next(f["stem"] for f in build()
                    if os.path.join(CASES, f["family"], f["toml"] + ".toml") == path)
        probe = os.path.join(d, os.path.basename(path))
        with open(probe, "w") as fh:
            fh.write(text)
        found = cc_classes_of(stem, probe)
        if _CC_CLASSES.get(path) != found:
            changed = True
        _CC_CLASSES[path] = found
    if not changed:
        return first
    second = _render_all()
    third = _render_all()
    if second != third:
        raise AssertionError("class measurement is not a fixed point")
    return second


# The gate reds a header is allowed to declare, and the command that decides
# each one. `--check` requires the declaration and the gate to AGREE (ruling 18
# #3: a non-match is an error, never a silent pass), so a paragraph saying a
# gate is expected-red cannot survive the gate going green, and a gate that goes
# red cannot survive being undeclared.
DECLARABLE = {
    "comment_coverage.py": ["tools/task-18-browser-pilot/comment_coverage.py"],
    "check_rationale_fn_names.py": ["tools/task-18-browser-pilot/check_rationale_fn_names.py"],
    "check_fixtures.py": ["tools/task-18-browser-pilot/check_fixtures.py"],
}

# WHICH `.rs` EACH GATE IS RUN AGAINST FOR A U4 TRIM PAIR, and the split is not
# ruling 12's "always the complement" (rulings 9 and 12 together):
#
#   * a gate that compares the case file's CLAIMS with the source's claims
#     (`check_fixtures.py` here; `audit-case-migration.py` and
#     `check_extra_claims.py` outside this table) wants the MIGRATED COMPLEMENT
#     -- ruling 12's own subject. Against the post-trim file the claims are
#     compared with a source stripped of the half that makes them.
#   * a gate whose subject is PROSE (`comment_coverage.py`,
#     `check_rationale_fn_names.py`) wants the PRE-TRIM BLOB, which is ruling 9's
#     original answer and is right for exactly the gates ruling 12 did not
#     narrow. A rationale legitimately names fns on BOTH sides of the trim: the
#     migrated fns it was built from and, in the trim paragraph, the retained one
#     it explains. Neither the post-trim file nor the complement carries both,
#     so each reports names the other explains -- measured, and both are red,
#     with the pre-trim blob green.
_TRIM_SIDE = {"check_fixtures.py": "complement"}
_DECL = None


def _declared_reds(text):
    """`{gate: rc}` for every EXPECTED-RED declaration in a case file's header.

    The header is joined by SPACE and whitespace-collapsed before matching,
    because these paragraphs are hard-wrapped and a marker that spans a line
    break is ruling 18's first worked example of a gate going silently
    one-armed."""
    global _DECL
    if _DECL is None:
        _DECL = re.compile(r"`([a-z_]+\.py)`[^`]{0,24}?IS EXPECTED-RED \(rc=(\d)\)")
    header = " ".join(l.lstrip("#").strip() for l in text.split("\n")
                      if l.startswith("#"))
    header = re.sub(r"\s+", " ", header)
    found = {}
    for gate, rc in _DECL.findall(header):
        if gate in found and found[gate] != int(rc):
            raise AssertionError(f"two different rc declared for {gate}")
        found[gate] = int(rc)
    return found


def _gate_source(stem, gate):
    """The `.rs` a gate should be run against for this pair.

    For an ordinary pair, the working-tree source. For a U4 trim pair, the side
    the gate's DIRECTION calls for -- see `_TRIM_SIDE`.
    """
    pre_trim, complement = _trim_sides(stem)
    if not pre_trim:
        return os.path.join(TESTS, stem + ".rs")
    return complement if _TRIM_SIDE.get(gate) == "complement" else pre_trim


def _trim_sides(stem):
    """`(pre_trim_path, complement_path)` for a trimmed stem, else `(None, None)`.

    The ref is read from the retained file's OWN `PRE-TRIM REF:` line -- the one
    place ruling 11 allows it to live -- and the complement is built by
    `migrated_complement.py` rather than by a second implementation here.
    """
    import subprocess
    import tempfile
    path = os.path.join(TESTS, stem + ".rs")
    text = open(path).read()
    m = re.search(r"PRE-TRIM REF:\s*([0-9a-f]{40})", text)
    if not (text.startswith("//!") and m):
        return None, None
    if stem not in _TRIM_CACHE:
        blob = subprocess.run(
            ["git", "show", f"{m.group(1)}:crates/kali_cli/tests/{stem}.rs"],
            cwd=REPO, capture_output=True, text=True)
        if blob.returncode:
            raise AssertionError(f"{stem}: cannot read PRE-TRIM REF {m.group(1)}")
        d = tempfile.mkdtemp(prefix="gen-trim-")
        pre = os.path.join(d, "pre.rs")
        with open(pre, "w") as fh:
            fh.write(blob.stdout)
        comp = subprocess.run(
            [sys.executable,
             os.path.join(REPO, "tools/task-18-browser-pilot/migrated_complement.py"),
             pre, path], capture_output=True, text=True)
        if comp.returncode:
            raise AssertionError(f"{stem}: migrated_complement.py: {comp.stderr}")
        cpath = os.path.join(d, "complement.rs")
        with open(cpath, "w") as fh:
            fh.write(comp.stdout)
        _TRIM_CACHE[stem] = (pre, cpath)
    return _TRIM_CACHE[stem]


_TRIM_CACHE = {}


def check_gate_declarations(files):
    import subprocess
    problems = []
    for path, text in sorted(files.items()):
        spec = next(f for f in build()
                    if os.path.join(CASES, f["family"], f["toml"] + ".toml") == path)
        declared = _declared_reds(text)
        for gate, cmd in DECLARABLE.items():
            source = _gate_source(spec["stem"], gate)
            rc = subprocess.run(
                [sys.executable, os.path.join(REPO, cmd[0]), source, path],
                cwd=REPO, capture_output=True).returncode
            if gate == "comment_coverage.py" and rc == 1:
                found = cc_classes_of(spec["stem"], path)
                m = re.search(r"comment_coverage\.py` IS EXPECTED-RED \(rc=1\) ON THIS "
                              r"PAIR \[classes: ([^\]]*)\]",
                              re.sub(r"\s+", " ", " ".join(
                                  l.lstrip("#").strip() for l in text.split("\n")
                                  if l.startswith("#"))))
                stated = [c.strip() for c in m.group(1).split(",")] if m else []
                if stated != found:
                    problems.append(
                        f"{spec['family']}/{spec['toml']}: header declares "
                        f"comment_coverage classes {stated}, the checker reports {found}")
            want = declared.get(gate)
            if rc == 0 and want is not None:
                problems.append(f"{spec['family']}/{spec['toml']}: header declares {gate} "
                                f"expected-red (rc={want}) but it exits 0")
            elif rc != 0 and want is None:
                problems.append(f"{spec['family']}/{spec['toml']}: {gate} exits {rc} and the "
                                f"header declares nothing about it")
            elif rc != 0 and want != rc:
                problems.append(f"{spec['family']}/{spec['toml']}: header declares {gate} "
                                f"rc={want}, it exits {rc}")
    return problems


def main(argv):
    mode = argv[1] if len(argv) > 1 else "--check"
    files = rendered()
    if mode == "--list":
        total = 0
        for f in build():
            product = 1
            for v in (f["matrix"] or {}).values():
                product *= len(v)
            n = len(f["cases"]) * product
            total += n
            print(f"  {f['family']}/{f['toml']}.toml  <- {f['stem']}.rs  "
                  f"{len(f['cases'])} case(s) x {product} = {n} trial(s)")
        print(f"  {len(files)} file(s), {total} trial(s)")
        return 0
    if mode == "--write":
        for path, text in files.items():
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w") as fh:
                fh.write(text)
            print(f"wrote {os.path.relpath(path, REPO)}")
        return 0
    if mode != "--check":
        print(__doc__)
        return 2
    drift = []
    for path, text in files.items():
        if not os.path.exists(path):
            drift.append((path, "absent from the tree"))
        elif open(path).read() != text:
            drift.append((path, "differs from this generator's output"))
    for path, why in drift:
        print(f"DRIFT: {os.path.relpath(path, REPO)} -- {why}")
    if drift:
        print(f"GENERATOR NOT A FIXED POINT -- {len(drift)} of {len(files)} file(s) drifted")
        return 1
    problems = check_gate_declarations(files)
    for p in problems:
        print(f"GATE DECLARATION MISMATCH: {p}")
    if problems:
        print(f"{len(problems)} header(s) disagree with the gate they describe")
        return 1
    print(f"GENERATOR FIXED POINT -- {len(files)} case file(s) reproduced byte-for-byte, "
          f"and every EXPECTED-RED declaration agrees with the gate it names")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
