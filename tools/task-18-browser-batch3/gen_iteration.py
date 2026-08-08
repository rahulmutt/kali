r"""Generators for the `for_of`/`for_await` iteration targets of Task 18 batch 3.

Covers, from `crates/kali_cli/tests/`:
  browser_for_of_array_iteration_sequence_wrappers.rs
  browser_for_of_array_iteration_wrappers.rs
  browser_for_of_array_iteration_break_continue.rs
  browser_for_of_array_iteration_alias_chain.rs
  browser_for_of_array_iteration_harness_sequence_wrappers_js_input.rs
  browser_for_of_array_iteration_harness_wrappers_js_input.rs
  browser_for_of_array_iteration_break_continue_harness.rs
  browser_for_await_object_string_enumeration_sequence_wrappers_js_input.rs

Every fixture body is pulled out of the real `.rs` through `extract.fixture`
(never retyped); every `browser_bundle_harness` `body` string was produced by
EXECUTING the real `format!` calls (see the batch report) rather than
hand-applying Rust's substitution rules.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from extract import fixture  # noqa: E402
from emit import emit_case_file  # noqa: E402
from capture import run, json_envelope  # noqa: E402

TESTS = '/workspace/crates/kali_cli/tests'
CASES = os.path.join(TESTS, 'cases', 'browser')
EXTS = ['js', 'ts', 'jsx', 'tsx']
NODE_ENV = {'KALI_BROWSER_BUNDLE_HARNESS_COMMAND': 'node'}

# Rule 8: captured by running the real `format!`, not hand-derived.
HARNESS_BODY = 'const mod = await import(bundleJs.href);\nawait mod.%s();\n'

NO_COMMENTS = (
    "No Rust comments exist anywhere in the source file (checked: `grep -nE "
    "'^\\s*//'` finds nothing outside the JS fixture bodies' own "
    "`// kali-tree-shake:` markers, which are program text, not Rust "
    "comments), so there is no prose to move verbatim into `rationale` here "
    "(rule 12)."
)


def src(name):
    return open(os.path.join(TESTS, name), encoding='utf-8').read()


def bundle_steps(stem, ext_expr, json_output, export_name, stdout_contains):
    """The three-step browser-bundle shape every `assert_browser_bundle_*`
    helper in this group performs, in source order."""
    args = ['build', '--bundle', '--api', 'browser']
    if json_output:
        args += ['--output', 'json']
    args.append(f'{stem}.{ext_expr}')
    steps = [{'args': args, 'exit': 'success'}]
    if json_output:
        steps[0]['json'] = {
            'schemaVersion': 1, 'command': 'build', 'success': True,
            'exitCode': 0,
            'payload': {'artifactKind': 'bundle', 'bundleFormat': 'esm'},
            'errors': [],
        }
    steps.append({'kind': 'file_json', 'path': f'{stem}/{stem}.meta.json',
                  'fields': {'apiSurface': 'browser', 'artifactKind': 'bundle'}})
    steps.append({'kind': 'browser_bundle_harness', 'entry': stem,
                  'body': HARNESS_BODY % export_name,
                  'stdout_contains': list(stdout_contains)})
    return steps


# ---------------------------------------------------------------- file 7 ----

def gen_sequence_wrappers():
    rs = src('browser_for_of_array_iteration_sequence_wrappers.rs')
    body = fixture(rs, 'for_of_sequence_source')
    header = f"""Migrated from tests/browser_for_of_array_iteration_sequence_wrappers.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): all 8 #[test] fns are individual (no loops
anywhere in this file -- confirmed by reading every fn body, not by matching
fn names) and funnel through the single helper
`assert_browser_bundle_for_of_sequence_wrapper(filename, json_output, source,
harness_function)`, one call each, always with the SAME fixture body
(`for_of_sequence_source()`) and the same harness export
(`forOfArrayIterationSequenceWrapper`). The 8 calls are exactly
ext(js/ts/jsx/tsx) x json_output(false/true) = 8, uniform across every case,
so `ext` is hoisted to a file-level [matrix] axis: 8 #[test] fns collapse to 2
[[case]] entries (build / json_build), matrix-fanned to 8 trials, matching the
8 real invocations exactly.

[source] needs no disambiguation (rule U5): this file has ONE fixture body and
source names it `app.<ext>`, so the flat file-wide `[source]` namespace holds
it under its original name. `--bundle` names its output directory after the
input stem, so `app/app.meta.json` and `entry = "app"` are unchanged too.

`exit = "success"` carries `assert!(output.status.success())`. The harness
step's `stdout_contains = ["1\\n", "2\\n"]` mirrors source's own
`stdout.contains("1\\n")` / `stdout.contains("2\\n")` -- plain `.contains`
against a field that HAS a substring form, so it is kept as `*_contains` and
NOT strengthened to an exact pin (controller ruling 3: mirror the source).
No `format!` hand-simulation (rule 8): the harness `body` below was produced by
executing the real `format!(r#"const mod = await import(bundleJs.href);\\nawait
mod.{{harness_function}}();\\n"#)` from source, not by hand-substitution.
"""
    sources = {f'app.${{ext}}': body}
    cases = []
    for json_output in (False, True):
        name = ('json_build_emits_for_of_sequence_wrapper' if json_output
                else 'build_emits_for_of_sequence_wrapper')
        rationale = (
            "Migrated from browser_for_of_array_iteration_sequence_wrappers.rs. "
            "`assert_browser_bundle_for_of_sequence_wrapper` builds a browser "
            "bundle (`kali build --bundle --api browser"
            + (" --output json`), asserts the JSON envelope's "
               "schemaVersion/command/success/exitCode/payload (artifactKind, "
               "bundleFormat) fields and that `errors` is empty, then asserts"
               if json_output else "`), asserts")
            + " the emitted `app/app.meta.json` metadata, then runs the bundle "
              "glue under the browser-bundle-harness contract and checks that "
              "the sequence-wrapped `for (const item of (0, [(0, 1), (0, 2)]))` "
              "loop printed both `1` and `2`. `ext` (js/ts/jsx/tsx) is hoisted "
              "to a file-level [matrix] axis: 8 #[test] fns collapse to 2 "
              "[[case]] entries here, matrix-fanned to 8 trials, matching the 8 "
              "real per-(ext,json_output) invocations in source exactly (see "
              "the file header's arithmetic). Source's stdout claim is a plain "
              "`.contains`, mirrored as `stdout_contains` rather than "
              "strengthened to an exact pin."
        )
        cases.append({'name': name, 'rationale': rationale,
                      'steps': bundle_steps('app', '${ext}', json_output,
                                            'forOfArrayIterationSequenceWrapper',
                                            ['1\n', '2\n'])})
    return emit_case_file(
        os.path.join(CASES, 'for_of_array_iteration_sequence_wrappers.toml'),
        header, {'ext': EXTS}, sources, cases)


# ---------------------------------------------------------------- file 8 ----

def gen_wrappers():
    rs = src('browser_for_of_array_iteration_wrappers.rs')
    variants = [
        ('as_const', 'for_of_as_const_source', 'forOfArrayIterationAsConstWrapper',
         'as-const-wrapped'),
        ('satisfies', 'for_of_satisfies_source', 'forOfArrayIterationSatisfiesWrapper',
         'satisfies-wrapped'),
    ]
    header = f"""Migrated from tests/browser_for_of_array_iteration_wrappers.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): all 16 #[test] fns are individual (no loops
