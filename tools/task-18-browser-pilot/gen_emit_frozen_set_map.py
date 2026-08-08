import sys, json
sys.path.insert(0, '/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18')
from lexer import find_string_literals
from toml_emit import toml_string, toml_str_array
from kali_run import run_kali

RS = open('/workspace/crates/kali_cli/tests/browser_frozen_set_map_constructor_result.rs').read()
lits = find_string_literals(RS)
def L(n):
    return lits[n]['value']

bodies = {
    ("plain", "run"): L(2), ("plain", "test"): L(3), ("plain", "bundle"): L(4),
    ("paren", "run"): L(5), ("paren", "test"): L(6), ("paren", "bundle"): L(7),
}

EXPECTED_STDOUT_CORE = "1\n2\n1\n3\n4\n5\n1\n2\n1\n3\n4\n5\n"

cases = []
source_map = {}

RATIONALE = (
    "Migrated from browser_frozen_set_map_constructor_result.rs. Verifies "
    "`Object.freeze(new Set(...))`/`Object.freeze(new Map(...))` iteration "
    "(plain) and the parenthesized-constructor-call variant (including "
    "`(null ?? Set)`/`(false || Map)` logical wrappers), across `run`, "
    "`test`, and `build`(+browser-bundle-harness), each in both plain-text "
    "and `--output json` modes. `ext` is a genuinely uniform file-level "
    "[matrix] axis here (every case exercises all of js/ts/jsx/tsx with an "
    "identical assertion shape, confirmed by reading every #[test] fn: "
    "individual js/ts fns plus a jsx-and-tsx loop fn together cover the "
    "full axis for every (variant, mode, json_output) combination) -- a "
    "second matrix candidate found in this pilot beyond file 5, not used "
    "as the pilot's designated example but noted for the batch cost "
    "estimate. `${ext}`-suffixed filenames are disambiguated with a "
    "`_plain`/`_paren` stem tag because [source] is one flat file-wide "
    "table and source reuses bare filenames (main.js, app.ts, ...) across "
    "variants that each get their own private tempdir in the .rs world; no "
    "assertion ever pins the literal filename, so this changes no claim. "
    "For `--output json` mode's nested `json.stdout` field, the source's "
    "own check is `.contains(...)` -- unexpressible directly (the `json` "
    "key is exact-equality-only per path); the exact value was captured "
    "live from the real binary and pinned exactly, strictly stronger."
)

def stem(fn):
    return fn.rsplit(".", 1)[0]

def make_requested_case(variant, mode, json_output):
    body = bodies[(variant, mode)]
    prefix = {"run": "main", "test": "smoke.test"}[mode]
    fname = f"{prefix}_{variant}.${{ext}}"
    source_map[fname] = body
    jtag = "json_" if json_output else ""
    vtag = "" if variant == "plain" else "parenthesized_"
    name = f"{jtag}{mode}_supports_{vtag}frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured"
    args = []
    if json_output:
        args += ["--output", "json"]
    args += [mode, "--api", "browser", "--max-threads", "0", "--max-spawned-processes", "0", fname]
    lines = [f"args = {toml_str_array(args)}"]
    lines.append('env = { KALI_BROWSER_BUNDLE_HARNESS_COMMAND = "node" }')
    lines.append('exit = "success"')
    if json_output:
        lines.append("__JSON__")  # placeholder filled by emitter with live-verified data
    else:
        lines.append(f"stdout_contains = {toml_str_array([EXPECTED_STDOUT_CORE])}")
        if mode == "test":
            lines.append('stdout_contains = ' + toml_str_array([EXPECTED_STDOUT_CORE]))
        lines.append('stderr = ""')
    return {"name": name, "variant": variant, "mode": mode, "json_output": json_output,
            "fname": fname, "body": body, "lines": lines}


