#!/usr/bin/env python3
r"""Generator for Task 19 batch 3 -- the seven `run_source` single-fixture targets.

WHAT THIS BATCH IS, AND THE ONE PREDICATE THAT SELECTED IT (ruling 13 -- the
command is in the report beside the figure). Every target here declares

    fn run_source(src: &str) -> std::process::Output

writes exactly one program to `main.ts` under a fresh temp directory, runs
`kali run <path>` on it, and constructs no other `Command`; and every `#[test]`
fn in the file calls that helper EXACTLY ONCE. Seven targets, 123 `#[test]`
fns, 123 invocations. The consequence is that rule 6's 1:1 mapping holds by
construction, rule 5 never fires, and rule 7's arithmetic is `N == N x 1` in
every file.

WHAT DECIDES A CASE'S CLAIMS -- NOT THIS FILE (and that is the point). Batch 2
listed 17 files' cases by hand. 123 is past where that is honest, so the claim
set of every case is DERIVED by `t19b3_extract.claims_of`, whose shape table is
CLOSED: an assertion it does not model raises `UnknownShape` naming the file,
the fn and the verbatim condition. A forward extractor that silently skips what
it does not understand turns a dropped claim into a green run, which is the
failure this project keeps finding; here it is a loud generator instead.

CLOSED OVER CLAIMS, NOT MERELY OVER `assert*!` MACROS. Enumerating assertion
macros says nothing about a claim written as `if !x.contains(y) { panic!() }`,
carried by an `.expect()`, or made anywhere outside a macro -- a source using one
would have migrated silently short. `residual_claims` blanks every assert span
and the handful of permitted non-asserting forms, then refuses on what is left.

Probed rather than trusted -- `probe_task19_batch3.py` section 1 mutates real
sources nine ways (an unmodelled `assert!`, an `assert_ne!`, a stdout
`.contains`, a removed exit assertion, a `format!`-built fixture, a second
`run_source` call, and the three claim-level shapes above) and requires every one
to raise, with the unmutated control clean.

FIXTURE TEXT IS COPIED, NEVER TYPED (rules 8/9). `t19b3_extract.fixture_of`
resolves the literal the test actually hands to `run_source` -- either the
call's own string literal or the `let`-bound one it names -- through
`lexer.find_string_literals`, and raises on anything else. Expected stdout is
the literal inside the source's own `assert_eq!`, copied the same way. Exactly
ONE value in the batch exists as no literal at all
(`runtime_module_globals::module_var_lcg_float_division`, whose expectation the
source COMPUTES in Rust); it is declared below, re-derived by EXECUTING the
source's own arithmetic (rule 8), and cross-checked against the real binary.

Source comment prose reaches a rationale the same way -- read out of the `.rs`
by `t19b3_extract.prose`, so rule 12's "copied, not retyped" holds by
construction and an em-dash cannot become `--` in transit.

  Usage:
    gen_task19_batch3.py            # CHECK: regenerate and diff; rc=1 on drift
    gen_task19_batch3.py --write    # emit the 7 case files
    gen_task19_batch3.py --list     # the file list and per-file trial counts

The default is the CHECK direction, for the reason batch 2 gave: a generator
that only writes is a fixed point nobody re-tests. It also requires every
`EXPECTED-RED (rc=N)` paragraph in the headers it renders to AGREE with the
gate it names (ruling 18 #3).
"""

from __future__ import annotations

import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "tools/task-18-browser-pilot"))
sys.path.insert(0, HERE)

from case_emit import emit  # noqa: E402
import t19b3_extract as X  # noqa: E402

# WHY THIS GENERATOR WRITES WITH `open()` AND NOT `case_emit.write`, AND THE
# MEASUREMENT THAT SETTLED IT.
#
# `case_emit.write` folds in `declare_source_ref`, which asks
# `deleted_by_family_deletion(<source>.rs)` -- "was this removed by batch 8C's
# browser-family deletion?" -- implemented as "the blob at FAMILY_DELETION_REF
# exists and does not open with `//!`". That predicate is sound INSIDE the
# browser family and wrong outside it: every non-browser source also existed at
# that commit and none of them opens with `//!`, so it answers True for all of
# them. `declare_source_ref` then finds a header declaring this batch's own
# `SOURCE REF:` and RAISES ("refusing to overwrite -- one of the two is wrong
# and this cannot tell which").
#
#     python3 - <<'EOF'
#     import sys; sys.path.insert(0, "tools/task-18-browser-pilot")
#     import case_emit as CE
#     print(CE.deleted_by_family_deletion("param_compound_assign.rs"))   # -> True
#     CE.declare_source_ref(open(
#         "crates/kali_cli/tests/cases/misc/heap_grow_runtime.toml").read())
#     # -> AssertionError: header declares SOURCE REF b7f2ed5d... but the family
#     #    deletion ref is 28df9ba0...
#     EOF
#
# So it would raise on a SHIPPED batch-2 file too. It is dormant only because no
# non-browser generator routes through `case_emit.write` -- batch 2's writes with
# `open()`, which this follows. Batch 2's report §6.2 states the opposite ("for a
# non-browser file ... `deleted_by_family_deletion` returns False, so no
# declaration is inserted"); that sentence is corrected in this batch's report.
# Not fixed here: `case_emit` is the emitter that renders 161 irreplaceable
# browser case files, and the failure is LOUD (an AssertionError), not a silent
# permission.

TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases")
PILOT = os.path.join(REPO, "tools/task-18-browser-pilot")

# The commit this batch's `SOURCE REF:` declarations name. All seven sources are
# still in the tree, so `citation_sweep.sh` content-validates each declaration
# against the working-tree file on every run -- which is the whole reason to
# declare it NOW rather than after the sources are deleted, when nothing could.
SOURCE_REF = "020b5c33707052455d68dc44cac72a12e961f6ae"

# THE ONE COMPUTED EXPECTATION IN THE BATCH, and how it was obtained.
#
# `runtime_module_globals::module_var_lcg_float_division` builds its expected
# stdout with a Rust loop (`let mut s: i64 = 42; ... lines.push_str(&format!(
# "{}\n", v))`), so it exists as no string literal anywhere in the `.rs` and
# `lexer.find_string_literals` cannot reach it. Rule 8 forbids hand-simulating
# it. It was obtained by EXECUTING the source's own block, copied verbatim into
# a standalone `main`, with rustc:
#
#     rustc -O -o lcg lcg.rs && ./lcg | cat -A
#     -> 0.3746499199817101$
#        0.729023776863283$
#
# and independently cross-checked against the real binary running the very
# fixture this case ships (`kali run main.ts` on that program prints the same
# two lines). Two derivations, one from each side of the assertion, agreeing --
# which is what makes it a capture rather than a transcription. `check_computed`
# re-asserts below that the source still builds it the same way.
LCG_STDOUT = "0.3746499199817101\n0.729023776863283\n"

