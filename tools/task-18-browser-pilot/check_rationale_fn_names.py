#!/usr/bin/env python3
"""U8: verify every fn-shaped name cited in a case file's prose really exists.

`audit-case-migration.py` deliberately never reads `rationale`, `name`,
comments or `[source]`; `comment_coverage.py` only checks that source comment
text APPEARS in a rationale, never that a rationale's own assertions are true.
Batch 2 shipped rationales citing source fn names that do not exist. U8 makes
the grep a required per-batch step; this automates it.

It collects every backticked identifier in the case file's `#` header, every
`name`, and every `rationale`, keeps the ones shaped like a Rust fn name, and
checks each against the fn list of the source `.rs` (plus an allowlist of
known non-source identifiers: case-file keys, other crates' items, and tool
names). Anything left over is printed for a human to adjudicate -- it is a
citation that may be invented.

Exits 1 if any cited fn-shaped name is unexplained, 0 otherwise.

Usage: check_rationale_fn_names.py SOURCE.rs TARGET.toml
"""

import os
import re
import sys
import tomllib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from submodules import read_with_submodules  # noqa: E402

BACKTICKED = re.compile(r"`([^`]+)`")
FN_SHAPED = re.compile(r"^[a-z][a-z0-9_]{4,}$")

# Identifiers that legitimately appear in prose but are not source fns:
# case-file/spec vocabulary, step kinds, assertion keys, tool and crate names.
ALLOW = {
    "stdout_contains", "stdout_absent", "stdout_count", "stderr_contains",
    "stderr_absent", "json_null", "json_count", "file_json", "browser_bundle_harness",
    "schemaVersion", "artifactKind", "bundleFormat", "apiSurface", "hostContract",
    "runtimeBackend", "exitCode", "rationale", "constants", "source", "matrix",
    "expand", "substitute", "at_least", "exact", "needle", "passed", "failed", "total",
    # U14/gate vocabulary that appears in EXTRA-OK declarations and gate prose.
    "extra", "missing", "claims", "fidelity", "assertion_strings", "matches",
    "skipped", "success", "command", "payload", "errors", "warnings", "stdout",
    "stderr", "entry", "fields", "ignore", "browser", "bundle", "esm", "node",
    "kali_common", "kali_runtime_contract", "kali_case_runner", "tempdir",
    "browser_bundle_harness_script", "browser_harness_command_parts_for",
    "audit", "contains", "matches", "count", "lines", "starts_with", "ends_with",
    # Named gate machinery a header legitimately cites when it records WHY a
    # gate behaves as it does. `disjunctive_contains_groups` is
    # `audit-case-migration.py`'s rule-11 OR arm (added in batch 6B);
    # `resolve_path_mods` is its U10 submodule resolver. Both are functions in
    # the gate, never in the source under migration, so the prefix rule below
    # can never explain them.
    "disjunctive_contains_groups", "resolve_path_mods", "assertion_diff",
}


def cited_names(toml_path):
    raw = open(toml_path).read()
    doc = tomllib.load(open(toml_path, "rb"))
    blobs = [ln for ln in raw.splitlines() if ln.lstrip().startswith("#")]
    for case in doc.get("case") or []:
        blobs.append(case.get("name", ""))
        blobs.append(case.get("rationale", ""))
    out = set()
    for blob in blobs:
        for m in BACKTICKED.finditer(blob):
            token = m.group(1).strip()
            token = token.split("(")[0].split("::")[-1].strip()
            if FN_SHAPED.match(token):
                out.add(token)
    return out


def source_names(rs_path):
    # U10: a `#[path]` carrier's `#[test]` fns all live in its submodules, and a
    # rationale that names the fn it was migrated from is naming one of those.
    # Reading the carrier alone flagged every single one as "not a fn/binding",
    # i.e. the gate went red on correct prose and could no longer distinguish a
    # real invention. Same submodule resolution `audit-case-migration.py` does.
    text = read_with_submodules(rs_path)
    names = set(re.findall(r"\bfn\s+([a-z_][a-z0-9_]*)", text))
    names |= set(re.findall(r"\blet\s+([a-z_][a-z0-9_]*)", text))
    names |= set(re.findall(r"\b([a-z_][a-z0-9_]*)\s*:", text))
    # `for (command, source_name, source, expected_stdout) in [...]` and
    # `for filename in [...]` -- loop bindings are real identifiers a rationale
    # legitimately cites, and they are the ones these files loop over.
    for m in re.finditer(r"\bfor\s+(\(([^)]*)\)|([a-z_][a-z0-9_]*))\s+in\b", text):
        if m.group(2) is not None:
            names |= {p.strip() for p in m.group(2).split(",") if p.strip()}
        elif m.group(3):
            names.add(m.group(3))
    return names


def sibling_case_stems(toml_path):
    """A rationale may legitimately cross-reference another case file by stem
    (e.g. "see math_log2_log10_bracketed_root.toml"). Those are file names, not
    fns, so they are not invented citations."""
    d = os.path.dirname(os.path.abspath(toml_path))
    return {os.path.splitext(f)[0] for f in os.listdir(d) if f.endswith(".toml")}


def main(argv):
    if len(argv) != 2:
        raise SystemExit(__doc__)
    rs_path, toml_path = argv
    cited = cited_names(toml_path)
    known = source_names(rs_path) | ALLOW | sibling_case_stems(toml_path)
    # A case name is a source fn name with its `_in_<ext>_input` suffix stripped,
    # so accept a cited token that is a prefix of, or prefixed by, a real source
    # fn. Bounded at 8 characters: without a floor, any short binding in `src`
    # (`source`, `command`, `dir`) makes every token starting with it "explained",
    # which would silently swallow real inventions -- this checker going green on
    # the defect it exists to catch is the exact failure mode this project keeps
    # hitting in its instruments.
    src = {s for s in source_names(rs_path) if len(s) >= 8}
    # ONE direction only (fix round 1, I7). The legitimate need is a case name
    # derived from a source fn by stripping its `_in_<ext>_input` suffix -- i.e.
    # the cited token is a PREFIX of a real fn, `s.startswith(n)`.
    #
    # The reverse, `n.startswith(s)`, accepted any token that merely EXTENDS a
    # real fn name -- and that is precisely the realistic invention: a rationale
    # citing `assert_browser_harness_math_log2_log10_bogus` when the helper is
    # `assert_browser_harness_math_log2_log10` passed silently. A wholly
    # unrelated name was caught; the plausible near-miss was not, which is the
    # wrong way round for a gate whose whole purpose is catching plausible-
    # looking prose. Dropped.
    unexplained = sorted(
        n for n in cited
        if n not in known and not any(s.startswith(n) for s in src)
    )
    print(f"{len(cited)} fn-shaped name(s) cited in {os.path.basename(toml_path)}; "
          f"{len(unexplained)} unexplained")
    for n in unexplained:
        print(f"  UNEXPLAINED: `{n}` -- not a fn/binding in {os.path.basename(rs_path)}")
    if unexplained:
        print("U8 CHECK FAILED — a rationale may be citing something that does not exist")
        return 1
    print("U8 CHECK OK — every cited fn-shaped name resolves")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