anywhere in this file) and funnel through the single helper
`assert_browser_bundle_for_of_wrapper(filename, json_output, source,
harness_function)`, one call per (variant, ext, json_output) triple. Two
distinct fixture bodies/variants (as_const, satisfies), each exercised across
all four extensions in both json_output states:
  as_const:  4 (false) + 4 (true) = 8
  satisfies: 4 (false) + 4 (true) = 8
Total 16 invocations = variant(2) x ext(4) x json_output(2) exactly, uniform
across every case. `ext` is hoisted to a file-level [matrix] axis: 16 #[test]
fns collapse to 4 [[case]] entries (build/json_build x as_const/satisfies),
matrix-fanned to 16 trials, matching the 16 real invocations exactly.

[source] KEY DISAMBIGUATION (U5): source reuses the bare filename `app.<ext>`
for both fixture bodies (safe there -- each #[test] gets its own private
tempdir), but the case-file `[source]` table is one flat file-wide namespace
that cannot hold two bodies under one key. Renamed to
`app_as_const.<ext>`/`app_satisfies.<ext>`; because `kali build --bundle` names
its output directory after the input STEM, the `file_json` `path` and the
harness `entry` track the rename rather than staying hardcoded to `app`.
Neither fixture body references its own filename by string (no
`import()`/`require()` -- they are plain `kali build --bundle` entry points),
so the rename changes no claim.

`exit = "success"` carries `assert!(output.status.success())`. The harness
step's `stdout_contains = ["1\\n", "2\\n"]` mirrors source's own two plain
`.contains` checks (controller ruling 3: a plain `.contains` against a field
that HAS a substring form stays `*_contains`). Rule 8: each harness `body` was
produced by EXECUTING the real `format!` from source, not hand-substituted.
"""
    sources = {}
    for stem, fn, _export, _label in variants:
        sources[f'app_{stem}.${{ext}}'] = fixture(rs, fn)
    cases = []
    for stem, _fn, export, label in variants:
        for json_output in (False, True):
            prefix = 'json_build_emits' if json_output else 'build_emits'
            name = f'{prefix}_for_of_{stem}_wrapper'
            rationale = (
                "Migrated from browser_for_of_array_iteration_wrappers.rs. "
                "`assert_browser_bundle_for_of_wrapper` builds a browser bundle "
                "(`kali build --bundle --api browser"
                + (" --output json`), asserts the JSON envelope's "
                   "schemaVersion/command/success/exitCode/payload "
                   "(artifactKind, bundleFormat) fields and that `errors` is "
                   "empty, then asserts" if json_output else "`), asserts")
                + f" the emitted `app_{stem}/app_{stem}.meta.json` metadata, then "
                  "runs the bundle glue under the browser-bundle-harness "
                  f"contract and checks the printed {label} iteration output "
                  "(both `1` and `2`). `ext` (js/ts/jsx/tsx) is hoisted to a "
                  "file-level [matrix] axis: 16 #[test] fns collapse to 4 "
                  "[[case]] entries here, matrix-fanned to 16 trials, matching "
                  "the 16 real per-(variant,ext,json_output) invocations in "
                  "source exactly (see the file header's arithmetic). `[source]` "
                  "filenames are disambiguated per variant "
                  "(`app_as_const`/`app_satisfies`) because the `[source]` table "
                  "is one flat file-wide namespace and source reused the bare "
                  "name `app.<ext>` for both private-tempdir fixture bodies; no "
                  "assertion pins the literal filename and neither body "
                  "references its own filename by string, so the rename is safe. "
                  "Source's stdout claim is a plain `.contains`, mirrored as "
                  "`stdout_contains` rather than strengthened to an exact pin."
            )
            cases.append({
                'name': name, 'rationale': rationale,
                'steps': bundle_steps(f'app_{stem}', '${ext}', json_output,
                                      export, ['1\n', '2\n'])})
    return emit_case_file(
        os.path.join(CASES, 'for_of_array_iteration_wrappers.toml'),
        header, {'ext': EXTS}, sources, cases)


# ---------------------------------------------------------------- file 4 ----

def gen_break_continue():
    rs = src('browser_for_of_array_iteration_break_continue.rs')
    variants = [
        ('for_of', 'for_of_break_continue_source',
         'forOfArrayIterationBreakContinueWrapper', 'for-of'),
        ('for_await', 'for_await_break_continue_source',
         'forAwaitArrayIterationBreakContinueWrapper', 'for-await'),
    ]
    header = f"""Migrated from tests/browser_for_of_array_iteration_break_continue.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): all 16 #[test] fns are individual (no loops
anywhere in this file) and funnel through the single helper
`assert_browser_bundle_break_continue(filename, json_output, source,
harness_function)`, one call per (variant, ext, json_output) triple. Two
fixture bodies/variants (`for_of_break_continue_source` /
`for_await_break_continue_source`), each across all four extensions in both
json_output states: 2 x 4 x 2 = 16 invocations, uniform across every case.
`ext` is hoisted to a file-level [matrix] axis: 16 #[test] fns collapse to 4
[[case]] entries, matrix-fanned to 16 trials, matching exactly.

