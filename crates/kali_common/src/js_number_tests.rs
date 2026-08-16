//! Threshold tests for the single JS number formatter.
//!
//! These pin the two ECMAScript `Number::toString` exponential thresholds from
//! both sides. The 1e21 pair is the interesting one: the formatter is correct
//! there, which is the evidence that R-55 (a literal at or past 1e21 rendering
//! as expanded digits in every sink) is NOT a formatter defect but an upstream
//! classification one. See the register's §7 R-55.
use super::*;

#[test]
fn formats_specials_and_zero() {
    assert_eq!(format_js_number(f64::NAN), "NaN");
    assert_eq!(format_js_number(f64::INFINITY), "Infinity");
    assert_eq!(format_js_number(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(format_js_number(0.0), "0");
    assert_eq!(format_js_number(-0.0), "0");
}

#[test]
fn formats_ordinary_magnitudes_with_shortest_round_trip() {
    assert_eq!(format_js_number(3.5), "3.5");
    assert_eq!(format_js_number(0.1), "0.1");
    assert_eq!(format_js_number(0.30000000000000004), "0.30000000000000004");
    assert_eq!(format_js_number(-42.0), "-42");
}

#[test]
fn formats_small_magnitudes_at_the_js_exponent_threshold() {
    // JS keeps decimal notation down to 1e-6 and switches at 1e-7.
    assert_eq!(format_js_number(1e-6), "0.000001");
    assert_eq!(format_js_number(1e-7), "1e-7");
    assert_eq!(format_js_number(1.0 / 10000000.0), "1e-7");
    assert_eq!(format_js_number(-1e-7), "-1e-7");
}

#[test]
fn formats_large_magnitudes_at_the_js_exponent_threshold() {
    // JS keeps decimal notation up to (excluding) 1e21, with a '+' sign above.
    assert_eq!(format_js_number(1e20), "100000000000000000000");
    assert_eq!(format_js_number(1e21), "1e+21");
    assert_eq!(format_js_number(-1e21), "-1e+21");
    assert_eq!(format_js_number(5e-324), "5e-324");
    assert_eq!(format_js_number(f64::MAX), "1.7976931348623157e+308");
}

#[test]
fn the_formatter_is_not_what_r55_is_about() {
    // R-55 reports `console.log(1e21)` printing 22 literal digits. This asserts
    // the formatter would have rendered it correctly if it had been reached, so
    // the defect is upstream of here and this test is what pins that reasoning.
    //
    // Deliberately SUBSUMED by `formats_large_magnitudes_at_the_js_exponent_threshold`
    // above, whose `assert_eq!(format_js_number(1e21), "1e+21")` implies this
    // `assert_ne!` outright. It is kept for its NAME and this comment: they are
    // the only place in the formatter's own tests where R-55's "the defect is
    // upstream" argument is anchored, and a threshold test renamed or retargeted
    // later would take the argument with it. Do not delete it as redundant.
    assert_ne!(format_js_number(1e21), "1000000000000000000000");
}

/// Differential pin against node's native `String(value)`. Values cross the
/// wire as exact f64 bit patterns so no text round-trip can mask a mismatch.
/// Node is a hard requirement of the harness lanes, so no skip-if-missing.
#[test]
fn matches_node_string_conversion() {
    let values: &[f64] = &[
        3.5,
        0.1,
        0.30000000000000004,
        1e-6,
        1e-7,
        1.0 / 10000000.0,
        -1e-7,
        0.000001234,
        1e20,
        1e21,
        -1e21,
        123456789.123,
        1.5e300,
        5e-324,
        f64::MAX,
        f64::MIN_POSITIVE,
    ];
    let bits: Vec<String> = values
        .iter()
        .map(|value| format!("{:#x}n", value.to_bits()))
        .collect();
    let script = format!(
        "for (const bits of [{}]) {{ const v = new Float64Array(new BigUint64Array([bits]).buffer)[0]; process.stdout.write(String(v) + \"\\n\"); }}",
        bits.join(",")
    );
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(&script)
        .output()
        .expect("node available (required by the harness lanes)");
    assert!(
        output.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let node_lines: Vec<&str> = stdout.trim_end().split('\n').collect();
    assert_eq!(node_lines.len(), values.len());
    for (value, expected) in values.iter().zip(node_lines) {
        assert_eq!(
            format_js_number(*value),
            expected,
            "bits {:#x}",
            value.to_bits()
        );
    }
}
