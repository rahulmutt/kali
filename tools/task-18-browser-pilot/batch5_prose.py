"""The fixed prose vocabulary for Task 18 batch 5's 20 migrated case files.

WHY THIS EXISTS. Batch 4 ran four implementers concurrently and shipped
cross-group prose divergence that every per-file check passed individually:
three wordings of the same argv-order fact, two labels for one convention, and
an inconsistent rule-13 discussion. Nothing mechanical could see it, because no
gate reads `#` header prose or `rationale` wording (U8: "rationale prose is
audited by NOTHING").

So batch 5 does not ask four implementers to word the same recurring fact the
same way. The recurring facts live here, once, as functions. A generator that
wants to say "this is a plain `.contains` against a field with a substring
form, so it stays `stdout_contains`" calls `ruling3_substring()`; it does not
retype the sentence. Divergence then requires deleting a call, which is visible
in a diff, rather than paraphrasing a sentence, which is not.

What does NOT belong here: anything file-specific. The program under test, the
invocation arithmetic, the assertion inventory and the `:N` citations are the
per-file spec, and the whole point of review is to read those.

CITATIONS. Every `:N` a caller passes must be derived by SEARCHING the source
for the construct at generation time (`cite_line` below), never by arithmetic
and never carried over from an earlier measurement -- four fix rounds in this
project have gone to stale line citations.
"""

import re

# --- controller ruling 3: mirror the source, one policy, no per-file judgment.


def ruling3_substring(surface="raw stdout", key="stdout_contains"):
    """A plain `.contains` against a field that HAS a substring form."""
    return (
        f"The source spells this as a plain `.contains(...)` against {surface}, so it is "
        f"carried as {key} and NOT strengthened to an exact pin -- controller ruling 3, "
        "mirror the source: a plain `.contains` against a field that HAS a substring form "
        "keeps the substring form even though the exact output was observed."
    )


def ruling3_json_leaf():
    """A plain `.contains` against `json["stdout"]`, which has no substring form."""
    return (
        "On the JSON branch the same claim is taken against the string leaf "
        'json["stdout"], which has NO substring form in the case format (there is no '
        "json_contains key), so per controller ruling 3 it becomes an exact `json.stdout` "
        "pin -- and, per U9, only after the value was captured from the real kali binary "
        "rather than hand-computed."
    )


def ruling3_count(needle_desc, bound, key="stdout_count"):
    """A `.matches(<literal>).count() >= N` claim."""
    return (
        f"The source spells this as `.matches({needle_desc}).count() >= {bound}`, carried "
        f"verbatim as `{key}` with `at_least = {bound}` per controller ruling 3 (mirror the "
        "source): NOT weakened to a plain contains, which a single occurrence would satisfy, "
        "and NOT strengthened to `exact`, which the source never says. Counting is "
        "non-overlapping and left-to-right, as Rust's `str::matches` is."
    )


def ruling3_count_exact(needle_desc, bound, key="stdout_count"):
    """A `.matches(<literal>).count() == N` claim -- the EXACT bound.

    Added in batch 6A fix round 1 (finding I2). `ruling3_count` above is the
    `>=` sentence, and it ends "NOT strengthened to `exact`, which the source
    never says" -- correct for all eleven files that carry it, and the exact
    OPPOSITE of the truth for a source spelling `assert_eq!(...count(), N)`.
    Batch 6A shipped 16 rationales that said it above an `exact = 6` key,
    because the shared sentence was patched with two `str.replace` calls that
    fixed its first half and missed its second. Hoisted on first use, as this
    module's own note asks, rather than patched again.

    EVERY NUMBER IN THE SENTENCE IS DERIVED FROM `bound` (fix round 2, N3). The
    first version wrote "a seventh time" while parameterising `bound` -- correct
    for the only call site that exists, and a sentence contradicting its own
    number for the next `==` site with any other bound. That is exactly the trap
    `ruling3_count` set for batch 6A, one level down, and it is not left in
    place for the same reason.
    """
    return (
        f"The source spells this as `.matches({needle_desc}).count() == {bound}` -- an "
        f"exact assertion, not a lower bound -- so it is carried as `{key}` with "
        f"`exact = {bound}` per controller ruling 3 (mirror the source): NOT weakened to "
        f"`at_least = {bound}`, which output carrying the value MORE than {bound} times "
        "would still satisfy, and NOT weakened to a plain contains, which a single "
        "occurrence would satisfy. Counting is non-overlapping and left-to-right, as "
        "Rust's `str::matches` is."
    )


