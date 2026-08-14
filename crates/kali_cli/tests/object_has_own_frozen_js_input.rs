//! U4 TRIM-AND-KEEP RETENTION (Task 19 batch 2). Four of this file's five
//! `#[test]` fns migrated to `cases/misc/object_has_own_frozen_js_input.toml`;
//! the one that remains stays hand-written per spec §5.11.
//!
//! PRE-TRIM REF: 8bb67edb9d0632fe42f3f41b7ff9050264409b4f
//!
//! THE BLOCKING CONSTRUCT, BY NAME AND LINE.
//! `assert_frozen_object_has_own` (`:132`) asserts nothing beyond
//! `output.status.success()` (`:154`) for any test in this file. Its
//! `json_output` branch and its `command == "run"` branch are both UNREACHABLE:
//! the only caller is
//! `check_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input` (`:187`),
//! which passes `json_output = false` and `command = "check"`, so neither branch
//! is ever taken. Eleven literals live in those two branches -- the marker
//! string the run branch greps for, and the ten JSON keys the json branch
//! indexes.
//!
//! WHY NEITHER THE AUDIT NOR THE FORMAT CAN CARRY IT. Those eleven are DEAD
//! LITERALS: values written in the source but asserted by no reachable path.
//! `audit-case-migration.py` is a literal-coverage tool and cannot see
//! reachability, so it demands all eleven of the case file; rule 2 forbids
//! inventing a claim to satisfy it, a value computed but never asserted not
//! being a claim; and rule 3 forbids shipping the resulting red. Controller
//! ruling R1 settles this shape -- an unreachable-code claim goes to a §5.11
//! retention, and both alternatives (a per-file audit exception, and teaching
//! the script Rust reachability analysis) are explicitly and permanently ruled
//! out.
//!
//! ONLY SOME TESTS REACH IT -- exactly one of five, which is why this is a TRIM
//! and not a whole-file retention. U4's second clause makes whole-file
//! legitimate only when EVERY test reaches the construct; here four of five do
//! not. Those four route through a disjoint fails-closed helper that shares no
//! assertion with this one, and they are migrated. Derived rather than chosen:
//! the enumerating command and its output are in the case file's header and in
//! the report.
//!
//! CONSEQUENCE FOR THE GATES -- THREE COLUMNS (rulings 9 and 12). The retained
//! half carries literal claims of its own, so the audit is red against BOTH the
//! post-trim file and the pre-trim blob, and the correct left-hand side is the
//! MIGRATED COMPLEMENT built mechanically by `migrated_complement.py`:
//!
//!   gate                          post-trim  pre-trim  complement  correct side
//!   audit-case-migration.py       RED        RED       GREEN       complement
//!   check_fixtures.py             GREEN      GREEN     GREEN       complement
//!   comment_coverage.py           RED        GREEN     GREEN       complement
//!   check_extra_claims.py         RED        GREEN     GREEN       pre-trim
//!   check_rationale_fn_names.py   RED        GREEN     RED         pre-trim
//!
//! MEASURED, NOT PREDICTED -- and two earlier versions of this table were
//! wrong, which is why it says so. The first predicted three cells it had not
//! run. The second OMITTED `check_rationale_fn_names.py` entirely, which is
//! literally the failure ruling 9 was minted for: a gate that goes red on a
//! retention pair through the standard entry point, and is not on the red-list.
//! `verify_pair.sh <stem> --family misc --pretrim <the ref above>` surfaces
//! FOUR reds; all four are in the table.
//!
//! THE CORRECT SIDE IS PER GATE, BY DIRECTION OF CHECK -- not prose-vs-claims,
//! which is what this header said before and what the reviewer disproved by
//! construction. A FORWARD coverage gate asks "did everything in the source
//! reach the case file", so it wants the migrated complement: give it the
//! pre-trim blob and it reports the retained half's content as missing.
//! `comment_coverage.py` is one of those, and it is green on both older sides
//! here ONLY because this trim's retained half happens to carry no comments --
//! add one and pre-trim goes red while the complement stays green. A REVERSE
//! existence gate asks "does everything the case file cites exist in the
//! source", so it wants the pre-trim blob, which is the only side carrying
//! both halves' names.
//!
//! The row that decides the pair is still the audit's: red against BOTH older
//! sides and green against the complement, which is ruling 12's discriminator
//! for a trim whose RETAINED half carries literal claims of its own.
//!
//! Reproduce the column that decides the pair:
//!
//!   cd "$(git rev-parse --show-toplevel)"
//!   git show 8bb67edb9d0632fe42f3f41b7ff9050264409b4f:crates/kali_cli/tests/object_has_own_frozen_js_input.rs \
//!     > /tmp/pre.rs
//!   python3 tools/task-18-browser-pilot/migrated_complement.py /tmp/pre.rs \
//!     crates/kali_cli/tests/object_has_own_frozen_js_input.rs > /tmp/complement.rs
//!   python3 scripts/audit-case-migration.py /tmp/complement.rs \
//!     crates/kali_cli/tests/cases/misc/object_has_own_frozen_js_input.toml
//!
//! The full reasoning is this header plus the reproduction above, both of
//! which ship. The batch's working report lived in git-ignored scratch and
//! is deliberately not cited: a citation that cannot resolve from a clean
//! checkout is worse than no citation.

use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_common::{
    object_has_own_frozen_callable_condition_source, object_has_own_frozen_callable_source,
    object_has_own_property_call_frozen_callable_condition_source,
    object_has_own_property_call_frozen_callable_source,
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn frozen_object_has_own_source() -> String {
    let frozen_callable_condition_source = format!(
        "{} || {}",
        object_has_own_frozen_callable_condition_source("wrapped", r#""a""#),
        object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#)
    );
    let frozen_callable_source = format!(
        "{} {}",
        object_has_own_frozen_callable_source(),
        object_has_own_property_call_frozen_callable_source()
    );
    format!(
        r#"const object = Object.freeze(Object.fromEntries([["a", 1], ["b", 2]]));
const alias = object;
const wrapped = (0, alias);
const hasOwn = Object.hasOwn;
const singleQuotedHasOwn = globalThis['Object']['hasOwn'];
const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];
const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);
const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);
{}
if (!Object.hasOwn(wrapped, "a") || !Object["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"]["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"].hasOwn(wrapped, "a") || !singleQuotedHasOwn(wrapped, "a") || !parenthesizedSingleQuotedHasOwn(wrapped, "a") || !frozenSingleQuotedHasOwn(wrapped, "a") || !frozenParenthesizedSingleQuotedHasOwn(wrapped, "a") || {} ||
  !Object.prototype.hasOwnProperty.call(wrapped, "a")) {{
  throw new Error('unexpected frozen Object.hasOwn result');
}}
console.log('frozen object hasOwn ok');
"#,
        frozen_callable_source, frozen_callable_condition_source
    )
}

fn assert_frozen_object_has_own<S: AsRef<str>>(
    command: &str,
    filename: &str,
    source: S,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source.as_ref()).expect("write source");

    let mut output = Command::new(kali_bin());
    output.current_dir(dir.path());
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
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

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert!(json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("frozen object hasOwn ok"));
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["payload"]["skipped"], 0);
        }
    } else if command == "run" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("frozen object hasOwn ok"),
            "stdout: {stdout}"
        );
    }
}

#[test]
fn check_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_frozen_object_has_own("check", filename, frozen_object_has_own_source(), false);
    }
}
