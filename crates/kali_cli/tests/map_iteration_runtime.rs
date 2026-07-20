use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn assert_map_iteration(command: &str, filename: &str, source: &str, _expected: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

fn map_iteration_run_source() -> &'static str {
    "const values = [[1, 2], [1, 3], [4, 5]]; const mapAlias = Map; const wrappedMapAlias = (mapAlias); const frozenMap = Object.freeze(Map); const frozenGlobalThisMap = Object.freeze(globalThis.Map); const frozenGlobalThisBracketedMap = Object.freeze(globalThis[\"Map\"]); const frozenSingleBracketedMap = Object.freeze(globalThis['Map']); const frozenParenthesizedMap = Object.freeze((Map)); const frozenParenthesizedGlobalThisMap = Object.freeze((globalThis.Map)); const frozenParenthesizedGlobalThisBracketedMap = Object.freeze((globalThis[\"Map\"])); const frozenParenthesizedSingleBracketedMap = Object.freeze((globalThis['Map'])); for (const entry of new Map(values)) { console.log('map constructor iteration ok'); } for (const entry of new mapAlias(values)) { console.log('map constructor iteration ok'); } for (const entry of new globalThis['Map'](values)) { console.log('map constructor iteration ok'); } for await (const entry of new (wrappedMapAlias)(values)) { console.log('map constructor iteration ok'); } const nullishEntries = []; for (const entry of new (null ?? Map)(values)) { nullishEntries.push(JSON.stringify(entry)); } const logicalOrEntries = []; for (const entry of new (false || Map)(values)) { logicalOrEntries.push(JSON.stringify(entry)); } if (nullishEntries.length !== 2 || nullishEntries[0] !== '[1,3]' || nullishEntries[1] !== '[4,5]') { throw new Error('unexpected nullish Map constructor iteration semantics'); } if (logicalOrEntries.length !== 2 || logicalOrEntries[0] !== '[1,3]' || logicalOrEntries[1] !== '[4,5]') { throw new Error('unexpected logical-or Map constructor iteration semantics'); } const frozenMapValues = Object.freeze(values); for (const entry of new Map(frozenMapValues)) { console.log('map constructor iteration ok'); } for (const entry of new (frozenMap)(values)) { console.log('frozen map constructor iteration ok'); } for (const entry of new (frozenGlobalThisMap)(values)) { console.log('frozen globalThis.Map constructor iteration ok'); } for (const entry of new (frozenGlobalThisBracketedMap)(values)) { console.log('frozen globalThis[\"Map\"] constructor iteration ok'); } for (const entry of new (frozenSingleBracketedMap)(values)) { console.log('frozen globalThis[\'Map\'] constructor iteration ok'); } for (const entry of new (frozenParenthesizedMap)(values)) { console.log('frozen parenthesized Map constructor iteration ok'); } for (const entry of new (frozenParenthesizedGlobalThisMap)(values)) { console.log('frozen parenthesized globalThis.Map constructor iteration ok'); } for (const entry of new (frozenParenthesizedGlobalThisBracketedMap)(values)) { console.log('frozen parenthesized globalThis[\"Map\"] constructor iteration ok'); } for (const entry of new (frozenParenthesizedSingleBracketedMap)(values)) { console.log('frozen parenthesized globalThis[\'Map\'] constructor iteration ok'); } for (const entry of new (Object.freeze(Map))(values)) { console.log('frozen direct Map constructor iteration ok'); } for (const entry of new (Object.freeze(globalThis.Map))(values)) { console.log('frozen direct globalThis.Map constructor iteration ok'); } for (const entry of new (Object.freeze(globalThis[\"Map\"]))(values)) { console.log('frozen direct globalThis[\"Map\"] constructor iteration ok'); }\n"
}

