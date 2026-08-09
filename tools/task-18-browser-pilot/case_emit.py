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
    "json", "json_null", "json_count",
]


def _render_step(step, prefix):
    out = []
    unknown = [k for k in step if k not in _STEP_ORDER]
    if unknown:
        raise AssertionError(f"unknown step key(s) {unknown} -- typo, or a new 5.4 key")
    for key in _STEP_ORDER:
        if key not in step:
            continue
        v = step[key]
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


def write(path, text):
    with open(path, "w") as f:
        f.write(text)
    print(f"wrote {path} ({len(text.splitlines())} lines)")
