import sys, json
sys.path.insert(0, '/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18')
from toml_emit import toml_string, toml_str_array
from kali_run import run_kali

with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/math_pow_bodies.json') as f:
    B = json.load(f)

RATIONALE = (
    "Migrated from browser_math_pow_exponent_one.rs. All 48 #[test] fns are "
    "individual (no loops at all in this file) and, grouped by (variant, "
    "mode), each group of 8 is exactly ext(js/ts/jsx/tsx) x json_output(2) "
    "with an identical assertion shape -- confirmed by reading all 48 fn "
    "bodies. `ext` is therefore a genuinely uniform file-level [matrix] "
    "axis; 48 #[test] fns collapse to 12 [[case]] entries, matrix-fanned to "
    "48 trials. `assert_browser_harness_math_pow_exponent_one_identity`'s "
    "`_expected_stdout` parameter is UNUSED (underscore-prefixed) at every "
    "one of its 32 call sites -- the real expected value is computed "
    "dynamically inside the helper from `source.matches(\"console.log(\").c"
    "ount()` and whether the fixture contains the literal `\"Math.pow(1, "
    "alias)\"` substring. The 32 hardcoded `_expected_stdout` string "
    "literals at the call sites (e.g. `\"2\\n2\\n2\\n2\\n2\\n2\\n2\\n2\"`) "
    "are therefore DEAD TEXT, never actually asserted -- not migrated as a "
    "claim (there is no claim to migrate), and not fabricated into one "
    "either (rule 2). The REAL per-case expected stdout (48 repeats of "
    "\"2\" or \"1\" joined by newlines, since every fixture makes exactly "
    "12 direct + 36 frozen-callable = 48 `console.log(...)` calls) was "
    "captured live from the real binary for every case and pinned exactly "
    "-- strictly stronger than source's own `.contains(...)` substring "
    "check on that value, in both `--output json` and text mode."
)

HEADER = """# Migrated from tests/browser_math_pow_exponent_one.rs.
#
# All 48 #[test] fns are individual calls (no loops in this file at all);
# grouped by (variant in {exponent_one, base_one}, mode in {build, run,
# test}), each of the 6 groups is exactly 8 fns = ext(js/ts/jsx/tsx) x
# json_output(2), with an identical assertion shape per group -- confirmed
# by reading all 48. `ext` is therefore a genuinely uniform file-level
# [matrix] axis: 48 #[test] fns -> 12 [[case]] entries, matrix-fanned to 48
# trials.
#
# DEAD LITERAL FOUND: `assert_browser_harness_math_pow_exponent_one_identity`
# takes an `_expected_stdout: &str` parameter (underscore-prefixed) that is
# NEVER READ inside the function -- the helper computes its own expected
# value dynamically instead (`source.matches("console.log(").count()`
# repeats of "2" or "1", chosen by whether the fixture text contains the
# literal substring "Math.pow(1, alias)"). The 32 hardcoded strings passed
# at call sites (e.g. "2\\n2\\n2\\n2\\n2\\n2\\n2\\n2") are dead text with no
# assertion behind them. Per rule 2, this is NOT migrated as a claim (there
# is none) and NOT fabricated into one; the real, dynamically-computed
# expected stdout (48 repeats of the value, one per `console.log`, since
# every fixture makes exactly 12 direct + 36 frozen-callable invocations)
# was captured live from the real binary for every case and pinned exactly.
#
# `format!`-built fixtures: `browser_bundle_math_pow_exponent_one_source`/
# `..._base_one_identity_source` and `browser_harness_math_pow_identity_
# run_source`/`..._test_source` all build their bodies via
# `kali_common::math_pow_invocation_lines_for_aliases`/
# `..._entries_for_aliases` -- functions in a LIBRARY CRATE, not string
# literals in this file at all. Every fixture body below was captured by
# actually EXECUTING these real functions via a temporary Rust test dump
# (never hand-derived or reimplemented in Python), the same rule-8
# discipline as a `format!` brace-collapse trap, applied one level further
# up the call chain.
"""

variants = {
    "exponent_one": {"expected_value": "2", "fn_name": "mathPowExponentOneIdentity",
                      "bundle_body": B["bundle_exponent_one"], "run_body": B["harness_run_exponent_one"],
                      "test_body": B["harness_test_exponent_one"]},
    "base_one": {"expected_value": "1", "fn_name": "mathPowBaseOneIdentity",
                 "bundle_body": B["bundle_base_one"], "run_body": B["harness_run_base_one"],
                 "test_body": B["harness_test_base_one"]},
}

cases = []
source_map = {}

# --- build cases ---
for variant, v in variants.items():
    stem = f"app_{variant}"
    fname = f"{stem}.${{ext}}"
    source_map[fname] = v["bundle_body"]
    for json_output, jtag in [(False, ""), (True, "json_")]:
        vtag = "exponent_one" if variant == "exponent_one" else "base_one"
        name = f"{jtag}build_emits_math_pow_{vtag}_identity_in_{{ext}}_input"
        cases.append({"kind": "build", "variant": variant, "json_output": json_output, "name_tmpl": name, "fname": fname, "stem_tmpl": stem})

