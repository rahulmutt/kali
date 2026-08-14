r"""Case-file emitter + round-trip verifier for Task 18 batch 3.

Everything a generator hands to `emit_case_file()` is written out as TOML and
then IMMEDIATELY re-parsed with `tomllib` and compared field-by-field against
the Python values it was built from. That round trip is the mechanical guard
that no fixture body, argv token or expected string was mangled by quoting --
"copy fixture text, never retype it" is only as good as the emitter.

Emission conventions (chosen to match the 25 case files already shipped under
`crates/kali_cli/tests/cases/browser/`):
  * `[source]` bodies and any newline-bearing `body` use a TOML multi-line basic string.
  * argv / `stdout_contains` use single-line basic strings.
  * `json` assertions are emitted as one inline table per step.
"""
import os
import sys
import textwrap
import tomllib

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                '..', 'task-18-browser-pilot'))
from toml_emit import toml_string, toml_str_array  # noqa: E402


def _key(k):
    return '"' + k.replace('\\', '\\\\').replace('"', '\\"') + '"'


def _scalar(v):
    if isinstance(v, bool):
        return 'true' if v else 'false'
    if isinstance(v, int):
        return str(v)
    if isinstance(v, str):
        return toml_string(v, multiline=False)
    if isinstance(v, list):
        if not v:
            return '[]'
        return '[' + ', '.join(_scalar(x) for x in v) + ']'
    if isinstance(v, dict):
        return '{ ' + ', '.join(f'{_key(k)} = {_scalar(x)}' for k, x in v.items()) + ' }'
    raise TypeError(f'unsupported TOML value: {v!r}')


def _step_lines(step):
    out = []
    order = ['kind', 'args', 'path', 'entry', 'env', 'body', 'exit', 'fields',
             'stdout', 'stdout_contains', 'stdout_absent',
             'stderr', 'stderr_contains', 'stderr_absent', 'json', 'json_null']
    for k in order:
        if k not in step:
            continue
        v = step[k]
        if k == 'body':
            out.append(f'body = {toml_string(v)}')
        elif k == 'exit':
            out.append(f'exit = {_scalar(v)}')
        elif k == 'json':
            out.append(f'json = {_scalar(v)}')
        else:
            out.append(f'{k} = {_scalar(v)}')
    for k in step:
        if k not in order:
            raise KeyError(f'unknown step key {k}')
    return out


def render(header, matrix, sources, cases):
    lines = []
    for line in header.rstrip('\n').split('\n'):
        # Header paragraphs are hand-wrapped; only re-wrap a line that would
        # otherwise run past the file's 78-column comment width.
        for piece in (textwrap.wrap(line, 76) if len(line) > 76 else [line]):
            lines.append(('# ' + piece).rstrip())
    lines.append('')
    if matrix:
        lines.append('[matrix]')
        for axis, values in matrix.items():
            lines.append(f'{axis} = {toml_str_array(values)}')
        lines.append('')
    if sources:
        lines.append('[source]')
        for name, body in sources.items():
            lines.append(f'{_key(name)} = {toml_string(body, multiline=True)}')
        lines.append('')
    for case in cases:
        lines.append('[[case]]')
        lines.append(f'name = {toml_string(case["name"], multiline=False)}')
        lines.append(f'rationale = {toml_string(case["rationale"], multiline=True)}')
        if case.get('ignore'):
            lines.append('ignore = true')
        steps = case['steps']
        if len(steps) == 1 and not steps[0].get('kind'):
            lines.extend(_step_lines(steps[0]))
        else:
            for step in steps:
                lines.append('')
                lines.append('[[case.step]]')
                lines.extend(_step_lines(step))
        lines.append('')
    return '\n'.join(lines).rstrip('\n') + '\n'


def _verify(text, matrix, sources, cases, path):
    doc = tomllib.loads(text)
    assert doc.get('matrix', {}) == (matrix or {}), f'{path}: matrix round-trip'
    assert doc.get('source', {}) == (sources or {}), f'{path}: [source] round-trip'
    got = doc.get('case', [])
    assert len(got) == len(cases), f'{path}: case count {len(got)} != {len(cases)}'
    for g, want in zip(got, cases):
        assert g['name'] == want['name'], f'{path}: name round-trip'
        assert g['rationale'] == want['rationale'], f'{path}: rationale round-trip'
        steps = want['steps']
        if len(steps) == 1 and not steps[0].get('kind'):
            gsteps = [{k: v for k, v in g.items()
                       if k not in ('name', 'rationale', 'ignore')}]
        else:
            gsteps = g['step']
        assert len(gsteps) == len(steps), f'{path}: step count in {want["name"]}'
        for gs, ws in zip(gsteps, steps):
            assert gs == ws, (f'{path}: step round-trip in {want["name"]}\n'
                              f'  got  {gs}\n  want {ws}')


def emit_case_file(path, header, matrix, sources, cases):
    text = render(header, matrix, sources, cases)
    _verify(text, matrix, sources, cases, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w', encoding='utf-8') as f:
        f.write(text)
    n_trials = len(cases)
    for values in (matrix or {}).values():
        n_trials *= len(values)
    print(f'  wrote {path}: {len(cases)} case(s), {n_trials} trial(s)')
    return n_trials