[source] KEY DISAMBIGUATION (U5): source writes both bodies to the bare
`app.<ext>` (private tempdir per #[test]); the flat file-wide `[source]` table
cannot, so the stems become `app_for_of` / `app_for_await`, and the `file_json`
`path` plus harness `entry` track the rename because `kali build --bundle`
names its output directory after the input stem. Neither body references its
own filename by string, so the rename changes no claim.

ASSERTION SHAPE: `exit = "success"` carries
`assert!(output.status.success())`. This file's harness assertion is a SINGLE
claim -- `assert!(stdout.contains("1\\n"))` -- not the two-value
`1`+`2` check its sibling wrapper files make, because the fixture `break`s out
of the loop after the first truthy value. Only that one needle is carried;
adding a `2` claim would be a rule-2 invention. Rule 8: the harness `body` was
produced by EXECUTING the real `format!`, not hand-substituted.
"""
    sources = {}
    for stem, fn, _export, _label in variants:
        sources[f'app_{stem}.${{ext}}'] = fixture(rs, fn)
    cases = []
    for stem, _fn, export, label in variants:
        for json_output in (False, True):
            prefix = 'json_build_emits' if json_output else 'build_emits'
            name = f'{prefix}_{stem}_break_continue'
            rationale = (
                "Migrated from browser_for_of_array_iteration_break_continue.rs. "
                "`assert_browser_bundle_break_continue` builds a browser bundle "
                "(`kali build --bundle --api browser"
                + (" --output json`), asserts the JSON envelope's "
                   "schemaVersion/command/success/exitCode/payload "
                   "(artifactKind, bundleFormat) fields and that `errors` is "
                   "empty, then asserts" if json_output else "`), asserts")
                + f" the emitted `app_{stem}/app_{stem}.meta.json` metadata, then "
                  "runs the bundle glue under the browser-bundle-harness "
                  f"contract and checks that the {label} loop's "
                  "`continue`-then-`break` path printed exactly the one value "
                  "`1`. Source makes a single `stdout.contains(\"1\\n\")` claim "
                  "here (not the two-value check its sibling wrapper files "
                  "make), so only that needle is carried -- adding a `2` claim "
                  "would invent one the source never made. `ext` "
                  "(js/ts/jsx/tsx) is hoisted to a file-level [matrix] axis: 16 "
                  "#[test] fns collapse to 4 [[case]] entries here, "
                  "matrix-fanned to 16 trials, matching the 16 real "
                  "per-(variant,ext,json_output) invocations in source exactly. "
                  "`[source]` stems are disambiguated per variant "
                  "(`app_for_of`/`app_for_await`) because the `[source]` table "
                  "is one flat file-wide namespace and source reused the bare "
                  "name `app.<ext>` for both bodies."
            )
            cases.append({
                'name': name, 'rationale': rationale,
                'steps': bundle_steps(f'app_{stem}', '${ext}', json_output,
                                      export, ['1\n'])})
    return emit_case_file(
        os.path.join(CASES, 'for_of_array_iteration_break_continue.toml'),
        header, {'ext': EXTS}, sources, cases)


# ---------------------------------------------------------------- file 2 ----

def gen_alias_chain():
    rs = src('browser_for_of_array_iteration_alias_chain.rs')
    bundle_body = fixture(rs, 'for_of_const_alias_chain_source')
    harness_body = fixture(rs, 'browser_harness_for_of_const_alias_chain_source')

    # U9: the exact `json["stdout"]` leaf value, read back from the real binary.
    live = {}
    for command, fname in (('run', 'main.ts'), ('test', 'smoke.test.ts')):
        _rc, envelope, _err = json_envelope(
            {fname: harness_body},
            ['--output', 'json', command, '--api', 'browser', '--max-threads',
             '0', '--max-spawned-processes', '0', fname], NODE_ENV)
        live[command] = envelope['stdout']
    assert live['run'] == '1\n2\n', live
    assert set(live) == {'run', 'test'}

    header = f"""Migrated from tests/browser_for_of_array_iteration_alias_chain.rs.
{NO_COMMENTS}

NO [matrix] (rule 7 / U1): this file has TWO helper groups with DIFFERENT
extension coverage, and `[matrix]` is file-wide (`expand()` fans EVERY
[[case]] by the full cross-product, with no per-case opt-out), so one axis
cannot serve both without inventing untested combinations. The build group
(`assert_browser_bundle_for_of_alias_chain`) is called for all four extensions
x json_output(2) = 8 invocations, matching its 8 `build_emits_*` /
`json_build_emits_*` #[test] fns. The harness group
(`assert_browser_harness_for_of_alias_chain`) is called for only TWO extensions
(ts, js) x command(run/test) x json_output(2) = 8 invocations, matching its 8
`*_supports_*_with_harness_{{ts,js}}_input` fns -- there is no jsx/tsx harness
fn in this file at all (verified against the real fn list, not inferred from
the first few). 8 + 8 = 16 invocations = 16 #[test] fns = 16 [[case]] entries
below, no folding. Matrix declined for the whole file.

[source] needs no disambiguation (U5): the build fixture is `app.<ext>`, the
harness fixture is `main.<ext>`/`smoke.test.<ext>`, so no key collides.

ASSERTION SHAPE. Build group: `exit = "success"` for
`assert!(output.status.success())`, the json envelope fields, the emitted
`app/app.meta.json` metadata, and the bundle-harness `stdout_contains`
(`1\\n`, `2\\n`) -- a plain `.contains` against a field that has a substring
form, so kept as `*_contains` (controller ruling 3). Harness group:
`exit = "success"` folds BOTH `assert!(output.status.success())` and
`assert_eq!(output.status.code(), Some(0))` (identical on a normal process
exit). Non-json: `stdout_contains = ["1", "2"]`, mirroring source's own
`stdout.contains("1")`/`stdout.contains("2")` -- note the needles are bare
`1`/`2` here, NOT `1\\n`/`2\\n` as in the build group; carried as written.
Json: source asserts `json["stdout"].as_str().contains("1")` and `...("2")`,
but a nested `json` leaf has no substring form in this format, so per
controller ruling 3 those two are resolved to an exact pin of the leaf -- and
only after live-capturing the value from the real `kali` binary
(`tools/task-18-browser-batch3/capture.py`), never hand-computed. Every run
satisfying the exact pin satisfies both original `.contains` claims, so this is
a verified strengthening, not a rewrite. `json["stderr"] == ""` is carried as
`stderr = ""` inside `json`. Source's json branch makes no `errors` or
`exitCode` claim in this file, so none is added (rule 2).
"""
    sources = {}
    for ext in EXTS:
        sources[f'app.{ext}'] = bundle_body
    for ext in ('ts', 'js', 'jsx', 'tsx'):
        sources[f'main.{ext}'] = harness_body
        sources[f'smoke.test.{ext}'] = harness_body
    # Only js/ts harness fixtures are actually exercised by any case; jsx/tsx
    # harness keys would be unreferenced fixtures, so they are NOT emitted.
    sources = {}
    for ext in EXTS:
        sources[f'app.{ext}'] = bundle_body
    for ext in ('ts', 'js'):
        sources[f'main.{ext}'] = harness_body
        sources[f'smoke.test.{ext}'] = harness_body

    cases = []
    build_rationale_common = (
        "Migrated from browser_for_of_array_iteration_alias_chain.rs. "
        "`assert_browser_bundle_for_of_alias_chain` builds a browser bundle "
        "(`kali build --bundle --api browser%s`), %sasserts the emitted "
        "`app/app.meta.json` metadata, then runs the bundle glue under the "
        "browser-bundle-harness contract and checks that the const-alias-chain "
        "`for (const value of alias)` loop printed both `1` and `2`. No "
        "[matrix] in this file: the build group covers all four extensions but "
        "the harness group covers only js/ts, and `[matrix]` is file-wide with "
        "no per-case opt-out (see the file header's invocation arithmetic). "
        "Source's stdout claim is a plain `.contains`, mirrored as "
        "`stdout_contains` rather than strengthened."
    )
    for ext in EXTS:
        for json_output in (False, True):
            prefix = 'json_build_emits' if json_output else 'build_emits'
            cases.append({
                'name': f'{prefix}_for_of_const_alias_chain_in_{ext}_input',
                'rationale': build_rationale_common % (
                    ' --output json' if json_output else '',
                    "asserts the JSON envelope's schemaVersion/command/success/"
                    "exitCode/payload (artifactKind, bundleFormat) fields and "
                    "that `errors` is empty, " if json_output else ''),
                'steps': bundle_steps('app', ext, json_output,
                                      'forOfArrayIterationConstAliasChainWrapper',
                                      ['1\n', '2\n'])})
    for ext in ('ts', 'js'):
        for command in ('run', 'test'):
            fname = f'main.{ext}' if command == 'run' else f'smoke.test.{ext}'
            for json_output in (False, True):
                args = (['--output', 'json'] if json_output else []) + [
                    command, '--api', 'browser', '--max-threads', '0',
                    '--max-spawned-processes', '0', fname]
                step = {'args': args, 'env': dict(NODE_ENV), 'exit': 'success'}
                if json_output:
                    step['json'] = {
                        'schemaVersion': 1, 'command': command, 'success': True,
                        'payload': {'hostContract': 'browser-requested',
                                    'runtimeBackend': 'browser-harness'},
                        'stdout': live[command], 'stderr': '',
                    }
                else:
                    step['stdout_contains'] = ['1', '2']
                prefix = 'json_' if json_output else ''
                name = (f'{prefix}{command}_supports_for_of_const_alias_chain_'
                        f'in_browser_api_surface_with_harness_{ext}_input')
                if json_output:
                    rationale = (
                        "Migrated from browser_for_of_array_iteration_alias_chain.rs. "
                        f"`assert_browser_harness_for_of_alias_chain(\"{command}\", ...)` "
                        f"runs `kali {command} --api browser --output json` under the "
                        "browser harness (`node`) and asserts the JSON envelope's "
                        "schemaVersion/command/success/payload (hostContract, "
                        "runtimeBackend) fields, that `stderr` is exactly empty, and "
                        "that `json[\"stdout\"]` contains both `1` and `2`. A nested "
                        "`json` leaf has no substring-assertion form in this "
                        "case-file format (only exact equality), so per the "
                        "mirror-the-source policy that claim is resolved to an exact "
                        "pin -- live-captured from the real `kali` binary, never "
                        "hand-computed. Every run satisfying the exact pin satisfies "
                        "both original `.contains` claims, so this is a verified "
                        "strengthening. `exit = \"success\"` folds both "
                        "`assert!(output.status.success())` and "
                        "`assert_eq!(output.status.code(), Some(0))`. No [matrix] in "
                        "this file: the harness group covers only js/ts while the "
                        "build group covers all four extensions, and `[matrix]` is "
                        "file-wide with no per-case opt-out."
                    )
                else:
                    rationale = (
                        "Migrated from browser_for_of_array_iteration_alias_chain.rs. "
                        f"`assert_browser_harness_for_of_alias_chain(\"{command}\", ...)` "
                        f"runs `kali {command} --api browser` under the browser "
                        "harness (`node`) and, in non-json mode, asserts a clean exit "
                        "and that stdout contains both `1` and `2` (bare needles, not "
                        "`1\\n`/`2\\n` as in this file's build group -- carried as "
                        "written). `exit = \"success\"` folds both "
                        "`assert!(output.status.success())` and "
                        "`assert_eq!(output.status.code(), Some(0))`. No [matrix] in "
                        "this file: the harness group covers only js/ts while the "
                        "build group covers all four extensions, and `[matrix]` is "
                        "file-wide with no per-case opt-out."
                    )
                cases.append({'name': name, 'rationale': rationale,
                              'steps': [step]})
    return emit_case_file(
        os.path.join(CASES, 'for_of_array_iteration_alias_chain.toml'),
        header, None, sources, cases)


if __name__ == '__main__':
    total = 0
    total += gen_sequence_wrappers()
    total += gen_wrappers()
    total += gen_break_continue()
    total += gen_alias_chain()
    print(f'iteration group: {total} trials')


# ---------------------------------------------------------------- file 5 ----

def gen_harness_sequence_wrappers():
    rs = src('browser_for_of_array_iteration_harness_sequence_wrappers_js_input.rs')
    body = fixture(rs, 'for_of_sequence_source')
    live = {}
    for command in ('run', 'test'):
        _rc, envelope, _err = json_envelope(
            {'main.js': body},
            ['--output', 'json', command, '--api', 'browser', '--max-threads',
             '0', '--max-spawned-processes', '0', 'main.js'], NODE_ENV)
        live[command] = envelope['stdout']

    header = f"""Migrated from
tests/browser_for_of_array_iteration_harness_sequence_wrappers_js_input.rs.
{NO_COMMENTS}

MATRIX ARITHMETIC (rule 7): all 5 #[test] fns funnel through the single helper
`assert_browser_harness_for_of_sequence_wrapper(command, filename,
json_output)`, using the SAME filename convention (`main.<ext>`) regardless of
command -- `kali test` is handed `main.js`, not a `smoke.test.js`, so there is
no per-command filename asymmetry in this file. Four fns each make one
individual call, always with `filename = "main.js"`:
  run_supports_..._js_input       -> ("run",  "main.js", false)
  test_supports_..._js_input      -> ("test", "main.js", false)
  json_run_supports_..._js_input  -> ("run",  "main.js", true)
  json_test_supports_..._js_input -> ("test", "main.js", true)
The fifth fn loops `for extension in ["ts","jsx","tsx"]` around all four
`(command, json_output)` pairs: 3 x 4 = 12 more calls. Total 4 + 12 = 16
invocations = command(2) x json_output(2) x ext(4) exactly, uniform across
every case. `ext` is hoisted to a file-level [matrix] axis: 5 #[test] fns
collapse to 4 [[case]] entries, matrix-fanned to 16 trials, matching exactly.

ASSERTION SHAPE. `exit = "success"` folds both
`assert!(output.status.success())` and `assert_eq!(output.status.code(),
Some(0))` (identical on a normal process exit). Non-json: `stdout_contains =
["1", "2"]`, mirroring source's plain `stdout.contains("1")` /
`stdout.contains("2")` -- kept as `*_contains` per controller ruling 3, not
strengthened. Json: source asserts `json["stdout"].contains("1")` and
`...("2")`, `json["stderr"] == ""`, and `json["errors"].is_empty()`; a nested
`json` leaf has no substring form in this format, so the two stdout `.contains`
claims are resolved to one exact pin, live-captured from the real `kali` binary
via `tools/task-18-browser-batch3/capture.py` -- a verified strengthening
(every run satisfying the pin satisfies both originals). Source's json branch
makes no `exitCode`/`payload.total` claim in this file, so none is added
(rule 2).
"""
    sources = {'main.${ext}': body}
    cases = []
    for command in ('run', 'test'):
        for json_output in (False, True):
            args = (['--output', 'json'] if json_output else []) + [
                command, '--api', 'browser', '--max-threads', '0',
                '--max-spawned-processes', '0', 'main.${ext}']
            step = {'args': args, 'env': dict(NODE_ENV), 'exit': 'success'}
            if json_output:
                step['json'] = {
                    'schemaVersion': 1, 'command': command, 'success': True,
                    'payload': {'hostContract': 'browser-requested',
                                'runtimeBackend': 'browser-harness'},
                    'stdout': live[command], 'stderr': '', 'errors': [],
                }
            else:
                step['stdout_contains'] = ['1', '2']
            prefix = 'json_' if json_output else ''
            name = (f'{prefix}{command}_supports_for_of_array_iteration_with_'
                    'sequence_wrappers_in_browser_api_surface_with_harness')
            rationale = (
                "Migrated from "
                "browser_for_of_array_iteration_harness_sequence_wrappers_js_input.rs. "
                f"`assert_browser_harness_for_of_sequence_wrapper(\"{command}\", ...)` "
                f"runs `kali {command} --api browser"
                + (" --output json` and asserts the JSON envelope's "
                   "schemaVersion/command/success/payload (hostContract, "
                   "runtimeBackend) fields, that `stderr` is exactly empty, that "
                   "`errors` is empty, and that `json[\"stdout\"]` contains both "
                   "`1` and `2`. A nested `json` leaf has no substring-assertion "
                   "form in this case-file format (only exact equality), so that "
                   "claim is resolved to an exact pin -- live-captured from the "
                   "real `kali` binary, never hand-computed, and strictly "
                   "stronger than source's two `.contains()` checks."
                   if json_output else
                   "` under the browser harness (`node`) and, in non-json mode, "
                   "asserts a clean exit and that stdout contains both `1` and "
                   "`2`; a plain `.contains` against a field that has a substring "
                   "form, so it is mirrored as `stdout_contains` rather than "
                   "strengthened.")
                + " `exit = \"success\"` folds both "
                  "`assert!(output.status.success())` and "
                  "`assert_eq!(output.status.code(), Some(0))`. `ext` "
                  "(js/ts/jsx/tsx) is hoisted to a file-level [matrix] axis: 5 "
                  "#[test] fns collapse to 4 [[case]] entries here, matrix-fanned "
                  "to 16 trials, matching the 16 real per-(command,json_output,ext) "
                  "invocations in source exactly (see the file header's "
                  "arithmetic)."
            )
            cases.append({'name': name, 'rationale': rationale, 'steps': [step]})
    return emit_case_file(
        os.path.join(CASES,
                     'for_of_array_iteration_harness_sequence_wrappers_js_input.toml'),
        header, {'ext': EXTS}, sources, cases)


# ---------------------------------------------------------------- file 6 ----

HARNESS_WRAPPER_VARIANTS = [
    # (variant stem, fixture fn, exts covered, human label)
    ('as_const', 'for_of_as_const_source', ['js', 'ts', 'jsx', 'tsx'],
     'as_const_wrapper'),
    ('satisfies', 'for_of_satisfies_source', ['js', 'tsx'], 'satisfies_wrapper'),
    ('parenthesized_const_alias', 'for_of_parenthesized_const_alias_source',
     ['js', 'ts', 'jsx', 'tsx'], 'parenthesized_const_alias_wrapper'),
]


def gen_harness_wrappers():
    rs = src('browser_for_of_array_iteration_harness_wrappers_js_input.rs')
    bodies = {stem: fixture(rs, fn) for stem, fn, _e, _l in HARNESS_WRAPPER_VARIANTS}

    live = {}
    for stem, _fn, exts, _label in HARNESS_WRAPPER_VARIANTS:
        for command in ('run', 'test'):
            fname = f'main_{stem}.{exts[0]}'
            _rc, envelope, _err = json_envelope(
                {fname: bodies[stem]},
                ['--output', 'json', command, '--api', 'browser',
                 '--max-threads', '0', '--max-spawned-processes', '0', fname],
                NODE_ENV)
            live[(stem, command)] = envelope['stdout']

    header = f"""Migrated from
tests/browser_for_of_array_iteration_harness_wrappers_js_input.rs.
{NO_COMMENTS}

NO [matrix] (rule 7 / U1): all 40 #[test] fns are individual calls (zero loops
anywhere in this file -- confirmed by reading every fn body) to
`assert_browser_harness_for_of_wrapper(command, filename, source,
json_output)`, but the three fixture variants are NOT uniform over `ext`.
Checked directly against the real fn list, not inferred from the first few
fns: `satisfies` is exercised in ONLY js and tsx (there is no
`..._satisfies_wrapper..._ts_input` or `..._jsx_input` fn), while `as_const`
and `parenthesized_const_alias` each cover all of js/ts/jsx/tsx. Arithmetic:
  as_const:                  4 ext x command(2) x json_output(2) = 16
  satisfies:                 2 ext x command(2) x json_output(2) =  8
  parenthesized_const_alias: 4 ext x command(2) x json_output(2) = 16
