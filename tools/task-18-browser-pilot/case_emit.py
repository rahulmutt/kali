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

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from toml_emit import toml_string, toml_str_array  # noqa: E402
from lexer import string_literals_in_range  # noqa: E402


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
        out.append(("# " + line).rstrip())
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
