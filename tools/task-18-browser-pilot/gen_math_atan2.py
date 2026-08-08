import sys, json, re
sys.path.insert(0, '/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18')
from lexer import find_string_literals
from toml_emit import toml_string, toml_str_array
from kali_run import run_kali

RS_TOP = open('/workspace/crates/kali_cli/tests/browser_math_atan2_bracketed_root.rs').read()
RS_RUN = open('/workspace/crates/kali_cli/tests/browser_math_atan2_bracketed_root/run.rs').read()
RS_BUILD = open('/workspace/crates/kali_cli/tests/browser_math_atan2_bracketed_root/build.rs').read()

lits_top = find_string_literals(RS_TOP)
def lit_top(n):
    return lits_top[n]['value']

# Verify extraction against expected known substrings (sanity, not exhaustive)
zero_slice_src = None
bracketed_method_src = None
single_quoted_src = None
for l in lits_top:
    if 'bracketedGlobalThisMathAtan2ZeroSlice()' in l['value'] and 'function bracketedGlobalThisMathAtan2ZeroSlice' in l['value']:
        zero_slice_src = l['value']
    if 'bracketedGlobalThisMathAtan2BracketedMethod()' in l['value'] and 'function bracketedGlobalThisMathAtan2BracketedMethod' in l['value']:
        bracketed_method_src = l['value']
    if 'singleQuotedGlobalThisMathAtan2ZeroSlice()' in l['value'] and 'function singleQuotedGlobalThisMathAtan2ZeroSlice' in l['value']:
        single_quoted_src = l['value']
assert zero_slice_src and bracketed_method_src and single_quoted_src

# Extract as_const / satisfies wrapper bodies + harness_function calls from build.rs literals
lits_build = find_string_literals(RS_BUILD)
as_const_src = None
satisfies_src = None
for l in lits_build:
    if 'bracketedGlobalThisMathAtan2AsConstWrapper' in l['value'] and 'function ' in l['value']:
        as_const_src = l['value']
    if 'bracketedGlobalThisMathAtan2SatisfiesWrapper' in l['value'] and 'function ' in l['value']:
        satisfies_src = l['value']
assert as_const_src and satisfies_src

harness_prelude_zero_slice = "const mod = await import(bundleJs.href);\nawait mod.bracketedGlobalThisMathAtan2ZeroSlice();\n"
harness_bracketed_method = "const mod = await import(bundleJs.href);\nawait mod.bracketedGlobalThisMathAtan2BracketedMethod();\n"
harness_single_quoted = "const mod = await import(bundleJs.href);\nawait mod.singleQuotedGlobalThisMathAtan2ZeroSlice();\n"
harness_as_const = "const mod = await import(bundleJs.href);\nawait mod.bracketedGlobalThisMathAtan2AsConstWrapper();\n"
harness_satisfies = "const mod = await import(bundleJs.href);\nawait mod.bracketedGlobalThisMathAtan2SatisfiesWrapper();\n"

# Sanity-check these against literals actually present in the top-level file
assert harness_prelude_zero_slice in RS_TOP
assert harness_bracketed_method in RS_TOP
assert harness_single_quoted in RS_TOP

cases = []  # list of dict(name, rationale, steps=[...])

BUILD_JSON_ASSERT = """[schemaVersion] 1
[command] "build"
[success] true
[exitCode] 0
[payload.artifactKind] "bundle"
[payload.bundleFormat] "esm"
[errors] []"""

def build_case(name, filename, json_output, source_key, harness_body, harness_stdout_contains="0\n"):
    # `kali build --bundle` emits into a directory named after the INPUT
    # FILE'S STEM (verified against the real binary: app_method.ts -> app_method/).
    # The original source always used the bare "app.{ext}" filename (so its
    # stem was always literally "app"); the disambiguating rename above
    # (app_method.ts, app_sq.ts, ...) means the emitted directory's name
    # changes to match -- checked here explicitly rather than hardcoded.
    stem = filename.rsplit(".", 1)[0]
    steps = []
    args = ["build", "--bundle", "--api", "browser"]
    if json_output:
        args = args + ["--output", "json"]
    args = args + [filename]
    cli = {"args": args, "exit": "success"}
    if json_output:
        cli["json"] = {
            "schemaVersion": 1, "command": "build", "success": True, "exitCode": 0,
            "payload": {"artifactKind": "bundle", "bundleFormat": "esm"},
            "errors": [],
        }
    steps.append(("cli", cli))
    steps.append(("file_json", {"path": f"{stem}/{stem}.meta.json", "fields": {"apiSurface": "browser", "artifactKind": "bundle"}}))
    steps.append(("browser_bundle_harness", {"entry": stem, "body": harness_body, "stdout_contains": [harness_stdout_contains]}))
    return {"name": name, "source_key": source_key, "steps": steps}