LCG_NEEDLES = [
    "let mut s: i64 = 42;",
    "s = (s * 3877 + 29573) % 139968;",
    "let v = (1.0 * s as f64) / 139968.0;",
    'lines.push_str(&format!("{}\\n", v));',
]


def check_computed(stem, needles):
    """A capture is only usable while the `.rs` still builds it the same way."""
    text = X.source(stem)
    missing = [n for n in needles if n not in text]
    if missing:
        raise AssertionError(
            f"the computed-stdout capture for {stem} is stale: {stem}.rs no longer "
            f"contains {missing!r}. Re-derive it by EXECUTING the source's own "
            "block; do not hand-simulate it (rule 8).")


# --------------------------------------------------------------------------
# The seven files. `subject` is the only prose written here rather than read
# out of the source, and it is a one-line summary, never a claim.
# --------------------------------------------------------------------------

FILES = [
    ("misc", "param_compound_assign", "param_compound_assign",
     "Compound assignment and `++` on a PARAMETER: the provably-scalar lanes run, "
     "and every shape whose scalar-ness interprocedural flow cannot positively "
     "prove rejects fail-closed instead of miscompiling."),
    ("runtime", "join", "runtime_join",
     "Runtime `Array.prototype.join`: the structural-receiver lanes print, and "
     "every non-structural receiver, unproven separator and mixed-element array "
     "rejects rather than emitting a silent `0`."),
    ("runtime", "module_globals", "runtime_module_globals",
     "Module-scope `var` promoted to a persistent mutable wasm global: reads, "
     "writes, compound assignment, float use sites and lexical shadowing, with "
     "module-scope OBJECTS still fail-closed."),
    ("runtime", "string_arrays", "runtime_string_arrays",
     "Runtime String-element arrays: element stores, reads, `.length`, "
     "`.substring` and reallocation work, while every non-structural store "
     "target and every mixed-element shape rejects."),
    ("runtime", "string_value_flow", "runtime_string_value_flow",
     "Runtime string VALUE flow: concatenation, accumulation, `+=` and content "
     "equality run, while truthiness, ordering and consumed mixed returns "
     "reject rather than compare raw handles."),
    ("runtime", "substring_length", "runtime_substring_length",
     "Runtime `String.prototype.substring` and `.length`: the ASCII lanes print "
     "JS-identical results, and non-ASCII receivers, fractional bounds and "
     "runtime-string stores reject."),
    ("runtime", "ternary", "runtime_ternary",
     "Runtime conditional expressions: arm selection, laziness, promotion and "
     "string-armed concatenation run, while arm-type conflicts, ternary "
     "`.length` and ternary-wrapped runtime-string stores reject."),
]


# --------------------------------------------------------------------------
# Header boilerplate, DERIVED per file rather than restated seven times.
# --------------------------------------------------------------------------

def head(stem, subject):
    return [f"Migrated from tests/{stem}.rs.",
            f"  SOURCE REF: {SOURCE_REF}",
            "",
            subject,
            ""]


THE_SHAPE = (
    "THE SHAPE THIS FILE WAS MIGRATED FROM, AND WHAT DERIVED ITS CLAIMS. The "
    "source declares one helper -- `fn run_source(src: &str) -> "
    "std::process::Output` -- which writes a single program to `main.ts` under a "
    "fresh temp directory and runs `kali run <path>` on it. Every `#[test]` fn in "
    "the file calls it EXACTLY ONCE and constructs no other command, so one "
    "`#[test]` fn is one `[[case]]` is one trial (rule 6, one-to-one) and rule 5's "
    "split never applies. (\"one-to-one\", not the arithmetic spelling: "
    "`citation_sweep.sh`'s `CITE` reads a backticked construct followed by a "
    "colon-digit as a line citation, and batch 2 had to reword the same phrase "
    "for the same reason.)\n"
    "\n"
    "The claim set of every case below is DERIVED from the source by "
    "`tools/migration/t19b3_extract.py`, not listed by hand, and its shape table "
    "is CLOSED: an `assert!`/`assert_eq!` it does not model raises rather than "
    "being skipped. That direction is the point -- a forward extractor that "
    "silently skips what it does not understand turns a DROPPED claim into a "
    "green run. The modelled shapes are exactly:\n"
    "\n"
    "  assert!(out.status.success(), ..)              -> exit = \"success\"\n"
    "  assert!(!out.status.success(), ..)             -> exit = \"failure\"\n"
    "  assert_eq!(String::from_utf8_lossy(&out.stdout), <str>)  -> exact `stdout`\n"
    "  assert!(out.stdout.is_empty(), ..)            -> exact `stdout = \"\"`\n"
    "  assert!(<stderr>.contains(<str>), ..)          -> `stderr_contains`\n"
    "  assert!(<stderr>.contains(A) || <stderr>.contains(B), ..)\n"
    "                                                 -> rule 11 / ruling 17\n"
    "\n"
    "An `assert!`'s second and later arguments are its PANIC MESSAGE, which the "
    "program under test never sees; they are not claims and are not migrated.\n"
    "\n"
    "AND THE TABLE IS CLOSED OVER CLAIMS, NOT MERELY OVER `assert*!` MACROS. "
    "Enumerating assertion macros says nothing about a claim written as `if "
    "!x.contains(y) {{ panic!() }}`, carried by an `.expect()`, or made anywhere "
    "outside a macro; a source using one would migrate silently short. "
    "`t19b3_extract.residual_claims` blanks every assert span and the handful of "
    "permitted non-asserting forms, then REFUSES on what is left. All nine "
    "refusals -- six on assertion shapes, three on claim shapes -- are fired by "
    "`probe_task19_batch3.py` section 1 against real sources, with the unmutated "
    "sources as the control. Reproduce the derivation with:\n"
    "\n"
    "  python3 tools/migration/t19b3_extract.py {stem}")

