import sys, pickle
sys.path.insert(0, '/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18')
from toml_emit import toml_string, toml_str_array

with open('/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18/object_keys_data.pkl', 'rb') as f:
    data = pickle.load(f)

cases = data["cases"]
source_map = data["source_map"]

HEADER = """# Migrated from tests/browser_object_keys_harness.rs.
#
# Honest re-pin (PR #16 rev2, family `object-enum`): kali fails closed/loud
# here (12 of this file's 41 worklist members were tagged class B by the
# automated classifier, but direct verification shows every one of them
# panics on this exact assertion too — a loud E5506/runtime-trap failure,
# not a silent wrong value; re-pinned as class A for all 41 members — see
# docs/superpowers/followups/pr16-honest-repin-inventory.md).
#
# All 41 #[test] fns in source reduce to exactly ONE assertion per
# invocation: `assert!(!output.status.success())` -- no stdout/stderr content
# is ever checked, and `--output json` does not change that assertion's
# shape (the helper never parses the JSON in either mode). This is therefore
# the pilot's designated `[matrix]` use beyond file 5's extension-loop
# candidate: the `ext` axis (js/ts/jsx/tsx) is genuinely uniform across every
# case in this file (verified: every one of the 4 distinct fixture bodies
# was directly re-run against the real kali binary here, in both output
# modes, and fails closed) -- so it is hoisted to a file-level [matrix] axis
# rather than expanded into 4x as many named [[case]] entries. 41 #[test]
# fns collapse to 16 [[case]] entries here, matrix-fanned to 64 trials --
# matching the 64 real assertions in source exactly (4 variants x 2 commands
# x 2 json_output x 4 extensions).
"""

RATIONALE = (
    "Migrated from browser_object_keys_harness.rs. Every one of this file's "
    "41 #[test] fns funnels through `assert_browser_harness_object_keys`, "
    "whose entire assertion is `assert!(!output.status.success())` -- fail "
    "closed, no stdout/stderr/JSON content is ever checked. See the file "
    "header for the honest re-pin note this rationale carries from source, "
    "and for why the `ext` axis is a file-level [matrix] here rather than "
    "per-case siblings."
)

out = [HEADER, ""]
out.append("[matrix]")
out.append('ext = ["js", "ts", "jsx", "tsx"]')
out.append("")
out.append("[source]")
for key in sorted(source_map.keys()):
    out.append(f"{toml_string(key)} = {toml_string(source_map[key], multiline=True)}")
out.append("")

for c in cases:
    out.append("[[case]]")
    out.append(f"name = {toml_string(c['name'])}")
    out.append(f"rationale = {toml_string(RATIONALE, multiline=True)}")
    kind, step = c["steps"][0]
    out.append(f"args = {toml_str_array(step['args'])}")
    env = step["env"]
    env_str = "{ " + ", ".join(f"{toml_string(k)} = {toml_string(v)}" for k, v in env.items()) + " }"
    out.append(f"env = {env_str}")
    out.append(f"exit = {toml_string(step['exit'])}")
    out.append("")

text = "\n".join(out).rstrip() + "\n"
outpath = "/workspace/crates/kali_cli/tests/cases/browser/object_keys_harness.toml"
with open(outpath, "w") as f:
    f.write(text)
print("wrote", outpath, len(text), "bytes, cases:", len(cases))
