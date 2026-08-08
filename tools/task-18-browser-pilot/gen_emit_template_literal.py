import sys, json
sys.path.insert(0, '/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18')
from toml_emit import toml_string, toml_str_array
from kali_run import run_kali

with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/tld_dump.json') as f:
    bodies = json.load(f)  # keys: run_{ext}, test_{ext}, seq_run_{ext}, seq_test_{ext}, freeze_{ext}

CHUNK_BODY = "export function lazyValue() { return 0n; }"

def expected_stdout(body):
    count = body.count("console.log(String(")
    return "0\n" * count + "main loaded\n"

# (variant, command, ext, json_output) -> (source_filename, chunk_filename, body_key, expect_test_runner)
combos = []
# default variant: full 4x2x2 uniform coverage
for ext in ["js", "ts", "jsx", "tsx"]:
    for command, body_key, prefix in [("run", f"run_{ext}", "main"), ("test", f"test_{ext}", "smoke.test")]:
        for json_output in [False, True]:
            combos.append(("default", command, ext, json_output, body_key))
# sequence variant: js is non-json ONLY (source has no json_run_seq_js /
# json_test_seq_js fn -- verified by reading the source fn list directly).
for command, body_key, prefix in [("run", "seq_run_js", "main"), ("test", "seq_test_js", "smoke.test")]:
    combos.append(("sequence", command, "js", False, body_key))
for ext in ["ts", "jsx", "tsx"]:
    for command, body_key in [("run", f"seq_run_{ext}"), ("test", f"seq_test_{ext}")]:
        for json_output in [False, True]:
            combos.append(("sequence", command, ext, json_output, body_key))
# freeze variant: full 4x2x2 uniform coverage
for ext in ["js", "ts", "jsx", "tsx"]:
    for command, body_key in [("run", f"freeze_{ext}"), ("test", f"freeze_{ext}")]:
        for json_output in [False, True]:
            combos.append(("freeze", command, ext, json_output, body_key))

print("total combos:", len(combos))
assert len(combos) == 46, len(combos)

RATIONALE = (
    "Migrated from browser_template_literal_dynamic_import_harness.rs. "
    "Every case funnels through `assert_browser_requested_template_literal_"
    "dynamic_import`, whose reconciliation comment (Stage P5 Task 5, moved "
    "verbatim into this file's header) explains why the program now "
    "SUCCEEDS with node-correct stdout rather than failing closed. The "
    "fixture bodies are built by `format!` with real `{{`/`}}` doubling "
    "around JS code blocks and the genuine `${name}` template-literal "
    "interpolation (35 `{{` occurrences total in source, the family's "
    "densest brace-collapse trap) -- every resolved fixture text below was "
    "captured by actually EXECUTING the real `format!` calls via a "
    "temporary Rust test (not hand-derived), matching rule 8. `exit = "
    "\"success\"` for every case: this file's whole premise, per the "
    "reconciliation comment, is that these dynamic imports now succeed. "
    "For a `test`-command, non-json case, source asserts `stdout.starts_"
    "with(&expected_stdout)` -- per the standing rule, this is NOT weakened "
    "to `contains`; the exact full `stdout` was captured from the real "
    "binary and pinned exactly instead, strictly stronger than the source's "
    "own prefix check. `--output json` mode already used exact `assert_eq!` "
    "in source (`json[\"stdout\"]` against the full computed "
    "`expected_stdout`), so no strengthening was needed there -- copied "
    "as-is."
)

verified = []
for variant, command, ext, json_output, body_key in combos:
    body = bodies[body_key]
    prefix = "main" if command == "run" else "smoke.test"
    src_filename = f"{prefix}_{variant}.{ext}"
    chunk_filename = f"lazy_{variant}.{ext}"
    body_resolved = body.replace(f"lazy.{ext}", chunk_filename)
    exp = expected_stdout(body_resolved)
    args = []
    if json_output:
        args += ["--output", "json"]
    args += [command, "--api", "browser", "--max-threads", "0", "--max-spawned-processes", "0", src_filename]
    env = {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"}
    files = {src_filename: body_resolved, chunk_filename: CHUNK_BODY}
    rc, out, err, _ = run_kali(files, args, env=env)
    assert rc == 0, (variant, command, ext, json_output, rc, out, err)
    expect_test_runner = command == "test"
    if json_output:
        obj = json.loads(out)
        assert obj["command"] == command
        assert obj["success"] is True
        assert obj["payload"]["hostContract"] == "browser-requested"
        assert obj["payload"]["runtimeBackend"] == "browser-harness"
        if expect_test_runner:
            assert obj["payload"]["passed"] == 1
            assert obj["payload"]["failed"] == 0
        assert obj["stdout"] == exp, (variant, command, ext, "json.stdout mismatch", repr(obj["stdout"]), repr(exp))
    else:
        stdout_text = out.decode("utf-8")
        if expect_test_runner:
            assert stdout_text.startswith(exp), (variant, command, ext, repr(stdout_text), repr(exp))
            assert "ok 1" in stdout_text and "not ok" not in stdout_text
        else:
            assert stdout_text == exp, (variant, command, ext, repr(stdout_text), repr(exp))
    verified.append({
        "variant": variant, "command": command, "ext": ext, "json_output": json_output,
        "src_filename": src_filename, "chunk_filename": chunk_filename,
        "body": body_resolved, "exp": exp,
        "full_stdout_text_mode": (stdout_text if not json_output else None),
        "expect_test_runner": expect_test_runner,
    })
    print("verified:", variant, command, ext, "json" if json_output else "text")

import pickle
with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/tld_data.pkl', 'wb') as f:
    pickle.dump({"verified": verified, "RATIONALE": RATIONALE, "CHUNK_BODY": CHUNK_BODY}, f)
print("OK dumped", len(verified))
