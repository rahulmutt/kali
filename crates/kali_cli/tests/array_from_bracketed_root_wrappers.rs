//! SPEC §5.11 RETENTION -- CONTROLLER RULING R1, CLASS A (UNREACHABLE-CODE
//! CLAIM). This file stays hand-written per spec §5.11. Adjudicated in Task 15
//! and upheld on re-review; this header was added in Task 19 batch 5, and its
//! absence until now is the whole reason a dispatch listed this file as a
//! migratable CLEAN target.
//!
//! THE PHRASE "hand-written per spec" ABOVE IS LOAD-BEARING, and that is worth
//! knowing before anyone rewords this paragraph. `screen_candidates.py`'s S27
//! arm decides "is this an adjudicated retention?" by matching one of six
//! marker phrases against the `//!` header -- ruling 18's fragile shape, where
//! the gate's input is the prose it is policing. Its `--selftest` DOES fail
//! loudly when the marker stops matching while `citation_sweep.sh` still adopts
//! the file as a retention, which is how this was caught rather than shipped.
//!
//! THE BLOCKING CONSTRUCT, BY NAME AND LINE, RE-MEASURED RATHER THAN CITED.
//! The `json_output: bool` parameter of each helper below is never passed
//! `true` at any call site, and each guards a block that carries claims. Both
//! halves are measured: an always-`false` bool guarding an EMPTY block would
//! block nothing, and reporting one would be a retention with no ground.
//!
//!   `assert_browser_requested`
//!       `if json_output {` at line 346, 16 dead claim(s) in 2 guarded block(s)
//!   `assert_browser_bundle`
//!       `if json_output {` at line 404, 7 dead claim(s) in 2 guarded block(s)
//!
//! Every call site passes `false`, so each `if json_output { … }` block is
//! UNREACHABLE and every literal inside it is DEAD: a value written in the
//! source and asserted by no reachable path.
//!
//! (The line numbers above are ruling 11's self-referential trap in miniature,
//! for the second time in this header: they are lines of THIS file, so the
//! length of this header is an input to them. `declined_headers` therefore
//! renders this header to a FIXED POINT -- render, write, re-measure against
//! the file it just wrote, repeat until two rounds agree -- rather than
//! measuring once against the previous revision.)
//!
//! The enumerating command, run before this sentence was written (ruling 13):
//!
//!   cd /workspace && python3 tools/migration/t19b5_extract.py --declined
//!
//! It is re-run on every generator invocation (`t19b5_extract.check_declined`),
//! and it RAISES if any of these branches ever becomes reachable -- so this
//! retention is re-derived rather than inherited, and a source that grew a
//! `json_output = true` call site would fail the gate instead of staying
//! silently declined.
//!
//! WHY NEITHER THE AUDIT NOR THE FORMAT CAN CARRY IT. Those literals are dead.
//! `audit-case-migration.py` is a literal-coverage tool and cannot see
//! reachability, so it demands all of them of a case file; rule 2 forbids
//! inventing a claim to satisfy it, a value computed but never asserted not
//! being a claim; and rule 3 forbids shipping the resulting red. Controller
//! ruling R1 settles exactly this shape and rules BOTH alternatives out
//! permanently -- a per-file audit exception, and teaching the audit Rust
//! reachability analysis.
//!
//! ONLY SOME TESTS REACH IT, AND THE TRIM WAS QUANTIFIED AND DECLINED. This is
//! NOT a case where U4's whole-file clause applies on its own terms: 2 of
//! this file's 8 test fns never reach a dead-branch helper, so a
//! trim-and-keep IS structurally available. (The attribute is spelled "test
//! fns" and not in full here on purpose: `screen_candidates.py` counts test
//! functions by matching the attribute over the whole file, so writing it in
//! this header would add to the number the screen reports about the file --
//! ruling 11's self-referential trap, in miniature.) It was measured across all four
//! Class A files -- 10 migratable of 36, against 26 retained across four new
//! retention pairs -- and DECLINED by the controller, with the human partner's
//! agreement, on this ground: **U4 exists to stop OVER-retention, and a trim
//! that retains 26 of 36 barely reduces retention** while adding four instances
//! of the apparatus rulings 9, 11, 12 and 19 record as this project's densest
//! defect source. The sibling precedent runs the other way and is what makes
//! the distinction rather than contradicting it: Task 19 batch 2 trimmed
//! `object_has_own_frozen_js_input.rs`, the FIFTH Class A file from the same
//! Task 15 ruling, migrating 4 of its 5. The yield is inverted here.
//!
//! CONSEQUENCE FOR THE GATES. There is no case file for this stem and no trim,
//! so no per-pair gate runs against it at all and there is no red-list to
//! carry: rulings 9, 12 and 19's three-column apparatus is for a retention
//! PAIR, and this is a whole-file retention with no pair. What this header does
//! change is the SCREEN: `screen_candidates.py` now classifies this file
//! `S27_self_documented` instead of CLEAN, which is precisely the U3 mechanism
//! that was missing.
//!
//! Report: `.superpowers/sdd/2026-07-29-test-binary-consolidation/task-19-batch5-report.md`.

