r"""Generators for the `browser_math_*` targets of Task 18 batch 3.

Each generator re-derives its own invocation arithmetic from the real call
sites (see `inventory.py`), pulls every fixture body out of the `.rs` through
`extract.fixture` (never retyped), and live-captures every exact `json.stdout`
leaf from the real built `kali` binary through `capture.py` before writing it
into a case file.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from extract import fixture  # noqa: E402
from emit import emit_case_file  # noqa: E402
from capture import json_envelope  # noqa: E402

TESTS = '/workspace/crates/kali_cli/tests'
CASES = os.path.join(TESTS, 'cases', 'browser')
EXTS = ['js', 'ts', 'jsx', 'tsx']
NODE_ENV = {'KALI_BROWSER_BUNDLE_HARNESS_COMMAND': 'node'}
HARNESS_BODY = 'const mod = await import(bundleJs.href);\nawait mod.%s();\n'
THROTTLE = ['--max-threads', '0', '--max-spawned-processes', '0']

NO_COMMENTS = (
    "No Rust comments exist anywhere in the source file (checked: `grep -nE "
    "'^\\s*//'` finds nothing outside the JS fixture bodies' own "
    "`// kali-tree-shake:` markers, which are program text, not Rust "
    "comments), so there is no prose to move verbatim into `rationale` here "
    "(rule 12)."
)


def src(name):
    return open(os.path.join(TESTS, name), encoding='utf-8').read()


def bundle_steps(stem, ext_expr, json_output, export_name, stdout_contains,
                 assert_errors_empty):
    args = ['build', '--bundle', '--api', 'browser']
    if json_output:
        args += ['--output', 'json']
    args.append(f'{stem}.{ext_expr}')
    step = {'args': args, 'exit': 'success'}
    if json_output:
        j = {'schemaVersion': 1, 'command': 'build', 'success': True,
             'exitCode': 0,
             'payload': {'artifactKind': 'bundle', 'bundleFormat': 'esm'}}
        if assert_errors_empty:
            j['errors'] = []
        step['json'] = j
    return [
        step,
        {'kind': 'file_json', 'path': f'{stem}/{stem}.meta.json',
         'fields': {'apiSurface': 'browser', 'artifactKind': 'bundle'}},
        {'kind': 'browser_bundle_harness', 'entry': stem,
         'body': HARNESS_BODY % export_name,
         'stdout_contains': list(stdout_contains)},
    ]


def harness_args(command, fname, json_output, throttle=True):
    return ((['--output', 'json'] if json_output else [])
            + [command, '--api', 'browser'] + (THROTTLE if throttle else [])
            + [fname])


def live_stdout(fname, body, command, json_output_args):
    _rc, envelope, _err = json_envelope({fname: body}, json_output_args, NODE_ENV)
    assert envelope['command'] == command, envelope
    return envelope['stdout']


# ============================================================== abs / sign ===

def gen_abs_sign():
    rs = src('browser_math_abs_sign_frozen_aliases.rs')
    bundle_body = fixture(rs, 'browser_bundle_global_this_math_abs_sign_frozen_source')
    run_body = fixture(rs, 'browser_harness_global_this_math_abs_sign_run_source')
    test_body = fixture(rs, 'browser_harness_global_this_math_abs_sign_test_source')

    live = {}
    for command, fname, body in (('run', 'main.js', run_body),
                                 ('test', 'smoke.test.js', test_body)):
        live[command] = live_stdout(
            fname, body, command,
            harness_args(command, fname, True, throttle=False))

    header = f"""Migrated from tests/browser_math_abs_sign_frozen_aliases.rs.
{NO_COMMENTS}

PARTIAL MIGRATION -- 24 of this file's 25 #[test] fns are migrated here. The
25th, `browser_bundle_global_this_math_abs_sign_frozen_source_includes_direct_
frozen_math_aliases`, is a FIXTURE SELF-INSPECTION test: it runs four
`assert!(source.contains("Object.freeze(...)"))` checks against the JS
fixture's own text and never builds a command at all. That shape is invisible
to `scripts/audit-case-migration.py` (its `.contains()` extractor cannot tell a
fixture-text read from an output assertion, and everything under `[source]` is
excluded from its search by construction), so migrating it would produce a
false green. It stays hand-written and is escalated per rule 3/4; see the
retention header on the `.rs` and the batch report. Nothing else in this file
reaches that construct -- the 24 fns below all route through
`assert_browser_bundle_global_this_math_abs_sign_frozen` or
`assert_browser_harness_global_this_math_abs_sign_frozen`, neither of which
reads fixture text -- so U4's trim-and-keep applies and only that one test is
retained.

MATRIX ARITHMETIC (rule 7), over the migrated 24: 8 bundle fns
(ext(4) x json_output(2), all individual, no loops) + 16 harness fns
(command(run/test) x ext(4) x json_output(2), likewise all individual) = 24
invocations = 24 #[test] fns. Both groups vary uniformly over all four
extensions, so `ext` is hoisted to a file-level [matrix] axis: 24 fns collapse
to 6 [[case]] entries, matrix-fanned to 24 trials, matching exactly.

[source] needs no disambiguation (U5): `app.<ext>` (bundle), `main.<ext>` (run)
and `smoke.test.<ext>` (test) are already distinct keys.

ASSERTION SHAPE. Bundle: `exit = "success"`; in json mode
schemaVersion/command/success/exitCode/payload(artifactKind, bundleFormat) --
source makes NO `errors` claim in this file's bundle helper, so none is added
(rule 2); then the emitted `app/app.meta.json` metadata; then the
bundle-harness `stdout_contains = ["3\\n", "-1\\n"]`, mirroring source's two
plain `.contains` checks. Harness: `exit = "success"`; non-json
`stdout_contains = ["3\\n", "-1\\n"]`; json mode carries
schemaVersion/command/success/payload(hostContract, runtimeBackend), then
`json["exitCode"]`/`payload.exitCode` for "run" or `payload.total/passed/failed`
for "test", `stderr = ""` and `errors = []`
(`assert_eq!(json["errors"], Value::Array(vec![]))`), plus an exact
`json.stdout` pin. That last one resolves source's
`json["stdout"].contains("3\\n")`/`...("-1\\n")`: a nested `json` leaf has no
substring form in this format, so it is pinned exactly and only after
live-capturing the value from the real `kali` binary -- a verified
strengthening. This helper's argv has NO `--max-threads`/
`--max-spawned-processes` throttle (unlike most siblings in this family);
mirrored exactly.
"""
    sources = {'app.${ext}': bundle_body,
               'main.${ext}': run_body,
               'smoke.test.${ext}': test_body}
    cases = []
    for json_output in (False, True):
        prefix = 'json_build_emits' if json_output else 'build_emits'
        cases.append({
            'name': f'{prefix}_global_this_math_abs_sign_frozen_aliases',
            'rationale': (
                "Migrated from browser_math_abs_sign_frozen_aliases.rs. "
                "`assert_browser_bundle_global_this_math_abs_sign_frozen` builds a "
                "browser bundle (`kali build --bundle --api browser"
                + (" --output json`), asserts the JSON envelope's "
                   "schemaVersion/command/success/exitCode/payload (artifactKind, "
                   "bundleFormat) fields -- source makes no `errors` claim in this "
                   "file's bundle helper, so none is added -- then asserts"
                   if json_output else "`), asserts")
                + " the emitted `app/app.meta.json` metadata, then runs the bundle "
                  "glue under the browser-bundle-harness contract and checks that "
                  "the frozen `Math.abs`/`Math.sign` alias calls printed `3` and "
                  "`-1`. Source's stdout claims are plain `.contains`, mirrored as "
                  "`stdout_contains` rather than strengthened. `ext` (js/ts/jsx/tsx) "
                  "is hoisted to a file-level [matrix] axis: the 24 migrated #[test] "
                  "fns collapse to 6 [[case]] entries here, matrix-fanned to 24 "
                  "trials (see the file header's arithmetic). The file's 25th fn, "
                  "`browser_bundle_global_this_math_abs_sign_frozen_source_includes_"
                  "direct_frozen_math_aliases`, is a fixture self-inspection test "
                  "and stays hand-written per rule 3/4."),
            'steps': bundle_steps('app', '${ext}', json_output,
                                  'globalThisMathAbsSignFrozenAliases',
                                  ['3\n', '-1\n'], assert_errors_empty=False)})
    for command in ('run', 'test'):
        fname = 'main.${ext}' if command == 'run' else 'smoke.test.${ext}'
        for json_output in (False, True):
            step = {'args': harness_args(command, fname, json_output,
                                         throttle=False),
                    'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                payload = {'hostContract': 'browser-requested',
                           'runtimeBackend': 'browser-harness'}
                j = {'schemaVersion': 1, 'command': command, 'success': True,
                     'payload': payload}
                if command == 'run':
                    j['exitCode'] = 0
                    payload['exitCode'] = 0
                else:
                    payload['total'] = 1
                    payload['passed'] = 1
                    payload['failed'] = 0
                j['stdout'] = live[command]
                j['stderr'] = ''
                j['errors'] = []
                step['json'] = j
            else:
                step['stdout_contains'] = ['3\n', '-1\n']
            prefix = 'json_' if json_output else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_global_this_math_abs_sign_'
                         'frozen_aliases_when_browser_harness_is_configured'),
                'rationale': (
                    "Migrated from browser_math_abs_sign_frozen_aliases.rs. "
                    "`assert_browser_harness_global_this_math_abs_sign_frozen("
                    f"\"{command}\", ...)` runs `kali {command} --api browser"
                    + (" --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`json[\"exitCode\"] == 0` and `payload.exitCode == 0`, "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed` == 1/1/0, ")
                       + "that `stderr` is exactly empty, that `errors` is exactly "
                         "the empty array, and that `json[\"stdout\"]` contains both "
                         "`3\\n` and `-1\\n`. A nested `json` leaf has no "
                         "substring-assertion form in this case-file format (only "
                         "exact equality), so that claim is resolved to an exact pin "
                         "-- live-captured from the real `kali` binary, never "
                         "hand-computed, and strictly stronger than source's two "
                         "`.contains()` checks."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json mode, "
                       "asserts a clean exit and that stdout contains both `3\\n` "
                       "and `-1\\n`; plain `.contains` against a field that has a "
                       "substring form, so mirrored as `stdout_contains`.")
                    + " This helper's argv carries no `--max-threads`/"
                      "`--max-spawned-processes` throttle, unlike most siblings in "
                      "this family; mirrored exactly. `ext` (js/ts/jsx/tsx) is "
                      "hoisted to a file-level [matrix] axis: the 24 migrated "
                      "#[test] fns collapse to 6 [[case]] entries here, "
                      "matrix-fanned to 24 trials (see the file header). The file's "
                      "25th fn is a fixture self-inspection test and stays "
                      "hand-written per rule 3/4."),
                'steps': [step]})
    return emit_case_file(os.path.join(CASES, 'math_abs_sign_frozen_aliases.toml'),
                          header, {'ext': EXTS}, sources, cases)


# ======================================================= inverse hyperbolic ==
# REMOVED in fix round 1. `gen_asinh()` used to emit
# cases/browser/math_asinh_acosh_atanh_identities.toml. The controller
# reversed that migration: every one of that target's 24 `#[test]` fns makes
# a `.matches(<needle>).count() >= N` claim, which the assertion vocabulary
# cannot carry -- the bare needle weakens it (rule 1), an exact stdout pin is
# barred by ruling 3, and the contiguous three-in-a-row needle this generator
# used to emit invents an adjacency claim the source never made (rule 2). The
# target is retained whole per spec 5.11; see its `//!` header. The generator
# is deleted rather than left in place so re-running this file cannot
# resurrect the reverted case file.


# ================================================================== clz32 ====

def gen_clz32():
    rs = src('browser_math_clz32_omitted_operands.rs')
    bundle_body = fixture(rs, 'browser_bundle_math_clz32_omitted_operands_source')
    run_body = fixture(rs, 'browser_harness_math_clz32_omitted_operands_run_source')
    test_body = fixture(rs, 'browser_harness_math_clz32_omitted_operands_test_source')
    expected = {'run': '32\n32\n32\n32\n32\n32\n32',
                'test': '32\n32\n32\n32\n32\n32\n32\nok 1'}

    header = f"""Migrated from tests/browser_math_clz32_omitted_operands.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): 8 bundle fns (ext(4) x json_output(2)) + 16 harness