def extra_ok(value, why):
    """The `# EXTRA-OK: <repr> -- <why>` declaration `check_extra_claims.py` reads."""
    return f"EXTRA-OK: {value!r} -- {why}"


EXTRA_CLAIM_PREAMBLE = [
    "EXTRA-CLAIM DECLARATIONS (U14's `extra` direction).",
    "check_extra_claims.py compares this file's claim strings against the source's and fails",
    "on any that appear nowhere in the .rs. The entries below are the deliberate exceptions;",
    "a genuinely new one will not be on this list and will fail the gate.",
]
# Added mid-batch, after group B's report pointed out that only the individual
# `EXTRA-OK:` lines were shared and the four-line preamble above was being
# retyped per group. Two groups had already produced the same sentences wrapped
# at different columns -- harmless in itself, and exactly the divergence class
# this module exists to make impossible. Carry it into batches 6-8.

EXTRA_OK_JSON_STDOUT = (
    "live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, "
    "which has no substring form, so ruling 3 requires an exact pin captured from the "
    "real binary"
)

EXTRA_OK_U5_RENAME = (
    "U5-renamed `[source]` entry filename; passed to `kali` on argv only and referenced by "
    "no fixture body (checked mechanically in this file's generator), so the rename cannot "
    "rewrite the program under test"
)
# The second recurring EXTRA-OK reason in this batch, and it was NOT here to begin
# with: three groups independently wrote three wordings of this one fact before it
# was hoisted. That is precisely the batch-4 divergence this module exists to
# prevent, and it recurred because the module supplied only the JSON-pin reason.
# When a third reason appears in batches 6-8, hoist it on first use, not on second.


# --- rule 13 -------------------------------------------------------------


def rule13_carried(docs):
    """Doc comments on library helpers whose OUTPUT the migrated case reproduces.

    `docs` is the list of `///` lines, verbatim, in call-chain order.
    """
    quoted = ", ".join(f'"{d}"' for d in docs[:-1])
    tail = f'"{docs[-1]}"'
    listed = f"{quoted} and {tail}" if len(docs) > 1 else tail
    return (
        "RULE 13 -- doc comments on the library helpers that produce this fixture, "
        f"carried verbatim: {listed}"
    )


def rule13_header(chain_fns, docs_carried=(), extra=None):
    """The file-header rule-13 accounting block.

    `chain_fns`: every fn in the migrated call chain that carries NO `///` doc.
    `docs_carried`: the `///` texts that ARE carried (into every rationale the
                    producing helper reaches, per U6 -- not pooled here).
    """
    lines = ["RULE 13 -- transitive helper docs. Checked every fn in each call chain:"]
    lines += _wrap_list(chain_fns, "-- none carries a `///` doc comment.")
    if docs_carried:
        lines.append(
            "The chain DOES reach `///`-documented helpers in `kali_common` whose OUTPUT this"
        )
        lines.append(
            "case file reproduces in [source], so their docs are claim prose and are carried"
        )
        lines.append(
            "into the rationale of every case that helper's text reaches (ruling 6's test:"
        )
        lines.append(
            'carry the docs if the migrated case still depends on what the helper computed).'
        )
    lines += RULE13_RUNNER_EXEMPTION
    if extra:
        lines += extra
    return lines


RULE13_RUNNER_EXEMPTION = [
    "The chain also reaches `kali_runtime_contract::browser_bundle_harness_script` and",
    "`::browser_harness_command_parts_for`, which DO carry one-line `///` docs. Those are",
    "NOT carried: in the migrated form this case file never calls them -- the",
    "`browser_bundle_harness` step kind means the case RUNNER does",
    "(crates/kali_case_runner/src/steps.rs) -- so their docs describe shared runner",
    "infrastructure documented in design spec 5.3, not what this case claims. That is",
    "controller ruling 6, which writes the exemption into the rule rather than leaving it",
    "to implementer discretion.",
]


# --- rule 7 / U1: matrix arithmetic --------------------------------------


