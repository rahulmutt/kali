r"""Generator for `browser_map_iteration_harness.rs` (Task 18 batch 3)."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from extract import fixture, comment_block  # noqa: E402
from emit import emit_case_file  # noqa: E402
from capture import run  # noqa: E402

TESTS = '/workspace/crates/kali_cli/tests'
CASES = os.path.join(TESTS, 'cases', 'browser')
NODE_ENV = {'KALI_BROWSER_BUNDLE_HARNESS_COMMAND': 'node'}
THROTTLE = ['--max-threads', '0', '--max-spawned-processes', '0']


def gen_map_iteration_harness():
    rs = open(os.path.join(TESTS, 'browser_map_iteration_harness.rs'),
              encoding='utf-8').read()
    run_body = fixture(rs, 'browser_harness_map_iteration_run_source')
    test_body = fixture(rs, 'browser_harness_map_iteration_test_source')
    # Rule 12: both comment blocks are COPIED out of the source, not retyped.
    TEST_SOURCE_PROSE = comment_block(rs, 172, 182)
    HELPER_PROSE = comment_block(rs, 374, 375)

    # Source's ONLY assertion is `!output.status.success()`. Confirm against the
    # real binary that every case really does fail closed before pinning it.
    for command, fname, body in (('run', 'main.js', run_body),
                                 ('test', 'smoke.test.js', test_body)):
        for json_output in (False, True):
            args = (['--output', 'json'] if json_output else []) + [
                command, '--api', 'browser'] + THROTTLE + [fname]
            rc, _out, _err = run({fname: body}, args, NODE_ENV)
            assert rc != 0, (command, json_output, rc)

    header = f"""Migrated from tests/browser_map_iteration_harness.rs.

{HELPER_PROSE}

NO [matrix] (rule 7 / U1): the FILENAME PATTERN is not uniform over the
extension axis. The four individual #[test] fns use `main.js` for "run" and
`smoke.test.js` for "test"; the fifth fn,
`supports_map_constructor_iteration_in_browser_api_surface_with_harness_ts_jsx_
tsx_input`, loops `for extension in ["ts","jsx","tsx"]` building a SINGLE
`format!("main.{{extension}}")` filename that it then uses for BOTH the "run"
and the "test" legs of its inner `for (command, json_output)` loop -- so
ts/jsx/tsx run `kali test main.ts`/`main.jsx`/`main.tsx`, never
`smoke.test.ts`/`.jsx`/`.tsx`. A file-level `ext` axis fans every [[case]]
uniformly (no per-case opt-out), so one axis could not express this without
inventing an untested `kali test smoke.test.ts` (if it followed the js
pattern) or an untested `kali test main.js` (if it followed the ts/jsx/tsx
pattern) -- a rule-2 violation either way. Declined for the whole file. This is
the same shape as batch 2's `array_iteration_spread_runtime`.

INVOCATION ARITHMETIC: 4 individual calls + (3 extensions x 4
(command, json_output) pairs) = 4 + 12 = 16 invocations across 5 #[test] fns,
expanded by reading the loops. 16 named sibling [[case]] entries below, one per
real invocation.

[source] KEY DISAMBIGUATION (U5 / U2): `assert_browser_harness_map_iteration`
picks its fixture body from the COMMAND (`if command == "test" {{
..._test_source() }} else {{ ..._run_source() }}`) and writes it to whatever
filename it was handed. For js that is two different names (`main.js` /
`smoke.test.js`), so no collision. For ts/jsx/tsx BOTH bodies would land on the
same `main.<ext>` key, which one flat file-wide `[source]` entry cannot carry,
so those stems are split into `main_run.<ext>` / `main_test.<ext>`. Only the
colliding keys are renamed; `main.js` and `smoke.test.js` keep their source
names. No fixture body references its own filename by string, so no claim
changes. All fixtures are present in every trial dir, which is harmless here:
no case's point is the presence or absence of a file, and each names its entry
explicitly on the argv (not the U2 conditional-fixture hazard).