fns = 24 invocations = 24 #[test] fns, all individual (no loops anywhere in
this file). Both groups vary uniformly over all four extensions, so `ext` is
hoisted to a file-level [matrix] axis: 24 fns collapse to 6 [[case]] entries,
matrix-fanned to 24 trials, matching exactly.

SOURCE ODDITY, CARRIED AS-IS (rule 6). The harness helper
`assert_browser_harness_math_clz32_omitted_operands(command, filename, source,
expected_stdout)` takes NO `json_output` parameter and never passes
`--output json`. Its 16 callers are nevertheless named in two groups of 8 --
`run_supports_*`/`test_supports_*` and `json_run_supports_*`/
`json_test_supports_*` -- and each `json_`-prefixed fn passes byte-identical
arguments to its unprefixed twin. So eight pairs of source #[test] fns are
literally the same invocation with the same assertion. They are NOT folded:
rule 6 keeps one [[case]] per source fn even when the invocations are
identical, because the case is the only remaining trace of the fn. The
`json_`-prefixed cases below therefore carry the same argv and the same
assertion as their twins, and no `--output json` is added to them -- adding it
would invent a claim source never made (rule 2). This is a fact about the
source, recorded rather than quietly corrected (U7).

[source] needs no disambiguation (U5): `app.<ext>`, `main.<ext>` and
`smoke.test.<ext>` are already distinct keys.

ASSERTION SHAPE. Bundle: `exit = "success"`; json mode
schemaVersion/command/success/exitCode/payload(artifactKind, bundleFormat)
(source makes no `errors` claim in this file, so none is added); the emitted
`app/app.meta.json` metadata; then the bundle-harness
`stdout_contains = ["32\\n32\\n32\\n32\\n32\\n32\\n32\\n"]`, mirroring source's
single `.contains` needle verbatim. Harness: `exit = "success"` plus
`stdout_contains` of the per-command `expected_stdout` the source passes in --
`32\\n` x7 for "run", the same plus `\\nok 1` for "test". Both are plain
`.contains` against a field that has a substring form, so they stay
`*_contains` (controller ruling 3), and the `\\nok 1` suffix is carried only on
the "test" cases, where source actually passes it.
"""
    sources = {'app.${ext}': bundle_body,
               'main.${ext}': run_body,
               'smoke.test.${ext}': test_body}
    cases = []
    for json_output in (False, True):
        prefix = 'json_build_emits' if json_output else 'build_emits'
        cases.append({
            'name': f'{prefix}_math_clz32_omitted_operands',
            'rationale': (
                "Migrated from browser_math_clz32_omitted_operands.rs. "
                "`assert_browser_bundle_math_clz32_omitted_operands` builds a "
                "browser bundle (`kali build --bundle --api browser"
                + (" --output json`), asserts the JSON envelope's "
                   "schemaVersion/command/success/exitCode/payload (artifactKind, "
                   "bundleFormat) fields -- source makes no `errors` claim in this "
                   "file, so none is added -- then asserts"
                   if json_output else "`), asserts")
                + " the emitted `app/app.meta.json` metadata, then runs the bundle "
                  "glue under the browser-bundle-harness contract and checks that "
                  "all seven omitted-operand `Math.clz32()` spellings printed `32`. "
                  "Source's stdout claim is a single plain `.contains` needle, "
                  "carried verbatim as `stdout_contains` rather than strengthened. "
                  "`ext` (js/ts/jsx/tsx) is hoisted to a file-level [matrix] axis: "
                  "24 #[test] fns collapse to 6 [[case]] entries here, matrix-fanned "
                  "to 24 trials (see the file header's arithmetic)."),
            'steps': bundle_steps('app', '${ext}', json_output,
                                  'mathClz32OmittedOperands',
                                  ['32\n32\n32\n32\n32\n32\n32\n'],
                                  assert_errors_empty=False)})
    for json_named in (False, True):
        for command in ('run', 'test'):
            fname = 'main.${ext}' if command == 'run' else 'smoke.test.${ext}'
            prefix = 'json_' if json_named else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_math_clz32_omitted_operands_'
                         'when_browser_harness_is_configured'),
                'rationale': (
                    "Migrated from browser_math_clz32_omitted_operands.rs. "
                    "`assert_browser_harness_math_clz32_omitted_operands("
                    f"\"{command}\", ...)` runs `kali {command} --api browser` under "
                    "the browser harness (`node`) and asserts a clean exit plus "
                    f"`stdout.contains({expected[command]!r})` -- a plain "
                    "`.contains` against a field that has a substring form, so it "
                    "is mirrored as `stdout_contains` rather than strengthened to an "
                    "exact pin. "
                    + ("This case's source fn is named `json_" + command
                       + "_supports_*`, but the helper it calls takes no "
                         "`json_output` parameter and never passes `--output json`: "
                         "the invocation and the assertion are byte-identical to the "
                         "unprefixed `" + command + "_supports_*` twin. The two are "
                         "kept as separate [[case]] entries rather than folded, "
                         "because the case is the only remaining trace of the source "
                         "fn (rule 6), and no `--output json` is added, because "
                         "source never passes it (rule 2). "
                       if json_named else "")
                    + "`ext` (js/ts/jsx/tsx) is hoisted to a file-level [matrix] "
                      "axis: 24 #[test] fns collapse to 6 [[case]] entries here, "
                      "matrix-fanned to 24 trials (see the file header's "
                      "arithmetic)."),
                'steps': [{'args': [command, '--api', 'browser'] + THROTTLE + [fname],
                           'env': dict(NODE_ENV), 'exit': 'success',
                           'stdout_contains': [expected[command]]}]})
    return emit_case_file(os.path.join(CASES, 'math_clz32_omitted_operands.toml'),
                          header, {'ext': EXTS}, sources, cases)


# ====================================================== exp2 globalThis root =

def gen_exp2_global_this():
    rs = src('browser_math_exp2_global_this_root.rs')
    bundle_body = fixture(rs, 'browser_bundle_global_this_math_exp2_source')
    run_body = fixture(rs, 'browser_harness_global_this_math_exp2_run_source')
    test_body = fixture(rs, 'browser_harness_global_this_math_exp2_test_source')
    live = {}
    for command, fname, body in (('run', 'main.js', run_body),
                                 ('test', 'smoke.test.js', test_body)):
        live[command] = live_stdout(fname, body, command,
                                    harness_args(command, fname, True))

    header = f"""Migrated from tests/browser_math_exp2_global_this_root.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): 8 bundle fns (ext(4) x json_output(2)) + 16 harness
fns (command(run/test) x ext(4) x json_output(2)) = 24 invocations = 24 #[test]
fns, all individual (no loops anywhere in this file). Both groups vary
uniformly over all four extensions, so `ext` is hoisted to a file-level
[matrix] axis: 24 fns collapse to 6 [[case]] entries, matrix-fanned to 24
trials, matching exactly.

[source] needs no disambiguation (U5): `app.<ext>`, `main.<ext>` and
`smoke.test.<ext>` are already distinct keys.