def matrix_arithmetic(*, test_fns, invocations, helpers, cases, axis, values,
                      non_axes=("command", "json_output")):
    """The header block for a file that DOES declare a `[matrix]`.

    helpers: list of (fn_name, count, explanation) -- explanation says how that
             helper's invocation count decomposes.
    """
    product = len(values)
    if cases * product != invocations:
        raise AssertionError(
            f"matrix arithmetic does not close: {cases} cases x {product} = "
            f"{cases * product}, but the source makes {invocations} invocations"
        )
    lines = [
        "RULE 7 / U1 -- MATRIX ARITHMETIC, and it closes exactly.",
        f"{test_fns} `#[test]` fns, {invocations} real helper invocations with every loop",
        "expanded:",
    ]
    for name, count, why in helpers:
        lines.append(f"  * `{name}` -- {count} invocation(s) = {why}")
    lines += [
        f"`{axis}` is the ONE axis every case varies over uniformly, and every helper covers",
        f"all {product} of its values, so a file-wide axis fans nothing the source never ran:",
        f"{cases} `[[case]]` entries x {axis}({product}) = {invocations} trials = "
        f"{invocations} invocations. Exact.",
    ]
    lines += matrix_not_axes(non_axes)
    return lines


def matrix_not_axes(non_axes):
    """Why the file's non-substituting dimensions are sibling cases, not axes.

    `non_axes` names only the dimensions the file ACTUALLY varies. It is a
    parameter and not a constant because the constant it replaced named
    `command` unconditionally, and four files in this batch only ever issue
    `build` -- so their headers explained why a dimension they do not have is
    not an axis. Vacuous, not false, but it is the same "prose describing a
    state the file does not have" failure class, so it is fixed rather than
    tolerated.
    """
    if not non_axes:
        raise AssertionError("a file with no non-axis dimension states nothing here")
    named = " and ".join(f"`{n}`" for n in non_axes)
    one = len(non_axes) == 1
    head = f"{named} is NOT a matrix axis" if one else f"{named} are NOT matrix axes"
    lines = [
        f"{head}, per rule 7 and design spec 5.6's own note:",
        f"{'it changes' if one else 'each changes'} the ASSERTION SHAPE rather than "
        "substituting a string.",
    ]
    if "json_output" in non_axes:
        lines.append(
            "`json_output` switches between a text claim and a JSON-envelope claim.")
    if "command" in non_axes:
        lines += [
            "`command` switches the envelope's payload between `exitCode` (run) and",
            "`total`/`passed`/`failed` (test).",
        ]
    lines.append(
        f"{'It is' if one else 'Each is'} written as sibling `[[case]]` entries instead.")
    return lines


# Kept as a name for callers that already imported it; it is the both-dimensions
# rendering, which is what most files in this batch need.
MATRIX_NOT_AXES = matrix_not_axes(("command", "json_output"))


def matrix_declined(*, test_fns, invocations, cases, reason):
    """The header block for a file that declines `[matrix]`, and why."""
    if cases != invocations:
        raise AssertionError(
            f"a matrix-free file writes one case per invocation: {cases} cases vs "
            f"{invocations} invocations"
        )
    return [
        "RULE 7 / U1 -- MATRIX DECLINED, and why.",
        f"{test_fns} `#[test]` fns, {invocations} real helper invocations with every loop",
        "expanded.",
    ] + reason + [
        "Per U1 a `[matrix]` axis is FILE-WIDE -- `expand()` fans EVERY `[[case]]` by the full",
        "cross-product whether or not that case references the axis, and there is no per-case",
        "opt-out -- so an axis that any one case does not vary over would manufacture trials",
        "the source never ran (also a rule-2 invention). Rule 7's own remedy applies: drop",
        "`[matrix]` for the whole file and write named siblings.",
        f"{invocations} invocations -> {cases} named `[[case]]` entries, no `[matrix]`.",
    ]


# --- rule 6 --------------------------------------------------------------


def rule6_matrix_fold(per_case_desc):
    """State the rule-7 matrix fold explicitly, as rule 6 requires."""
    return [
        "RULE 6 -- 1:1 MAPPING, and the fold is stated here as the rule requires.",
        f"Each `[[case]]` below corresponds to {per_case_desc}; the assertion mapping stays",
        "1:1 per trial, so no claim is lost, but a failing trial's id reads its matrix cell",
        "rather than a source fn name.",
        # The cell is described, not exemplified. This sentence used to end
        # "reads `[ext=jsx]`", which is false for any file whose axis does not
        # carry that value -- one file in this batch has a two-value js/ts axis
        # and had to carry a correcting sentence right after the shared block.
        # Prose describing a state the file does not have is the recurring
        # failure class here; the fix is to stop naming the state.
    ]


