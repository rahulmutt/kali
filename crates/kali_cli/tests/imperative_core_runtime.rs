use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_js(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Runs `source` and asserts the compiler REJECTS it (non-zero exit), returning
/// combined stdout+stderr so tests can assert on the diagnostic. Used for shapes
/// the direct-runtime path cannot lower and must refuse rather than miscompile.
fn run_js_expect_failure(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "expected compilation to be rejected, but it succeeded\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    combined
}

#[test]
fn string_typed_variable_plus_operands_are_rejected() {
    // Codegen's `is_string_valued` recognizes only string/template literals and
    // literal-rooted `+` chains — NOT a variable that holds a string. So ANY `+`
    // with a string-typed variable operand miscompiles (it either integer-adds
    // two string handles or coerces a string handle through `int_to_string`),
    // regardless of the other operand's type. The checker must reject the whole
    // family (E3200) rather than silently emit garbage.

    // string variable + number.
    let diag = run_js_expect_failure("let s = \"x\";\nconsole.log(s + 3);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // number + string variable.
    let diag = run_js_expect_failure("let s = \"x\";\nconsole.log(3 + s);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // string variable + numeric variable.
    let diag = run_js_expect_failure("let s = \"x\";\nlet n = 3;\nconsole.log(s + n);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // string literal + string variable.
    let diag = run_js_expect_failure("let b = \"y\";\nconsole.log(\"x\" + b);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // string variable + string literal.
    let diag = run_js_expect_failure("let b = \"y\";\nconsole.log(b + \"x\");\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // string variable + string variable.
    let diag = run_js_expect_failure("let a = \"x\";\nlet b = \"y\";\nconsole.log(a + b);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // concatenation-result variable + string literal (the result of `a + "z"` is
    // itself a string-typed variable).
    let diag =
        run_js_expect_failure("let a = \"x\";\nlet s = a + \"z\";\nconsole.log(s + \"q\");\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // Reassignment TO a string is tracked (binding became a string after decl).
    let diag = run_js_expect_failure("let s = 5;\ns = \"x\";\nconsole.log(s + 3);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // Reassignment to another string variable is tracked too.
    let diag = run_js_expect_failure("let x = \"hi\";\nlet s = 5;\ns = x;\nconsole.log(s + 3);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // `var`-declared strings are tracked (hoisted binding).
    let diag = run_js_expect_failure("var s = \"x\";\nconsole.log(s + 3);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
}

#[test]
fn literal_rooted_concatenation_and_integer_addition_stay_supported() {
    // None of these has a string-typed *variable* operand, so they must keep
    // compiling and producing correct output.
    assert_eq!(run_js("console.log(\"x\" + 3);\n"), "x3\n");
    assert_eq!(run_js("console.log(\"x\" + \"y\");\n"), "xy\n");
    assert_eq!(run_js("let n = 7;\nconsole.log(\"n=\" + n);\n"), "n=7\n");
    assert_eq!(
        run_js("let n = 7;\nlet m = 16;\nconsole.log(\"Pfannkuchen(\" + n + \") = \" + m);\n"),
        "Pfannkuchen(7) = 16\n"
    );
    // A number after reassignment is not a string: flow-aware detection keeps this
    // compiling and correct.
    assert_eq!(
        run_js("let s = \"x\";\ns = 5;\nconsole.log(s + 1);\n"),
        "6\n"
    );
    // Numeric concatenation into a variable stays numeric (not a false positive).
    assert_eq!(
        run_js("let a = 1;\nlet b = 2;\nlet x = a + b;\nconsole.log(x + 5);\n"),
        "8\n"
    );
}

#[test]
fn relational_operators_compute_booleans() {
    assert_eq!(run_js("console.log(3 < 5);\n"), "1\n");
    assert_eq!(run_js("console.log(5 < 3);\n"), "0\n");
    assert_eq!(run_js("console.log(5 > 3);\n"), "1\n");
    assert_eq!(run_js("console.log(3 >= 3);\n"), "1\n");
    assert_eq!(run_js("console.log(2 <= 1);\n"), "0\n");
}

#[test]
fn functions_return_computed_values() {
    assert_eq!(
        run_js("function id(x) { return x; }\nconsole.log(id(42));\n"),
        "42\n"
    );
    assert_eq!(
        run_js("function add(a, b) { return a + b; }\nconsole.log(add(40, 2));\n"),
        "42\n"
    );
    assert_eq!(
        run_js("function dbl(x) { return x * 2; }\nconsole.log(dbl(21));\n"),
        "42\n"
    );
}

#[test]
fn mutable_locals_round_trip() {
    assert_eq!(run_js("let x = 5;\nconsole.log(x);\n"), "5\n");
    assert_eq!(run_js("let x = 5;\nx = x + 1;\nconsole.log(x);\n"), "6\n");
    assert_eq!(run_js("let x = 0;\nx = 9;\nconsole.log(x);\n"), "9\n");
    assert_eq!(
        run_js("function f() { let x = 5; x = x + 1; return x; }\nconsole.log(f());\n"),
        "6\n"
    );
}

#[test]
fn loops_iterate() {
    // while with a counter and accumulator
    assert_eq!(
        run_js(
            "let s = 0;\nlet i = 0;\nwhile (i < 5) { s = s + i; i = i + 1; }\nconsole.log(s);\n"
        ),
        "10\n"
    );
    // for loop
    assert_eq!(
        run_js("let s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + i; }\nconsole.log(s);\n"),
        "10\n"
    );
    // break out of while(true)
    assert_eq!(
        run_js("let i = 0;\nwhile (true) { if (i >= 3) { break; } i = i + 1; }\nconsole.log(i);\n"),
        "3\n"
    );
    // do-while runs body first
    assert_eq!(
        run_js("let i = 0;\nlet n = 0;\ndo { n = n + 1; i = i + 1; } while (i < 4);\nconsole.log(n);\n"),
        "4\n"
    );
    // recursion now terminates (relational base case + real calls)
    assert_eq!(
        run_js(
            "function s(n) { if (n < 1) { return 0; } return n + s(n - 1); }\nconsole.log(s(5));\n"
        ),
        "15\n"
    );
    // `continue` in a `while` re-tests the condition (skips i === 3): 1 + 2 + 4 + 5 + 6 = 18
    assert_eq!(
        run_js(
            "let s = 0;\nlet i = 0;\nwhile (i < 6) { i = i + 1; if (i === 3) { continue; } s = s + i; }\nconsole.log(s);\n"
        ),
        "18\n"
    );
    // `break` inside an `if` inside a conditional (non-`true`) loop
    assert_eq!(
        run_js(
            "let i = 0;\nwhile (i < 100) { if (i === 4) { break; } i = i + 1; }\nconsole.log(i);\n"
        ),
        "4\n"
    );
}

#[test]
fn integer_arrays_read_write() {
    assert_eq!(
        run_js("const a = new Array(3);\na[0] = 10;\na[1] = 20;\na[2] = a[0] + a[1];\nconsole.log(a[2]);\n"),
        "30\n"
    );
    // dynamic index from a loop variable
    assert_eq!(
        run_js("const a = new Array(5);\nfor (let i = 0; i < 5; i = i + 1) { a[i] = i * i; }\nlet s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + a[i]; }\nconsole.log(s);\n"),
        "30\n"
    );
    // swap via a temp (the fannkuch inner idiom)
    assert_eq!(
        run_js("const a = new Array(2);\na[0] = 7;\na[1] = 9;\nconst t = a[0];\na[0] = a[1];\na[1] = t;\nconsole.log(a[0]);\nconsole.log(a[1]);\n"),
        "9\n7\n"
    );
}

#[test]
fn strict_equality_operators_parse_and_compute() {
    // constant-fold path
    assert_eq!(run_js("console.log(4 === 4);\n"), "1\n");
    assert_eq!(run_js("console.log(4 === 5);\n"), "0\n");
    assert_eq!(run_js("console.log(4 !== 5);\n"), "1\n");
    // dynamic path
    assert_eq!(run_js("let r = 4;\nconsole.log(r === 4);\n"), "1\n");
    assert_eq!(run_js("let r = 4;\nconsole.log(r === 5);\n"), "0\n");
    // === inside a loop condition / if (the fannkuch shape)
    assert_eq!(
        run_js("let r = 0;\nwhile (r !== 4) { r = r + 1; }\nconsole.log(r);\n"),
        "4\n"
    );
    assert_eq!(
        run_js("let s = 0;\nlet i = 0;\nwhile (i < 6) { i = i + 1; if (i === 3) { continue; } s = s + i; }\nconsole.log(s);\n"),
        "18\n"
    );
}

#[test]
fn computed_array_subscripts() {
    // read with a[i+1] / a[i-1]
    assert_eq!(
        run_js("const a = new Array(3);\na[0]=10;\na[1]=20;\na[2]=30;\nlet i = 0;\nconsole.log(a[i + 1]);\n"),
        "20\n"
    );
    // write with a[r-1]
    assert_eq!(
        run_js("const a = new Array(3);\nlet r = 2;\na[r - 1] = 99;\nconsole.log(a[1]);\n"),
        "99\n"
    );
    // the fannkuch shift idiom: perm1[i] = perm1[i+1]
    assert_eq!(
        run_js("const a = new Array(4);\nfor (let i = 0; i < 4; i = i + 1) { a[i] = i; }\nlet i = 0;\nwhile (i < 3) { a[i] = a[i + 1]; i = i + 1; }\nconsole.log(a[0]);\nconsole.log(a[1]);\nconsole.log(a[2]);\n"),
        "1\n2\n3\n"
    );
    // literal-index static fold must still work (regression guard)
    assert_eq!(
        run_js("const a = [10, 20, 30];\nconsole.log(a[0] + a[1] + a[2]);\n"),
        "60\n"
    );
    // identifier index still works (Task 5 path)
    assert_eq!(
        run_js("const a = new Array(2);\na[0]=7;\na[1]=9;\nlet j=1;\nconsole.log(a[j]);\n"),
        "9\n"
    );
}

#[test]
fn runtime_string_building() {
    assert_eq!(run_js("let n = 7;\nconsole.log(\"n=\" + n);\n"), "n=7\n");
    assert_eq!(
        run_js("let n = 7;\nlet m = 16;\nconsole.log(\"Pfannkuchen(\" + n + \") = \" + m);\n"),
        "Pfannkuchen(7) = 16\n"
    );
    // concatenation of a computed integer
    assert_eq!(
        run_js("let x = 20;\nconsole.log(\"v=\" + (x + 1));\n"),
        "v=21\n"
    );
}

#[test]
fn f64_scalar_arithmetic_observed_via_comparison() {
    // Division yields a float; 1.5 < 2 is true (=> 1).
    assert_eq!(run_js("console.log((3 / 2) < 2);\n"), "1\n");
    assert_eq!(run_js("console.log((3 / 2) < 1);\n"), "0\n");
    // int promoted into a float add: 1 + 0.5 = 1.5 < 2.
    assert_eq!(run_js("console.log((1 + 1 / 2) < 2);\n"), "1\n");
    // f64 local round-trips through local.set/get.
    assert_eq!(run_js("let x = 3 / 2;\nconsole.log(x < 2);\n"), "1\n");
    // f64-returning function + f64 param propagation across a call.
    assert_eq!(
        run_js("function half(x) { return 1 / x; }\nconsole.log(half(4) < 1);\n"),
        "1\n"
    );
}

#[test]
fn f64_arithmetic_distinguishes_from_integer_division() {
    // These operands are chosen so i64 truncation and real f64 division DISAGREE:
    // 3/2 is 1.5 (f64) vs 1 (i64 trunc); `1.5 > 1` is true, `1 > 1` is false.
    assert_eq!(run_js("console.log((3 / 2) > 1);\n"), "1\n");
    // promoted int-into-float add: 1 + 0.5 = 1.5 > 1 (f64) vs 1 + 0 = 1, not > 1 (i64).
    assert_eq!(run_js("console.log((1 + 1 / 2) > 1);\n"), "1\n");
    // f64 local round-trip: 1.5 > 1 (f64) vs 1 > 1 (i64).
    assert_eq!(run_js("let x = 3 / 2;\nconsole.log(x > 1);\n"), "1\n");
    // f64 return + f64 param across a call: half(4) = 0.25 > 0 (f64) vs 0 > 0 (i64).
    assert_eq!(
        run_js("function half(x) { return 1 / x; }\nconsole.log(half(4) > 0);\n"),
        "1\n"
    );
}

#[test]
fn integer_programs_are_unchanged_by_repr_plumbing() {
    // Regression guard: pure-int program still prints the same.
    assert_eq!(
        run_js("let s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + i; }\nconsole.log(s);\n"),
        "10\n"
    );
}