ASSERTION SHAPE. Bundle: `exit = "success"`; json mode carries
schemaVersion/command/success/exitCode/payload(artifactKind, bundleFormat) AND
`errors = []` (this file's bundle helper does assert
`envelope["errors"].is_empty()`, unlike several of its siblings); then the
emitted `app/app.meta.json` metadata; then the bundle-harness
`stdout_contains = ["4\\n"]`. Harness: `exit = "success"`; non-json
`stdout_contains = ["4\\n"]`; json mode carries
schemaVersion/command/success/payload(hostContract, runtimeBackend), then
`json["exitCode"]`/`payload.exitCode` for "run" or
`payload.total/passed/failed` for "test", `errors = []`, `warnings = []`
(`assert_eq!(json["warnings"], Value::Array(vec![]))` -- an exact
empty-array claim this file makes and most of its siblings do not),
`stderr = ""`, and an exact `json.stdout` pin resolving source's
`json["stdout"].contains("4\\n")`, live-captured from the real `kali` binary.
"""
    sources = {'app.${ext}': bundle_body,
               'main.${ext}': run_body,
               'smoke.test.${ext}': test_body}
    cases = []
    for json_output in (False, True):
        prefix = 'json_build_emits' if json_output else 'build_emits'
        cases.append({
            'name': f'{prefix}_global_this_math_exp2_zero_identity',
            'rationale': (
                "Migrated from browser_math_exp2_global_this_root.rs. "
                "`assert_browser_bundle_global_this_math_exp2` builds a browser "
                "bundle (`kali build --bundle --api browser"
                + (" --output json`), asserts the JSON envelope's "
                   "schemaVersion/command/success/exitCode/payload (artifactKind, "
                   "bundleFormat) fields and that `errors` is empty, then asserts"
                   if json_output else "`), asserts")
                + " the emitted `app/app.meta.json` metadata, then runs the bundle "
                  "glue under the browser-bundle-harness contract and checks that "
                  "every `globalThis`-rooted `Math.exp2` spelling printed `4`. "
                  "Source's stdout claim is a plain `.contains`, mirrored as "
                  "`stdout_contains`. `ext` (js/ts/jsx/tsx) is hoisted to a "
                  "file-level [matrix] axis: 24 #[test] fns collapse to 6 [[case]] "
                  "entries here, matrix-fanned to 24 trials (see the file header's "
                  "arithmetic)."),
            'steps': bundle_steps('app', '${ext}', json_output,
                                  'globalThisMathExp2NonNegativeIntegerLiterals',
                                  ['4\n'], assert_errors_empty=True)})
    for command in ('run', 'test'):
        fname = 'main.${ext}' if command == 'run' else 'smoke.test.${ext}'
        for json_output in (False, True):
            step = {'args': harness_args(command, fname, json_output),
                    'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                payload = {'hostContract': 'browser-requested',
                           'runtimeBackend': 'browser-harness'}
                j = {'schemaVersion': 1, 'command': command, 'success': True,
                     'payload': payload}
                if command == 'run':
                    j['exitCode'] = 0
                    payload['exitCode'] = 0
                else:
                    payload['total'] = 1
                    payload['passed'] = 1
                    payload['failed'] = 0
                j['errors'] = []
                j['warnings'] = []
                j['stdout'] = live[command]
                j['stderr'] = ''
                step['json'] = j
            else:
                step['stdout_contains'] = ['4\n']
            prefix = 'json_' if json_output else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_global_this_math_exp2_zero_'
                         'identity_when_browser_harness_is_configured'),
                'rationale': (
                    "Migrated from browser_math_exp2_global_this_root.rs. "
                    f"`assert_browser_harness_global_this_math_exp2(\"{command}\", "
                    f"...)` runs `kali {command} --api browser"
                    + (" --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`json[\"exitCode\"] == 0` and `payload.exitCode == 0`, "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed` == 1/1/0, ")
                       + "that `errors` is empty, that `warnings` is exactly the "
                         "empty array, that `stderr` is exactly empty, and that "
                         "`json[\"stdout\"]` contains `4\\n`. A nested `json` leaf "
                         "has no substring-assertion form in this case-file format "
                         "(only exact equality), so that claim is resolved to an "
                         "exact pin -- live-captured from the real `kali` binary, "
                         "never hand-computed, and strictly stronger than source's "
                         "`.contains()` check."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json mode, "
                       "asserts a clean exit and that stdout contains `4\\n`; a "
                       "plain `.contains` against a field that has a substring form, "
                       "so mirrored as `stdout_contains`.")
                    + " `ext` (js/ts/jsx/tsx) is hoisted to a file-level [matrix] "
                      "axis: 24 #[test] fns collapse to 6 [[case]] entries here, "
                      "matrix-fanned to 24 trials (see the file header's "
                      "arithmetic)."),
                'steps': [step]})
    return emit_case_file(os.path.join(CASES, 'math_exp2_global_this_root.toml'),
                          header, {'ext': EXTS}, sources, cases)


# ==================================================== exp2 zero identity =====

def gen_exp2_zero_identity():
    rs = src('browser_math_exp2_zero_identity.rs')
    run_body = fixture(rs, 'browser_harness_math_exp2_run_source')
    test_body = fixture(rs, 'browser_harness_math_exp2_test_source')
    live = {}
    for command, fname, body in (('run', 'main.js', run_body),
                                 ('test', 'smoke.test.js', test_body)):
        live[command] = live_stdout(
            fname, body, command,
            harness_args(command, fname, True, throttle=False))

    header = f"""Migrated from tests/browser_math_exp2_zero_identity.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): all 16 #[test] fns are individual (no loops) calls
to the single helper `assert_browser_harness_math_exp2(command, filename,
source, json_output)` = command(run/test) x ext(js/ts/jsx/tsx) x
json_output(2) = 16 invocations, uniform over `ext`. `ext` is hoisted to a
file-level [matrix] axis: 16 fns collapse to 4 [[case]] entries, matrix-fanned
to 16 trials, matching exactly. This file has no bundle group at all.

ASSERTION SHAPE. `exit = "success"`. Non-json: `stdout_contains = ["4\\n"]`,
plus `"ok 1"` on the "test" cases ONLY -- source guards that needle with
`if command == "test"`, so the "run" cases must not carry it (rule 2). Json:
schemaVersion/command/success/payload(hostContract, runtimeBackend), then
`json["exitCode"]`/`payload.exitCode` for "run" or
`payload.total/passed/failed/skipped` for "test" -- note this file also pins
`payload.skipped == 0`, which most of its siblings do not -- `stderr = ""`, and
an exact `json.stdout` pin resolving source's `json["stdout"].contains("4\\n")`,
live-captured from the real `kali` binary. This helper's argv carries no
`--max-threads`/`--max-spawned-processes` throttle; mirrored exactly.
"""
    sources = {'main.${ext}': run_body, 'smoke.test.${ext}': test_body}
    cases = []
    for command in ('run', 'test'):
        fname = 'main.${ext}' if command == 'run' else 'smoke.test.${ext}'
        for json_output in (False, True):
            step = {'args': harness_args(command, fname, json_output,
                                         throttle=False),
                    'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                payload = {'hostContract': 'browser-requested',
                           'runtimeBackend': 'browser-harness'}
                j = {'schemaVersion': 1, 'command': command, 'success': True,
                     'payload': payload}
                if command == 'run':
                    j['exitCode'] = 0
                    payload['exitCode'] = 0
                else:
                    payload['total'] = 1
                    payload['passed'] = 1
                    payload['failed'] = 0
                    payload['skipped'] = 0
                j['stdout'] = live[command]
                j['stderr'] = ''
                step['json'] = j
            else:
                step['stdout_contains'] = (['4\n', 'ok 1'] if command == 'test'
                                           else ['4\n'])
            prefix = 'json_' if json_output else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_math_exp2_zero_identity_'
                         'when_browser_harness_is_configured'),
                'rationale': (
                    "Migrated from browser_math_exp2_zero_identity.rs. "
                    f"`assert_browser_harness_math_exp2(\"{command}\", ...)` runs "
                    f"`kali {command} --api browser"
                    + (" --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`json[\"exitCode\"] == 0` and `payload.exitCode == 0`, "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed`/`skipped` == 1/1/0/0, ")
                       + "that `stderr` is exactly empty, and that "
                         "`json[\"stdout\"]` contains `4\\n`. A nested `json` leaf "
                         "has no substring-assertion form in this case-file format "
                         "(only exact equality), so that claim is resolved to an "
                         "exact pin -- live-captured from the real `kali` binary and "
                         "strictly stronger than source's `.contains()` check."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json mode, "
                       "asserts a clean exit and that stdout contains `4\\n`"
                       + (", plus `ok 1` -- source guards that second needle with "
                          "`if command == \"test\"`, so it is carried on the test "
                          "cases only" if command == 'test' else
                          "; source's second needle `ok 1` is guarded by "
                          "`if command == \"test\"` and is therefore NOT carried "
                          "here")
                       + ". Plain `.contains` against a field that has a substring "
                         "form, so mirrored as `stdout_contains`.")
                    + " This helper's argv carries no `--max-threads`/"
                      "`--max-spawned-processes` throttle; mirrored exactly. `ext` "
                      "(js/ts/jsx/tsx) is hoisted to a file-level [matrix] axis: 16 "
                      "#[test] fns collapse to 4 [[case]] entries here, "
                      "matrix-fanned to 16 trials (see the file header's "
                      "arithmetic)."),
                'steps': [step]})
    return emit_case_file(os.path.join(CASES, 'math_exp2_zero_identity.toml'),
                          header, {'ext': EXTS}, sources, cases)


# ================================================== exp/log identities =======

def gen_exp_log_identities():
    rs = src('browser_math_exp_log_identities.rs')
    run_body = fixture(rs, 'browser_harness_math_exp_log_run_source')
    test_body = fixture(rs, 'browser_harness_math_exp_log_test_source')
    live = {}
    for command, fname, body in (('run', 'main.js', run_body),
                                 ('test', 'smoke.test.js', test_body)):
        live[command] = live_stdout(
            fname, body, command,
            harness_args(command, fname, True, throttle=False))

    header = f"""Migrated from tests/browser_math_exp_log_identities.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): all 16 #[test] fns are individual (no loops) calls
to the single helper `assert_browser_harness_math_exp_log(command, filename,
source, json_output)` = command(run/test) x ext(js/ts/jsx/tsx) x
json_output(2) = 16 invocations, uniform over `ext`. `ext` is hoisted to a
file-level [matrix] axis: 16 fns collapse to 4 [[case]] entries, matrix-fanned
to 16 trials, matching exactly. This file has no bundle group.