RULE6_ONE_TO_ONE = [
    "RULE 6 -- 1:1 MAPPING. With no `[matrix]` there is no fold: every `[[case]]` below is",
    "exactly one real helper invocation of the source, named after the `#[test]` fn that made",
    "it. Two source fns are never folded into one case even when their invocations are",
    "literally identical, because the case is the only remaining trace of the fn.",
]


# --- U2 ------------------------------------------------------------------


def u2_source_file_wide(fixtures, *, entry_named_on_argv=True):
    """`[source]` is file-wide; say why that is safe for this file."""
    lines = [
        "U2 -- `[source]` is FILE-WIDE, and that is safe here. Every fixture",
        "(" + ", ".join(f"`{f}`" for f in fixtures) + ")",
        "is written unconditionally by the source into a fresh temp dir: no fixture is written",
        "behind an `if`, and no case's point is the presence or absence of a file.",
    ]
    if entry_named_on_argv:
        lines.append(
            "Every command below names its entry explicitly on argv, so the unused siblings in"
        )
        lines.append("a trial dir are inert.")
    return lines


# --- U5 ------------------------------------------------------------------


def u5_renames(renames):
    """renames: list of (original filename, new [source] key, why)."""
    lines = [
        "U5 -- `[source]` KEY RENAMES. `[source]` is one flat file-wide namespace, and this",
        "source writes two different program texts to the same filename in different tests, so",
        "the keys are variant-suffixed:",
    ]
    for original, new, why in renames:
        lines.append(f"  * `{original}` -> `{new}` -- {why}")
    lines += [
        "U5's safety condition holds for every rename: each of these filenames is passed to",
        "`kali` as a CLI argument only. None is referenced by string from inside any fixture",
        "body -- no dynamic-import call and no CommonJS require call names any of them, checked",
        "against every `[source]` value in this file -- so renaming does not rewrite the program",
        "under test (rule 9).",
        # The two JS call forms are spelled out rather than backticked. U8's checker
        # (`check_rationale_fn_names.py`) treats every backticked lower-case identifier as a
        # cited fn and resolves it against the source `.rs`, where neither exists, so the
        # obvious wording turned the U8 gate red on every file that renamed a `[source]` key.
        # Caught by running the gate, not by reading the prose.
    ]
    return lines


# --- ruling 7 ------------------------------------------------------------


RULING7_NO_HOIST = [
    "RULING 7 -- DUPLICATE `[source]` BODIES ARE NOT HOISTED. Several entries below hold",
    "byte-identical program text. U13 would hoist them into `[constants]`; controller ruling 7",
    "declines that for `browser/` (a hoisted body makes `check_fixtures.py` go red on a correct",
    "file, and hoisting moves program text onto the surface `assertion_strings()` searches).",
    "The mandatory half of that ruling is honoured instead: the duplication is asserted",
    "MECHANICALLY in this file's generator, not eyeballed -- the generator compares the",
    "extracted fixture strings and raises if they are not byte-identical.",
]


# --- ruling 8 ------------------------------------------------------------


def migration_note_stale_fn_name(fn_name, discrepancy):
    """Ruling 8: a source fn whose name misdescribes its own body."""
    return (
        f"MIGRATION NOTE (controller ruling 8): the source fn `{fn_name}` has a name that "
        f"misdescribes its own body -- {discrepancy} The source is NOT corrected: a fn name is "
        "not a comment so U7 does not literally apply, the `.rs` files are deleted wholesale "
        "after batch 8, and editing a source invalidates every audit run against its pre-trim "
        "blob. The discrepancy is recorded here so the case file preserves what the source "
        "actually did."
    )


# --- U4 partial-retention pointer ---------------------------------------


