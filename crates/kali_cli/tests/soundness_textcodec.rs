// Stage P5 — String() coercion + TextEncoder/TextDecoder soundness pins.
use std::fs;
use std::process::{Command, Output};
use tempfile::tempdir;

fn kali_bin() -> String {
    env!("CARGO_BIN_EXE_kali").to_string()
}

fn run(source: &str) -> Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// Compile+run, assert success, return trimmed stdout.
fn run_ok(source: &str) -> String {
    let out = run(source);
    assert!(
        out.status.success(),
        "expected success\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Compile+run, assert fail-closed, return stderr.
fn run_e5506(source: &str) -> String {
    let out = run(source);
    assert!(
        !out.status.success(),
        "expected fail-closed E5506\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("E5506"), "expected E5506, got: {stderr}");
    stderr
}

#[test]
fn string_of_i64_renders_decimal() {
    assert_eq!(run_ok("console.log(String(40n + 2n));"), "42");
}

#[test]
fn string_of_negative_i64_renders_sign() {
    assert_eq!(run_ok("console.log(String(0n - 7n));"), "-7");
}

#[test]
fn string_of_float_renders() {
    assert_eq!(run_ok("console.log(String(3.5));"), "3.5");
}

#[test]
fn string_of_boolean_renders_word() {
    assert_eq!(run_ok("console.log(String(1n === 1n));"), "true");
}

#[test]
fn string_of_string_is_identity() {
    assert_eq!(run_ok("console.log(String('hi'));"), "hi");
}

#[test]
fn string_of_object_fails_closed() {
    run_e5506("const o = { a: 1n }; console.log(String(o));");
}

#[test]
fn string_zero_arg_fails_closed() {
    run_e5506("console.log(String());");
}

#[test]
fn string_multi_arg_fails_closed() {
    run_e5506("console.log(String(1n, 2n));");
}

#[test]
fn string_of_function_ref_fails_closed() {
    run_e5506("function foo(){ return 1n; } console.log(String(foo));");
}

#[test]
fn string_of_arrow_fails_closed() {
    run_e5506("console.log(String(() => 1n));");
}

// --- encode provenance (Task 3) ---

#[test]
fn digest_consumes_bound_encode_bytes() {
    // digest over a bound encode result must still succeed (migrated consumer).
    let out = run_ok(
        "const e = new TextEncoder(); const b = e.encode('hi'); \
         const h = crypto.subtle.digest('SHA-256', b); console.log('ok');",
    );
    assert_eq!(out, "ok");
}

#[test]
fn encode_result_cannot_print() {
    // Was: silent `hi` (Repr::String hazard). Now: fail closed.
    run_e5506("const b = new TextEncoder().encode('hi'); console.log(b);");
}

#[test]
fn encode_bound_result_cannot_print() {
    run_e5506("const e = new TextEncoder(); const b = e.encode('hi'); console.log(b);");
}

#[test]
fn encode_result_cannot_return() {
    run_e5506(
        "function f() { const b = new TextEncoder().encode('hi'); return b; } console.log(f());",
    );
}

#[test]
fn encode_result_cannot_concat() {
    run_e5506("const b = new TextEncoder().encode('hi'); console.log('' + b);");
}

#[test]
fn encode_result_cannot_length() {
    run_e5506("const b = new TextEncoder().encode('hi'); console.log(b.length);");
}

#[test]
fn encode_non_string_arg_fails_closed() {
    run_e5506("const b = new TextEncoder().encode(42n); console.log('x');");
}

#[test]
fn encode_inline_unbound_bytelength_fails_closed() {
    run_e5506("console.log(new TextEncoder().encode('hi').byteLength);");
}

#[test]
fn encode_inline_unbound_length_fails_closed() {
    run_e5506("console.log(new TextEncoder().encode('hi').length);");
}

// --- decode roundtrip (Task 4) ---

#[test]
fn encode_decode_roundtrip_ascii() {
    assert_eq!(
        run_ok(
            "const e=new TextEncoder(); const d=new TextDecoder(); \
                const b=e.encode('hi'); console.log(d.decode(b));"
        ),
        "hi"
    );
}

#[test]
fn encode_decode_roundtrip_non_ascii() {
    assert_eq!(
        run_ok(
            "const e=new TextEncoder(); const d=new TextDecoder(); \
                const b=e.encode('héllo'); console.log(d.decode(b));"
        ),
        "héllo"
    );
}

#[test]
fn decode_result_is_a_real_string() {
    // decode output is a normal string: CONTENT comparison + concat work.
    //
    // The brief's literal expectation was `"true"`; kali renders a RUNTIME
    // comparison result as `1`/`0`, not `true`/`false` (pre-existing and
    // unrelated to this lane — `let x='ab'; console.log(x === 'ab')` prints `1`
    // on the parent commit too; only STATICALLY FOLDED comparisons render the
    // JS word). Asserting `1`/`0` pins the same property the brief wanted —
    // that the decode result takes the `__streq` content-equality lane instead
    // of failing closed or comparing raw handles — without smuggling an
    // unrelated console-rendering change into this task.
    assert_eq!(
        run_ok(
            "const e=new TextEncoder(); const d=new TextDecoder(); \
                const b=e.encode('42'); console.log(d.decode(b) === '42');"
        ),
        "1"
    );
    assert_eq!(
        run_ok(
            "const e=new TextEncoder(); const d=new TextDecoder(); \
                const b=e.encode('42'); console.log(d.decode(b) === '43');"
        ),
        "0"
    );
    assert_eq!(
        run_ok(
            "const e=new TextEncoder(); const d=new TextDecoder(); \
                const b=e.encode('42'); console.log('v=' + d.decode(b));"
        ),
        "v=42"
    );
}

#[test]
fn decode_of_string_literal_fails_closed() {
    run_e5506("const d = new TextDecoder(); console.log(d.decode('hi'));");
}

#[test]
fn decode_of_i64_fails_closed() {
    run_e5506("const d = new TextDecoder(); console.log(d.decode(42n));");
}

#[test]
fn decode_marker_cannot_print() {
    run_e5506("const d = new TextDecoder(); console.log(d);");
}

#[test]
fn decode_inline_unbound_roundtrip() {
    // Fully inline (neither the decoder nor the byte buffer is bound): the
    // hoisted-`new` wrapper passes through to the decode arm instead of the
    // drop-and-push-`0` aggregate fallback.
    assert_eq!(
        run_ok("console.log(new TextDecoder().decode(new TextEncoder().encode('hi')));"),
        "hi"
    );
}

#[test]
fn decode_of_unproven_identifier_fails_closed() {
    // A same-shaped i64 that is NOT byte-provenance must not be relabelled as a
    // string handle (that is the miscompile the provenance gate exists for).
    run_e5506("const d = new TextDecoder(); const b = 42n; console.log(d.decode(b));");
}

#[test]
fn decode_multi_arg_fails_closed() {
    run_e5506(
        "const e = new TextEncoder(); const d = new TextDecoder(); const b = e.encode('hi'); \
         console.log(d.decode(b, b));",
    );
}

#[test]
fn decode_zero_arg_fails_closed() {
    run_e5506("const d = new TextDecoder(); console.log(d.decode());");
}

#[test]
fn decode_result_length_fails_closed() {
    // Structural static-fold hazard (the Task 3 lesson): a `Call` base is
    // invisible to every name-keyed lane, so `render_length` would have rendered
    // the call node's CHILD COUNT as the length. The decoded bytes have no ASCII
    // proof, so `.length` fails closed rather than reporting a byte count.
    run_e5506(
        "const e = new TextEncoder(); const d = new TextDecoder(); const b = e.encode('héllo'); \
         console.log(d.decode(b).length);",
    );
}

#[test]
fn decode_bound_result_length_fails_closed() {
    // The BOUND twin: `const s = d.decode(b); s.length` would have reported the
    // handle's byte count (6) where node reports the character count (5). The
    // decode repr seed is marked NON-ASCII, so the shared ASCII gate rejects it.
    run_e5506(
        "const e = new TextEncoder(); const d = new TextDecoder(); const b = e.encode('héllo'); \
         const s = d.decode(b); console.log(s.length);",
    );
}

#[test]
fn decode_bound_result_prints_and_compares() {
    // A bound decode result is a first-class runtime string binding.
    assert_eq!(
        run_ok(
            "const e = new TextEncoder(); const d = new TextDecoder(); \
             const b = e.encode('héllo'); const s = d.decode(b); console.log(s);"
        ),
        "héllo"
    );
}

#[test]
fn decode_marker_cannot_escape_by_return() {
    run_e5506("function f() { const d = new TextDecoder(); return d; } console.log(f());");
}

// --- Stage P5 Task 4 review fixes ---------------------------------------------
//
// C-1: `TextDecoder` constructor arguments are SEMANTIC (encoding label,
// `{fatal}` options) and only the default utf-8 / non-fatal decoder is
// implemented. Before the fix the ctor filter matched on callee TEXT only, so
// `new TextDecoder('latin1').decode(b)` silently decoded as UTF-8 (kali printed
// `héllo` where node prints `hÃ©llo`).

#[test]
fn decode_bound_ctor_label_arg_fails_closed() {
    run_e5506(
        "const e = new TextEncoder(); const b = e.encode('héllo'); \
         const d = new TextDecoder('latin1'); console.log(d.decode(b));",
    );
}

#[test]
fn decode_bound_ctor_options_arg_fails_closed() {
    run_e5506(
        "const e = new TextEncoder(); const b = e.encode('hi'); \
         const d = new TextDecoder({ fatal: true }); console.log(d.decode(b));",
    );
}

#[test]
fn decode_inline_ctor_label_arg_fails_closed() {
    run_e5506("console.log(new TextDecoder('utf-16le').decode(new TextEncoder().encode('hi')));");
}

#[test]
fn decode_ctor_label_arg_fails_closed_even_unused() {
    // The construction itself is unsupported, so it is denied at the declarator
    // rather than left on the undefined-callee lane (which pushes a silent 0).
    run_e5506("const d = new TextDecoder('utf-8'); console.log('unused');");
}

// C-2: the INLINE recognizers had no shadow guard, so a user-defined
// `TextEncoder`/`TextDecoder` was hijacked into the intrinsic (kali printed the
// intrinsic result where node runs the user function).

#[test]
fn inline_decode_does_not_hijack_user_text_decoder() {
    run_e5506(
        "function TextDecoder() { return { decode: function (x) { return 'USER'; } }; } \
         const e = new TextEncoder(); const b = e.encode('hi'); \
         console.log(new TextDecoder().decode(b));",
    );
}

#[test]
fn inline_encode_does_not_hijack_user_text_encoder() {
    run_e5506(
        "function TextEncoder() { return { encode: function (x) { return 'USER'; } }; } \
         console.log(new TextEncoder().encode('hi'));",
    );
}

/// The legitimate zero-argument forms must keep working after the C-1/C-2 gates.
#[test]
fn zero_arg_decoder_forms_still_roundtrip() {
    assert_eq!(
        run_ok(
            "const e = new TextEncoder(); const d = new TextDecoder(); \
             console.log(d.decode(e.encode('héllo')));"
        ),
        "héllo"
    );
    assert_eq!(
        run_ok("console.log(new TextDecoder().decode(new TextEncoder().encode('hi')));"),
        "hi"
    );
}