fn map_iteration_test_source() -> &'static str {
    r#"Kali.test('map constructor iteration', () => { const values = [[1, 2], [1, 3], [4, 5]]; const mapAlias = Map; const wrappedMapAlias = (mapAlias); const frozenMap = Object.freeze(Map); const frozenGlobalThisMap = Object.freeze(globalThis.Map); const frozenGlobalThisBracketedMap = Object.freeze(globalThis["Map"]); const frozenSingleBracketedMap = Object.freeze(globalThis['Map']); const frozenParenthesizedMap = Object.freeze((Map)); const frozenParenthesizedGlobalThisMap = Object.freeze((globalThis.Map)); const frozenParenthesizedGlobalThisBracketedMap = Object.freeze((globalThis["Map"])); const frozenParenthesizedSingleBracketedMap = Object.freeze((globalThis['Map'])); for (const entry of new Map(values)) { console.log('map constructor iteration ok'); } for (const entry of new mapAlias(values)) { console.log('map constructor iteration ok'); } for (const entry of new globalThis['Map'](values)) { console.log('map constructor iteration ok'); } for await (const entry of new (wrappedMapAlias)(values)) { console.log('map constructor iteration ok'); } const nullishEntries = []; for (const entry of new (null ?? Map)(values)) { nullishEntries.push(JSON.stringify(entry)); } const logicalOrEntries = []; for (const entry of new (false || Map)(values)) { logicalOrEntries.push(JSON.stringify(entry)); } if (nullishEntries.length !== 2 || nullishEntries[0] !== '[1,3]' || nullishEntries[1] !== '[4,5]') { throw new Error('unexpected nullish Map constructor iteration semantics'); } if (logicalOrEntries.length !== 2 || logicalOrEntries[0] !== '[1,3]' || logicalOrEntries[1] !== '[4,5]') { throw new Error('unexpected logical-or Map constructor iteration semantics'); } const frozenMapValues = Object.freeze(values); for (const entry of new Map(frozenMapValues)) { console.log('map constructor iteration ok'); } for (const entry of new (frozenMap)(values)) { console.log('frozen map constructor iteration ok'); } for (const entry of new (frozenGlobalThisMap)(values)) { console.log('frozen globalThis.Map constructor iteration ok'); } for (const entry of new (frozenGlobalThisBracketedMap)(values)) { console.log('frozen globalThis["Map"] constructor iteration ok'); } for (const entry of new (frozenSingleBracketedMap)(values)) { console.log('frozen globalThis[\'Map\'] constructor iteration ok'); } for (const entry of new (frozenParenthesizedMap)(values)) { console.log('frozen parenthesized Map constructor iteration ok'); } for (const entry of new (frozenParenthesizedGlobalThisMap)(values)) { console.log('frozen parenthesized globalThis.Map constructor iteration ok'); } for (const entry of new (frozenParenthesizedGlobalThisBracketedMap)(values)) { console.log('frozen parenthesized globalThis["Map"] constructor iteration ok'); } for (const entry of new (frozenParenthesizedSingleBracketedMap)(values)) { console.log('frozen parenthesized globalThis[\'Map\'] constructor iteration ok'); } for (const entry of new (Object.freeze(Map))(values)) { console.log('frozen direct Map constructor iteration ok'); } for (const entry of new (Object.freeze(globalThis.Map))(values)) { console.log('frozen direct globalThis.Map constructor iteration ok'); } for (const entry of new (Object.freeze(globalThis["Map"]))(values)) { console.log('frozen direct globalThis["Map"] constructor iteration ok'); } });
"#
}

#[test]
fn run_supports_map_constructor_iteration_in_js_input() {
    assert_map_iteration(
        "run",
        "main.js",
        map_iteration_run_source(),
        "map constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\n",
    );
}

#[test]
fn run_supports_map_constructor_iteration_in_ts_input() {
    assert_map_iteration(
        "run",
        "main.ts",
        map_iteration_run_source(),
        "map constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\n",
    );
}

#[test]
fn test_supports_map_constructor_iteration_in_js_input() {
    assert_map_iteration("test", "smoke.test.js", map_iteration_test_source(), "ok 1");
}

#[test]
fn test_supports_map_constructor_iteration_in_ts_input() {
    assert_map_iteration("test", "smoke.test.ts", map_iteration_test_source(), "ok 1");
}

#[test]
fn run_supports_map_constructor_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_map_iteration(
            "run",
            filename,
            map_iteration_run_source(),
            "map constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\n",
        );
    }
}

#[test]
fn test_supports_map_constructor_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_map_iteration("test", filename, map_iteration_test_source(), "ok 1");
    }
}