def partial_retention_note(*, stem, retained_fn, migrated, total, blocking):
    """The header block a case file carries when its `.rs` was trimmed."""
    return [
        f"PARTIAL MIGRATION (U4 trim-and-keep) -- {migrated} of the source's {total} `#[test]`",
        f"fns are migrated here. The remaining one, `{retained_fn}`, is a FIXTURE",
        f"SELF-INSPECTION test: {blocking} It builds no command, runs no binary, and asserts",
        "nothing about behaviour -- it checks the fixture's own text. That claim has no",
        "expressible form in the case format (no step kind asserts on `[source]` text;",
        "`[source]` is program text by construction, not a claim), and it is invisible to",
        "`audit-case-migration.py`, whose `.contains()` extractor cannot tell a fixture-text",
        "read from an output assertion and which excludes everything under `[source]` from its",
        "search by design. Migrating it would produce a false green. It is escalated per rule",
        "3/4 and RETAINED hand-written; controller ruling 4 is explicit that the audit script",
        "is NOT extended for this shape.",
        "",
        "THE `.rs` HAS SINCE BEEN TRIMMED to exactly that test plus the fixture builders it",
        "reads, carrying a `//!` retention header. TWO CONSEQUENCES A LATER READER MUST NOT",
        "MISREAD:",
        "  * EVERY `:N` LINE CITATION IN THIS FILE IS A PRE-TRIM LINE NUMBER. Audit and diff",
        "    this pair against the pre-trim source from git history, not against the working",
        "    tree.",
        "  * THE POST-TRIM PAIR IS THE WRONG COMPARISON FOR EVERY GATE, not just the audit. The",
        "    retained `.rs` carries the COMPLETE measured red-list (ruling 9) -- which gates go",
        "    red post-trim and which are green. Read it there, so there is one source of truth.",
    ]


# --- argv order ----------------------------------------------------------


ARGV_ORDER = [
    "ARGV ORDER is transcribed in the exact order the source's `Command` builder appends it,",
    "which differs between the two helper shapes and is not normalised here:",
    '  * build:   `build --bundle --api browser [--output json] <entry>` -- the `--output json`',
    "             pair is appended AFTER the subcommand and its flags.",
    '  * run/test: `[--output json] <run|test> --api browser [--max-threads 0',
    "             --max-spawned-processes 0] <entry>` -- the `--output json` pair is appended",
    "             BEFORE the subcommand.",
    "The source passes an absolute `dir.path().join(filename)` as the entry; the case runner",
    "passes the bare filename relative to the trial dir, matching every previously shipped",
    "`browser/` case file.",
]


# --- rule 12 -------------------------------------------------------------
# `math_shapes.rule12_no_comments_prose(rs_path, stem)` already derives this
# from the source and raises if the source does carry Rust comments. Batch 5
# re-exports it here so a generator has one import for prose.

from math_shapes import rule12_no_comments_prose  # noqa: E402,F401


# --- citation derivation -------------------------------------------------


def cite_line(rs_text, pattern, *, label=None, expect=1):
    """1-based line number(s) of `pattern` (a regex) in `rs_text`.

    THE ONLY sanctioned way to produce a `:N` citation in batch 5. Deriving a
    citation by arithmetic, or carrying one over from a pre-edit measurement,
    is what four fix rounds in this project were spent on. Raises unless the
    pattern matches exactly `expect` times, so an ambiguous or vanished anchor
    is a generator error rather than a silently wrong number.
    """
    hits = [i + 1 for i, line in enumerate(rs_text.split("\n"))
            if re.search(pattern, line)]
    if len(hits) != expect:
        raise AssertionError(
            f"citation anchor {label or pattern!r}: {len(hits)} match(es) "
            f"{hits}, wanted {expect}"
        )
    return hits[0] if expect == 1 else hits


def cite_range(rs_text, first_pattern, last_pattern, *, label=None):
    """`:A-B` for a claim that spans lines, both ends derived by search."""
    a = cite_line(rs_text, first_pattern, label=label)
    b = cite_line(rs_text, last_pattern, label=label)
    return f":{a}-{b}"


def assert_identical(label, *values):
    """Ruling 7's mandatory mechanical duplicate-identity assertion."""
    first = values[0]
    for i, v in enumerate(values[1:], start=1):
        if v != first:
            raise AssertionError(
                f"{label}: value {i} is not byte-identical to value 0 "
                f"({v[:60]!r} vs {first[:60]!r})"
            )
    return first


def _wrap_list(names, tail):
    """Render a long fn-name list as `#`-header lines without over-long rows."""
    out, row = [], "  "
    for n in names:
        piece = f"`{n}`, "
        if len(row) + len(piece) > 86:
            out.append(row.rstrip())
            row = "  "
        row += piece
    row = row.rstrip().rstrip(",")
    out.append(f"{row} {tail}")
    return out