# --- run/test harness cases ---
for variant, v in variants.items():
    for mode, body_key in [("run", "run_body"), ("test", "test_body")]:
        prefix = "main" if mode == "run" else "smoke.test"
        fname = f"{prefix}_{variant}.${{ext}}"
        source_map[fname] = v[body_key]
        for json_output, jtag in [(False, ""), (True, "json_")]:
            vtag = "" if variant == "exponent_one" else "base_one_"
            vtag2 = "exponent_one_" if variant == "exponent_one" else "base_one_"
            name = f"{jtag}{mode}_supports_math_pow_{vtag2}identity_when_browser_harness_is_configured_in_{{ext}}_input"
            cases.append({"kind": "harness", "mode": mode, "variant": variant, "json_output": json_output, "name_tmpl": name, "fname": fname})

print("total cases (pre-matrix):", len(cases))
assert len(cases) == 12, len(cases)

# --- Live-verify every case on ext=js and capture exact stdout ---
verified = {}
for c in cases:
    fname_js = c["fname"].replace("${ext}", "js")
    if c["kind"] == "build":
        args = ["build", "--bundle", "--api", "browser"]
        if c["json_output"]:
            args += ["--output", "json"]
        args += [fname_js]
        rc, out, err, _ = run_kali({fname_js: variants[c["variant"]]["bundle_body"]}, args)
        assert rc == 0, (c["name_tmpl"], rc, out, err)
        stem = fname_js.rsplit(".", 1)[0]
        if c["json_output"]:
            obj = json.loads(out)
            assert obj["schemaVersion"] == 1 and obj["command"] == "build" and obj["success"] is True
            assert obj["exitCode"] == 0
            assert obj["payload"]["artifactKind"] == "bundle" and obj["payload"]["bundleFormat"] == "esm"
        # now build+run harness to get exact stdout
        fn_name = variants[c["variant"]]["fn_name"]
        harness_body = f"const mod = await import(bundleJs.href);\nawait mod.{fn_name}();\n"
        files = {fname_js: variants[c["variant"]]["bundle_body"]}
        # re-run build in a keep dir to build the harness step manually
        import tempfile, subprocess, os
        d = tempfile.mkdtemp()
        with open(os.path.join(d, fname_js), "w") as f:
            f.write(variants[c["variant"]]["bundle_body"])
        subprocess.run(["/workspace/.cache/cargo-target/debug/kali", "build", "--bundle", "--api", "browser", fname_js], cwd=d, check=True, capture_output=True)
        harness_path = os.path.join(d, "browser-bundle-smoke.mjs")
        # write harness via node -e using the same runtime contract shape is overkill; reuse kali's own harness gen would be ideal
        # -- but simplest: shell out to node directly with an ad hoc script mirroring browser_bundle_harness_script
        prelude = f"""import fs from 'node:fs/promises';
import {{ fileURLToPath }} from 'node:url';
const bundleJs = new URL('./{stem}/{stem}.js', import.meta.url);
const wasmUrl = new URL('./{stem}/{stem}.wasm', import.meta.url);
globalThis.fetch = async (input) => {{
  const url = input instanceof URL ? input : new URL(String(input));
  if (url.href === wasmUrl.href) {{
    const bytes = await fs.readFile(fileURLToPath(url));
    return new Response(bytes, {{ headers: {{ 'content-type': 'application/wasm' }} }});
  }}
  throw new Error(`unexpected fetch ${{String(input)}}`);
}};
"""
        with open(harness_path, "w") as f:
            f.write(prelude + harness_body)
        hres = subprocess.run(["node", harness_path], cwd=d, capture_output=True)
        assert hres.returncode == 0, (c["name_tmpl"], hres.stdout, hres.stderr)
        exact_stdout = hres.stdout.decode("utf-8")
        expval = variants[c["variant"]]["expected_value"]
        expected_count = 12 + 36
        assert exact_stdout.count(expval) >= expected_count, (c["name_tmpl"], exact_stdout)
        verified[c["name_tmpl"]] = {"harness_stdout": exact_stdout}
        print("verified build:", c["name_tmpl"], "harness exit", hres.returncode, "stdout len", len(exact_stdout))
    else:
        mode = c["mode"]
        args = []
        if c["json_output"]:
            args += ["--output", "json"]
        args += [mode, "--api", "browser", "--max-threads", "0", "--max-spawned-processes", "0", fname_js]
        env = {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"}
        body = variants[c["variant"]][("run_body" if mode == "run" else "test_body")]
        rc, out, err, _ = run_kali({fname_js: body}, args, env=env)
        assert rc == 0, (c["name_tmpl"], rc, out, err)
        if c["json_output"]:
            obj = json.loads(out)
            assert obj["command"] == mode and obj["success"] is True
            assert obj["payload"]["hostContract"] == "browser-requested"
            assert obj["payload"]["runtimeBackend"] == "browser-harness"
            if mode == "run":
                assert obj["exitCode"] == 0 and obj["payload"]["exitCode"] == 0
            else:
                assert obj["payload"]["total"] == 1 and obj["payload"]["passed"] == 1
                assert obj["payload"]["failed"] == 0 and obj["payload"]["skipped"] == 0
            verified[c["name_tmpl"]] = {"stdout": obj["stdout"]}
        else:
            stdout_text = out.decode("utf-8")
            verified[c["name_tmpl"]] = {"stdout": stdout_text}
        print("verified harness:", c["name_tmpl"], "exit", rc)

import pickle
with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/math_pow_data.pkl', 'wb') as f:
    pickle.dump({"cases": cases, "source_map": source_map, "verified": verified,
                 "variants": variants, "RATIONALE": RATIONALE}, f)
print("OK dumped")