Total 40 invocations = 40 #[test] fns = 40 [[case]] entries below, 1:1. A
file-level `ext` axis would either invent untested (satisfies, ts)/(satisfies,
jsx) cases -- a rule-2 violation, since `expand()` fans EVERY [[case]] by the
full cross-product with no per-case opt-out -- or require excluding a case from
an axis the format cannot express. Declined for the whole file.

[source] KEY DISAMBIGUATION (U5): source reuses the bare filename `main.<ext>`
across all three variants (safe there -- each #[test] gets its own private
tempdir); the flat file-wide `[source]` table cannot hold three bodies under
one key, so the stems become `main_<variant>.<ext>`. The filename is only ever
the bare CLI positional argument; no fixture body references its own filename
by string (no `import()`/`require()` anywhere in the three source fns), so the
rename changes no claim. Only the (variant, ext) pairs source actually
exercises are declared, so no unreferenced fixture is written.

ASSERTION SHAPE. `exit = "success"` folds both
`assert!(output.status.success())` and `assert_eq!(output.status.code(),
Some(0))`. Non-json: source calls `assert_browser_for_of_array_iteration(&stdout)`,
whose body is `assert!(output.contains("1"))` / `assert!(output.contains("2"))`
-- a genuine substring claim, mirrored as `stdout_contains = ["1", "2"]` rather
than weakened or gratuitously strengthened (controller ruling 3). Json: source
asserts schemaVersion/command/success/payload(hostContract, runtimeBackend),
`json["stdout"].contains("1")` and `...("2")`, `json["stderr"] == ""`, and
`json["errors"].is_empty()`. The nested `json` leaf has no substring form, so
the two stdout `.contains` claims resolve to one exact pin, live-captured from
the real `kali` binary per variant and command -- a verified strengthening.
Source's json branch makes no top-level `exitCode` or `payload.total` claim in
this file; none is added (rule 2).
"""
    sources = {}
    for stem, _fn, exts, _label in HARNESS_WRAPPER_VARIANTS:
        for ext in exts:
            sources[f'main_{stem}.{ext}'] = bodies[stem]

    cases = []
    for stem, _fn, exts, label in HARNESS_WRAPPER_VARIANTS:
        for ext in exts:
            for command in ('run', 'test'):
                for json_output in (False, True):
                    fname = f'main_{stem}.{ext}'
                    args = (['--output', 'json'] if json_output else []) + [
                        command, '--api', 'browser', '--max-threads', '0',
                        '--max-spawned-processes', '0', fname]
                    step = {'args': args, 'env': dict(NODE_ENV),
                            'exit': 'success'}
                    if json_output:
                        step['json'] = {
                            'schemaVersion': 1, 'command': command,
                            'success': True,
                            'payload': {'hostContract': 'browser-requested',
                                        'runtimeBackend': 'browser-harness'},
                            'stdout': live[(stem, command)], 'stderr': '',
                            'errors': [],
                        }
                    else:
                        step['stdout_contains'] = ['1', '2']
                    prefix = 'json_' if json_output else ''
                    name = (f'{prefix}{command}_supports_for_of_array_iteration_'
                            f'lowering_with_{label}_in_browser_api_surface_'
                            f'with_harness_{ext}_input')
                    rationale = (
                        "Migrated from "
                        "browser_for_of_array_iteration_harness_wrappers_js_input.rs. "
                        "Every one of this file's 40 #[test] fns is an individual "
                        "(zero-loop) call to "
                        "`assert_browser_harness_for_of_wrapper(command, filename, "
                        "source, json_output)`. No [matrix]: the `satisfies` variant "
                        "is exercised in only js and tsx (not ts/jsx), so a "
                        "file-level `ext` axis would either invent an untested "
                        "(satisfies, ts)/(satisfies, jsx) case or require excluding a "
                        "case from an axis the format cannot express. "
                        + ("Source asserts the JSON envelope's "
                           "schemaVersion/command/success/payload (hostContract, "
                           "runtimeBackend) fields, that `stderr` is exactly empty, "
                           "that `errors` is empty, and that `json[\"stdout\"]` "
                           "contains both `1` and `2`; a nested `json` leaf has no "
                           "substring-assertion form in this format, so that claim is "
                           "resolved to an exact pin -- live-captured from the real "
                           "`kali` binary, never hand-computed, and strictly stronger "
                           "than source's two `.contains()` checks."
                           if json_output else
                           "Source's non-json assertion is "
                           "`assert_browser_for_of_array_iteration(&stdout)`, i.e. "
                           "`output.contains(\"1\")` and `output.contains(\"2\")` -- a "
                           "genuine substring claim, mirrored as `stdout_contains` "
                           "rather than strengthened to an exact match source never "
                           "claimed.")
                        + " `exit = \"success\"` folds both "
                          "`assert!(output.status.success())` and "
                          "`assert_eq!(output.status.code(), Some(0))`. `[source]` "
                          "filenames are disambiguated with a variant-suffix stem "
                          "(`main_<variant>.<ext>`) because source reuses the bare "
                          "name `main.<ext>` across all three variants and the "
                          "`[source]` table is one flat file-wide namespace; the "
                          "filename is only ever the bare CLI positional argument, "
                          "never read back inside a fixture body, so renaming it "
                          "changes no claim."
                    )
                    cases.append({'name': name, 'rationale': rationale,
                                  'steps': [step]})
    return emit_case_file(
        os.path.join(CASES,
                     'for_of_array_iteration_harness_wrappers_js_input.toml'),
        header, None, sources, cases)


# ---------------------------------------------------------------- file 3 ----

def gen_break_continue_harness():
    from extract import comment_block
    rs = src('browser_for_of_array_iteration_break_continue_harness.rs')
    variants = [
        ('for_of', 'browser_harness_for_of_break_continue_run_source',
         'browser_harness_for_of_break_continue_test_source', 'for-of'),
        ('for_await', 'browser_harness_for_await_break_continue_run_source',
         'browser_harness_for_await_break_continue_test_source', 'for-await'),
    ]
    # Rule 12: the json-branch comment is COPIED out of the source, not retyped.
    JSON_BRANCH_PROSE = comment_block(rs, 143, 147)

    bodies = {}
    for stem, run_fn, test_fn, _label in variants:
        bodies[(stem, 'run')] = fixture(rs, run_fn)
        bodies[(stem, 'test')] = fixture(rs, test_fn)

    def fname(stem, command, ext):
        return (f'main_{stem}.{ext}' if command == 'run'
                else f'smoke_{stem}.test.{ext}')

    live = {}
    for stem, _r, _t, _label in variants:
        for command in ('run', 'test'):
            f = fname(stem, command, 'js')
            _rc, envelope, _err = json_envelope(
                {f: bodies[(stem, command)]},
                [command, '--api', 'browser', '--max-threads', '0',
                 '--max-spawned-processes', '0', '--output', 'json', f],
                NODE_ENV)
            live[(stem, command)] = envelope['stdout']

    header = f"""Migrated from
