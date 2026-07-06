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
fn string_typed_variable_plus_operands_flow_at_runtime() {
    // Runtime string value flow: codegen's `is_string_valued` consults the
    // repr axis (`Repr::String`) for identifiers and calls, so a variable
    // proven to hold a string lowers `+` to real runtime concatenation
    // (matching JS output) instead of being rejected with E3200. The E3200
    // gate stays for string sources the repr axis cannot prove (see
    // `mixed_string_number_bindings_are_rejected` below and
    // `runtime_string_value_flow.rs`).

    // string variable + number.
    assert_eq!(run_js("let s = \"x\";\nconsole.log(s + 3);\n"), "x3\n");
    // number + string variable.
    assert_eq!(run_js("let s = \"x\";\nconsole.log(3 + s);\n"), "3x\n");
    // string variable + numeric variable.
    assert_eq!(
        run_js("let s = \"x\";\nlet n = 3;\nconsole.log(s + n);\n"),
        "x3\n"
    );
    // string literal + string variable.
    assert_eq!(run_js("let b = \"y\";\nconsole.log(\"x\" + b);\n"), "xy\n");
    // string variable + string literal.
    assert_eq!(run_js("let b = \"y\";\nconsole.log(b + \"x\");\n"), "yx\n");
    // string variable + string variable.
    assert_eq!(
        run_js("let a = \"x\";\nlet b = \"y\";\nconsole.log(a + b);\n"),
        "xy\n"
    );
    // concatenation-result variable + string literal (the result of `a + "z"`
    // is itself a string-typed variable).
    assert_eq!(
        run_js("let a = \"x\";\nlet s = a + \"z\";\nconsole.log(s + \"q\");\n"),
        "xzq\n"
    );
    // `var`-declared strings flow too (hoisted binding).
    assert_eq!(run_js("var s = \"x\";\nconsole.log(s + 3);\n"), "x3\n");
}

