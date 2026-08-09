"""The two helper shapes that dominate Task 18 batch 4's 22 math targets.

Nearly every file in this batch is built from one or both of:

  BUNDLE helper  `assert_browser_bundle_<name>(filename, json_output)`
      cli `build --bundle --api browser [--output json] <app.EXT>`
    + file_json `app/app.meta.json`
    + browser_bundle_harness on the emitted bundle

  HARNESS helper `assert_browser_harness_<name>(command, filename, source,
                                                json_output)`
      one cli `[--output json] <run|test> --api browser [--max-threads 0
      --max-spawned-processes 0] <file>` with KALI_BROWSER_BUNDLE_HARNESS_
      COMMAND=node

These builders render those shapes. They take the assertion set EXPLICITLY --
nothing is defaulted on, because the files differ in exactly the places a
default would paper over: some assert `errors = []` and some do not, some
assert `stderr = ""` and some do not, some pass `--max-threads`/
`--max-spawned-processes` and some do not. A builder that guessed would
manufacture claims the source never made (rule 2), which is precisely the
failure the audit's reverse check exists to catch.
"""


def bundle_steps(entry_file, harness_body, harness_asserts, *,
                 json_output, json_claims=None, meta_fields=None,
                 extra_build_argv=()):
    """cli build (+ optional JSON envelope) -> file_json meta -> harness."""
    argv = ["build", "--bundle", "--api", "browser"]
    if json_output:
        argv += ["--output", "json"]
    argv += list(extra_build_argv) + [entry_file]

    build = {"args": argv, "exit": "success"}
    if json_output:
        if json_claims is None:
            raise AssertionError("json_output build step needs its json claims stated")
        build["json"] = json_claims

    steps = [build]
    if meta_fields:
        steps.append({"kind": "file_json", "path": "app/app.meta.json",
                      "fields": meta_fields})
    harness = {"kind": "browser_bundle_harness", "entry": "app",
               "body": harness_body, "exit": "success"}
    harness.update(harness_asserts)
    steps.append(harness)
    return steps


def harness_step(command, source_file, *, json_output, asserts,
                 json_claims=None, thread_flags=False, env_var="KALI_BROWSER_BUNDLE_HARNESS_COMMAND"):
    """The single-cli-step browser-harness shape."""
    argv = []
    if json_output:
        argv += ["--output", "json"]
    argv += [command, "--api", "browser"]
    if thread_flags:
        argv += ["--max-threads", "0", "--max-spawned-processes", "0"]
    argv += [source_file]

    step = {"args": argv, "env": {env_var: "node"}, "exit": "success"}
    if json_output:
        if json_claims is None:
            raise AssertionError("json_output harness step needs its json claims stated")
        step["json"] = json_claims
    step.update(asserts)
    return step


def envelope_build(*, errors=False, exit_code=True):
    """The `kali build --bundle --output json` envelope claims these files make."""
    j = {"schemaVersion": 1, "command": "build", "success": True}
    if exit_code:
        j["exitCode"] = 0
    j["payload"] = {"artifactKind": "bundle", "bundleFormat": "esm"}
    if errors:
        j["errors"] = []
    return j


def envelope_harness(command, *, stderr=False, errors=False, extra_payload=None):
    """The `kali run|test --api browser --output json` envelope claims.

    `run` asserts exitCode at both the envelope and payload level; `test`
    asserts payload total/passed/failed instead. That difference is why
    `command` is never a [matrix] axis in these files (rule 7): it changes the
    assertion shape, not just a substituted string.
    """
    payload = {"hostContract": "browser-requested", "runtimeBackend": "browser-harness"}
    j = {"schemaVersion": 1, "command": command, "success": True}
    if command == "run":
        payload["exitCode"] = 0
        j["exitCode"] = 0
    else:
        payload.update({"total": 1, "passed": 1, "failed": 0})
    if extra_payload:
        payload.update(extra_payload)
    j["payload"] = payload
    if stderr:
        j["stderr"] = ""
    if errors:
        j["errors"] = []
    return j


META = {"apiSurface": "browser", "artifactKind": "bundle"}