# The third bullet's population is DERIVED, because on two of this batch's seven
# targets it is ZERO: `runtime_join.rs` and `runtime_string_arrays.rs` carry no
# `//!` doc and no comment on `run_source`/`kali_bin`, so there is no file-wide
# prose at all. Boilerplate that describes a population a file does not have is
# the same defect as a header naming a gate red that has gone green -- it reads
# as a carry a reader can go looking for and not find.
FW_PRESENT = (
    "  * FILE-WIDE prose -- the `//!` module doc and the comments attached to "
    "`run_source`/`kali_bin` -- is carried in this `#` header, which is the one "
    "place rule 12 puts prose that every case reaches equally. This source HAS "
    "such prose; it is the FILE-WIDE SOURCE PROSE paragraph below.")

FW_ABSENT = (
    "  * FILE-WIDE prose would be carried in this `#` header, which is the one "
    "place rule 12 puts prose that every case reaches equally. THIS SOURCE HAS "
    "NONE: it carries no `//!` module doc and no comment on "
    "`run_source`/`kali_bin`, so every comment in it belongs to a test or to a "
    "section and there is no file-wide paragraph below. Stated rather than left "
    "to silence -- *no prose* and *prose missed* are otherwise indistinguishable "
    "to a later reader, which is the reason ruling 5 gave `comment_coverage.py` "
    "a zero-line floor.")

RULE_12_ATTRIBUTION = (
    "RULE 12 ATTRIBUTION, DERIVED FROM THE SOURCE'S OWN LAYOUT (U6). Every "
    "comment in the source is carried, and which rationale it lands in is decided "
    "by position rather than per-comment judgement -- "
    "`t19b3_extract.prose` implements the split and this generator reads it:\n"
    "\n"
    "  * a comment paragraph (or `///` doc block) INSIDE a `#[test]`'s body, or "
    "directly abutting it, goes into THAT case's rationale and no other. U6: "
    "over-attribution is forbidden even though it turns `comment_coverage.py` "
    "green. The inside-the-body arm is not decoration: an abutting-only "
    "predicate filed this batch's in-body comments as file-wide, which is "
    "under-attribution, and the shipped predicate checks it first.\n"
    "  * a SECTION BANNER -- a paragraph the source itself brackets with `----` "
    "divider lines, or a single line wrapped in them (`---- Finding 2: ... ----`) "
    "-- goes into the rationale of every case in its section, the section "
    "running from the banner to the next banner or to end of file. That is rule "
    "12's own sentence for prose attached to a section, applied mechanically to "
    "the source's own delimiters instead of re-deciding per test which of the "
    "following fns a banner \"really\" means.\n"
    "{file_wide_bullet}\n"
    "\n"
    "TRAILING comments (U16) are attributed by line to whichever `#[test]` body "
    "encloses them. `comment_coverage.py` was blind to that whole shape until "
    "Task 19 batch 2 closed it.")

RULING6 = (
    "RULING 6 EXEMPTION, APPLIED TO THE HARNESS HELPER'S OWN PROSE. The comments "
    "carried in the paragraph above describe how the Rust harness kept its own "
    "temp directories from colliding (a pid + atomic-counter slug). The migrated "
    "case depends on nothing that helper computed: the case runner gives every "
    "trial its own directory, so the helper's job is now the runner's job -- "
    "which is ruling 6's own test for the exemption. Carried verbatim here so "
    "nothing is dropped (rule 12), and deliberately NOT replicated into every "
    "rationale, because attributing harness mechanics to every case is the "
    "over-attribution U6 forbids.")

ASSERTION_POLICY = (
    "ASSERTION STRENGTH -- RULING 3, MIRROR THE SOURCE, CLAUSE BY CLAUSE.\n"
    "\n"
    "  * an exact `assert_eq!` on the WHOLE of stdout is an exact source "
    "assertion, so it becomes an exact `stdout` pin (clause 1). The pinned text "
    "is COPIED out of the source's own literal by "
    "`lexer.find_string_literals`; it is not a live capture and this header does "
    "not claim it is. Batch 2 shipped a header saying \"captured from the real "
    "binary, never computed\" about values that were in fact copied out of the "
    "source, and had to correct it -- the distinction is recorded here so it is "
    "not lost again.\n"
    "  * `assert!(out.stdout.is_empty())` is an exact assertion about the whole "
    "of stdout, so it becomes `stdout = \"\"`. That is the same exact-stdout "
    "discipline used everywhere else on a taken, verified path, not a rule-2 "
    "invention.\n"
    "  * a plain `.contains` against stderr stays `stderr_contains` (clause 3): "
    "the field has a substring form, and it is NOT strengthened to an exact pin "
    "just because the exact output was observed.\n"
    "  * a test whose only assertion is `!out.status.success()` pins `exit = "
    "\"failure\"` and NOTHING about either stream. The real binary emits a "
    "diagnostic on every one of them, and pinning it would be rule 2's exact "
    "prohibition: never add an assertion the source did not make merely because "
    "it is true.\n"
    "\n"
    "REALISM IS PROVEN SEPARATELY FROM FIDELITY (U9). Every trial in this file "
    "runs against the real `kali` binary in the suite; that proves the case "
    "matches what the binary does today. It does not prove nothing was dropped -- "
    "only the source-vs-TOML direction does that, which is what the derived "
    "extraction and `check_extra_claims.py` are for.")


def u5(n, stem):
    return (
        f"`[source]` KEYS ARE VARIANT-SUFFIXED (U5), ALL {n} OF THEM. The source "
        f"wrote every one of these programs to `main.ts`, under a fresh temp "
        f"directory per `#[test]`. `[source]` here is one FILE-WIDE namespace that "
        f"`expand()` clones into every trial, so {n} programs cannot share one key "
        f"-- the last body written would win and every other case would silently "
        f"run the wrong program. Each case's program is keyed by the source "
        f"`#[test]` fn that wrote it, and that name is what the case passes as "
        f"argv, so the mapping from trial to source fn survives the deletion of "
        f"`{stem}.rs`. Every renamed key is declared `# EXTRA-OK:` below: it is a "
        f"fixture FILENAME surfaced as an argv token, not a claim about behaviour. "
        f"No program in this file references a sibling filename by string, which "
        f"is the check U5 actually asks for and which "
        f"`gen_task19_batch3.check_no_fixture_names_referenced` runs over every "
        f"fixture body rather than over argv alone.")


