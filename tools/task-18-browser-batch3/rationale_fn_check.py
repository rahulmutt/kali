r"""U8: `rationale` prose is audited by NOTHING -- verify its factual claims.

`audit-case-migration.py` deliberately never reads `rationale`, and
`comment_coverage.py` only checks that source comment text APPEARS in a
rationale, never that a rationale's own assertions are true. Batch 2 shipped
four rationales citing source fn names that do not exist.

This script cross-references every backticked, fn-shaped identifier in a case
file's `#` header and in every `rationale` against the real `fn` list of the
`.rs` it was migrated from, and exits non-zero on any citation that names no
real function. Identifiers that are obviously not fn citations (JS/TOML
vocabulary, CLI flags, file stems) are filtered by an explicit allowlist rather
than by a heuristic, so a genuinely wrong fn name cannot slip through as
"probably not a function".

Usage: rationale_fn_check.py SOURCE.rs TARGET.toml
"""
import re
import sys
import tomllib

# Backticked identifiers that are vocabulary, not source-fn citations.
ALLOW = {
    # case-file format / runner vocabulary
    'source', 'matrix', 'case', 'constants', 'rationale', 'name', 'ignore',
    'kind', 'args', 'env', 'body', 'entry', 'path', 'fields', 'exit',
    'stdout', 'stderr', 'stdout_contains', 'stderr_contains', 'stdout_absent',
    'stderr_absent', 'json', 'json_null', 'expand', 'ext', 'errors',
    'warnings', 'payload', 'command', 'success', 'schemaVersion', 'exitCode',
    'total', 'passed', 'failed', 'skipped', 'errorCount', 'message', 'code',
    'hostContract', 'runtimeBackend', 'artifactKind', 'bundleFormat',
    'apiSurface',
    # Rust / test vocabulary
    'contains', 'matches', 'count', 'assert', 'assert_eq', 'format', 'match',
    'str', 'let', 'fn', 'mod', 'bool', 'true', 'false', 'node', 'kali',
    'json_output', 'output_json', 'command_', 'expected_stdout', 'filename',
    'source_name', 'extension', 'bundle', 'harness_function', 'export_name',
    'expect_test_runner', 'main', 'run', 'test', 'check', 'build',
    # JS keywords / identifiers quoted inside prose about the fixtures
    'break', 'continue', 'bump', 'yield',
    # [source] fixture stems and variant labels this batch introduces. Each was
    # read back by hand against the case file it appears in and is a filename
    # stem or a variant label, never a claim about a source function.
    'app', 'app_as_const', 'app_satisfies', 'app_for_of', 'app_for_await',
    'main_run', 'main_test', 'main_for_of', 'main_for_await',
    'as_const', 'satisfies', 'parenthesized_const_alias', 'transparent_await',
    'for_of', 'for_await', 'zero_slice', 'frozen', 'await_wrapped', 'variant',
    'json_', '_js_input', 'exp_log',
    # sibling case-file basenames cited as precedent
    'array_iteration_spread_runtime', 'math_atan2_bracketed_root',
    'template_literal_dynamic_import_harness', 'file_json',
}
FN_SHAPED = re.compile(r'`([a-z_][a-z0-9_]*)`')


def source_fns(rs_text):
    return set(re.findall(r'\bfn\s+([a-z_][A-Za-z0-9_]*)', rs_text))


def cited(toml_path):
    out = {}
    text = open(toml_path, encoding='utf-8').read()
    header = '\n'.join(l for l in text.split('\n') if l.startswith('#'))
    doc = tomllib.loads(text)
    blobs = [('<header>', header)]
    for case in doc.get('case', []):
        blobs.append((case.get('name', '<unnamed>'), case.get('rationale', '')))
    for where, blob in blobs:
        for ident in FN_SHAPED.findall(blob):
            if ident in ALLOW:
                continue
            out.setdefault(ident, set()).add(where)
    return out


def main():
    rs, toml = sys.argv[1], sys.argv[2]
    real = source_fns(open(rs, encoding='utf-8').read())
    bad = []
    n = 0
    for ident, wheres in sorted(cited(toml).items()):
        n += 1
        if ident not in real:
            bad.append((ident, sorted(wheres)[:2]))
    print(f'{toml}: {n} backticked fn-shaped citation(s) checked against '
          f'{len(real)} real fns in {rs}, {len(bad)} unresolved')
    for ident, wheres in bad:
        print(f'  UNRESOLVED `{ident}` (cited in {wheres})')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
