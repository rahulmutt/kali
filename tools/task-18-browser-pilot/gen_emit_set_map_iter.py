import sys, json
sys.path.insert(0, '/tmp/claude-1000/-workspace/b356efad-2db0-402c-90e5-60e87c3d691d/scratchpad/t18')
from lexer import find_string_literals
from toml_emit import toml_string, toml_str_array
from kali_run import run_kali

RS = open('/workspace/crates/kali_cli/tests/browser_set_map_iteration_bundle.rs').read()
lits = find_string_literals(RS)
def L(n):
    return lits[n]['value']

set_src = None
map_src = None
for l in lits:
    if l['value'].startswith('// kali-tree-shake: browserSetIteration'):
        set_src = l['value']
    if l['value'].startswith('// kali-tree-shake: browserMapIteration'):
        map_src = l['value']
assert set_src and map_src

# Live-verify E5506's stream in both output modes, for both fixtures, on one
# extension (the file's own 4x4 matrix already establishes ext-uniformity;
# the STREAM question -- which the "narrow an absence claim" rule cares about
# -- does not depend on extension at all, so one extension is the right unit
# to check here, not a reason to skip verification).
for label, src in [("set", set_src), ("map", map_src)]:
    rc, out, err, _ = run_kali({"app.js": src}, ["build", "--bundle", "--api", "browser", "app.js"])
    assert rc != 0
    assert b"E5506" in err and b"E5506" not in out, (label, out, err)
    rc2, out2, err2, _ = run_kali({"app.js": src}, ["build", "--bundle", "--api", "browser", "--output", "json", "app.js"])
    assert rc2 != 0
    obj = json.loads(out2)
    assert obj["errors"][0]["code"] == "E5506"
    assert err2 == b""
    print(label, "verified: text mode -> stderr only; json mode -> errors[0].code == E5506, stderr empty")

source_map = {
    "app_set.${ext}": set_src,
    "app_map.${ext}": map_src,
}

RATIONALE = (
    "Migrated from browser_set_map_iteration_bundle.rs. Honest re-pin (PR "
    "#16 rev2): kali fails closed/loud here; see "
    "docs/superpowers/followups/pr16-honest-repin-inventory.md. Source "
    "asserts `stderr.contains(\"E5506\") || stdout.contains(\"E5506\")` -- an "
    "OR across streams, not two codes (rule 11's shape, applied to streams "
    "instead of codes). Verified directly against the real binary: in "
    "non-json mode E5506 appears on stderr only (never stdout); in "
    "`--output json` mode the diagnostic is JSON-encoded on stdout as "
    "`errors[0].code` and stderr is empty. Pinned to the stream/field that "
    "actually carries it in each mode -- a verified strengthening of the "
    "source's OR, carrying the disjunction sentence here per rule 11 -- "
    "rather than reproducing the OR as `stdout_absent`+`stderr_contains`. "
    "This is the pilot's designated extension-loop / [matrix] candidate "
    "(task brief): all 4 #[test] fns loop `for extension in [\"js\",\"ts\","
    "\"jsx\",\"tsx\"]` with an identical fail-closed assertion, so `ext` is "
    "hoisted to a file-level [matrix] axis rather than 4 named siblings per "
    "case."
)

cases = []
for label, prefix in [("set", "app_set"), ("map", "app_map")]:
    for json_output, jprefix in [(False, ""), (True, "json_")]:
        name = f"{jprefix}build_emits_{label}_constructor_iteration_in_js_ts_jsx_and_tsx_input"
        source_key = f"{prefix}.${{ext}}"
        args = ["build", "--bundle", "--api", "browser"]
        if json_output:
            args += ["--output", "json"]
        args += [source_key]
        step = {"args": args, "exit": "failure"}
        if json_output:
            # `errors` is a real JSON ARRAY at runtime, but flatten_expected
            # treats a TOML *array* as one leaf (exact-length-and-contents
            # equality against all 3 diagnostics kali emits here, which is
            # not what this case claims). Writing it as a nested TOML TABLE
            # keyed "0" instead makes flatten_expected produce the dotted
            # path "errors.0.code" -- and jsonpath.rs's `lookup`/`step`
            # dispatch by the ACTUAL node's JSON type, so a numeric TOML
            # table key against a real JSON array is still read as an index
            # (see model.rs's dotted-path doc comment). This asserts only
            # the first diagnostic's code, matching what this case actually
            # claims (E5506 is present), not the exact diagnostic count.
            step["json_errors0code"] = True
            step["stderr"] = ""
        else:
            step["stderr_contains"] = ["E5506"]
        cases.append({"name": name, "steps": step})

HEADER = """# Migrated from tests/browser_set_map_iteration_bundle.rs.
#
# Honest re-pin (PR #16 rev2): kali fails closed/loud here; see
# docs/superpowers/followups/pr16-honest-repin-inventory.md.
#
# All 4 #[test] fns loop `for extension in ["js", "ts", "jsx", "tsx"]` around
# an identical fail-closed assertion (`stderr.contains("E5506") ||
# stdout.contains("E5506")`) -- the pilot's designated extension-loop /
# [matrix] candidate (task brief). Hoisted to a file-level [matrix] axis: 4
# #[test] fns -> 4 [[case]] entries, matrix-fanned to 16 trials, matching the
# 16 real per-extension assertions in source exactly.
#
# STREAM NOTE: source's assertion is an OR across STREAMS (stderr.contains
# OR stdout.contains the SAME code), not an OR of two different codes --
# rule 11's shape, applied to stream selection rather than code selection.
# Verified directly against the real binary for both fixtures: in text mode
# E5506 is written to stderr only; in `--output json` mode it is JSON-encoded
# on stdout as `errors[0].code` and stderr is empty. Each mode is pinned to
# the stream/field that actually carries it -- a verified strengthening,
# with the disjunction sentence carried into every case's rationale per rule
# 11, not silently narrowed.
"""

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
    step = c["steps"]
    out.append(f"args = {toml_str_array(step['args'])}")
    out.append(f"exit = {toml_string(step['exit'])}")
    if "stderr_contains" in step:
        out.append(f"stderr_contains = {toml_str_array(step['stderr_contains'])}")
    if "stderr" in step:
        out.append(f"stderr = {toml_string(step['stderr'])}")
    if step.get("json_errors0code"):
        out.append('json = { errors = { "0" = { code = "E5506" } } }')
    out.append("")

text = "\n".join(out).rstrip() + "\n"
outpath = "/workspace/crates/kali_cli/tests/cases/browser/set_map_iteration_bundle.toml"
with open(outpath, "w") as f:
    f.write(text)
print("wrote", outpath, len(text), "bytes, cases:", len(cases))