ASSERTION SHAPE. Source's ONLY assertion in this file is
`assert!(!output.status.success(), "must fail closed: {{output:?}}")` -- there
is no exit-code claim, no stdout/stderr needle, no envelope field. So every
case carries exactly `exit = "failure"` and nothing else. Adding a diagnostic
code or a stream claim here would invent a claim the source never made
(rule 2), even though the real binary does emit E5506.

PROSE ATTRIBUTION (rule 12 / U6): this file has two Rust comment blocks.
`:374-375` sits in `assert_browser_harness_map_iteration`, which every case
reaches, so it is carried into all 16 rationales. `:172-182` sits in
`browser_harness_map_iteration_test_source()`, which is called only when
`command == "test"`, so it is carried into the 8 "test" rationales only.
`comment_coverage.py` has no per-helper attribution and will therefore report
those 11 lines "missing" from the 8 "run" [[case]] entries; that is the
checker's known limitation, recorded here rather than papered over by copying
the prose into cases whose producing helper never runs (which U6 forbids even
though it would turn the checker green).
"""
    sources = {'main.js': run_body, 'smoke.test.js': test_body}
    for ext in ('ts', 'jsx', 'tsx'):
        sources[f'main_run.{ext}'] = run_body
        sources[f'main_test.{ext}'] = test_body

    def fname_for(command, ext):
        if ext == 'js':
            return 'main.js' if command == 'run' else 'smoke.test.js'
        return f'main_run.{ext}' if command == 'run' else f'main_test.{ext}'

    def case(command, ext, json_output):
        f = fname_for(command, ext)
        args = (['--output', 'json'] if json_output else []) + [
            command, '--api', 'browser'] + THROTTLE + [f]
        prefix = 'json_' if json_output else ''
        rationale = (
            "Migrated from browser_map_iteration_harness.rs. " + HELPER_PROSE
            + f" `assert_browser_harness_map_iteration(\"{command}\", \"{f}\", "
              f"{'true' if json_output else 'false'})` runs `kali {command} --api "
              "browser" + (" --output json" if json_output else "") + "` under the "
              "browser harness (`node`) against the Map-constructor iteration probe "
              "and asserts ONLY that the process fails closed. That is the source's "
              "single assertion in this file -- no exit code, no stdout/stderr "
              "needle, no envelope field -- so this case carries exactly "
              "`exit = \"failure\"` and nothing else; adding a diagnostic code or a "
              "stream claim would invent a claim the source never made. No [matrix] "
              "in this file: the four individual #[test] fns use "
              "`main.js`/`smoke.test.js` while the looping fn uses a single "
              "`main.<ext>` filename for BOTH commands, so a file-wide `ext` axis "
              "could not express the filename pattern without inventing an untested "
              "combination (see the file header's arithmetic). The `[source]` stems "
              "for ts/jsx/tsx are split into `main_run`/`main_test` because the "
              "helper picks its body from the command while writing both to the same "
              "`main.<ext>` name, which one flat file-wide `[source]` key cannot "
              "carry; `main.js`/`smoke.test.js` keep their source names."
        )
        if command == 'test':
            rationale += '\n\n' + TEST_SOURCE_PROSE
        name = (f'{prefix}{command}_supports_map_constructor_iteration_in_browser_'
                f'api_surface_with_harness_{ext}_input')
        return {'name': name, 'rationale': rationale,
                'steps': [{'args': args, 'env': dict(NODE_ENV),
                           'exit': 'failure'}]}

    cases = []
    for command, json_output in (('run', False), ('test', False),
                                 ('run', True), ('test', True)):
        cases.append(case(command, 'js', json_output))
    for ext in ('ts', 'jsx', 'tsx'):
        for command, json_output in (('run', False), ('test', False),
                                     ('run', True), ('test', True)):
            cases.append(case(command, ext, json_output))
    assert len(cases) == 16 and len({c['name'] for c in cases}) == 16
    return emit_case_file(os.path.join(CASES, 'map_iteration_harness.toml'),
                          header, None, sources, cases)


if __name__ == '__main__':
    gen_map_iteration_harness()
