import sys, pickle, json
sys.path.insert(0, '/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18')
from toml_emit import toml_string, toml_str_array

with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/atan2_data.pkl', 'rb') as f:
    data = pickle.load(f)

build_cases = data["build_cases"]
source_bodies = data["source_bodies"]
run_cases = data["run_cases"]

out = []
out.append("# Migrated from tests/browser_math_atan2_bracketed_root.rs and its sibling")
out.append("# submodule directory tests/browser_math_atan2_bracketed_root/{run,build}.rs")
out.append("# (the `#[path]` submodule shape: the top-level file declares zero top-level")
out.append("# #[test] fns and pulls its 29 tests in via `#[path = \"...\"] mod run;` /")
out.append("# `mod build;`). No Rust comments exist anywhere in the three source files, so")
out.append("# there is no prose to move verbatim into `rationale` here.")
out.append("")

# [source]: one entry per distinct fixture body used by the build.rs-derived
# cases (keyed by an internal disambiguating key, not the bare filename,
# since e.g. app.ts carries three DIFFERENT bodies across different cases).
all_source = dict(source_bodies)
for c in run_cases:
    key, body = c["source_key"], c["source_body"]
    if key in all_source:
        assert all_source[key] == body, f"conflicting [source] body for {key!r}"
    else:
        all_source[key] = body

out.append("[source]")
for key in sorted(all_source.keys()):
    out.append(f"{toml_string(key)} = {toml_string(all_source[key], multiline=True)}")
out.append("")

def emit_step_inline(step_kind, fields):
    lines = []
    if step_kind != "cli":
        lines.append(f"kind = {toml_string(step_kind)}")
    if "args" in fields:
        lines.append(f"args = {toml_str_array(fields['args'])}")
    if "env" in fields:
        env = fields["env"]
        env_str = "{ " + ", ".join(f"{toml_string(k)} = {toml_string(v)}" for k, v in env.items()) + " }"
        lines.append(f"env = {env_str}")
    if "exit" in fields:
        lines.append(f"exit = {toml_string(fields['exit'])}")
    if "path" in fields:
        lines.append(f"path = {toml_string(fields['path'])}")
    if "entry" in fields:
        lines.append(f"entry = {toml_string(fields['entry'])}")
    if "body" in fields:
        lines.append(f"body = {toml_string(fields['body'], multiline=True)}")
    if "stdout_contains" in fields:
        lines.append(f"stdout_contains = {toml_str_array(fields['stdout_contains'])}")
    if "fields" in fields:
        # nested table -> inline TOML table
        def render(v):
            if isinstance(v, dict):
                return "{ " + ", ".join(f"{toml_string(k)} = {render(vv)}" for k, vv in v.items()) + " }"
            if isinstance(v, bool):
                return "true" if v else "false"
            if isinstance(v, int):
                return str(v)
            if isinstance(v, list):
                return "[" + ", ".join(render(x) for x in v) + "]"
            return toml_string(v)
        lines.append(f"fields = {render(fields['fields'])}")
    if "json" in fields:
        def render(v):
            if isinstance(v, dict):
                return "{ " + ", ".join(f"{toml_string(k)} = {render(vv)}" for k, vv in v.items()) + " }"
            if isinstance(v, bool):
                return "true" if v else "false"
            if isinstance(v, int):
                return str(v)
            if isinstance(v, list):
                return "[" + ", ".join(render(x) for x in v) + "]"
            return toml_string(v)
        lines.append(f"json = {render(fields['json'])}")
    return lines


def emit_case(case, rationale=None):
    out.append("[[case]]")
    out.append(f"name = {toml_string(case['name'])}")
    if rationale:
        out.append(f"rationale = {toml_string(rationale, multiline=True)}")
    steps = case["steps"]
    if len(steps) == 1:
        kind, fields = steps[0]
        for line in emit_step_inline(kind, fields):
            out.append(line)
    else:
        for kind, fields in steps:
            out.append("")
            out.append("[[case.step]]")
            for line in emit_step_inline(kind, fields):
                out.append(line)
    out.append("")


# build.rs-derived cases
build_rationale_generic = (
    "Migrated from browser_math_atan2_bracketed_root/build.rs. Builds a browser "
    "bundle (`kali build --bundle --api browser`), asserts the emitted "
    "`app/app.meta.json` metadata, then runs the bundle glue under the browser-"
    "bundle-harness contract and checks the printed atan2 result."
)
for c in build_cases:
    key = c["source_key"]
    # patch the [source] key in the args/case's cli step and file_json path
    steps = c["steps"]
    emit_case(c, rationale=build_rationale_generic)

run_rationale_generic = (
    "Migrated from browser_math_atan2_bracketed_root/run.rs. A single #[test] fn "
    "in source loops over multiple (command, filename, json_output) combinations "
    "sharing one assertion shape; split here into one named sibling per "
    "combination (rule 5), each with its own [source] fixture. `kali <run|test> "
    "--api browser` with KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node. In "
    "`--output json` mode the exact `json.stdout` value was captured from the "
    "real built binary and pinned exactly -- strictly stronger than the source's "
    "own `.contains(\"0\")` substring check on that same field, which the case "
    "file format cannot express directly (`json` only supports exact equality "
    "per path, not substring)."
)
for c in run_cases:
    emit_case(c, rationale=run_rationale_generic)

text = "\n".join(out).rstrip() + "\n"
outpath = "/workspace/crates/kali_cli/tests/cases/browser/math_atan2_bracketed_root.toml"
with open(outpath, "w") as f:
    f.write(text)
print("wrote", outpath, len(text), "bytes")
print("cases:", len(build_cases) + len(run_cases))