ASSERTION SHAPE. `exit = "success"`. Non-json: `stdout_contains = ["1\\n",
"0\\n"]`, mirroring source's two plain `.contains` checks. Json:
schemaVersion/command/success/payload(hostContract, runtimeBackend), then
`json["exitCode"]`/`payload.exitCode` for "run" or
`payload.total/passed/failed` for "test", `stderr = ""`, and an exact
`json.stdout` pin resolving source's two `json["stdout"].contains(...)` claims
-- a nested `json` leaf has no substring form, so it is pinned exactly and only
after live-capturing the value from the real `kali` binary. This helper's argv
carries no `--max-threads`/`--max-spawned-processes` throttle, and it sets the
env var by its literal name rather than through
`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`; both are the same
`KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`, mirrored exactly.
"""
    sources = {'main.${ext}': run_body, 'smoke.test.${ext}': test_body}
    cases = []
    for command in ('run', 'test'):
        fname = 'main.${ext}' if command == 'run' else 'smoke.test.${ext}'
        for json_output in (False, True):
            step = {'args': harness_args(command, fname, json_output,
                                         throttle=False),
                    'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                payload = {'hostContract': 'browser-requested',
                           'runtimeBackend': 'browser-harness'}
                j = {'schemaVersion': 1, 'command': command, 'success': True,
                     'payload': payload}
                if command == 'run':
                    j['exitCode'] = 0
                    payload['exitCode'] = 0
                else:
                    payload['total'] = 1
                    payload['passed'] = 1
                    payload['failed'] = 0
                j['stdout'] = live[command]
                j['stderr'] = ''
                step['json'] = j
            else:
                step['stdout_contains'] = ['1\n', '0\n']
            prefix = 'json_' if json_output else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_math_exp_and_log_identity_'
                         'literals_when_browser_harness_is_configured'),
                'rationale': (
                    "Migrated from browser_math_exp_log_identities.rs. "
                    f"`assert_browser_harness_math_exp_log(\"{command}\", ...)` runs "
                    f"`kali {command} --api browser"
                    + (" --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`json[\"exitCode\"] == 0` and `payload.exitCode == 0`, "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed` == 1/1/0, ")
                       + "that `stderr` is exactly empty, and that "
                         "`json[\"stdout\"]` contains both `1\\n` and `0\\n`. A "
                         "nested `json` leaf has no substring-assertion form in this "
                         "case-file format (only exact equality), so that claim is "
                         "resolved to an exact pin -- live-captured from the real "
                         "`kali` binary and strictly stronger than source's two "
                         "`.contains()` checks."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json mode, "
                       "asserts a clean exit and that stdout contains both `1\\n` "
                       "and `0\\n`; plain `.contains` against a field that has a "
                       "substring form, so mirrored as `stdout_contains`.")
                    + " `ext` (js/ts/jsx/tsx) is hoisted to a file-level [matrix] "
                      "axis: 16 #[test] fns collapse to 4 [[case]] entries here, "
                      "matrix-fanned to 16 trials (see the file header's "
                      "arithmetic)."),
                'steps': [step]})
    return emit_case_file(os.path.join(CASES, 'math_exp_log_identities.toml'),
                          header, {'ext': EXTS}, sources, cases)


def _harness_json(command, stdout, errors=None):
    payload = {'hostContract': 'browser-requested',
               'runtimeBackend': 'browser-harness'}
    j = {'schemaVersion': 1, 'command': command, 'success': True,
         'payload': payload}
    if command == 'run':
        j['exitCode'] = 0
        payload['exitCode'] = 0
    else:
        payload['total'] = 1
        payload['passed'] = 1
        payload['failed'] = 0
    j['stdout'] = stdout
    j['stderr'] = ''
    if errors is not None:
        j['errors'] = errors
    return j


# ============================================= bracketed globalThis exp/log ==

def _inline_pairs(rs, fn_name, n_pairs):
    """The (command, filename, source, expected_stdout) tuples an inline
    `for (...) in [...]` test fn iterates, read out of the real literal stream
    in source order -- so the fixture bodies are copied, never retyped."""
    from extract import fixtures
    lits = fixtures(rs, fn_name)
    out = []
    for i in range(n_pairs):
        out.append(tuple(lits[i * 4:i * 4 + 4]))
    return out


def gen_exp_log_bracketed_root():
    rs = src('browser_math_exp_log_bracketed_root.rs')
    bundle_body = fixture(rs, 'browser_bundle_bracketed_global_this_math_exp_log_source')
    pairs = _inline_pairs(
        rs,
        'run_and_test_supports_bracketed_global_this_math_exp_log_identities_'
        'when_browser_harness_is_configured_in_js_and_ts_input', 4)
    assert [p[0] for p in pairs] == ['run', 'test', 'run', 'test'], pairs
    assert [p[1] for p in pairs] == ['main.js', 'smoke.test.js',
                                     'main.ts', 'smoke.test.ts'], pairs
    assert pairs[0][2] == pairs[2][2] and pairs[1][2] == pairs[3][2]
    assert len({p[3] for p in pairs}) == 1, pairs
    expected = pairs[0][3]

    live = {}
    for command, fname, body in (('run', 'main.js', pairs[0][2]),
                                 ('test', 'smoke.test.js', pairs[1][2])):
        live[command] = live_stdout(fname, body, command,
                                    harness_args(command, fname, True))
        assert expected in live[command], (command, live[command])

    header = f"""Migrated from tests/browser_math_exp_log_bracketed_root.rs.
{NO_COMMENTS}

NO [matrix] (rule 7 / U1): the two groups in this file do NOT cover the same
extensions. The bundle group
(`assert_browser_bundle_bracketed_global_this_math_exp_log`) has 8 individual
#[test] fns = ext(js/ts/jsx/tsx) x json_output(2). The harness group is a
SINGLE #[test] fn, `run_and_test_supports_..._in_js_and_ts_input`, whose body
loops over four `(command, source_name, source, expected_stdout)` tuples --
("run","main.js"), ("test","smoke.test.js"), ("run","main.ts"),
("test","smoke.test.ts"), i.e. js and ts ONLY, no jsx/tsx -- and over
`for output_json in [false, true]`, giving 4 x 2 = 8 more invocations. Total
8 + 8 = 16 invocations against 9 #[test] fns. A file-level `ext` axis would fan
the harness cases over jsx/tsx, inventing four untested combinations the source
never runs (rule 2), because `expand()` fans every [[case]] uniformly with no
per-case opt-out. Declined for the whole file: 16 named sibling [[case]]
entries, one per real invocation.