use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_runtime_contract::{
    browser_bundle_harness_script, browser_harness_command_parts_for, BROWSER_HARNESS_COMMAND_ENV,
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn standalone_source() -> &'static str {
    r##"const values = [1, 2, 1]; const mapValues = [[1, 2], [1, 3], [4, 5]]; const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from)); const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from)); const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from)); const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from)); const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from)); const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from'])); for (const value of nullishBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of andBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of orBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of andBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of orBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); }"##
}

fn test_source() -> &'static str {
    r##"Kali.test('nullish/logical bracketed globalThis.Array.from wrappers', () => { const values = [1, 2, 1]; const mapValues = [[1, 2], [1, 3], [4, 5]]; const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from)); const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from)); const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from)); const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from)); const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from)); const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from'])); for (const value of nullishBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of andBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of orBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of andBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of orBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } });"##
}

fn browser_run_source() -> &'static str {
    r##"async function browserArrayFromBracketedRootWrappers() {
  const values = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from));
  const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from));
  const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from));
  const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from));
  const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from));
  const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from']));
  for (const value of nullishBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of andBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of orBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of andBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of orBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

browserArrayFromBracketedRootWrappers();
"##
}

fn browser_test_source() -> &'static str {
    r##"Kali.test('nullish/logical bracketed globalThis.Array.from wrappers', () => {
  async function browserArrayFromBracketedRootWrappers() {
    const values = [1, 2, 1];
    const mapValues = [[1, 2], [1, 3], [4, 5]];
    const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from));
    const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from));
    const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from));
    const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from));
    const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from));
    const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from']));
    for (const value of nullishBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of andBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of orBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of andBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of orBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return browserArrayFromBracketedRootWrappers();
});
"##
}

fn browser_bundle_source() -> &'static str {
    r##"// kali-tree-shake: browserArrayFromBracketedRootWrappers
export async function browserArrayFromBracketedRootWrappers() {
  const values = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from));
  const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from));
  const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from));
  const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from));
  const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from));
  const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from']));
  for (const value of nullishBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of andBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of orBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of andBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of orBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn assert_standalone(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        test_source()
    } else {
        standalone_source()
    };
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "stdout: {stdout}");
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

fn assert_browser_requested(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_test_source()
    } else {
        browser_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(BROWSER_HARNESS_COMMAND_ENV, "node")
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
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
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
        assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "stdout: {stdout}");
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

fn assert_browser_bundle(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_source()).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = browser_bundle_harness_script(
        "app",
        false,
        "const mod = await import(bundleJs.href);\nawait mod.browserArrayFromBracketedRootWrappers();\nconsole.log('browser bracketed Array.from wrapper aliases ok');\n",
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_harness_command_parts_for(
        std::env::var(BROWSER_HARNESS_COMMAND_ENV).ok().as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "stdout: {stdout}");
}

#[test]
fn run_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input() {
    assert_standalone("run", "main.js");
}

#[test]
fn test_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input() {
    assert_standalone("test", "smoke.test.js");
}

#[test]
fn run_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested("run", "main.js", false);
}

#[test]
fn run_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_ts_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_requested("run", filename, false);
    }
}

#[test]
fn test_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested("test", "smoke.test.js", false);
}

#[test]
fn test_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_ts_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested("test", filename, false);
    }
}

#[test]
fn build_bundles_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input() {
    assert_browser_bundle("app.js", false);
}

#[test]
fn build_bundles_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_ts_jsx_and_tsx_input(
) {
    for filename in ["app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle(filename, false);
    }
}