def make_build_case(variant, json_output):
    body = bodies[(variant, "bundle")]
    fname = f"app_{variant}.${{ext}}"
    source_map[fname] = body
    jtag = "json_" if json_output else ""
    vtag = "" if variant == "plain" else "parenthesized_"
    name = f"{jtag}build_emits_{vtag}frozen_set_map_constructor_result_in_all_browser_bundle_input_variants_when_configured"
    return {"name": name, "variant": variant, "mode": "build", "json_output": json_output,
            "fname": fname, "body": body}


requested_cases = []
for variant in ["plain", "paren"]:
    for mode in ["run", "test"]:
        for json_output in [False, True]:
            requested_cases.append(make_requested_case(variant, mode, json_output))

build_cases = []
for variant in ["plain", "paren"]:
    for json_output in [False, True]:
        build_cases.append(make_build_case(variant, json_output))

print("requested cases:", len(requested_cases), "build cases:", len(build_cases))
assert len(requested_cases) == 8 and len(build_cases) == 4

# --- Live-verify each requested-case body on one representative extension,
# in both output modes, and capture the exact json.stdout text. ---
verified_json = {}
for c in requested_cases:
    fname_concrete = c["fname"].replace("${ext}", "js")
    args = [a.replace("${ext}", "js") for a in []]
    mode = c["mode"]
    args = []
    if c["json_output"]:
        args += ["--output", "json"]
    args += [mode, "--api", "browser", "--max-threads", "0", "--max-spawned-processes", "0", fname_concrete]
    env = {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"}
    rc, out, err, _ = run_kali({fname_concrete: c["body"]}, args, env=env)
    assert rc == 0, (c["name"], rc, out, err)
    if c["json_output"]:
        obj = json.loads(out)
        assert obj["schemaVersion"] == 1
        assert obj["command"] == mode
        assert obj["success"] is True
        assert obj["payload"]["hostContract"] == "browser-requested"
        assert obj["payload"]["runtimeBackend"] == "browser-harness"
        assert obj["payload"]["threadTopology"]["totalInstances"] == 0
        assert obj["payload"]["threadTopology"]["terminatedInstances"] == 0
        assert obj["payload"]["threadTopology"]["liveInstances"] == []
        assert obj["stderr"] == ""
        assert obj["errors"] == []
        assert EXPECTED_STDOUT_CORE in obj["stdout"]
        jf = {"stdout": obj["stdout"]}
        if mode == "run":
            assert obj["exitCode"] == 0 and obj["payload"]["exitCode"] == 0
            jf["exitCode"] = True
        else:
            assert obj["payload"]["total"] == 1 and obj["payload"]["passed"] == 1 and obj["payload"]["failed"] == 0
            jf["test_counts"] = True
        verified_json[c["name"]] = jf
    else:
        stdout_text = out.decode("utf-8")
        assert EXPECTED_STDOUT_CORE in stdout_text, (c["name"], stdout_text)
        if mode == "test":
            assert "ok 1" in stdout_text
        assert err == b""
    print("verified requested:", c["name"], "exit", rc)

for c in build_cases:
    fname_concrete = c["fname"].replace("${ext}", "js")
    args = ["build", "--bundle", "--api", "browser"]
    if c["json_output"]:
        args += ["--output", "json"]
    args += [fname_concrete]
    rc, out, err, _ = run_kali({fname_concrete: c["body"]}, args)
    assert rc == 0, (c["name"], rc, out, err)
    if c["json_output"]:
        obj = json.loads(out)
        assert obj["schemaVersion"] == 1 and obj["command"] == "build" and obj["success"] is True
        assert obj["exitCode"] == 0
        assert obj["payload"]["artifactKind"] == "bundle"
        assert obj["payload"]["bundleFormat"] == "esm"
        assert obj["errors"] == []
    print("verified build:", c["name"], "exit", rc)

import pickle
with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/frozen_data.pkl', 'wb') as f:
    pickle.dump({
        "requested_cases": requested_cases, "build_cases": build_cases,
        "source_map": source_map, "verified_json": verified_json,
        "EXPECTED_STDOUT_CORE": EXPECTED_STDOUT_CORE, "RATIONALE": RATIONALE,
    }, f)
print("OK dumped")