#[test]
fn mixed_string_number_bindings_are_rejected() {
    // A binding that holds a string at one program point and a number at
    // another cannot get a single runtime repr: the repr inference is
    // flow-insensitive, so codegen would read a raw integer as a string
    // handle (or vice versa). REJECT-DON'T-MISCOMPILE: the whole family
    // fails to compile — E3200 when the resolver still sees a string-typed
    // operand in `+` (the repr axis records a conflict instead of
    // `Repr::String`, so the gate is NOT suppressed), E5506 when only the
    // repr shape conflict catches it (no string-typed `+` at the use point).

    // Reassignment TO a string after a numeric init.
    let diag = run_js_expect_failure("let s = 5;\ns = \"x\";\nconsole.log(s + 3);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // Reassignment to another string variable is tracked too.
    let diag = run_js_expect_failure("let x = \"hi\";\nlet s = 5;\ns = x;\nconsole.log(s + 3);\n");
    assert!(diag.contains("E3200"), "expected E3200, got: {diag}");
    // Reassignment to a NUMBER after a string init: the resolver's flow-aware
    // string flag is cleared (no E3200), but the repr conflict still rejects
    // (previously this ran as integer arithmetic; with codegen now trusting
    // `Repr::String`, compiling it would read the raw 5 as a string handle).
    let diag = run_js_expect_failure("let s = \"x\";\ns = 5;\nconsole.log(s + 1);\n");
    assert!(
        diag.contains("E5506") && diag.contains("both a string and a number"),
        "expected E5506 string/number conflict, got: {diag}"
    );
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
    // NOTE: `let s = "x"; s = 5; console.log(s + 1);` used to run here as
    // integer arithmetic (flow-aware string tracking cleared the string flag).
    // With codegen now trusting `Repr::String` for identifier reads, that
    // program would read the raw 5 as a string handle, so the flow-insensitive
    // repr axis rejects the string/number mix instead — pinned in
    // `mixed_string_number_bindings_are_rejected`.
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
    // (1/2)+(1/2) = 1.0 under f64 (=> 1<1 is false => 0); under i64 it is 0+0=0 (=> 0<1 => 1).
    // Discriminating assertion: FAILS under i64 lowering.
    assert_eq!(run_js("console.log((1 / 2) + (1 / 2) < 1);\n"), "0\n");
}

#[test]
fn f64_local_init_and_reassign_promote() {
    // Integer-literal init into an inferred-f64 local, then plain reassign with a
    // float rhs: 0 promoted to 0.0, then 0.0 + 0.5 = 0.5 < 1.
    assert_eq!(
        run_js("let t = 0;\nt = t + 1 / 2;\nconsole.log(t < 1);\n"),
        "1\n"
    );
    // Loop accumulator seeded with an integer literal: 0 + 0.5*3 = 1.5 < 2.
    assert_eq!(
        run_js(
            "let s = 0;\nfor (let i = 0; i < 3; i = i + 1) { s = s + 1 / 2; }\nconsole.log(s < 2);\n"
        ),
        "1\n"
    );
    // f64 compound-assign: 0.5 += 0.5 = 1.0 < 2.
    assert_eq!(
        run_js("let a = 1 / 2;\na += 1 / 2;\nconsole.log(a < 2);\n"),
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

#[test]
fn f64_arrays_store_and_load() {
    // a is a float array (element written from a division).
    assert_eq!(
        run_js("const a = new Array(2);\na[0] = 3 / 2;\nconsole.log(a[0] < 2);\n"),
        "1\n"
    );
    // read-modify across elements stays float.
    assert_eq!(
        run_js(
            "const a = new Array(2);\na[0] = 1 / 2;\na[1] = a[0] + a[0];\nconsole.log(a[1] < 2);\n"
        ),
        "1\n" // 1.0 < 2
    );
    // integer arrays are unchanged.
    assert_eq!(
        run_js("const a = new Array(2);\na[0] = 10;\na[1] = 20;\nconsole.log(a[0] + a[1]);\n"),
        "30\n"
    );
}

#[test]
fn array_length_reads_header() {
    assert_eq!(
        run_js("const a = new Array(3);\nconsole.log(a.length);\n"),
        "3\n"
    );
    // length drives a loop bound (the spectral-norm idiom).
    assert_eq!(
        run_js("const a = new Array(4);\nlet n = 0;\nfor (let i = 0; i < a.length; i = i + 1) { n = n + 1; }\nconsole.log(n);\n"),
        "4\n"
    );
}

#[test]
fn array_fill_initializes_all_elements() {
    // integer fill
    assert_eq!(
        run_js("const a = new Array(3).fill(7);\nconsole.log(a[0] + a[1] + a[2]);\n"),
        "21\n"
    );
    // float fill: a is a float array (used in a float add), fill(1) stores 1.0.
    assert_eq!(
        run_js("const a = new Array(2).fill(1);\nconsole.log((a[0] + 1 / 2) < 2);\n"),
        "1\n" // 1.0 + 0.5 = 1.5 < 2
    );
    // I6-1 strengthening: genuinely gate the F64Store fill path. `a` is F64 via
    // the `a[0] = 1/2` store; a[1] was filled with 1.0. 1.0 > 0.5 => 1. A broken
    // or wrong-width F64 fill would leave a[1] ~0, yielding 0 > 0.5 => 0.
    assert_eq!(
        run_js("const a = new Array(2).fill(1);\na[0] = 1 / 2;\nconsole.log(a[1] > a[0]);\n"),
        "1\n"
    );
}

// ---- Task 6b: array parameters passed between functions --------------------

#[test]
fn array_params_int_store_across_call() {
    // A callee stores into its array param; the caller sees the mutation.
    assert_eq!(
        run_js(
            "function store(v){v[0]=7;}\nconst u=new Array(3);\nstore(u);\nconsole.log(u[0]);\n"
        ),
        "7\n"
    );
}

#[test]
fn array_params_int_read_across_call() {
    // A callee reads and returns an element of its array param.
    assert_eq!(
        run_js(
            "function get(v){return v[0];}\nconst u=new Array(2);\nu[0]=5;\nconsole.log(get(u));\n"
        ),
        "5\n"
    );
}

#[test]
fn array_params_int_length_loop_sum() {
    // `.length` and element reads on an array param drive a loop.
    assert_eq!(
        run_js("function sum(v){let s=0;for(let i=0;i<v.length;i=i+1){s=s+v[i];}return s;}\nconst u=new Array(3);\nu[0]=1;u[1]=2;u[2]=3;\nconsole.log(sum(u));\n"),
        "6\n"
    );
}

#[test]
fn array_params_float_store_across_call() {
    // A float store through an array param must use F64Store (was E4201 crash).
    assert_eq!(
        run_js("function store(v){v[0]=1/2;}\nconst u=new Array(3);\nstore(u);\nconsole.log(u[0] > 0);\n"),
        "1\n"
    );
}

#[test]
fn array_params_float_round_trip() {
    // Float stored in one callee, read back in another: 0.5 < 1 => 1.
    assert_eq!(
        run_js("function store(v){v[0]=1/2;}\nfunction get(v){return v[0];}\nconst u=new Array(2);\nstore(u);\nconsole.log(get(u) < 1);\n"),
        "1\n"
    );
}

#[test]
fn array_params_float_fill_interproc_spectral_norm_shape() {
    // spectral-norm shape: fill(1) makes a float array, a callee overwrites [0]
    // with 0.5; u[1] (=1.0) > u[0] (=0.5) => 1.
    assert_eq!(
        run_js("function store(v){v[0]=1/2;}\nconst u=new Array(3).fill(1);\nstore(u);\nconsole.log(u[1] > u[0]);\n"),
        "1\n"
    );
}

#[test]
fn array_param_store_with_call_rhs_across_two_calls() {
    // spectral-norm multi-call pattern: two functions that loop over an array
    // param and store into another, chained through a driver. `A` returns a
    // float (`/`), and `Au` uses `u.length` on a param it never subscripts.
    // Previously the `.length` node was misclassified as a float element read,
    // making the loop condition emit `f64.lt` with an i64 `.length` operand
    // (E4201: expected f64, found i64), and `u.length` fell back to a
    // placeholder 0 so the loop never ran. Must now compute w[0]=A(0,0)=1 and
    // copy it to v[0], so v[0] > 0 => 1.
    let source = "\
function A(i,j){ return 1 / (i + j + 1); }\n\
function Au(u, v) { for (let i = 0; i < u.length; i = i + 1) { v[i] = A(i, 0); } }\n\
function Atu(u, v) { for (let i = 0; i < u.length; i = i + 1) { v[i] = u[i]; } }\n\
function AtAu(u, v, w) { Au(u, w); Atu(w, v); }\n\
const u = new Array(2).fill(1);\n\
const v = new Array(2);\n\
const w = new Array(2);\n\
AtAu(u, v, w);\n\
console.log(v[0] > 0);\n";
    assert_eq!(run_js(source), "1\n");
}

#[test]
fn array_param_store_with_float_call_rhs() {
    // `v[i] = <call>()` where the callee returns a float, stored into a float
    // array param. The store path must keep the call result as f64 (no bogus
    // i64/f64 stack-type mismatch). half(0)=0.5 => u[0] < 1 => 1.
    let source = "\
function half(i){ return 1 / (i + 2); }\n\
function fillIt(v){ for (let i = 0; i < v.length; i = i + 1) { v[i] = half(i); } }\n\
const u = new Array(2);\n\
fillIt(u);\n\
console.log(u[0] < 1);\n";
    assert_eq!(run_js(source), "1\n");
}

#[test]
fn array_param_length_only_receiver_is_array() {
    // A param used ONLY via `.length` (never subscripted) must still be treated
    // as an array so `.length` reads the real header, not a placeholder 0.
    let source = "\
function count(u){ let n = 0; for (let i = 0; i < u.length; i = i + 1) { n = n + 1; } return n; }\n\
const a = new Array(3).fill(1);\n\
console.log(count(a));\n";
    assert_eq!(run_js(source), "3\n");
}

#[test]
fn float_literals_emit_as_f64() {
    // A bare non-integer numeric literal must lower to an `f64.const`, not a
    // string handle. Previously these fell through the integer parser into the
    // string-interning path, producing an i64 string handle where an f64 was
    // expected (E4201: WebAssembly translation error).
    assert_eq!(run_js("console.log(1.5 > 1);\n"), "1\n");
    assert_eq!(run_js("console.log(0.5 < 1);\n"), "1\n");
    // float literal stored into an f64 local (inferred via repr inference).
    assert_eq!(run_js("let x = 1.5;\nconsole.log(x > 1);\n"), "1\n");
    assert_eq!(run_js("let y = 0.5;\nconsole.log(y < 1);\n"), "1\n");
    // float literal in mixed arithmetic: 1.5 + 1/2 = 2.0.
    assert_eq!(run_js("console.log((1.5 + 1 / 2) < 3);\n"), "1\n");
    assert_eq!(run_js("console.log((1.5 + 1 / 2) > 1);\n"), "1\n");
    // byte-identity guards: integer + string literals unaffected.
    assert_eq!(run_js("console.log(5);\n"), "5\n");
    assert_eq!(run_js("console.log(\"hi\");\n"), "hi\n");
}

#[test]
fn math_sqrt_runtime_f64() {
    // non-perfect-square: was FEATURE_UNAVAILABLE, now a real f64 sqrt.
    assert_eq!(run_js("console.log(Math.sqrt(2) < 2);\n"), "1\n"); // 1.414… < 2
    assert_eq!(run_js("console.log(Math.sqrt(2) < 1);\n"), "0\n");
    // perfect square still constant-folds correctly.
    assert_eq!(run_js("console.log(Math.sqrt(9) < 4);\n"), "1\n"); // 3 < 4
                                                                   // sqrt of a computed float (the spectral-norm shape).
    assert_eq!(
        run_js("let r = 1 / 4;\nconsole.log(Math.sqrt(r) < 1);\n"),
        "1\n"
    ); // 0.5 < 1
}

#[test]
fn to_fixed_formats_floats() {
    assert_eq!(run_js("console.log((1.5).toFixed(1));\n"), "1.5\n");
    assert_eq!(run_js("console.log((1 / 3).toFixed(6));\n"), "0.333333\n");
    assert_eq!(
        run_js("console.log((1 / 2).toFixed(9));\n"),
        "0.500000000\n"
    );
    // integer value formatted to fixed decimals.
    assert_eq!(run_js("console.log((2 / 1).toFixed(3));\n"), "2.000\n");
    // sqrt then format (spectral-norm output shape).
    assert_eq!(
        run_js("console.log(Math.sqrt(2).toFixed(9));\n"),
        "1.414213562\n"
    );
}

#[test]
fn heavy_loop_runs_under_raised_default_fuel_budget() {
    // Regression for the default CPU-fuel budget raise in
    // `RuntimeCtx::run` (crates/kali_runtime/src/execute.rs): with no
    // sandbox policy, the fuel budget used to default to 10_000 *
    // 1_000 = 10M fuel. A plain 2,000,000-iteration counting loop
    // consumes well over 10M fuel (manually confirmed to trap with a
    // runtime-trap diagnostic under a temporarily-reverted 10_000
    // default: breaks somewhere between 650K and 800K iterations) but
    // comfortably completes under the new 60_000 * 1_000 = 60M fuel
    // default (breaks only between ~3.7M and 3.9M iterations), leaving
    // ample margin in both directions. This must run to completion with
    // the correct triangular-number output rather than trapping.
    let source = "\
let n = 2000000;\n\
let sum = 0;\n\
for (let i = 0; i < n; i = i + 1) {\n\
  sum = sum + i;\n\
}\n\
console.log(sum);\n";
    assert_eq!(run_js(source), "1999999000000\n");
}

#[test]
fn exponent_notation_literals_run() {
    assert_eq!(run_js("console.log(2e3);"), "2000\n");
    assert_eq!(run_js("console.log((1.5e1).toFixed(1));"), "15.0\n");
    assert_eq!(run_js("console.log((1e-2).toFixed(2));"), "0.01\n");
}

#[test]
fn object_shape_mismatch_is_rejected() {
    let combined = run_js_expect_failure(
        "let p = { x: 1.0 };\np = { y: 2.0 };\np.y = 3.0;\nconsole.log(p.y);\n",
    );
    assert!(
        combined.contains("5506"),
        "expected E5506 gate, got: {combined}"
    );
}

/// Review fix (promotion hole): a structurally-unsupported object literal
/// (non-identifier property key) written and read through the same local
/// binding must be rejected with E5506 instead of silently falling through
/// to the buggy fold-lane codegen (which ignores the write and prints `0`;
/// node prints `2`). Before the fix, this compiled and ran with no error.
#[test]
fn locally_written_structural_object_literal_is_rejected() {
    let combined = run_js_expect_failure("const p = {\"a-b\": 1};\np.c = 2;\nconsole.log(p.c);\n");
    assert!(
        combined.contains("5506"),
        "expected E5506 gate, got: {combined}"
    );
}

/// Review fix (promotion hole): a structurally-unsupported object literal
/// passed as a call argument (via a const binding, not a direct literal
/// argument — which Task 3 already rejected) and field-read inside the
/// callee must also be rejected with E5506. Before the fix, this compiled
/// and printed `0` (node prints `undefined`).
#[test]
fn structural_object_literal_passed_as_call_argument_is_rejected() {
    let combined = run_js_expect_failure(
        "function f(o) { return o.c; }\nconst o = {\"a-b\": 1};\nconsole.log(f(o));\n",
    );
    assert!(
        combined.contains("5506"),
        "expected E5506 gate, got: {combined}"
    );
}

/// Review fix (IMPORTANT, fold-first both ways): a read-only, non-
/// materialized object literal with a known shape must NOT reject on an
/// unknown-field read (matches node's `undefined`); the same shape, once
/// materialized by a write, must still reject on an unknown-field read.
#[test]
fn unknown_field_read_is_fold_first_until_materialized() {
    assert_eq!(
        run_js("const p = { x: 1.0 };\nconsole.log(p.y);\n"),
        "0\n",
        "a read-only unknown-field access must stay on the fold lane and compile"
    );

    let combined = run_js_expect_failure("const p = { x: 1.0 };\np.x = 2.0;\nconsole.log(p.y);\n");
    assert!(
        combined.contains("5506"),
        "expected E5506 gate once the object is materialized, got: {combined}"
    );
}

#[test]
fn object_field_write_and_read_round_trip() {
    assert_eq!(
        run_js("const p = { x: 1.0 };\np.x = p.x + 1.5;\nconsole.log(p.x.toFixed(1));\n"),
        "2.5\n"
    );
}

#[test]
fn object_field_read_through_alias() {
    assert_eq!(
        run_js(
            "const p = { x: 1.0, y: 2.5 };\np.x = 4.0;\nconst q = p;\nconsole.log((q.x + q.y).toFixed(1));\n"
        ),
        "6.5\n"
    );
}

#[test]
fn integer_object_field_round_trip() {
    assert_eq!(
        run_js("const p = { n: 3 };\np.n = p.n + 4;\nconsole.log(p.n);\n"),
        "7\n"
    );
}

#[test]
fn array_of_object_literals_reads_and_writes() {
    assert_eq!(
        run_js(
            "const a = [{ x: 1.0 }, { x: 2.0 }];\na[1].x = 5.0;\nconsole.log((a[0].x + a[1].x).toFixed(1));\n"
        ),
        "6.0\n"
    );
}

#[test]
fn array_element_alias_mutation_is_shared() {
    assert_eq!(
        run_js(
            "const a = [{ x: 1.5 }, { x: 2.0 }];\nconst b = a[0];\nb.x = b.x + 1.0;\nconsole.log(a[0].x.toFixed(1));\n"
        ),
        "2.5\n"
    );
}

#[test]
fn objects_cross_function_boundaries() {
    let src = "\
function mk(v) { return { x: v }; }\n\
function getx(p) { return p.x; }\n\
const a = mk(3.5);\nconsole.log(getx(a).toFixed(1));\n";
    assert_eq!(run_js(src), "3.5\n");
}

#[test]
fn factory_array_advance_shape_miniature() {
    let src = "\
function mk(x, vx) { return { x: x, vx: vx }; }\n\
function advance(bs, dt) {\n\
  for (let i = 0; i < bs.length; i = i + 1) {\n\
    const b = bs[i];\n\
    b.x = b.x + dt * b.vx;\n\
  }\n\
}\n\
const bs = [mk(1.0, 2.0), mk(0.5, 4.0)];\n\
advance(bs, 0.5);\n\
console.log((bs[0].x + bs[1].x).toFixed(2));\n";
    assert_eq!(run_js(src), "4.50\n");
}

#[test]
fn factory_returned_objects_are_distinct_instances() {
    let src = "\
function mk(v) { return { x: v }; }\n\
const p = mk(1.0);\n\
const q = mk(2.0);\n\
q.x = 5.0;\n\
console.log((p.x + q.x).toFixed(1));\n";
    assert_eq!(run_js(src), "6.0\n"); // p.x=1.0 unchanged, q.x=5.0 — distinct instances
}

#[test]
fn console_log_of_object_reference_is_rejected() {
    let combined = run_js_expect_failure("const p = { x: 1.0 };\np.x = 2.0;\nconsole.log(p);\n");
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}

#[test]
fn object_in_arithmetic_is_rejected() {
    let combined =
        run_js_expect_failure("const p = { x: 1.0 };\np.x = 2.0;\nconsole.log(p + 1);\n");
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}

#[test]
fn unknown_field_write_is_rejected() {
    let combined = run_js_expect_failure("const p = { x: 1.0 };\np.x = 2.0;\np.z = 1.0;\n");
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}

#[test]
fn object_literal_direct_argument_is_rejected() {
    let combined = run_js_expect_failure(
        "function f(o) { return o.x; }\nconsole.log(f({ x: 1.0 }).toFixed(1));\n",
    );
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}

#[test]
fn object_reference_reassignment_still_compiles() {
    // `q = p` aliases; both refer to the same object. Must NOT be E5506-rejected.
    assert_eq!(
        run_js("const p = { x: 1.0 };\nlet q = { x: 2.0 };\nq.x = 3.0;\nq = p;\nconsole.log(q.x.toFixed(1));\n"),
        "1.0\n"
    );
}

#[test]
fn module_consts_read_from_functions() {
    assert_eq!(
        run_js("const K = 3;\nfunction f() { return K + 1; }\nconsole.log(f());\n"),
        "4\n"
    );
    assert_eq!(
        run_js(
            "const PI = 3.141592653589793;\nconst SOLAR_MASS = 4 * PI * PI;\nfunction m() { return 9.54791938424326609e-4 * SOLAR_MASS; }\nconsole.log(m().toFixed(9));\n"
        ),
        "0.037693675\n"
    );
    assert_eq!(
        run_js(
            "const DPY = 365.24;\nfunction v(x) { return x * DPY; }\nconsole.log(v(2.0).toFixed(2));\n"
        ),
        "730.48\n"
    );
}

#[test]
fn shadowing_local_wins_over_module_const() {
    assert_eq!(
        run_js("const K = 3;\nfunction f() { const K = 10; return K + 1; }\nconsole.log(f());\n"),
        "11\n"
    );
}

#[test]
fn for_of_loop_var_shadows_module_const() {
    // for-of binding `K` shadows module `const K`; must compile and use the loop value.
    // node ground truth: prints `6` (1+2+3).
    assert_eq!(
        run_js("const K = 2.5;\nfunction f() {\n  let s = 0;\n  for (const K of [1, 2, 3]) { s = s + K; }\n  return s;\n}\nconsole.log(f());\n"),
        "6\n"
    );
}

#[test]
fn catch_param_shadows_module_const() {
    // catch (K) shadows module `const K`. `TryStatement`/`CatchClause`
    // currently have NO lowering support anywhere in the direct-runtime
    // codegen pipeline (`kali_codegen` has no "try"/"catch"/"throw"
    // instruction handling at all) — confirmed independently of this shadow
    // by running a plain, non-colliding `catch (e) { return e + 1; }`
    // through the same compiler, which is rejected at an EARLIER stage
    // (E3100 "undefined identifier 'e'", from `kali_types::resolve`) before
    // ever reaching the repr-inference/codegen layers this review fix
    // touches. So try/catch support is a pre-existing, orthogonal gap, not
    // something this fix can (or should) close. The achievable, honest
    // assertion here is that the module-const-shadowed catch program is
    // REJECTED rather than silently miscompiled to a wrong answer.
    run_js_expect_failure(
        "const K = 2.5;\nfunction f() {\n  try {\n    throw 1;\n  } catch (K) {\n    return K + 1;\n  }\n}\nconsole.log(f());\n",
    );
}

#[test]
fn module_let_read_from_function_is_rejected() {
    let combined = run_js_expect_failure(
        "let counter = 0;\nfunction f() { return counter + 1; }\nconsole.log(f());\n",
    );
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}

#[test]
fn impure_module_const_read_from_function_is_rejected() {
    let combined = run_js_expect_failure(
        "const t = Math.sqrt(2);\nfunction f() { return t; }\nconsole.log(f());\n",
    );
    assert!(combined.contains("5506"), "expected E5506, got: {combined}");
}