U2_INERT = (
    "U2 CHECK, RUN RATHER THAN ASSUMED. `[source]` is one flat FILE-WIDE map that "
    "`expand()` clones into every trial, so every sibling case's program is "
    "present in this case's trial directory too. That is inert here, and the "
    "controls were run rather than reasoned about:\n"
    "\n"
    "  * every case names its own program explicitly on argv, exactly as the "
    "source did (`kali run <fn-name>.ts`), so a sibling `.ts` in the same "
    "directory is never read. Control: `kali run a.ts` in a directory also "
    "holding a `b.ts` that fails to compile still prints a.ts's output, rc=0; "
    "the same command naming `b.ts` fails, rc=1. The discriminator is argv, and "
    "argv is what these cases carry.\n"
    "  * `kali.json` IS auto-discovered as a manifest and would NOT be inert. No "
    "fixture in this file is named `kali.json` -- every key is a source `#[test]` "
    "fn name with a `.ts` suffix -- and gen_task19_batch3's "
    "check_no_manifest_named_fixture asserts it rather than leaving it to "
    "inspection. (Tool function names are written unbackticked in this header on "
    "purpose: `check_rationale_fn_names.py` reads a backticked fn-shaped token as "
    "a citation into the SOURCE, and a generator's own function is not one.)\n"
    "\n"
    "No fixture here is one whose mere PRESENCE or ABSENCE is a case's point, "
    "which is the shape U2 exists to stop: every case in this family is a single "
    "program run by name.")


def extra_ok(pairs):
    return [f"EXTRA-OK: {v!r} -- {why}" for v, why in pairs]


CC_RED_PER_CASE = (
    "PER-CASE ATTRIBUTION (`from N/M cases` lines). A comment attached to one "
    "`#[test]` fn belongs in the rationale of the case that fn produced and "
    "nowhere else, and a SECTION banner belongs in the rationales of its own "
    "section and nowhere else. U6 says so and calls copying all of a file's "
    "comments into all of its cases \"over-attribution ... forbidden, even "
    "though it turns the checker green\".")

CC_RED_FILE_WIDE = (
    "FILE-WIDE PROSE IN THE HEADER (`from ALL N cases` lines). The source's "
    "`//!` module doc, the prose attached to `run_source`/`kali_bin` carried "
    "under ruling 6's exemption, and the `----` divider ornament that "
    "`comment_coverage.py` counts as a comment line whenever it shares a "
    "paragraph with text, all describe the whole file rather than any one "
    "case. Rule 12 puts them in this `#` header -- which `comment_coverage.py` "
    "deliberately does not read as coverage. Every such line is prose that IS "
    "carried, in the one place the rule says it belongs.")

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
    "`gen_task19_batch3.check_gate_declarations` re-runs the checker and "
    "requires the classes its output actually contains to match the ones named "
    "here. Reproduce with:\n"
    "\n"
    "  python3 tools/task-18-browser-pilot/comment_coverage.py \\\n"
    "    crates/kali_cli/tests/{stem}.rs \\\n"
    "    crates/kali_cli/tests/cases/{family}/{toml}.toml")

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
    "`gen_task19_batch3.u8_reason` re-runs the gate, reads back the identifiers "
    "it could not explain, and decides for each whether the source DEFINES a "
    "JavaScript function of that name inside its own fixture programs. Reproduce "
    "with:\n"
    "\n"
    "  python3 tools/task-18-browser-pilot/check_rationale_fn_names.py \\\n"
    "    crates/kali_cli/tests/{stem}.rs \\\n"
    "    crates/kali_cli/tests/cases/{family}/{toml}.toml")

U8_REMEDY_ALLOWLIST = (
    "REMEDY: a CLI-family entry in the gate's own known-identifier allowlist. "
    "These name items in other crates, in the compiler under test, or in the "
    "Rust standard library, which this file's source never declares, so no "
    "amount of correct prose resolves them. The pilot flagged the same gap on "
    "`nullish/assign_reject.toml` and `misc/growable_array_fail_closed.toml`, "
    "and batch 2 on four more.")

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


def _run(cmd):
    return subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)


# Which file the gate-derived paragraphs are MEASURED against. `rendered()`
# points this at the rendering in progress rather than at the shipped file:
# measuring the shipped file makes every derived paragraph lag one iteration
# behind the spec, so a first `--write` after a spec change emits a paragraph
# describing the PREVIOUS rendering and only converges on a second run.
_MEASURE_PATH = {}


def _measure(toml_path):
    return _MEASURE_PATH.get(toml_path, toml_path)


def u8_reason(stem, toml_path):
    """The U8 paragraph for this pair, with its names and remedy DERIVED."""
    out = _run([sys.executable, os.path.join(PILOT, "check_rationale_fn_names.py"),
                os.path.join(TESTS, stem + ".rs"), _measure(toml_path)])
    names = re.findall(r"UNEXPLAINED: `([^`]+)`", out.stdout)
    if not names:
        return None
    # THE PARAGRAPH BELOW CLAIMS THESE NAMES ARE RULE-12 CARRIES. DERIVE IT
    # RATHER THAN ASSERT IT (ruling 18 #1, and batch 2's `U8_RED` false reason).
    # A name that appears nowhere in the source `.rs` is not a carry -- it came
    # out of THIS GENERATOR'S OWN boilerplate, which is mine to reword, and
    # shipping it under a "carried verbatim, U7 forbids rewording" banner would
    # be a false header reason. Caught on the first run: the U2 paragraph
    # backticked its own checker's name.
    src = X.source(stem)
    mine = [n for n in names if n not in src]
    if mine:
        raise AssertionError(
            f"{stem}: {mine!r} are reported UNEXPLAINED by check_rationale_fn_names.py "
            "but appear nowhere in the source `.rs` -- they come from this "
            "generator's own header prose, not from a rule-12 carry. Reword the "
            "generator's prose (it is not carried text); do not describe them as "
            "carries.")
    fixtures = "\n".join(x["value"] for x in
                         __import__("lexer").find_string_literals(X.source(stem)))
    in_fixture = [n for n in names
                  if re.search(r"\bfunction\s+" + re.escape(n) + r"\s*\(", fixtures)]
    rendered = ", ".join(f"`{n}`" for n in sorted(names))
    if in_fixture and len(in_fixture) == len(names):
        return (U8_RED.replace("{names}", rendered + " -- every one of them a function "
                               "DEFINED IN THIS SOURCE'S OWN FIXTURE PROGRAMS.")
                      .replace("{remedy}", U8_REMEDY_FIXTURE))
    if in_fixture:
        return (U8_RED.replace(
            "{names}", rendered + " -- of which "
            + ", ".join(f"`{n}`" for n in sorted(in_fixture))
            + " are functions DEFINED IN THIS SOURCE'S OWN FIXTURE PROGRAMS and the "
              "rest name items in other crates, in the compiler under test, or in "
              "the standard library.")
            .replace("{remedy}", U8_REMEDY_FIXTURE + " " + U8_REMEDY_ALLOWLIST))
    return U8_RED.replace("{names}", rendered + ".").replace(
        "{remedy}", U8_REMEDY_ALLOWLIST)


