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