RULE 5 (split, don't fold): the single harness #[test] fn makes 8 independent
assertions over 8 independently-written programs, so it becomes 8 sibling
[[case]] entries named descriptively (not numbered), not one case.

[source] needs no disambiguation (U5): `app.<ext>` (bundle), `main.<ext>` and
`smoke.test.<ext>` (harness) are already distinct keys. Only the js/ts harness
fixtures the source actually writes are declared, so no unreferenced fixture
is emitted.

ASSERTION SHAPE. Bundle: `exit = "success"`; json mode
schemaVersion/command/success/exitCode/payload(artifactKind, bundleFormat)
(source makes no `errors` claim here, so none is added); the emitted
`app/app.meta.json` metadata; then the bundle-harness
`stdout_contains = ["1\\n", "0\\n"]`. Harness: `exit = "success"`; non-json
`stdout_contains` of the source's own `expected_stdout` literal
({expected!r}); json mode carries schemaVersion/command/success/
payload(hostContract, runtimeBackend), `json["exitCode"]`/`payload.exitCode`
for "run" or `payload.total/passed/failed` for "test", `stderr = ""`, and an
exact `json.stdout` pin resolving source's
`json["stdout"].contains(expected_stdout)` -- live-captured from the real
`kali` binary and checked to contain the source's own expected literal before
being written.
"""
    sources = {f'app.{ext}': bundle_body for ext in EXTS}
    sources['main.js'] = pairs[0][2]
    sources['smoke.test.js'] = pairs[1][2]
    sources['main.ts'] = pairs[2][2]
    sources['smoke.test.ts'] = pairs[3][2]

    cases = []
    for ext in EXTS:
        for json_output in (False, True):
            prefix = 'json_build_emits' if json_output else 'build_emits'
            cases.append({
                'name': (f'{prefix}_bracketed_global_this_math_exp_log_identity_'
                         f'literals_in_{ext}_input'),
                'rationale': (
                    "Migrated from browser_math_exp_log_bracketed_root.rs. "
                    "`assert_browser_bundle_bracketed_global_this_math_exp_log` "
                    "builds a browser bundle (`kali build --bundle --api browser"
                    + (" --output json`), asserts the JSON envelope's "
                       "schemaVersion/command/success/exitCode/payload "
                       "(artifactKind, bundleFormat) fields -- source makes no "
                       "`errors` claim here, so none is added -- then asserts"
                       if json_output else "`), asserts")
                    + " the emitted `app/app.meta.json` metadata, then runs the "
                      "bundle glue under the browser-bundle-harness contract and "
                      "checks the bracketed-root exp(0)/log(1) identity output "
                      "(`1` and `0`). Source's stdout claims are plain `.contains`, "
                      "mirrored as `stdout_contains`. No [matrix] in this file: the "
                      "bundle group covers all four extensions but the harness group "
                      "covers only js/ts, and `[matrix]` is file-wide with no "
                      "per-case opt-out (see the file header's arithmetic)."),
                'steps': bundle_steps('app', ext, json_output,
                                      'bracketedGlobalThisMathExpLogIdentities',
                                      ['1\n', '0\n'], assert_errors_empty=False)})
    for command, fname, body, exp in pairs:
        ext = fname.rsplit('.', 1)[1]
        for json_output in (False, True):
            step = {'args': harness_args(command, fname, json_output),
                    'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                step['json'] = _harness_json(command, live[command])
            else:
                step['stdout_contains'] = [exp]
            prefix = 'json_' if json_output else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_bracketed_global_this_math_'
                         f'exp_log_identities_when_browser_harness_is_configured_'
                         f'in_{ext}_input'),
                'rationale': (
                    "Migrated from browser_math_exp_log_bracketed_root.rs. This case "
                    "is one of the 8 invocations the single #[test] fn "
                    "`run_and_test_supports_bracketed_global_this_math_exp_log_"
                    "identities_when_browser_harness_is_configured_in_js_and_ts_"
                    "input` performs by looping over four "
                    "(command, source_name, source, expected_stdout) tuples and over "
                    "`for output_json in [false, true]`; per the split-don't-fold "
                    "rule each independent program gets its own sibling [[case]] "
                    f"rather than being folded. It runs `kali {command} --api browser"
                    + (" --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`json[\"exitCode\"] == 0` and `payload.exitCode == 0`, "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed` == 1/1/0, ")
                       + "that `stderr` is exactly empty, and that "
                         "`json[\"stdout\"]` contains the source's own "
                         f"`expected_stdout` literal {exp!r}. A nested `json` leaf "
                         "has no substring-assertion form in this case-file format "
                         "(only exact equality), so that claim is resolved to an "
                         "exact pin -- live-captured from the real `kali` binary and "
                         "checked to contain the source's literal before being "
                         "written."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json mode, "
                       f"asserts a clean exit and `stdout.contains({exp!r})`; a plain "
                       "`.contains` against a field that has a substring form, so "
                       "mirrored as `stdout_contains`.")
                    + " No [matrix] in this file: the harness group covers only js/ts "
                      "while the bundle group covers all four extensions, and "
                      "`[matrix]` is file-wide with no per-case opt-out."),
                'steps': [step]})
    return emit_case_file(os.path.join(CASES, 'math_exp_log_bracketed_root.toml'),
                          header, None, sources, cases)


# ======================================= fully bracketed globalThis exp/log ==

def gen_exp_log_fully_bracketed_root():
    rs = src('browser_math_exp_log_fully_bracketed_root.rs')
    bundle_body = fixture(
        rs, 'browser_bundle_fully_bracketed_global_this_math_exp_log_source')
    run_body = fixture(
        rs, 'browser_harness_fully_bracketed_global_this_math_exp_log_run_source')
    test_body = fixture(
        rs, 'browser_harness_fully_bracketed_global_this_math_exp_log_test_source')
    live = {}
    for command, fname, body in (('run', 'main.js', run_body),
                                 ('test', 'smoke.test.js', test_body)):
        live[command] = live_stdout(fname, body, command,
                                    harness_args(command, fname, True))

    header = f"""Migrated from tests/browser_math_exp_log_fully_bracketed_root.rs.
{NO_COMMENTS}

NO [matrix] (rule 7 / U1): the two groups do NOT cover the same extensions. The
bundle group (`assert_browser_bundle_fully_bracketed_global_this_math_exp_log`)
has 8 individual #[test] fns = ext(js/ts/jsx/tsx) x json_output(2). The harness
group is a SINGLE #[test] fn whose body loops over four
`(command, source_name, source)` tuples -- ("run","main.js"),
("test","smoke.test.js"), ("run","main.ts"), ("test","smoke.test.ts"), i.e.
js and ts ONLY -- and over `for output_json in [false, true]`, giving 8 more
invocations. 8 + 8 = 16 invocations against 9 #[test] fns. A file-level `ext`
axis would fan the harness cases over jsx/tsx and invent four untested
combinations (rule 2). Declined for the whole file: 16 named sibling [[case]]
entries, one per real invocation, per the split-don't-fold rule.

[source] needs no disambiguation (U5). Only the js/ts harness fixtures the
source actually writes are declared.

ASSERTION SHAPE. Bundle: `exit = "success"`; json mode
schemaVersion/command/success/exitCode/payload(artifactKind, bundleFormat)
(no `errors` claim in source, so none added); the emitted `app/app.meta.json`
metadata; then the bundle-harness `stdout_contains = ["1\\n", "0\\n"]`.
Harness: `exit = "success"`; non-json `stdout_contains = ["1\\n", "0\\n"]`;
json mode carries schemaVersion/command/success/payload(hostContract,
runtimeBackend), `json["exitCode"]`/`payload.exitCode` for "run" or
`payload.total/passed/failed` for "test", `stderr = ""`, and an exact
`json.stdout` pin resolving source's two `json["stdout"].contains(...)` claims,
live-captured from the real `kali` binary.
"""
    sources = {f'app.{ext}': bundle_body for ext in EXTS}
    sources['main.js'] = run_body
    sources['smoke.test.js'] = test_body
    sources['main.ts'] = run_body
    sources['smoke.test.ts'] = test_body

    cases = []
    for ext in EXTS:
        for json_output in (False, True):
            prefix = 'json_build_emits' if json_output else 'build_emits'
            cases.append({
                'name': (f'{prefix}_fully_bracketed_global_this_math_exp_log_'
                         f'identity_literals_in_{ext}_input'),
                'rationale': (
                    "Migrated from browser_math_exp_log_fully_bracketed_root.rs. "
                    "`assert_browser_bundle_fully_bracketed_global_this_math_exp_log`"
                    " builds a browser bundle (`kali build --bundle --api browser"
                    + (" --output json`), asserts the JSON envelope's "
                       "schemaVersion/command/success/exitCode/payload "
                       "(artifactKind, bundleFormat) fields -- source makes no "
                       "`errors` claim here, so none is added -- then asserts"
                       if json_output else "`), asserts")
                    + " the emitted `app/app.meta.json` metadata, then runs the "
                      "bundle glue under the browser-bundle-harness contract and "
                      "checks the fully-bracketed "
                      "`globalThis[\"Math\"][\"exp\"]`/`[\"log\"]` identity output "
                      "(`1` and `0`). Source's stdout claims are plain `.contains`, "
                      "mirrored as `stdout_contains`. No [matrix] in this file: the "
                      "bundle group covers all four extensions but the harness group "
                      "covers only js/ts (see the file header's arithmetic)."),
                'steps': bundle_steps(
                    'app', ext, json_output,
                    'fullyBracketedGlobalThisMathExpLogIdentities',
                    ['1\n', '0\n'], assert_errors_empty=False)})
    for command, fname in (('run', 'main.js'), ('test', 'smoke.test.js'),
                           ('run', 'main.ts'), ('test', 'smoke.test.ts')):
        ext = fname.rsplit('.', 1)[1]
        for json_output in (False, True):
            step = {'args': harness_args(command, fname, json_output),
                    'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                step['json'] = _harness_json(command, live[command])
            else:
                step['stdout_contains'] = ['1\n', '0\n']
            prefix = 'json_' if json_output else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_fully_bracketed_global_this_'
                         'math_exp_log_identities_when_browser_harness_is_'
                         f'configured_in_{ext}_input'),
                'rationale': (
                    "Migrated from browser_math_exp_log_fully_bracketed_root.rs. "
                    "This case is one of the 8 invocations the single #[test] fn "
                    "`run_and_test_supports_fully_bracketed_global_this_math_exp_log"
                    "_identities_when_browser_harness_is_configured_in_js_and_ts_"
                    "input` performs by looping over four (command, source_name, "
                    "source) tuples and over `for output_json in [false, true]`; per "
                    "the split-don't-fold rule each independent program gets its own "
                    "sibling [[case]]. It calls "
                    "`assert_browser_harness_fully_bracketed_global_this_math_exp_log"
                    f"`, which runs `kali {command} --api browser"
                    + (" --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`json[\"exitCode\"] == 0` and `payload.exitCode == 0`, "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed` == 1/1/0, ")
                       + "that `stderr` is exactly empty, and that "
                         "`json[\"stdout\"]` contains both `1\\n` and `0\\n`. A "
                         "nested `json` leaf has no substring-assertion form in this "
                         "case-file format (only exact equality), so that claim is "
                         "resolved to an exact pin -- live-captured from the real "
                         "`kali` binary and strictly stronger than source's two "
                         "`.contains()` checks."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json mode, "
                       "asserts a clean exit and that stdout contains both `1\\n` "
                       "and `0\\n`; plain `.contains` against a field that has a "
                       "substring form, so mirrored as `stdout_contains`.")
                    + " No [matrix] in this file: the harness group covers only "
                      "js/ts while the bundle group covers all four extensions."),
                'steps': [step]})
    return emit_case_file(
        os.path.join(CASES, 'math_exp_log_fully_bracketed_root.toml'),
        header, None, sources, cases)


# ================================================ bracketed root core suite ==

def gen_bracketed_root_core_suite():
    rs = src('browser_math_bracketed_root_core_suite.rs')
    bundle_body = fixture(
        rs, 'browser_bundle_bracketed_global_this_math_core_suite_source')
    pairs = _inline_pairs(
        rs,
        'run_and_test_supports_bracketed_global_this_math_core_suite_when_'
        'browser_harness_is_configured_in_js_and_ts_input', 8)
    assert [p[0] for p in pairs] == ['run', 'test'] * 4, pairs
    assert [p[1] for p in pairs] == [
        'main.js', 'smoke.test.js', 'main.ts', 'smoke.test.ts',
        'main.jsx', 'smoke.test.jsx', 'main.tsx', 'smoke.test.tsx'], pairs
    run_bodies = {p[2] for p in pairs if p[0] == 'run'}
    test_bodies = {p[2] for p in pairs if p[0] == 'test'}
    assert len(run_bodies) == 1 and len(test_bodies) == 1, 'bodies differ'
    assert len({p[3] for p in pairs}) == 1, pairs
    run_body, test_body, expected = (run_bodies.pop(), test_bodies.pop(),
                                     pairs[0][3])

    live = {}
    for command, fname, body in (('run', 'main.js', run_body),
                                 ('test', 'smoke.test.js', test_body)):
        live[command] = live_stdout(fname, body, command,
                                    harness_args(command, fname, True))
        assert expected in live[command], (command, live[command])

    header = f"""Migrated from tests/browser_math_bracketed_root_core_suite.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): 8 bundle fns