# --- build.rs: 24 cases ---
for ext in ["js", "ts", "jsx", "tsx"]:
    fn = f"app.{ext}"
    for jo, prefix in [(False, "build"), (True, "json_build")]:
        name = f"{prefix}_emits_bracketed_global_this_math_atan2_zero_slice_in_{ext}_input"
        cases.append(build_case(name, fn, jo, fn, harness_prelude_zero_slice))

# NOTE on filenames: the source reuses the bare "app.{ext}" filename across
# ALL these variants (each #[test] gets its own private tempdir in the .rs
# world, so "app.ts" meaning "bracketed_method" and "app.ts" meaning
# "zero_slice" never collide there). The case-file format's [source] table is
# a single flat, file-wide namespace, so the same key cannot hold two
# different bodies. Disambiguated with a variant suffix BEFORE the extension
# (app_method.ts, app_sq.ts, ...) -- the extension itself (what kali uses for
# language detection) is preserved exactly; only the disambiguating stem
# differs from source, and no test ever asserts on the literal filename, so
# this changes no claim.
for ext in ["js", "ts"]:
    fn = f"app_method.{ext}"
    for jo, prefix in [(False, "build"), (True, "json_build")]:
        name = f"{prefix}_emits_bracketed_global_this_math_atan2_bracketed_method_in_{ext}_input"
        cases.append(build_case(name, fn, jo, fn, harness_bracketed_method))

for ext in ["js", "ts", "jsx", "tsx"]:
    fn = f"app_sq.{ext}"
    for jo, prefix in [(False, "build"), (True, "json_build")]:
        name = f"{prefix}_emits_single_quoted_global_this_math_atan2_zero_slice_in_{ext}_input"
        cases.append(build_case(name, fn, jo, fn, harness_single_quoted))

for jo, prefix in [(False, "build"), (True, "json_build")]:
    name = f"{prefix}_emits_bracketed_global_this_math_atan2_as_const_wrapper_in_ts_input"
    cases.append(build_case(name, "app_as_const.ts", jo, "app_as_const.ts", harness_as_const))
for jo, prefix in [(False, "build"), (True, "json_build")]:
    name = f"{prefix}_emits_bracketed_global_this_math_atan2_satisfies_wrapper_in_ts_input"
    cases.append(build_case(name, "app_satisfies.ts", jo, "app_satisfies.ts", harness_satisfies))

assert len(cases) == 24, len(cases)

# source bodies keyed by the actual on-disk filename used in that case's args
# (== the [source] table key expand.rs will write to disk).
source_bodies = {}
for ext in ["js", "ts", "jsx", "tsx"]:
    source_bodies[f"app.{ext}"] = zero_slice_src
for ext in ["js", "ts"]:
    source_bodies[f"app_method.{ext}"] = bracketed_method_src
for ext in ["js", "ts", "jsx", "tsx"]:
    source_bodies[f"app_sq.{ext}"] = single_quoted_src
source_bodies["app_as_const.ts"] = as_const_src
source_bodies["app_satisfies.ts"] = satisfies_src

print("build.rs cases:", len(cases))

# --- run.rs: 5 fns, split into named siblings ---
# Every fixture body/expected-stdout literal below is pulled by INDEX from
# lits_run (find_string_literals over the real run.rs text), never hand-typed
# -- this is the same copy mechanism lexer.py provides everywhere else in this
# pilot. (command, source_name) pairs and expected_stdout are transcribed from
# direct reading since they are short and mechanically checked below via a
# live run against the real binary.
lits_run = find_string_literals(RS_RUN)
def L(n):
    return lits_run[n]['value']

run_groups = [
    ("run_and_test_supports_bracketed_global_this_math_atan2_zero_slice_when_browser_harness_is_configured", [
        ("run", "main.js", L(2), "0"),
        ("test", "smoke.test.js", L(6), "0\nok 1"),
        ("run", "main.ts", L(10), "0"),
        ("test", "smoke.test.ts", L(14), "0\nok 1"),
        ("run", "main.jsx", L(18), "0"),
        ("test", "smoke.test.jsx", L(22), "0\nok 1"),
        ("run", "main.tsx", L(26), "0"),
        ("test", "smoke.test.tsx", L(30), "0\nok 1"),
    ]),
    ("run_and_test_supports_bracketed_global_this_math_atan2_as_const_wrapper_when_browser_harness_is_configured_in_ts_input", [
        ("run", "main.ts", L(76), "0"),
        ("test", "smoke.test.ts", L(80), "0\nok 1"),
    ]),
    ("run_and_test_supports_bracketed_global_this_math_atan2_satisfies_wrapper_when_browser_harness_is_configured_in_ts_input", [
        ("run", "main.ts", L(126), "0"),
        ("test", "smoke.test.ts", L(130), "0\nok 1"),
    ]),
    ("run_and_test_supports_bracketed_global_this_math_atan2_bracketed_method_when_browser_harness_is_configured_in_js_and_ts_input", [
        ("run", "main.js", L(176), "0"),
        ("test", "smoke.test.js", L(180), "0\nok 1"),
        ("run", "main.ts", L(184), "0"),
        ("test", "smoke.test.ts", L(188), "0\nok 1"),
    ]),
    ("run_and_test_supports_single_quoted_global_this_math_atan2_zero_slice_when_browser_harness_is_configured", [
        ("run", "main.js", L(234), "0"),
        ("test", "smoke.test.js", L(238), "0\nok 1"),
        ("run", "main.ts", L(242), "0"),
        ("test", "smoke.test.ts", L(246), "0\nok 1"),
        ("run", "main.jsx", L(250), "0"),
        ("test", "smoke.test.jsx", L(254), "0\nok 1"),
        ("run", "main.tsx", L(258), "0"),
        ("test", "smoke.test.tsx", L(262), "0\nok 1"),
    ]),
]

