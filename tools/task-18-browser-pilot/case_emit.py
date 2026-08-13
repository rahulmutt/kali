"""Generic case-file emitter for Task 18 browser/ batches (added in batch 4).

Why this exists: the pilot's per-file `gen_*.py` scripts were deleted because
they hardcoded scratchpad paths and uncommitted intermediates (see README).
Batch 4 needed to emit 22 files, so the *shape-independent* half of that job
is factored here: TOML rendering, deterministic key order, and the discipline
that every `[source]` body and every fixture literal is pulled through
`lexer.py` from the real `.rs` rather than retyped (rule 9).

This module renders; it decides nothing. The per-file mapping (rule 5 split vs
rule 6 1:1 vs rule 7 matrix), the assertion set, and the prose all live in the
caller's spec, which is what review needs to read.

Step keys are emitted in a fixed order so a regenerated file diffs cleanly:
kind, entry, path, args, env, body, fields, exit, then the assertion keys in
the order §5.4 lists them.
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from toml_emit import toml_string, toml_str_array  # noqa: E402
from lexer import string_literals_in_range, find_string_literals  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")

# THE COMMIT BATCH 8C's FAMILY DELETION REMOVED THE SOURCES FROM.
#
# `citation_sweep.sh` gave a case file a `SOURCE REF:` so its `:N` citations
# still resolve once the `.rs` is gone. The GENERATORS need the same thing and
# for a stronger reason: `source_text` raises rather than guessing when a source
# is missing, so without this every generator that reads a deleted source would
# crash, and `classify_drift.py` counts a crashing generator as a gate failure,
# not a skip. Six of the sixteen read at least one source 8C deletes.
#
# This names the deletion commit's PARENT -- the last commit where all 139
# `browser_*.rs` are still present -- which is the same rule `_ref_carries`
# enforces for a case file's own declaration.
FAMILY_DELETION_REF = "28df9ba02962143a1fc1735e418b2f67caee8fc6"

_BLOB_CACHE = {}


def _blob_at(ref, name, *, missing_ok=False):
    """`crates/kali_cli/tests/<name>` at `<ref>`, or a hard stop.

    A hard stop rather than an empty string on purpose: `citation_tiers`
    learned this the expensive way (its N5 fix round), where an unreadable ref
    was written into the blob unchecked and the instrument carried on printing
    silently wrong figures. A generator that cannot read its source must not
    emit a case file built from nothing.

    THE TWO FAILURES HAVE DIFFERENT REMEDIES AND MUST NOT SHARE A MESSAGE --
    the same split `citation_tiers._ref_carries` records. An UNREACHABLE ref is
    a shallow clone and the fix is to fetch history; a ref that resolves but
    does not carry the path means the file was already gone at that commit,
    which for this family means an EARLIER batch deleted it, and telling that
    caller to re-fetch sends it to re-derive a ref that was right all along.
    `missing_ok` returns None for the second case so callers can act on it.
    """
    import subprocess
    key = (ref, name)
    if key not in _BLOB_CACHE:
        if subprocess.run(["git", "rev-parse", "-q", "--verify", f"{ref}^{{commit}}"],
                          cwd=REPO, capture_output=True).returncode:
            raise AssertionError(
                f"`{ref}` is not reachable in this repository. This instrument "
                "needs FULL history: in CI, actions/checkout must be given "
                "`fetch-depth: 0` (a default checkout is shallow and cannot "
                "resolve it); locally, git fetch --unshallow.")
        got = subprocess.run(
            ["git", "cat-file", "blob", f"{ref}:crates/kali_cli/tests/{name}"],
            cwd=REPO, capture_output=True, text=True)
        _BLOB_CACHE[key] = got.stdout if got.returncode == 0 else None
    blob = _BLOB_CACHE[key]
    if blob is None and not missing_ok:
        raise AssertionError(
            f"{ref} resolves but does not contain crates/kali_cli/tests/{name}. "
            "The ref must name a commit where the source still EXISTS -- a "
            "deletion commit's PARENT, not the deletion commit.")
    return blob


def source_bytes(name, *, toml_text=None):
    """`crates/kali_cli/tests/<name>`, from the working tree or from history.

    ONE resolver for "which bytes is this `.rs`", reachable from both the
    generation path (`source_text`) and the reword path
    (`reword_ungated_citations._pretrim_lines`). Those two already had a
    near-miss of exactly this kind -- see `source_text_at`'s docstring, where
    two resolvers keyed on different lines and each looked locally correct --
    so the second one delegates here rather than growing its own git call.

    A case file's OWN `SOURCE REF:` wins when it has one, because sources
    deleted by earlier batches carry different refs (two distinct shas are in
    the tree already) and the constant would be the wrong answer for them.
    """
    p = os.path.join(TESTS, name)
    if os.path.exists(p):
        return open(p).read()
    declared = None
    if toml_text is not None:
        m = re.search(r"SOURCE REF:\s*([0-9a-f]{40})", toml_text)
        declared = m.group(1) if m else None
    return _blob_at(declared or FAMILY_DELETION_REF, name)


def deleted_by_family_deletion(name):
    """Was `browser_<...>.rs` removed by batch 8C's family deletion?

    DERIVED, NOT LISTED (ruling 18 #1). At `FAMILY_DELETION_REF` every retained
    target carries a `//!` header -- U3 requires one of every retention, and 8C's
    first commit added the last missing one to
    `browser_harness_failing_test_propagates_failure.rs` -- while every migrated
    target has none. So "headerless at the ref" IS the delete set, and it stays
    the delete set forever because the ref is immutable. No manifest to fall out
    of date, and the answer does not change when the working tree does.

    8C verified this against the independent three-fact classifier (`//!`
    header x same-stem case file x `Migrated from` claimant) over all 139
    sources at that commit: both sides returned the same 118 names, with an
    empty symmetric difference.

    A source ABSENT at the ref was deleted by an EARLIER batch (the pilot and
    batch 2 removed 23 such sources, whose case files carry their own, different
    refs). That is not 8C's deletion, so the answer is False and their
    declarations are left exactly as they are.
    """
    blob = _blob_at(FAMILY_DELETION_REF, name, missing_ok=True)
    if blob is None:
        return False
    return not blob.startswith("//!")


def source_text(name, *, quiet=False):
    """The source a case file is generated FROM -- not always the working tree.

    THREE generators crashed outright because they read the working-tree `.rs`
    directly: once a U4 trim-and-keep retention lands, the working tree holds
    only the RETAINED half, so the migrated fn the generator extracts its
    fixture from no longer exists and `fixture_in_fn` raises `no fn ... in
    source`. `gen_batch5_group_a/b/c` all died that way, on
    `math_pow_bracketed_frozen_wrapper`, `math_pow_bracketed_frozen_wrapper_
    harness` and `math_max_min_frozen_aliases` respectively.

    The rule itself is settled (ruling 9): a trimmed source's case file is
    numbered and extracted against the PRE-TRIM blob, and the ref comes from the
    retained file's own `PRE-TRIM REF:` line rather than from a constant in the
    generator -- a ref carried anywhere but the header is the moving figure
    ruling 11 forbids.

    It is NOT implemented only here. Other live readers of the same header line,
    enumerated rather than remembered:

        $ grep -rn 'PRE-TRIM REF:' tools/task-18-browser-pilot/ | grep -v '^\\S*:\\s*#' \\
              | grep -vE '(\\.md|classify_drift)'

    -- `citation_sweep.sh`, `citation_tiers.py` (twice), `gen_batch6a.rs`, this
    function, and (until this round) `reword_ungated_citations._pretrim_lines`,
    plus `gen_batch4_group_b`'s hardcoded `PRETRIM_REF` constant. What this
    function does is make ONE of them serve the whole GENERATION path:
    `_pretrim_lines` now delegates here, because `case_emit.write` puts it and
    this function inside the same generator invocation, where two resolvers that
    disagree would silently produce two different answers about the same file.
    Nothing was deleted from the sweep-side readers; consolidating those is a
    separate question about their contract.

    A `//!` header with NO `PRE-TRIM REF:` is a whole-file retention, not a
    trim: nothing was removed, so the working tree IS the source. A missing
    `.rs` raises rather than returning empty -- a generator whose source has
    been deleted must be told so, not silently emit a case file with no
    fixtures.
    """
    path = os.path.join(TESTS, f"browser_{name}.rs")
    if not os.path.exists(path):
        # BATCH 8C: the family deletion is what makes this arm reachable, and
        # it is now told where to look instead of refusing. The guard below is
        # the "it will not guess" half, kept: a source absent from the tree AND
        # carrying a `//!` header at the ref was a RETENTION, so its absence is
        # not the family deletion and this function has no business inventing a
        # source for it.
        if not deleted_by_family_deletion(f"browser_{name}.rs"):
            raise AssertionError(
                f"browser_{name}.rs is absent from the tree but carries a `//!` "
                f"header at {FAMILY_DELETION_REF}, so it was RETAINED there and "
                "the family deletion is not why it is missing. Refusing to "
                "guess which blob this generator meant.")
        if not quiet:
            print(f"    reading browser_{name}.rs at the family-deletion ref "
                  f"{FAMILY_DELETION_REF}")
        return _blob_at(FAMILY_DELETION_REF, f"browser_{name}.rs")
    return source_text_at(path, quiet=quiet)


def source_text_at(path, *, quiet=False):
    """`source_text`, by path -- for a `#[path]` submodule carrier or any other
    caller that already holds one. Same rule, one implementation.

    THE `//!` AND THE `PRE-TRIM REF:` MUST AGREE, AND A NON-MATCH IS AN ERROR.
    This function keyed on the `//!` header and `_pretrim_lines` keyed on the
    ref line alone; both now run inside one generator invocation, so on a file
    that carried a ref without a header they would have returned two different
    sources for the same `.rs` -- and each would have looked locally correct.
    Ruling 18: a non-match is raised rather than resolved, so a file that grows
    a ref without a U3 header fails loudly instead of being read two ways.
    All ten such files carry both today:

        $ for f in $(grep -rl 'PRE-TRIM REF:' crates/kali_cli/tests --include=*.rs); do
              head -c3 "$f" | grep -q '^//!' || echo "NOT //!: $f"; done
        (prints nothing)
    """
    text = open(path).read()
    m = re.search(r"PRE-TRIM REF:\s*(\S+)", text)
    if not text.startswith("//!"):
        if m:
            raise AssertionError(
                f"{path} declares PRE-TRIM REF {m.group(1)} but has no `//!` header. "
                "A trim retention carries both (U3); one without the other means two "
                "readers of this line disagree about which source this file is, and "
                "which one you get depends on which reader ran.")
        return text
    if not m:
        return text
    import subprocess
    ref = m.group(1)
    rel = os.path.relpath(os.path.abspath(path), REPO)
    if not quiet:
        print(f"    reading {rel} at its own PRE-TRIM REF {ref}")
    blob = subprocess.run(["git", "show", f"{ref}:{rel}"],
                          cwd=REPO, capture_output=True, text=True)
    if blob.returncode != 0:
        raise AssertionError(
            f"{rel} declares PRE-TRIM REF {ref} but `git show` cannot read it: "
            f"{blob.stderr.strip()}")
    return blob.stdout


def cargo_target_dir():
    """Cargo's REAL target directory, asked of cargo rather than assumed.

    `gen_batch5_group_d` computed it as `<its own repo root>/.cache/cargo-target`
    and aborted with `no built libkali_common rlib under ...` from anywhere that
    is not /workspace -- an undeclared build precondition (batch 7A gap 4). The
    cause is that `~/.cargo/config.toml` pins `build.target-dir` to an ABSOLUTE
    path, so every worktree and scratchpad shares one target dir and a
    repo-relative guess is wrong everywhere but the one checkout it was written
    in. In a fresh worktree the generator therefore died, and a census run there
    silently came out short.

    `cargo metadata --no-deps` honours the config, needs no build (~30ms), and
    is the authority. If cargo cannot be asked, this raises with the reason
    rather than falling back to a guess that reintroduces the same bug.
    """
    import json
    import subprocess
    p = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=REPO, capture_output=True, text=True)
    if p.returncode != 0:
        raise AssertionError(
            "cannot resolve cargo's target directory: `cargo metadata` exited "
            f"{p.returncode}. This tooling will not guess a repo-relative path -- "
            "`~/.cargo/config.toml` pins build.target-dir absolutely, so a guess "
            f"is wrong outside the pinned checkout. stderr: {p.stderr.strip()[-300:]}")
    return json.loads(p.stdout)["target_directory"]


def require_debug_artifact(relpath, *, why):
    """An absolute path under cargo's debug profile, or a loud, specific abort.

    `why` names the build that produces it, so the failure states the
    precondition instead of the symptom.
    """
    path = os.path.join(cargo_target_dir(), "debug", relpath)
    if not os.path.exists(os.path.dirname(path)):
        raise AssertionError(
            f"{os.path.dirname(path)} does not exist -- nothing has been built into "
            f"cargo's target dir ({cargo_target_dir()}). Precondition: {why}")
    return path


def fixture(rs_text, first_line, last_line, index=0):
    """The decoded value of a string literal in the source, by line range.

    Never retype a fixture: this is the only sanctioned way to get one into a
    case file (rule 9). `index` picks among several literals opening in range.
    """
    lits = string_literals_in_range(rs_text, first_line, last_line)
    if not lits:
        raise AssertionError(
            f"no string literal opens in lines {first_line}-{last_line}"
        )
    if index >= len(lits):
        raise AssertionError(
            f"only {len(lits)} literal(s) open in lines {first_line}-{last_line}, "
            f"wanted index {index}"
        )
    return lits[index]


def fixture_in_fn(rs_text, fn_name, index=0):
    """The index-th string literal inside `fn <fn_name>`'s body.

    Prefer this over `fixture()`. Line ranges are NOT stable across a migration:
    inserting or deleting a `//!` retention header shifts every line below it,
    after which a hardcoded range silently extracts the WRONG literal and the
    generated case file still parses. That happened on
    browser_math_asinh_acosh_atanh_identities.rs in this batch -- deleting its
    85-line header made `[source]` come out as
    `"app.${ext}" = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"`. Anchoring on the fn
    name is immune to that, and it is also what a reader can check by eye.
    """
    marker = re.search(r"\bfn\s+" + re.escape(fn_name) + r"\s*[(<]", rs_text)
    if not marker:
        raise AssertionError(f"no `fn {fn_name}` in source")
    brace = rs_text.find("{", marker.end() - 1)
    if brace == -1:
        raise AssertionError(f"no body brace for `fn {fn_name}`")
    depth, i, n = 0, brace, len(rs_text)
    while i < n:
        if rs_text[i] == "{":
            depth += 1
        elif rs_text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    body = rs_text[brace:i + 1]
    lits = [x["value"] for x in find_string_literals(body)]
    if index >= len(lits):
        raise AssertionError(
            f"`fn {fn_name}` has {len(lits)} string literal(s), wanted index {index}")
    return lits[index]


def fixture_starting(rs_text, fn_name, prefix):
    """The one string literal inside `fn <fn_name>` whose value starts with
    `prefix`. Content-anchored, so it survives line shifts AND does not depend
    on counting past every `.arg("...")` and `.expect("...")` literal in the
    body. Fails if the prefix matches zero or more than one literal -- an
    ambiguous match is a silent wrong-fixture bug otherwise.
    """
    marker = re.search(r"\bfn\s+" + re.escape(fn_name) + r"\s*[(<]", rs_text)
    if not marker:
        raise AssertionError(f"no `fn {fn_name}` in source")
    brace = rs_text.find("{", marker.end() - 1)
    depth, i, n = 0, brace, len(rs_text)
    while i < n:
        if rs_text[i] == "{":
            depth += 1
        elif rs_text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    hits = [x["value"] for x in find_string_literals(rs_text[brace:i + 1])
            if x["value"].startswith(prefix)]
    if len(hits) != 1:
        raise AssertionError(
            f"`fn {fn_name}`: {len(hits)} literal(s) start with {prefix!r}, wanted exactly 1")
    return hits[0]


def _toml_scalar(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, str):
        return toml_string(v, multiline=False)
    if isinstance(v, list):
        return "[" + ", ".join(_toml_scalar(x) for x in v) + "]"
    if isinstance(v, dict):
        return "{ " + ", ".join(f"{_key(k)} = {_toml_scalar(x)}" for k, x in v.items()) + " }"
    raise TypeError(f"unsupported TOML value: {v!r}")


def _key(k):
    """Bare key where TOML allows it, quoted otherwise."""
    ok = k and all(c.isalnum() or c in "_-" for c in k)
    return k if ok else toml_string(k, multiline=False)


# Emission order for a step's keys. Assertion keys follow design spec 5.4's
# own listing order so a reader can diff a step against the spec table.
_STEP_ORDER = [
    "kind", "entry", "path", "args", "env", "body", "fields",
    "exit",
    "stdout", "stdout_contains", "stdout_absent", "stdout_count",
    "stderr", "stderr_contains", "stderr_absent",
    "json", "json_paths", "json_null", "json_count",
]


def _render_step(step, prefix):
    out = []
    unknown = [k for k in step if k not in _STEP_ORDER]
    if unknown:
        raise AssertionError(f"unknown step key(s) {unknown} -- typo, or a new 5.4 key")
    if "json" in step and "json_paths" in step:
        raise AssertionError("a step declares both `json` and `json_paths`")
    for key in _STEP_ORDER:
        if key not in step:
            continue
        v = step[key]
        if key == "json_paths":
            # The SAME §5.4 `json` key, rendered one dotted path per line
            # instead of as one inline table. Not a new assertion: TOML parses
            # `json.errors.0.code = "E5506"` into exactly the nested table the
            # inline form produces, so the runner, `audit-case-migration.py`
            # and `check_extra_claims.py` all see an identical document. It
            # exists because a deep, long-valued path (a pinned diagnostic
            # `message` is ~230 characters) is unreadable inside an inline
            # table, and `cases/array/concat_static.toml` already spells this
            # shape by hand. Added in batch 6B; nothing that does not ask for
            # it renders differently.
            for path, val in v.items():
                out.append(f"json.{path} = {_toml_scalar(val)}")
            continue
        if key == "body":
            out.append(f"{key} = {toml_string(v)}")
        elif key in ("stdout", "stderr") and isinstance(v, str):
            out.append(f"{key} = {toml_string(v, multiline=False)}")
        elif key in ("stdout_contains", "stdout_absent", "stderr_contains",
                     "stderr_absent", "json_null"):
            out.append(f"{key} = {toml_str_array(v)}")
        elif key in ("stdout_count", "json_count"):
            items = ", ".join(_toml_scalar(c) for c in v)
            out.append(f"{key} = [{items}]")
        else:
            out.append(f"{key} = {_toml_scalar(v)}")
    return out


def emit(header_lines, matrix, source, cases):
    """Render a whole case file.

    header_lines: list[str], rendered as `# ` comment lines (rule 12 prose that
                  is file-wide, plus the matrix arithmetic per rule 7).
    matrix:       dict[axis] = list[str], or None/{} for no [matrix].
    source:       dict[filename] = body. Emitted in insertion order.
    cases:        list of {name, rationale, steps: [step, ...]}. A single-step
                  case is emitted inline on [[case]] per 5.2.
    """
    out = []
    for line in header_lines:
        # Split on embedded newlines rather than prefixing once. A caller that
        # builds a header entry with an f-string spanning several lines used to
        # get ONE `# ` and the rest of its text bare, which is not a comment --
        # `tomllib` then rejects the whole file with "key with no value". That
        # is a hard, visible failure rather than a silent one, but it is also
        # entirely avoidable, and it cost batch 5 a red `cargo test` on a file
        # whose content was correct. Idempotent for single-line entries.
        for piece in str(line).split("\n"):
            out.append(("# " + piece).rstrip())
    out.append("")

    if matrix:
        out.append("[matrix]")
        for axis, values in matrix.items():
            out.append(f"{axis} = {toml_str_array(values)}")
        out.append("")

    if source:
        out.append("[source]")
        for name, body in source.items():
            out.append(f"{_key(name)} = {toml_string(body)}")
        out.append("")

    for case in cases:
        steps = case["steps"]
        out.append("[[case]]")
        out.append(f"name = {toml_string(case['name'], multiline=False)}")
        out.append(f"rationale = {toml_string(case['rationale'], multiline=True)}")
        if len(steps) == 1:
            out.extend(_render_step(steps[0], ""))
        else:
            for step in steps:
                out.append("")
                out.append("[[case.step]]")
                out.extend(_render_step(step, ""))
        out.append("")

    return "\n".join(out).rstrip() + "\n"


MIGRATED_FROM_LINE = re.compile(
    r"^# Migrated from tests/(browser_[A-Za-z0-9_]+\.rs)")


def declare_source_ref(text):
    """Fold the `SOURCE REF:` declaration into a rendered case file's header.

    WHY THIS IS IN `write` AND NOT IN SIXTEEN GENERATORS: the same reason the
    citation reword is (see `write`). Every generator writes through here, so
    one edit gives all 108 generated case files whose source the family deletion
    removes a declaration that `citation_sweep.sh` can resolve afterwards --
    and none of the 12 whose source is RETAINED gets one, which matters, because
    a retained U4 carrier's citations are numbered against its `PRE-TRIM REF:`
    blob and an 8C ref would be content-checked against the wrong side.

    IDEMPOTENT, and a disagreement is an ERROR rather than a silent overwrite
    (ruling 18 #3): re-rendering a file that already carries the right ref is a
    no-op, and one carrying a DIFFERENT ref raises instead of being rewritten,
    because that is either an earlier batch's deletion (a different, correct
    sha) or a mistake, and this function cannot tell which.

    The declaration goes after the whole `Migrated from` PARAGRAPH, not after
    its first line. Three of these sentences wrap onto continuation lines
    (`runtime_summary_fallback_*` names its two `#[path]` submodules), and
    splitting a sentence around the ref would leave prose the next reader has to
    reassemble. `_declared_ref` only requires it to be somewhere in the leading
    `#` block.
    """
    lines = text.split("\n")
    idx = next((i for i, l in enumerate(lines) if MIGRATED_FROM_LINE.match(l)),
               None)
    if idx is None:
        return text
    name = MIGRATED_FROM_LINE.match(lines[idx]).group(1)

    header_end = next((i for i, l in enumerate(lines)
                       if l.strip() and not l.startswith("#")), len(lines))
    header = "\n".join(lines[:header_end])
    present = re.findall(r"SOURCE REF:\s*(\S+)", header)
    if len(present) > 1:
        raise AssertionError(
            f"{name}: {len(present)} `SOURCE REF:` lines in one header "
            f"({', '.join(present)}); keep one.")

    if not deleted_by_family_deletion(name):
        if present and present[0] == FAMILY_DELETION_REF:
            raise AssertionError(
                f"{name} is RETAINED at {FAMILY_DELETION_REF} (it carries a "
                "`//!` header there), so its case file must not declare the "
                "family-deletion ref. Its citations resolve against the "
                "working tree, or against its own `PRE-TRIM REF:`.")
        return text
    if present:
        if present[0] != FAMILY_DELETION_REF:
            raise AssertionError(
                f"{name}: header declares SOURCE REF {present[0]} but the "
                f"family deletion ref is {FAMILY_DELETION_REF}. Refusing to "
                "overwrite -- one of the two is wrong and this cannot tell "
                "which.")
        return text

    end = idx + 1
    while end < len(lines) and lines[end].startswith("#") and lines[end].lstrip("#").strip():
        end += 1
    lines.insert(end, f"#   SOURCE REF: {FAMILY_DELETION_REF}")
    return "\n".join(lines)


def write(path, text):
    """Render-to-disk, WITH the citation reword folded in.

    Why the reword lives here and not in each generator's own `cite()`: every
    one of this project's generators writes through this function -- 14 of 14,
    by a command that actually agrees with the claim beside it:

        $ python3 - <<'EOF'   # run in tools/task-18-browser-pilot/
        import ast, os
        bad = []
        for f in sorted(x for x in os.listdir(".")
                        if x.startswith("gen_batch") and x.endswith(".py")):
            names = {a.name for n in ast.walk(ast.parse(open(f).read()))
                     if isinstance(n, ast.ImportFrom) and n.module == "case_emit"
                     for a in n.names}
            if "write" not in names:
                bad.append(f)
        print(bad or "NONE -- 14 of 14 import case_emit.write")
        EOF

    The line-oriented `grep -L 'from case_emit import.*write' gen_batch*.py`
    that used to be recorded here PRINTS `gen_batch5_group_d.py`, whose import
    is parenthesised across lines -- the claim was true and the command was
    wrong, which is ruling 13's exact defect.

    And the reword is a *derivation*, not a transcription: it reads the construct it
    inserts out of the very source lines the citation points at
    (`reword_ungated_citations`'s module docstring states why that matters).
    Folding it here therefore gives every generator the reworded form without
    hard-coding one byte of post-processed output into any of them.

    Before this fold, `reword_ungated_citations.py --apply` was run as a
    post-pass over the shipped tree and no generator was ever taught the result,
    so a shipped `` `console.log` (:77) `` regenerated as `(:77)` and every
    generator that emits a citation drifted. The reword is idempotent on
    already-gated citations, so the three generators that were already fixed
    points stay fixed points.

    Unresolvable sites are left BARE on purpose, because that is what the tree
    carries and what `citation_sweep.sh` already declares (UNGATED_REDLIST /
    NO_NEEDLE_DECLARED). They are printed rather than raised: raising here would
    turn a declared, gated condition into a generator crash. A STALE citation --
    one pointing past the end of its source -- is a different thing and does
    raise, because nothing else in the pipeline reads it.
    """
    from reword_ungated_citations import rework_text  # noqa: E402  (cycle-free; imported late for import cost)

    stem = os.path.basename(path)
    if stem.endswith(".toml"):
        stem = stem[:-5]
    # BEFORE the reword, not after: `_pretrim_lines` resolves a deleted source
    # through the case file's own declaration, so the declaration has to be in
    # the text by the time it reads it.
    text = declare_source_ref(text)
    text, done, failed = rework_text(stem, text)
    stale = [f for f in failed if "STALE" in f]
    if stale:
        raise AssertionError(
            "citation past the end of its source -- the number is wrong, and no "
            "reword can paper over it:\n  " + "\n  ".join(stale))
    with open(path, "w") as f:
        f.write(text)
    note = f", {len(done)} citation(s) reworded" if done else ""
    print(f"wrote {path} ({len(text.splitlines())} lines{note})")
    for f in failed:
        print(f"  UNGATED (left bare, must be declared to the sweep): {f}")