(`assert_browser_bundle_bracketed_global_this_math_core_suite`, ext(4) x
json_output(2), all individual) + one harness #[test] fn whose body loops over
EIGHT `(command, source_name, source, expected_stdout)` tuples -- run/test for
each of js/ts/jsx/tsx -- and over `for output_json in [false, true]`, i.e.
8 x 2 = 16 invocations. Total 8 + 16 = 24 invocations against 9 #[test] fns.
Unlike this file's `exp_log` siblings, the harness tuple list here DOES cover
all four extensions, so both groups vary uniformly over `ext` and the axis is
safe: `ext` is hoisted to a file-level [matrix] axis and the 24 invocations
become 6 [[case]] entries (2 bundle + 4 harness) matrix-fanned to 24 trials,
matching exactly. Checked against the real tuple list, not inferred.

RULE 5 (split, don't fold): the harness #[test] fn's 16 invocations become 4
matrix-fanned sibling [[case]] entries (run/test x plain/json), one per
independent invocation shape, rather than one case doing everything.

[source] needs no disambiguation (U5): `app.<ext>`, `main.<ext>` and
`smoke.test.<ext>` are already distinct keys, and the run/test fixture bodies
were asserted byte-identical across all four extensions before being collapsed
onto a single `${{ext}}`-keyed entry (mechanical identity check in the
generator, not eyeballed).

ASSERTION SHAPE. Bundle: `exit = "success"`; json mode
schemaVersion/command/success/exitCode/payload(artifactKind, bundleFormat)
(source makes no `errors` claim here, so none is added); the emitted
`app/app.meta.json` metadata; then the bundle-harness five-needle
`stdout_contains = ["3\\n", "1\\n", "-2\\n", "31\\n", "-1\\n"]`, mirroring
source's five plain `.contains` checks in order. Harness: `exit = "success"`;
non-json `stdout_contains` of the source's own `expected_stdout` literal
({expected!r}); json mode carries schemaVersion/command/success/
payload(hostContract, runtimeBackend), `json["exitCode"]`/`payload.exitCode`
for "run" or `payload.total/passed/failed` for "test", `stderr = ""`, and an
exact `json.stdout` pin resolving source's
`json["stdout"].contains(expected_stdout)`, live-captured from the real `kali`
binary and checked to contain the source's own literal before being written.
"""
    sources = {'app.${ext}': bundle_body,
               'main.${ext}': run_body,
               'smoke.test.${ext}': test_body}
    cases = []
    for json_output in (False, True):
        prefix = 'json_build_emits' if json_output else 'build_emits'
        cases.append({
            'name': f'{prefix}_bracketed_global_this_math_core_suite',
            'rationale': (
                "Migrated from browser_math_bracketed_root_core_suite.rs. "
                "`assert_browser_bundle_bracketed_global_this_math_core_suite` "
                "builds a browser bundle (`kali build --bundle --api browser"
                + (" --output json`), asserts the JSON envelope's "
                   "schemaVersion/command/success/exitCode/payload (artifactKind, "
                   "bundleFormat) fields -- source makes no `errors` claim here, so "
                   "none is added -- then asserts"
                   if json_output else "`), asserts")
                + " the emitted `app/app.meta.json` metadata, then runs the bundle "
                  "glue under the browser-bundle-harness contract and checks all "
                  "five needles source checks for the bracketed "
                  "max/min/abs/sign/imul/clz32 suite (`3\\n`, `1\\n`, `-2\\n`, "
                  "`31\\n`, `-1\\n`). Source's stdout claims are plain `.contains`, "
                  "mirrored as `stdout_contains` in source order. `ext` "
                  "(js/ts/jsx/tsx) is hoisted to a file-level [matrix] axis: 24 real "
                  "invocations across 9 #[test] fns collapse to 6 [[case]] entries "
                  "here, matrix-fanned to 24 trials (see the file header's "
                  "arithmetic)."),
            'steps': bundle_steps('app', '${ext}', json_output,
                                  'bracketedGlobalThisMathCoreSuite',
                                  ['3\n', '1\n', '-2\n', '31\n', '-1\n'],
                                  assert_errors_empty=False)})
    for command in ('run', 'test'):
        fname = 'main.${ext}' if command == 'run' else 'smoke.test.${ext}'
        for json_output in (False, True):
            step = {'args': harness_args(command, fname, json_output),
                    'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                step['json'] = _harness_json(command, live[command])
            else:
                step['stdout_contains'] = [expected]
            prefix = 'json_' if json_output else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_bracketed_global_this_math_'
                         'core_suite_when_browser_harness_is_configured'),
                'rationale': (
                    "Migrated from browser_math_bracketed_root_core_suite.rs. This "
                    "case is one of the 16 invocations the single #[test] fn "
                    "`run_and_test_supports_bracketed_global_this_math_core_suite_"
                    "when_browser_harness_is_configured_in_js_and_ts_input` performs "
                    "by looping over eight (command, source_name, source, "
                    "expected_stdout) tuples and over `for output_json in [false, "
                    "true]`; per the split-don't-fold rule each independent "
                    "invocation shape gets its own sibling [[case]] rather than "
                    f"being folded. It runs `kali {command} --api browser"
                    + (" --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`json[\"exitCode\"] == 0` and `payload.exitCode == 0`, "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed` == 1/1/0, ")
                       + "that `stderr` is exactly empty, and that "
                         "`json[\"stdout\"]` contains the source's own "
                         f"`expected_stdout` literal {expected!r}. A nested `json` "
                         "leaf has no substring-assertion form in this case-file "
                         "format (only exact equality), so that claim is resolved to "
                         "an exact pin -- live-captured from the real `kali` binary "
                         "and checked to contain the source's literal before being "
                         "written."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json mode, "
                       "asserts a clean exit and "
                       f"`stdout.contains({expected!r})`; a plain `.contains` against "
                       "a field that has a substring form, so mirrored as "
                       "`stdout_contains`.")
                    + " `ext` (js/ts/jsx/tsx) is hoisted to a file-level [matrix] "
                      "axis -- safe here because this file's harness tuple list "
                      "genuinely covers all four extensions, unlike its `exp_log` "
                      "siblings (see the file header's arithmetic)."),
                'steps': [step]})
    return emit_case_file(os.path.join(CASES, 'math_bracketed_root_core_suite.toml'),
                          header, {'ext': EXTS}, sources, cases)


# =========================================== atan2 trailing argument bundle ==

def gen_atan2_trailing_bundle():
    rs = src('browser_math_atan2_trailing_argument_evaluation_bundle.rs')
    body = fixture(rs, 'browser_bundle_atan2_trailing_argument_evaluation_source')
    header = f"""Migrated from
tests/browser_math_atan2_trailing_argument_evaluation_bundle.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): all 8 #[test] fns are individual (no loops
anywhere in this file) calls to the single helper
`assert_browser_bundle_atan2_trailing_argument_evaluation(filename,
json_output)` = ext(js/ts/jsx/tsx) x json_output(2) = 8 invocations, uniform
over `ext`. `ext` is hoisted to a file-level [matrix] axis: 8 fns collapse to 2
[[case]] entries, matrix-fanned to 8 trials, matching exactly.

[source] needs no disambiguation (U5): one fixture body, named `app.<ext>` in
source, kept as-is.

ASSERTION SHAPE. `exit = "success"`; json mode carries
schemaVersion/command/success/exitCode/payload(artifactKind, bundleFormat) AND
`errors = []` (this file's helper does assert
`envelope["errors"].is_empty()`); then the emitted `app/app.meta.json`
metadata; then the bundle-harness `stdout_contains = ["bump", "0\\n"]`,
mirroring source's two plain `.contains` checks -- `bump` proves the
extra trailing argument's side effect ran, `0\\n` is `Math.atan2(0, 1)`.
"""
    sources = {'app.${ext}': body}
    cases = []
    for json_output in (False, True):
        prefix = 'json_build_supports' if json_output else 'build_supports'
        cases.append({
            'name': f'{prefix}_math_atan2_trailing_argument_evaluation',
            'rationale': (
                "Migrated from "
                "browser_math_atan2_trailing_argument_evaluation_bundle.rs. "
                "`assert_browser_bundle_atan2_trailing_argument_evaluation` builds a "
                "browser bundle (`kali build --bundle --api browser"
                + (" --output json`), asserts the JSON envelope's "
                   "schemaVersion/command/success/exitCode/payload (artifactKind, "
                   "bundleFormat) fields and that `errors` is empty, then asserts"
                   if json_output else "`), asserts")
                + " the emitted `app/app.meta.json` metadata, then runs the bundle "
                  "glue under the browser-bundle-harness contract and checks that "
                  "the surplus trailing argument to `Math.atan2(0, 1, bump())` was "
                  "still evaluated (`bump` printed) and that the call itself "
                  "returned `0`. Source's stdout claims are plain `.contains`, "
                  "mirrored as `stdout_contains`. `ext` (js/ts/jsx/tsx) is hoisted "
                  "to a file-level [matrix] axis: 8 #[test] fns collapse to 2 "
                  "[[case]] entries here, matrix-fanned to 8 trials (see the file "
                  "header's arithmetic)."),
            'steps': bundle_steps('app', '${ext}', json_output,
                                  'atan2TrailingArgumentEvaluation',
                                  ['bump', '0\n'], assert_errors_empty=True)})
    return emit_case_file(
        os.path.join(CASES, 'math_atan2_trailing_argument_evaluation_bundle.toml'),
        header, {'ext': EXTS}, sources, cases)


# ========================================== atan2 trailing argument harness ==

def gen_atan2_trailing_harness():
    rs = src('browser_math_atan2_trailing_argument_evaluation_harness.rs')
    lits = None
    from extract import fixtures
    lits = fixtures(rs, 'atan2_trailing_argument_source')
    # lits[0] is the `match` arm's `"test"` pattern literal, not a fixture.
    assert lits[0] == 'test', lits[0]
    assert len(lits) == 3, lits
    test_body, run_body = lits[1], lits[2]
    assert test_body.startswith('Kali.test('), test_body
    assert run_body.startswith('const bump'), run_body
    exts = ['ts', 'jsx', 'tsx']

    live = {}
    for command, fname, body in (('run', 'main.ts', run_body),
                                 ('test', 'smoke.test.ts', test_body)):
        live[command] = live_stdout(fname, body, command,
                                    harness_args(command, fname, True))
        assert 'bump' in live[command] and '0' in live[command], live[command]

    header = f"""Migrated from