_CC_CLASSES = {}


def cc_classes_of(stem, toml_path):
    """The classes `comment_coverage.py` actually reports for a rendered pair.

    DERIVED FROM THE GATE'S OWN OUTPUT (ruling 18 #1), so a header cannot name a
    class the checker does not report, or omit one it does. Not circular:
    `comment_coverage.py` reads only `rationale` fields and never the `#`
    header, so the classes depend on the half of the file this arm does not
    write; `rendered()` renders, measures, renders again, and asserts the second
    rendering is a fixed point.
    """
    p = _run([sys.executable, os.path.join(PILOT, "comment_coverage.py"),
              os.path.join(TESTS, stem + ".rs"), _measure(toml_path)])
    found = []
    if re.search(r"from \d+/\d+ cases", p.stdout):
        found.append("per-case")
    if "from ALL " in p.stdout:
        found.append("file-wide")
    return found


def gates(entries, stem, family, toml):
    out = []
    toml_path = os.path.join(CASES, family, toml + ".toml")
    for e in entries:
        text = e
        if e is U8_RED:
            text = u8_reason(stem, toml_path)
            if text is None:
                continue
        elif "{classes}" in e:
            found = _CC_CLASSES.get(toml_path, ["per-case", "file-wide"])
            text = (e.replace("{classes}", ", ".join(found) or "none")
                     .replace("{per_case}", CC_RED_PER_CASE if "per-case" in found
                              else "(no per-case lines on this pair)")
                     .replace("{file_wide}", CC_RED_FILE_WIDE if "file-wide" in found
                              else "(no file-wide lines on this pair)"))
        out.append(text.format(stem=stem, family=family, toml=toml))
        out.append("")
    return out


# --------------------------------------------------------------------------
# Mechanical checks this generator runs before it will emit anything
# --------------------------------------------------------------------------

def check_no_fixture_names_referenced(stem, keys, fixtures):
    """U5's real check: no program references a sibling fixture FILENAME.

    An entry filename passed as a CLI argument is always safe to rename; a
    filename the program itself references by string (`import()`, `require()`)
    is not -- renaming it is a rule-9 violation. Checked against every fixture
    BODY, not just against argv.
    """
    bad = [(k, other) for k in keys for other in keys
           if other in fixtures[k]]
    if bad:
        raise AssertionError(
            f"{stem}: a fixture body references a sibling `[source]` key by name "
            f"{bad!r} -- renaming it would be a rule-9 violation.")
    stems = {k[:-3] for k in keys}
    bad = [(k, s) for k in keys for s in stems
           if re.search(r"['\"]" + re.escape(s) + r"['\"]", fixtures[k])]
    if bad:
        raise AssertionError(f"{stem}: fixture body quotes a sibling key stem {bad!r}")


HOIST_LINE_FLOOR = 5


def u13_measure(fixtures):
    """U13, BOTH halves, measured: byte-identical bodies AND long shared prefixes.

    U13 hoists "any `[source]` value duplicated across entries, OR a long common
    prefix shared by two or more entries", and the identity must be asserted
    mechanically rather than eyeballed. The first half is easy to answer here and
    was; the second half is the one a reader would otherwise have to take on
    trust, so it is computed: the longest run of identical leading LINES over
    every unordered pair of bodies in the file.

    Returns `(duplicate_count, longest_shared_prefix_lines)`. The generator
    writes both into the header and raises if the prefix reaches
    `HOIST_LINE_FLOOR`, so a future source whose fixtures grow a hoistable prefix
    breaks the generator instead of silently declining the hoist. That is ruling
    15's answer 1: the figure IS the gate's own output, recorded from inside the
    gate's own loop.
    """
    bodies = list(fixtures.values())
    dup = sum(1 for i in range(len(bodies)) for j in range(i + 1, len(bodies))
              if bodies[i] == bodies[j])
    longest = 0
    for i in range(len(bodies)):
        for j in range(i + 1, len(bodies)):
            a, b = bodies[i].split("\n"), bodies[j].split("\n")
            n = 0
            while n < min(len(a), len(b)) and a[n] == b[n] and a[n].strip():
                n += 1
            longest = max(longest, n)
    return dup, longest


U13_NOTE = (
    "U13, BOTH HALVES, MEASURED RATHER THAN EYEBALLED. The rule hoists a "
    "`[source]` value duplicated across entries **or** a long common prefix "
    "shared by two or more entries, and requires the identity to be asserted "
    "mechanically. Over this file's {n} bodies: **{dup} byte-identical pair(s)** "
    "and a **longest shared leading-line prefix of {prefix} line(s)**. Nothing "
    "to hoist on either half, so `[constants]` is declined -- and the figures "
    "are this generator's own output, recorded from inside the loop that "
    "produces them (ruling 15's answer 1), with gen_task19_batch3's u13_measure "
    "raising if the prefix ever reaches " + str(HOIST_LINE_FLOOR) + " lines. The "
    "second half is stated because batch 2 answered only the first, and an "
    "unanswered half of a rule reads the same as a satisfied one.")


_NO_STREAM_CLAIM = "makes no claim about either stream"
_NO_STDOUT_CLAIM = "makes no claim about STDOUT"


def check_rationales_match_their_claims(stem, cases):
    """A rationale may not describe a claim set the case does not have.

    U8: rationale prose is audited by NOTHING, and this batch shipped four
    rationales saying "It makes no claim about either stream" two sentences
    before rendering their own `stderr_contains` clause. The claims were correct
    and the prose a reader of a failing trial sees was false. Both directions are
    checked, against the STEP rather than against the intermediate claim set, so
    the assertion cannot be satisfied by the same variable that produced the
    error.
    """
    for case in cases:
        step, r = case["steps"][0], case["rationale"]
        streams = ("stdout" in step) or ("stderr_contains" in step)
        if _NO_STREAM_CLAIM in r and streams:
            raise AssertionError(
                f"{stem}::{case['name']}: the rationale says {_NO_STREAM_CLAIM!r} "
                f"but the case pins {sorted(k for k in step if k not in ('args', 'exit'))}")
        if _NO_STDOUT_CLAIM in r and "stdout" in step:
            raise AssertionError(
                f"{stem}::{case['name']}: the rationale says {_NO_STDOUT_CLAIM!r} "
                "but the case pins `stdout`")
        if _NO_STDOUT_CLAIM in r and "stderr_contains" not in step:
            raise AssertionError(
                f"{stem}::{case['name']}: the rationale promises a stderr claim "
                "below and the case carries none")