# (No raw-text substring check here: L(n) values are DECODED string contents
# -- real '"' characters where the source has escaped \" -- so they never
# appear verbatim in RS_RUN's raw text by construction. Correctness instead
# comes from indexing directly into lits_run, itself built from RS_RUN.)

total_run_combos = sum(len(v) for _, v in run_groups)
print("run.rs combos (pre json-split):", total_run_combos)

ext_suffix = {"main.js": "js", "main.ts": "ts", "main.jsx": "jsx", "main.tsx": "tsx",
              "smoke.test.js": "js", "smoke.test.ts": "ts", "smoke.test.jsx": "jsx", "smoke.test.tsx": "tsx"}
# Short per-group filename tag. Same reasoning as the build.rs section above:
# "main.js"/"smoke.test.js" are reused across groups with DIFFERENT bodies in
# source (each #[test] has its own private tempdir there); [source] here is
# one flat file-wide table, so each group gets a distinct on-disk stem before
# the (preserved) extension.
group_tags = [
    "zeroslice", "asconst", "satisfies", "method", "sq",
]

run_cases = []
verify_log = []
for gi, (fn_name, entries) in enumerate(run_groups):
    tag = group_tags[gi]
    for (command, source_name, source, expected_stdout) in entries:
        for json_output in [False, True]:
            ext = ext_suffix[source_name]
            is_test = source_name.startswith("smoke.test.")
            disk_name = (f"smoke.test.{tag}.{ext}" if is_test else f"main_{tag}.{ext}")
            suffix = f"_{command}_{ext}_{'json' if json_output else 'text'}"
            name = fn_name + suffix
            args = []
            if json_output:
                args += ["--output", "json"]
            args += [command, "--api", "browser", "--max-threads", "0", "--max-spawned-processes", "0", disk_name]
            env = {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"}
            step = {"args": args, "env": env, "exit": "success"}
            # Verify against the real binary
            rc, out, err, _ = run_kali({disk_name: source}, args, env=env)
            verify_log.append((name, rc, len(out), len(err)))
            assert rc == 0, (name, rc, out, err)
            if json_output:
                obj = json.loads(out)
                assert obj["schemaVersion"] == 1
                assert obj["command"] == command
                assert obj["success"] is True
                assert obj["payload"]["hostContract"] == "browser-requested"
                assert obj["payload"]["runtimeBackend"] == "browser-harness"
                assert obj["stderr"] == ""
                assert obj["errors"] == []
                jf = {
                    "schemaVersion": 1, "command": command, "success": True,
                    "payload": {"hostContract": "browser-requested", "runtimeBackend": "browser-harness"},
                    "stderr": "", "errors": [], "stdout": obj["stdout"],
                }
                if command == "run":
                    assert obj["exitCode"] == 0
                    assert obj["payload"]["exitCode"] == 0
                    jf["exitCode"] = 0
                    jf["payload"]["exitCode"] = 0
                else:
                    assert obj["payload"]["total"] == 1
                    assert obj["payload"]["passed"] == 1
                    assert obj["payload"]["failed"] == 0
                    jf["payload"]["total"] = 1
                    jf["payload"]["passed"] = 1
                    jf["payload"]["failed"] = 0
                assert "0" in obj["stdout"]
                step["json"] = jf
            else:
                stdout_text = out.decode("utf-8")
                assert expected_stdout in stdout_text, (name, expected_stdout, stdout_text)
                step["stdout_contains"] = [expected_stdout]
            run_cases.append({"name": name, "source_key": disk_name, "source_body": source, "steps": [("cli", step)]})

print("run.rs generated cases:", len(run_cases))
assert len(run_cases) == 48

import pickle
with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/atan2_data.pkl', 'wb') as f:
    pickle.dump({
        "build_cases": cases,
        "source_bodies": source_bodies,
        "run_cases": run_cases,
    }, f)
print("OK, dumped")