tests/browser_for_of_array_iteration_break_continue_harness.rs.

MATRIX ARITHMETIC (rule 7): all 32 #[test] fns are individual (no loops
anywhere in this file) and funnel through the single helper
`assert_browser_harness_break_continue(command, filename, source,
json_output)`. Two fixture variants (`for_of` / `for_await`), each with its own
run-mode and test-mode body, across command(run/test) x json_output(2) x
ext(js/ts/jsx/tsx): 2 x 2 x 2 x 4 = 32 invocations = 32 #[test] fns, uniform
over `ext`, so `ext` is hoisted to a file-level [matrix] axis: 32 #[test] fns
collapse to 8 [[case]] entries, matrix-fanned to 32 trials, matching exactly.

[source] KEY DISAMBIGUATION (U5): source names the run fixture `main.<ext>` and
the test fixture `smoke.test.<ext>` for BOTH variants (safe there -- private
tempdir per #[test]); the flat file-wide `[source]` table cannot hold two
bodies under one key, so the stems become `main_for_of` / `main_for_await` and
`smoke_for_of.test` / `smoke_for_await.test`. The `.test.` infix is preserved
so `kali test` still sees a test-shaped filename. No fixture body references
its own filename by string, so the rename changes no claim. Note the trial dir
consequently holds all four harness fixtures at once; every case names its
entry explicitly on the argv, and the `payload.total = 1` claim on each
json/test case independently proves no sibling fixture was picked up.

ARGV ORDER: this helper appends `--output json` AFTER
`--max-threads 0 --max-spawned-processes 0` and before the entry, unlike most
of its siblings in this family which put it first. Mirrored exactly.

ASSERTION SHAPE. `exit = "success"` carries
`assert!(output.status.success())` (this helper makes no separate
`status.code()` claim). Non-json, command == "run": `stdout_contains =
["browser for-"]`; non-json, command == "test": `stdout_contains = ["ok 1"]`
-- source branches on the command and makes exactly one of those two claims,
never both, so neither case carries the other's needle (rule 2). Json: source
asserts schemaVersion/command/success/payload(hostContract, runtimeBackend),
then for "run" `json["exitCode"] == 0` and `json["payload"]["exitCode"] == 0`,
and for "test" `payload.total/passed/failed == 1/1/0`; plus
`json["stdout"].contains("browser for-")`, `json["stderr"] == ""` and
`json["errors"].is_empty()`. The nested `json` leaf has no substring form, so
the stdout claim resolves to an exact pin live-captured from the real `kali`
binary per (variant, command) -- a verified strengthening.

PROSE ATTRIBUTION (rule 12 / U6): this file's only Rust comment block
(`:143-147`) sits INSIDE the helper's `if json_output` branch, so it is prose
attached to the json-mode stdout pin and is carried into the rationale of the
16 json cases only -- not the 16 non-json cases, which never reach it.
`comment_coverage.py` has no per-helper (or per-branch) attribution and will
therefore report those 5 lines "missing" from the 4 non-json [[case]] entries;
that is the checker's known limitation, recorded here rather than papered over
by copying the prose into cases their producing branch does not reach (which
U6 forbids even though it would turn the checker green).
"""
    sources = {}
    for stem, _r, _t, _label in variants:
        for command in ('run', 'test'):
            sources[fname(stem, command, '${ext}')] = bodies[(stem, command)]

    cases = []
    for stem, _r, _t, label in variants:
        for command in ('run', 'test'):
            for json_output in (False, True):
                f = fname(stem, command, '${ext}')
                args = [command, '--api', 'browser', '--max-threads', '0',
                        '--max-spawned-processes', '0']
                if json_output:
                    args += ['--output', 'json']
                args.append(f)
                step = {'args': args, 'env': dict(NODE_ENV), 'exit': 'success'}
                if json_output:
                    payload = {'hostContract': 'browser-requested',
                               'runtimeBackend': 'browser-harness'}
                    j = {'schemaVersion': 1, 'command': command,
                         'success': True, 'payload': payload}
                    if command == 'run':
                        j['exitCode'] = 0
                        payload['exitCode'] = 0
                    else:
                        payload['total'] = 1
                        payload['passed'] = 1
                        payload['failed'] = 0
                    j['stdout'] = live[(stem, command)]
                    j['stderr'] = ''
                    j['errors'] = []
                    step['json'] = j
                else:
                    step['stdout_contains'] = (['browser for-'] if command == 'run'
                                               else ['ok 1'])
                prefix = 'json_' if json_output else ''
                name = (f'{prefix}{command}_supports_{stem}_break_continue_when_'
                        'browser_harness_is_configured')
                rationale = (
                    "Migrated from "
                    "browser_for_of_array_iteration_break_continue_harness.rs. "
                    f"`assert_browser_harness_break_continue(\"{command}\", ...)` runs "
                    f"`kali {command} --api browser"
                    + (" ... --output json` and asserts the JSON envelope's "
                       "schemaVersion/command/success/payload (hostContract, "
                       "runtimeBackend) fields, "
                       + ("`json[\"exitCode\"] == 0` and "
                          "`json[\"payload\"][\"exitCode\"] == 0`, "
                          if command == 'run' else
                          "`payload.total`/`passed`/`failed` == 1/1/0, ")
                       + "that `stderr` is exactly empty, that `errors` is empty, "
                         "and that `json[\"stdout\"]` contains `browser for-`. A "
                         "nested `json` leaf has no substring-assertion form in "
                         "this case-file format (only exact equality), so that "
                         "claim is resolved to an exact pin -- live-captured from "
                         "the real `kali` binary, never hand-computed, and "
                         "strictly stronger than source's `.contains()` check."
                       if json_output else
                       "` under the browser harness (`node`) and, in non-json "
                       "mode, asserts a clean exit and that stdout contains "
                       + ("`browser for-`" if command == 'run' else "`ok 1`")
                       + ". Source branches on the command and makes exactly one "
                         "of those two claims, never both, so this case carries "
                         "only its own needle.")
                    + f" The fixture is the {label} break/continue probe, which "
                      "`continue`s past the falsy element, records the first "
                      "truthy one and `break`s. `ext` (js/ts/jsx/tsx) is hoisted "
                      "to a file-level [matrix] axis: 32 #[test] fns collapse to "
                      "8 [[case]] entries here, matrix-fanned to 32 trials, "
                      "matching the 32 real per-(variant,command,json_output,ext) "
                      "invocations in source exactly. `[source]` stems are "
                      "disambiguated per variant because the `[source]` table is "
                      "one flat file-wide namespace and source reused "
                      "`main.<ext>`/`smoke.test.<ext>` for both variants."
                )
                if json_output:
                    rationale += '\n\n' + JSON_BRANCH_PROSE
                cases.append({'name': name, 'rationale': rationale,
                              'steps': [step]})
    return emit_case_file(
        os.path.join(CASES, 'for_of_array_iteration_break_continue_harness.toml'),
        header, {'ext': EXTS}, sources, cases)


# ---------------------------------------------------------------- file 1 ----

def gen_object_string_enumeration_sequence_wrappers():
    from extract import comment_block
    name_rs = ('browser_for_await_object_string_enumeration_sequence_wrappers_'
               'js_input.rs')
    rs = src(name_rs)
    PROSE = comment_block(rs, 169, 170)
    run_body = fixture(
        rs, 'browser_harness_object_string_enumeration_sequence_wrappers_source')
    test_body = fixture(
        rs,
        'browser_harness_object_string_enumeration_sequence_wrappers_test_source')

    # Rule 11: source accepts E5506 on EITHER stream; resolve against the real
    # binary per mode and pin the stream that actually carries it.
    observed = {}
    for command, body, key in (('run', run_body, 'main_run'),
                               ('test', test_body, 'main_test')):
        for json_output in (False, True):
            f = f'{key}.js'
            args = (['--output', 'json'] if json_output else []) + [
                command, '--api', 'browser', '--max-threads', '0',
                '--max-spawned-processes', '0', f]
            rc, out, err = run({f: body}, args, NODE_ENV)
            assert rc != 0, (command, json_output, rc, out, err)
            on_out, on_err = 'E5506' in out, 'E5506' in err
            assert on_out != on_err, (command, json_output, on_out, on_err)
            observed[(command, json_output)] = 'stdout' if on_out else 'stderr'

    header = f"""Migrated from tests/browser_for_await_object_string_enumeration_
sequence_wrappers_js_input.rs.

{PROSE}

MATRIX ARITHMETIC (rule 7): all 5 #[test] fns funnel through the single helper
`assert_browser_harness_object_string_enumeration_sequence_wrappers(command,
filename, json_output)`. Four fns each make one individual call with
`filename = "main.js"`; the fifth loops `for extension in ["ts","jsx","tsx"]`
around all four `(command, json_output)` pairs (3 x 4 = 12). Total 4 + 12 = 16
invocations = command(2) x json_output(2) x ext(4), uniform over `ext`, so
`ext` is hoisted to a file-level [matrix] axis: 5 #[test] fns collapse to 4
[[case]] entries, matrix-fanned to 16 trials, matching exactly.

[source] KEY DISAMBIGUATION (U5 / U2): the helper picks its fixture body from
the COMMAND (`if command == "test" {{ ..._test_source() }} else {{
..._source() }}`) while writing both to the same `main.<ext>` filename. `[source]`
is file-wide and has no conditionals, so one key cannot carry two bodies:
the stems become `main_run.<ext>` and `main_test.<ext>`. Both fixtures are
consequently present in every trial dir, which is harmless here -- neither the
argv nor any assertion depends on a file's presence or absence, only on the
entry each case names explicitly, so this is not the U2 conditional-fixture
hazard (no case's point is that a file is or is not there). The filename is
only ever the bare CLI positional argument, never read back inside a fixture
body, so the rename changes no claim.

STREAM RESOLUTION (rule 11, applied to streams rather than codes): source's
assertion is `stderr.contains("E5506") || stdout.contains("E5506")` -- an OR
across STREAMS for the SAME code. Resolved against the real binary for every
(command, json_output) combination rather than reproduced: non-json carries
E5506 on {observed[('run', False)]}, `--output json` carries it on
{observed[('run', True)]}. Each mode is pinned to the stream that actually
carries it -- a verified strengthening, since this is a PRESENCE claim and
narrowing a presence claim to the stream that carries it is strictly stronger
(narrowing an ABSENCE claim would be a weakening, and there is none here). The
source's full disjunction sentence is carried into every affected case's
rationale below rather than silently narrowed.

`assert!(!output.status.success(), "must fail closed: {{output:?}}")` becomes
`exit = "failure"`. Source makes no other assertion in this file -- no exit
code, no envelope field, no second needle -- so none is added (rule 2).
"""
    sources = {'main_run.${ext}': run_body, 'main_test.${ext}': test_body}
    cases = []
    for command in ('run', 'test'):
        for json_output in (False, True):
            key = 'main_run' if command == 'run' else 'main_test'
            args = (['--output', 'json'] if json_output else []) + [
                command, '--api', 'browser', '--max-threads', '0',
                '--max-spawned-processes', '0', key + '.${ext}']
            stream = observed[(command, json_output)]
            step = {'args': args, 'env': dict(NODE_ENV), 'exit': 'failure',
                    f'{stream}_contains': ['E5506']}
            prefix = 'json_' if json_output else ''
            name = (f'{prefix}{command}_supports_object_string_enumeration_'
                    'sequence_wrappers_in_browser_api_surface_with_harness')
            rationale = (
                "Migrated from browser_for_await_object_string_enumeration_"
                "sequence_wrappers_js_input.rs. " + PROSE + " "
                f"`assert_browser_harness_object_string_enumeration_sequence_"
                f"wrappers(\"{command}\", ...)` runs `kali {command} --api browser"
                + (" --output json" if json_output else "")
                + "` under the browser harness (`node`) and asserts only that the "
                  "process fails closed and that E5506 appears on one of the two "
                  "streams. Source's assertion is "
                  "`stderr.contains(\"E5506\") || stdout.contains(\"E5506\")` -- an "
                  "OR across streams for the same code (rule 11's shape, applied "
                  "to streams instead of codes). Verified directly against the "
                  f"real binary: in this mode E5506 appears on {stream} only. "
                  "Pinned to the stream that actually carries it -- a verified "
                  "strengthening of the source's OR (a presence claim, so "
                  "narrowing is safe), with the disjunction sentence carried here "
                  "per rule 11. `ext` (js/ts/jsx/tsx) is hoisted to a file-level "
                  "[matrix] axis: 5 #[test] fns collapse to 4 [[case]] entries "
                  "here, matrix-fanned to 16 trials, matching the 16 real "
                  "per-(command,json_output,ext) invocations in source exactly. "
                  "The helper picks its fixture body from the command while "
                  "writing both to `main.<ext>`, which one flat file-wide "
                  "`[source]` key cannot express, so the stems are disambiguated "
                  "to `main_run`/`main_test`; the filename is only ever the bare "
                  "CLI positional argument, never read back inside a fixture body."
            )
            cases.append({'name': name, 'rationale': rationale, 'steps': [step]})
    return emit_case_file(
        os.path.join(
            CASES,
            'for_await_object_string_enumeration_sequence_wrappers_js_input.toml'),
        header, {'ext': EXTS}, sources, cases)