def check_no_manifest_named_fixture(stem, keys):
    """`kali.json` IS auto-discovered as a manifest; nothing here may be named it."""
    bad = [k for k in keys if k.lower() in ("kali.json", "package.json", "tsconfig.json")]
    if bad:
        raise AssertionError(f"{stem}: auto-discovered manifest name used as a "
                             f"fixture key: {bad!r} (U2)")


def arithmetic(invocations, cases):
    """Rule 7, asserted rather than stated: a header cannot record arithmetic
    that does not close."""
    if invocations != cases:
        raise AssertionError(
            f"matrix arithmetic does not close: {invocations} invocation(s) != "
            f"{cases} case(s) x 1")
    return [(f"MATRIX DECLINED. {invocations} source helper invocation(s) == "
             f"{cases} case(s) x 1 -- no axis every case varies over uniformly "
             f"(rule 7, U1). Every `#[test]` fn in this source runs a DIFFERENT "
             f"program with a different expectation; there is no extension, "
             f"command or output-mode axis to fan them over, and inventing one "
             f"would produce combinations the source never exercised (rule 2). "
             f"The identity is asserted inside `gen_task19_batch3.arithmetic`, so "
             f"this sentence cannot state arithmetic that does not close."), ""]


# --------------------------------------------------------------------------
# Rendering
# --------------------------------------------------------------------------

def _claim_sentence(c):
    """The ruling-3 sentence for one case, derived from its own claim set."""
    bits = []
    if c["exit"] == "success":
        bits.append("The source asserts `out.status.success()`, pinned as "
                    "`exit = \"success\"`.")
    else:
        bits.append("The source asserts `!out.status.success()`, pinned as "
                    "`exit = \"failure\"`.")
    if c["stdout_source"] == "copied":
        bits.append("Its `assert_eq!` on the whole of stdout is an exact source "
                    "assertion, so it becomes an exact `stdout` pin (ruling 3 "
                    "clause 1); the value is COPIED out of that literal, not "
                    "retyped and not captured.")
    elif c["stdout_source"] == "computed":
        bits.append("Its expected stdout is COMPUTED in Rust by the source itself, "
                    "so it exists as no literal: the pin was obtained by executing "
                    "the source's own block (rule 8) and cross-checked against the "
                    "real binary running this very fixture.")
    elif c["stdout_source"] == "is_empty":
        bits.append("It also asserts `out.stdout.is_empty()` -- an exact assertion "
                    "about the whole of stdout -- pinned as `stdout = \"\"`.")
    elif not c["stderr_contains"] and not c["disjunctions"]:
        bits.append("It makes no claim about either stream, so none is written: the "
                    "trial does emit a diagnostic, and pinning it would be rule 2's "
                    "exact prohibition against adding an assertion merely because "
                    "it is true.")
    else:
        # DERIVED FROM WHAT THE CASE ACTUALLY CARRIES, not from `stdout_source`
        # alone. The first version chose the "no claim about either stream"
        # sentence whenever there was no stdout pin, and then rendered the
        # `stderr_contains` clause two sentences later -- four shipped
        # rationales contradicted themselves, and a reader of a failing trial
        # sees exactly that prose. The claims were right; the sentence was not,
        # which is U8's whole subject (rationale prose is audited by nothing).
        bits.append("It makes no claim about STDOUT, so none is written; its only "
                    "stream claim is the stderr one below (rule 2: nothing is "
                    "pinned merely because the trial happens to emit it).")
    for needle in c["stderr_contains"]:
        bits.append(f"Its `stderr.contains({needle!r})` is a plain `.contains` "
                    "against a field that HAS a substring form, so it stays a "
                    "`stderr_contains` rather than being strengthened to an exact "
                    "pin (ruling 3 clause 3).")
    for cond, alts in c["disjunctions"]:
        bits.append(
            "RULE 11 / RULING 17 -- AN OR-SHAPED ASSERTION, RESOLVED BY "
            "OBSERVATION. The source's own sentence is carried verbatim: "
            f"`{cond}`. The format has no disjunction, so the OR was resolved "
            "against the real binary. Observed on this cell: the stream is "
            "stderr and EVERY disjunct holds -- it emits `error[E5506]: compound "
            "assignment on binding 'p' is unavailable: it is not a provably "
            "scalar number or string (an array or object value has no "
            "compound-assignment lowering)`. Ruling 17 therefore applies: among "
            "disjuncts universally true on that stream, pin the FIRST IN SOURCE "
            f"ORDER -- {alts[0]!r} -- and disclose the others, which are "
            f"{', '.join(repr(a) for a in alts[1:])}. Pinning all of them would "
            "be a rule-2 invention: `A` and `A and B` are both stronger than `A "
            "or B`, but they are ordered, so pinning one is the strictly weaker "
            "and more faithful strengthening, and the source never asserted the "
            "second unconditionally.")
    return " ".join(bits)


