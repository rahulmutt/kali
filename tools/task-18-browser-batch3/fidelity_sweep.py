r"""U14 fidelity sweep for Task 18 batch 3: run the bidirectional source-vs-TOML
diff for every pair, and print BOTH directions -- but classify the raw output so
the `extra` side can actually be justified entry by entry instead of drowning in
noise.

`tools/task-18-browser-pilot/fidelity.py` compares every string literal in the
`.rs` against every string in the `.toml`, including things that are not claims
on either side (Rust `expect()`/panic-format messages; TOML structural keys,
case `name`s and `rationale` prose). Those categories are enumerated here and
reported as counts; whatever is left over is the real signal:

  * MISSING (in source, not in toml) that is NOT a panic/expect message is a
    candidate DROPPED claim -- the thing rule 1 forbids.
Three further categories are recognised as non-signal and counted rather than
listed: a `${axis}` template plus its `[matrix]` values (which `expand()`
rebuilds into exactly the concrete filename the source used), the `file_json`
`path` / `browser_bundle_harness` `entry` file references (excluded from the
audit's claim search for the same reason), and Rust panic/expect messages.

  * EXTRA (in toml, not in source) that is NOT structure/name/prose is a
    candidate INVENTED claim -- the thing rule 2 forbids. Every such entry must
    be justified in the batch report.

Usage: fidelity_sweep.py PAIRS_FILE   (lines: "<rs stem> <toml stem>")
"""
import os
import re
import subprocess
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
FIDELITY = os.path.join(HERE, '..', 'task-18-browser-pilot', 'fidelity.py')
TESTS = '/workspace/crates/kali_cli/tests'
CASES = os.path.join(TESTS, 'cases', 'browser')

# Strings the runner never turns into an assertion on either side.
TOML_STRUCTURE = {
    'case', 'step', 'source', 'matrix', 'constants', 'name', 'rationale',
    'ignore', 'kind', 'args', 'env', 'body', 'entry', 'path', 'fields',
    'exit', 'stdout', 'stdout_contains', 'stdout_absent', 'stderr',
    'stderr_contains', 'stderr_absent', 'json', 'json_null', 'ext',
    'file_json', 'browser_bundle_harness', 'cli',
}
# Rust `expect()` / assert-message formats: never claims about behavior.
PANIC_MSG = re.compile(
    r'^(tempdir|write source|run kali|kali binary path|read meta|'
    r'parse metadata json|payload object|errors array|valid json stdout|'
    r'json stdout|stdout string|stdout|error message|bundle root parent|'
    r'browser-bundle-smoke\.mjs|app\.meta\.json|write browser bundle harness|'
    r'run browser bundle harness|CARGO_BIN_EXE_kali|'
    r'stdout: \{.*|json: \{json\}|messages: \{messages.*|'
    r'unexpected errors: \{errors.*|source: \{source\}|'
    r'must fail closed: \{output.*|errors array should not be empty|'
    r'stderr: \{stderr\})$', re.DOTALL)


def main():
    pairs = [l.split() for l in open(sys.argv[1]) if l.strip()]
    total_bad = 0
    for rs_stem, toml_stem in pairs:
        rs = os.path.join(TESTS, rs_stem + '.rs')
        toml_path = os.path.join(CASES, toml_stem + '.toml')
        out = subprocess.run([sys.executable, FIDELITY, rs, '--', toml_path],
                             capture_output=True, text=True).stdout
        doc = tomllib.load(open(toml_path, 'rb'))
        names = {c.get('name', '') for c in doc.get('case', [])}
        missing, extra = [], []
        bucket = None
        for line in out.split('\n'):
            if line.startswith('MISSING ('):
                bucket = missing
            elif line.startswith('EXTRA ('):
                bucket = extra
            elif line.strip().startswith(('MISSING: ', 'EXTRA: ')):
                bucket.append(line.strip().split(': ', 1)[1])
        def unquote(s):
            try:
                return eval(s, {'__builtins__': {}}, {})
            except Exception:
                return s
        matrix = doc.get('matrix') or {}
        axis_values = {v for vs in matrix.values() for v in vs}
        # A concrete filename in source is not "dropped" when the case file
        # carries its `${axis}` template plus that axis value: expand() rebuilds
        # exactly the same argv. Recognise that pairing rather than reporting it.
        templated = set()
        for k in list(doc.get('source', {})) + [
                a for c in doc.get('case', [])
                for st in c.get('step', [c]) for a in st.get('args', [])]:
            for axis, vs in matrix.items():
                if '${' + axis + '}' in k:
                    templated.update(k.replace('${' + axis + '}', v) for v in vs)
        # `file_json` `path` / `entry` are file references, not claims (the audit
        # script excludes them for the same reason).
        file_refs = set()
        for c in doc.get('case', []):
            for st in c.get('step', [c]):
                for key in ('path', 'entry'):
                    if key in st:
                        file_refs.add(st[key])
        real_missing = [m for m in (unquote(x) for x in missing)
                        if not PANIC_MSG.match(m) and '{harness_function}' not in m
                        and '{export_name}' not in m and m not in TOML_STRUCTURE
                        and m not in templated]
        real_extra = []
        for x in extra:
            v = unquote(x)
            if v in TOML_STRUCTURE or v in names:
                continue
            if v.startswith('Migrated from ') or v.startswith('Task 18 '):
                continue
            if any(v == n[:len(v)] for n in names):
                continue
            if v in axis_values or v in file_refs:
                continue
            if any('${' + axis + '}' in v for axis in matrix):
                continue
            real_extra.append(v)
        n_bad = len(real_missing) + len(real_extra)
        total_bad += n_bad
        print(f'{toml_stem:52} missing={len(missing):>3} '
              f'(unexplained {len(real_missing)})  extra={len(extra):>3} '
              f'(unexplained {len(real_extra)})')
        for m in real_missing:
            print(f'    MISSING-UNEXPLAINED {m!r}')
        for e in sorted(set(real_extra)):
            print(f'    EXTRA-TO-JUSTIFY    {e!r}')
    print(f'\ntotal entries needing justification: {total_bad}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
