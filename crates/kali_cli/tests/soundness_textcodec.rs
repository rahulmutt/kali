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

/// Compile+run, assert fail-closed with EITHER diagnostic code — used where the
/// value carries a SEEDED `Repr::String` (T-new-F) so a numeric-sink rejection
/// surfaces as E3200 (the pre-existing "runtime string in a numeric position"
/// type-mismatch, the accurate match for node's `TypeError`) rather than E5506.
/// Both are fail-closed (non-zero exit); the point is that a raw handle is never
/// materialized as a number.
fn run_fail_closed(source: &str) -> String {
    let out = run(source);
    assert!(
        !out.status.success(),
        "expected fail-closed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("E3200") || stderr.contains("E5506"),
        "expected E3200/E5506 fail-closed, got: {stderr}"
    );
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

// C-3: construction-position allowlist. `new TextEncoder()` / `new TextDecoder()`
// are lowered ONLY as (a) a `const` declarator initializer or (b) an immediate
// `.encode`/`.decode` receiver. Every other construction position previously fell
// to the undefined-callee zero placeholder: a `let`/`var` codec binding never
// became a marker, so `d.decode(b)` silently evaluated to `0` (node prints the
// decoded string) with only an `E3100` WARNING and exit 0.

#[test]
fn let_bound_decoder_fails_closed() {
    run_e5506("let d = new TextDecoder(); console.log(d.decode(new TextEncoder().encode('hi')));");
}

#[test]
fn let_bound_decoder_with_ctor_arg_fails_closed() {
    run_e5506(
        "let d = new TextDecoder('latin1'); \
         console.log(d.decode(new TextEncoder().encode('hi')));",
    );
}

#[test]
fn let_bound_encoder_byte_length_fails_closed() {
    run_e5506("let e = new TextEncoder(); console.log(e.encode('hi').byteLength);");
}

#[test]
fn let_bound_encoder_fails_closed() {
    run_e5506("let e = new TextEncoder(); console.log(e.encode('hi'));");
}

#[test]
fn var_bound_encoder_fails_closed() {
    run_e5506("var e = new TextEncoder(); console.log(e.encode('hi'));");
}

#[test]
fn bare_encoder_construction_fails_closed() {
    run_e5506("console.log(new TextEncoder());");
}

#[test]
fn bare_decoder_construction_fails_closed() {
    run_e5506("console.log(new TextDecoder());");
}

#[test]
fn assigned_encoder_construction_fails_closed() {
    run_e5506("let e; e = new TextEncoder(); console.log(e.encode('hi'));");
}

#[test]
fn returned_encoder_construction_fails_closed() {
    run_e5506("function f(){ return new TextEncoder(); } console.log(f());");
}

// C-4: PRODUCE-side escape choke for the raw byte handle. The identifier choke
// only guards BOUND handles, so an inline, unbound `encode(...)` in a value
// position escaped and printed the DECODED string (`hi`) where node prints
// `Uint8Array(2) [ 104, 105 ]`.

#[test]
fn inline_unbound_encode_console_log_fails_closed() {
    run_e5506("console.log(new TextEncoder().encode('hi'));");
}

#[test]
fn bound_receiver_inline_encode_console_log_fails_closed() {
    run_e5506("const e = new TextEncoder(); console.log(e.encode('hi'));");
}

#[test]
fn inline_encode_string_concat_fails_closed() {
    run_e5506("const e = new TextEncoder(); console.log('' + e.encode('hi'));");
}

#[test]
fn nested_encode_of_encode_fails_closed() {
    run_e5506("const e = new TextEncoder(); const b = e.encode(e.encode('hi')); console.log('x');");
}

/// The three admitted producer positions must keep working after the C-4 gate.
#[test]
fn admitted_encode_producer_positions_still_work() {
    // (a) `const` declarator binding, then an allowlisted consumer.
    assert_eq!(
        run_ok("const e = new TextEncoder(); const b = e.encode('hi'); console.log(b.byteLength);"),
        "2"
    );
    // (b) inline `TextDecoder().decode` operand.
    assert_eq!(
        run_ok("console.log(new TextDecoder().decode(new TextEncoder().encode('hi')));"),
        "hi"
    );
    // (c) inline `crypto.subtle.digest` operand.
    assert_eq!(
        run_ok(
            "const h = crypto.subtle.digest('SHA-256', new TextEncoder().encode('hi')); \
             console.log(h.byteLength);"
        ),
        "32"
    );
}

// --- Stage P5 T-new-B: `encode` admits a bare `String(x)` result -------------
//
// The acceptance fixture (`browser_bundle_web_baseline_source`) spells
// `encoder.encode(String(left + right))` — a BARE `String()` call in argument
// position. The gate proved string-ness with `is_string_valued`, which had no
// arm for the Task-1 coercion call, so the whole fixture failed closed. The fix
// is an `is_string_valued` arm keyed on the SAME recognizer the coercion arm
// dispatches with, so oracle and emission agree by construction: a `String()`
// form Task 1 DENIES (0-arg / multi-arg / aggregate / function-valued / shadowed)
// is not admitted here either.

#[test]
fn encode_of_bare_string_call_i64() {
    assert_eq!(
        run_ok(
            "const e = new TextEncoder(); const b = e.encode(String(42n)); \
             console.log(b.byteLength);"
        ),
        "2"
    );
}

#[test]
fn encode_of_bare_string_call_runtime_i64() {
    // A RUNTIME i64 (not a const-foldable literal), so the coercion ladder — not
    // the static fold — produces the string handle the encode gate consumes.
    assert_eq!(
        run_ok(
            "function f(x) { return x + 1n; } const v = f(41n); \
             const e = new TextEncoder(); const b = e.encode(String(v)); \
             console.log(b.byteLength);"
        ),
        "2"
    );
}

#[test]
fn encode_of_bare_string_call_concat() {
    // The acceptance fixture's exact shape: `encode(String(left + right))` with
    // two bound (parameter) operands, plus a decode roundtrip recovering the
    // same text. node: `2` then `42`.
    assert_eq!(
        run_ok(
            "function smoke(left, right) {\n\
               const e = new TextEncoder();\n\
               const d = new TextDecoder();\n\
               const b = e.encode(String(left + right));\n\
               console.log(b.byteLength);\n\
               console.log(d.decode(b));\n\
               return 0n;\n\
             }\n\
             smoke(40n, 2n);"
        ),
        "2\n42"
    );
}

#[test]
fn encode_decode_roundtrip_through_string_call() {
    assert_eq!(
        run_ok(
            "function f(x) { return x + 1n; } const v = f(41n); \
             const e = new TextEncoder(); const d = new TextDecoder(); \
             console.log(d.decode(e.encode(String(v))));"
        ),
        "42"
    );
}

#[test]
fn encode_of_bare_string_call_non_ascii_byte_length() {
    // Byte length (6) differs from the character count (5), so a
    // character-count bug cannot pass by coincidence.
    assert_eq!(
        run_ok(
            "function id(s) { return s; } const t = id('héllo'); \
             const e = new TextEncoder(); const b = e.encode(String(t)); \
             console.log(b.byteLength);"
        ),
        "6"
    );
}

#[test]
fn encode_of_bare_string_call_non_ascii_roundtrips() {
    assert_eq!(
        run_ok(
            "function id(s) { return s; } const t = id('héllo'); \
             const e = new TextEncoder(); const d = new TextDecoder(); \
             console.log(d.decode(e.encode(String(t))));"
        ),
        "héllo"
    );
}

// Fail-closed pins that must NOT regress now that the same lane is wider.

#[test]
fn encode_of_zero_arg_string_call_fails_closed() {
    run_e5506("const e = new TextEncoder(); const b = e.encode(String()); console.log('x');");
}

#[test]
fn encode_of_multi_arg_string_call_fails_closed() {
    run_e5506("const e = new TextEncoder(); const b = e.encode(String(1n, 2n)); console.log('x');");
}

#[test]
fn encode_of_function_valued_string_call_fails_closed() {
    run_e5506(
        "function foo() { return 1n; } const e = new TextEncoder(); \
         const b = e.encode(String(foo)); console.log('x');",
    );
}

#[test]
fn encode_of_arrow_valued_string_call_fails_closed() {
    run_e5506(
        "const e = new TextEncoder(); const b = e.encode(String(() => 1n)); console.log('x');",
    );
}

#[test]
fn encode_of_object_valued_string_call_fails_closed() {
    run_e5506(
        "const o = { a: 1n }; const e = new TextEncoder(); \
         const b = e.encode(String(o)); console.log('x');",
    );
}

#[test]
fn encode_of_shadowed_string_call_fails_closed() {
    // A user-defined `String` keeps its own lane: the intrinsic recognizer is
    // unshadowed-only, so this is NOT admitted as a proven string.
    run_e5506(
        "function String(x) { return 1n; } const e = new TextEncoder(); \
         const b = e.encode(String(1n)); console.log('x');",
    );
}

// The Step-5 remainder: everything outside the widened set must still fail
// closed, not fall through to a silent `0` or a divergent value.

#[test]
fn encode_remainder_still_denies() {
    run_e5506("const e = new TextEncoder(); const b = e.encode(42n); console.log('x');");
    run_e5506(
        "const o = { a: 1n }; const e = new TextEncoder(); const b = e.encode(o); \
         console.log('x');",
    );
    run_e5506(
        "const e = new TextEncoder(); const b = e.encode('hi'); const c = e.encode(b); \
         console.log('x');",
    );
    run_e5506("const e = new TextEncoder(); const b = e.encode(e.encode('hi')); console.log('x');");
}

// Consumers of the same `is_string_valued` proof that the widened arm also
// makes correct (they silently miscompiled before: a raw tagged handle rendered
// as an integer, a call node's CHILD COUNT rendered as `.length`, and a handle
// compared numerically instead of by content).

#[test]
fn bare_string_call_length_is_the_string_length() {
    assert_eq!(
        run_ok(
            "function f(x) { return x + 1n; } const v = f(3999n); console.log(String(v).length);"
        ),
        "4"
    );
}

#[test]
fn bare_string_call_compares_by_content() {
    assert_eq!(
        run_ok(
            "function f(x) { return x + 1n; } const v = f(41n); console.log(String(v) === '42');"
        ),
        "1"
    );
    assert_eq!(
        run_ok(
            "function f(x) { return x + 1n; } const v = f(41n); console.log(String(v) === '43');"
        ),
        "0"
    );
}

// --- Stage P5 T-new-B stage review: the positive argument proof --------------
//
// C-1: the T-new-B recognizer admitted `String(<anything not syntactically an
// aggregate>)`, which is NOT a proof that `emit_as_string` renders it — every
// unproven shape fell into the terminal `int_to_string` and printed a tagged
// handle (or an unmaterialized aggregate's placeholder `0`) as digits. Each
// case below was measured divergent-vs-node on the parent build (8cd1f3c83) and
// now fails closed. E5506 is the pin: fail-closed is always allowed, a silent
// wrong number never is.

/// Helper: assert the program fails closed, whatever the exact E5506 site.
fn assert_fails_closed(source: &str) {
    run_e5506(source);
}

#[test]
fn encode_of_string_call_on_object_field_fails_closed() {
    // parent: byteLength 20 (node: 5)
    assert_fails_closed(
        "const e = new TextEncoder(); const o = { s: 'hello' }; \
         const b = e.encode(String(o.s)); console.log(b.byteLength);",
    );
}

#[test]
fn decode_of_string_call_on_object_field_fails_closed() {
    // parent: printed -9223354444668731387 (node: hello)
    assert_fails_closed(
        "const e = new TextEncoder(); const d = new TextDecoder(); \
         const o = { s: 'hello' }; const b = e.encode(String(o.s)); \
         console.log(d.decode(b));",
    );
}

#[test]
fn encode_of_string_call_on_array_element_fails_closed() {
    // parent: byteLength 20 (node: 5)
    assert_fails_closed(
        "const e = new TextEncoder(); const a = ['hello']; \
         const b = e.encode(String(a[0])); console.log(b.byteLength);",
    );
}

#[test]
fn encode_of_string_call_on_object_returning_call_fails_closed() {
    // parent: byteLength 1 (node: 15) — the syntactic aggregate denylist is
    // defeated by a call boundary.
    assert_fails_closed(
        "function h() { return { a: 1n }; } const e = new TextEncoder(); \
         const b = e.encode(String(h())); console.log(b.byteLength);",
    );
}

#[test]
fn encode_of_string_call_on_array_returning_call_fails_closed() {
    // parent: byteLength 1 (node: 3)
    assert_fails_closed(
        "function h() { return [1n, 2n]; } const e = new TextEncoder(); \
         const b = e.encode(String(h())); console.log(b.byteLength);",
    );
}

#[test]
fn encode_of_string_call_on_global_this_fails_closed() {
    // parent: byteLength 1 (node: 15)
    assert_fails_closed(
        "const e = new TextEncoder(); const b = e.encode(String(globalThis)); \
         console.log(b.byteLength);",
    );
}

#[test]
fn encode_of_string_call_on_undefined_fails_closed() {
    // parent: byteLength 5 (node: 9) — the T-new-B report claimed this was
    // "unreachable from this task's widening"; it was reachable and divergent.
    assert_fails_closed(
        "const e = new TextEncoder(); const b = e.encode(String(undefined)); \
         console.log(b.byteLength);",
    );
}

#[test]
fn encode_of_string_call_on_null_fails_closed() {
    // parent: byteLength 1 (node: 4)
    assert_fails_closed(
        "const e = new TextEncoder(); const b = e.encode(String(null)); \
         console.log(b.byteLength);",
    );
}

#[test]
fn encode_of_array_wrapped_string_call_fails_closed() {
    // C-2. parent: byteLength 0 (node: 2). `unwrap_transparent` tunnels a
    // single-element ARRAY literal, so `[String(v)]` was proven a string and the
    // array literal's placeholder `0` was encoded.
    assert_fails_closed(
        "function f(x) { return x + 1n; } const v = f(41n); \
         const e = new TextEncoder(); const b = e.encode([String(v)]); \
         console.log(b.byteLength);",
    );
}

#[test]
fn string_call_length_on_unproven_receiver_fails_closed() {
    // I-1. parent: 20 (node: 5) via the runtime handle byte count; with the
    // static-fold bail removed it would render the CALL node's child count (2).
    assert_fails_closed("const o = { s: 'hello' }; console.log(String(o.s).length);");
    assert_fails_closed("const a = ['hello']; console.log(String(a[0]).length);");
}

#[test]
fn string_call_of_unproven_receiver_fails_closed_in_every_position() {
    // Siblings of the same class found while probing: the coercion itself, not
    // just its `encode`/`.length` consumers, must fail closed.
    assert_fails_closed("const o = { s: 'hello' }; console.log(String(o.s));");
    assert_fails_closed("const o = { s: 'hello' }; console.log(String(o.s) + '!');");
    assert_fails_closed("console.log(String(new Error('m')));");
    assert_fails_closed(
        "function h() { return { a: 1n }; } const w = h(); console.log(String(w));",
    );
}

// No-over-deny pins: the shapes the proof must keep admitting, each verified
// against node v26.5.0.

#[test]
fn string_call_proof_admits_scalars_and_proven_strings() {
    let encode = "const e = new TextEncoder(); const b = e.encode(";
    // String(42n) -> "42" (2 bytes)
    assert_eq!(
        run_ok(&format!("{encode}String(42n)); console.log(b.byteLength);")),
        "2"
    );
    // String(true) -> "true" (4 bytes)
    assert_eq!(
        run_ok(&format!(
            "{encode}String(true)); console.log(b.byteLength);"
        )),
        "4"
    );
    // String(1.5) -> "1.5" (3 bytes)
    assert_eq!(
        run_ok(&format!("{encode}String(1.5)); console.log(b.byteLength);")),
        "3"
    );
    // repr-seeded string binding
    assert_eq!(
        run_ok(
            "function id(s) { return s; } const t = id('hello'); \
             const e = new TextEncoder(); const d = new TextDecoder(); \
             const b = e.encode(String(t)); console.log(b.byteLength); \
             console.log(d.decode(b));"
        ),
        "5\nhello"
    );
    // non-ASCII string binding: 6 bytes, roundtrips
    assert_eq!(
        run_ok(
            "function id(s) { return s; } const t = id('h\u{e9}llo'); \
             const e = new TextEncoder(); const d = new TextDecoder(); \
             const b = e.encode(String(t)); console.log(b.byteLength); \
             console.log(d.decode(b));"
        ),
        "6\nh\u{e9}llo"
    );
    // runtime i64 through a fold-lane const binding
    assert_eq!(
        run_ok(
            "function f(x) { return x + 1n; } const v = f(41n); \
             const e = new TextEncoder(); const b = e.encode(String(v)); \
             console.log(b.byteLength);"
        ),
        "2"
    );
    // comparison operands stay renderable as booleans
    assert_eq!(run_ok("console.log(String(1n === 1n));"), "true");
}

#[test]
fn string_call_proof_admits_the_acceptance_fixture_shape() {
    // `encode(String(left + right))` with bigint PARAMS — the shape T-new-B
    // exists for. node: 2 then 42.
    assert_eq!(
        run_ok(
            "function smoke(left, right) {\n\
             const e = new TextEncoder();\n\
             const d = new TextDecoder();\n\
             const b = e.encode(String(left + right));\n\
             if (d.decode(b) !== String(left + right)) { throw new Error('bad'); }\n\
             console.log(b.byteLength);\n\
             return left - left;\n\
             }\n\
             console.log(smoke(40n, 2n));"
        ),
        "2\n0"
    );
}

#[test]
fn string_call_proof_admits_the_scalar_shapes_the_parent_build_rendered() {
    // Shapes the positive proof must keep — each verified against node v26.5.0
    // and against the parent build (8cd1f3c83), which rendered them correctly.
    // Without these arms the proof would be a NARROWING, not just a soundness
    // fix. Ordered: fold-lane object field, fold-lane array element, boolean
    // field, materialized object field, runtime array element, static
    // `.length`, ternary, USP `get()`, float call.
    assert_eq!(
        run_ok("const o = { n: 42n }; console.log(String(o.n));"),
        "42"
    );
    assert_eq!(
        run_ok("const o = { n: 1.5 }; console.log(String(o.n));"),
        "1.5"
    );
    assert_eq!(
        run_ok("const a = [7n, 8n]; console.log(String(a[0]));"),
        "7"
    );
    assert_eq!(
        run_ok("const o = { b: true }; console.log(String(o.b));"),
        "true"
    );
    assert_eq!(
        run_ok("function g() { const o = { n: 1n }; o.n = 42n; return String(o.n); } console.log(g());"),
        "42"
    );
    assert_eq!(
        run_ok("const a = new Array(2); a[0] = 7n; console.log(String(a[0]));"),
        "7"
    );
    assert_eq!(
        run_ok("const a = [1n, 2n]; console.log(String(a.length));"),
        "2"
    );
    assert_eq!(
        run_ok("const c = 1n; console.log(String(c > 0n ? 1n : 2n));"),
        "1"
    );
    assert_eq!(
        run_ok("const q = new URLSearchParams('a=1'); console.log(String(q.get('a')));"),
        "1"
    );
    assert_eq!(
        run_ok("console.log(String(Math.sqrt(2)));"),
        "1.4142135623730951"
    );
}

#[test]
fn string_call_of_string_valued_object_field_fails_closed() {
    // The fold lane substitutes a STRING field's literal, but `emit_as_string`
    // keys its string arm on the ORIGINAL receiver — so the handle would go
    // through `int_to_string`. Measured on the parent build:
    // `String(o.s)` → -9223354444668731387, `String(a[0])` → -9223354444668731390.
    assert_fails_closed("const o = { s: 'hello' }; console.log(String(o.s));");
    assert_fails_closed("const a = ['hi']; console.log(String(a[0]));");
}

// ---------------------------------------------------------------------------
// Stage P5 T-new-B, round-2 review. Two arms of the argument proof rested on
// `Repr::I64`, which is the UNRECORDED DEFAULT rather than evidence — the same
// "default is not a proof" fallacy the round-1 fix rejected elsewhere.
// ---------------------------------------------------------------------------

#[test]
fn string_call_of_materialized_string_object_field_fails_closed() {
    // REVIEW C-5. The MATERIALIZED spelling (the WRITE is what takes `o` off
    // the fold lane and onto the shape-table lane) — deliberately kept
    // ALONGSIDE the fold-lane pin above rather than replacing it: that
    // `const`-shaped probe never reaches the shape-table arm at all, and its
    // green result MASKED this hole (the bound-vs-unbound masking hazard).
    //
    // Measured on b73a45c6d: `encode(String(o.s)).byteLength` → 20 (node 5),
    // `decode` → -9223354440373764091 (node hello), `String(o.s).length` → 20
    // (node 5), console.log inside a function → -9223354440373764091.
    assert_fails_closed(
        "const e = new TextEncoder(); const o = { s: 'x' }; o.s = 'hello'; \
         const b = e.encode(String(o.s)); console.log(b.byteLength);",
    );
    assert_fails_closed(
        "const e = new TextEncoder(); const d = new TextDecoder(); \
         const o = { s: 'x' }; o.s = 'hello'; const b = e.encode(String(o.s)); \
         console.log(d.decode(b));",
    );
    assert_fails_closed("const o = { s: 'x' }; o.s = 'hello'; console.log(String(o.s).length);");
    assert_fails_closed(
        "function g(o) { console.log(String(o.s)); } \
         const o = { s: 'x' }; o.s = 'hello'; g(o);",
    );
}

#[test]
fn string_call_of_binding_initialized_by_a_string_result_call_now_renders() {
    // REVIEW C-6 (T-new-B) closed this fail-closed because `const s = g(1n)` kept
    // the DEFAULT `Repr::I64` (F-newB-1), so `String(s)` read the default as a
    // number. T-new-F now SEEDS `s` `Repr::String` (its return is monomorphically
    // a String() result), so `String(s)` is a proven-string IDENTITY coercion and
    // renders CORRECTLY — no longer the I64-default fallacy, a real positive
    // proof. All three forms verified against node (`1`).
    assert_eq!(
        run_ok(
            "const e = new TextEncoder(); function g(y) { return String(y); } \
             const s = g(1n); const b = e.encode(String(s)); console.log(b.byteLength);"
        ),
        "1"
    );
    assert_eq!(
        run_ok("function g(y) { return String(y); } const s = g(1n); console.log(String(s));"),
        "1"
    );
    assert_eq!(
        run_ok("function g(y) { return String(y); } let s = g(1n); console.log(String(s));"),
        "1"
    );
}

#[test]
fn string_call_proof_reclaims_the_positively_numeric_shapes() {
    // REVIEW I-2. Three shapes the round-1 fix over-denied, reclaimed with
    // GENUINE positive proofs (not a default repr):
    //   * a call whose callee's return is proven numeric by `repr_infer`
    //     (`return_is_proven_numeric`: non-string axes AND every return is
    //     arithmetic over literals/scalar-proven params) — note the unproven
    //     twin `function g(y){ return String(y) }` is pinned fail-closed above,
    //     which is what makes this evidence rather than a default;
    //   * `Math.floor`/`trunc`/`ceil`, whose emit arm yields a plain integer
    //     (the allowlist was inconsistent: `Math.sqrt` was already admitted);
    //   * `typeof`, which yields a string — now proven in `is_string_valued`,
    //     keyed on the same two lanes `emit_unary` lowers.
    assert_eq!(
        run_ok("function f(x) { return x + 1n; } console.log(String(f(41n)));"),
        "42"
    );
    assert_eq!(run_ok("console.log(String(Math.floor(1.7)));"), "1");
    assert_eq!(run_ok("console.log(String(Math.trunc(1.7)));"), "1");
    assert_eq!(run_ok("console.log(String(typeof 1n));"), "bigint");
    assert_eq!(run_ok("console.log(String(typeof 'a'));"), "string");
    // The reclaimed call proof also feeds the encode lane.
    assert_eq!(
        run_ok(
            "const e = new TextEncoder(); function f(x) { return x + 1n; } \
             const b = e.encode(String(f(41n))); console.log(b.byteLength);"
        ),
        "2"
    );
    // `Date.now()` was ALREADY divergent on the parent build (`0` where node
    // renders a real timestamp), so it stays denied.
    assert_fails_closed("console.log(String(Date.now()));");
}

#[test]
fn string_call_of_a_proven_numeric_mutable_local_renders_the_number() {
    // ROUND 3 — this test previously pinned the OPPOSITE (that `let i = 0n;
    // i++` fails closed) and encoded an over-deny as intended behavior. The
    // round-2 C-6 close required a resolvable declarator initializer, but
    // codegen's `self.bindings` holds `const` FOLD-ALIASES only, so every
    // `let`/`var` fell through to "must be a parameter" and was denied. That
    // was a real stage-progress regression: the structuredClone/event fixture's
    // `let count = 0; count += 1; String(count)` stopped COMPILING.
    //
    // The close is now a positive proof instead of an over-deny —
    // `repr_infer`'s `numeric_bindings` allowlist (every write arithmetic over
    // numeric literals / the binding itself / scalar-inflow-proven params) —
    // so the genuinely numeric mutable local renders its number and the
    // handle-returning twin below stays fail-closed.
    assert_eq!(run_ok("let i = 0n; i++; console.log(String(i));"), "1");
    // node: 0 / 1 / 1. All three were E5506 on f5217e65a.
    assert_eq!(run_ok("let count = 0; console.log(String(count));"), "0");
    assert_eq!(
        run_ok("let count = 0; count += 1; console.log(String(count));"),
        "1"
    );
    assert_eq!(
        run_ok("let count = 0n; count = count + 1n; console.log(String(count));"),
        "1"
    );
    // `var` spelling, and the encode lane this task widened.
    assert_eq!(run_ok("var n = 7n; n *= 6n; console.log(String(n));"), "42");
    assert_eq!(
        run_ok(
            "const e = new TextEncoder(); let count = 0; count += 1; \
             const b = e.encode(String(count)); console.log(b.byteLength);"
        ),
        "1"
    );
    // And the round-trip through the decoder, so the admitted value is proven
    // to be the NUMBER's digits and not a raw handle rendered by coincidence.
    assert_eq!(
        run_ok(
            "const e = new TextEncoder(); const d = new TextDecoder(); \
             let count = 0; count += 41; \
             const b = e.encode(String(count)); console.log(d.decode(b));"
        ),
        "41"
    );
}

#[test]
fn string_result_binding_is_a_real_proof_not_the_default_repr() {
    // T-new-F does not re-open the "`Repr::I64` default is a number" fallacy: it
    // seeds `Repr::String` only from a POSITIVE monomorphic String()-result
    // proof. A binding a String()-returning callee monomorphically initializes IS
    // provably a string, so `String(s)` (identity) renders correctly in all three
    // declaration spellings + the encode lane (node: `1`).
    for kind in ["const", "let", "var"] {
        assert_eq!(
            run_ok(&format!(
                "function g(y) {{ return String(y); }} {kind} s = g(1n); \
                 console.log(String(s));"
            )),
            "1"
        );
    }
    assert_eq!(
        run_ok(
            "const e = new TextEncoder(); function g(y) { return String(y); } let s = g(1n); \
             const b = e.encode(String(s)); console.log(b.byteLength);"
        ),
        "1"
    );
    // But a binding WITHOUT a monomorphic proof keeps the default `Repr::I64` and
    // still fails CLOSED — the seed never reads the default as evidence:
    //
    // numeric at its declarator, LATER overwritten from the callee (a plain `0n`
    // write is not a String() result, so `x` is not monomorphic — this proof is
    // not flow-sensitive and must not pretend the dead `0n` away).
    assert_fails_closed(
        "function g(y) { return String(y); } let x = 0n; x = g(1n); console.log(String(x));",
    );
    // A declarator with no initializer holds `undefined` (no rendering for it).
    assert_fails_closed("let z; console.log(String(z));");
    // A `for..of` element is never proven a String() result.
    assert_fails_closed("const a = [1n, 2n]; for (const v of a) { console.log(String(v)); }");
    // `||=` is not the `=` the taint records, so `x` (plain `0n` declarator) is
    // not seeded and fails closed.
    assert_fails_closed(
        "function g(y) { return String(y); } let x = 0n; x ||= g(1n); console.log(String(x));",
    );
}

// --- Stage P5 T-new-D: the UNIFIED stale-provenance shadow guard ------------
// `text_encoder_locals` / `text_decoder_locals` / `bytes_locals` are name-keyed
// and flat and had no arm at either binding choke. Measured on parent
// e14c40004, both codec rows COMPILED AND RAN, printing `hi` (exit 0), where
// node v26.5.0 throws a TypeError (`enc.encode` / `dec.decode` is not a
// function on a string).

/// Assert a fail-closed compile whose diagnostic names BOTH E5506 and the lane.
fn assert_e5506_containing(source: &str, needle: &str) {
    let stderr = run_e5506(source);
    assert!(
        stderr.contains(needle),
        "expected '{needle}' in diagnostic, got: {stderr}"
    );
}

/// T-new-D, for-of choke (NEW), ENCODER marker: measured pre-fix `hi`, exit 0.
#[test]
fn text_encoder_marker_shadowed_by_for_of_binding_fails_closed() {
    assert_e5506_containing(
        "const enc = new TextEncoder(); const dec = new TextDecoder();\n\
         for (const enc of ['aa']) { console.log(dec.decode(enc.encode('hi'))); }\n",
        "for-of loop binding may not shadow a name bound to a TextEncoder",
    );
}

/// T-new-D, for-of choke (NEW), DECODER marker: measured pre-fix `hi`, exit 0.
#[test]
fn text_decoder_marker_shadowed_by_for_of_binding_fails_closed() {
    assert_e5506_containing(
        "const enc = new TextEncoder(); const dec = new TextDecoder();\n\
         const b = enc.encode('hi');\n\
         for (const dec of ['aa']) { console.log(dec.decode(b)); }\n",
        "for-of loop binding may not shadow a name bound to a TextDecoder",
    );
}

/// T-new-D, for-of choke, BYTE HANDLE: measured NOT hijacked pre-fix (the
/// string `.length` lane wins first), but the handle table is equally flat, so
/// the unified guard covers it too — a lane one sink away from divergence.
#[test]
fn bytes_handle_shadowed_by_for_of_binding_fails_closed() {
    assert_e5506_containing(
        "const enc = new TextEncoder(); const dec = new TextDecoder();\n\
         const b = enc.encode('hi');\n\
         for (const b of ['aa']) { console.log(dec.decode(b)); }\n",
        "for-of loop binding may not shadow a name bound to a TextEncoder().encode() byte handle",
    );
}

/// T-new-D, declarator choke (NEW), encoder marker.
#[test]
fn text_encoder_marker_redeclared_in_an_inner_block_fails_closed() {
    assert_e5506_containing(
        "const enc = new TextEncoder(); const dec = new TextDecoder();\n\
         { const enc = 5; console.log(enc); }\n\
         console.log(dec.decode(enc.encode('hi')));\n",
        "redeclaring a name bound to a TextEncoder",
    );
}

/// T-new-D, declarator choke (NEW), decoder marker.
#[test]
fn text_decoder_marker_redeclared_in_an_inner_block_fails_closed() {
    assert_e5506_containing(
        "const enc = new TextEncoder(); const dec = new TextDecoder();\n\
         const b = enc.encode('hi');\n\
         { const dec = 5; console.log(dec); }\n\
         console.log(dec.decode(b));\n",
        "redeclaring a name bound to a TextDecoder",
    );
}

/// T-new-D, declarator choke (NEW), byte handle.
#[test]
fn bytes_handle_redeclared_in_an_inner_block_fails_closed() {
    assert_e5506_containing(
        "const enc = new TextEncoder(); const dec = new TextDecoder();\n\
         const b = enc.encode('hi');\n\
         { const b = 5; console.log(b); }\n\
         console.log(dec.decode(b));\n",
        "redeclaring a name bound to a TextEncoder().encode() byte handle",
    );
}

/// T-new-D no-over-deny control: the roundtrip still works next to a for-of
/// binding whose name does not shadow any codec handle. node v26.5.0:
/// "2\n2\nhi\n2\n".
#[test]
fn for_of_binding_without_codec_shadow_is_unaffected() {
    assert_eq!(
        run_ok(
            "const enc = new TextEncoder(); const dec = new TextDecoder();\n\
             const b = enc.encode('hi');\n\
             for (const x of ['aa','bb']) { console.log(x.length); }\n\
             console.log(dec.decode(b));\n\
             console.log(b.byteLength);\n"
        ),
        "2\n2\nhi\n2"
    );
}

// --- Task 6: deliberate fail-closed boundary tripwires -----------------------
//
// Every test below pins a shape that CORRECTLY denies with E5506 today and must
// keep denying. A future change that accidentally opens one of these turns the
// tripwire red. Each was RUN on a freshly built binary and confirmed to exit
// non-zero with `E5506` in stderr before being pinned — none prints a value.
//
// DROPPED, recorded in the inventory instead (docs/superpowers/followups/
// stageD-triage.md §8.6 and the silent-miscompile register): the member-call
// form `globalThis.String(1n)` was expected to deny but instead prints `0`
// (exit 0, no warning) where node prints `1` — a SILENT MISCOMPILE, not a
// boundary, so pinning kali's `0` as "expected" would bake a wrong value into
// the suite. It is filed as P5-R-globalthis-string.

/// Zero-arg `String()` is not the single-argument coercion arm; denies.
#[test]
fn p5_boundary_string_zero_arg_denies() {
    run_e5506("console.log(String());");
}

/// Multi-arg `String(1n, 2n)` is not the single-argument coercion arm; denies.
#[test]
fn p5_boundary_string_multi_arg_denies() {
    run_e5506("console.log(String(1n, 2n));");
}

/// The function-valued argument hole Task 1 closed — arrow form.
#[test]
fn p5_boundary_string_of_arrow_function_denies() {
    run_e5506("console.log(String(() => 1n));");
}

/// The function-valued argument hole Task 1 closed — named-function form.
#[test]
fn p5_boundary_string_of_named_function_denies() {
    run_e5506("function foo() { return 1n; }\nconsole.log(String(foo));");
}

/// The escape choke: a bytes handle in a nested position (array literal element,
/// read back by index) may not escape as a value.
#[test]
fn p5_boundary_bytes_handle_in_array_element_denies() {
    run_e5506(
        "const b = new TextEncoder().encode('hi');\n\
         const a = [b];\n\
         console.log(a[0]);\n",
    );
}

/// The T-new-C/T4 ctor-arg boundary: ANY `new TextDecoder(<arg>)` denies — even
/// the explicit default label `'utf-8'` (conservative over-deny by design).
#[test]
fn p5_boundary_text_decoder_with_ctor_arg_denies() {
    run_e5506("const d = new TextDecoder('utf-8');\nconsole.log(1);");
}

/// `decode` on a non-bytes argument — string form.
#[test]
fn p5_boundary_decode_of_string_arg_denies() {
    run_e5506("const d = new TextDecoder();\nconsole.log(d.decode('hi'));");
}

/// `decode` on a non-bytes argument — i64 form.
#[test]
fn p5_boundary_decode_of_i64_arg_denies() {
    run_e5506("const d = new TextDecoder();\nconsole.log(d.decode(42n));");
}

/// T-new-D unified guard, ENCODER lane: a for-of binding shadowing a codec name
/// denies (kali otherwise RUNS a program node rejects with a TypeError).
#[test]
fn p5_r_for_of_shadow_of_encoder_name_denies() {
    run_e5506(
        "const e = new TextEncoder();\n\
         for (const e of ['aa']) { console.log(e.encode('x')); }\n",
    );
}

/// T-new-D unified guard, DECODER lane: for-of shadow of a decoder name denies.
#[test]
fn p5_r_for_of_shadow_of_decoder_name_denies() {
    run_e5506(
        "const d = new TextDecoder();\n\
         const b = new TextEncoder().encode('hi');\n\
         for (const d of ['aa']) { console.log(d.decode(b)); }\n",
    );
}

/// T-new-D unified guard, BYTES-HANDLE lane: for-of shadow of a bytes-handle
/// name denies (the flat handle table is one sink away from divergence).
#[test]
fn p5_r_for_of_shadow_of_bytes_handle_name_denies() {
    run_e5506(
        "const dec = new TextDecoder();\n\
         const b = new TextEncoder().encode('hi');\n\
         for (const b of ['aa']) { console.log(dec.decode(b)); }\n",
    );
}

/// T-new-D unified guard, declarator choke: a block redeclaration shadow of a
/// codec name denies.
#[test]
fn p5_r_block_redeclaration_shadow_of_codec_name_denies() {
    run_e5506(
        "const e = new TextEncoder();\n\
         { const e = 5; console.log(e); }\n\
         console.log(e.encode('x'));\n",
    );
}

// --- Stage P5 T-new-F: String()-result render provenance (F-newB-1) CLOSED ----
// `repr_infer` now SEEDS `Repr::String` for a value proven MONOMORPHICALLY a
// `String()` result (every write/return-path/composite-arm is a String()
// result), reusing the round-2 value-flow fixpoint. The seeded value carries its
// string repr, so a let/var/return/launder/ternary result renders CORRECTLY at
// every string sink (`+`, template, console, `===`, `.length`) BY CONSTRUCTION —
// no sink enumeration. Parent ee8e2571e/70a5a7660 fail-closed (E5506, F-newB-1);
// these rows FLIP to the node-correct output. A NON-monomorphic value
// (reassign-with-a-numeric-write, a param, a `&&`/`||`/`??`/sequence composite)
// stays UNSEEDED and fails CLOSED via the round-2 taint backstop (below). Each
// value below verified against node.

/// let-bound String() result reaching `+` renders correctly (node: `x1`).
#[test]
fn p5_string_result_let_bound_render_renders() {
    assert_eq!(run_ok("let s = String(1n); console.log('x' + s);"), "x1");
}

/// var-bound String() result reaching `+` (node: `x1`).
#[test]
fn p5_string_result_var_bound_render_renders() {
    assert_eq!(run_ok("var s = String(1n); console.log('x' + s);"), "x1");
}

/// function-return-bound String() result reaching `+` — provenance crosses the
/// function boundary via a String()-result-returning function (node: `x1`).
#[test]
fn p5_string_result_function_return_render_renders() {
    assert_eq!(
        run_ok("function g(y){ return String(y) } const s = g(1n); console.log('x' + s);"),
        "x1"
    );
}

/// function-return-bound String() result reaching a TEMPLATE LITERAL (the
/// template ladder shares the render lane) (node: `x1`).
#[test]
fn p5_string_result_template_literal_render_renders() {
    assert_eq!(
        run_ok("function g(y){ return String(y) } const s = g(1n); console.log(`x${s}`);"),
        "x1"
    );
}

/// direct `g(1n)` inline in `+` (the return-provenance call site itself, no
/// binding) renders via the seeded return repr (node: `x1`).
#[test]
fn p5_string_result_direct_call_render_renders() {
    assert_eq!(
        run_ok("function g(y){ return String(y) } console.log('x' + g(1n));"),
        "x1"
    );
}

/// LAUNDERING through a second binding (`let t = s`) — the seed propagates the
/// String repr through the copy (node: `x1`).
#[test]
fn p5_string_result_launder_through_second_binding_renders() {
    assert_eq!(
        run_ok("let s = String(1n); let t = s; console.log('x' + t);"),
        "x1"
    );
}

/// MULTI-argument `console.log('x', s)` renders the seeded string in each
/// argument slot (node: `x 1`, space-separated).
#[test]
fn p5_string_result_multi_arg_console_renders() {
    assert_eq!(run_ok("let s = String(1n); console.log('x', s);"), "x 1");
}

/// NO-OVER-DENY, single-argument console: the single-arg lane hands the host the
/// raw tagged handle, which the host decodes and prints as text — so
/// `console.log(s)` for a `String()`-result binding stays CORRECT (`1`, matching
/// node) and must NOT be tainted. This is the divergence's boundary: it is
/// confined to the wasm `int_to_string` ladder, not the host renderer.
#[test]
fn p5_string_result_single_arg_console_stays_correct() {
    assert_eq!(run_ok("let s = String(1n); console.log(s);"), "1");
}

/// bare-identifier REASSIGNMENT `let s = 0n; s = String(1n)` is NOT
/// monomorphically a String() result — its declarator write (`0n`) is a plain
/// numeric write, so T-new-F does NOT seed it `Repr::String` (this proof is not
/// flow-sensitive, so it cannot know the `0n` is dead). It stays fail-closed via
/// the round-2 taint backstop. node would render `x1`; kali conservatively fails
/// closed here — sound (no silent bits), never a miscompile.
#[test]
fn p5_string_result_reassignment_render_fails_closed() {
    run_e5506("let s = 0n; s = String(1n); console.log('x' + s);");
}

// --- no-over-deny: the must-stay-correct shapes ------------------------------

/// INLINE `String(1n)` as a `+` operand renders correctly (never tainted — a
/// proven string handle).
#[test]
fn p5_string_result_inline_plus_stays_correct() {
    assert_eq!(run_ok("console.log('x' + String(1n));"), "x1");
}

/// fold-aliased `const s = String(1n)` renders correctly (resolves to a proven
/// string handle; exempt from the render-taint deny by the `is_string_valued`
/// guard).
#[test]
fn p5_string_result_const_fold_alias_stays_correct() {
    assert_eq!(run_ok("const s = String(1n); console.log('x' + s);"), "x1");
}

/// Acceptance-path position 1: a String() result INLINE as the `encode`
/// argument, over genuine bigint params, must keep working (a real i64
/// `a + b`, NOT String()-result taint).
#[test]
fn p5_string_result_no_over_deny_encode_arg() {
    assert_eq!(
        run_ok(
            "function f(a,b){ const e=new TextEncoder(); \
             const enc=e.encode(String(a+b)); console.log(enc.byteLength); } f(1n,2n);"
        ),
        "1"
    );
}

/// Acceptance-path position 2: a String() result INLINE as a print argument
/// over a genuine bigint param.
#[test]
fn p5_string_result_no_over_deny_print_arg() {
    assert_eq!(
        run_ok("function f(a){ console.log(String(a)); } f(42n);"),
        "42"
    );
}

/// Acceptance-path position 3: a String() result INLINE in a `!==`
/// content-equality (`__streq`), the exact fixture shape.
#[test]
fn p5_string_result_no_over_deny_streq_compare() {
    assert_eq!(
        run_ok(
            "function f(a,b){ const e=new TextEncoder(); const d=new TextDecoder(); \
             const enc=e.encode(String(a+b)); \
             if (d.decode(enc) !== String(a+b)) { throw new Error('x'); } \
             console.log('ok'); } f(1n,2n);"
        ),
        "ok"
    );
}

/// A genuine bigint param subtraction (`left - left`, the fixture's numeric
/// return path) must NOT be tainted — proves the deny keys on String()-result
/// provenance, not on the `I64` default.
#[test]
fn p5_string_result_no_over_deny_genuine_i64_render() {
    assert_eq!(
        run_ok("function f(a){ console.log('n=' + (a - a)); } f(5n);"),
        "n=0"
    );
}

/// `String(42n)` byte length via encode (bound result consumed by digest/length
/// lanes) stays available.
#[test]
fn p5_string_result_no_over_deny_string_literal_encode_bytelength() {
    assert_eq!(
        run_ok("const b = new TextEncoder().encode(String(42n)); console.log(b.byteLength);"),
        "2"
    );
}

// --- T-new-F: the round-2 value-flow rows now RENDER (were fail-closed) -------
// The taint fixpoint that made each fail closed now feeds the MONOMORPHIC seed,
// so these render CORRECTLY (verified against node). The ARITHMETIC rows stay
// fail-closed (a string in a BigInt-numeric position — node throws TypeError).

/// Root A — RETURN-OF-LOCAL: `g` returns a String()-result LOCAL. The return is
/// monomorphically a String() result → seeded → `'x'+r` renders (node: `x1`).
#[test]
fn p5_string_result_return_of_local_renders() {
    assert_eq!(
        run_ok("function g(y){ let s=String(y); return s } const r=g(1n); console.log('x'+r);"),
        "x1"
    );
}

/// Root A — RETURN-OF-REASSIGN: `s` is written by a REASSIGNMENT after a plain
/// `0n` declarator, so the BINDING `s` is not seeded — but the RETURN `return s`
/// is monomorphically a String() result (the only return path), so the seed
/// lands on the return and the call renders (node: `x1`).
#[test]
fn p5_string_result_return_of_reassign_renders() {
    assert_eq!(
        run_ok(
            "function g(y){ let s=0n; s=String(y); return s } const r=g(1n); console.log('x'+r);"
        ),
        "x1"
    );
}

/// Root A — TRANSITIVE RETURN: `h` returns `g(y)` where `g` returns a String()
/// result; the seed propagates `g` → `h` through the return-from-return edge
/// (node: `x1`).
#[test]
fn p5_string_result_transitive_return_renders() {
    assert_eq!(
        run_ok(
            "function g(y){return String(y)} function h(y){return g(y)} console.log('x'+h(1n));"
        ),
        "x1"
    );
}

/// Root A — TEMPLATE OF INDIRECT return: a return-of-local String() result in a
/// TEMPLATE literal (node: `v=1`).
#[test]
fn p5_string_result_template_of_indirect_return_renders() {
    assert_eq!(
        run_ok("function g(y){let s=String(y);return s} console.log(`v=${g(1n)}`);"),
        "v=1"
    );
}

/// Root B — FN-EXPR BOUND: `const g = function(y){ return String(y) }`. The
/// return is seeded under the synthetic `__kali_fn_N` key; `is_string_valued`'s
/// Call arm resolves the bound-name `g` through the fold-alias to that key, so
/// the render is proven a string (node: `x1`).
#[test]
fn p5_string_result_fn_expr_bound_render_renders() {
    assert_eq!(
        run_ok("const g = function(y){ return String(y) }; console.log('x'+g(1n));"),
        "x1"
    );
}

/// Root B — ARROW BOUND: `const g = (y) => String(y)`. The expression body IS the
/// implicit return, seeded like a block-bodied `return String(y)` (node: `x1`).
#[test]
fn p5_string_result_arrow_bound_render_renders() {
    assert_eq!(
        run_ok("const g = (y) => String(y); console.log('x'+g(1n));"),
        "x1"
    );
}

/// Root C — ARITHMETIC (`*`): a String()-result binding in a MULTIPLY position.
/// node throws `TypeError` (BigInt/string mixing). Seeded `Repr::String` makes
/// the pre-existing "runtime string in arithmetic" reject fire (E3200) — the
/// accurate diagnostic for node's throw; still fail-closed, never raw bits.
#[test]
fn p5_string_result_arithmetic_mul_fails_closed() {
    run_fail_closed("let s=String(1n); console.log('n='+(s*2n));");
}

/// Root C — ARITHMETIC (`-`): the subtract twin (node throws `TypeError`).
#[test]
fn p5_string_result_arithmetic_sub_fails_closed() {
    run_fail_closed("let s=String(1n); console.log('n='+(s-1n));");
}

/// NO-OVER-DENY: a genuinely-numeric function (`return y + 1n`) must NOT be
/// tainted — the deny keys on String()-result provenance, never the `I64`
/// default. `'x'+f(1n)` renders `x2`.
#[test]
fn p5_string_result_no_over_deny_numeric_function() {
    assert_eq!(
        run_ok("function f(y){return y+1n} console.log('x'+f(1n));"),
        "x2"
    );
}

/// NO-OVER-DENY: a genuine bigint arithmetic operand (`a * b`, the exact root-C
/// operator shape but over untainted params) keeps its numeric lowering.
#[test]
fn p5_string_result_no_over_deny_genuine_arithmetic() {
    assert_eq!(
        run_ok("function f(a,b){ console.log('n=' + (a * b)); } f(3n,4n);"),
        "n=12"
    );
}

/// Root A SIBLING (caller→callee): a String()-result passed as an ARGUMENT
/// taints the callee's param, so a `'x'+p` render INSIDE the callee fails closed
/// rather than over-rendering the raw handle. Parent: silent `x-9223…`.
#[test]
fn p5_string_result_arg_into_param_fails_closed() {
    run_e5506("function g(p){ return 'x'+p } console.log(g(String(1n)));");
}

/// arg→param through a String()-result-RETURNING function (the taint reaches the
/// param via the return-taint edge, then denies at the render).
#[test]
fn p5_string_result_arg_into_param_via_fn_return_fails_closed() {
    run_e5506(
        "function mk(y){return String(y)} function g(p){return 'x'+p} console.log(g(mk(1n)));",
    );
}

/// NO-OVER-DENY: a NUMERIC argument to the same param shape keeps rendering — a
/// param is tainted only when a String() result actually flows to it, never by
/// the `I64` default.
#[test]
fn p5_string_result_no_over_deny_numeric_arg_into_param() {
    assert_eq!(
        run_ok("function g(p){ return 'x'+p } console.log(g(2n));"),
        "x2"
    );
}

// === T-new-E ROUND 3 — the remaining NUMERIC-CONSUMPTION sinks =============
//
// Round 2 consulted the String()-result taint at only two sinks
// (`emit_as_string`, `emit_binary`). These pins cover the sinks round 2 left
// UNGUARDED: unary operators, the update expression, compound-assign, and the
// dynamic computed-index. All are now routed through the single
// `emit_numeric_operand` materialization choke (or, for the update expression,
// consult the same predicate directly), so a String()-result value carried in
// an `I64` slot fails CLOSED at every numeric consumption. Each was silent
// (exit 0, raw handle bits) on parent 7b683abb0.

/// UNARY negate `-s`: seeded `Repr::String` makes the pre-existing "runtime
/// string under unary `-`" reject fire (E3200, fail-closed). node coerces
/// `-"1"` to `-1` (no throw), but kali has no string→number unary-minus lowering,
/// so it fail-closes soundly rather than miscompiling.
#[test]
fn p5_string_result_unary_neg_fails_closed() {
    run_fail_closed("let s=String(1n); console.log('n='+(-s));");
}

/// UNARY plus `+s`: a seeded `Repr::String` takes the EXISTING inline
/// decimal-parse coercion (`emit_string_to_i64_parse`, fasta Spec 5), so `+"1"`
/// parses to `1` — matching node (`+"1"` === 1). FLIPPED from fail-closed.
#[test]
fn p5_string_result_unary_plus_renders() {
    assert_eq!(run_ok("let s=String(1n); console.log('n='+(+s));"), "n=1");
}

/// UNARY bitwise-not `~s`: seeded string under `~` → E3200 fail-closed (kali has
/// no string→number bitnot lowering; node would coerce).
#[test]
fn p5_string_result_unary_bitnot_fails_closed() {
    run_fail_closed("let s=String(1n); console.log('n='+(~s));");
}

/// UNARY logical-not `!s`: reaches `emit_numeric_operand`, which now fails closed
/// on a seeded string (`is_string_valued`) → E5506. node truthiness (`!"1"` ===
/// false) is not soundly lowerable on a handle here, so fail-closed is correct.
#[test]
fn p5_string_result_unary_lognot_fails_closed() {
    run_e5506("let s=String(1n); console.log('n='+(!s));");
}

/// UNARY negate on a String()-result reached VIA A FUNCTION RETURN (the seed
/// flows through the return edge; `-s` then fails closed at the unary sink).
#[test]
fn p5_string_result_unary_neg_via_return_fails_closed() {
    run_fail_closed("function g(y){let s=String(y);return s} let s=g(1n); console.log('n='+(-s));");
}

/// UPDATE expression `s++`: parent read the handle and ran `i64.add` on the raw
/// bits. Isolated from the render guard by stashing the (postfix) OLD value in a
/// plain-i64 array element and rendering THAT element — a bare `I64`, NOT a
/// String()-result, so `emit_as_string`'s round-2 guard never fires; only the
/// update-expression choke closes it. Parent: silent `r=-9223…`.
#[test]
fn p5_string_result_update_increment_fails_closed() {
    run_e5506("let s=String(1n); let a=new Array(2); a[0]=s++; console.log('r='+a[0]);");
}

/// COMPOUND-ASSIGN `n += s` (i64 accumulator): parent ran `i64.add` on the raw
/// handle (`n=51`-class garbage). The RHS now routes through the numeric choke.
#[test]
fn p5_string_result_compound_add_assign_fails_closed() {
    run_e5506("let n=5n; let s=String(1n); n+=s; console.log('n='+n);");
}

/// COMPOUND-ASSIGN `n -= s`: the subtract twin of the above.
#[test]
fn p5_string_result_compound_sub_assign_fails_closed() {
    run_e5506("let n=5n; let s=String(1n); n-=s; console.log('n='+n);");
}

/// COMPUTED-INDEX READ `a[s]` on a working dynamic array (`new Array` + element
/// stores — the shape whose index read actually executes): parent used the
/// handle bits as the offset. Now the index operand fails closed. (A
/// module-scope scalar array LITERAL `[10n,20n]` is a separate pre-existing
/// unsupported-read placeholder that returns `0` for every index and never
/// materializes the index as a number — see the report.)
#[test]
fn p5_string_result_computed_index_read_fails_closed() {
    run_e5506("let a=new Array(3); a[0]=10n; a[1]=20n; let s=String(1n); console.log(a[s]);");
}

/// COMPUTED-INDEX STORE `a[s] = v` on a working dynamic array: the index in a
/// store position is the same `emit_array_element_address_node` choke, so the
/// store fails closed too. Parent silently stored at the handle-derived offset
/// (exit 0, `r=99`). (`new Array` because a literal-array store `[10n,20n][s]=v`
/// is separately rejected by the pre-existing literal-mutation gate.)
#[test]
fn p5_string_result_computed_index_store_fails_closed() {
    run_e5506(
        "let a=new Array(3); a[0]=10n; a[1]=20n; let s=String(1n); a[s]=99n; console.log('r='+a[1]);",
    );
}

// --- NO-OVER-DENY: every guarded sink keeps a GENUINE numeric operand correct.

/// A genuine numeric unary negate stays correct (`-5`).
#[test]
fn p5_string_result_no_over_deny_genuine_unary_neg() {
    assert_eq!(run_ok("let n=5n; console.log('n='+(-n));"), "n=-5");
}

/// A genuine numeric update expression stays correct (`6`).
#[test]
fn p5_string_result_no_over_deny_genuine_update() {
    assert_eq!(run_ok("let n=5n; n++; console.log('n='+n);"), "n=6");
}

/// A genuine numeric compound-assign stays correct (`10`).
#[test]
fn p5_string_result_no_over_deny_genuine_compound_assign() {
    assert_eq!(run_ok("let n=5n; n+=5n; console.log('n='+n);"), "n=10");
}

/// A genuine numeric dynamic-array index read stays correct (`20`).
#[test]
fn p5_string_result_no_over_deny_genuine_index_read() {
    assert_eq!(
        run_ok("let a=new Array(3); a[0]=10n; a[1]=20n; let i=1n; console.log(a[i]);"),
        "20"
    );
}

// === T-new-F: new coverage — composites, string sinks, the fn-expr literal fix ===
// Ternary is SEED-SAFE (arm selected on a separate test, handle stored verbatim)
// and renders; `&&`/`||`/`??` (operand-truthiness select) and sequence
// (mis-emitted value) are seed-UNSAFE and fail CLOSED. All measured SILENT
// (raw-bit render, exit 0) on parent 70a5a7660.

/// TERNARY, both arms a String() result: monomorphic → seeded → renders (node
/// `x2`). Both arms are `String()` calls (both default I64 in the union-find, so
/// no merge conflict), and the seed lands on the binding.
#[test]
fn p5_string_result_ternary_both_arms_renders() {
    assert_eq!(
        run_ok("let b=1n; let s=b===1n?String(2n):String(9n); console.log('x'+s);"),
        "x2"
    );
}

/// TERNARY with a NON-string arm (`5n`): not monomorphic → not seeded → fails
/// closed (node would render `x2`/`x5`; kali conservatively fails closed).
#[test]
fn p5_string_result_ternary_mixed_arm_fails_closed() {
    run_e5506("let b=1n; let s=b===1n?String(2n):5n; console.log('x'+s);");
}

/// TERNARY both arms String() in an ARITHMETIC sink → fail closed (the seeded
/// string reaches `*` — node throws `TypeError`). The round-3 leak, now sound.
#[test]
fn p5_string_result_ternary_arithmetic_fails_closed() {
    run_fail_closed("let b=1n; let s=b===1n?String(2n):String(9n); console.log('n='+(s*2n));");
}

/// LOGICAL `||` of two String() results — seed-UNSAFE (kali cannot evaluate a
/// string handle's truthiness: every handle, empty or not, is non-zero). Fails
/// closed via the taint backstop rather than rendering raw bits.
#[test]
fn p5_string_result_logical_or_fails_closed() {
    run_e5506("let s=String(1n)||String(2n); console.log('x'+s);");
}

/// LOGICAL `&&` twin.
#[test]
fn p5_string_result_logical_and_fails_closed() {
    run_e5506("let s=String(1n)&&String(2n); console.log('x'+s);");
}

/// SEQUENCE `(a, String(1n))` — kali's sequence codegen mis-emits the value (as
/// the FIRST operand), so a String() sequence is seed-UNSAFE and fails closed.
#[test]
fn p5_string_result_sequence_fails_closed() {
    run_fail_closed("let s=(0n,String(1n)); console.log('x'+s);");
}

/// NO-OVER-DENY: a genuine-numeric `&&`/`||` keeps its value-select lowering.
#[test]
fn p5_string_result_no_over_deny_genuine_logical() {
    assert_eq!(run_ok("let s=5n||9n; console.log('n='+(s+1n));"), "n=6");
}

/// STRING SINK `.length` of a let-bound String() result (node: `3`).
#[test]
fn p5_string_result_length_renders() {
    assert_eq!(run_ok("let s=String(123n); console.log(s.length);"), "3");
}

/// STRING SINK `===` of a let-bound String() result vs a literal: the seeded
/// string routes through `__streq` content-equality (`s==='1'` is true → the
/// `eq` branch). (`console.log(<bool>)` prints `1`/`0` pre-existing, so the
/// comparison is exercised via an `if` instead.)
#[test]
fn p5_string_result_streq_renders() {
    assert_eq!(
        run_ok("let s=String(1n); if (s==='1') { console.log('eq'); } else { console.log('ne'); }"),
        "eq"
    );
}

/// A `String()` result INLINE as the `encode` argument over genuine bigint
/// params still builds+runs (acceptance-path position; the seed must not perturb
/// the inline-coercion admission). node: `1`.
#[test]
fn p5_string_result_bound_encode_arg_renders() {
    assert_eq!(
        run_ok(
            "function f(a,b){ const e=new TextEncoder(); let s=String(a+b); \
             const enc=e.encode(s); console.log(enc.byteLength); } f(1n,2n);"
        ),
        "1"
    );
}

/// T-new-F Step-1 incidental fix: a fn-EXPR bound to a `const` returning a STRING
/// LITERAL (`__kali_fn_N`-keyed, return-String-seeded by the normal solve) was a
/// SILENT raw-bit render at the call site because `is_string_valued`'s Call arm
/// did not resolve the fold-alias. Resolving it (mirroring the taint's callee
/// resolution) fixes it (node: `xhi`). (The expression-bodied ARROW `()=>'hi'`
/// twin is a SEPARATE pre-existing bug — its return is never String-seeded by the
/// normal solve, memory F-AB-1 — and is out of scope here.)
#[test]
fn p5_fn_expr_literal_string_return_now_renders() {
    assert_eq!(
        run_ok("const g = function(){return 'hi'}; console.log('x'+g());"),
        "xhi"
    );
}

// ===========================================================================
// T-new-F fix — Math.* / host-numeric-call argument sink.
//
// A value carrying a seeded `Repr::String` (a `String()` result bound to a
// let/var/reassignment/return) OR a tainted string result reaching a `Math.*`
// numeric-argument position was materialized RAW via `emit_integer_math_arg`'s
// bare `emit_node`, silently miscompiling (measured on parent ccc9b5345:
// `Math.abs(String(1n))` → `9223354375949254655`, node THROWS TypeError).
// The fix routes the single shared `emit_integer_math_arg` choke through the
// same `is_string_valued || string_result_render_taint` guard the other numeric
// sinks use — closing every Math.* handler by construction AND the pre-existing
// substring→Math twin. All rows below were RUN on parent ccc9b5345 (silent
// value, exit 0) before the fix and fail closed (E5506/E3200, exit 1) after.
// ===========================================================================

/// `Math.abs` of a seeded String() result → fail-closed (node THROWS TypeError
/// for BigInt→string coercion). Parent ccc9b5345: `9223354375949254655`, exit 0.
#[test]
fn p5_math_abs_of_string_result_fails_closed() {
    run_fail_closed("let s = String(1n); console.log(Math.abs(s));");
}

/// `Math.floor` of a seeded String() result → fail-closed. Parent: raw bits.
#[test]
fn p5_math_floor_of_string_result_fails_closed() {
    run_fail_closed("let s = String(1n); console.log(Math.floor(s));");
}

/// `Math.round` of a seeded String() result → fail-closed. Parent: raw bits.
#[test]
fn p5_math_round_of_string_result_fails_closed() {
    run_fail_closed("let s = String(1n); console.log(Math.round(s));");
}

/// `Math.sign` of a seeded String() result → fail-closed. Parent: `-1`.
#[test]
fn p5_math_sign_of_string_result_fails_closed() {
    run_fail_closed("let s = String(1n); console.log(Math.sign(s));");
}

/// `Math.max` with a seeded String() result argument → fail-closed. Parent: `2`.
#[test]
fn p5_math_max_of_string_result_fails_closed() {
    run_fail_closed("let s = String(1n); console.log(Math.max(s, 2n));");
}

/// `Math.min` with a seeded String() result argument → fail-closed.
#[test]
fn p5_math_min_of_string_result_fails_closed() {
    run_fail_closed("let s = String(1n); console.log(Math.min(s, 2n));");
}

/// PRE-EXISTING general twin this same choke also closes: a runtime `Repr::String`
/// from `.substring` (predates P5) into `Math.abs`. Parent ccc9b5345: raw bits,
/// exit 0 — a silent miscompile that was never String()-specific.
#[test]
fn p5_math_abs_of_substring_fails_closed() {
    run_fail_closed("let s = \"hi\".substring(0, 2); console.log(Math.abs(s));");
}

// --- No-over-deny: genuine numeric Math arguments must still execute. ---

/// `Math.abs(5n)` → `5` (genuine bigint literal).
#[test]
fn p5_math_abs_positive_still_works() {
    assert_eq!(run_ok("console.log(Math.abs(5n));"), "5");
}

/// `Math.abs(-3n)` → `3`.
#[test]
fn p5_math_abs_negative_still_works() {
    assert_eq!(run_ok("console.log(Math.abs(-3n));"), "3");
}

/// `Math.floor(1.7)` → `1`.
#[test]
fn p5_math_floor_float_still_works() {
    assert_eq!(run_ok("console.log(Math.floor(1.7));"), "1");
}

/// `Math.max(1n, 2n)` → `2`.
#[test]
fn p5_math_max_bigints_still_works() {
    assert_eq!(run_ok("console.log(Math.max(1n, 2n));"), "2");
}

/// A genuine NUMERIC binding fed to `Math.abs` — the binding is not a string, so
/// the guard must not over-deny it. `let n=5n; Math.abs(n)` → `5`.
#[test]
fn p5_math_abs_of_numeric_binding_still_works() {
    assert_eq!(run_ok("let n = 5n; console.log(Math.abs(n));"), "5");
}
