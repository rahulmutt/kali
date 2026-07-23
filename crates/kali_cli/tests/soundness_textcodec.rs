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