def build(family, toml, stem, subject):
    text = X.source(stem)
    fns = X.test_fns(text)
    pr = X.prose(stem, text)
    section_of = {}
    for block, reached in pr["sections"]:
        for nm in reached:
            section_of.setdefault(nm, []).append(block)

    keys, fixtures, cases = [], {}, []
    for f in fns:
        name = f["name"]
        key = name + ".ts"
        body = X.fixture_of(stem, f)
        computed = LCG_STDOUT if (stem, name) in X.COMPUTED else None
        if computed is not None:
            check_computed(stem, LCG_NEEDLES)
        c = X.claims_of(stem, f, computed_stdout=computed)
        keys.append(key)
        fixtures[key] = body

        parts = [f"Migrated from {stem}.rs, source `#[test]` fn `{name}` -- one "
                 f"`[[case]]` per source `#[test]` fn (rule 6), and the "
                 f"`[source]` key is that fn's name so the mapping survives the "
                 f"source's deletion.",
                 _claim_sentence(c)]
        parts += section_of.get(name, [])
        parts += pr["per_fn"][name]
        step = {"args": ["run", key], "exit": c["exit"]}
        if c["stdout"] is not None:
            step["stdout"] = c["stdout"]
        if c["stderr_contains"]:
            step["stderr_contains"] = list(c["stderr_contains"])
        for _cond, alts in c["disjunctions"]:
            step.setdefault("stderr_contains", []).append(alts[0])
        cases.append(dict(name=name, rationale="\n\n".join(parts), steps=[step]))

    check_no_fixture_names_referenced(stem, keys, fixtures)
    check_no_manifest_named_fixture(stem, keys)
    check_rationales_match_their_claims(stem, cases)
    dup, prefix = u13_measure(fixtures)
    if dup or prefix >= HOIST_LINE_FLOOR:
        raise AssertionError(
            f"{stem}: U13 has something to hoist -- {dup} byte-identical `[source]` "
            f"pair(s), longest shared leading-line prefix {prefix}. Hoist it into "
            "`[constants]` and assert the identity, or state why not.")

    file_wide = pr["file_wide"]
    fw = ""
    if file_wide:
        fw = ("FILE-WIDE SOURCE PROSE, CARRIED VERBATIM (rule 12). Read out of "
              f"`{stem}.rs` by `t19b3_extract.prose`, never retyped:\n\n"
              + "\n\n".join("  " + b.replace("\n", "\n  ") for b in file_wide))

    extras = [(key, "a U5 variant-suffixed `[source]` key surfaced as an argv "
                    "token; it is a fixture FILENAME named after the source "
                    "`#[test]` fn that wrote the program, not a claim about "
                    "behaviour") for key in keys]
    if any(c["name"] == "module_var_lcg_float_division" for c in cases):
        extras.append((LCG_STDOUT,
                       "a deliberate exact pin whose value the SOURCE COMPUTES in "
                       "Rust (`lines.push_str(&format!(\"{}\\n\", v))` over an LCG "
                       "loop), so it exists as no literal in the `.rs` and "
                       "`check_extra_claims.py` cannot find it there. Obtained by "
                       "EXECUTING the source's own block (rule 8) and cross-checked "
                       "against the real binary running this fixture"))
    if any("stdout" in c["steps"][0] and c["steps"][0]["stdout"] == "" for c in cases):
        extras.append(("", "the exact `stdout = \"\"` pin that `assert!(out.stdout."
                           "is_empty())` becomes. The empty string is not a literal "
                           "anywhere and cannot be; it is the source's own exact "
                           "claim about the whole of stdout"))

    header = (
        head(stem, subject)
        + arithmetic(len(fns), len(cases))
        + [THE_SHAPE.replace("{stem}", stem), ""]
        + [ASSERTION_POLICY, ""]
        + [u5(len(keys), stem), ""]
        + [U13_NOTE.format(n=len(fixtures), dup=dup, prefix=prefix), ""]
        + [U2_INERT, ""]
        + [RULE_12_ATTRIBUTION.replace("{file_wide_bullet}", FW_PRESENT if file_wide
                                       else FW_ABSENT), ""]
        + ([fw, ""] if fw else [])
        + ([RULING6, ""] if fw else [])
        + extra_ok(extras)
        + [""]
        + gates([CC_RED, U8_RED], stem, family, toml))
    return header, None, fixtures, cases


def rendered(family, toml, stem, subject):
    """Render to a FIXED POINT, measuring against the rendering in progress.

    Two of this header's paragraphs are derived from gates that read the
    rendered file -- `comment_coverage.py`'s class list and U8's unexplained-name
    list -- so rendering changes the input to the measurement that shapes the
    rendering. That is not circular (neither gate reads the `#` header as
    coverage: `comment_coverage.py` reads only `rationale` fields, and the U8
    arm's own paragraph names only tokens that are already in it), but it does
    need iterating: render, measure, render again, and stop when the text stops
    moving. The measurement is taken against a temporary copy of the CURRENT
    rendering, never against the shipped file -- measuring the shipped file
    makes every derived paragraph lag one spec revision.
    """
    toml_path = os.path.join(CASES, family, toml + ".toml")
    tmp = toml_path + ".genprobe"
    _MEASURE_PATH[toml_path] = tmp
    prev = None
    try:
        for _ in range(6):
            text = emit(*build(family, toml, stem, subject))
            if text == prev:
                return text
            prev = text
            with open(tmp, "w") as fh:
                fh.write(text)
            _CC_CLASSES[toml_path] = cc_classes_of(stem, toml_path)
        raise AssertionError(
            f"{toml}: rendering did not reach a fixed point in 6 iterations")
    finally:
        _MEASURE_PATH.pop(toml_path, None)
        if os.path.exists(tmp):
            os.unlink(tmp)


# --------------------------------------------------------------------------
# EXPECTED-RED agreement (ruling 18 #3)
# --------------------------------------------------------------------------

# ALL FIVE OF RULING 19's GATES, NOT THE THREE THIS BATCH HAPPENS TO NEED.
#
# Ruling 9 says every pair red-lists EVERY gate that is expected-red, and names
# the failure it exists to catch: a fifth unnamed red. The first version of this
# table listed only the three gates that are non-zero on this batch, which makes
# the mechanism agree with the corpus rather than with the rule -- and the two it
# omitted, `audit-case-migration.py` and `check_extra_claims.py`, are exactly the
# two a future CLAIM change would move. Demonstrated before the fix: deleting one
# `# EXTRA-OK:` line from `module_globals.toml` takes `check_extra_claims.py` to
# rc=1 while `check_gate_declarations()` returned `[]`.
#
# `audit-case-migration.py` is run from `crates/kali_cli/tests` with paths
# relative to it, which is the only calling convention it accepts.
GATE_CMDS = {
    "audit-case-migration.py": lambda stem, toml_path: (
        [sys.executable, os.path.join(REPO, "scripts/audit-case-migration.py"),
         stem + ".rs", os.path.relpath(toml_path, TESTS)], TESTS),
    "check_fixtures.py": lambda stem, toml_path: (
        [sys.executable, os.path.join(PILOT, "check_fixtures.py"),
         os.path.join(TESTS, stem + ".rs"), toml_path], REPO),
    "check_extra_claims.py": lambda stem, toml_path: (
        [sys.executable, os.path.join(PILOT, "check_extra_claims.py"),
         os.path.join(TESTS, stem + ".rs"), toml_path], REPO),
    "comment_coverage.py": lambda stem, toml_path: (
        [sys.executable, os.path.join(PILOT, "comment_coverage.py"),
         os.path.join(TESTS, stem + ".rs"), toml_path], REPO),
    "check_rationale_fn_names.py": lambda stem, toml_path: (
        [sys.executable, os.path.join(PILOT, "check_rationale_fn_names.py"),
         os.path.join(TESTS, stem + ".rs"), toml_path], REPO),
}