tests/browser_math_atan2_trailing_argument_evaluation_harness.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): 6 #[test] fns, each one call to
`assert_browser_harness_atan2_trailing_argument_evaluation(command, filename)`,
whose body loops `for output_json in [false, true]` -- so each fn is TWO real
invocations, not one. 6 x 2 = 12 invocations = command(run/test) x
ext(ts/jsx/tsx) x json_output(2). Note the extension axis here is THREE values,
not four: this file has no `_js_input` fn at all (verified against the real fn
list). `ext` is hoisted to a file-level [matrix] axis over ["ts","jsx","tsx"]:
6 fns / 12 invocations collapse to 4 [[case]] entries, matrix-fanned to 12
trials, matching exactly.

[source] SELECTION (U5 / U2): `atan2_trailing_argument_source(command)` is a
`match` returning the `Kali.test`-wrapped body for "test" and the bare-script
body otherwise, written to `main.<ext>` or `smoke.test.<ext>` respectively --
already distinct keys, so no rename is needed and no conditional-fixture
hazard arises.

ASSERTION SHAPE. `exit = "success"` folds both
`assert!(output.status.success())` and `assert_eq!(output.status.code(),
Some(0))`. Non-json: `stdout_contains = ["bump", "0"]`, mirroring source's
`stdout.contains("bump")` and `stdout.contains('0')` -- note the second needle
is a CHAR literal `'0'` in source, carried as the one-character string. Json:
schemaVersion/command/success/payload(hostContract, runtimeBackend), then
`payload.exitCode == 0` for "run" -- this file asserts ONLY the nested
`payload.exitCode`, never the top-level `json["exitCode"]`, unlike most of its
siblings, and nothing is added -- or `payload.total/passed/failed` for "test";
`stderr = ""`, `errors = []`, and an exact `json.stdout` pin resolving source's
two `.contains` claims on that leaf, live-captured from the real `kali` binary.
"""
    sources = {'main.${ext}': run_body, 'smoke.test.${ext}': test_body}
    cases = []
    for command in ('run', 'test'):
        fname = 'main.${ext}' if command == 'run' else 'smoke.test.${ext}'
        for json_output in (False, True):
            step = {'args': harness_args(command, fname, json_output),
                    'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                payload = {'hostContract': 'browser-requested',
                           'runtimeBackend': 'browser-harness'}
                j = {'schemaVersion': 1, 'command': command, 'success': True,
                     'payload': payload}
                if command == 'run':
                    payload['exitCode'] = 0
                else:
                    payload['total'] = 1
                    payload['passed'] = 1
                    payload['failed'] = 0
                j['stdout'] = live[command]
                j['stderr'] = ''
                j['errors'] = []
                step['json'] = j
            else:
                step['stdout_contains'] = ['bump', '0']
            prefix = 'json_' if json_output else ''
            cases.append({
                'name': (f'{prefix}{command}_supports_math_atan2_trailing_argument_'
                         'evaluation_when_browser_harness_is_configured'),
                'rationale': (
                    "Migrated from "
                    "browser_math_atan2_trailing_argument_evaluation_harness.rs. "
                    "`assert_browser_harness_atan2_trailing_argument_evaluation("
                    f"\"{command}\", ...)` loops `for output_json in [false, true]` "
                    "inside its body, so each of the 6 source #[test] fns is two real "
                    f"invocations. This one runs `kali {command} --api browser"
                    + (" --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`payload.exitCode == 0` -- this file asserts only the "
                          "nested payload field, never the top-level "
                          "`json[\"exitCode\"]`, so none is added -- "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed` == 1/1/0, ")
                       + "that `stderr` is exactly empty, that `errors` is empty, and "
                         "that `json[\"stdout\"]` contains both `bump` and `0`. A "
                         "nested `json` leaf has no substring-assertion form in this "
                         "case-file format (only exact equality), so that claim is "
                         "resolved to an exact pin -- live-captured from the real "
                         "`kali` binary and strictly stronger than source's two "
                         "`.contains()` checks."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json mode, "
                       "asserts a clean exit and that stdout contains both `bump` "
                       "(proving the surplus trailing argument's side effect still "
                       "ran) and `0` (the `Math.atan2(0, 1)` result); the second "
                       "needle is a char literal `'0'` in source, carried as the "
                       "one-character string. Plain `.contains` against a field that "
                       "has a substring form, so mirrored as `stdout_contains`.")
                    + " `exit = \"success\"` folds both "
                      "`assert!(output.status.success())` and "
                      "`assert_eq!(output.status.code(), Some(0))`. `ext` is hoisted "
                      "to a file-level [matrix] axis over THREE values "
                      "(ts/jsx/tsx) -- this file has no `_js_input` fn at all -- so "
                      "6 #[test] fns / 12 invocations collapse to 4 [[case]] entries "
                      "here, matrix-fanned to 12 trials (see the file header's "
                      "arithmetic)."),
                'steps': [step]})
    return emit_case_file(
        os.path.join(CASES, 'math_atan2_trailing_argument_evaluation_harness.toml'),
        header, {'ext': exts}, sources, cases)


# ============================================== atan2 globalThis root ========

ATAN2_BUNDLE_VARIANTS = {
    'zero_slice': ('browser_bundle_global_this_math_atan2_source',
                   'globalThisMathAtan2ZeroSlice'),
    'frozen': ('browser_bundle_global_this_math_atan2_frozen_source',
               'globalThisMathAtan2FrozenCallableAliases'),
    'await_wrapped': ('browser_bundle_global_this_math_atan2_await_wrapped_source',
                      'globalThisMathAtan2AwaitWrappedZeroSlice'),
}
ATAN2_HARNESS_VARIANTS = {
    'zero_slice': ('browser_harness_global_this_math_atan2_run_source',
                   'browser_harness_global_this_math_atan2_test_source'),
    'frozen': ('browser_harness_global_this_math_atan2_frozen_run_source',
               'browser_harness_global_this_math_atan2_frozen_test_source'),
    'await_wrapped': (
        'browser_harness_global_this_math_atan2_await_wrapped_run_source',
        'browser_harness_global_this_math_atan2_await_wrapped_test_source'),
}


def gen_atan2_global_this_root():
    rs = src('browser_math_atan2_global_this_root.rs')
    bundle_bodies = {k: fixture(rs, fn) for k, (fn, _e) in ATAN2_BUNDLE_VARIANTS.items()}
    harness_bodies = {}
    for k, (run_fn, test_fn) in ATAN2_HARNESS_VARIANTS.items():
        harness_bodies[(k, 'run')] = fixture(rs, run_fn)
        harness_bodies[(k, 'test')] = fixture(rs, test_fn)

    live = {}
    for k in ATAN2_HARNESS_VARIANTS:
        for command in ('run', 'test'):
            fname = (f'main_{k}.js' if command == 'run'
                     else f'smoke_{k}.test.js')
            live[(k, command)] = live_stdout(
                fname, harness_bodies[(k, command)], command,
                harness_args(command, fname, True))
            assert '0\n' in live[(k, command)], live[(k, command)]

    header = f"""Migrated from tests/browser_math_atan2_global_this_root.rs.
{NO_COMMENTS}

PARTIAL MIGRATION -- 18 of this file's 19 #[test] fns are migrated here. The
19th, `browser_bundle_global_this_math_atan2_frozen_source_includes_direct_
frozen_callable_aliases`, is a FIXTURE SELF-INSPECTION test: it runs three
`assert!(source.contains("Object.freeze(...)"))` checks (one of them itself an
OR across two quoting spellings) against the JS fixture's own text and never
builds a command at all. That shape is invisible to
`scripts/audit-case-migration.py` -- its `.contains()` extractor cannot tell a
fixture-text read from an output assertion, and everything under `[source]` is
excluded from its search by construction -- so migrating it would produce a
false green. It stays hand-written and is escalated per rule 3/4; see the
retention header on the `.rs` and the batch report. No other fn in this file
reaches that construct (the other 18 route through
`assert_browser_bundle_global_this_math_atan2{{,_source,_await_wrapped}}` or
`assert_browser_harness_global_this_math_atan2`, none of which reads fixture
text), so U4's trim-and-keep applies and only that one test is retained.

INVOCATION ARITHMETIC (rule 7), over the migrated 18. Every fn except
`build_emits_global_this_math_atan2_zero_slice_in_js_input` loops
`for filename in ["app.js","app.ts","app.jsx","app.tsx"]` or over a
four-element `(filename, source)` tuple list, i.e. 4 real invocations each;
that one fn makes a single `("app.js", false)` call. 1 + (17 x 4) = 69
invocations, expanded by reading every loop rather than by counting fn names.
This file maps to 69 named sibling [[case]] entries below, one per real
invocation, no folding.

[matrix] DECLINED (rule 7 / U1). `ext` looks uniform at a glance -- 68 of the
69 invocations are a clean 17 x 4 fan over js/ts/jsx/tsx -- but
`build_emits_global_this_math_atan2_zero_slice_in_js_input` is js-ONLY, and
`[matrix]` is file-wide: `expand()` fans EVERY [[case]] by the full
cross-product with no per-case opt-out. A file-level `ext` axis would fan that
one case to ts/jsx/tsx as well, producing three duplicate trials that no source
fn performs and breaking rule 7's `total invocations == cases x axis product`
arithmetic. Declined for the whole file, on the same reasoning as batch 2's
`array_iteration_spread_runtime`.

VARIANT COVERAGE IS ALSO ASYMMETRIC, so no `variant` axis is possible either:
`frozen` has a `run` harness fn only in NON-json mode -- there is no
`json_run_supports_global_this_math_atan2_frozen_callable_aliases_*` fn in
source, while `zero_slice` and `await_wrapped` both have one. Verified against
the real fn list. That missing combination is not invented here (rule 2).

