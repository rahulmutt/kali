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
fn string_call_of_binding_initialized_by_an_unproven_call_fails_closed() {
    // REVIEW C-6. `function g(y){ return String(y) }` has no `Repr::String`
    // return seed (F-newB-1), so `const s = g(1n)` keeps the DEFAULT `Repr::I64`
    // and the identifier arm read that default as "proven scalar" — rendering a
    // tagged string handle as digits. The console form is pre-existing; the
    // ENCODE form was introduced by this task's widening (measured on
    // b73a45c6d: `encode(String(s)).byteLength` → 20 where node says 1, while
    // the parent 06b6dcc87 failed closed). A binding with a resolvable
    // declarator initializer now requires that INITIALIZER proven.
    assert_fails_closed(
        "const e = new TextEncoder(); function g(y) { return String(y); } \
         const s = g(1n); const b = e.encode(String(s)); console.log(b.byteLength);",
    );
    assert_fails_closed(
        "function g(y) { return String(y); } const s = g(1n); console.log(String(s));",
    );
    assert_fails_closed(
        "function g(y) { return String(y); } let s = g(1n); console.log(String(s));",
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
fn numeric_binding_proof_is_evidence_not_the_default_repr() {
    // The round-3 admission must not become a back door to the very fallacy
    // that caused three Criticals in this task ("`Repr::I64` is the default,
    // therefore the value is a number"). Each of these bindings has the default
    // I64 repr and an UNPROVABLE write, so each must still fail closed.
    //
    // A handle-returning callee, in all three declaration spellings (node: 1).
    for kind in ["const", "let", "var"] {
        assert_fails_closed(&format!(
            "function g(y) {{ return String(y); }} {kind} s = g(1n); \
             console.log(String(s));"
        ));
    }
    // A binding that is numeric at its declarator but is LATER overwritten from
    // the handle-returning callee: one unprovable write denies the binding
    // (this proof is not flow-sensitive and must not pretend to be).
    assert_fails_closed(
        "function g(y) { return String(y); } let x = 0n; x = g(1n); console.log(String(x));",
    );
    // A declarator with no initializer holds `undefined` (node prints
    // `undefined`; kali has no rendering for it).
    assert_fails_closed("let z; console.log(String(z));");
    // A `for..of` element and a `catch` parameter are never proven numbers.
    assert_fails_closed("const a = [1n, 2n]; for (const v of a) { console.log(String(v)); }");
    // Non-arithmetic compound assignment can write the RHS's own value through.
    assert_fails_closed(
        "function g(y) { return String(y); } let x = 0n; x ||= g(1n); console.log(String(x));",
    );
    // Encode lane: same denials at the widened argument gate.
    assert_fails_closed(
        "const e = new TextEncoder(); function g(y) { return String(y); } let s = g(1n); \
         const b = e.encode(String(s)); console.log(b.byteLength);",
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

// --- Stage P5 T-new-E: String()-result render provenance (F-newB-1) ----------
// `repr_infer` seeds no `Repr::String` for a `String()` RESULT, so a result
// bound to a let/var/const or returned from a function carries a real string
// handle in an `I64` slot. Reaching a `+` / template-literal / console
// numeric-render site it was run through `int_to_string` and printed as raw
// handle bits — measured on parent ee8e2571e as
// `x-9223354375949254655` (exit 0, SILENT) where node prints `x1`. The
// merge-base (694607bb2) failed CLOSED because `String` was deny-set, so this
// is a stage-introduced fail-closed -> silent-divergent REGRESSION. This task
// restores the fail-closed invariant (E5506); correct-output support (seeding
// `Repr::String`) is the top-queued follow-up F-newB-1.

/// let-bound String() result reaching `+`. Parent: silent `x-9223…`.
#[test]
fn p5_string_result_let_bound_render_fails_closed() {
    let stderr = run_e5506("let s = String(1n); console.log('x' + s);");
    assert!(stderr.contains("F-newB-1"), "stderr: {stderr}");
}

/// var-bound String() result reaching `+`. Parent: silent `x-9223…`.
#[test]
fn p5_string_result_var_bound_render_fails_closed() {
    run_e5506("var s = String(1n); console.log('x' + s);");
}

/// function-return-bound String() result reaching `+` (provenance crosses the
/// function boundary via a String()-result-returning function). Parent: silent.
#[test]
fn p5_string_result_function_return_render_fails_closed() {
    run_e5506("function g(y){ return String(y) } const s = g(1n); console.log('x' + s);");
}

/// function-return-bound String() result reaching a TEMPLATE LITERAL. Parent:
/// silent `x-9223…` (the template ladder shares `emit_as_string`).
#[test]
fn p5_string_result_template_literal_render_fails_closed() {
    run_e5506("function g(y){ return String(y) } const s = g(1n); console.log(`x${s}`);");
}

/// direct `g(1n)` inline in `+` (the return-provenance call site itself, no
/// binding) also fails closed. Parent: silent.
#[test]
fn p5_string_result_direct_call_render_fails_closed() {
    run_e5506("function g(y){ return String(y) } console.log('x' + g(1n));");
}

/// LAUNDERING through a second binding (`let t = s`) still fails closed — the
/// provenance survives the copy. Parent: silent.
#[test]
fn p5_string_result_launder_through_second_binding_fails_closed() {
    run_e5506("let s = String(1n); let t = s; console.log('x' + t);");
}

/// MULTI-argument `console.log('x', s)` routes each argument through
/// `emit_as_string` (the wasm `int_to_string` ladder), so a tainted operand
/// fails closed. Parent: silent raw-handle render for `s`.
#[test]
fn p5_string_result_multi_arg_console_fails_closed() {
    run_e5506("let s = String(1n); console.log('x', s);");
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

/// bare-identifier REASSIGNMENT `s = String(1n)` records provenance too. Parent:
/// silent `x-9223…`. Now fails closed.
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

// --- T-new-E round-2: the structural-taint leak rows the round-1 name-sets ----
// missed. Each is SILENT (`x-9223…`, exit 0) on parent e8177812b and fails
// closed (E5506) now — the taint follows value-flow in `repr_infer`'s fixpoint.

/// Root A — RETURN-OF-LOCAL: `g` returns a String()-result LOCAL (not a direct
/// `return String(...)`), so round-1's direct-return-only recognizer missed it.
/// Provenance: seed `s` → return(`g`) → call-result `r` → render `'x'+r`.
#[test]
fn p5_string_result_return_of_local_fails_closed() {
    run_e5506("function g(y){ let s=String(y); return s } const r=g(1n); console.log('x'+r);");
}

/// Root A — RETURN-OF-REASSIGN: `s` is seeded by a REASSIGNMENT, not its
/// declarator; the taint still reaches the return and the call result.
#[test]
fn p5_string_result_return_of_reassign_fails_closed() {
    run_e5506(
        "function g(y){ let s=0n; s=String(y); return s } const r=g(1n); console.log('x'+r);",
    );
}

/// Root A — TRANSITIVE RETURN: `h` returns `g(y)` where `g` returns a String()
/// result; the return taint propagates `g` → `h` through the fixpoint's
/// return-from-return edge, and the direct `'x'+h(1n)` render fails closed.
#[test]
fn p5_string_result_transitive_return_fails_closed() {
    run_e5506("function g(y){return String(y)} function h(y){return g(y)} console.log('x'+h(1n));");
}

/// Root A — TEMPLATE OF INDIRECT return: a return-of-local String() result
/// reaching a TEMPLATE literal (the template ladder shares `emit_as_string`).
#[test]
fn p5_string_result_template_of_indirect_return_fails_closed() {
    run_e5506("function g(y){let s=String(y);return s} console.log(`v=${g(1n)}`);");
}

/// Root B — FN-EXPR BOUND: `const g = function(y){ return String(y) }`. The
/// function's repr_infer key is its synthetic `__kali_fn_N` name; round-1 keyed
/// the callee set on the fn NODE text and the declarator name `g` never matched.
/// Now the render resolves `g` through the fold-alias binding to `__kali_fn_N`.
#[test]
fn p5_string_result_fn_expr_bound_render_fails_closed() {
    run_e5506("const g = function(y){ return String(y) }; console.log('x'+g(1n));");
}

/// Root B — ARROW BOUND: `const g = (y) => String(y)`. An expression-bodied
/// arrow's body IS its implicit return, so the taint seeds the arrow's return
/// exactly like a block-bodied `return String(y)`.
#[test]
fn p5_string_result_arrow_bound_render_fails_closed() {
    run_e5506("const g = (y) => String(y); console.log('x'+g(1n));");
}

/// Root C — ARITHMETIC (`*`): a String()-result binding in a MULTIPLY position.
/// Parent silently ran `int_to_string`/`i64.mul` on the raw handle bits
/// (`n=35321811042306`); now the arithmetic operator lowering fails closed.
#[test]
fn p5_string_result_arithmetic_mul_fails_closed() {
    run_e5506("let s=String(1n); console.log('n='+(s*2n));");
}

/// Root C — ARITHMETIC (`-`): a String()-result binding in a SUBTRACT position
/// (node throws a TypeError for BigInt/string mixing; E5506 is the sound
/// fail-closed outcome). Parent: silent `n=-9223…`.
#[test]
fn p5_string_result_arithmetic_sub_fails_closed() {
    run_e5506("let s=String(1n); console.log('n='+(s-1n));");
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

/// UNARY negate `-s`: parent ran `0 - <handle>` and rendered `n=9223…`.
#[test]
fn p5_string_result_unary_neg_fails_closed() {
    run_e5506("let s=String(1n); console.log('n='+(-s));");
}

/// UNARY plus `+s` (numeric-coercion identity on a non-string-repr operand):
/// parent pushed the raw handle as the number and rendered `n=-9223…`.
#[test]
fn p5_string_result_unary_plus_fails_closed() {
    run_e5506("let s=String(1n); console.log('n='+(+s));");
}

/// UNARY bitwise-not `~s`: parent ran `-1 - <handle>` and rendered garbage.
#[test]
fn p5_string_result_unary_bitnot_fails_closed() {
    run_e5506("let s=String(1n); console.log('n='+(~s));");
}

/// UNARY logical-not `!s`: the handle bits reach `i64.eqz`; a tainted value has
/// no sound truthiness lowering here, so it fails closed.
#[test]
fn p5_string_result_unary_lognot_fails_closed() {
    run_e5506("let s=String(1n); console.log('n='+(!s));");
}

/// UNARY negate on a String()-result reached VIA A FUNCTION RETURN (the taint
/// flows through the return edge, then denies at the unary sink).
#[test]
fn p5_string_result_unary_neg_via_return_fails_closed() {
    run_e5506("function g(y){let s=String(y);return s} let s=g(1n); console.log('n='+(-s));");
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
