//! The single JS `Number::toString` formatter.
//!
//! Lives here rather than in `kali_runtime` because BOTH the wasmtime host and
//! `kali_codegen`'s static-literal fold must render numbers identically. When
//! they were two functions they disagreed: the fold returned a literal's own
//! source text, so `console.log(1e-7)` printed `0.0000001` while every dynamic
//! lane printed `1e-7`. That divergence is register entry R-32's small-magnitude
//! half, and one shared function is what closes it by construction.
//!
//! `ryu_js` implements the ECMAScript `Number::toString` algorithm, including
//! both exponential thresholds; the arms above it are the cases the algorithm
//! does not cover.
//!
//! JS `String(number)` semantics: `NaN`, `Infinity`, `-Infinity`, `0` for
//! ±0, and the ECMA-262 Number-to-String algorithm (via `ryu-js`) for every
//! other double -- byte-identical to the JS glue mirrors' native
//! `String(value)`, including exponent notation for |x| >= 1e21 and
//! magnitudes below 1e-6.

/// Renders `value` the way JavaScript's `String(number)` does.
pub fn format_js_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    if value == 0.0 {
        // Covers -0.0 too: JS renders both as "0" in string position, and the
        // `==` comparison is true for both.
        return "0".to_owned();
    }
    ryu_js::Buffer::new().format_finite(value).to_owned()
}

#[cfg(test)]
#[path = "js_number_tests.rs"]
mod js_number_tests;
