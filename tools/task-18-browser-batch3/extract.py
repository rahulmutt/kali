r"""Fixture-text extraction for Task 18 batch 3.

`fixture(rs_text, fn_name)` returns the DECODED value of the first Rust string
literal inside `fn <fn_name>(...)`'s body -- i.e. the exact program text the
source test writes to disk. The literal is pulled through
`tools/task-18-browser-pilot/lexer.py`'s character-cursor scanner, never
retyped, so `\"`/`\n`/raw-string bodies survive byte-identically (the
"copy fixture text, never retype it" discipline).

`assert_same(a, b)` is the mechanical identity assertion used before hoisting a
shared body into `[constants]` -- eyeballing is not enough.
"""
import re
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                '..', 'task-18-browser-pilot'))
from lexer import find_string_literals  # noqa: E402


def _fn_span(text, fn_name):
    m = re.search(r'^fn\s+' + re.escape(fn_name) + r'\b', text, re.MULTILINE)
    if not m:
        raise KeyError(f'no `fn {fn_name}` in source')
    # find the body's opening brace, then match to its close
    i = text.index('{', m.start())
    depth = 0
    j = i
    while j < len(text):
        if text[j] == '{':
            depth += 1
        elif text[j] == '}':
            depth -= 1
            if depth == 0:
                return i, j
        j += 1
    raise ValueError(f'unbalanced body for fn {fn_name}')


def fixture(text, fn_name):
    lo, hi = _fn_span(text, fn_name)
    for lit in find_string_literals(text):
        if lo <= lit['start'] < hi:
            return lit['value']
    raise KeyError(f'no string literal in body of fn {fn_name}')


def fixtures(text, fn_name):
    """Every string literal in the fn body, in source order."""
    lo, hi = _fn_span(text, fn_name)
    return [l['value'] for l in find_string_literals(text) if lo <= l['start'] < hi]


def assert_same(a, b, what='bodies'):
    if a != b:
        raise AssertionError(f'{what} differ; refusing to hoist into [constants]')
    return a


def comment_block(text, first_line, last_line):
    r"""Rule 12: source prose is COPIED, never retyped. Returns the `//`
    comment text on the 1-indexed inclusive line range, leading marker and one
    space stripped, joined with single spaces -- so an em-dash or an ellipsis
    survives byte-identically into the `rationale` it lands in."""
    lines = text.split('\n')[first_line - 1:last_line]
    out = []
    for line in lines:
        stripped = line.lstrip()
        if not stripped.startswith('//'):
            raise ValueError(f'line {first_line}..{last_line} is not all comment: {line!r}')
        out.append(stripped[2:].lstrip('/!').removeprefix(' '))
    return ' '.join(out)