[source] KEY DISAMBIGUATION (U5): source writes all three bundle fixtures to
`app.<ext>` and all six harness fixtures to `main.<ext>`/`smoke.test.<ext>`
(safe there -- private tempdir per invocation). The flat file-wide `[source]`
table cannot, so the stems become `app_<variant>`, `main_<variant>` and
`smoke_<variant>.test`; because `kali build --bundle` names its output
directory after the input stem, the `file_json` `path` and the harness `entry`
track the rename. No fixture body references its own filename by string, so no
claim changes.

RULE 8: the harness `body` for the zero_slice and frozen bundle cases comes
from `format!("const mod = await import(bundleJs.href);\\nawait
mod.{{export_name}}();\\n")`; both resolved strings were produced by EXECUTING
that real `format!`, not by hand-substitution. The await_wrapped helper uses a
plain raw-string literal with no substitution at all.

ASSERTION SHAPE. Bundle: `exit = "success"`; json mode
schemaVersion/command/success/exitCode/payload(artifactKind, bundleFormat) and
`errors = []`; the emitted `app_<variant>/app_<variant>.meta.json` metadata;
then the bundle-harness `stdout_contains = ["0\\n"]`. Harness:
`exit = "success"`; non-json `stdout_contains = ["0\\n"]`; json mode carries
schemaVersion/command/success/payload(hostContract, runtimeBackend), then
`json["exitCode"]`/`payload.exitCode` for "run" or
`payload.total/passed/failed` for "test", `stderr = ""`, `errors = []`, and an
exact `json.stdout` pin resolving source's `json["stdout"].contains("0\\n")` --
live-captured from the real `kali` binary per (variant, command).
"""
    sources = {}
    for k in ATAN2_BUNDLE_VARIANTS:
        for ext in EXTS:
            sources[f'app_{k}.{ext}'] = bundle_bodies[k]
    for k in ATAN2_HARNESS_VARIANTS:
        for ext in EXTS:
            sources[f'main_{k}.{ext}'] = harness_bodies[(k, 'run')]
            sources[f'smoke_{k}.test.{ext}'] = harness_bodies[(k, 'test')]

    common = (
        "No [matrix] in this file: 68 of its 69 real invocations fan uniformly over "
        "js/ts/jsx/tsx, but `build_emits_global_this_math_atan2_zero_slice_in_js_"
        "input` is js-only and `[matrix]` is file-wide with no per-case opt-out, so "
        "an `ext` axis would produce three duplicate trials no source fn performs. "
        "Variant coverage is asymmetric too (`frozen` has no json-mode `run` harness "
        "fn), so no variant axis is possible either. 69 named sibling [[case]] "
        "entries, one per real invocation. `[source]` stems are disambiguated per "
        "variant because source reuses `app.<ext>`/`main.<ext>`/`smoke.test.<ext>` "
        "across all of them and the `[source]` table is one flat file-wide "
        "namespace; the `file_json` path and harness `entry` track the rename "
        "because `kali build --bundle` names its output directory after the input "
        "stem. This file's 19th fn, `browser_bundle_global_this_math_atan2_frozen_"
        "source_includes_direct_frozen_callable_aliases`, is a fixture "
        "self-inspection test and stays hand-written per rule 3/4."
    )

    def bundle_case(name, variant, ext, json_output):
        _fn, export = ATAN2_BUNDLE_VARIANTS[variant]
        helper = ('assert_browser_bundle_global_this_math_atan2_await_wrapped'
                  if variant == 'await_wrapped'
                  else 'assert_browser_bundle_global_this_math_atan2_source')
        return {
            'name': name,
            'rationale': (
                "Migrated from browser_math_atan2_global_this_root.rs. "
                f"`{helper}` builds a browser bundle (`kali build --bundle --api "
                "browser"
                + (" --output json`), asserts the JSON envelope's "
                   "schemaVersion/command/success/exitCode/payload (artifactKind, "
                   "bundleFormat) fields and that `errors` is empty, then asserts"
                   if json_output else "`), asserts")
                + f" the emitted `app_{variant}/app_{variant}.meta.json` metadata, "
                  "then runs the bundle glue under the browser-bundle-harness "
                  f"contract and checks that the {variant.replace('_', ' ')} "
                  "`globalThis`-rooted `Math.atan2` spellings printed `0`. Source's "
                  "stdout claim is a plain `.contains`, mirrored as "
                  "`stdout_contains`. " + common),
            'steps': bundle_steps(f'app_{variant}', ext, json_output, export,
                                  ['0\n'], assert_errors_empty=True)}

    def harness_case(name, variant, command, ext, json_output):
        fname = (f'main_{variant}.{ext}' if command == 'run'
                 else f'smoke_{variant}.test.{ext}')
        step = {'args': harness_args(command, fname, json_output),
                'env': dict(NODE_ENV), 'exit': 'success'}
        if json_output:
            step['json'] = _harness_json(command, live[(variant, command)],
                                         errors=[])
        else:
            step['stdout_contains'] = ['0\n']
        return {
            'name': name,
            'rationale': (
                "Migrated from browser_math_atan2_global_this_root.rs. "
                f"`assert_browser_harness_global_this_math_atan2(\"{command}\", ...)` "
                f"runs `kali {command} --api browser"
                + (" --output json` and asserts the JSON envelope's "
                   "schemaVersion/command/success/payload (hostContract, "
                   "runtimeBackend) fields, "
                   + ("`json[\"exitCode\"] == 0` and `payload.exitCode == 0`, "
                      if command == 'run' else
                      "`payload.total`/`passed`/`failed` == 1/1/0, ")
                   + "that `stderr` is exactly empty, that `errors` is empty, and "
                     "that `json[\"stdout\"]` contains `0\\n`. A nested `json` leaf "
                     "has no substring-assertion form in this case-file format (only "
                     "exact equality), so that claim is resolved to an exact pin -- "
                     "live-captured from the real `kali` binary and strictly stronger "
                     "than source's `.contains()` check."
                   if json_output else
                   "` under the browser harness (`node`) and, in non-json mode, "
                   "asserts a clean exit and that stdout contains `0\\n`; a plain "
                   "`.contains` against a field that has a substring form, so "
                   "mirrored as `stdout_contains`.")
                + f" The fixture is the {variant.replace('_', ' ')} variant. "
                + common),
            'steps': [step]}

    cases = []
    # -- bundle group, in source fn order ------------------------------------
    cases.append(bundle_case(
        'build_emits_global_this_math_atan2_zero_slice_in_js_input',
        'zero_slice', 'js', False))
    for variant, json_output, stem in (
            ('zero_slice', False, 'build_emits_global_this_math_atan2_zero_slice_in_js_like_input'),
            ('zero_slice', True, 'json_build_emits_global_this_math_atan2_zero_slice_in_js_like_input'),
            ('frozen', False, 'build_emits_global_this_math_atan2_frozen_callable_aliases_in_js_like_input'),
            ('frozen', True, 'json_build_emits_global_this_math_atan2_frozen_callable_aliases_in_js_like_input'),
            ('await_wrapped', False, 'build_emits_global_this_math_atan2_await_wrapped_zero_slice_in_js_like_input'),
            ('await_wrapped', True, 'json_build_emits_global_this_math_atan2_await_wrapped_zero_slice_in_js_like_input')):
        for ext in EXTS:
            cases.append(bundle_case(f'{stem}_{ext}', variant, ext, json_output))
    # -- harness group, in source fn order -----------------------------------
    for variant, command, json_output, stem in (
            ('zero_slice', 'run', False, 'run_supports_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_js_like_input'),
            ('frozen', 'run', False, 'run_supports_global_this_math_atan2_frozen_callable_aliases_when_browser_harness_is_configured_in_js_like_input'),
            ('zero_slice', 'test', False, 'test_supports_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_js_like_input'),
            ('frozen', 'test', False, 'test_supports_global_this_math_atan2_frozen_callable_aliases_when_browser_harness_is_configured_in_js_like_input'),
            ('zero_slice', 'run', True, 'run_supports_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_json_js_like_input'),
            ('zero_slice', 'test', True, 'test_supports_global_this_math_atan2_zero_slice_when_browser_harness_is_configured_in_json_js_like_input'),
            ('frozen', 'test', True, 'test_supports_global_this_math_atan2_frozen_callable_aliases_when_browser_harness_is_configured_in_json_js_like_input'),
            ('await_wrapped', 'run', False, 'run_supports_global_this_math_atan2_await_wrapped_zero_slice_when_browser_harness_is_configured_in_js_like_input'),
            ('await_wrapped', 'test', False, 'test_supports_global_this_math_atan2_await_wrapped_zero_slice_when_browser_harness_is_configured_in_js_like_input'),
            ('await_wrapped', 'run', True, 'run_supports_global_this_math_atan2_await_wrapped_zero_slice_when_browser_harness_is_configured_in_json_js_like_input'),
            ('await_wrapped', 'test', True, 'test_supports_global_this_math_atan2_await_wrapped_zero_slice_when_browser_harness_is_configured_in_json_js_like_input')):
        for ext in EXTS:
            cases.append(harness_case(f'{stem}_{ext}', variant, command, ext,
                                      json_output))
    assert len(cases) == 69, len(cases)
    assert len({c['name'] for c in cases}) == 69, 'duplicate case name'
    return emit_case_file(os.path.join(CASES, 'math_atan2_global_this_root.toml'),
                          header, None, sources, cases)


if __name__ == '__main__':
    total = 0
    for fn in (gen_abs_sign, gen_clz32, gen_exp2_global_this,
               gen_exp2_zero_identity, gen_exp_log_identities,
               gen_exp_log_bracketed_root, gen_exp_log_fully_bracketed_root,
               gen_bracketed_root_core_suite, gen_atan2_trailing_bundle,
               gen_atan2_trailing_harness, gen_atan2_global_this_root):
        total += fn()
    print(f'math group: {total} trials')
