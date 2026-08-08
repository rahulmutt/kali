import sys, re
sys.path.insert(0, '/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18')
from lexer import find_string_literals
from toml_emit import toml_string, toml_str_array
from kali_run import run_kali

RS = open('/workspace/crates/kali_cli/tests/browser_object_keys_harness.rs').read()
lits = find_string_literals(RS)
def L(n):
    return lits[n]['value']

variants = {
    "default": {"run": L(2), "test": L(3)},
    "const_bound": {"run": L(4), "test": L(5)},
    "direct": {"run": L(6), "test": L(7)},
    "global": {"run": L(8), "test": L(9)},
}

prefixes = {"run": "main", "test": "smoke.test"}

cases = []
source_map = {}
for variant, bodies in variants.items():
    for command in ["run", "test"]:
        body = bodies[command]
        prefix = prefixes[command]
        source_key = f"{prefix}.{variant}.${{ext}}"
        source_map[source_key] = body
        for json_output, jprefix in [(False, ""), (True, "json_")]:
            name_variant = "" if variant == "default" else f"{variant}_"
            name = f"{jprefix}{command}_supports_{name_variant}object_keys_iteration_when_browser_harness_is_configured"
            args = []
            if json_output:
                args += ["--output", "json"]
            args += [command, "--api", "browser", "--max-threads", "0", "--max-spawned-processes", "0", source_key]
            step = {
                "args": args,
                "env": {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"},
                "exit": "failure",
            }
            cases.append({"name": name, "source_key": source_key, "steps": [("cli", step)]})

print("total cases (pre-matrix):", len(cases))
assert len(cases) == 16, len(cases)

# Verify: run a representative sample against the real binary (one ext per
# variant/command/json_output combination -- 16 spot checks; source itself
# already documents that ALL 41 original worklist members were directly
# re-verified to fail closed on this exact assertion, so exhaustively
# re-deriving all 64 (variant x command x json x ext) combinations here would
# not be testing anything the source's own history didn't already establish
# uniformly across the ext axis -- but every DISTINCT fixture body is checked
# at least once, live, against the real kali binary).
for variant, bodies in variants.items():
    for command in ["run", "test"]:
        body = bodies[command]
        prefix = prefixes[command]
        fname = f"{prefix}.{variant}.js"
        args = [command, "--api", "browser", "--max-threads", "0", "--max-spawned-processes", "0", fname]
        env = {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"}
        rc, out, err, _ = run_kali({fname: body}, args, env=env)
        assert rc != 0, (variant, command, "expected failure, got", rc, out, err)
        # also verify --output json variant fails closed too
        rc2, out2, err2, _ = run_kali({fname: body}, ["--output", "json"] + args, env=env)
        assert rc2 != 0, (variant, command, "json mode expected failure, got", rc2, out2, err2)
        print("verified fail-closed:", variant, command, "exit", rc, rc2)

import pickle
with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/object_keys_data.pkl', 'wb') as f:
    pickle.dump({"cases": cases, "source_map": source_map}, f)
print("OK dumped")
