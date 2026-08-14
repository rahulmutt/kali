//! Task 18 batch 2 audit escalation: kept 100% hand-written, not migrated.
//!
//! All 8 `#[test]` fns in this file route through
//! `assert_browser_harness_array_from_set_map` (`:264`), which runs 21
//! `assert!(source.contains(...))` self-checks (`:272-292`) on the JS
//! fixture's OWN TEXT -- a dev-time invariant check that the fixture still
//! literally embeds every `Array.from`/bracket-notation/logical-operator
//! variant this file means to exercise -- before the fixture is ever
//! written to disk or `kali` is ever invoked. These are not claims about
//! process output.
//!
//! `audit-case-migration.py` deliberately excludes everything under a
//! migrated case file's `[source]` table from its claim search (see that
//! script's module docstring: "`body` and everything under `[source]` are
//! program text, not claims about behavior"). A full draft migration of
//! this file was built and verified against the real `kali` binary, then
//! audited with the real `audit-case-migration.py` -- AUDIT FAILED, all 21
//! of the literals above reported MISSING, despite being genuinely,
//! verbatim present in the migrated `[source]` fixture body (confirmed by
//! construction: that body is a byte-for-byte copy of this file's own
//! `browser_harness_array_from_set_map_run_source()`). This is the same
//! shape as the Task 18 pilot's `browser_math_pow_exponent_one.rs` finding
//! (see that pilot's own working report -- git-ignored scratch that does
//! not ship, so it is not cited by path), except here EVERY `#[test]` fn (not a
//! subset) reaches the flagged helper unconditionally, so the pilot's
//! §5.11 "trim-and-keep" disposition degenerates to whole-file retention --
//! there is no complementary migratable subset to split off. No case file
//! exists for this target.
//!
//! CONSEQUENCE FOR THE GATES (ruling 9), added retroactively by Task 18 batch 5:
//! THIS FILE HAS NO RED-LIST, and that is the finding, not an omission. Ruling 9
//! addresses a U4 trim-and-keep retention, where the on-disk `.rs` is shorter
//! than the source its case file was migrated from and every literal-comparison
//! gate therefore goes red against the wrong left-hand side. This is a
//! WHOLE-FILE retention: nothing was trimmed, so there is no pre-trim/post-trim
//! divergence and no pre-trim ref to run anything against. There is also no
//! right-hand side -- `verify_pair.sh array_from_set_map_harness` exits 2 with
//! `missing .../cases/browser/array_from_set_map_harness.toml` before running any
//! gate. FIVE of the six gates take a `.rs`/`.toml` pair and therefore cannot
//! run here at all. The SIXTH is the exception, and it changes this paragraph:
//! `batch5_crosscheck.py`, the citation gate that batch 6 wired into
//! `verify_pair.sh`, needs no case file -- it resolves THIS header's own `:N`
//! citations against this very file. So a whole-file retention is no longer
//! ungated: run it directly, as
//! `batch5_crosscheck.py --citations-only array_from_set_map_harness`, because `verify_pair.sh`
//! still exits 2 before reaching it. It exits 0 today. Ruling 11 exempts `:N` from the
//! no-moving-numbers rule only because it is mechanically gated, and this is
//! where that gating applies to a file with no pair. Verified by
//! running it, not assumed. The batch-8 family gate's carve-out for this file is
//! the retention statement above, not a gate red-list.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_array_from_set_map_run_source() -> &'static str {
    r##"async function browserArrayFromSetMapWrappers() {
  const setValues = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  for (const value of Array.from(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis["Array"].from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis["Array"])["from"])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis["Array"]).from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array']).from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array'])["from"])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array'])['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis["Array"]['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((null ?? globalThis["Array"].from))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((true && globalThis["Array"].from))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((false || globalThis["Array"].from))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((null ?? globalThis["Array"]["from"]))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((true && globalThis["Array"]["from"]))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((false || globalThis["Array"]["from"]))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis['Array']['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis.Array['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis['Array'].from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((null ?? globalThis['Array']['from']))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((null ?? globalThis['Array']['from']))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis["Array"]))["from"](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array']))["from"](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array']))['from'](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array).from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array)["from"])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array)['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array))["from"](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array))['from'](new Set(setValues))) {
    console.log(value);
  }
  for await (const value of Array.from(new Set(setValues))) {
    console.log(value);
  }
  for (const entry of Array.from(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of Array.from(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const value of globalThis["Array"]["from"](new Set(setValues))) {
    console.log(value);
  }
  for await (const entry of Object.freeze((globalThis["Array"])["from"])(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

browserArrayFromSetMapWrappers();
"##
}

fn browser_harness_array_from_set_map_test_source() -> &'static str {
    r##"Kali.test('array.from set/map wrappers', () => {
  async function browserArrayFromSetMapWrappers() {
    const setValues = [1, 2, 1];
    const mapValues = [[1, 2], [1, 3], [4, 5]];
    for (const value of Array.from(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze(globalThis["Array"].from)(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((null ?? globalThis['Array']['from']))(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis["Array"])["from"])(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis["Array"]).from)(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis['Array'])["from"])(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis['Array'])['from'])(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze(globalThis['Array']["from"])(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze(globalThis["Array"]['from'])(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((null ?? globalThis["Array"].from))(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((true && globalThis["Array"].from))(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((false || globalThis["Array"].from))(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((null ?? globalThis["Array"]["from"]))(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((true && globalThis["Array"]["from"]))(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((false || globalThis["Array"]["from"]))(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze(globalThis['Array']['from'])(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis["Array"]))["from"](new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis['Array']).from)(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis['Array']))['from'](new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis.Array).from)(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis.Array)["from"])(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis.Array)['from'])(new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis.Array))["from"](new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis.Array))['from'](new Set(setValues))) {
      console.log(value);
    }
    for await (const value of Array.from(new Set(setValues))) {
      console.log(value);
    }
    for (const entry of Array.from(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for await (const entry of Array.from(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return browserArrayFromSetMapWrappers();
});
"##
}

fn assert_browser_harness_array_from_set_map(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_harness_array_from_set_map_test_source()
    } else {
        browser_harness_array_from_set_map_run_source()
    };
    assert!(source.contains(r#"Object.freeze((globalThis["Array"])["from"])"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array']).from)"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array'])["from"])"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array'])['from'])"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array).from)"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array)["from"])"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array)['from'])"#));
    assert!(source.contains(r#"Object.freeze((globalThis["Array"]))["from"]"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array']))['from']"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array))["from"]"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array))['from']"#));
    assert!(source.contains(r#"Object.freeze(globalThis["Array"]['from'])"#));
    assert!(source.contains(r#"Object.freeze((null ?? globalThis['Array']['from']))"#));
    assert!(source.contains(r#"Object.freeze((globalThis["Array"])["from"])"#));
    assert!(source.contains(r#"Object.freeze((globalThis["Array"]).from)"#));
    assert!(source.contains(r#"Object.freeze((null ?? globalThis["Array"].from))"#));
    assert!(source.contains(r#"Object.freeze((true && globalThis["Array"].from))"#));
    assert!(source.contains(r#"Object.freeze((false || globalThis["Array"].from))"#));
    assert!(source.contains(r#"Object.freeze((null ?? globalThis["Array"]["from"]))"#));
    assert!(source.contains(r#"Object.freeze((true && globalThis["Array"]["from"]))"#));
    assert!(source.contains(r#"Object.freeze((false || globalThis["Array"]["from"]))"#));
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert_eq!(json["payload"]["threadTopology"]["totalInstances"], 0);
        assert_eq!(json["payload"]["threadTopology"]["terminatedInstances"], 0);
        assert_eq!(
            json["payload"]["threadTopology"]["liveInstances"],
            serde_json::json!([])
        );
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(
            stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_array_from_new_set_and_new_map_in_js_input() {
    assert_browser_harness_array_from_set_map("run", "main.js", false);
}

#[test]
fn run_supports_array_from_new_set_and_new_map_in_ts_input() {
    assert_browser_harness_array_from_set_map("run", "main.ts", false);
}

#[test]
fn run_supports_array_from_new_set_and_new_map_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_set_map("run", filename, false);
    }
}

#[test]
fn json_run_supports_array_from_new_set_and_new_map_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_set_map("run", filename, true);
    }
}

#[test]
fn test_supports_array_from_new_set_and_new_map_in_js_input() {
    assert_browser_harness_array_from_set_map("test", "smoke.test.js", false);
}

#[test]
fn test_supports_array_from_new_set_and_new_map_in_ts_input() {
    assert_browser_harness_array_from_set_map("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_array_from_new_set_and_new_map_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_array_from_set_map("test", filename, false);
    }
}

#[test]
fn json_test_supports_array_from_new_set_and_new_map_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_array_from_set_map("test", filename, true);
    }
}