DECL = re.compile(r"`([a-z_0-9]+\.py)` (?:\(U8\) )?IS EXPECTED-RED \(rc=(\d+)\)")


def check_gate_declarations(verbose=True):
    """Every `EXPECTED-RED (rc=N)` paragraph must agree with the gate it names,
    and every non-zero gate must be declared. Both directions: a header that
    claims a red that has gone green fails here too.

    RUN OVER ALL FIVE OF RULING 19's GATES (see `GATE_CMDS`), not over the three
    that are non-zero on today's corpus. Its own known positive runs first:
    a synthetic pair whose header declares nothing is required to be reported by
    every gate that is red on it, so a table that silently stopped invoking a
    gate cannot pass.
    """
    bad = _selftest_declaration_gate()
    for family, toml, stem, _subject in FILES:
        toml_path = os.path.join(CASES, family, toml + ".toml")
        bad += _evaluate_pair(family, toml, stem, toml_path, verbose=verbose)
    return bad


def _evaluate_pair(family, toml, stem, toml_path, *, verbose):
    """One pair against all five gates. Split out so the selftest below can run
    the SAME code over a deliberately-broken copy -- a known positive that
    re-implemented the comparison would prove nothing about the shipped one."""
    bad = []
    text = open(toml_path).read()
    head_blob = " ".join(
        l.lstrip("#").strip() for l in text.splitlines() if l.startswith("#"))
    head_blob = re.sub(r"\s+", " ", head_blob)
    declared = {g: int(rc) for g, rc in DECL.findall(head_blob)}
    for gate, cmd in GATE_CMDS.items():
        argv, cwd = cmd(stem, toml_path)
        rc = subprocess.run(argv, cwd=cwd, capture_output=True, text=True).returncode
        if rc and gate not in declared:
            bad.append(f"{family}/{toml}: {gate} exits {rc} and the header "
                       f"declares no EXPECTED-RED for it (ruling 9: name EVERY "
                       f"gate that is expected-red)")
        elif not rc and gate in declared:
            bad.append(f"{family}/{toml}: the header declares {gate} "
                       f"EXPECTED-RED (rc={declared[gate]}) but it exits 0")
        elif rc and declared.get(gate) != rc:
            bad.append(f"{family}/{toml}: {gate} exits {rc}, header declares "
                       f"rc={declared[gate]}")
        elif verbose:
            state = f"EXPECTED-RED rc={rc}" if rc else "green"
            print(f"  ok  {family}/{toml:<22} {gate:<30} {state}")
    return bad


# The two arms the first version of `GATE_CMDS` omitted, each with a mutation
# that makes it red, and each required to be REPORTED. A declaration gate whose
# table silently stops invoking a gate is indistinguishable from one where that
# gate is green -- which is ruling 9's "fifth unnamed red" in the mechanism
# rather than in a header.
_DECL_PROBES = [
    ("check_extra_claims.py", "runtime", "module_globals", "runtime_module_globals",
     lambda t: re.sub(r"^# EXTRA-OK: '0\.37[^\n]*\n", "", t, count=1, flags=re.M)),
    ("audit-case-migration.py", "runtime", "join", "runtime_join",
     lambda t: t.replace('stdout = "xyz\\nx-y-z\\nx,y,z\\n"',
                         'stdout = "xyQ\\nx-y-z\\nx,y,z\\n"', 1)),
]


def _selftest_declaration_gate():
    """Make the declaration gate go red on purpose, once per newly-added arm."""
    bad = []
    for gate, family, toml, stem, mutate in _DECL_PROBES:
        real = os.path.join(CASES, family, toml + ".toml")
        # NOT a `.toml` suffix: the case runner discovers `cases/**/*.toml`, and
        # `check_fixtures.py --census` globs the same pattern. A probe file
        # leaked by a kill between the write and the `finally` would become a
        # duplicate discovered case rather than an obvious orphan.
        probe = os.path.join(CASES, family, toml + ".declprobe")
        original = open(real).read()
        mutated = mutate(original)
        if mutated == original:
            bad.append(f"declaration-gate probe for {gate} is STALE: its mutation "
                       f"no longer changes {family}/{toml}.toml")
            continue
        try:
            with open(probe, "w") as fh:
                fh.write(mutated)
            found = _evaluate_pair(family, toml, stem, probe, verbose=False)
        finally:
            if os.path.exists(probe):
                os.unlink(probe)
        if not any(gate in f for f in found):
            bad.append(f"declaration-gate probe FAILED: a {family}/{toml} whose "
                       f"{gate} is red was not reported. That gate is not being "
                       f"invoked, or its red is not being read.")
        else:
            print(f"  ok  declaration-gate probe: an undeclared {gate} red is caught")
    return bad


# --------------------------------------------------------------------------
# main
# --------------------------------------------------------------------------

def main(argv):
    if "--list" in argv:
        total = 0
        for family, toml, stem, _s in FILES:
            n = len(X.test_fns(X.source(stem)))
            total += n
            print(f"  cases/{family}/{toml}.toml  <- {stem}.rs  {n} case(s)/trial(s)")
        print(f"  {len(FILES)} file(s), {total} trial(s)")
        return 0

    if "--write" in argv:
        for family, toml, stem, subject in FILES:
            path = os.path.join(CASES, family, toml + ".toml")
            os.makedirs(os.path.dirname(path), exist_ok=True)
            text = rendered(family, toml, stem, subject)
            with open(path, "w") as fh:
                fh.write(text)
            print(f"wrote {os.path.relpath(path, REPO)} ({len(text.splitlines())} lines)")
        return 0

    drift = []
    for family, toml, stem, subject in FILES:
        path = os.path.join(CASES, family, toml + ".toml")
        want = rendered(family, toml, stem, subject)
        have = open(path).read() if os.path.exists(path) else None
        if have != want:
            drift.append(f"{family}/{toml}.toml")
    bad = check_gate_declarations(verbose=not drift)
    if drift or bad:
        print("\nGENERATOR FAILED")
        for d in drift:
            print(f"  DRIFT: {d} is not what this spec renders "
                  f"(run --write to see the diff)")
        for b in bad:
            print(f"  {b}")
        return 1
    n = sum(len(X.test_fns(X.source(s))) for _f, _t, s, _x in FILES)
    print(f"\nGENERATOR FIXED POINT -- {len(FILES)} case file(s), {n} case(s), "
          "reproduced byte-for-byte, and every EXPECTED-RED declaration agrees "
          "with the gate it names")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