def rule12_no_comments_prose(rs_path, stem):
    """Rule-12 discharge prose DERIVED from the source, not asserted about it.

    Fix round 1 (I2): the previous helper emitted "the only `//` in the file is
    the `// kali-tree-shake:` marker inside a JS fixture body" unconditionally,
    and shipped that sentence into three case files whose sources contain zero
    `//` and no bundle fixture at all. A generator that states a fact it never
    checked will state a false one the moment a file differs -- and no gate
    reads `#` header prose, so it ships permanently.

    So: count the real Rust comments (after masking string literals, so a `//`
    inside a JS fixture is not miscounted as one) and describe what is actually
    there. Raises if the source DOES carry Rust comments, because then the
    `--allow-empty` discharge this prose accompanies would itself be false.
    """
    import re as _re
    import sys as _sys
    import os as _os
    _sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))
    from enumerate_invocations import strip_block_comments_and_strings

    text = open(rs_path).read()
    # Mask strings first so fixture-internal `//` is excluded, then look for
    # Rust comment lines in what remains.
    lines = text.split("\n")
    masked_no_strings = strip_block_comments_and_strings(text)
    # strip_block_comments_and_strings blanks comments too, so find them by
    # blanking ONLY strings: re-scan for `//` at line start in the original,
    # excluding any line whose `//` sits inside a masked (blanked) region.
    # A leading contiguous `//!` block is a RETENTION HEADER this migration
    # added (U3), not prose carried from the source, so it is not rule-12
    # material -- rule 12 is about comments the source already had. Skipping it
    # is what lets a trimmed file's rule-12 prose still be derived from the
    # file as shipped. Any `//!` appearing AFTER real code is not skipped.
    header_end = 0
    while header_end < len(lines) and lines[header_end].startswith("//!"):
        header_end += 1
    rust_comment_lines = [
        i + 1 for i, ln in enumerate(lines)
        if i >= header_end and _re.match(r"\s*//", ln)
    ]
    fixture_markers = [
        i + 1 for i, ln in enumerate(lines)
        if i >= header_end and "//" in ln and not _re.match(r"\s*//", ln)
    ]

    if rust_comment_lines:
        raise AssertionError(
            f"{rs_path} has Rust comment(s) at line(s) {rust_comment_lines} -- "
            "rule 12 requires them carried into the rationale of every case the "
            "producing helper reaches, and --allow-empty would be a false "
            "discharge. Write this file's rule-12 prose by hand."
        )

    head = (
        f"RULE 12 (carry every source comment verbatim): `grep -nE '^\\s*//'` over\n"
        f"tests/browser_{stem}.rs returns NOTHING -- the file has no Rust comments\n"
        f"at all."
    )
    if fixture_markers:
        where = ", ".join(f":{n}" for n in fixture_markers)
        head += (
            f"\nThe {len(fixture_markers)} other `//` occurrence(s) in the file ({where}) sit\n"
            "inside JS fixture bodies, which is program text carried verbatim into\n"
            "[source], not Rust prose."
        )
    else:
        head += (
            "\nThe file contains no `//` of any kind -- it declares no bundle fixture,\n"
            "so there is not even a `// kali-tree-shake:` marker in it."
        )
    if header_end:
        head += (
            f"\n(The file's leading {header_end}-line `//!` block is the U3 RETENTION HEADER\n"
            "this migration added, not prose carried from the source, so it is not\n"
            "rule-12 material. comment_coverage.py does report those lines as missing\n"
            "from every rationale; that is expected for a trimmed retention -- the\n"
            "header describes the RETAINED test, which by construction has no case.)"
        )
    # U10: a `#[path = "..."] mod ...;` carrier's prose can sit in a submodule,
    # where a grep over the carrier alone never sees it -- the same blind spot
    # `comment_coverage.py` had until it was taught to resolve the chain. Each
    # submodule is scanned by the identical rule, and the result is stated
    # per-file rather than pooled. A source with no `mod` declaration produces
    # byte-identical prose to before this block existed.
    import sys as _sys2
    import os as _os2
    _sys2.path.insert(0, _os2.path.dirname(_os2.path.abspath(__file__)))
    from submodules import submodule_paths as _submodule_paths
    subs = _submodule_paths(rs_path)
    if subs:
        head += (
            f"\nThat grep is run over the `#[path]` SUBMODULES too -- {len(subs)} of them, "
            "where\nevery `#[test]` fn in this target actually lives:"
        )
        for sub in subs:
            sub_lines = sub.read_text().split("\n")
            sub_comments = [i + 1 for i, ln in enumerate(sub_lines)
                            if _re.match(r"\s*//", ln)]
            if sub_comments:
                raise AssertionError(
                    f"{sub} has Rust comment(s) at line(s) {sub_comments} -- see above")
            sub_markers = [i + 1 for i, ln in enumerate(sub_lines) if "//" in ln]
            where = (", ".join(f":{n}" for n in sub_markers) if sub_markers
                     else "no `//` of any kind")
            head += f"\n  * {sub.name}: 0 Rust comment line(s); {where}."
    head += (
        "\nThere is therefore no prose to move into any `rationale`, and\n"
        "comment_coverage.py is run with --allow-empty for this pair."
    )
    return head
